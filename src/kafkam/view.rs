// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use super::{
    // components::clusters_panel::{self, show_clusters_metadata_info},
    components::show_clusters_metadata_info,
    presenter::{Kafka, KafkaConsumer, KafkaConsumerMessage, KafkaMessage, KafkaProducer},
    state::{Cluster, KafkaAppState, KafkaLocalState},
};
use crate::{
    app_state::AppState,
    common::{internationalization::I18n, traits::Sidenav as _},
    kafkam::state::KafkaPanel,
};
use eframe::egui;
use egui_extras::{Column, TableBuilder};
use rdkafka::metadata::Metadata;
use rdkafka::producer::Producer;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;

pub struct KafkaView {
    pub state: KafkaLocalState,
    pub messages: Arc<Mutex<Vec<KafkaConsumerMessage>>>,
}

impl Default for KafkaView {
    fn default() -> Self {
        Self {
            state: KafkaLocalState::default(),
            messages: Arc::new(Mutex::new(Vec::default())),
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
            for _ in app_state.kafka.clusters.iter() {
                self.state.clusters_metadata.push(None);
            }
        }

        // --> Recibimos mensaje <--
        while let Ok(msg) = self.state.rx.try_recv() {
            match msg {
                KafkaMessage::Str(data) => {
                    println!("Receiving data: {data:?}");
                }
                KafkaMessage::ClusterMetadata((idx, mtd)) => {
                    self.state.clusters_metadata.insert(idx, Some(mtd));
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
            let d1 = self
                .state
                .clusters_metadata
                .get(self.state.current_cluster_idx);

            // --> Mostramos la información en función de la selección del usuario <--
            if self.state.current_view == KafkaPanel::Brokers {
                // Control de si hay ya algún elemento en el vector.
                if let Some(Some(metadata)) = d1 {
                    // Control de si ya hemos insertado valores en vez del None.
                    show_clusters_metadata_info(ui, metadata, i18n);
                }
            } else if self.state.current_view == KafkaPanel::Topics {
                // Control de si hay ya algún elemento en el vector.
                if let Some(Some(metadata)) = d1 {
                    // Control de si ya hemos insertado valores en vez del None.
                    show_topics_metadata_info(ui, metadata);
                }
            } else if self.state.current_view == KafkaPanel::Subscribe {
                // Control de si ya hemos insertado valores en vez del None.
                let messages = self.messages.lock().unwrap();
                KafkaView::show_messages_table(ui, &messages);
            }
        });
    }

    // TODO: Hay que ver si es fácil ajustar para que sirva para todos los paneles.
    fn show_messages_table(ui: &mut egui::Ui, messages: &[KafkaConsumerMessage]) {
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
}

fn show_topics_metadata_info(ui: &mut egui::Ui, metadata: &Metadata) {
    for topic in metadata.topics() {
        ui.heading(topic.name());
        for partition in topic.partitions() {
            ui.label(format!(
                "     Partition: {}  Leader: {}  Replicas: {:?}  ISR: {:?}  Err: {:?}",
                partition.id(),
                partition.leader(),
                partition.replicas(),
                partition.isr(),
                partition.error()
            ));
        }
        ui.separator();
    }
}
