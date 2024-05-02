// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use eframe::egui;
use std::sync::Arc;
use tokio::runtime::Runtime;

use crate::{
    error,
    kafkam::{
        presenter::KafkaConsumer,
        state::{Cluster, KafkaPanel},
        view::KafkaView,
    },
};

impl KafkaView {
    pub fn subscribe(&mut self, idx: usize, cluster: &mut Cluster, ctx: &egui::Context, rt: &Runtime) {
        self.state.current_cluster_idx = idx;
        self.state.current_view = KafkaPanel::Subscribe;
        let broker = format!("{}:{}", cluster.host, cluster.port);
        let data = Arc::clone(&self.messages);

        ctx.request_repaint();
        rt.spawn(async move {
            // TODO: Esto hay que llevarlo a la propia vista porque
            // hay que poder elegir group_id y topic(s). Incluso podemos
            // hacerlo como este listado, con un sub-tree en el menú
            // lateral donde introducir esos valores.
            let consumer = KafkaConsumer::create_consumer(&broker, "random", false).await;

            KafkaConsumer::subscribe(&consumer, &["prueba1"], Arc::clone(&data)).await;
            error!("After <-- should never arrive here");
        });
    }
}
