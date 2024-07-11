// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use std::collections::BTreeMap;
use redis::{self, Commands, Connection, RedisError, RedisResult, Value};

use crate::{parser::redis_value_to_string, state::RedisStringState};

use super::{read_operation, RedisResponse};

pub enum NumericValue {
    Int,
    Float,
}

pub struct StringPresenter;

impl StringPresenter {
    pub fn lcs(conn: &mut Connection, st: &RedisStringState) -> RedisResponse {
        let is_len_filled = !st.lcs_len.is_empty();
        let is_idx_filled = !st.lcs_idx.is_empty();

        let result = if is_len_filled && is_idx_filled {
            redis::cmd("LCS")
                .arg(&st.lcs_k1)
                .arg(&st.lcs_k2)
                .arg(&st.lcs_len)
                .arg(&st.lcs_idx)
                .query::<Value>(conn)
        } else {
            redis::cmd("LCS")
                .arg(&st.lcs_k1)
                .arg(&st.lcs_k2)
                .query::<Value>(conn)
        };

        read_operation("LCS", result)
    }

    pub fn str_len(conn: &mut Connection, st: &RedisStringState) -> RedisResponse {
        read_operation("STRLEN", conn.strlen(&st.strlen_k))
    }

    pub fn append(
        conn: &mut Connection,
        strings: &mut BTreeMap<String, String>,
        st: &RedisStringState,
    ) -> RedisResponse {
        let (k, v) = (&st.set_k, &st.append_str);

        match conn.append::<&str, &str, redis::Value>(k, v) {
            Ok(redis_v) => {
                strings.insert(
                    k.clone(),
                    strings.get(k).unwrap_or(&"".to_string()).to_owned() + v,
                );
                Ok(format!(
                    "APPEND :: Resultado: {v} caracteres totales",
                    v = redis_value_to_string(&redis_v)
                ))
            }
            Err(err) => Ok(format!("ERROR APPEND :: {err:?}")),
        }
    }

    pub fn set(
        conn: &mut Connection,
        strings: &mut BTreeMap<String, String>,
        st: &RedisStringState,
    ) -> RedisResponse {
        Self::write_string_operation("SET", &st.set_k, &st.set_v, strings, || {
            conn.set(&st.set_k, &st.set_v)
        })
    }

    pub fn set_nx(
        conn: &mut Connection,
        strings: &mut BTreeMap<String, String>,
        st: &RedisStringState,
    ) -> RedisResponse {
        Self::write_string_operation("SETNX", &st.set_k, &st.set_v, strings, || {
            conn.set_nx(&st.set_k, &st.set_v)
        })
    }

    pub fn set_range(
        conn: &mut Connection,
        strings: &mut BTreeMap<String, String>,
        st: &RedisStringState,
    ) -> RedisResponse {
        let i_result = st.set_offset.parse::<isize>();

        match i_result {
            Ok(i) => {
                Self::write_string_operation("SETRANGE", &st.set_k, &st.set_v, strings, || {
                    conn.setrange(&st.set_k, i, &st.set_v)
                })
            }
            Err(err) => Err(format!("PARSE in SETRANGE :: {err:?}")),
        }
    }

    // Muy rimbombante para ver cómo podía hacerlo. No queda muy bien.
    pub fn _incr(
        conn: &mut Connection,
        strings: &mut BTreeMap<String, String>,
        k: &str,
        v: &str,
        t: NumericValue,
        m: &str,
    ) -> RedisResponse {
        match t {
            NumericValue::Int => match v.parse::<i64>() {
                Ok(i) => Self::write_string_operation(m, k, v, strings, || conn.incr(k, i)),
                _ => Err(format!("PARSEINT in {m}")),
            },
            NumericValue::Float => match v.parse::<f64>() {
                Ok(f) => Self::write_string_operation(m, k, v, strings, || conn.incr(k, f)),
                _ => Err(format!("PARSEFLOAT in {m}")),
            },
        }
    }

    #[inline]
    pub fn incr(
        conn: &mut Connection,
        strings: &mut BTreeMap<String, String>,
        st: &RedisStringState,
    ) -> RedisResponse {
        Self::_incr(conn, strings, &st.incr_k, "1", NumericValue::Int, "INCR")
    }

    #[inline]
    pub fn incr_by(
        conn: &mut Connection,
        strings: &mut BTreeMap<String, String>,
        st: &RedisStringState,
    ) -> RedisResponse {
        Self::_incr(
            conn,
            strings,
            &st.incr_k,
            &st.incr_by_v.to_owned(),
            NumericValue::Int,
            "INCR_BY",
        )
    }

