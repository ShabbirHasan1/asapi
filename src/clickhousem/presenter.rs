// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use clickhouse::Client;
use std::collections::HashMap;
use tokio::sync::mpsc::Sender;

use crate::{
    common::traits::ToUrl,
    sqlx_common::presenter::{self as sqlpresenter, Action},
};

use super::domain::{ClickHouseConnectionDefinition, ClickHouseMessage};

pub fn connect(c: &ClickHouseConnectionDefinition) -> Client {
    let client = Client::default()
        .with_url(c.to_url())
        .with_user(&c.user)
        .with_password(&c.password);
    // .with_database("test");

    client
}

pub async fn list_connection_databases(pool: &Client) -> Vec<String> {
    let rows = pool
        .query("SELECT name FROM system.databases")
        .fetch_all()
        .await;

    rows.unwrap_or(vec![])
}

pub async fn list_connection_tables(pool: &Client) -> Vec<String> {
    let rows = pool
        .query(
            "SELECT table_name FROM information_schema.tables
         WHERE table_schema NOT IN ('information_schema', 'pg_catalog')",
        )
        .fetch_all()
        .await;

    rows.unwrap_or(vec![])
}

pub async fn list_database_tables(c: &Client, db: &str) -> Vec<String> {
    (c.query(format!("SHOW TABLES FROM {db}").as_str())
        .fetch_all()
        .await)
        .unwrap_or(vec![])
}

// TODO:
pub async fn tables_info(client: &Client, t_names: &[String]) -> HashMap<String, Vec<Vec<String>>> {
    HashMap::default()
}

pub async fn run_statement_with_delete_control(
    client: &Client,
    tx: &Sender<ClickHouseMessage>,
    stmt: &str,
    make_all_visible: bool,
    delete_allowed: bool,
) -> bool {
    match sqlpresenter::extract_stmt_action(stmt) {
        Action::Delete(_) => {
            if delete_allowed {
                run_statement(client, tx, stmt, make_all_visible).await;
                true
            } else {
                false
            }
        }
        _ => {
            run_statement(client, tx, stmt, make_all_visible).await;
            true
        }
    }
}

pub async fn run_statement(
    client: &Client,
    tx: &Sender<ClickHouseMessage>,
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
        | Action::DropTable(t_name) => {
            // | Action::CreateTable(t_name) => match sqlx::query(stmt).execute(pool).await {
            // Ok(_) => {
            //     select_all_with_column_description(pool, tx, &t_name, QuerySort::None).await;
            // }
            // Err(err) => {
            //     let _ = tx.send(ClickHouseMessage::Error(err.to_string())).await;
            // }
        }
        // Action::Select(_) => match sqlx::query(stmt).fetch_all(pool).await {
        // Ok(result) => match sqlpresenter::extract_info_from_stmt_result(result) {
        // Some(rows) => {
        //     let _ = tx
        //         .send(ClickHouseMessage::SelectResponse((
        //             rows.0,
        //             rows.1,
        //             make_all_visible,
        //         )))
        //         .await;
        // }
        // None => {
        //     let _ = tx.send(ClickHouseMessage::Empty).await;
        // }
        // },
        // Err(err) => {
        //     let _ = tx.send(ClickHouseMessage::Error(err.to_string())).await;
        // }
        // },
        Action::None => {
            let _ = tx
                .send(ClickHouseMessage::Error("Acción no permitida".to_string()))
                .await;
        }
        _ => (),
    }
}
