// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use std::sync::Arc;

use bollard::secret::ImageSummary;
use bollard::Docker;
use components::empty_button_with_gray_stroke;
use eframe::egui;
use egui::{Color32, Stroke};
use futures_util::StreamExt;
use tokio::{runtime::Runtime, sync::mpsc};

use common::I18nDocker;
use components::widgets::wrap_dark_gray_text;

use crate::domain::{ContainerInfo, DockerContainerStats, DockerInfo, DockerMessage};
use crate::presenter::{self, DockerContainerPresenter, DockerImagePresenter, DockerPresenter};
use crate::view::DockerView;
use crate::{network_item, volume_item};

impl DockerView {
    pub fn show_sidenav(&mut self, rt: &Runtime, ctx: &egui::Context, i18n: &I18nDocker) {
        egui::SidePanel::left("docker_sidenav_panel").show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                if self.connection.is_none() && ui.button(&i18n.btn_connect).clicked() {
                    let data_images = Arc::clone(&self.state.images);
                    let data_containers = Arc::clone(&self.state.containers);
                    let data_networks = Arc::clone(&self.state.networks);
                    let data_volumes = Arc::clone(&self.state.volumes);

                    data_images.lock().unwrap().clear();
                    data_containers.lock().unwrap().clear();
                    data_networks.lock().unwrap().clear();
                    data_volumes.lock().unwrap().clear();

                    if self.connection.is_none() {
                        self.connection = presenter::connect();
                    }

                    let conn = self.connection.clone();

                    let tx = self.tx.clone();
                    rt.spawn(async move {
                        DockerPresenter::populate_state(
                            conn,
                            data_images,
                            data_containers,
                            data_networks,
                            data_volumes,
                        )
                            .await;
                        let _ = tx.send(DockerMessage::StatsReady).await;
                    });
                }

                let tx = self.tx.clone();
                if let Some(conn) = self.connection.clone() {
                    egui::CollapsingHeader::new(&i18n.images)
                        .show_background(true)
                        .show(ui, |ui| {
                            let tx = self.tx.clone();
                            for (idx, image) in
                                self.state.images.lock().unwrap().iter_mut().enumerate()
                            {
                                image_item(ui, idx, image, i18n, &conn, rt, &tx);
                            }
                        });

                    egui::CollapsingHeader::new(egui::RichText::new(&i18n.containers))
                        .show_background(true)
                        .show(ui, |ui| {
                            if empty_button_with_gray_stroke!(ui, "Show All").clicked() {
                                let tx_cloned = tx.clone();
                                rt.spawn(async move {
                                    let _ = tx_cloned
                                        .send(DockerMessage::Select((
                                            usize::MAX,
                                            DockerInfo::ContainerAll,
                                        )))
                                        .await;
                                });
                            }

                            for (idx, container) in
                                self.state.containers.lock().unwrap().iter_mut().enumerate()
                            {
                                container_item(ui, idx, container, i18n, &conn, rt, &tx);
                            }
                        });

                    egui::CollapsingHeader::new(&i18n.volumes)
                        .show_background(true)
                        .show(ui, |ui| {
                            for volume in self.state.volumes.lock().unwrap().iter_mut() {
                                volume_item!(ui, volume, 40, i18n);
                            }
                        });

                    egui::CollapsingHeader::new(&i18n.networks)
                        .show_background(true)
                        .show(ui, |ui| {
                            for network in self.state.networks.lock().unwrap().iter_mut() {
                                network_item!(ui, network, i18n);
                            }
                        });
                }
            });
        });
    }
}

#[inline(always)]
fn image_item(
    ui: &mut egui::Ui,
    position: usize,
    img_summary: &ImageSummary,
    i18n: &I18nDocker,
    docker: &Docker,
    rt: &Runtime,
    tx: &mpsc::Sender<DockerMessage>,
) {
    if let Some(repo_tags) = img_summary.repo_tags.first() {
        let parts: Vec<&str> = repo_tags.split(':').collect();
        let (image_name, image_tag) = match &parts[..] {
            [name] => (*name, ""), // No tag present
            [name, tag] => (*name, *tag),
            _ => return,
        };

        let id = ui.make_persistent_id(image_name);
        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false)
            .show_header(ui, |ui| {
                if ui.add(egui::Button::new(image_name).frame(false)).clicked() {
                    log::info!("Clicked button {image_name:}");
                    let img_name = image_name.to_owned();
                    let dc = docker.to_owned();
                    let tx_cloned = tx.clone();
                    let img_cloned = img_summary.clone();

                    rt.spawn(async move {
                        let _ = tx_cloned.send(DockerMessage::Loading).await;
                        log::info!("Asking for image: {img_name:}");
                        let msg = DockerImagePresenter::get_image_info(&dc, img_cloned, img_name)
                            .await
                            .map_or_else(
                                |error| DockerMessage::Error(error),
                                |image_info| {
                                    DockerMessage::Select((position, DockerInfo::Image(image_info)))
                                },
                            );
                        let _ = tx_cloned.send(msg).await;
                    });
                }
            })
            .body(|ui| {
                let job = wrap_dark_gray_text(format!("ID: {:}", img_summary.id));
                ui.label(job);
                ui.label(format!("Tag: {}", image_tag));
                ui.label(format!("{}: {} MB", i18n.size, img_summary.size >> 20));
            });
    }
}

#[inline(always)]
fn container_item(
    ui: &mut egui::Ui,
    position: usize,
    container: &ContainerInfo,
    i18n: &I18nDocker,
    _docker: &Docker,
    rt: &Runtime,
    tx: &mpsc::Sender<DockerMessage>,
) {
    let id = ui.make_persistent_id(&container.name);
    let color = if container.state == "running" {
        eframe::epaint::Color32::DARK_GREEN
    } else {
        eframe::epaint::Color32::DARK_GRAY
    };
    egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false)
        .show_header(ui, |ui| {
            if ui
                .add(
                    egui::Button::new(
                        egui::RichText::new(&container.name)
                            .color(color)
                            .monospace()
                            .size(12.0),
                    )
                    .frame(false),
                )
                .clicked()
            {
                let tx_cloned = tx.clone();
                let cont_cloned = container.clone();

                rt.spawn(async move {
                    let _ = tx_cloned.send(DockerMessage::Loading).await;
                    let msg = DockerMessage::Select((position, DockerInfo::Container(cont_cloned)));

                    let _ = tx_cloned.send(msg).await;
                });
            }
        })
        .body(|ui| {
            let job = wrap_dark_gray_text(format!("ID: {:}", container.id));
            ui.label(job);
            ui.label(format!("{}: {}", i18n.image, container.image));
        });
}
