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
    components::{result_panel::ui_response_panel, widgets::ui_text_edit_singleline_hint},
    error, info,
    redism::{
        presenter::{self, RedisMenu},
        presenters::{json::JsonPresenter, run_cmd},
        view::RedisView,
    },
    ui_button_w100,
};

impl RedisView {
    pub fn show_json(&mut self, ui: &mut egui::Ui, i18n: &I18n) {
        if self.state.selected_menu == RedisMenu::Json {
            egui::CollapsingHeader::new("Comandos Disponibles")
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

            if !self.state.last_result.is_empty() {
                ui.label(&self.state.last_result);
            }

            ui_response_panel(ui, &self.state.opt_last_result);
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
                                    self.state.opt_last_result =
                                        run_cmd(&self.state.current_connection, |conn| {
                                            JsonPresenter::json_type(conn, &mut self.state.json_st)
                                        });
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
                                    self.state.opt_last_result =
                                        run_cmd(&self.state.current_connection, |conn| {
                                            JsonPresenter::json_numincrby(
                                                conn,
                                                &mut self.state.jsons,
                                                &mut self.state.json_st,
                                            )
                                        });
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
                                    self.state.opt_last_result =
                                        run_cmd(&self.state.current_connection, |conn| {
                                            JsonPresenter::json_nummultby(
                                                conn,
                                                &mut self.state.jsons,
                                                &mut self.state.json_st,
                                            )
                                        });
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
                                    self.state.opt_last_result =
                                        run_cmd(&self.state.current_connection, |conn| {
                                            JsonPresenter::json_merge(
                                                conn,
                                                &mut self.state.jsons,
                                                &mut self.state.json_st,
                                            )
                                        });
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
                                    self.state.opt_last_result =
                                        run_cmd(&self.state.current_connection, |conn| {
                                            JsonPresenter::json_toggle(
                                                conn,
                                                &mut self.state.jsons,
                                                &mut self.state.json_st,
                                            )
                                        });
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
                                    self.state.opt_last_result =
                                        run_cmd(&self.state.current_connection, |conn| {
                                            JsonPresenter::json_arrindex(
                                                conn,
                                                &mut self.state.json_st,
                                            )
                                        });
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
                                    self.state.opt_last_result =
                                        run_cmd(&self.state.current_connection, |conn| {
                                            JsonPresenter::json_arrlen(
                                                conn,
                                                &mut self.state.json_st,
                                            )
                                        });
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
                                    self.state.opt_last_result =
                                        run_cmd(&self.state.current_connection, |conn| {
                                            JsonPresenter::json_arrinsert(
                                                conn,
                                                &mut self.state.jsons,
                                                &mut self.state.json_st,
                                            )
                                        });
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
                                    self.state.opt_last_result =
                                        run_cmd(&self.state.current_connection, |conn| {
                                            JsonPresenter::json_arrappend(
                                                conn,
                                                &mut self.state.jsons,
                                                &mut self.state.json_st,
                                            )
                                        });
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
                                    self.state.opt_last_result =
                                        run_cmd(&self.state.current_connection, |conn| {
                                            JsonPresenter::json_arrpop(
                                                conn,
                                                &mut self.state.jsons,
                                                &mut self.state.json_st,
                                            )
                                        });
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
                                    self.state.opt_last_result =
                                        run_cmd(&self.state.current_connection, |conn| {
                                            JsonPresenter::json_arrtrim(
                                                conn,
                                                &mut self.state.jsons,
                                                &mut self.state.json_st,
                                            )
                                        });
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
                                egui::ComboBox::from_id_source("json.set_NX_XX")
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
                                    self.state.opt_last_result =
                                        run_cmd(&self.state.current_connection, |conn| {
                                            JsonPresenter::json_set(
                                                conn,
                                                &mut self.state.jsons,
                                                &mut self.state.json_st,
                                            )
                                        });
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
                                    self.state.opt_last_result =
                                        run_cmd(&self.state.current_connection, |conn| {
                                            JsonPresenter::json_del(
                                                conn,
                                                &mut self.state.jsons,
                                                &mut self.state.json_st,
                                            )
                                        });
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
                                    self.state.opt_last_result =
                                        run_cmd(&self.state.current_connection, |conn| {
                                            JsonPresenter::json_forget(
                                                conn,
                                                &mut self.state.jsons,
                                                &mut self.state.json_st,
                                            )
                                        });
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
                                    self.state.opt_last_result =
                                        run_cmd(&self.state.current_connection, |conn| {
                                            JsonPresenter::json_clear(
                                                conn,
                                                &mut self.state.jsons,
                                                &mut self.state.json_st,
                                            )
                                        });
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
                                    self.state.opt_last_result =
                                        run_cmd(&self.state.current_connection, |conn| {
                                            JsonPresenter::json_strappend(
                                                conn,
                                                &mut self.state.jsons,
                                                &mut self.state.json_st,
                                            )
                                        });
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
                                    self.state.opt_last_result =
                                        run_cmd(&self.state.current_connection, |conn| {
                                            JsonPresenter::json_get(conn, &mut self.state.json_st)
                                        });
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
                                    self.state.opt_last_result =
                                        run_cmd(&self.state.current_connection, |conn| {
                                            JsonPresenter::json_mget(conn, &mut self.state.json_st)
                                        });
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
                                    self.state.opt_last_result =
                                        run_cmd(&self.state.current_connection, |conn| {
                                            JsonPresenter::json_objkeys(
                                                conn,
                                                &mut self.state.json_st,
                                            )
                                        });
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
                                    self.state.opt_last_result =
                                        run_cmd(&self.state.current_connection, |conn| {
                                            JsonPresenter::json_objlen(
                                                conn,
                                                &mut self.state.json_st,
                                            )
                                        });
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
                                    self.state.opt_last_result =
                                        run_cmd(&self.state.current_connection, |conn| {
                                            JsonPresenter::json_strlen(
                                                conn,
                                                &mut self.state.json_st,
                                            )
                                        });
                                }
                            });
                        });
                });
            });
    }

    fn display_json(&mut self, ui: &mut egui::Ui, i18n: &I18n) {
        egui::Grid::new("json objects")
            .spacing(egui::vec2(ui.spacing().item_spacing.x * 2.0, 0.0))
            .show(ui, |ui| {
                for header in &self.state.jsons {
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
                    ui.label(header.1.clone());
                    ui.end_row();
                }
            });
    }
}
