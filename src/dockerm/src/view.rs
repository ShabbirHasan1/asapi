// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use std::collections::HashMap;

use bollard::Docker;
use common::I18nDocker;
use eframe::egui;
use futures_util::StreamExt;
use tokio::runtime::Runtime;

use crate::{
    domain::{
        DockerAppState, DockerContainerStats, DockerDefaults, DockerElementSelection, DockerInfo,
        DockerLocalState, DockerMessage, DockerSelection, DockerViewMode,
    },
    presenter::DockerContainerPresenter,
};

pub struct DockerView {
    pub defaults: DockerDefaults,
    pub state: DockerLocalState,
    pub connection: Option<Docker>,
    pub view_mode: DockerViewMode,
    pub tx: tokio::sync::mpsc::Sender<DockerMessage>,
    rx: tokio::sync::mpsc::Receiver<DockerMessage>,
}

impl Default for DockerView {
    fn default() -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(8);

        Self {
            defaults: Default::default(),
            state: Default::default(),
            connection: Default::default(),
            view_mode: Default::default(),
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
            self.process_message(rt, message);
            ctx.request_repaint();
        }
        if self.state.container.show_stats {
            ctx.request_repaint();
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
        if self.state.current_selection.is_some() {
            self.show_central_panel(rt, ctx, i18n);
        }
    }

    fn process_message(&mut self, rt: &Runtime, message: DockerMessage) {
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
            DockerMessage::StatsReady => {
                let names = self
                    .state
                    .containers
                    .lock()
                    .unwrap()
                    .iter()
                    .map(|cont| cont.name.clone())
                    .collect::<Vec<_>>();

                let empty_dt = self.defaults.empty_dt.clone();

                for name in names {
                    log::info!("nombre: {name:}");
                    let conn = self.connection.clone().unwrap();
                    self.state.container.logs.clear();
                    let tx = self.tx.clone();

                    rt.spawn(async move {
                        let stream = &mut DockerContainerPresenter::stream_stats(&conn, &name);
                        while let Some(Ok(stats)) = stream.next().await {
                            if stats.read != empty_dt {
                                let msg = DockerMessage::Stats((
                                    stats.cpu_stats,
                                    stats.memory_stats,
                                    stats.storage_stats,
                                    stats.read,
                                ));
                                let _ = tx.send(msg).await;
                            }
                        }
                    });
                }
            }
            DockerMessage::Stats(msg) => {
                let (cpu, mem, disk, date) = msg;
                let name = &self.state.container.info.name;
                match self.state.container.stats.get_mut(name) {
                    None => {
                        self.state.container.stats.insert(
                            name.to_owned(),
                            DockerContainerStats {
                                dates: HashMap::from([(0, date.to_rfc2822())]),
                                cpu: vec![(0, cpu.cpu_usage.total_usage as f64)],
                                mem: vec![mem],
                                disk: vec![disk],
                            },
                        );
                    }
                    Some(st) => {
                        log::info!("{date:}");
                        let previous_cpu = st.cpu.last().unwrap();
                        let len = st.cpu.len();
                        let current_cpu = cpu.cpu_usage.total_usage;
                        let new_cpu_entry = (len, current_cpu as f64 - previous_cpu.1 as f64);
                        st.dates.insert(len, date.to_rfc2822());
                        st.cpu.push(new_cpu_entry);
                        st.mem.push(mem);
                        st.disk.push(disk);
                    }
                }
            }
        }
    }
}
