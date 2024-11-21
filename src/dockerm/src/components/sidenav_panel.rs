// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use std::sync::Arc;

use eframe::egui;
use log;
use tokio::runtime::Runtime;

use common::I18nDocker;

use crate::presenter::{self, DockerPresenter};
use crate::{state::DockerAppState, view::DockerView};

impl DockerView {
    pub fn show_sidenav(
        &mut self,
        rt: &Runtime,
        ctx: &egui::Context,
        app_st: &mut DockerAppState,
        i18n: &I18nDocker,
    ) {
        egui::SidePanel::left("docker_sidenav_panel").show(ctx, |ui| {
            if ui.button(&i18n.btn_connect).clicked() {
                let data = Arc::clone(&self.state.images);

                if self.connection.is_none() {
                    self.connection = presenter::connect();
                }

                let conn = self.connection.clone();

                rt.spawn(async move {
                    DockerPresenter::populate_images(conn, data).await;
                });
            }
            // Mostramos imágenes
            for (idx, image) in self.state.images.lock().unwrap().iter_mut().enumerate() {
                if let Some(repo_tag) = image.repo_tags.first() {
                    let parts: Vec<&str> = repo_tag.split(':').collect();
                    let (image_name, image_tag) = match &parts[..] {
                        [name] => (*name, ""), // No tag present
                        [name, tag] => (*name, *tag),
                        _ => continue, // Formato inválido
                    };

                    egui::CollapsingHeader::new(image_name).show(ui, |ui| {
                        ui.label(format!("Tag: {}", image_tag));
                        ui.label(format!("Size: {} MB", image.size / 1048576));
                    });
                }
            }

            // if !self.flag {
            //     log::info!("trying to connect with docker");
            //     rt.spawn(async {
            //         let conn = presenter::connect().unwrap();

            //         let _ = presenter::stats(&conn).await;
            //     });
            // }
        });
    }
}
