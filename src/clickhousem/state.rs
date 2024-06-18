// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use clickhouse::Client;

use crate::sqlx_common::state::SqlState;

use super::domain::ClickHouseConnectionDefinition;


#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ClickHouseAppState {
    pub show_sidebar: bool,
    pub performance_table: bool,
    pub connections: Vec<ClickHouseConnectionDefinition>,
}

impl Default for ClickHouseAppState {
    fn default() -> Self {
        Self {
            show_sidebar: true,
            connections: Default::default(),
            performance_table: Default::default(),
        }
    }
}

#[derive(Default)]
pub struct ClickHouseState {
    // Se reusa o clona, no se crea por petición.
    pub pool: Option<Client>,
    pub sql: SqlState,
    pub current_connection: ClickHouseConnectionDefinition,
    pub tmp_connection: ClickHouseConnectionDefinition,
}
