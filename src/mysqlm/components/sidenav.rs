// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use crate::{
    mysqlm::{
        components::contextual_menus::TableInfo,
        presenter,
        state::{MySqlAppState, MySqlState},
    },
    sqlx_common::{
        components::context_menus::TableContextMenu,
        state::{QuerySort, SqlConnectionDefinition, SqlxMessage},
    },
    common::internationalization::I18n,
};
use eframe::egui;
use egui_extras::{Size, StripBuilder};
use std::collections::{HashMap, HashSet};
use tokio::{runtime::Runtime, sync::mpsc::Sender};

pub struct MySqlSideNav;

impl MySqlSideNav {
    pub fn show(
        ctx: &egui::Context,
        rt: &Runtime,
        tx: &Sender<SqlxMessage>,
        tx_sync: &std::sync::mpsc::Sender<SqlxMessage>,
        mysql_app_state: &mut MySqlAppState,
        mysql_local_st: &mut MySqlState,
        i18n: &I18n,
    ) {
        if mysql_app_state.show_sidebar {
            egui::SidePanel::left("mysql_connections_panel").show(ctx, |ui| {
                // --> Decidimos qué mostrar <--
                ui.horizontal(|ui| {
                    let s1 = if mysql_local_st.sql.hide_connections {
                        "\u{229e}"
                    } else {
                        "\u{229f}"
                    };
                    let s2 = if mysql_local_st.sql.hide_tables {
                        "\u{229e}"
                    } else {
                        "\u{229f}"
                    };

                    if ui.button(format!("{s1} Connections")).clicked() {
                        mysql_local_st.sql.hide_connections = !mysql_local_st.sql.hide_connections;
                    }
                    if ui.button(format!("{s2} Tables")).clicked() {
                        mysql_local_st.sql.hide_tables = !mysql_local_st.sql.hide_tables;
                    }
                });

                // --> Abrimos ventana para definir conexión <--
                ui.menu_button(&i18n.pg_btn_add_connection, |ui| {
                    ui.set_min_width(200.0);

                    ui.horizontal(|ui| {
                        ui.label(&i18n.pg_connection_host);
                        ui.text_edit_singleline(&mut mysql_local_st.tmp_pg_connection.host);
                    });

                    ui.horizontal(|ui| {
                        ui.label(&i18n.pg_connection_port);
                        ui.text_edit_singleline(&mut mysql_local_st.tmp_pg_connection.port);
                    });

                    ui.horizontal(|ui| {
                        ui.label(&i18n.pg_connection_user);
                        ui.text_edit_singleline(&mut mysql_local_st.tmp_pg_connection.user);
                    });

                    ui.horizontal(|ui| {
                        ui.label(&i18n.pg_connection_password);
                        ui.text_edit_singleline(&mut mysql_local_st.tmp_pg_connection.password);
                    });

                    ui.horizontal(|ui| {
                        ui.label(&i18n.pg_connection_dbname);
                        ui.text_edit_singleline(&mut mysql_local_st.tmp_pg_connection.dbname);
                    });

                    ui.horizontal(|ui| {
                        if ui.button(&i18n.kafka_edit_cluster_cancel).clicked() {
                            ui.close_menu();
                        }
                        if ui.button(&i18n.kafka_edit_cluster_save).clicked() {
                            // TODO: Añadir al listado
                            mysql_app_state
                                .connections
                                .push(mysql_local_st.tmp_pg_connection.clone());
                            mysql_local_st.tmp_pg_connection = SqlConnectionDefinition::default();
                            ui.close_menu();
                        }
                    });
                });

                // --> Añadimos archivo a conexiones si no está ya incluido <--

                // --> Mostramos Conexiones <--
                if !mysql_local_st.sql.hide_connections && !mysql_local_st.sql.hide_tables {
                    StripBuilder::new(ui)
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .vertical(|mut strip| {
                            strip.cell(|ui| {
                                MySqlConnectionsSubpanel::show(
                                    rt,
                                    ui,
                                    mysql_app_state,
                                    mysql_local_st,
                                    i18n,
                                );
                            });
                            strip.cell(|ui| {
                                ui.vertical_centered(|ui| {
                                    ui.separator();
                                    MySqlTablesSubpanel::show(
                                        rt,
                                        ui,
                                        tx,
                                        tx_sync,
                                        mysql_local_st,
                                        i18n,
                                    );
                                });
                            });
                        });
                } else if !mysql_local_st.sql.hide_connections {
                    MySqlConnectionsSubpanel::show(
                        rt,
                        ui,
                        mysql_app_state,
                        mysql_local_st,
                        i18n,
                    );
                } else if !mysql_local_st.sql.hide_tables {
                    MySqlTablesSubpanel::show(
                        rt,
                        ui,
                        tx,
                        tx_sync,
                        mysql_local_st,
                        i18n,
                    );
                }
            });
        }
    }
}

pub struct MySqlConnectionsSubpanel;

impl MySqlConnectionsSubpanel {
    pub fn show(
        rt: &Runtime,
        ui: &mut egui::Ui,
        pg_app_state: &mut MySqlAppState,
        local_state: &mut MySqlState,
        _i18n: &I18n,
    ) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            let mut connections_to_delete: HashSet<usize> = HashSet::new();

