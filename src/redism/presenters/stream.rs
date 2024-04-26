// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use std::collections::HashMap;

use redis::streams::{StreamReadOptions, StreamReadReply};
use redis::{self, Client, Commands, Connection, RedisResult, Value};

use crate::info;
use crate::redism::connection::create_conn;

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
