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
        if let Some(table_info) = table_info_opt {
            egui::Grid::new("mysql_table_info")
                .num_columns(6)
                .show(ui, |ui| {
                    ui.label("Name");
                    ui.label("Data Type");
                    ui.label("Column Type");
                    ui.label("Is Nullable");
                    ui.label("Column Default");
                    ui.label("Column Key");
                    ui.end_row();

                    for data in table_info {
                        ui.code(&data[0]);
                        ui.monospace(&data[1]);
                        ui.monospace(&data[2]);
                        ui.monospace(&data[3]);
                        ui.monospace(&data[4]);
                        ui.monospace(&data[5]);
                        ui.end_row();
                    }
                });
        }
    }
}
