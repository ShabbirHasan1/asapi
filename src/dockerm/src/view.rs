// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use std::collections::HashMap;

use bollard::{
    container::{CPUStats, MemoryStats, MemoryStatsStats},
    Docker,
};
use chrono::Timelike;
use common::I18nDocker;
use eframe::egui;
use futures_util::StreamExt;
use tokio::runtime::Runtime;

use crate::{
    domain::{
        DockerAppState, DockerContainerStats, DockerDefaults, DockerElementSelection, DockerInfo,
        DockerLocalState, DockerMessage, DockerSelection, DockerViewMode,
    },
    presenter::{self, DockerContainerPresenter},
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
        let (tx, rx) = tokio::sync::mpsc::channel(1024);

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
            DockerMessage::Connected => {
                self.connection = presenter::connect();
            }
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
                    self.state.container.current_info = container_info;
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
            DockerMessage::LogStdIn((cont_name, msg))
            | DockerMessage::LogStdOut((cont_name, msg))
            | DockerMessage::LogStdErr((cont_name, msg))
            | DockerMessage::LogConsole((cont_name, msg)) => {
                self.state
                    .container
                    .logs
                    .entry(cont_name)
                    .and_modify(|v| v.push(msg.clone()))
                    .or_insert(vec![msg]);
            }
            DockerMessage::ContainerStatsRequest(container_name) => {
                // Si ya está en el hashmap de estadísticas, no lanzamos el proceso para manejarlas.
                if self.state.container.stats.contains_key(&container_name) {
                    return;
                }

                let empty_dt = self.defaults.empty_dt.clone();
                let conn = self.connection.clone().unwrap();
                let tx = self.tx.clone();

                rt.spawn(async move {
                    let stream =
                        &mut DockerContainerPresenter::stream_stats(&conn, &container_name);
                    while let Some(Ok(stats)) = stream.next().await {
                        // Primer filtro -- filtramos estadísticas sin fecha real.
                        if stats.read != empty_dt {
                            // Segundo filtro -- filtramos y solo enviamos cada 5 segundos;
                            //                   podría ser configurable pero no le veo utilidad.
                            let seconds = stats.read.second();
                            if seconds % 5 != 0 {
                                continue;
                            }

                            let cpu_usage =
                                compute_cpu_usage(&stats.precpu_stats, &stats.cpu_stats);
                            log::info!("CPU usage to insert: {cpu_usage:}");
                            let mem_usage = compute_mem_usage(&stats.memory_stats);

                            let msg = DockerMessage::Stats((
                                container_name.clone(),
                                cpu_usage,
                                mem_usage,
                                stats.storage_stats,
                                stats.read,
                            ));
                            let _ = tx.send(msg).await;
                        }
                    }
                });
            }
            // DockerMessage::StatsReady => {
            //     let names = self
            //         .state
            //         .containers
            //         .lock()
            //         .unwrap()
            //         .iter()
            //         .map(|cont| cont.name.clone())
            //         .collect::<Vec<_>>();

            //     let empty_dt = self.defaults.empty_dt.clone();

            //     for name in names {
            //         log::info!("nombre: {name:}");
            //         let conn = self.connection.clone().unwrap();
            //         self.state.container.logs.clear();
            //         let tx = self.tx.clone();

            //         rt.spawn(async move {
            //             let stream = &mut DockerContainerPresenter::stream_stats(&conn, &name);
            //             while let Some(Ok(stats)) = stream.next().await {
            //                 // Primer filtro -- filtramos estadísticas sin fecha real.
            //                 if stats.read != empty_dt {
            //                     // Segundo filtro -- filtramos y solo enviamos cada 5 segundos;
            //                     //                   podría ser configurable pero no le veo utilidad.
            //                     let seconds = stats.read.second();
            //                     if seconds % 5 != 0 {
            //                         continue;
            //                     }

            //                     let cpu_usage =
            //                         compute_cpu_usage(&stats.precpu_stats, &stats.cpu_stats);
            //                     log::info!("CPU usage to insert: {cpu_usage:}");
            //                     let mem_usage = compute_mem_usage(&stats.memory_stats);

            //                     let msg = DockerMessage::Stats((
            //                         cpu_usage,
            //                         mem_usage,
            //                         stats.storage_stats,
            //                         stats.read,
            //                     ));
            //                     let _ = tx.send(msg).await;
            //                 }
            //             }
            //         });
            //     }
            // }
            DockerMessage::Stats(msg) => {
                let (name, cpu, mem, disk, date) = msg;
                self.state
                    .container
                    .stats
                    .entry(name)
                    .and_modify(|st| {
                        let len = st.cpu.len();
                        st.dates.insert(len, date.to_rfc2822());
                        st.cpu.push([len as f64, cpu]);
                        st.mem.push([len as f64, mem.0]);
                        st.disk.push(disk);
                    })
                    .or_insert(DockerContainerStats {
                        dates: HashMap::from([(0, date.to_rfc2822())]),
                        cpu: vec![[0.0, cpu]],
                        mem: vec![[0.0, mem.0]],
                        disk: vec![disk],
                    });

                // match self.state.container.stats.get_mut(&name) {
                //     None => {
                //         self.state.container.stats.insert(
                //             name,
                //             DockerContainerStats {
                //                 dates: HashMap::from([(0, date.to_rfc2822())]),
                //                 cpu: vec![[0.0, cpu]],
                //                 mem: vec![[0.0, mem.0]],
                //                 disk: vec![disk],
                //             },
                //         );
                //     }
                //     Some(st) => {
                //         let len = st.cpu.len();
                //         st.dates.insert(len, date.to_rfc2822());
                //         st.cpu.push([len as f64, cpu]);
                //         st.mem.push([len as f64, mem.0]);
                //         st.disk.push(disk);
                //     }
                // }
            }
        }
    }
}

fn compute_cpu_usage(prev_cpu: &CPUStats, cpu: &CPUStats) -> f64 {
    if prev_cpu.cpu_usage.total_usage == 0 {
        return 0.0;
    }

    let cpu_delta = cpu.cpu_usage.total_usage - prev_cpu.cpu_usage.total_usage;
    let system_cpu_delta = cpu
        .system_cpu_usage
        .and_then(|v| prev_cpu.system_cpu_usage.map(|p| v - p))
        .unwrap_or(1);
    let number_cpus = cpu.online_cpus.unwrap_or(1);

    ((cpu_delta * number_cpus) as f64) * 100.0 / system_cpu_delta as f64
}

fn compute_mem_usage(mem: &MemoryStats) -> (f64, f64, f64) {
    let used_mem = mem
        .usage
        .and_then(|v| {
            mem.stats.map(|s| match s {
                MemoryStatsStats::V1(v1) => (v - v1.cache) >> 20,
                MemoryStatsStats::V2(_) => v >> 20,
            })
        })
        .unwrap_or(1) as f64;

    let limit = mem.limit.map_or(1, |v| v >> 20) as f64;

    (used_mem, limit, used_mem / limit * 100.0)
}
