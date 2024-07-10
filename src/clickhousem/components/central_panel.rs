// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use eframe::egui;
use egui_extras::{Size, StripBuilder};
use tokio::runtime::Runtime;

use crate::common::icon_moon::IconMoon;
use crate::common::internationalization::I18nClickHouse;
use crate::components::result_panel::ui_response_panel;
use crate::quote;
use crate::{
    clickhousem::presenter, clickhousem::state::ClickHouseAppState,
    clickhousem::view::ClickHouseView,
};

use super::paginator::Paginator;
use super::table::{PerformanceTable, RegularTable};

impl ClickHouseView {
    pub fn show_central_panel(
        &mut self,
        ctx: &egui::Context,
        rt: &Runtime,
        app_st: &mut ClickHouseAppState,
        i18n: &I18nClickHouse,
    ) {
        self.show_edit_row_window(ctx, rt, app_st);

        egui::CentralPanel::default().show(ctx, |ui| {
            StripBuilder::new(ui)
                .size(Size::remainder())
                .size(Size::exact(50.0))
                .vertical(|mut strip| {
                    strip.cell(|ui| {
                        ui.set_width(ui.available_width());

                        ui_response_panel(ui, &self.state.sql.last_response_error);

                        egui::CollapsingHeader::new(&i18n.table_columns)
                            .default_open(false)
                            .show_background(true)
                            .show(ui, |ui| {
                                ui.horizontal_wrapped(|ui| {
                                    for (idx, (c_name, _c_type)) in
                                        self.state.sql.current_table_columns.iter().enumerate()
                                    {
                                        if ui
                                            .selectable_label(
                                                self.state.sql.column_visible[idx],
                                                c_name,
                                            )
                                            .clicked()
                                        {
                                            self.state.sql.column_visible[idx] =
                                                !self.state.sql.column_visible[idx];
                                        }
                                    }
                                });
                            });

                        ui.separator();

                        // --> Definimos la entrada y lanzar stmt por parte del usuario <--
                        let theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(ctx);
                        let mut sql_layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
                            let mut layout_job = egui_extras::syntax_highlighting::highlight(
                                ui.ctx(),
                                &theme,
                                string,
                                "sql",
                            );
                            layout_job.wrap.max_width = wrap_width;
                            ui.fonts(|f| f.layout_job(layout_job))
                        };
                        let sql_stmt_response = ui.add(
                            egui::TextEdit::multiline(&mut self.state.sql.sql_statement)
                                .font(egui::TextStyle::Monospace) // for cursor height
                                .code_editor()
                                .desired_rows(5)
                                .lock_focus(true)
                                .desired_width(f32::INFINITY)
                                .layouter(&mut sql_layouter),
                        );

                        if sql_stmt_response.has_focus()
                            && ctx.input(|i| {
                                i.key_down(egui::Key::Enter) && i.key_down(egui::Key::ArrowRight)
                            })
                        {
                            self.run_statement(
                                ctx,
                                rt,
                                self.state.sql.sql_statement.clone(),
                                !app_st.performance_table,
                                true,
                            );
                        }

                        if let Some(err) = self.state.sql.last_response.clone() {
                            ui.label(&err);
                        }

                        let data_len = self.state.sql.current_table_rows.len();

                        // --> Ejecutamos la consulta introducida por el usuario <--
                        let hover_menu = |ui: &mut egui::Ui| {
                            ui.label("Lanzar con \u{27a1} + \u{2ba8}");
                        };
                        ui.horizontal(|ui| {
                            if ui.button("\u{25b6}").on_hover_ui(hover_menu).clicked() {
                                self.run_statement(
                                    ctx,
                                    rt,
                                    self.state.sql.sql_statement.clone(),
                                    !app_st.performance_table,
                                    true,
                                );
                                sql_stmt_response.request_focus();
                            }

                            Paginator::show(ui, &mut self.state.sql);
                        });

                        // --> Definimos el rango de filas a mostrar <--
                        let end = self.state.sql.first_row_idx + self.state.sql.n_rows_to_show;
                        if end > data_len {
                            self.state.sql.last_row_idx = data_len;
                        } else {
                            self.state.sql.last_row_idx = end;
                        }

                        // --> Mostramos resultados de consultas en tabla que ocupa todo el espacio restante <--
                        egui::ScrollArea::horizontal().show(ui, |ui| {
                            if app_st.performance_table {
                                PerformanceTable::show(ui, &mut self.state.sql);
                            } else {
                                RegularTable::show(ui, &mut self.state.sql, rt, &self.tx);
                            }
                        });
                    });
                    strip.cell(|ui| {
                        ui.add(egui::Label::new(IconMoon::Letteri.as_str()))
                            .on_hover_ui(|ui| {
                                ui.label("Support for ClickHouse is experimental.");
                                ui.label("Be careful and use by your own risk.");
                            });
                    });
                })
        });
    }

    fn show_edit_row_window(
        &mut self,
        ctx: &egui::Context,
        rt: &Runtime,
        app_st: &mut ClickHouseAppState,
    ) {
        if let Some(row_idx) = self.state.sql.row_being_editted.selected_row {
            egui::Window::new("Editar Fila")
                .collapsible(false)
                .resizable(true)
                .show(ctx, |ui| {
                    let table_name =
                        self.state.sql.tables[self.state.sql.current_table_idx].clone();
                    egui::Grid::new("row-insertion-definition")
                        .num_columns(3)
                        .show(ui, |ui| {
                            for (idx, (n, t)) in
                                self.state.sql.current_table_columns.iter().enumerate()
                            {
                                ui.code(n);
                                ui.monospace(t.clone());
                                ui.text_edit_singleline(
                                    &mut self.state.sql.row_being_editted.row_data[idx],
                                );

                                ui.end_row();
                            }
                        });

                    let _with_layout =
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::LEFT), |ui| {
                            if ui.button("Cancel").clicked() {
                                self.state.sql.row_being_editted.selected_row = None;
                            }

                            if ui.button("Update").clicked() {
                                let user_values_string = self
                                    .state
                                    .sql
                                    .row_being_editted
                                    .row_data
                                    .iter()
                                    .enumerate()
                                    .map(|(idx, col_data)| {
                                        let (col_name, t) =
                                            self.state.sql.current_table_columns[idx].clone();

                                        let wrapped = quote!(&col_data);
                                        let value = if presenter::should_be_wrapped(&t) {
                                            &wrapped
                                        } else {
                                            col_data
                                        };
                                        format!("{}={}", col_name, value)
                                    })
                                    .collect::<Vec<String>>()
                                    .join(", ");

                                let filters = self.statement_filter(row_idx);

                                let stmt = format!(
                                    "UPDATE {} SET {} WHERE {}",
                                    table_name,
                                    user_values_string,
                                    filters.join(" AND ")
                                );

                                self.run_statement(ctx, rt, stmt, !app_st.performance_table, true);
                                self.state.sql.row_being_editted.selected_row = None;
                            }
                        });
                });
        }
    }
}
