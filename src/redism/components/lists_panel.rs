// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------
use eframe::egui;
use egui_extras::{Size, StripBuilder};

use crate::{
    common::internationalization::I18n,
    components::widgets::ui_text_edit_singleline_hint,
    error, info,
    redism::{
        presenter::{self, run_redis_command, ListPresenter, RedisMenu},
        view::RedisView,
    },
    ui_button_w, ui_button_w100, ui_button_w50,
};

///
/// Comandos
/// acciones sobre izquierda que hay sobre derecha
/// done - LPOP
/// done - LPUSH
/// done - LPUSHX
///
/// acciones sobre derecha
/// done - RPOP
/// done - RPUSH
/// done - RPUSHX
///
/// info
/// LLEN
/// LRANGE
/// LINDEX
///
/// ediciones
/// LMOVE
/// LTRIM
/// LINSERT
/// LREM
/// LSET
/// LTRIM
///
/// acciones bloqueantes
/// BLPOP
/// BRPOP
/// BLMOVE
///
impl RedisView {
    pub fn show_lists(&mut self, ui: &mut egui::Ui, i18n: &I18n) {
        if self.state.selected_menu == RedisMenu::List {
            egui::CollapsingHeader::new("Comandos Disponibles")
                .default_open(true)
                .show(ui, |ui| {
                    ui.columns(2, |uis| {
                        self.left_modify_cmds(&mut uis[0]);
                        self.right_modify_cmds(&mut uis[1]);
                    });
                    ui.separator();

                    ui.columns(2, |_uis| {});
                    ui.separator();

                    ui.columns(2, |_uis| {});
                });

            if !self.state.command_last_result.is_empty() {
                ui.label(&self.state.command_last_result);
            }
        }

        self.lists_display(ui, i18n);
    }

    fn right_modify_cmds(&mut self, ui: &mut egui::Ui) {
        StripBuilder::new(ui)
            .size(Size::exact(20.0))
            .size(Size::exact(20.0))
            .vertical(|mut strip| {
                strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::exact(50.0))
                        .size(Size::exact(70.0))
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Key",
                                    &mut self.state.list_st.rpush_k,
                                );
                            });
                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Element (& Elements)",
                                    &mut self.state.list_st.rpush_vs,
                                );
                            });
                            strip.cell(|ui| {
                                if ui_button_w50!(ui, "RPUSH") {
                                    self.state.command_last_result =
                                        run_redis_command(&self.state.current_connection, |conn| {
                                            ListPresenter::rpush(
                                                conn,
                                                &mut self.state.lists,
                                                &mut self.state.list_st,
                                            )
                                        });
                                }
                            });
                            strip.cell(|ui| {
                                if ui_button_w!(ui, "RPUSHX", 70.0) {
                                    self.state.command_last_result =
                                        run_redis_command(&self.state.current_connection, |conn| {
                                            ListPresenter::rpushx(
                                                conn,
                                                &mut self.state.lists,
                                                &mut self.state.list_st,
                                            )
                                        });
                                }
                            });
                        });
                });

                strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::exact(128.0))
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Key",
                                    &mut self.state.list_st.rpop_k,
                                );
                            });
                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "(Count)",
                                    &mut self.state.list_st.rpop_count,
                                );
                            });
                            strip.cell(|ui| {
                                if ui_button_w!(ui, "RPOP", 120.0) {
                                    self.state.command_last_result =
                                        run_redis_command(&self.state.current_connection, |conn| {
                                            ListPresenter::rpop(
                                                conn,
                                                &mut self.state.lists,
                                                &mut self.state.list_st,
                                            )
                                        });
                                }
                            });
                        });
                });
            });
    }

    fn left_modify_cmds(&mut self, ui: &mut egui::Ui) {
        StripBuilder::new(ui)
            .size(Size::exact(20.0))
            .size(Size::exact(20.0))
            .vertical(|mut strip| {
                strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::exact(50.0))
                        .size(Size::exact(70.0))
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Key",
                                    &mut self.state.list_st.lpush_k,
                                );
                            });
                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Element (& Elements)",
                                    &mut self.state.list_st.lpush_vs,
                                );
                            });
                            strip.cell(|ui| {
                                if ui_button_w50!(ui, "LPUSH") {
                                    self.state.command_last_result =
                                        run_redis_command(&self.state.current_connection, |conn| {
                                            ListPresenter::lpush(
                                                conn,
                                                &mut self.state.lists,
                                                &mut self.state.list_st,
                                            )
                                        });
                                }
                            });
                            strip.cell(|ui| {
                                if ui_button_w!(ui, "LPUSHX", 70.0) {
                                    self.state.command_last_result =
                                        run_redis_command(&self.state.current_connection, |conn| {
                                            ListPresenter::lpushx(
                                                conn,
                                                &mut self.state.lists,
                                                &mut self.state.list_st,
                                            )
                                        });
                                }
                            });
                        });
                });

                strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::exact(128.0))
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Key",
                                    &mut self.state.list_st.lpop_k,
                                );
                            });
                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "(Count)",
                                    &mut self.state.list_st.lpop_count,
                                );
                            });
                            strip.cell(|ui| {
                                if ui_button_w!(ui, "LPOP", 120.0) {
                                    self.state.command_last_result =
                                        run_redis_command(&self.state.current_connection, |conn| {
                                            ListPresenter::lpop(
                                                conn,
                                                &mut self.state.lists,
                                                &mut self.state.list_st,
                                            )
                                        });
                                }
                            });
                        });
                });
            });
    }

    fn lists_display(&mut self, ui: &mut egui::Ui, i18n: &I18n) {
        egui::Grid::new("redis_lists")
            .spacing(egui::vec2(ui.spacing().item_spacing.x * 2.0, 0.0))
            .show(ui, |ui| {
                for header in &self.state.lists {
                    ui.code(header.0.clone()).context_menu(|ui| {
                        if ui.button(&i18n.redis_delete_ds).clicked() {
                            match presenter::delete_key(
                                &self.state.current_connection.host,
                                &self.state.current_connection.port,
                                header.0,
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
                    ui.label(format!("{:?}", header.1));
                    ui.end_row();
                }
            });
    }
}
