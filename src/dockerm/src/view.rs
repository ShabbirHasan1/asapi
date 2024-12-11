// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use eframe::egui;
use tokio::runtime::Runtime;

use bollard::Docker;
use common::I18nDocker;

use crate::domain::{
    DockerAppState, DockerElementSelection, DockerInfo, DockerLocalState, DockerMessage,
    DockerSelection,
};

pub struct DockerView {
    pub state: DockerLocalState,
    pub connection: Option<Docker>,
    pub tx: tokio::sync::mpsc::Sender<DockerMessage>,
    rx: tokio::sync::mpsc::Receiver<DockerMessage>,
}

impl Default for DockerView {
    fn default() -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(8);

        Self {
            state: Default::default(),
            connection: Default::default(),
            tx,
            rx,
        }
    }
}
impl DockerView {
    pub fn update(
        &mut self,
        ctx: &egui::Context,
        rt: &Runtime,
        app_st: &mut DockerAppState,
        i18n: &I18nDocker,
    ) {
        // =======================================
        // Preparación de cada ciclo
        // =======================================
        while let Ok(message) = self.rx.try_recv() {
            log::info!("New message");
            self.process_message(message);
        }

        // =======================================
        // Panel Lateral
        // =======================================

        if app_st.show_sidebar {
            self.show_sidenav(rt, ctx, i18n);
        }

        // =======================================
        // Panel Central
        // =======================================
        if self.state.current_selection.is_none() {
            return;
        }

        self.show_central_panel(rt, ctx, i18n);
    }

    fn process_message(&mut self, message: DockerMessage) {
        match message {
            DockerMessage::Error(err) => {
                log::error!("{err:}");
            }
            DockerMessage::Loading => {
                log::info!("Loading");
            }
            DockerMessage::Select(info) => match info.1 {
                DockerInfo::Image(image_info) => {
                    self.state.selected_image_info = image_info;
                    self.state.current_selection = Some(DockerSelection {
                        selected_idx: info.0,
                        selected_view: DockerElementSelection::Image,
                    });
                }
                DockerInfo::Container(container_info) => {
                    self.state.container.info = container_info;
                    self.state.current_selection = Some(DockerSelection {
                        selected_idx: info.0,
                        selected_view: DockerElementSelection::Container,
                    });
                }
                DockerInfo::ContainerAll => {
                    self.state.current_selection = Some(DockerSelection {
                        selected_idx: info.0,
                        selected_view: DockerElementSelection::ContainerAll,
                    })
                }
            },
            DockerMessage::LogStdIn(msg) => {
                self.state.container.logs.push(msg);
            }
            DockerMessage::LogStdOut(msg) => {
                self.state.container.logs.push(msg);
            }
            DockerMessage::LogStdErr(message) => {
                self.state.container.logs.push(message);
            }
            DockerMessage::LogConsole(message) => {
                self.state.container.logs.push(message);
            }
        }
    }
}