    #[inline]
    pub fn incr_byfloat(
        conn: &mut Connection,
        strings: &mut BTreeMap<String, String>,
        st: &RedisStringState,
    ) -> RedisResponse {
        Self::_incr(
            conn,
            strings,
            &st.incr_k,
            &st.incr_byfloat_v.to_owned(),
            NumericValue::Float,
            "INCR_BYFLOAT",
        )
    }

    pub fn _decr(
        conn: &mut Connection,
        strings: &mut BTreeMap<String, String>,
        k: &str,
        v: &str,
    ) -> RedisResponse {
        let response = v
            .parse::<i64>()
            .map_err(|err| format!("{err:?}"))
            .and_then(|i| {
                conn.decr::<&str, i64, redis::Value>(k, i)
                    .map_err(|err| format!("{err:?}"))
                    .map(|value| redis_value_to_string(&value))
            });

        match response {
            Ok(v) => {
                strings.insert(k.to_owned(), v.to_owned());
                Ok(format!("DECR :: {v}"))
            }
            Err(err) => Err(format!("DECR :: {err:?}")),
        }
    }

    #[inline]
    pub fn decr(
        conn: &mut Connection,
        strings: &mut BTreeMap<String, String>,
        st: &RedisStringState,
    ) -> RedisResponse {
        Self::_decr(conn, strings, &st.decr_k, "1")
    }

    #[inline]
    pub fn decr_by(
        conn: &mut Connection,
        strings: &mut BTreeMap<String, String>,
        st: &RedisStringState,
    ) -> RedisResponse {
        Self::_decr(conn, strings, &st.decr_k, &st.decr_by_v)
    }

    pub fn get(conn: &mut Connection, st: &RedisStringState) -> RedisResponse {
        read_operation("GET", conn.get(&st.get_k))
    }

    pub fn get_del(
        conn: &mut Connection,
        strings: &mut BTreeMap<String, String>,
        st: &RedisStringState,
    ) -> RedisResponse {
        let k = &st.get_k;
        let result = conn.get_del::<&str, String>(k);

        match result {
            Ok(msg) => {
                strings.remove(k);
                Ok(format!("GETDEL :: Key: {k}, Value: {msg}"))
            }
            Err(err) => Err(format!("GETDEL :: {err:?}")),
        }
    }

    pub fn get_set(
        conn: &mut Connection,
        strings: &mut BTreeMap<String, String>,
        st: &RedisStringState,
    ) -> RedisResponse {
        let (k, v) = (&st.get_k, &st.getset_v);
        Self::write_string_operation("GETSET", k, v, strings, || conn.getset(k, v))
    }

    pub fn get_range(conn: &mut Connection, st: &RedisStringState) -> RedisResponse {
        match (
            st.get_offset_from.parse::<isize>(),
            st.get_offset_to.parse::<isize>(),
        ) {
            (Ok(f), Ok(t)) => read_operation("GETRANGE", conn.getrange(&st.get_k, f, t)),
            (Err(err1), Err(err2)) => Err(format!("{err1:?}\n{err2:?}")),
            (Err(err), _) | (_, Err(err)) => Err(format!("{err:?}")),
        }
    }

    pub fn get_ex(conn: &mut Connection, st: &RedisStringState) -> RedisResponse {
        match st.get_expire_seconds.parse::<usize>() {
            Ok(ex) => conn
                .get_ex::<&str, String>(&st.get_k, redis::Expiry::EX(ex))
                .map_or_else(
                    |err| Err(format!("GETEX :: {err:?}")),
                    |msg| Ok(format!("GETEX :: Key: {k}, Value: {msg}", k = st.get_k)),
                ),
            Err(err) => Err(format!("PARSE USIZE {err:?}")),
        }
    }

    pub fn _get(conn: &mut redis::Connection, k: &str) -> Result<String, RedisError> {
        conn.get::<&str, String>(k)
    }

    #[inline(always)]
    fn write_string_operation(
        // conn: &mut Connection,
        m: &str,
        k: &str,
        v: &str,
        strings: &mut BTreeMap<String, String>,
        mut cb: impl FnMut() -> RedisResult<Value>,
    ) -> RedisResponse {
        let result = cb();
        match result {
            Ok(rresp) => {
                strings.insert(k.to_owned(), v.to_owned());
                Ok(format!("{m} :: {rr}", rr = redis_value_to_string(&rresp)))
            }
            Err(err) => Err(format!("{m} :: {err:?}")),
        }
    }
}
