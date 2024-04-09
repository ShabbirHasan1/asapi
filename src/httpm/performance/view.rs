// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use eframe::egui;
use egui::{Color32, Pos2, Response};
use egui_plot::{CoordinatesFormatter, Corner, Legend, Line, LineStyle, Plot, PlotPoints};
use futures::future::join_all;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::Semaphore;

use crate::common::generator::{Gen, SimpleRGen};
use crate::common::internationalization::I18n;
use crate::httpm::methods::HttpMethod;
use crate::httpm::request::api_request;
use crate::httpm::workspace::Request;
use crate::{error, info};

use super::components::params::Params;

#[derive(Default)]
pub struct HttpPerformanceState {
    pub request: Request,
}

#[derive(Debug)]
struct PerformanceRequestMessage {
    pub _msg: String,
    pub duration: Duration,
}

pub struct HttpPerformanceView {
    tx: Sender<PerformanceRequestMessage>,
    rx: Receiver<PerformanceRequestMessage>,
    _response: String,
    show_headers: bool,
    show_body: bool,
    params: Params,
    _state: HttpPerformanceState,
    line_demo: LineDemo,
    n_total_requests: String,
    n_concurrent_requests: String,
    chart: Vec<PerformanceRequestMessage>,
}

impl Default for HttpPerformanceView {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel(8);

        Self {
            tx,
            rx,
            _response: String::new(),
            params: Params::default(),
            show_headers: true,
            show_body: true,
            _state: HttpPerformanceState::default(),
            line_demo: LineDemo::default(),
            chart: Vec::new(),
            n_total_requests: String::default(),
            n_concurrent_requests: String::default(),
        }
    }
}

impl HttpPerformanceView {
    //  Esta función se llama continuamente pero no de forma directa sino que es controlada por HttpView.
    pub fn ui(
        &mut self,
        // ctx: &egui::Context,
        ui: &mut egui::Ui,
        // _frame: &mut eframe::Frame,
        // state: &mut AppState,
        rt: &Runtime,
        i18n: &I18n,
        request: &mut Request,
    ) -> bool {
        let _ = i18n;
        let mut response = false;
        // =======================================
        // Preparación de cada ciclo
        // =======================================
        while let Ok(response) = self.rx.try_recv() {
            info!("{:?}", response.duration);
            self.chart.push(response);
        }

        // =======================================
        // Presentación de datos en central de HttpView
        // =======================================
        // --> Mostramos datos de petición. NO EDITABLE <--
        ui.horizontal(|ui| {
            let url_text_edit = egui::TextEdit::singleline(&mut request.url).interactive(false);
            ui.label("URL:");
            let _ = ui.add_sized(ui.available_size(), url_text_edit);
        });

        ui.horizontal(|ui| {
            ui.label("Method:");
            ui.label(request.method.to_string());
            if ui.button("Regular").clicked() {
                response = true
            }

            ui.add(
                egui::TextEdit::singleline(&mut self.n_total_requests).hint_text("Total Requests"),
            );
            ui.add(
                egui::TextEdit::singleline(&mut self.n_concurrent_requests)
                    .hint_text("Concurrent Requests"),
            );
            if ui.button("Test").clicked() {
                let req = request.clone();
                let tx_cloned = self.tx.clone();

                match (
                    self.n_total_requests.parse::<u32>(),
                    self.n_concurrent_requests.parse::<u16>(),
                ) {
                    (Ok(t), Ok(c)) => {
                        rt.spawn(async move {
                            send_concurrent_requests(tx_cloned, req, t, c).await;
                        });
                    }
                    _ => error!("Wrong params as n requests"),
                }
            }
        });

        ui.separator();

        let method = request.method;
        ui.horizontal(|ui| {
            if ui.selectable_label(self.show_headers, "Headers").clicked() {
                self.show_headers = !self.show_headers;
            }
            if !(method == HttpMethod::Get || method == HttpMethod::Delete)
                && ui.selectable_label(self.show_body, "Body").clicked()
            {
                self.show_body = !self.show_body;
            }
        });

        if self.show_headers {
            self.params
                .create(ui, request.headers_params.clone(), "Headers".to_string());
        }

        if !(method == HttpMethod::Get || method == HttpMethod::Delete) {
            if self.show_body {
                self.params
                    .create(ui, request.body_params.clone(), "Body".to_string());
            }
        }

        ui.separator();

        // self.state.plot_demo.ui(ui);
        self.line_demo.ui(ui, &self.chart);
        response
    }
}

