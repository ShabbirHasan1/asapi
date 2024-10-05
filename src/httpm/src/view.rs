// -------------------------------------------------------------------------
// Copyright (C) 2023 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use components::toggle_switch;
use eframe::egui;
use egui_json_tree::JsonTree;
use reqwest::header::HeaderMap;
use serde_json::Value;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{self, Receiver, Sender};

use common::internationalization::I18nHttp;

use super::components::body_params::BodyParams;
use super::components::header_params::HeaderParams;
use super::methods::HttpMethod;
use super::request::{self, api_request};
use super::state::{HttpAppState, HttpLocalState, HttpPanel};


pub struct HttpView {
    tx: Sender<(String, HeaderMap)>,
    rx: Receiver<(String, HeaderMap)>,
    request_allowed: bool,
    pub url: String,
    pub method: HttpMethod,
    pub response: String,
    show_headers: bool,
    show_body: bool,
    pub body: BodyParams,
    pub headers: HeaderParams,
    pub state: HttpLocalState,
}

impl Default for HttpView {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel(8); // Realmente con 1 sería suficiente. No podemos usar oneshot porque no permite clonarlo y no quiero meter la función send_request en línea

        Self {
            tx,
            rx,
            request_allowed: true,
            url: String::from("https://jsonplaceholder.typicode.com/todos"), //String::from::new(),
            method: Default::default(),
            response: String::new(),
            body: BodyParams::default(),
            headers: Default::default(),
            state: Default::default(),
            show_headers: true,
            show_body: true,
        }
    }
}

