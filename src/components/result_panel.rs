// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------



use eframe::{
    egui::{self, RichText},
    epaint::{text::LayoutJob, Color32},
};

const MAX_ROWS: usize = 6;
const MAX_WIDTH: f32 = 200.0;
const BREAK_ANYWHERE: bool = true;
const OVERFLOW_CHARACTER: char = '…';
// const EXTRA_LETTER_SPACING_PIXELS: usize = 0;
// const LINE_HEIGHT_PIXELS: usize = 0;

const DEFAULT_FORMAT: egui::TextFormat = egui::TextFormat {
    color: egui::Color32::BLACK,
    font_id: egui::FontId {
        size: 14.0,
        family: egui::FontFamily::Proportional,
    },
    extra_letter_spacing: 0.0,
    line_height: None,
    background: Color32::TRANSPARENT,
    italics: false,
    underline: egui::Stroke::NONE,
    strikethrough: egui::Stroke::NONE,
    valign: egui::Align::BOTTOM,
};

// Comento por ahorrarme todas esas operaciones, realmente no son necesarias
pub fn job_fn(text: String) -> LayoutJob {
    // let points_per_pixel = 1.0 / pixels_per_point;
    // let line_height =
    // (LINE_HEIGHT_PIXELS != 0).then_some(points_per_pixel * LINE_HEIGHT_PIXELS as f32);
    // let extra_letter_spacing = points_per_pixel * EXTRA_LETTER_SPACING_PIXELS as f32;
    let mut job = LayoutJob::single_section(text, DEFAULT_FORMAT);
    job.wrap = egui::text::TextWrapping {
        max_rows: MAX_ROWS,
        max_width: MAX_WIDTH,
        break_anywhere: BREAK_ANYWHERE,
        overflow_character: Some(OVERFLOW_CHARACTER),
    };
    job
}

pub fn ui_response_panel(ui: &mut egui::Ui, result: &Option<Result<String, String>>) {
    if let Some(result) = result {
        egui::Frame::default()
            .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
            .show(ui, |ui| {
                match result {
                    Ok(msg) => {
                        ui.label(job_fn(msg.clone()));
                    }
                    Err(err) => {
                        ui.label(
                            RichText::new("ERROR")
                                .color(Color32::RED)
                                .strong()
                                .monospace(),
                        );
                        ui.set_width(ui.available_width());
                        ui.label(job_fn(err.clone()));
                    }
                };
            });
    }
}
