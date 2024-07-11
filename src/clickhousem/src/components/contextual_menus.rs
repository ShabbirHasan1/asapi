// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------
use eframe::egui;

use common::internationalization::I18nClickHouse;

use crate::{domain::ClickHouseMessage, state::SqlState};

pub struct ClickHouseTableInfo;

impl ClickHouseTableInfo {
    pub fn show(ui: &mut egui::Ui, ch_st: &SqlState, t_name: &str) {
        let table_info_opt = ch_st.current_connection_tables_info.get(t_name);
        if let Some(t_info) = table_info_opt {
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

                    for data in t_info {
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
    }
}

pub struct ClickHouseTableContextMenu;

// Menú donde elegimos si queremos abrir
//     - ventana de generación
//     - ventana de inserción
//     - otras opciones
impl ClickHouseTableContextMenu {
    pub fn show(
        ui: &mut egui::Ui,
        tx_sync: &std::sync::mpsc::Sender<ClickHouseMessage>,
        sql_st: &mut SqlState,
        i18n: &I18nClickHouse,
        t_name: &str,
    ) {
        sql_st.data_gen.table_to_generate_data = Some(String::from(t_name));
        let table_info_opt = sql_st.current_connection_tables_info.get(t_name);

        if let Some(t_info) = table_info_opt {
            if ui.button(&i18n.btn_table_data_generator).clicked() {
                // Primera y segunda columna tienen la representación que me interesa.
                let name_and_types: Vec<(String, String)> = t_info
                    .iter()
                    .map(|v| (v[0].clone(), v[2].clone()))
                    .collect();
                sql_st.data_gen.selected_table_definition = name_and_types;
                let len = sql_st.data_gen.selected_table_definition.len();
                sql_st.data_gen.fixed_value_flags = vec![false; len];
                sql_st.data_gen.nullable_column_flags = vec![false; len];
                sql_st.data_gen.fixed_string_value = vec![String::default(); len];
                sql_st.data_gen.show_generator_window = true;
                ui.close_menu();
            }
            if ui.button(&i18n.btn_table_data_insertion).clicked() {
                let name_and_types: Vec<(String, String)> = t_info
                    .iter()
                    .map(|v| (v[0].clone(), v[2].clone()))
                    .collect();
                sql_st.data_gen.selected_table_definition = name_and_types;
                let len = sql_st.data_gen.selected_table_definition.len();
                sql_st.data_gen.nullable_column_flags = vec![false; len];
                sql_st.data_gen.fixed_string_value = vec![String::default(); len];
                sql_st.data_gen.show_insertion_window = true;
                ui.close_menu();
            }
            ui.separator();
            ui.menu_button("Mas Acciones", |ui| {
                if ui.button(&i18n.btn_clean_table).clicked() {
                    let _ = tx_sync
                        .to_owned()
                        .send(ClickHouseMessage::DeleteAllStmt(t_name.to_owned()));
                    ui.close_menu();
                }
            });
        }
    }
}
