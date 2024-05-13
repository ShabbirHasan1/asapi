// -------------------------------------------------------------------------
// Copyright (C) 2023 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use eframe::egui;

use crate::{common::internationalization::I18n, httpm::state::HttpRequestAction};

pub fn request<F>(
    ui: &mut egui::Ui,
    idx: usize,
    selected_idx: &mut Option<usize>,
    action_to_perform: &mut HttpRequestAction,
    show_update: bool,
    i18n: &I18n,
    cb: F,
) where
    F: Fn(&mut egui::Ui),
{
    if ui.button(&i18n.http_context_menu_rename).clicked() {
        *action_to_perform = HttpRequestAction::Rename;
        *selected_idx = Some(idx);
        cb(ui);
        ui.close_menu();
    }

    if ui.button(&i18n.http_context_menu_delete).clicked() {
        *action_to_perform = HttpRequestAction::Delete;
        *selected_idx = Some(idx);
        ui.close_menu();
    }

    if show_update && ui.button(&i18n.http_context_menu_update).clicked() {
        *action_to_perform = HttpRequestAction::Update;
        *selected_idx = Some(idx);
        ui.close_menu();
    }
}
