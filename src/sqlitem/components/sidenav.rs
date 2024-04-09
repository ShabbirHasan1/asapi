// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use crate::{
    sqlitem::{
        components::contextual_menus::TableInfo,
        presenter,
        state::{SQLiteAppState, SQLiteConnectionDefinition, SQLiteState},
    },
    sqlx_common::{
        components::context_menus::TableContextMenu,
        state::{QuerySort, SqlxMessage},
    },
    common::internationalization::I18n,
};
use eframe::egui;
use egui_extras::{Size, StripBuilder};
use std::collections::HashSet;
use tokio::{runtime::Runtime, sync::mpsc::Sender};

pub struct SQLiteSideNav;

impl SQLiteSideNav {
    pub fn show(
        ctx: &egui::Context,
        rt: &Runtime,
        tx: &Sender<SqlxMessage>,
        tx_sync: &std::sync::mpsc::Sender<SqlxMessage>,
        sqlite_app_state: &mut SQLiteAppState,
        local_state: &mut SQLiteState,
        i18n: &I18n,
    ) {
        if sqlite_app_state.show_sidebar {
            egui::SidePanel::left("sqlite_connections_panel").show(ctx, |ui| {
                // --> Abrimos archivo sqlite y conectamos <--
                if ui
                    .button(&i18n.sqlite_btn_add_connection)
                    .on_hover_ui(|ui| {
                        ui.label("Para conectar, clicar en definición de la conexión");
                    })
                    .clicked()
                {
                    local_state.file_dialog.select_file();
                }

                // --> Decidimos qué mostrar <--
                ui.horizontal(|ui| {
                    let s1 = if local_state.sql.hide_connections {
                        "\u{229e}"
                    } else {
                        "\u{229f}"
                    };
                    let s2 = if local_state.sql.hide_tables {
                        "\u{229e}"
                    } else {
                        "\u{229f}"
                    };

                    if ui.button(format!("{s1} Connections")).clicked() {
                        local_state.sql.hide_connections = !local_state.sql.hide_connections;
                    }
                    if ui.button(format!("{s2} Tables")).clicked() {
                        local_state.sql.hide_tables = !local_state.sql.hide_tables;
                    }
                });

                // --> Procesamos `FileDialog` para extrar ruta del archivo.
                // `Update` es el método que se encarga de abrir/mantener abierto el diálogo
                // entre frames cuando su estado es `DialogOpened`, lo que ocurre en el
                // `select_file` anterior.
                // Por cómo trabaja FileDialog, es mucho más fácil crear al abrir y luego
                // conectar que no conectar nada más abrir.
                local_state.file_dialog.update(ctx);
                let opt_conn_definition = local_state.file_dialog.selected().and_then(|path| {
                    path.to_str().map(|p| SQLiteConnectionDefinition {
                        path: p.to_string(),
                    })
                });
                // --> Añadimos archivo a conexiones si no está ya incluido <--
                if let Some(conn_definition) = opt_conn_definition {
                    if sqlite_app_state
                        .connections
                        .iter()
                        .all(|s| s.path != conn_definition.path)
                    {
                        sqlite_app_state.connections.push(conn_definition);
                    }
                }

                // --> Mostramos Conexiones <--
                if !local_state.sql.hide_connections && !local_state.sql.hide_tables {
                    StripBuilder::new(ui)
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .vertical(|mut strip| {
                            strip.cell(|ui| {
                                SQLiteConnectionsSubpanel::show(
                                    ctx,
                                    rt,
                                    ui,
                                    sqlite_app_state,
                                    local_state,
                                    i18n,
                                );
                            });
                            strip.cell(|ui| {
                                ui.vertical_centered(|ui| {
                                    ui.separator();
                                    SQLiteTablesSubpanel::show(
                                        ctx,
                                        rt,
                                        ui,
                                        tx,
                                        tx_sync,
                                        sqlite_app_state,
                                        local_state,
                                        i18n,
                                    );
                                });
                            });
                        });
                } else if !local_state.sql.hide_connections {
                    SQLiteConnectionsSubpanel::show(
                        ctx,
                        rt,
                        ui,
                        sqlite_app_state,
                        local_state,
                        i18n,
                    );
                } else if !local_state.sql.hide_tables {
                    SQLiteTablesSubpanel::show(
                        ctx,
                        rt,
                        ui,
                        tx,
                        tx_sync,
                        sqlite_app_state,
                        local_state,
                        i18n,
                    );
                }
            });
        }
    }
}

pub struct SQLiteConnectionsSubpanel;

