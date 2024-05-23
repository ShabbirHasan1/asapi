// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

// Basado en type_info.rs de sqlx, extraigo lo que allí es privado.

use crate::{
    common::traits::ShowVec,
    mysqlm::mysql_type::{ty_to_type, MySqlType},
};
use rust_decimal::Decimal;
use sqlx::{
    mysql::{MySqlColumn, MySqlRow},
    Column, Decode, MySql, Row, Type,
};
use std::fmt;

impl ShowVec for MySqlRow {
    fn to_string_vec(&self) -> Vec<String> {
        self.columns()
            .iter()
            .enumerate()
            .map(|(idx, col)| mysqlrow_value_to_string(self, idx, col))
            .collect()
    }
}

pub fn mysqlrow_value_to_string(row: &MySqlRow, idx: usize, col: &MySqlColumn) -> String {
    let result = row.try_get_raw(idx);
    let mysql_type_opt = ty_to_type(col.type_info());

    if result.is_err() || mysql_type_opt.is_none() {
        return "NULL".to_owned();
    }

    // println!("{} : {}", col.name(), col.type_info());

    let mysql_type = mysql_type_opt.as_ref().unwrap();

    // https://docs.rs/sqlx/latest/sqlx/mysql/types/index.html
    match mysql_type {
        MySqlType::Bit => value_to_string::<String>(row, idx),
        MySqlType::Blob => value_vecu8_to_utf8_string(row, idx),
        MySqlType::BlobBinary => value_vecu8_to_utf8_string(row, idx),
        MySqlType::Boolean => value_to_string::<bool>(row, idx),
        MySqlType::Date => value_to_string::<chrono::NaiveDate>(row, idx),
        MySqlType::Datetime => value_to_string::<chrono::NaiveDateTime>(row, idx),
        MySqlType::Decimal => value_to_string::<Decimal>(row, idx),
        MySqlType::Double => value_to_string::<f64>(row, idx),
        MySqlType::Float => value_to_string::<f32>(row, idx),
        MySqlType::Int24 => value_to_string::<i64>(row, idx),
        MySqlType::Int24Unsigned => value_to_string::<u64>(row, idx),
        // MySqlType::Json => value_vecu8_to_utf8_string(row, idx),
        MySqlType::Long => value_to_string::<i32>(row, idx),
        MySqlType::LongUnsigned => value_to_string::<u32>(row, idx),
        MySqlType::LongBlob => value_vecu8_to_utf8_string(row, idx),
        MySqlType::LongBlobBinary => value_vecu8_to_utf8_string(row, idx),
        MySqlType::LongLong => value_to_string::<i64>(row, idx),
        MySqlType::LongLongUnsigned => value_to_string::<u64>(row, idx),
        MySqlType::MediumBlob => value_vecu8_to_utf8_string(row, idx),
        MySqlType::MediumBlobBinary => value_vecu8_to_utf8_string(row, idx),
        MySqlType::Null => String::from("NULL"),
        MySqlType::Short => value_to_string::<i16>(row, idx),
        MySqlType::ShortUnsigned => value_to_string::<u16>(row, idx),
        MySqlType::String => value_to_string::<String>(row, idx),
        MySqlType::Time => value_to_string::<chrono::NaiveTime>(row, idx),
        MySqlType::Timestamp => value_to_string::<chrono::DateTime<chrono::Utc>>(row, idx)
            .to_string()
            .strip_suffix(" UTC")
            .unwrap()
            .to_owned(),
        MySqlType::Tiny => value_to_string::<i8>(row, idx),
        MySqlType::TinyUnsigned => value_to_string::<u8>(row, idx),
        MySqlType::TinyBlob => value_vecu8_to_utf8_string(row, idx),
        MySqlType::TinyBlobBinary => value_vecu8_to_utf8_string(row, idx),
        MySqlType::Uuid => value_to_string::<String>(row, idx),
        MySqlType::VarChar => value_to_string::<String>(row, idx),
        MySqlType::VarCharBinary => value_vecu8_to_utf8_string(row, idx),
        // TODO
        MySqlType::Binary => value_vecu8_to_utf8_string(row, idx),
        // Estos cinco están en `ColumnType` de sqlx.
        MySqlType::Enum => value_to_string::<String>(row, idx),
        MySqlType::Year => value_to_string::<u16>(row, idx),
        MySqlType::Geometry => value_vecu8_to_utf8_string(row, idx),
        // Estos me salen como MySqlType::Text y MySqlType::Char, realmente aquí no llegamos nunca
        //   - fecha   24/05/23
        //   - versión    0.7.4
        MySqlType::Json => String::default(),
        MySqlType::Set => String::default(),
    }
    // IpAddr	VARCHAR, TEXT
    // Ipv4Addr	INET4 (MariaDB-only), VARCHAR, TEXT
    // Ipv6Addr	INET6 (MariaDB-only), VARCHAR, TEXT
    // uuid::Uuid	BINARY(16), VARCHAR, CHAR, TEXT
    // uuid::fmt::Hyphenated	CHAR(36), UUID (MariaDB-only)
    // uuid::fmt::Simple	CHAR(32)
    // Json<T>	JSON
    // serde_json::JsonValue	JSON
    // &serde_json::value::RawValue	JSON
}

fn value_to_string<'r, T>(row: &'r MySqlRow, idx: usize) -> String
where
    T: Decode<'r, MySql> + Type<MySql> + fmt::Display,
{
    // Option para poder representar columnas NULLABLE
    match row.try_get::<Option<T>, usize>(idx) {
        Ok(v) => v.map_or("NULL".to_string(), |v| format!("{}", v)),
        Err(_err) => String::from("ERR parsing"),
    }
}

fn value_vecu8_to_utf8_string(row: &MySqlRow, idx: usize) -> String {
    // Option para poder representar columnas NULLABLE
    match row.try_get::<Option<Vec<u8>>, usize>(idx) {
        Ok(v) => v.map_or("NULL".to_string(), |v| {
            String::from_utf8(v).map_or(String::from("ERR parsing Vec<u8>"), |v| v)
        }),
        Err(_err) => String::from("ERR parsing Vec<u8>"),
    }
}

// fn value_to_string_debug<'r, T>(row: &'r MySqlRow, idx: usize) -> String
// where
//     T: Decode<'r, MySql> + Type<MySql> + fmt::Debug,
// {
//     // Option para poder representar columnas NULLABLE
//     match row.try_get::<Option<T>, usize>(idx) {
//         Ok(v) => v.map_or("NULL".to_string(), |v| format!("{:?}", v)),
//         Err(_err) => String::from("ERR parsing"),
//     }
// }

pub fn mysql_type_from_string(s: &str) -> MySqlType {
    MySqlType::from_column_type(s.to_ascii_uppercase().as_str())
}
