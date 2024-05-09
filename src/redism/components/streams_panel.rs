// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use eframe::egui::{self, Label, Sense};
use egui_extras::{Size, StripBuilder};
use egui_json_tree::{DefaultExpand, JsonTree};
use redis::Connection;
use std::collections::HashMap;
use std::time::SystemTime;

use crate::{
    common::internationalization::I18n,
    components::result_panel::ui_response_panel,
    redism::{
        connection::RedisMenu,
        presenters::{
            delete_key, run_read_generic, run_write_generic,
            stream::{self, blocking_xread, blocking_xread_group, read_stream_id},
            RedisResponse,
        },
        state::RedisStreamState,
        utils::value_map_to_string_btree_map,
        view::RedisView,
    },
    strip_combo_box, strip_text_edit, ui_button_w100,
};

use super::contextual_menus;

impl RedisView {
    pub fn show_streams(&mut self, ui: &mut egui::Ui, i18n: &I18n) {
        if self.state.selected_menu == RedisMenu::Stream {
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(
                        self.state.show_regular_commands,
                        &i18n.redis_regular_commands,
                    )
                    .clicked()
                {
                    self.state.show_regular_commands = !self.state.show_regular_commands;
                }
                if ui
                    .selectable_label(self.state.show_read_commands, &i18n.redis_read_commands)
                    .clicked()
                {
                    self.state.show_read_commands = !self.state.show_read_commands;
                }
                if ui.button(&i18n.redis_clean_result).clicked() {
                    self.state.last_result = None;
                    self.state.last_stream_read_error = None;
                    self.state.stream_st.streams.clear();
                }
            });
            ui.separator();

