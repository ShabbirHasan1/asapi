// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use std::marker::PhantomData;

use regex::Regex;
use sqlx::{Column, Database, Row, TypeInfo};

use super::state::SqlDataGenState;
use crate::common::traits::ShowVec;

#[derive(Debug, PartialEq, Clone)]
pub enum Action {
    Insert(String),
    Update(String),
    Delete(String),
    Select(String),
    CreateTable(String),
    DropTable(String),
    None,
}

pub struct SqlPresenter<DB>
where
    DB: Database,
{
    _marker: PhantomData<DB>,
}

impl<DB> Default for SqlPresenter<DB>
where
    DB: Database,
{
    fn default() -> Self {
        Self {
            _marker: Default::default(),
        }
    }
}

pub fn extract_stmt_action(sql: &str) -> Action {
    use Action::*;

    let re = Regex::new(r"(?i)(INSERT INTO|UPDATE|DELETE FROM|CREATE TABLE|DROP TABLE)\s+(\w+)")
        .unwrap();
    let re_select = Regex::new(r"FROM\s+([^\s,]+)").unwrap();

    match re.captures(sql) {
        Some(caps) => {
            let action_str = caps.get(1).map_or("", |m| m.as_str()).to_uppercase();
            let table_str = caps.get(2).map_or("", |m| m.as_str());
            let table = String::from(table_str);

            if action_str == "INSERT INTO" {
                Insert(table)
            } else if action_str == "UPDATE" {
                Update(table)
            } else if action_str == "DELETE FROM" {
                Delete(table)
            } else if action_str == "CREATE TABLE" {
                DropTable(table)
            } else if action_str == "DROP TABLE" {
                CreateTable(table)
            } else {
                None
            }
        }
        _ => match re_select.captures(sql) {
            Some(caps) => {
                let table_str = caps.get(1).map_or("", |m| m.as_str());
                let table = String::from(table_str);

                Select(table)
            }
            _ => None,
        },
    }
}

type PairMatrixTupleString = (Vec<Vec<String>>, Vec<(String, String)>);

pub fn extract_info_from_stmt_result<T>(data: Vec<T>) -> Option<PairMatrixTupleString>
where
    T: Row + ShowVec,
{
    if data.is_empty() {
        None
    } else {
        let rows_as_vecs = data.iter().map(|row| row.to_string_vec()).collect();
        let columns = data[0].columns();
        Some((
            rows_as_vecs,
            columns
                .iter()
                .map(|c| (c.name().to_string(), c.type_info().name().to_string()))
                .collect(),
        ))
    }
}

pub fn create_columns_string(state: &SqlDataGenState) -> String {
    state
        .selected_table_definition
        .iter()
        .map(|e| e.0.clone())
        .collect::<Vec<String>>()
        .join(",")
}
