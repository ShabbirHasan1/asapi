// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use eframe::egui;
use egui_extras::{Size, StripBuilder};
use std::collections::{HashMap, HashSet};
use tokio::{runtime::Runtime, sync::mpsc::Sender};

use crate::{
    common::internationalization::I18nSqlx,
    mysqlm::{
        components::contextual_menus::TableInfo,
        presenter,
        state::{MySqlAppState, MySqlState},
    },
    sqlx_common::{
        components::context_menus::TableContextMenu,
        state::{QuerySort, SqlConnectionDefinition, SqlxMessage},
    },
};

pub struct MySqlSideNav {
    connections_subpanel: MySqlConnectionsSubpanel,
}

impl Default for MySqlSideNav {
    fn default() -> Self {
        MySqlSideNav {
            connections_subpanel: MySqlConnectionsSubpanel {
                is_edit_connection_menu_opened: false,
            },
        }
    }
}

impl MySqlSideNav {
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        rt: &Runtime,
        tx: &Sender<SqlxMessage>,
        tx_sync: &std::sync::mpsc::Sender<SqlxMessage>,
        app_st: &mut MySqlAppState,
        local_st: &mut MySqlState,
        i18n: &I18nSqlx,
    ) {
        if app_st.show_sidebar {
            egui::SidePanel::left("mysql_connections_panel").show(ctx, |ui| {
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
                ui.menu_button(&i18n.pg.btn_add_connection, |ui| {
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
                                    .show(rt, tx_sync, ui, app_st, local_st, i18n);
                            });
                            strip.cell(|ui| {
                                ui.vertical_centered(|ui| {
                                    ui.separator();
                                    MySqlTablesSubpanel::show(rt, ui, tx, tx_sync, local_st, i18n);
                                });
                            });
                        });
                } else if !local_st.sql.hide_connections {
                    self.connections_subpanel
                        .show(rt, tx_sync, ui, app_st, local_st, i18n);
                } else if !local_st.sql.hide_tables {
                    MySqlTablesSubpanel::show(rt, ui, tx, tx_sync, local_st, i18n);
                }
            });
        }
    }
}

pub struct MySqlConnectionsSubpanel {
    is_edit_connection_menu_opened: bool,
}

