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
    Binary,
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
    // StringBinary, // Lo creo yo -> Binary que es lo que hay en la librería, no sé por qué creé este.
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
            "BINARY" => MySqlType::Binary,
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
            "BINARY" => MySqlType::Binary,
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

// Los `ty.name()` se extraen a partir del `ColumnType` de la feature `mysql` del crate `sqlx`:
// pub enum ColumnType {
//     Decimal = 0x00,
//     Tiny = 0x01,
//     Short = 0x02,
//     Long = 0x03,
//     Float = 0x04,
//     Double = 0x05,
//     Null = 0x06,
//     Timestamp = 0x07,
//     LongLong = 0x08,
//     Int24 = 0x09,
//     Date = 0x0a,
//     Time = 0x0b,
//     Datetime = 0x0c,
//     Year = 0x0d,
//     VarChar = 0x0f,
//     Bit = 0x10,
//     Json = 0xf5,
//     NewDecimal = 0xf6,
//     Enum = 0xf7,
//     Set = 0xf8,
//     TinyBlob = 0xf9,
//     MediumBlob = 0xfa,
//     LongBlob = 0xfb,
//     Blob = 0xfc,
//     VarString = 0xfd,
//     String = 0xfe,
//     Geometry = 0xff,
// }

pub fn ty_to_type(ty: &MySqlTypeInfo) -> Option<MySqlType> {
    let t = MySqlType::from_string(ty.name());

    match t {
        MySqlType::Null => None,
        _ => Some(t),
    }
}
