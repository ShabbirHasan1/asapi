// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use eframe::egui;
use egui_extras::{Size, StripBuilder};
use std::collections::HashSet;
use tokio::{runtime::Runtime, sync::mpsc::Sender};

use crate::{
    clickhousem::{
        components::contextual_menus::{ClickHouseTableContextMenu, ClickHouseTableInfo},
        domain::{ClickHouseConnectionDefinition, ClickHouseMessage},
        presenter,
        state::{ClickHouseAppState, ClickHouseState},
    },
    common::internationalization::I18nClickHouse,
    sqlx_common::state::QuerySort,
};

pub struct ClickHouseSideNav {
    connections_subpanel: ClickHouseConnectionsSubpanel,
}

impl Default for ClickHouseSideNav {
    fn default() -> Self {
        ClickHouseSideNav {
            connections_subpanel: ClickHouseConnectionsSubpanel {
                is_edit_connection_menu_opened: false,
            },
        }
    }
}

impl ClickHouseSideNav {
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        rt: &Runtime,
        tx: &Sender<ClickHouseMessage>,
        tx_sync: &std::sync::mpsc::Sender<ClickHouseMessage>,
        app_st: &mut ClickHouseAppState,
        local_st: &mut ClickHouseState,
        i18n: &I18nClickHouse,
    ) {
        if app_st.show_sidebar {
            egui::SidePanel::left("ClickHouse_connections_panel").show(ctx, |ui| {
                // --> Decidimos qué mostrar <--
                ui.horizontal(|ui| {
                    let s1 = if local_st.sql.hide_connections {
                        "\u{229e}"
                    } else {
                        "\u{229f}"
                    };
                    let s2 = if local_st.sql.hide_tables {
                        "\u{229e}"
                    } else {
                        "\u{229f}"
                    };

                    if ui.button(format!("{s1} {}", i18n.connections)).clicked() {
                        local_st.sql.hide_connections = !local_st.sql.hide_connections;
                    }
                    if ui.button(format!("{s2} {}", i18n.tables)).clicked() {
                        local_st.sql.hide_tables = !local_st.sql.hide_tables;
                    }
                });

                // --> Abrimos ventana para definir conexión <--
                ui.menu_button(&i18n.btn_add_connection, |ui| {
                    edit_connection(ui, i18n, &mut local_st.tmp_connection, tx_sync, None);
                });

                // --> Mostramos Conexiones <--
                if !local_st.sql.hide_connections && !local_st.sql.hide_tables {
                    StripBuilder::new(ui)
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .vertical(|mut strip| {
                            strip.cell(|ui| {
                                self.connections_subpanel
                                    .show(ctx, rt, ui, app_st, local_st, tx_sync, i18n);
                            });
                            strip.cell(|ui| {
                                ui.vertical_centered(|ui| {
                                    ui.separator();
                                    ClickHouseTablesSubpanel::show(
                                        ctx, rt, ui, tx, tx_sync, local_st, i18n,
                                    );
                                });
                            });
                        });
                } else if !local_st.sql.hide_connections {
                    self.connections_subpanel
                        .show(ctx, rt, ui, app_st, local_st, tx_sync, i18n);
                } else if !local_st.sql.hide_tables {
                    ClickHouseTablesSubpanel::show(ctx, rt, ui, tx, tx_sync, local_st, i18n);
                }
            });
        }
    }
}

pub struct ClickHouseConnectionsSubpanel {
    is_edit_connection_menu_opened: bool,
}

impl ClickHouseConnectionsSubpanel {
    pub fn show(
        &mut self,
        _ctx: &egui::Context,
        rt: &Runtime,
        ui: &mut egui::Ui,
        app_st: &mut ClickHouseAppState,
        local_st: &mut ClickHouseState,
        tx_sync: &std::sync::mpsc::Sender<ClickHouseMessage>,
        i18n: &I18nClickHouse,
    ) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            let mut connections_to_delete: HashSet<usize> = HashSet::new();