impl HttpView {
    pub fn update(
        &mut self,
        ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        app_st: &mut HttpAppState,
        rt: &Runtime,
        i18n: &I18nHttp,
    ) {
        // =======================================
        // Preparación de cada ciclo
        // =======================================
        // Cuando estamos en mododo `Performance` necesitamos repintado continuo.
        if self.state.panel == HttpPanel::Performance {
            ctx.request_repaint();
        }
        // Solo en el primer renderizado y si hay workspaces cogemos la primera petición
        // del `workspace` y rellenamos los datos con ella para tener una por defecto.
        if !self.state.is_not_first_render {
            if !app_st.workspaces[app_st.current_workspace_idx]
                .requests
                .is_empty()
            {
                let idx = 0;
                let request = app_st.workspaces[app_st.current_workspace_idx].requests[idx].clone();
                self.state.selected_request_idx = Some(idx);
                self.method = request.method;
                self.url = request.url;
                self.body.params = request.body_params;
                self.headers.params = request.headers_params;
                self.state.has_request_some_change = false;
            }
            self.state.is_not_first_render = true;
        }

        while let Ok(tuple) = self.rx.try_recv() {
            self.response = tuple.0;
            self.state.response_headers = tuple.1;
            self.request_allowed = true;
        }

        // ===================================================================
        // == Subheader
        // ===================================================================
        self.show_ws_subheaders(ctx, app_st, i18n);

        // ===================================================================
        // == Lateral
        // ===================================================================
        if app_st.show_sidebar {
            self.show_sidenav(ctx, app_st, i18n);
        }

        // =================================
        // == Central
        // =================================
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.state.panel == HttpPanel::Regular {
                // --> Introducción URL <--
                ui.horizontal(|ui| {
                    let url_text = egui::TextEdit::singleline(&mut self.url);
                    ui.label("URL:");
                    let url_input_widget = ui.add_sized(ui.available_size(), url_text);

                    if url_input_widget.changed() {
                        self.state.has_request_some_change = true;
                    }

                    if url_input_widget.lost_focus()
                        && ctx.input(|i| i.key_pressed(egui::Key::Enter))
                    {
                        self.send_request(ctx, rt);
                        url_input_widget.request_focus();
                    }
                });

                // --> Elección Verbo HTTP <--
                ui.horizontal(|ui| {
                    ui.label(&i18n.http_request_method);
                    let response = egui::ComboBox::from_id_salt(&i18n.http_request_method)
                        .selected_text(self.method.to_string())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.method, HttpMethod::Get, "GET");
                            ui.selectable_value(&mut self.method, HttpMethod::Post, "POST");
                            ui.selectable_value(&mut self.method, HttpMethod::Put, "PUT");
                            ui.selectable_value(&mut self.method, HttpMethod::Delete, "DELETE");
                        });

                    if response.response.changed() {
                        self.state.has_request_some_change = true;
                    }

                    if self.request_allowed && ui.button(&i18n.http_btn_send_request).clicked() {
                        self.send_request(ctx, rt);
                    }

                    // --> Solo mostramos el rendimiento en caso de que tengamos una petición seleccionada <--
                    // Esto implica que para poder testear rendimiento hay que guardar la petición.
                    if self.state.selected_request_idx.is_some()
                        && self.request_allowed
                        && ui.button(&i18n.http_send_to_http_performance).clicked()
                    {
                        self.state.panel = HttpPanel::Performance;
                    }
                });

                ui.separator();

                // --> Elección header/body <--
                ui.horizontal(|ui| {
                    if ui.selectable_label(self.show_headers, "Headers").clicked() {
                        self.show_headers = !self.show_headers;
                    }

                    if ui.selectable_label(self.show_body, "Body").clicked() {
                        self.show_body = !self.show_body;
                    }
                });

                if self.show_headers {
                    if let Some(value) = self.headers.show_headers(ui) {
                        self.state.has_request_some_change = value;
                    }
                }

                if self.show_body {
                    if let Some(value) = self.body.show(ctx, ui, self.method, &mut self.state, i18n)
                    {
                        self.state.has_request_some_change = value;
                    }
                }

                if self.state.files.must_read {
                    self.state.files.selected_mode = Some(self.state.files.file_dialog.mode());
                    self.state.files.current_state = Some(self.state.files.file_dialog.state());

                    self.state.files.file_dialog.update(ctx);
                }

                ui.separator();

                // --> Zona para mostrar respuesta <--
                egui::CollapsingHeader::new("Response headers")
                    .default_open(false)
                    .show(ui, |ui| {
                        egui::Grid::new("response_headers")
                            .spacing(egui::vec2(ui.spacing().item_spacing.x * 2.0, 0.0))
                            .show(ui, |ui| {
                                for h in &self.state.response_headers {
                                    ui.label(h.0.to_string());
                                    ui.label(h.1.to_str().unwrap());
                                    ui.end_row();
                                }
                            })
                    });

                ui.horizontal(|ui| {
                    ui.label("Parse JSON Response");
                    ui.add(toggle_switch::toggle(
                        &mut self.state.show_hide_json_response,
                    ));
                    if !self.response.is_empty() && ui.button("Copy Response").clicked() {
                        ui.ctx().copy_text(self.response.clone());
                    }
                });

                // Mostramos la respuesta, bien como JSONTree parseado para navegar por él, bien
                // como code_editor, a partir del ejemplo de egui.com/#demo
                egui::ScrollArea::vertical().show(ui, |ui| {
                    if self.state.show_hide_json_response {
                        let v: Value = serde_json::from_str(
                            self.response.trim_start_matches('"').trim_end_matches('"'),
                        )
                        .unwrap_or("Error Parsing".into());
                        ui.set_width(ui.available_width());
                        JsonTree::new("http_response", &v)
                            .default_expand(egui_json_tree::DefaultExpand::ToLevel(10))
                            .show(ui);
                    } else {
                        let theme =
                            egui_extras::syntax_highlighting::CodeTheme::from_memory(ctx, &ui.style());
                        let mut json_layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
                            let mut layout_job = egui_extras::syntax_highlighting::highlight(
                                ui.ctx(),
                                &ui.style(),
                                &theme,
                                string,
                                "json",
                            );
                            layout_job.wrap.max_width = wrap_width;
                            ui.fonts(|f| f.layout_job(layout_job))
                        };
                        ui.add(
                            egui::TextEdit::multiline(&mut self.response)
                                .interactive(false)
                                .font(egui::TextStyle::Monospace) // for cursor height
                                .code_editor()
                                .desired_rows(10)
                                .lock_focus(true)
                                .desired_width(f32::INFINITY)
                                .layouter(&mut json_layouter),
                        );
                    }
                });
            } else if let Some(idx) = self.state.selected_request_idx {
                let mut request =
                    app_st.workspaces[app_st.current_workspace_idx].requests[idx].clone();

                let close_performance_panel =
                    self.state.performance_panel.show(ui, rt, i18n, &mut request);

                if close_performance_panel {
                    self.state.panel = HttpPanel::Regular;
                }
            }
        });
    }

    fn send_request(&mut self, ctx: &egui::Context, rt: &Runtime) {
        self.response.clear();
        if self.body.multipart && self.method == HttpMethod::Post {
            self.file_request(ctx, rt);
        } else {
            self.regular_request(ctx, rt);
        }
    }

    fn file_request(&mut self, ctx: &egui::Context, rt: &Runtime) {
        let url = self.url.clone();
        let body = self.body.params.clone();
        let headers = self.headers.params.clone();
        let method = self.method;
        self.request_allowed = false;

        if method == HttpMethod::Post {
            let tx_cloned = self.tx.clone();
            let ctx_cloned = ctx.clone();
            let files = self.body.files.clone();

            rt.spawn(async move {
                let response =
                    match request::upload_files_in_body_params(&url, &headers, &body, &files).await
                    {
                        Ok(response) => (response, HeaderMap::default()),
                        Err(error) => (
                            format!("Error al realizar la solicitud: {:?}", error),
                            HeaderMap::default(),
                        ),
                    };

                let _ = tx_cloned.send(response).await;
                ctx_cloned.request_repaint();
            });
        }
    }

    // fn _file_request(&mut self, ctx: &egui::Context, rt: &Runtime, upload_many: bool) {
    //     let url = self.url.clone();
    //     let body = self.body.params.clone();
    //     let headers = self.headers.params.clone();
    //     let method = self.method;
    //     self.request_allowed = false;

    //     if method == HttpMethod::Post && self.state.files.files_in_selected_folder.len() > 0 {
    //         let file_path = &self.state.files.files_in_selected_folder[0];
    //         let path_cloned: PathBuf = file_path.clone(); // Esto está bien
    //         let tx_cloned = self.tx.clone();
    //         let ctx_cloned = ctx.clone();
    //         let paths = self.state.files.files_in_selected_folder.clone();

    //         rt.spawn(async move {
    //             let response = match if upload_many {
    //                 request::upload_files(paths, &url, &body, &headers).await
    //             } else {
    //                 request::upload_file(&path_cloned, &url, &body, &headers).await
    //             } {
    //                 Ok(response) => (response, HeaderMap::default()),
    //                 Err(error) => (
    //                     format!("Error al realizar la solicitud: {:?}", error),
    //                     HeaderMap::default(),
    //                 ),
    //             };

    //             let _ = tx_cloned.send(response).await;
    //             ctx_cloned.request_repaint();
    //         });
    //     }
    // }

    fn regular_request(&mut self, ctx: &egui::Context, rt: &Runtime) {
        let url = self.url.clone();
        let body = self.body.params.clone();
        let headers = self.headers.params.clone();
        let method = self.method;
        self.request_allowed = false;

        let tx_cloned = self.tx.clone();
        let ctx_cloned = ctx.clone();
        rt.spawn(async move {
            let response = match api_request(method, &url, &body, &headers).await {
                Ok((response, header_map)) => (response, header_map),
                Err(e) => (
                    format!("Error al realizar la solicitud: {:?}", e),
                    HeaderMap::default(),
                ),
            };
            // TODO: Aquí es posible que sea necesario a futuro manejar el error. Ahora mismo cuando falla la petición
            // la comuncación es correcta y se permite volver a realizar otra petición. Mi miedo es que haya casos en
            // que eso no sea posible y que en dichos casos sea necesario un manejo extra de errores. Por ejemplo, que
            // en caso de fallo mediante otro canal se envíe un mensaje de finalización. Pero ya digo, ahora mismo
            // (231230) no es necesario porque los errores se pueden enviar por el canal.
            let _ = tx_cloned.send(response).await;
            ctx_cloned.request_repaint();
        });
    }
}
