// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use eframe::egui::{self, Label};

use crate::{
    common::internationalization::I18n,
    error, info,
    redism::{presenter, view::RedisView},
};

impl RedisView {
    pub fn show_hashes(&mut self, ui: &mut egui::Ui, _i18n: &I18n) {
        ui.set_width(ui.available_width());
        for (h_name, v) in &self.state.hashes {
            // --> Manejamos acciones sobre elemento que muestra nombre del hash
            ui.collapsing(h_name, |ui| {
                // TODO: Borrar todos en cascada con el menú contextual del hash.
                egui::Grid::new(h_name)
                    .spacing(egui::vec2(ui.spacing().item_spacing.x * 2.0, 0.0))
                    .show(ui, |ui| {
                        for (field_key, field_value) in v {
                            let field_label =
                                ui.add(Label::new(format!("    {} : {}", field_key, field_value)));

                            // --> Cada campo se puede borrar con menú contextual <--
                            field_label.context_menu(|ui| {
                                if ui.button("Delete").clicked() {
                                    match presenter::delete_hashkey(
                                        &self.state.current_connection.host,
                                        &self.state.current_connection.port,
                                        h_name,
                                        field_key,
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
                            ui.end_row();
                        }
                    });
            });
        }
    }
}
