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
use common::icon_moon::IconMoon;
use common::internationalization::{I18n, I18nOptions};

use super::app_state::{AppState, ViewType};

#[derive(Default)]
pub struct AppTopBar {
    show_settings: bool,
    is_export_confirmation_open: bool,
    file_name: &'static str,
    version: u16,
}

impl AppTopBar {
    pub fn new(file_name: &'static str, version: u16) -> Self {
        AppTopBar {
            version,
            show_settings: Default::default(),
            is_export_confirmation_open: Default::default(),
            file_name,
        }
    }

    pub fn update(
        self: &mut AppTopBar,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
        _rt: &Runtime,
        app_state: &mut AppState,
        i18n: &I18n,
    ) {
        if self.is_export_confirmation_open {
            egui::Window::new("Confirmar Exportación")
                // .open(&mut self.is_export_confirmation_open)
                // .title_bar(false) // Sin botón de cerrar ni título, pero no llamando al método open no se crea
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.label(&i18n.top_export_warning);
                    ui.horizontal(|ui| {
                        if ui.button("Exportar").clicked() {
                            let _ = fs::save_state(app_state, self.file_name, true);
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
                IconMoon::Sun.as_str()
            } else {
                IconMoon::Moon.as_str()
            };
            if ui.button(i18n.top_menu_config.clone()).clicked() {
                self.show_settings = !self.show_settings;
            }
            if ui.add(egui::Button::new(icon)).clicked() {
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
                    *app_state = match fs::load_state(self.file_name) {
                        Ok(state) => state,
                        Err(_e) => AppState::default(),
                    };
                    ui.close_menu();
                }
            });

            // --> Aquí ponemos botones de la barra <--

            ui.horizontal(|ui| {
                let http_btn =
                    ui.selectable_value(&mut app_state.selected_view, ViewType::Http, "Http");
                http_btn.context_menu(|ui| {
                    if ui
                        .add(egui::Button::new(&i18n.top_http_toggle_sidebar))
                        .clicked()
                    {
                        app_state.http.show_sidebar = !app_state.http.show_sidebar;
                        ui.close_menu();
                    }
                });

                let kafka_btn =
                    ui.selectable_value(&mut app_state.selected_view, ViewType::Kafka, "Kafka");
                kafka_btn.context_menu(|ui| {
                    if ui
                        .add(egui::Button::new(&i18n.top_kafka_toggle_sidebar_cluster))
                        .clicked()
                    {
                        app_state.kafka.show_sidebar = !app_state.kafka.show_sidebar;
                        ui.close_menu();
                    }
                });

                let pg_btn =
                    ui.selectable_value(&mut app_state.selected_view, ViewType::Pg, "Postgres");
                pg_btn.context_menu(|ui| {
                    if ui
                        .add(egui::Button::new(&i18n.top_pg_toggle_sidebar_connections))
                        .clicked()
                    {
                        app_state.pg.show_sidebar = !app_state.pg.show_sidebar;
                        ui.close_menu();
                    }
                    if ui
                        .checkbox(
                            &mut app_state.pg.performance_table,
                            &i18n.sqlx.pg.performance_table,
                        )
                        .on_hover_text(&i18n.sqlx.pg.info_performance_table)
                        .clicked()
                    {
                        app_state.pg.show_sidebar = !app_state.pg.show_sidebar;
                        ui.close_menu();
                    }
                });

                let mysql_btn = ui.selectable_value(
                    &mut app_state.selected_view,
                    ViewType::MySql,
                    "MySql/MariaDB",
                );
                mysql_btn.context_menu(|ui| {
                    if ui
                        .add(egui::Button::new(
                            &i18n.top_mysql_toggle_sidebar_connections,
                        ))
                        .clicked()
                    {
                        app_state.mysql.show_sidebar = !app_state.mysql.show_sidebar;
                        ui.close_menu();
                    }
                    if ui
                        .checkbox(
                            &mut app_state.mysql.performance_table,
                            &i18n.sqlx.mysql.performance_table,
                        )
                        .on_hover_text(&i18n.sqlx.mysql.info_performance_table)
                        .clicked()
                    {
                        ui.close_menu();
                    }
                });

                let sqlite_btn =
                    ui.selectable_value(&mut app_state.selected_view, ViewType::SQLite, "SQLite");
                sqlite_btn.context_menu(|ui| {
                    if ui
                        .add(egui::Button::new(
                            &i18n.top_sqlite_toggle_sidebar_connections,
                        ))
                        .clicked()
                    {
                        app_state.sqlite.show_sidebar = !app_state.sqlite.show_sidebar;
                        ui.close_menu();
                    }
                    if ui
                        .checkbox(
                            &mut app_state.sqlite.performance_table,
                            &i18n.sqlx.sqlite.performance_table,
                        )
                        .on_hover_text(&i18n.sqlx.sqlite.info_performance_table)
                        .clicked()
                    {
                        ui.close_menu();
                    }
                });

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

                let mongo_btn =
                    ui.selectable_value(&mut app_state.selected_view, ViewType::Mongo, "Mongo");
                mongo_btn.context_menu(|ui| {
                    if ui
                        .add(egui::Button::new(
                            &i18n.top_mongo_toggle_sidebar_connections,
                        ))
                        .clicked()
                    {
                        app_state.mongo.show_sidebar = !app_state.mongo.show_sidebar;
                        ui.close_menu();
                    }
                });

                let clickhouse_btn =
                    ui.selectable_value(&mut app_state.selected_view, ViewType::ClickHouse, "ClickHouse");
                mongo_btn.context_menu(|ui| {
                    if ui
                        .add(egui::Button::new(
                            &i18n.top_clickhouse_toggle_sidebar_connections,
                        ))
                        .clicked()
                    {
                        app_state.clickhouse.show_sidebar = !app_state.clickhouse.show_sidebar;
                        ui.close_menu();
                    }
                });

                // if http_btn.clicked()
                //     || pg_btn.clicked()
                //     || mysql_btn.clicked()
                //     || sqlite_btn.clicked()
                //     || mongo_btn.clicked()
                //     || redis_btn.clicked()
                //     || kafka_btn.clicked()
                //     || clickhouse_btn.clicked()
                // {
                //     // No tenemos que grabar cada vez.
                //     // let cloned_state = app_state.clone();
                //     // rt.spawn(async move {
                //     // let _ = async_save_state(&cloned_state, FILE_NAME).await;
                //     // });
                // }
            });
        });
    }
}
