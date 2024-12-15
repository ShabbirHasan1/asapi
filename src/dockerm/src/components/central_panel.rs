// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use std::{collections::HashMap, ops::RangeInclusive};

use bollard::container::LogOutput;
use eframe::egui;
use egui::Color32;
use egui_extras::{Column, TableBuilder};
use egui_plot::{CoordinatesFormatter, Corner, Legend, Line, LineStyle, Plot, PlotPoints};

use futures_util::StreamExt;
use tokio::runtime::Runtime;

use crate::{
    domain::{DockerElementSelection, DockerMessage},
    info_table_row,
    presenter::DockerContainerPresenter,
    view::DockerView,
};
use common::{icon_moon::IconMoon, I18nDocker};

impl DockerView {
    pub fn show_central_panel(&mut self, rt: &Runtime, ctx: &egui::Context, i18n: &I18nDocker) {
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.state.current_selection.as_ref().unwrap().selected_view {
                DockerElementSelection::Image => {
                    self.image_info_panel(ui, i18n);
                }
                DockerElementSelection::Container => {
                    self.container_info_panel(ui, i18n);
                    self.container_logs_stats_panel(rt, ui, i18n);
                }
                DockerElementSelection::ContainerAll => {
                    self.all_containers_table(rt, ui, i18n);
                }
                DockerElementSelection::Volume => todo!(),
                DockerElementSelection::Network => todo!(),
            }
        });
    }

    fn all_containers_table(&self, rt: &Runtime, ui: &mut egui::Ui, i18n: &I18nDocker) {
        let available_height = ui.available_height();

        egui::ScrollArea::both().show(ui, |ui| {
            TableBuilder::new(ui)
                .auto_shrink(true)
                .striped(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::exact(80.0))
                .columns(Column::initial(150.0).range(40.0..).resizable(true), 5)
                .min_scrolled_height(0.0)
                .max_scroll_height(available_height)
                .resizable(true)
                .header(32.0, |mut header| {
                    header.col(|_ui| {});
                    header.col(|ui| {
                        ui.strong(&i18n.name);
                    });
                    header.col(|ui| {
                        ui.strong("ID");
                    });
                    header.col(|ui| {
                        ui.strong(&i18n.image);
                    });
                    header.col(|ui| {
                        ui.strong(&i18n.ports);
                    });
                    header.col(|ui| {
                        ui.strong(&i18n.size);
                    });
                })
                .body(|mut body| {
                    for container in self.state.containers.lock().unwrap().iter() {
                        body.row(32.0, |mut row| {
                            row.col(|ui| {
                                ui.horizontal(|ui| {
                                    let icon = if container.state == "running" {
                                        IconMoon::Stop
                                    } else {
                                        IconMoon::Play
                                    }
                                    .as_str();
                                    if ui.button(icon).clicked() {
                                        let tx_cloned = self.tx.clone();
                                        let name = container.name.clone();
                                        let conn = self.connection.clone().unwrap();

                                        if container.state != "running" {
                                            rt.spawn(async move {
                                                match DockerContainerPresenter::start_container(
                                                    &conn, &name,
                                                )
                                                .await
                                                {
                                                    Ok(_) => {}
                                                    Err(err) => {
                                                        let _ = tx_cloned
                                                            .send(DockerMessage::Error(err))
                                                            .await;
                                                    }
                                                }
                                            });
                                        } else if container.state == "running" {
                                            rt.spawn(async move {
                                                match DockerContainerPresenter::stop_container(
                                                    &conn, &name,
                                                )
                                                .await
                                                {
                                                    Ok(_) => {}
                                                    Err(err) => {
                                                        let _ = tx_cloned
                                                            .send(DockerMessage::Error(err))
                                                            .await;
                                                    }
                                                }
                                            });
                                        }
                                    }
                                    if ui.button(IconMoon::GarbageCan.as_str()).clicked() {
                                        let tx_cloned = self.tx.clone();
                                        let name = container.name.clone();
                                        let conn = self.connection.clone().unwrap();

                                        rt.spawn(async move {
                                            match DockerContainerPresenter::remove_container(
                                                &conn, &name,
                                            )
                                            .await
                                            {
                                                Ok(_) => {}
                                                Err(err) => {
                                                    let _ = tx_cloned
                                                        .send(DockerMessage::Error(err))
                                                        .await;
                                                }
                                            }
                                        });
                                    }
                                });
                            });
                            row.col(|ui| {
                                ui.label(&container.name);
                            });
                            row.col(|ui| {
                                ui.label(format!("{}", &container.id[0..15]));
                            });
                            row.col(|ui| {
                                ui.label(&container.image);
                            });
                            row.col(|ui| {
                                ui.label(&container.ports_string);
                            });
                            row.col(|ui| {
                                ui.label(format!("{} MB", container.size_root_fs >> 20));
                            });
                        });
                    }
                });
        });
    }

    fn container_logs_stats_panel(&mut self, rt: &Runtime, ui: &mut egui::Ui, i18n: &I18nDocker) {
        ui.horizontal(|ui| {
            if ui
                .selectable_label(!self.state.container.show_stats, &i18n.logs)
                .clicked()
            {
                self.state.container.show_stats = false;
                let name = self.state.container.info.name.clone();
                let conn = self.connection.clone().unwrap();
                let tx = self.tx.clone();
                self.state.container.logs.clear();
                ui.ctx().request_repaint();

                rt.spawn(async move {
                    let stream = &mut DockerContainerPresenter::stream_logs(&conn, &name);
                    while let Some(Ok(logs)) = stream.next().await {
                        let message = match logs {
                            LogOutput::StdErr { message } => DockerMessage::LogStdErr(format!(
                                "{}",
                                String::from_utf8_lossy(&message)
                            )),
                            LogOutput::StdOut { message } => DockerMessage::LogStdErr(format!(
                                "{}",
                                String::from_utf8_lossy(&message)
                            )),
                            LogOutput::StdIn { message } => DockerMessage::LogStdErr(format!(
                                "{}",
                                String::from_utf8_lossy(&message)
                            )),
                            LogOutput::Console { message } => DockerMessage::LogStdErr(format!(
                                "{}",
                                String::from_utf8_lossy(&message)
                            )),
                        };
                        let _ = tx.send(message).await;
                    }
                });
            }

            if ui
                .selectable_label(self.state.container.show_stats, &i18n.stats)
                .clicked()
            {
                self.state.container.show_stats = true;
            }
        });

        if self.state.container.show_stats {
            let selected_container = self.state.container.info.name.clone();
            let statsopt = self.state.container.stats.get(&selected_container);
            if statsopt.is_none() {
                return;
            }

            let stats = statsopt.unwrap();
            let cpu_total_usage: PlotPoints = stats
                .cpu
                .iter()
                .enumerate()
                .map(|(idx, c)| [idx as f64, c.cpu_usage.total_usage as f64])
                .collect();
            let cpu_line = Line::new(cpu_total_usage)
                .color(Color32::from_rgb(100, 200, 100))
                .style(LineStyle::Solid)
                .name("cpu total usage");

            let labels: HashMap<usize, String> = stats
                .dates
                .iter()
                .enumerate()
                .map(|(i, &label)| (i, label.to_rfc2822()))
                .collect();

            let mem_total_usage: PlotPoints = stats
                .mem
                .iter()
                .enumerate()
                .map(|(idx, c)| [idx as f64, c.usage.unwrap_or_default() as f64])
                .collect();
            let mem_line = Line::new(mem_total_usage)
                .color(Color32::from_rgb(200, 0, 100))
                .style(LineStyle::Solid)
                .name("mem total usage");

            let len = labels.len();

            let cpu_plot = Plot::new("docker_cpu_stats")
                .legend(Legend::default())
                .y_axis_min_width(12.0 * 4 as f32)
                .show_axes(true)
                .show_grid(true)
                .x_axis_formatter({
                    |grid_mark, _range: &RangeInclusive<f64>| {
                        let idx = grid_mark.value as usize;
                        if idx < len {
                            labels[&idx].to_string()
                        } else {
                            "".to_string()
                        }
                    }
                });

            let mem_plot = Plot::new("docker_mem_stats")
                .legend(Legend::default())
                .y_axis_min_width(12.0 * 4 as f32)
                .show_axes(true)
                .show_grid(true)
                .x_axis_formatter({
                    |grid_mark, _range: &RangeInclusive<f64>| {
                        let idx = grid_mark.value as usize;
                        if idx < len {
                            labels[&idx].to_string()
                        } else {
                            "".to_string()
                        }
                    }
                });

            let _ = cpu_plot
                .show(ui, |plot_ui| {
                    plot_ui.line(cpu_line);
                })
                .response;
            let _ = mem_plot
                .show(ui, |plot_ui| {
                    plot_ui.line(mem_line);
                })
                .response;
        } else {
            egui::ScrollArea::both().show(ui, |ui| {
                // TODO: Esta tabla puede almanacenarse de inicio y no tener que crearla cada vez?
                let table = TableBuilder::new(ui)
                    .striped(true)
                    .resizable(false)
                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                    .column(Column::remainder())
                    .min_scrolled_height(0.0)
                    .max_scroll_height(f32::INFINITY);

                table.body(|mut body| {
                    for log in self.state.container.logs.iter() {
                        body.row(32.0, |mut row| {
                            row.col(|ui| {
                                ui.label(log);
                            });
                        });
                    }
                });
            });
        }
    }

    fn container_info_panel(&self, ui: &mut egui::Ui, i18n: &I18nDocker) {
        let info = &self.state.container.info;

        egui::CollapsingHeader::new(format!("{} : {}", i18n.container_info, &info.name))
            .default_open(true)
            .show_background(true)
            .show(ui, |ui| {
                let available_height = ui.available_height();
                egui::ScrollArea::horizontal().show(ui, |ui| {
                    TableBuilder::new(ui)
                        .auto_shrink(true)
                        .striped(true)
                        .columns(Column::initial(150.0).range(40.0..).resizable(true), 1)
                        .column(Column::remainder())
                        .min_scrolled_height(0.0)
                        .max_scroll_height(available_height)
                        .resizable(true)
                        .body(|mut body| {
                            info_table_row!(body, &i18n.name, &info.name);
                            info_table_row!(body, "ID", &info.id);
                            info_table_row!(body, &i18n.image, &self.state.selected_image_info.0);
                            info_table_row!(
                                body,
                                &i18n.image_id,
                                &self.state.selected_image_info.1.id
                            );
                            info_table_row!(body, &i18n.image, &info.image);
                            info_table_row!(body, &i18n.ports, &info.ports_string);
                        });
                });
            });
    }

    fn image_info_panel(&self, ui: &mut egui::Ui, i18n: &I18nDocker) {
        let img_name = &self.state.selected_image_info.0;
        let img_inspect = &self.state.selected_image_info.1;
        let img_summary = &self.state.selected_image_info.2;

        egui::CollapsingHeader::new(format!(
            "{} : {}",
            i18n.image_info, self.state.selected_image_info.0
        ))
        .default_open(true)
        .show_background(true)
        .show(ui, |ui| {
            egui::ScrollArea::horizontal().show(ui, |ui| {
                TableBuilder::new(ui)
                    .auto_shrink(true)
                    .striped(true)
                    .min_scrolled_height(0.0)
                    .columns(Column::initial(150.0).range(40.0..).resizable(true), 1)
                    .column(Column::remainder())
                    .body(|mut body| {
                        info_table_row!(body, &i18n.name, img_name);
                        info_table_row!(body, "ID", &img_inspect.id);
                        info_table_row!(body, "OS", &img_inspect.os);
                        info_table_row!(body, "Version OS", &img_inspect.os_version);
                        info_table_row!(body, &i18n.author, &img_inspect.author);
                        info_table_row!(body, &i18n.architecture, &img_inspect.architecture);
                        info_table_row!(body, &i18n.parent, &img_inspect.parent);
                        info_table_row!(body, &i18n.created, &img_inspect.created);
                        info_table_row!(body, &i18n.size, format!("{} MB", img_summary.size >> 20));
                        info_table_row!(
                            body,
                            &i18n.containers,
                            &img_summary.containers.to_string()
                        );
                    });
            });
        });
    }
}

