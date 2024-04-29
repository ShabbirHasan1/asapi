// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use redis::streams::{StreamId, StreamReadOptions, StreamReadReply};
use redis::{self, Commands, Connection, RedisResult, Value};
use std::collections::{BTreeMap, HashMap};
use std::time::SystemTime;

use crate::common::traits::{Show, Tree};
use crate::error;
use crate::redism::connection::{create_conn, create_redis_connection};
use crate::redism::parser::redis_value_to_string;
use crate::redism::state::{RedisConnectionDefinition, RedisStreamReaderStorage, RedisStreamState};
use crate::redism::utils::value_map_to_string_btree_map;

use super::{read_operation, RedisResponse};

pub fn delete_stream_message(
    host: &str,
    port: i16,
    stream_name: &str,
    msg_id: &str,
) -> RedisResult<i8> {
    create_conn(host, port)
        .and_then(|mut con: redis::Connection| con.xdel::<&str, &str, i8>(stream_name, &[msg_id]))
}

// Leemos los datos de un stream concreto
// TODO: ... no tengo muy claro ya para qué lo gasto.
pub fn read_stream_id(
    conn_def: &RedisConnectionDefinition,
    stream_key: &str,
    id: &str,
    state: &mut HashMap<String, HashMap<String, Value>>,
) -> RedisResult<()> {
    let mut conn = create_redis_connection(conn_def)?;

    // This gives the next one to the one inside the array, so filtering with rust until other option.
    let opts = StreamReadOptions::default().count(1);
    let result: RedisResult<StreamReadReply> = conn.xread_options(&[stream_key], &[id], &opts);

    match result {
        Ok(stream_keys) => {
            for entry in stream_keys.keys {
                for stream_id in entry.ids {
                    state.insert(stream_id.id, stream_id.map.clone());
                }
            }
        }
        Err(e) => error!("Ocurrió un error {}", e),
    }

    Ok(())
}

pub fn info_stream(conn: &mut redis::Connection, st: &RedisStreamState) -> RedisResponse {
    let c = st.info_stream_count.parse::<usize>().unwrap_or(10);
    let result = if st.info_stream_full {
        redis::cmd("XINFO")
            .arg("STREAM")
            .arg(&st.info_stream_k)
            .arg("FULL")
            .arg("COUNT")
            .arg(c)
            .query::<Value>(conn)
    } else {
        redis::cmd("XINFO")
            .arg("STREAM")
            .arg(&st.info_stream_k)
            .query::<Value>(conn)
    };

    read_operation("INFO STREAM", result)
}

#[inline]
pub fn info_groups(conn: &mut redis::Connection, st: &RedisStreamState) -> RedisResponse {
    read_operation("INFO GROUPS", conn.xinfo_groups(&st.info_groups_k))
}

#[inline]
pub fn info_consumers(conn: &mut redis::Connection, st: &RedisStreamState) -> RedisResponse {
    read_operation(
        "INFO CONSUMERS",
        conn.xinfo_consumers(&st.info_consumers_k, &st.info_consumers_g),
    )
}

#[inline]
pub fn xlen(conn: &mut redis::Connection, st: &RedisStreamState) -> RedisResponse {
    read_operation("XLEN", conn.xlen(&st.xlen_k))
}

pub fn xrange(conn: &mut redis::Connection, st: &RedisStreamState) -> RedisResponse {
    let result = if st.xrange_start.is_empty() || st.xrange_start.is_empty() {
        conn.xrange_all(&st.xrange_k)
    } else if st.xrange_count.is_empty() {
        conn.xrange(&st.xrange_k, &st.xrange_start, &st.xrange_end)
    } else {
        conn.xrange_count(
            &st.xrange_k,
            &st.xrange_start,
            &st.xrange_end,
            &st.xrange_count,
        )
    };
    read_operation("XRANGE", result)
}

pub fn xrevrange(conn: &mut redis::Connection, st: &RedisStreamState) -> RedisResponse {
    let result = if st.xrevrange_start.is_empty() || st.xrevrange_start.is_empty() {
        conn.xrevrange_all(&st.xrevrange_k)
    } else if st.xrevrange_count.is_empty() {
        conn.xrevrange(&st.xrevrange_k, &st.xrevrange_start, &st.xrevrange_end)
    } else {
        conn.xrevrange_count(
            &st.xrevrange_k,
            &st.xrevrange_start,
            &st.xrevrange_end,
            &st.xrevrange_count,
        )
    };
    read_operation("XREVRANGE", result)
}

