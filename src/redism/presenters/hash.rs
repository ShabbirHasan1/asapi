// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use redis::{self, Commands, Connection, RedisResult, Value};
use std::collections::HashMap;

use crate::redism::parser::redis_value_to_string;
use crate::redism::state::RedisHashState;

use super::{read_operation, RedisResponse};

pub struct HashPresenter;

impl HashPresenter {
    pub fn hrandfield(conn: &mut redis::Connection, st: &mut RedisHashState) -> RedisResponse {
        let result = redis::cmd("STRLEN")
            .arg(&st.hrandfield_k)
            .arg(&st.hrandfield_count)
            .arg("WITHVALUES")
            .query::<Value>(conn);

        read_operation("HRANDFIELD", result)
    }

    pub fn hexists(conn: &mut redis::Connection, st: &mut RedisHashState) -> RedisResponse {
        read_operation("HEXISTS", conn.hexists(&st.hexists_f, &st.hexists_k))
    }

    pub fn hstrlen(conn: &mut redis::Connection, st: &mut RedisHashState) -> RedisResponse {
        let result = redis::cmd("STRLEN")
            .arg(&st.hstrlen_k)
            .arg(&st.hstrlen_f)
            .query::<Value>(conn);

        read_operation("HSTRLEN", result)
    }

    pub fn hlen(conn: &mut redis::Connection, st: &mut RedisHashState) -> RedisResponse {
        read_operation("HLEN", conn.hlen(&st.hlen_k))
    }

    pub fn hincrbyfloat(
        conn: &mut redis::Connection,
        hm: &mut HashMap<String, Vec<(String, String)>>,
        st: &mut RedisHashState,
    ) -> RedisResponse {
        let f = st.hincrbyfloat_increment.parse::<f64>().unwrap_or_default();

        HashPresenter::_hincrby(
            conn,
            &st.hincrbyfloat_k,
            &st.hincrbyfloat_f,
            f,
            hm,
            "HINCRBYFLOAT",
        )
    }

    pub fn hincrby(
        conn: &mut redis::Connection,
        hm: &mut HashMap<String, Vec<(String, String)>>,
        st: &mut RedisHashState,
    ) -> RedisResponse {
        let i = st.hincrby_increment.parse::<i64>().unwrap_or_default();

        HashPresenter::_hincrby(conn, &st.hincrby_k, &st.hincrby_f, i as f64, hm, "HINCRBY")
    }

    fn _hincrby(
        conn: &mut Connection,
        k: &str,
        f: &str,
        value: f64,
        hm: &mut HashMap<String, Vec<(String, String)>>,
        name: &str,
    ) -> RedisResponse {
        HashPresenter::_write_hash_operation(conn, name, k, hm, |conn: &mut Connection| {
            conn.hincr(k, f, value)
        })
    }

    pub fn hsetnx(
        conn: &mut redis::Connection,
        hm: &mut HashMap<String, Vec<(String, String)>>,
        st: &mut RedisHashState,
    ) -> RedisResponse {
        let callback = |conn: &mut Connection| conn.hset_nx(&st.hsetnx_k, &st.hsetnx_f, &st.hset_v);

        HashPresenter::_write_hash_operation(conn, "HSETNX", &st.hsetnx_k, hm, callback)
    }

    pub fn hset(
        conn: &mut redis::Connection,
        hm: &mut HashMap<String, Vec<(String, String)>>,
        st: &mut RedisHashState,
    ) -> RedisResponse {
        let callback = |conn: &mut Connection| conn.hset(&st.hset_k, &st.hset_f, &st.hset_v);

        HashPresenter::_write_hash_operation(conn, "HSET", &st.hset_k, hm, callback)
    }

    pub fn hdel(
        conn: &mut redis::Connection,
        hm: &mut HashMap<String, Vec<(String, String)>>,
        st: &mut RedisHashState,
    ) -> RedisResponse {
        let fs = st.hdel_fs.split(' ').collect::<Vec<&str>>();
        let callback = |conn: &mut Connection| conn.hdel(&st.hdel_k, &fs);

        HashPresenter::_write_hash_operation(conn, "HDEL", &st.hdel_k, hm, callback)
    }

    pub fn hvals(conn: &mut redis::Connection, st: &mut RedisHashState) -> RedisResponse {
        read_operation("HVALS", conn.hvals(&st.hvals_k))
    }

    pub fn hkeys(conn: &mut redis::Connection, st: &mut RedisHashState) -> RedisResponse {
        read_operation("HKEYS", conn.hkeys(&st.hkeys_k))
    }

    pub fn hgetall(conn: &mut redis::Connection, st: &mut RedisHashState) -> RedisResponse {
        read_operation("HGETALL", conn.hgetall(&st.hgetall_k))
    }

    pub fn hmget(conn: &mut redis::Connection, st: &mut RedisHashState) -> RedisResponse {
        let fs = st.hmget_fs.split(' ').collect::<Vec<&str>>();
        read_operation("HMGET", conn.hget(&st.hmget_k, &fs))
    }

    pub fn hget(conn: &mut redis::Connection, st: &mut RedisHashState) -> RedisResponse {
        read_operation("HGET", conn.hget(&st.hget_k, &st.hget_f))
    }

    #[inline(always)]
    fn _write_hash_operation(
        conn: &mut redis::Connection,
        m: &str,
        k: &str,
        hm: &mut HashMap<String, Vec<(String, String)>>,
        cl: impl Fn(&mut redis::Connection) -> RedisResult<Value>,
    ) -> RedisResponse {
        let result = cl(conn);
        match result {
            Ok(rresp) => {
                let value: Vec<(String, String)> = conn.hgetall(&k).unwrap_or_default();
                hm.insert(k.to_string(), value);
                Ok(format!("{m} :: {rr}", rr = redis_value_to_string(&rresp)))
            }
            Err(err) => Err(format!("{m} :: {err:?}")),
        }
    }
}
