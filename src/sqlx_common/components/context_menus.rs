// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use eframe::egui;
use std::sync::mpsc::Sender;

use crate::{
    common::internationalization::I18nSqlx,
    sqlx_common::state::{SqlState, SqlxMessage},
};

pub struct TableContextMenu;

// Menú donde elegimos si queremos abrir
//     - ventana de generación
//     - ventana de inserción
//     - otras opciones
impl TableContextMenu {
    pub fn show(
        ui: &mut egui::Ui,
        tx: &Sender<SqlxMessage>,
        sql_st: &mut SqlState,
        i18n: &I18nSqlx,
        t_name: &str,
    ) {
        sql_st.data_gen.table_to_generate_data = Some(String::from(t_name));
        let table_info_opt = sql_st.current_connection_tables_info.get(t_name);

        if let Some(t_info) = table_info_opt {
            if ui.button(&i18n.pg.btn_table_data_generator).clicked() {
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
            if ui.button(&i18n.pg.btn_table_data_insertion).clicked() {
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
                if ui.button(&i18n.pg.btn_clean_table).clicked() {
                    let _ = tx
                        .to_owned()
                        .send(SqlxMessage::DeleteAllStmt(t_name.to_owned()));
                    ui.close_menu();
                }
            });
        }
    }
}
