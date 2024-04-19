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

use crate::app_state::{AppState, ViewType};
use crate::common::fs::{async_save_state, load_state, save_state};
use crate::common::internationalization::{I18n, I18nOptions};

pub struct AppTopBar {
    show_settings: bool,
    is_export_confirmation_open: bool,
}

impl Default for AppTopBar {
    fn default() -> Self {
        Self {
            show_settings: false,
            is_export_confirmation_open: false,
        }
    }
}

impl AppTopBar {
    pub fn update(
        self: &mut AppTopBar,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
        rt: &Runtime,
        app_state: &mut AppState,
        i18n: &I18n,
    ) {
        const FILE_NAME: &str = "asapi_workspaces.json";
        if self.is_export_confirmation_open {
            egui::Window::new("Confirmar Exportación")
                // .open(&mut self.is_export_confirmation_open)
                // .title_bar(false) // Sin botón de cerrar ni título, pero no llamando al método open no se crea
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.label(&i18n.top_export_warning);
                    ui.horizontal(|ui| {
                        if ui.button("Exportar").clicked() {
                            let _ = save_state(app_state, FILE_NAME);
                            self.is_export_confirmation_open = false;
                        }

                        if ui.button("Cancelar").clicked() {
                            self.is_export_confirmation_open = false;
                        }
                    });
                });
        }
        egui::menu::bar(ui, |ui| {
            // ui.set_width(200.0);
            ui.heading("ASAPI");

            let icon = if app_state.app_config.dark_theme {
                "\u{2600}" // Icono de sol
            } else {
                "\u{1F319}" // Icono de luna
            };
            if ui.button(i18n.top_menu_config.clone()).clicked() {
                self.show_settings = !self.show_settings;
            }
            if ui.add(egui::Button::new(icon).small()).clicked() {
                app_state.app_config.dark_theme = !app_state.app_config.dark_theme;
            }

            egui::Window::new(&i18n.top_menu_config)
                .open(&mut self.show_settings)
                .vscroll(true)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.radio_value(
                            &mut app_state.app_config.language,
                            I18nOptions::ES,
                            "Español",
                        );
                        ui.radio_value(
                            &mut app_state.app_config.language,
                            I18nOptions::EN,
                            "English",
                        );
                    });
                    ctx.settings_ui(ui);
                });

            ui.menu_button("File", |ui| {
                ui.set_min_width(220.0);
                ui.style_mut().wrap = Some(false);

                // On the web the browser controls the zoom
                #[cfg(not(target_arch = "wasm32"))]
                {
                    egui::gui_zoom::zoom_menu_buttons(ui);
                    ui.separator();
                }

                if ui
                    .add(egui::Button::new(&i18n.top_export_json_state))
                    .clicked()
                {
                    self.is_export_confirmation_open = true;
                    ui.close_menu();
                }
                if ui
                    .add(egui::Button::new(&i18n.top_import_json_state))
                    .clicked()
                {
                    *app_state = match load_state(FILE_NAME) {
                        Ok(state) => state,
                        Err(_e) => AppState::default(),
                    };
                    ui.close_menu();
                }
            });

            // --> Aquí ponemos botones de la barra <--

            ui.horizontal(|ui| {
                let redis_btn =
                    ui.selectable_value(&mut app_state.selected_view, ViewType::Redis, "Redis");
                redis_btn.context_menu(|ui| {
                    if ui
                        .add(egui::Button::new(&i18n.top_redis_toggle_sidebar))
                        .clicked()
                    {
                        app_state.redis.show_sidebar = !app_state.redis.show_sidebar;
                        ui.close_menu();
                    }
                });

                if redis_btn.clicked() {
                    let cloned_state = app_state.clone();
                    rt.spawn(async move {
                        let _ = async_save_state(&cloned_state, FILE_NAME).await;
                    });
                }
            });
        });
    }
}
