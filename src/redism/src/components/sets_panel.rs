use std::collections::HashMap;

// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------
use common::internationalization::I18n;
use components::ui_button_w100;
use components::{result_panel::ui_response_panel, widgets::ui_text_edit_singleline_hint};
use eframe::egui;
use egui_extras::{Size, StripBuilder};
use redis::Connection;

use crate::{
    connection::RedisMenu,
    presenters::{
        delete_key, run_read_generic, run_write_generic, set::SetsPresenter,
        zset::SortedSetsPresenter, RedisResponse,
    },
    state::{RedisSetsState, RedisZSetsState},
    view::RedisView,
};

impl RedisView {
    pub fn show_sets(&mut self, ui: &mut egui::Ui, i18n: &I18n) {
        if self.state.selected_menu == RedisMenu::Set {
            egui::CollapsingHeader::new(i18n.redis_commands_header.to_ascii_uppercase())
                .show_background(true)
                .default_open(true)
                .show(ui, |ui| {
                    ui.columns(2, |uis| {
                        self.basic_cmds(&mut uis[0]);
                        self.set_info_cmds(&mut uis[1]);
                    });
                    ui.separator();

                    ui.columns(2, |uis| {
                        self.inter_cmds(&mut uis[0]);
                        self.diff_and_union_cmds(&mut uis[1]);
                    });
                    ui.separator();
                });

            ui_response_panel(ui, &self.state.last_result);
        }

        self.display_sets(ui, i18n, RedisMenu::Set);
    }

    pub fn show_sorted_sets(&mut self, ui: &mut egui::Ui, i18n: &I18n) {
        if self.state.selected_menu == RedisMenu::SortedSet {
            egui::CollapsingHeader::new(i18n.redis_commands_header.to_ascii_uppercase())
                .show_background(true)
                .default_open(true)
                .show(ui, |ui| {
                    ui.columns(2, |uis| {
                        self.sset_basic_cmds(&mut uis[0]);
                        self.sset_set_info_cmds(&mut uis[1]);
                    });
                    ui.separator();

                    ui.columns(2, |uis| {
                        self.sset_inter_cmds(&mut uis[0]);
                        self.sset_inter_and_union_cmds(&mut uis[1]);
                    });
                    ui.separator();

                    ui.columns(2, |uis| {
                        self.sset_rank_cmds(&mut uis[0]);
                    });
                    ui.separator();
                });

            ui_response_panel(ui, &self.state.last_result);
        }

        self.display_sets(ui, i18n, RedisMenu::SortedSet);
    }

    fn display_sets(&mut self, ui: &mut egui::Ui, i18n: &I18n, menu: RedisMenu) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.set_width(ui.available_width());
            let tmp = if menu == RedisMenu::Set {
                &self.state.sets
            } else {
                &self.state.zsets
            };

