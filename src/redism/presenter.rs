// -------------------------------------------------------------------------
// Copyright (C) 2023 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use redis::streams::{StreamReadOptions, StreamReadReply};
use redis::{self, Client, Commands, Connection, Msg as PubSubMsg, RedisError, RedisResult, Value};
use redis::{JsonCommands, ToRedisArgs};
use std::collections::HashMap;
use std::convert::Infallible;
use std::num::ParseIntError;
use std::str::FromStr;
use std::string::ParseError;

use crate::{error, info};

use super::state::RedisLocalState;

// #[derive(Debug)]
// pub enum Command {
//     Get { key: String },
//     Set { key: String, val: Bytes },
// }

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

// pub async fn create_async_conn() -> Result<aio::Connection, RedisError> {
//     let client = redis::Client::open("redis://127.0.0.1/")?;
//     client.get_async_connection().await
// }

// pub fn set_and_get(
//     rt: &tokio::runtime::Runtime,
//     _cmd: Command,
//     tx: Sender<Command>,
//     // ctx: egui::Context
// ) -> Result<(), RedisError> {
//     rt.spawn(async move {
//         let mut conn = create_async_conn().await?;
//         conn.set("my_key", "Hello world!").await?;
//         let _result = conn.get("my_key").await?;

//         let cmd2 = Command::Get {
//             key: "foo".to_string(),
//         };
//         let _ = tx.send(cmd2);
//         info!("-->> bar1");
//         Ok::<(), RedisError>(())
//         // ctx.request_repaint();
//     });

//     Ok(())
// }

// pub async fn set_value() -> Result<(), RedisError> {
//     let mut con = create_async_conn().await?;
//     let _ = con.set("my_key", "Hello world!").await?;
//     let result: String = con.get("my_key").await?;

//     info!("->> my_key: {}\n", result);

//     Ok(())
// }

#[derive(Clone, Debug, PartialEq, Copy)]
pub enum RedisMenu {
    All,
    String,
    Json,
    List,
    Set,
    SortedSet,
    Hash,
    Stream,
    PubSub,
}

impl Default for RedisMenu {
    fn default() -> Self {
        RedisMenu::String
    }
}

pub fn create_conn(host: &str, port: i16) -> Result<redis::Connection, RedisError> {
    //if Redis server needs secure connection // https://medium.com/swlh/tutorial-getting-started-with-rust-and-redis-69041dd38279
    // let uri_scheme = match env::var("IS_TLS") {
    //     Ok(_) => "rediss",
    //     Err(_) => "redis",
    // };

    let client = Client::open(format!("redis://{}:{}", host, port))?;
    client.get_connection()
}

pub fn create_conn_with_default(host: &str, port: &str) -> Result<redis::Connection, RedisError> {
    let port = port.parse::<i16>().unwrap_or(6379); // Using 6379 as default value;
    create_conn(&host, port)
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
    create_conn_with_default(&host, &port).and_then(|mut con| con.del(key))
}

// Borrado por entrada, un hash entero no se puede borrar. Se borra cuando no le quedan entradas.
pub fn delete_hashkey(host: &str, port: &str, hash_name: &str, field_key: &str) -> RedisResult<i8> {
    create_conn_with_default(&host, &port).and_then(|mut con| con.hdel(hash_name, field_key))
}

pub fn publish_to_channel(
    host: &str,
    port: &str,
    channel: &str,
    message: &str,
) -> RedisResult<bool> {
    create_conn_with_default(&host, &port).and_then(|mut conn| conn.publish(channel, message))
}

