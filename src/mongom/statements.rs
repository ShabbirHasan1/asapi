// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

// use regex::Regex;

// use super::state::MongoState;

// #[derive(Debug, PartialEq)]
// pub enum Action {
//     INSERT(String),
//     UPDATE(String),
//     DELETE(String),
//     SELECT(String),
//     NONE,
// }

// pub fn extract_stmt_action(sql: &str) -> Action {
//     use Action::*;

//     let re = Regex::new(r"(?i)(INSERT INTO|UPDATE|DELETE FROM|SELECT)\s+(\w+)").unwrap();

//     match re.captures(sql) {
//         Some(caps) => {
//             let action_str = caps.get(1).map_or("", |m| m.as_str()).to_uppercase();
//             let table_str = caps.get(2).map_or("", |m| m.as_str());
//             let table = String::from(table_str);

//             let action = if action_str == "INSERT INTO" {
//                 INSERT(table)
//             } else if action_str == "UPDATE" {
//                 UPDATE(table)
//             } else if action_str == "DELETE FROM" {
//                 DELETE(table)
//             } else if action_str == "SELECT" {
//                 SELECT(table)
//             } else {
//                 NONE
//             };

//             // ESTO PARA PONER EN EL CLIENTE
//             match action {
//                 Action::NONE => println!("wrong action"),
//                 _ => {
//                     println!("Action: {:?}", action);
//                 }
//             }

//             action
//         }
//         None => NONE,
//     }
// }

// pub fn columns_names_and_types(client: &mut Client, col_name: &str) -> Vec<(String, String)> {
//     let stmt = format!(
//         "
// SELECT
//     pg_attribute.attname AS column_name,
//     pg_catalog.format_type(pg_attribute.atttypid, pg_attribute.atttypmod) AS data_type
// FROM
//     pg_catalog.pg_attribute
// INNER JOIN
//     pg_catalog.pg_class ON pg_class.oid = pg_attribute.attrelid
// INNER JOIN
//     pg_catalog.pg_namespace ON pg_namespace.oid = pg_class.relnamespace
// WHERE
//     pg_attribute.attnum > 0
//     AND NOT pg_attribute.attisdropped
//     AND pg_class.relname = '{}'
// ORDER BY
//     attnum ASC",
//         col_name
//     );
//     let mut result = Vec::<(String, String)>::new();

//     // if let Some(ref mut client) = *self.state.conn {
//     // let query_results = client.query(stmt.as_str(), &[]);
//     // match query_results {
//     //     Ok(rows) => {
//     //         for row in &rows {
//     //             result.push((row.get::<usize, String>(0), row.get::<usize, String>(1)));
//     //         }
//     //     }
//     //     Err(e) => println!("{:?}", e),
//     // }

//     result
// }

// pub fn run_statement_with_delete_control(
//     stmt: &str,
//     local_state: &mut MongoState,
//     make_all_visible: bool,
//     delete_allowed: bool,
// ) -> bool {
//     // match extract_stmt_action(stmt) {
//     //     Action::DELETE(_) => {
//     //         if delete_allowed {
//     //             run_statement(stmt, local_state, make_all_visible);
//     //             true
//     //         } else {
//     //             false
//     //         }
//     //     }
//     //     _ => {
//     //         run_statement(stmt, local_state, make_all_visible);
//     //         true
//     //     }
//     // }
//     todo!()
// }

// pub fn run_statement(stmt: &str, local_state: &mut MongoState, _make_all_visible: bool) {
//     local_state.current_table_row_idx = 0;
//     local_state.current_table_end_idx = 0;
//     // let _action = extract_stmt_action(stmt);

//     // if let Some(ref mut client) = *local_state.conn {
//     //     println!("Statement: {}", stmt);

//     //     // let query_results = client.query(stmt, &[]);
//     //     // println!("RESULTS: {:?}", query_results);

//     //     // match query_results {
//     //     //     Ok(rows) => {
//     //     //         if let Some(data) = rows.get(0) {
//     //     //             if make_all_visible {
//     //     //                 local_state.column_visible =
//     //     //                     std::iter::repeat(true).take(data.len()).collect::<Vec<_>>();
//     //     //             }
//     //     //             local_state.current_table_columns.clear();
//     //     //             local_state.current_table_data.clear();
//     //     //             local_state.query_sort = QuerySort::NONE;
//     //     //             for col in data.columns().iter() {
//     //     //                 local_state
//     //     //                     .current_table_columns
//     //     //                     .push((col.name().to_string(), col.type_().to_string()));
//     //     //             }
//     //     //         }
//     //     //         // Extraemos los campos de las filas.
//     //     //         for row in &rows {
//     //     //             presenter::row_iterate(row, &mut local_state.current_table_data);
//     //     //         }
//     //     //         match action {
//     //     //             Action::NONE => {
//     //     //                 // println!("ru_statement :: wrong action");
//     //     //                 local_state.last_response = None;
//     //     //             }
//     //     //             Action::INSERT(t_name) | Action::UPDATE(t_name) | Action::DELETE(t_name) => {
//     //     //                 let (columns, rows) = select_all(client, t_name.as_str());
//     //     //                 local_state.column_visible = std::iter::repeat(true)
//     //     //                     .take(columns.len())
//     //     //                     .collect::<Vec<_>>();
//     //     //                 local_state.current_table_columns = columns;
//     //     //                 local_state.current_table_data = rows;
//     //     //                 println!(
//     //     //                     "Len after deleting: {}",
//     //     //                     local_state.current_table_data.len()
//     //     //                 );
//     //     //                 local_state.last_response = None;
//     //     //             }
//     //     //             Action::SELECT(_) => {
//     //     //                 println!("run_statement :: No extra select to do");
//     //     //                 local_state.last_response = None;
//     //     //             }
//     //     //         }
//     //     //     }
//     //     //     Err(e) => {
//     //     //         let msg = format!("{:?}", e);
//     //     //         println!("{msg}");
//     //     //         local_state.last_response = Some(msg);
//     //     //     }
//     //     // }
//     // } else {
//     //     local_state.last_response = Some("No connection stablished.".to_string());
//     // }
// }
