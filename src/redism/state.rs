// -------------------------------------------------------------------------
// Copyright (C) 2023 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use redis::Msg as PubSubMsg;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fmt::Display;

use crate::{common::traits::ToUrl, redism::connection::RedisMenu};

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

impl Display for RedisConnectionDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Redis {{ host: {}, port: {} }}", self.host, self.port)
    }
}

impl ToUrl for RedisConnectionDefinition {
    fn to_url(&self) -> String {
        format!("redis://{}:{}", self.host, self.port)
    }
}

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
            tx,
            value: String::default(),
            rx,
            messages: HashMap::default(),
            n_columns: 0,
        }
    }
}

#[derive(Default)]
pub struct RedisListState {
    pub lpush_k: String,
    pub lpush_vs: String,
    pub lpop_k: String,
    pub lpop_count: String,
    pub rpush_k: String,
    pub rpush_vs: String,
    pub rpop_k: String,
    pub rpop_count: String,
    pub llen_k: String,
    pub lrange_k: String,
    pub lrange_start: String,
    pub lrange_stop: String,
    pub lindex_k: String,
    pub lindex_idx: String,
    pub ltrim_k: String,
    pub ltrim_start: String,
    pub ltrim_stop: String,
    pub linsert_k: String,
    pub linsert_pivot: String,
    pub linsert_value: String,
    pub lrem_k: String,
    pub lrem_count: String,
    pub lrem_value: String,
    pub lset_k: String,
    pub lset_index: String,
    pub lset_value: String,
}

#[derive(Default)]
pub struct RedisStringState {
    pub set_k: String,
    pub set_v: String,
    pub set_offset: String, // SETRANGE
    pub append_str: String,
    pub get_k: String,
    pub getset_v: String,
    pub get_offset_from: String,    // GETRANGE
    pub get_offset_to: String,      // GETRANGE
    pub get_expire_seconds: String, // GET
    pub incr_k: String,
    pub incr_by_v: String,
    pub incr_byfloat_v: String,
    pub decr_k: String,
    pub decr_by_v: String,
    pub strlen_k: String,
    pub lcs_k1: String,
    pub lcs_k2: String,
    pub lcs_len: String,
    pub lcs_idx: String,
}

#[derive(Default)]
pub struct RedisHashState {
    pub hget_k: String,
    pub hget_f: String,
    pub hmget_k: String,
    pub hmget_fs: String,
    pub hgetall_k: String,
    pub hkeys_k: String,
    pub hvals_k: String,
    pub hdel_k: String,
    pub hdel_fs: String,
    pub hset_k: String,
    pub hset_f: String,
    pub hset_v: String,
    pub hsetnx_k: String,
    pub hsetnx_f: String,
    pub hsetnx_v: String,
    pub hincrby_k: String,
    pub hincrby_f: String,
    pub hincrby_increment: String,
    pub hincrbyfloat_k: String,
    pub hincrbyfloat_f: String,
    pub hincrbyfloat_increment: String,
    pub hlen_k: String,
    pub hstrlen_k: String,
    pub hstrlen_f: String,
    pub hexists_k: String,
    pub hexists_f: String,
    pub hrandfield_k: String,
    pub hrandfield_count: String,
}

#[derive(Default)]
pub struct RedisSetsState {
    pub sadd_k: String,
    pub sadd_vs: String,
    pub srem_k: String,
    pub srem_vs: String,
    pub spop_k: String,
    pub srandmember_k: String,
    pub srandmember_count: String,
    pub sismember_k: String,
    pub sismember_m: String,
    pub smismember_k: String,
    pub smismember_ms: String,
    pub scard_k: String,
    pub smembers_k: String,
    pub sinter_ks: String,
    pub sintercard_numkeys: String,
    pub sintercard_ks: String,
    pub sinterstore_destination: String,
    pub sinterstore_ks: String,
    pub sdiff_ks: String,
    pub sdiffstore_destination: String,
    pub sdiffstore_ks: String,
    pub sunion_ks: String,
    pub sunionstore_destination: String,
    pub sunionstore_ks: String,
}

#[derive(Default)]
pub struct RedisZSetsState {
    pub zadd_k: String,
    pub zadd_score: String,
    pub zadd_v: String,
    pub zrem_k: String,
    pub zrem_vs: String,
    pub zmpop_ks: String,
    pub zmpop_min_max: String,
    pub zmpop_count: String,
    pub zrandmember_k: String,
    pub zrandmember_count: String,
    pub zcard_k: String,
    pub zrange_k: String,
    pub zrange_start: String,
    pub zrange_stop: String,
    pub zrangestore_k: String,
    pub zrangestore_start: String,
    pub zrangestore_stop: String,
    pub zrangestore_destination: String,
    pub zrangebylex_k: String,
    pub zrangebylex_min: String,
    pub zrangebylex_max: String,
    pub zrangebyscore_k: String,
    pub zrangebyscore_min: String,
    pub zrangebyscore_max: String,
    pub zinter_ks: String,
    pub zintercard_ks: String,
    pub zinterstore_ks: String,
    pub zinterstore_destination: String,
    pub zunionstore_ks: String,
    pub zunionstore_destination: String,
    pub zunionstore_min_max: String,
    pub zrank_k: String,
    pub zrank_m: String,
    pub zrevrank_k: String,
    pub zrevrank_m: String,
    pub zremrangebyrank_k: String,
    pub zremrangebyrank_start: String,
    pub zremrangebyrank_stop: String,
}

