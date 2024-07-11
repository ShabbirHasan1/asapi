// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use common::internationalization::I18n;
use eframe::egui;
use log::info;
use std::ops::RangeInclusive;

use crate::{
    {
        presenters::pubsub::{publish_to_channel, subscribe_to_channel_std_thread},
        view::RedisView,
    },
};

impl RedisView {
    pub fn show_pubsub(&mut self, ui: &mut egui::Ui, i18n: &I18n) {
        egui::CollapsingHeader::new(format!("PubSub {}", &i18n.redis_channel_publish))
            .show_background(true)
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.pubsub.channel)
                            .hint_text(&i18n.redis_channel),
                    );
                    ui.add(
                        egui::TextEdit::singleline(&mut self.pubsub.value)
                            .hint_text(&i18n.redis_channel_value),
                    );
                    if ui.button(&i18n.redis_channel_publish).clicked() {
                        let publish_response = publish_to_channel(
                            &self.state.current_connection.host,
                            &self.state.current_connection.port,
                            &self.pubsub.channel,
                            &self.pubsub.value,
                        );
                        info!("Publish response: {:?}", publish_response);
                    }
                });
            });
        egui::CollapsingHeader::new(format!("PubSub {}", &i18n.redis_channel_subscribe))
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.pubsub.channel)
                            .hint_text(&i18n.redis_channel),
                    );
                    if ui.button(&i18n.redis_channel_subscribe).clicked() {
                        // Podríamos hacer esto arriba y activar/desactivar o mostrar/ocultar
                        // el botón, pero una (nimia) comprobación que nos ahorramos.
                        if self.pubsub.messages.contains_key(&self.pubsub.channel) {
                            info!("Already subscribed to {}", self.pubsub.channel);
                        } else {
                            let subscription = subscribe_to_channel_std_thread(
                                &self.state.current_connection.host,
                                &self.state.current_connection.port,
                                &self.pubsub.channel,
                                // rt,
                                &self.pubsub.tx,
                            );
                            match subscription {
                                Ok(_) => {
                                    self.pubsub
                                        .messages
                                        .insert(self.pubsub.channel.clone(), Vec::new());
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

        let min_n_cols = if self.pubsub.messages.is_empty() {
            0
        } else {
            1
        };
        ui.horizontal(|ui| {
            ui.label(&i18n.redis_n_columns);
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
                for (channel_idx, (chan, msg_ls)) in self.pubsub.messages.iter_mut().enumerate() {
                    let column_idx = channel_idx % self.pubsub.n_columns;
                    // Hacemos n x m, pero solo pintamos una vez cada uno de esos bucles, alli
                    // donde le toca (en qué columna) a cada lista de mensajes
                    if n_col == column_idx {
                        ui.horizontal(|ui| {
                            ui.heading(
                                egui::RichText::new(format!("{} {}", &i18n.redis_channel, chan))
                                    .strong(),
                            )
                            .context_menu(|ui| {
                                if ui.button(&i18n.redis_clean_messages).clicked() {
                                    msg_ls.clear();
                                }

                                // Para cerrar publicamos mensaje concreto en el canal que queremos cerrar.
                                if ui.button(&i18n.redis_close_subscription).clicked() {
                                    let _ = publish_to_channel(
                                        &self.state.current_connection.host,
                                        &self.state.current_connection.port,
                                        chan,
                                        "#break#",
                                    );
                                }

                                if ui.button(&i18n.redis_delete_subscription).clicked() {
                                    let _ = publish_to_channel(
                                        &self.state.current_connection.host,
                                        &self.state.current_connection.port,
                                        chan,
                                        "#break#",
                                    );
                                    msg_ls.clear();
                                    pubsub_channel_to_delete = chan.to_string();
                                }
                            });
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
}
