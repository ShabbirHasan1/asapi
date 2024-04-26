// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use redis::{self, Commands, Value};
use std::{collections::HashMap, num::NonZeroUsize};

use crate::redism::parser::redis_value_to_string;

use super::RedisResponse;
use crate::redism::state::RedisListState;

pub enum RedisPosition {
    Before,
    End,
}

pub struct ListPresenter;

impl ListPresenter {
    pub fn lset(
        conn: &mut redis::Connection,
        hm: &mut HashMap<String, Vec<String>>,
        st: &RedisListState,
    ) -> RedisResponse {
        let index = st.lset_index.parse::<isize>().unwrap_or(0);
        let k = st.lset_k.clone();

        match conn.lset(&k, index, &st.lset_value) {
            Ok(rresp) => {
                let value: Vec<String> = conn.lrange(&k, 0, isize::MAX).unwrap();
                hm.insert(k, value);
                Ok(format!("LSET :: {rr}", rr = redis_value_to_string(&rresp)))
            }
            Err(err) => Err(format!("LSET :: {err:?}")),
        }
    }

    pub fn lrem(
        conn: &mut redis::Connection,
        hm: &mut HashMap<String, Vec<String>>,
        st: &RedisListState,
    ) -> RedisResponse {
        let count = st.lrem_count.parse::<isize>().unwrap_or(0);
        let k = st.lrem_k.clone();

        match conn.lrem(&k, count, &st.lrem_value) {
            Ok(rresp) => {
                let value: Vec<String> = conn.lrange(&k, 0, isize::MAX).unwrap();
                hm.insert(k, value);
                Ok(format!("LREM :: {rr}", rr = redis_value_to_string(&rresp)))
            }
            Err(err) => Err(format!("ERROR LREM :: {err:?}")),
        }
    }

    pub fn linsert(
        conn: &mut redis::Connection,
        hm: &mut HashMap<String, Vec<String>>,
        st: &RedisListState,
        position: RedisPosition,
    ) -> RedisResponse {
        let response = match position {
            RedisPosition::Before => {
                conn.linsert_before(&st.linsert_k, &st.linsert_pivot, &st.linsert_value)
            }
            RedisPosition::End => {
                conn.linsert_after(&st.linsert_k, &st.linsert_pivot, &st.linsert_value)
            }
        };
        match response {
            Ok(rresp) => {
                let value: Vec<String> = conn.lrange(&st.linsert_k, 0, isize::MAX).unwrap();
                hm.insert(st.linsert_k.clone(), value);
                Ok(format!(
                    "LINSERT :: {rr}",
                    rr = redis_value_to_string(&rresp)
                ))
            }
            Err(err) => Err(format!("LINSERT :: {err:?}")),
        }
    }

    pub fn ltrim(
        conn: &mut redis::Connection,
        hm: &mut HashMap<String, Vec<String>>,
        st: &RedisListState,
    ) -> RedisResponse {
        let (start, stop) = (
            st.lrange_start.parse::<isize>(),
            st.lrange_stop.parse::<isize>(),
        );

        match (start, stop) {
            (Ok(b), Ok(e)) => match conn.ltrim(&st.ltrim_k, b, e) {
                Ok(rresp) => {
                    let value: Vec<String> = conn.lrange(&st.ltrim_k, 0, isize::MAX).unwrap();
                    hm.insert(st.ltrim_k.clone(), value);
                    Ok(format!("LTRIM :: {rr}", rr = redis_value_to_string(&rresp)))
                }
                Err(err) => Err(format!("LTRIM :: {err:?}")),
            },
            (Err(err1), Err(err2)) => Err(format!("LTRIM (1) :: {err1:?}\nLTRIM (2) :: {err2:?}")),
            (_, Err(err)) | (Err(err), _) => Err(format!("LTRIM :: {err:?}")),
        }
    }

    pub fn lrange(conn: &mut redis::Connection, st: &RedisListState) -> RedisResponse {
        // En caso del parseo fallar por la razón que sea devolvemos nada.
        let (start, stop) = (
            st.lrange_start.parse::<isize>().unwrap_or(0),
            st.lrange_stop.parse::<isize>().unwrap_or(0),
        );

        match conn.lrange(&st.lindex_k, start, stop) {
            Ok(rresp) => Ok(format!(
                "LRANGE :: {rr}",
                rr = redis_value_to_string(&rresp)
            )),
            Err(err) => Err(format!("LRANGE :: {err:?}")),
        }
    }

    pub fn lindex(conn: &mut redis::Connection, st: &RedisListState) -> RedisResponse {
        // En caso del parseo fallar por la razón que sea devolvemos nada.
        let idx = st.lindex_idx.parse::<isize>().unwrap_or(0);

        match conn.lindex(&st.lindex_k, idx) {
            Ok(rresp) => Ok(format!(
                "LINDEX :: {rr}",
                rr = redis_value_to_string(&rresp)
            )),
            Err(err) => Err(format!("LINDEX :: {err:?}")),
        }
    }

    pub fn llen(conn: &mut redis::Connection, st: &RedisListState) -> RedisResponse {
        match conn.llen(&st.llen_k) {
            Ok(rresp) => Ok(format!("LLEN :: {rr}", rr = redis_value_to_string(&rresp))),
            Err(err) => Err(format!("LLEN :: {err:?}")),
        }
    }

