// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

// use std::collections::VecDeque;

// use bson::{doc, Bson};
// use serde_json::Value;

// use super::{document::find::MongoOperator, parser::json_value_to_bson};

// pub struct MongoFilter {
//     pub op: MongoOperator,
//     pub key: Option<String>,
//     pub val: Option<Value>,
//     pub children: VecDeque<MongoFilter>,
// }

// impl MongoFilter {
//     pub fn add_child(&mut self, child: MongoFilter) {
//         self.children.push_back(child);
//     }

//     pub fn build_mongo_query(&self) -> Bson {
//         match self.op {
//             MongoOperator::AND => Bson::Document(
//                 doc! { "$and": self.children.iter().map(|child| child.build_mongo_query()).collect::<Vec<Bson>>() },
//             ),
//             MongoOperator::OR => Bson::Document(
//                 doc! { "$or": self.children.iter().map(|child| child.build_mongo_query()).collect::<Vec<Bson>>() },
//             ),
//             MongoOperator::NOT => Bson::Document(
//                 doc! { "$not": self.children.iter().map(|child| child.build_mongo_query()).collect::<Vec<Bson>>() },
//             ),
//             _ => {
//                 let value_bson = self
//                     .val
//                     .as_ref()
//                     .map_or(Bson::Null, |v| json_value_to_bson(v));
//                 Bson::Document(
//                     doc! { self.key.as_ref().unwrap().clone(): { self.op.as_mongo_operator(): value_bson } },
//                 )
//             }
//         }
//     }
// }

use std::{
    collections::VecDeque,
    fmt::{self, Display},
};

use bson::{doc, Bson, Document};
use serde_json::Value;

use crate::info;

#[derive(PartialEq, Clone, Debug)]
pub enum MongoOperator {
    EQ,
    NEQ,
    IN,
    NIN,
    GT,
    GTE,
    LT,
    LTE,
    Exists,
    HasType,
    ArrayContainsAll,
    Regex,
    // Lógicos
    AND,
    OR,
    NOT,
    NOR,
}

impl MongoOperator {
    pub fn variants() -> &'static [MongoOperator] {
        static VARIANTS: [MongoOperator; 12] = [
            MongoOperator::EQ,
            MongoOperator::NEQ,
            MongoOperator::Exists,
            MongoOperator::IN,
            MongoOperator::NIN,
            MongoOperator::HasType,
            MongoOperator::ArrayContainsAll,
            MongoOperator::GT,
            MongoOperator::GTE,
            MongoOperator::LT,
            MongoOperator::LTE,
            MongoOperator::Regex,
        ];
        &VARIANTS
    }

    pub fn as_str(&self) -> &str {
        match self {
            MongoOperator::EQ => "Equals",
            MongoOperator::NEQ => "Doesn't Equal",
            MongoOperator::Exists => "Exists",
            MongoOperator::IN => "In",
            MongoOperator::NIN => "Not In",
            MongoOperator::HasType => "Has Type",
            MongoOperator::ArrayContainsAll => "Array Contains All",
            MongoOperator::GT => ">",
            MongoOperator::GTE => ">=",
            MongoOperator::LT => "<",
            MongoOperator::LTE => "<=",
            MongoOperator::Regex => "Regex",
            _ => "",
        }
    }

    pub fn extract_operator(&self) -> &str {
        match self {
            MongoOperator::EQ => "$eq",
            MongoOperator::NEQ => "$ne",
            MongoOperator::Exists => "$exists",
            MongoOperator::IN => "$in",
            MongoOperator::NIN => "$nin",
            MongoOperator::HasType => "$type",
            MongoOperator::ArrayContainsAll => "$all",
            MongoOperator::GT => "$gt",
            MongoOperator::GTE => "$gte",
            MongoOperator::LT => "$lt",
            MongoOperator::LTE => "$lte",
            MongoOperator::Regex => "$regex",
            MongoOperator::AND => "$and",
            MongoOperator::OR => "$or",
            MongoOperator::NOT => "$or",
            MongoOperator::NOR => "$nor",
        }
    }
}

impl Display for MongoOperator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let action_str = self.as_str();
        write!(f, "{}", action_str)
    }
}