pub fn subscribe_to_channel_std_thread(
    host: &str,
    port: &str,
    channel: &str,
    tx: &std::sync::mpsc::Sender<PubSubMsg>,
) -> Result<(), RedisError> {
    let mut conn = create_conn_with_default(&host, &port)?;
    let owned_channel = channel.to_owned();
    let tx_owned = tx.to_owned();

    std::thread::spawn(move || -> Result<(), RedisError> {
        let mut pubsub = conn.as_pubsub();
        let _ = pubsub.subscribe(&owned_channel).unwrap();

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
        let connection =
            create_conn_with_default(&st.current_connection.host, &st.current_connection.port);

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
        let connection =
            create_conn_with_default(&st.current_connection.host, &st.current_connection.port);

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
        let connection =
            create_conn_with_default(&st.current_connection.host, &st.current_connection.port);

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
        let connection =
            create_conn_with_default(&st.current_connection.host, &st.current_connection.port);

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
        let connection =
            create_conn_with_default(&st.current_connection.host, &st.current_connection.port);

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
        let connection =
            create_conn_with_default(&st.current_connection.host, &st.current_connection.port);

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
        let connection =
            create_conn_with_default(&st.current_connection.host, &st.current_connection.port);

        if let Ok(mut conn) = connection {
            let k = st.string_st.incr_k.clone();
            let response = match t {
                NumericValue::Int => v
                    .parse::<i64>()
                    .or_else(|err| Err(format!("PARSE ERROR :: {err:?}")))
                    .and_then(|i| {
                        conn.incr::<&str, i64, redis::Value>(&k, i)
                            .or_else(|err| Err(format!("{err:?}")))
                            .and_then(|value| Ok(redis_value_to_string(&value)))
                    }),
                NumericValue::Float => v
                    .parse::<f32>()
                    .or_else(|err| Err(format!("PARSE ERROR :: {err:?}")))
                    .and_then(|i| {
                        conn.incr::<&str, f32, redis::Value>(&k, i)
                            .or_else(|err| Err(format!("{err:?}")))
                            .and_then(|value| Ok(redis_value_to_string(&value)))
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
        let connection =
            create_conn_with_default(&st.current_connection.host, &st.current_connection.port);

        if let Ok(mut conn) = connection {
            let k = st.string_st.decr_k.clone();
            let response = v
                .parse::<i64>()
                .or_else(|err| Err(format!("{err:?}")))
                .and_then(|i| {
                    conn.decr::<&str, i64, redis::Value>(&k, i)
                        .or_else(|err| Err(format!("{err:?}")))
                        .and_then(|value| Ok(redis_value_to_string(&value)))
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
        let connection =
            create_conn_with_default(&st.current_connection.host, &st.current_connection.port);

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
        let connection =
            create_conn_with_default(&st.current_connection.host, &st.current_connection.port);

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
        let connection =
            create_conn_with_default(&st.current_connection.host, &st.current_connection.port);

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
        let connection =
            create_conn_with_default(&st.current_connection.host, &st.current_connection.port);

        if let Ok(mut conn) = connection {
            let (k, from_s, to_s) = (
                st.string_st.get_k.clone(),
                st.string_st.get_offset_from.clone(),
                st.string_st.get_offset_to.clone(),
            );

            match (from_s.parse::<isize>(), to_s.parse::<isize>()) {
                (Ok(f), Ok(t)) => {
                    let result = conn.getrange::<&str, String>(&k, f, t);
                    st.command_last_result = match result {
                        Ok(msg) => format!("GETRANGE :: Key: {k}, Value: {msg}"),
                        Err(err) => format!("ERROR GETRANGE :: {err:?}"),
                    }
                }
                _ => {}
            }
        }
    }

    pub fn get_ex(st: &mut RedisLocalState) {
        let connection =
            create_conn_with_default(&st.current_connection.host, &st.current_connection.port);

        if let Ok(mut conn) = connection {
            let (k, expire_at_s) = (
                st.string_st.get_k.clone(),
                st.string_st.get_expire_seconds.clone(),
            );

            match expire_at_s.parse::<usize>() {
                Ok(ex) => {
                    let result = conn.get_ex::<&str, String>(&k, redis::Expiry::EX(ex));
                    st.command_last_result = match result {
                        Ok(msg) => format!("GETEX :: Key: {k}, Value: {msg}"),
                        Err(err) => format!("ERROR GETEX :: {err:?}"),
                    }
                }
                _ => {}
            }
        }
    }

    pub fn _get(conn: &mut redis::Connection, k: &str) -> Result<String, RedisError> {
        let result = conn.get::<&str, String>(&k);
        result
    }
}

// fn redis_value_to_string(v: redis::Value) -> &'static str {
fn redis_value_to_string(v: &redis::Value) -> String {
    match v {
        Value::Nil => "Nil".to_string(),
        Value::Int(i) => i.to_string(),
        Value::Data(d) => String::from_utf8(d.clone())
            .unwrap_or_else(|err| format!("ERROR {err:?} parsing {d:?}")),
        Value::Bulk(b) => b
            .iter()
            .map(|v| redis_value_to_string(&v))
            .collect::<Vec<String>>()
            .join(", "),
        Value::Status(s) => s.clone(),
        Value::Okay => "OK".to_string(),
    }
}
