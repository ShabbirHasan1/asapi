// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use eframe::egui;
use tokio::{
    runtime::Runtime,
    sync::mpsc::{Receiver, Sender},
};

use crate::common::internationalization::I18nClickHouse;
use crate::quote;
use crate::sqlx_common::state::QuerySort;

use super::{
    components::ClickHouseSideNav,
    domain::ClickHouseMessage,
    presenter,
    state::{ClickHouseAppState, ClickHouseState},
};

pub struct ClickHouseView {
    sidenav: ClickHouseSideNav,
    pub state: ClickHouseState,
    pub tx: Sender<ClickHouseMessage>,
    pub rx: Receiver<ClickHouseMessage>,

    pub tx_sync: std::sync::mpsc::Sender<ClickHouseMessage>,
    pub rx_sync: std::sync::mpsc::Receiver<ClickHouseMessage>,
}

impl Default for ClickHouseView {
    fn default() -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(8);
        let (tx_sync, rx_sync) = std::sync::mpsc::channel();

        Self {
            sidenav: Default::default(),
            state: ClickHouseState::default(),
            tx,
            rx,
            tx_sync,
            rx_sync,
        }
    }
}

impl ClickHouseView {
    pub fn update(
        &mut self,
        ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        app_st: &mut ClickHouseAppState,
        rt: &Runtime,
        i18n: &I18nClickHouse,
    ) {
        // =======================================
        // Preparación de cada ciclo
        // =======================================
        // --> Ejecutamos query al cambiar el orden en la tabla <--
        if self.state.sql.change_order {
            self.state.sql.change_order = false;
            let selected_column_name = self.state.sql.current_table_columns
                [self.state.sql.column_index_selected]
                .0
                .clone();
            let stmt = match self.state.sql.query_sort {
                QuerySort::None => self.state.sql.sql_statement.clone(),
                QuerySort::Asc => format!(
                    "{} ORDER BY {} ASC",
                    self.state.sql.sql_statement, selected_column_name
                ),
                QuerySort::Desc => format!(
                    "{} ORDER BY {} DESC",
                    self.state.sql.sql_statement, selected_column_name
                ),
            };

            // Esto acaba llegando al `while let ... self.rx.try_recv` justo debajo.
            // false porque mantenemos lo que se mostraba
            self.run_statement(ctx, rt, stmt, !app_st.performance_table, false);
        }

        // --> Procesamos mensajes asíncronos <--
        while let Ok(message) = self.rx.try_recv() {
            self.process_message(ctx, rt, app_st, message);
        }

        // --> Procesamos mensajes síncrones <--
        while let Ok(message) = self.rx_sync.try_recv() {
            self.process_message(ctx, rt, app_st, message);
        }

        // =======================================
        // Paneles laterales
        // =======================================
        self.sidenav.show(
            ctx,
            rt,
            &self.tx,
            &self.tx_sync,
            app_st,
            &mut self.state,
            i18n,
        );

        // =======================================
        // Panel Central
        // =======================================
        self.show_central_panel(ctx, rt, app_st, i18n);
    }

    // =======================================
    // Métodos y Funciones Auxiliares
    // =======================================
    pub fn run_statement(
        &mut self,
        ctx: &egui::Context,
        rt: &Runtime,
        stmt: String,
        delete_allowed: bool,
        make_all_visible: bool,
    ) {
        // Guarda por si lanzamos query cuando no hay conexión.
        // Poddríamos hacer renderizado condicional, pero así reducimos algo la indentación.
        if self.state.pool.is_none() {
            return;
        }

        let pool_ref = self.state.pool.as_ref().unwrap().clone();
        let tx_cloned = self.tx.clone();
        let cloned_ctx = ctx.clone();

        println!("Statement para ejecutar: {stmt}");

        rt.spawn(async move {
            // run_statement_with_delete_control(
            //     &pool_ref,
            //     &tx_cloned,
            //     stmt.as_ref(),
            //     make_all_visible,
            //     delete_allowed,
            // )
            // .await;

            cloned_ctx.request_repaint(); // Tras acabar la ejecución, pedimos repintado.
        });
    }

    fn process_message(
        &mut self,
        ctx: &egui::Context,
        rt: &Runtime,
        app_st: &mut ClickHouseAppState,
        message: ClickHouseMessage,
    ) {
        match message {
            ClickHouseMessage::DeleteStatement((table_name, row_idx)) => {
                // let filters = self.statement_filter(row_idx);
                // let delete_stmt = format!(
                //     "DELETE FROM {:} WHERE {}",
                //     table_name,
                //     filters.join(" AND ")
                // );
                // self.run_statement(ctx, rt, delete_stmt, !app_st.pg.performance_table, true);
            }
            ClickHouseMessage::SelectResponse((data, columns, make_all_visible)) => {
                self.state.sql.reset();
                self.state.sql.current_table_rows = data;
                self.state.sql.current_table_columns = columns;
                if make_all_visible {
                    self.state.sql.column_visible = std::iter::repeat(true)
                        .take(self.state.sql.current_table_columns.len())
                        .collect::<Vec<_>>();
                }
            }
            ClickHouseMessage::Empty => {
                // self.state.sql.reset();
            }
            ClickHouseMessage::Error(msg) => {
                // self.state.sql.last_response = Some(msg);
            }
            ClickHouseMessage::InsertStatement(stmt) => {
                // self.run_statement(ctx, rt, stmt, !app_st.pg.performance_table, true)
            }
            ClickHouseMessage::DeleteAllStmt(t_name) => {
                // log::info!("{t_name}");
                // let delete_stmt = format!("DELETE FROM {:}", t_name);
                // log::info!("{delete_stmt}");
                // self.run_statement(ctx, rt, delete_stmt, !app_st.pg.performance_table, true);
            }
            ClickHouseMessage::AddConnection(def) => {
                app_st.connections.push(def);
            }
            ClickHouseMessage::EditConnection((idx, def)) => {
                app_st.connections[idx] = def;
            }
            ClickHouseMessage::DatabaseTables(tables) => {
                self.state.current_selection.tables = tables;
            }
        }
    }

    pub fn statement_filter(&self, row_idx: usize) -> Vec<String> {
        self.state
            .sql
            .current_table_columns
            .iter()
            .enumerate()
            .filter(|(_, e)| presenter::should_be_added_to_delete_stmt(&e.1))
            .map(|(col_idx, e)| {
                format!(
                    "{} = {}",
                    e.0,
                    if presenter::should_be_wrapped(&e.1) {
                        quote!(&self.state.sql.current_table_rows[row_idx][col_idx])
                    } else {
                        self.state.sql.current_table_rows[row_idx][col_idx].clone()
                    }
                )
            })
            .collect()
    }
}
