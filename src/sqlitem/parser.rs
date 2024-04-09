// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use regex::Regex;
use sqlx::sqlite::{SqliteColumn, SqliteRow};
use sqlx::{Column, Decode, Row, Sqlite, Type, TypeInfo};
use std::fmt;

use crate::common::traits::ShowVec;

// ==============================================================
// INFORMACIÓN PRELIMINAR
// Enlaces necesarios para saber qué ocurre/cómo funciona SQLite
//    https://sqlite.org/datatype3.html
//    https://sqlite.org/flextypegood.html
//
// Column Datatype	| Types Allowed In That Column
//   INTEGER       	|   INTEGER, REAL, TEXT, BLOB
//   REAL          	|   REAL, TEXT, BLOB
//   TEXT	          |   TEXT, BLOB
//   BLOB	          |   INTEGER, REAL, TEXT, BLOB
//
// Cada valor almacenado en una base de datos de SQLite (o manipulado por el engine)
// tiene una de las siguientes `storage class`:
//
//              NULL    INTEGER    REAL    TEXT    BLOB
//
// Estas `storage class` son más generales que los `datatypes`. Por ejemplo a INTEGER
// le corresponden 7 `datatypes`.
// No está muy claro qué me devuelve qué. `PRAGMA table_info` me devuelve varchar y demás,
// y la llamada a `type_info` en `row.column` una mezcla de `storage class`  y `datatypes`
// igual que PRAGMA, pero no tienen porqué coincidir.
//
// ==============================================================

/// DataType es público solo dentro del crate. Define lo siguiente:
// pub struct SqliteTypeInfo(pub(crate) DataType);

// Lo que vamos a hacer es extraer la inversa, redefino esto aquí y a partir del nombre, que sí
// lo puedo obtener, extraemos el tipo.
// Esto nos servirá para generar valores, representar, etc.

// Es muy importante diferenciar entre el tipo de verdad y el tipo con afinidad
// (`affinity type`). Cada columna tiene un tipo, que define por ejemplo su capacidad
// y tiene que ser lo que usemos para generar los valores.
// Es algo único (al menos que yo sepa) en las bbdd SQL.
// Aquí el tipo de datos está asociado con el valor, no con el contenedor (columna) en que
// se encuentra como el resto de motores SQL.

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
pub(crate) enum SqliteType {
    Null,
    Int,
    Float,
    Text,
    Blob,
    Numeric,
    Char(usize), // Aparece en una tabla de wagtail para uuid: wagtailcore_modellogentry
    Varchar(usize),
    // non-standard extensions
    Bool,
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt64,
    Date,
    Time,
    Datetime,
    // TODO: Extraer smallint unsigned como algo que no sé qué puede ser.
}

impl fmt::Display for SqliteType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SqliteType::Null => write!(f, "NULL"),
            SqliteType::Int | SqliteType::Int64 => write!(f, "INTEGER"),
            SqliteType::Float => write!(f, "REAL"),
            SqliteType::Text => write!(f, "TEXT"),
            SqliteType::Blob => write!(f, "BLOB"),
            SqliteType::Numeric => write!(f, "NUMERIC"),
            SqliteType::Bool => write!(f, "BOOLEAN"),
            SqliteType::Date => write!(f, "DATE"),
            SqliteType::Time => write!(f, "TIME"),
            SqliteType::Datetime => write!(f, "DATETIME"),
            SqliteType::Char(n_chars) => write!(f, "CHAR({n_chars})"),
            SqliteType::Varchar(n_chars) => write!(f, "VARCHAR({n_chars})"),
            SqliteType::Int8 => write!(f, "TINYINT"),
            SqliteType::Int16 => write!(f, "SMALLINT"),
            SqliteType::Int32 => write!(f, "MEDIUMINT"),
            SqliteType::UInt8 => write!(f, "SMALLINT UNSIGNED"),
            SqliteType::UInt64 => write!(f, "INT8"),
        }
    }
}

