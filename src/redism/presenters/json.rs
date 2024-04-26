// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use redis::JsonCommands;
use redis::{self, RedisResult, Value};
use std::collections::BTreeMap;

use crate::redism::parser::redis_value_to_string;
use crate::redism::state::RedisJsonState;

use super::{read_operation, RedisResponse};

pub struct JsonPresenter;

impl JsonPresenter {
    pub fn json_toggle(
        conn: &mut redis::Connection,
        jsons: &mut BTreeMap<String, String>,
        st: &RedisJsonState,
    ) -> RedisResponse {
        JsonPresenter::_write_json_operation(
            conn,
            "JSON.TOGGLE",
            st.json_forget_k.as_ref(),
            jsons,
            |conn: &mut redis::Connection| conn.json_toggle(&st.json_toggle_k, &st.json_toggle_p),
        )
    }

    pub fn json_merge(
        conn: &mut redis::Connection,
        jsons: &mut BTreeMap<String, String>,
        st: &RedisJsonState,
    ) -> RedisResponse {
        JsonPresenter::_write_json_operation(
            conn,
            "JSON.MERGE",
            st.json_forget_k.as_ref(),
            jsons,
            |conn: &mut redis::Connection| {
                redis::cmd("JSON.MERGE")
                    .arg(&st.json_merge_k)
                    .arg(&st.json_merge_p)
                    .arg(&st.json_merge_v)
                    .query::<Value>(conn)
            },
        )
    }

    pub fn json_type(conn: &mut redis::Connection, st: &RedisJsonState) -> RedisResponse {
        read_operation(
            "JSON.TYPE",
            conn.json_type(&st.json_type_k, &st.json_type_p),
        )
    }

    pub fn json_nummultby(
        conn: &mut redis::Connection,
        jsons: &mut BTreeMap<String, String>,
        st: &RedisJsonState,
    ) -> RedisResponse {
        JsonPresenter::_write_json_operation(
            conn,
            "JSON.NUMMULTBY",
            st.json_forget_k.as_ref(),
            jsons,
            |conn: &mut redis::Connection| {
                conn.json_num_incr_by(
                    &st.json_nummultby_k,
                    &st.json_nummultby_p,
                    st.json_nummultby_v.parse::<i64>().unwrap_or_default(),
                )
            },
        )
    }

    pub fn json_numincrby(
        conn: &mut redis::Connection,
        jsons: &mut BTreeMap<String, String>,
        st: &RedisJsonState,
    ) -> RedisResponse {
        JsonPresenter::_write_json_operation(
            conn,
            "JSON.NUMINCRBY",
            st.json_forget_k.as_ref(),
            jsons,
            |conn: &mut redis::Connection| {
                conn.json_num_incr_by(
                    &st.json_numincrby_k,
                    &st.json_numincrby_p,
                    st.json_numincrby_v.parse::<i64>().unwrap_or_default(),
                )
            },
        )
    }

    pub fn json_arrtrim(
        conn: &mut redis::Connection,
        jsons: &mut BTreeMap<String, String>,
        st: &RedisJsonState,
    ) -> RedisResponse {
        let callback = |conn: &mut redis::Connection| {
            conn.json_arr_trim(
                &st.json_arrtrim_k,
                &st.json_arrtrim_p,
                st.json_arrtrim_start.parse::<i64>().unwrap_or_default(),
                st.json_arrtrim_stop.parse::<i64>().unwrap_or_default(),
            )
        };

        JsonPresenter::_write_json_operation(
            conn,
            "JSON.ARRTRIM",
            st.json_forget_k.as_ref(),
            jsons,
            callback,
        )
    }

    pub fn json_arrpop(
        conn: &mut redis::Connection,
        jsons: &mut BTreeMap<String, String>,
        st: &RedisJsonState,
    ) -> RedisResponse {
        let callback = |conn: &mut redis::Connection| {
            conn.json_arr_pop(
                &st.json_arrpop_k,
                &st.json_arrpop_p,
                st.json_arrpop_idx.parse::<i64>().unwrap_or(-1),
            )
        };

        JsonPresenter::_write_json_operation(
            conn,
            "JSON.ARRPOP",
            st.json_forget_k.as_ref(),
            jsons,
            callback,
        )
    }