#[derive(Debug, Clone)]
pub struct MongoFilter {
    pub op: MongoOperator,
    pub key: Option<String>,
    pub val: Option<Value>,
    pub children: VecDeque<MongoFilter>,
    pub idx: usize,
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

impl MongoFilter {
    pub fn new(op: MongoOperator, key: Option<String>, value: Option<Value>, idx: usize) -> Self {
        MongoFilter {
            op,
            key,
            val: value,
            children: VecDeque::default(),
            idx,
        }
    }

    pub fn add_child(&mut self, child: MongoFilter) {
        self.children.push_back(child);
    }

    pub fn pretty_print(&self, indent_level: usize) {
        let indent = "  ".repeat(indent_level);

        // Imprimir la información del filtro actual con el indentado adecuado
        info!("{}Operador: {:?}", indent, self.op);
        if let Some(ref key) = self.key {
            info!("  {}Clave: {}", indent, key);
        }
        if let Some(ref val) = self.val {
            info!("  {}Valor: {:?}", indent, val);
        }

        // Recursivamente imprimir los filtros hijos con un nivel de indentación incrementado
        for child in &self.children {
            child.pretty_print(indent_level + 1);
        }
    }

    pub fn build_mongo_query(&self) -> Bson {
        match self.op {
            MongoOperator::AND => Bson::Document(
                doc! { "$and": self.children.iter().map(|child| child.build_mongo_query()).collect::<Vec<Bson>>() },
            ),
            MongoOperator::OR => Bson::Document(
                doc! { "$or": self.children.iter().map(|child| child.build_mongo_query()).collect::<Vec<Bson>>() },
            ),
            MongoOperator::NOT => Bson::Document(
                doc! { "$not": self.children.iter().map(|child| child.build_mongo_query()).collect::<Vec<Bson>>() },
            ),
            MongoOperator::NOR => Bson::Document(
                doc! { "$nor": self.children.iter().map(|child| child.build_mongo_query()).collect::<Vec<Bson>>() },
            ),
            // En esta rama siempre debe entrar con key/value no None, tengo que ver cómo puedo
            // hacer que para estos lo otro sea obligado, no sé si se podrá.
            MongoOperator::EQ
            | MongoOperator::NEQ
            | MongoOperator::GT
            | MongoOperator::GTE
            | MongoOperator::LT
            | MongoOperator::LTE
            | MongoOperator::IN
            | MongoOperator::NIN
            | MongoOperator::Exists
            | MongoOperator::HasType
            | MongoOperator::ArrayContainsAll
            | MongoOperator::Regex => {
                let value_bson = self
                    .val
                    .as_ref()
                    .map_or(Bson::Null, |v| json_value_to_bson(v));
                Bson::Document(
                    doc! { self.key.as_ref().unwrap().clone(): { self.op.extract_operator(): value_bson } },
                )
            }
        }
    }
}

use eframe::egui;
use egui::Ui;

#[derive(PartialEq, Debug)]
pub enum UserAction {
    None,
    Delete(usize),
    AddAnd(usize),
    AddOr(usize),
    // Otras acciones según sea necesario...
}

impl MongoFilter {
    pub fn show(&self, ui: &mut Ui, actions: &mut Vec<UserAction>, index: usize) {
        ui.horizontal(|ui| {
            ui.label(format!("Operador: {:?}", self.op));
            if let Some(ref key) = self.key {
                ui.label(format!("Clave: {}", key));
            }
            if let Some(ref val) = self.val {
                ui.label(format!("Valor: {:?}", val));
            }

            if ui.button("AND").clicked() {
                actions.push(UserAction::AddAnd(index));
            }
            if ui.button("OR").clicked() {
                actions.push(UserAction::AddOr(index));
            }
            if ui.button("Delete").clicked() {
                actions.push(UserAction::Delete(index));
            }
        });

        // Recursivamente mostrar filtros hijos sin pasar `&mut self`
        ui.indent(
            format!(
                "{}/{:?}/{:?}",
                self.op.extract_operator(),
                self.key,
                self.val
            ),
            |ui| {
                for (i, child) in self.children.iter().enumerate() {
                    child.show(ui, actions, i);
                }
            },
        );
    }
}