impl SqliteType {
    // Implementación liosa pero intento que sea la más eficiente
    pub fn from_string(name: &str) -> SqliteType {
        let name_uppercased = name.to_uppercase();

        // --> 1. Casos mayoritarios/menos costosos de comprobar <--
        match name_uppercased.as_ref() {
            "NULL" => SqliteType::Null,
            "TEXT" => SqliteType::Text,
            "REAL" => SqliteType::Float,
            "BLOB" => SqliteType::Blob,
            "INTEGER" => SqliteType::Int,
            "NUMERIC" => SqliteType::Numeric,
            "BOOL" | "BOOLEAN" => SqliteType::Bool,
            "DATE" => SqliteType::Date,
            "TIME" => SqliteType::Time,
            "DATETIME" => SqliteType::Datetime,
            _ => {
                let re_varchar = Regex::new(r"(?i)VARCHAR\((\d+)\)").unwrap();
                if let Some(caps) = re_varchar.captures(&name_uppercased) {
                    return match caps.get(1).and_then(|v| v.as_str().parse::<usize>().ok()) {
                        Some(v) => SqliteType::Varchar(v),
                        None => SqliteType::Text, // Match varchar y no sabemos cúal : TEXT
                    };
                }

                let re_char = Regex::new(r"(?i)CHAR\((\d+)\)").unwrap();
                if let Some(caps) = re_char.captures(&name_uppercased) {
                    return match caps.get(1).and_then(|v| v.as_str().parse::<usize>().ok()) {
                        Some(v) => SqliteType::Char(v),
                        None => SqliteType::Text, // Match varchar y no sabemos cúal : TEXT
                    };
                }

                let re_integer = Regex::new(r"(?i)(.+int.+)").unwrap();
                if let Some(caps) = re_integer.captures(&name_uppercased) {
                    return match caps.get(1).map(|m| m.as_str()) {
                        Some(extracted_str) => {
                            if extracted_str == "INTEGER"
                                || extracted_str == "INT8"
                                || extracted_str == "INT"
                                || extracted_str == "BIGINT"
                            {
                                SqliteType::Int64
                            } else if extracted_str == "TINYINT" {
                                SqliteType::Int8
                            } else if extracted_str.starts_with("SMALLINT") {
                                // por si acaso, como he visto smallint unsigned, siempre que empiece
                                // por smallint genero uint8 (no uint16 porque en smallint a secas
                                // no cabría)
                                SqliteType::UInt8
                            } else if extracted_str == "INT2" {
                                SqliteType::Int16
                            } else if extracted_str == "MEDIUMINT" {
                                SqliteType::Int32
                            } else if extracted_str == "UNSIGNED BIG INT" {
                                return SqliteType::UInt64;
                            } else {
                                SqliteType::Int64
                            }
                        }
                        // Si match varchar pero no puedo parsear por lo que sea, por defecto INTEGER
                        None => SqliteType::Int,
                    };
                }

                // Fallback. Prefiero Text porque casi todo puede acabar siendo TEXT.
                SqliteType::Null
            }
        }
    }
}

impl ShowVec for SqliteRow {
    fn to_string_vec(&self) -> Vec<String> {
        self.columns()
            .iter()
            .map(|col| sqlite_value_to_string(self, col))
            .collect()
    }
}

pub fn sqlite_value_to_string<'a>(row: &SqliteRow, col: &SqliteColumn) -> String {
    let data_type = SqliteType::from_string(col.type_info().name());
    if data_type == SqliteType::Null {
        return "NULL".to_string();
    }

    // Mejor usar `col.ordinal()` que `col.name()` porque al final este último recae en aquél
    // en la implementación dentro de sqlx.
    let col_idx = col.ordinal();
    let repr = match data_type {
        SqliteType::Int | SqliteType::Int64 => row_value_to_string::<i64>(&row, col_idx),
        SqliteType::Text | SqliteType::Varchar(_) => row_value_to_string::<String>(&row, col_idx),
        SqliteType::Float => row_value_to_string::<f64>(&row, col_idx),
        SqliteType::Blob => row
            .try_get::<Vec<u8>, usize>(col_idx)
            .map_or("NULL".to_string(), |v| {
                String::from_utf8(v).unwrap_or_default()
            }),
        // Según especificación
        //   https://sqlite.org/datatype3.html
        // Numeric puede ser cualquier cosa prácticamente, pero primero se intenta ver
        // si es un entero válido y sino un real válido.
        // Si no ya pasa a intentarse como String y si no ya doy error.
        // Esto con un flatMapErr o similar sería mucho más fácil de leer.
        SqliteType::Numeric => row.try_get::<i64, usize>(col_idx).map_or_else(
            |v| format!("{}", v),
            |_e| {
                row.try_get::<f64, usize>(col_idx).map_or_else(
                    |v| format!("{}", v),
                    |_e| {
                        row.try_get::<String, usize>(col_idx)
                            .map_or("NULL".to_string(), |v| v)
                    },
                )
            },
        ),

        SqliteType::Bool => row_value_to_string::<bool>(&row, col_idx),
        SqliteType::Date => row_value_to_string::<chrono::NaiveDate>(&row, col_idx),
        SqliteType::Time => row_value_to_string::<chrono::NaiveTime>(&row, col_idx),
        SqliteType::Datetime => row_value_to_string::<chrono::DateTime<chrono::Utc>>(&row, col_idx),
        _ => row_value_to_string::<String>(&row, col_idx),
    };

    repr
}

fn row_value_to_string<'r, T>(row: &'r SqliteRow, idx: usize) -> String
where
    T: Decode<'r, Sqlite> + Type<Sqlite> + fmt::Display,
{
    row.try_get::<T, usize>(idx)
        .map_or("NULL".to_string(), |v| format!("{}", v))
}
