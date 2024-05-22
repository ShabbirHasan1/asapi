// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use egui_file_dialog::FileDialog;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::sqlx_common::state::SqlState;

#[derive(Serialize, Clone, Debug, Deserialize)]
pub struct SQLiteAppState {
    pub show_sidebar: bool,
    pub performance_table: bool,
    pub connections: Vec<SQLiteConnectionDefinition>,
}

impl Default for SQLiteAppState {
    fn default() -> Self {
        Self {
            show_sidebar: true,
            connections: Default::default(),
            performance_table: Default::default(),
        }
    }
}

#[derive(Default)]
pub struct SQLiteState {
    pub pool: Option<SqlitePool>,
    pub file_dialog: FileDialog,
    pub connect_to_file: bool,
    pub current_connection: SQLiteConnectionDefinition,
    pub sql: SqlState,
    pub tmp_connection_name: String,
}

/// No tengo muy claro cómo hacerlo mejor.
/// Path y OsStr son más apropiadas pero problemáticas.
/// Voy con String y ya se verá si necesito cambiar.
#[derive(Clone, Serialize, Deserialize, Default, Debug, PartialEq)]
pub struct SQLiteConnectionDefinition {
    // pub path: std::path::Path,
    // pub path: OsStr
    pub name: String,
    pub path: String,
}
