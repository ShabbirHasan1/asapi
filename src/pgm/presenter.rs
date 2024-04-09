// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use sqlx::postgres::PgPoolOptions;
use sqlx::{postgres::PgRow, Pool, Postgres, Row};
use std::collections::HashMap;
use tokio::sync::mpsc::Sender;

use super::state::PgConnDefinition;
use crate::sqlx_common::presenter::{self as sqlpresenter, Action, SqlPresenter};
use crate::sqlx_common::state::{QuerySort, SqlConnectionDefinition, SqlxMessage};
use crate::sqlx_common::traits::{Presenter, ToUrl as _};

pub async fn connect(
    conn_definition: SqlConnectionDefinition,
) -> Result<sqlx::Pool<Postgres>, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&PgConnDefinition(conn_definition).to_url())
        .await
}

pub async fn list_connection_tables(pool: &Pool<Postgres>) -> Vec<String> {
    sqlx::query(
        "SELECT table_name FROM information_schema.tables
         WHERE table_schema NOT IN ('information_schema', 'pg_catalog')",
    )
    .fetch_all(&*pool)
    .await
    .map_or(vec![], |rows| rows.iter().map(|row| row.get(0)).collect())
}

pub async fn tables_info(
    pool: &Pool<Postgres>,
    db_name: &str,
    t_names: &Vec<String>,
) -> HashMap<String, Vec<Vec<String>>> {
    let stmt = format!(
        "SELECT
             cols.table_name,
             cols.column_name,
             cols.data_type,
             cols.udt_name,
             cols.is_nullable,
             cols.column_default,
             cols.character_maximum_length,
             (CASE WHEN pk.constraint_type = 'PRIMARY KEY' THEN 'YES' ELSE 'NO' END) AS is_primary_key,
             fk.is_foreign_key,
             fk.foreign_table_name AS fk_table_name,
             fk.foreign_column_name AS fk_column_name
         FROM
             information_schema.columns AS cols
         LEFT JOIN (
             SELECT
                 tc.table_schema,
                 tc.table_name,
                 kcu.column_name,
                 tc.constraint_type
             FROM
                information_schema.table_constraints tc
            JOIN information_schema.key_column_usage kcu ON tc.constraint_schema = kcu.constraint_schema
                AND tc.constraint_name = kcu.constraint_name
            WHERE
                tc.constraint_type = 'PRIMARY KEY'
        ) AS pk ON cols.table_schema = pk.table_schema AND cols.table_name = pk.table_name AND cols.column_name = pk.column_name
        LEFT JOIN (
            SELECT
                tc.constraint_type,
                tc.table_schema,
                tc.table_name,
                kcu.column_name,
                'YES' AS is_foreign_key,
                ccu.table_schema AS foreign_table_schema,
                ccu.table_name AS foreign_table_name,
                ccu.column_name AS foreign_column_name
            FROM
                information_schema.table_constraints AS tc
            JOIN information_schema.key_column_usage AS kcu ON tc.constraint_schema = kcu.constraint_schema
                AND tc.constraint_name = kcu.constraint_name
            JOIN information_schema.constraint_column_usage AS ccu ON ccu.constraint_schema = tc.constraint_schema
                AND ccu.constraint_name = tc.constraint_name
            WHERE
                tc.constraint_type = 'FOREIGN KEY'
        ) AS fk ON cols.table_schema = fk.table_schema AND cols.table_name = fk.table_name AND cols.column_name = fk.column_name
        WHERE
            cols.table_catalog = '{}'
            AND cols.table_schema NOT IN ('information_schema', 'pg_catalog')
        ORDER BY
            cols.table_schema,
            cols.table_name,
            cols.ordinal_position
        ",db_name
    );

    match sqlx::query(stmt.as_ref()).fetch_all(pool).await {
        Ok(rows) => {
            let mut result: HashMap<String, Vec<Vec<String>>> =
                HashMap::with_capacity(t_names.len());
            for row in rows {
                let t_name: String = row.get("table_name");
                let name: String = row.get("column_name");
                let data_type: String = row.get("data_type");
                let udt_name: String = row.get("udt_name");
                let is_nullable: String = row.get("is_nullable");
                let column_default: Option<String> = row.get("column_default");
                let character_maximum_length: Option<i32> = row.get("character_maximum_length");
                let is_primary_key: String = row.get("is_primary_key");
                let is_foreign_key: Option<String> = row.get("is_foreign_key");
                let fk_table_name: Option<String> = row.get("fk_table_name");
                let fk_column_name: Option<String> = row.get("fk_column_name");

                let column_details = vec![
                    name,
                    data_type,
                    udt_name,
                    is_primary_key,
                    is_nullable,
                    column_default.map_or(String::default(), |v| v),
                    character_maximum_length.map_or(String::default(), |v| format!("{}", v)),
                    is_foreign_key.map_or(String::default(), |v| v),
                    fk_table_name.map_or(String::default(), |v| v),
                    fk_column_name.map_or(String::default(), |v| v),
                ];

                result
                    .entry(t_name)
                    .or_insert_with(Vec::new)
                    .push(column_details);
            }
            result
        }
        Err(_) => HashMap::new(),
    }
}

