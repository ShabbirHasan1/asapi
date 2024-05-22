// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use eframe::egui;
use sqlx::sqlite::SqliteRow;
use sqlx::Sqlite;
use tokio::runtime::Runtime;

use crate::app_state::AppState;
use crate::common::internationalization::{I18n, I18nSqlx};
use crate::quote;
use crate::sqlx_common::components::window_generator::GeneratorWindow;
use crate::sqlx_common::components::window_insertion::InsertionWindow;
use crate::sqlx_common::pagination::Paginator;
use crate::sqlx_common::presenter::SqlPresenter;
use crate::sqlx_common::state::{QuerySort, SqlxMessage};
use crate::sqlx_common::table::{PerformanceTable, RegularTable};
use crate::sqlx_common::traits::{Presenter as _, Show};

use super::components::sidenav::SQLiteSideNav;
use super::data_generation::generate_sqlite_value;
use super::parser::SqliteType;
use super::presenter::{self, run_statement_with_delete_control};
use super::state::{SQLiteAppState, SQLiteState};

pub struct SQLiteView {
    state: SQLiteState,
    tx: tokio::sync::mpsc::Sender<SqlxMessage>,
    rx: tokio::sync::mpsc::Receiver<SqlxMessage>,
    // Para uso sin necesidad de Runtime. Nos simplifica objetos y firmas.
    tx_sync: std::sync::mpsc::Sender<SqlxMessage>,
    rx_sync: std::sync::mpsc::Receiver<SqlxMessage>,
    generator_window: GeneratorWindow<SqliteRow, Sqlite, SqliteType>,
    insertion_window: InsertionWindow<SqliteRow, Sqlite, SqliteType>,
}

impl Default for SQLiteView {
    fn default() -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(8);
        let (tx_sync, rx_sync) = std::sync::mpsc::channel();
        let gen_window = GeneratorWindow::default();
        let ins_window = InsertionWindow::default();

        Self {
            state: SQLiteState::default(),
            tx,
            rx,
            tx_sync,
            rx_sync,
            generator_window: gen_window,
            insertion_window: ins_window,
        }
    }
}

impl SQLiteView {
    pub fn update(
        &mut self,
        ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        app_state: &mut AppState,
        rt: &Runtime,
        i18n: &I18nSqlx
    ) {
        // =======================================
        // Acciones iniciales
        // =======================================
        // --> Conectamos a archivo <--
        // En caso de que se abra un nuevo archivo, flag a true y nos intentamos
        // conectar. Desconectamos en caso de que ya haya una conexión.
        if self.state.connect_to_file {
            // --> Cerramos conexión si existe una abierta <--
            if let Some(pool) = self.state.pool.take() {
                // Bloqueo para asegurar que todo cerrado antes de reconectar. Puedo
                // de todas formas lanzar con `spawn` sin problemas.
                rt.block_on(async move {
                    pool.close().await;
                });
            }

            let path = self.state.current_connection.path.to_owned();
            self.state.pool = rt.block_on(async move { presenter::connect(&path).await.ok() });

            if let Some(ref pool_ref) = self.state.pool {
                // let pool_ref = self.state.pool.as_ref().unwrap().clone();
                self.state.sql.current_connection_tables_info =
                    rt.block_on(async move { presenter::tables_info(pool_ref).await });
                self.state.sql.tables = self
                    .state
                    .sql
                    .current_connection_tables_info
                    .keys()
                    .map(|k| k.to_ascii_lowercase())
                    .collect::<Vec<String>>();
                self.state.sql.tables.sort();
                self.state.connect_to_file = false;
            }
        }

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
            self.run_statement(ctx, rt, stmt, !app_state.sqlite.performance_table, false);
        }

        // --> Recibimos resultados de statements async/sync <--
        while let Ok(message) = self.rx.try_recv() {
            self.process_message(ctx, rt, app_state, message);
        }

        while let Ok(message) = self.rx_sync.try_recv() {
            self.process_message(ctx, rt, app_state, message);
        }

        // =======================================
        // Paneles laterales
        // =======================================
        SQLiteSideNav::show(
            ctx,
            rt,
            &self.tx,
            &self.tx_sync,
            &mut app_state.sqlite,
            &mut self.state,
            i18n,
        );

        // =======================================
        // Panel central
        // =======================================
        self.show_edit_row_window(ctx, rt, &mut app_state.sqlite);

