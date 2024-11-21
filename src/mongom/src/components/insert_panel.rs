// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use bson::Document;
use eframe::egui;
use tokio::runtime::Runtime;

use common::I18nMongo as I18n;

use crate::{actions::MongoAction, presenter, state::MongoMessage, view::MongoView};

impl MongoView {
    pub fn insert(&mut self, rt: &Runtime, ctx: &egui::Context, i18n: &I18n) {
        let docs: Result<Vec<Document>, String> =
            if self.state.selected_action == MongoAction::InsertMany {
                serde_json::from_str::<Vec<Document>>(&self.state.current_selection.user_free_input)
                    .map_or_else(|e| Err(format!("{e:?}")), Ok)
            } else if self.state.selected_action == MongoAction::InsertOne {
                serde_json::from_str::<Document>(&self.state.current_selection.user_free_input)
                    .map_or_else(|e| Err(format!("{e:?}")), |d| Ok(vec![d]))
            } else {
                Err(i18n.mongo_wrong_action.to_owned())
            };

        let tx = self.tx.clone();
        let ctx_cloned = ctx.clone();

        match docs {
            Ok(docs) => {
                let client = self.state.conn.client.as_ref().unwrap().clone();
                let db_name = self.state.current_selection.db_name.to_owned();
                let col_name = self.state.current_selection.col_name.to_owned();
                let action = self.state.selected_action.clone();
                let i18n_cloned = i18n.clone();

                rt.spawn(async move {
                    let result = presenter::insert(
                        &tx,
                        &i18n_cloned,
                        &client,
                        &db_name,
                        &col_name,
                        docs,
                        action,
                    )
                    .await;
                    if let Err(e) = result {
                        let _ = tx.send(MongoMessage::Error(format!("{e:?}"))).await;
                    }
                    ctx_cloned.request_repaint();
                });
            }
            Err(e) => {
                rt.spawn(async move {
                    let _ = tx.send(MongoMessage::Error(e)).await;
                    ctx_cloned.request_repaint();
                });
            }
        }
    }
}
