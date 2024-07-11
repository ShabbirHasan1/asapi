// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use eframe::egui;
use egui_extras::{Size, StripBuilder};
use egui_json_tree::JsonTree;
use std::collections::HashSet;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::Sender;

use common::internationalization::I18n;
use components::toggle_switch::toggle;

use crate::{
    connection::{close_connection, connect_with_default},
    presenter::{self, list_database_collections, list_database_names_in_connection},
    state::{MongoAppState, MongoConnectionDefinition, MongoError, MongoLocalState, MongoMessage},
};

pub struct MongoSideNav {
    connections_subpanel: MongoConnectionsSubpanel,
}

impl Default for MongoSideNav {
    fn default() -> Self {
        MongoSideNav {
            connections_subpanel: MongoConnectionsSubpanel {
                edit_menu_open: false,
            },
        }
    }
}

impl MongoSideNav {
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        rt: &Runtime,
        tx: &Sender<MongoMessage>,
        app_st: &mut MongoAppState,
        local_st: &mut MongoLocalState,
        i18n: &I18n,
    ) {
        if app_st.show_sidebar {
            egui::SidePanel::left("mongo_sidenav_panel").show(ctx, |ui| {
                // --> Decidimos qué mostrar <--
                ui.horizontal(|ui| {
                    let s1 = if local_st.hide_connections {
                        "\u{229e}"
                    } else {
                        "\u{229f}"
                    };
                    let s2 = if local_st.hide_databases {
                        "\u{229e}"
                    } else {
                        "\u{229f}"
                    };
                    let s3 = if local_st.hide_collections {
                        "\u{229e}"
                    } else {
                        "\u{229f}"
                    };

                    if ui
                        .button(format!("{s1} {}", i18n.mongo_connections))
                        .clicked()
                    {
                        local_st.hide_connections = !local_st.hide_connections;
                    }
                    if ui
                        .button(format!("{s2} {}", i18n.mongo_databases))
                        .clicked()
                    {
                        local_st.hide_databases = !local_st.hide_databases;
                    }
                    if ui
                        .button(format!("{s3} {}", i18n.mongo_collections))
                        .clicked()
                    {
                        local_st.hide_collections = !local_st.hide_collections;
                    }
                });

                // --> Abrimos ventana para definir conexión <--
                ui.menu_button(&i18n.sqlx.pg.btn_add_connection, |ui| {
                    self.connections_subpanel
                        .edit_connection(rt, tx, ui, local_st, None, i18n);
                });

                // --> Mostramos Conexiones <--
                if !local_st.hide_connections
                    && !local_st.hide_databases
                    && !local_st.hide_collections
                {
                    StripBuilder::new(ui)
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .vertical(|mut strip| {
                            strip.cell(|ui| {
                                self.connections_subpanel
                                    .show(ctx, rt, tx, ui, app_st, local_st, i18n);
                            });
                            strip.cell(|ui| {
                                ui.vertical_centered(|ui| {
                                    ui.separator();
                                    MongoDatabasesSubpanel::show(
                                        ctx, rt, tx, ui, app_st, local_st, i18n,
                                    );
                                });
                            });
                            strip.cell(|ui| {
                                ui.vertical_centered(|ui| {
                                    ui.separator();
                                    MongoCollectionsSubpanel::show(
                                        ctx, rt, tx, ui, app_st, local_st, i18n,
                                    );
                                });
                            });
                        });
                } else if local_st.hide_collections
                    && local_st.hide_databases
                    && local_st.hide_connections
                {
                    // No mostramos nada.
                } else if local_st.hide_collections && local_st.hide_databases {
                    self.connections_subpanel
                        .show(ctx, rt, tx, ui, app_st, local_st, i18n);
                } else if local_st.hide_connections && local_st.hide_databases {
                    MongoCollectionsSubpanel::show(ctx, rt, tx, ui, app_st, local_st, i18n);
                } else if local_st.hide_connections && local_st.hide_collections {
                    MongoDatabasesSubpanel::show(ctx, rt, tx, ui, app_st, local_st, i18n);
                } else if local_st.hide_collections {
                    StripBuilder::new(ui)
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .vertical(|mut strip| {
                            strip.cell(|ui| {
                                self.connections_subpanel
                                    .show(ctx, rt, tx, ui, app_st, local_st, i18n);
                            });
                            strip.cell(|ui| {
                                ui.vertical_centered(|ui| {
                                    ui.separator();
                                    MongoDatabasesSubpanel::show(
                                        ctx, rt, tx, ui, app_st, local_st, i18n,
                                    );
                                });
                            });
                        });
                } else if local_st.hide_databases {
                    StripBuilder::new(ui)
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .vertical(|mut strip| {
                            strip.cell(|ui| {
                                self.connections_subpanel
                                    .show(ctx, rt, tx, ui, app_st, local_st, i18n);
                            });
                            strip.cell(|ui| {
                                ui.vertical_centered(|ui| {
                                    ui.separator();
                                    MongoCollectionsSubpanel::show(
                                        ctx, rt, tx, ui, app_st, local_st, i18n,
                                    );
                                });
                            });
                        });
                } else if local_st.hide_connections {
                    StripBuilder::new(ui)
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .vertical(|mut strip| {
                            strip.cell(|ui| {
                                MongoDatabasesSubpanel::show(
                                    ctx, rt, tx, ui, app_st, local_st, i18n,
                                );
                            });
                            strip.cell(|ui| {
                                ui.vertical_centered(|ui| {
                                    ui.separator();
                                    MongoCollectionsSubpanel::show(
                                        ctx, rt, tx, ui, app_st, local_st, i18n,
                                    );
                                });
                            });
                        });
                }
            });
        }
    }
}

