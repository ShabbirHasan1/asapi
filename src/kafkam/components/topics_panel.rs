// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use eframe::egui;
use rdkafka::metadata::Metadata;

use crate::{common::internationalization::I18n, heading_strong, kafkam::view::KafkaView};

impl KafkaView {
    pub fn show_topics(&self, ui: &mut egui::Ui, metadata: &Metadata, i18n: &I18n) {
        self.topics_admin(ui, i18n);
        ui.separator();
        self.show_topics_info(ui, metadata, i18n);
    }

    fn topics_admin(&self, ui: &mut egui::Ui, i18n: &I18n) {
        ui.horizontal(|ui| if ui.button(&i18n.kafka_create_topic).clicked() {});
    }

    fn show_topics_info(&self, ui: &mut egui::Ui, metadata: &Metadata, i18n: &I18n) {
        egui::ScrollArea::both().show(ui, |ui| {
            for topic in metadata.topics() {
                heading_strong!(ui, topic.name());

                egui::CollapsingHeader::new("Topcis-resumen")
                    .default_open(true)
                    .id_source(format!("Topcis-resumen-{}", topic.name()))
                    .show_background(true)
                    .show(ui, |ui| {
                        egui::Grid::new(format!("Topcis-resumen-Count-{}", topic.name()))
                            .num_columns(4)
                            .show(ui, |ui| {
                                ui.monospace(&i18n.kafka_n_messages_in_topic);
                                ui.label(
                                    self.state
                                        .clusters_metadata_count
                                        .get(topic.name())
                                        .unwrap_or(&0)
                                        .to_string(),
                                );
                                ui.monospace(&i18n.kafka_n_partitions_in_topic);
                                ui.label(topic.partitions().len().to_string());
                                ui.end_row();

                                ui.monospace(&i18n.kafka_replication_factor);
                                let replication_factor = topic
                                    .partitions()
                                    .get(0)
                                    .map(|p| p.replicas().len())
                                    .unwrap_or_default();
                                ui.label(replication_factor.to_string());

                                ui.end_row();
                            });
                    });

                egui::CollapsingHeader::new("Particiones")
                    .default_open(false)
                    .id_source(format!("Particiones-{}", topic.name()))
                    .show_background(true)
                    .show(ui, |ui| {
                        egui::Grid::new(topic.name())
                            .num_columns(10)
                            .show(ui, |ui| {
                                for partition in topic.partitions() {
                                    ui.monospace("Partition");
                                    ui.label(partition.id().to_string());
                                    ui.monospace("Leader");
                                    ui.label(partition.leader().to_string());
                                    ui.monospace("Replicas");
                                    ui.label(format!("{r:?}", r = partition.replicas()));
                                    ui.monospace("ISR");
                                    ui.label(format!("{i:?}", i = partition.isr()));

                                    if let Some(err) = partition.error() {
                                        ui.monospace("Err");
                                        ui.label(format!("{err:?}"));
                                    }

                                    ui.end_row();
                                }
                            });
                    });

                ui.separator();
            }
        });
    }
}
