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

use std::any::Any;

use regex::Regex;
use sqlx::{mysql::MySqlTypeInfo, TypeInfo};

// Tipos copiados de la librería y algunos añadidos para no tener que usar flags y extraer con regexp.
#[derive(Debug)]
#[repr(u32)]
pub enum MySqlType {
    Bit(usize),
    Binary(u32),
    // Blob
    Blob(u32),
    TinyBlob,
    MediumBlob,
    LongBlob,
    // Text
    Text(u32),
    TinyText,
    MediumText,
    LongText,

    Boolean, // Lo creo yo.
    Date,
    Datetime,
    Decimal,
    Double,
    Enum(String),
    Float,
    Geometry,
    Int24,
    Int24Unsigned, // Lo creo yo.
    Json,
    Long,
    LongUnsigned, // Lo creo yo.

    LongLong,
    LongLongUnsigned, // Lo creo yo.

    // NewDecimal, // Comento porque no sé cómo extraerla, se representa igual que `Decimal`.
    Null,
    Set(String),
    Short,
    ShortUnsigned, // Lo creo yo.
    String,
    // StringBinary, // Lo creo yo -> Binary que es lo que hay en la librería, no sé por qué creé este.
    // StringEnum,   // Lo creo yo. // comento porque no sé cómo extraer, se representa igual que enum
    Time,
    Timestamp,
    Tiny,
    TinyUnsigned, // Lo creo yo.

