// -------------------------------------------------------------------------
// Copyright (C) 2023 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use redis::streams::{StreamReadOptions, StreamReadReply};
use redis::JsonCommands;
use redis::{self, Client, Commands, Connection, Msg as PubSubMsg, RedisError, RedisResult, Value};
use std::collections::HashMap;
use std::num::NonZeroUsize;

use crate::{error, info};

use super::state::{
    RedisConnectionDefinition, RedisListState, RedisLocalState, RedisSetsState,
    RedisSortedSetsState,
};

// ========================================================================
// I do not create Presenter to share client/connection because reading the
// docs, seems like the way to go is creating new pair each time we want to
// make something and not sharing it in any other place.
// ========================================================================
// pub struct RedisPresenter {
//     client: Option<redis::Client>,
// }

// impl RedisPresenter {
//     fn new(host: String, port: i8) -> Self {
//         Self {
//             client: redis::Client::open(format!("redis://{}:{}", host, port)),
//         }
//     }
// }

#[derive(Clone, Debug, PartialEq, Copy, Default)]
pub enum RedisMenu {
    All,
    String,
    List,
    #[default]
    Set,
    Hash,
    SortedSet,
    Json,
    Stream,
    PubSub,
}

#[inline(always)]
pub fn create_conn(host: &str, port: i16) -> Result<redis::Connection, RedisError> {
    //if Redis server needs secure connection // https://medium.com/swlh/tutorial-getting-started-with-rust-and-redis-69041dd38279
    // let uri_scheme = match env::var("IS_TLS") {
    //     Ok(_) => "rediss",
    //     Err(_) => "redis",
    // };

    let client = Client::open(format!("redis://{}:{}", host, port))?;
    client.get_connection()
}

#[inline(always)]
pub fn create_conn_with_default(host: &str, port: &str) -> Result<redis::Connection, RedisError> {
    let port = port.parse::<i16>().unwrap_or(6379); // Using 6379 as default value;
    create_conn(host, port)
}

#[inline(always)]
fn create_redis_connection(
    conn: &RedisConnectionDefinition,
) -> Result<redis::Connection, RedisError> {
    create_conn_with_default(&conn.host, &conn.port)
}

/// Escaneo de toda la instancia de redis.
///
/// Pasamos de forma explícita la opción elegida aunque ya hay una en el estado
/// para permitir recarga bajo demanda en menú contextual de cada estructura de datos.
pub fn scan(state: &mut RedisLocalState, option: RedisMenu) -> RedisResult<()> {
    // let option = state.selected_menu;
    // let client = Client::open("redis://127.0.0.1/")?;
    // let mut con: Connection = client.get_connection()?;
    // let port = app_state.redis.port.parse::<i16>().unwrap_or(6379); // Using 6379 as default value;
    // let mut con: Connection = create_conn(&app_state.redis.host, port)?;
    let mut con: Connection = create_conn_with_default(
        &state.current_connection.host,
        &state.current_connection.port,
    )?;
    let mut cursor: u64 = 0;

    match option {
        RedisMenu::All => state.reset(),
        RedisMenu::String => state.strings.clear(),
        RedisMenu::Json => state.jsons.clear(),
        RedisMenu::List => state.lists.clear(),
        RedisMenu::Set => state.sets.clear(),
        RedisMenu::Hash => state.hashes.clear(),
        RedisMenu::SortedSet => state.sorted_sets.clear(),
        RedisMenu::Stream => state.streams.clear(),
        _ => (),
    };

    loop {
        let scan_result: (u64, Vec<String>) = redis::cmd("SCAN").arg(cursor).query(&mut con)?;

        cursor = scan_result.0;

        for key in scan_result.1 {
            let key_type = redis::Cmd::key_type(&key).query::<String>(&mut con);

            match key_type {
                Ok(value) => match value.as_str() {
                    "string" => {
                        if option == RedisMenu::String || option == RedisMenu::All {
                            let value = con.get(key.clone()).unwrap();
                            state.strings.insert(key, value);
                        }
                    }
                    "ReJSON-RL" => {
                        if option == RedisMenu::Json || option == RedisMenu::All {
                            let value = con.json_get(key.clone(), "$").unwrap();
                            state.jsons.push((key, value));
                        }
                    }
                    "list" => {
                        if option == RedisMenu::List || option == RedisMenu::All {
                            let value = con.lrange(key.clone(), 0, isize::MAX).unwrap();
                            state.lists.insert(key, value);
                        }
                    }
                    "set" => {
                        if option == RedisMenu::Set || option == RedisMenu::All {
                            let value = con.smembers(key.clone()).unwrap();
                            state.sets.insert(key, value);
                        }
                    }
                    "zset" => {
                        if option == RedisMenu::SortedSet || option == RedisMenu::All {
                            let value = con.zrange(key.clone(), 0, -1).unwrap();
                            state.sorted_sets.insert(key, value);
                        }
                    }
                    "hash" => {
                        if option == RedisMenu::Hash || option == RedisMenu::All {
                            let result = con.hgetall(key.clone()).unwrap();
                            state.hashes.insert(key.clone(), result);
                        }
                    }
                    "stream" => {
                        if option == RedisMenu::Stream || option == RedisMenu::All {
                            let key_c = key.clone();
                            state.streams.insert(key_c, Vec::new());
                            let opts = StreamReadOptions::default(); //.count(10);
                            let result: RedisResult<StreamReadReply> =
                                con.xread_options(&[key.as_str()], &["0"], &opts);

                            match result {
                                Ok(stream_keys) => {
                                    let ids = state.streams.get_mut(&key).unwrap();

                                    for k in stream_keys.keys {
                                        for v in k.ids {
                                            ids.push(v.id.clone());
                                        }
                                    }
                                }
                                Err(e) => error!("Ocurrió un error {}", e),
                            }
                        }
                    }
                    _ => error!("Unknown type: {}", value),
                },
                Err(e) => {
                    error!("Ocurrió un error: {}", e);
                }
            }
        }

        if cursor == 0 {
            break;
        }
    }

    Ok(())
}

fn redis_value_to_string(v: &redis::Value) -> String {
    match v {
        Value::Nil => "Nil".to_string(),
        Value::Int(i) => i.to_string(),
        Value::Data(d) => String::from_utf8(d.clone())
            .unwrap_or_else(|err| format!("ERROR {err:?} parsing {d:?}")),
        Value::Bulk(b) => b
            .iter()
            .map(redis_value_to_string)
            .collect::<Vec<String>>()
            .join(", "),
        Value::Status(s) => s.clone(),
        Value::Okay => "OK".to_string(),
    }
}

pub fn run_redis_command<F: FnMut(&mut redis::Connection) -> String>(
    conn_def: &RedisConnectionDefinition,
    mut cb: F,
) -> String {
    let connection = create_redis_connection(conn_def);

    if let Ok(mut conn) = connection {
        cb(&mut conn)
    } else {
        "ERROR :: Not able to connect to {conn}.".to_string()
    }
}

