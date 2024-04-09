// -------------------------------------------------------------------------
// Copyright (C) 2023 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use super::state::RedisLocalState;
use crate::app_state::AppState;
use crate::info;
use redis::streams::{StreamReadOptions, StreamReadReply};
use redis::{self, Client, Commands, Connection, Msg as PubSubMsg, RedisError, RedisResult, Value};
use std::collections::HashMap;
use std::slice::Iter;

#[derive(Clone, Debug, PartialEq, Copy)]
pub enum RedisMenu {
    All,
    String,
    Hash,
    Streams,
    PubSub,
}

impl RedisMenu {
    pub fn iterator() -> Iter<'static, RedisMenu> {
        static MENUS: [RedisMenu; 5] = [
            RedisMenu::All,
            RedisMenu::String,
            RedisMenu::Hash,
            RedisMenu::Streams,
            RedisMenu::PubSub,
        ];
        MENUS.iter()
    }
}

impl Default for RedisMenu {
    fn default() -> Self {
        RedisMenu::All
    }
}

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

pub fn scan(state: &mut RedisLocalState, app_state: &AppState) -> RedisResult<()> {
    // let client = Client::open("redis://127.0.0.1/")?;
    // let mut con: Connection = client.get_connection()?;
    // let port = app_state.redis.port.parse::<i16>().unwrap_or(6379); // Using 6379 as default value;
    // let mut con: Connection = create_conn(&app_state.redis.host, port)?;
    let mut con: Connection =
        create_conn_with_default(&app_state.redis.host, &app_state.redis.port)?;
    let mut cursor: u64 = 0;

    state.reset();

    loop {
        let scan_result: (u64, Vec<String>) = redis::cmd("SCAN").arg(cursor).query(&mut con)?;

        cursor = scan_result.0;

        for key in scan_result.1 {
            let key_type = redis::Cmd::key_type(&key).query::<String>(&mut con);

            match key_type {
                Ok(value) => match value.as_str() {
                    "string" => {
                        let value = con.get(key.clone()).unwrap();
                        state.strings.push((key, value));
                    }
                    "list" => info!("List: {}", key),
                    "set" => info!("Set: {}", key),
                    "zset" => info!("Sorted Set: {}", key),
                    "hash" => {
                        let result: RedisResult<Vec<(String, String)>> = con.hgetall(key.clone());

                        info!("Hash: {}", key);

                        match result {
                            Ok(ls) => {
                                state.hashes.insert(key.clone(), ls);
                            }
                            Err(_) => {
                                info!("error");
                                state.hashes.insert(key.clone(), Vec::new());
                            }
                        }
                        // con.xread_options(&[key.as_str()], &["0"], &opts);
                    }
                    "stream" => {
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
                            Err(e) => info!("Ocurrió un error {}", e),
                        }
                    }
                    _ => info!("Unknown type: {}", value),
                },
                Err(e) => {
                    info!("Ocurrió un error: {}", e);
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

pub fn run_command(host: &str, port: &str, command: &str) -> Result<String, String> {
    if let Some((command, args)) = parse_command(command) {
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
