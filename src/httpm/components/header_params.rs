// -------------------------------------------------------------------------
// Copyright (C) 2023 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use eframe::egui;

#[derive(Default)]
pub struct HeaderParams {
    pub params: Vec<(String, String)>,
}

impl HeaderParams {
    pub fn create(&mut self, ui: &mut egui::Ui) -> Option<bool> {
        let mut has_changed = None;
        let mut idx_to_del: Option<usize> = None;

        ui.horizontal(|ui| {
            ui.label("Headers");
            if ui.button("+").clicked() {
                self.params.push((String::new(), String::new()));
                has_changed = Some(true);
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

                ui.label(":");
                ui.add(
                    egui::TextEdit::singleline(header_value)
                        .hint_text("value")
                        .desired_width(f32::INFINITY),
                );
            });
        }
        if let Some(idx) = idx_to_del {
            if idx < self.params.len() {
                self.params.remove(idx);
            }
        }

        has_changed
    }
}
