// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

use super::parser::SqliteType;
use crate::sqlx_module::data_generation::GenericGenerator;
use crate::utils::generator::{Gen, SimpleRGen};
use crate::utils::traits::Runner as _;
use crate::utils::wrap_with_single_quote;

pub trait SQLiteRunner<T> {
    fn run() -> T;
}

// // TODO: Esto iría mejor o bien en `parser.rs`, o bien directamente no existiendo.
// pub fn get_generator(col_type: &str) -> (String, SqliteType) {
//     let data_type = SqliteType::from_string(col_type);

//     match data_type {
//         SqliteType::Null => ("NULL".to_string(), data_type),
//         _ => (data_type.to_string(), data_type),
//     }
// }

pub fn generate_sqlite_value(data_type: &SqliteType) -> String {
    match data_type {
        SqliteType::Bool => GenericGenerator::<bool>::run().to_string(),
        SqliteType::Int | SqliteType::Int64 => GenericGenerator::<i64>::run().to_string(),
        SqliteType::Text => generate_sqlite_value(&SqliteType::Varchar(32)),
        SqliteType::Varchar(n_chars) => wrap_with_single_quote(
            &Gen::gen_alpha_lower_with_max_len(*n_chars).sample(&SimpleRGen::new()),
        ),
        SqliteType::Char(n_chars) => generate_sqlite_value(&SqliteType::Varchar(*n_chars)),
        SqliteType::Float => GenericGenerator::<f64>::run().to_string(),
        SqliteType::Null => "NULL".to_string(),
        // TODO: No tengo nada, es generar Vec<u8> en ppio.
        SqliteType::Blob => todo!(),
        SqliteType::Numeric => generate_sqlite_value(&SqliteType::Float),
        SqliteType::Datetime => wrap_with_single_quote(&NaiveDateTime::default().to_string()),
        SqliteType::Date => wrap_with_single_quote(&NaiveDate::default().to_string()),
        SqliteType::Time => wrap_with_single_quote(&NaiveTime::default().to_string()),
        SqliteType::Int8 => GenericGenerator::<i8>::run().to_string(),
        SqliteType::Int16 => GenericGenerator::<i16>::run().to_string(),
        SqliteType::Int32 => GenericGenerator::<i32>::run().to_string(),
        SqliteType::UInt8 => GenericGenerator::<u8>::run().to_string(),
        SqliteType::UInt64 => GenericGenerator::<u64>::run().to_string(),
    }
}