// Leemos los datos de un stream concreto
// TODO: ... no tengo muy claro aún para qué lo gasto.
pub fn read_stream_id(
    stream_key: &str,
    id: &str,
    state: &mut HashMap<String, HashMap<String, Value>>,
) -> RedisResult<()> {
    // pub fn read_stream_id(stream_key: &str, id: &str) -> RedisResult<()> {
    // TODO: Connection in state??
    let client = Client::open("redis://127.0.0.1/")?;
    let mut con: Connection = client.get_connection()?;

    // This gives the next one to the one inside the array, so filtering with rust until other option.
    let opts = StreamReadOptions::default().count(1);
    let result: RedisResult<StreamReadReply> = con.xread_options(&[stream_key], &[id], &opts);
    // let result: RedisResult<StreamReadReply> = con.xread(&[stream_key], &["0"]);

    match result {
        Ok(stream_keys) => {
            for entry in stream_keys.keys {
                info!("Stream keys: {}, asked for {}", entry.key, id);
                info!("  {}", entry.ids.len());
                for stream_id in entry.ids {
                    // if stream_id.id != id {
                    //     continue;
                    // }
                    info!(" - entry id: {}", stream_id.id);
                    state.insert(stream_id.id, stream_id.map.clone());
                    info!("{:?}", stream_id.map);
                    // .stream_id_values
                    // for (k, v) in stream_id.map {
                    //     match v {
                    //         redis::Value::Nil => info!("     NIL       k: {}", k),
                    //         redis::Value::Int(i) => info!("     INT       k: {}, {}", k, i),
                    //         redis::Value::Data(d) => {
                    //             info!(
                    //                 "     DATA      k: {}, {}",
                    //                 k,
                    //                 String::from_utf8(d).unwrap()
                    //             )
                    //         }
                    //         redis::Value::Bulk(b) => info!("     BULK      k: {}", k),
                    //         redis::Value::Status(s) => info!("     STATUS    k: {}", k),
                    //         redis::Value::Okay => info!("     OKAY      k: {}", k),
                    //     }
                    // }
                }
            }
        }
        Err(e) => info!("Ocurrió un error {}", e),
    }

    Ok(())
}

pub fn run_command(host: &str, port: &str, cmd: &str) -> Result<String, String> {
    if let Some((command, args)) = parse_command(cmd) {
        info!("\nCommando\n{:?}\n{:?}", command, args);

        match create_conn_with_default(host, port) {
            Ok(mut c) => {
                let response = redis::cmd(command).arg(&args).query::<Value>(&mut c);
                match response {
                    Ok(value) => return Ok(format!("{:?}", value)),
                    Err(e) => return Err(format!("Error parsing response: {:?}", e)),
                }
            }
            Err(e) => return Err(format!("{}", e)),
        }
    }

    Err("error parsing command".to_string())
}

fn parse_command<'a>(input: &'a str) -> Option<(&'a str, Vec<&'a str>)> {
    let mut words = input.split_whitespace().collect::<Vec<&'a str>>();

    if words.is_empty() {
        return None;
    }

    let initial = words.remove(0);

    Some((initial, words))
}

pub fn delete_stream_message(
    host: &str,
    port: i16,
    stream_name: &str,
    msg_id: &str,
) -> RedisResult<i8> {
    create_conn(host, port).and_then(|mut con| con.xdel::<&str, &str, i8>(stream_name, &[msg_id]))
}

pub fn delete_key(host: &str, port: &str, key: &str) -> RedisResult<i8> {
    create_conn_with_default(host, port).and_then(|mut con| con.del(key))
}

// Borrado por entrada, un hash entero no se puede borrar. Se borra cuando no le quedan entradas.
pub fn delete_hashkey(host: &str, port: &str, hash_name: &str, field_key: &str) -> RedisResult<i8> {
    create_conn_with_default(host, port).and_then(|mut con| con.hdel(hash_name, field_key))
}

pub fn publish_to_channel(
    host: &str,
    port: &str,
    channel: &str,
    message: &str,
) -> RedisResult<bool> {
    create_conn_with_default(host, port).and_then(|mut conn| conn.publish(channel, message))
}

pub fn subscribe_to_channel_std_thread(
    host: &str,
    port: &str,
    channel: &str,
    tx: &std::sync::mpsc::Sender<PubSubMsg>,
) -> Result<(), RedisError> {
    let mut conn = create_conn_with_default(host, port)?;
    let owned_channel = channel.to_owned();
    let tx_owned = tx.to_owned();

    std::thread::spawn(move || -> Result<(), RedisError> {
        let mut pubsub = conn.as_pubsub();
        pubsub.subscribe(&owned_channel).unwrap();

        loop {
            let msg = pubsub.get_message()?;
            let payload: String = msg.get_payload()?;

            match payload.as_ref() {
                "#break#" => {
                    info!(
                        ">>> Finishing subscription to channel {} <<<",
                        msg.get_channel_name()
                    );
                    break;
                } //ControlFlow::Break(()),
                _ => (), // ControlFlow::Continue
            }

            let _ = tx_owned.send(msg);
        }
        pubsub.unsubscribe(&owned_channel)?;

        Ok(())
    });
    Ok(())
}

pub enum NumericValue {
    Int,
    Float,
}

pub struct StringPresenter;

impl StringPresenter {
    pub fn lcs(st: &mut RedisLocalState) {
        let connection = create_redis_connection(&st.current_connection);

        if let Ok(mut conn) = connection {
            let k1 = st.string_st.lcs_k1.clone();
            let k2 = st.string_st.lcs_k2.clone();
            let is_len_filled = !st.string_st.lcs_len.is_empty();
            let is_idx_filled = !st.string_st.lcs_idx.is_empty();

            // let response = redis::cmd(command).arg(&args).query::<Value>(&mut c);
            let result = if is_len_filled && is_idx_filled {
                redis::cmd("LCS")
                    .arg(k1)
                    .arg(k2)
                    .arg(&st.string_st.lcs_len)
                    .arg(&st.string_st.lcs_idx)
                    .query::<Value>(&mut conn)
            } else {
                redis::cmd("LCS").arg(k1).arg(k2).query::<Value>(&mut conn)
            };

            match result {
                Ok(v) => {
                    let parsed_v = redis_value_to_string(&v);
                    st.command_last_result = format!("LCS :: {parsed_v}");
                }
                Err(err) => {
                    st.command_last_result = format!("ERROR LCS :: {err:?}");
                }
            }
        }
    }

    pub fn str_len(st: &mut RedisLocalState) {
        let connection = create_redis_connection(&st.current_connection);

        if let Ok(mut conn) = connection {
            let k = st.string_st.strlen_k.clone();

            match conn.strlen::<&str, i64>(&k) {
                Ok(len) => {
                    st.command_last_result = format!("STRLEN :: Key: {k}, Len: {len}");
                }
                Err(err) => {
                    st.command_last_result = format!("ERROR STRLEN :: {err:?}");
                }
            }
        }
    }

    pub fn append(st: &mut RedisLocalState) {
        let connection = create_redis_connection(&st.current_connection);

        if let Ok(mut conn) = connection {
            let (k, v) = (st.string_st.set_k.clone(), st.string_st.append_str.clone());

            match conn.append::<&str, &str, redis::Value>(&k, &v) {
                Ok(redis_v) => {
                    st.strings.insert(
                        k.clone(),
                        st.strings.get(&k).unwrap_or(&"".to_string()).to_owned() + &v,
                    );
                    st.command_last_result = format!(
                        "APPEND :: Resultado: {v} caracteres totales",
                        v = redis_value_to_string(&redis_v)
                    );
                }
                Err(err) => {
                    st.command_last_result = format!("ERROR APPEND :: {err:?}");
                }
            }
        }
    }