pub struct MongoConnectionsSubpanel {
    edit_menu_open: bool,
}

impl MongoConnectionsSubpanel {
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        rt: &Runtime,
        tx: &Sender<MongoMessage>,
        ui: &mut egui::Ui,
        app_st: &mut MongoAppState,
        local_st: &mut MongoLocalState,
        i18n: &I18n,
    ) {
        egui::ScrollArea::vertical()
            .id_source("connections_scroll_area")
            .show(ui, |ui| {
                let mut connections_to_delete: HashSet<usize> = HashSet::new();

                for (idx, conn_definition) in app_st.connections.iter().enumerate() {
                    ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                        ui.set_width(ui.available_width());
                        let button_text = format!(
                            "{}:{}\n{}, User: {}",
                            &conn_definition.host,
                            &conn_definition.port,
                            &conn_definition.name,
                            &conn_definition.user,
                        );
                        let raw_button = if local_st.conn.client.is_some() {
                            egui::Button::new(button_text)
                                .min_size(egui::vec2(200.0, 24.0))
                                .stroke(if idx == local_st.current_selection.conn_idx {
                                    egui::Stroke::new(1.0, egui::Color32::DARK_BLUE)
                                } else {
                                    egui::Stroke::new(0.0, egui::Color32::RED)
                                })
                        } else {
                            egui::Button::new(button_text).min_size(egui::vec2(200.0, 24.0))
                        };

                        let button = ui.add(raw_button);

                        // --> Menú contextual para manejo de las conexiones <--
                        button.context_menu(|ui| {
                            if ui.button(&i18n.mongo_close_connection).clicked() {
                                close_connection(rt, local_st);
                                local_st.current_selection.conn_idx = usize::MAX;
                                ui.close_menu();
                            }
                            if ui.button(&i18n.mongo_delete_connection).clicked() {
                                connections_to_delete.insert(idx);
                                // Si la conexión que borramos existe, cerramos antes
                                if local_st.current_selection.conn_idx != idx {
                                    close_connection(rt, local_st);
                                }
                                local_st.current_selection.conn_idx = usize::MAX;
                                ui.close_menu();
                            }

                            // --> Editamos <--
                            let mut menu_open = false;
                            ui.menu_button(&i18n.mongo_edit_connection, |ui| {
                                menu_open = true;
                                if menu_open && !self.edit_menu_open {
                                    local_st.tmp_conn_definition = conn_definition.clone();
                                }
                                self.edit_connection(rt, tx, ui, local_st, Some(idx), i18n);
                            });

                            self.edit_menu_open = menu_open;
                        });

                        // --> Al clicar sobre conexión, conectamos y listamos tablas <--
                        // Si estamos ya mostrando esta conexión, clicar sobre ella no lanza ninguna acción.
                        if button.clicked() && local_st.current_selection.conn_idx != idx {
                            local_st.reset();
                            local_st.current_selection.conn_idx = idx;
                            // Este método pone `pool` a `None`.
                            close_connection(rt, local_st);

                            // Si no conexión o la que existe no es la que clico, la defino
                            if local_st.conn.client.is_none() {
                                let conn = MongoConnectionDefinition {
                                    name: conn_definition.name.clone(),
                                    host: conn_definition.host.clone(),
                                    port: conn_definition.port.clone(),
                                    user: conn_definition.user.clone(),
                                    password: conn_definition.password.clone(),
                                    is_srv: conn_definition.is_srv,
                                };
                                local_st.conn.conn_definition = conn.clone();
                                local_st.conn.client = rt.block_on(async move {
                                    match connect_with_default(&conn).await {
                                        Ok(client) => Some(client),
                                        Err(err) => {
                                            let _ = tx.send(MongoMessage::Error(err)).await;
                                            None
                                        }
                                    }
                                });
                                // Si hemos conectado con éxito, mostramos colecciones en la conexión.
                                if local_st.conn.client.is_some() {
                                    log::info!(
                                        "Conectado con éxito a {:?}",
                                        local_st.conn.conn_definition
                                    );
                                    let tx_cloned = tx.clone();
                                    let i18n_cloned = i18n.clone();
                                    let client = local_st.conn.client.as_ref().unwrap().clone();
                                    let ctx_cloned = ctx.clone();

                                    rt.spawn(async move {
                                        list_database_names_in_connection(
                                            &tx_cloned,
                                            &client,
                                            &i18n_cloned,
                                        )
                                        .await;
                                        ctx_cloned.request_repaint();
                                    });
                                }
                            }
                        }
                    });
                }

                if !connections_to_delete.is_empty() {
                    let mut i = 0;
                    let mut to_retain: Vec<MongoConnectionDefinition> = Vec::new();
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

    fn edit_connection(
        &self,
        rt: &Runtime,
        tx: &Sender<MongoMessage>,
        ui: &mut egui::Ui,
        local_st: &mut MongoLocalState,
        idx: Option<usize>,
        i18n: &I18n,
    ) {
        ui.set_min_width(200.0);
        ui.horizontal(|ui| {
            ui.label(&i18n.mongo_connection_name);
            ui.text_edit_singleline(&mut local_st.tmp_conn_definition.name);
        });
        ui.horizontal(|ui| {
            ui.label(&i18n.mongo_connection_host);
            ui.text_edit_singleline(&mut local_st.tmp_conn_definition.host);
        });
        ui.horizontal(|ui| {
            ui.label(&i18n.mongo_connection_port);
            ui.text_edit_singleline(&mut local_st.tmp_conn_definition.port);
        });
        ui.horizontal(|ui| {
            ui.label(&i18n.mongo_connection_user);
            ui.text_edit_singleline(&mut local_st.tmp_conn_definition.user);
        });
        ui.horizontal(|ui| {
            ui.label(&i18n.mongo_connection_password);
            ui.text_edit_singleline(&mut local_st.tmp_conn_definition.password);
        });
        ui.horizontal(|ui| {
            ui.label(&i18n.mongo_connection_srv);
            ui.add(toggle(&mut local_st.tmp_conn_definition.is_srv));
        });
        ui.horizontal(|ui| {
            if ui.button(&i18n.kafka_edit_cluster_cancel).clicked() {
                ui.close_menu();
            }
            if ui.button(&i18n.kafka_edit_cluster_save).clicked() {
                let tx_cloned = tx.clone();
                let tmp = local_st.tmp_conn_definition.clone();
                rt.spawn(async move {
                    let _ = match idx {
                        Some(idx) => tx_cloned.send(MongoMessage::EditConnection((idx, tmp))),
                        _ => tx_cloned.send(MongoMessage::AddConnection(tmp)),
                    }
                    .await;
                });
                ui.close_menu();
            }
        });
    }
}

pub struct MongoDatabasesSubpanel;

impl MongoDatabasesSubpanel {
    pub fn show(
        _ctx: &egui::Context,
        rt: &Runtime,
        tx: &Sender<MongoMessage>,
        ui: &mut egui::Ui,
        _app_st: &mut MongoAppState,
        local_st: &mut MongoLocalState,
        i18n: &I18n,
    ) {
        egui::ScrollArea::vertical()
            .id_source("databases_scroll_area")
            .show(ui, |ui| {
                egui::Grid::new("mongo_databases")
                    .num_columns(2)
                    .show(ui, |ui| {
                        for (db_idx, db_name) in local_st.db_names.iter().enumerate() {
                            ui.label(
                                egui::RichText::new("Info")
                                    .color(egui::Color32::from_rgb(128, 128, 128)),
                            )
                            .on_hover_ui(|ui| {
                                db_info(rt, ui, local_st, i18n, db_name);
                            });

                            let db_btn = ui.selectable_value(
                                &mut local_st.current_selection.db_idx,
                                db_idx,
                                db_name,
                            );
                            db_btn.context_menu(|ui| {
                                if ui.button(&i18n.mongo_copy_database_info).clicked() {
                                    let client_ref = local_st.conn.client.as_ref().unwrap().clone();
                                    match rt.block_on(async move {
                                        presenter::get_db_stats(&client_ref, db_name).await
                                    }) {
                                        Ok(document) => match serde_json::to_string(&document) {
                                            Ok(d) => {
                                                ui.ctx().copy_text(d);
                                            }
                                            Err(err) => {
                                                ui.ctx().copy_text(format!("{:?}", err));
                                            }
                                        },
                                        Err(err) => {
                                            ui.ctx().copy_text(format!("{:?}", err));
                                        }
                                    };
                                    ui.close_menu();
                                }
                            });
                            if db_btn.clicked() && local_st.conn.client.is_some() {
                                db_name.clone_into(&mut local_st.current_selection.db_name);
                                // local_st.current_selection.db_name = db_name.to_owned();
                                let tx_cloned = tx.clone();
                                let client_ref = local_st.conn.client.as_ref().unwrap().clone();
                                let db_name_cloned = db_name.clone();
                                local_st.current_selection.reset_to_new_db();
                                local_st.current_col_find_json_result.clear();
                                local_st.current_col_find_document_result.clear();

                                rt.spawn(async move {
                                    list_database_collections(
                                        &tx_cloned,
                                        &client_ref,
                                        &db_name_cloned,
                                    )
                                    .await;
                                });
                            }
                            ui.end_row();
                        }
                    });
            });
    }
}

pub struct MongoCollectionsSubpanel;

impl MongoCollectionsSubpanel {
    pub fn show(
        ctx: &egui::Context,
        rt: &Runtime,
        tx: &Sender<MongoMessage>,
        ui: &mut egui::Ui,
        _app_st: &mut MongoAppState,
        local_st: &mut MongoLocalState,
        i18n: &I18n,
    ) {
        egui::ScrollArea::vertical()
            .id_source("collections_scroll_area")
            .show(ui, |ui| {
                egui::Grid::new("mongo_collections")
                    .num_columns(2)
                    .show(ui, |ui| {
                        for (col_idx, col_name) in
                            local_st.current_db_collections.iter().enumerate()
                        {
                            ui.label(
                                egui::RichText::new("Info")
                                    .color(egui::Color32::from_rgb(128, 128, 128)),
                            )
                            .on_hover_ui(|ui| {
                                col_info(rt, ui, local_st, i18n, col_name);
                            });

                            let col_btn = ui.selectable_value(
                                &mut local_st.current_selection.col_idx,
                                col_idx,
                                col_name,
                            );
                            col_btn.context_menu(|ui| {
                                if ui.button(&i18n.mongo_copy_collection_info).clicked() {
                                    let client_ref = local_st.conn.client.as_ref().unwrap().clone();
                                    let db_name = local_st.current_selection.db_name.clone();
                                    match rt.block_on(async move {
                                        presenter::get_collection_stats(
                                            &client_ref,
                                            &db_name,
                                            col_name,
                                        )
                                        .await
                                    }) {
                                        Ok(document) => match serde_json::to_string(&document) {
                                            Ok(d) => {
                                                ui.ctx().copy_text(d);
                                            }
                                            Err(err) => {
                                                ui.ctx().copy_text(format!("{:?}", err));
                                            }
                                        },
                                        Err(err) => {
                                            ui.ctx().copy_text(format!("{:?}", err));
                                        }
                                    };
                                    ui.close_menu();
                                }
                            });
                            if col_btn.clicked() && local_st.conn.client.is_some() {
                                col_name.clone_into(&mut local_st.current_selection.col_name);
                                // local_st.current_selection.col_name = col_name.to_owned();
                                let ctx_cloned = ctx.clone();
                                let tx_cloned = tx.clone();
                                let client_ref = local_st.conn.client.as_ref().unwrap().clone();
                                let db_name = local_st.current_selection.db_name.clone();
                                let col = col_name.clone();

                                rt.spawn(async move {
                                    let _ = presenter::list_collection_documents(
                                        &tx_cloned,
                                        &client_ref,
                                        &db_name,
                                        &col,
                                    )
                                    .await;
                                    ctx_cloned.request_repaint();
                                });
                            }

                            ui.end_row();
                        }
                    });
            });
    }
}

async fn get_db_info(
    local_st: &MongoLocalState,
    db_name: &str,
) -> Result<bson::Document, MongoError> {
    let client_ref = local_st.conn.client.as_ref().unwrap().clone();

    presenter::get_db_stats(&client_ref, db_name).await
}

fn db_info(
    rt: &Runtime,
    ui: &mut egui::Ui,
    local_st: &MongoLocalState,
    i18n: &I18n,
    db_name: &str,
) {
    let info = rt.block_on(async move { get_db_info(local_st, db_name).await });

    match info {
        Ok(info) => {
            egui::Grid::new("mongodb_db_info")
                .num_columns(2)
                .show(ui, |ui| {
                    for (k, v) in info.iter() {
                        ui.label(k);
                        ui.monospace(format!("{:?}", v));
                        ui.end_row();
                    }
                });
        }
        Err(err) => {
            match err {
                MongoError::ClientNotInitialized => {
                    ui.label(&i18n.mongo_error_client_uninitialized)
                }
                MongoError::CommandError(msg) => ui.label(msg),
            };
        }
    }
}

async fn get_col_info(
    local_st: &MongoLocalState,
    col_name: &str,
) -> Result<bson::Document, MongoError> {
    let client_ref = local_st.conn.client.as_ref().unwrap().clone();
    let db_name = local_st.current_selection.db_name.clone();

    presenter::get_collection_stats(&client_ref, &db_name, col_name).await
}

/// Mostramos info en caso de acierto y error si fallo, en misma venta flotante
///
/// Función cerrada que se encarga de manejar ambos casos. No actualiza el error
/// en el estado global, porque la ubicación de ambos elementos es demasiado
/// diferente, por lo que a nivel de UI creo que es mejor así, y a nivel de préstamo
/// de variables simplifica mucho el código.
fn col_info(
    rt: &Runtime,
    ui: &mut egui::Ui,
    local_st: &MongoLocalState,
    i18n: &I18n,
    col_name: &str,
) {
    let info = rt.block_on(async move { get_col_info(local_st, col_name).await });

    match info {
        Ok(info) => {
            egui::Grid::new("mongodb_col_info")
                .num_columns(2)
                .show(ui, |ui| {
                    for (k, v) in info.iter() {
                        ui.label(k);
                        match v.as_document() {
                            Some(d) => {
                                JsonTree::new(
                                    k,
                                    &serde_json::to_value(d).unwrap_or(serde_json::Value::Null),
                                )
                                .show(ui);
                            }
                            None => {
                                ui.monospace(format!("{:?}", v));
                            }
                        };
                        ui.end_row();
                    }
                });
        }
        Err(err) => {
            match err {
                MongoError::ClientNotInitialized => {
                    ui.label(&i18n.mongo_error_client_uninitialized)
                }
                MongoError::CommandError(msg) => ui.label(msg),
            };
        }
    }
}
