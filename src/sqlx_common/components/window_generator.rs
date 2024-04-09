// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use eframe::egui;
use sqlx::Database;
use std::fmt::Debug;
use std::{marker::PhantomData, ops::RangeInclusive};

use crate::common::traits::Runner;
use crate::components::toggle_switch;
use crate::quote;
use crate::{
    common::internationalization::I18n,
    sqlx_common::{
        data_generation::GenericGenerator,
        presenter::create_columns_string,
        state::{SqlState, SqlxMessage},
        traits::Presenter,
    },
};

pub struct GeneratorWindow<R, DB, T>(PhantomData<R>, PhantomData<DB>, PhantomData<T>);

impl<R, DB, T> Default for GeneratorWindow<R, DB, T> {
    fn default() -> Self {
        Self(Default::default(), Default::default(), Default::default())
    }
}

impl<R, DB, T> GeneratorWindow<R, DB, T>
where
    T: Debug,
    DB: Database,
{
    pub fn show(
        &self,
        tx: &std::sync::mpsc::Sender<SqlxMessage>,
        ctx: &egui::Context,
        state: &mut SqlState,
        t_name: &str,
        _i18n: &I18n,
        presenter: &impl Presenter,
        parse_type_representation: impl Fn(&str) -> T,
        generate_value: impl Fn(&T) -> String,
    ) {
        egui::Window::new(format!("Generar Datos Aleatorios {}", t_name))
            .collapsible(false)
            .resizable(false) // hace falta porque al alinear la fila con los botones permite redimensionar pero no tiene sentido.
            .show(ctx, |ui| {
                // 2. Coger el tipo de cada campo
                egui::Grid::new("generator-definition")
                    .num_columns(6)
                    .show(ui, |ui| {
                        for (idx, (n, t)) in
                            state.data_gen.selected_table_definition.iter().enumerate()
                        {
                            ui.checkbox(&mut state.data_gen.nullable_column_flags[idx], "Null");
                            ui.code(n);
                            ui.monospace(t.clone());
                            let text = if state.data_gen.fixed_value_flags[idx] {
                                "Fijo"
                            } else {
                                "Aleatorio"
                            };

                            ui.label(text);
                            ui.add(toggle_switch::toggle(
                                &mut state.data_gen.fixed_value_flags[idx],
                            ));
                            if state.data_gen.fixed_value_flags[idx] {
                                ui.add(
                                    egui::TextEdit::singleline(
                                        &mut state.data_gen.fixed_string_value[idx],
                                    )
                                    .hint_text("Introducir Valor"),
                                );
                            } else {
                                ui.label(String::from("-----"));
                            }
                            ui.end_row();
                        }
                    });

                ui.horizontal(|ui| {
                    ui.label("Number of Rows to Add");
                    ui.add(
                        egui::DragValue::new(&mut state.data_gen.n_rows_to_generate)
                            .clamp_range(RangeInclusive::new(1, 65535)),
                    );
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::LEFT), |ui| {
                    if ui.button("Cancel").clicked() {
                        state.data_gen.reset();
                    }

                    if ui.button("Generate").clicked() {
                        state.data_gen.reset();

                        let column_names_string: String = create_columns_string(&state.data_gen);

                        // Lo ponemos aquí para solo hacerlo al clicar.
                        let types: Vec<T> = state
                            .data_gen
                            .selected_table_definition
                            .iter()
                            .map(|(_, t)| parse_type_representation(t))
                            .collect();

                        for _ in 0..state.data_gen.n_rows_to_generate {
                            let values = types
                                .iter()
                                .enumerate()
                                .map(|(idx, t)| {
                                    let is_null = state.data_gen.nullable_column_flags[idx];
                                    if is_null && GenericGenerator::<bool>::run() {
                                        "NULL".to_string()
                                    } else if state.data_gen.fixed_value_flags[idx] {
                                        let (_, t) =
                                            state.data_gen.selected_table_definition[idx].clone();

                                        if presenter.should_be_wrapped(t.as_str()) {
                                            quote!(&state.data_gen.fixed_string_value[idx])
                                        } else {
                                            state.data_gen.fixed_string_value[idx].clone()
                                        }
                                    } else {
                                        generate_value(t)
                                    }
                                })
                                .collect::<Vec<String>>()
                                .join(",");

                            let stmt = format!(
                                "INSERT INTO {} ({}) VALUES ({})",
                                t_name, column_names_string, values
                            );
                            let _ = tx.to_owned().send(SqlxMessage::InsertStatement(stmt));
                        }

                        state.data_gen.reset();
                    }
                });
            });
    }
}