    pub fn rpop(
        conn: &mut redis::Connection,
        hm: &mut HashMap<String, Vec<String>>,
        st: &RedisListState,
    ) -> RedisResponse {
        let count = st.rpop_count.parse::<usize>().ok().and_then(|v| {
            if v > 0 {
                NonZeroUsize::new(v)
            } else {
                None
            }
        });
        let (k, v) = (st.rpop_k.clone(), count);
        let result = conn.rpop(&k, v);

        match result {
            Ok(rresp) => {
                // Lo más fácil tras éxito es recuperar los valores presentes en redis de nuevo
                let value: Vec<String> = conn.lrange(&k, 0, isize::MAX).unwrap();
                hm.insert(k, value);
                Ok(format!("RPOP :: {rr}", rr = redis_value_to_string(&rresp)))
            }
            Err(err) => Err(format!("RPOP :: {err:?}")),
        }
    }

    pub fn lpop(
        conn: &mut redis::Connection,
        hm: &mut HashMap<String, Vec<String>>,
        st: &RedisListState,
    ) -> RedisResponse {
        let count = st.lpop_count.parse::<usize>().ok().and_then(|v| {
            if v > 0 {
                NonZeroUsize::new(v)
            } else {
                None
            }
        });
        let (k, v) = (st.lpop_k.clone(), count);

        match conn.lpop(&k, v) {
            Ok(rresp) => {
                // Lo más fácil tras éxito es recuperar los valores presentes en redis de nuevo
                let value: Vec<String> = conn.lrange(&k, 0, isize::MAX).unwrap();
                hm.insert(k, value);
                Ok(format!("LPOP :: {rr}", rr = redis_value_to_string(&rresp)))
            }
            Err(err) => Err(format!("ERROR LPOP :: {err:?}")),
        }
    }

    pub fn rpush(
        conn: &mut redis::Connection,
        hm: &mut HashMap<String, Vec<String>>,
        st: &RedisListState,
    ) -> RedisResponse {
        let vs = st
            .rpush_vs
            .split(' ')
            .map(|s| s.to_owned())
            .collect::<Vec<String>>();
        let (k, v) = (st.rpush_k.clone(), vs);

        match conn.rpush(&k, v) {
            Ok(rresp) => {
                // Lo más fácil tras éxito es recuperar los valores presentes en redis de nuevo
                let value: Vec<String> = conn.lrange(&k, 0, isize::MAX).unwrap();
                hm.insert(k, value);
                Ok(format!("RPUSH :: {rr}", rr = redis_value_to_string(&rresp)))
            }
            Err(err) => Err(format!("ERROR RPUSH :: {err:?}")),
        }
    }

    pub fn lpush(
        conn: &mut redis::Connection,
        hm: &mut HashMap<String, Vec<String>>,
        st: &RedisListState,
    ) -> RedisResponse {
        let vs = st
            .lpush_vs
            .split(' ')
            .map(|s| s.to_owned())
            .collect::<Vec<String>>();
        let (k, v) = (st.lpush_k.clone(), vs);

        match conn.lpush(&k, v) {
            Ok(rresp) => {
                // Lo más fácil tras éxito es recuperar los valores presentes en redis de nuevo
                let value: Vec<String> = conn.lrange(&k, 0, isize::MAX).unwrap();
                hm.insert(k, value);
                Ok(format!("LPUSH :: {rr}", rr = redis_value_to_string(&rresp)))
            }
            Err(err) => Err(format!("LPUSH :: {err:?}")),
        }
    }

    pub fn rpushx(
        conn: &mut redis::Connection,
        hm: &mut HashMap<String, Vec<String>>,
        st: &RedisListState,
    ) -> RedisResponse {
        let vs = st
            .rpush_vs
            .split(' ')
            .map(|s| s.to_owned())
            .collect::<Vec<String>>();
        let (k, v) = (st.rpush_k.clone(), vs);

        match conn.rpush_exists(&k, v) {
            Ok(rresp) => {
                // Lo más fácil tras éxito es recuperar los valores presentes en redis de nuevo
                match rresp {
                    Value::Int(n_elements) => {
                        if n_elements > 0 {
                            let value: Vec<String> = conn.lrange(&k, 0, isize::MAX).unwrap();
                            hm.insert(k, value);
                        }
                        Ok(format!(
                            "RPUSHX :: {rr}",
                            rr = redis_value_to_string(&rresp)
                        ))
                    }
                    // Rama no alcanzable, lpushx devuelve n elementos insertados.
                    _ => todo!(),
                }
            }
            Err(err) => Err(format!("RPUSHX :: {err:?}")),
        }
    }

    pub fn lpushx(
        conn: &mut redis::Connection,
        hm: &mut HashMap<String, Vec<String>>,
        st: &RedisListState,
    ) -> RedisResponse {
        let vs = st
            .lpush_vs
            .split(' ')
            .map(|s| s.to_owned())
            .collect::<Vec<String>>();
        let (k, v) = (st.lpush_k.clone(), vs);

        match conn.lpush_exists(&k, v) {
            Ok(rresp) => {
                // Lo más fácil tras éxito es recuperar los valores presentes en redis de nuevo
                match rresp {
                    Value::Int(n_elements) => {
                        if n_elements > 0 {
                            let value: Vec<String> = conn.lrange(&k, 0, isize::MAX).unwrap();
                            hm.insert(k, value);
                        }
                        Ok(format!(
                            "LPUSHX :: {rr}",
                            rr = redis_value_to_string(&rresp),
                        ))
                    }
                    // Rama no alcanzable, lpushx devuelve n elementos insertados.
                    _ => todo!(),
                }
            }
            Err(err) => Err(format!("LPUSHX :: {err:?}")),
        }
    }
}