    pub fn set(st: &mut RedisLocalState) {
        let connection = create_redis_connection(&st.current_connection);

        if let Ok(mut conn) = connection {
            let (k, v) = (st.string_st.set_k.clone(), st.string_st.set_v.clone());
            let result = conn.set(&k, &v);

            match result {
                Ok(rresp) => {
                    st.strings.insert(k, v);
                    let parsed_rresp = redis_value_to_string(&rresp);
                    st.command_last_result = format!("SET :: {parsed_rresp}");
                }
                Err(err) => {
                    st.command_last_result = format!("ERROR SET :: {err:?}");
                }
            }
        }
    }

    pub fn set_nx(st: &mut RedisLocalState) {
        let connection = create_redis_connection(&st.current_connection);

        if let Ok(mut conn) = connection {
            let (k, v) = (st.string_st.set_k.clone(), st.string_st.set_v.clone());
            let result = conn.set_nx(&k, &v);

            match result {
                Ok(rresp) => {
                    st.strings.insert(k, v);
                    let parsed_rresp = redis_value_to_string(&rresp);
                    st.command_last_result = format!("SETNX :: {parsed_rresp}");
                }
                Err(err) => {
                    st.command_last_result = format!("ERROR SETNX: {err:?}");
                }
            }
        }
    }

    pub fn set_range(st: &mut RedisLocalState) {
        let connection = create_redis_connection(&st.current_connection);

        if let Ok(mut conn) = connection {
            let (k, v) = (st.string_st.set_k.clone(), st.string_st.set_v.clone());

            let i_result = st.string_st.set_offset.parse::<isize>();

            match i_result {
                Ok(i) => {
                    let result = conn.setrange(&k, i, &v);

                    match result {
                        Ok(rresp) => {
                            st.strings.insert(k, v.to_string());
                            let parsed_rresp = redis_value_to_string(&rresp);
                            st.command_last_result = format!("SETRANGE :: {parsed_rresp}");
                        }
                        Err(err) => {
                            st.command_last_result = format!("ERROR SETRANGE :: {err:?}");
                        }
                    }
                }
                Err(err) => {
                    st.command_last_result = format!("PARSE ERROR :: {err:?}");
                }
            }
        }
    }

    // Muy rimbombante para ver cómo podía hacerlo. No queda muy bien.
    pub fn _incr(st: &mut RedisLocalState, v: &str, t: NumericValue) {
        let connection = create_redis_connection(&st.current_connection);

        if let Ok(mut conn) = connection {
            let k = st.string_st.incr_k.clone();
            let response = match t {
                NumericValue::Int => v
                    .parse::<i64>()
                    .map_err(|err| format!("PARSE ERROR :: {err:?}"))
                    .and_then(|i| {
                        conn.incr::<&str, i64, redis::Value>(&k, i)
                            .map_err(|err| format!("{err:?}"))
                            .map(|v| redis_value_to_string(&v))
                    }),
                NumericValue::Float => v
                    .parse::<f32>()
                    .map_err(|err| format!("PARSE ERROR :: {err:?}"))
                    .and_then(|i| {
                        conn.incr::<&str, f32, redis::Value>(&k, i)
                            .map_err(|err| format!("{err:?}"))
                            .map(|v| redis_value_to_string(&v))
                    }),
            };

            match response {
                Ok(v) => {
                    st.strings.insert(k, v.clone());
                    st.command_last_result = format!("INCR :: {v}");
                }
                Err(err) => {
                    st.command_last_result = format!("ERROR INCR :: {err:?}");
                }
            }
        }
    }

    pub fn incr(st: &mut RedisLocalState) {
        Self::_incr(st, "1", NumericValue::Int);
    }

    pub fn incr_by(st: &mut RedisLocalState) {
        Self::_incr(st, &st.string_st.incr_by_v.to_owned(), NumericValue::Int);
    }

    pub fn incr_byfloat(st: &mut RedisLocalState) {
        Self::_incr(
            st,
            &st.string_st.incr_byfloat_v.to_owned(),
            NumericValue::Float,
        );
    }

    pub fn _decr(st: &mut RedisLocalState, v: &str) {
        let connection = create_redis_connection(&st.current_connection);

        if let Ok(mut conn) = connection {
            let k = st.string_st.decr_k.clone();
            let response = v
                .parse::<i64>()
                .map_err(|err| format!("{err:?}"))
                .and_then(|i| {
                    conn.decr::<&str, i64, redis::Value>(&k, i)
                        .map_err(|err| format!("{err:?}"))
                        .map(|value| redis_value_to_string(&value))
                });

            match response {
                Ok(v) => {
                    st.strings.insert(k, v.clone());
                    st.command_last_result = format!("DECR :: {v}");
                }
                Err(err) => {
                    st.command_last_result = format!("ERROR DECR :: {err:?}");
                }
            }
        }
    }

    pub fn decr(st: &mut RedisLocalState) {
        Self::_decr(st, "1");
    }

    pub fn decr_by(st: &mut RedisLocalState) {
        Self::_decr(st, &st.string_st.decr_by_v.to_owned());
    }

    pub fn get(st: &mut RedisLocalState) {
        let connection = create_redis_connection(&st.current_connection);

        if let Ok(mut conn) = connection {
            let k = st.string_st.get_k.clone();
            let result = conn.get::<&str, String>(&k);

            st.command_last_result = match result {
                Ok(msg) => format!("GET :: Key: {k}, Value: {msg}"),
                Err(err) => format!("ERROR GET :: {err:?}"),
            }
        }
    }

    pub fn get_del(st: &mut RedisLocalState) {
        let connection = create_redis_connection(&st.current_connection);

        if let Ok(mut conn) = connection {
            let k = st.string_st.get_k.clone();
            let result = conn.get_del::<&str, String>(&k);

            match result {
                Ok(msg) => {
                    st.strings.remove(&k);
                    st.command_last_result = format!("GETDEL :: Key: {k}, Value: {msg}");
                }
                Err(err) => {
                    st.command_last_result = format!("ERROR GETDEL :: {err:?}");
                }
            }
        }
    }

    pub fn get_set(st: &mut RedisLocalState) {
        let connection = create_redis_connection(&st.current_connection);

        if let Ok(mut conn) = connection {
            let (k, v) = (st.string_st.get_k.clone(), st.string_st.getset_v.clone());
            let result = conn.getset::<&str, &str, String>(&k, &v);

            match result {
                Ok(msg) => {
                    st.strings.insert(k.clone(), v.to_string());
                    st.command_last_result = format!("GETSET :: Key: {k}, Old Value: {msg}");
                }
                Err(err) => {
                    st.command_last_result = format!("ERROR GETSET :: {err:?}");
                }
            }
        }
    }

    pub fn get_range(st: &mut RedisLocalState) {
        let connection = create_redis_connection(&st.current_connection);

        if let Ok(mut conn) = connection {
            let (k, from_s, to_s) = (
                st.string_st.get_k.clone(),
                st.string_st.get_offset_from.clone(),
                st.string_st.get_offset_to.clone(),
            );

            if let (Ok(f), Ok(t)) = (from_s.parse::<isize>(), to_s.parse::<isize>()) {
                let result = conn.getrange::<&str, String>(&k, f, t);
                st.command_last_result = match result {
                    Ok(msg) => format!("GETRANGE :: Key: {k}, Value: {msg}"),
                    Err(err) => format!("ERROR GETRANGE :: {err:?}"),
                }
            }
        }
    }

