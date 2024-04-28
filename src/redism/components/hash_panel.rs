use std::collections::HashMap;

// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------
use eframe::egui;
use egui_extras::{Size, StripBuilder};
use redis::Connection;

use crate::{
    common::internationalization::I18n,
    components::{result_panel::ui_response_panel, widgets::ui_text_edit_singleline_hint},
    error, info,
    redism::{
        connection::RedisMenu,
        presenters::{
            delete_hashkey, hash::HashPresenter, run_cmd, run_read_generic, run_write_generic,
            RedisResponse,
        },
        state::RedisHashState,
        view::RedisView,
    },
    strip_text_edit, ui_button_w100,
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

            ui_response_panel(ui, &self.state.last_result);
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
                            strip_text_edit!(strip, "Key", self.state.hash_st.hexists_k);
                            strip_text_edit!(strip, "Field", self.state.hash_st.hexists_f);

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "HEXISTS") {
                                    self.state.last_result =
                                        self.run_read_hash(HashPresenter::hexists);
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
                            strip_text_edit!(strip, "Key", self.state.hash_st.hrandfield_k);
                            strip_text_edit!(strip, "Count", self.state.hash_st.hrandfield_count);

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "HRANDFIELD") {
                                    self.state.last_result =
                                        self.run_read_hash(HashPresenter::hrandfield);
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
                            strip_text_edit!(strip, "Key", self.state.hash_st.hget_k);

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "HLEN") {
                                    self.state.last_result =
                                        self.run_read_hash(HashPresenter::hlen);
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
                            strip_text_edit!(strip, "Key", self.state.hash_st.hstrlen_k);
                            strip_text_edit!(strip, "Field", self.state.hash_st.hstrlen_f);

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "HSTRLEN") {
                                    self.state.last_result =
                                        self.run_read_hash(HashPresenter::hstrlen);
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
                            strip_text_edit!(strip, "Key", self.state.hash_st.hdel_k);
                            strip_text_edit!(strip, "Field (& Fields)", self.state.hash_st.hdel_fs);

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "HDEL") {
                                    self.state.last_result =
                                        self.run_write_hash(HashPresenter::hdel);
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
                            strip_text_edit!(strip, "Key", self.state.hash_st.hset_k);
                            strip_text_edit!(strip, "Field", self.state.hash_st.hset_f);
                            strip_text_edit!(strip, "Value", self.state.hash_st.hset_v);

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "HSET") {
                                    self.state.last_result =
                                        self.run_write_hash(HashPresenter::hset);
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
                            strip_text_edit!(strip, "Key", self.state.hash_st.hsetnx_k);
                            strip_text_edit!(strip, "Field", self.state.hash_st.hsetnx_f);
                            strip_text_edit!(strip, "Value", self.state.hash_st.hsetnx_v);

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "HSETNX") {
                                    self.state.last_result =
                                        self.run_write_hash(HashPresenter::hsetnx);
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
                            strip_text_edit!(strip, "Key", self.state.hash_st.hincrby_k);
                            strip_text_edit!(strip, "Field", self.state.hash_st.hincrby_f);
                            strip_text_edit!(
                                strip,
                                "Increment Value",
                                self.state.hash_st.hincrby_increment
                            );

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "HINCRBY") {
                                    self.state.last_result =
                                        self.run_write_hash(HashPresenter::hincrby);
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
                            strip_text_edit!(strip, "Key", self.state.hash_st.hincrbyfloat_k);
                            strip_text_edit!(strip, "Field", self.state.hash_st.hincrbyfloat_f);
                            strip_text_edit!(
                                strip,
                                "Increment Value",
                                self.state.hash_st.hincrbyfloat_increment
                            );

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "HINCRBYFLOAT") {
                                    self.state.last_result =
                                        self.run_write_hash(HashPresenter::hincrbyfloat);
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
                            strip_text_edit!(strip, "Key", self.state.hash_st.hget_k);
                            strip_text_edit!(strip, "Field", self.state.hash_st.hget_f);

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "HGET") {
                                    self.state.last_result =
                                        self.run_read_hash(HashPresenter::hget);
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
                            strip_text_edit!(strip, "Key", self.state.hash_st.hmget_k);
                            strip_text_edit!(strip, "Field", self.state.hash_st.hmget_fs);

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "HMGET") {
                                    self.state.last_result =
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
                            strip_text_edit!(strip, "Key", self.state.hash_st.hgetall_k);

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "HGETALL") {
                                    self.state.last_result =
                                        self.run_read_hash(HashPresenter::hgetall);
                                }
                            });
                        });
                });

                outter_strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::exact(108.0))
                        .horizontal(|mut strip| {
                            strip_text_edit!(strip, "Key", self.state.hash_st.hkeys_k);

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "HKEYS") {
                                    self.state.last_result =
                                        self.run_read_hash(HashPresenter::hkeys);
                                }
                            });
                        });
                });

                outter_strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::exact(108.0))
                        .horizontal(|mut strip| {
                            strip_text_edit!(strip, "Key", self.state.hash_st.hvals_k);

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "HVALS") {
                                    self.state.last_result =
                                        self.run_read_hash(HashPresenter::hvals);
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
                                    match delete_hashkey(
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

    #[inline(always)]
    fn run_read_hash(
        &mut self,
        cb: impl Fn(&mut Connection, &RedisHashState) -> RedisResponse,
    ) -> Option<RedisResponse> {
        run_read_generic(&self.state.current_connection, &self.state.hash_st, cb)
    }

    #[inline(always)]
    fn run_write_hash(
        &mut self,
        cb: impl Fn(
            &mut Connection,
            &mut HashMap<String, Vec<(String, String)>>,
            &RedisHashState,
        ) -> RedisResponse,
    ) -> Option<RedisResponse> {
        run_write_generic(
            &self.state.current_connection,
            &self.state.hash_st,
            &mut self.state.hashes,
            cb,
        )
    }
}
