// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use std::path::PathBuf;

use eframe::egui;

use crate::common::internationalization::I18nHttp;
use crate::httpm::state::HttpAppState;
use crate::httpm::view::HttpView;
use crate::httpm::workspace::Workspace;

impl HttpView {
    pub fn show_ws_subheaders(
        &mut self,
        ctx: &egui::Context,
        app_st: &mut HttpAppState,
        i18n: &I18nHttp,
    ) {
        egui::TopBottomPanel::top("subheader").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("+").clicked() {
                    let new_workspace = Workspace {
                        id: app_st.workspaces.len(),
                        name: format!("Workspace {}", app_st.workspaces.len() + 1),
                        ..Workspace::default()
                    };
                    app_st.workspaces.push(new_workspace);
                }

                let edit_button = ui.button("edit");
                let popup_id = ui.make_persistent_id("my_unique_id");
                if edit_button.clicked() {
                    ui.memory_mut(|mem| mem.toggle_popup(popup_id));
                }
                let mut idx_to_delete: Option<usize> = None;

                if let Some(workspace) = app_st.workspaces.get_mut(app_st.current_workspace_idx) {
                    egui::popup::popup_below_widget(ui, popup_id, &edit_button, |ui| {
                        ui.set_min_width(200.0);
                        ui.label(&i18n.http_btn_edit_ws_name);
                        ui.text_edit_singleline(&mut workspace.name).request_focus();
                        if ui.button(&i18n.http_btn_delete_ws).clicked() {
                            idx_to_delete = Some(app_st.current_workspace_idx);
                        }
                    });
                }

                if let Some(idx) = idx_to_delete {
                    if idx == app_st.workspaces.len() - 1 && app_st.current_workspace_idx > 0 {
                        app_st.current_workspace_idx -= 1;
                    }

                    app_st.workspaces.remove(idx);

                    if app_st.workspaces.is_empty() {
                        app_st.workspaces = vec![Workspace::default()];
                    }
                }

                for (idx, workspace) in app_st.workspaces.iter_mut().enumerate() {
                    if ui
                        .selectable_value(&mut app_st.current_workspace_idx, idx, &workspace.name)
                        .clicked()
                    {
                        self.state.selected_request_idx = Some(0);
                        let fst_request = workspace.requests.first();

                        if let Some(req) = fst_request {
                            self.method = req.method;
                            self.url = req.url.clone();
                            self.body.multipart = req.multipart;
                            self.body.params = req.body_params.clone();
                            self.body.files = vec![vec![]; req.body_params.len()];
                            for (idx, param) in self.body.params.iter().enumerate() {
                                let has_files = param.2;
                                if has_files {
                                    self.body.files[idx] = param
                                        .1
                                        .split(',')
                                        .map(|s| PathBuf::from(s))
                                        .collect::<Vec<PathBuf>>();
                                }
                            }

                            self.headers.params = req.headers_params.clone();
                            self.response.clear();
                            self.state.has_request_some_change = false;
                        }
                    }
                }
            });
        });
    }
}
