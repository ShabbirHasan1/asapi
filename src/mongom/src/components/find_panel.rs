// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use bson::{doc, oid::ObjectId, Document};
use eframe::egui;
use egui_json_tree::JsonTree;
use tokio::{runtime::Runtime, sync::mpsc::Sender};

use common::I18nMongo as I18n;

use crate::{actions::MongoAction, presenter, state::MongoMessage, view::MongoView};

impl MongoView {
    pub fn find_panel(
        &mut self,
        rt: &Runtime,
        tx: &Sender<MongoMessage>,
        ui: &mut egui::Ui,
        i18n: &I18n,
    ) {
        for (idx, doc) in self.state.current_col_find_json_result.iter().enumerate() {
            // TODO: 24/04/01
            // Usar idx hasta que seca cómo extraer el `_id` del documento.
            ui.horizontal(|ui| {
                ui.menu_button((1 + idx).to_string(), |ui| {
                    if ui.button(&i18n.mongo_doc_menu_copy).clicked() {
                        ui.ctx().copy_text(format!("{:?}", doc));
                        ui.close_menu();
                    }

                    // --> Borrar Fila <--
                    if ui.button(&i18n.mongo_doc_menu_delete_by_id).clicked() {
                        let id_string = doc.get("_id");
                        let oid = id_string
                            .and_then(|s| s.as_str())
                            .and_then(|s| ObjectId::parse_str(s).ok());

                        match oid {
                            Some(oid) => {
                                self.delete_action(
                                    rt,
                                    ui.ctx(),
                                    doc! { "_id": oid },
                                    MongoAction::DeleteOne,
                                );
                            }
                            None => {
                                let msg = format!("{:?} no parseable a ObjectId", &id_string);
                                let tx_cloned = tx.clone();
                                rt.spawn(async move {
                                    let _ = tx_cloned.send(MongoMessage::Error(msg)).await;
                                });
                            }
                        }

                        ui.close_menu();
                    }
                });
                JsonTree::new(idx, doc).show(ui);
            });
        }
    }

    pub fn find_all(&self, rt: &Runtime, ctx: &egui::Context) {
        self.find(rt, ctx, doc! {});
    }

    pub fn find(&self, rt: &Runtime, ctx: &egui::Context, filter: Document) {
        if self.state.conn.client.is_none() {
            return;
        }

        let tx = self.tx.clone();
        let ctx_cloned = ctx.clone();
        let client = self.state.conn.client.as_ref().unwrap().clone();
        let db_name = self.state.current_selection.db_name.to_owned();
        let col_name = self.state.current_selection.col_name.to_owned();
        let action = self.state.selected_action.clone();

        rt.spawn(async move {
            let result = presenter::find(&tx, &client, &db_name, &col_name, filter, action).await;
            if let Err(err) = result {
                let _ = tx.send(MongoMessage::Error(format!("{:?}", err))).await;
            }
            ctx_cloned.request_repaint();
        });
    }
}
