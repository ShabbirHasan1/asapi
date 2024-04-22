// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use eframe::egui;

use super::{state::SqlState, traits::Show};

pub struct Paginator {}

impl Show for Paginator {
    fn show(ui: &mut egui::Ui, state: &mut SqlState) {
        let data_len = state.current_table_rows.len();

        // --> Paginación (botones para avanzar/retroceder e información) <--
        if data_len > state.n_rows_to_show {
            ui.add_enabled_ui(state.first_row_idx > 0, |ui| {
                if ui.button("<").clicked() {
                    let new_index = state.first_row_idx as isize - state.n_rows_to_show as isize;

                    if new_index > 0 {
                        state.first_row_idx = new_index as usize;
                    } else {
                        state.first_row_idx = 0;
                    }
                }
            });

            let new_index = state.first_row_idx + state.n_rows_to_show;
            ui.add_enabled_ui(new_index < data_len, |ui| {
                if ui.button(">").clicked() && new_index < data_len {
                    state.first_row_idx = new_index;
                }
            });

            ui.label(format!(
                "{} : {} [{}]",
                state.first_row_idx, state.last_row_idx, data_len
            ));
        }
    }
}