            for (idx, conn_definition) in pg_app_state.connections.iter_mut().enumerate() {
                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    ui.set_width(ui.available_width());
                    let button_text = format!(
                        "{}:{}\n{} / {}",
                        conn_definition.host.clone(),
                        conn_definition.port.clone(),
                        conn_definition.dbname.clone(),
                        conn_definition.user.clone()
                    );
                    let raw_button = if local_state.pool.is_some() {
                        egui::Button::new(button_text)
                            .min_size(egui::vec2(200.0, 24.0))
                            .stroke(if idx == local_state.sql.current_connection_idx {
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
                        if ui.button("Close Connection").clicked() {
                            close_connection(rt, local_state);
                            local_state.sql.current_connection_idx = usize::MAX;
                            ui.close_menu();
                        }
                        if ui.button("Delete Connection").clicked() {
                            connections_to_delete.insert(idx);
                            // Si la conexión que borramos existe, cerramos antes
                            if local_state.sql.current_connection_idx != idx {
                                close_connection(rt, local_state);
                            }
                            local_state.sql.current_connection_idx = usize::MAX;
                            ui.close_menu();
                        }
                    });

                    // --> Al clicar sobre conexión, conectamos y listamos tablas <--
                    // Si estamos ya mostrando esta conexión, clicar sobre ella no lanza ninguna acción.
                    if button.clicked() && local_state.sql.current_connection_idx != idx {
                        local_state.sql.current_connection_idx = idx;
                        // Este método pone `pool` a `None`.
                        close_connection(rt, local_state);

                        // Si no conexión o la que existe no es la que clico, la defino
                        if local_state.pool.is_none() {
                            let conn = SqlConnectionDefinition {
                                host: conn_definition.host.clone(),
                                port: conn_definition.port.clone(),
                                user: conn_definition.user.clone(),
                                password: conn_definition.password.clone(),
                                dbname: conn_definition.dbname.clone(),
                            };
                            local_state.current_connection = conn.clone();
                            local_state.pool = rt
                                .block_on(async move { presenter::connect(conn).await })
                                .ok();
                            if local_state.pool.is_some() {
                                let name1 = conn_definition.dbname.as_str();
                                let pool_ref = local_state.pool.as_ref().unwrap().clone();
                                let pool_ref2 = local_state.pool.as_ref().unwrap().clone();

                                local_state.sql.tables = rt.block_on(async move {
                                    presenter::list_connection_tables(&pool_ref, name1).await
                                });

                                let tables = local_state.sql.tables.clone();
                                local_state.sql.current_connection_tables_info =
                                    rt.block_on(async move {
                                        presenter::tables_info(
                                            &pool_ref2,
                                            conn_definition.dbname.as_str(),
                                            tables.as_ref(),
                                        )
                                        .await
                                        .map_or(HashMap::new(), |v| v)
                                    });
                            }
                        }
                    }
                });
            }

            if !connections_to_delete.is_empty() {
                let mut i = 0;
                let mut to_retain: Vec<SqlConnectionDefinition> = Vec::new();
                while i < pg_app_state.connections.len() {
                    if !connections_to_delete.contains(&i) {
                        to_retain.push(pg_app_state.connections.get(i).unwrap().clone());
                    }
                    i += 1;
                }
                pg_app_state.connections = to_retain;
            }
        });
    }
}

pub struct MySqlTablesSubpanel;

impl MySqlTablesSubpanel {
    pub fn show(
        rt: &Runtime,
        ui: &mut egui::Ui,
        tx: &Sender<SqlxMessage>,
        tx_sync: &std::sync::mpsc::Sender<SqlxMessage>,
        local_state: &mut MySqlState,
        i18n: &I18n,
    ) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("mysql_db_tables")
                .num_columns(2)
                .show(ui, |ui| {
                    for (t_idx, t_name) in local_state.sql.tables.clone().iter().enumerate() {
                        ui.label(
                            egui::RichText::new("Info")
                                .color(egui::Color32::from_rgb(128, 128, 128)),
                        )
                        .on_hover_ui(|ui| {
                            TableInfo::show(ui, &local_state.sql, t_name);
                        });

                        let table_btn = ui.selectable_value(
                            &mut local_state.sql.current_table_idx,
                            t_idx,
                            t_name,
                        );

                        // Sin problemas, para que esto se muestre tiene que existir conexión.
                        table_btn.context_menu(|ui| {
                            TableContextMenu::show(
                                // rt,
                                // &pool_ref.clone(),
                                ui,
                                tx_sync,
                                &mut local_state.sql,
                                i18n,
                                t_name,
                            );
                        });

                        if table_btn.clicked() {
                            let pool_ref = local_state.pool.as_ref().unwrap().clone();
                            let tx_cloned = tx.clone();
                            let t_name_string = t_name.to_string();
                            rt.spawn(async move {
                                presenter::select_all_with_column_description(
                                    &pool_ref,
                                    &tx_cloned,
                                    &t_name_string,
                                    QuerySort::None,
                                )
                                .await
                            });
                            // Para desmarcar orden de búsqueda.
                            local_state.sql.query_sort = QuerySort::None;
                            local_state.sql.sql_statement = format!("SELECT * FROM {}", t_name);
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
fn close_connection(rt: &Runtime, local_state: &mut MySqlState) {
    // Usar `guard` facilita mucho porque take sobre referencia no puede usarse,
    // y usar is_some y dentro hacer algo genera problemas de prestado de
    // referencia.
    if local_state.pool.is_none() {
        return;
    }
    let pool_cloned = local_state.pool.as_ref().unwrap();
    // local_state.current_connection.path = String::default();

    // Bloqueo para asegurar que todo cerrado antes de reconectar. Puedo
    // de todas formas lanzar con `spawn` sin problemas.
    rt.block_on(async move {
        pool_cloned.close().await;
    });

    local_state.pool = None;
}
