// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use chrono::{DateTime, TimeZone, Utc};

use clickhouse_rs::types::{Complex, Decimal, Enum16, Enum8, FromSql, Row, SqlType};
use clickhouse_rs::Block;
use clickhouse_rs::Pool;
use futures_util::StreamExt;
use std::collections::HashMap;
use tokio::sync::mpsc::Sender;

use crate::{
    common::traits::ToUrl,
    sqlx_common::presenter::{self as sqlpresenter, Action},
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

            let result_blocks = c.query(stmt).fetch_all().await;
            if let Err(e) = result_blocks {
                return Err(format!(
                    "Error selectin all.\nStatement: {stmt}\nError: {e}"
                ));
            }

            let all_rows_block = result_blocks.unwrap();

            // for row in all_rows_block.rows() {
            //     for (col_idx, column) in all_rows_block.columns().iter().enumerate() {
            //         let col_name = column.name();
            //         let col_type = column.sql_type();
            //         clickhouse_columns
            //             .entry(col_name.to_string())
            //             .or_insert_with(Default::default);
            //         let clickhouse_data = clickhouse_columns.get_mut(col_name).unwrap();
            //         clickhouse_data.col_type = col_type.clone();

            //         let v = fun_name(&col_type, &row, col_idx);

            //         clickhouse_data.row_data.push(v);
            //     }
            // }

            // ESTO ES IDEAL TAMBIÉN Y NO PUEDO POR UN ERROR EXTRAÑO QUE NO SÉ RESOLVER
            // SOBRE T NO CUMPLIENDO FROMSQL CUANDO QUIERO EXTRAER UN VEC<T>, PERO SOLO
            // A TRAVÉS DE LA FUNCIÓN USADA POR DEBAJO POR `FUN_NAME_BLOCKS`.
            for column in all_rows_block.columns() {
                let col_name = column.name();
                let col_type = column.sql_type();
                let col_values = fun_name_blocks(&all_rows_block, &col_name, &col_type);

                clickhouse_columns.insert(
                    col_name.to_string(),
                    ClickHouseColumnData {
                        col_type,
                        row_data: col_values,
                    },
                );
            }

            // CON STREAM NO PUEDO POR POROBLEMA CON BLOCK<SIMPLE> Y NECESITO QUE SEA
            // COMPLEX EN CIERTOS CASOS.
            // let mut blocks_cursor = c.query(stmt).stream_blocks();

            // while let Some(blocks) = blocks_cursor.next().await {
            //     if let Err(_) = blocks {
            //         break;
            //     }
            //     // Un bloque tiene muchas filas.
            //     let block: Block<Complex> = blocks.unwrap();

            //     for (idx, col) in block.columns().iter().enumerate() {
            //         let col_name = col.name();
            //         let col_type = col.sql_type();
            //         clickhouse_columns
            //             .entry(col_name.to_string())
            //             .or_insert_with(Default::default);
            //         let clickhouse_data = clickhouse_columns.get_mut(col_name).unwrap();
            //         clickhouse_data.col_type = col.sql_type().clone();

            //         let v = fun_name_blocks(block, col_name, &col_type);

            //         // for row in block.rows() {
            //         //     fun_name(&col_type, row, idx, clickhouse_data);
            //         // }
            //     }
            // }

            for (col_name, col_data) in clickhouse_columns.iter() {
                println!("Column: {}, Data: {:?}", col_name, col_data.row_data);
            }

            Ok((data, col_info))
        }
        Err(e) => Err(format!("Stmt: {stmt}\nError: {e:?}")),
    }
}

