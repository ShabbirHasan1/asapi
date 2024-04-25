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

use super::{read_operation, RedisResponse};
use crate::redism::state::RedisSetsState;

pub struct SetsPresenter;

impl SetsPresenter {
    pub fn sadd(
        conn: &mut redis::Connection,
        hm: &mut HashMap<String, Vec<String>>,
        st: &mut RedisSetsState,
    ) -> RedisResponse {
        SetsPresenter::write_set_operation(conn, "SADD", &st.sadd_k, hm, |conn| {
            conn.sadd(&st.sadd_k, st.sadd_vs.split(' ').collect::<Vec<&str>>())
        })
    }

    pub fn srem(
        conn: &mut redis::Connection,
        hm: &mut HashMap<String, Vec<String>>,
        st: &mut RedisSetsState,
    ) -> RedisResponse {
        SetsPresenter::write_set_operation(conn, "SREM", &st.srem_k, hm, |conn| {
            conn.spop(&st.srem_k)
        })
    }

    pub fn spop(
        conn: &mut redis::Connection,
        hm: &mut HashMap<String, Vec<String>>,
        st: &mut RedisSetsState,
    ) -> RedisResponse {
        SetsPresenter::write_set_operation(conn, "SPOP", &st.spop_k, hm, |conn| {
            conn.spop(&st.spop_k)
        })
    }

    pub fn srandmember(conn: &mut redis::Connection, st: &mut RedisSetsState) -> RedisResponse {
        let k = st.srandmember_k.clone();
        let count = st.srandmember_count.parse::<usize>().unwrap_or(1);
        let response = if count <= 1 {
            conn.srandmember_multiple(&k, 1)
        } else {
            conn.srandmember_multiple(&k, count)
        };

        read_operation("SRANDMEMBER", response)
    }

    pub fn sismember(conn: &mut redis::Connection, st: &mut RedisSetsState) -> RedisResponse {
        read_operation(
            "SISMEMBER",
            conn.sismember(&st.sismember_k, &st.sismember_m),
        )
    }

    pub fn smismember(conn: &mut redis::Connection, st: &mut RedisSetsState) -> RedisResponse {
        let vs = st.smismember_ms.split(' ').collect::<Vec<&str>>();
        read_operation("SMISMEMBER", conn.smismember(&st.smismember_k, &vs))
    }

    pub fn scard(conn: &mut redis::Connection, st: &mut RedisSetsState) -> RedisResponse {
        read_operation("SCARD", conn.scard(&st.scard_k))
    }

    pub fn smembers(
        conn: &mut redis::Connection,
        hm: &mut HashMap<String, Vec<String>>,
        st: &mut RedisSetsState,
    ) -> RedisResponse {
        SetsPresenter::_smembers(conn, &st.smembers_k, hm)
    }

    pub fn sinter(conn: &mut redis::Connection, st: &mut RedisSetsState) -> RedisResponse {
        let ks = st.sinter_ks.split(' ').collect::<Vec<&str>>();
        read_operation("SINTER", conn.sinter(ks))
    }

    pub fn sintercard(conn: &mut redis::Connection, st: &mut RedisSetsState) -> RedisResponse {
        let ks = st.sintercard_ks.split(' ').collect::<Vec<&str>>();
        let result = redis::cmd("SINTERCARD")
            .arg(&st.sintercard_numkeys)
            .arg(ks)
            .query::<Value>(conn);

        read_operation("SINTERCARD", result)
    }

    pub fn sinterstore(
        conn: &mut redis::Connection,
        hm: &mut HashMap<String, Vec<String>>,
        st: &mut RedisSetsState,
    ) -> RedisResponse {
        let ks = st
            .sinterstore_ks
            .split(' ')
            .map(|s| s.to_owned())
            .collect::<Vec<String>>();

        match conn
            .sinterstore::<&String, Vec<String>, redis::Value>(&st.sinterstore_destination, ks)
        {
            Ok(rresp) => {
                let _ = SetsPresenter::_smembers(conn, &st.sinterstore_destination, hm);
                Ok(format!("SINTERSTORE :: {rresp:?}"))
            }
            Err(err) => Err(format!("SINTERSTORE :: {err:?}")),
        }
    }

    pub fn sdiff(conn: &mut redis::Connection, st: &mut RedisSetsState) -> RedisResponse {
        let ks = st.sdiff_ks.split(' ').collect::<Vec<&str>>();

        read_operation("SDIFF", conn.sdiff(ks))
    }

    pub fn sdiffstore(
        conn: &mut redis::Connection,
        hm: &mut HashMap<String, Vec<String>>,
        st: &mut RedisSetsState,
    ) -> RedisResponse {
        let ks = st
            .sdiffstore_ks
            .split(' ')
            .map(|s| s.to_owned())
            .collect::<Vec<String>>();

        match conn.sdiffstore::<&String, Vec<String>, redis::Value>(&st.sdiffstore_destination, ks)
        {
            Ok(rresp) => {
                let _ = SetsPresenter::_smembers(conn, &st.sdiffstore_destination, hm);
                Ok(format!("SDIFFSTORE :: {rresp:?}"))
            }
            Err(err) => Err(format!("SDIFFSTORE :: {err:?}")),
        }
    }

    pub fn sunion(conn: &mut redis::Connection, st: &mut RedisSetsState) -> RedisResponse {
        let ks = st.sunion_ks.split(' ').collect::<Vec<&str>>();

        read_operation("SUNION", conn.sunion(ks))
    }

    pub fn sunionstore(
        conn: &mut redis::Connection,
        hm: &mut HashMap<String, Vec<String>>,
        st: &mut RedisSetsState,
    ) -> RedisResponse {
        let ks = st
            .sunionstore_ks
            .split(' ')
            .map(|s| s.to_owned())
            .collect::<Vec<String>>();

        match conn
            .sunionstore::<&String, Vec<String>, redis::Value>(&st.sunionstore_destination, ks)
        {
            Ok(rresp) => {
                let _ = SetsPresenter::_smembers(conn, &st.sunionstore_destination, hm);
                Ok(format!("SUNIONSTORE :: {rresp:?}"))
            }
            Err(err) => Err(format!("SUNIONSTORE :: {err:?}")),
        }
    }

    fn _smembers(
        conn: &mut Connection,
        k: &str,
        hm: &mut HashMap<String, Vec<String>>,
    ) -> RedisResponse {
        match conn.smembers::<&str, Vec<String>>(k) {
            Ok(rresp) => {
                let resp = format!("SMEMBERS :: {rr}", rr = rresp.join(", "));
                hm.insert(k.to_string(), rresp);

                Ok(resp)
            }
            Err(err) => Err(format!("SMEMBERS :: {err:?}")),
        }
    }

    #[inline(always)]
    fn write_set_operation(
        conn: &mut redis::Connection,
        m: &str,
        k: &str,
        hm: &mut HashMap<String, Vec<String>>,
        cl: impl Fn(&mut redis::Connection) -> RedisResult<Value>,
    ) -> RedisResponse {
        let result = cl(conn);
        match result {
            Ok(rresp) => {
                let value: Vec<String> = conn.smembers(k).unwrap();
                hm.insert(k.to_string(), value);
                Ok(format!("{m} :: {rr}", rr = redis_value_to_string(&rresp)))
            }
            Err(err) => Err(format!("{m} :: {err:?}")),
        }
    }
}
