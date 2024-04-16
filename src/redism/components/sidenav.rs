// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use std::collections::HashSet;

use eframe::egui::{self, Context};
use egui_extras::{Size, StripBuilder};
use redis::ConnectionLike;
use tokio::runtime::Runtime;

use crate::{
    app_state,
    common::internationalization::I18n,
    info,
    redism::{
        presenter::{self, RedisMenu},
        state::{RedisAppState, RedisConnectionDefinition, RedisLocalState},
        view::RedisView,
    },
};

impl RedisView {
    pub fn show_sidenav(&mut self, ctx: &Context, app_st: &mut RedisAppState, i18n: &I18n) {
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
                        app_st.connections.push(self.state.tmp_connection.clone());
                        info!("{:?}", app_st.connections);
                        self.state.tmp_connection = RedisConnectionDefinition::default();
                        ui.close_menu();
                    }
                });
            });

            ui.separator();
            ui.set_width(200.0);

            // TODO: Mejor separar PubSub puesto que no está conectado con los demás... solo puede ser
            // conexión en vivo.
            // TODO: Mostrar conexiones
            if !self.state.hide_connections && !self.state.hide_data_structures {
                StripBuilder::new(ui)
                    .size(Size::remainder())
                    .size(Size::remainder())
                    .vertical(|mut strip| {
                        strip.cell(|ui| {
                            self.show_connections(ui, app_st, i18n);
                        });
                        strip.cell(|ui| {
                            for option in RedisMenu::iterator() {
                                let opt = option.clone();
                                ui.selectable_value(
                                    &mut self.state.selected_menu,
                                    opt,
                                    format!("{:#?}", option),
                                );
                            }
                        });
                    });
            } else if !self.state.hide_connections {
                self.show_connections(ui, app_st, i18n);
            } else if !self.state.hide_data_structures {
                for option in RedisMenu::iterator() {
                    let opt = option.clone();
                    ui.selectable_value(
                        &mut self.state.selected_menu,
                        opt,
                        format!("{:#?}", option),
                    );
                }
            }
        });
    }

    fn show_connections(&mut self, ui: &mut egui::Ui, app_st: &mut RedisAppState, i18n: &I18n) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            let mut connections_to_delete: HashSet<usize> = HashSet::new();

            for (idx, conn_definition) in app_st.connections.iter_mut().enumerate() {
                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    ui.set_width(ui.available_width());
                    let button_text = format!(
                        "{}:{}",
                        conn_definition.host.clone(),
                        conn_definition.port.clone()
                    );

                    let raw_button = if self.state.conn.is_some() {
                        egui::Button::new(button_text)
                            .min_size(egui::vec2(200.0, 24.0))
                            .stroke(if idx == self.state.current_connection_idx {
                                egui::Stroke::new(1.0, egui::Color32::DARK_BLUE)
                            } else {
                                egui::Stroke::new(0.0, egui::Color32::LIGHT_BLUE)
                            })
                    } else {
                        egui::Button::new(button_text).min_size(egui::vec2(200.0, 24.0))
                    };

                    let button = ui.add(raw_button);

                    // --> Menú contextual para manejo de las conexiones <--
                    button.context_menu(|ui| {
                        if ui.button("Close Connection").clicked() {
                            // Este crate no requiere que cerremos explícitamente. Es más
                            // por cómo es redis que por que en sí haga algo especial.
                            self.state.conn = None;
                            self.state.current_connection_idx = usize::MAX;
                            ui.close_menu();
                        }
                        if ui.button("Delete Connection").clicked() {
                            connections_to_delete.insert(idx);
                            // Si la conexión que borramos existe, cerramos antes
                            if self.state.current_connection_idx != idx {
                                self.state.conn = None;
                            }
                            self.state.current_connection_idx = usize::MAX;
                            ui.close_menu();
                        }
                    });

                    // --> Al clicar sobre conexión, conectamos y listamos tablas <--
                    // Si estamos ya mostrando esta conexión, clicar sobre ella no lanza ninguna acción.
                    if button.clicked() && self.state.current_connection_idx != idx {
                        self.state.current_connection_idx = idx;

                        // Si no conexión o la que existe no es la que clico, la defino
                        let conn = RedisConnectionDefinition {
                            host: conn_definition.host.clone(),
                            port: conn_definition.port.clone(),
                        };

                        self.state.current_connection = conn.clone();
                        self.connect();
                    }
                });
            }

            if !connections_to_delete.is_empty() {
                app_st.connections = app_st
                    .connections
                    .iter()
                    .enumerate()
                    .filter(|(idx, _)| !connections_to_delete.contains(&idx))
                    .map(|(_, e)| e.to_owned())
                    .collect();
            }
        });
    }
}
