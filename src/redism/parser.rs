// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use redis::Value;

pub fn redis_value_to_string(v: &redis::Value) -> String {
    match v {
        Value::Nil => "Nil".to_string(),
        Value::Int(i) => i.to_string(),
        Value::Data(d) => String::from_utf8(d.clone())
            .unwrap_or_else(|err| format!("ERROR {err:?} parsing {d:?}")),
        Value::Bulk(b) => b
            .iter()
            .map(redis_value_to_string)
            .collect::<Vec<String>>()
            .join(", "),
        Value::Status(s) => s.clone(),
        Value::Okay => "OK".to_string(),
    }
}
