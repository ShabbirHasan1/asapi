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
    error, info,
    redism::{
        presenter::{self, RedisMenu},
        view::RedisView,
    },
};

impl RedisView {
    pub fn show_sets(&mut self, ui: &mut egui::Ui, i18n: &I18n) {
        self.show(ui, i18n, RedisMenu::Set);
    }

    pub fn show_sorted_sets(&mut self, ui: &mut egui::Ui, i18n: &I18n) {
        self.show(ui, i18n, RedisMenu::SortedSet);
    }

    fn show(&mut self, ui: &mut egui::Ui, i18n: &I18n, menu: RedisMenu) {
        ui.set_width(ui.available_width());
        let tmp = if menu == RedisMenu::Set {
            &self.state.sets
        } else {
            &self.state.sorted_sets
        };

        for (set_key, set_values) in tmp {
            ui.code(set_key).context_menu(|ui| {
                if ui.button(&i18n.redis_delete_ds).clicked() {
                    match presenter::delete_key(
                        &self.state.current_connection.host,
                        &self.state.current_connection.port,
                        set_key,
                    ) {
                        Ok(s) => {
                            self.state.must_scan = true;
                            info!("{:?}", s);
                        }
                        Err(e) => error!("{:?}", e),
                    }
                    ui.close_menu();
                }
            });
            ui.indent(set_key, |ui| ui.label(set_values.join(", ")));

            ui.end_row();
        }
    }
}
