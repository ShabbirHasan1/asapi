// -------------------------------------------------------------------------
// Copyright (C) 2023 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use eframe::egui;
use tokio;
use tokio::runtime::Runtime;
use common::fs;
use common::I18nRedis;
use components::result_panel::ui_response_panel;

use super::connection::{self, RedisMenu};
use super::state::RedisAppState;
use super::state::{PubSubState, RedisLocalState};

#[derive(Default)]
pub struct RedisView {
    pub state: RedisLocalState,
    pub pubsub: PubSubState,
}

impl RedisView {
    pub fn update(
        &mut self,
        ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        app_st: &mut RedisAppState,
        _rt: &Runtime,
        i18n: &I18nRedis,
    ) {
        // =======================================
        // Preparación de cada ciclo
        // =======================================
        // --> Repintado continuo si estamos en subscripción <--
        if self.state.selected_menu == RedisMenu::PubSub {
            ctx.request_repaint();
        }

        if self.state.must_scan {
            let option = self.state.selected_menu;
            let _ = connection::scan(&mut self.state, option);
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
            if self.state.selected_menu == RedisMenu::All {
                // --> Historia, movimiento y ejecución de comandos <--
                ui.horizontal(|ui| {
                    let command_textedit =
                        egui::TextEdit::singleline(&mut self.state.current_command);
                    let send_command_button = ui.button(&i18n.redis_send_command);
                    let command_input = ui.add_sized(ui.available_size(), command_textedit);

                    // ArrowUp    ->  dirección pasado
                    // ArrowDown  ->  dirección presente
                    if (command_input.lost_focus()
                        && ctx.input(|i| i.key_pressed(egui::Key::Enter)))
                        || send_command_button.clicked()
                    {
                        command_input.request_focus();
                        log::info!("{}", self.state.current_command);

                        self.state
                            .cmd_history
                            .push(self.state.current_command.clone());

                        let file_path = "redis-history";

                        // --> Ejecución de Comandos <--
                        match connection::run_user_string_command(
                            &self.state.current_connection.host,
                            &self.state.current_connection.port,
                            self.state.current_command.as_str(),
                        ) {
                            Ok(result) => {
                                let option = self.state.selected_menu;
                                let _ = connection::scan(&mut self.state, option);
                                self.state.last_result = Some(Ok(result));
                            }
                            // TODO: Change color
                            Err(e) => {
                                log::info!("Error: {:?}", e);
                                self.state.last_result = Some(Err(e));
                            }
                        }
                        if let Err(e) =
                            fs::append_to_file(file_path, &self.state.current_command.to_string())
                        {
                            log::error!("Error al escribir en el archivo: {}", e);
                        }
                        self.state.current_command.clear();
                        self.state.current_history_index = self.state.cmd_history.len();
                    } else if ctx.input(|i| i.key_pressed(egui::Key::ArrowUp))
                        && (self.state.current_history_index != 0)
                    {
                        self.state.reset_command();
                        self.state.current_history_index -= 1;
                        self.state.current_command.clone_from(&self.state.cmd_history[self.state.current_history_index]);
                        // self.state.current_command =
                            // self.state.cmd_history[self.state.current_history_index].clone();
                        // info!(
                        //     "UP {}  --  {}",
                        //     self.state.current_history_index, self.state.current_command
                        // );
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
                        // );
                    }
                });

                ui_response_panel(ui, &self.state.last_result);
            }

            // ===========================================
            // Bloques para mostrar unos u otros elementos
            // ===========================================
            match self.state.selected_menu {
                RedisMenu::All => self.show_all(ui, i18n),
                RedisMenu::String => {
                    ui.heading(egui::RichText::new("Strings").strong());
                    self.show_strings(ui, i18n);
                }
                RedisMenu::List => {
                    ui.heading(egui::RichText::new("Lists").strong());
                    self.show_lists(ui, i18n);
                }
                RedisMenu::Set => {
                    ui.heading(egui::RichText::new("Set").strong());
                    self.show_sets(ui, i18n);
                }
                RedisMenu::Hash => {
                    ui.heading(egui::RichText::new("Hashes").strong());
                    self.show_hashes(ui, i18n);
                }
                RedisMenu::SortedSet => {
                    ui.heading(egui::RichText::new("SortedSet").strong());
                    self.show_sorted_sets(ui, i18n);
                }
                RedisMenu::Json => {
                    ui.heading(egui::RichText::new("Json").strong());
                    self.show_json(ui, i18n);
                }
                RedisMenu::Stream => {
                    ui.heading(egui::RichText::new("Streams").strong());
                    self.show_streams(ui, i18n);
                }
                RedisMenu::PubSub => {
                    ui.heading(egui::RichText::new("PubSub").strong());
                    self.show_pubsub(ui, i18n);
                }
            };
        });
    }

    fn show_all(&mut self, ui: &mut egui::Ui, i18n: &I18nRedis) {
        egui::CollapsingHeader::new("Strings")
            .default_open(true)
            .show(ui, |ui| self.show_strings(ui, i18n));

        egui::CollapsingHeader::new("Lists")
            .default_open(true)
            .show(ui, |ui| self.show_lists(ui, i18n));

        egui::CollapsingHeader::new("Sets")
            .default_open(true)
            .show(ui, |ui| self.show_sets(ui, i18n));

        egui::CollapsingHeader::new("Hashes")
            .default_open(true)
            .show(ui, |ui| self.show_hashes(ui, i18n));

        egui::CollapsingHeader::new("Sorted Sets")
            .default_open(true)
            .show(ui, |ui| self.show_sorted_sets(ui, i18n));
    }
}
