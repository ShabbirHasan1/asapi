// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

#[macro_export]
macro_rules! ui_button_w50 {
    ($ui:expr, $arg:expr) => {{
        $ui.add_sized(egui::vec2(50.0, 20.0), egui::Button::new($arg))
            .clicked()
    }};
}

#[macro_export]
macro_rules! ui_button_w {
    ($ui:expr, $arg:expr, $w:expr) => {{
        $ui.add_sized(egui::vec2($w, 20.0), egui::Button::new($arg))
            .clicked()
    }};
}

#[macro_export]
macro_rules! ui_button_w100 {
    ($ui:expr, $arg:expr) => {{
        $ui.add_sized(egui::vec2(100.0, 20.0), egui::Button::new($arg))
            .clicked()
    }};
}

#[macro_export]
macro_rules! heading_strong {
    ($ui:expr, $text:expr) => {{
        $ui.label(egui::RichText::new($text).heading().strong())
    }};
}
#[macro_export]
macro_rules! ui_text_edit_singleline_hint {
    ($ui:expr, $hint:expr, $var:expr) => {{
        $ui.add(egui::TextEdit::singleline(&mut $var).hint_text(&hint));
    }};
}

// -->> Estas dos no funcionan
#[macro_export]
macro_rules! ted_singleline_w100 {
    ($ui:expr, $var:expr, $arg:expr) => {{
        $ui.add_sized(
            egui::vec2(100.0, 20.0),
            egui::TextEdit::singleline(&mut $var).hint_text(&$arg),
        )
    }};
}

#[macro_export]
macro_rules! ted_singleline_w50 {
    ($ui:expr, $var:expr, $arg:expr) => {{
        $ui.add_sized(
            egui::vec2(50.0, 20.0),
            egui::TextEdit::singleline(&mut $var).hint_text($arg),
        )
    }};
}
// <<--

#[macro_export]
macro_rules! ui_title_and_value_grid_row_with_hint {
    ($ui:expr, $title:expr, $value:expr, $hint:expr) => {{
        $ui.monospace($title);
        $ui.label($value).on_hover_ui(|ui| {
            ui.label($hint);
        });
        $ui.end_row();
    }};
}

#[macro_export]
macro_rules! ui_title_and_value_grid_row {
    ($ui:expr, $title:expr, $value:expr) => {{
        $ui.monospace($title);
        $ui.label($value);
        $ui.end_row();
    }};
}
