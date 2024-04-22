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

use crate::{
    common::internationalization::I18n,
    mongom::{presenter, state::MongoMessage, view::MongoView},
};

impl MongoView {
    pub fn update_doc(&mut self, rt: &Runtime, ctx: &egui::Context, _i18n: &I18n) {
        let filter: Document =
            serde_json::from_str::<Document>(&self.state.current_selection.user_free_input)
                .map_or_else(
                    |e| {
                        self.state.last_error = Some(format!("{:?}", e));
                        doc! {}
                    },
                    |d| d,
                );
        let doc: Document =
            serde_json::from_str(&self.state.current_selection.replace_new_document).map_or_else(
                |e| {
                    self.state.last_error = Some(format!("{:?}", e));
                    doc! {}
                },
                |d| d,
            );
        // Guarda para no crear objeto vacío.
        if doc.is_empty() {
            return;
        }

        let tx = self.tx.clone();
        let ctx_cloned = ctx.clone();
        let client = self.state.conn.client.as_ref().unwrap().clone();
        let db_name = self.state.current_selection.db_name.to_owned();
        let col_name = self.state.current_selection.col_name.to_owned();
        let action = self.state.selected_action.clone();

        rt.spawn(async move {
            let result = presenter::update(
                &tx,
                &client,
                &db_name,
                &col_name,
                filter,
                doc,
                action,
            )
            .await;
            if let Err(err) = result {
                let _ = tx.send(MongoMessage::Error(format!("{:?}", err))).await;
            }
            ctx_cloned.request_repaint();
        });
    }
}