impl SQLiteConnectionsSubpanel {
    pub fn show(
        _ctx: &egui::Context,
        rt: &Runtime,
        ui: &mut egui::Ui,
        sqlite_app_state: &mut SQLiteAppState,
        local_state: &mut SQLiteState,
        _i18n: &I18n,
    ) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            let mut connections_to_delete: HashSet<usize> = HashSet::new();

            for (idx, conn_definition) in sqlite_app_state.connections.iter_mut().enumerate() {
                let sqlite_db_file_name = conn_definition
                    .path
                    .rfind("/")
                    .map_or(conn_definition.path.as_ref(), |last_slash_idx| {
                        &conn_definition.path.split_at(last_slash_idx).1[1..]
                    });

                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    ui.set_width(ui.available_width());

                    let button = ui
                        .add(
                            egui::Button::new(sqlite_db_file_name)
                                .min_size(egui::vec2(200.0, 24.0))
                                .stroke(if idx == local_state.sql.current_connection_idx {
                                    egui::Stroke::new(1.0, egui::Color32::DARK_BLUE)
                                } else {
                                    egui::Stroke::new(0.0, egui::Color32::LIGHT_BLUE)
                                }),
                        )
                        .on_hover_ui(|ui| {
                            ui.label(&conn_definition.path);
                        });

                    // --> Menú contextual para manejo de las conexiones <--
                    button.context_menu(|ui| {
                        if ui.button("Close Connection").clicked() {
                            close_connection(rt, local_state);
                            local_state.sql.current_connection_idx = usize::MAX;
                            ui.close_menu();
                        }
                        if ui.button("Delete Connection").clicked() {
                            connections_to_delete.insert(idx);
                            // Si la conexión que borramos existe, cerramos
                            if conn_definition.path == local_state.current_connection.path {
                                close_connection(rt, local_state);
                            }
                            local_state.sql.current_connection_idx = usize::MAX;
                            ui.close_menu();
                        }
                    });

                    // Si estamos ya mostrando esta conexión, clicar sobre ella no lanza ninguna acción.
                    if button.clicked() && local_state.sql.current_connection_idx != idx {
                        local_state.sql.current_connection_idx = idx;
                        close_connection(rt, local_state);
                        let file_path = conn_definition.path.clone();

                        if local_state.pool.is_none()
                            || file_path != local_state.current_connection.path
                        {
                            local_state.current_connection =
                                SQLiteConnectionDefinition { path: file_path };
                            local_state.connect_to_file = true;
                        }
                    }
                });
            }

            if !connections_to_delete.is_empty() {
                let mut i = 0;
                let mut to_retain: Vec<SQLiteConnectionDefinition> = Vec::new();
                while i < sqlite_app_state.connections.len() {
                    if !connections_to_delete.contains(&i) {
                        to_retain.push(sqlite_app_state.connections.get(i).unwrap().clone());
                    }
                    i += 1;
                }
                sqlite_app_state.connections = to_retain;
            }
        });
    }
}

pub struct SQLiteTablesSubpanel;
impl SQLiteTablesSubpanel {
    pub fn show(
        _ctx: &egui::Context,
        rt: &Runtime,
        ui: &mut egui::Ui,
        tx: &Sender<SqlxMessage>,
        tx_sync: &std::sync::mpsc::Sender<SqlxMessage>,
        _app_st: &mut SQLiteAppState,
        local_st: &mut SQLiteState,
        i18n: &I18n,
    ) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("sqlite_db_tables")
                .num_columns(2)
                .show(ui, |ui| {
                    // egui::ScrollArea::vertical().show(ui, |ui| {
                    for (table_idx, table_name) in local_st.sql.tables.clone().iter().enumerate() {
                        ui.label(
                            egui::RichText::new("Info")
                                .color(egui::Color32::from_rgb(128, 128, 128)),
                        )
                        .on_hover_ui(|ui| {
                            TableInfo::show(ui, &local_st.sql, table_name);
                        });

                        let table_btn = ui.selectable_value(
                            &mut local_st.sql.current_table_idx,
                            table_idx,
                            table_name,
                        );

                        table_btn.context_menu(|ui| {
                            TableContextMenu::show(
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
                            rt.spawn(async move {
                                presenter::select_all_with_column_description(
                                    &pool_ref,
                                    &tx_cloned,
                                    &t_name,
                                    QuerySort::NONE,
                                )
                                .await
                            });
                            // Para desmarcar orden de búsqueda.
                            local_st.sql.query_sort = QuerySort::NONE;
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
fn close_connection(rt: &Runtime, local_state: &mut SQLiteState) {
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
