// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

// use clickhouse::Client;
use clickhouse_rs::Pool;

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

#[derive(Clone, Debug)]
pub struct ClickHouseCurrentSelection {
    // TODO: Aquí hay que meter también el índice de la tabla seleccionada para poder resetearlo
    // al seleccionar otra base de datos.
    pub db_idx: usize,
    pub db_name: String,
    pub tables: Vec<String>
}

impl Default for ClickHouseCurrentSelection {
    fn default() -> Self {
        Self {
            db_idx: usize::MAX,
            db_name: Default::default(),
            tables: Default::default(),
        }
    }
}


impl ClickHouseCurrentSelection {
    pub fn reset_to_new_db(&mut self) {
    }
}

#[derive(Default)]
pub struct ClickHouseState {
    // Se reusa o clona, no se crea por petición.
    pub pool: Option<Pool>,
    pub sql: SqlState,
    pub databases: Vec<String>, // Vector con bases de datos que existen en nuestra conexión.
    pub current_selection: ClickHouseCurrentSelection,
    pub current_connection: ClickHouseConnectionDefinition,
    pub tmp_connection: ClickHouseConnectionDefinition,
    pub info_messages: Vec<String>
}
