// -------------------------------------------------------------------------
// Copyright (C) 2023 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use eframe::egui;
use egui_json_tree::JsonTree;
use reqwest::header::HeaderMap;
use serde_json::Value;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{self, Receiver, Sender};

use super::components::body_params::BodyParams;
use super::components::header_params::HeaderParams;
use super::methods::HttpMethod;
use super::request::api_request;
use super::state::{HttpLocalState, HttpPanel};
use super::workspace::{Request, Workspace};

use crate::app_state::AppState;
use crate::common::fs::save_state;
use crate::common::internationalization::I18n;
use crate::common::syntax_highlighting::{highlight, CodeTheme};

pub struct HttpView {
    tx: Sender<(String, HeaderMap)>,
    rx: Receiver<(String, HeaderMap)>,
    request_allowed: bool,
    url: String,
    method: HttpMethod,
    response: String,
    show_headers: bool,
    show_body: bool,
    body: BodyParams,
    headers: HeaderParams,
    state: HttpLocalState,
    file_name: String,
}

impl Default for HttpView {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel(8); // Realmente con 1 sería suficiente. No podemos usar oneshot porque no permite clonarlo y no quiero meter la función send_request en línea

        Self {
            tx,
            rx,
            request_allowed: true,
            url: String::from("https://jsonplaceholder.typicode.com/todos"), //String::from::new(),
            method: HttpMethod::Get,
            response: String::new(),
            body: BodyParams::default(),
            headers: HeaderParams::default(),
            state: HttpLocalState::default(),
            show_headers: true,
            show_body: true,
            file_name: "asapi_workspaces.json".into(),
        }
    }
}

