// -------------------------------------------------------------------------
// Copyright (C) 2023 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use std::path::PathBuf;

use eframe::egui;
use egui_file_dialog::{DialogMode, DialogState};
use egui_json_tree::JsonTree;
use serde_json::Value as JsonValue;

use crate::{
    common::{fs::list_files_in_directory, internationalization::I18n},
    httpm::{methods::HttpMethod, state::HttpLocalState},
};

#[derive(Default)]
pub struct BodyParams {
    pub params: Vec<(String, String)>,
    pub has_files: Vec<bool>,
    pub files: Vec<Vec<PathBuf>>,
    pub selected_idx: usize,
}

impl BodyParams {
    pub fn create(
        &mut self,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
        method: HttpMethod,
        state: &mut HttpLocalState,
        i18n: &I18n,
    ) -> Option<bool> {
        let mut has_changed = None;
        let mut idx_to_del: Option<usize> = None;
        let editable = !(method == HttpMethod::Get || method == HttpMethod::Delete);

        if !editable {
            return None;
        }

        if state.files.must_read {
            match (&state.files.current_state, &state.files.selected_mode) {
                (Some(st), Some(mode)) => {
                    match (st, mode) {
                        (DialogState::Selected(path), DialogMode::SelectDirectory) => {
                            self.files[self.selected_idx] = list_files_in_directory(path.as_path());
                            state.files.must_read = false;
                            self.selected_idx = usize::MAX;
                        }
                        (DialogState::Selected(path), DialogMode::SelectFile) => {
                            self.files[self.selected_idx] = vec![path.to_path_buf()];
                            state.files.must_read = false;
                            self.selected_idx = usize::MAX;
                        }
                        _ => (),
                    };
                }
                _ => (),
            }
        }

        ui.horizontal(|ui| {
            ui.label("Body");
            if editable && ui.button("+").clicked() {
                self.params.push((String::new(), String::new()));
                self.has_files.push(false);
                self.files.push(vec![]);
                has_changed = Some(true);
            }
            if method == HttpMethod::Post {
                ui.checkbox(&mut state.upload_files, "Multipart")
                    .on_hover_text(&i18n.http_multipart_help);
                // if state.upload_files {
                //     if ui.button(&i18n.http_select_folder).clicked() {
                //         state.files.file_dialog.select_directory();
                //         state.files.must_read = true;
                //     }
                //     if ui.button(&i18n.http_select_file).clicked() {
                //         state.files.file_dialog.select_file();
                //         state.files.must_read = true;
                //     }
                //     ui.label(format!(
                //         "{} {}",
                //         state.files.files_in_selected_folder.len(),
                //         i18n.http_selected_files_prefix
                //     ))
                //     .on_hover_ui(|ui| {
                //         ui.label(
                //             &state
                //                 .files
                //                 .files_in_selected_folder
                //                 .iter()
                //                 .map(|p| p.to_str())
                //                 .filter(|p| p.is_some())
                //                 .map(|p| p.unwrap())
                //                 .collect::<Vec<&str>>()
                //                 .join("\n"),
                //         );
                //     });
                //     if ui.button(&i18n.http_clean_file_folder_selection).clicked() {
                //         state.files.reset();
                //     }
                // }
            }
        });

        for i in 0..self.params.len() {
            ui.horizontal(|ui| {
                if ui.button("-").clicked() {
                    idx_to_del = Some(i);
                    has_changed = Some(true);
                }

                let (header_key, header_value) = &mut self.params[i];
                ui.add(egui::TextEdit::singleline(header_key).hint_text("key"));

                if !(state.upload_files && self.has_files[i]) {
                    ui.label(":");
                    ui.add(if state.upload_files {
                        egui::TextEdit::singleline(header_value).hint_text("value")
                    } else {
                        egui::TextEdit::singleline(header_value)
                            .hint_text("value")
                            .desired_width(f32::INFINITY)
                    });
                }
                if state.upload_files {
                    ui.checkbox(&mut self.has_files[i], &i18n.http_body_add_files);
                    if self.has_files[i] {
                        if ui.button(&i18n.http_select_folder).clicked() {
                            self.selected_idx = i;
                            state.files.file_dialog.select_directory();
                            state.files.must_read = true;
                        }
                        if ui.button(&i18n.http_select_file).clicked() {
                            self.selected_idx = i;
                            state.files.file_dialog.select_file();
                            state.files.must_read = true;
                        }
                        // TODO: Mostrar en hover sobre este botón la lista de archivos. El texto que ponga `Borrar n Seleccionados`.
                        let len = self.files[i].len();
                        if ui
                            .add(egui::Button::new(format!("Borrar {len} archivos")))
                            .on_hover_ui_at_pointer(|ui| {
                                ui.label(
                                    &self.files[i]
                                        .iter()
                                        .map(|p| p.to_str())
                                        .filter(|p| p.is_some())
                                        .map(|p| p.unwrap())
                                        .collect::<Vec<&str>>()
                                        .join("\n"),
                                );
                            })
                            .clicked()
                        {
                            self.files[i].clear();
                        }
                    }
                }
            });
        }
        if let Some(idx) = idx_to_del {
            if idx < self.params.len() {
                self.params.remove(idx);
            }
        }

        if !self.params.is_empty() {
            let json_map: serde_json::Map<String, JsonValue> = self
                .params
                .iter()
                .map(|(k, v)| (k.clone(), serde_json::from_str(v).unwrap_or_default()))
                .collect();
            let json_value = JsonValue::Object(json_map);
            JsonTree::new("http_body", &json_value)
                .default_expand(egui_json_tree::DefaultExpand::ToLevel(2))
                .show(ui);
        }

        if !editable {
            ctx.set_style(ctx.style().clone());
        }

        has_changed
    }
}
