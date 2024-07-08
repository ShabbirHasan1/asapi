// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

// use clickhouse::{Client, Row};
use clickhouse_rs::{
    types::{Block, DateTimeType, Row, Simple, SqlType},
    Pool,
};
use futures_util::StreamExt;
use std::{any::Any, collections::HashMap, error::Error};
use tokio::sync::mpsc::Sender;

use crate::{
    common::traits::ToUrl,
    sqlx_common::{
        presenter::{self as sqlpresenter, Action},
        state::QuerySort,
    },
};

use super::domain::{ClickHouseConnectionDefinition, ClickHouseMessage};

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
                if let Err(_) = row {
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
    pool: &Pool,
    stmt: &str,
) -> Result<(Vec<Vec<String>>, Vec<(String, String)>), String> {
    let client = pool.get_handle().await;

    match client {
        Ok(mut c) => {
            let mut clickhouse_columns: HashMap<String, ClickHouseColumnData> = HashMap::default();
            let mut data: Vec<Vec<String>> = Vec::new();
            let mut col_info = Vec::new();

            let mut blocks_cursor = c.query(stmt).stream_blocks();

            while let Some(blocks) = blocks_cursor.next().await {
                if let Err(_) = blocks {
                    break;
                }
                // Un bloque tiene muchas filas.
                let block = blocks.unwrap();

                for (idx, col) in block.columns().iter().enumerate() {
                    let col_name = col.name();
                    let col_type = col.sql_type();
                    clickhouse_columns
                        .entry(col_name.to_string())
                        .or_insert_with(Default::default);
                    let clickhouse_data = clickhouse_columns.get_mut(col_name).unwrap();
                    clickhouse_data.col_type = col.sql_type().clone();

                    for row in block.rows() {
                        let col_value: String = match col_type {
                            SqlType::Bool => row.get(idx).unwrap_or(false).to_string(),
                            SqlType::UInt8 => row.get(idx).unwrap_or(0_u8).to_string(),
                            SqlType::UInt16 => row.get(idx).unwrap_or(0_u16).to_string(),
                            SqlType::UInt32 => row.get(idx).unwrap_or(0_u32).to_string(),
                            SqlType::UInt64 => row.get(idx).unwrap_or(0_u64).to_string(),
                            SqlType::Int8 => row.get(idx).unwrap_or(0_i8).to_string(),
                            SqlType::Int16 => row.get(idx).unwrap_or(0_i16).to_string(),
                            SqlType::Int32 => row.get(idx).unwrap_or(0_i32).to_string(),
                            SqlType::Int64 => row.get(idx).unwrap_or(0_i64).to_string(),
                            SqlType::String => {
                                row.get::<String, usize>(idx).unwrap_or("".to_string())
                            }
                            SqlType::FixedString(_) => {
                                row.get::<String, usize>(idx).unwrap_or("".to_string())
                            }
                            SqlType::Float32 => {
                                row.get::<f32, usize>(idx).unwrap_or(0.0).to_string()
                            }
                            SqlType::Float64 => {
                                row.get::<f64, usize>(idx).unwrap_or(0.0).to_string()
                            }
                            SqlType::Date => row
                                .get::<chrono::NaiveDate, usize>(idx)
                                .unwrap_or(chrono::NaiveDate::default())
                                .to_string(),
                            // SqlType::DateTime(_) => row.get::<chrono::NaiveDateTime, usize>(idx).unwrap_or(chrono::NaiveDateTime::from_timestamp(0, 0)).to_string(),
                            SqlType::DateTime(dt) => {
                                match dt {
                                    DateTimeType::DateTime32 => {
                                        let timestamp = row.get::<i32, usize>(idx).unwrap_or(0);
                                        let naive = chrono::NaiveDateTime::from_timestamp(
                                            (timestamp / 1_000_000).into(),
                                            (timestamp % 1_000_000) as u32 * 1_000,
                                        );
                                        let datetime = chrono::DateTime::<chrono::Utc>::from_utc(
                                            naive,
                                            chrono::Utc,
                                        );
                                        datetime.to_string()
                                    }
                                    DateTimeType::DateTime64(_, tz) => {
                                        let timestamp = row.get::<i64, usize>(idx).unwrap_or(0);
                                        let naive = chrono::NaiveDateTime::from_timestamp(
                                            timestamp / 1_000_000,
                                            (timestamp % 1_000_000) as u32 * 1_000,
                                        );
                                        let datetime = chrono::DateTime::<chrono::Utc>::from_utc(
                                            naive,
                                            chrono::Utc,
                                        );
                                        datetime.with_timezone(&tz).to_string()
                                    }
                                    DateTimeType::Chrono => row
                                        .get::<u32, usize>(idx).unwrap_or(0)
                                        .to_string(),
                                }
                                // row.get::<clickhouse_rs::types::DateTime, usize>(idx).unwrap_or_else(|_| clickhouse_rs::types::DateTime::from_secs(0)).to_string(),
                            }
                            SqlType::Ipv4 => row
                                .get::<std::net::Ipv4Addr, usize>(idx)
                                .unwrap_or(std::net::Ipv4Addr::new(0, 0, 0, 0))
                                .to_string(),
                            SqlType::Ipv6 => row
                                .get::<std::net::Ipv6Addr, usize>(idx)
                                .unwrap_or(std::net::Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0))
                                .to_string(),
                            SqlType::Uuid => row
                                .get::<uuid::Uuid, usize>(idx)
                                .unwrap_or(uuid::Uuid::nil())
                                .to_string(),
                            // SqlType::Nullable(_) => String::from("NULL"),
                            SqlType::Nullable(ref inner_type) => match **inner_type {
                                SqlType::Bool => row
                                    .get::<Option<bool>, usize>(idx)
                                    .unwrap_or(None)
                                    .map_or("NULL".to_string(), |v| v.to_string()),
                                SqlType::UInt8 => row
                                    .get::<Option<u8>, usize>(idx)
                                    .unwrap_or(None)
                                    .map_or("NULL".to_string(), |v| v.to_string()),
                                SqlType::UInt16 => row
                                    .get::<Option<u16>, usize>(idx)
                                    .unwrap_or(None)
                                    .map_or("NULL".to_string(), |v| v.to_string()),
                                SqlType::UInt32 => row
                                    .get::<Option<u32>, usize>(idx)
                                    .unwrap_or(None)
                                    .map_or("NULL".to_string(), |v| v.to_string()),
                                SqlType::UInt64 => row
                                    .get::<Option<u64>, usize>(idx)
                                    .unwrap_or(None)
                                    .map_or("NULL".to_string(), |v| v.to_string()),
                                SqlType::Int8 => row
                                    .get::<Option<i8>, usize>(idx)
                                    .unwrap_or(None)
                                    .map_or("NULL".to_string(), |v| v.to_string()),
                                SqlType::Int16 => row
                                    .get::<Option<i16>, usize>(idx)
                                    .unwrap_or(None)
                                    .map_or("NULL".to_string(), |v| v.to_string()),
                                SqlType::Int32 => row
                                    .get::<Option<i32>, usize>(idx)
                                    .unwrap_or(None)
                                    .map_or("NULL".to_string(), |v| v.to_string()),
                                SqlType::Int64 => row
                                    .get::<Option<i64>, usize>(idx)
                                    .unwrap_or(None)
                                    .map_or("NULL".to_string(), |v| v.to_string()),
                                SqlType::String => row
                                    .get::<Option<String>, usize>(idx)
                                    .unwrap_or(None)
                                    .map_or("NULL".to_string(), |v| v),
                                SqlType::FixedString(_) => row
                                    .get::<Option<String>, usize>(idx)
                                    .unwrap_or(None)
                                    .map_or("NULL".to_string(), |v| v),
                                SqlType::Float32 => row
                                    .get::<Option<f32>, usize>(idx)
                                    .unwrap_or(None)
                                    .map_or("NULL".to_string(), |v| v.to_string()),
                                SqlType::Float64 => row
                                    .get::<Option<f64>, usize>(idx)
                                    .unwrap_or(None)
                                    .map_or("NULL".to_string(), |v| v.to_string()),
                                SqlType::Date => row
                                    .get::<Option<chrono::NaiveDate>, usize>(idx)
                                    .unwrap_or(None)
                                    .map_or("NULL".to_string(), |v| v.to_string()),
                                // SqlType::DateTime(_) => row
                                // .get::<Option<chrono::NaiveDateTime>, usize>(idx)
                                // .unwrap_or(None)
                                // .map_or("NULL".to_string(), |v| v.to_string()),
                                SqlType::Ipv4 => row
                                    .get::<Option<std::net::Ipv4Addr>, usize>(idx)
                                    .unwrap_or(None)
                                    .map_or("NULL".to_string(), |v| v.to_string()),
                                SqlType::Ipv6 => row
                                    .get::<Option<std::net::Ipv6Addr>, usize>(idx)
                                    .unwrap_or(None)
                                    .map_or("NULL".to_string(), |v| v.to_string()),
                                SqlType::Uuid => row
                                    .get::<Option<uuid::Uuid>, usize>(idx)
                                    .unwrap_or(None)
                                    .map_or("NULL".to_string(), |v| v.to_string()),
                                _ => "Unsupported Nullable Type".to_string(),
                            },

                            SqlType::Array(inner_type) => "Array Type".to_string(), // Array handling can be complex; implement as needed
                            SqlType::Decimal(_, _) => "Decimal Type".to_string(), // Decimal handling; implement as needed
                            SqlType::Enum8(_) => {
                                row.get::<i8, usize>(idx).unwrap_or(0_i8).to_string()
                            }
                            SqlType::Enum16(_) => {
                                row.get::<i16, usize>(idx).unwrap_or(0_i16).to_string()
                            }
                            SqlType::SimpleAggregateFunction(_, _) => {
                                "Aggregate Function Type".to_string()
                            } // Aggregate function handling; implement as needed
                            SqlType::Map(_, _) => "Map Type".to_string(), // Map handling; implement as needed
                        };

                        clickhouse_data.row_data.push(col_value);
                    }
                }
            }

            for (col_name, col_data) in clickhouse_columns.iter() {
                println!("Column: {}, Data: {:?}", col_name, col_data.row_data);
            }

            Ok((data, col_info))
        }
        Err(e) => Err(format!("Stmt: {stmt}\nError: {e:?}")),
    }
}

