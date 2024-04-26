// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

pub mod hash;
pub mod json;
pub mod list;
pub mod pubsub;
pub mod set;
pub mod stream;
pub mod string;
pub mod zset;

use redis::{Commands as _, Connection, RedisResult, Value};

use crate::redism::parser::redis_value_to_string;

use super::{
    connection::{create_conn_with_default, create_redis_connection},
    state::RedisConnectionDefinition,
};

pub type RedisResponse = Result<String, String>;

#[inline(always)]
pub fn read_operation(m: &str, result: RedisResult<Value>) -> RedisResponse {
    match result {
        Ok(rresp) => Ok(format!("{m} :: {pr}", pr = redis_value_to_string(&rresp))),
        Err(err) => Err(format!("{m} :: {err:?}")),
    }
}

#[inline(always)]
pub fn run_cmd<F: FnMut(&mut redis::Connection) -> RedisResponse>(
    conn_def: &RedisConnectionDefinition,
    mut cb: F,
) -> Option<RedisResponse> {
    Some(create_redis_connection(conn_def).map_or_else(
        |err| Err(format!(":: Not able to connect to {conn_def} ({err:?}).")),
        |mut conn| cb(&mut conn),
    ))
}

#[inline(always)]
pub fn delete_key(host: &str, port: &str, key: &str) -> RedisResult<i8> {
    create_conn_with_default(host, port).and_then(|mut con| con.del(key))
}

// Borrado por entrada, un hash entero no se puede borrar. Se borra cuando no le quedan entradas.
#[inline(always)]
pub fn delete_hashkey(host: &str, port: &str, hash_name: &str, field_key: &str) -> RedisResult<i8> {
    create_conn_with_default(host, port).and_then(|mut con| con.hdel(hash_name, field_key))
}

#[inline(always)]
pub fn run_read_generic<S>(
    conn_def: &RedisConnectionDefinition,
    state: &S,
    mut cb: impl FnMut(&mut Connection, &S) -> RedisResponse,
) -> Option<RedisResponse> {
    run_cmd(conn_def, |conn| cb(conn, state))
}

#[inline(always)]
pub fn run_write_generic<S, C>(
    conn_def: &RedisConnectionDefinition,
    state: &S,
    col: &mut C,
    mut cb: impl FnMut(&mut Connection, &mut C, &S) -> RedisResponse,
) -> Option<RedisResponse> {
    run_cmd(conn_def, |conn| cb(conn, col, state))
}
