// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use eframe::egui;

use crate::httpm::request::Request;
use crate::httpm::state::{HttpAppState, HttpRequestAction};

use crate::common::internationalization::I18n;
use crate::httpm::view::HttpView;

use super::context_menus;

impl HttpView {
    pub fn show_sidenav(&mut self, ctx: &egui::Context, app_st: &mut HttpAppState, i18n: &I18n) {
        egui::SidePanel::left("side_panel")
            .resizable(true)
            .show(ctx, |ui| {
                let current_workspace = &mut app_st.workspaces[app_st.current_workspace_idx];

                if ui.button("Guardar petición").clicked() {
                    let new_request = Request {
                        name: self.url.clone(),
                        method: self.method,
                        url: self.url.clone(),
                        multipart: self.state.upload_files,
                        body_params: self.body.params.clone(),
                        headers_params: self.headers.params.clone(),
                    };
                    current_workspace.requests.push(new_request);
                    self.state.has_request_some_change = false;
                    self.state.selected_request_idx = None;
                }

                ui.separator();

                let mut buttons = Vec::with_capacity(current_workspace.requests.len());
                let popup_id = ui.make_persistent_id("edit-request-popup-id");

                // --> Listado de peticiones en el ws actual <--
                egui::ScrollArea::vertical().show(ui, |ui| {
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

                                let show_update = self.state.has_request_some_change
                                    && self.state.selected_request_idx.unwrap_or(usize::MAX) == idx;
                                button.context_menu(|ui| {
                                    context_menus::request(
                                        ui,
                                        idx,
                                        &mut self.state.selected_request_idx,
                                        &mut self.state.selected_request_action,
                                        show_update,
                                        i18n,
                                        |lui| lui.memory_mut(|mem| mem.toggle_popup(popup_id)),
                                    )
                                });

                                if button.clicked() {
                                    self.state.selected_request_idx = Some(idx);
                                    self.method = request.method;
                                    self.url = request.url.clone();
                                    self.body.params = request.body_params.clone();
                                    self.body.multipart = request.multipart;
                                    self.headers.params = request.headers_params.clone();
                                    self.response.clear();
                                    self.state.has_request_some_change = false;
                                }
                                buttons.push(button);
                            });
                        });
                    }
                });

                // Para evitar que se cierre la próxima vez.
                if let Some(idx) = self.state.selected_request_idx {
                    match self.state.selected_request_action {
                        HttpRequestAction::None => (),
                        HttpRequestAction::Rename => {
                            let button = &buttons[idx];
                            egui::popup::popup_below_widget(ui, popup_id, button, |ui| {
                                ui.set_min_width(200.0);
                                ui.label(&i18n.http_edit_request_name);
                                ui.text_edit_singleline(&mut current_workspace.requests[idx].name)
                                    .request_focus();
                            });
                            self.state.selected_request_action = HttpRequestAction::None;
                        }
                        HttpRequestAction::Delete => {
                            app_st.workspaces[app_st.current_workspace_idx]
                                .requests
                                .remove(idx);
                            self.state.selected_request_action = HttpRequestAction::None;
                            self.state.selected_request_idx = None;
                            self.state.has_request_some_change = false;
                        }
                        HttpRequestAction::Update => {
                            let current_wsp = &mut app_st.workspaces[app_st.current_workspace_idx];
                            let current_req =
                                &mut current_wsp.requests[self.state.selected_request_idx.unwrap()];
                            current_req.method = self.method;
                            current_req.url = self.url.clone();
                            current_req.body_params = self.body.params.clone();
                            current_req.headers_params = self.headers.params.clone();
                            current_req.multipart = self.body.multipart;
                            println!("Current request multipart? {}", current_req.multipart);
                            self.state.has_request_some_change = false;
                            self.state.selected_request_action = HttpRequestAction::None;
                        }
                    };
                }
            });
    }
}
