// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use std::collections::HashMap;

// use clickhouse::Client;
use clickhouse_rs::Pool;

use crate::domain::QuerySort;

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
    pub tables: Vec<String>,
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
    pub fn reset_to_new_db(&mut self) {}
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
}

#[derive(Default)]
pub struct SqlDataGenState {
    // Para no tener que castear de isize (-1) a usize (con el que puedo indexar vector).
    pub n_rows_to_generate: u16,
    // Mostramos ventana para insertar valores generados aleatoriamente.
    pub show_generator_window: bool,
    // Mostramos ventana para insertar valores a mano.
    pub show_insertion_window: bool,
    // Tabla selccionada para generar datos.
    pub table_to_generate_data: Option<String>,
    // (Nombre, tipo) de cada columna
    pub selected_table_definition: Vec<(String, String)>,
    // En la ventana de generación de datos, vector que controla si usamos valor aleatorio o el usuaio introduce el valor que quiere.
    pub fixed_value_flags: Vec<bool>,
    // Vector para almacenar valor en caso de que sea introducido por el usuario.
    pub fixed_string_value: Vec<String>,
    // Vector para controlar si el campo será nulo.
    pub nullable_column_flags: Vec<bool>,
}

impl SqlDataGenState {
    pub fn reset(&mut self) {
        self.show_generator_window = false;
        self.show_insertion_window = false;
        self.table_to_generate_data = None;
        // self.fixed_value_flags = vec![];
        // self.fixed_string_value = vec![];
        // self.nullable_column_flags = vec![];
    }
}

#[derive(Default)]
pub struct SqlEditRowState {
    pub row_data: Vec<String>,
    pub selected_row: Option<usize>,
}

// Traído de Sqlx_common
pub struct SqlState {
    pub column_visible: Vec<bool>,
    pub tables: Vec<String>,
    pub current_connection_tables_info: HashMap<String, Vec<Vec<String>>>,
    pub current_connection_idx: usize,
    pub current_table_idx: usize,
    pub current_table_columns: Vec<(String, String)>, // (Nombre, Tipo)
    pub current_table_rows: Vec<Vec<String>>,
    pub hide_connections: bool,
    pub hide_tables: bool,
    pub sql_statement: String,
    // almacenamos última respuesta a run_statemente para poder enseñar resultado
    pub last_response: Option<String>,
    pub last_response_error: Option<Result<String, String>>,
    pub first_row_idx: usize,
    pub last_row_idx: usize,
    pub n_rows_to_show: usize, // número de filas a mostrar, para paginar
    pub row_being_editted: SqlEditRowState,
    pub query_sort: QuerySort,
    pub column_index_selected: usize,
    pub change_order: bool,
    pub data_gen: SqlDataGenState,
}

impl SqlState {
    pub fn reset(&mut self) {
        self.first_row_idx = 0;
        self.last_row_idx = 0;
        self.current_table_rows.clear();
        self.current_table_columns.clear();
        self.last_response = None;
        self.last_response_error = None;
    }
}

impl Default for SqlState {
    fn default() -> Self {
        Self {
            column_visible: Default::default(),
            tables: Default::default(),
            current_connection_idx: usize::MAX,
            current_connection_tables_info: Default::default(),
            current_table_idx: usize::MAX,
            hide_connections: Default::default(),
            hide_tables: Default::default(),
            current_table_rows: Default::default(),
            current_table_columns: Default::default(),
            sql_statement: Default::default(),
            last_response: None,
            last_response_error: None,
            first_row_idx: 0,
            last_row_idx: 0,
            n_rows_to_show: 50,
            row_being_editted: Default::default(),
            query_sort: Default::default(),
            column_index_selected: usize::MAX,
            change_order: false,
            data_gen: Default::default(),
        }
    }
}
