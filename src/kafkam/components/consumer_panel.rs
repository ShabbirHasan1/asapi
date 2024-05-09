// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use eframe::egui;
use egui_extras::{Column, TableBuilder};
use std::sync::Arc;
use tokio::runtime::Runtime;

use crate::{
    kafkam::{
        presenter::KafkaConsumer,
        state::{Cluster, KafkaConsumerMessage},
        view::KafkaView,
    },
    qk_error,
};

pub fn show_messages_table(ui: &mut egui::Ui, messages: &[KafkaConsumerMessage]) {
    let n_columns = 6;
    // let text_height = egui::TextStyle::Body
    //     .resolve(ui.style())
    //     .size
    //     .max(ui.spacing().interact_size.y);

    egui::ScrollArea::horizontal().show(ui, |ui| {
        TableBuilder::new(ui)
            .auto_shrink(false)
            .striped(true)
            .resizable(true)
            // .max_scroll_height(f32::INFINITY)
            .columns(
                Column::initial(150.0).range(40.0..).resizable(true),
                n_columns - 1,
            )
            .column(Column::remainder())
            .min_scrolled_height(0.0)
            .sense(egui::Sense::click())
            .header(24.0, |mut header| {
                header.col(|ui| {
                    ui.strong("Key");
                });
                header.col(|ui| {
                    ui.strong("Topic");
                });
                header.col(|ui| {
                    ui.strong("Partition");
                });
                header.col(|ui| {
                    ui.strong("offset");
                });
                header.col(|ui| {
                    ui.strong("timestemp");
                });
                header.col(|ui| {
                    ui.strong("Payload");
                });
            })
            .body(|body| {
                // body.rows(text_height, messages.len(), |mut row| {
                body.rows(24.0, messages.len(), |mut row| {
                    let row_index = row.index();
                    let msg = messages.get(row_index).unwrap();

                    row.col(|ui| {
                        ui.label(&msg.key);
                    });
                    row.col(|ui| {
                        ui.label(&msg.topic);
                    });
                    row.col(|ui| {
                        ui.label(&msg.partition);
                    });
                    row.col(|ui| {
                        ui.label(&msg.offset);
                    });
                    row.col(|ui| {
                        ui.label(&msg.offset);
                    });
                    row.col(|ui| {
                        ui.label(&msg.payload);
                    });
                });
            });
    });
}

impl KafkaView {
    pub fn subscribe(
        &mut self,
        idx: usize,
        cluster: &mut Cluster,
        ctx: &egui::Context,
        rt: &Runtime,
    ) {
        self.state.current_cluster_idx = idx;
        let broker = format!("{}:{}", cluster.host, cluster.port);
        let data = Arc::clone(&self.messages);

        ctx.request_repaint();
        rt.spawn(async move {
            // TODO: Esto hay que llevarlo a la propia vista porque
            // hay que poder elegir group_id y topic(s). Incluso podemos
            // hacerlo como este listado, con un sub-tree en el menú
            // lateral donde introducir esos valores.
            let consumer = KafkaConsumer::create_async_consumer(&broker, None, false);

            KafkaConsumer::subscribe(&consumer.consumer, &["prueba1"], Arc::clone(&data)).await;
            qk_error!("After <-- should never arrive here");
        });
    }
}
