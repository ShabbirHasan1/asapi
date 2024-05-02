// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------
use eframe::{
    egui::{self, Sense},
    epaint::{vec2, Stroke},
};

/// Separador custom usando la base de `Separator::default`.
pub fn ui_color_separator(ui: &mut egui::Ui, color: egui::Color32) {
    let available_space = ui.available_size_before_wrap();
    let (rect, response) = ui.allocate_at_least(vec2(available_space.x, 6.0), Sense::hover());

    if ui.is_rect_visible(response.rect) {
        let stroke = Stroke {
            width: ui.visuals().widgets.noninteractive.bg_stroke.width,
            color,
        };
        let painter = ui.painter();
        painter.hline(
            (rect.left())..=(rect.right()),
            painter.round_to_pixel(rect.center().y),
            stroke,
        );
    }
}
