// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use bson::{doc, Document};
use eframe::egui;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{Receiver, Sender};

use common::internationalization::I18n;
use components::result_panel::ui_response_panel;

use crate::mongom::state::MongoLocalState;

use super::actions::MongoAction;
use super::presenter;
use super::state::MongoAppState;
use super::{components::sidenav::MongoSideNav, state::MongoMessage};

pub struct MongoView {
    sidenav: MongoSideNav,
    pub state: MongoLocalState,
    pub tx: Sender<MongoMessage>,
    rx: Receiver<MongoMessage>,
    first_render: bool,
}

impl Default for MongoView {
    fn default() -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(8);

        Self {
            sidenav: Default::default(),
            state: Default::default(),
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
        app_st: &mut MongoAppState,
        rt: &Runtime,
        i18n: &I18n,
    ) {
        // =======================================
        // Acciones iniciales
        // =======================================

        // Para debugear, me bloquea un poco porque si la conexión no es correcta (si al cluster)
        // pero credenciales malas, lo reintenta por este block.
        #[cfg(debug_assertions)]
        if self.state.conn.client.is_some()
            && !self.first_render
            && !app_st.connections.is_empty()
        {
            // self.state.current_selection.conn_idx = 0;
            let tx = self.tx.clone();
            let client = self.state.conn.client.as_ref().unwrap().clone();

            let databases = rt.block_on(async {
                presenter::list_database_names_in_connection(&tx, &client, i18n).await
            });

            if !databases.is_empty() {
                self.state.current_selection.db_name = databases.last().unwrap().to_owned();
                self.state.current_selection.db_idx = databases.len() - 1;
                let db_name = self.state.current_selection.db_name.to_owned();

                let collections = rt.block_on(async {
                    presenter::list_database_collections(&tx, &client, &db_name).await
                });

                if !collections.is_empty() {
                    self.state.current_selection.col_name = collections.last().unwrap().to_owned();
                    self.state.current_selection.col_idx = collections.len() - 1;
                    let col_name = self.state.current_selection.col_name.to_owned();

                    let _ = rt.block_on(async {
                        presenter::list_collection_documents(&tx, &client, &db_name, &col_name)
                            .await
                    });
                    self.first_render = true;
                }
            }

            // No quitar porque si no entra en bucle de intentar conectar y se queda congelada.
            self.first_render = true;
        }

        while let Ok(message) = self.rx.try_recv() {
            // Lo proceso independiente para no tener que pasar ctx/rt/tx a la función.
            if message == MongoMessage::InsertionSuccess
                || message == MongoMessage::DeleteSuccess
                || message == MongoMessage::ReplaceSuccess
                || message == MongoMessage::UpdateSuccess
            {
                self.find_all(rt, ctx);
            } else {
                self.process_message(app_st, message);
            }
        }

        // =======================================
        // Paneles laterales
        // =======================================
        self.sidenav
            .show(ctx, rt, &self.tx, app_st, &mut self.state, i18n);

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

            ui_response_panel(ui, &self.state.last_error);

            // TODO:
            // Según la acción seleccionada, tenemos que mostrar una u otra cosa para montar
            // el doc! a ejecutar. Ir poco a poco, abarcar todas las opciones de Mongo de
            // inicio sería un infierno.
            let show_user_free = self.state.current_selection.show_user_free_input;
            let compound_filter_available = self.state.selected_action == MongoAction::Find
                || self.state.selected_action == MongoAction::FindOne;

            if !show_user_free && compound_filter_available {
                self.compound_filter_constructor(rt, ctx, i18n, ui);
            } else {
                self.user_defined_filter_input(ctx, ui, i18n);
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
                if (show_user_free
                    || !self.state.current_filter_value.is_empty()
                    || !compound_filter_available) // Este caso es cuando queremos insertar/replacer/delete/update.
                    && ui.button("\u{25b6}").clicked()
                {
                    self.state.last_error = None;

                    match self.state.selected_action {
                        MongoAction::Find | MongoAction::FindOne => {
                            let docs: Vec<Document> =
                                if !compound_filter_available || show_user_free {
                                    let value = &self.state.current_selection.user_free_input;
                                    serde_json::from_str(value).map_or(vec![], |d| d)
                                } else {
                                    self.state
                                        .filters
                                        .iter()
                                        .map(|f| f.build_mongo_query())
                                        .collect::<Vec<Document>>()
                                };
                            self.find(rt, ctx, doc! {"$and": docs});
                        }
                        MongoAction::InsertOne | MongoAction::InsertMany => {
                            self.insert(rt, ctx, i18n);
                        }
                        MongoAction::DeleteOne | MongoAction::DeleteMany => {
                            self.delete(rt, ctx);
                        }
                        MongoAction::ReplaceOne => {
                            self.replace(rt, ctx);
                        }
                        MongoAction::UpdateOne | MongoAction::UpdateMany => {
                            self.update_doc(rt, ctx, i18n);
                        }
                    }
                }
            });

            ui.separator();

            egui::ScrollArea::vertical()
                .id_source("mongo_central_panel")
                .show(ui, |ui| {
                    self.find_panel(rt, &self.tx.clone(), ui, i18n);
                });
        });
    }

    fn process_message(&mut self, app_st: &mut MongoAppState, message: MongoMessage) {
        match message {
            // Recibimos las bases de datos que hay en la conexión clicada.
            MongoMessage::Databases(ddbb) => {
                self.state.db_names = ddbb;
                // if !cfg!(debug_assertions) {
                //     self.state.reset();
                // }
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
                self.state.last_error = Some(Err(s));
            }
            // `MongoMessage::InsertionSuccess/DeleteSuccess/ReplaceSuccess/UpdateSuccess` está
            // procesado arriba, no aquí bajo, de forma independiente, para no tener que pasar
            // ctx/rt/tx a la función.
            MongoMessage::DeleteSuccess
            | MongoMessage::InsertionSuccess
            | MongoMessage::ReplaceSuccess
            | MongoMessage::UpdateSuccess => {}
            MongoMessage::AddConnection(conn_definition) => {
                app_st.connections.push(conn_definition);
            }
            MongoMessage::EditConnection((idx, conn_definition)) => {
                app_st.connections[idx] = conn_definition;
            }
        }
    }
}
