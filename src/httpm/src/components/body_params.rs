// -------------------------------------------------------------------------
// Copyright (C) 2023 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use eframe::egui;
use egui_file_dialog::{DialogMode, DialogState};
use egui_json_tree::JsonTree;
use serde_json::Value as JsonValue;
use std::path::PathBuf;

use common::{fs::list_files_in_directory, icon_moon::IconMoon, internationalization::I18nHttp};

use crate::{methods::HttpMethod, state::HttpLocalState};

#[derive(Default)]
pub struct BodyParams {
    pub multipart: bool, // Petición con posibilidad de archivos mediante multipart??
    pub selected_idx: usize,

    // Estos tres vectores tienen que estar sincronizados, a.k.a., tener la misma longitud.
    pub params: Vec<(String, String, bool)>,
    // pub has_files: Vec<bool>,     // Cada campo tiene archivos?
    pub files: Vec<Vec<PathBuf>>, // Archivos seleccionados, cada índice para cada parámetro.
}

impl BodyParams {
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
        method: HttpMethod,
        state: &mut HttpLocalState,
        i18n: &I18nHttp,
    ) -> Option<bool> {
        let mut has_changed = None;
        let mut idx_to_del: Option<usize> = None;
        let editable = !(method == HttpMethod::Get || method == HttpMethod::Delete);

        if !editable {
            return None;
        }

        // --> Gestión de ventanas para selección archivo / carpeta <--
        if state.files.must_read {
            if let (Some(st), Some(mode)) = (&state.files.current_state, &state.files.selected_mode)
            {
                match (st, mode) {
                    (DialogState::Selected(path), DialogMode::SelectDirectory) => {
                        let files = list_files_in_directory(path.as_path());
                        self.files[self.selected_idx].clone_from(&files);
                        // self.files[self.selected_idx] = files.clone();
                        let files_as_str = files
                            .iter()
                            .map(|s| s.as_os_str().to_str().unwrap_or_default())
                            .collect::<Vec<&str>>()
                            .join(", ");
                        println!("{files_as_str}");
                        self.params[self.selected_idx].1 = format!("\"{}\"", files_as_str);
                        println!(
                            "{} : {}",
                            self.params[self.selected_idx].0, self.params[self.selected_idx].1
                        );

                        // .flat_map(OsStr::to_str)
                        // .map(String::from);
                        // self.params[self.selected_idx] =

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
            // match (&state.files.current_state, &state.files.selected_mode) {
            //     (Some(st), Some(mode)) => {
            //         match (st, mode) {
            //             (DialogState::Selected(path), DialogMode::SelectDirectory) => {
            //                 let files = list_files_in_directory(path.as_path());
            //                 self.files[self.selected_idx].clone_from(&files);
            //                 // self.files[self.selected_idx] = files.clone();
            //                 let files_as_str = files
            //                     .iter()
            //                     .map(|s| s.as_os_str().to_str().unwrap_or_default())
            //                     .collect::<Vec<&str>>()
            //                     .join(", ");
            //                 println!("{files_as_str}");
            //                 self.params[self.selected_idx].1 = format!("\"{}\"", files_as_str);
            //                 println!(
            //                     "{} : {}",
            //                     self.params[self.selected_idx].0, self.params[self.selected_idx].1
            //                 );

            //                 // .flat_map(OsStr::to_str)
            //                 // .map(String::from);
            //                 // self.params[self.selected_idx] =

            //                 state.files.must_read = false;
            //                 self.selected_idx = usize::MAX;
            //             }
            //             (DialogState::Selected(path), DialogMode::SelectFile) => {
            //                 self.files[self.selected_idx] = vec![path.to_path_buf()];
            //                 state.files.must_read = false;
            //                 self.selected_idx = usize::MAX;
            //             }
            //             _ => (),
            //         };
            //     }
            //     _ => (),
            // }
        }

        ui.horizontal(|ui| {
            ui.label("Body");
            if editable && ui.button("+").clicked() {
                self.params.push((String::new(), String::new(), false));
                self.files.push(vec![]);
                has_changed = Some(true);
            }
            if method == HttpMethod::Post
                && ui
                    .checkbox(&mut self.multipart, "Multipart")
                    .on_hover_text(&i18n.http_multipart_help)
                    .clicked()
            {
                has_changed = Some(true);
            }
            if method == HttpMethod::Post
                && ui
                    .checkbox(&mut self.multipart, "Multipart")
                    .on_hover_text(&i18n.http_multipart_help)
                    .clicked()
            {
                has_changed = Some(true);
            }
        });

        for i in 0..self.params.len() {
            ui.horizontal(|ui| {
                if ui.button("-").clicked() {
                    idx_to_del = Some(i);
                    has_changed = Some(true);
                }

                let (k, v, param_with_files) = &mut self.params[i];
                ui.add(egui::TextEdit::singleline(k).hint_text("key"));

                if !(self.multipart && *param_with_files) {
                    ui.label(":");
                    ui.add(if self.multipart {
                        egui::TextEdit::singleline(v).hint_text("value")
                    } else {
                        egui::TextEdit::singleline(v)
                            .hint_text("value")
                            .desired_width(f32::INFINITY)
                    });
                }

                if self.multipart {
                    ui.checkbox(&mut self.params[i].2, &i18n.http_body_add_files);
                    if self.params[i].2 {
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
                        let add_contents = |ui: &mut egui::Ui| {
                            ui.label(
                                &self.files[i]
                                    .iter()
                                    .filter_map(|p| p.to_str())
                                    .collect::<Vec<&str>>()
                                    .join("\n"),
                            );
                        };
                        if ui
                            .add(egui::Button::new(format!(
                                "Borrar {len} archivos {}",
                                if len > 0 {
                                    IconMoon::Letteri.as_str()
                                } else {
                                    ""
                                }
                            )))
                            .on_hover_ui_at_pointer(add_contents)
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
            egui::CollapsingHeader::new("Body JSON")
                .id_source("body_as_json")
                .show(ui, |ui| {
                    let json_map: serde_json::Map<String, JsonValue> = self
                        .params
                        .iter()
                        .map(|(k, v, _)| (k.clone(), serde_json::from_str(v).unwrap_or_default()))
                        .collect();
                    let json_value = JsonValue::Object(json_map);
                    egui::ScrollArea::vertical()
                        .id_source("body_scroll")
                        .show(ui, |ui| {
                            JsonTree::new("http_body", &json_value)
                                .default_expand(egui_json_tree::DefaultExpand::ToLevel(2))
                                .show(ui);
                        });
                });
        }

        if !editable {
            ctx.set_style(ctx.style().clone());
        }

        has_changed
    }
}
