// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use clickhouse::Client;
use std::collections::HashMap;

use super::domain::ClickHouseConnectionDefinition;

pub fn connect(
    conn_definition: ClickHouseConnectionDefinition,
) -> Client {
    let client = Client::default()
        .with_url("http://localhost:8123")
        .with_user("name")
        .with_password("123")
        .with_database("test");

    client
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

// TODO:
pub async fn tables_info(
    pool: &Client,
    db_name: &str,
    t_names: &[String],
) -> HashMap<String, Vec<Vec<String>>> {
    HashMap::default()
}