fn fun_name(col_type: &SqlType, row: &Row<'_, Complex>, col_idx: usize) -> String {
    let cell_value: String = match *col_type {
        SqlType::Bool => row.get::<bool, usize>(col_idx).unwrap_or(false).to_string(),
        SqlType::UInt8 => row.get::<u8, usize>(col_idx).unwrap_or(0_u8).to_string(),
        SqlType::UInt16 => row.get::<u16, usize>(col_idx).unwrap_or(0_u16).to_string(),
        SqlType::UInt32 => row.get::<u32, usize>(col_idx).unwrap_or(0_u32).to_string(),
        SqlType::UInt64 => row.get::<u64, usize>(col_idx).unwrap_or(0_u64).to_string(),
        SqlType::Int8 => row.get::<i8, usize>(col_idx).unwrap_or(0_i8).to_string(),
        SqlType::Int16 => row.get::<i16, usize>(col_idx).unwrap_or(0_i16).to_string(),
        SqlType::Int32 => row.get::<i32, usize>(col_idx).unwrap_or(0_i32).to_string(),
        SqlType::Int64 => row.get::<i64, usize>(col_idx).unwrap_or(0_i64).to_string(),
        SqlType::String => row.get::<String, usize>(col_idx).unwrap_or("".to_string()),
        SqlType::FixedString(_) => row.get::<String, usize>(col_idx).unwrap_or("".to_string()),
        SqlType::Float32 => row.get::<f32, usize>(col_idx).unwrap_or(0.0).to_string(),
        SqlType::Float64 => row.get::<f64, usize>(col_idx).unwrap_or(0.0).to_string(),
        SqlType::Date => row
            .get::<chrono::NaiveDate, usize>(col_idx)
            .unwrap_or(chrono::NaiveDate::default())
            .to_string(),
        SqlType::DateTime(_) => {
            let timestamp: i64 = row.get(col_idx).unwrap_or(0);

            // Convertir el timestamp a DateTime<Utc>
            let datetime: DateTime<Utc> = Utc
                .timestamp_opt(timestamp, 0)
                .single()
                .unwrap_or(Utc::now());

            // Formatear como string
            datetime.format("%Y-%m-%d %H:%M:%S").to_string()
        }
        // SqlType::DateTime(dt) => match dt {
        //     DateTimeType::DateTime32 => {
        //         println!("dt: {dt:?}");
        //         let v = row.get::<u32, usize>(col_idx).unwrap_or_default();
        //         println!("Extraído DateTime32: {v}");
        //         let time = chrono::DateTime::from_timestamp_nanos(v as i64);
        //         time.to_rfc2822()
        //     }
        //     DateTimeType::DateTime64(precision, tz) => {
        //         let v = row.get::<i64, usize>(col_idx).unwrap_or_default();
        //         println!("Extraído DateTime64: {v}");
        //         let base10: i64 = 10;

        //         let nano = if precision < 19 {
        //             v * base10.pow(9 - precision)
        //         } else {
        //             0_i64
        //         };

        //         let sec = nano / 1_000_000_000;
        //         let nsec = nano - sec * 1_000_000_000;

        //         match tz.timestamp_opt(sec, nsec as u32) {
        //             LocalResult::Single(datetime) => {
        //                 format!("Datetime as String: {}", datetime.to_rfc3339())
        //             }
        //             _ => format!("Invalid datetime conversion."),
        //         }
        //     }
        //     _ => "Unsupported DateTime format".to_string(),
        // },
        SqlType::Ipv4 => row
            .get::<std::net::Ipv4Addr, usize>(col_idx)
            .unwrap_or(std::net::Ipv4Addr::new(0, 0, 0, 0))
            .to_string(),
        SqlType::Ipv6 => row
            .get::<std::net::Ipv6Addr, usize>(col_idx)
            .unwrap_or(std::net::Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0))
            .to_string(),
        SqlType::Uuid => row
            .get::<uuid::Uuid, usize>(col_idx)
            .unwrap_or(uuid::Uuid::nil())
            .to_string(),
        // SqlType::Nullable(_) => String::from("NULL"),
        SqlType::Nullable(ref inner_type) => match **inner_type {
            SqlType::Bool => row
                .get::<Option<bool>, usize>(col_idx)
                .unwrap_or(None)
                .map_or("NULL".to_string(), |v| v.to_string()),
            SqlType::UInt8 => row
                .get::<Option<u8>, usize>(col_idx)
                .unwrap_or(None)
                .map_or("NULL".to_string(), |v| v.to_string()),
            SqlType::UInt16 => row
                .get::<Option<u16>, usize>(col_idx)
                .unwrap_or(None)
                .map_or("NULL".to_string(), |v| v.to_string()),
            SqlType::UInt32 => row
                .get::<Option<u32>, usize>(col_idx)
                .unwrap_or(None)
                .map_or("NULL".to_string(), |v| v.to_string()),
            SqlType::UInt64 => row
                .get::<Option<u64>, usize>(col_idx)
                .unwrap_or(None)
                .map_or("NULL".to_string(), |v| v.to_string()),
            SqlType::Int8 => row
                .get::<Option<i8>, usize>(col_idx)
                .unwrap_or(None)
                .map_or("NULL".to_string(), |v| v.to_string()),
            SqlType::Int16 => row
                .get::<Option<i16>, usize>(col_idx)
                .unwrap_or(None)
                .map_or("NULL".to_string(), |v| v.to_string()),
            SqlType::Int32 => row
                .get::<Option<i32>, usize>(col_idx)
                .unwrap_or(None)
                .map_or("NULL".to_string(), |v| v.to_string()),
            SqlType::Int64 => row
                .get::<Option<i64>, usize>(col_idx)
                .unwrap_or(None)
                .map_or("NULL".to_string(), |v| v.to_string()),
            SqlType::String => row
                .get::<Option<String>, usize>(col_idx)
                .unwrap_or(None)
                .map_or("NULL".to_string(), |v| v),
            SqlType::FixedString(_) => row
                .get::<Option<String>, usize>(col_idx)
                .unwrap_or(None)
                .map_or("NULL".to_string(), |v| v),
            SqlType::Float32 => row
                .get::<Option<f32>, usize>(col_idx)
                .unwrap_or(None)
                .map_or("NULL".to_string(), |v| v.to_string()),
            SqlType::Float64 => row
                .get::<Option<f64>, usize>(col_idx)
                .unwrap_or(None)
                .map_or("NULL".to_string(), |v| v.to_string()),
            SqlType::Date => row
                .get::<Option<chrono::NaiveDate>, usize>(col_idx)
                .unwrap_or(None)
                .map_or("NULL".to_string(), |v| v.to_string()),
            // SqlType::DateTime(_) => row
            // .get::<Option<Tz>, usize>(col_idx)
            // .unwrap_or(None)
            // .map_or("NULL".to_string(), |v| v.to_string()),
            SqlType::Ipv4 => row
                .get::<Option<std::net::Ipv4Addr>, usize>(col_idx)
                .unwrap_or(None)
                .map_or("NULL".to_string(), |v| v.to_string()),
            SqlType::Ipv6 => row
                .get::<Option<std::net::Ipv6Addr>, usize>(col_idx)
                .unwrap_or(None)
                .map_or("NULL".to_string(), |v| v.to_string()),
            SqlType::Uuid => row
                .get::<Option<uuid::Uuid>, usize>(col_idx)
                .unwrap_or(None)
                .map_or("NULL".to_string(), |v| v.to_string()),
            _ => "Unsupported Nullable Type".to_string(),
        },
        SqlType::Array(inner_type) => match inner_type {
            SqlType::Bool => row
                .get::<Vec<bool>, usize>(col_idx)
                .unwrap_or_default()
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join(","),
            SqlType::UInt8 => row
                .get::<Vec<u8>, usize>(col_idx)
                .unwrap_or_default()
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join(","),
            SqlType::UInt16 => row
                .get::<Vec<u16>, usize>(col_idx)
                .unwrap_or_default()
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join(","),
            SqlType::UInt32 => row
                .get::<Vec<u32>, usize>(col_idx)
                .unwrap_or_default()
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join(","),
            SqlType::UInt64 => row
                .get::<Vec<u64>, usize>(col_idx)
                .unwrap_or_default()
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join(","),
            SqlType::Int8 => row
                .get::<Vec<i8>, usize>(col_idx)
                .unwrap_or_default()
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join(","),
            SqlType::Int16 => row
                .get::<Vec<i16>, usize>(col_idx)
                .unwrap_or_default()
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join(","),
            SqlType::Int32 => row
                .get::<Vec<i32>, usize>(col_idx)
                .unwrap_or_default()
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join(","),
            SqlType::Int64 => row
                .get::<Vec<i64>, usize>(col_idx)
                .unwrap_or_default()
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join(","),
            SqlType::String => row
                .get::<Vec<String>, usize>(col_idx)
                .unwrap_or_default()
                .join(","),
            SqlType::FixedString(_) => row
                .get::<Vec<String>, usize>(col_idx)
                .unwrap_or_default()
                .join(","),
            SqlType::Float32 => row
                .get::<Option<f32>, usize>(col_idx)
                .unwrap_or_default()
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join(","),
            SqlType::Float64 => row
                .get::<Option<f64>, usize>(col_idx)
                .unwrap_or_default()
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join(","),
            SqlType::Date => row
                .get::<Option<chrono::NaiveDate>, usize>(col_idx)
                .unwrap_or_default()
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join(","),
            // SqlType::DateTime(_) => row
            // .get::<Option<chrono::NaiveDateTime>, usize>(idx)
            // .unwrap_or_default().iter().map(|e| e.to_string()).collect::<Vec<_>>().join(","),
            SqlType::Ipv4 => row
                .get::<Option<std::net::Ipv4Addr>, usize>(col_idx)
                .unwrap_or_default()
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join(","),
            SqlType::Ipv6 => row
                .get::<Option<std::net::Ipv6Addr>, usize>(col_idx)
                .unwrap_or_default()
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join(","),
            SqlType::Uuid => row
                .get::<Option<uuid::Uuid>, usize>(col_idx)
                .unwrap_or_default()
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join(","),
            _ => "Unsupported Nullable Type".to_string(),
        },
        SqlType::Decimal(_, _) => "Decimal Type".to_string(),
        SqlType::Enum8(_) => row_value_to_string::<u8>(&row, col_idx),
        SqlType::Enum16(_) => row.get::<i16, usize>(col_idx).unwrap_or(0_i16).to_string(),
        SqlType::SimpleAggregateFunction(_, _) => "Aggregate Function Type".to_string(), // Aggregate function handling; implement as needed
        SqlType::Map(_, _) => "Map Type".to_string(), // Map handling; implement as needed
    };

    return cell_value;
}

