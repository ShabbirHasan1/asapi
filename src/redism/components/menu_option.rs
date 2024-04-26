// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use eframe::egui;

use crate::{
    common::internationalization::I18n,
    redism::{
        connection::{scan, RedisMenu},
        state::RedisLocalState,
    },
};

#[inline(always)]
pub fn selectable_and_info(
    ui: &mut egui::Ui,
    st: &mut RedisLocalState,
    i18n: &I18n,
    menu_option: RedisMenu,
    hover_cb: impl Fn(&mut egui::Ui, &RedisLocalState),
) {
    ui.label(egui::RichText::new("Info").color(egui::Color32::from_rgb(128, 128, 128)))
        .on_hover_ui(|ui| {
            hover_cb(ui, st);
        });
    ui.selectable_value(
        &mut st.selected_menu,
        menu_option,
        format!("{menu_option:?}"),
    )
    .context_menu(|ui| {
        if ui.button(&i18n.redis_load).clicked() {
            if let Err(err) = scan(st, menu_option) {
                st.last_result = Some(Err(err.to_string()));
            }
            ui.close_menu();
        }
    });
}