    pub fn json_arrinsert(
        conn: &mut redis::Connection,
        jsons: &mut BTreeMap<String, String>,
        st: &RedisJsonState,
    ) -> RedisResponse {
        let vs = &st.json_arrinsert_vs.split(' ').collect::<Vec<&str>>();
        let callback = |conn: &mut redis::Connection| {
            conn.json_arr_insert(
                &st.json_arrinsert_k,
                &st.json_arrinsert_p,
                st.json_arrinsert_idx.parse::<i64>().unwrap_or_default(),
                &vs,
            )
        };

        JsonPresenter::_write_json_operation(
            conn,
            "JSON.ARRINSERT",
            st.json_forget_k.as_ref(),
            jsons,
            callback,
        )
    }

    pub fn json_arrlen(conn: &mut redis::Connection, st: &RedisJsonState) -> RedisResponse {
        read_operation(
            "JSON.ARRLEN",
            conn.json_arr_len(&st.json_arrlen_k, &st.json_arrlen_p),
        )
    }

    pub fn json_arrindex(conn: &mut redis::Connection, st: &RedisJsonState) -> RedisResponse {
        let (start, stop) = (
            st.json_arrindex_start.parse::<isize>().unwrap_or_default(),
            st.json_arrindex_stop.parse::<isize>().unwrap_or_default(),
        );
        let result = conn.json_arr_index_ss(
            &st.json_arrindex_k,
            &st.json_arrindex_p,
            &st.json_arrindex_v,
            &start,
            &stop,
        );

        read_operation("JSON.ARRINDEX", result)
    }

    pub fn json_arrappend(
        conn: &mut redis::Connection,
        jsons: &mut BTreeMap<String, String>,
        st: &RedisJsonState,
    ) -> RedisResponse {
        let vs = &st.json_arrappend_vs.split(' ').collect::<Vec<&str>>();
        let callback = |conn: &mut redis::Connection| {
            conn.json_arr_append(&st.json_arrappend_k, &st.json_arrappend_p, &vs)
        };

        JsonPresenter::_write_json_operation(
            conn,
            "JSON.ARRAPPEND",
            st.json_forget_k.as_ref(),
            jsons,
            callback,
        )
    }

    pub fn json_strappend(
        conn: &mut redis::Connection,
        jsons: &mut BTreeMap<String, String>,
        st: &RedisJsonState,
    ) -> RedisResponse {
        let callback = |conn: &mut redis::Connection| {
            conn.json_str_append(
                &st.json_strappend_k,
                &st.json_strappend_p,
                &st.json_strappend_v,
            )
        };

        JsonPresenter::_write_json_operation(
            conn,
            "JSON.STRAPPEND",
            st.json_forget_k.as_ref(),
            jsons,
            callback,
        )
    }

    pub fn json_clear(
        conn: &mut redis::Connection,
        jsons: &mut BTreeMap<String, String>,
        st: &RedisJsonState,
    ) -> RedisResponse {
        let callback = |conn: &mut redis::Connection| {
            redis::cmd("JSON.CLEAR")
                .arg(&st.json_clear_k)
                .arg(&st.json_clear_p)
                .query::<Value>(conn)
        };

        JsonPresenter::_write_json_operation(
            conn,
            "JSON.CLEAR",
            st.json_forget_k.as_ref(),
            jsons,
            callback,
        )
    }

    pub fn json_forget(
        conn: &mut redis::Connection,
        jsons: &mut BTreeMap<String, String>,
        st: &RedisJsonState,
    ) -> RedisResponse {
        let callback = |conn: &mut redis::Connection| {
            redis::cmd("JSON.DEL")
                .arg(&st.json_del_k)
                .arg(&st.json_del_p)
                .query::<Value>(conn)
        };

        JsonPresenter::_write_json_operation(
            conn,
            "JSON.FORGET",
            st.json_forget_k.as_ref(),
            jsons,
            callback,
        )
    }

