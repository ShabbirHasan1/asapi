// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use std::sync::Arc;

use eframe::egui;
use tokio::runtime::Runtime;

use common::I18nDocker;
use components::widgets::wrap_dark_gray_text;

use crate::presenter::{self, DockerPresenter};
use crate::{container_item, image_item, network_item, volume_item};
use crate::{state::DockerAppState, view::DockerView};


impl DockerView {
    pub fn show_sidenav(
        &mut self,
        rt: &Runtime,
        ctx: &egui::Context,
        i18n: &I18nDocker,
    ) {
        egui::SidePanel::left("docker_sidenav_panel").show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                if ui.button(&i18n.btn_connect).clicked() {
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

                    rt.spawn(async move {
                        DockerPresenter::populate_state(
                            conn,
                            data_images,
                            data_containers,
                            data_networks,
                            data_volumes,
                        )
                        .await;
                    });
                }

                egui::CollapsingHeader::new(&i18n.images).show(ui, |ui| {
                    for image in self.state.images.lock().unwrap().iter_mut() {
                        if let Some(repo_tags) = image.repo_tags.first() {
                            image_item!(ui, repo_tags, image, i18n);
                        }
                    }
                });

                egui::CollapsingHeader::new(&i18n.containers).show(ui, |ui| {
                    for container in self.state.containers.lock().unwrap().iter_mut() {
                        container_item!(ui, container, i18n.image);
                    }
                });

                egui::CollapsingHeader::new(&i18n.volumes).show(ui, |ui| {
                    let line_width = 40;

                    for volume in self.state.volumes.lock().unwrap().iter_mut() {
                        volume_item!(ui, volume, line_width, i18n);
                    }
                });

                egui::CollapsingHeader::new(&i18n.networks).show(ui, |ui| {
                    for network in self.state.networks.lock().unwrap().iter_mut() {
                        network_item!(ui, network, i18n);
                    }
                });
            });
        });
    }
}