            if self.state.show_regular_commands {
                self.show_regular_streams_commands(ui, i18n);
                if self.state.show_read_commands && self.state.last_result.is_some() {
                    ui.separator();
                }
            }
            if self.state.show_read_commands {
                self.show_read_streams(ui, i18n);
            }
            if self.state.show_read_commands || self.state.show_regular_commands {
                ui.separator();
            }
        }
        self.display_streams(ui, i18n);
    }

    fn show_read_streams(&mut self, ui: &mut egui::Ui, i18n: &I18n) {
        while let Ok(msg) = self.state.stream_st.rx.try_recv() {
            match msg {
                stream::RedisStreamMessage::Pending(msg) => {
                    self.state.stream_st.streams.push(msg);
                }
                stream::RedisStreamMessage::Ready(msg) => {
                    let position = self
                        .state
                        .stream_st
                        .streams
                        .iter()
                        .position(|stream| stream.stream == msg.stream);
                    if let Some(position) = position {
                        self.state.stream_st.streams[position] = msg;
                    }
                }
                stream::RedisStreamMessage::Error(err) => {
                    self.state.last_stream_read_error = Some(err);
                }
            }
        }
        egui::CollapsingHeader::new(
            i18n.redis_stream_reader_commands_header
                .to_ascii_uppercase(),
        )
        .show_background(true)
        .default_open(true)
        .show(ui, |ui| {
            ui.columns(1, |uis| {
                self.stream_read_commands_1(&mut uis[0]);
            });
            ui.columns(1, |uis| {
                self.stream_read_commands_2(&mut uis[0]);
            });
            ui.separator();
        });

        let mut n_col = 0;

        ui.columns(2, |uis| {
            for ui in uis {
                for (idx, reader_storage) in self.state.stream_st.streams.iter().enumerate() {
                    let column_idx = idx % 2;
                    if n_col == column_idx {
                        egui::Frame::default()
                            .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.monospace(&i18n.redis_stream_stream_prefix);
                                    ui.label(&reader_storage.stream);
                                });
                                if reader_storage.group.is_some() {
                                    ui.monospace(&i18n.redis_stream_group_prefix);
                                    ui.label(reader_storage.group.as_ref().unwrap());
                                }
                                JsonTree::new(
                                    &reader_storage.stream,
                                    &serde_json::json!(&reader_storage.messages),
                                )
                                .default_expand(DefaultExpand::ToLevel(0))
                                .show(ui);
                                let now = SystemTime::now();
                                let duration =
                                    now.duration_since(reader_storage.system_time).map_or_else(
                                        |_| "error".to_string(),
                                        |elapsed| {
                                            if reader_storage.block_ms.is_some_and(|ms| {
                                                (ms as u128) > elapsed.as_millis()
                                            }) {
                                                ui.ctx().request_repaint();
                                                format!(
                                                    "{v}",
                                                    v = (reader_storage.block_ms.unwrap() as u128
                                                        - elapsed.as_millis())
                                                        / 1000
                                                )
                                            } else {
                                                "0".to_string()
                                            }
                                        },
                                    );
                                ui.label(format!(
                                    "{} {duration} s",
                                    &i18n.redis_stream_block_prefix
                                ));
                            });
                    }
                }
                n_col += 1;
            }
        });
    }

    fn stream_read_commands_1(&mut self, ui: &mut egui::Ui) {
        StripBuilder::new(ui)
            .size(Size::exact(20.0))
            .vertical(|mut strip| {
                strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::exact(128.0))
                        .horizontal(|mut strip| {
                            strip_text_edit!(strip, "(Count)", self.state.stream_st.xread_count);
                            strip_text_edit!(
                                strip,
                                "(Block ms)",
                                self.state.stream_st.xread_block_ms
                            );
                            strip_text_edit!(strip, "Keys", self.state.stream_st.xread_keys);
                            strip_text_edit!(strip, "Ids", self.state.stream_st.xread_ids);

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "XREAD") {
                                    blocking_xread(
                                        self.state.current_connection.clone(),
                                        &self.state.stream_st,
                                        &self.state.stream_st.tx,
                                    );

                                    self.state.stream_st.streams.clear();
                                    self.state.last_stream_read_error = None;
                                }
                            });
                        });
                });
            });
    }

    fn stream_read_commands_2(&mut self, ui: &mut egui::Ui) {
        StripBuilder::new(ui)
            .size(Size::exact(20.0))
            .vertical(|mut strip| {
                strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::exact(128.0))
                        .horizontal(|mut strip| {
                            strip_text_edit!(strip, "Group", self.state.stream_st.xreadgroup_group);
                            strip_text_edit!(
                                strip,
                                "Consumer",
                                self.state.stream_st.xreadgroup_consumer
                            );
                            strip_text_edit!(
                                strip,
                                "(Count)",
                                self.state.stream_st.xreadgroup_count
                            );
                            strip_text_edit!(
                                strip,
                                "(Block ms)",
                                self.state.stream_st.xreadgroup_block_ms
                            );
                            strip.cell(|ui| {
                                ui.checkbox(&mut self.state.stream_st.xreadgroup_noack, "NOACK");
                            });

                            strip_text_edit!(strip, "Keys", self.state.stream_st.xreadgroup_keys);
                            strip_text_edit!(strip, "Ids", self.state.stream_st.xreadgroup_ids);

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "XREAD GROUP") {
                                    blocking_xread_group(
                                        self.state.current_connection.clone(),
                                        &self.state.stream_st,
                                        &self.state.stream_st.tx,
                                    );

                                    self.state.stream_st.streams.clear();
                                    self.state.last_stream_read_error = None;
                                }
                            });
                        });
                })
            });
    }

    fn show_regular_streams_commands(&mut self, ui: &mut egui::Ui, i18n: &I18n) {
        egui::CollapsingHeader::new(i18n.redis_commands_header.to_ascii_uppercase())
            .show_background(true)
            .default_open(true)
            .show(ui, |ui| {
                ui.columns(2, |uis| {
                    self.stream_info_commands(&mut uis[0]);
                    self.stream_extra_info_commands(&mut uis[1]);
                });
                ui.separator();

                ui.columns(2, |uis| {
                    self.stream_basic_modification_commands(&mut uis[0]);
                    self.stream_group_modification_commands(&mut uis[1]);
                });
                ui.separator();
            });

        ui_response_panel(ui, &self.state.last_result);
    }

    fn stream_group_modification_commands(&mut self, ui: &mut egui::Ui) {
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
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::exact(128.0))
                        .horizontal(|mut strip| {
                            strip_text_edit!(strip, "Key", self.state.stream_st.xgroup_create_k);
                            strip_text_edit!(
                                strip,
                                "Group",
                                self.state.stream_st.xgroup_create_group
                            );
                            strip_text_edit!(strip, "Id", self.state.stream_st.xgroup_create_id);

                            strip.cell(|ui| {
                                ui.checkbox(
                                    &mut self.state.stream_st.xgroup_create_mkstream,
                                    "MKSTREAM",
                                );
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "XGROUP CREATE") {
                                    self.state.last_result =
                                        self.run_write_stream(stream::xgroup_create);
                                }
                            });
                        });
                });

                strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::exact(178.0))
                        .horizontal(|mut strip| {
                            strip_text_edit!(
                                strip,
                                "Key",
                                self.state.stream_st.xgroup_create_consumer_k
                            );
                            strip_text_edit!(
                                strip,
                                "Group",
                                self.state.stream_st.xgroup_create_consumer_group
                            );
                            strip_text_edit!(
                                strip,
                                "Consumer",
                                self.state.stream_st.xgroup_create_consumer
                            );

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "XGROUP CREATE CONSUMER") {
                                    self.state.last_result =
                                        self.run_write_stream(stream::xgroup_create_consumer);
                                }
                            });
                        });
                });

                strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::exact(178.0))
                        .horizontal(|mut strip| {
                            strip_text_edit!(
                                strip,
                                "Key",
                                self.state.stream_st.xgroup_del_consumer_k
                            );
                            strip_text_edit!(
                                strip,
                                "Group",
                                self.state.stream_st.xgroup_del_consumer_group
                            );
                            strip_text_edit!(
                                strip,
                                "Consumer",
                                self.state.stream_st.xgroup_del_consumer
                            );

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "XGROUP DEL CONSUMER") {
                                    self.state.last_result =
                                        self.run_write_stream(stream::xgroup_del_consumer);
                                }
                            });
                        });
                });

                strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::exact(178.0))
                        .horizontal(|mut strip| {
                            strip_text_edit!(strip, "Key", self.state.stream_st.xgroup_destroy_k);
                            strip_text_edit!(
                                strip,
                                "Group",
                                self.state.stream_st.xgroup_destroy_group
                            );

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "XGROUP DESTROY") {
                                    self.state.last_result =
                                        self.run_write_stream(stream::xgroup_destroy);
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
                            strip_text_edit!(strip, "Key", self.state.stream_st.xgroup_setid_k);
                            strip_text_edit!(strip, "Group", self.state.stream_st.xgroup_setid_g);
                            strip_text_edit!(strip, "Id", self.state.stream_st.xgroup_setid_id);

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "XGROUP SETID") {
                                    self.state.last_result =
                                        self.run_write_stream(stream::xgroup_setid);
                                }
                            });
                        });
                });
            });
    }

    fn stream_basic_modification_commands(&mut self, ui: &mut egui::Ui) {
        StripBuilder::new(ui)
            .size(Size::exact(20.0))
            .size(Size::exact(20.0))
            .size(Size::exact(20.0))
            .vertical(|mut strip| {
                strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::exact(128.0))
                        .horizontal(|mut strip| {
                            strip_text_edit!(strip, "Key", self.state.stream_st.xadd_k);

                            strip.cell(|ui| {
                                ui.checkbox(
                                    &mut self.state.stream_st.xadd_nomkstream,
                                    "NOMKSTREAM",
                                );
                            });

                            strip_text_edit!(strip, "Id", self.state.stream_st.xadd_id);
                            strip_text_edit!(strip, "Items", self.state.stream_st.xadd_items);

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "XADD") {
                                    self.state.last_result = self.run_write_stream(stream::xadd);
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
                            strip_text_edit!(strip, "Key", self.state.stream_st.xdel_k);
                            strip_text_edit!(strip, "Id (& Ids)", self.state.stream_st.xdel_ids);

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "XDEL") {
                                    self.state.last_result = self.run_write_stream(stream::xdel);
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
                        .size(Size::remainder())
                        .size(Size::exact(128.0))
                        .horizontal(|mut strip| {
                            strip_text_edit!(strip, "Key", self.state.stream_st.xtrim_k);
                            strip_combo_box!(
                                strip,
                                "xtrim_maxlen_minid",
                                self.state.stream_st.xtrim_maxlen_minid,
                                "MAXLEN",
                                "MINID"
                            );
                            strip_combo_box!(
                                strip,
                                "xtrim_aprox_equal",
                                self.state.stream_st.xtrim_aprox_equal,
                                "=",
                                "~"
                            );
                            strip_text_edit!(
                                strip,
                                "Threshold",
                                self.state.stream_st.xtrim_threshold
                            );
                            strip_text_edit!(strip, "(Limit)", self.state.stream_st.xtrim_limit);

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "XTRIM") {
                                    self.state.last_result = self.run_write_stream(stream::xtrim);
                                }
                            });
                        });
                });
            });
    }

    fn stream_extra_info_commands(&mut self, ui: &mut egui::Ui) {
        StripBuilder::new(ui)
            .size(Size::exact(20.0))
            .size(Size::exact(20.0))
            .size(Size::exact(20.0))
            .size(Size::exact(20.0))
            .vertical(|mut strip| {
                strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::exact(128.0))
                        .horizontal(|mut strip| {
                            strip_text_edit!(strip, "Key", self.state.stream_st.xlen_k);

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "XLEN") {
                                    self.state.last_result = self.run_read_stream(stream::xlen);
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
                        .size(Size::exact(128.0))
                        .horizontal(|mut strip| {
                            strip_text_edit!(strip, "Key", self.state.stream_st.xrange_k);
                            strip_text_edit!(strip, "Start", self.state.stream_st.xrange_start);
                            strip_text_edit!(strip, "End", self.state.stream_st.xrange_end);
                            strip_text_edit!(strip, "(Count)", self.state.stream_st.xrange_count);

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "XRANGE") {
                                    self.state.last_result = self.run_read_stream(stream::xrange);
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
                        .size(Size::exact(128.0))
                        .horizontal(|mut strip| {
                            strip_text_edit!(strip, "Key", self.state.stream_st.xrevrange_k);
                            strip_text_edit!(strip, "Start", self.state.stream_st.xrevrange_start);
                            strip_text_edit!(strip, "End", self.state.stream_st.xrevrange_end);
                            strip_text_edit!(
                                strip,
                                "(Count)",
                                self.state.stream_st.xrevrange_count
                            );

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "XREVRANGE") {
                                    self.state.last_result =
                                        self.run_read_stream(stream::xrevrange);
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
                            strip_text_edit!(strip, "Key", self.state.stream_st.xack_k);
                            strip_text_edit!(strip, "Group", self.state.stream_st.xack_group);
                            strip_text_edit!(strip, "Id (& Ids)", self.state.stream_st.xack_ids);

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "XACK") {
                                    self.state.last_result = self.run_read_stream(stream::xack);
                                }
                            });
                        });
                })
            });
    }

    fn stream_info_commands(&mut self, ui: &mut egui::Ui) {
        StripBuilder::new(ui)
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
                            strip_text_edit!(strip, "Key", self.state.stream_st.info_stream_k);
                            strip_text_edit!(
                                strip,
                                "(Count)",
                                self.state.stream_st.info_stream_count
                            );
                            strip.cell(|ui| {
                                ui.checkbox(&mut self.state.stream_st.info_stream_full, "Full");
                            });

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "XINFO STREAM") {
                                    self.state.last_result =
                                        self.run_read_stream(stream::info_stream);
                                }
                            });
                        });
                });

                strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::exact(128.0))
                        .horizontal(|mut strip| {
                            strip_text_edit!(strip, "Key", self.state.stream_st.info_groups_k);

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "XINFO GROUPS") {
                                    self.state.last_result =
                                        self.run_read_stream(stream::info_groups);
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
                            strip_text_edit!(strip, "Key", self.state.stream_st.info_consumers_k);
                            strip_text_edit!(strip, "Group", self.state.stream_st.info_consumers_g);

                            strip.cell(|ui| {
                                if ui_button_w100!(ui, "XINFO CONSUMERS") {
                                    self.state.last_result =
                                        self.run_read_stream(stream::info_consumers);
                                }
                            });
                        });
                });
            });
    }

    fn display_streams(&mut self, ui: &mut egui::Ui, i18n: &I18n) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.set_width(ui.available_width());
            for (stream_name, v) in &self.state.streams {
                // ==> Gestión de Stream y todos los mensajes en él
                ui.collapsing(stream_name, |ui| {
                    for (idx, id) in v.iter().enumerate() {
                        // --> Gestión de cada mensaje <--
                        let label = match self.state.stream_id_values.get(id) {
                            Some(_) => ui.add(Label::new(id).sense(Sense::click())),
                            _ => ui
                                .add(Label::new(id).sense(Sense::click()))
                                .on_hover_text(&i18n.redis_stream_hover_info),
                        };

                        label.context_menu(|ui| {
                            // TODO: Aquí estoy cogiendo valores leídos
                            let option = self.state.stream_id_values.get(id);
                            self.state.must_scan = contextual_menus::stream_msg(
                                ui,
                                stream_name,
                                id.to_string().to_string(),
                                option,
                                &mut self.state.current_command,
                            );
                        });
                        if label.clicked() {
                            match self.state.stream_id_values.get(id) {
                                Some(_) => {
                                    self.state.stream_id_values.remove(id);
                                }
                                None => {
                                    // Hace falta esto porque cuando busco, si no es desde 0, el
                                    // que me devuelve es el siguiente al que selecciono, por
                                    // eso me hace falta el `idx-1`.
                                    let from_when = if idx == 0 { "0" } else { &v[idx - 1] };
                                    let _ = read_stream_id(
                                        &self.state.current_connection,
                                        stream_name,
                                        from_when,
                                        &mut self.state.stream_id_values,
                                    );
                                }
                            }
                        }
                        ui.end_row();
                        // TODO: Cambiar y almacenar los serde_json::Value para no estar
                        // haciendo el parseo continumamente. Eso nos permite volver a usar
                        // HashMap en vez de BTreeMap, aunque lo mejor sería comprobar el
                        // rendimiento al crear cada uno.
                        if let Some(value) = self.state.stream_id_values.get(id) {
                            // let value = serde_json::json!(value_map_to_string_map(value));
                            let value = serde_json::json!(value_map_to_string_btree_map(value));
                            JsonTree::new(id, &value).show(ui);
                        }
                    }
                })
                .header_response
                .context_menu(|ui| {
                    if ui.button(&i18n.redis_delete_ds).clicked() {
                        match delete_key(
                            &self.state.current_connection.host,
                            &self.state.current_connection.port,
                            stream_name,
                        ) {
                            Ok(_s) => {
                                self.state.must_scan = true;
                            }
                            Err(e) => log::error!("{:?}", e),
                        }
                        ui.close_menu();
                    }
                });
            }
        });
    }

    #[inline(always)]
    fn run_read_stream(
        &mut self,
        cb: impl Fn(&mut Connection, &RedisStreamState) -> RedisResponse,
    ) -> Option<RedisResponse> {
        run_read_generic(&self.state.current_connection, &self.state.stream_st, cb)
    }

    #[inline(always)]
    fn run_write_stream(
        &mut self,
        cb: impl Fn(
            &mut Connection,
            &mut HashMap<String, Vec<String>>,
            &RedisStreamState,
        ) -> RedisResponse,
    ) -> Option<RedisResponse> {
        run_write_generic(
            &self.state.current_connection,
            &self.state.stream_st,
            &mut self.state.streams,
            cb,
        )
    }
}
