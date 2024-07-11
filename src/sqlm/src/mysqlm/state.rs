// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;

use crate::sqlx_common::{
    state::{SqlConnectionDefinition, SqlState},
    traits::ToUrl,
};

#[derive(Serialize, Clone, Debug, Deserialize, PartialEq)]
pub struct MySqlAppState {
    pub show_sidebar: bool,
    pub performance_table: bool,
    pub connections: Vec<SqlConnectionDefinition>,
}

impl Default for MySqlAppState {
    fn default() -> Self {
        Self {
            show_sidebar: true,
            connections: Default::default(),
            performance_table: Default::default(),
        }
    }
}

#[derive(Default)]
pub struct MySqlState {
    pub pool: Option<MySqlPool>,
    pub current_connection: SqlConnectionDefinition,
    // Datos que almacenamos de forma temporal.
    pub tmp_connection: SqlConnectionDefinition,
    pub sql: SqlState,
}

pub struct MySqlConnDefinition(pub SqlConnectionDefinition);

impl ToUrl for MySqlConnDefinition {
    fn to_url(&self) -> String {
        format!(
            "mysql://{}:{}@{}:{}/{}",
            self.0.user, self.0.password, self.0.host, self.0.port, self.0.dbname
        )
    }
}
