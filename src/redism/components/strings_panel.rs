use std::collections::BTreeMap;

// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------
use eframe::egui;
use egui_extras::{Size, StripBuilder};
use redis::{Connection, RedisResult};

use crate::{
    common::internationalization::I18n,
    components::{result_panel::ui_response_panel, widgets::ui_text_edit_singleline_hint},
    error,
    redism::{
        presenter::{self, RedisMenu},
        presenters::{run_cmd, string::StringPresenter, RedisResponse},
        state::RedisStringState,
        view::RedisView,
    },
    ui_button_w, ui_button_w100, ui_button_w50,
};

/// Comandos a 240419
// done - SET
// done - SETNX
// done - SETRANGE
// done - APPEND
// done - GET
// done - GETSET
// done - GETDEL
// done - GETRANGE
// done - GETEX
// done - INCR
// done - INCRBY
// done - INCRBYFLOAT
// done - DECR
// done - DECRBY
// done - LCS
// done - STRLEN
impl RedisView {
    pub fn show_strings(&mut self, ui: &mut egui::Ui, i18n: &I18n) -> RedisResult<()> {
        if self.state.selected_menu == RedisMenu::String {
            egui::CollapsingHeader::new("Comandos Disponibles")
                .default_open(true)
                .show(ui, |ui| {
                    ui.columns(2, |uis| {
                        self.set_commands_display(&mut uis[0]);
                        self.get_commands_display(&mut uis[1]);
                    });
                    ui.separator();

                    ui.columns(2, |uis| {
                        self.incr_commands(&mut uis[0]);
                        self.decr_commands(&mut uis[1]);
                    });
                    ui.separator();

                    ui.columns(2, |uis| {
                        self.info_commands(&mut uis[1]);
                    });
                    ui.separator();
                });

            ui_response_panel(ui, &self.state.last_result);
        }

        self.strings_display(ui, i18n);

        Ok(())
    }

