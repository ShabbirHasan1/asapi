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
    pub fn insert(&self, rt: &Runtime, ctx: &egui::Context, i18n: &I18n, docs: Vec<Document>) {
        let tx = self.tx.clone();
        let ctx_cloned = ctx.clone();
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
            if let Err(err) = result {
                let _ = tx.send(MongoMessage::Error(format!("{:?}", err))).await;
            }
            ctx_cloned.request_repaint();
        });
    }
}
