// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

#[macro_export]
macro_rules! strip_text_edit {
    ($strip:expr, $name:expr, $variable:expr) => {{
        $strip.cell(|ui| {
            ui_text_edit_singleline_hint(ui, $name, &mut $variable);
        })
    }};
}
