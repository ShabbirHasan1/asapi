// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use common::{fs, I18nSqlx};

use crate::{
    sqlitem::{
        components::contextual_menus::TableInfo,
        presenter,
        state::{SQLiteAppState, SQLiteConnectionDefinition, SQLiteState},
    },
    sqlx_common::{
        components::context_menus::TableContextMenu,
        state::{QuerySort, SqlConnectionDefinition, SqlxMessage},
    },
};
use eframe::egui;
use egui_extras::{Size, StripBuilder};
use std::collections::HashSet;
use tokio::{runtime::Runtime, sync::mpsc::Sender};

pub struct SQLiteSideNav {
    connections_subpanel: SQLiteConnectionsSubpanel,
}

impl Default for SQLiteSideNav {
    fn default() -> Self {
        SQLiteSideNav {
            connections_subpanel: SQLiteConnectionsSubpanel {
                is_edit_connection_menu_opened: false,
            },
        }
    }
}

impl SQLiteSideNav {
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        rt: &Runtime,
        tx: &Sender<SqlxMessage>,
        tx_sync: &std::sync::mpsc::Sender<SqlxMessage>,
        sqlite_app_state: &mut SQLiteAppState,
        local_state: &mut SQLiteState,
        i18n: &I18nSqlx,
    ) {
        if sqlite_app_state.show_sidebar {
            egui::SidePanel::left("sqlite_connections_panel").show(ctx, |ui| {
                // --> Abrimos archivo sqlite y conectamos <--
                let _hover_menu = |ui: &mut egui::Ui| {
                    ui.label("Para conectar, clicar en definición de la conexión.");
                };
                let add_contents = |ui: &mut egui::Ui| {
                    ui.label(&i18n.sqlite.connection_btn_help);
                };
                if ui
                    .button(&i18n.sqlite.btn_add_connection)
                    .on_hover_ui(add_contents)
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

                    if ui.button(format!("{s1} {}", i18n.connections)).clicked() {
                        local_state.sql.hide_connections = !local_state.sql.hide_connections;
                    }

                    if ui.button(format!("{s2} {}", i18n.tables)).clicked() {
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
                        name: p.to_string(),
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
                                self.connections_subpanel.show(
                                    ctx,
                                    rt,
                                    tx_sync,
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
                                        local_state,
                                        i18n,
                                    );
                                });
                            });
                        });
                } else if !local_state.sql.hide_connections {
                    self.connections_subpanel.show(
                        ctx,
                        rt,
                        tx_sync,
                        ui,
                        sqlite_app_state,
                        local_state,
                        i18n,
                    );
                } else if !local_state.sql.hide_tables {
                    SQLiteTablesSubpanel::show(ctx, rt, ui, tx, tx_sync, local_state, i18n);
                }
            });
        }
    }
}

pub struct SQLiteConnectionsSubpanel {
    is_edit_connection_menu_opened: bool,
}

impl SQLiteConnectionsSubpanel {
    pub fn show(
        &mut self,
        _ctx: &egui::Context,
        rt: &Runtime,
        tx_sync: &std::sync::mpsc::Sender<SqlxMessage>,
        ui: &mut egui::Ui,
        app_st: &mut SQLiteAppState,
        local_st: &mut SQLiteState,
        i18n: &I18nSqlx,
    ) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            let mut connections_to_delete: HashSet<usize> = HashSet::new();

            for (idx, conn_definition) in app_st.connections.iter_mut().enumerate() {
                let sqlite_db_file_name = conn_definition
                    .path
                    .rfind('/')
                    .map_or(conn_definition.path.as_ref(), |last_slash_idx| {
                        &conn_definition.path.split_at(last_slash_idx).1[1..]
                    });

                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    ui.set_width(ui.available_width());

                    let button = ui
                        .add(
                            egui::Button::new(format!(
                                "Connection Name: {}\nFile Name: {}",
                                conn_definition.name, sqlite_db_file_name
                            ))
                            .min_size(egui::vec2(200.0, 24.0))
                            .stroke(
                                if idx == local_st.sql.current_connection_idx {
                                    egui::Stroke::new(1.0, egui::Color32::DARK_BLUE)
                                } else {
                                    egui::Stroke::new(0.0, egui::Color32::LIGHT_BLUE)
                                },
                            ),
                        )
                        .on_hover_ui(|ui| {
                            ui.label(format!("Path: {}", conn_definition.path));
                        });

                    // --> Menú contextual para manejo de las conexiones <--
                    button.context_menu(|ui| {
                        if ui.button(&i18n.sqlite.close_connection).clicked() {
                            close_connection(rt, local_st);
                            local_st.sql.current_connection_idx = usize::MAX;
                            ui.close_menu();
                        }
                        if ui.button(&i18n.sqlite.delete_connection).clicked() {
                            connections_to_delete.insert(idx);
                            // Si la conexión que borramos existe, cerramos
                            if conn_definition.path == local_st.current_connection.path {
                                close_connection(rt, local_st);
                            }
                            local_st.sql.current_connection_idx = usize::MAX;
                            ui.close_menu();
                        }

                        let mut menu_open = false;
                        ui.menu_button(&i18n.sqlite.edit_connection, |ui| {
                            menu_open = true;
                            if menu_open && !self.is_edit_connection_menu_opened {
                                local_st.tmp_connection = SqlConnectionDefinition {
                                    name: conn_definition.name.clone(),
                                    host: Default::default(),
                                    port: Default::default(),
                                    user: Default::default(),
                                    password: Default::default(),
                                    dbname: Default::default(),
                                }
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

                        if ui.button(&i18n.pg.reload_tables).clicked() {
                            local_st.connect_to_file = true;
                        }
                    });

                    // Si estamos ya mostrando esta conexión, clicar sobre ella no lanza ninguna acción.
                    if button.clicked() && local_st.sql.current_connection_idx != idx {
                        local_st.sql.current_connection_idx = idx;
                        close_connection(rt, local_st);
                        let file_path = conn_definition.path.clone();

                        if (local_st.pool.is_none()
                            || file_path != local_st.current_connection.path)
                            && fs::file_exists(&file_path)
                        {
                            local_st.current_connection = SQLiteConnectionDefinition {
                                name: conn_definition.name.clone(),
                                path: file_path,
                            };
                            local_st.connect_to_file = true;
                        }
                    }
                });
            }

            if !connections_to_delete.is_empty() {
                let mut i = 0;
                let mut to_retain: Vec<SQLiteConnectionDefinition> = Vec::new();
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

pub struct SQLiteTablesSubpanel;
impl SQLiteTablesSubpanel {
    pub fn show(
        _ctx: &egui::Context,
        rt: &Runtime,
        ui: &mut egui::Ui,
        tx: &Sender<SqlxMessage>,
        tx_sync: &std::sync::mpsc::Sender<SqlxMessage>,
        local_st: &mut SQLiteState,
        i18n: &I18nSqlx,
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
                                    QuerySort::None,
                                )
                                .await
                            });
                            // Para desmarcar orden de búsqueda.
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

            tmp_connection.name.clear();
            ui.close_menu();
        }
    });
}
