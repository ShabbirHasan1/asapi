// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use serde::{Deserialize, Serialize};
use sqlx::{Database, Pool};
use std::collections::HashMap;

#[derive(PartialEq, Debug, Default)]
pub enum QuerySort {
    #[default]
    None,
    Asc,
    Desc,
}

/// No tengo muy claro cómo hacerlo mejor.
/// Path y OsStr son más apropiadas pero problemáticas.
/// Voy con String y ya se verá si necesito cambiar.
#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct SqlConnectionDefinition {
    pub name: String,
    pub host: String,
    pub port: String,
    pub user: String,
    pub password: String,
    pub dbname: String,
}

impl PartialEq for SqlConnectionDefinition {
    fn eq(&self, other: &Self) -> bool {
        // No uso password xq no tiene sentido conexiones al mismo sitio
        // con distinto password.
        self.name == other.name
            && self.host == other.host
            && self.port == other.port
            && self.dbname == other.dbname
            && self.user == other.user
    }
}

#[derive(Serialize, Clone, Debug, Deserialize)]
pub struct SqlAppState {
    pub show_sidebar: bool,
    pub performance_table: bool,
    pub connections: Vec<SqlConnectionDefinition>,
}

impl Default for SqlAppState {
    fn default() -> Self {
        Self {
            show_sidebar: true,
            connections: Default::default(),
            performance_table: Default::default(),
        }
    }
}

#[derive(Default)]
pub struct SqlLocalState<T: Database> {
    pub pool: Option<Pool<T>>,
    // pub current_connection: SqlConnectionDefinition,
    // Datos que almacenamos de forma temporal.
    // pub tmp_sql_connection: SqlConnectionDefinition,
    pub sql: SqlState,
}

#[derive(Debug)]
pub enum SqlxMessage {
    InsertStatement(String), // stmt
    DeleteStatement((String, usize)),
    DeleteAllStmt(String), // table name
    // Datos -- Definición columnas -- Mostrar todas las columnas
    SelectResponse((Vec<Vec<String>>, Vec<(String, String)>, bool)),
    Error(String),
    Empty, // para errores, pero para poder resetear (o cualquier otra cosa que necesitemos).
    AddConnection(SqlConnectionDefinition),
    EditConnection((usize, SqlConnectionDefinition)),
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

pub struct SqlState {
    pub column_visible: Vec<bool>,
    pub tables: Vec<String>,
    // Información de las tablas de la conexión, para limitar consultas.
    // {
    //  "tabla1": [["Nombre1", "Tipo1", ...],["Nombre2", "Tipo2", ...]]
    //  "tabla2": [["Nombre1", "Tipo1", ...],["Nombre2", "Tipo2", ...]]
    //  ...
    // }
    // Con Vector<Vector<String>> porque puede ser que distintos motores nos proporcionen distinta información que nos pueda interesar más o menos.
    pub current_connection_tables_info: HashMap<String, Vec<Vec<String>>>,
    // La conexión/archivo elegido en menú lateral normalmente.
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

#[derive(Default)]
pub struct SqlEditRowState {
    pub row_data: Vec<String>,
    pub selected_row: Option<usize>,
}
