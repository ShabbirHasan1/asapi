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
        presenters::{hash::HashPresenter, run_cmd},
        view::RedisView,
    },
    ui_button_w100,
};

impl RedisView {
    pub fn show_hashes(&mut self, ui: &mut egui::Ui, i18n: &I18n) {
        if self.state.selected_menu == RedisMenu::Hash {
            egui::CollapsingHeader::new("Comandos Disponibles")
                .default_open(true)
                .show(ui, |ui| {
                    ui.columns(2, |uis| {
                        // 1 - HGET HGETALL HMGET HKEYS HVALS
                        self.hash_basic_info_cmds(&mut uis[0]);
                        // 2 - HDEL HSET HSETNX HINCRBY INCRBYFLOAT
                        self.hash_basic_setter_cmds(&mut uis[1]);
                    });
                    ui.separator();

                    ui.columns(2, |uis| {
                        // 1 - HLEN HSTRLEN
                        self.hash_len_cmds(&mut uis[0]);
                        // 2 - HEXISTS HRANDFIELD
                        self.hash_other_cmds(&mut uis[1]);
                    });
                    ui.separator();
                });

            ui_response_panel(ui, &self.state.opt_last_result);
        }

        self.hashes_display(ui, i18n);
    }

    fn hash_other_cmds(&mut self, ui: &mut egui::Ui) {
        StripBuilder::new(ui)
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
                                    &mut self.state.hash_st.hexists_k,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Field",
                                    &mut self.state.hash_st.hexists_f,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "HEXISTS") {
                                    self.state.opt_last_result =
                                        run_cmd(&self.state.current_connection, |conn| {
                                            HashPresenter::hexists(conn, &mut self.state.hash_st)
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
                                    &mut self.state.hash_st.hrandfield_k,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Count",
                                    &mut self.state.hash_st.hrandfield_count,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "HRANDFIELD") {
                                    self.state.opt_last_result =
                                        run_cmd(&self.state.current_connection, |conn| {
                                            HashPresenter::hrandfield(conn, &mut self.state.hash_st)
                                        });
                                }
                            });
                        });
                });
            });
    }

    fn hash_len_cmds(&mut self, ui: &mut egui::Ui) {
        StripBuilder::new(ui)
            .size(Size::exact(20.0))
            .size(Size::exact(20.0))
            .vertical(|mut outter_strip| {
                outter_strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::exact(108.0))
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Key",
                                    &mut self.state.hash_st.hget_k,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "HLEN") {
                                    self.state.opt_last_result =
                                        run_cmd(&self.state.current_connection, |conn| {
                                            HashPresenter::hlen(conn, &mut self.state.hash_st)
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
                                    &mut self.state.hash_st.hstrlen_k,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Field",
                                    &mut self.state.hash_st.hstrlen_f,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "HSTRLEN") {
                                    self.state.opt_last_result =
                                        run_cmd(&self.state.current_connection, |conn| {
                                            HashPresenter::hstrlen(conn, &mut self.state.hash_st)
                                        });
                                }
                            });
                        });
                });
            });
    }

    fn hash_basic_setter_cmds(&mut self, ui: &mut egui::Ui) {
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
                                    &mut self.state.hash_st.hdel_k,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Field (& Fields)",
                                    &mut self.state.hash_st.hdel_fs,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "HDEL") {
                                    self.state.opt_last_result =
                                        run_cmd(&self.state.current_connection, |conn| {
                                            HashPresenter::hdel(
                                                conn,
                                                &mut self.state.hashes,
                                                &mut self.state.hash_st,
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
                                    &mut self.state.hash_st.hset_k,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Field",
                                    &mut self.state.hash_st.hset_f,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Value",
                                    &mut self.state.hash_st.hset_v,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "HSET") {
                                    self.state.opt_last_result =
                                        run_cmd(&self.state.current_connection, |conn| {
                                            HashPresenter::hset(
                                                conn,
                                                &mut self.state.hashes,
                                                &mut self.state.hash_st,
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
                                    &mut self.state.hash_st.hsetnx_k,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Field",
                                    &mut self.state.hash_st.hsetnx_f,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Value",
                                    &mut self.state.hash_st.hsetnx_v,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "HSETNX") {
                                    self.state.opt_last_result =
                                        run_cmd(&self.state.current_connection, |conn| {
                                            HashPresenter::hsetnx(
                                                conn,
                                                &mut self.state.hashes,
                                                &mut self.state.hash_st,
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
                                    &mut self.state.hash_st.hincrby_k,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Field",
                                    &mut self.state.hash_st.hincrby_f,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Increment Value",
                                    &mut self.state.hash_st.hincrby_increment,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "HINCRBY") {
                                    self.state.opt_last_result =
                                        run_cmd(&self.state.current_connection, |conn| {
                                            HashPresenter::hincrby(
                                                conn,
                                                &mut self.state.hashes,
                                                &mut self.state.hash_st,
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
                                    &mut self.state.hash_st.hincrbyfloat_k,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Field",
                                    &mut self.state.hash_st.hincrbyfloat_f,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Increment Value",
                                    &mut self.state.hash_st.hincrbyfloat_increment,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "HINCRBYFLOAT") {
                                    self.state.opt_last_result =
                                        run_cmd(&self.state.current_connection, |conn| {
                                            HashPresenter::hincrbyfloat(
                                                conn,
                                                &mut self.state.hashes,
                                                &mut self.state.hash_st,
                                            )
                                        });
                                }
                            });
                        });
                });
            });
    }

    fn hash_basic_info_cmds(&mut self, ui: &mut egui::Ui) {
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
                                    &mut self.state.hash_st.hget_k,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Field",
                                    &mut self.state.hash_st.hget_f,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "HGET") {
                                    self.state.opt_last_result =
                                        run_cmd(&self.state.current_connection, |conn| {
                                            HashPresenter::hget(conn, &mut self.state.hash_st)
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
                                    &mut self.state.hash_st.hmget_k,
                                );
                            });

                            strip.cell(|ui| {
                                ui_text_edit_singleline_hint(
                                    ui,
                                    "Field (& Fields)",
                                    &mut self.state.hash_st.hmget_fs,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "HMGET") {
                                    self.state.opt_last_result =
                                        run_cmd(&self.state.current_connection, |conn| {
                                            HashPresenter::hmget(conn, &mut self.state.hash_st)
                                        });
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
                                    "Key",
                                    &mut self.state.hash_st.hgetall_k,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "HGETALL") {
                                    self.state.opt_last_result =
                                        run_cmd(&self.state.current_connection, |conn| {
                                            HashPresenter::hgetall(conn, &mut self.state.hash_st)
                                        });
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
                                    "Key",
                                    &mut self.state.hash_st.hkeys_k,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "HKEYS") {
                                    self.state.opt_last_result =
                                        run_cmd(&self.state.current_connection, |conn| {
                                            HashPresenter::hkeys(conn, &mut self.state.hash_st)
                                        });
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
                                    "Key",
                                    &mut self.state.hash_st.hvals_k,
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "HVALS") {
                                    self.state.opt_last_result =
                                        run_cmd(&self.state.current_connection, |conn| {
                                            HashPresenter::hvals(conn, &mut self.state.hash_st)
                                        });
                                }
                            });
                        });
                });
            });
    }

    fn hashes_display(&mut self, ui: &mut egui::Ui, _i18n: &I18n) {
        ui.set_width(ui.available_width());
        for (h_name, v) in &self.state.hashes {
            // --> Manejamos acciones sobre elemento que muestra nombre del hash
            ui.collapsing(h_name, |ui| {
                // TODO: Borrar todos en cascada con el menú contextual del hash.
                egui::Grid::new(h_name)
                    .spacing(egui::vec2(ui.spacing().item_spacing.x * 2.0, 0.0))
                    .show(ui, |ui| {
                        for (field_key, field_value) in v {
                            let field_label =
                                ui.label(format!("    {} : {}", field_key, field_value));

                            // --> Cada campo se puede borrar con menú contextual <--
                            field_label.context_menu(|ui| {
                                if ui.button("Delete").clicked() {
                                    match presenter::delete_hashkey(
                                        &self.state.current_connection.host,
                                        &self.state.current_connection.port,
                                        h_name,
                                        field_key,
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
                            ui.end_row();
                        }
                    });
            });
        }
    }
}
