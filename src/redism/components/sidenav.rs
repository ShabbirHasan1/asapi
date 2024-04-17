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

use crate::{
    common::internationalization::I18n,
    components::separators::ui_color_separator,
    info,
    redism::{
        presenter::{self, RedisMenu},
        state::{PubSubState, RedisAppState, RedisConnectionDefinition, RedisLocalState},
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
                            self.show_data_structures(ui);
                        });
                    });
            } else if !self.state.hide_connections {
                self.show_connections(ui, app_st, i18n);
            } else if !self.state.hide_data_structures {
                self.show_data_structures(ui);
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
                        if ui.button(&i18n.redis_close_connection).clicked() {
                            // Este crate no requiere que cerremos explícitamente. Es más
                            // por cómo es redis que por que en sí haga algo especial.
                            self.state.conn = None;
                            self.state.current_connection_idx = usize::MAX;
                            ui.close_menu();
                        }
                        if ui.button(&i18n.redis_delete_connection).clicked() {
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
                        self.state.reset();

                        // Si no conexión o la que existe no es la que clico, la defino
                        let conn = RedisConnectionDefinition {
                            host: conn_definition.host.clone(),
                            port: conn_definition.port.clone(),
                        };

                        self.state.current_connection = conn.clone();
                        self.connect();
                        self.state.command_last_result = "".to_owned();
                        // TODO: No compruebo la existencia de conexión porque `scan` crea la suya
                        // propia. Habría que pasársela para no necesitar dicha conexión local,
                        // aunque realmente podemos crear todas las que queramos.
                        if let Err(err) = presenter::scan(&mut self.state) {
                            // TODO: Mostrar con color rojo.
                            self.state.command_last_result = format!("ERROR {:?}", err);
                        }
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

    fn show_data_structures(&mut self, ui: &mut egui::Ui) {
        ui.separator();

        egui::Grid::new("mongo_all_data_structures")
            .num_columns(2)
            .show(ui, |ui| {
                ui.label(egui::RichText::new("Info").color(egui::Color32::from_rgb(128, 128, 128)))
                    .on_hover_ui(|ui| {
                        show_ds_info(ui, &self.state);
                    });
                ui.selectable_value(
                    &mut self.state.selected_menu,
                    RedisMenu::All,
                    format!("{:#?}", RedisMenu::All),
                );
                ui.end_row()
            });

        ui_color_separator(ui, egui::Color32::LIGHT_GRAY);

        egui::Grid::new("mongo_data_structures")
            .num_columns(2)
            .show(ui, |ui| {
                for option in RedisMenu::iter().filter(|e| {
                    **e != RedisMenu::All && **e != RedisMenu::PubSub && **e != RedisMenu::Json
                }) {
                    ui.label(
                        egui::RichText::new("Info").color(egui::Color32::from_rgb(128, 128, 128)),
                    )
                    .on_hover_ui(|ui| {
                        show_ds_info(ui, &self.state);
                    });

                    ui.selectable_value(
                        &mut self.state.selected_menu,
                        option.clone(),
                        format!("{:#?}", option),
                    );

                    ui.end_row();
                }
            });

        ui.separator();

        egui::Grid::new("mongo_json_data_structure")
            .num_columns(2)
            .show(ui, |ui| {
                ui.label(egui::RichText::new("Info").color(egui::Color32::from_rgb(128, 128, 128)))
                    .on_hover_ui(|ui| {
                        show_json_info(ui, &self.state);
                    });
                ui.selectable_value(
                    &mut self.state.selected_menu,
                    RedisMenu::Json,
                    format!("{:#?}", RedisMenu::Json),
                );
                ui.end_row()
            });

        egui::Grid::new("mongo_pubsub_data_structure")
            .num_columns(2)
            .show(ui, |ui| {
                ui.label(egui::RichText::new("Info").color(egui::Color32::from_rgb(128, 128, 128)))
                    .on_hover_ui(|ui| {
                        show_pubsub_info(ui, &self.pubsub);
                    });
                ui.selectable_value(
                    &mut self.state.selected_menu,
                    RedisMenu::PubSub,
                    format!("{:#?}", RedisMenu::PubSub),
                );
                ui.end_row()
            });
    }
}

fn show_json_info(ui: &mut egui::Ui, st: &RedisLocalState) {
    egui::Grid::new("redis_json_info")
        .num_columns(2)
        .show(ui, |ui| {
            ui.label("Object");
            ui.label("#Chars");
            ui.end_row();

            for (key, value) in st.jsons.iter() {
                ui.label(key);
                ui.monospace(value.len().to_string());
                ui.end_row();
            }
        });
}

fn show_pubsub_info(ui: &mut egui::Ui, st: &PubSubState) {
    egui::Grid::new("redis_pubsub_info")
        .num_columns(2)
        .show(ui, |ui| {
            ui.label("Name");
            ui.label("#Messages");
            ui.end_row();

            for (key, value) in st.messages.iter() {
                ui.label(key);
                ui.monospace(value.len().to_string());
                ui.end_row();
            }
        });
}

fn show_ds_info(ui: &mut egui::Ui, st: &RedisLocalState) {
    egui::Grid::new("redis_data_st_info")
        .num_columns(6)
        .show(ui, |ui| {
            ui.label("Name");
            ui.label("Data Type");
            ui.label("Column Type");
            ui.label("Is Nullable");
            ui.label("Column Default");
            ui.label("Column Key");
            ui.end_row();
        });
}
