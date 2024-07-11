// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use eframe::egui;
use egui::Ui;

#[inline(always)]
pub fn text_edit_singleline_w100(ui: &mut Ui, s: &str, v: &mut String) {
    text_edit_singleline(ui, s, v, 100.0, 20.0);
}

#[inline(always)]
pub fn text_edit_singleline_w50(ui: &mut Ui, s: &str, v: &mut String) {
    text_edit_singleline(ui, s, v, 50.0, 20.0);
}

#[inline(always)]
pub fn text_edit_singleline_w(ui: &mut Ui, s: &str, v: &mut String, w: f32) {
    text_edit_singleline(ui, s, v, w, 20.0);
}

#[inline(always)]
fn text_edit_singleline(ui: &mut Ui, s: &str, v: &mut String, w: f32, h: f32) {
    ui.add_sized(egui::vec2(w, h), egui::TextEdit::singleline(v).hint_text(s));
}

#[inline(always)]
pub fn ui_text_edit_singleline_hint(ui: &mut Ui, s: &str, v: &mut String) {
    ui.add(egui::TextEdit::singleline(v).hint_text(s));
}
