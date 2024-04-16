// -------------------------------------------------------------------------
// Copyright (C) 2023 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use std::ops::RangeInclusive;

use eframe::egui;
use egui::{Label, Sense};
use egui_json_tree::JsonTree;
use serde_json;
use tokio;
use tokio::runtime::Runtime;

use crate::common::fs::append_to_file;
use crate::common::internationalization::I18n;
use crate::error;
use crate::info;

use super::components::contextual_menus;
use super::presenter::{self, RedisMenu};
use super::state::RedisAppState;
use super::state::{PubSubState, RedisLocalState};
use super::utils::value_map_to_string_btree_map;

pub struct RedisView {
    pub state: RedisLocalState,
    pubsub: PubSubState,
}

impl Default for RedisView {
    fn default() -> Self {
        Self {
            state: RedisLocalState::default(),
            pubsub: PubSubState::default(),
        }
    }
}

impl RedisView {
    pub fn update(
        &mut self,
        ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        app_st: &mut RedisAppState,
        _rt: &Runtime,
        i18n: &I18n,
    ) {
        // =======================================
        // Preparación de cada ciclo
        // =======================================
        // --> Repintado continuo si estamos en subscripción <--
        if self.state.selected_menu == RedisMenu::PubSub {
            ctx.request_repaint();
        }

        if self.state.must_scan {
            let _ = presenter::scan(&mut self.state);
            self.state.must_scan = false;
        }
        if self.state.is_first_update {
            self.state.current_history_index = self.state.cmd_history.len();
            self.state.is_first_update = false;
        }

        // ===================================================================
        // Panel Lateral
        // ===================================================================
        if app_st.show_sidebar {
            self.show_sidenav(ctx, app_st, i18n);
        }

        // ===================================================================
        // Panel Central
        // ===================================================================
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.state.selected_menu != RedisMenu::PubSub {
                // --> Definición de conexión <--
                // ui.horizontal(|ui| {
                //     ui.add(egui::TextEdit::singleline(&mut self.state.current_connection.host).hint_text("host"));
                //     ui.add(egui::TextEdit::singleline(&mut self.state.current_connection.port).hint_text("port"));
                //     if ui.button("Connect").clicked() {
                //         if let Ok(port) = self.state.current_connection.port.parse::<i16>() {
                //             // app_state.redis.port = port;
                //             match presenter::create_conn(&self.state.current_connection.host, port) {
                //                 Ok(conn) => {
                //                     // TODO: Poner algún indicador visual de que tenemos conexión.
                //                     self.state.conn = Some(conn);
                //                     info!("Connected to Redis.");
                //                 }
                //                 Err(e) => {
                //                     self.state.conn = None;
                //                     info!("Error trying to connect to Redis: {:?}", e);
                //                 }
                //             }
                //         }
                //     }
                // });

                // --> Historia, movimiento y ejecución de comandos <--
                ui.horizontal(|ui| {
                    let command_textedit =
                        egui::TextEdit::singleline(&mut self.state.current_command);
                    let send_command_button = ui.button("Send Command");
                    let command_input = ui.add_sized(ui.available_size(), command_textedit);

                    // ArrowUp    ->  dirección pasado
                    // ArrowDown  ->  dirección presente
                    if (command_input.lost_focus()
                        && ctx.input(|i| i.key_pressed(egui::Key::Enter)))
                        || send_command_button.clicked()
                    {
                        command_input.request_focus();
                        info!("{}", self.state.current_command);

                        self.state
                            .cmd_history
                            .push(self.state.current_command.clone());

                        let file_path = "redis-history";

                        // --> Ejecución de Comandos <--
                        match presenter::run_command(
                            &self.state.current_connection.host,
                            &self.state.current_connection.port,
                            self.state.current_command.as_str(),
                        ) {
                            Ok(result) => {
                                info!("Result: {:?}", result);
                                self.state.command_last_result = result;
                                let _ = presenter::scan(&mut self.state);
                            }
                            // TODO: Change color
                            Err(e) => {
                                info!("Error: {:?}", e);
                                self.state.command_last_result = e;
                            }
                        }
                        if let Err(e) =
                            append_to_file(file_path, &self.state.current_command.to_string())
                        {
                            error!("Error al escribir en el archivo: {}", e);
                        }
                        self.state.current_command.clear();
                        self.state.current_history_index = self.state.cmd_history.len();
                    } else if ctx.input(|i| i.key_pressed(egui::Key::ArrowUp))
                        && (self.state.current_history_index != 0)
                    {
                        self.state.reset_command();
                        self.state.current_history_index -= 1;
                        self.state.current_command =
                            self.state.cmd_history[self.state.current_history_index].clone();
                        info!(
                            "UP {}  --  {}",
                            self.state.current_history_index, self.state.current_command
                        );
                    } else if ctx.input(|i| i.key_pressed(egui::Key::ArrowDown))
                        && (self.state.current_history_index != self.state.cmd_history.len())
                    {
                        self.state.reset_command();
                        self.state.current_history_index += 1;
                        match self.state.cmd_history.get(self.state.current_history_index) {
                            Some(s) => {
                                self.state.current_command = s.to_string();
                            }
                            None => {
                                self.state.current_command.clear();
                            }
                        }
                        info!(
                            "DOWN {}  --  {}",
                            self.state.current_history_index, self.state.current_command
                        );
                    }
                });

                if !self.state.command_last_result.is_empty() {
                    ui.label(&self.state.command_last_result);
                }
            }

            // ===========================================
            // Bloques para mostrar unos u otros elementos
            // ===========================================
            egui::ScrollArea::vertical().show(ui, |ui| {
                // --> Strings <--
                if self.state.selected_menu == RedisMenu::All
                    || self.state.selected_menu == RedisMenu::Hash
                {
                    egui::CollapsingHeader::new("Hashes")
                        .default_open(true)
                        .show(ui, |ui| self.hash_component(ui));
                }

                // --> Hashes <--
                if self.state.selected_menu == RedisMenu::All
                    || self.state.selected_menu == RedisMenu::String
                {
                    egui::CollapsingHeader::new("Strings")
                        .default_open(true)
                        .show(ui, |ui| self.string_component(ui));
                }

                // --> Streams (sólo mostrar/borrar/enviar, sin subscribirnos) <--
                if self.state.selected_menu == RedisMenu::All
                    || self.state.selected_menu == RedisMenu::Streams
                {
                    egui::CollapsingHeader::new("Streams")
                        .default_open(true)
                        .show(ui, |ui| self.stream_component(ui));
                }

                // --> PubSub <--
                if self.state.selected_menu == RedisMenu::PubSub {
                    egui::CollapsingHeader::new("PubSub Publish")
                        .default_open(true)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.pubsub.channel)
                                        .hint_text("Channel"),
                                );
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.pubsub.value)
                                        .hint_text("Value"),
                                );
                                if ui.button("Publish").clicked() {
                                    let publish_response = presenter::publish_to_channel(
                                        &self.state.current_connection.host,
                                        &self.state.current_connection.port,
                                        &self.pubsub.channel,
                                        &self.pubsub.value,
                                    );
                                    info!("Publish response: {:?}", publish_response);
                                }
                            });
                        });
                    egui::CollapsingHeader::new("PubSub Subscribe")
                        .default_open(true)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.pubsub.channel)
                                        .hint_text("Channel"),
                                );
                                if ui.button("Subscribe").clicked() {
                                    // Podríamos hacer esto arriba y activar/desactivar o mostrar/ocultar
                                    // el botón, pero una (nimia) comprobación que nos ahorramos.
                                    if self.pubsub.messages.contains_key(&self.pubsub.channel) {
                                        info!("Already subscribed to {}", self.pubsub.channel);
                                    } else {
                                        let subscription =
                                            presenter::subscribe_to_channel_std_thread(
                                                &self.state.current_connection.host,
                                                &self.state.current_connection.port,
                                                &self.pubsub.channel,
                                                // rt,
                                                &self.pubsub.tx,
                                            );
                                        match subscription {
                                            Ok(_) => {
                                                self.pubsub.messages.insert(
                                                    self.pubsub.channel.clone(),
                                                    Vec::new(),
                                                );
                                            }
                                            Err(err) => info!(
                                                "Error trying to subscribe to {}\nError {:?}",
                                                self.pubsub.channel, err
                                            ),
                                        }
                                    }
                                }
                            });
                        });

                    // --> Leemos mensajes <--
                    let max_n_cols = if self.pubsub.messages.len() > 3 {
                        4
                    } else {
                        self.pubsub.messages.len()
                    };

                    while let Ok(message) = self.pubsub.rx.try_recv() {
                        info!("Message: {:?}", message);
                        let channel = message.get_channel_name();
                        let msg_text: String = message.get_payload().unwrap();
                        let reference_to_messages = self.pubsub.messages.get_mut(channel);
                        match reference_to_messages {
                            Some(vs) => vs.push(msg_text),
                            // Aquí no debemos llegar nunca porque creamos la entrada al clicar `Subscribe`.
                            None => {
                                self.pubsub
                                    .messages
                                    .insert(channel.to_string(), vec![msg_text]);
                            }
                        }
                        // Cada vez que leemos
                    }

                    // --> Mostramos mensajes de PubSub <--
                    let mut pubsub_channel_to_delete = "".to_string();
                    let mut n_col = 0;

                    let min_n_cols = if self.pubsub.messages.len() > 0 { 1 } else { 0 };
                    ui.horizontal(|ui| {
                        ui.label("Number of Columns");
                        ui.add(
                            egui::DragValue::new(&mut self.pubsub.n_columns)
                                .clamp_range(RangeInclusive::new(min_n_cols, max_n_cols)),
                        );
                    });
                    // Necesario para cuando empiezan a haber datos
                    if min_n_cols == 1 && self.pubsub.n_columns == 0 {
                        self.pubsub.n_columns = max_n_cols;
                    }

                    ui.columns(self.pubsub.n_columns, |uis| {
                        for ui in uis {
                            for (channel_idx, (chan, msg_ls)) in
                                self.pubsub.messages.iter_mut().enumerate()
                            {
                                let column_idx = channel_idx % self.pubsub.n_columns;
                                // Hacemos n x m, pero solo pintamos una vez cada uno de esos bucles, alli
                                // donde le toca (en qué columna) a cada lista de mensajes
                                if n_col == column_idx {
                                    info!(
                                        "channel_idx {}, n_col {}, column_idx {}",
                                        channel_idx, n_col, column_idx
                                    );
                                    ui.horizontal(|ui| {
                                        ui.heading(format!("Channel {}", chan)).context_menu(
                                            |ui| {
                                                if ui.button("Clear Messages").clicked() {
                                                    msg_ls.clear();
                                                }

                                                // Para cerrar publicamos mensaje concreto en el canal que queremos cerrar.
                                                if ui.button("Close Subscription").clicked() {
                                                    let _ = presenter::publish_to_channel(
                                                        &self.state.current_connection.host,
                                                        &self.state.current_connection.port,
                                                        chan,
                                                        "#break#",
                                                    );
                                                }

                                                if ui.button("Delete Channel").clicked() {
                                                    let _ = presenter::publish_to_channel(
                                                        &self.state.current_connection.host,
                                                        &self.state.current_connection.port,
                                                        chan,
                                                        "#break#",
                                                    );
                                                    msg_ls.clear();
                                                    pubsub_channel_to_delete = chan.to_string();
                                                }
                                            },
                                        );
                                    });

                                    for msg in msg_ls {
                                        ui.label(format!("    {}", msg));
                                    }
                                    ui.separator();
                                }
                            }
                            n_col += 1;
                        }
                    });
                    let _ = self.pubsub.messages.remove(&pubsub_channel_to_delete);
                }
            });
        });
    }

    fn hash_component(&mut self, ui: &mut egui::Ui) {
        ui.set_width(ui.available_width());
        for (h_name, v) in &self.state.hashes {
            // --> Manejamos acciones sobre elemento que muestra nombre del hash
            ui.collapsing(h_name, |ui| {
                // TODO: Borrar todos en cascada con el menú contextual del hash.
                egui::Grid::new(h_name)
                    .spacing(egui::vec2(ui.spacing().item_spacing.x * 2.0, 0.0))
                    .show(ui, |ui| {
                        for (field_key, field_value) in v {
                            let field_label =
                                ui.add(Label::new(format!("    {} : {}", field_key, field_value)));

                            // --> Cada campo se puede borrar con menú contextual <--
                            field_label.context_menu(|ui| {
                                if ui.button("Delete").clicked() {
                                    match presenter::delete_hashkey(
                                        &self.state.current_connection.host,
                                        &self.state.current_connection.port,
                                        h_name,
                                        field_key,
                                    ) {
                                        Ok(s) => {
                                            self.state.must_scan = true;
                                            info!("{:?}", s);
                                        }
                                        Err(e) => info!("{:?}", e),
                                    }
                                    ui.close_menu();
                                }
                            });
                            ui.end_row();
                        }
                    });
            });
        }
    }

    fn string_component(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("key/value")
            .spacing(egui::vec2(ui.spacing().item_spacing.x * 2.0, 0.0))
            .show(ui, |ui| {
                for header in &self.state.strings {
                    ui.label(header.0.clone()).context_menu(|ui| {
                        if ui.button("Delete").clicked() {
                            match presenter::delete_key(
                                &self.state.current_connection.host,
                                &self.state.current_connection.port,
                                &header.0,
                            ) {
                                Ok(s) => {
                                    self.state.must_scan = true;
                                    info!("{:?}", s);
                                }
                                Err(e) => info!("{:?}", e),
                            }
                            ui.close_menu();
                        }
                    });
                    ui.label(" : ");
                    ui.label(header.1.clone());
                    ui.end_row();
                }
            });
    }

    fn stream_component(&mut self, ui: &mut egui::Ui) {
        {
            ui.set_width(ui.available_width());
            for (stream_name, v) in &self.state.streams {
                // ==> Gestión de Stream y todos los mensajes en él
                ui.collapsing(stream_name, |ui| {
                    for (idx, id) in v.iter().enumerate() {
                        // --> Gestión de cada mensaje <--
                        let label = match self.state.stream_id_values.get(id) {
                            Some(_) => ui.add(Label::new(id).sense(Sense::click())),
                            _ => ui
                                .add(Label::new(id).sense(Sense::click()))
                                .on_hover_text("Click to Open Stream and enabling resend"),
                        };

                        label.context_menu(|ui| {
                            // TODO: Aquí estoy cogiendo valores leídos
                            let option = self.state.stream_id_values.get(id);
                            self.state.must_scan = contextual_menus::stream_msg(
                                ui,
                                stream_name,
                                id.to_string().to_string(),
                                option,
                                &mut self.state.current_command,
                            );
                        });
                        if label.clicked() {
                            match self.state.stream_id_values.get(id) {
                                Some(_) => {
                                    self.state.stream_id_values.remove(id);
                                }
                                None => {
                                    // Hace falta esto porque cuando busco, si no es desde 0, el
                                    // que me devuelve es el siguiente al que selecciono, por
                                    // eso me hace falta el `idx-1`.
                                    let from_when = if idx == 0 { "0" } else { &v[idx - 1] };
                                    let _ = presenter::read_stream_id(
                                        &stream_name,
                                        from_when,
                                        &mut self.state.stream_id_values,
                                    );
                                    // if idx == 0 {
                                    //     let _ = presenter::read_stream_id(
                                    //         &stream_name,
                                    //         "0",
                                    //         &mut self.state.stream_id_values,
                                    //     );
                                    // } else {
                                    //     let _ = presenter::read_stream_id(
                                    //         &stream_name,
                                    //         &v[idx - 1],
                                    //         &mut self.state.stream_id_values,
                                    //     );
                                    // }
                                }
                            }
                        }
                        ui.end_row();
                        // TODO: Cambiar y almacenar los serde_json::Value para no estar
                        // haciendo el parseo continumamente. Eso nos permite volver a usar
                        // HashMap en vez de BTreeMap, aunque lo mejor sería comprobar el
                        // rendimiento al crear cada uno.
                        if let Some(value) = self.state.stream_id_values.get(id) {
                            // let value = serde_json::json!(value_map_to_string_map(value));
                            let value = serde_json::json!(value_map_to_string_btree_map(value));
                            JsonTree::new(id, &value).show(ui);
                        }
                    }
                })
                .header_response
                .context_menu(|ui| {
                    if ui.button("Delete").clicked() {
                        match presenter::delete_key(
                            &self.state.current_connection.host,
                            &self.state.current_connection.port,
                            &stream_name,
                        ) {
                            Ok(s) => {
                                self.state.must_scan = true;
                                info!("{:?}", s);
                            }
                            Err(e) => info!("{:?}", e),
                        }
                        ui.close_menu();
                    }
                });
            }
        }
    }

    pub fn connect(&mut self) {
        if let Ok(port) = self.state.current_connection.port.parse::<i16>() {
            match presenter::create_conn(&self.state.current_connection.host, port) {
                Ok(conn) => {
                    self.state.conn = Some(conn);
                }
                Err(e) => {
                    self.state.conn = None;
                }
            }
        }
    }
}
