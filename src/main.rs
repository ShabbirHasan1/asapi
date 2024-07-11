// -------------------------------------------------------------------------
// Copyright (C) 2023 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

/// Punto de entrada a la apliación.
///
/// Donde se define el struct Asapi, sus implementaciones y aquellas instancias
/// que forman parte de su estado.
// Añadimos módulos para que se carguen en el proyecto.
mod app_state;
mod clickhousem;
mod httpm;
mod kafkam;
mod mongom;
mod top_bar;
extern crate common;
extern crate components;
extern crate redism;
extern crate sqlm;

use clickhousem::view::ClickHouseView;
use common::internationalization::language_selector;
use eframe::egui;
use kafkam::view::KafkaView;
use log::info;
use mongom::view::MongoView;
use redism::view::RedisView;
use sqlm::mysqlm::view::MySqlView;
use sqlm::pgm::view::PostgresView;
use sqlm::sqlitem::view::SQLiteView;
use std::fs::{self, OpenOptions};
use top_bar::AppTopBar;

use crate::app_state::{load_state, read_state_and_adapt, AppState, ViewType};
use crate::common::fs as asapi_fs;
use crate::httpm::view::HttpView;

/// Struct con los atributos que podemos pasar a cualquier parte de la apliación.
///
/// Fundamental el campo `AppState`: se serializa a json, es decir, cuando exportamos
/// a json, exportamos todo ese campo, y cuando importamos, lo que estamos haciendo
/// es rellenar esa estructura de datos con los datos que hay en el json.
pub struct Asapi {
    version: u16,
    top_bar: AppTopBar,
    app_state: AppState,
    rt: tokio::runtime::Runtime,
    http: HttpView,
    pg: PostgresView,
    sqlite: SQLiteView,
    mysql: MySqlView,
    redis: RedisView,
    mongo: MongoView,
    kafka: KafkaView,
    clickhouse: ClickHouseView,
}

impl Asapi {
    /// Creación de la apliación en si.
    ///
    /// Lo único que necesita para poder configurar la aplicación en sí.
    /// De la documentación de `egui` sobre `CreationContext`:
    ///
    ///  """" Data that is passed to [`AppCreator`] that can be used
    ///       to setup and initialize your app."""
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let version = 1;
        let mut tmp = Vec::new();
        configure_text_styles(&cc.egui_ctx);
        // En caso de que no podamos abrir la historia de redis, la creamos.
        if OpenOptions::new()
            .write(true)
            .create_new(true)
            .open("redis-history")
            .is_err()
        {
            let s = fs::read_to_string("redis-history").unwrap();
            s.lines()
                .map(|l| l.to_string())
                .for_each(|s| tmp.push(s.clone()));
        }

        const FILE_NAME: &str = "asapi_workspaces.json";
        // ==================================================
        // Manejo de versión
        // ==================================================
        let file_version = match asapi_fs::load_version(FILE_NAME) {
            Ok(state) => {
                log::info!("{state:?}");
                state
            }
            Err(err) => {
                log::error!("{err:?}");
                Default::default()
            }
        };

        log::info!(
            "Versiones\narchivo: {v}\naplicación: {version}",
            v = file_version.app_config.version
        );

        if file_version.app_config.version == version {
            log::info!("No hay que ajustar versión de configuración");
        } else {
            log::info!(
                "Hay que ajustar de {} a {version}",
                file_version.app_config.version
            );
        }

        // ==================================================
        // ==================================================

        let state = match load_state(FILE_NAME) {
            Ok(state) => state,
            Err(err) => {
                log::error!("{err:?}");
                read_state_and_adapt(FILE_NAME)
            }
        };

        Self {
            version,
            top_bar: AppTopBar::new(FILE_NAME, version),
            app_state: state,
            rt: tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap(),
            http: Default::default(),
            pg: Default::default(),
            mysql: Default::default(),
            sqlite: Default::default(),
            redis: Default::default(),
            mongo: Default::default(),
            kafka: Default::default(),
            clickhouse: Default::default(),
        }
    }
}

impl eframe::App for Asapi {
    /// Método del trait `App` que se llama cada vez que se va a repintar la aplicación.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let style = if self.app_state.app_config.dark_theme {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        ctx.set_visuals(style);

        // Aquí decido qué lenguage implementamos.
        let i18n = language_selector(self.app_state.app_config.language);

        egui::TopBottomPanel::top("decoration").show(ctx, |ui| {
            egui::warn_if_debug_build(ui);
            self.top_bar
                .update(ctx, ui, &self.rt, &mut self.app_state, &i18n);
        });

        match self.app_state.selected_view {
            ViewType::Http => {
                self.http
                    .update(ctx, _frame, &mut self.app_state.http, &self.rt, &i18n.http)
            }
            ViewType::Pg => {
                self.pg
                    .update(ctx, _frame, &mut self.app_state.pg, &self.rt, &i18n.sqlx)
            }
            ViewType::MySql => {
                self.mysql
                    .update(ctx, _frame, &mut self.app_state.mysql, &self.rt, &i18n.sqlx)
            }
            ViewType::SQLite => self.sqlite.update(
                ctx,
                _frame,
                &mut self.app_state.sqlite,
                &self.rt,
                &i18n.sqlx,
            ),
            ViewType::Mongo => {
                self.mongo
                    .update(ctx, _frame, &mut self.app_state.mongo, &self.rt, &i18n)
            }
            ViewType::Kafka => {
                self.kafka
                    .update(ctx, _frame, &mut self.app_state.kafka, &self.rt, &i18n)
            }
            ViewType::Redis => {
                self.redis
                    .update(ctx, _frame, &mut self.app_state.redis, &self.rt, &i18n)
            }
            ViewType::ClickHouse => self.clickhouse.update(
                ctx,
                _frame,
                &mut self.app_state.clickhouse,
                &self.rt,
                &i18n.clickhouse,
            ),
        }
    }
}

fn configure_text_styles(ctx: &egui::Context) {
    let _ = ctx;
}

fn main() {
    use env_logger::Env;
    let native_options = eframe::NativeOptions::default();
    // TODO: Según producción o build, `debug` o `info`.
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    info!("Inicio ASAPI");

    let _result = eframe::run_native(
        "asapi",
        native_options,
        Box::new(|cc| Box::new(Asapi::new(cc))),
    );
}
