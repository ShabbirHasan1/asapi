// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use clickhouse_rs::types::{Complex, Decimal, Enum16, Enum8, FromSql, SqlType};
use clickhouse_rs::Block;
use std::collections::HashMap;
use std::hash::Hash;

pub fn map_to_vec_string(
    ktype: &SqlType,
    vtype: &SqlType,
    block: &Block<Complex>,
    column: &str,
) -> Vec<String> {
    match ktype {
        SqlType::Bool => extract_map_value::<bool>(block, column, vtype),
        SqlType::UInt8 => extract_map_value::<u8>(block, column, vtype),
        SqlType::UInt16 => extract_map_value::<u16>(block, column, vtype),
        SqlType::UInt32 => extract_map_value::<u32>(block, column, vtype),
        SqlType::UInt64 => extract_map_value::<u64>(block, column, vtype),
        SqlType::Int8 => extract_map_value::<i8>(block, column, vtype),
        SqlType::Int16 => extract_map_value::<i16>(block, column, vtype),
        SqlType::Int32 => extract_map_value::<i32>(block, column, vtype),
        SqlType::Int64 => extract_map_value::<i64>(block, column, vtype),
        SqlType::String => extract_map_value::<String>(block, column, vtype),
        SqlType::FixedString(_) => extract_map_value::<String>(block, column, vtype),
        SqlType::Date => extract_map_value::<chrono::NaiveDate>(block, column, vtype),
        SqlType::DateTime(_) => {
            extract_map_value::<chrono::DateTime<chrono_tz::Tz>>(block, column, vtype)
        }
        SqlType::Ipv4 => extract_map_value::<std::net::Ipv4Addr>(block, column, vtype),
        SqlType::Ipv6 => extract_map_value::<std::net::Ipv6Addr>(block, column, vtype),
        SqlType::Uuid => extract_map_value::<uuid::Uuid>(block, column, vtype),
        // No permitidos por la librería porque no implementan Eq
        SqlType::Decimal(_, _) | SqlType::Enum8(_) | SqlType::Enum16(_) => {
            vec![
                String::from("ASAPI does not support neither Decimal nor Enum as keys.");
                block.row_count()
            ]
        }
        // SqlType::Decimal(_, _) => extract_map_value::<Decimal>(block, column, vtype),
        // SqlType::Enum8(_) => extract_map_value::<Enum8>(block, column, vtype),
        // SqlType::Enum16(_) => extract_map_value::<Enum16>(block, column, vtype),
        // No permitidos por la librearía porque no implementan Hash
        // SqlType::Float32 => extract_map_value::<f32>(block, column, vtype),
        // SqlType::Float64 => extract_map_value::<f64>(block, column, vtype),
        SqlType::Float32 | SqlType::Float64 => {
            vec![
                String::from("ASAPI does not support floats as key because not hashables.");
                block.row_count()
            ]
        }
        SqlType::Nullable(_) => {
            vec![
                String::from("ClickHouse does not support nullable as Map key.");
                block.row_count()
            ]
        }
        _ => vec![
            String::from("ASAPI does not support nested/complex types as keys.");
            block.row_count()
        ],
    }
}

