// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use super::{
    presenter::{Kafka, KafkaConsumer, KafkaConsumerMessage, KafkaMessage, KafkaProducer},
    state::{Cluster, KafkaAppState, KafkaLocalState},
};
use crate::{
    app_state::AppState, common::internationalization::I18n, info, kafkam::state::KafkaPanel,
};
use eframe::egui;
use egui_extras::{Column, TableBuilder};
use rdkafka::metadata::Metadata;
use rdkafka::producer::Producer;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;

pub struct KafkaView {
    state: KafkaLocalState,
    messages: Arc<Mutex<Vec<KafkaConsumerMessage>>>,
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
                    info!("Receiving data: {data:?}");
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
        // --> Listado de conexiones (aunque solo se mantiene la última clicada activa) <--
        if app_state.kafka.show_sidebar {
            egui::SidePanel::left("kafka_cluster_panel").show(ctx, |ui| {
                ui.menu_button(i18n.kafka_btn_add_connection.clone(), |ui| {
                    self.edit_cluster_menu(ui, &mut app_state.kafka, i18n);
                });

                for (idx, cluster) in app_state.kafka.clusters.iter().enumerate() {
                    ui.collapsing(cluster.name.clone(), |ui| {
                        ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                            ui.label(format!("{}:{}", cluster.host, cluster.port));

                            let current_cluster_metadata =
                                self.state.clusters_metadata.get(idx).unwrap();

                            let btn_text = match current_cluster_metadata {
                                None => &i18n.kafka_btn_connect,
                                _ => &i18n.kafka_btn_disconnect,
                            };

                            let connect_btn =
                                egui::Button::new(btn_text).min_size(egui::vec2(200.0, 16.0));

                            if ui.add(connect_btn).clicked() {
                                self.state.current_cluster_idx = idx;
                                let broker = format!("{}:{}", cluster.host, cluster.port);

                                match current_cluster_metadata {
                                    Some(_) => (),
                                    None => {
                                        let tx_cloned = self.state.tx.clone();

                                        // --> Conectamos con el clúster y recogemos metadatos y estadísticas <--
                                        rt.spawn(async move {
                                            let producer = KafkaProducer::stats_listener(&broker);
                                            let metadata =
                                                Kafka::extract_cluster_metadata_from_client(
                                                    producer.client(),
                                                );
                                            if let Some(data) = metadata {
                                                let _ = tx_cloned
                                                    .send(KafkaMessage::ClusterMetadata((
                                                        idx, data,
                                                    )))
                                                    .await;
                                            }
                                        });

                                        // TODO: Mover a algún sitio donde se pida de forma explícita por parte
                                        // del usuario las estadísticas.
                                        // std::thread::spawn(move || {
                                        // let stats_producer = create_stats_producer(&broker);
                                        // let running = Arc::new(AtomicBool::new(true));
                                        // // flag::register_usize(SIGINT, Arc::clone(&running), 0).unwrap();
                                        // run_producer_loop(stats_producer, running);
                                        // info!("Closing stats_producer");
                                        // });
                                    }
                                }
                            }

                            // --> Selección entre una u otra vista y acciones <--
                            let show_brokers_btn = ui.add(egui::SelectableLabel::new(
                                self.state.current_cluster_idx == idx
                                    && self.state.current_view == KafkaPanel::Brokers,
                                &i18n.kafka_btn_show_brokers,
                            ));
                            let show_topics_btn = ui.add(egui::SelectableLabel::new(
                                self.state.current_cluster_idx == idx
                                    && self.state.current_view == KafkaPanel::Topics,
                                &i18n.kafka_btn_show_topics,
                            ));
                            let show_subscription_btn = ui.add(egui::SelectableLabel::new(
                                self.state.current_cluster_idx == idx
                                    && self.state.current_view == KafkaPanel::Subscribe,
                                &i18n.kafka_btn_show_subscription,
                            ));

                            // TODO: Hay que parar subscripción existente cuando hacemos click
                            if show_brokers_btn.clicked() {
                                self.state.current_cluster_idx = idx;
                                self.state.current_view = KafkaPanel::Brokers;
                            } else if show_topics_btn.clicked() {
                                self.state.current_cluster_idx = idx;
                                self.state.current_view = KafkaPanel::Topics;
                            } else if show_subscription_btn.clicked() {
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
                                    let consumer =
                                        KafkaConsumer::create_consumer(&broker, "random", false)
                                            .await;

                                    KafkaConsumer::subscribe(
                                        &consumer,
                                        &["prueba1"],
                                        Arc::clone(&data),
                                    )
                                    .await;
                                    info!("After <-- should never arrive here");
                                });
                            }
                        });
                    });
                }
            });
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
                    show_clusters_metadata_info(ui, metadata);
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

    fn edit_cluster_menu(
        &mut self,
        ui: &mut egui::Ui,
        kafka_state: &mut KafkaAppState,
        i18n: &I18n,
    ) {
        ui.set_min_width(200.0);
        // let mut tmp_cluster = Cluster::default();

        ui.horizontal(|ui| {
            ui.label(&i18n.kafka_edit_cluster_name_label);
            ui.text_edit_singleline(&mut self.state.tmp_cluster_config.name);
        });

        ui.horizontal(|ui| {
            ui.label(&i18n.kafka_edit_cluster_host_label);
            ui.text_edit_singleline(&mut self.state.tmp_cluster_config.host);
        });

        ui.horizontal(|ui| {
            ui.label(&i18n.kafka_edit_cluster_port_label);
            ui.text_edit_singleline(&mut self.state.tmp_cluster_config.port);
        });

        ui.horizontal(|ui| {
            if ui.button(&i18n.kafka_edit_cluster_cancel).clicked() {
                ui.close_menu();
            }
            if ui.button(&i18n.kafka_edit_cluster_save).clicked() {
                kafka_state
                    .clusters
                    .push(self.state.tmp_cluster_config.clone());
                self.state.clusters_metadata.push(None);
                self.state.tmp_cluster_config = Cluster::default();
                ui.close_menu();
            }
        });
    }
}

fn show_topics_metadata_info(ui: &mut egui::Ui, metadata: &Metadata) {
    for topic in metadata.topics() {
        // info!("  Topic: {}  Err: {:?}", topic.name(), topic.error());
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

fn show_clusters_metadata_info(ui: &mut egui::Ui, metadata: &Metadata) {
    ui.label("Cluster information:".to_string());
    ui.label(format!("  Broker count: {}", metadata.brokers().len()));
    ui.label(format!("  Topics count: {}", metadata.topics().len()));
    ui.label(format!(
        "  Metadata broker name: {}",
        metadata.orig_broker_name()
    ));
    ui.label(format!(
        "  Metadata broker id: {}\n",
        metadata.orig_broker_id()
    ));

    ui.label("Brokers:");
    for broker in metadata.brokers() {
        ui.label(format!(
            "  Id: {}  Host: {}:{}  ",
            broker.id(),
            broker.host(),
            broker.port(),
        ));
    }
}
