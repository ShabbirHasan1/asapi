// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use std::collections::HashMap;
use sqlx::{sqlite::SqliteRow, Pool, Row, Sqlite};
use tokio::sync::mpsc::Sender;

use crate::sqlx_common::presenter::{self as sqlpresenter, Action, SqlPresenter};
use crate::sqlx_common::state::{QuerySort, SqlxMessage};
use crate::sqlx_common::traits::Presenter;

pub async fn connect(url: &str) -> Result<sqlx::Pool<sqlx::Sqlite>, sqlx::Error> {
    Pool::<Sqlite>::connect(url).await
}

// pub async fn list_connection_tables(pool: &Pool<Sqlite>) -> Result<Vec<String>, sqlx::Error> {
//     let rows = sqlx::query("SELECT name FROM sqlite_master WHERE type='table'")
//         .fetch_all(pool)
//         .await?;

//     let mut table_names = vec![String::default(); rows.len()];
//     for (idx, row) in rows.iter().enumerate() {
//         let name: String = row.get("name");
//         table_names[idx] = name;
//     }

//     Ok(table_names)
// }

pub async fn tables_info(
    pool: &Pool<Sqlite>,
    // t_names: &Vec<String>,
) -> HashMap<String, Vec<Vec<String>>> {
    let stmt = "WITH all_tables AS (SELECT name FROM sqlite_master WHERE type = 'table')
           SELECT at.name table_name, pti.*
           FROM all_tables at INNER JOIN pragma_table_info(at.name) pti
           ORDER BY table_name";
    match sqlx::query(stmt).fetch_all(pool).await {
        Ok(rows) => {
            let mut result: HashMap<String, Vec<Vec<String>>> = HashMap::new();
            // HashMap::with_capacity(t_names.len());
            for row in rows {
                let t_name: String = row.get("table_name");
                let name: String = row.get("name");
                let pk: i64 = row.get("pk");
                let type_: String = row.get("type");
                let not_null: bool = row.get("notnull");
                let default_val: Option<String> = row.get("dflt_value");

                // El 0 ha de ser nombre y el 2 tipo en todas.
                let column_details = vec![
                    name,
                    pk.to_string(),
                    type_,
                    not_null.to_string(),
                    default_val.map_or(String::default(), |v| v),
                ];
                result
                    .entry(t_name)
                    .or_default()
                    .push(column_details);
            }
            result
        }
        Err(_) => HashMap::new(),
    }
}

pub async fn get_all_with_column_description(
    pool: &Pool<Sqlite>,
    table_name: &str,
    sort_order: QuerySort,
) -> Option<(Vec<Vec<String>>, Vec<(String, String)>)> {
    let data = select_all(pool, table_name, sort_order).await;
    sqlpresenter::extract_info_from_stmt_result::<SqliteRow>(data)
}

pub async fn select_all_with_column_description(
    pool: &Pool<Sqlite>,
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

// Me da miedo usar un `run_statement` común a varias bases de datos
// por la idiosincracia de cada una.
// ========================== Usar de ejemplo ==========================
// Creo que es el primero de su clase que no recibe y modifica el estado
// =====================================================================
pub async fn run_statement(
    pool: &Pool<Sqlite>,
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
        | Action::CreateTable(t_name) => match sqlx::query(stmt).execute(pool).await {
            Ok(_) => {
                select_all_with_column_description(pool, tx, &t_name, QuerySort::None).await;
            }
            Err(err) => {
                let _ = tx.send(SqlxMessage::Error(err.to_string())).await;
            }
        },
        Action::Select(_) => match sqlx::query(stmt).fetch_all(pool).await {
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
        Action::None => {
            let _ = tx
                .send(SqlxMessage::Error("Acción no permitida".to_string()))
                .await;
        }
    }
}

// TODO: Esto podríamos simplificarlo para depender de `run_statement`, pero no corre ninguna prisa.
pub async fn select_all(
    pool: &Pool<Sqlite>,
    table_name: &str,
    sort_order: QuerySort,
) -> Vec<SqliteRow> {
    let order_by = match sort_order {
        QuerySort::None => String::default(),
        QuerySort::Asc => format!("ORDER BY {} ASC", table_name),
        QuerySort::Desc => format!("ORDER BY {} DESC", table_name),
    };

    match sqlx::query(format!("SELECT * FROM {} {}", table_name, order_by).as_ref())
        .fetch_all(pool)
        .await
    {
        Ok(rows) => rows,
        Err(_) => Vec::default(),
    }
}

// pub async fn get_table_info(
//     pool: &Pool<Sqlite>,
//     table_name: &str,
// ) -> Result<Vec<(String, String, bool, Option<String>, i64)>, sqlx::Error> {
//     let mut list = Vec::new();
//     let rows = sqlx::query(format!("PRAGMA table_info ({});", table_name).as_ref())
//         .bind(table_name)
//         .fetch_all(pool)
//         .await?;

//     for row in rows {
//         let name: String = row.get("name");
//         let type_: String = row.get("type");
//         let not_null: bool = row.get("notnull");
//         let default_val: Option<String> = row.get("dflt_value");
//         let pk: i64 = row.get("pk");

//         list.push((name, type_, not_null, default_val, pk));
//     }
//     Ok(list)
// }

pub async fn run_statement_with_delete_control(
    pool: &Pool<Sqlite>,
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

impl Presenter for SqlPresenter<Sqlite> {
    fn should_be_added_to_delete_stmt(&self, col_type: &str) -> bool {
        col_type == "INTEGER" || col_type == "TEXT" || col_type.starts_with("VARCHAR")
    }

    fn should_be_wrapped(&self, col_type: &str) -> bool {
        col_type == "TEXT"
            || col_type == "DATETIME"
            || col_type == "DATE"
            || col_type == "TIME"
            || col_type.starts_with("VARCHAR")
    }
}
