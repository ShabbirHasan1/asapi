// -------------------------------------------------------------------------
// Copyright (C) 2023 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use redis::Value;
use serde_json::{self, Value as JsonValue};
use std::collections::{BTreeMap, HashMap};

pub fn value_map_to_string(hm: &HashMap<String, Value>) -> String {
    let mut result = String::new();
    for (key, val) in hm {
        let val_str = value_to_string(val);
        result.push_str(&format!("{} {} ", key, val_str));
    }
    result
}

fn value_to_string(val: &Value) -> String {
    let val_str = match val {
        Value::Data(v) => {
            let data_str = String::from_utf8_lossy(v);
            match serde_json::from_str::<JsonValue>(&data_str) {
                Ok(_) => format!("'{}'", data_str), // Es un JSON o un array
                Err(_) => data_str.to_string(),     // No es un JSON
            }
        }
        Value::Int(v) => v.to_string(),
        Value::Status(s) => s.clone(),
        Value::Okay => "OK".to_string(),
        Value::Nil => "nil".to_string(),
        _ => "".to_string(),
    };
    val_str
}

// Esta función si solo se llama una vez no da problemas. Realmente ella no da problemas,
// es el llamarla cada ciclo de renderizado lo que lo da porque la iteración sobre un
// el diccionario es desordenada (esto no es python).
// Para solucionar puedo:
// - usar create::itertools, muy buena forma, mucha star en github
// - ordenar cutremente yo.
// - usar std::collections::BTreeMap
// opto por lo último.
// 090224: Si meto parámetro extra para decidir si ordeno o no, problemas con implementaciones que tengo que usan de forma limpia como callback en map (en redis::contextual_menus.rs)
//         Si uso trait (`) -> Box<dyn Iterable>`), no me sirve al usar para montar JsonTree en redis::view.rs
//         Así que lo que hago es usar dos implementaciones distintas, una que devuelva BTreeMap y otra HashMap
pub fn value_map_to_string_map(hm: &HashMap<String, Value>) -> HashMap<String, String> {
    let mut map: HashMap<String, String> = HashMap::default();

    for (k, v) in hm {
        if let redis::Value::Data(d) = v {
            if let Ok(s) = String::from_utf8(d.to_vec()) {
                map.insert(k.clone(), s);
            }
        }
    }
    map
}

pub fn value_map_to_string_btree_map(hm: &HashMap<String, Value>) -> BTreeMap<String, String> {
    let mut map: BTreeMap<String, String> = BTreeMap::default();

    for (k, v) in hm {
        if let redis::Value::Data(d) = v {
            if let Ok(s) = String::from_utf8(d.to_vec()) {
                map.insert(k.clone(), s);
            }
        }
    }
    map
}

// pub fn pair_vector_to_string_map(v: &Vec<(String, String)>) -> HashMap<String, String> {
//     let mut map: HashMap<String, String> = HashMap::default();
//     for (k, v) in v {
//         map.insert(k.clone(), v.clone());
//     }
//     map
// }
