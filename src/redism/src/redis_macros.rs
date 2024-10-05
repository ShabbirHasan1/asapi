// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

// No está muy bien que sea la macro la que sabe qué nivel de exclusividad
// exige en vez de ser el cliente el que llame con `&mut ...`, pero es más
// flexible cuando requiero distintos niveles de acceso como por ejemplo
// en `strip_combo_box`, así que prefiero pasar la variable directamente.
#[macro_export]
macro_rules! strip_text_edit {
    ($strip:expr, $name:expr, $variable:expr) => {{
        $strip.cell(|ui| {
            ui.add(egui::TextEdit::singleline(&mut $variable).hint_text($name));
        })
    }};
}

#[macro_export]
macro_rules! strip_combo_box {
    ($strip:expr, $id:expr, $variable:expr, $($options:expr),+) => {{
        $strip.cell(|ui| {
            egui::ComboBox::from_id_salt($id)
                .selected_text(&$variable)
                .width(ui.available_width())
                .show_ui(ui, |ui| {
                    $(ui.selectable_value(&mut $variable, $options.to_string(), $options);)+
                });
        })
    }};
}
