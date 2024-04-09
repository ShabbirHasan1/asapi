// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use bson::{doc, Bson, Document};
use serde_json::Value;

use crate::{error, info};

use super::{document::find::MongoOperator, state::MongoFilter};

pub fn parse_user_input_to_bson(value: &str) -> Result<Bson, String> {
    let data: serde_json::Result<Value> = serde_json::from_str(value);
    match data {
        Ok(r_data) => match mongodb::bson::to_bson(&r_data) {
            Ok(r_data) => Ok(r_data),
            Err(err) => Err(format!("{:?}", err)),
        },
        Err(err) => Err(format!("{:?}", err)),
    }
}

fn json_value_to_bson(value: &Value) -> Bson {
    match value {
        Value::Null => Bson::Null,
        Value::Bool(b) => Bson::Boolean(*b),
        // serde::Number no tiene i32
        Value::Number(n) if n.is_i64() => Bson::Int64(n.as_i64().unwrap()),
        // Bson no soporta u64
        Value::Number(n) if n.is_u64() => Bson::Int64(n.as_u64().unwrap() as i64),
        Value::Number(n) if n.is_f64() => Bson::Double(n.as_f64().unwrap()),
        Value::String(s) => Bson::String(s.clone()),
        Value::Array(arr) => {
            let vec: Vec<Bson> = arr.iter().map(json_value_to_bson).collect();
            Bson::Array(vec)
        }
        Value::Object(obj) => {
            let doc: Document = obj
                .iter()
                .map(|(k, v)| (k.clone(), json_value_to_bson(v)))
                .collect();
            Bson::Document(doc)
        }
        _ => Bson::Null,
    }
}

pub fn json_to_document(value: &Value) -> Option<Document> {
    match json_value_to_bson(value) {
        Bson::Document(doc) => Some(doc),
        _ => None,
    }
}

/// Parseo de filtro propio a bson::Bson
fn build_filter_bson(filter: &MongoFilter, ls: &[MongoFilter]) -> Bson {
    match filter.op {
        MongoOperator::EQ
        | MongoOperator::NEQ
        | MongoOperator::GT
        | MongoOperator::GTE
        | MongoOperator::LT
        | MongoOperator::LTE
        | MongoOperator::IN
        | MongoOperator::NIN
        | MongoOperator::NOT
        | MongoOperator::Exists
        | MongoOperator::HasType
        | MongoOperator::ArrayContainsAll
        | MongoOperator::Regex
        | MongoOperator::NOR => {
            let value_bson = filter
                .val
                .as_ref()
                .map_or(Bson::Null, |v| json_value_to_bson(v));
            Bson::Document(
                doc! { filter.key.as_ref().unwrap().clone(): { filter.op.as_mongo_operator(): value_bson } },
            )
        }
        MongoOperator::AND | MongoOperator::OR => {
            let children_bson = filter
                .children
                .iter()
                .map(|&child_idx| {
                    let child_filter = &ls.iter().find(|&f| f.idx == child_idx).unwrap();
                    build_filter_bson(child_filter, ls)
                })
                .collect::<Vec<Bson>>();
            let operator = if filter.op == MongoOperator::AND {
                "$and"
            } else {
                "$or"
            };
            Bson::Document(doc! { operator: children_bson })
        }
    }
}

/// Construimos bson::Document a partir de filtros introducidos
///
/// Hacemos un `fold` (realmente recursivo, no hay llamada a `fold` en sí)
/// y construimos bson::Document para poder consultar a MongoDB.
pub fn build_mongo_query(ls: &[MongoFilter]) -> Document {
    let root_filters = ls
        .iter()
        .filter(|f| f.parent.is_none()) // Empezamos con filtros raíz
        .map(|f| build_filter_bson(f, ls))
        .collect::<Vec<Bson>>();

    match root_filters.len() {
        // 1  => match root_filters.first().unwrap()
        1 => match &root_filters[0] {
            Bson::Document(doc) => doc.clone(),
            _ => doc! {},
        },
        // Me es más fácil juntarlo todo bajo un `$and` que gestionarlo de forma implícita.
        _ => doc! {"$and": root_filters},
    }
}

/// Convertimos BSON a JSON e imprimimos
///
/// Para debuggear mucho más útil que el parseo normal, en cuanto hay varios
/// niveles de anidación es muy útil
pub fn pprint_bson(doc_bson: &Document) {
    let json: Value = bson::to_bson(doc_bson)
        .ok()
        .and_then(|b| b.as_document().cloned())
        .and_then(|bson_doc| serde_json::to_value(&bson_doc).ok())
        .unwrap_or_else(|| serde_json::Value::Null);

    let pretty_json = serde_json::to_string_pretty(&json).unwrap();
    info!("{}", pretty_json);
}

/// Forma sencilla de convertir, no me funciona pero dejo como referencia
pub fn convert_str_to_document(s: &str) -> Option<Document> {
    if let Ok(bson_value) = bson::from_slice::<bson::Bson>(s.as_bytes()) {
        info!("{:?}", bson_value);
        if let Bson::Document(doc) = bson_value {
            return Some(doc);
        }
    } else {
        let foo = bson::from_slice::<bson::Bson>(s.as_bytes());
        error!("{:?}", foo);
    }

    None
}

pub fn convert_json_value_to_bson_document(value: &Value) -> Option<Document> {
    serde_json::to_string(value)
        .ok()
        .and_then(|j| convert_str_to_document(&j))
}
