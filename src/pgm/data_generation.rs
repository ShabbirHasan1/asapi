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

use super::pg_type::PgType;
use crate::common::generator::{Gen, SimpleRGen};
use crate::common::traits::Runner as _;
use crate::sqlx_common::data_generation::GenericGenerator;

pub fn generate_pg_value(data_type: &PgType) -> String {
    match data_type {
        PgType::Bool => GenericGenerator::<bool>::run().to_string(),
        // PgType::Int | PgType::Int64 => GenericGenerator::<i64>::run().to_string(),
        // PgType::Text => generate_pg_value(&PgType::Varchar(32)),
        // PgType::Varchar(n_chars) => quote!(
        //     &Gen::gen_alpha_lower_with_max_len(*n_chars).sample(&SimpleRGen::new()),
        // ),
        // PgType::Char(n_chars) => generate_pg_value(&PgType::Varchar(*n_chars)),
        // PgType::Float => GenericGenerator::<f64>::run().to_string(),
        // PgType::Null => "NULL".to_string(),
        // TODO: No tengo nada, es generar Vec<u8> en ppio.
        // PgType::Blob => todo!(),
        // PgType::Numeric => generate_pg_value(&PgType::Float),
        // PgType::Datetime => quote!(&NaiveDateTime::default().to_string()),
        PgType::Date => format!("'{}'", &NaiveDate::default().to_string()),
        PgType::Time => format!("'{}'", &NaiveTime::default().to_string()),
        PgType::Timestamp => format!("'{}'", &NaiveDateTime::default().to_string()),
        PgType::Timestamptz => format!("'{}'", &DateTime::<Utc>::default().to_rfc2822()),
        PgType::Int2 => GenericGenerator::<i16>::run().to_string(),
        PgType::Int4 => GenericGenerator::<i32>::run().to_string(),
        PgType::Int8 => GenericGenerator::<i64>::run().to_string(),
        PgType::Float4 => GenericGenerator::<f32>::run().to_string(),
        PgType::Float8 => GenericGenerator::<f64>::run().to_string(),
        PgType::Char => format!(
            "'{}'",
            Gen::gen_alpha_lower_with_len(1)
                .sample(&SimpleRGen::new())
                .chars()
                .next()
                .unwrap_or_default()
        ),
        PgType::Text => format!(
            "'{}'",
            &Gen::gen_alpha_lower_with_max_len(20).sample(&SimpleRGen::new()),
        ),
        PgType::Uuid => format!("'{}'", &Gen::gen_random_uuid().sample(&SimpleRGen::new())),
        PgType::Varchar => generate_pg_value(&PgType::Text),
        PgType::Int2Array => todo!(),
        PgType::Int4Array => todo!(),
        PgType::Float4Array => todo!(),
        PgType::Float8Array => todo!(),
        PgType::BoolArray => todo!(),
        PgType::TimetzArray => todo!(),
        PgType::UuidArray => todo!(),
        PgType::TimestampArray => todo!(),
        PgType::DateArray => todo!(),
        PgType::TimeArray => todo!(),

        // TODO:
        // Para ir implementando
        PgType::Bytea => todo!(),
        PgType::Name => todo!(),
        PgType::Oid => todo!(),
        PgType::Json => todo!(),
        PgType::JsonArray => todo!(),
        PgType::Point => todo!(),
        PgType::Lseg => todo!(),
        PgType::Path => todo!(),
        PgType::Box => todo!(),
        PgType::Polygon => todo!(),
        PgType::Line => todo!(),
        PgType::LineArray => todo!(),
        PgType::Cidr => todo!(),
        PgType::CidrArray => todo!(),
        PgType::Unknown => todo!(),
        PgType::Circle => todo!(),
        PgType::CircleArray => todo!(),
        PgType::Macaddr8 => todo!(),
        PgType::Macaddr8Array => todo!(),
        PgType::Macaddr => todo!(),
        PgType::Inet => todo!(),
        PgType::ByteaArray => todo!(),
        PgType::CharArray => todo!(),
        PgType::NameArray => todo!(),
        PgType::TextArray => todo!(),
        PgType::BpcharArray => todo!(),
        PgType::VarcharArray => todo!(),
        PgType::Int8Array => todo!(),
        PgType::PointArray => todo!(),
        PgType::LsegArray => todo!(),
        PgType::PathArray => todo!(),
        PgType::BoxArray => todo!(),
        PgType::PolygonArray => todo!(),
        PgType::OidArray => todo!(),
        PgType::MacaddrArray => todo!(),
        PgType::InetArray => todo!(),
        PgType::Bpchar => todo!(),
        PgType::TimestamptzArray => todo!(),
        PgType::Interval => todo!(),
        PgType::IntervalArray => todo!(),
        PgType::NumericArray => todo!(),
        PgType::Timetz => todo!(),
        PgType::Bit => todo!(),
        PgType::BitArray => todo!(),
        PgType::Varbit => todo!(),
        PgType::VarbitArray => todo!(),
        PgType::Record => todo!(),
        PgType::RecordArray => todo!(),
        PgType::Jsonb => todo!(),
        PgType::JsonbArray => todo!(),
        PgType::Int4Range => todo!(),
        PgType::Int4RangeArray => todo!(),
        PgType::NumRange => todo!(),
        PgType::NumRangeArray => todo!(),
        PgType::TsRange => todo!(),
        PgType::TsRangeArray => todo!(),
        PgType::TstzRange => todo!(),
        PgType::TstzRangeArray => todo!(),
        PgType::DateRange => todo!(),
        PgType::DateRangeArray => todo!(),
        PgType::Int8Range => todo!(),
        PgType::Int8RangeArray => todo!(),
        PgType::Jsonpath => todo!(),
        PgType::JsonpathArray => todo!(),
        PgType::Money => todo!(),
        PgType::MoneyArray => todo!(),
        PgType::Void => todo!(),
        PgType::Custom(_) => todo!(),
        // PgType::DeclareWithName(_) => todo!(),
        // PgType::DeclareWithOid(_) => todo!(),
        PgType::Numeric => todo!(),
        // PgType::Int16 => GenericGenerator::<i16>::run().to_string(),
        // PgType::Int32 => GenericGenerator::<i32>::run().to_string(),
        // PgType::UInt8 => GenericGenerator::<u8>::run().to_string(),
        // PgType::UInt64 => GenericGenerator::<u64>::run().to_string(),
    }
}
