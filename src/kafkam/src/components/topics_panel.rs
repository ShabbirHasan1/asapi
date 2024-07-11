// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use eframe::egui;
use log::info;

use rdkafka::error::KafkaError;
use rdkafka::metadata::Metadata;
use std::ops::RangeInclusive;
use tokio::runtime::Runtime;

use common::internationalization::I18n;
use components::heading_strong;
use components::widgets::ui_text_edit_singleline_hint;

use crate::admin as admin_presenter;
use crate::state::KafkaMessage;
use crate::view::KafkaView;

impl KafkaView {
    pub fn topics_admin(&mut self, rt: &Runtime, ui: &mut egui::Ui, i18n: &I18n) {
        ui.horizontal(|ui| {
            if ui.button(&i18n.kafka_create_topic).clicked() {
                self.state.new_topic.show = true;
            }
            if self.state.new_topic.show {
                self.create_topic_window(rt, ui, i18n);
            }

            if ui.button(&i18n.kafka_delete_topic).clicked() {
                self.state.delete_topic.show = true;
            }
            if self.state.delete_topic.show {
                self.delete_topic_window(rt, ui, i18n);
            }
        });

        // --> Mostramos estadísticas <--
    }
    pub fn topics_stats(&self, _ui: &mut egui::Ui, _metadata: &Metadata, _i18n: &I18n) {}

    fn delete_topic_window(&mut self, rt: &Runtime, ui: &mut egui::Ui, i18n: &I18n) {
        egui::Window::new(&i18n.kafka_delete_topic)
            .collapsible(false)
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    egui::ComboBox::from_id_source("kafka_delete_topic")
                        .selected_text(&self.state.delete_topic.selected_topic_name)
                        .show_ui(ui, |ui| {
                            self.state
                                .current_cluster_metadata
                                .as_ref()
                                .unwrap()
                                .topics()
                                .iter()
                                .map(|t| t.name().to_owned())
                                .for_each(|k| {
                                    // let k = t.name().to_owned(); // Es ~ a nivel de rendimiento y queda mejor con `map` que con este bind local.
                                    ui.selectable_value(
                                        &mut self.state.delete_topic.selected_topic_name,
                                        k.clone(),
                                        k,
                                    );
                                });
                        });
                    if ui.button(&i18n.kafka_cancel).clicked() {
                        self.state.delete_topic.show = false;
                    }
                    if ui.button(&i18n.kafka_delete_topic).clicked() {
                        self.state.delete_topic.show = false;
                        let broker_url = self
                            .state
                            .current_cluster_config
                            .as_ref()
                            .map(|md| format!("{}:{}", md.host, md.port));
                        let tx_cloned = self.state.tx.clone();
                        let ctx = ui.ctx().clone();
                        let name = self.state.delete_topic.selected_topic_name.clone();

                        rt.spawn(async move {
                            match broker_url {
                                Some(url) => {
                                    let result = admin_presenter::delete_topic(&url, &name).await;
                                    match result {
                                        Ok(_) => {
                                            let _ = tx_cloned
                                                .send(KafkaMessage::AskForMetadata(url))
                                                .await;
                                        }
                                        Err(err) => {
                                            let _ = tx_cloned.send(KafkaMessage::Error(err)).await;
                                        }
                                    }
                                }
                                None => todo!(),
                            }

                            ctx.request_repaint();
                        });
                    }
                });
            });
    }

    fn create_topic_window(&mut self, rt: &Runtime, ui: &mut egui::Ui, i18n: &I18n) {
        egui::Window::new(&i18n.kafka_create_topic)
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
                        let topic_config: Vec<(String, String)> = self
                            .state
                            .new_topic
                            .raw_config
                            .split_whitespace()
                            .collect::<Vec<&str>>()
                            .chunks(2)
                            .filter_map(|chunk| {
                                if chunk.len() == 2 {
                                    Some((chunk[0].to_owned(), chunk[1].to_owned()))
                                } else {
                                    None
                                }
                            })
                            .collect();

                        let broker_url = self
                            .state
                            .current_cluster_config
                            .as_ref()
                            .map(|md| format!("{}:{}", md.host, md.port));

                        match broker_url {
                            Some(url) => {
                                let name = self.state.new_topic.name.clone();
                                let num_partitions = self.state.new_topic.n_partitions;
                                let replication = self.state.new_topic.fixed_topic_replication;
                                let tx_cloned = self.state.tx.clone();
                                let ctx = ui.ctx().clone();

                                info!("{url} / {name} -- {num_partitions} {replication}");

                                rt.spawn(async move {
                                    // Puedo crear aquí el topic y enviarlo allí formado pero entiendo que
                                    // el método `create_topic` es el que tiene que saber todo acerca de
                                    // cómo hacerlo y que el cliente (este punto del código) simplemente
                                    // le pasa la información que tiene.
                                    let result = admin_presenter::create_topic(
                                        &url,
                                        name.as_str(),
                                        num_partitions,
                                        replication,
                                        topic_config,
                                    )
                                    .await;
                                    match result {
                                        Ok(_) => {
                                            // Si está ok, volvemos a pedir estadísticas de forma indirecta
                                            let _ = tx_cloned
                                                .send(KafkaMessage::AskForMetadata(url))
                                                .await;
                                        }
                                        Err(err) => {
                                            let _ = tx_cloned
                                                .send(KafkaMessage::Error(KafkaError::AdminOp(
                                                    err.1,
                                                )))
                                                .await;
                                        }
                                    }
                                    ctx.request_repaint();
                                });
                            }
                            None => {
                                self.state.last_error = Some(KafkaError::AdminOpCreation(
                                    "No se pudo extraer url del broker.".to_string(),
                                ));
                            }
                        }
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
                                    .first()
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