            for (set_key, set_values) in tmp {
                ui.code(set_key).context_menu(|ui| {
                    if ui.button(&i18n.redis_delete_ds).clicked() {
                        match delete_key(
                            &self.state.current_connection.host,
                            &self.state.current_connection.port,
                            set_key,
                        ) {
                            Ok(s) => {
                                self.state.must_scan = true;
                                log::info!("{:?}", s);
                            }
                            Err(e) => log::error!("{:?}", e),
                        }
                        ui.close_menu();
                    }
                });
                ui.indent(set_key, |ui| ui.label(set_values.join(", ")));

                ui.end_row();
            }
        });
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
                                    self.state.last_result =
                                        self.run_write_set(SetsPresenter::sadd);
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
                                    self.state.last_result =
                                        self.run_write_set(SetsPresenter::srem);
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
                                    self.state.last_result =
                                        self.run_write_set(SetsPresenter::spop);
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
                                    self.state.last_result =
                                        self.run_read_set(SetsPresenter::srandmember);
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
                                    self.state.last_result =
                                        self.run_read_set(SetsPresenter::sismember);
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
                                    self.state.last_result =
                                        self.run_read_set(SetsPresenter::smismember);
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
                                    &mut self.state.sets_st.scard_k,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "SCARD") {
                                    self.state.last_result =
                                        self.run_read_set(SetsPresenter::scard);
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
                                    &mut self.state.sets_st.smembers_k,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "SMEMBERS") {
                                    self.state.last_result =
                                        self.run_write_set(SetsPresenter::smembers);
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
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Key (& Keys)",
                                    &mut self.state.sets_st.sinter_ks,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "SINTER") {
                                    self.state.last_result =
                                        self.run_read_set(SetsPresenter::sinter);
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
                                    "Numkeys",
                                    &mut self.state.sets_st.sintercard_numkeys,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Key (& Keys)",
                                    &mut self.state.sets_st.sintercard_ks,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "SINTERCARD") {
                                    self.state.last_result =
                                        self.run_read_set(SetsPresenter::sintercard);
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
                                    "Destination",
                                    &mut self.state.sets_st.sinterstore_destination,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Key (& Keys)",
                                    &mut self.state.sets_st.sinterstore_ks,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "SINTERSTORE") {
                                    self.state.last_result =
                                        self.run_write_set(SetsPresenter::sinterstore);
                                }
                            });
                        });
                });
            });
    }

    fn diff_and_union_cmds(&mut self, ui: &mut egui::Ui) {
        StripBuilder::new(ui)
            .size(Size::exact(20.0))
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
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Key (& Keys)",
                                    &mut self.state.sets_st.sdiff_ks,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "SDIFF") {
                                    self.state.last_result =
                                        self.run_read_set(SetsPresenter::sdiff);
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
                                    "Destination",
                                    &mut self.state.sets_st.sdiffstore_destination,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Key (& Keys)",
                                    &mut self.state.sets_st.sdiffstore_ks,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "SDIFFSTORE") {
                                    self.state.last_result =
                                        self.run_write_set(SetsPresenter::sdiffstore);
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
                                    "Key (& Keys)",
                                    &mut self.state.sets_st.sunion_ks,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "SUNION") {
                                    self.state.last_result =
                                        self.run_read_set(SetsPresenter::sunion);
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
                                    "Destination",
                                    &mut self.state.sets_st.sunionstore_destination,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Key (& Keys)",
                                    &mut self.state.sets_st.sunionstore_ks,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "SUNIONSTORE") {
                                    self.state.last_result =
                                        self.run_write_set(SetsPresenter::sunionstore);
                                }
                            });
                        });
                });
            });
    }

    // -------------------------------------------------------
    // -------------------------------------------------------
    // -------------------------------------------------------
    fn sset_basic_cmds(&mut self, ui: &mut egui::Ui) {
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
                        .size(Size::remainder())
                        .size(Size::exact(138.0))
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Key",
                                    &mut self.state.zsets_st.zadd_k,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Score",
                                    &mut self.state.zsets_st.zadd_score,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Member",
                                    &mut self.state.zsets_st.zadd_v,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "ZADD") {
                                    self.state.last_result =
                                        self.run_write_zset(SortedSetsPresenter::zadd);
                                }
                            });
                        });
                });

                strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::exact(138.0))
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Key",
                                    &mut self.state.zsets_st.zrem_k,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Value (& Values)",
                                    &mut self.state.zsets_st.zrem_vs,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "ZREM") {
                                    self.state.last_result =
                                        self.run_write_zset(SortedSetsPresenter::zrem);
                                }
                            });
                        });
                });

                strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::exact(138.0))
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Key (& Keys)",
                                    &mut self.state.zsets_st.zmpop_ks,
                                );
                            });

                            strip.cell(|ui| {
                                egui::ComboBox::from_id_salt("zpop_min_max")
                                    .selected_text(&self.state.zsets_st.zmpop_min_max)
                                    .width(ui.available_width())
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(
                                            &mut self.state.zsets_st.zmpop_min_max,
                                            "MIN".to_string(),
                                            "Min",
                                        );
                                        ui.selectable_value(
                                            &mut self.state.zsets_st.zmpop_min_max,
                                            "MAX".to_string(),
                                            "Max",
                                        );
                                    });
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "(Count)",
                                    &mut self.state.zsets_st.zmpop_count,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "ZMPOP") {
                                    self.state.last_result =
                                        self.run_write_zset(SortedSetsPresenter::zmpop);
                                }
                            });
                        });
                });

                strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::exact(138.0))
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
                                if ui_button_w100!(ui, "ZRANDMEMBER") {
                                    self.state.last_result =
                                        self.run_read_zset(SortedSetsPresenter::zrandmember);
                                }
                            });
                        });
                });
            });
    }

    fn sset_set_info_cmds(&mut self, ui: &mut egui::Ui) {
        StripBuilder::new(ui)
            .size(Size::exact(20.0))
            .size(Size::exact(20.0))
            .size(Size::exact(20.0))
            .size(Size::exact(20.0))
            .size(Size::exact(20.0))
            .vertical(|mut strip| {
                strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::exact(138.0))
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Key",
                                    &mut self.state.zsets_st.zcard_k,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "ZCARD") {
                                    self.state.last_result =
                                        self.run_read_zset(SortedSetsPresenter::zcard);
                                }
                            });
                        });
                });

                strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::exact(138.0))
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Key",
                                    &mut self.state.zsets_st.zrange_k,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Start",
                                    &mut self.state.zsets_st.zrange_start,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Stop",
                                    &mut self.state.zsets_st.zrange_stop,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "ZRANGE") {
                                    self.state.last_result =
                                        self.run_read_zset(SortedSetsPresenter::zrange);
                                }
                            });
                        });
                });

                strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::exact(138.0))
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Destination",
                                    &mut self.state.zsets_st.zrangestore_destination,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Key",
                                    &mut self.state.zsets_st.zrangestore_k,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Start",
                                    &mut self.state.zsets_st.zrangestore_start,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Stop",
                                    &mut self.state.zsets_st.zrangestore_stop,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "ZRANGESTORE") {
                                    self.state.last_result =
                                        self.run_write_zset(SortedSetsPresenter::zrangestore);
                                }
                            });
                        });
                });

                strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::exact(138.0))
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Key",
                                    &mut self.state.zsets_st.zrangebylex_k,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Min",
                                    &mut self.state.zsets_st.zrangebylex_min,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Max",
                                    &mut self.state.zsets_st.zrangebylex_max,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "ZRANGEBYLEX") {
                                    self.state.last_result =
                                        self.run_read_zset(SortedSetsPresenter::zrangebylex);
                                }
                            });
                        });
                });

                strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::exact(138.0))
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Key",
                                    &mut self.state.zsets_st.zrangebyscore_k,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Min",
                                    &mut self.state.zsets_st.zrangebyscore_min,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Max",
                                    &mut self.state.zsets_st.zrangebyscore_max,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "ZRANGEBYSCORE") {
                                    self.state.last_result =
                                        self.run_read_zset(SortedSetsPresenter::zrangebyscore);
                                }
                            });
                        });
                });
            });
    }

    fn sset_inter_cmds(&mut self, ui: &mut egui::Ui) {
        StripBuilder::new(ui)
            .size(Size::exact(20.0))
            .size(Size::exact(20.0))
            .size(Size::exact(20.0))
            .vertical(|mut strip| {
                strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::exact(138.0))
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Key (& Keys)",
                                    &mut self.state.zsets_st.zinter_ks,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "ZINTER") {
                                    self.state.last_result =
                                        self.run_read_zset(SortedSetsPresenter::zinter);
                                }
                            });
                        });
                });

                strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::exact(138.0))
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Key (& Keys)",
                                    &mut self.state.zsets_st.zintercard_ks,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "ZINTERCARD") {
                                    self.state.last_result =
                                        self.run_read_zset(SortedSetsPresenter::zintercard);
                                }
                            });
                        });
                });

                strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::exact(138.0))
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Destination",
                                    &mut self.state.zsets_st.zinterstore_destination,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Key (& Keys)",
                                    &mut self.state.zsets_st.zinterstore_ks,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "ZINTERSTORE") {
                                    self.state.last_result =
                                        self.run_write_zset(SortedSetsPresenter::zinterstore);
                                }
                            });
                        });
                });
            });
    }

    fn sset_inter_and_union_cmds(&mut self, ui: &mut egui::Ui) {
        StripBuilder::new(ui)
            .size(Size::exact(20.0))
            .size(Size::exact(20.0))
            .vertical(|mut strip| {
                strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::exact(138.0))
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Destination",
                                    &mut self.state.zsets_st.zunionstore_destination,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Key (& Keys)",
                                    &mut self.state.zsets_st.zunionstore_ks,
                                );
                            });

                            strip.cell(|ui| {
                                egui::ComboBox::from_id_salt("zunionstore_min_max")
                                    .selected_text(&self.state.zsets_st.zunionstore_min_max)
                                    .width(ui.available_width())
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(
                                            &mut self.state.zsets_st.zunionstore_min_max,
                                            "NONE".to_string(),
                                            " ",
                                        );
                                        ui.selectable_value(
                                            &mut self.state.zsets_st.zunionstore_min_max,
                                            "MIN".to_string(),
                                            "Min",
                                        );
                                        ui.selectable_value(
                                            &mut self.state.zsets_st.zunionstore_min_max,
                                            "MAX".to_string(),
                                            "Max",
                                        );
                                    });
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "ZUNIONSTORE") {
                                    self.state.last_result =
                                        self.run_write_zset(SortedSetsPresenter::zunionstore);
                                }
                            });
                        });
                });
            });
    }

    fn sset_rank_cmds(&mut self, ui: &mut egui::Ui) {
        StripBuilder::new(ui)
            .size(Size::exact(20.0))
            .size(Size::exact(20.0))
            .size(Size::exact(20.0))
            .vertical(|mut strip| {
                strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::exact(138.0))
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Key",
                                    &mut self.state.zsets_st.zrank_k,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Member",
                                    &mut self.state.zsets_st.zrank_m,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "ZRANK") {
                                    self.state.last_result =
                                        self.run_read_zset(SortedSetsPresenter::zrank);
                                }
                            });
                        });
                });

                strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::exact(138.0))
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Key",
                                    &mut self.state.zsets_st.zrevrank_k,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Member",
                                    &mut self.state.zsets_st.zrevrank_m,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "ZREVRANK") {
                                    self.state.last_result =
                                        self.run_read_zset(SortedSetsPresenter::zrevrank);
                                }
                            });
                        });
                });

                strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::exact(138.0))
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Key",
                                    &mut self.state.zsets_st.zremrangebyrank_k,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Start",
                                    &mut self.state.zsets_st.zremrangebyrank_start,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Stop",
                                    &mut self.state.zsets_st.zremrangebyrank_stop,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "ZREMRANGEBYRANK") {
                                    self.state.last_result =
                                        self.run_write_zset(SortedSetsPresenter::zremrangebyrank);
                                }
                            });
                        });
                });
            });
    }

    #[inline(always)]
    fn run_read_set(
        &mut self,
        cb: impl Fn(&mut Connection, &RedisSetsState) -> RedisResponse,
    ) -> Option<RedisResponse> {
        run_read_generic(&self.state.current_connection, &self.state.sets_st, cb)
    }

    #[inline(always)]
    fn run_read_zset(
        &mut self,
        cb: impl Fn(&mut Connection, &RedisZSetsState) -> RedisResponse,
    ) -> Option<RedisResponse> {
        run_read_generic(&self.state.current_connection, &self.state.zsets_st, cb)
    }

    #[inline(always)]
    fn run_write_set(
        &mut self,
        cb: impl Fn(
            &mut Connection,
            &mut HashMap<String, Vec<String>>,
            &RedisSetsState,
        ) -> RedisResponse,
    ) -> Option<RedisResponse> {
        run_write_generic(
            &self.state.current_connection,
            &self.state.sets_st,
            &mut self.state.sets,
            cb,
        )
    }

    #[inline(always)]
    fn run_write_zset(
        &mut self,
        cb: impl Fn(
            &mut Connection,
            &mut HashMap<String, Vec<String>>,
            &RedisZSetsState,
        ) -> RedisResponse,
    ) -> Option<RedisResponse> {
        run_write_generic(
            &self.state.current_connection,
            &self.state.zsets_st,
            &mut self.state.zsets,
            cb,
        )
    }
}