// TODO:
pub async fn tables_info(pool: &Pool, t_names: &[String]) -> HashMap<String, Vec<Vec<String>>> {
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
    let action = sqlpresenter::extract_stmt_action(stmt);
    println!("Action: {action:?}");

    // Fetch_all funciona con update, insert y delete, además de con select. Pero en aquellos devuelve vacío.
    // Diferenciar entre usar fetch/execute según la acción simplifica/mejora/limpia el código mucho.
    match action {
        Action::Delete(t_name)
        | Action::Insert(t_name)
        | Action::Update(t_name)
        | Action::DropTable(t_name) => {
            // | Action::CreateTable(t_name) => match sqlx::query(stmt).execute(pool).await {
            // Ok(_) => {
            //     select_all_with_column_description(pool, tx, &t_name, QuerySort::None).await;
            // }
            // Err(err) => {
            //     let _ = tx.send(ClickHouseMessage::Error(err.to_string())).await;
            // }
        }
        Action::Select(_) => {
            let result = select_all(pool, stmt).await;
            println!("Result: {result:?}");
        }
        // Action::Select(_) => match client.query(stmt).fetch_all::<Row<'_>>().await {
        //     Ok(result) => {
        //         //match sqlpresenter::extract_info_from_stmt_result(result) {
        //         //         Some(rows) => {
        //         //             let _ = tx
        //         //                 .send(ClickHouseMessage::SelectResponse((
        //         //                     rows.0,
        //         //                     rows.1,
        //         //                     make_all_visible,
        //         //                 )))
        //         //                 .await;
        //         //         }
        //         //         None => {
        //         //             let _ = tx.send(ClickHouseMessage::Empty).await;
        //         //         }
        //         println!("Resultado: {result:?}");
        //     }
        //     Err(err) => {
        //         println!("Error: {err:?}");
        //         let _ = tx.send(ClickHouseMessage::Error(err.to_string())).await;
        //     }
        // },
        Action::None => {
            let _ = tx
                .send(ClickHouseMessage::Error("Acción no permitida".to_string()))
                .await;
        }
        _ => (),
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
