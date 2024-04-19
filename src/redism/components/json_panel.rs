// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------
use eframe::egui;

use crate::{
    common::internationalization::I18n,
    error, info,
    redism::{presenter, view::RedisView},
};

impl RedisView {
    pub fn show_json(&mut self, ui: &mut egui::Ui, _i18n: &I18n) {
        egui::Grid::new("json objects")
            .spacing(egui::vec2(ui.spacing().item_spacing.x * 2.0, 0.0))
            .show(ui, |ui| {
                for header in &self.state.jsons {
                    ui.code(header.0.clone()).context_menu(|ui| {
                        if ui.button("Delete").clicked() {
                            match presenter::delete_key(
                                &self.state.current_connection.host,
                                &self.state.current_connection.port,
                                &header.0,
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
                    ui.label(header.1.clone());
                    ui.end_row();
                }
            });
    }
}
