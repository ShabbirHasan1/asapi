use std::collections::BTreeMap;

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
        delete_key, json::JsonPresenter, run_read_generic, run_write_generic, RedisResponse,
    },
    state::RedisJsonState,
    view::RedisView,
};

impl RedisView {
    pub fn show_json(&mut self, ui: &mut egui::Ui, i18n: &I18n) {
        if self.state.selected_menu == RedisMenu::Json {
            egui::CollapsingHeader::new(i18n.redis_commands_header.to_ascii_uppercase())
                .show_background(true)
                .default_open(true)
                .show(ui, |ui| {
                    ui.columns(2, |uis| {
                        // 1 - JSON.GET JSON.MGET JSON.OBJKEYS JSON.OBJLEN JSON.STRLEN
                        self.json_basic_info_cmds(&mut uis[0]);
                        // 2 - JSON.SET JSON.DEL  JSON.FORGET  JSON.CLEAR  JSON.STRAPPEND
                        self.json_basic_setter_cmds(&mut uis[1]);
                    });
                    ui.separator();

                    ui.columns(2, |uis| {
                        // 1 - JSON.{ARRINDEX | ARRLEN | ARRINSERT | ARRAPPEND | ARRPOP | ARRTRIM}
                        self.json_array_methods_cmds(&mut uis[0]);
                        // 2 - JSON.NUMINCRBY JSON.NUMMULTBY JSON.TYPE JSON.MERGE JSON.TOGGLE
                        self.json_other_cmds(&mut uis[1]);
                    });
                    ui.separator();
                });

            ui_response_panel(ui, &self.state.last_result);
        }
        self.display_json(ui, i18n)
    }