// fn main() {
//     // Example data
//     let labels = vec!["A", "B", "C", "D"];
//     let values = vec![10.0, 20.0, 30.0, 40.0];

//     // Create a mapping from labels to indices
//     let label_to_index: std::collections::HashMap<&str, f64> =
//         labels.iter().enumerate().map(|(i, &label)| (label, i as f64)).collect();

//     // Convert string labels to indices for plotting
//     let plot_data: Vec<(f64, f64)> =
//         labels.iter().zip(&values).map(|(&label, &value)| (label_to_index[label], value)).collect();

//     // Create a plot with custom axis configuration
//     let mut plot = Plot::new("string_plot")
//         .data_label("Values")
//         .x_axis_formatter(move |v| labels[v as usize].to_string());

//     // Add data to the plot
//     for (label, value) in plot_data.iter() {
//         plot = plot.add(PlotLine::new(vec![(*label, *value)]));
//     }

//     // Use egui to create a window and display the plot
//     let mut app = App { plot };
//     eframe::run_simple(
//         "String X-Axis Plot",
//         Default::default(),
//         move |ctx: &Context, _frame: &mut eframe::Frame| {
//             CentralPanel::default().show(ctx, |ui| {
//                 ui.heading("String X-Axis Plot Example");
//                 ui.add(app.plot);
//             });
//         },
//     );
// }
