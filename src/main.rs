// -------------------------------------------------------------------------
// Copyright (C) 2023 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

mod app_state;
mod common;
mod components;
mod httpm;

use eframe::egui;
use std::fs::{self, OpenOptions};

use crate::app_state::{AppState, ViewType};
use crate::common::fs::load_state;
use crate::httpm::view::HttpView;
use common::internationalization::language_selector;
use components::top_bar::AppTopBar;

/// Struct con los atributos que podemos pasar a cualquier parte de la apliación.
///
/// Fundamental el campo `AppState`: se serializa a json, es decir, cuando exportamos
/// a json, exportamos todo ese campo, y cuando importamos, lo que estamos haciendo
/// es rellenar esa estructura de datos con los datos que hay en el json.
pub struct Asapi {
    top_bar: AppTopBar,
    http: HttpView,
    app_state: AppState,
    rt: tokio::runtime::Runtime,
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
        let mut tmp = Vec::new();
        configure_text_styles(&cc.egui_ctx);
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open("redis-history")
        {
            Ok(_) => println!("redis-history created."),
            Err(_) => {
                let s = fs::read_to_string("redis-history").unwrap();
                s.lines()
                    .map(|l| l.to_string())
                    .for_each(|s| tmp.push(s.clone()));
            }
        }
        const FILE_NAME: &str = "asapi_workspaces.json";
        let state = match load_state(FILE_NAME) {
            Ok(state) => state,
            Err(err) => {
                eprintln!("{err:?}");
                AppState::default()
            }
        };

        Self {
            top_bar: AppTopBar::default(),
            http: HttpView::default(),
            app_state: state,
            rt: tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap(),
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

        let i18n = language_selector(self.app_state.app_config.language);

        egui::TopBottomPanel::top("decoration").show(ctx, |ui| {
            egui::warn_if_debug_build(ui);
            self.top_bar
                .update(ctx, ui, &self.rt, &mut self.app_state, i18n.clone());
        });

        match self.app_state.selected_view {
            ViewType::Http => {
                self.http
                    .update(ctx, _frame, &mut self.app_state.http, &self.rt, &i18n)
            }
        }
    }
}

fn configure_text_styles(ctx: &egui::Context) {
    let _ = ctx;
}

fn main() {
    let native_options = eframe::NativeOptions::default();

    let _result = eframe::run_native(
        "asapi",
        native_options,
        Box::new(|cc| Box::new(Asapi::new(cc))),
    );
}
