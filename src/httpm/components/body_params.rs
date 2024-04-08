// -------------------------------------------------------------------------
// Copyright (C) 2023 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use crate::http_module::methods::HttpMethod;
use eframe::egui;
use egui_json_tree::JsonTree;
use serde_json::Value as JsonValue;

#[derive(Default)]
pub struct BodyParams {
    pub params: Vec<(String, String)>,
}

impl BodyParams {
    pub fn create(
        &mut self,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
        method: HttpMethod,
    ) -> Option<bool> {
        let mut has_changed = None;
        let mut idx_to_del: Option<usize> = None;
        let editable = !(method == HttpMethod::Get || method == HttpMethod::Delete);

        if !editable {
            return None;
        }

        ui.horizontal(|ui| {
            ui.label("Body");
            if editable {
                if ui.button("+").clicked() {
                    self.params.push((String::new(), String::new()));
                    has_changed = Some(true);
                }
            }
        });

        for i in 0..self.params.len() {
            ui.horizontal(|ui| {
                if ui.button("-").clicked() {
                    idx_to_del = Some(i);
                    has_changed = Some(true);
                }

                let (header_key, header_value) = &mut self.params[i];
                ui.add(egui::TextEdit::singleline(header_key).hint_text("key"));

                ui.label(":");
                ui.add(
                    egui::TextEdit::singleline(header_value)
                        .hint_text("value")
                        .desired_width(f32::INFINITY),
                );
            });
        }
        if let Some(idx) = idx_to_del {
            if idx < self.params.len() {
                self.params.remove(idx);
            }
        }

        if !self.params.is_empty() {
            let json_map: serde_json::Map<String, JsonValue> = self
                .params
                .iter()
                .map(|(k, v)| (k.clone(), serde_json::from_str(v).unwrap_or_default()))
                .collect();
            let json_value = JsonValue::Object(json_map);
            JsonTree::new("http_body", &json_value)
                .default_expand(egui_json_tree::DefaultExpand::ToLevel(2))
                .show(ui);
        }

        if !editable {
            ctx.set_style(ctx.style().clone());
        }

        has_changed
    }
}