impl MySqlConnectionsSubpanel {
    pub fn show(
        &mut self,
        rt: &Runtime,
        tx_sync: &std::sync::mpsc::Sender<SqlxMessage>,
        ui: &mut egui::Ui,
        app_st: &mut MySqlAppState,
        local_st: &mut MySqlState,
        i18n: &I18nSqlx,
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
                        if ui.button(&i18n.mysql.close_connection).clicked() {
                            close_connection(rt, local_st);
                            local_st.sql.current_connection_idx = usize::MAX;
                            ui.close_menu();
                        }
                        if ui.button(&i18n.mysql.delete_connection).clicked() {
                            connections_to_delete.insert(idx);
                            // Si la conexión que borramos existe, cerramos antes
                            if local_st.sql.current_connection_idx != idx {
                                close_connection(rt, local_st);
                            }
                            local_st.sql.current_connection_idx = usize::MAX;
                            ui.close_menu();
                        }

                        let mut menu_open = false;
                        ui.menu_button(&i18n.mysql.edit_connection, |ui| {
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

                        if ui.button(&i18n.mysql.reload_tables).clicked() {
                            let pool_ref = local_st.pool.as_ref().unwrap().clone();
                            let pool_ref_2 = local_st.pool.as_ref().unwrap().clone();
                            let db_name = conn_definition.dbname.clone();

                            local_st.sql.tables = rt.block_on(async move {
                                presenter::list_connection_tables(&pool_ref, &db_name).await
                            });

                            let tables = local_st.sql.tables.clone();
                            let db_name2 = conn_definition.dbname.clone();

                            local_st.sql.current_connection_tables_info = rt.block_on(async move {
                                presenter::tables_info(&pool_ref_2, &db_name2, tables.as_ref())
                                    .await
                                    .map_or(HashMap::new(), |v| v)
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
                        local_st.sql.current_table_idx = usize::MAX;
                        local_st.sql.sql_statement.clear();

                        // Si no conexión o la que existe no es la que clico, la defino
                        if local_st.pool.is_none() {
                            let conn = SqlConnectionDefinition {
                                name: conn_definition.name.clone(),
                                host: conn_definition.host.clone(),
                                port: conn_definition.port.clone(),
                                user: conn_definition.user.clone(),
                                password: conn_definition.password.clone(),
                                dbname: conn_definition.dbname.clone(),
                            };
                            local_st.current_connection = conn.clone();
                            let result = rt.block_on(async move { presenter::connect(conn).await });

                            match result {
                                Ok(pool) => {
                                    local_st.pool = Some(pool);
                                }
                                Err(err) => {
                                    // No hace falta poner `local_st.pool` a `None` porque en `close_connection`
                                    // ya lo estamos haciendo.
                                    local_st.sql.last_response_error =
                                        Some(Err(format!("{err:?}")));
                                }
                            }
                            if local_st.pool.is_some() {
                                let name1 = conn_definition.dbname.as_str();
                                let pool_ref = local_st.pool.as_ref().unwrap().clone();
                                let pool_ref2 = local_st.pool.as_ref().unwrap().clone();

                                local_st.sql.tables = rt.block_on(async move {
                                    presenter::list_connection_tables(&pool_ref, name1).await
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

pub struct MySqlTablesSubpanel;

impl MySqlTablesSubpanel {
    pub fn show(
        rt: &Runtime,
        ui: &mut egui::Ui,
        tx: &Sender<SqlxMessage>,
        tx_sync: &std::sync::mpsc::Sender<SqlxMessage>,
        local_st: &mut MySqlState,
        i18n: &I18nSqlx,
    ) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("mysql_db_tables")
                .num_columns(2)
                .show(ui, |ui| {
                    for (t_idx, t_name) in local_st.sql.tables.clone().iter().enumerate() {
                        ui.label(
                            egui::RichText::new("Info")
                                .color(egui::Color32::from_rgb(128, 128, 128)),
                        )
                        .on_hover_ui(|ui| {
                            TableInfo::show(ui, &local_st.sql, t_name);
                        });

                        let table_btn = ui.selectable_value(
                            &mut local_st.sql.current_table_idx,
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
                                &mut local_st.sql,
                                i18n,
                                t_name,
                            );
                        });

                        if table_btn.clicked() {
                            local_st.sql.reset();
                            let pool_ref = local_st.pool.as_ref().unwrap().clone();
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
                            local_st.sql.query_sort = QuerySort::None;
                            local_st.sql.sql_statement = format!("SELECT * FROM {}", t_name);
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

fn edit_connection(
    ui: &mut egui::Ui,
    i18n: &I18nSqlx,
    tmp_connection: &mut SqlConnectionDefinition,
    tx: &std::sync::mpsc::Sender<SqlxMessage>,
    idx: Option<usize>,
) {
    ui.set_min_width(200.0);

    ui.horizontal(|ui| {
        ui.label(&i18n.pg.connection_name);
        ui.text_edit_singleline(&mut tmp_connection.name);
    });

    ui.horizontal(|ui| {
        ui.label(&i18n.pg.connection_host);
        ui.text_edit_singleline(&mut tmp_connection.host);
    });

    ui.horizontal(|ui| {
        ui.label(&i18n.pg.connection_port);
        ui.text_edit_singleline(&mut tmp_connection.port);
    });

    ui.horizontal(|ui| {
        ui.label(&i18n.pg.connection_user);
        ui.text_edit_singleline(&mut tmp_connection.user);
    });

    ui.horizontal(|ui| {
        ui.label(&i18n.pg.connection_password);
        ui.text_edit_singleline(&mut tmp_connection.password);
    });

    ui.horizontal(|ui| {
        ui.label(&i18n.pg.connection_dbname);
        ui.text_edit_singleline(&mut tmp_connection.dbname);
    });

    ui.horizontal(|ui| {
        if ui.button(&i18n.pg.edit_connection_cancel).clicked() {
            ui.close_menu();
        }
        if ui.button(&i18n.pg.edit_connection_confirm).clicked() {
            match idx {
                Some(idx) => {
                    let _ = tx.send(SqlxMessage::EditConnection((idx, tmp_connection.clone())));
                }
                _ => {
                    let _ = tx.send(SqlxMessage::AddConnection(tmp_connection.clone()));
                }
            }

            *tmp_connection = SqlConnectionDefinition::default();
            ui.close_menu();
        }
    });
}
