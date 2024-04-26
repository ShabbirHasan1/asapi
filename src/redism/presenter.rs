// -------------------------------------------------------------------------
// Copyright (C) 2023 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use crate::{error, info};
use redis::streams::{StreamReadOptions, StreamReadReply};
use redis::JsonCommands;
use redis::{self, Client, Commands, Connection, Msg as PubSubMsg, RedisError, RedisResult, Value};
use std::collections::HashMap;

use super::presenters::RedisResponse;
use super::state::RedisLocalState;

#[derive(Clone, Debug, PartialEq, Copy, Default)]
pub enum RedisMenu {
    All,
    String,
    List,
    Set,
    Hash,
    SortedSet,
    Json,
    #[default]
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

/// Escaneo de toda la instancia de redis.
///
/// Pasamos de forma explícita la opción elegida aunque ya hay una en el estado
/// para permitir recarga bajo demanda en menú contextual de cada estructura de datos.
pub fn scan(state: &mut RedisLocalState, option: RedisMenu) -> RedisResult<()> {
    let mut con: Connection = create_conn_with_default(
        &state.current_connection.host,
        &state.current_connection.port,
    )?;
    let mut cursor: u64 = 0;
    state.reset(option);

    loop {
        let scan_result: (u64, Vec<String>) = redis::cmd("SCAN").arg(cursor).query(&mut con)?;

        cursor = scan_result.0;

        for key in scan_result.1 {
            let key_type = redis::Cmd::key_type(&key).query::<String>(&mut con);

            match key_type {
                Ok(value) => match value.as_str() {
                    "string" => {
                        if option == RedisMenu::String || option == RedisMenu::All {
                            let value = con.get(&key).unwrap();
                            state.strings.insert(key, value);
                        }
                    }
                    "ReJSON-RL" => {
                        if option == RedisMenu::Json || option == RedisMenu::All {
                            let value = con.json_get(&key, "$").unwrap();
                            state.jsons.insert(key, value);
                        }
                    }
                    "list" => {
                        if option == RedisMenu::List || option == RedisMenu::All {
                            let value = con.lrange(&key, 0, isize::MAX).unwrap();
                            state.lists.insert(key, value);
                        }
                    }
                    "set" => {
                        if option == RedisMenu::Set || option == RedisMenu::All {
                            let value = con.smembers(&key).unwrap();
                            state.sets.insert(key, value);
                        }
                    }
                    "zset" => {
                        if option == RedisMenu::SortedSet || option == RedisMenu::All {
                            let value = con.zrange(&key, 0, -1).unwrap();
                            state.sorted_sets.insert(key, value);
                        }
                    }
                    "hash" => {
                        if option == RedisMenu::Hash || option == RedisMenu::All {
                            let result = con.hgetall(&key).unwrap();
                            state.hashes.insert(key, result);
                        }
                    }
                    "stream" => {
                        if option == RedisMenu::Stream || option == RedisMenu::All {
                            let opts = StreamReadOptions::default(); //.count(10);
                            let result: RedisResult<StreamReadReply> =
                                con.xread_options(&[&key], &["0"], &opts);
                            state.streams.insert(key.clone(), Vec::new());

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

// pub fn run_redis_command<F: FnMut(&mut redis::Connection) -> String>(
//     conn_def: &RedisConnectionDefinition,
//     mut cb: F,
// ) -> String {
//     let connection = create_redis_connection(conn_def);

//     if let Ok(mut conn) = connection {
//         cb(&mut conn)
//     } else {
//         "ERROR :: Not able to connect to {conn}.".to_string()
//     }
// }

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
                    info!(" - entry id: {}", stream_id.id);
                    state.insert(stream_id.id, stream_id.map.clone());
                    info!("{:?}", stream_id.map);
                }
            }
        }
        Err(e) => info!("Ocurrió un error {}", e),
    }

    Ok(())
}

pub fn run_user_string_command(host: &str, port: &str, cmd: &str) -> RedisResponse {
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
