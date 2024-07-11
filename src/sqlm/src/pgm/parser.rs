// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

// Basado en type_info.rs de sqlx, extraigo lo que allí es privado.

use sqlx::postgres::{PgColumn, PgRow};
use sqlx::{Column, Decode, Postgres, Row, Type};
use std::fmt;

use crate::sqlx_common::traits::ShowVec;
use crate::pgm::pg_type::PgType;

use super::pg_type::ty_to_type;

impl ShowVec for PgRow {
    fn to_string_vec(&self) -> Vec<String> {
        self.columns()
            .iter()
            .enumerate()
            .map(|(idx, col)| pgrow_value_to_string(self, idx, col))
            .collect()
    }
}

pub fn pgrow_value_to_string(row: &PgRow, idx: usize, col: &PgColumn) -> String {
    let result = row.try_get_raw(idx);
    if let Err(_err) = result {
        return "NULL".to_string();
    }

    let pg_type_opt = ty_to_type(col.type_info());

    if pg_type_opt.is_none() {
        return "NULL".to_string();
    }
    let pg_type = pg_type_opt.as_ref().unwrap().clone();

    // https://docs.rs/sqlx/latest/sqlx/postgres/types/index.html
    match pg_type {
        PgType::Bool => value_to_string::<bool>(row, idx),
        PgType::Int2 => value_to_string::<i16>(row, idx),
        PgType::Int4 => value_to_string::<i32>(row, idx),
        PgType::Int8 => value_to_string::<i64>(row, idx),
        PgType::Float4 => value_to_string::<f32>(row, idx),
        PgType::Float8 => value_to_string::<f64>(row, idx),
        PgType::Uuid => value_to_string::<uuid::Uuid>(row, idx),
        // chrono::DateTime<Utc>	TIMESTAMPTZ
        // chrono::DateTime<Local>	TIMESTAMPTZ
        // chrono::NaiveDateTime	TIMESTAMP
        // chrono::NaiveDate	DATE
        // chrono::NaiveTime	TIME
        PgType::Date => value_to_string::<chrono::NaiveDate>(row, idx),
        PgType::Time => value_to_string::<chrono::NaiveTime>(row, idx),
        PgType::Timestamp => value_to_string::<chrono::NaiveDateTime>(row, idx),
        PgType::Timestamptz => value_to_string::<chrono::DateTime<chrono::Utc>>(row, idx),
        PgType::Float4Array => array_value_to_string::<f32>(row, idx),
        PgType::Float8Array => array_value_to_string::<f64>(row, idx),
        PgType::TimestampArray => array_value_to_string::<f64>(row, idx),
        PgType::DateArray => array_value_to_string::<chrono::NaiveDate>(row, idx),
        PgType::TimeArray => array_value_to_string::<chrono::NaiveTime>(row, idx),
        PgType::TimestamptzArray => {
            array_value_to_string::<chrono::DateTime<chrono::Utc>>(row, idx)
        }
        PgType::UuidArray => array_value_to_string::<uuid::Uuid>(row, idx),
        PgType::BoolArray => array_value_to_string::<bool>(row, idx),
        PgType::Int2Array => array_value_to_string::<i16>(row, idx),
        PgType::Int4Array => array_value_to_string::<i32>(row, idx),
        PgType::Int8Array => array_value_to_string::<i64>(row, idx),

        // TODO:
        PgType::JsonArray => todo!(),
        PgType::CharArray => todo!(),
        PgType::TextArray => todo!(),
        PgType::VarcharArray => todo!(),
        PgType::JsonbArray => todo!(),
        PgType::ByteaArray => todo!(),
        PgType::NameArray => todo!(),
        PgType::BpcharArray => todo!(),
        PgType::PointArray => todo!(),
        PgType::LsegArray => todo!(),
        PgType::PathArray => todo!(),
        PgType::BoxArray => todo!(),
        PgType::PolygonArray => todo!(),
        PgType::OidArray => todo!(),
        PgType::MacaddrArray => todo!(),
        PgType::InetArray => todo!(),
        PgType::Bpchar => todo!(),
        PgType::Interval => todo!(),
        PgType::IntervalArray => todo!(),
        PgType::NumericArray => todo!(),
        PgType::Timetz => todo!(),
        PgType::TimetzArray => todo!(),
        PgType::Bit => todo!(),
        PgType::BitArray => todo!(),
        PgType::Varbit => todo!(),
        PgType::VarbitArray => todo!(),
        PgType::Numeric => todo!(),
        PgType::Record => todo!(),
        PgType::RecordArray => todo!(),
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
        // PgType::DeclareWithName(_) => todo!(),
        // PgType::DeclareWithOid(_) => todo!(),
        PgType::Bytea => todo!(),
        PgType::Name => value_to_string::<String>(row, idx),
        PgType::Oid => todo!(),
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

        // Para el resto intentamos extraer valor de:
        //   VARCHAR
        //   TEXT
        //   JSON
        //   JSONB
        //   ENUM
        //   No podemos antes porque entonces algunos como INT4 entra aquí cuando no es correcto.
        // _ => {
        PgType::Varchar
        | PgType::Json
        | PgType::Jsonb
        | PgType::Char
        | PgType::Text
        | PgType::Custom(_) => {
            if let Ok(s) = result.unwrap().as_str() {
                s.to_string()
            } else {
                "NULL".to_string()
            }
        }
    }
}

fn array_value_to_string<'r, T>(row: &'r PgRow, idx: usize) -> String
where
    T: std::fmt::Display,
    Vec<T>: Decode<'r, Postgres> + Type<Postgres>,
{
    let ls: Result<Option<Vec<T>>, sqlx::Error> = row.try_get(idx);
    match ls {
        Ok(v) => v.map_or("NULL".to_string(), |ls| {
            ls.iter()
                .map(|el| el.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        }),
        Err(_err) => String::from("ERR parsing"),
    }
}

fn value_to_string<'r, T>(row: &'r PgRow, idx: usize) -> String
where
    T: Decode<'r, Postgres> + Type<Postgres> + fmt::Display,
{
    // Option para poder representar columnas NULLABLE
    match row.try_get::<Option<T>, usize>(idx) {
        Ok(v) => v.map_or("NULL".to_string(), |v| format!("{}", v)),
        Err(_err) => String::from("ERR parsing"),
    }
}

pub fn pg_type_from_string(s: &str) -> PgType {
    PgType::from_string(s)
}