impl HttpView {
    pub fn update(
        &mut self,
        ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        state: &mut AppState,
        rt: &Runtime,
        i18n: &I18n,
    ) {
        // =======================================
        // Preparación de cada ciclo
        // =======================================
        if self.state.panel == HttpPanel::Performance {
            ctx.request_repaint();
        }
        if !self.state.has_been_updated {
            if state.http.workspaces[state.http.current_workspace_idx]
                .requests
                .len()
                > 0
            {
                let idx = 0;
                let request =
                    state.http.workspaces[state.http.current_workspace_idx].requests[idx].clone();
                self.state.selected_request_idx = Some(idx);
                self.method = request.method;
                self.url = request.url.clone();
                self.body.params = request.body_params.clone();
                self.headers.params = request.headers_params.clone();
                self.state.has_request_some_change = false;
            }
            self.state.has_been_updated = true;
        }
        while let Ok(tuple) = self.rx.try_recv() {
            // self.messages.push(msg);
            self.response = tuple.0;
            self.state.response_headers = tuple.1;
            self.request_allowed = true;
        }
        let events = ctx.input(|i| i.events.clone());
        for event in &events {
            if let egui::Event::Paste(pasted_text) = event {
                println!("{}", pasted_text);
            }
        }

        // egui::introspection::font_id_ui(ui, &mut self.configuration.font_id);

        // ===================================================================
        // == Subheader
        // ===================================================================
        egui::TopBottomPanel::top("subheader").show(ctx, |ui| {
            // --> Workspaces <--
            ui.horizontal(|ui| {
                if ui.button("+").clicked() {
                    let new_workspace = Workspace {
                        id: state.http.workspaces.len(),
                        name: format!("Workspace {}", state.http.workspaces.len() + 1),
                        ..Workspace::default()
                    };
                    state.http.workspaces.push(new_workspace);
                }

                let edit_button = ui.button("edit");
                let popup_id = ui.make_persistent_id("my_unique_id");
                if edit_button.clicked() {
                    ui.memory_mut(|mem| mem.toggle_popup(popup_id));
                }
                let mut idx_to_delete: Option<usize> = None;

                if let Some(workspace) = state
                    .http
                    .workspaces
                    .get_mut(state.http.current_workspace_idx)
                {
                    egui::popup::popup_below_widget(ui, popup_id, &edit_button, |ui| {
                        ui.set_min_width(200.0);
                        ui.label(&i18n.http_btn_edit_ws_name);
                        ui.text_edit_singleline(&mut workspace.name).request_focus();
                        if ui.button(&i18n.http_btn_delete_ws).clicked() {
                            idx_to_delete = Some(state.http.current_workspace_idx);
                        }
                    });
                }

                if let Some(idx) = idx_to_delete {
                    if idx == state.http.workspaces.len() - 1
                        && state.http.current_workspace_idx > 0
                    {
                        state.http.current_workspace_idx -= 1;
                    }

                    state.http.workspaces.remove(idx);

                    if state.http.workspaces.is_empty() {
                        state.http.workspaces = vec![Workspace::default()];
                    }
                }

                for (idx, workspace) in state.http.workspaces.iter_mut().enumerate() {
                    let selectable_value = ui.selectable_value(
                        &mut state.http.current_workspace_idx,
                        idx,
                        &workspace.name,
                    );

                    if selectable_value.clicked() {
                        // Acciones cuando se selecciona un espacio de trabajo
                        println!(
                            "CLICK  current_idx: {}, idx: {}",
                            state.http.current_workspace_idx, idx
                        );
                    }
                }
            });
        });

        // ===================================================================
        // == Lateral
        // ===================================================================
        if state.http.show_sidebar {
            egui::SidePanel::left("side_panel")
                .resizable(true)
                // .max_width(200.0)
                .show(ctx, |ui| {
                    // ui.set_width(200.0);
                    // ui.heading("Requests");

                    let current_workspace =
                        &mut state.http.workspaces[state.http.current_workspace_idx];

                    if ui.button("Guardar petición").clicked() {
                        let new_request = Request {
                            name: self.url.clone(),
                            method: self.method,
                            url: self.url.clone(),
                            body_params: self.body.params.clone(),
                            headers_params: self.headers.params.clone(),
                        };
                        println!("{:?}", new_request);
                        current_workspace.requests.push(new_request);
                        self.state.has_request_some_change = false;
                        self.state.selected_request_idx = None;
                    }

                    ui.separator();

                    let mut buttons = Vec::with_capacity(current_workspace.requests.len());
                    let popup_id = ui.make_persistent_id("edit-request-popup-id");

                    // --> Listado de peticiones en el ws actual <--
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        // let (response, painter) = ui.allocate_space(ui.available_size());
                        // let button_min_width = if state.http.show_sidebar { 200.0 } else { 0.0 };
                        for (idx, request) in current_workspace.requests.iter().enumerate() {
                            ui.horizontal(|ui| {
                                let stroke_color = if self.state.selected_request_idx.is_some()
                                    && self.state.has_request_some_change
                                {
                                    egui::Color32::DARK_RED
                                } else {
                                    egui::Color32::LIGHT_GREEN
                                };
                                let stroke_width =
                                    if let Some(selected_idx) = self.state.selected_request_idx {
                                        if selected_idx == idx {
                                            1.0
                                        } else {
                                            0.0
                                        }
                                    } else {
                                        0.0
                                    };
                                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                                    let button = ui.add(
                                        egui::Button::new(format!(
                                            "{} - {}",
                                            request.method, request.name
                                        ))
                                        .min_size(egui::vec2(200.0, 16.0))
                                        .stroke(egui::Stroke::new(stroke_width, stroke_color)),
                                    );
                                    // if self.state.has_request_some_change
                                    // && self.state.selected_request_idx.is_some()
                                    // && ui
                                    // .add(
                                    // egui::Button::new(&i18n.http_btn_update_request)
                                    // .fill(egui::Color32::LIGHT_RED),
                                    // )
                                    // .clicked()
                                    // {
                                    // let current_wsp =
                                    // &mut state.http.workspaces[state.http.current_workspace_idx];
                                    // let current_req =
                                    // &mut current_wsp.requests[self.state.selected_request_idx.unwrap()];
                                    // current_req.method = self.method;
                                    // current_req.url = self.url.clone();
                                    // current_req.body_params = self.body.params.clone();
                                    // current_req.headers_params = self.headers.params.clone();

                                    // let _ = save_state(state, &self.file_name);
                                    // self.state.has_request_some_change = false;
                                    // ui.separator();
                                    // }

                                    let show_update = self.state.has_request_some_change
                                        && self.state.selected_request_idx.is_some()
                                        && self.state.selected_request_idx.unwrap() == idx;
                                    button.context_menu(|ui| {
                                        super::components::context_menus::request(
                                            ui,
                                            idx,
                                            &mut self.state.selected_request_idx,
                                            &mut self.state.selected_request_action,
                                            show_update,
                                            |lui| lui.memory_mut(|mem| mem.toggle_popup(popup_id)),
                                        )
                                    });

                                    if button.clicked() {
                                        self.state.selected_request_idx = Some(idx);
                                        self.method = request.method;
                                        self.url = request.url.clone();
                                        self.body.params = request.body_params.clone();
                                        self.headers.params = request.headers_params.clone();
                                        self.response.clear();
                                        self.state.has_request_some_change = false;
                                        println!("{} {}", idx, state.http.current_workspace_idx);
                                    }
                                    buttons.push(button);
                                });
                            });
                        }
                    });

