// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use bson::{doc, Bson, Document};
use serde_json::{Map, Value};

use crate::info;

pub fn _json_value_to_document(value: &Value) -> Document {
    match value {
        Value::Object(obj) => {
            let doc: Document = obj
                .iter()
                .map(|(k, v)| (k.clone(), json_value_to_bson(v)))
                .collect();
            doc
        }
        _ => doc! {},
    }
}
pub fn json_value_to_bson(value: &Value) -> Bson {
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

/// Parseo de filtro propio a bson::Bson
// fn build_filter_bson(filter: &MongoFilter, ls: &[MongoFilter]) -> Bson {
//     match filter.op {
//         MongoOperator::EQ
//         | MongoOperator::NEQ
//         | MongoOperator::GT
//         | MongoOperator::GTE
//         | MongoOperator::LT
//         | MongoOperator::LTE
//         | MongoOperator::IN
//         | MongoOperator::NIN
//         | MongoOperator::NOT
//         | MongoOperator::Exists
//         | MongoOperator::HasType
//         | MongoOperator::ArrayContainsAll
//         | MongoOperator::Regex
//         | MongoOperator::NOR => {
//             let value_bson = filter
//                 .val
//                 .as_ref()
//                 .map_or(Bson::Null, |v| json_value_to_bson(v));
//             Bson::Document(
//                 doc! { filter.key.as_ref().unwrap().clone(): { filter.op.as_mongo_operator(): value_bson } },
//             )
//         }
//         MongoOperator::AND | MongoOperator::OR => {
//             let children_bson = filter
//                 .children
//                 .iter()
//                 .map(|&child_idx| {
//                     let child_filter = &ls.iter().find(|&f| f.idx == child_idx).unwrap();
//                     build_filter_bson(child_filter, ls)
//                 })
//                 .collect::<Vec<Bson>>();
//             let operator = if filter.op == MongoOperator::AND {
//                 "$and"
//             } else {
//                 "$or"
//             };
//             Bson::Document(doc! { operator: children_bson })
//         }
//     }
// }

/// Construimos bson::Document a partir de filtros introducidos
///
/// Hacemos un `fold` (realmente recursivo, no hay llamada a `fold` en sí)
/// y construimos bson::Document para poder consultar a MongoDB.
// pub fn build_mongo_query(ls: &[MongoFilter]) -> Document {
//     let root_filters = ls
//         .iter()
//         .filter(|f| f.parent.is_none()) // Empezamos con filtros raíz
//         .map(|f| build_filter_bson(f, ls))
//         .collect::<Vec<Bson>>();

//     match root_filters.len() {
//         // 1  => match root_filters.first().unwrap()
//         1 => match &root_filters[0] {
//             Bson::Document(doc) => doc.clone(),
//             _ => doc! {},
//         },
//         // Me es más fácil juntarlo todo bajo un `$and` que gestionarlo de forma implícita.
//         _ => doc! {"$and": root_filters},
//     }
// }

pub fn doc_to_pretty_string(docs: &[Document]) -> String {
    let json: Vec<Value> = docs.iter().map(doc_to_serde_value).collect();
    serde_json::to_string_pretty(&json).unwrap()
}

/// Convertimos BSON a serde_json::Value
///
/// En caso de error en alguno de los pasos que se dan para hacer la
/// transformación, devolvemos un `Value::Null`.
/// Hay que tratar con especial cuidado el `ObjectId` por cómo lo maneja
/// `serde_json`, porque al ser un `enum::ObjectId(oid: String)` crea un
/// nuevo campo y el `_id` queda de la forma
///     {"_id": {"$oid": "...."}}
/// lo que es incorrecto.
pub fn doc_to_serde_value(doc_bson: &Document) -> Value {
    serde_json::to_value(doc_bson)
        .ok()
        .map(|value: serde_json::Value| adjust_object_id(&value))
        .unwrap_or_else(|| serde_json::Value::Null)
}

fn adjust_object_id(value: &Value) -> Value {
    match value {
        Value::Object(obj) => {
            let mut new_obj = Map::new();
            for (key, val) in obj {
                if key == "_id" && val.is_object() && val.get("$oid").is_some() {
                    // Extraemos la cadena del $oid si es posible y la usamos como el nuevo valor de "_id".
                    if let Some(oid_value) = val.get("$oid") {
                        new_obj.insert(key.clone(), oid_value.clone());
                    }
                } else {
                    // Recursión por si hay más $oid
                    new_obj.insert(key.clone(), adjust_object_id(val));
                }
            }
            Value::Object(new_obj)
        }
        Value::Array(arr) => Value::Array(arr.iter().map(adjust_object_id).collect()),
        _ => value.clone(), // Los primitivos los copio.
    }
}

pub fn _pprint_docs(docs: &[Document]) {
    info!("{}", doc_to_pretty_string(docs));
}

pub fn _pprint_doc(doc: &Document) {
    let json: Value = doc_to_serde_value(doc);
    info!("{}", serde_json::to_string_pretty(&json).unwrap());
}
