// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use eframe::egui::{self, Context};
use serde_json::Value;
use tokio::runtime::Runtime;

use crate::common::syntax_highlighting::{highlight, CodeTheme};
use crate::components::toggle_selector::toggle_label;
use crate::mongom::document::find::MongoOperator;
use crate::mongom::parser::{build_mongo_query, pprint_bson};
use crate::mongom::state::{MongoFilter, MongoLocalState};
use crate::mongom::view::MongoView;
use crate::{error, info};

pub fn add_filter(
    op: MongoOperator,
    key: Option<String>,
    val: Option<Value>,
    parent: Option<usize>,
    state: &mut MongoLocalState,
    // ls: &mut Vec<MongoFilter>,
) -> usize {
    let idx = state.filters.len();
    let filter = MongoFilter {
        op,
        key,
        val,
        idx,
        children: Vec::new(),
        parent,
    };

    // Si hay padre seleccionado, este nuevo filtro es su hijo.
    if let Some(parent_idx) = parent {
        if let Some(parent_filter) = state.filters.get_mut(parent_idx) {
            // if let Some(parent_filter) = ls.get_mut(parent_idx) {
            parent_filter.children.push(idx);
        }
    }

    state.filters.push(filter);

    idx
}

#[derive(PartialEq, Debug)]
enum UserAction {
    None,
    AddAnd(usize),
    AddOr(usize),
    // Otras acciones según sea necesario...
}
impl MongoView {
    fn show_filters(&self, ui: &mut egui::Ui, parent_idx: Option<usize>) -> UserAction {
        let mut action = UserAction::None;

        for (idx, filter) in self
            .state
            .filters
            .iter()
            .enumerate()
            .filter(|(_, f)| f.parent == parent_idx)
        {
            ui.horizontal(|ui| {
                ui.label(format!("{:?}: ", filter.op));
                if let Some(key) = &filter.key {
                    ui.label(format!("Key: {}", key));
                }
                if let Some(val) = &filter.val {
                    ui.label(format!("Value: {:?}", val));
                }
                if ui.button("AND").clicked() {
                    action = UserAction::AddAnd(idx);
                }
                if ui.button("OR").clicked() {
                    action = UserAction::AddOr(idx);
                }
            });

            let child_action = self.show_filters(ui, Some(filter.idx));
            if child_action != UserAction::None {
                action = child_action;
            }
        }

        action
    }

    pub fn compound_filter_constructor(
        &mut self,
        rt: &Runtime,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
    ) {
        let user_action = self.show_filters(ui, None);

        match user_action {
            UserAction::AddAnd(idx) | UserAction::AddOr(idx) => {
                let op = if user_action == UserAction::AddAnd(idx) {
                    MongoOperator::AND
                } else {
                    MongoOperator::OR
                };
                let mut old_filter = self.state.filters[idx].clone();
                let new_and_or_filter_idx =
                    add_filter(op.clone(), None, None, old_filter.parent, &mut self.state);
                self.state.filters[new_and_or_filter_idx].children.push(idx);
                old_filter.parent = Some(new_and_or_filter_idx);
                self.state.filters[idx] = old_filter;
                // Actualizamos el padre actual al nuevo filtro AND/OR.
                self.state.current_parent = Some(new_and_or_filter_idx);

                // info!("Filtros tras {:?}: {:?}", op, self.state.filters);
            }
            _ => (),
        }

        ui.horizontal(|ui| {
            toggle_label(ui, &mut self.state.current_selection.is_not, "Not");
            self.available_keys_combo(ui);
            self.select_action_options(ui);
            ui.text_edit_singleline(&mut self.state.current_filter_value);
            self.select_bson_data_type(ui);
            let data: serde_json::Result<Value> =
                serde_json::from_str(&self.state.current_filter_value);

            if ui.button("ADD").clicked() {
                match data {
                    Ok(ref value) => {
                        let _ = add_filter(
                            self.state.current_operator.clone(),
                            Some(self.state.current_selected_key.clone()),
                            Some(value.clone()),
                            self.state.current_parent,
                            &mut self.state,
                        );
                        self.state.last_error = None;

                        println!();
                        info!("{:?}", &self.state.filters);
                        println!();
                        info!("{:?}", build_mongo_query(&self.state.filters));
                        println!();
                        pprint_bson(&build_mongo_query(&self.state.filters));

                        // Al añadir sin más no modificamos el padre.
                    }
                    Err(ref e) => {
                        self.state.last_error = Some(format!("{:?}", e));
                        error!("{}", self.state.last_error.as_ref().unwrap());
                    }
                }
            }
        });

        if ui.button("Clean Filter").clicked() {
            self.state.clean_filter();
            self.find_all(rt, ctx);
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