fn row_value_to_string<'a, T>(row: &'a Row<'a, Complex>, idx: usize) -> String
where
    T: clickhouse_rs::types::FromSql<'a> + ToString + Default,
{
    row.get::<T, usize>(idx).unwrap_or_default().to_string()
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

#[inline(always)]
fn vector_to_string<T: ToString>(v: Vec<T>) -> String {
    v.iter()
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

// lo dejo por interés aunque no gaste
// No puedo usarla para vectores!!! Ni idea por qué!!!
// https://github.com/suharev7/clickhouse-rs/blob/e47ba334bd1f28de20dd0c85b9af66fe029a6dea/tests/clickhouse.rs#L48
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

fn fun_name_blocks<'b>(block: &'b Block<Complex>, column: &str, col_type: &SqlType) -> Vec<String> {
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
        SqlType::Enum8(_) => collect_values::<Enum8>(block, column),
        SqlType::Enum16(_) => collect_values::<Enum16>(block, column),
        SqlType::DateTime(_) => vec![String::from("TODO"); block.row_count()],
        SqlType::Decimal(_, _) => collect_values::<Decimal>(block, column),
        SqlType::SimpleAggregateFunction(_, _) => vec![String::from("TODO"); block.row_count()],
        SqlType::Map(ktype, vtype) => {
            println!("Key type: {ktype:?} -- Value type: {vtype:?}");
            vec![String::from("TODO"); block.row_count()]
        }
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
    }
}
