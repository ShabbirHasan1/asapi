// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use std::collections::{HashMap, VecDeque};

use bson::Document;
use eframe::egui::{self, Context};
use egui_json_tree::JsonTree;
use serde_json::{json, Value};
use tokio::runtime::Runtime;

use crate::common::internationalization::I18n;
use crate::common::syntax_highlighting::{highlight, CodeTheme};
use crate::components::toggle_selector::toggle_label;
use crate::mongom::filter::UserAction;
use crate::mongom::filter::{MongoFilter, MongoOperator};
use crate::mongom::parser::doc_to_pretty_string;
use crate::mongom::view::MongoView;
use crate::{error, info};

impl MongoView {
    fn show_filters(
        filters: &mut VecDeque<MongoFilter>,
        i18n: &I18n,
        ui: &mut egui::Ui,
        level: usize,
        parent: Option<usize>,
        parent_operator: MongoOperator,
    ) -> UserAction {
        let mut action = UserAction::None;

        for f in filters {
            let id_source = format!("{}/{:?}/{:?}/{}", f.op, f.key, f.val, level);

            let element = ui.indent(id_source, |ui| {
                ui.horizontal(|ui| {
                    ui.monospace(format!("{:?}", f.op));

                    if let (Some(key), Some(val)) = (&f.key, &f.val) {
                        JsonTree::new(format!("{}/{}/", f.idx, f.op), &json!({ key: val }))
                            .show(ui);
                    }

                    ui.add_enabled_ui(parent_operator != MongoOperator::NOT, |ui| {
                        if ui.button("AND").clicked() {
                            action = UserAction::AddAnd(f.idx);
                        }
                        if ui.button("OR").clicked() {
                            action = UserAction::AddOr(f.idx);
                        }
                        if ui.button("NOR").clicked() {
                            action = UserAction::AddNor(f.idx);
                        }
                    });

                    if ui.button("Delete").clicked() {
                        action = UserAction::Delete(f.idx);
                    }
                });

                let child_action = MongoView::show_filters(
                    &mut f.children,
                    i18n,
                    ui,
                    level + 1,
                    parent,
                    f.op.clone(),
                );
                if child_action != UserAction::None {
                    action = child_action;
                }
            });

            if parent.unwrap_or(usize::MAX) == f.idx {
                ui.painter().rect_stroke(
                    element.response.rect,
                    0.0,
                    egui::Stroke::new(2.0, egui::Color32::BLACK),
                );
            }
        }

        action
    }

    fn find_filter(fs: &VecDeque<MongoFilter>, idx: usize) -> Option<MongoFilter> {
        let mut filter = None;
        for f in fs {
            if f.idx == idx {
                return Some(f.clone());
            } else if filter.is_none() {
                filter = MongoView::find_filter(&f.children, idx);
            }
        }

        return filter;
    }

    /// Buscamos índice y devolvemos filtro y padre
    ///
    /// Si el filtro no existe se devuelve None, y si el filtro existe pero no tiene padre se
    /// devuelve None como segundo elemento de la tupla.
    /// En caso de tener padre sí se devuelve Some(...) en el segundo elemento.
    fn delete_filter(fs: &mut VecDeque<MongoFilter>, idx_to_delete: usize) -> bool {
        // Flag para evitarnos seguir atravesando una vez en alguna rama de la
        // jerarquía ya se borró.
        let mut was_deleted = false;

        for (idx_in_deque, f) in fs.iter_mut().enumerate() {
            if f.idx == idx_to_delete {
                fs.remove(idx_in_deque);
                return true;
            } else if !was_deleted {
                was_deleted = MongoView::delete_filter(&mut f.children, idx_to_delete);
            } else {
                break;
            }
        }

        was_deleted
    }

    fn add_child(fs: &mut VecDeque<MongoFilter>, idx: usize, child: &MongoFilter) {
        for f in fs.iter_mut() {
            if f.idx == idx {
                f.add_child(child.clone());
            } else {
                MongoView::add_child(&mut f.children, idx, child);
            }
        }
    }

    /// Intercambio de dos filtros de posición.
    ///
    /// Usado para AND/OR/NOT/NOR, coloca este en la posición donde había un filtro con `idx`,
    /// y ese filtro lo pone como hijo del AND/OR/NOT/NOR.
    fn swap_filters(fs: &mut VecDeque<MongoFilter>, idx: usize, new_filter: &MongoFilter) {
        for f in fs.iter_mut() {
            if f.idx == idx {
                let c = f.clone();
                let mut nf = new_filter.clone();
                nf.add_child(c);
                *f = nf;
            } else {
                MongoView::swap_filters(&mut f.children, idx, new_filter);
            }
        }
    }