    pub fn get_ex(st: &mut RedisLocalState) {
        let connection = create_redis_connection(&st.current_connection);

        if let Ok(mut conn) = connection {
            let (k, expire_at_s) = (
                st.string_st.get_k.clone(),
                st.string_st.get_expire_seconds.clone(),
            );

            if let Ok(ex) = expire_at_s.parse::<usize>() {
                let result = conn.get_ex::<&str, String>(&k, redis::Expiry::EX(ex));
                st.command_last_result = match result {
                    Ok(msg) => format!("GETEX :: Key: {k}, Value: {msg}"),
                    Err(err) => format!("ERROR GETEX :: {err:?}"),
                }
            }
        }
    }

    pub fn _get(conn: &mut redis::Connection, k: &str) -> Result<String, RedisError> {
        conn.get::<&str, String>(k)
    }
}

pub enum RedisPosition {
    Before,
    End,
}

pub struct ListPresenter;

impl ListPresenter {
    pub fn lset(
        conn: &mut redis::Connection,
        hm: &mut HashMap<String, Vec<String>>,
        st: &mut RedisListState,
    ) -> String {
        let index = st.lset_index.parse::<isize>().unwrap_or(0);
        let k = st.lset_k.clone();

        match conn.lset(&k, index, &st.lset_value) {
            Ok(rresp) => {
                let value: Vec<String> = conn.lrange(&k, 0, isize::MAX).unwrap();
                hm.insert(k, value);
                format!(
                    "LSET :: {parsed_rresp}",
                    parsed_rresp = redis_value_to_string(&rresp)
                )
            }
            Err(err) => format!("ERROR LSET :: {err:?}"),
        }
    }

    pub fn lrem(
        conn: &mut redis::Connection,
        hm: &mut HashMap<String, Vec<String>>,
        st: &mut RedisListState,
    ) -> String {
        let count = st.lrem_count.parse::<isize>().unwrap_or(0);
        let k = st.lrem_k.clone();

        match conn.lrem(&k, count, &st.lrem_value) {
            Ok(rresp) => {
                let value: Vec<String> = conn.lrange(&k, 0, isize::MAX).unwrap();
                hm.insert(k, value);
                format!(
                    "LREM :: {parsed_rresp}",
                    parsed_rresp = redis_value_to_string(&rresp)
                )
            }
            Err(err) => format!("ERROR LREM :: {err:?}"),
        }
    }

    pub fn linsert(
        conn: &mut redis::Connection,
        hm: &mut HashMap<String, Vec<String>>,
        st: &mut RedisListState,
        position: RedisPosition,
    ) -> String {
        let response = match position {
            RedisPosition::Before => {
                conn.linsert_before(&st.linsert_k, &st.linsert_pivot, &st.linsert_value)
            }
            RedisPosition::End => {
                conn.linsert_after(&st.linsert_k, &st.linsert_pivot, &st.linsert_value)
            }
        };
        match response {
            Ok(rresp) => {
                let value: Vec<String> = conn.lrange(&st.linsert_k, 0, isize::MAX).unwrap();
                hm.insert(st.linsert_k.clone(), value);
                format!(
                    "LINSERT :: {parsed_rresp}",
                    parsed_rresp = redis_value_to_string(&rresp)
                )
            }
            Err(err) => format!("ERROR LINSERT :: {err:?}"),
        }
    }

    pub fn ltrim(
        conn: &mut redis::Connection,
        hm: &mut HashMap<String, Vec<String>>,
        st: &mut RedisListState,
    ) -> String {
        let (start, stop) = (
            st.lrange_start.parse::<isize>(),
            st.lrange_stop.parse::<isize>(),
        );

        match (start, stop) {
            (Ok(b), Ok(e)) => match conn.ltrim(&st.ltrim_k, b, e) {
                Ok(rresp) => {
                    let value: Vec<String> = conn.lrange(&st.ltrim_k, 0, isize::MAX).unwrap();
                    hm.insert(st.ltrim_k.clone(), value);
                    format!(
                        "LTRIM :: {parsed_rresp}",
                        parsed_rresp = redis_value_to_string(&rresp)
                    )
                }
                Err(err) => format!("ERROR LTRIM :: {err:?}"),
            },
            (Err(err1), Err(err2)) => {
                format!("ERROR LTRIM (1) :: {err1:?}\nERROR LTRIM (2) :: {err2:?}")
            }
            (_, Err(err)) | (Err(err), _) => format!("ERROR LTRIM :: {err:?}"),
        }
    }

    pub fn lrange(conn: &mut redis::Connection, st: &mut RedisListState) -> String {
        // En caso del parseo fallar por la razón que sea devolvemos nada.
        let (start, stop) = (
            st.lrange_start.parse::<isize>().unwrap_or(0),
            st.lrange_stop.parse::<isize>().unwrap_or(0),
        );

        match conn.lrange(&st.lindex_k, start, stop) {
            Ok(rresp) => {
                // Lo más fácil tras éxito es recuperar los valores presentes en redis de nuevo
                let parsed_rresp = redis_value_to_string(&rresp);
                format!("LLEN :: {parsed_rresp}")
            }
            Err(err) => {
                format!("ERROR LLEN :: {err:?}")
            }
        }
    }

    pub fn lindex(conn: &mut redis::Connection, st: &mut RedisListState) -> String {
        // En caso del parseo fallar por la razón que sea devolvemos nada.
        let idx = st.lindex_idx.parse::<isize>().unwrap_or(0);

        match conn.lindex(&st.lindex_k, idx) {
            Ok(rresp) => {
                // Lo más fácil tras éxito es recuperar los valores presentes en redis de nuevo
                let parsed_rresp = redis_value_to_string(&rresp);
                format!("LLEN :: {parsed_rresp}")
            }
            Err(err) => {
                format!("ERROR LLEN :: {err:?}")
            }
        }
    }

    pub fn llen(conn: &mut redis::Connection, st: &mut RedisListState) -> String {
        match conn.llen(&st.llen_k) {
            Ok(rresp) => {
                // Lo más fácil tras éxito es recuperar los valores presentes en redis de nuevo
                let parsed_rresp = redis_value_to_string(&rresp);
                format!("LLEN :: {parsed_rresp}")
            }
            Err(err) => {
                format!("ERROR LLEN :: {err:?}")
            }
        }
    }

    pub fn rpop(
        conn: &mut redis::Connection,
        hm: &mut HashMap<String, Vec<String>>,
        st: &mut RedisListState,
    ) -> String {
        let count = st.rpop_count.parse::<usize>().ok().and_then(|v| {
            if v > 0 {
                NonZeroUsize::new(v)
            } else {
                None
            }
        });
        let (k, v) = (st.rpop_k.clone(), count);
        let result = conn.rpop(&k, v);

        match result {
            Ok(rresp) => {
                // Lo más fácil tras éxito es recuperar los valores presentes en redis de nuevo
                let value: Vec<String> = conn.lrange(&k, 0, isize::MAX).unwrap();
                hm.insert(k, value);
                let parsed_rresp = redis_value_to_string(&rresp);
                format!("RPOP :: {parsed_rresp}")
            }
            Err(err) => {
                format!("ERROR RPOP :: {err:?}")
            }
        }
    }