    VarChar,
    VarBinary(u32), // Lo creo yo. Existe en mysql pero no en MySqlTypeColumn o como se llame en sqlx.
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
            // Esto, que es un poco locura, nos da igual en realidad porque este tipo lo gastamos solamente
            // para representar el valor de lo que hay en la base de datos, no para generar valores.
            "BINARY" => MySqlType::Binary(u32::MAX),
            "BIT" => MySqlType::Bit(usize::MAX),
            "BLOB" => MySqlType::Blob(u32::MAX),
            "BOOLEAN" => MySqlType::Boolean,
            "CHAR" => MySqlType::String,
            "DATE" => MySqlType::Date,
            "DATETIME" => MySqlType::Datetime,
            "DECIMAL" => MySqlType::Decimal,
            "DOUBLE" => MySqlType::Double,
            "ENUM" => MySqlType::Enum(String::new()),
            "FLOAT" => MySqlType::Float,
            "GEOMETRY" => MySqlType::Geometry,
            "INT UNSIGNED" => MySqlType::LongUnsigned,
            "INT" => MySqlType::Long,
            "JSON" => MySqlType::Json,
            "LONGBLOB" => MySqlType::LongBlob,
            "LONGTEXT" => MySqlType::LongText,
            "MEDIUMBLOB" => MySqlType::MediumBlob,
            "MEDIUMINT UNSIGNED" => MySqlType::Int24Unsigned,
            "MEDIUMINT" => MySqlType::Int24,
            "MEDIUMTEXT" => MySqlType::MediumText,
            "NULL" => MySqlType::Null,
            "SET" => MySqlType::Set(String::new()),
            "SMALLINT UNSIGNED" => MySqlType::ShortUnsigned,
            "SMALLINT" => MySqlType::Short,
            "TEXT" => MySqlType::Text(u32::MAX),
            "TIME" => MySqlType::Time,
            "TIMESTAMP" => MySqlType::Timestamp,
            "TINYBLOB" => MySqlType::TinyBlob,
            "TINYINT UNSIGNED" => MySqlType::TinyUnsigned,
            "TINYINT" => MySqlType::Tiny,
            "TINYTEXT" => MySqlType::TinyText,
            "VARBINARY" => MySqlType::VarBinary(u32::MAX),
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
            // "BIT" => MySqlType::Bit,
            "BOOLEAN" => MySqlType::Boolean,
            "CHAR" => MySqlType::String,
            "DATE" => MySqlType::Date,
            "DATETIME" => MySqlType::Datetime,
            "DECIMAL" => MySqlType::Decimal,
            "DOUBLE" => MySqlType::Double,
            // "ENUM" => MySqlType::Enum,
            // "ENUM" => MySqlType::StringEnum,
            "FLOAT" => MySqlType::Float,
            "GEOMETRY" => MySqlType::Geometry,
            "JSON" => MySqlType::Json,
            "LONGTEXT" => MySqlType::LongText,
            "MEDIUMINT UNSIGNED" => MySqlType::Int24Unsigned,
            "MEDIUMINT" => MySqlType::Int24,
            "MEDIUMTEXT" => MySqlType::MediumText,
            "NULL" => MySqlType::Null,
            // "SET" => MySqlType::Set,
            "SMALLINT UNSIGNED" => MySqlType::ShortUnsigned,
            "SMALLINT" => MySqlType::Short,
            "TEXT" => MySqlType::Text(65355), // máximo valor si no se especifica
            "TIME" => MySqlType::Time,
            "TIMESTAMP" => MySqlType::Timestamp,
            "BLOB" => MySqlType::Blob(65355), // máximo valor si no se especifica
            "LONGBLOB" => MySqlType::LongBlob,
            "MEDIUMBLOB" => MySqlType::MediumBlob,
            "TINYBLOB" => MySqlType::TinyBlob,
            "TINYINT UNSIGNED" => MySqlType::TinyUnsigned,
            "TINYINT" => MySqlType::Tiny,
            "TINYTEXT" => MySqlType::TinyText,
            "YEAR" => MySqlType::Year,
            // Tenemos que extraer.
            _ => {
                let re_enum = Regex::new(r"(?i)ENUM\((.*?)\)").unwrap();
                if let Some(caps) = re_enum.captures(s) {
                    return match caps.get(1).map(|v| v.as_str()) {
                        Some(v) => MySqlType::Enum(v.to_string()),
                        None => MySqlType::String, // Match Binary y no sabemos cúal : STRING
                    };
                }

                let re_set = Regex::new(r"(?i)SET\((.*?)\)").unwrap();
                if let Some(caps) = re_set.captures(s) {
                    return match caps.get(1).map(|v| v.as_str()) {
                        Some(v) => MySqlType::Enum(v.to_string()),
                        None => MySqlType::String, // Match Binary y no sabemos cúal : STRING
                    };
                }

                let re_binary = Regex::new(r"(?i)BINARY\((\d+)\)").unwrap();
                if let Some(caps) = re_binary.captures(s) {
                    return match caps.get(1).and_then(|v| v.as_str().parse::<u32>().ok()) {
                        Some(v) => MySqlType::Binary(v),
                        None => MySqlType::String, // Match Binary y no sabemos cúal : STRING
                    };
                }

                let re_text = Regex::new(r"(?i)TEXT\((\d+)\)").unwrap();
                if let Some(caps) = re_text.captures(s) {
                    return match caps.get(1).and_then(|v| v.as_str().parse::<u32>().ok()) {
                        Some(v) => MySqlType::Text(v),
                        None => MySqlType::String, // Match Binary y no sabemos cúal : STRING
                    };
                }

                let re_varbinary = Regex::new(r"(?i)VARBINARY\((\d+)\)").unwrap();
                if let Some(caps) = re_varbinary.captures(s) {
                    return match caps.get(1).and_then(|v| v.as_str().parse::<u32>().ok()) {
                        Some(v) => MySqlType::VarBinary(v),
                        None => MySqlType::String, // Match Binary y no sabemos cúal : STRING
                    };
                }

                let re_bit = Regex::new(r"(?i)BIT\((\d+)\)").unwrap();
                if let Some(caps) = re_bit.captures(s) {
                    return match caps.get(1).and_then(|v| v.as_str().parse::<usize>().ok()) {
                        Some(v) => MySqlType::Bit(v),
                        None => MySqlType::String, // Match Binary y no sabemos cúal : STRING
                    };
                }

                let re_year = Regex::new(r"(?i)YEAR\((\d+)\)").unwrap();
                if let Some(caps) = re_year.captures(s) {
                    return match caps.get(1).and_then(|v| v.as_str().parse::<usize>().ok()) {
                        Some(_) => MySqlType::Year, // No usamos ancho, deprecated, siempre 4: https://dev.mysql.com/doc/refman/8.0/en/year.html
                        None => MySqlType::String,
                    };
                }

                // Blob puede no tener (entonces len = 2^16 - 1) o tener y ajustar al blob de menor tamaño que ajuste al valor que pasemos
                // https://mariadb.com/kb/en/blob/
                let re_blob = Regex::new(r"(?i)BLOB\((\d+)\)").unwrap();
                if let Some(caps) = re_blob.captures(s) {
                    return match caps.get(1).and_then(|v| v.as_str().parse::<u32>().ok()) {
                        Some(v) => MySqlType::Blob(v),
                        None => MySqlType::String, // Match Binary y no sabemos cúal : STRING
                    };
                }

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

    println!(
        "\n\nid: {:?}\nname: {}\n{t:?}\n{ty:?}\n\n",
        ty.type_id(),
        ty.name()
    );

    match t {
        MySqlType::Null => None,
        _ => Some(t),
    }
}
