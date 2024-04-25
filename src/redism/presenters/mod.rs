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
pub mod set;
pub mod zset;

use redis::{Client, RedisError, RedisResult, Value};

use crate::redism::parser::redis_value_to_string;

use super::state::RedisConnectionDefinition;

pub type RedisResponse = Result<String, String>;

#[inline(always)]
pub fn create_conn(host: &str, port: i16) -> Result<redis::Connection, RedisError> {
    //if Redis server needs secure connection
    // https://medium.com/swlh/tutorial-getting-started-with-rust-and-redis-69041dd38279
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

#[inline(always)]
pub fn read_operation(m: &str, result: RedisResult<Value>) -> RedisResponse {
    match result {
        Ok(rresp) => Ok(format!("{m} :: {pr}", pr = redis_value_to_string(&rresp))),
        Err(err) => Err(format!("{m} :: {err:?}")),
    }
}

pub fn run_cmd<F: FnMut(&mut redis::Connection) -> RedisResponse>(
    conn_def: &RedisConnectionDefinition,
    mut cb: F,
) -> Option<RedisResponse> {
    let connection = create_redis_connection(conn_def);

    Some(if let Ok(mut conn) = connection {
        cb(&mut conn)
    } else {
        Err(":: Not able to connect to {conn}.".to_string())
    })
}
