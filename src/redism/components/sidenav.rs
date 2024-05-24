// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use eframe::egui::{self, Context};
use egui_extras::{Size, StripBuilder};
use std::collections::HashSet;

use crate::{
    common::internationalization::I18n,
    components::separators::ui_color_separator,
    info,
    redism::{
        connection::{self, create_conn, scan, RedisMenu},
        state::{PubSubState, RedisAppState, RedisConnectionDefinition, RedisLocalState},
        view::RedisView,
    },
};

use super::menu_option::selectable_and_info;

impl RedisView {
    pub fn show_sidenav(&mut self, ctx: &Context, app_st: &mut RedisAppState, i18n: &I18n) {
        egui::SidePanel::left("redis_side_panel").show(ctx, |ui| {
            // --> Decidimos qué mostrar <--
            ui.horizontal(|ui| {
                if ui.button("\u{27f3} Load").clicked() {
                    let _ = connection::scan(&mut self.state, RedisMenu::All);
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
            ui.menu_button(&i18n.sqlx.pg.btn_add_connection, |ui| {
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
                            self.show_data_structures(ui, i18n);
                        });
                    });
            } else if !self.state.hide_connections {
                self.show_connections(ui, app_st, i18n);
            } else if !self.state.hide_data_structures {
                self.show_data_structures(ui, i18n);
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

                    let connection_button = ui.add(raw_button);

                    // --> Menú contextual para manejo de las conexiones <--
                    connection_button.context_menu(|ui| {
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
                    if connection_button.clicked() && self.state.current_connection_idx != idx {
                        self.state.reset(RedisMenu::All);

                        let conn = RedisConnectionDefinition {
                            host: conn_definition.host.clone(),
                            port: conn_definition.port.clone(),
                        };

                        self.state.current_connection = conn.clone();
                        // Esto lo he traído de una función a incluir aquí, para ser más claro
                        // qué ocurre.
                        // Realmente no necesito `conn` en el estado, pero lo hago porque para
                        // saber si puedo conectar, y por hacer algo con esta conexión.
                        if let Ok(port) = self.state.current_connection.port.parse::<i16>() {
                            match create_conn(&self.state.current_connection.host, port) {
                                Ok(conn) => {
                                    self.state.current_connection_idx = idx;
                                    self.state.conn = Some(conn);
                                    self.state.last_result = None;
                                    let option = self.state.selected_menu;
                                    if let Err(err) = scan(&mut self.state, option) {
                                        self.state.last_result = Some(Err(err.to_string()));
                                    }
                                }
                                Err(err) => {
                                    self.state.conn = None;
                                    self.state.last_result = Some(Err(err.to_string()));
                                }
                            }
                        }
                        self.state.last_result = None;
                        let option = self.state.selected_menu;
                        if let Err(err) = scan(&mut self.state, option) {
                            self.state.last_result = Some(Err(err.to_string()));
                        }
                    }
                });
            }

            if !connections_to_delete.is_empty() {
                app_st.connections = app_st
                    .connections
                    .iter()
                    .enumerate()
                    .filter(|(idx, _)| !connections_to_delete.contains(idx))
                    .map(|(_, e)| e.to_owned())
                    .collect();
            }
        });
    }

    fn show_data_structures(&mut self, ui: &mut egui::Ui, i18n: &I18n) {
        egui::ScrollArea::vertical()
            .id_source("scroll_para_ds")
            .show(ui, |ui| {
                ui.separator();

                egui::Grid::new("mongo_all_data_structures")
                    .num_columns(2)
                    .show(ui, |ui| {
                        selectable_and_info(
                            ui,
                            &mut self.state,
                            i18n,
                            RedisMenu::All,
                            show_ds_info,
                        );
                        ui.end_row()
                    });

                ui_color_separator(ui, egui::Color32::LIGHT_GRAY);

                egui::Grid::new("mongo_data_structures")
                    .num_columns(2)
                    .show(ui, |ui| {
                        selectable_and_info(
                            ui,
                            &mut self.state,
                            i18n,
                            RedisMenu::String,
                            show_strings_info,
                        );
                        ui.end_row();

                        selectable_and_info(
                            ui,
                            &mut self.state,
                            i18n,
                            RedisMenu::List,
                            show_lists_info,
                        );
                        ui.end_row();

                        selectable_and_info(
                            ui,
                            &mut self.state,
                            i18n,
                            RedisMenu::Set,
                            show_sets_info,
                        );
                        ui.end_row();

                        selectable_and_info(
                            ui,
                            &mut self.state,
                            i18n,
                            RedisMenu::Hash,
                            show_hashes_info,
                        );
                        ui.end_row();

                        selectable_and_info(
                            ui,
                            &mut self.state,
                            i18n,
                            RedisMenu::SortedSet,
                            show_sorted_sets_info,
                        );
                        ui.end_row();
                    });

                ui.separator();

                egui::Grid::new("mongo_json_data_structure")
                    .num_columns(2)
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new("Info")
                                .color(egui::Color32::from_rgb(128, 128, 128)),
                        )
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

                egui::Grid::new("mongo_streams_data_structure")
                    .num_columns(2)
                    .show(ui, |ui| {
                        selectable_and_info(
                            ui,
                            &mut self.state,
                            i18n,
                            RedisMenu::Stream,
                            show_streams_info,
                        );
                    });

                egui::Grid::new("mongo_pubsub_data_structure")
                    .num_columns(2)
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new("Info")
                                .color(egui::Color32::from_rgb(128, 128, 128)),
                        )
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
            });
    }
}