    pub fn compound_filter_constructor(
        &mut self,
        rt: &Runtime,
        ctx: &egui::Context,
        i18n: &I18n,
        ui: &mut egui::Ui,
    ) {
        // --> Mostramos los filtros ya grabados <--
        let user_action = MongoView::show_filters(
            &mut self.state.filters,
            i18n,
            ui,
            0,
            self.state.current_parent,
            MongoOperator::AND, // Lo es de forma implícita.
        );

        // Según la acción y el índice, insertamos aquí o allá
        match user_action {
            UserAction::AddAnd(idx) | UserAction::AddOr(idx) | UserAction::AddNor(idx) => {
                let op = match user_action {
                    UserAction::AddAnd(_) => MongoOperator::AND,
                    UserAction::AddNor(_) => MongoOperator::NOR,
                    _ => MongoOperator::OR,
                };

                self.state.current_parent = Some(self.state.next_idx);
                let new_and_filter = MongoFilter::new(op, None, None, self.state.next_idx);
                MongoView::swap_filters(&mut self.state.filters, idx, &new_and_filter);

                self.state.next_idx += 1;
            }
            UserAction::Delete(idx) => {
                let filter = MongoView::find_filter(&self.state.filters, idx);
                info!("\nFiltro a Borrar (idx: {idx})\n {:?}", filter);
                let _ = MongoView::delete_filter(&mut self.state.filters, idx);
            }
            UserAction::None => (),
        };

        // --> Mostramos la entrada de datos <--
        ui.horizontal(|ui| {
            toggle_label(ui, &mut self.state.current_selection.is_not, "Not");
            self.available_keys_combo(ui);
            self.select_action_options(ui);
            ui.text_edit_singleline(&mut self.state.current_filter_value);
            self.select_bson_data_type(ui);

            if ui.button("ADD").clicked() {
                let data: serde_json::Result<Value> =
                    serde_json::from_str(&self.state.current_filter_value);
                match data {
                    Ok(ref value) => {
                        let mut f = MongoFilter::new(
                            self.state.current_operator.clone(),
                            Some(self.state.current_selected_key.clone()),
                            Some(value.clone()),
                            self.state.next_idx,
                        );

                        if self.state.current_selection.is_not {
                            self.state.next_idx += 1;
                            let mut not_filter = MongoFilter::new(
                                MongoOperator::NOT,
                                None,
                                None,
                                self.state.next_idx,
                            );
                            not_filter.add_child(f);

                            f = not_filter;
                        }
                        // Si no hay, añado sin más.
                        if self.state.current_parent.is_none() || self.state.filters.is_empty() {
                            self.state.filters.push_back(f);
                            self.state.next_idx += 1;
                            self.state.last_error = None;
                            self.state.current_selection.is_not = false;
                        } else if let Some(idx) = self.state.current_parent {
                            MongoView::add_child(&mut self.state.filters, idx, &f);
                            self.state.next_idx += 1;
                            self.state.last_error = None;
                            self.state.current_selection.is_not = false;
                        }
                    }
                    Err(ref e) => {
                        self.state.last_error = Some(format!("{:?}", e));
                        error!("{}", self.state.last_error.as_ref().unwrap());
                    }
                }
            }
        });

        if !self.state.filters.is_empty() {
            ui.horizontal(|ui| {
                if ui.button(&i18n.mongo_clean_filter).clicked() {
                    self.state.clean_filter();
                    self.find_all(rt, ctx);
                }
                if ui.button(&i18n.mongo_clean_parent).clicked() {
                    self.state.current_parent = None;
                }
                ui.label(
                    egui::RichText::new(&i18n.mongo_previsualize_filter)
                        .color(egui::Color32::from_rgb(128, 128, 128)),
                )
                .on_hover_ui(|ui| {
                    let docs = self
                        .state
                        .filters
                        .iter()
                        .map(|f| f.build_mongo_query())
                        .collect::<Vec<Document>>();
                    ui.monospace(doc_to_pretty_string(&docs));
                })
            });
        }
    }

    pub fn user_defined_filter_input(&mut self, ctx: &Context, ui: &mut egui::Ui) {
        let theme = CodeTheme::from_memory(ctx);
        let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
            let mut layout_job = highlight(ui.ctx(), &theme, string, "json");
            layout_job.wrap.max_width = wrap_width;
            ui.fonts(|f| f.layout_job(layout_job))
        };

        ui.add(
            egui::TextEdit::multiline(&mut self.state.current_selection.user_free_input)
                .font(egui::TextStyle::Monospace)
                .code_editor()
                .desired_rows(5)
                .lock_focus(true)
                .desired_width(f32::INFINITY)
                .layouter(&mut layouter),
        );
    }
}
