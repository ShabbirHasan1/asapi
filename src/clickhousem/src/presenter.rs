// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use clickhouse_rs::types::{Complex, Decimal, Enum16, Enum8, FromSql, SqlType};
use clickhouse_rs::{Block, ClientHandle, Pool};
use futures_util::StreamExt;
use regex::Regex;
use std::collections::HashMap;
use tokio::sync::mpsc::Sender;

use common::traits::ToUrl;
// use sqlm::sqlx_common::presenter::{self as sqlpresenter, Action};

use crate::domain::Action;

use super::domain::{ClickHouseConnectionDefinition, ClickHouseMessage};
use super::map_data_type_helpers as map;

pub fn connect(c: &ClickHouseConnectionDefinition) -> Pool {
    Pool::new(c.to_url())
}

pub async fn list_connection_databases(pool: &Pool) -> Result<Vec<String>, String> {
    let databases = select_name(pool, "SELECT name FROM system.databases").await;
    databases.map(|p| p.0)
}

pub async fn list_connection_tables(pool: &Pool) -> Result<Vec<String>, String> {
    let result = select_name(pool, "SHOW TABLES FROM {db}").await;
    result.map(|p| p.0)
}

pub async fn list_database_tables(pool: &Pool, db: &str) -> Result<Vec<String>, String> {
    let result = select_name(pool, format!("SHOW TABLES FROM {db}").as_ref()).await;
    result.map(|p| p.0)
}

async fn select_name(pool: &Pool, stmt: &str) -> Result<(Vec<String>, Vec<String>), String> {
    let client = pool.get_handle().await;

    match client {
        Ok(mut c) => {
            let mut data = Vec::new();
            let mut types = Vec::new();
            let mut stream = c.query(stmt).stream();

            while let Some(row) = stream.next().await {
                if row.is_err() {
                    break;
                }
                let row = row.unwrap();

                data.push(row.get("name").unwrap_or(String::new()));
                types.push(format!("{}", row.sql_type("name").unwrap()));
            }

            Ok((data, types))
        }
        Err(e) => Err(format!("Stmt: {stmt}\nError: {e:?}")),
    }
}

pub struct ClickHouseColumnData {
    pub col_type: SqlType,
    pub row_data: Vec<String>,
}
impl Default for ClickHouseColumnData {
    fn default() -> Self {
        Self {
            col_type: SqlType::Bool,
            row_data: Default::default(),
        }
    }
}

async fn select_all(
    client: &mut ClientHandle,
    stmt: &str,
) -> Result<(Vec<Vec<String>>, Vec<(String, String)>), String> {
    let mut clickhouse_columns: HashMap<String, ClickHouseColumnData> = HashMap::default();
    let mut col_info = Vec::new();
    let mut n_rows: usize = 0;

    let result_blocks = client.query(stmt).fetch_all().await;
    if let Err(e) = result_blocks {
        return Err(format!(
            "Error selectin all.\nStatement: {stmt}\nError: {e}"
        ));
    }

    let all_rows_block = result_blocks.unwrap();

    for column in all_rows_block.columns() {
        let col_name = column.name();
        let col_type = column.sql_type();
        let col_values = extract_block_data(&all_rows_block, col_name, &col_type);

        n_rows = col_values.len();

        clickhouse_columns.insert(
            col_name.to_string(),
            ClickHouseColumnData {
                col_type,
                row_data: col_values,
            },
        );
    }

    let mut data = vec![vec![String::new(); clickhouse_columns.len()]; n_rows];
    for (col_idx, (col_name, col_data)) in clickhouse_columns.iter().enumerate() {
        col_info.push((col_name.clone(), format!("{}", col_data.col_type)));
        for (row_idx, cell) in col_data.row_data.iter().enumerate() {
            data[row_idx][col_idx].clone_from(cell);
        }
    }

    Ok((data, col_info))
}

// TODO:
pub async fn tables_info(_pool: &Pool, _t_names: &[String]) -> HashMap<String, Vec<Vec<String>>> {
    HashMap::default()
}

