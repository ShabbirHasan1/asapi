// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

// Casi todo aquí copiado/replicado de sqlx, no exponen PgType y por uvas o peras
// es un follón depender de ese tipo, así que lo que hago es a partir de PgType.name()
// de una columna extraigo este PgType.

// No puedo usar name porque entonces pierdo el enum, puesto cada enum tiene un name
// distinto, y para eso me habría quedado con la implementación con Postgres.

use sqlx::{mysql::MySqlTypeInfo, TypeInfo};
use std::fmt::Display;

// Tipos copiados de la librería y algunos añadidos para no tener que usar flags y extraer con regexp.
#[derive(Debug)]
#[repr(u32)]
pub enum MySqlType {
    Bit,
    Blob,
    BlobBinary, // Lo creo yo.
    Boolean,    // Lo creo yo.
    Date,
    Datetime,
    Decimal,
    Double,
    Enum,
    Float,
    Geometry,
    Int24,
    Int24Unsigned, // Lo creo yo.
    Json,
    Long,
    LongUnsigned, // Lo creo yo.
    LongBlob,
    LongBlobBinary, // Lo creo yo.
    LongLong,
    LongLongUnsigned, // Lo creo yo.
    MediumBlob,
    MediumBlobBinary, // Lo creo yo.
    // NewDecimal, // Comento porque no sé cómo extraerla, se representa igual que `Decimal`.
    Null,
    Set,
    Short,
    ShortUnsigned, // Lo creo yo.
    String,
    StringBinary, // Lo creo yo.
    // StringEnum,   // Lo creo yo. // comento porque no sé cómo extraer, se representa igual que enum
    Time,
    Timestamp,
    Tiny,
    TinyUnsigned, // Lo creo yo.
    TinyBlob,
    TinyBlobBinary, // Lo creo yo.
    VarChar,
    VarCharBinary, // Lo creo yo.
    // VarString, // Comento porque no sé cómo extraerla, se representa igual que `VarChar`.
    Year,
    // Especiales, solo para generación
    Uuid,
}

impl MySqlType {
    pub fn from_string(name: &str) -> Self {
        // Representaciones generadas por Sqlx
        match name {
            "BIGINT UNSIGNED" => MySqlType::LongLongUnsigned,
            "BIGINT" => MySqlType::LongLong,
            "BINARY" => MySqlType::StringBinary,
            "BIT" => MySqlType::Bit,
            "BLOB" => MySqlType::BlobBinary,
            "BOOLEAN" => MySqlType::Boolean,
            "CHAR" => MySqlType::String,
            "DATE" => MySqlType::Date,
            "DATETIME" => MySqlType::Datetime,
            "DECIMAL" => MySqlType::Decimal,
            "DOUBLE" => MySqlType::Double,
            "ENUM" => MySqlType::Enum,
            // "ENUM" => MySqlType::StringEnum,
            "FLOAT" => MySqlType::Float,
            "GEOMETRY" => MySqlType::Geometry,
            "INT UNSIGNED" => MySqlType::LongUnsigned,
            "INT" => MySqlType::Long,
            "JSON" => MySqlType::Json,
            "LONGBLOB" => MySqlType::LongBlobBinary,
            "LONGTEXT" => MySqlType::LongBlob,
            "MEDIUMBLOB" => MySqlType::MediumBlobBinary,
            "MEDIUMINT UNSIGNED" => MySqlType::Int24Unsigned,
            "MEDIUMINT" => MySqlType::Int24,
            "MEDIUMTEXT" => MySqlType::MediumBlob,
            "NULL" => MySqlType::Null,
            "SET" => MySqlType::Set,
            "SMALLINT UNSIGNED" => MySqlType::ShortUnsigned,
            "SMALLINT" => MySqlType::Short,
            "TEXT" => MySqlType::Blob,
            "TIME" => MySqlType::Time,
            "TIMESTAMP" => MySqlType::Timestamp,
            "TINYBLOB" => MySqlType::TinyBlobBinary,
            "TINYINT UNSIGNED" => MySqlType::TinyUnsigned,
            "TINYINT" => MySqlType::Tiny,
            "TINYTEXT" => MySqlType::TinyBlob,
            "VARBINARY" => MySqlType::VarCharBinary,
            "VARCHAR" => MySqlType::VarChar,
            "YEAR" => MySqlType::Year,
            _ => MySqlType::Null,
        }
    }

