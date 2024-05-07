// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use eframe::egui;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;

use crate::{
    common::{internationalization::I18n, traits::Sidenav as _},
    kafkam::state::KafkaPanel,
};

use super::producer::{self as producer_presenter, KafkaStatsProducerPresenter};
use super::{
    components::{
        // show_cluster_configuration,
        show_clusters_metadata_info,
        show_messages_table,
        show_stats,
        widgets::ui_error_panel,
    },
    state::{KafkaAppState, KafkaConsumerMessage, KafkaLocalState, KafkaMessage},
};

pub struct KafkaView {
    pub state: KafkaLocalState,
    pub stats_presenter: Option<KafkaStatsProducerPresenter>,
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
        app_st: &mut KafkaAppState,
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
        }

        // --> Recibimos mensaje <--
        while let Ok(msg) = self.state.rx.try_recv() {
            match msg {
                KafkaMessage::Str(data) => {
                    println!("Receiving data: {data:?}");
                }
                KafkaMessage::ClusterMetadata((idx, metadata, count)) => {
                    self.state.current_cluster_metadata = Some(metadata);
                    self.state.current_cluster_config = Some(app_st.clusters[idx].clone());
                    self.state.current_cluster_idx = idx;
                    self.state.clusters_metadata_count = count;
                }
                // No en uso porque actualizo variable compartido entre hilos,
                // dejo por si cambio de idea.
                // KafkaMessage::ConsumerMessage(_) => {}
                KafkaMessage::Error(kafka_error) => {
                    self.state.last_error = Some(kafka_error);
                }
                KafkaMessage::AskForMetadata(broker_url) => {
                    producer_presenter::get_cluster_metadata(
                        rt,
                        &self.state.tx,
                        ctx,
                        broker_url,
                        self.state.current_cluster_idx,
                    );
                }
            }
        }

        // =======================================
        // Panel Lateral
        // =======================================
        if app_st.show_sidebar {
            self.show_sidenav(rt, ctx, app_st, i18n);
        }

        // =======================================
        // Panel Central
        // =======================================
        egui::CentralPanel::default().show(ctx, |ui| {
            ui_error_panel(ui, &self.state.last_error);

            if self.state.current_view == KafkaPanel::Brokers {
                if let Some(ref metadata) = self.state.current_cluster_metadata {
                    show_clusters_metadata_info(ui, metadata, i18n);
                    // show_cluster_configuration(ui, i18n);
                }
            } else if self.state.current_view == KafkaPanel::Topics {
                self.topics_admin(rt, ui, i18n);
                ui.separator();

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
                if let Ok(arr) = self.stats_presenter.as_ref().unwrap().stats.lock() {
                    arr.first().map_or_else(
                        || (),
                        |fst| {
                            show_stats(ui, fst);
                        },
                    );
                }
            }
        });
    }
}