pub async fn run_statement(
    pool: &Pool,
    tx: &Sender<ClickHouseMessage>,
    stmt: &str,
    make_all_visible: bool,
) {
    let result_client = pool.get_handle().await;
    if let Err(e) = result_client {
        let _ = tx
            .send(ClickHouseMessage::Error(format!("No client from pool {e}")))
            .await;
        return;
    }

    let mut client = result_client.unwrap();
    let action = extract_stmt_action(stmt);

    // Fetch_all funciona con update, insert y delete, además de con select. Pero en aquellos devuelve vacío.
    // Diferenciar entre usar fetch/execute según la acción simplifica/mejora/limpia el código mucho.
    match action {
        Action::Delete(t_name)
        | Action::Insert(t_name)
        | Action::Update(t_name)
        | Action::DropTable(t_name)
        | Action::CreateTable(t_name) => match client.execute(stmt).await {
            Ok(_) => {
                match select_all(&mut client, format!("SELECT * from {t_name}").as_ref()).await {
                    Ok(result) => {
                        let _ = tx
                            .send(ClickHouseMessage::SelectResponse((
                                result.0,
                                result.1,
                                make_all_visible,
                            )))
                            .await;
                    }
                    Err(err) => {
                        println!(
                            "Error ClickHouse Select After Statement: {err:?}\nStament: {stmt}"
                        );
                        let _ = tx.send(ClickHouseMessage::Error(err.to_string())).await;
                    }
                }
            }
            Err(err) => {
                println!("Error ClickHouse Statement: {err:?}\nStatement: {stmt}");
                let _ = tx.send(ClickHouseMessage::Error(err.to_string())).await;
            }
        },
        Action::Select(_) => match select_all(&mut client, stmt).await {
            Ok(result) => {
                let _ = tx
                    .send(ClickHouseMessage::SelectResponse((
                        result.0,
                        result.1,
                        make_all_visible,
                    )))
                    .await;
            }
            Err(err) => {
                println!("Error ClickHouse Select: {err:?}\nQuery: {stmt}");
                let _ = tx.send(ClickHouseMessage::Error(err.to_string())).await;
            }
        },
        Action::None => {
            let _ = tx
                .send(ClickHouseMessage::Error("Acción no permitida".to_string()))
                .await;
        }
    }
}

pub fn should_be_wrapped(col_type: &str) -> bool {
    let c = col_type.to_ascii_uppercase();
    c == "TEXT" || c == "UUID" || c == "DATETIME" || c == "DATE" || c == "NAME" || c == "TIME"
}

pub fn should_be_added_to_delete_stmt(col_type: &str) -> bool {
    let c = col_type.to_ascii_uppercase();
    c == "UUID" || c == "INTEGER" || c == "TEXT"
}

