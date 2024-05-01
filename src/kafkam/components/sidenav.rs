// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use eframe::egui;
use egui_extras::{Column, TableBuilder};
use rdkafka::metadata::Metadata;
use rdkafka::producer::Producer;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;

use crate::{
    app_state::AppState,
    common::{icon_moon::IconMoon, internationalization::I18n, traits::Sidenav},
    kafkam::{
        presenter::{Kafka, KafkaConsumer, KafkaMessage, KafkaProducer},
        state::{Cluster, KafkaAppState, KafkaPanel},
        view::KafkaView,
    },
};

impl Sidenav for KafkaView {
    fn show_sidenav(
        &mut self,
        rt: &Runtime,
        ctx: &egui::Context,
        app_state: &mut AppState,
        i18n: &I18n,
    ) {
        egui::SidePanel::left("kafka_cluster_panel").show(ctx, |ui| {
            ui.menu_button(i18n.kafka_btn_add_connection.clone(), |ui| {
                self.add_new_cluster_menu(ui, &mut app_state.kafka.clusters, i18n);
            });

            let popup_id = ui.make_persistent_id("cluster-edit-window");
            let mut buttons = Vec::with_capacity(app_state.kafka.clusters.len());

            for (idx, cluster) in app_state.kafka.clusters.iter_mut().enumerate() {
                ui.collapsing(cluster.name.clone(), |ui| {
                    ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                        ui.horizontal(|ui| {
                            ui.monospace(format!("{}:{}", cluster.host, cluster.port));
                            let edit_btn = ui.button(IconMoon::Pencil.as_str());
                            if edit_btn.clicked() {
                                ui.memory_mut(|mem| mem.toggle_popup(popup_id));
                                self.state.selected_cluster_to_edit_idx = Some(idx);
                                self.state.tmp_cluster_config = cluster.clone();
                            }
                            buttons.push(edit_btn);
                        });

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
                                // TODO: Desconectar
                                Some(_) => (),
                                None => {
                                    self.connect_to_cluster(rt, broker, idx);
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
                                    KafkaConsumer::create_consumer(&broker, "random", false).await;

                                KafkaConsumer::subscribe(
                                    &consumer,
                                    &["prueba1"],
                                    Arc::clone(&data),
                                )
                                .await;
                                println!("After <-- should never arrive here");
                            });
                        }
                    });
                });
            }

            if let Some(cluster_to_edit_idx) = self.state.selected_cluster_to_edit_idx {
                // let selected_cluster = &app_state.kafka.clusters[cluster_to_edit_idx];

                egui::Window::new("Confirmar Exportación")
                // .open(&mut self.is_export_confirmation_open)
                // .title_bar(false) // Sin botón de cerrar ni título, pero no llamando al método open no se crea
                .collapsible(false)
                    .show(ctx, |ui| {
                        self.add_new_cluster_menu(ui, &mut app_state.kafka.clusters, i18n);
                });
                // egui::popup_below_widget(ui, popup_id, &buttons[cluster_to_edit_idx], |ui| {
                    // self.add_new_cluster_menu(ui, &mut app_state.kafka.clusters, i18n);
                // });
            }
        });
    }
}

impl KafkaView {
    fn add_new_cluster_menu(&mut self, ui: &mut egui::Ui, clusters: &mut Vec<Cluster>, i18n: &I18n) {
        ui.set_min_width(200.0);

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
                self.state.selected_cluster_to_edit_idx = None;
                ui.close_menu();
            }
            if ui.button(&i18n.kafka_edit_cluster_save).clicked() {
                // kafka_state
                match self.state.selected_cluster_to_edit_idx {
                    Some(idx) => {
                        clusters[idx] = self.state.tmp_cluster_config.clone();
                    }
                    None => {
                        clusters.push(self.state.tmp_cluster_config.clone());
                    }
                }

                self.state.clusters_metadata.push(None);
                self.state.tmp_cluster_config = Default::default();
                self.state.selected_cluster_to_edit_idx = None;
                ui.close_menu();
            }
        });
    }

    fn connect_to_cluster(&mut self, rt: &Runtime, broker: String, idx: usize) {
        let tx_cloned = self.state.tx.clone();

        // --> Conectamos con el clúster y recogemos metadatos y estadísticas <--
        rt.spawn(async move {
            let producer = KafkaProducer::stats_listener(&broker);
            let metadata = Kafka::extract_cluster_metadata_from_client(
                producer.client(),
            );
            if let Some(data) = metadata {
                let _ = tx_cloned
                    .send(KafkaMessage::ClusterMetadata((idx, data)))
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
        // println!("Closing stats_producer");
        // });
    }
}
