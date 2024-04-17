// -------------------------------------------------------------------------
// Copyright (C) 2023 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use redis::Msg as PubSubMsg;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{common::traits::ToUrl, redism::presenter::RedisMenu};

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct RedisAppState {
    pub show_sidebar: bool,
    pub connections: Vec<RedisConnectionDefinition>,
}

pub struct PubSubState {
    pub channel: String,
    pub value: String,
    // pub tx: Sender<String>,
    // pub rx: Receiver<String>,
    pub tx: std::sync::mpsc::Sender<PubSubMsg>,
    pub rx: std::sync::mpsc::Receiver<PubSubMsg>,
    pub messages: HashMap<String, Vec<String>>,
    pub n_columns: usize,
}

impl Default for PubSubState {
    fn default() -> Self {
        // use tokio::sync::mpsc::{self, Receiver, Sender};
        // let (tx, rx) = mpsc::channel(8);
        let (tx, rx) = std::sync::mpsc::channel();

        Self {
            channel: String::default(),
            value: String::default(),
            tx,
            rx,
            messages: HashMap::default(),
            n_columns: 0,
        }
    }
}

pub struct RedisLocalState {
    pub cmd_history: Vec<String>,
    pub strings: Vec<(String, String)>,
    pub jsons: Vec<(String, String)>,
    pub streams: HashMap<String, Vec<String>>,
    pub hashes: HashMap<String, Vec<(String, String)>>, // nombre_hash: Lista de pares
    // Para poder mostrar y quitar a voluntad, donde guardo los valores de los streams. No guardo todo el listado de
    // mensajes porque puede ser eterno. Cuando hago click busco y pongo, y cuando click otra vez borro.
    pub stream_id_values: HashMap<String, HashMap<String, redis::Value>>,
    pub current_history_index: usize,
    pub current_command: String,
    pub is_first_update: bool,
    pub must_scan: bool,
    pub command_last_result: String,
    pub conn: Option<redis::Connection>, // La estoy gastando?
    pub selected_menu: RedisMenu,
    pub hide_connections: bool,
    pub hide_data_structures: bool,
    pub tmp_connection: RedisConnectionDefinition,
    pub current_connection: RedisConnectionDefinition,
    pub current_connection_idx: usize,
}

impl Default for RedisLocalState {
    fn default() -> Self {
        Self {
            cmd_history: Default::default(),
            strings: Default::default(),
            streams: Default::default(),
            hashes: Default::default(),
            stream_id_values: Default::default(),
            current_history_index: Default::default(),
            current_command: Default::default(),
            is_first_update: Default::default(),
            must_scan: Default::default(),
            command_last_result: Default::default(),
            conn: Default::default(),
            selected_menu: Default::default(),
            hide_connections: Default::default(),
            hide_data_structures: Default::default(),
            tmp_connection: Default::default(),
            current_connection: Default::default(),
            current_connection_idx: usize::MAX,
            jsons: Default::default(),
        }
    }
}

impl RedisLocalState {
    pub fn reset(&mut self) {
        self.strings.clear();
        self.streams.clear();
        self.hashes.clear();
    }

    pub fn reset_command(&mut self) {
        self.current_command.clear();
        self.command_last_result.clear();
    }
}

/// No tengo muy claro cómo hacerlo mejor.
/// Path y OsStr son más apropiadas pero problemáticas.
/// Voy con String y ya se verá si necesito cambiar.
#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct RedisConnectionDefinition {
    pub host: String,
    pub port: String,
    // pub user: String,
    // pub password: String,
}

impl ToUrl for RedisConnectionDefinition {
    fn to_url(&self) -> String {
        format!("redis://{}:{}", self.host, self.port)
    }
}
