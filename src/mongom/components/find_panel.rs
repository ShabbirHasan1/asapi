// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use crate::mongom::{presenter, state::MongoMessage, view::MongoView};
use bson::{doc, Document};
use eframe::egui;
use egui_json_tree::JsonTree;
use tokio::runtime::Runtime;

impl MongoView {
    pub fn find_panel(&mut self, ui: &mut egui::Ui) {
        for (idx, doc) in self.state.current_col_find_json_result.iter().enumerate() {
            // TODO: 24/04/01
            // Usar idx hasta que seca cómo extraer el `_id` del documento.
            JsonTree::new(idx, doc).show(ui);
        }
    }

    pub fn find_all(&self, rt: &Runtime, ctx: &egui::Context) {
        self.find(rt, ctx, doc! {});
    }

    pub fn find(&self, rt: &Runtime, ctx: &egui::Context, filter: Document) {
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