// Está bien, pero uso la otra para ser más similar a como lo hago en
// otras bases de datos, y porque permite extraer más información.
// pub async fn get_table_info(
//     pool: &Pool<Postgres>,
//     table_name: &str,
// ) -> Result<Vec<(String, String)>, sqlx::Error> {
//     let stmt = format!(
//         "
// SELECT
//     pg_attribute.attname AS column_name,
//     pg_catalog.format_type(pg_attribute.atttypid, pg_attribute.atttypmod) AS data_type
// FROM
//     pg_catalog.pg_attribute
// INNER JOIN
//     pg_catalog.pg_class ON pg_class.oid = pg_attribute.attrelid
// INNER JOIN
//     pg_catalog.pg_namespace ON pg_namespace.oid = pg_class.relnamespace
// WHERE
//     pg_attribute.attnum > 0
//     AND NOT pg_attribute.attisdropped
//     AND pg_class.relname = '{}'
// ORDER BY
//     attnum ASC",
//         table_name
//     );
//     let rows = sqlx::query(stmt.as_ref()).fetch_all(pool).await?;
//     let list: Vec<(String, String)> = rows
//         .iter()
//         .map(|r| (r.get("column_name"), r.get("data_type")))
//         .collect();

//     Ok(list)
// }

pub async fn select_all_with_column_description(
    pool: &Pool<Postgres>,
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
    pool: &Pool<Postgres>,
    table_name: &str,
    sort_order: QuerySort,
) -> Option<(Vec<Vec<String>>, Vec<(String, String)>)> {
    let data: Vec<PgRow> = select_all(pool, table_name, sort_order).await;
    sqlpresenter::extract_info_from_stmt_result::<PgRow>(data)
}

// Me da miedo usar un `run_statement` común a varias bases de datos
// por la idiosincracia de cada una.
// ========================== Usar de ejemplo ==========================
// Creo que es el primero de su clase que no recibe y modifica el estado
// =====================================================================
pub async fn run_statement(
    pool: &Pool<Postgres>,
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
    pool: &Pool<Postgres>,
    table_name: &str,
    sort_order: QuerySort,
) -> Vec<PgRow> {
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
    pool: &Pool<Postgres>,
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

impl Presenter for SqlPresenter<Postgres> {
    fn should_be_added_to_delete_stmt(&self, col_type: &str) -> bool {
        let c = col_type.to_ascii_uppercase();
        c == "UUID" || c == "INTEGER" || c == "TEXT"
    }

    fn should_be_wrapped(&self, col_type: &str) -> bool {
        let c = col_type.to_ascii_uppercase();
        c == "TEXT" || c == "UUID" || c == "DATETIME" || c == "DATE" || c == "NAME" || c == "TIME"
    }
}
