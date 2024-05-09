// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use eframe::egui;

#[derive(Default)]
pub struct Params {}

impl Params {
    pub fn create(&mut self, ui: &mut egui::Ui, mut params: Vec<(String, String)>, label: String) {
        ui.horizontal(|ui| {
            ui.label(label);
        });

        for (header_key, header_value) in &mut params {
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(header_key).interactive(false));

                ui.label(":");
                ui.add(
                    egui::TextEdit::singleline(header_value)
                        .interactive(false)
                        .desired_width(f32::INFINITY),
                );
            });
        }
    }
}
