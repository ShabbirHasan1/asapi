// -------------------------------------------------------------------------
// Copyright (C) 2023 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use eframe::egui;

pub fn request<F>(
    ui: &mut egui::Ui,
    idx: usize,
    selected_idx: &mut Option<usize>,
    action_to_perform: &mut Option<String>,
    show_update: bool,
    cb: F,
) where
    F: Fn(&mut egui::Ui),
{
    if ui.button("Rename").clicked() {
        *action_to_perform = Some(String::from("Rename"));
        *selected_idx = Some(idx);
        cb(ui);
        ui.close_menu();
    }
    if ui.button("Delete").clicked() {
        *action_to_perform = Some(String::from("Delete"));
        *selected_idx = Some(idx);
        ui.close_menu();
    }
    if show_update && ui.button("Actualizar").clicked() {
        *action_to_perform = Some("Update".to_string());
        *selected_idx = Some(idx);
        ui.close_menu();
    }
}