            for (idx, conn_definition) in app_st.connections.iter_mut().enumerate() {
                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    ui.set_width(ui.available_width());
                    let button_text = format!(
                        "{}\n{}:{}\n{} / {}",
                        conn_definition.name.clone(),
                        conn_definition.host.clone(),
                        conn_definition.port.clone(),
                        conn_definition.dbname.clone(),
                        conn_definition.user.clone()
                    );
                    let raw_button = if local_st.pool.is_some() {
                        egui::Button::new(button_text)
                            .min_size(egui::vec2(200.0, 24.0))
                            .stroke(if idx == local_st.sql.current_connection_idx {
                                egui::Stroke::new(1.0, egui::Color32::DARK_BLUE)
                            } else {
                                egui::Stroke::new(0.0, egui::Color32::LIGHT_BLUE)
                            })
                    } else {
                        egui::Button::new(button_text).min_size(egui::vec2(200.0, 24.0))
                    };

                    let button = ui.add(raw_button);

                    // --> Menú contextual para manejo de las conexiones <--
                    button.context_menu(|ui| {
                        if ui.button(&i18n.close_connection).clicked() {
                            close_connection(rt, local_st);
                            local_st.sql.current_connection_idx = usize::MAX;
                            ui.close_menu();
                        }
                        if ui.button(&i18n.delete_connection).clicked() {
                            connections_to_delete.insert(idx);
                            // Si la conexión que borramos existe, cerramos antes
                            if local_st.sql.current_connection_idx != idx {
                                close_connection(rt, local_st);
                            }
                            local_st.sql.current_connection_idx = usize::MAX;
                            ui.close_menu();
                        }

                        let mut menu_open = false;
                        ui.menu_button(&i18n.edit_connection, |ui| {
                            menu_open = true;
                            if menu_open && !self.is_edit_connection_menu_opened {
                                local_st.tmp_connection = conn_definition.clone();
                            }
                            edit_connection(
                                ui,
                                i18n,
                                &mut local_st.tmp_connection,
                                tx_sync,
                                Some(idx),
                            );
                        });

                        self.is_edit_connection_menu_opened = menu_open;

                        if ui.button(&i18n.reload_tables).clicked() {
                            let pool_ref = local_st.pool.as_ref().unwrap().clone();
                            let pool_ref_2 = local_st.pool.as_ref().unwrap().clone();

                            local_st.sql.tables = rt.block_on(async move {
                                presenter::list_connection_tables(&pool_ref).await
                            });

                            let tables = local_st.sql.tables.clone();
                            let db_name = conn_definition.dbname.clone();

                            local_st.sql.current_connection_tables_info = rt.block_on(async move {
                                presenter::tables_info(&pool_ref_2, &db_name, tables.as_ref()).await
                            });
                        }
                    });

                    // --> Al clicar sobre conexión, conectamos y listamos tablas <--
                    // Si estamos ya mostrando esta conexión, clicar sobre ella no lanza ninguna acción.
                    if button.clicked() && local_st.sql.current_connection_idx != idx {
                        local_st.sql.current_connection_idx = idx;
                        // Este método pone `pool` a `None`.
                        close_connection(rt, local_st);
                        local_st.sql.reset();
                        local_st.sql.tables.clear();

                        // Si no conexión o la que existe no es la que clico, la defino
                        if local_st.pool.is_none() {
                            let conn = ClickHouseConnectionDefinition {
                                name: conn_definition.name.clone(),
                                host: conn_definition.host.clone(),
                                port: conn_definition.port.clone(),
                                user: conn_definition.user.clone(),
                                password: conn_definition.password.clone(),
                                dbname: conn_definition.dbname.clone(),
                                options: todo!(),
                            };
                            local_st.current_connection = conn.clone();

                            local_st.pool = Some(presenter::connect(conn));

                            if local_st.pool.is_some() {
                                let pool_ref = local_st.pool.as_ref().unwrap().clone();
                                let pool_ref2 = local_st.pool.as_ref().unwrap().clone();

                                local_st.sql.tables = rt.block_on(async move {
                                    presenter::list_connection_tables(&pool_ref).await
                                });

                                let tables = local_st.sql.tables.clone();
                                local_st.sql.current_connection_tables_info =
                                    rt.block_on(async move {
                                        presenter::tables_info(
                                            &pool_ref2,
                                            &conn_definition.dbname,
                                            tables.as_ref(),
                                        )
                                        .await
                                    });
                            }
                        }
                    }
                });
            }

            if !connections_to_delete.is_empty() {
                let mut i = 0;
                let mut to_retain: Vec<ClickHouseConnectionDefinition> = Vec::new();
                while i < app_st.connections.len() {
                    if !connections_to_delete.contains(&i) {
                        to_retain.push(app_st.connections.get(i).unwrap().clone());
                    }
                    i += 1;
                }
                app_st.connections = to_retain;
            }
        });
    }
}