    // Extracción a partir del column type extraído del catálogo
    pub fn from_column_type(s: &str) -> MySqlType {
        match s {
            "BIGINT UNSIGNED" => MySqlType::LongLongUnsigned,
            "BIGINT" => MySqlType::LongLong,
            "BINARY" => MySqlType::StringBinary,
            "BIT" => MySqlType::Bit,
            "BLOB" => MySqlType::BlobBinary,
            "BOOLEAN" => MySqlType::Boolean,
            "CHAR" => MySqlType::String,
            "DATE" => MySqlType::Date,
            "DATETIME" => MySqlType::Datetime,
            "DECIMAL" => MySqlType::Decimal,
            "DOUBLE" => MySqlType::Double,
            "ENUM" => MySqlType::Enum,
            // "ENUM" => MySqlType::StringEnum,
            "FLOAT" => MySqlType::Float,
            "GEOMETRY" => MySqlType::Geometry,
            "JSON" => MySqlType::Json,
            "LONGBLOB" => MySqlType::LongBlobBinary,
            "LONGTEXT" => MySqlType::LongBlob,
            "MEDIUMBLOB" => MySqlType::MediumBlobBinary,
            "MEDIUMINT UNSIGNED" => MySqlType::Int24Unsigned,
            "MEDIUMINT" => MySqlType::Int24,
            "MEDIUMTEXT" => MySqlType::MediumBlob,
            "NULL" => MySqlType::Null,
            "SET" => MySqlType::Set,
            "SMALLINT UNSIGNED" => MySqlType::ShortUnsigned,
            "SMALLINT" => MySqlType::Short,
            "TEXT" => MySqlType::Blob,
            "TIME" => MySqlType::Time,
            "TIMESTAMP" => MySqlType::Timestamp,
            "TINYBLOB" => MySqlType::TinyBlobBinary,
            "TINYINT UNSIGNED" => MySqlType::TinyUnsigned,
            "TINYINT" => MySqlType::Tiny,
            "TINYTEXT" => MySqlType::TinyBlob,
            "VARBINARY" => MySqlType::VarCharBinary,
            "YEAR" => MySqlType::Year,
            // Tenemos que extraer.
            _ => {
                if s.starts_with("INT") {
                    MySqlType::Long
                } else if s.starts_with("INT UNSIGNED") {
                    MySqlType::LongUnsigned
                } else if s.starts_with("VARCHAR") {
                    MySqlType::VarChar
                } else {
                    MySqlType::Null
                }
            }
        }
    }
}

impl Display for MySqlType {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

pub fn ty_to_type(ty: &MySqlTypeInfo) -> Option<MySqlType> {
    let name = ty.name();
    let t = MySqlType::from_string(name);

    match t {
        MySqlType::Null => None,
        _ => Some(t),
    }
}

impl MySqlType {
    // pub fn to_string(&self) -> String {
    //     self.to_str().to_string()
    // }

    // pub fn to_str(&self) -> &str {
    //     match self {
    //         MySqlType::LongLongUnsigned => "BIGINT UNSIGNED",
    //         MySqlType::LongLong => "BIGINT",
    //         MySqlType::StringBinary => "BINARY",
    //         MySqlType::Bit => "BIT",
    //         MySqlType::BlobBinary => "BLOB",
    //         MySqlType::Boolean => "BOOLEAN",
    //         MySqlType::String => "CHAR",
    //         MySqlType::Date => "DATE",
    //         MySqlType::Datetime => "DATETIME",
    //         MySqlType::Decimal => "DECIMAL",
    //         MySqlType::Double => "DOUBLE",
    //         MySqlType::Enum => "ENUM",
    //         // MySqlType::StringEnum => "ENUM",
    //         MySqlType::Float => "FLOAT",
    //         MySqlType::Geometry => "GEOMETRY",
    //         MySqlType::LongUnsigned => "INT UNSIGNED",
    //         MySqlType::Long => "INT",
    //         MySqlType::Json => "JSON",
    //         MySqlType::LongBlobBinary => "LONGBLOB",
    //         MySqlType::LongBlob => "LONGTEXT",
    //         MySqlType::MediumBlobBinary => "MEDIUMBLOB",
    //         MySqlType::Int24Unsigned => "MEDIUMINT UNSIGNED",
    //         MySqlType::Int24 => "MEDIUMINT",
    //         MySqlType::MediumBlob => "MEDIUMTEXT",
    //         MySqlType::Null => "NULL",
    //         MySqlType::Set => "SET",
    //         MySqlType::ShortUnsigned => "SMALLINT UNSIGNED",
    //         MySqlType::Short => "SMALLINT",
    //         MySqlType::Blob => "TEXT",
    //         MySqlType::Time => "TIME",
    //         MySqlType::Timestamp => "TIMESTAMP",
    //         MySqlType::TinyBlobBinary => "TINYBLOB",
    //         MySqlType::TinyUnsigned => "TINYINT UNSIGNED",
    //         MySqlType::Tiny => "TINYINT",
    //         MySqlType::TinyBlob => "TINYTEXT",
    //         MySqlType::VarCharBinary => "VARBINARY",
    //         MySqlType::VarChar => "VARCHAR",
    //         MySqlType::Year => "YEAR",
    //         _ => "NULL",
    //     }
    // }
}