pub fn xack(conn: &mut redis::Connection, st: &RedisStreamState) -> RedisResponse {
    let result = conn.xack(
        &st.xack_k,
        &st.xack_group,
        &st.xack_ids.split(' ').collect::<Vec<&str>>(),
    );
    read_operation("XACK", result)
}

pub fn xadd(
    conn: &mut redis::Connection,
    streams: &mut HashMap<String, Vec<String>>,
    st: &RedisStreamState,
) -> RedisResponse {
    let callback = |conn: &mut Connection| {
        if st.xadd_nomkstream {
            redis::cmd("XADD")
                .arg(&st.xadd_k)
                .arg("NOMKSTREAM")
                .arg(&st.xadd_id)
                .arg(
                    &st.xadd_items
                        .split(' ')
                        .collect::<Vec<&str>>()
                        .chunks(2)
                        .collect::<Vec<&[&str]>>(),
                )
                .query::<Value>(conn)
        } else {
            redis::cmd("XADD")
                .arg(&st.xadd_k)
                .arg(&st.xadd_id)
                .arg(
                    &st.xadd_items
                        .split(' ')
                        .collect::<Vec<&str>>()
                        .chunks(2)
                        .collect::<Vec<&[&str]>>(),
                )
                .query::<Value>(conn)
        }
    };

    write_stream_operation(conn, "XADD", &st.xadd_k, streams, callback)
}

pub fn xdel(
    conn: &mut redis::Connection,
    streams: &mut HashMap<String, Vec<String>>,
    st: &RedisStreamState,
) -> RedisResponse {
    let callback = |conn: &mut Connection| {
        conn.xdel(&st.xdel_k, &st.xdel_ids.split(' ').collect::<Vec<&str>>())
    };
    write_stream_operation(conn, "XDEL", &st.xdel_k, streams, callback)
}

pub fn xtrim(
    conn: &mut redis::Connection,
    streams: &mut HashMap<String, Vec<String>>,
    st: &RedisStreamState,
) -> RedisResponse {
    let callback = |conn: &mut Connection| {
        if st.xtrim_limit.is_empty() {
            redis::cmd("XTRIM")
                .arg(&st.xtrim_k)
                .arg(&st.xtrim_maxlen_minid)
                .arg(&st.xtrim_aprox_equal)
                .arg(&st.xtrim_threshold)
                .query::<Value>(conn)
        } else {
            redis::cmd("XTRIM")
                .arg(&st.xtrim_k)
                .arg(&st.xtrim_maxlen_minid)
                .arg(&st.xtrim_aprox_equal)
                .arg(&st.xtrim_threshold)
                .arg("LIMIT")
                .arg(&st.xtrim_limit)
                .query::<Value>(conn)
        }
    };

    write_stream_operation(conn, "XTRIM", &st.xtrim_k, streams, callback)
}

pub enum RedisStreamMessage {
    Pending(RedisStreamReaderStorage),
    Ready(RedisStreamReaderStorage),
    Error(String),
}
// pub type RedisStreamMessage = Result<RedisStreamReaderStorage, String>;
// pub type RedisStreamMessage = Result<HashMap<String, HashMap<String, BTreeMap<String, String>>>, String>;

/// --------------------------------------------------
/// Funciones para lectura de mensajes
/// --------------------------------------------------
pub fn blocking_xread(
    conn_def: RedisConnectionDefinition,
    // streams: &mut HashMap<String, HashMap<String, BTreeMap<String, String>>>,
    st: &RedisStreamState,
    tx: &std::sync::mpsc::Sender<RedisStreamMessage>,
) {
    let count = st.xread_count.parse::<usize>().unwrap_or_default();
    let block = st.xread_block_ms.parse::<usize>().unwrap_or_default();
    let keys = st
        .xread_keys
        .split(' ')
        .map(|s| s.to_owned())
        .collect::<Vec<String>>();
    let ids = st
        .xread_ids
        .split(' ')
        .map(|s| s.to_owned())
        .collect::<Vec<String>>();
    let tx_cloned = tx.clone();
    let ms = st.xread_block_ms.parse::<usize>().ok();
    let opts = if count > 0 && block > 0 {
        StreamReadOptions::default().count(count).block(block)
    } else if count > 0 {
        StreamReadOptions::default().count(count)
    } else if block > 0 {
        println!("Bloqueo {block} seconds");
        StreamReadOptions::default().block(block)
    } else {
        StreamReadOptions::default()
    };

    std::thread::spawn(move || {
        let conn_result = create_redis_connection(&conn_def);
        if conn_result.is_err() {
            return;
        }
        let mut conn = conn_result.unwrap();

        let now = SystemTime::now();
        for k in &keys {
            let msg = RedisStreamReaderStorage {
                stream: k.to_owned(),
                group: None,
                messages: HashMap::default(),
                system_time: now,
                block_ms: ms,
            };
            let _ = tx_cloned.send(RedisStreamMessage::Pending(msg));
        }
        let result: RedisResult<StreamReadReply> = conn.xread_options(&[keys], &[ids], &opts);

        match result {
            Ok(stream_keys) => {
                for entry in stream_keys.keys {
                    let mut tmp: HashMap<String, BTreeMap<String, String>> =
                        HashMap::with_capacity(entry.ids.len());
                    for stream_id in entry.ids {
                        tmp.insert(stream_id.id, value_map_to_string_btree_map(&stream_id.map));
                    }
                    let msg = RedisStreamReaderStorage {
                        stream: entry.key.to_owned(),
                        group: None,
                        messages: tmp,
                        system_time: now,
                        block_ms: ms,
                    };
                    let _ = tx_cloned.send(RedisStreamMessage::Ready(msg));
                }
            }
            Err(e) => {
                error!("Ocurrió un error {}", e);
                let _ = tx_cloned.send(RedisStreamMessage::Error(format!("{e:?}")));
            }
        }
    });
}