pub struct ClickHouseTablesSubpanel;
impl ClickHouseTablesSubpanel {
    pub fn show(
        _ctx: &egui::Context,
        rt: &Runtime,
        ui: &mut egui::Ui,
        tx: &Sender<ClickHouseMessage>,
        tx_sync: &std::sync::mpsc::Sender<ClickHouseMessage>,
        local_st: &mut ClickHouseState,
        i18n: &I18nClickHouse,
    ) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("pg_db_tables")
                .num_columns(2)
                .show(ui, |ui| {
                    for (table_idx, table_name) in local_st.sql.tables.clone().iter().enumerate() {
                        ui.label(
                            egui::RichText::new("Info")
                                .color(egui::Color32::from_rgb(128, 128, 128)),
                        )
                        .on_hover_ui(|ui| {
                            ClickHouseTableInfo::show(ui, &local_st.sql, table_name);
                        });

                        let table_btn = ui.selectable_value(
                            &mut local_st.sql.current_table_idx,
                            table_idx,
                            table_name,
                        );

                        // Sin problemas, para que esto se muestre tiene que existir conexión.
                        table_btn.context_menu(|ui| {
                            ClickHouseTableContextMenu::show(
                                ui,
                                tx_sync,
                                &mut local_st.sql,
                                i18n,
                                table_name,
                            );
                        });

                        if table_btn.clicked() {
                            let pool_ref = local_st.pool.as_ref().unwrap().clone();
                            let tx_cloned = tx.clone();
                            let t_name = table_name.to_string();
                            // rt.spawn(async move {
                            //     presenter::select_all_with_column_description(
                            //         &pool_ref,
                            //         &tx_cloned,
                            //         &t_name,
                            //         QuerySort::None,
                            //     )
                            //     .await
                            // });
                            local_st.sql.query_sort = QuerySort::None;
                            local_st.sql.sql_statement = format!("SELECT * FROM {}", table_name);
                        }

                        ui.end_row();
                    }
                });
        });
    }
}

// ==================================================
// Funciones comunes
// ==================================================
fn close_connection(_rt: &Runtime, local_state: &mut ClickHouseState) {
    // Usar `guard` facilita mucho porque take sobre referencia no puede usarse,
    // y usar is_some y dentro hacer algo genera problemas de prestado de
    // referencia.
    if local_state.pool.is_none() {
        return;
    }

    // Clickhouse.rs no da la opción de cerrar. Entiendo que el Drop se encarga
    // TODO: Revisar esto
    local_state.pool = None;
}

fn edit_connection(
    ui: &mut egui::Ui,
    i18n: &I18nClickHouse,
    tmp_connection: &mut ClickHouseConnectionDefinition,
    tx: &std::sync::mpsc::Sender<ClickHouseMessage>,
    idx: Option<usize>,
) {
    ui.set_min_width(200.0);

    ui.horizontal(|ui| {
        ui.label(&i18n.connection_name);
        ui.text_edit_singleline(&mut tmp_connection.name);
    });

    ui.horizontal(|ui| {
        ui.label(&i18n.connection_host);
        ui.text_edit_singleline(&mut tmp_connection.host);
    });

    ui.horizontal(|ui| {
        ui.label(&i18n.connection_port);
        ui.text_edit_singleline(&mut tmp_connection.port);
    });

    ui.horizontal(|ui| {
        ui.label(&i18n.connection_user);
        ui.text_edit_singleline(&mut tmp_connection.user);
    });

    ui.horizontal(|ui| {
        ui.label(&i18n.connection_password);
        ui.text_edit_singleline(&mut tmp_connection.password);
    });

    ui.horizontal(|ui| {
        ui.label(&i18n.connection_dbname);
        ui.text_edit_singleline(&mut tmp_connection.dbname);
    });

    ui.horizontal(|ui| {
        if ui.button(&i18n.edit_connection_cancel).clicked() {
            ui.close_menu();
        }
        if ui.button(&i18n.edit_connection_confirm).clicked() {
            match idx {
                Some(idx) => {
                    let _ = tx.send(ClickHouseMessage::EditConnection((
                        idx,
                        tmp_connection.clone(),
                    )));
                }
                _ => {
                    let _ = tx.send(ClickHouseMessage::AddConnection(tmp_connection.clone()));
                }
            }

            *tmp_connection = ClickHouseConnectionDefinition::default();
            ui.close_menu();
        }
    });
}