async fn send_request(request: &Request) -> Result<String, String> {
    let url = request.url.clone();
    let body = request.body_params.clone();
    let headers = request.headers_params.clone();
    let method = request.method;

    let rng = SimpleRGen::new();
    let (wait_ms, _) = Gen::gen_in_range(1000, 6000).run(&rng);
    info!("Waiting {wait_ms:?} ms.");

    tokio::time::sleep(tokio::time::Duration::from_millis(wait_ms as u64)).await;
    match api_request(method, &url, &body, &headers).await {
        Ok((response, _header_map)) => Ok(format!("Respuesta exitosa: {:?}", response)),
        Err(e) => Err(format!("Error al realizar la solicitud: {:?}", e)),
    }
}

async fn send_concurrent_requests(
    tx: Sender<PerformanceRequestMessage>,
    request: Request,
    n_total_requests: u32,
    n_concurrent_requests: u16,
) {
    let semaphore = Arc::new(Semaphore::new(n_concurrent_requests as usize));

    let mut tasks = Vec::with_capacity(n_total_requests as usize);

    for _ in 0..n_total_requests {
        let data = request.clone();
        let tx_cloned = tx.clone();
        let permit = semaphore.clone().acquire_owned().await.unwrap();

        let task = tokio::spawn(async move {
            let _permit = permit; // Este es el "hueco" que se libera al terminar la tarea
            send_and_process_request(data, tx_cloned).await
        });

        tasks.push(task);
    }

    join_all(tasks).await;
}

async fn send_and_process_request(request: Request, tx: Sender<PerformanceRequestMessage>) {
    // TODO: Tendré que añadir:
    //   - el tiempo que tarda la petición en ejecutarse
    //   - si la petición es exitosa o no (campo HttpStatus en Performancerequestmessage)
    //   - el tamaño de la respueta recibida
    // Request is send_request, process is tx.send
    let begin = Instant::now();

    match send_request(&request).await {
        Ok(response) => {
            let end = Instant::now();
            let msg = PerformanceRequestMessage {
                _msg: response,
                duration: end - begin,
            };
            tx.send(msg).await.unwrap();
        }
        Err(e) => {
            let end = Instant::now();
            let msg = PerformanceRequestMessage {
                _msg: e,
                duration: end - begin,
            };
            tx.send(msg).await.unwrap();
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
struct LineDemo {
    animate: bool,
    time: f64,
    circle_radius: f64,
    circle_center: Pos2,
    square: bool,
    proportional: bool,
    coordinates: bool,
    show_axes: bool,
    show_grid: bool,
    line_style: LineStyle,
}

impl Default for LineDemo {
    fn default() -> Self {
        Self {
            animate: !cfg!(debug_assertions),
            time: 0.0,
            circle_radius: 1.5,
            circle_center: Pos2::new(0.0, 0.0),
            square: false,
            proportional: true,
            coordinates: true,
            show_axes: true,
            show_grid: true,
            line_style: LineStyle::Solid,
        }
    }
}

impl LineDemo {
    fn ui(&mut self, ui: &mut egui::Ui, chart: &Vec<PerformanceRequestMessage>) -> Response {
        let points: PlotPoints = chart
            .iter()
            .enumerate()
            .map(|(idx, el)| {
                [
                    idx as f64, // r * t.cos() + self.circle_center.x as f64,
                    el.duration.as_secs_f64(),
                ]
            })
            .collect();
        let line = Line::new(points)
            .color(Color32::from_rgb(100, 200, 100))
            .style(self.line_style)
            .name("circle");

        let mut plot = Plot::new("lines_demo")
            .legend(Legend::default())
            .y_axis_width(4)
            .show_axes(self.show_axes)
            .show_grid(self.show_grid);
        if self.square {
            plot = plot.view_aspect(1.0);
        }
        if self.proportional {
            plot = plot.data_aspect(1.0);
        }
        if self.coordinates {
            plot = plot.coordinates_formatter(Corner::LeftBottom, CoordinatesFormatter::default());
        }
        plot.show(ui, |plot_ui| {
            plot_ui.line(line);
        })
        .response
    }
}
