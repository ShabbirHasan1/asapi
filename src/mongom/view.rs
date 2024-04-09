// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use super::actions::MongoAction;
use super::presenter;
use super::{components::sidenav::MongoSideNav, state::MongoMessage};
use crate::error;
use crate::mongom::parser::{build_mongo_query, pprint_bson};
use crate::mongom::state::MongoLocalState;
use crate::app_state::AppState;
use crate::common::internationalization::I18n;
use bson::{doc, Document};
use eframe::egui;
use serde_json::Value;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{Receiver, Sender};

pub struct MongoView {
    pub state: MongoLocalState,
    pub tx: Sender<MongoMessage>,
    rx: Receiver<MongoMessage>,
    first_render: bool,
}

impl Default for MongoView {
    fn default() -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(8);
        let mut state = MongoLocalState::default();
        state.rows_to_show = 100;

        Self {
            state,
            tx,
            rx,
            first_render: false,
        }
    }
}

impl MongoView {
    pub fn update(
        &mut self,
        ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        app_state: &mut AppState,
        rt: &Runtime,
        i18n: &I18n,
    ) {
        // =======================================
        // Acciones iniciales
        // =======================================
        #[cfg(debug_assertions)]
        if self.state.conn.client.is_some() && !self.first_render {
            if app_state.mongo.connections.len() > 0 {
                self.state.current_selection.conn_idx = 0;
                let tx = self.tx.clone();
                let client = self.state.conn.client.as_ref().unwrap().clone();

                let databases = rt.block_on(async {
                    presenter::list_database_names_in_connection(&tx, &client).await
                });

                if databases.len() > 0 {
                    self.state.current_selection.db_name = databases.last().unwrap().to_owned();
                    self.state.current_selection.db_idx = databases.len() - 1;
                    let db_name = self.state.current_selection.db_name.to_owned();

                    let collections = rt.block_on(async {
                        presenter::list_database_collections(&tx, &client, &db_name).await
                    });

                    if collections.len() > 0 {
                        self.state.current_selection.col_name =
                            collections.last().unwrap().to_owned();
                        self.state.current_selection.col_idx = collections.len() - 1;
                        let col_name = self.state.current_selection.col_name.to_owned();

                        let _ = rt.block_on(async {
                            presenter::list_collection_documents(&tx, &client, &db_name, &col_name)
                                .await
                        });
                        self.first_render = true;
                    }
                }
            }
        }

        while let Ok(message) = self.rx.try_recv() {
            // Lo proceso independiente para no tener que pasar ctx/rt/tx a la función.
            if message == MongoMessage::InsertionSuccess {
                self.find_all(rt, ctx);
            } else {
                self.process_message(message);
            }
        }

        // =======================================
        // Paneles laterales
        // =======================================
        MongoSideNav::show(
            &ctx,
            rt,
            &self.tx,
            &mut app_state.mongo,
            &mut self.state,
            i18n,
        );

        // =======================================
        // Panel central
        // =======================================
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.state.current_selection.col_idx != usize::MAX {
                // --> Aquí mostramos opciones que podemos ejecutar <--
                ui.set_width(ui.available_width());

                egui::CollapsingHeader::new(format!(
                    "{}  /  DB: {}  /  Col: {}",
                    i18n.mongo_actions,
                    self.state.current_selection.db_name,
                    self.state.current_selection.col_name
                ))
                .default_open(true)
                .show(ui, |ui| {
                    self.action_selector_header(ui);
                });
            }

            ui.separator();

            if let Some(ref error_message) = self.state.last_error {
                // TODO: Mostrar en color el mensaje de error.
                ui.label(error_message);
            }

            // TODO:
            // Según la acción seleccionada, tenemos que mostrar una u otra cosa para montar
            // el doc! a ejecutar. Ir poco a poco, abarcar todas las opciones de Mongo de
            // inicio sería un infierno.
            let show_user_free = self.state.current_selection.show_user_free_input;
            let compound_filter_available = self.state.selected_action == MongoAction::Find
                || self.state.selected_action == MongoAction::FindOne;

            if !show_user_free && compound_filter_available {
                self.compound_filter_constructor(rt, ctx, ui);
            } else {
                self.user_defined_filter_input(ctx, ui);
            }

            ui.horizontal(|ui| {
                // --> Recargar documentos de la colección <--
                if self.state.current_selection.col_idx != usize::MAX
                    && ui.button("\u{27f3}").clicked()
                {
                    self.state.last_error = None;
                    self.find_all(rt, ctx);
                }

                // --> Ejecutar <--
                if (show_user_free || !self.state.current_filter_value.is_empty())
                    && ui.button("\u{25b6}").clicked()
                {
                    self.state.last_error = None;

                    // Aunque le llame `filter`, es más cosas, por ejemplo el objeto a insertar
                    let filter: Document = if show_user_free {
                        let value = &self.state.current_selection.user_free_input;
                        serde_json::from_str(value).map_or(doc! {}, |d| d)
                    } else {
                        build_mongo_query(&self.state.filters)
                    };

                    pprint_bson(&filter);

                    match self.state.selected_action {
                        MongoAction::Find | MongoAction::FindOne => self.find(rt, ctx, filter),
                        MongoAction::InsertOne | MongoAction::InsertMany => {
                            let value = &self.state.current_selection.user_free_input;
                            let result: serde_json::Result<Value> = serde_json::from_str(value);
                            // Tenemos que reparsear para ver si es un array.
                            match result {
                                Ok(docs) => match docs {
                                    Value::Array(arr) => {
                                        let docs: Vec<Document> = arr
                                            .iter()
                                            .map(|a| match mongodb::bson::to_bson(a) {
                                                Ok(bs) => match bs {
                                                    bson::Bson::Document(doc) => doc,
                                                    _ => doc! {},
                                                },
                                                Err(_) => doc! {},
                                            })
                                            .collect();
                                        self.insert(rt, ctx, docs);
                                    }
                                    _ => {
                                        self.insert(rt, ctx, vec![filter]);
                                    }
                                },
                                Err(e) => {
                                    error!("{:?}", e);
                                    self.state.last_error =
                                        Some("Invalid document to Insert".into());
                                }
                            }
                        }
                        MongoAction::UpdateOne | MongoAction::UpdateMany => {}
                        MongoAction::DeleteOne | MongoAction::DeleteMany => {}
                        MongoAction::ReplaceOne | MongoAction::ReplaceMany => {}
                    }
                }
            });

            ui.separator();

            egui::ScrollArea::vertical()
                .id_source("mongo_central_panel")
                .show(ui, |ui| {
                    self.find_panel(ui);
                });
        });
    }

    fn process_message(&mut self, message: MongoMessage) {
        match message {
            // Recibimos las bases de datos que hay en la conexión clicada.
            MongoMessage::Databases(ddbb) => {
                self.state.db_names = ddbb;
                if !cfg!(debug_assertions) {
                    self.state.reset();
                }
            }
            // Nos llegan las colecciones que existen en la db seleccionada.
            MongoMessage::Collections(collections) => {
                self.state.current_db_collections = collections;
                if !cfg!(debug_assertions) {
                    self.state.reset();
                }
            }
            // Nos llegan los documentos encontrados según la búsqueda que hayamos hecho.
            MongoMessage::Documents((docs, jsons)) => {
                self.state.current_col_find_json_result = jsons;
                self.state.current_col_find_document_result = docs;
            }
            // Recibimos las claves del primer nivel de los documentos.
            MongoMessage::FirstLevelCollectionKeys(keys) => {
                self.state.current_available_keys = keys;
            }
            MongoMessage::Error(s) => {
                self.state.last_error = Some(s);
            }
            // Mensajes que no quiero procesar porque proceso antes de la llamada
            // a esta función. Dejo explicitado para que me dé error, si uso `_`
            // se me colará algún bug.
            MongoMessage::InsertionSuccess => (),
        }
    }
}
