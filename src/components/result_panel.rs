// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use eframe::{
    egui::{self, RichText},
    epaint::Color32,
};

pub fn ui_response_panel(ui: &mut egui::Ui, result: &Option<Result<String, String>>) {
    if let Some(result) = result {
        ui.add_enabled_ui(false, |ui| {
            ui.horizontal(|ui| {
                match result {
                    Ok(msg) => {
                        ui.label(msg);
                    }
                    Err(err) => {
                        ui.label(
                            RichText::new("ERROR")
                                .color(Color32::RED)
                                .strong()
                                .monospace(),
                        );
                        ui.label(err);
                    }
                };
            });
        });
    }
}
