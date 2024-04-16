// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use eframe::egui::{self, Context};

use crate::{
    common::internationalization::I18n,
    redism::{
        presenter::{self, RedisMenu},
        state::{RedisAppState, RedisConnectionDefinition},
        view::RedisView,
    },
};

impl RedisView {
    pub fn sidenav_show(&mut self, ctx: &Context, app_st: &mut RedisAppState, i18n: &I18n) {
        egui::SidePanel::left("redis_side_panel").show(ctx, |ui| {
            // --> Decidimos qué mostrar <--
            ui.horizontal(|ui| {
                if ui.button("\u{27f3} Load").clicked() {
                    let _ = presenter::scan(&mut self.state);
                }

                let s1 = if self.state.hide_connections {
                    "\u{229e}"
                } else {
                    "\u{229f}"
                };
                let s2 = if self.state.hide_data_structures {
                    "\u{229e}"
                } else {
                    "\u{229f}"
                };

                if ui
                    .button(format!("{s1} {}", &i18n.redis_connections))
                    .clicked()
                {
                    self.state.hide_connections = !self.state.hide_connections;
                }
                if ui
                    .button(format!("{s2} {}", &i18n.redis_data_structures))
                    .clicked()
                {
                    self.state.hide_data_structures = !self.state.hide_data_structures;
                }
            });

            // --> Abrimos ventana para definir conexión <--
            ui.menu_button(&i18n.pg_btn_add_connection, |ui| {
                ui.set_min_width(200.0);

                ui.horizontal(|ui| {
                    ui.label(&i18n.redis_connection_host);
                    ui.text_edit_singleline(&mut self.state.tmp_connection.host);
                });

                ui.horizontal(|ui| {
                    ui.label(&i18n.redis_connection_port);
                    ui.text_edit_singleline(&mut self.state.tmp_connection.port);
                });

                ui.horizontal(|ui| {
                    if ui.button(&i18n.redis_edit_connection_cancel).clicked() {
                        ui.close_menu();
                    }
                    if ui.button(&i18n.redis_edit_connection_save).clicked() {
                        // TODO: Añadir al listado
                        app_st.connections.push(self.state.tmp_connection.clone());
                        self.state.tmp_connection = RedisConnectionDefinition::default();
                        ui.close_menu();
                    }
                });
            });

            ui.separator();
            ui.set_width(200.0);
            // TODO: Mejor separar PubSub puesto que no está conectado con los demás... solo puede ser
            // conexión en vivo.
            for option in RedisMenu::iterator() {
                // ui.label(format!("{idx} -- {:#?}", option));
                let opt = option.clone();
                if ui
                    .selectable_value(&mut self.state.selected_menu, opt, format!("{:#?}", option))
                    .clicked()
                {}
            }
        });
    }
}
