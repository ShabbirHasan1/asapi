// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use super::bson_type::BsonType;
use super::filter::MongoFilter;
use super::filter::MongoOperator;
use super::{actions::MongoAction, connection::MongoConnection};
use bson::Document;
use serde::{Deserialize, Serialize};
use std::collections::{HashSet, VecDeque};

#[derive(Debug, PartialEq)]
pub enum MongoMessage {
    Databases(Vec<String>),
    Collections(Vec<String>),
    Documents((Vec<Document>, Vec<serde_json::Value>)),
    Error(String),
    InsertionSuccess,
    ReplaceSuccess,
    DeleteSuccess,
    UpdateSuccess,
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
    pub replace_new_document: String,
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
            replace_new_document: Default::default(),
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
    pub db_names: Vec<String>,
    pub collections: Vec<String>,
    pub last_error: Option<String>,
    pub current_db_collections: Vec<String>, // Colecciones de la db seleccionada en panel laterial
    // Filtro
    pub filters: VecDeque<MongoFilter>,
    pub current_parent: Option<usize>,
    pub next_idx: usize,
}

impl Default for MongoLocalState {
    fn default() -> Self {
        Self {
            current_operator: MongoOperator::Eq,
            current_available_keys: Default::default(),
            current_selected_key: Default::default(),
            current_filter_value: Default::default(),
            current_selected_type_bson_type: BsonType::Null,
            conn: Default::default(),
            tmp_conn_definition: Default::default(),
            current_selection: Default::default(),
            filters: VecDeque::default(), // En este punto solo filtros que no tienen padre, esto es, que están en el primmer nivel.
            current_parent: None,
            next_idx: 0,
            current_col_find_json_result: vec![],
            current_col_find_document_result: vec![],
            hide_databases: Default::default(),
            hide_connections: Default::default(),
            hide_collections: Default::default(),
            db_names: Default::default(),
            collections: Default::default(),
            last_error: Default::default(),
            current_db_collections: Default::default(),
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

    pub fn _parse_filters(&self) -> Vec<Document> {
        self.filters
            .iter()
            .map(|f| f.build_mongo_query())
            .collect::<Vec<Document>>()
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

#[derive(Debug)]
pub enum MongoError {
    ClientNotInitialized,
    CommandError(String),
}