    pub fn lpop(
        conn: &mut redis::Connection,
        hm: &mut HashMap<String, Vec<String>>,
        st: &mut RedisListState,
    ) -> String {
        let count = st.lpop_count.parse::<usize>().ok().and_then(|v| {
            if v > 0 {
                NonZeroUsize::new(v)
            } else {
                None
            }
        });
        let (k, v) = (st.lpop_k.clone(), count);

        match conn.lpop(&k, v) {
            Ok(rresp) => {
                // Lo más fácil tras éxito es recuperar los valores presentes en redis de nuevo
                let value: Vec<String> = conn.lrange(&k, 0, isize::MAX).unwrap();
                hm.insert(k, value);
                let parsed_rresp = redis_value_to_string(&rresp);
                format!("LPOP :: {parsed_rresp}")
            }
            Err(err) => {
                format!("ERROR LPOP :: {err:?}")
            }
        }
    }

    pub fn rpush(
        conn: &mut redis::Connection,
        hm: &mut HashMap<String, Vec<String>>,
        st: &mut RedisListState,
    ) -> String {
        let vs = st
            .rpush_vs
            .split(' ')
            .map(|s| s.to_owned())
            .collect::<Vec<String>>();
        let (k, v) = (st.rpush_k.clone(), vs);

        match conn.rpush(&k, v) {
            Ok(rresp) => {
                // Lo más fácil tras éxito es recuperar los valores presentes en redis de nuevo
                let value: Vec<String> = conn.lrange(&k, 0, isize::MAX).unwrap();
                hm.insert(k, value);
                let parsed_rresp = redis_value_to_string(&rresp);
                format!("RPUSH :: {parsed_rresp}")
            }
            Err(err) => {
                format!("ERROR RPUSH :: {err:?}")
            }
        }
    }

    pub fn lpush(
        conn: &mut redis::Connection,
        hm: &mut HashMap<String, Vec<String>>,
        st: &mut RedisListState,
    ) -> String {
        let vs = st
            .lpush_vs
            .split(' ')
            .map(|s| s.to_owned())
            .collect::<Vec<String>>();
        let (k, v) = (st.lpush_k.clone(), vs);

        match conn.lpush(&k, v) {
            Ok(rresp) => {
                // Lo más fácil tras éxito es recuperar los valores presentes en redis de nuevo
                let value: Vec<String> = conn.lrange(&k, 0, isize::MAX).unwrap();
                hm.insert(k, value);
                let parsed_rresp = redis_value_to_string(&rresp);
                format!("LPUSH :: {parsed_rresp}")
            }
            Err(err) => {
                format!("ERROR LPUSH :: {err:?}")
            }
        }
    }

    pub fn rpushx(
        conn: &mut redis::Connection,
        hm: &mut HashMap<String, Vec<String>>,
        st: &mut RedisListState,
    ) -> String {
        let vs = st
            .rpush_vs
            .split(' ')
            .map(|s| s.to_owned())
            .collect::<Vec<String>>();
        let (k, v) = (st.rpush_k.clone(), vs);

        match conn.rpush_exists(&k, v) {
            Ok(rresp) => {
                // Lo más fácil tras éxito es recuperar los valores presentes en redis de nuevo
                match rresp {
                    Value::Int(n_elements) => {
                        if n_elements > 0 {
                            let value: Vec<String> = conn.lrange(&k, 0, isize::MAX).unwrap();
                            hm.insert(k, value);
                        }
                        let parsed_rresp = redis_value_to_string(&rresp);
                        format!("RPUSHX :: {parsed_rresp}")
                    }
                    // Rama no alcanzable, lpushx devuelve n elementos insertados.
                    _ => todo!(),
                }
            }
            Err(err) => {
                format!("ERROR RPUSHX :: {err:?}")
            }
        }
    }

    pub fn lpushx(
        conn: &mut redis::Connection,
        hm: &mut HashMap<String, Vec<String>>,
        st: &mut RedisListState,
    ) -> String {
        let vs = st
            .lpush_vs
            .split(' ')
            .map(|s| s.to_owned())
            .collect::<Vec<String>>();
        let (k, v) = (st.lpush_k.clone(), vs);

        match conn.lpush_exists(&k, v) {
            Ok(rresp) => {
                // Lo más fácil tras éxito es recuperar los valores presentes en redis de nuevo
                match rresp {
                    Value::Int(n_elements) => {
                        if n_elements > 0 {
                            let value: Vec<String> = conn.lrange(&k, 0, isize::MAX).unwrap();
                            hm.insert(k, value);
                        }
                        let parsed_rresp = redis_value_to_string(&rresp);
                        format!("LPUSHX :: {parsed_rresp}")
                    }
                    // Rama no alcanzable, lpushx devuelve n elementos insertados.
                    _ => todo!(),
                }
            }
            Err(err) => {
                format!("ERROR LPUSHX :: {err:?}")
            }
        }
    }
}

pub struct SetsPresenter;

impl SetsPresenter {
    pub fn sadd(
        conn: &mut redis::Connection,
        hm: &mut HashMap<String, Vec<String>>,
        st: &mut RedisSetsState,
    ) -> String {
        let vs = st.sadd_vs.split(' ').collect::<Vec<&str>>();
        let k = st.sadd_k.clone();

        match conn.sadd(&k, vs) {
            Ok(rresp) => {
                let value: Vec<String> = conn.smembers(&k).unwrap();
                hm.insert(k, value);
                let parsed_rresp = redis_value_to_string(&rresp);
                format!("SADD :: {parsed_rresp}")
            }
            Err(err) => {
                format!("ERROR SADD :: {err:?}")
            }
        }
    }

    pub fn srem(
        conn: &mut redis::Connection,
        hm: &mut HashMap<String, Vec<String>>,
        st: &mut RedisSetsState,
    ) -> String {
        let vs = st.srem_vs.split(' ').collect::<Vec<&str>>();
        let k = st.srem_k.clone();

        match conn.srem(&k, vs) {
            Ok(rresp) => {
                let value: Vec<String> = conn.smembers(&k).unwrap();
                hm.insert(k, value);
                let parsed_rresp = redis_value_to_string(&rresp);
                format!("SREM :: {parsed_rresp}")
            }
            Err(err) => {
                format!("ERROR SREM :: {err:?}")
            }
        }
    }

    pub fn spop(
        conn: &mut redis::Connection,
        hm: &mut HashMap<String, Vec<String>>,
        st: &mut RedisSetsState,
    ) -> String {
        let k = st.spop_k.clone();

        match conn.spop(&k) {
            Ok(rresp) => {
                let value: Vec<String> = conn.smembers(&k).unwrap();
                hm.insert(k, value);
                let parsed_rresp = redis_value_to_string(&rresp);
                format!("SPOP :: {parsed_rresp}")
            }
            Err(err) => {
                format!("ERROR SPOP :: {err:?}")
            }
        }
    }

    pub fn srandmember(conn: &mut redis::Connection, st: &mut RedisSetsState) -> String {
        let k = st.srandmember_k.clone();
        let count = st.srandmember_count.parse::<usize>().unwrap_or(1);
        let value = if count <= 1 {
            conn.srandmember_multiple(&k, 1)
        } else {
            conn.srandmember_multiple(&k, count)
        };

        match value {
            Ok(rresp) => {
                let parsed_rresp = redis_value_to_string(&rresp);
                format!("SRANDMEMBER :: {parsed_rresp}")
            }
            Err(err) => {
                format!("ERROR SRANDMEMBER :: {err:?}")
            }
        }
    }

