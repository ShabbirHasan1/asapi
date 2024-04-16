// -------------------------------------------------------------------------
// Copyright (C) 2023 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use crate::{
    error, info,
    redism::{
        presenter::delete_stream_message,
        utils::{value_map_to_string, value_map_to_string_map},
    },
};
use eframe::egui;
use egui::Ui;
use redis::Value;
use serde_json::{self};
use std::collections::HashMap;

pub fn stream_msg(
    ui: &mut Ui,
    stream_name: &str,
    id: String,
    option: Option<&HashMap<String, Value>>,
    command: &mut String,
) -> bool {
    ui.add_enabled_ui(option.is_some(), |ui| {
        if ui.button("Send Again").clicked() {
            let json_string_option = option.map(|hm| {
                let pairs = value_map_to_string(hm);
                format!("XADD {} * {}", stream_name, pairs)
            });

            if let Some(json_string) = json_string_option {
                *command = json_string;
            }

            ui.close_menu();
        }
    });

    ui.menu_button("Copy Message as...", |ui| {
        if ui.button("JSON").clicked() {
            if let Some(text) = option.map(value_map_to_string_map) {
                if let Ok(json_string) = serde_json::to_string(&text) {
                    ui.ctx().copy_text(json_string);
                }
            }
            ui.close_menu();
        }

        if ui.button("Redis Params").clicked() {
            if let Some(text) = option.map(value_map_to_string) {
                ui.ctx().copy_text(text);
            }
            ui.close_menu();
        }
    });

    if ui.button("Delete").clicked() {
        match delete_stream_message("127.0.0.1", 6379, stream_name, &id) {
            Ok(k) => {
                info!("Borrada clave {}.", k);
                ui.close_menu();
                return true;
            }
            Err(e) => error!("Error borrando clave {}", e),
        }
        ui.close_menu();
    }

    false
}