pub fn xread(
    conn_def: &RedisConnectionDefinition,
    st: &RedisStreamState,
) -> Result<HashMap<String, HashMap<String, BTreeMap<String, String>>>, String> {
    let mut streams: HashMap<String, HashMap<String, BTreeMap<String, String>>> = HashMap::new();
    create_redis_connection(conn_def).map_or_else(
        |err| {
            Err(format!(
                "CONNECTION :: Not able to connect to {conn_def} ({err:?})."
            ))
        },
        |mut conn| {
            let count = st.xread_count.parse::<usize>().unwrap_or_default();
            let block = st.xread_block_ms.parse::<usize>().unwrap_or_default();

            let opts = if count > 0 && block > 0 {
                StreamReadOptions::default().count(count).block(block)
            } else if count > 0 {
                StreamReadOptions::default().count(count)
            } else if block > 0 {
                println!("Bloqueo {block} seconds");
                StreamReadOptions::default().block(block)
            } else {
                StreamReadOptions::default()
            };

            let result: RedisResult<StreamReadReply> = conn.xread_options(
                &[st.xread_keys.split(' ').collect::<Vec<&str>>()],
                &[st.xread_ids.split(' ').collect::<Vec<&str>>()],
                &opts,
            );

            println!("===== Después del block");

            match result {
                Ok(stream_keys) => {
                    for entry in stream_keys.keys {
                        let mut tmp: HashMap<String, BTreeMap<String, String>> =
                            HashMap::with_capacity(entry.ids.len());
                        for stream_id in entry.ids {
                            tmp.insert(stream_id.id, value_map_to_string_btree_map(&stream_id.map));
                        }
                        streams.insert(entry.key, tmp);
                    }
                    Ok(streams)
                }
                Err(e) => {
                    error!("Ocurrió un error {}", e);
                    Err(format!("{e:?}"))
                }
            }
        },
    )
}

impl Tree<String, HashMap<String, String>> for StreamId {
    fn to_tree(&self) -> HashMap<String, HashMap<String, String>> {
        let mut response: HashMap<String, HashMap<String, String>> = HashMap::default();
        response.insert(
            self.id.clone(),
            self.map.iter().fold(HashMap::default(), |mut acc, (k, v)| {
                acc.insert(k.to_owned(), redis_value_to_string(v));
                acc
            }),
        );
        response
    }
}

impl Show for StreamId {
    fn to_string(&self) -> String {
        serde_json::to_string(&self.to_tree()).unwrap_or_else(|err| {
            error!("{err:?}");
            format!("{self:?}")
        })
    }
}

impl Tree<String, Vec<HashMap<String, HashMap<String, String>>>> for StreamReadReply {
    fn to_tree(&self) -> HashMap<String, Vec<HashMap<String, HashMap<String, String>>>> {
        self.keys
            .iter()
            .fold(HashMap::new(), |mut acc, stream_key| {
                acc.insert(
                    stream_key.key.to_owned(),
                    stream_key
                        .ids
                        .iter()
                        .map(|e| e.to_tree())
                        .collect::<Vec<HashMap<String, HashMap<String, String>>>>(),
                );

                acc
            })
    }
}
impl Show for StreamReadReply {
    fn to_string(&self) -> String {
        serde_json::to_string(&self.to_tree()).unwrap_or_else(|err| {
            error!("{err:?}");
            format!("{self:?}")
        })
    }
}

