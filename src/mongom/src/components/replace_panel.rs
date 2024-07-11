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

use crate::{presenter, state::MongoMessage, view::MongoView};

impl MongoView {
    pub fn replace(&mut self, rt: &Runtime, ctx: &egui::Context) {
        let filter =
            serde_json::from_str::<Document>(&self.state.current_selection.user_free_input);
        let doc =
            serde_json::from_str::<Document>(&self.state.current_selection.replace_new_document);

        let tx = self.tx.clone();
        let ctx_cloned = ctx.clone();

        match (filter, doc) {
            (Ok(filter), Ok(doc)) => {
                let client = self.state.conn.client.as_ref().unwrap().clone();
                let db_name = self.state.current_selection.db_name.to_owned();
                let col_name = self.state.current_selection.col_name.to_owned();

                rt.spawn(async move {
                    let result =
                        presenter::replace(&tx, &client, &db_name, &col_name, filter, &doc).await;

                    if let Err(err) = result {
                        let _ = tx.send(MongoMessage::Error(format!("{:?}", err))).await;
                    }

                    ctx_cloned.request_repaint();
                });
            }
            (Ok(_), Err(e)) | (Err(e), Ok(_)) => {
                rt.spawn(async move {
                    let _ = tx
                        .send(MongoMessage::Error(format!("Replace Error\n{e:?}")))
                        .await;
                    ctx_cloned.request_repaint();
                });
            }
            (Err(e1), Err(e2)) => {
                rt.spawn(async move {
                    let _ = tx
                        .send(MongoMessage::Error(format!(
                            "Replace Error\n{e1:?}\n{e2:?}"
                        )))
                        .await;
                    ctx_cloned.request_repaint();
                });
            }
        }
    }
}