                    // Para evitar que se cierre la próxima vez.
                    if let Some(idx) = self.state.selected_request_idx {
                        if let Some(action) = &self.state.selected_request_action {
                            if action == "Delete" {
                                state.http.workspaces[state.http.current_workspace_idx]
                                    .requests
                                    .remove(idx);
                                self.state.selected_request_action = None;
                                self.state.selected_request_idx = None;
                                self.state.has_request_some_change = false;
                            } else if action == "Rename" {
                                let button = &buttons[idx];
                                // ui.memory_mut(|mem| mem.toggle_popup(popup_id));
                                egui::popup::popup_below_widget(ui, popup_id, button, |ui| {
                                    ui.set_min_width(200.0);
                                    ui.label("Editar nombre de la petición");
                                    ui.text_edit_singleline(
                                        &mut current_workspace.requests[idx].name,
                                    )
                                    .request_focus();
                                });
                            } else if action == "Update" {
                                let current_wsp =
                                    &mut state.http.workspaces[state.http.current_workspace_idx];
                                let current_req = &mut current_wsp.requests
                                    [self.state.selected_request_idx.unwrap()];
                                current_req.method = self.method;
                                current_req.url = self.url.clone();
                                current_req.body_params = self.body.params.clone();
                                current_req.headers_params = self.headers.params.clone();

                                let _ = save_state(state, &self.file_name);
                                self.state.has_request_some_change = false;
                            }
                        }
                    }
                });
        }

        // =================================
        // == Central
        // =================================
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.state.panel == HttpPanel::Regular {
                // --> Introducción URL <--
                ui.horizontal(|ui| {
                    // if self.state.has_request_some_change
                    //     && self.state.selected_request_idx.is_some()
                    //     && ui
                    //         .add(
                    //             egui::Button::new(&i18n.http_btn_update_request)
                    //                 .fill(egui::Color32::LIGHT_RED),
                    //         )
                    //         .clicked()
                    // {
                    //     let current_wsp =
                    //         &mut state.http.workspaces[state.http.current_workspace_idx];
                    //     let current_req =
                    //         &mut current_wsp.requests[self.state.selected_request_idx.unwrap()];
                    //     current_req.method = self.method;
                    //     current_req.url = self.url.clone();
                    //     current_req.body_params = self.body.params.clone();
                    //     current_req.headers_params = self.headers.params.clone();

                    //     let _ = save_state(state, &self.file_name);
                    //     self.state.has_request_some_change = false;
                    //     ui.separator();
                    // }

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
                    ui.label("Method:");
                    let response = egui::ComboBox::from_id_source("Method")
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

                    // ui.with_layout(egui::Layout::left_to_right(egui::Align::Max), |ui| {
                    // --> Solo mostramos el rendimiento en caso de que tengamos una petición seleccionada <--
                    // Esto implica que para poder testear rendimiento hay que guardar la petición.
                    if let Some(_) = self.state.selected_request_idx {
                        if self.request_allowed
                            && ui.button(&i18n.http_send_to_http_performance).clicked()
                        {
                            self.state.panel = HttpPanel::Performance;
                        }
                    }
                    // });
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
                    if let Some(value) = self.headers.create(ui) {
                        self.state.has_request_some_change = value;
                    }
                }

                if self.show_body {
                    if let Some(value) = self.body.create(ctx, ui, self.method) {
                        self.state.has_request_some_change = value;
                    }
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

                let theme = CodeTheme::from_memory(ui.ctx());
                let mut json_layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
                    let mut layout_job = highlight(ui.ctx(), &theme, string, "json");
                    layout_job.wrap.max_width = wrap_width;
                    ui.fonts(|f| f.layout_job(layout_job))
                };
                ui.horizontal(|ui| {
                    ui.label("Parse JSON Response");
                    ui.add(crate::widgets::toggle_switch::toggle(
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
                            &self
                                .response
                                .trim_start_matches("\"")
                                .trim_end_matches("\""),
                        )
                        .unwrap_or("Error Parsing".into());
                        ui.set_width(ui.available_width());
                        JsonTree::new("http_response", &v)
                            .default_expand(egui_json_tree::DefaultExpand::ToLevel(10))
                            .show(ui);
                    } else {
                        ui.add(
                            egui::TextEdit::multiline(&mut self.response)
                                .font(egui::TextStyle::Monospace) // for cursor height
                                .code_editor()
                                .desired_rows(10)
                                .lock_focus(true)
                                .desired_width(f32::INFINITY)
                                .layouter(&mut json_layouter),
                        );
                    }
                });
            } else {
                match self.state.selected_request_idx {
                    Some(idx) => {
                        let mut request = state.http.workspaces[state.http.current_workspace_idx]
                            .requests[idx]
                            .clone();

                        let close_performance_panel =
                            self.state.performance_panel.ui(ui, rt, &i18n, &mut request);

                        if close_performance_panel {
                            self.state.panel = HttpPanel::Regular;
                        }
                    }
                    None => (),
                }
            }
        });
    }

    fn send_request(&mut self, ctx: &egui::Context, rt: &Runtime) {
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