// ------------------------------------------------------------
// Paneles de Información
// ------------------------------------------------------------
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
            ui.code("Name");
            ui.code("#Messages");
            ui.end_row();

            for (key, value) in st.messages.iter() {
                ui.label(key);
                ui.monospace(value.len().to_string());
                ui.end_row();
            }
        });
}

fn show_strings_info(ui: &mut egui::Ui, st: &RedisLocalState) {
    egui::Grid::new("redis_strings_info")
        .num_columns(1)
        .show(ui, |ui| {
            ui.code("#Strings");
            ui.end_row();

            ui.label(st.strings.len().to_string());
            ui.end_row();
        });
}

fn show_lists_info(ui: &mut egui::Ui, st: &RedisLocalState) {
    egui::Grid::new("redis_lists_info")
        .num_columns(1)
        .show(ui, |ui| {
            ui.code("List");
            ui.code("#Elements");
            ui.end_row();

            for (k, v) in &st.lists {
                ui.label(k);
                ui.monospace(v.len().to_string());
                ui.end_row();
            }
        });
}

fn show_sets_info(ui: &mut egui::Ui, st: &RedisLocalState) {
    egui::Grid::new("redis_sets_info")
        .num_columns(1)
        .show(ui, |ui| {
            ui.code("Sets");
            ui.code("#Elements");
            ui.end_row();

            for (k, v) in &st.sets {
                ui.label(k);
                ui.monospace(v.len().to_string());
                ui.end_row();
            }
        });
}

fn show_hashes_info(ui: &mut egui::Ui, st: &RedisLocalState) {
    egui::Grid::new("redis_hash_info")
        .num_columns(1)
        .show(ui, |ui| {
            ui.code("Hashes");
            ui.code("#Elements");
            ui.end_row();

            for (k, v) in &st.hashes {
                ui.label(k);
                ui.monospace(v.len().to_string());
                ui.end_row();
            }
        });
}

fn show_sorted_sets_info(ui: &mut egui::Ui, st: &RedisLocalState) {
    egui::Grid::new("redis_sorted_sets_info")
        .num_columns(1)
        .show(ui, |ui| {
            ui.code("SortedSet");
            ui.code("#Elements");
            ui.end_row();

            for (k, v) in &st.zsets {
                ui.label(k);
                ui.monospace(v.len().to_string());
                ui.end_row();
            }
        });
}

fn show_streams_info(ui: &mut egui::Ui, st: &RedisLocalState) {
    egui::Grid::new("redis_streams_info")
        .num_columns(1)
        .show(ui, |ui| {
            ui.code("Stream");
            ui.code("#Elements");
            ui.end_row();

            for (k, v) in &st.streams {
                ui.label(k);
                ui.monospace(v.len().to_string());
                ui.end_row();
            }
        });
}

fn show_ds_info(ui: &mut egui::Ui, st: &RedisLocalState) {
    egui::Grid::new("redis_data_st_info")
        .num_columns(2)
        .show(ui, |ui| {
            ui.code("Name");
            ui.code("#Elements");
            ui.end_row();

            ui.label("Strings");
            ui.monospace(st.strings.len().to_string());
            ui.end_row();

            ui.label("Lists");
            ui.monospace(st.lists.len().to_string());
            ui.end_row();

            ui.label("Sets");
            ui.monospace(st.sets.len().to_string());
            ui.end_row();

            ui.label("Hashes");
            ui.monospace(st.hashes.len().to_string());
            ui.end_row();

            ui.label("SortedSets");
            ui.monospace(st.zsets.len().to_string());
            ui.end_row();

            ui.label("Streams");
            ui.monospace(st.streams.len().to_string());
            ui.end_row();
        });
}
