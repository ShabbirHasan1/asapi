// -------------------------------------------------------------------------
// Copyright (C) 2023 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use log::error;
use redis::streams::{StreamReadOptions, StreamReadReply};
use redis::JsonCommands;
use redis::{self, Client, Commands, Connection, RedisError, RedisResult, Value};

use super::presenters::RedisResponse;
use super::state::{RedisConnectionDefinition, RedisLocalState};

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

#[inline(always)]
pub fn create_redis_connection(
    conn: &RedisConnectionDefinition,
) -> Result<redis::Connection, RedisError> {
    create_conn_with_default(&conn.host, &conn.port)
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
                            state.zsets.insert(key, value);
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
                            state.streams.insert(key.clone(), vec![]);

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

pub fn run_user_string_command(host: &str, port: &str, cmd: &str) -> RedisResponse {
    match parse_command(cmd) {
        Some((command, args)) => match create_conn_with_default(host, port) {
            Ok(mut c) => {
                let response = redis::cmd(command).arg(&args).query::<Value>(&mut c);
                match response {
                    Ok(value) => Ok(format!("{:?}", value)),
                    Err(e) => Err(format!("Error parsing response: {:?}", e)),
                }
            }
            Err(e) => Err(format!("{}", e)),
        },
        None => Err(format!("Error parsing command {cmd}.")),
    }
}

fn parse_command<'a>(input: &'a str) -> Option<(&'a str, Vec<&'a str>)> {
    let mut words = input.split_whitespace().collect::<Vec<&'a str>>();

    if words.is_empty() {
        return None;
    }

    let initial = words.remove(0);

    Some((initial, words))
}
