// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use super::{
    components::{
        show_cluster_configuration, show_clusters_metadata_info, show_messages_table, show_stats,
    },
    presenter::KafkaProducerPresenter,
    state::{KafkaConsumerMessage, KafkaLocalState, KafkaMessage},
};
use crate::{
    app_state::AppState,
    common::{internationalization::I18n, traits::Sidenav as _},
    kafkam::state::KafkaPanel,
};
use eframe::egui;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;

pub struct KafkaView {
    pub state: KafkaLocalState,
    pub stats_presenter: Option<KafkaProducerPresenter>,
    pub messages: Arc<Mutex<Vec<KafkaConsumerMessage>>>,
}

impl Default for KafkaView {
    fn default() -> Self {
        Self {
            state: KafkaLocalState::default(),
            messages: Arc::new(Mutex::new(Vec::default())),
            stats_presenter: Default::default(),
        }
    }
}

impl KafkaView {
    pub fn update(
        &mut self,
        ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        app_state: &mut AppState,
        rt: &Runtime,
        i18n: &I18n,
    ) {
        // =======================================
        // Preparación de cada ciclo
        // =======================================
        // --> Repintado continuo si estamos en subscripción <--
        if self.state.current_view == KafkaPanel::Subscribe {
            ctx.request_repaint();
        }

        if self.state.is_first_update {
            self.state.is_first_update = false;
            // for _ in app_state.kafka.clusters.iter() {
            // self.state.clusters_metadata.push(None);
            // }
        }

        // --> Recibimos mensaje <--
        while let Ok(msg) = self.state.rx.try_recv() {
            match msg {
                KafkaMessage::Str(data) => {
                    println!("Receiving data: {data:?}");
                }
                KafkaMessage::ClusterMetadata((idx, metadata, count)) => {
                    self.state.current_cluster_metadata = Some(metadata);
                    self.state.current_cluster_idx = idx;
                    self.state.clusters_metadata_count = count;
                } // No en uso porque actualizo variable compartido entre hilos,
                  // dejo por si cambio de idea.
                  // KafkaMessage::ConsumerMessage(_) => {}
            }
        }

        // =======================================
        // Panel Lateral
        // =======================================
        if app_state.kafka.show_sidebar {
            self.show_sidenav(rt, ctx, app_state, i18n);
        }

        // =======================================
        // Panel Central
        // =======================================
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.state.current_view == KafkaPanel::Brokers {
                // let current_cluster_metadata = self
                //     .state
                //     .clusters_metadata
                //     .get(self.state.current_cluster_idx);

                if let Some(ref metadata) = self.state.current_cluster_metadata {
                    show_clusters_metadata_info(ui, metadata, i18n);
                    show_cluster_configuration(ui, i18n);
                }
            } else if self.state.current_view == KafkaPanel::Topics {
                self.topics_admin(ui, i18n);
                ui.separator();
                // let current_cluster_metadata = self
                // .state
                // .clusters_metadata
                // .get(self.state.current_cluster_idx);

                if let Some(ref metadata) = self.state.current_cluster_metadata {
                    self.topics_stats(ui, metadata, i18n);
                    ui.separator();
                    self.show_topics_info(ui, metadata, i18n);
                }
            } else if self.state.current_view == KafkaPanel::Subscribe {
                let messages = self.messages.lock().unwrap();
                show_messages_table(ui, &messages);
            } else if self.state.current_view == KafkaPanel::Stats && self.stats_presenter.is_some()
            {
                show_stats(
                    ui,
                    &self.stats_presenter.as_ref().unwrap().stats.lock().unwrap(),
                );
            }
        });
    }
}
