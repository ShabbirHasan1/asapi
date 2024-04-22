// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use eframe::egui;
use egui_extras::{Column, TableBuilder, TableRow};
use tokio::runtime::Runtime;
use tokio::sync::mpsc::Sender;

use super::state::{QuerySort, SqlState, SqlxMessage};
use super::traits::Show;

pub struct RegularTable;
pub struct PerformanceTable;

impl RegularTable {
    pub fn show(ui: &mut egui::Ui, state: &mut SqlState, rt: &Runtime, tx: &Sender<SqlxMessage>) {
        let count = state.column_visible.iter().filter(|e| **e).count();
        let n_columns = if count == 0 { 0 } else { count - 1 };

        TableBuilder::new(ui)
            .auto_shrink(false) // Si ponemos true hace un glitch al cargar bastante horrible.
            .striped(true)
            // Bug visual que duraba meses, tabla no rellenaba verticalmente, fijado gracias a
            // https://github.com/emilk/egui/issues/2430#issuecomment-1608774435
            .max_scroll_height(f32::INFINITY)
            .column(Column::exact(50.0))
            .columns(
                Column::initial(150.0).range(40.0..).resizable(true),
                n_columns,
            )
            .column(Column::remainder())
            .header(24.0, |mut header| {
                insert_header(state, &mut header);
            })
            .body(|mut body| {
                let begin = state.first_row_idx;
                let end = state.last_row_idx;
                let rows_to_show = &state.current_table_rows[begin..end];

                for row_data in rows_to_show {
                    body.row(24.0, |mut row| {
                        let row_idx = row.index();
                        row.col(|ui| {
                            ui.menu_button((1 + row_idx + state.first_row_idx).to_string(), |ui| {
                                // --> Copiar Fila <--
                                if ui.button("Copy Row").clicked() {
                                    ui.ctx().copy_text(format!("{:?}", row_data));
                                    ui.close_menu();
                                }

                                // --> Borrar Fila <--
                                if ui.button("Delete Row").clicked() {
                                    let table_name = state.tables[state.current_table_idx].clone();
                                    let tx_cloned = tx.clone();

                                    rt.spawn(async move {
                                        let _ = tx_cloned
                                            .send(SqlxMessage::DeleteStatement((
                                                table_name, row_idx,
                                            )))
                                            .await;
                                    });
                                    ui.close_menu();
                                }

                                // --> Editar Fila <--
                                if ui.button("Edit Row").clicked() {
                                    ui.close_menu();
                                    state.row_being_editted.selected_row = Some(row_idx);
                                    state.row_being_editted.row_data =
                                        state.current_table_rows[row_idx].clone();
                                }
                            });
                        });

                        for (col_idx, col) in row_data.iter().enumerate() {
                            if state.column_visible[col_idx] {
                                row.col(|ui| {
                                    if ui
                                        .add(
                                            egui::TextEdit::singleline(&mut col.as_str())
                                                .desired_width(f32::INFINITY),
                                        )
                                        .clicked()
                                    {}
                                });
                            }
                        }
                    })
                }
            });
    }
}

impl Show for PerformanceTable {
    fn show(ui: &mut egui::Ui, state: &mut SqlState) {
        let count = state.column_visible.iter().filter(|e| **e).count();
        let n_columns = if count == 0 { 0 } else { count - 1 };

        TableBuilder::new(ui)
            .auto_shrink(false)
            .striped(true)
            .max_scroll_height(f32::INFINITY)
            .column(Column::exact(50.0))
            .columns(
                Column::initial(150.0).range(40.0..).resizable(true),
                n_columns, // Poco eficiente, almacenar en set cuando se clica y cambiamos memoria (aprox 0) por rendimiento.
            )
            .column(Column::remainder())
            .header(24.0, |mut header| {
                insert_header(state, &mut header);
            })
            .body(|body| {
                let begin = state.first_row_idx;
                let end = state.last_row_idx;
                let len = end - begin;

                body.rows(24.0, len, |mut row| {
                    let row_idx = row.index();
                    let row_data = &state.current_table_rows[begin..end][row_idx].clone();

                    row.col(|ui| {
                        ui.menu_button((1 + row_idx + state.first_row_idx).to_string(), |ui| {
                            if ui.button("Copy Row").clicked() {
                                ui.ctx().copy_text(format!("{:?}", row_data));
                                ui.close_menu();
                            }

                            ui.add_enabled_ui(false, |ui| ui.button("Delete Row"));

                            if ui.button("Edit Row").clicked() {
                                ui.close_menu();
                                state.row_being_editted.selected_row = Some(row_idx);
                                state.row_being_editted.row_data =
                                    state.current_table_rows[row_idx].clone();
                            }
                        });
                    });

                    for (col_idx, col) in row_data.iter().enumerate() {
                        if state.column_visible[col_idx] {
                            row.col(|ui| {
                                if ui
                                    .add(
                                        egui::TextEdit::singleline(&mut col.as_str())
                                            .desired_width(f32::INFINITY),
                                    )
                                    .clicked()
                                {}
                            });
                        }
                    }
                });
            });
    }
}

fn insert_header(state: &mut SqlState, header: &mut TableRow) {
    header.col(|_| {});
    for (idx, (n, t)) in state.current_table_columns.iter().enumerate() {
        if state.column_visible[idx] {
            header.col(|ui| {
                ui.horizontal(|ui| {
                    ui.heading(n).on_hover_ui(|ui| {
                        ui.label(t);
                    });
                    if ui
                        .selectable_label(
                            state.query_sort == QuerySort::Asc
                                && state.column_index_selected == idx,
                            "\u{2b06}",
                        )
                        .clicked()
                    {
                        state.query_sort = QuerySort::Asc;
                        state.column_index_selected = idx;
                        state.change_order = true;
                    }
                    if ui
                        .selectable_label(
                            state.query_sort == QuerySort::Desc
                                && state.column_index_selected == idx,
                            "\u{2b07}",
                        )
                        .clicked()
                    {
                        state.query_sort = QuerySort::Desc;
                        state.column_index_selected = idx;
                        state.change_order = true;
                    }
                });
            });
        }
    }
}
