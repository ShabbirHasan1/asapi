// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use std::collections::HashMap;

use redis::streams::{StreamId, StreamReadOptions, StreamReadReply};
use redis::{self, Client, Commands, Connection, RedisResult, Value};

use crate::redism::connection::create_conn;
use crate::redism::state::RedisStreamState;
use crate::{error, info};

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
            .arg("COUNT")
            .arg(c)
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
    // pub fn xadd<'a, K: ToRedisArgs, ID: ToRedisArgs, F: ToRedisArgs, V: ToRedisArgs>(
    //     key: K,
    //     id: ID,
    //     items: &'a [(F, V)]
    // ) -> Self
    let result = if st.xadd_nomkstream {
        redis::cmd("XADD")
            .arg(&st.xadd_k)
            .arg("NOMKSTREAM")
            .arg(&st.xadd_id)
            .arg(&st.xadd_items)
            .query::<Value>(conn)
    } else {
        redis::cmd("XADD")
            .arg(&st.xadd_k)
            .arg(&st.xadd_id)
            .arg(&st.xadd_items)
            .query::<Value>(conn)
    };

    // TODO: Esto intentar con thread.
    match result {
        Ok(rresp) => {
            let opts = StreamReadOptions::default();
            let result: RedisResult<StreamReadReply> =
                conn.xread_options(&[&st.xadd_k], &["0"], &opts);

            match result {
                Ok(stream_keys) => {
                    streams.insert(st.xadd_k.clone(), Vec::new());
                    let ids = streams.get_mut(&st.xadd_k).unwrap();
                    let stream_ids: Vec<String> = stream_keys
                        .keys
                        .iter()
                        .flat_map(|k| k.ids.iter().map(|v| v.id).collect())
                        .collect();
                    ids.extend_from_slice(&stream_ids);
                }
                Err(e) => error!("Ocurrió un error {}", e),
            }
        }
        Err(_) => todo!(),
    }
}

// Redis
// XREAD [COUNT count] [BLOCK milliseconds] STREAMS key [key ...] id   [id ...]
// Redis/Rust
// pub fn xread<'a, K: ToRedisArgs, ID: ToRedisArgs>(
//   keys: &'a [K],
//   ids: &'a [ID]
// ) -> Self {
//   cmd("XREAD").arg("STREAMS").arg(keys).arg(ids)
//}
pub fn xread(conn: &mut redis::Connection, st: &RedisStreamState) -> RedisResponse {
    read_operation(
        "XREAD",
        conn.xinfo_consumers(&st.info_consumers_k, &st.info_consumers_g),
    )
}

pub fn xread_group(conn: &mut redis::Connection, st: &RedisStreamState) -> RedisResponse {
    read_operation(
        "XREAD",
        conn.xinfo_consumers(&st.info_consumers_k, &st.info_consumers_g),
    )
}
