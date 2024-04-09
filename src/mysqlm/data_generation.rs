// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------
use chrono::NaiveDateTime;
use chrono::{DateTime, Utc};
use chrono::{NaiveDate, NaiveTime};
use rust_decimal::Decimal;

use super::mysql_type::MySqlType;
use crate::common::generator::{Gen, SimpleRGen};
use crate::common::traits::Runner as _;
use crate::quote;
use crate::sqlx_common::data_generation::GenericGenerator;

// pub fn generate_mysql_value_from_type_info(ty: &MySqlTypeInfo) -> String {
//     ty_to_type(ty).map_or("NULL".to_string(), |t| generate_mysql_value(&t))
// }

// https://docs.rs/sqlx/latest/sqlx/mysql/types/index.html
pub fn generate_mysql_value(data_type: &MySqlType) -> String {
    match data_type {
        MySqlType::Bit => todo!(),
        MySqlType::Blob => todo!(),
        MySqlType::BlobBinary => todo!(),
        MySqlType::Boolean => GenericGenerator::<bool>::run().to_string(),
        MySqlType::Date => quote!(&NaiveDate::default().to_string()),
        MySqlType::Datetime => quote!(&NaiveDateTime::default().to_string()),
        MySqlType::Decimal => decimal_generation(),
        MySqlType::Double => GenericGenerator::<f64>::run().to_string(),
        MySqlType::Enum => todo!(),
        MySqlType::Float => GenericGenerator::<f32>::run().to_string(),
        MySqlType::Geometry => todo!(),
        MySqlType::Int24 => GenericGenerator::<i64>::run().to_string(),
        MySqlType::Int24Unsigned => GenericGenerator::<u64>::run().to_string(),
        MySqlType::Json => todo!(),
        MySqlType::Long => GenericGenerator::<i32>::run().to_string(),
        MySqlType::LongUnsigned => GenericGenerator::<u32>::run().to_string(),
        MySqlType::LongBlob => todo!(),
        MySqlType::LongBlobBinary => todo!(),
        MySqlType::LongLong => GenericGenerator::<i64>::run().to_string(),
        MySqlType::LongLongUnsigned => GenericGenerator::<u64>::run().to_string(),
        MySqlType::MediumBlob => todo!(),
        MySqlType::MediumBlobBinary => todo!(),
        MySqlType::Null => String::from("NULL"),
        MySqlType::Set => todo!(),
        MySqlType::Short => GenericGenerator::<i16>::run().to_string(),
        MySqlType::ShortUnsigned => GenericGenerator::<u16>::run().to_string(),
        MySqlType::String => {
            quote!(&Gen::gen_alpha_lower_with_max_len(20).sample(&SimpleRGen::new()))
        }
        MySqlType::StringBinary => todo!(),
        MySqlType::Time => quote!(&NaiveTime::default().to_string()),
        MySqlType::Timestamp => quote!(&GenericGenerator::<DateTime<Utc>>::run()
            .to_string()
            .strip_suffix(" UTC")
            .unwrap()),
        MySqlType::Tiny => GenericGenerator::<i8>::run().to_string(),
        MySqlType::TinyUnsigned => GenericGenerator::<u8>::run().to_string(),
        MySqlType::TinyBlob => todo!(),
        MySqlType::TinyBlobBinary => todo!(),
        MySqlType::Uuid => {
            quote!(&Gen::gen_random_uuid().sample(&SimpleRGen::new()))
        }
        MySqlType::VarChar => generate_mysql_value(&MySqlType::String),
        MySqlType::VarCharBinary => todo!(),
        MySqlType::Year => todo!(),
    }
}

fn decimal_generation() -> String {
    let integer64: i64 = GenericGenerator::<i64>::run();
    let n_decimals: u32 = GenericGenerator::<u32>::run();
    let decimal = Decimal::new(integer64, n_decimals);

    decimal.to_string()
}