#[derive(Default)]
pub struct RedisJsonState {
    pub json_get_k: String,
    pub json_get_p: String,
    pub json_mget_ks: String,
    pub json_mget_p: String,
    pub json_objkeys_k: String,
    pub json_objkeys_p: String,
    pub json_objlen_k: String,
    pub json_objlen_p: String,
    pub json_strlen_k: String,
    pub json_strlen_p: String,
    pub json_set_k: String,
    pub json_set_p: String,
    pub json_set_v: String,
    pub json_set_nx_xx: String,
    pub json_del_k: String,
    pub json_del_p: String,
    pub json_forget_k: String,
    pub json_forget_p: String,
    pub json_clear_k: String,
    pub json_clear_p: String,
    pub json_strappend_k: String,
    pub json_strappend_p: String,
    pub json_strappend_v: String,
    pub json_arrappend_k: String,
    pub json_arrappend_p: String,
    pub json_arrappend_vs: String,
    pub json_arrindex_k: String,
    pub json_arrindex_p: String,
    pub json_arrindex_v: String,
    pub json_arrindex_start: String,
    pub json_arrlen_k: String,
    pub json_arrlen_p: String,
    pub json_arrindex_stop: String,
    pub json_arrinsert_k: String,
    pub json_arrinsert_p: String,
    pub json_arrinsert_vs: String,
    pub json_arrinsert_idx: String,
    pub json_arrpop_k: String,
    pub json_arrpop_p: String,
    pub json_arrpop_idx: String,
    pub json_arrtrim_k: String,
    pub json_arrtrim_p: String,
    pub json_arrtrim_start: String,
    pub json_arrtrim_stop: String,
    pub json_numincrby_k: String,
    pub json_numincrby_p: String,
    pub json_numincrby_v: String,
    pub json_nummultby_k: String,
    pub json_nummultby_p: String,
    pub json_nummultby_v: String,
    pub json_type_k: String,
    pub json_type_p: String,
    pub json_merge_k: String,
    pub json_merge_p: String,
    pub json_merge_v: String,
    pub json_toggle_k: String,
    pub json_toggle_p: String,
}

pub struct RedisLocalState {
    pub cmd_history: Vec<String>,
    pub strings: BTreeMap<String, String>,
    pub lists: HashMap<String, Vec<String>>,
    pub sets: HashMap<String, Vec<String>>,
    pub hashes: HashMap<String, Vec<(String, String)>>, // nombre_hash: Lista de pares
    pub sorted_sets: HashMap<String, Vec<String>>,
    // El valor es el json como string.
    pub jsons: BTreeMap<String, String>,
    pub streams: HashMap<String, Vec<String>>,
    // Para poder mostrar y quitar a voluntad, donde guardo los valores de los streams. No guardo todo el listado de
    // mensajes porque puede ser eterno. Cuando hago click busco y pongo, y cuando click otra vez borro.
    pub stream_id_values: HashMap<String, HashMap<String, redis::Value>>,
    pub current_history_index: usize,
    pub current_command: String,
    pub is_first_update: bool,
    pub must_scan: bool,
    // pub last_result: String,
    pub last_result: Option<Result<String, String>>,
    pub conn: Option<redis::Connection>, // La estoy gastando?
    pub selected_menu: RedisMenu,
    pub hide_connections: bool,
    pub hide_data_structures: bool,
    pub tmp_connection: RedisConnectionDefinition,
    pub current_connection: RedisConnectionDefinition,
    pub current_connection_idx: usize,
    pub string_st: RedisStringState,
    pub list_st: RedisListState,
    pub sets_st: RedisSetsState,
    pub hash_st: RedisHashState,
    pub ssets_st: RedisZSetsState,
    pub json_st: RedisJsonState,
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
            // last_result: Default::default(),
            last_result: Default::default(),
            conn: Default::default(),
            selected_menu: Default::default(),
            hide_connections: Default::default(),
            hide_data_structures: Default::default(),
            tmp_connection: Default::default(),
            current_connection: Default::default(),
            current_connection_idx: usize::MAX,
            jsons: Default::default(),
            lists: Default::default(),
            sets: Default::default(),
            sorted_sets: Default::default(),
            string_st: Default::default(),
            list_st: Default::default(),
            sets_st: Default::default(),
            hash_st: Default::default(),
            ssets_st: Default::default(),
            json_st: Default::default(),
        }
    }
}

impl RedisLocalState {
    pub fn reset(&mut self, menu_option: RedisMenu) {
        match menu_option {
            RedisMenu::All => self.clean_all(),
            RedisMenu::String => self.strings.clear(),
            RedisMenu::Json => self.jsons.clear(),
            RedisMenu::List => self.lists.clear(),
            RedisMenu::Set => self.sets.clear(),
            RedisMenu::Hash => self.hashes.clear(),
            RedisMenu::SortedSet => self.sorted_sets.clear(),
            RedisMenu::Stream => self.streams.clear(),
            _ => (),
        };
    }

    fn clean_all(&mut self) {
        self.strings.clear();
        self.streams.clear();
        self.hashes.clear();
        self.jsons.clear();
        self.lists.clear();
        self.sets.clear();
        self.sorted_sets.clear();
    }

    pub fn reset_command(&mut self) {
        self.current_command.clear();
        self.last_result = None;
    }
}
