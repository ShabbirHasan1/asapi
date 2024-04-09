// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use std::collections::HashMap;

use super::state::MySqlConnDefinition;
use crate::sqlx_module::presenter::{self as sqlpresenter, Action, SqlPresenter};
use crate::sqlx_module::state::{QuerySort, SqlConnectionDefinition, SqlxMessage};
use crate::sqlx_module::traits::{Presenter, ToUrl as _};
use sqlx::mysql::{MySqlPoolOptions, MySqlRow};
use sqlx::{MySql, Pool, Row};
use tokio::sync::mpsc::Sender;

pub async fn connect(
    conn_definition: SqlConnectionDefinition,
) -> Result<sqlx::Pool<MySql>, sqlx::Error> {
    MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&MySqlConnDefinition(conn_definition).to_url())
        .await
}

pub async fn list_connection_tables(pool: &Pool<MySql>, db_name: &str) -> Vec<String> {
    sqlx::query(
        format!(
            "SELECT table_name FROM information_schema.tables
             WHERE table_schema = '{}' ORDER by table_name",
            db_name
        )
        .as_str(),
    )
    .fetch_all(&*pool)
    .await
    .map_or(vec![], |rows| rows.iter().map(|row| row.get(0)).collect())
}

pub async fn tables_info(
    pool: &Pool<MySql>,
    db_name: &str,
    t_names: &Vec<String>,
) -> Result<HashMap<String, Vec<Vec<String>>>, sqlx::Error> {
    let names_joined = t_names
        .iter()
        .map(|e| format!("'{}'", e))
        .collect::<Vec<String>>()
        .join(", ");
    let stmt = format!("
         SELECT TABLE_NAME, COLUMN_NAME, DATA_TYPE, COLUMN_TYPE, IS_NULLABLE, COLUMN_DEFAULT, COLUMN_KEY
         FROM INFORMATION_SCHEMA.COLUMNS
         WHERE TABLE_SCHEMA = '{}' AND TABLE_NAME IN ({})", db_name, names_joined);
    let mut result: HashMap<String, Vec<Vec<String>>> = HashMap::with_capacity(t_names.len());
    let rows = sqlx::query(stmt.as_ref()).fetch_all(pool).await?;

    for row in rows {
        let t_name: String = row.get("TABLE_NAME");
        let name: String = row.get("COLUMN_NAME");
        let data_type: String = row.get("DATA_TYPE");
        let column_type: String = row.get("COLUMN_TYPE");
        let is_nullable: String = row.get("IS_NULLABLE");
        let column_default: Option<String> = row.get("COLUMN_DEFAULT");
        let column_key: String = row.get("COLUMN_KEY");

        let column_details = vec![
            name,
            data_type,
            column_type,
            is_nullable,
            column_default.map_or(String::default(), |v| v),
            column_key,
        ];

        result
            .entry(t_name)
            .or_insert_with(Vec::new)
            .push(column_details);
    }

    Ok(result)
}

pub async fn select_all_with_column_description(
    pool: &Pool<MySql>,
    tx: &Sender<SqlxMessage>,
    table_name: &str,
    sort_order: QuerySort,
) {
    // let data = select_all(pool, table_name, sort_order).await;
    match get_all_with_column_description(pool, table_name, sort_order).await {
        Some(rows) => {
            let _ = tx
                .send(SqlxMessage::SelectResponse((rows.0, rows.1, true)))
                .await;
        }
        None => {
            let _ = tx.send(SqlxMessage::Empty).await;
        }
    }
}

pub async fn get_all_with_column_description(
    pool: &Pool<MySql>,
    table_name: &str,
    sort_order: QuerySort,
) -> Option<(Vec<Vec<String>>, Vec<(String, String)>)> {
    let data: Vec<MySqlRow> = select_all(pool, table_name, sort_order).await;

    sqlpresenter::extract_info_from_stmt_result::<MySqlRow>(data)
}

// Me da miedo usar un `run_statement` común a varias bases de datos
// por la idiosincracia de cada una.
// ========================== Usar de ejemplo ==========================
// Creo que es el primero de su clase que no recibe y modifica el estado
// =====================================================================
pub async fn run_statement(
    pool: &Pool<MySql>,
    tx: &Sender<SqlxMessage>,
    stmt: &str,
    make_all_visible: bool,
) {
    let action = sqlpresenter::extract_stmt_action(stmt);

    // Fetch_all funciona con update, insert y delete, además de con select. Pero en aquellos devuelve vacío.
    // Diferenciar entre usar fetch/execute según la acción simplifica/mejora/limpia el código mucho.
    match action {
        Action::Delete(t_name)
        | Action::Insert(t_name)
        | Action::Update(t_name)
        | Action::DropTable(t_name)
        | Action::CreateTable(t_name) => match sqlx::query(stmt).execute(&*pool).await {
            Ok(_) => {
                select_all_with_column_description(pool, tx, &t_name, QuerySort::NONE).await;
            }
            Err(err) => {
                let _ = tx.send(SqlxMessage::Error(err.to_string())).await;
            }
        },
        Action::Select(_) => match sqlx::query(stmt).fetch_all(&*pool).await {
            Ok(result) => match sqlpresenter::extract_info_from_stmt_result(result) {
                Some(rows) => {
                    let _ = tx
                        .send(SqlxMessage::SelectResponse((
                            rows.0,
                            rows.1,
                            make_all_visible,
                        )))
                        .await;
                }
                None => {
                    let _ = tx.send(SqlxMessage::Empty).await;
                }
            },
            Err(err) => {
                let _ = tx.send(SqlxMessage::Error(err.to_string())).await;
            }
        },
        Action::NONE => {
            let _ = tx
                .send(SqlxMessage::Error("Acción no permitida".to_string()))
                .await;
        }
    }
}

// TODO:
// Esto podríamos simplificarlo para depender de `run_statement`,
// pero no corre ninguna prisa.
pub async fn select_all(
    pool: &Pool<MySql>,
    table_name: &str,
    sort_order: QuerySort,
) -> Vec<MySqlRow> {
    let order_by = match sort_order {
        QuerySort::NONE => String::default(),
        QuerySort::ASC => format!("ORDER BY {} ASC", table_name),
        QuerySort::DESC => format!("ORDER BY {} DESC", table_name),
    };

    match sqlx::query(format!("SELECT * FROM {} {}", table_name, order_by).as_ref())
        .fetch_all(&*pool)
        .await
    {
        Ok(rows) => rows,
        Err(_) => Vec::default(),
    }
}

pub async fn run_statement_with_delete_control(
    pool: &Pool<MySql>,
    tx: &Sender<SqlxMessage>,
    stmt: &str,
    make_all_visible: bool,
    delete_allowed: bool,
) -> bool {
    match sqlpresenter::extract_stmt_action(stmt) {
        Action::Delete(_) => {
            if delete_allowed {
                run_statement(pool, tx, stmt, make_all_visible).await;
                true
            } else {
                false
            }
        }
        _ => {
            run_statement(pool, tx, stmt, make_all_visible).await;
            true
        }
    }
}

impl Presenter for SqlPresenter<MySql> {
    fn should_be_added_to_delete_stmt(&self, col_type: &str) -> bool {
        col_type == "UUID"
            || col_type == "INTEGER"
            || col_type == "TEXT"
            || col_type.starts_with("INT")
            || col_type.starts_with("VARCHAR")
    }

    fn should_be_wrapped(&self, col_type: &str) -> bool {
        col_type == "TEXT"
            || col_type == "UUID"
            || col_type == "DATETIME"
            || col_type == "TIMESTAMP"
            || col_type == "DATE"
            || col_type == "TIME"
            || col_type.starts_with("VARCHAR")
    }
}
