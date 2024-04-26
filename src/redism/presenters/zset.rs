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
use crate::redism::state::RedisZSetsState;

pub struct SortedSetsPresenter;

impl SortedSetsPresenter {
    pub fn zadd(
        conn: &mut redis::Connection,
        hm: &mut HashMap<String, Vec<String>>,
        st: &RedisZSetsState,
    ) -> RedisResponse {
        let s = st.zadd_score.parse::<f64>().unwrap_or(0.0);

        SortedSetsPresenter::write_zset_operation(conn, "ZADD", &st.zadd_k, hm, |conn| {
            conn.zadd(&st.zadd_k, &st.zadd_v, s)
        })
    }

    pub fn zrem(
        conn: &mut redis::Connection,
        hm: &mut HashMap<String, Vec<String>>,
        st: &RedisZSetsState,
    ) -> RedisResponse {
        SortedSetsPresenter::write_zset_operation(conn, "ZREM", &st.zrem_k, hm, |conn| {
            conn.zrem(&st.zrem_k, st.zrem_vs.split(' ').collect::<Vec<&str>>())
        })
    }

    pub fn zmpop(
        conn: &mut redis::Connection,
        hm: &mut HashMap<String, Vec<String>>,
        st: &RedisZSetsState,
    ) -> RedisResponse {
        let ks = st.zmpop_ks.split(' ').collect::<Vec<&str>>();
        let count = st.zmpop_count.parse::<isize>().unwrap_or_default();

        let result = match st.zmpop_min_max.as_ref() {
            "MIN" => conn.zmpop_min(&ks, count),
            "MAX" => conn.zmpop_min(&ks, count),
            _ => conn.zmpop_min(&ks, count),
        };

        match result {
            Ok(rresp) => {
                // TODO: Podría llamar a scan pero esto es menos costoso, aunque
                // podemos bloquear si es salvaje la cantidad de info en `sorted_sets`.
                // Implementar `zscan` basada en ZSCAN.
                for k in ks {
                    let value: Vec<String> = conn.zrange(k, 0, -1).unwrap();
                    hm.insert(k.to_string(), value);
                }
                let parsed_rresp = redis_value_to_string(&rresp);
                Ok(format!("ZMPOP :: {parsed_rresp}"))
            }
            Err(err) => Err(format!("ZMPOP :: {err:?}")),
        }
    }

    pub fn zrandmember(conn: &mut redis::Connection, st: &RedisZSetsState) -> RedisResponse {
        let k = st.zrandmember_k.clone();
        let count = st.zrandmember_count.parse::<isize>().unwrap_or(1);
        let response = if count <= 1 {
            conn.zrandmember(&k, None)
        } else {
            conn.zrandmember(&k, Some(count))
        };

        read_operation("ZRANDMEMBER", response)
    }

    pub fn zcard(conn: &mut redis::Connection, st: &RedisZSetsState) -> RedisResponse {
        read_operation("ZCARD", conn.zcard(&st.zcard_k))
    }

    pub fn zrange(conn: &mut redis::Connection, st: &RedisZSetsState) -> RedisResponse {
        let (start, stop) = (
            st.zrange_start.parse::<isize>(),
            st.zrange_stop.parse::<isize>(),
        );

        match (start, stop) {
            (Ok(start), Ok(stop)) => {
                read_operation("ZRANGE", conn.zrange(&st.zrange_k, start, stop))
            }
            (Err(err1), Err(err2)) => {
                Err(format!("ZRANGE (1) :: {err1:?}\nZRANGE (2) :: {err2:?}"))
            }
            (_, Err(err)) | (Err(err), _) => Err(format!("ZRANGE :: {err:?}")),
        }
    }

    pub fn zrangestore(
        conn: &mut redis::Connection,
        hm: &mut HashMap<String, Vec<String>>,
        st: &RedisZSetsState,
    ) -> RedisResponse {
        let (start, stop) = (
            st.zrange_start.parse::<isize>(),
            st.zrange_stop.parse::<isize>(),
        );

        match (start, stop) {
            (Ok(b), Ok(e)) => {
                let callback = |conn: &mut Connection| {
                    redis::cmd("ZRANGESTORE")
                        .arg(&st.zrangestore_destination)
                        .arg(&st.zrange_k)
                        .arg(b)
                        .arg(e)
                        .query::<Value>(conn)
                };

                SortedSetsPresenter::write_zset_operation(
                    conn,
                    "ZRANGESTORE",
                    &st.zrangestore_destination,
                    hm,
                    callback,
                )
            }
            (Err(err1), Err(err2)) => Err(format!(
                "ZRANGESTORE (1) :: {err1:?}\nZRANGESTORE (2) :: {err2:?}"
            )),
            (_, Err(err)) | (Err(err), _) => Err(format!("ZRANGESTORE :: {err:?}")),
        }
    }

    pub fn zrangebylex(conn: &mut redis::Connection, st: &RedisZSetsState) -> RedisResponse {
        let (min, max) = (
            st.zrangebylex_min.parse::<isize>(),
            st.zrangebylex_max.parse::<isize>(),
        );
        let k = st.zrange_k.clone();

        match (min, max) {
            (Ok(_min), Ok(_max)) => read_operation("ZRANGEBYLEX", conn.zrangebylex(k, _min, _max)),
            (Err(err1), Err(err2)) => Err(format!(
                "ZRANGEBYLEX (1) :: {err1:?}\nZRANGEBYLEX (2) :: {err2:?}"
            )),
            (_, Err(err)) | (Err(err), _) => Err(format!("ZRANGEBYLEX :: {err:?}")),
        }
    }

