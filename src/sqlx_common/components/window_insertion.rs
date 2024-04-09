// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------
use crate::{
    sqlx_module::{
        presenter::create_columns_string,
        state::{SqlState, SqlxMessage},
    },
    utils::{internatiolization::I18n, wrap_with_single_quote},
};
use eframe::egui;
use sqlx::Database;
use std::fmt::Debug;
use std::marker::PhantomData;

pub struct InsertionWindow<R, DB, T>(PhantomData<R>, PhantomData<DB>, PhantomData<T>);

impl<R, DB, T> Default for InsertionWindow<R, DB, T> {
    fn default() -> Self {
        Self(Default::default(), Default::default(), Default::default())
    }
}

impl<R, DB, T> InsertionWindow<R, DB, T>
where
    T: Debug,
    DB: Database,
{
    pub fn show(
        &self,
        tx: &std::sync::mpsc::Sender<SqlxMessage>,
        ctx: &egui::Context,
        state: &mut SqlState,
        table_name: &str,
        _i18n: &I18n,
        should_be_wrapped: impl Fn(&str) -> bool,
    ) {
        egui::Window::new(format!("Insertar Fila {}", table_name))
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                egui::Grid::new("row-insertion-definition")
                    .num_columns(3)
                    .show(ui, |ui| {
                        for (idx, (n, t)) in
                            state.data_gen.selected_table_definition.iter().enumerate()
                        {
                            ui.checkbox(&mut state.data_gen.nullable_column_flags[idx], "Null");
                            ui.code(n);

                            ui.add_enabled_ui(!state.data_gen.nullable_column_flags[idx], |ui| {
                                ui.add(
                                    egui::TextEdit::singleline(
                                        &mut state.data_gen.fixed_string_value[idx],
                                    )
                                    .hint_text("Valor predeterminado"),
                                );
                                ui.monospace(t.clone());
                            });

                            ui.end_row();
                        }
                    });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::LEFT), |ui| {
                    if ui.button("Cancel").clicked() {
                        state.data_gen.reset();
                    }
                    if ui.button("Insert").clicked() {
                        state.data_gen.reset();

                        let columns = create_columns_string(&state.data_gen);
                        let values = state
                            .data_gen
                            .fixed_string_value
                            .iter()
                            .enumerate()
                            .map(|(idx, v)| {
                                let is_null = state.data_gen.nullable_column_flags[idx];
                                if is_null {
                                    "NULL".to_string()
                                } else {
                                    let (_, t) =
                                        state.data_gen.selected_table_definition[idx].clone();
                                    if should_be_wrapped(t.to_ascii_uppercase().as_str()) {
                                        wrap_with_single_quote(v)
                                    } else {
                                        v.clone()
                                    }
                                }
                            })
                            .collect::<Vec<String>>()
                            .join(",");

                        let stmt = format!(
                            "INSERT INTO {} ({}) VALUES ({})",
                            table_name, columns, values
                        );
                        let _ = tx.to_owned().send(SqlxMessage::InsertStatement(stmt));
                    }
                });
            });
    }
}
