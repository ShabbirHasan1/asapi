// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

/// Selectores distintos, normalmente más simples, que los que hay en la egui.
use eframe::egui;

/// Label que se activa/desactiva al clicar.
///
/// Para su uso como swith sencillo sin necesidad de varios elementos.
///
/// # Ejemplo:
/// ``` ignore
/// toggle_label(ui, &mut self.flag, "Flag");
/// ```
pub fn toggle_label(ui: &mut egui::Ui, current_value: &mut bool, text: &str) -> egui::Response {
    let mut response = ui.selectable_label(*current_value, text);
    if response.clicked() {
        *current_value = !*current_value;
        response.mark_changed();
    }

    response
}
