use std::collections::HashMap;

// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------
use common::I18nRedis as I18n;
use components::{result_panel::ui_response_panel, widgets::ui_text_edit_singleline_hint};
use components::{ui_button_w, ui_button_w50};
use eframe::egui;
use egui_extras::{Size, StripBuilder};
use redis::Connection;

use crate::{
    connection::RedisMenu,
    presenters::{
        delete_key,
        list::{ListPresenter, RedisPosition},
        run_cmd, run_read_generic, run_write_generic, RedisResponse,
    },
    state::RedisListState,
    view::RedisView,
};

impl RedisView {
    pub fn show_lists(&mut self, ui: &mut egui::Ui, i18n: &I18n) {
        if self.state.selected_menu == RedisMenu::List {
            egui::CollapsingHeader::new(i18n.redis_commands_header.to_ascii_uppercase())
                .show_background(true)
                .default_open(true)
                .show(ui, |ui| {
                    ui.columns(2, |uis| {
                        self.left_modify_cmds(&mut uis[0]);
                        self.right_modify_cmds(&mut uis[1]);
                    });
                    ui.separator();

                    ui.columns(2, |uis| {
                        self.info_cmds(&mut uis[0]);
                        self.modifier_cmds(&mut uis[1]);
                    });
                    ui.separator();
                });

            ui_response_panel(ui, &self.state.last_result);
        }

        self.lists_display(ui, i18n);
    }

    fn modifier_cmds(&mut self, ui: &mut egui::Ui) {
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
                        .size(Size::exact(128.0))
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Key",
                                    &mut self.state.list_st.ltrim_k,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Start",
                                    &mut self.state.list_st.ltrim_start,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Stop",
                                    &mut self.state.list_st.ltrim_stop,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w!(ui, "LTRIM", 128.0) {
                                    self.state.last_result =
                                        self.run_write_list(ListPresenter::ltrim);
                                }
                            });
                        });
                });

                strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::exact(60.0))
                        .size(Size::exact(60.0))
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Key to Insert",
                                    &mut self.state.list_st.linsert_k,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Pivot Before/After",
                                    &mut self.state.list_st.linsert_pivot,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Value",
                                    &mut self.state.list_st.linsert_value,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w!(ui, "BEFORE", 60.0) {
                                    self.state.last_result =
                                        run_cmd(&self.state.current_connection, |conn| {
                                            ListPresenter::linsert(
                                                conn,
                                                &mut self.state.lists,
                                                &self.state.list_st,
                                                RedisPosition::Before,
                                            )
                                        });
                                }
                            });

                            strip.cell(|ui| {
                                if ui_button_w!(ui, "AFTER", 60.0) {
                                    self.state.last_result =
                                        run_cmd(&self.state.current_connection, |conn| {
                                            ListPresenter::linsert(
                                                conn,
                                                &mut self.state.lists,
                                                &self.state.list_st,
                                                RedisPosition::End,
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
                        .size(Size::remainder())
                        .size(Size::exact(128.0))
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Key",
                                    &mut self.state.list_st.lrem_k,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Count",
                                    &mut self.state.list_st.lrem_count,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Value",
                                    &mut self.state.list_st.lrem_value,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w!(ui, "LREM", 128.0) {
                                    self.state.last_result =
                                        self.run_write_list(ListPresenter::lrem);
                                }
                            });
                        });
                });

                strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::exact(128.0))
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Key",
                                    &mut self.state.list_st.lset_k,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Index",
                                    &mut self.state.list_st.lset_index,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Value",
                                    &mut self.state.list_st.lset_value,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w!(ui, "LSET", 128.0) {
                                    self.state.last_result =
                                        self.run_write_list(ListPresenter::lset);
                                }
                            });
                        });
                });
            });
    }

    fn info_cmds(&mut self, ui: &mut egui::Ui) {
        StripBuilder::new(ui)
            .size(Size::exact(20.0))
            .size(Size::exact(20.0))
            .size(Size::exact(20.0))
            .vertical(|mut strip| {
                strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::exact(128.0))
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Key",
                                    &mut self.state.list_st.llen_k,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w!(ui, "LLEN", 128.0) {
                                    self.state.last_result =
                                        self.run_read_list(ListPresenter::llen);
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
                                    &mut self.state.list_st.lindex_k,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Index",
                                    &mut self.state.list_st.lindex_idx,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w!(ui, "LINDEX", 128.0) {
                                    self.state.last_result =
                                        self.run_read_list(ListPresenter::lindex);
                                }
                            });
                        });
                });

                strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::exact(128.0))
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Key",
                                    &mut self.state.list_st.lrange_k,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Start",
                                    &mut self.state.list_st.lrange_start,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Stop",
                                    &mut self.state.list_st.lrange_stop,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w!(ui, "LRANGE", 128.0) {
                                    self.state.last_result =
                                        self.run_read_list(ListPresenter::lrange);
                                }
                            });
                        });
                });
            });
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
                                    self.state.last_result =
                                        run_cmd(&self.state.current_connection, |conn| {
                                            ListPresenter::rpush(
                                                conn,
                                                &mut self.state.lists,
                                                &self.state.list_st,
                                            )
                                        });
                                }
                            });
                            strip.cell(|ui| {
                                if ui_button_w!(ui, "RPUSHX", 70.0) {
                                    self.state.last_result =
                                        self.run_write_list(ListPresenter::rpushx);
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
                                    self.state.last_result =
                                        self.run_write_list(ListPresenter::rpop);
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
                                    self.state.last_result =
                                        self.run_write_list(ListPresenter::lpush);
                                }
                            });
                            strip.cell(|ui| {
                                if ui_button_w!(ui, "LPUSHX", 70.0) {
                                    self.state.last_result =
                                        self.run_write_list(ListPresenter::lpushx);
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
                                    self.state.last_result =
                                        self.run_write_list(ListPresenter::lpop);
                                }
                            });
                        });
                });
            });
    }

    fn lists_display(&mut self, ui: &mut egui::Ui, i18n: &I18n) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("redis_lists")
                .spacing(egui::vec2(ui.spacing().item_spacing.x * 2.0, 0.0))
                .show(ui, |ui| {
                    for header in &self.state.lists {
                        ui.code(header.0.clone()).context_menu(|ui| {
                            if ui.button(&i18n.redis_delete_ds).clicked() {
                                match delete_key(
                                    &self.state.current_connection.host,
                                    &self.state.current_connection.port,
                                    header.0,
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
                        ui.label(format!("{:?}", header.1));
                        ui.end_row();
                    }
                });
        });
    }

    #[inline(always)]
    fn run_read_list(
        &mut self,
        cb: impl Fn(&mut Connection, &RedisListState) -> RedisResponse,
    ) -> Option<RedisResponse> {
        run_read_generic(&self.state.current_connection, &self.state.list_st, cb)
    }

    #[inline(always)]
    fn run_write_list(
        &mut self,
        cb: impl Fn(
            &mut Connection,
            &mut HashMap<String, Vec<String>>,
            &RedisListState,
        ) -> RedisResponse,
    ) -> Option<RedisResponse> {
        run_write_generic(
            &self.state.current_connection,
            &self.state.list_st,
            &mut self.state.lists,
            cb,
        )
    }
}