    pub fn sismember(conn: &mut redis::Connection, st: &mut RedisSetsState) -> String {
        match conn.sismember(&st.sismember_k, &st.sismember_m) {
            Ok(rresp) => {
                let parsed_rresp = redis_value_to_string(&rresp);
                format!("SISMEMBER :: {parsed_rresp}")
            }
            Err(err) => {
                format!("ERROR SISMEMBER :: {err:?}")
            }
        }
    }

    pub fn smismember(conn: &mut redis::Connection, st: &mut RedisSetsState) -> String {
        let vs = st.smismember_ms.split(' ').collect::<Vec<&str>>();

        match conn.smismember(&st.smismember_k, &vs) {
            Ok(rresp) => {
                let parsed_rresp = redis_value_to_string(&rresp);
                format!("SMISMEMBER :: {parsed_rresp}")
            }
            Err(err) => {
                format!("ERROR SMISMEMBER :: {err:?}")
            }
        }
    }

    pub fn scard(conn: &mut redis::Connection, st: &mut RedisSetsState) -> String {
        match conn.scard(&st.scard_k) {
            Ok(rresp) => {
                let parsed_rresp = redis_value_to_string(&rresp);
                format!("SCARD :: {parsed_rresp}")
            }
            Err(err) => {
                format!("ERROR SCARD :: {err:?}")
            }
        }
    }

    pub fn smembers(
        conn: &mut redis::Connection,
        hm: &mut HashMap<String, Vec<String>>,
        st: &mut RedisSetsState,
    ) -> String {
        let k = st.smembers_k.clone();

        redis_smembers(conn, k, hm)
    }

    pub fn sinter(conn: &mut redis::Connection, st: &mut RedisSetsState) -> String {
        let ks = st.sinter_ks.split(' ').collect::<Vec<&str>>();

        match conn.sinter::<Vec<&str>, Vec<String>>(ks) {
            Ok(rresp) => {
                format!("SINTER :: {parsed_rresp}", parsed_rresp = rresp.join(", "))
            }
            Err(err) => {
                format!("ERROR SINTER :: {err:?}")
            }
        }
    }

    pub fn sintercard(conn: &mut redis::Connection, st: &mut RedisSetsState) -> String {
        let ks = st.sintercard_ks.split(' ').collect::<Vec<&str>>();
        let result = redis::cmd("SINTERCARD")
            .arg(&st.sintercard_numkeys)
            .arg(ks)
            .query::<Value>(conn);

        match result {
            Ok(rresp) => {
                let parsed_rresp = redis_value_to_string(&rresp);
                format!("SINTERCARD :: {parsed_rresp}")
            }
            Err(err) => {
                format!("ERROR SINTERCARD :: {err:?}")
            }
        }
    }

    pub fn sinterstore(
        conn: &mut redis::Connection,
        hm: &mut HashMap<String, Vec<String>>,
        st: &mut RedisSetsState,
    ) -> String {
        let ks = st
            .sinterstore_ks
            .split(' ')
            .map(|s| s.to_owned())
            .collect::<Vec<String>>();
        let set_dst = st.sinterstore_destination.clone();

        match conn.sinterstore::<&String, Vec<String>, redis::Value>(&set_dst, ks) {
            Ok(rresp) => {
                let response = format!("SINTERSTORE :: {rresp:?}");
                let _ = redis_smembers(conn, set_dst, hm);

                response
            }
            Err(err) => {
                format!("ERROR SINTERSTORE :: {err:?}")
            }
        }
    }

    pub fn sdiff(conn: &mut redis::Connection, st: &mut RedisSetsState) -> String {
        let ks = st.sdiff_ks.split(' ').collect::<Vec<&str>>();

        match conn.sdiff::<Vec<&str>, Vec<String>>(ks) {
            Ok(rresp) => {
                format!("SDIFF :: {parsed_rresp}", parsed_rresp = rresp.join(", "))
            }
            Err(err) => {
                format!("ERROR SDIFF :: {err:?}")
            }
        }
    }

    pub fn sdiffstore(
        conn: &mut redis::Connection,
        hm: &mut HashMap<String, Vec<String>>,
        st: &mut RedisSetsState,
    ) -> String {
        let ks = st
            .sdiffstore_ks
            .split(' ')
            .map(|s| s.to_owned())
            .collect::<Vec<String>>();
        let set_dst = st.sdiffstore_destination.clone();

        match conn.sdiffstore::<&String, Vec<String>, redis::Value>(&set_dst, ks) {
            Ok(rresp) => {
                let response = format!("SDIFFSTORE :: {rresp:?}");
                let _ = redis_smembers(conn, set_dst, hm);

                response
            }
            Err(err) => {
                format!("ERROR SDIFFSTORE :: {err:?}")
            }
        }
    }

    pub fn sunion(conn: &mut redis::Connection, st: &mut RedisSetsState) -> String {
        let ks = st.sunion_ks.split(' ').collect::<Vec<&str>>();

        match conn.sunion::<Vec<&str>, Vec<String>>(ks) {
            Ok(rresp) => {
                format!("SUNION :: {parsed_rresp}", parsed_rresp = rresp.join(", "))
            }
            Err(err) => {
                format!("ERROR SUNION :: {err:?}")
            }
        }
    }

    pub fn sunionstore(
        conn: &mut redis::Connection,
        hm: &mut HashMap<String, Vec<String>>,
        st: &mut RedisSetsState,
    ) -> String {
        let ks = st
            .sunionstore_ks
            .split(' ')
            .map(|s| s.to_owned())
            .collect::<Vec<String>>();
        let set_dst = st.sunionstore_destination.clone();

        match conn.sunionstore::<&String, Vec<String>, redis::Value>(&set_dst, ks) {
            Ok(rresp) => {
                let response = format!("SUNIONSTORE :: {rresp:?}");
                let _ = redis_smembers(conn, set_dst, hm);

                response
            }
            Err(err) => {
                format!("ERROR SUNIONSTORE :: {err:?}")
            }
        }
    }
}

fn redis_smembers(
    conn: &mut Connection,
    k: String,
    hm: &mut HashMap<String, Vec<String>>,
) -> String {
    match conn.smembers::<&str, Vec<String>>(&k) {
        Ok(rresp) => {
            let resp = format!(
                "SMEMBERS :: {parsed_rresp}",
                parsed_rresp = rresp.join(", ")
            );
            hm.insert(k, rresp);

            resp
        }
        Err(err) => {
            format!("ERROR SMEMBERS :: {err:?}")
        }
    }
}

pub struct SortedSetsPresenter;

impl SortedSetsPresenter {
    pub fn zadd(
        conn: &mut redis::Connection,
        hm: &mut HashMap<String, Vec<String>>,
        st: &mut RedisSortedSetsState,
    ) -> String {
        let v = st.zadd_v.clone();
        let k = st.zadd_k.clone();
        let s = st.zadd_score.parse::<f64>().unwrap_or(0.0);

        match conn.zadd(&k, &v, s) {
            Ok(rresp) => {
                let value: Vec<String> = conn.zrange(&k, 0, -1).unwrap();
                hm.insert(k, value);
                let parsed_rresp = redis_value_to_string(&rresp);
                format!("ZADD :: {parsed_rresp}")
            }
            Err(err) => {
                format!("ERROR ZADD :: {err:?}")
            }
        }
    }