    fn info_commands(&mut self, ui: &mut egui::Ui) {
        StripBuilder::new(ui)
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
                                    "Key",
                                    &mut self.state.string_st.strlen_k,
                                );
                            });
                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "STRLEN") {
                                    self.state.last_result =
                                        run_cmd(&self.state.current_connection, |conn| {
                                            StringPresenter::str_len(conn, &self.state.string_st)
                                        })
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
                        .size(Size::exact(108.0))
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Key1",
                                    &mut self.state.string_st.lcs_k1,
                                );
                            });
                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Key2",
                                    &mut self.state.string_st.lcs_k2,
                                );
                            });
                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "[Optional] Len",
                                    &mut self.state.string_st.lcs_len,
                                );
                            });
                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "[Optional] Idx",
                                    &mut self.state.string_st.lcs_idx,
                                );
                            });
                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "LCS") {
                                    self.state.last_result = self.run_read(StringPresenter::lcs);
                                }
                            });
                        });
                });
            });
    }

    fn incr_commands(&mut self, ui: &mut egui::Ui) {
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
                                    "Key",
                                    &mut self.state.string_st.incr_k,
                                );
                            });
                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "INCR") {
                                    self.state.last_result = self.run_write(StringPresenter::incr);
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
                                    "Incr By Value",
                                    &mut self.state.string_st.incr_by_v,
                                );
                            });
                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "INCRBY") {
                                    self.state.last_result =
                                        self.run_write(StringPresenter::incr_by);
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
                                    "Incr By Value (Float)",
                                    &mut self.state.string_st.incr_byfloat_v,
                                );
                            });
                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "INCRBYFLOAT") {
                                    self.state.last_result =
                                        self.run_write(StringPresenter::incr_byfloat);
                                }
                            });
                        });
                });
            });
    }

    fn decr_commands(&mut self, ui: &mut egui::Ui) {
        StripBuilder::new(ui)
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
                                    "Key",
                                    &mut self.state.string_st.decr_k,
                                );
                            });
                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "DECR") {
                                    self.state.last_result = self.run_write(StringPresenter::decr);
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
                                    "Decr By Value",
                                    &mut self.state.string_st.decr_by_v,
                                );
                            });
                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "DECRBY") {
                                    self.state.last_result =
                                        self.run_write(StringPresenter::decr_by);
                                }
                            });
                        });
                });
            });
    }

    fn set_commands_display(&mut self, ui: &mut egui::Ui) {
        StripBuilder::new(ui)
            .size(Size::exact(20.0))
            .size(Size::exact(20.0))
            .size(Size::exact(20.0))
            .vertical(|mut strip| {
                strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .sizes(Size::exact(50.0), 2)
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Key",
                                    &mut self.state.string_st.set_k,
                                );
                            });
                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Value",
                                    &mut self.state.string_st.set_v,
                                );
                            });
                            strip.cell(|ui| {
                                if ui_button_w50!(ui, "SET") {
                                    self.state.last_result = self.run_write(StringPresenter::set);
                                }
                            });
                            strip.cell(|ui| {
                                if ui_button_w50!(ui, "SETNX") {
                                    self.state.last_result =
                                        self.run_write(StringPresenter::set_nx);
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
                                    "Offset",
                                    &mut self.state.string_st.set_offset,
                                );
                            });
                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "SETRANGE") {
                                    self.state.last_result =
                                        self.run_write(StringPresenter::set_range);
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
                                    "String to Append",
                                    &mut self.state.string_st.append_str,
                                );
                            });
                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "APPEND") {
                                    self.state.last_result =
                                        self.run_write(StringPresenter::append);
                                }
                            });
                        });
                });
            });
    }

    fn get_commands_display(&mut self, ui: &mut egui::Ui) {
        StripBuilder::new(ui)
            .size(Size::exact(20.0))
            .size(Size::exact(20.0))
            .size(Size::exact(20.0))
            .size(Size::exact(20.0))
            .vertical(|mut outter_strip| {
                outter_strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::exact(30.0))
                        .size(Size::exact(70.0))
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Key",
                                    &mut self.state.string_st.get_k,
                                );
                            });
                            strip.cell(|ui| {
                                if ui_button_w!(ui, "GET", 26.0) {
                                    self.state.last_result = self.run_read(StringPresenter::get);
                                }
                            });
                            strip.cell(|ui| {
                                if ui_button_w!(ui, "GETDEL", 64.0) {
                                    self.state.last_result =
                                        self.run_write(StringPresenter::get_del);
                                }
                            });
                        });
                });

                outter_strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::exact(108.0))
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Value",
                                    &mut self.state.string_st.getset_v,
                                );
                            });
                            strip.cell(|ui| {
                                if ui_button_w50!(ui, "GETSET") {
                                    self.state.last_result =
                                        self.run_write(StringPresenter::get_set);
                                }
                            });
                        });
                });

                outter_strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::exact(108.0))
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "From",
                                    &mut self.state.string_st.get_offset_from,
                                );
                            });
                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "To",
                                    &mut self.state.string_st.get_offset_to,
                                );
                            });
                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "GETRANGE") {
                                    self.state.last_result =
                                        self.run_read(StringPresenter::get_range);
                                }
                            });
                        });
                });

                outter_strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::exact(108.0))
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Expiration (seconds)",
                                    &mut self.state.string_st.get_expire_seconds,
                                );
                            });
                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "GETEX") {
                                    self.state.last_result = self.run_read(StringPresenter::get_ex);
                                }
                            });
                        });
                });
            });
    }

    fn strings_display(&mut self, ui: &mut egui::Ui, i18n: &I18n) {
        egui::Grid::new("key/value")
            .spacing(egui::vec2(ui.spacing().item_spacing.x * 2.0, 0.0))
            .show(ui, |ui| {
                for header in &self.state.strings {
                    ui.code(header.0.clone()).context_menu(|ui| {
                        if ui.button(&i18n.redis_delete_ds).clicked() {
                            match presenter::delete_key(
                                &self.state.current_connection.host,
                                &self.state.current_connection.port,
                                header.0,
                            ) {
                                Ok(_) => {
                                    self.state.must_scan = true;
                                }
                                Err(e) => error!("{:?}", e),
                            }
                            ui.close_menu();
                        }
                    });
                    ui.label(header.1.clone());
                    ui.end_row();
                }
            });
    }

    #[inline(always)]
    fn run_read(
        &mut self,
        mut cb: impl FnMut(&mut Connection, &RedisStringState) -> RedisResponse,
    ) -> Option<RedisResponse> {
        run_cmd(&self.state.current_connection, |conn| {
            cb(conn, &mut self.state.string_st)
        })
    }

    #[inline(always)]
    fn run_write(
        &mut self,
        mut cb: impl FnMut(
            &mut Connection,
            &mut BTreeMap<String, String>,
            &RedisStringState,
        ) -> RedisResponse,
    ) -> Option<RedisResponse> {
        run_cmd(&self.state.current_connection, |conn| {
            cb(conn, &mut self.state.strings, &mut self.state.string_st)
        })
    }
}
