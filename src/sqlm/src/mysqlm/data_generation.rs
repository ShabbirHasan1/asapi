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

use common::generator::{random_select_from_pair, random_select_from_vec, Gen, SimpleRGen};
use common::quote;
use common::traits::Runner as _;

use crate::sqlx_common::data_generation::geom::{
    linestring_generator, multipoint_generator, multipolygon_generator, point_generator,
    polygon_generator,
};
use crate::sqlx_common::data_generation::json::simple_json_generator;
use crate::sqlx_common::data_generation::GenericGenerator;

use super::mysql_type::MySqlType;

// https://docs.rs/sqlx/latest/sqlx/mysql/types/index.html
pub fn generate_mysql_value(data_type: &MySqlType) -> String {
    match data_type {
        MySqlType::Boolean => GenericGenerator::<bool>::run().to_string(),
        MySqlType::Date => quote!(&NaiveDate::default().to_string()),
        MySqlType::Datetime => quote!(&NaiveDateTime::default().to_string()),
        MySqlType::Decimal => decimal_generation(),
        MySqlType::Double => GenericGenerator::<f64>::run().to_string(),
        MySqlType::Float => GenericGenerator::<f32>::run().to_string(),
        MySqlType::Int24 => GenericGenerator::<i64>::run().to_string(),
        MySqlType::Int24Unsigned => GenericGenerator::<u64>::run().to_string(),
        MySqlType::Long => GenericGenerator::<i32>::run().to_string(),
        MySqlType::LongUnsigned => GenericGenerator::<u32>::run().to_string(),
        MySqlType::LongLong => GenericGenerator::<i64>::run().to_string(),
        MySqlType::LongLongUnsigned => GenericGenerator::<u64>::run().to_string(),
        MySqlType::Null => String::from("NULL"),
        MySqlType::Short => GenericGenerator::<i16>::run().to_string(),
        MySqlType::ShortUnsigned => GenericGenerator::<u16>::run().to_string(),
        MySqlType::String => {
            quote!(&Gen::gen_alpha_lower_with_max_len(20).sample(&SimpleRGen::new()))
        }
        MySqlType::Time => quote!(&NaiveTime::default().to_string()),
        MySqlType::Timestamp => quote!(&GenericGenerator::<DateTime<Utc>>::run()
            .to_string()
            .strip_suffix(" UTC")
            .unwrap()),
        MySqlType::Tiny => GenericGenerator::<i8>::run().to_string(),
        MySqlType::TinyUnsigned => GenericGenerator::<u8>::run().to_string(),
        MySqlType::Uuid => {
            quote!(&Gen::gen_random_uuid().sample(&SimpleRGen::new()))
        }
        MySqlType::VarChar => generate_mysql_value(&MySqlType::String),
        MySqlType::Binary(len) => {
            quote!(Gen::gen_alpha_lower_with_max_len(*len as usize).sample(&SimpleRGen::new()))
        }
        MySqlType::VarBinary(len) => generate_mysql_value(&MySqlType::Binary(*len)),
        // TODO:
        MySqlType::Year => quote!(Gen::gen_in_range(1901, 2156).sample(&SimpleRGen::new())),
        MySqlType::Bit(len) => {
            let mut bits = String::with_capacity(*len);
            let mut s = SimpleRGen::new();

            for _ in 0..*len {
                let (b, _s) = random_select_from_pair(('0', '1')).run(&s);
                bits.push(b);
                s = _s;
            }

            format!("b{}", quote!(bits))
        }
        MySqlType::Blob(len) => generate_mysql_value(&MySqlType::Text(*len)),
        MySqlType::Text(len) => {
            quote!(Gen::gen_alpha_lower_with_max_len(*len as usize).sample(&SimpleRGen::new()))
        }
        // No uso la longitud máximo y pongo el número de bits porque se me va de madre.
        // MySqlType::LongBlob => generate_mysql_value(&MySqlType::Blob(2_u32.pow(32) - 1)),
        MySqlType::MediumBlob => generate_mysql_value(&MySqlType::Blob(24)),
        MySqlType::MediumText => generate_mysql_value(&MySqlType::Text(24)),
        MySqlType::LongBlob => generate_mysql_value(&MySqlType::Blob(32)),
        MySqlType::LongText => generate_mysql_value(&MySqlType::Text(32)),
        MySqlType::TinyBlob => generate_mysql_value(&MySqlType::Blob(8)),
        MySqlType::TinyText => generate_mysql_value(&MySqlType::Text(8)),
        // TODO:
        MySqlType::Set(s) => {
            let options = s.split(',').collect::<Vec<&str>>();
            // TODO: Podemos hacer que se seleccionen `n` elementos, con 0 < n < options.len()
            let selected = random_select_from_vec(options).sample(&SimpleRGen::new());

            selected.to_owned()
        }
        MySqlType::Enum(s) => {
            let options = s.split(',').collect::<Vec<&str>>();
            let selected = random_select_from_vec(options).sample(&SimpleRGen::new());

            selected.to_owned()
        }
        MySqlType::Geometry => {
            let (i, s) = Gen::gen_in_range(0, 5).run(&SimpleRGen::new());
            let geom = if i == 0 {
                point_generator().sample(&s)
            } else if i == 1 {
                linestring_generator(5).sample(&s)
            } else if i == 2 {
                polygon_generator(5).sample(&s)
            } else if i == 3 {
                multipoint_generator(5).sample(&s)
            } else if i == 4 {
                multipolygon_generator(5).sample(&s)
            } else {
                point_generator().sample(&s)
            };

            format!("ST_GeomFromText('{geom}')")
        }
        MySqlType::Json => quote!(simple_json_generator().sample(&SimpleRGen::new())),
    }
}

fn decimal_generation() -> String {
    let integer64: i64 = GenericGenerator::<i64>::run();
    let n_decimals: u32 = GenericGenerator::<u32>::run();
    let decimal = Decimal::new(integer64, n_decimals);

    decimal.to_string()
}
