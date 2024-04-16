// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use eframe::egui::{self, Context};

use crate::redism::{
    presenter::{self, RedisMenu},
    state::RedisAppState,
    view::RedisView,
};

impl RedisView {
    pub fn sidenav_show(&mut self, ctx: &Context, app_st: &RedisAppState) {
        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            if ui.button("\u{27f3} Load").clicked() {
                let _ = presenter::scan(&mut self.state, &app_st);
            }
            ui.separator();
            ui.set_width(200.0);
            // TODO: Mejor separar PubSub puesto que no está conectado con los demás... solo puede ser
            // conexión en vivo.
            for option in RedisMenu::iterator() {
                // ui.label(format!("{idx} -- {:#?}", option));
                let opt = option.clone();
                if ui
                    .selectable_value(&mut self.state.selected_menu, opt, format!("{:#?}", option))
                    .clicked()
                {}
            }
        });
    }
}