    fn json_other_cmds(&mut self, ui: &mut egui::Ui) {
        StripBuilder::new(ui)
            .size(Size::exact(20.0))
            .size(Size::exact(20.0))
            .size(Size::exact(20.0))
            .size(Size::exact(20.0))
            .size(Size::exact(20.0))
            .size(Size::exact(20.0))
            .vertical(|mut outter_strip| {
                outter_strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::exact(108.0))
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Key",
                                    &mut self.state.json_st.json_type_k,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Path",
                                    &mut self.state.json_st.json_type_p,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "JSON.TYPE") {
                                    self.state.last_result =
                                        self.run_read_json(JsonPresenter::json_type);
                                }
                            });
                        });
                });

                outter_strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::exact(108.0))
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Key",
                                    &mut self.state.json_st.json_numincrby_k,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Path",
                                    &mut self.state.json_st.json_numincrby_p,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Value",
                                    &mut self.state.json_st.json_numincrby_v,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "J.NUMINCRBY") {
                                    self.state.last_result =
                                        self.run_write_json(JsonPresenter::json_numincrby);
                                }
                            });
                        });
                });

                outter_strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::exact(108.0))
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Key",
                                    &mut self.state.json_st.json_nummultby_k,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Path",
                                    &mut self.state.json_st.json_nummultby_p,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Value",
                                    &mut self.state.json_st.json_nummultby_v,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "J.NUMMULTBY") {
                                    self.state.last_result =
                                        self.run_write_json(JsonPresenter::json_nummultby);
                                }
                            });
                        });
                });

                outter_strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::exact(108.0))
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Key",
                                    &mut self.state.json_st.json_merge_k,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Path",
                                    &mut self.state.json_st.json_merge_p,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Value",
                                    &mut self.state.json_st.json_merge_v,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "JSON.MERGE") {
                                    self.state.last_result =
                                        self.run_write_json(JsonPresenter::json_merge);
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
                                    "Key",
                                    &mut self.state.json_st.json_toggle_k,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Path",
                                    &mut self.state.json_st.json_toggle_p,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "JSON.TOGGLE") {
                                    self.state.last_result =
                                        self.run_write_json(JsonPresenter::json_toggle);
                                }
                            });
                        });
                });
            });
    }

    fn json_array_methods_cmds(&mut self, ui: &mut egui::Ui) {
        StripBuilder::new(ui)
            .size(Size::exact(20.0))
            .size(Size::exact(20.0))
            .size(Size::exact(20.0))
            .size(Size::exact(20.0))
            .size(Size::exact(20.0))
            .size(Size::exact(20.0))
            .vertical(|mut outter_strip| {
                outter_strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::exact(108.0))
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Key",
                                    &mut self.state.json_st.json_arrindex_k,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Path",
                                    &mut self.state.json_st.json_arrindex_p,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Value",
                                    &mut self.state.json_st.json_arrindex_v,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "(Start)",
                                    &mut self.state.json_st.json_arrindex_start,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "(Stop)",
                                    &mut self.state.json_st.json_arrindex_stop,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "JSON.ARRINDEX") {
                                    self.state.last_result =
                                        self.run_read_json(JsonPresenter::json_arrindex);
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
                                    "Key",
                                    &mut self.state.json_st.json_arrindex_k,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Path",
                                    &mut self.state.json_st.json_arrindex_p,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "JSON.ARRLEN") {
                                    self.state.last_result =
                                        self.run_read_json(JsonPresenter::json_arrlen);
                                }
                            });
                        });
                });

                outter_strip.strip(|builder| {
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
                                    "Key",
                                    &mut self.state.json_st.json_arrinsert_k,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Path",
                                    &mut self.state.json_st.json_arrinsert_p,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Index",
                                    &mut self.state.json_st.json_arrinsert_idx,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Value (& Values)",
                                    &mut self.state.json_st.json_arrinsert_vs,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "J.ARRINSERT") {
                                    self.state.last_result =
                                        self.run_write_json(JsonPresenter::json_arrinsert);
                                }
                            });
                        });
                });

                outter_strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::exact(108.0))
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Key",
                                    &mut self.state.json_st.json_arrappend_k,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Path",
                                    &mut self.state.json_st.json_arrappend_p,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Value (& Values)",
                                    &mut self.state.json_st.json_arrappend_vs,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "J.ARRAPPEND") {
                                    self.state.last_result =
                                        self.run_write_json(JsonPresenter::json_arrappend);
                                }
                            });
                        });
                });

                outter_strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::exact(108.0))
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Key",
                                    &mut self.state.json_st.json_arrpop_k,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "(Path)",
                                    &mut self.state.json_st.json_arrpop_p,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "(Index)",
                                    &mut self.state.json_st.json_arrpop_idx,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "JSON.ARRPOP") {
                                    self.state.last_result =
                                        self.run_write_json(JsonPresenter::json_arrpop);
                                }
                            });
                        });
                });

                outter_strip.strip(|builder| {
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
                                    "Key",
                                    &mut self.state.json_st.json_arrtrim_k,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Path",
                                    &mut self.state.json_st.json_arrtrim_p,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Start",
                                    &mut self.state.json_st.json_arrtrim_start,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Stop",
                                    &mut self.state.json_st.json_arrtrim_stop,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "JSON.ARRTRIM") {
                                    self.state.last_result =
                                        self.run_write_json(JsonPresenter::json_arrtrim);
                                }
                            });
                        });
                });
            });
    }

    fn json_basic_setter_cmds(&mut self, ui: &mut egui::Ui) {
        StripBuilder::new(ui)
            .size(Size::exact(20.0))
            .size(Size::exact(20.0))
            .size(Size::exact(20.0))
            .size(Size::exact(20.0))
            .size(Size::exact(20.0))
            .vertical(|mut outter_strip| {
                outter_strip.strip(|builder| {
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
                                    "Key",
                                    &mut self.state.json_st.json_set_k,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Path",
                                    &mut self.state.json_st.json_set_p,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Value",
                                    &mut self.state.json_st.json_set_v,
                                );
                            });

                            strip.cell(|ui| {
                                egui::ComboBox::from_id_salt("json.set_NX_XX")
                                    .selected_text(&self.state.json_st.json_set_nx_xx)
                                    .width(ui.available_width())
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(
                                            &mut self.state.json_st.json_set_nx_xx,
                                            "Indistinct".to_string(),
                                            "Indistinct",
                                        );
                                        ui.selectable_value(
                                            &mut self.state.json_st.json_set_nx_xx,
                                            "NX".to_string(),
                                            "NX",
                                        );
                                        ui.selectable_value(
                                            &mut self.state.json_st.json_set_nx_xx,
                                            "XX".to_string(),
                                            "XX",
                                        );
                                    });
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "JSON.SET") {
                                    self.state.last_result =
                                        self.run_write_json(JsonPresenter::json_set);
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
                                    "Key",
                                    &mut self.state.json_st.json_del_k,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "(Path)",
                                    &mut self.state.json_st.json_del_p,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "JSON.DEL") {
                                    self.state.last_result =
                                        self.run_write_json(JsonPresenter::json_del);
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
                                    "Key",
                                    &mut self.state.json_st.json_forget_k,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "(Path)",
                                    &mut self.state.json_st.json_forget_p,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "JSON.FORGET") {
                                    self.state.last_result =
                                        self.run_write_json(JsonPresenter::json_forget);
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
                                    "Key",
                                    &mut self.state.json_st.json_clear_k,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "(Path)",
                                    &mut self.state.json_st.json_clear_p,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "JSON.CLEAR") {
                                    self.state.last_result =
                                        self.run_write_json(JsonPresenter::json_clear);
                                }
                            });
                        });
                });

                outter_strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::exact(108.0))
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Key",
                                    &mut self.state.json_st.json_strappend_k,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "(Path)",
                                    &mut self.state.json_st.json_strappend_p,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Value",
                                    &mut self.state.json_st.json_strappend_k,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "J.STRAPPEND") {
                                    self.state.last_result =
                                        self.run_write_json(JsonPresenter::json_strappend);
                                }
                            });
                        });
                });
            });
    }

    fn json_basic_info_cmds(&mut self, ui: &mut egui::Ui) {
        StripBuilder::new(ui)
            .size(Size::exact(20.0))
            .size(Size::exact(20.0))
            .size(Size::exact(20.0))
            .size(Size::exact(20.0))
            .size(Size::exact(20.0))
            .vertical(|mut outter_strip| {
                outter_strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::exact(108.0))
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Key",
                                    &mut self.state.json_st.json_get_k,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Path",
                                    &mut self.state.json_st.json_get_p,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "JSON.GET") {
                                    self.state.last_result =
                                        self.run_read_json(JsonPresenter::json_get);
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
                                    "Key (& Keys)",
                                    &mut self.state.json_st.json_mget_ks,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Path",
                                    &mut self.state.json_st.json_mget_p,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "JSON.MGET") {
                                    self.state.last_result =
                                        self.run_read_json(JsonPresenter::json_mget);
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
                                    "Key",
                                    &mut self.state.json_st.json_objkeys_k,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Path",
                                    &mut self.state.json_st.json_objkeys_p,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "JSON.OBJKEYS") {
                                    self.state.last_result =
                                        self.run_read_json(JsonPresenter::json_objkeys);
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
                                    "Key",
                                    &mut self.state.json_st.json_objlen_k,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "(Path)",
                                    &mut self.state.json_st.json_objlen_p,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "JSON.OBJLEN") {
                                    self.state.last_result =
                                        self.run_read_json(JsonPresenter::json_objlen);
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
                                    "Key",
                                    &mut self.state.json_st.json_strlen_k,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "(Path)",
                                    &mut self.state.json_st.json_strlen_p,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "JSON.STRLEN") {
                                    self.state.last_result =
                                        self.run_read_json(JsonPresenter::json_strlen);
                                }
                            });
                        });
                });
            });
    }

    fn display_json(&mut self, ui: &mut egui::Ui, i18n: &I18n) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("json objects")
                .spacing(egui::vec2(ui.spacing().item_spacing.x * 2.0, 0.0))
                .show(ui, |ui| {
                    for header in &self.state.jsons {
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
                        ui.label(header.1.clone());
                        ui.end_row();
                    }
                });
        });
    }

    #[inline(always)]
    fn run_read_json(
        &mut self,
        cb: impl Fn(&mut Connection, &RedisJsonState) -> RedisResponse,
    ) -> Option<RedisResponse> {
        run_read_generic(&self.state.current_connection, &self.state.json_st, cb)
    }

    #[inline(always)]
    fn run_write_json(
        &mut self,
        cb: impl Fn(&mut Connection, &mut BTreeMap<String, String>, &RedisJsonState) -> RedisResponse,
    ) -> Option<RedisResponse> {
        run_write_generic(
            &self.state.current_connection,
            &self.state.json_st,
            &mut self.state.jsons,
            cb,
        )
    }
}