    pub fn zrem(
        conn: &mut redis::Connection,
        hm: &mut HashMap<String, Vec<String>>,
        st: &mut RedisSortedSetsState,
    ) -> String {
        let vs = st.zrem_vs.split(' ').collect::<Vec<&str>>();
        let k = st.zrem_k.clone();

        match conn.zrem(&k, vs) {
            Ok(rresp) => {
                let value: Vec<String> = conn.zrange(&k, 0, -1).unwrap();
                hm.insert(k, value);
                let parsed_rresp = redis_value_to_string(&rresp);
                format!("ZREM :: {parsed_rresp}")
            }
            Err(err) => {
                format!("ERROR ZREM :: {err:?}")
            }
        }
    }

    pub fn zmpop(
        conn: &mut redis::Connection,
        hm: &mut HashMap<String, Vec<String>>,
        st: &mut RedisSortedSetsState,
    ) -> String {
        let ks = st.zmpop_ks.split(' ').collect::<Vec<&str>>();
        let count = st.zmpop_count.parse::<isize>().unwrap_or_default();

        let result = match st.zmpop_min_max.as_ref() {
            "MIN" => conn.zmpop_min(&ks, count),
            "MAX" => conn.zmpop_min(&ks, count),
            _ => conn.zmpop_min(&ks, count),
        };

        match result {
            Ok(rresp) => {
                // TODO: Podría llamar a scan pero esto es menos costoso, aunque
                // podemos bloquear si es salvaje la cantidad de info en `sorted_sets`.
                // Implementar `zscan` basada en ZSCAN.
                for k in ks {
                    let value: Vec<String> = conn.zrange(k, 0, -1).unwrap();
                    hm.insert(k.to_string(), value);
                }
                let parsed_rresp = redis_value_to_string(&rresp);
                format!("ZMPOP :: {parsed_rresp}")
            }
            Err(err) => {
                format!("ERROR ZMPOP :: {err:?}")
            }
        }
    }

    pub fn zrandmember(conn: &mut redis::Connection, st: &mut RedisSortedSetsState) -> String {
        let k = st.zrandmember_k.clone();
        let count = st.zrandmember_count.parse::<isize>().unwrap_or(1);
        let value = if count <= 1 {
            conn.zrandmember(&k, None)
        } else {
            conn.zrandmember(&k, Some(count))
        };

        match value {
            Ok(rresp) => {
                let parsed_rresp = redis_value_to_string(&rresp);
                format!("ZRANDMEMBER :: {parsed_rresp}")
            }
            Err(err) => {
                format!("ERROR ZRANDMEMBER :: {err:?}")
            }
        }
    }

    pub fn zcard(conn: &mut redis::Connection, st: &mut RedisSortedSetsState) -> String {
        match conn.zcard(&st.zcard_k) {
            Ok(rresp) => {
                let parsed_rresp = redis_value_to_string(&rresp);
                format!("ZCARD :: {parsed_rresp}")
            }
            Err(err) => {
                format!("ERROR ZCARD :: {err:?}")
            }
        }
    }

    pub fn zrange(conn: &mut redis::Connection, st: &mut RedisSortedSetsState) -> String {
        let (start, stop) = (
            st.zrange_start.parse::<isize>(),
            st.zrange_stop.parse::<isize>(),
        );

        match (start, stop) {
            (Ok(b), Ok(e)) => match conn.zrange(&st.zrange_k, b, e) {
                Ok(rresp) => {
                    format!(
                        "ZRANGE :: {parsed_rresp}",
                        parsed_rresp = redis_value_to_string(&rresp)
                    )
                }
                Err(err) => format!("ERROR ZRANGE :: {err:?}"),
            },
            (Err(err1), Err(err2)) => {
                format!("ERROR ZRANGE (1) :: {err1:?}\nERROR ZRANGE (2) :: {err2:?}")
            }
            (_, Err(err)) | (Err(err), _) => format!("ERROR ZRANGE :: {err:?}"),
        }
    }

    pub fn zrangestore(
        conn: &mut redis::Connection,
        hm: &mut HashMap<String, Vec<String>>,
        st: &mut RedisSortedSetsState,
    ) -> String {
        let (start, stop) = (
            st.zrange_start.parse::<isize>(),
            st.zrange_stop.parse::<isize>(),
        );

        match (start, stop) {
            (Ok(b), Ok(e)) => {
                let result = redis::cmd("ZRANGESTORE")
                    .arg(&st.zrangestore_destination)
                    .arg(&st.zrange_k)
                    .arg(b)
                    .arg(e)
                    .query::<Value>(conn);

                match result {
                    Ok(rresp) => {
                        let value: Vec<String> =
                            conn.zrange(&st.zrangestore_destination, 0, -1).unwrap();
                        hm.insert(st.zrangestore_destination.clone(), value);
                        format!(
                            "ZRANGESTORE :: {parsed_rresp}",
                            parsed_rresp = redis_value_to_string(&rresp)
                        )
                    }
                    Err(err) => format!("ERROR ZRANGESTORE :: {err:?}"),
                }
            }
            (Err(err1), Err(err2)) => {
                format!("ERROR ZRANGESTORE (1) :: {err1:?}\nERROR ZRANGESTORE (2) :: {err2:?}")
            }
            (_, Err(err)) | (Err(err), _) => format!("ERROR ZRANGESTORE :: {err:?}"),
        }
    }

    pub fn zrangebylex(conn: &mut redis::Connection, st: &mut RedisSortedSetsState) -> String {
        let (min, max) = (
            st.zrangebylex_min.parse::<isize>(),
            st.zrangebylex_max.parse::<isize>(),
        );
        let k = st.zrange_k.clone();

        match (min, max) {
            (Ok(_min), Ok(_max)) => match conn.zrangebylex(k, _min, _max) {
                Ok(rresp) => {
                    format!(
                        "ZRANGEBYLEX :: {parsed_rresp}",
                        parsed_rresp = redis_value_to_string(&rresp)
                    )
                }
                Err(err) => format!("ERROR ZRANGEBYLEX :: {err:?}"),
            },
            (Err(err1), Err(err2)) => {
                format!("ERROR ZRANGEBYLEX (1) :: {err1:?}\nERROR ZRANGEBYLEX (2) :: {err2:?}")
            }
            (_, Err(err)) | (Err(err), _) => format!("ERROR ZRANGEBYLEX :: {err:?}"),
        }
    }

    pub fn zrangebyscore(conn: &mut redis::Connection, st: &mut RedisSortedSetsState) -> String {
        let (min, max) = (
            st.zrangebyscore_min.parse::<isize>(),
            st.zrangebyscore_max.parse::<isize>(),
        );
        let k = st.zrange_k.clone();

        match (min, max) {
            (Ok(_min), Ok(_max)) => match conn.zrangebyscore(k, _min, _max) {
                Ok(rresp) => {
                    format!(
                        "ZRANGEBYSCORE :: {parsed_rresp}",
                        parsed_rresp = redis_value_to_string(&rresp)
                    )
                }
                Err(err) => format!("ERROR ZRANGEBYSCORE :: {err:?}"),
            },
            (Err(err1), Err(err2)) => {
                format!("ERROR ZRANGEBYSCORE (1) :: {err1:?}\nERROR ZRANGEBYSCORE (2) :: {err2:?}")
            }
            (_, Err(err)) | (Err(err), _) => format!("ERROR ZRANGEBYSCORE :: {err:?}"),
        }
    }

