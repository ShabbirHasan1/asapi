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
        presenter::{self, run_redis_command, RedisMenu, SetsPresenter},
        view::RedisView,
    },
    ui_button_w, ui_button_w100,
};

///
/// Básico
/// done - SADD
/// done - SREM
/// done - SPOP
/// done - SRANDMEMBER
///
/// Info Básica
/// done - SISMBEMBER
/// done - SCARD
/// done - SMEMBERS
///
/// Operaciones Conjuntos
/// SINTER
/// SINTERCARD
/// SINTERSTORE
/// SDIFF
/// SDIFFSTORE
/// SUNION
/// SUNIONSTORE
///
impl RedisView {
    pub fn show_sets(&mut self, ui: &mut egui::Ui, i18n: &I18n) {
        if self.state.selected_menu == RedisMenu::Set {
            egui::CollapsingHeader::new("Comandos Disponibles")
                .default_open(true)
                .show(ui, |ui| {
                    ui.columns(2, |uis| {
                        self.basic_cmds(&mut uis[0]);
                        self.set_info_cmds(&mut uis[1]);
                    });
                    ui.separator();

                    ui.columns(2, |uis| {
                        self.inter_cmds(&mut uis[0]);
                        // self.diff_and_union_cmds(&mut uis[1]);
                    });
                });

            if !self.state.command_last_result.is_empty() {
                ui.label(&self.state.command_last_result);
            }
        }

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

    fn basic_cmds(&mut self, ui: &mut egui::Ui) {
        StripBuilder::new(ui)
            .size(Size::exact(20.0))
            .size(Size::exact(20.0))
            .size(Size::exact(20.0))
            .size(Size::exact(20.0))
            .vertical(|mut strip| {
                strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::exact(108.0))
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Key",
                                    &mut self.state.sets_st.sadd_k,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Value (& Values)",
                                    &mut self.state.sets_st.sadd_vs,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "SADD") {
                                    self.state.command_last_result =
                                        run_redis_command(&self.state.current_connection, |conn| {
                                            SetsPresenter::sadd(
                                                conn,
                                                &mut self.state.sets,
                                                &mut self.state.sets_st,
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
                        .size(Size::exact(108.0))
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Key",
                                    &mut self.state.sets_st.srem_k,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Value (& Values)",
                                    &mut self.state.sets_st.srem_vs,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "SREM") {
                                    self.state.command_last_result =
                                        run_redis_command(&self.state.current_connection, |conn| {
                                            SetsPresenter::srem(
                                                conn,
                                                &mut self.state.sets,
                                                &mut self.state.sets_st,
                                            )
                                        });
                                }
                            });
                        });
                });

                strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::exact(108.0))
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Key",
                                    &mut self.state.sets_st.spop_k,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "SPOP") {
                                    self.state.command_last_result =
                                        run_redis_command(&self.state.current_connection, |conn| {
                                            SetsPresenter::spop(
                                                conn,
                                                &mut self.state.sets,
                                                &mut self.state.sets_st,
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
                        .size(Size::exact(108.0))
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Key",
                                    &mut self.state.sets_st.srandmember_k,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Count (1 if no value provided)",
                                    &mut self.state.sets_st.srandmember_count,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "SRANDMEMBER") {
                                    self.state.command_last_result =
                                        run_redis_command(&self.state.current_connection, |conn| {
                                            SetsPresenter::srandmember(
                                                conn,
                                                &mut self.state.sets_st,
                                            )
                                        });
                                }
                            });
                        });
                });
            });
    }

    fn set_info_cmds(&mut self, ui: &mut egui::Ui) {
        StripBuilder::new(ui)
            .size(Size::exact(20.0))
            .size(Size::exact(20.0))
            .size(Size::exact(20.0))
            .size(Size::exact(20.0))
            .vertical(|mut strip| {
                strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::exact(108.0))
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.state.sets_st.sismember_k)
                                        .hint_text("Key"),
                                );
                            });

                            strip.cell(|ui| {
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.state.sets_st.sismember_m)
                                        .hint_text("Member"),
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "SISMEMBER") {
                                    self.state.command_last_result =
                                        run_redis_command(&self.state.current_connection, |conn| {
                                            SetsPresenter::sismember(conn, &mut self.state.sets_st)
                                        });
                                }
                            });
                        });
                });

                strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::exact(108.0))
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui.add(
                                    egui::TextEdit::singleline(
                                        &mut self.state.sets_st.smismember_k,
                                    )
                                    .hint_text("Key"),
                                );
                            });

                            strip.cell(|ui| {
                                ui.add(
                                    egui::TextEdit::singleline(
                                        &mut self.state.sets_st.smismember_ms,
                                    )
                                    .hint_text("Member (& Members)"),
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "SMISMEMBER") {
                                    self.state.command_last_result =
                                        run_redis_command(&self.state.current_connection, |conn| {
                                            SetsPresenter::smismember(conn, &mut self.state.sets_st)
                                        });
                                }
                            });
                        });
                });

                strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::exact(108.0))
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.state.sets_st.scard_k)
                                        .hint_text("Key"),
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "SCARD") {
                                    self.state.command_last_result =
                                        run_redis_command(&self.state.current_connection, |conn| {
                                            SetsPresenter::scard(conn, &mut self.state.sets_st)
                                        });
                                }
                            });
                        });
                });

                strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::exact(108.0))
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.state.sets_st.smembers_k)
                                        .hint_text("Key"),
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "SMEMBERS") {
                                    self.state.command_last_result =
                                        run_redis_command(&self.state.current_connection, |conn| {
                                            SetsPresenter::smembers(
                                                conn,
                                                &mut self.state.sets,
                                                &mut self.state.sets_st,
                                            )
                                        });
                                }
                            });
                        });
                });
            });
    }

    fn inter_cmds(&mut self, ui: &mut egui::Ui) {
        StripBuilder::new(ui)
            .size(Size::exact(20.0))
            .size(Size::exact(20.0))
            .size(Size::exact(20.0))
            .vertical(|mut strip| {
                strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::exact(108.0))
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.state.sets_st.sinter_ks)
                                        .hint_text("Key (& Keys)"),
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "SINTER") {
                                    self.state.command_last_result =
                                        run_redis_command(&self.state.current_connection, |conn| {
                                            SetsPresenter::sinter(conn, &mut self.state.sets_st)
                                        });
                                }
                            });
                        });
                });

                strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::exact(108.0))
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui.add(
                                    egui::TextEdit::singleline(
                                        &mut self.state.sets_st.sintercard_numkeys,
                                    )
                                    .hint_text("Numkeys"),
                                );
                            });

                            strip.cell(|ui| {
                                ui.add(
                                    egui::TextEdit::singleline(
                                        &mut self.state.sets_st.sintercard_ks,
                                    )
                                    .hint_text("Key (& Keys)"),
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "SINTERCARD") {
                                    self.state.command_last_result =
                                        run_redis_command(&self.state.current_connection, |conn| {
                                            SetsPresenter::sintercard(conn, &mut self.state.sets_st)
                                        });
                                }
                            });
                        });
                });

                strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::exact(108.0))
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui.add(
                                    egui::TextEdit::singleline(
                                        &mut self.state.sets_st.sinterstore_destination,
                                    )
                                    .hint_text("Destination"),
                                );
                            });

                            strip.cell(|ui| {
                                ui.add(
                                    egui::TextEdit::singleline(
                                        &mut self.state.sets_st.sinterstore_ks,
                                    )
                                    .hint_text("Key (& Keys)"),
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "SINTERSTORE") {
                                    self.state.command_last_result =
                                        run_redis_command(&self.state.current_connection, |conn| {
                                            SetsPresenter::sinterstore(
                                                conn,
                                                &mut self.state.sets,
                                                &mut self.state.sets_st,
                                            )
                                        });
                                }
                            });
                        });
                });
            });
    }
}