    pub fn json_del(
        conn: &mut redis::Connection,
        jsons: &mut BTreeMap<String, String>,
        st: &RedisJsonState,
    ) -> RedisResponse {
        let callback = |conn: &mut redis::Connection| conn.json_del(&st.json_del_k, &st.json_del_p);
        JsonPresenter::_write_json_operation(
            conn,
            "JSON.DEL",
            st.json_del_k.as_ref(),
            jsons,
            callback,
        )
    }

    pub fn json_set(
        conn: &mut redis::Connection,
        jsons: &mut BTreeMap<String, String>,
        st: &RedisJsonState,
    ) -> RedisResponse {
        let k = st.json_set_k.clone();
        let callback = |conn: &mut redis::Connection| match st.json_set_nx_xx.as_ref() {
            "NX" => redis::cmd("JSON.SET")
                .arg(&k)
                .arg(&st.json_set_p)
                .arg(&st.json_set_v)
                .arg("NX")
                .query::<Value>(conn),
            "XX" => redis::cmd("JSON.SET")
                .arg(&k)
                .arg(&st.json_set_p)
                .arg(&st.json_set_v)
                .arg("XX")
                .query::<Value>(conn),
            _ => conn.json_set(&k, &st.json_set_p, &st.json_set_v),
        };

        JsonPresenter::_write_json_operation(
            conn,
            "JSON.SET",
            st.json_set_k.as_ref(),
            jsons,
            callback,
        )
    }

    pub fn json_mget(conn: &mut redis::Connection, st: &RedisJsonState) -> RedisResponse {
        let ks = st.json_mget_ks.split(' ').collect::<Vec<&str>>();
        JsonPresenter::_json_get(conn, "JSON.MGET", &ks, &st.json_mget_p)
    }

    pub fn json_get(conn: &mut redis::Connection, st: &RedisJsonState) -> RedisResponse {
        JsonPresenter::_json_get(conn, "JSON.GET", &[&st.json_get_k], &st.json_mget_p)
    }

    pub fn json_objlen(conn: &mut redis::Connection, st: &RedisJsonState) -> RedisResponse {
        read_operation(
            "JSON.OBJLEN",
            conn.json_obj_len(&st.json_objlen_k, &st.json_objlen_p),
        )
    }

    pub fn json_objkeys(conn: &mut redis::Connection, st: &RedisJsonState) -> RedisResponse {
        read_operation(
            "JSON.OBJKEYS",
            conn.json_obj_keys(&st.json_objkeys_k, &st.json_objkeys_p),
        )
    }

    pub fn json_strlen(conn: &mut redis::Connection, st: &RedisJsonState) -> RedisResponse {
        read_operation(
            "JSON.STRLEN",
            conn.json_str_len(&st.json_strlen_k, &st.json_strlen_p),
        )
    }

    // ==================================================
    //          Funciones auxialiares privadas
    // ==================================================
    #[inline(always)]
    pub fn _json_get(conn: &mut redis::Connection, m: &str, k: &[&str], p: &str) -> RedisResponse {
        read_operation(m, conn.json_get(k, p))
    }

    #[inline(always)]
    fn _write_json_operation(
        conn: &mut redis::Connection,
        m: &str,
        k: &str,
        jsons: &mut BTreeMap<String, String>,
        cl: impl Fn(&mut redis::Connection) -> RedisResult<Value>,
    ) -> RedisResponse {
        let result = cl(conn);
        match result {
            Ok(rresp) => {
                let value = conn.json_get(k, "$").unwrap();
                jsons.insert(k.to_string(), value);
                Ok(format!("{m} :: {rr}", rr = redis_value_to_string(&rresp)))
            }
            Err(err) => Err(format!("{m} :: {err:?}")),
        }
    }
}