    pub fn zinter(conn: &mut redis::Connection, st: &mut RedisSortedSetsState) -> String {
        let ks = st.zinter_ks.split(' ').collect::<Vec<&str>>();
        let result = redis::cmd("ZINTER")
            .arg(ks.len())
            .arg(ks)
            .query::<Value>(conn);

        match result {
            Ok(rresp) => {
                let parsed_rresp = redis_value_to_string(&rresp);
                format!("ZINTER :: {parsed_rresp}")
            }
            Err(err) => {
                format!("ERROR ZINTER :: {err:?}")
            }
        }
    }

    pub fn zintercard(conn: &mut redis::Connection, st: &mut RedisSortedSetsState) -> String {
        let ks = st.zintercard_ks.split(' ').collect::<Vec<&str>>();
        let result = redis::cmd("ZINTECARD")
            .arg(ks.len())
            .arg(ks)
            .query::<Value>(conn);

        match result {
            Ok(rresp) => {
                let parsed_rresp = redis_value_to_string(&rresp);
                format!("ZINTERCARD :: {parsed_rresp}")
            }
            Err(err) => {
                format!("ERROR ZINTERCARD :: {err:?}")
            }
        }
    }

    pub fn zinterstore(
        conn: &mut redis::Connection,
        hm: &mut HashMap<String, Vec<String>>,
        st: &mut RedisSortedSetsState,
    ) -> String {
        let ks = st.zintercard_ks.split(' ').collect::<Vec<&str>>();

        match conn.zinterstore(&st.zinterstore_destination, &ks) {
            Ok(rresp) => {
                let parsed_rresp = redis_value_to_string(&rresp);
                let value: Vec<String> = conn.zrange(&st.zinterstore_destination, 0, -1).unwrap();
                hm.insert(st.zinterstore_destination.clone(), value);
                format!("ZINTERCARD :: {parsed_rresp}")
            }
            Err(err) => {
                format!("ERROR ZINTERCARD :: {err:?}")
            }
        }
    }

    // pub fn sintercard(conn: &mut redis::Connection, st: &mut RedisSortedSetsState) -> String {
    //     let ks = st.sintercard_ks.split(' ').collect::<Vec<&str>>();
    //     let result = redis::cmd("SINTERCARD")
    //         .arg(&st.sintercard_numkeys)
    //         .arg(ks)
    //         .query::<Value>(conn);

    //     match result {
    //         Ok(rresp) => {
    //             let parsed_rresp = redis_value_to_string(&rresp);
    //             format!("SINTERCARD :: {parsed_rresp}")
    //         }
    //         Err(err) => {
    //             format!("ERROR SINTERCARD :: {err:?}")
    //         }
    //     }
    // }

    // pub fn sinterstore(
    //     conn: &mut redis::Connection,
    //     hm: &mut HashMap<String, Vec<String>>,
    //     st: &mut RedisSortedSetsState,
    // ) -> String {
    //     let ks = st
    //         .sinterstore_ks
    //         .split(' ')
    //         .map(|s| s.to_owned())
    //         .collect::<Vec<String>>();
    //     let set_dst = st.sinterstore_destination.clone();

    //     match conn.sinterstore::<&String, Vec<String>, redis::Value>(&set_dst, ks) {
    //         Ok(rresp) => {
    //             let response = format!("SINTERSTORE :: {rresp:?}");
    //             let _ = redis_smembers(conn, set_dst, hm);

    //             response
    //         }
    //         Err(err) => {
    //             format!("ERROR SINTERSTORE :: {err:?}")
    //         }
    //     }
    // }

    // pub fn sdiff(conn: &mut redis::Connection, st: &mut RedisSortedSetsState) -> String {
    //     let ks = st.sdiff_ks.split(' ').collect::<Vec<&str>>();

    //     match conn.sdiff::<Vec<&str>, Vec<String>>(ks) {
    //         Ok(rresp) => {
    //             format!("SDIFF :: {parsed_rresp}", parsed_rresp = rresp.join(", "))
    //         }
    //         Err(err) => {
    //             format!("ERROR SDIFF :: {err:?}")
    //         }
    //     }
    // }

    // pub fn sdiffstore(
    //     conn: &mut redis::Connection,
    //     hm: &mut HashMap<String, Vec<String>>,
    //     st: &mut RedisSortedSetsState,
    // ) -> String {
    //     let ks = st
    //         .sdiffstore_ks
    //         .split(' ')
    //         .map(|s| s.to_owned())
    //         .collect::<Vec<String>>();
    //     let set_dst = st.sdiffstore_destination.clone();

    //     match conn.sdiffstore::<&String, Vec<String>, redis::Value>(&set_dst, ks) {
    //         Ok(rresp) => {
    //             let response = format!("SDIFFSTORE :: {rresp:?}");
    //             let _ = redis_smembers(conn, set_dst, hm);

    //             response
    //         }
    //         Err(err) => {
    //             format!("ERROR SDIFFSTORE :: {err:?}")
    //         }
    //     }
    // }

    // pub fn sunion(conn: &mut redis::Connection, st: &mut RedisSortedSetsState) -> String {
    //     let ks = st.sunion_ks.split(' ').collect::<Vec<&str>>();

    //     match conn.sunion::<Vec<&str>, Vec<String>>(ks) {
    //         Ok(rresp) => {
    //             format!("SUNION :: {parsed_rresp}", parsed_rresp = rresp.join(", "))
    //         }
    //         Err(err) => {
    //             format!("ERROR SUNION :: {err:?}")
    //         }
    //     }
    // }

    // pub fn sunionstore(
    //     conn: &mut redis::Connection,
    //     hm: &mut HashMap<String, Vec<String>>,
    //     st: &mut RedisSortedSetsState,
    // ) -> String {
    //     let ks = st
    //         .sunionstore_ks
    //         .split(' ')
    //         .map(|s| s.to_owned())
    //         .collect::<Vec<String>>();
    //     let set_dst = st.sunionstore_destination.clone();

    //     match conn.sunionstore::<&String, Vec<String>, redis::Value>(&set_dst, ks) {
    //         Ok(rresp) => {
    //             let response = format!("SUNIONSTORE :: {rresp:?}");
    //             let _ = redis_smembers(conn, set_dst, hm);

    //             response
    //         }
    //         Err(err) => {
    //             format!("ERROR SUNIONSTORE :: {err:?}")
    //         }
    //     }
    // }
}

fn redis_zmembers(
    conn: &mut Connection,
    k: String,
    hm: &mut HashMap<String, Vec<String>>,
) -> String {
    match conn.smembers::<&str, Vec<String>>(&k) {
        Ok(rresp) => {
            let resp = format!(
                "SMEMBERS :: {parsed_rresp}",
                parsed_rresp = rresp.join(", ")
            );
            hm.insert(k, rresp);

            resp
        }
        Err(err) => {
            format!("ERROR SMEMBERS :: {err:?}")
        }
    }
}
