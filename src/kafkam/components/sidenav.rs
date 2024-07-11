// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use common::{icon_moon::IconMoon, internationalization::I18n};
use eframe::egui::{self, Response};
use std::collections::HashSet;
use tokio::runtime::Runtime;

use crate::kafkam::{
    producer::{self as producer_presenter, KafkaStatsProducerPresenter},
    state::{Cluster, KafkaAppState, KafkaPanel},
    view::KafkaView,
};

impl KafkaView {
    pub fn show_sidenav(
        &mut self,
        rt: &Runtime,
        ctx: &egui::Context,
        app_st: &mut KafkaAppState,
        i18n: &I18n,
    ) {
        egui::SidePanel::left("kafka_cluster_panel").show(ctx, |ui| {
            ui.menu_button(i18n.kafka_btn_add_connection.clone(), |ui| {
                self.edit_cluster_menu(ui, &mut app_st.clusters, i18n);
            });

            let popup_id = ui.make_persistent_id("cluster-edit-window");
            let mut buttons: Vec<Response> = Vec::with_capacity(app_st.clusters.len());
            let mut idxs_to_delete: HashSet<usize> = HashSet::default();

            for (idx, cluster) in app_st.clusters.iter_mut().enumerate() {
                egui::CollapsingHeader::new(cluster.name.clone())
                    .show_background(true)
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                            let idx_to_delete = self.cluster_connection_row(
                                ui,
                                idx,
                                i18n,
                                rt,
                                popup_id,
                                cluster,
                                &mut buttons,
                            );
                            idxs_to_delete.insert(idx_to_delete);

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

                            let show_stats_btn = if self.stats_presenter.is_some() {
                                ui.add(egui::SelectableLabel::new(
                                    self.state.current_cluster_idx == idx
                                        && self.state.current_cluster_idx != usize::MAX
                                        && self.state.current_view == KafkaPanel::Stats,
                                    // Lo quito de aquí para evitar tanto trabajo.
                                    // && self.stats_presenter.is_some()
                                    // && self.stats_presenter.unwrap().stats,
                                    &i18n.kafka_btn_show_stats,
                                ))
                            } else {
                                ui.add_enabled(
                                    false,
                                    egui::SelectableLabel::new(false, &i18n.kafka_btn_show_stats),
                                )
                            };

                            // TODO: Hay que parar subscripción existente cuando hacemos click
                            if show_brokers_btn.clicked() {
                                self.state.current_view = KafkaPanel::Brokers;
                            } else if show_topics_btn.clicked() {
                                self.state.current_view = KafkaPanel::Topics;
                            } else if show_subscription_btn.clicked() {
                                self.state.current_view = KafkaPanel::Subscribe;
                                self.subscribe(idx, cluster, ctx, rt);
                            } else if show_stats_btn.clicked() {
                                self.state.current_view = KafkaPanel::Stats;
                                ctx.request_repaint();
                            }
                        });
                    });
            }

            // info!("{idx_to_delete}, len={}", app_state.kafka.clusters.len());
            for idx in idxs_to_delete {
                if idx < app_st.clusters.len() {
                    println!("Borrar {idx}");
                    app_st.clusters.remove(idx);
                }
            }

            if self.state.selected_cluster_to_edit_idx.is_some() {
                egui::Window::new(&i18n.kafka_edit_cluster)
                    .collapsible(false)
                    .show(ctx, |ui| {
                        self.edit_cluster_menu(ui, &mut app_st.clusters, i18n);
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

                self.state.tmp_cluster_config = Default::default();
                self.state.selected_cluster_to_edit_idx = None;
                ui.close_menu();
            }
        });
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
            // let current_cluster_metadata = self.state.clusters_metadata.get(idx).unwrap();

            match self.state.current_cluster_metadata {
                // Esto significa que hay un cluster seleccionado y es este.
                Some(_) if idx == self.state.current_cluster_idx => {
                    ui.add_enabled(false, egui::Button::new(&i18n.kafka_btn_connected));
                }
                _ => {
                    if ui.button(&i18n.kafka_btn_connect).clicked() {
                        self.state.current_cluster_idx = idx;
                        let broker_url = format!("{}:{}", cluster.host, cluster.port);
                        let ctx = ui.ctx().clone();

                        match self.stats_presenter {
                            // (ref presenter) aquí no lo quiero para nada porque es productor y por lo tanto no necesita `unsubscribe`.
                            Some(_) => {
                                self.stats_presenter = None;
                                let producer = KafkaStatsProducerPresenter::new(ctx, &broker_url);
                                self.stats_presenter = Some(producer);
                            }
                            None => {
                                let producer = KafkaStatsProducerPresenter::new(ctx, &broker_url);
                                self.stats_presenter = Some(producer);
                            }
                        }

                        producer_presenter::get_cluster_metadata_and_stats(
                            rt,
                            &self.state.tx,
                            ui.ctx(),
                            broker_url,
                            idx,
                        );
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
                tmp = idx;
            }
        });

        tmp
    }
}
