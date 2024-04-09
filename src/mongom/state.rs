// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use super::document::bson_type::BsonType;
use super::document::find::MongoOperator;
use super::{actions::MongoAction, connection::MongoConnection};
use bson::Document;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;

#[derive(Debug, PartialEq)]
pub enum MongoMessage {
    Databases(Vec<String>),
    Collections(Vec<String>),
    Documents((Vec<Document>, Vec<serde_json::Value>)),
    Error(String),
    InsertionSuccess,
    // Para enviar las claves que hay en el primer nivel de los documentos
    FirstLevelCollectionKeys(HashSet<String>),
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct MongoAppState {
    pub show_sidebar: bool,
    pub performance_table: bool,
    pub connections: Vec<MongoConnectionDefinition>,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct MongoConnectionDefinition {
    pub host: String,
    pub port: String,
    pub user: String,
    pub password: String,
    pub is_srv: bool,
}

#[derive(Clone, Debug)]
pub struct MongoCurrentSelection {
    pub conn_idx: usize,
    pub db_idx: usize,
    pub db_name: String,
    pub col_idx: usize,
    pub col_name: String,
    pub is_not: bool,
    // Dos distintos para poder mantener estado, que en caso de usar Option<String> no podría.
    pub show_user_free_input: bool,
    pub user_free_input: String,
}

impl Default for MongoCurrentSelection {
    fn default() -> Self {
        Self {
            conn_idx: usize::MAX,
            db_idx: usize::MAX,
            db_name: Default::default(),
            col_idx: usize::MAX,
            col_name: Default::default(),
            is_not: false,
            show_user_free_input: false,
            user_free_input: Default::default(),
        }
    }
}

pub struct MongoLocalState {
    // mantengo y cierro cuando elegimos otra como con postgres. En la documentaicón queda claro que esta es la forma de actuar y no crear un cliente cada vez, cuando habla de usarla en web frameworks.
    // pub conn: Box<Option<Client>>,
    pub conn: MongoConnection,
    pub tmp_conn_definition: MongoConnectionDefinition,
    pub current_selection: MongoCurrentSelection,
    pub current_col_find_json_result: Vec<serde_json::Value>, // Documentos encontrados con `find`.
    pub current_col_find_document_result: Vec<Document>,      // Documentos encontrados con `find`.
    pub current_available_keys: HashSet<String>,
    pub current_selected_key: String,
    pub current_selected_type_bson_type: BsonType,
    pub hide_connections: bool,
    pub hide_databases: bool,
    pub hide_collections: bool,
    pub selected_action: MongoAction,
    pub current_operator: MongoOperator,
    pub current_filter_value: String,
    pub filters: Vec<MongoFilter>,
    pub current_parent: Option<usize>,
    pub db_names: Vec<String>,
    pub collections: Vec<String>,
    pub last_error: Option<String>,
    // Aún no claro para qué las gasto en `Mongo`.
    // pub column_visible: Vec<bool>,
    pub current_db_collections: Vec<String>, // Colecciones de la db seleccionada en panel laterial
    // pub current_collection: String,          // Colleción seleccionada en panel central
    // pub current_table_data: Vec<Vec<String>>,
    // pub current_table_row_idx: usize, // para controlar qué muestro y qué no
    // pub current_table_end_idx: usize, // para controlar qué muestro y qué no
    pub rows_to_show: usize, // número de filas a mostrar, para paginar
                             // pub edit_row: MongoEditRowState,
                             // pub query_sort: QuerySort,
                             // pub column_index_selected: usize,
}

impl Default for MongoLocalState {
    fn default() -> Self {
        Self {
            current_operator: MongoOperator::EQ,
            current_available_keys: Default::default(),
            current_selected_key: Default::default(),
            // current_find_filter_value: "",
            current_filter_value: Default::default(),
            current_selected_type_bson_type: BsonType::Null,
            conn: Default::default(),
            tmp_conn_definition: Default::default(),
            current_selection: Default::default(),
            filters: vec![],
            current_parent: None,
            current_col_find_json_result: vec![],
            current_col_find_document_result: vec![],
            hide_databases: Default::default(),
            hide_connections: Default::default(),
            hide_collections: Default::default(),
            db_names: Default::default(),
            collections: Default::default(),
            last_error: Default::default(),

            // column_visible: Default::default(),
            // current_db_name: Default::default(),
            current_db_collections: Default::default(),
            // current_collection: Default::default(),
            // current_table_data: Default::default(),
            // current_table_row_idx: Default::default(),
            // current_table_end_idx: Default::default(),
            rows_to_show: Default::default(),
            // edit_row: Default::default(),
            selected_action: MongoAction::Find,
        }
    }
}

impl MongoLocalState {
    /// Resetamos el estado local
    ///
    /// Para ello limpiamos las colecciones, los documentos y
    /// la colección seleccionada.
    pub fn reset(&mut self) {
        // self.current_collection.clear();
        self.current_selection.col_idx = usize::MAX;
        self.current_selection.col_name.clear();
        self.current_col_find_json_result.clear();
        self.current_available_keys.clear();

        self.clean_filter();
    }

    pub fn clean_filter(&mut self) {
        self.filters.clear();
        self.current_selected_key.clear();
        self.current_filter_value.clear();
        self.current_parent = None;
        self.last_error = None;
    }
}

#[derive(Default)]
pub struct MongoEditRowState {
    pub row_data: Vec<String>,
    pub selected_row: Option<usize>,
    pub selected_table_name: Option<String>,
    pub n_rows_to_generate: u16,
    pub show_insertion_window: bool, // mostramos ventana para insertar valores a mano
    pub table_to_insert_data: Option<String>,
    pub selected_table_definition: Vec<(String, String)>,
    pub fixed_string_value: Vec<String>,
}

/// Filtro para buscar según características un clave y un valor
///
/// No usamos referencias directas a un padre y por lo tanto hijos dentro
/// del struct porque nos complicamos infinitamente mantener la referencia
/// de cuál es el padre actual.
#[derive(Debug, Clone)]
pub struct MongoFilter {
    pub op: MongoOperator,
    pub key: Option<String>, // Para And/Or
    pub val: Option<Value>,  // `serde_json::Value` para bson <-> str <-> serde fácilmente
    // pub children: Vec<MongoFilter>, // Para And/Or
    pub idx: usize,
    pub children: Vec<usize>,
    pub parent: Option<usize>, // todos tendrán salvo el inicial
}

#[derive(Debug, Clone)]
pub struct FilterList {
    pub filters: Vec<MongoFilter>,
    pub next_idx: usize, // Mantén un contador para asignar un nuevo idx único a cada nuevo filtro
}
