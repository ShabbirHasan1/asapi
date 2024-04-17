// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use eframe::egui::{self, Label, Sense};
use egui_json_tree::JsonTree;

use crate::{
    common::internationalization::I18n,
    error, info,
    redism::{presenter, utils::value_map_to_string_btree_map, view::RedisView},
};

use super::contextual_menus;

impl RedisView {
    pub fn show_streams(&mut self, ui: &mut egui::Ui, i18n: &I18n) {
        {
            ui.set_width(ui.available_width());
            for (stream_name, v) in &self.state.streams {
                // ==> Gestión de Stream y todos los mensajes en él
                ui.collapsing(stream_name, |ui| {
                    for (idx, id) in v.iter().enumerate() {
                        // --> Gestión de cada mensaje <--
                        let label = match self.state.stream_id_values.get(id) {
                            Some(_) => ui.add(Label::new(id).sense(Sense::click())),
                            _ => ui
                                .add(Label::new(id).sense(Sense::click()))
                                .on_hover_text("Click to Open Stream and enabling resend"),
                        };

                        label.context_menu(|ui| {
                            // TODO: Aquí estoy cogiendo valores leídos
                            let option = self.state.stream_id_values.get(id);
                            self.state.must_scan = contextual_menus::stream_msg(
                                ui,
                                stream_name,
                                id.to_string().to_string(),
                                option,
                                &mut self.state.current_command,
                            );
                        });
                        if label.clicked() {
                            match self.state.stream_id_values.get(id) {
                                Some(_) => {
                                    self.state.stream_id_values.remove(id);
                                }
                                None => {
                                    // Hace falta esto porque cuando busco, si no es desde 0, el
                                    // que me devuelve es el siguiente al que selecciono, por
                                    // eso me hace falta el `idx-1`.
                                    let from_when = if idx == 0 { "0" } else { &v[idx - 1] };
                                    let _ = presenter::read_stream_id(
                                        &stream_name,
                                        from_when,
                                        &mut self.state.stream_id_values,
                                    );
                                    // if idx == 0 {
                                    //     let _ = presenter::read_stream_id(
                                    //         &stream_name,
                                    //         "0",
                                    //         &mut self.state.stream_id_values,
                                    //     );
                                    // } else {
                                    //     let _ = presenter::read_stream_id(
                                    //         &stream_name,
                                    //         &v[idx - 1],
                                    //         &mut self.state.stream_id_values,
                                    //     );
                                    // }
                                }
                            }
                        }
                        ui.end_row();
                        // TODO: Cambiar y almacenar los serde_json::Value para no estar
                        // haciendo el parseo continumamente. Eso nos permite volver a usar
                        // HashMap en vez de BTreeMap, aunque lo mejor sería comprobar el
                        // rendimiento al crear cada uno.
                        if let Some(value) = self.state.stream_id_values.get(id) {
                            // let value = serde_json::json!(value_map_to_string_map(value));
                            let value = serde_json::json!(value_map_to_string_btree_map(value));
                            JsonTree::new(id, &value).show(ui);
                        }
                    }
                })
                .header_response
                .context_menu(|ui| {
                    if ui.button("Delete").clicked() {
                        match presenter::delete_key(
                            &self.state.current_connection.host,
                            &self.state.current_connection.port,
                            &stream_name,
                        ) {
                            Ok(s) => {
                                self.state.must_scan = true;
                                info!("{:?}", s);
                            }
                            Err(e) => error!("{:?}", e),
                        }
                        ui.close_menu();
                    }
                });
            }
        }
    }
}
