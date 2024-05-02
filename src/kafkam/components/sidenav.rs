// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use std::{collections::HashMap, time::Duration};
use eframe::egui::{self, Context, Response};
use rdkafka::producer::Producer;
use tokio::runtime::Runtime;

use crate::{
    app_state::AppState,
    common::{icon_moon::IconMoon, internationalization::I18n, traits::Sidenav},
    info,
    kafkam::{
        presenter::{KafkaConsumer, KafkaProducer},
        state::{Cluster, KafkaMessage, KafkaPanel},
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
                self.edit_cluster_menu(ui, &mut app_state.kafka.clusters, i18n);
            });

            let popup_id = ui.make_persistent_id("cluster-edit-window");
            let mut buttons: Vec<Response> = Vec::with_capacity(app_state.kafka.clusters.len());

            let mut idx_to_delete = usize::MAX;
            for (idx, cluster) in app_state.kafka.clusters.iter_mut().enumerate() {
                egui::CollapsingHeader::new(cluster.name.clone())
                    .show_background(true)
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                            idx_to_delete = self.cluster_connection_row(
                                ui,
                                idx,
                                i18n,
                                rt,
                                popup_id,
                                cluster,
                                &mut buttons,
                            );

                            // --> Selección entre una u otra vista y acciones <--
                            let show_brokers_btn = ui.add(egui::SelectableLabel::new(
                                self.state.current_cluster_idx == idx
                                    && self.state.current_cluster_idx != usize::MAX
                                    && self.state.current_view == KafkaPanel::Brokers,
                                &i18n.kafka_btn_show_brokers,
                            ));
                            let show_topics_btn = ui.add(egui::SelectableLabel::new(
                                self.state.current_cluster_idx == idx
                                    && self.state.current_cluster_idx != usize::MAX
                                    && self.state.current_view == KafkaPanel::Topics,
                                &i18n.kafka_btn_show_topics,
                            ));
                            let show_subscription_btn = ui.add(egui::SelectableLabel::new(
                                self.state.current_cluster_idx == idx
                                    && self.state.current_cluster_idx != usize::MAX
                                    && self.state.current_view == KafkaPanel::Subscribe,
                                &i18n.kafka_btn_show_subscription,
                            ));

                            // TODO: Hay que parar subscripción existente cuando hacemos click
                            if show_brokers_btn.clicked() {
                                self.state.current_view = KafkaPanel::Brokers;
                            } else if show_topics_btn.clicked() {
                                self.state.current_view = KafkaPanel::Topics;
                            } else if show_subscription_btn.clicked() {
                                self.subscribe(idx, cluster, ctx, rt);
                            }
                        });
                    });
            }

            if idx_to_delete < app_state.kafka.clusters.len() {
                app_state.kafka.clusters.remove(idx_to_delete);
            }

            if self.state.selected_cluster_to_edit_idx.is_some() {
                egui::Window::new(&i18n.kafka_edit_cluster)
                    .collapsible(false)
                    .show(ctx, |ui| {
                        self.edit_cluster_menu(ui, &mut app_state.kafka.clusters, i18n);
                    });
            }
        });
    }
}

impl KafkaView {
    /// Editamos o añadimos clúster
    ///
    /// Según si tmp_cluster_config tenga datos o no, se crea o se edita.
    /// Por eso es necesario pasar los clusters, cuando si fuese editar
    /// sería suficiente con pasar el clúster que queremos editar.
    fn edit_cluster_menu(&mut self, ui: &mut egui::Ui, clusters: &mut Vec<Cluster>, i18n: &I18n) {
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

    fn get_cluster_metadata(&self, rt: &Runtime, ctx: &Context, broker_url: String, idx: usize) {
        let tx_cloned = self.state.tx.clone();
        let ctx_cloned = ctx.clone();

        // --> Conectamos con el clúster y recogemos metadatos y estadísticas <--
        rt.spawn(async move {
            let producer = KafkaProducer::stats_listener(&broker_url);
            let client = producer.client();
            // let client = KafkaConsumer::create_consumer(&broker_url, "fake", false).await;
            let metadata = client.fetch_metadata(None, Duration::from_secs(20)).ok();

            if let Some(data) = metadata {
                let mut count: HashMap<String, i64> = HashMap::default();
                for topic in data.topics() {
                    let mut message_count: i64 = 0;
                    for partition in topic.partitions() {
                        let (low, high) = client
                            .fetch_watermarks(topic.name(), partition.id(), Duration::from_secs(1))
                            .unwrap_or((-1, -1));

                        message_count += high - low;
                    }
                    count.insert(topic.name().to_owned(), message_count);
                }

                let _ = tx_cloned
                    .send(KafkaMessage::ClusterMetadata((idx, data, count)))
                    .await;

                let _groups = KafkaConsumer::create_async_consumer(&broker_url, None, false).groups(None);
            }

            ctx_cloned.request_repaint();
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

    fn cluster_connection_row(
        &mut self,
        ui: &mut egui::Ui,
        idx: usize,
        i18n: &I18n,
        rt: &Runtime,
        popup_id: egui::Id,
        cluster: &mut Cluster,
        buttons: &mut Vec<Response>,
    ) -> usize {
        let mut tmp = usize::MAX;

        ui.horizontal(|ui| {
            ui.monospace(format!("{}:{}", cluster.host, cluster.port));
            let current_cluster_metadata = self.state.clusters_metadata.get(idx).unwrap();

            match current_cluster_metadata {
                Some(_) if idx == self.state.current_cluster_idx => {
                    ui.add_enabled(false, egui::Button::new(&i18n.kafka_btn_connected));
                }
                _ => {
                    if ui.button(&i18n.kafka_btn_connect).clicked() {
                        self.state.current_cluster_idx = idx;
                        let broker_url = format!("{}:{}", cluster.host, cluster.port);

                        match current_cluster_metadata {
                            Some(_) => (),
                            None => self.get_cluster_metadata(rt, ui.ctx(), broker_url, idx),
                        };
                    }
                }
            };

            let edit_btn = ui.button(IconMoon::Pencil.as_str());
            if edit_btn.clicked() {
                ui.memory_mut(|mem| mem.toggle_popup(popup_id));
                self.state.selected_cluster_to_edit_idx = Some(idx);
                self.state.tmp_cluster_config = cluster.clone();
            }
            buttons.push(edit_btn);

            if ui.button(IconMoon::GarbageCan.as_str()).clicked() {
                if self.state.current_cluster_idx == idx {
                    self.state.current_cluster_idx = usize::MAX;
                }
                info!("clicked");
                tmp = idx;
            }
        });

        tmp
    }
}
