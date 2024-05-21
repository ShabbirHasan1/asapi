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

use crate::mongom::{actions::MongoAction, presenter, state::MongoMessage, view::MongoView};

impl MongoView {
    pub fn delete(&mut self, rt: &Runtime, ctx: &egui::Context) {
        let doc = serde_json::from_str::<Document>(&self.state.current_selection.user_free_input);
        let action = self.state.selected_action.clone();

        match doc {
            Ok(doc) => {
                self.delete_action(rt, ctx, doc, action);
            }
            Err(e) => {
                let tx = self.tx.clone();
                let ctx_cloned = ctx.clone();
                rt.spawn(async move {
                    let _ = tx.send(MongoMessage::Error(format!("{e:?}"))).await;

                    ctx_cloned.request_repaint();
                });
            }
        }
    }

    pub fn delete_action(
        &self,
        rt: &Runtime,
        ctx: &egui::Context,
        doc: Document,
        action: MongoAction,
    ) {
        let tx = self.tx.clone();
        let ctx_cloned = ctx.clone();
        let client = self.state.conn.client.as_ref().unwrap().clone();
        let db_name = self.state.current_selection.db_name.to_owned();
        let col_name = self.state.current_selection.col_name.to_owned();

        rt.spawn(async move {
            let result = presenter::delete(&tx, &client, &db_name, &col_name, doc, action).await;

            if let Err(err) = result {
                let _ = tx.send(MongoMessage::Error(format!("{:?}", err))).await;
            }

            ctx_cloned.request_repaint();
        });
    }
}