        egui::CentralPanel::default().show(ctx, |ui| {
            // if self.state.sql.current_table_rows.len() > 0 {
            ui.set_width(ui.available_width());
            egui::CollapsingHeader::new("Table Columns")
                .default_open(false)
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        for (idx, (c_name, _c_type)) in
                            self.state.sql.current_table_columns.iter().enumerate()
                        {
                            if ui
                                .selectable_label(self.state.sql.column_visible[idx], c_name)
                                .clicked()
                            {
                                self.state.sql.column_visible[idx] =
                                    !self.state.sql.column_visible[idx];
                            }
                        }
                    });
                });
            // }

            ui.separator();

            // --> Definimos la entrada y lanzar stmt por parte del usuario <--
            let theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(ctx);
            let mut sql_layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
                let mut layout_job = egui_extras::syntax_highlighting::highlight(ui.ctx(), &theme, string, "sql");
                layout_job.wrap.max_width = wrap_width;
                ui.fonts(|f| f.layout_job(layout_job))
            };
            let sql_stmt_response = ui.add(
                egui::TextEdit::multiline(&mut self.state.sql.sql_statement)
                    .font(egui::TextStyle::Monospace) // for cursor height
                    .code_editor()
                    .desired_rows(5)
                    .lock_focus(true)
                    .desired_width(f32::INFINITY)
                    .layouter(&mut sql_layouter),
            );

            if sql_stmt_response.has_focus()
                && ctx.input(|i| i.key_down(egui::Key::Enter) && i.key_down(egui::Key::ArrowRight))
            {
                self.run_statement(
                    ctx,
                    rt,
                    self.state.sql.sql_statement.clone(),
                    !app_state.sqlite.performance_table,
                    true,
                );
            }

            if let Some(err) = self.state.sql.last_response.clone() {
                ui.label(&err);
            }

            let data_len = self.state.sql.current_table_rows.len();

            // --> Ejecutamos la consulta introducida por el usuario <--
            let hover_menu = |ui: &mut egui::Ui| {
                ui.label("Lanzar con \u{27a1} + \u{2ba8}");
            };
            ui.horizontal(|ui| {
                if ui.button("\u{25b6}").on_hover_ui(hover_menu).clicked() {
                    self.run_statement(
                        ctx,
                        rt,
                        self.state.sql.sql_statement.clone(),
                        !app_state.sqlite.performance_table,
                        true,
                    );
                    sql_stmt_response.request_focus();
                }

                Paginator::show(ui, &mut self.state.sql);
            });

            // --> Definimos el rango de filas a mostrar <--
            let end = self.state.sql.first_row_idx + self.state.sql.n_rows_to_show;
            if end > data_len {
                self.state.sql.last_row_idx = data_len;
            } else {
                self.state.sql.last_row_idx = end;
            }

            // --> Mostramos resultados de consultas en tabla que ocupa todo el espacio restante <--
            egui::ScrollArea::horizontal().show(ui, |ui| {
                if app_state.sqlite.performance_table {
                    PerformanceTable::show(ui, &mut self.state.sql);
                } else {
                    RegularTable::show(ui, &mut self.state.sql, rt, &self.tx);
                }
            });
        });

        // =============================================================================
        // Diálogos
        // =============================================================================
        if self.state.sql.data_gen.table_to_generate_data.is_some() {
            let t_name = self
                .state
                .sql
                .data_gen
                .table_to_generate_data
                .as_ref()
                .unwrap()
                .clone();

            if self.state.sql.data_gen.show_generator_window {
                let pr = SqlPresenter::<Sqlite>::default();
                self.generator_window.show(
                    &self.tx_sync,
                    ctx,
                    &mut self.state.sql,
                    &t_name,
                    i18n,
                    &pr,
                    SqliteType::from_string,
                    generate_sqlite_value,
                )
            } else if self.state.sql.data_gen.show_insertion_window {
                let pr = SqlPresenter::<Sqlite>::default();
                self.insertion_window.show(
                    &self.tx_sync,
                    ctx,
                    &mut self.state.sql,
                    &t_name,
                    i18n,
                    |t| pr.should_be_wrapped(t.to_ascii_uppercase().as_str()),
                )
            }
        }
    }

    fn process_message(
        &mut self,
        ctx: &egui::Context,
        rt: &Runtime,
        app_state: &mut AppState,
        message: SqlxMessage,
    ) {
        match message {
            SqlxMessage::DeleteStatement((table_name, row_idx)) => {
                let filters = self.statement_filter(row_idx);
                let delete_stmt = format!(
                    "DELETE FROM {:} WHERE {}",
                    table_name,
                    filters.join(" AND ")
                );
                self.run_statement(
                    ctx,
                    rt,
                    delete_stmt,
                    !app_state.sqlite.performance_table,
                    true,
                );
            }
            SqlxMessage::SelectResponse((data, columns, make_all_visible)) => {
                self.state.sql.reset();
                self.state.sql.current_table_rows = data;
                self.state.sql.current_table_columns = columns;
                if make_all_visible {
                    self.state.sql.column_visible = std::iter::repeat(true)
                        .take(self.state.sql.current_table_columns.len())
                        .collect::<Vec<_>>();
                }
            }
            SqlxMessage::Empty => {
                self.state.sql.reset();
            }
            SqlxMessage::Error(msg) => {
                self.state.sql.last_response = Some(msg);
            }
            SqlxMessage::InsertStatement(stmt) => {
                self.run_statement(ctx, rt, stmt, !app_state.pg.performance_table, true)
            }
            SqlxMessage::DeleteAllStmt(t_name) => {
                let delete_stmt = format!("DELETE FROM {:}", t_name);
                self.run_statement(ctx, rt, delete_stmt, !app_state.pg.performance_table, true);
            }
            // Dos casos imposibles.
            SqlxMessage::AddConnection(_) | SqlxMessage::EditConnection(_) => ()
        }
    }
    fn run_statement(
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

        rt.spawn(async move {
            run_statement_with_delete_control(
                &pool_ref,
                &tx_cloned,
                stmt.as_ref(),
                make_all_visible,
                delete_allowed,
            )
            .await;

            cloned_ctx.request_repaint(); // Tras acabar la ejecución, pedimos repintado.
        });
    }

    fn statement_filter(&self, row_idx: usize) -> Vec<String> {
        let pr = SqlPresenter::<Sqlite>::default();
        self.state
            .sql
            .current_table_columns
            .iter()
            .enumerate()
            .filter(|(_, e)| pr.should_be_added_to_delete_stmt(&e.1))
            .map(|(col_idx, e)| {
                format!(
                    "{} = {}",
                    e.0,
                    if pr.should_be_wrapped(&e.1) {
                        quote!(&self.state.sql.current_table_rows[row_idx][col_idx])
                    } else {
                        self.state.sql.current_table_rows[row_idx][col_idx].clone()
                    }
                )
            })
            .collect()
    }

    fn show_edit_row_window(
        &mut self,
        ctx: &egui::Context,
        rt: &Runtime,
        sqlite_app_state: &mut SQLiteAppState,
    ) {
        if let Some(row_idx) = self.state.sql.row_being_editted.selected_row {
            egui::Window::new("Editar Fila")
                .collapsible(false)
                .resizable(true)
                .show(ctx, |ui| {
                    let table_name =
                        self.state.sql.tables[self.state.sql.current_table_idx].clone();
                    egui::Grid::new("row-insertion-definition")
                        .num_columns(3)
                        .show(ui, |ui| {
                            for (idx, (n, t)) in
                                self.state.sql.current_table_columns.iter().enumerate()
                            {
                                ui.code(n);
                                ui.monospace(t.clone());
                                ui.text_edit_singleline(
                                    &mut self.state.sql.row_being_editted.row_data[idx],
                                );

                                ui.end_row();
                            }
                        });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::LEFT), |ui| {
                        if ui.button("Cancel").clicked() {
                            self.state.sql.row_being_editted.selected_row = None;
                        }

                        if ui.button("Update").clicked() {
                            let pr = SqlPresenter::<Sqlite>::default();
                            let user_values_string = self
                                .state
                                .sql
                                .row_being_editted
                                .row_data
                                .iter()
                                .enumerate()
                                .map(|(idx, col_data)| {
                                    let (col_name, t) =
                                        self.state.sql.current_table_columns[idx].clone();

                                    let wrapped = quote!(&col_data);
                                    let value = if pr.should_be_wrapped(&t) {
                                        &wrapped
                                    } else {
                                        col_data
                                    };
                                    format!("{}={}", col_name, value)
                                })
                                .collect::<Vec<String>>()
                                .join(", ");

                            let filters = self.statement_filter(row_idx);

                            let stmt = format!(
                                "UPDATE {} SET {} WHERE {}",
                                table_name,
                                user_values_string,
                                filters.join(" AND ")
                            );

                            self.run_statement(
                                ctx,
                                rt,
                                stmt,
                                !sqlite_app_state.performance_table,
                                true,
                            );
                            self.state.sql.row_being_editted.selected_row = None;
                        }
                    });
                });
        }
    }
}