    pub fn zrangebyscore(conn: &mut redis::Connection, st: &RedisZSetsState) -> RedisResponse {
        let (min, max) = (
            st.zrangebyscore_min.parse::<isize>(),
            st.zrangebyscore_max.parse::<isize>(),
        );

        match (min, max) {
            (Ok(min), Ok(max)) => {
                read_operation("ZRANGEBYSCORE", conn.zrangebyscore(&st.zrange_k, min, max))
            }
            (Err(err1), Err(err2)) => Err(format!(
                "ZRANGEBYSCORE (1) :: {err1:?}\nZRANGEBYSCORE (2) :: {err2:?}"
            )),
            (_, Err(err)) | (Err(err), _) => Err(format!("ZRANGEBYSCORE :: {err:?}")),
        }
    }

    pub fn zinter(conn: &mut redis::Connection, st: &RedisZSetsState) -> RedisResponse {
        let ks = st.zinter_ks.split(' ').collect::<Vec<&str>>();
        let result = redis::cmd("ZINTER")
            .arg(ks.len())
            .arg(ks)
            .query::<Value>(conn);

        read_operation("ZINTER", result)
    }

    pub fn zintercard(conn: &mut redis::Connection, st: &RedisZSetsState) -> RedisResponse {
        let ks = st.zintercard_ks.split(' ').collect::<Vec<&str>>();
        let result = redis::cmd("ZINTECARD")
            .arg(ks.len())
            .arg(ks)
            .query::<Value>(conn);

        read_operation("ZINTERCARD", result)
    }

    pub fn zinterstore(
        conn: &mut redis::Connection,
        hm: &mut HashMap<String, Vec<String>>,
        st: &RedisZSetsState,
    ) -> RedisResponse {
        let ks = st.zinterstore_ks.split(' ').collect::<Vec<&str>>();

        SortedSetsPresenter::write_zset_operation(
            conn,
            "ZINTERSTORE",
            &st.zinterstore_destination,
            hm,
            |conn| conn.zinterstore(&st.zinterstore_destination, &ks),
        )
    }

    pub fn zunionstore(
        conn: &mut redis::Connection,
        hm: &mut HashMap<String, Vec<String>>,
        st: &RedisZSetsState,
    ) -> RedisResponse {
        let ks = st.zunionstore_ks.split(' ').collect::<Vec<&str>>();

        let callback = |conn: &mut Connection| match st.zunionstore_min_max.as_ref() {
            "MIN" => conn.zunionstore_min(&st.zunionstore_destination, &ks),
            "MAX" => conn.zunionstore_max(&st.zunionstore_destination, &ks),
            _ => conn.zunionstore(&st.zunionstore_destination, &ks),
        };

        SortedSetsPresenter::write_zset_operation(
            conn,
            "ZUNIONSTORE",
            &st.zunionstore_destination,
            hm,
            callback,
        )
    }

    pub fn zrank(conn: &mut redis::Connection, st: &RedisZSetsState) -> RedisResponse {
        read_operation("ZRANK", conn.zrank(&st.zrank_k, &st.zrank_m))
    }

    pub fn zrevrank(conn: &mut redis::Connection, st: &RedisZSetsState) -> RedisResponse {
        read_operation("ZREVRANK", conn.zrevrank(&st.zrevrank_k, &st.zrevrank_m))
    }

    pub fn zremrangebyrank(
        conn: &mut redis::Connection,
        hm: &mut HashMap<String, Vec<String>>,
        st: &RedisZSetsState,
    ) -> RedisResponse {
        let (b, e) = (
            st.zremrangebyrank_start.parse::<isize>(),
            st.zremrangebyrank_stop.parse::<isize>(),
        );

        match (b, e) {
            (Ok(bb), Ok(ee)) => SortedSetsPresenter::write_zset_operation(
                conn,
                "ZREMRANGEBYRANK",
                &st.zrange_k,
                hm,
                |conn: &mut Connection| conn.zremrangebyrank(&st.zrange_k, bb, ee),
            ),
            (Err(err1), Err(err2)) => Err(format!(
                "ZREMRANGEBYRANK (1) :: {err1:?}\nZREMRANGEBYRANK (2) :: {err2:?}"
            )),
            (_, Err(err)) | (Err(err), _) => Err(format!("ZREMRANGEBYRANK :: {err:?}")),
        }
    }

    #[inline(always)]
    fn write_zset_operation(
        conn: &mut redis::Connection,
        m: &str,
        k: &str,
        hm: &mut HashMap<String, Vec<String>>,
        cl: impl Fn(&mut redis::Connection) -> RedisResult<Value>,
    ) -> RedisResponse {
        let result = cl(conn);
        match result {
            Ok(rresp) => {
                let value: Vec<String> = conn.zrange(k, 0, -1).unwrap_or_default();
                hm.insert(k.to_string(), value);
                Ok(format!("{m} :: {rr}", rr = redis_value_to_string(&rresp)))
            }
            Err(err) => Err(format!("{m} :: {err:?}")),
        }
    }
}
