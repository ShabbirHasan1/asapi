// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------
use eframe::egui;

use crate::sqlx_common::state::SqlState;

pub struct TableInfo;

impl TableInfo {
    pub fn show(ui: &mut egui::Ui, sql_st: &SqlState, t_name: &str) {
        let table_info_opt = sql_st.current_connection_tables_info.get(t_name);
        match table_info_opt {
            Some(table_info) => {
                egui::Grid::new("pg_table_info")
                    .num_columns(10)
                    .show(ui, |ui| {
                        ui.label("Name");
                        ui.label("Data Type");
                        ui.label("Data Name");
                        ui.label("Is Primary Key");
                        ui.label("Is Nullable");
                        ui.label("Column Default");
                        ui.label("Character Maximum Length");
                        ui.label("Is Foreign Key");
                        ui.label("FK Table");
                        ui.label("FK Column");
                        ui.end_row();

                        for data in table_info {
                            ui.code(&data[0]);
                            ui.monospace(&data[1]);
                            ui.monospace(&data[2]);
                            ui.monospace(&data[3]);
                            ui.monospace(&data[4]);
                            ui.monospace(&data[5]);
                            ui.monospace(&data[6]);
                            ui.monospace(&data[7]);
                            ui.monospace(&data[8]);
                            ui.monospace(&data[9]);
                            ui.end_row();
                        }
                    });
            }
            None => {}
        }
    }
}