#[inline(always)]
fn vector_to_string<T: ToString>(v: Vec<T>) -> String {
    v.iter()
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

pub fn extract_stmt_action(sql: &str) -> Action {
    use Action::*;

    let re = Regex::new(r"(?i)(INSERT INTO|UPDATE|DELETE FROM|CREATE TABLE|DROP TABLE)\s+(\w+)")
        .unwrap();
    let re_select = Regex::new(r"FROM\s+([^\s,]+)").unwrap();

    match re.captures(sql) {
        Some(caps) => {
            let action_str = caps.get(1).map_or("", |m| m.as_str()).to_uppercase();
            let table_str = caps.get(2).map_or("", |m| m.as_str());
            let table = String::from(table_str);

            if action_str == "INSERT INTO" {
                Insert(table)
            } else if action_str == "UPDATE" {
                Update(table)
            } else if action_str == "DELETE FROM" {
                Delete(table)
            } else if action_str == "CREATE TABLE" {
                DropTable(table)
            } else if action_str == "DROP TABLE" {
                CreateTable(table)
            } else {
                None
            }
        }
        _ => match re_select.captures(sql) {
            Some(caps) => {
                let table_str = caps.get(1).map_or("", |m| m.as_str());
                let table = String::from(table_str);

                Select(table)
            }
            _ => None,
        },
    }
}

fn collect_values<'b, T: ToString + FromSql<'b>>(
    block: &'b Block<Complex>,
    column: &str,
) -> Vec<String> {
    (0..block.row_count())
        .map(|i| {
            block
                .get::<T, &str>(i, column)
                .map(|e| e.to_string())
                .unwrap_or("ERROR".to_string())
        })
        .collect()
}

fn collect_nullable_values<'b, T: ToString + FromSql<'b>>(
    block: &'b Block<Complex>,
    column: &str,
) -> Vec<String> {
    (0..block.row_count())
        .map(|i| {
            block
                .get::<Option<T>, &str>(i, column)
                .map(|e| e.map_or("NULL".to_string(), |value| value.to_string()))
                .unwrap_or("ERROR".to_string())
        })
        .collect()
}

fn extract_block_data(
    block: &Block<Complex>,
    column: &str,
    col_type: &SqlType,
) -> Vec<String> {
    match *col_type {
        SqlType::Bool => collect_values::<bool>(block, column),
        SqlType::UInt8 => collect_values::<u8>(block, column),
        SqlType::UInt16 => collect_values::<u16>(block, column),
        SqlType::UInt32 => collect_values::<u32>(block, column),
        SqlType::UInt64 => collect_values::<u64>(block, column),
        SqlType::Int8 => collect_values::<i8>(block, column),
        SqlType::Int16 => collect_values::<i16>(block, column),
        SqlType::Int32 => collect_values::<i32>(block, column),
        SqlType::Int64 => collect_values::<i64>(block, column),
        SqlType::String => collect_values::<String>(block, column),
        SqlType::FixedString(_) => collect_values::<String>(block, column),
        SqlType::Float32 => collect_values::<f32>(block, column),
        SqlType::Float64 => collect_values::<f64>(block, column),
        SqlType::Date => collect_values::<chrono::NaiveDate>(block, column),
        SqlType::Ipv4 => collect_values::<std::net::Ipv4Addr>(block, column),
        SqlType::Ipv6 => collect_values::<std::net::Ipv6Addr>(block, column),
        SqlType::Uuid => collect_values::<uuid::Uuid>(block, column),
        SqlType::Enum8(ref v) => (0..block.row_count())
            .map(|i| {
                block
                    .get::<Enum8, &str>(i, column)
                    .unwrap_or_default()
                    .internal()
            })
            .map(|i| {
                v.iter()
                    .find(|p| p.1 == i)
                    .map(|p| p.0.clone())
                    .unwrap_or_default()
            })
            .collect::<Vec<String>>(),
        SqlType::Enum16(ref v) => (0..block.row_count())
            .map(|i| {
                block
                    .get::<Enum16, &str>(i, column)
                    .unwrap_or_default()
                    .internal()
            })
            .map(|i| {
                v.iter()
                    .find(|p| p.1 == i)
                    .map(|p| p.0.clone())
                    .unwrap_or_default()
            })
            .collect::<Vec<String>>(),
        SqlType::DateTime(_) => collect_values::<chrono::DateTime<chrono_tz::Tz>>(block, column),
        SqlType::Decimal(_, _) => collect_values::<Decimal>(block, column),
        SqlType::SimpleAggregateFunction(_, _) => vec![String::from("TODO"); block.row_count()],
        SqlType::Nullable(inner_type) => match inner_type {
            SqlType::Bool => collect_nullable_values::<bool>(block, column),
            SqlType::UInt8 => collect_nullable_values::<u8>(block, column),
            SqlType::UInt16 => collect_nullable_values::<u16>(block, column),
            SqlType::UInt32 => collect_nullable_values::<u32>(block, column),
            SqlType::UInt64 => collect_nullable_values::<u64>(block, column),
            SqlType::Int8 => collect_nullable_values::<i8>(block, column),
            SqlType::Int16 => collect_nullable_values::<i16>(block, column),
            SqlType::Int32 => collect_nullable_values::<i32>(block, column),
            SqlType::Int64 => collect_nullable_values::<i64>(block, column),
            SqlType::String => collect_nullable_values::<String>(block, column),
            SqlType::FixedString(_) => collect_nullable_values::<String>(block, column),
            SqlType::Float32 => collect_nullable_values::<f32>(block, column),
            SqlType::Float64 => collect_nullable_values::<f64>(block, column),
            SqlType::Date => collect_nullable_values::<chrono::NaiveDate>(block, column),
            // SqlType::DateTime(_) => row
            // .get::<Option<Tz>, usize>(col_idx)
            // .unwrap_or(None)
            // .map_or("NULL".to_string(), |v| v.to_string()),
            SqlType::Ipv4 => collect_nullable_values::<std::net::Ipv4Addr>(block, column),
            SqlType::Ipv6 => collect_nullable_values::<std::net::Ipv6Addr>(block, column),
            SqlType::Uuid => collect_nullable_values::<uuid::Uuid>(block, column),
            _ => vec![String::from("TODO"); block.row_count()],
        },
        SqlType::Array(inner_type) => match inner_type {
            SqlType::Bool => (0..block.row_count())
                .map(|i| {
                    block
                        .get::<Vec<bool>, &str>(i, column)
                        .map(vector_to_string)
                        .unwrap_or("ERROR".to_string())
                })
                .collect(),
            SqlType::UInt8 => (0..block.row_count())
                .map(|i| {
                    block
                        .get::<Vec<u8>, &str>(i, column)
                        .map(vector_to_string)
                        .unwrap_or("ERROR".to_string())
                })
                .collect(),
            SqlType::UInt16 => (0..block.row_count())
                .map(|i| {
                    block
                        .get::<Vec<u16>, &str>(i, column)
                        .map(vector_to_string)
                        .unwrap_or("ERROR".to_string())
                })
                .collect(),
            SqlType::UInt32 => (0..block.row_count())
                .map(|i| {
                    block
                        .get::<Vec<u32>, &str>(i, column)
                        .map(vector_to_string)
                        .unwrap_or("ERROR".to_string())
                })
                .collect(),
            SqlType::UInt64 => (0..block.row_count())
                .map(|i| {
                    block
                        .get::<Vec<u64>, &str>(i, column)
                        .map(vector_to_string)
                        .unwrap_or("ERROR".to_string())
                })
                .collect(),
            SqlType::Int8 => (0..block.row_count())
                .map(|i| {
                    block
                        .get::<Vec<i8>, &str>(i, column)
                        .map(vector_to_string)
                        .unwrap_or("ERROR".to_string())
                })
                .collect(),
            SqlType::Int16 => (0..block.row_count())
                .map(|i| {
                    block
                        .get::<Vec<i16>, &str>(i, column)
                        .map(vector_to_string)
                        .unwrap_or("ERROR".to_string())
                })
                .collect(),
            SqlType::Int32 => (0..block.row_count())
                .map(|i| {
                    block
                        .get::<Vec<i32>, &str>(i, column)
                        .map(vector_to_string)
                        .unwrap_or("ERROR".to_string())
                })
                .collect(),
            SqlType::Int64 => (0..block.row_count())
                .map(|i| {
                    block
                        .get::<Vec<i64>, &str>(i, column)
                        .map(vector_to_string)
                        .unwrap_or("ERROR".to_string())
                })
                .collect(),
            SqlType::String => (0..block.row_count())
                .map(|i| {
                    block
                        .get::<Vec<String>, &str>(i, column)
                        .map(vector_to_string)
                        .unwrap_or("ERROR".to_string())
                })
                .collect(),
            SqlType::FixedString(_) => (0..block.row_count())
                .map(|i| {
                    block
                        .get::<Vec<String>, &str>(i, column)
                        .map(vector_to_string)
                        .unwrap_or("ERROR".to_string())
                })
                .collect(),
            SqlType::Float32 => (0..block.row_count())
                .map(|i| {
                    block
                        .get::<Vec<f32>, &str>(i, column)
                        .map(vector_to_string)
                        .unwrap_or("ERROR".to_string())
                })
                .collect(),
            SqlType::Float64 => (0..block.row_count())
                .map(|i| {
                    block
                        .get::<Vec<f64>, &str>(i, column)
                        .map(vector_to_string)
                        .unwrap_or("ERROR".to_string())
                })
                .collect(),
            SqlType::Date => (0..block.row_count())
                .map(|i| {
                    block
                        .get::<Vec<chrono::NaiveDate>, &str>(i, column)
                        .map(vector_to_string)
                        .unwrap_or("ERROR".to_string())
                })
                .collect(),
            SqlType::Enum8(_) => (0..block.row_count())
                .map(|i| {
                    block
                        .get::<Vec<Enum8>, &str>(i, column)
                        .map(vector_to_string)
                        .unwrap_or("ERROR".to_string())
                })
                .collect(),
            SqlType::Enum16(_) => (0..block.row_count())
                .map(|i| {
                    block
                        .get::<Vec<Enum16>, &str>(i, column)
                        .map(vector_to_string)
                        .unwrap_or("ERROR".to_string())
                })
                .collect(),
            _ => vec![String::from("Not supported by ASAPI"); block.row_count()],
        },
        SqlType::Map(ktype, vtype) => map::map_to_vec_string(ktype, vtype, block, column),
    }
}
