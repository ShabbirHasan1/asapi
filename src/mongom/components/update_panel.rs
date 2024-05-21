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

use crate::{
    common::internationalization::I18n,
    mongom::{presenter, state::MongoMessage, view::MongoView},
};

impl MongoView {
    pub fn update_doc(&mut self, rt: &Runtime, ctx: &egui::Context, _i18n: &I18n) {
        let filter: Result<Document, serde_json::Error> =
            serde_json::from_str(&self.state.current_selection.user_free_input);
        let doc: Result<Document, serde_json::Error> =
            serde_json::from_str(&self.state.current_selection.replace_new_document);

        let tx = self.tx.clone();
        let ctx_cloned = ctx.clone();

        match (filter, doc) {
            (Ok(filter), Ok(doc)) => {
                let client = self.state.conn.client.as_ref().unwrap().clone();
                let db_name = self.state.current_selection.db_name.to_owned();
                let col_name = self.state.current_selection.col_name.to_owned();
                let action = self.state.selected_action.clone();

                rt.spawn(async move {
                    let result =
                        presenter::update(&tx, &client, &db_name, &col_name, filter, doc, action)
                            .await;
                    if let Err(err) = result {
                        let _ = tx.send(MongoMessage::Error(format!("Update Error\n{:?}", err))).await;
                    }
                    ctx_cloned.request_repaint();
                });
            }
            (Err(e1), Err(e2)) => {
                rt.spawn(async move {
                    let _ = tx
                        .send(MongoMessage::Error(format!("Update Error\n{e1:?}\n{e2:?}")))
                        .await;
                    ctx_cloned.request_repaint();
                });
            }
            (_, Err(e)) | (Err(e), _) => {
                rt.spawn(async move {
                    let _ = tx.send(MongoMessage::Error(format!("Update Error\n{e:?}"))).await;
                    ctx_cloned.request_repaint();
                });
            }
        }
    }
}