pub fn xread_group(
    conn: &mut Connection,
    streams: &mut HashMap<String, Vec<String>>,
    st: &RedisStreamState,
) -> RedisResponse {
    read_operation(
        "XREAD",
        conn.xinfo_consumers(&st.info_consumers_k, &st.info_consumers_g),
    )
}

/// --------------------------------------------------
/// Funciones para manipulación de grupos.
/// --------------------------------------------------
pub fn xgroup_create(
    conn: &mut redis::Connection,
    streams: &mut HashMap<String, Vec<String>>,
    st: &RedisStreamState,
) -> RedisResponse {
    let k = &st.xgroup_create_k;
    let callback = |conn: &mut Connection| {
        if st.xgroup_create_mkstream {
            conn.xgroup_create_mkstream(k, &st.xgroup_create_group, &st.xgroup_create_id)
        } else {
            conn.xgroup_create(k, &st.xgroup_create_group, &st.xgroup_create_id)
        }
    };

    write_stream_operation(conn, "XGROUP CREATE", k, streams, callback)
}

pub fn xgroup_create_consumer(
    conn: &mut redis::Connection,
    streams: &mut HashMap<String, Vec<String>>,
    st: &RedisStreamState,
) -> RedisResponse {
    let k = &st.xgroup_create_consumer_k;
    let callback = |conn: &mut Connection| {
        redis::cmd("XGROUP")
            .arg("CREATECONSUMER")
            .arg(k)
            .arg(&st.xgroup_create_consumer_group)
            .arg(&st.xgroup_create_consumer)
            .query::<Value>(conn)
    };

    write_stream_operation(conn, "XGROUP CREATECONSUMER", k, streams, callback)
}

pub fn xgroup_del_consumer(
    conn: &mut redis::Connection,
    streams: &mut HashMap<String, Vec<String>>,
    st: &RedisStreamState,
) -> RedisResponse {
    let k = &st.xgroup_del_consumer_k;
    let callback = |conn: &mut Connection| {
        conn.xgroup_delconsumer(k, &st.xgroup_del_consumer_group, &st.xgroup_del_consumer)
    };

    write_stream_operation(conn, "XGROUP DELCONSUMER", k, streams, callback)
}

pub fn xgroup_destroy(
    conn: &mut redis::Connection,
    streams: &mut HashMap<String, Vec<String>>,
    st: &RedisStreamState,
) -> RedisResponse {
    let k = &st.xgroup_destroy_k;
    let callback = |conn: &mut Connection| conn.xgroup_destroy(k, &st.xgroup_destroy_group);

    write_stream_operation(conn, "XGROUP DESTROY", k, streams, callback)
}

pub fn xgroup_setid(
    conn: &mut redis::Connection,
    streams: &mut HashMap<String, Vec<String>>,
    st: &RedisStreamState,
) -> RedisResponse {
    let k = &st.xgroup_setid_k;
    let callback =
        |conn: &mut Connection| conn.xgroup_setid(k, &st.xgroup_setid_g, &st.xgroup_setid_id);

    write_stream_operation(conn, "XGROUP SETID", k, streams, callback)
}

/// --------------------------------------------------
/// Funciones Auxiliares
/// --------------------------------------------------

fn write_stream_operation(
    conn: &mut Connection,
    m: &str,
    k: &str,
    hm: &mut HashMap<String, Vec<String>>,
    cb: impl Fn(&mut Connection) -> RedisResult<Value>,
) -> RedisResponse {
    let result = cb(conn);
    match result {
        Ok(rresp) => {
            let opts = StreamReadOptions::default();
            let result: RedisResult<StreamReadReply> = conn.xread_options(&[k], &["0"], &opts);

            match result {
                Ok(stream_keys) => {
                    hm.insert(k.to_owned(), Vec::default());
                    let ids = hm.get_mut(k).unwrap();
                    let stream_ids: Vec<String> = stream_keys
                        .keys
                        .iter()
                        .flat_map(|k| {
                            k.ids
                                .iter()
                                .map(|v| v.id.to_string())
                                .collect::<Vec<String>>()
                        })
                        .collect();
                    ids.extend(stream_ids);
                    // ids.extend_from_slice(&stream_ids);
                    Ok(format!("{m} :: {rr}", rr = redis_value_to_string(&rresp)))
                }
                Err(e) => Ok(format!("{m} :: {e:?}")),
            }
        }
        Err(e) => Err(format!("{m} :: {e:?}")),
    }
}