pub fn extract_map_value<'b, T: ToString + Hash + PartialEq + Eq + FromSql<'b>>(
    block: &'b Block<Complex>,
    column: &str,
    vtype: &SqlType,
) -> Vec<String> {
    match vtype {
        SqlType::Bool => (0..block.row_count())
            .map(|i| {
                block
                    .get::<HashMap<T, bool>, &str>(i, column)
                    .map(|m| {
                        m.iter()
                            .map(|(k, v)| format!("{}: {}", k.to_string(), v))
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or("ERROR".to_string())
            })
            .collect::<Vec<_>>(),
        SqlType::UInt8 => (0..block.row_count())
            .map(|i| {
                block
                    .get::<HashMap<T, u8>, &str>(i, column)
                    .map(|m| {
                        m.iter()
                            .map(|(k, v)| format!("{}: {}", k.to_string(), v))
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or("ERROR".to_string())
            })
            .collect::<Vec<_>>(),
        SqlType::UInt16 => (0..block.row_count())
            .map(|i| {
                block
                    .get::<HashMap<T, u16>, &str>(i, column)
                    .map(|m| {
                        m.iter()
                            .map(|(k, v)| format!("{}: {}", k.to_string(), v))
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or("ERROR".to_string())
            })
            .collect::<Vec<_>>(),
        SqlType::UInt32 => (0..block.row_count())
            .map(|i| {
                block
                    .get::<HashMap<T, u32>, &str>(i, column)
                    .map(|m| {
                        m.iter()
                            .map(|(k, v)| format!("{}: {}", k.to_string(), v))
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or("ERROR".to_string())
            })
            .collect::<Vec<_>>(),
        SqlType::UInt64 => (0..block.row_count())
            .map(|i| {
                block
                    .get::<HashMap<T, u64>, &str>(i, column)
                    .map(|m| {
                        m.iter()
                            .map(|(k, v)| format!("{}: {}", k.to_string(), v))
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or("ERROR".to_string())
            })
            .collect::<Vec<_>>(),
        SqlType::Int8 => (0..block.row_count())
            .map(|i| {
                block
                    .get::<HashMap<T, i8>, &str>(i, column)
                    .map(|m| {
                        m.iter()
                            .map(|(k, v)| format!("{}: {}", k.to_string(), v))
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or("ERROR".to_string())
            })
            .collect::<Vec<_>>(),
        SqlType::Int16 => (0..block.row_count())
            .map(|i| {
                block
                    .get::<HashMap<T, i16>, &str>(i, column)
                    .map(|m| {
                        m.iter()
                            .map(|(k, v)| format!("{}: {}", k.to_string(), v))
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or("ERROR".to_string())
            })
            .collect::<Vec<_>>(),
        SqlType::Int32 => (0..block.row_count())
            .map(|i| {
                block
                    .get::<HashMap<T, i32>, &str>(i, column)
                    .map(|m| {
                        m.iter()
                            .map(|(k, v)| format!("{}: {}", k.to_string(), v))
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or("ERROR".to_string())
            })
            .collect::<Vec<_>>(),
        SqlType::Int64 => (0..block.row_count())
            .map(|i| {
                block
                    .get::<HashMap<T, i64>, &str>(i, column)
                    .map(|m| {
                        m.iter()
                            .map(|(k, v)| format!("{}: {}", k.to_string(), v))
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or("ERROR".to_string())
            })
            .collect::<Vec<_>>(),
        SqlType::String => (0..block.row_count())
            .map(|i| {
                block
                    .get::<HashMap<T, String>, &str>(i, column)
                    .map(|m| {
                        m.iter()
                            .map(|(k, v)| format!("{}: {}", k.to_string(), v))
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or("ERROR".to_string())
            })
            .collect::<Vec<_>>(),
        SqlType::FixedString(_) => (0..block.row_count())
            .map(|i| {
                block
                    .get::<HashMap<T, String>, &str>(i, column)
                    .map(|m| {
                        m.iter()
                            .map(|(k, v)| format!("{}: {}", k.to_string(), v))
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or("ERROR".to_string())
            })
            .collect::<Vec<_>>(),
        SqlType::Float32 => (0..block.row_count())
            .map(|i| {
                block
                    .get::<HashMap<T, f32>, &str>(i, column)
                    .map(|m| {
                        m.iter()
                            .map(|(k, v)| format!("{}: {}", k.to_string(), v))
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or("ERROR".to_string())
            })
            .collect::<Vec<_>>(),
        SqlType::Float64 => (0..block.row_count())
            .map(|i| {
                block
                    .get::<HashMap<T, f64>, &str>(i, column)
                    .map(|m| {
                        m.iter()
                            .map(|(k, v)| format!("{}: {}", k.to_string(), v))
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or("ERROR".to_string())
            })
            .collect::<Vec<_>>(),
        SqlType::Date => (0..block.row_count())
            .map(|i| {
                block
                    .get::<HashMap<T, chrono::NaiveDate>, &str>(i, column)
                    .map(|m| {
                        m.iter()
                            .map(|(k, v)| format!("{}: {}", k.to_string(), v))
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or("ERROR".to_string())
            })
            .collect::<Vec<_>>(),
        SqlType::DateTime(_) => (0..block.row_count())
            .map(|i| {
                block
                    .get::<HashMap<T, chrono::DateTime<chrono_tz::Tz>>, &str>(i, column)
                    .map(|m| {
                        m.iter()
                            .map(|(k, v)| format!("{}: {}", k.to_string(), v))
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or("ERROR".to_string())
            })
            .collect::<Vec<_>>(),
        SqlType::Ipv4 => (0..block.row_count())
            .map(|i| {
                block
                    .get::<HashMap<T, std::net::Ipv4Addr>, &str>(i, column)
                    .map(|m| {
                        m.iter()
                            .map(|(k, v)| format!("{}: {}", k.to_string(), v))
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or("ERROR".to_string())
            })
            .collect::<Vec<_>>(),
        SqlType::Ipv6 => (0..block.row_count())
            .map(|i| {
                block
                    .get::<HashMap<T, std::net::Ipv6Addr>, &str>(i, column)
                    .map(|m| {
                        m.iter()
                            .map(|(k, v)| format!("{}: {}", k.to_string(), v))
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or("ERROR".to_string())
            })
            .collect::<Vec<_>>(),
        SqlType::Uuid => (0..block.row_count())
            .map(|i| {
                block
                    .get::<HashMap<T, uuid::Uuid>, &str>(i, column)
                    .map(|m| {
                        m.iter()
                            .map(|(k, v)| format!("{}: {}", k.to_string(), v))
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or("ERROR".to_string())
            })
            .collect::<Vec<_>>(),
        SqlType::Decimal(_, _) => (0..block.row_count())
            .map(|i| {
                block
                    .get::<HashMap<T, Decimal>, &str>(i, column)
                    .map(|m| {
                        m.iter()
                            .map(|(k, v)| format!("{}: {}", k.to_string(), v))
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or("ERROR".to_string())
            })
            .collect::<Vec<_>>(),
        SqlType::Enum8(_) => (0..block.row_count())
            .map(|i| {
                block
                    .get::<HashMap<T, Enum8>, &str>(i, column)
                    .map(|m| {
                        m.iter()
                            .map(|(k, v)| format!("{}: {}", k.to_string(), v))
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or("ERROR".to_string())
            })
            .collect::<Vec<_>>(),
        SqlType::Enum16(_) => (0..block.row_count())
            .map(|i| {
                block
                    .get::<HashMap<T, Enum16>, &str>(i, column)
                    .map(|m| {
                        m.iter()
                            .map(|(k, v)| format!("{}: {}", k.to_string(), v))
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or("ERROR".to_string())
            })
            .collect::<Vec<_>>(),
        // No soportado
        _ => vec![
            String::from("Not supported nested/complex types as value by ASAPI");
            block.row_count()
        ],
        // SqlType::Nullable(_) => todo!(),
        // SqlType::Array(_) => todo!(),
        // SqlType::SimpleAggregateFunction(_, _) => todo!(),
        // SqlType::Map(_, _) => todo!(),
    }
}
