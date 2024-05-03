// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use eframe::egui;
use rdkafka::admin::{NewTopic, TopicReplication};
use rdkafka::metadata::Metadata;
use std::ops::RangeInclusive;

use crate::{
    common::internationalization::I18n, components::widgets::ui_text_edit_singleline_hint,
    heading_strong, kafkam::view::KafkaView,
};

impl KafkaView {
    pub fn topics_admin(&mut self, ui: &mut egui::Ui, i18n: &I18n) {
        ui.horizontal(|ui| {
            if ui.button(&i18n.kafka_create_topics).clicked() {
                self.state.new_topic.show = true;
            }
            if self.state.new_topic.show {
                self.show_create_topic_window(ui, i18n);
            }

            if ui.button(&i18n.kafka_delete_topics).clicked() {}

            if ui.button(&i18n.kafka_create_partitions).clicked() {}
        });

        // --> Mostramos estadísticas <--
    }
    pub fn topics_stats(&self, ui: &mut egui::Ui, metadata: &Metadata, i18n: &I18n) {}

    fn show_create_topic_window(&mut self, ui: &mut egui::Ui, i18n: &I18n) {
        egui::Window::new(&i18n.kafka_create_topics)
            .collapsible(false)
            .show(ui.ctx(), |ui| {
                ui_text_edit_singleline_hint(
                    ui,
                    &i18n.kafka_new_topic_name_hint,
                    &mut self.state.new_topic.name,
                );
                ui.label(&i18n.kafka_n_partitions_in_topic);

                // TODO: Estos dos valores de `10` son completamente arbitrarios,
                // no tengo los conocimientos ahora mismo (240502) para saber qué
                // valores son los más probables.
                ui.add(
                    egui::DragValue::new(&mut self.state.new_topic.n_partitions)
                        .clamp_range(RangeInclusive::new(0, 10)),
                );
                ui.label(&i18n.kafka_topic_replication);
                ui.add(
                    egui::DragValue::new(&mut self.state.new_topic.fixed_topic_replication)
                        .clamp_range(RangeInclusive::new(0, 10)),
                );
                ui_text_edit_singleline_hint(
                    ui,
                    &i18n.kafka_new_topic_config,
                    &mut self.state.new_topic.raw_config,
                );

                ui.horizontal(|ui| {
                    if ui.button(&i18n.kafka_cancel).clicked() {
                        self.state.new_topic.show = false;
                    }
                    if ui.button(&i18n.kafka_accept).clicked() {
                        self.state.new_topic.show = false;
                        let topic_config: Vec<(&str, &str)> = self
                            .state
                            .new_topic
                            .raw_config
                            .split_whitespace()
                            .collect::<Vec<&str>>()
                            .chunks(2)
                            .filter_map(|chunk| {
                                if chunk.len() == 2 {
                                    Some((chunk[0], chunk[1]))
                                } else {
                                    None
                                }
                            })
                            .collect();
                        let new_topic = NewTopic {
                            name: self.state.new_topic.name.as_str(),
                            num_partitions: self.state.new_topic.n_partitions,
                            replication: TopicReplication::Fixed(
                                self.state.new_topic.fixed_topic_replication,
                            ),
                            config: topic_config,
                        };
                    }
                });
            });
    }

    pub fn show_topics_info(&self, ui: &mut egui::Ui, metadata: &Metadata, i18n: &I18n) {
        egui::ScrollArea::both().show(ui, |ui| {
            for topic in metadata.topics() {
                heading_strong!(ui, topic.name());

                egui::CollapsingHeader::new(&i18n.kafka_topics_info)
                    .default_open(true)
                    .id_source(format!("Topics-resumen-{}", topic.name()))
                    .show_background(true)
                    .show(ui, |ui| {
                        egui::Grid::new(format!("Topics-resumen-Count-{}", topic.name()))
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

                egui::CollapsingHeader::new(&i18n.kafka_partitions_info)
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
