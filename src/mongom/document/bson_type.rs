// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use bson::Bson;

/// Necesario para poder abstraerme de los valores asociados cuando los haya.
///
/// No podemos usar una variante como valor sin especificar el valor asociado,
/// luego no puedo usarlas directamente en un `selectable_value`, luego
/// esto es necesario.

#[derive(PartialEq, Clone)]
pub enum BsonType {
    Double,
    String,
    Array,
    Document,
    Boolean,
    Null,
    RegularExpression,
    Int32,
    Int64,
    Timestamp,
    Binary,
    ObjectId,
    DateTime,
    Symbol,
    /// [128-bit decimal floating point](https://github.com/mongodb/specifications/blob/master/source/bson-decimal128/decimal128.rst)
    Decimal128,
    // MaxKey,
    // MinKey,
    // JavaScriptCode(String),
    // JavaScriptCodeWithScope,
    // Undefined, /// Undefined value (Deprecated)
    // DbPointer(DbPointer), /// DBPointer (Deprecated)
}

impl Copy for BsonType {}

impl From<&Bson> for BsonType {
    fn from(bs: &Bson) -> Self {
        match bs {
            Bson::Double(_) => BsonType::Double,
            Bson::String(_) => BsonType::String,
            Bson::Array(_) => BsonType::Array,
            Bson::Document(_) => BsonType::Document,
            Bson::Boolean(_) => BsonType::Boolean,
            Bson::RegularExpression(_) => BsonType::RegularExpression,
            Bson::Int32(_) => BsonType::Int32,
            Bson::Int64(_) => BsonType::Int64,
            Bson::Timestamp(_) => BsonType::Timestamp,
            Bson::Binary(_) => BsonType::Binary,
            Bson::ObjectId(_) => BsonType::ObjectId,
            Bson::DateTime(_) => BsonType::DateTime,
            Bson::Symbol(_) => BsonType::Symbol,
            Bson::Decimal128(_) => BsonType::Decimal128,
            Bson::Null => BsonType::Null,
            // Bson::DbPointer(_) => BsonType::Null,
            // Bson::JavaScriptCode(_) => BsonType::Null,
            // Bson::JavaScriptCodeWithScope(_) => BsonType::Null,
            // Bson::MinKey => BsonType::Null,
            // Bson::MaxKey => BsonType::Null,
            // Bson::Undefined => BsonType::Null,
            _ => BsonType::Null,
        }
    }
}

impl BsonType {
    pub fn as_str(&self) -> &str {
        match self {
            BsonType::Double => "Double",
            BsonType::String => "String",
            BsonType::Array => "Array",
            BsonType::Document => "Document",
            BsonType::Boolean => "Boolean",
            BsonType::Null => "Null",
            BsonType::RegularExpression => "RegEx",
            BsonType::Int32 => "Int32",
            BsonType::Int64 => "Int64",
            BsonType::Timestamp => "Timestamp",
            BsonType::Binary => "Binary",
            BsonType::ObjectId => "ObjectId",
            BsonType::DateTime => "DateTime",
            BsonType::Symbol => "Symbol",
            BsonType::Decimal128 => "Decimal128",
        }
    }
}
