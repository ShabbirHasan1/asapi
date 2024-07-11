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
use rdkafka::error::KafkaError;

use components::result_panel::job_fn;

pub fn ui_error_panel(ui: &mut egui::Ui, optional_error: &Option<KafkaError>) {
    if let Some(error) = optional_error {
        egui::Frame::default()
            .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
            .show(ui, |ui| {
                ui.label(
                    RichText::new("ERROR")
                        .color(Color32::RED)
                        .strong()
                        .monospace(),
                );
                ui.set_width(ui.available_width());
                ui.label(job_fn(error.to_string()));
            });
    }
}
