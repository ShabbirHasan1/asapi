// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

#[derive(Clone)]
pub struct I18nPg {
    pub connection_name: String,
    pub connection_host: String,
    pub connection_port: String,
    pub connection_user: String,
    pub connection_password: String,
    pub connection_dbname: String,
    pub edit_connection_confirm: String,
    pub edit_connection_cancel: String,
    pub btn_add_connection: String,
    pub btn_connect: String,
    pub btn_connected: String,
    pub btn_load_tables: String,
    pub btn_clean_table: String,
    pub header_connection: String,
    pub btn_query: String,
    pub btn_table_data_generator: String,
    pub btn_table_data_insertion: String,
    pub impossible_to_connect: String,
    pub connection: String,
    pub no_connection: String,
    pub info_performance_table: String,
    pub performance_table: String,
    pub close_connection: String,
    pub delete_connection: String,
    pub edit_connection: String,
    pub reload_tables: String,
}

#[derive(Clone)]
pub struct I18nMySql {
    pub info_performance_table: String,
    pub performance_table: String,
    pub close_connection: String,
    pub delete_connection: String,
    pub edit_connection: String,
    pub reload_tables: String,
}

#[derive(Clone)]
pub struct I18nSqlite {
    pub btn_add_connection: String,
    pub btn_table_data_generator: String,
    pub btn_clean_table: String,
    pub btn_table_data_insertion: String,
    pub info_performance_table: String,
    pub performance_table: String,
    pub connection_btn_help: String,
    pub close_connection: String,
    pub delete_connection: String,
    pub edit_connection: String,
    pub reload_tables: String,
}

#[derive(Clone)]
pub struct I18nSqlx {
    pub connections: String,
    pub tables: String,
    pub table_columns: String,

    // Sub-structs para cada `DBMS`.
    pub pg: I18nPg,
    pub mysql: I18nMySql,
    pub sqlite: I18nSqlite,
}

impl I18nSqlx {
    pub fn new_es() -> Self {
        I18nSqlx {
            connections: String::from("Conexiones"),
            tables: String::from("Tablas"),
            table_columns: String::from("Columnas Existentes"),
            pg: I18nPg {
                connection_name: String::from("Nombre"),
                connection_host: "Host".to_owned(),
                connection_port: "Puerto".to_owned(),
                connection_user: "Usuario".to_owned(),
                connection_password: "Contraseña".to_owned(),
                connection_dbname: "Base de Datos".to_owned(),
                edit_connection_confirm: String::from("Guardar"),
                edit_connection_cancel: String::from("Cancelar"),
                btn_add_connection: "Anadir Conexión".to_owned(),
                btn_connect: "Conectar".to_owned(),
                btn_connected: "Conectado".to_owned(),
                btn_load_tables: "Cargar Tablas".to_owned(),
                btn_clean_table: "Limpiar Tabla".to_owned(),
                btn_query: "Ejecutar".to_owned(),
                btn_table_data_generator: "Generación de Datos".to_owned(),
                btn_table_data_insertion: "Insertar Fila".to_owned(),
                header_connection: "Conexión".to_owned(),
                impossible_to_connect: "Imposible conectar con Postgres.".to_owned(),
                connection: "Conectado.".to_owned(),
                no_connection: "No conectado.".to_owned(),
                info_performance_table: "El borrado de filas no es posible con tabla con mejor rendimiento.\nSeleccione esta opción para una cantidad de celdas (celdas x columnas) masiva, del orden de 1e5, o cuando la cantidad de datos dentro de las celdas pueda ser muy grande y tenga centeneras o miles de estas celdas, como puede ser el caso de grandes textos, json, binarios, arrays o datos geográficos.".to_owned(),
                performance_table: "Tabla Optimizada".to_owned(),
                close_connection: String::from("Cerrar Conexión"),
                delete_connection: String::from("Borrar Conexión"),
                edit_connection: String::from("Editar Conexión"),
                reload_tables: String::from("Recargar Tablas"),
            },

            mysql: I18nMySql {
                info_performance_table: "El borrado de filas no es posible con tabla con mejor rendimiento.\nSeleccione esta opción para una cantidad de celdas (celdas x columnas) masiva, del orden de 1e5, o cuando la cantidad de datos dentro de las celdas pueda ser muy grande y tenga centeneras o miles de estas celdas, como puede ser el caso de grandes textos, json, binarios o datos geográficos.".to_owned(),
                performance_table: "Tabla Optimizada".to_owned(),
                close_connection: String::from("Cerrar Conexión"),
                delete_connection: String::from("Borrar Conexión"),
                edit_connection: String::from("Editar Conexión"),
                reload_tables: String::from("Recargar Tablas"),
            },

            sqlite: I18nSqlite {
                // Sqlite
                btn_add_connection: "Abrir Archivo".to_owned(),
                btn_table_data_generator: "Generación de Datos".to_owned(),
                btn_clean_table: "Limpiar Tabla".to_owned(),
                btn_table_data_insertion: "Insertar Fila".to_owned(),
                info_performance_table: "El borrado de filas no es posible con tabla con mejor rendimiento.\nSeleccione esta opción para una cantidad de celdas (celdas x columnas) masiva, del orden de 1e5, o cuando la cantidad de datos dentro de las celdas pueda ser muy grande y tenga centeneras o miles de estas celdas, como puede ser el caso de grandes textos, binarios o datos geográficos.".to_owned(),
                performance_table: "Tabla Optimizada".to_owned(),
                connection_btn_help: String::from("Para conectar, clicar en definición de la conexión"),
                close_connection: String::from("Cerrar Conexión"),
                delete_connection: String::from("Borrar Conexión"),
                reload_tables: String::from("Recargar Tablas"),
                edit_connection: String::from("Editar Nombre de la Conexión"),
            },
        }
    }

    pub fn new_en() -> Self {
        I18nSqlx {
            connections: String::from("Connections"),
            tables: String::from("Tables"),
            table_columns: String::from("Table Columns"),

            pg: I18nPg {
                info_performance_table: "Deletion is forbidden for performance table.\nSelect this for massive quantity of cells (rows x columns), order of 1e5, or when massive amount of data inside the cells, like long varchar, big json/binaries/arrays or geographical data.".to_owned(),
                connection_name: String::from("Name"),
                connection_host: "Host".to_owned(),
                connection_port: "Port".to_owned(),
                connection_user: "User".to_owned(),
                connection_password: "Password".to_owned(),
                connection_dbname: "Database".to_owned(),
                edit_connection_confirm: String::from("Save"),
                edit_connection_cancel: String::from("Cancel"),
                btn_add_connection: "Add Connection".to_owned(),

                btn_connect: "Connect".to_owned(),
                btn_connected: "Connected".to_owned(),
                btn_load_tables: "Load Tables".to_owned(),
                btn_clean_table: "Clean Table".to_owned(),
                btn_query: "Run".to_owned(),
                btn_table_data_generator: "Data Generation".to_owned(),
                btn_table_data_insertion: "Insert Row".to_owned(),
                header_connection: "Connection".to_owned(),
                impossible_to_connect: "Impossible to connect with Postgres.".to_owned(),
                connection: "Connected.".to_owned(),
                no_connection: "No connected.".to_owned(),
                performance_table: "Performance Table".to_owned(),
                close_connection: String::from("Close Connection"),
                delete_connection: String::from("Delete Connection"),
                edit_connection: String::from("Edit Connection"),
                reload_tables: String::from("Reload Tables"),
            },

            mysql: I18nMySql {
                info_performance_table: "Deletion is forbidden for performance table.\nSelect this for massive quantity of cells (rows x columns), order of 1e5, or when massive amount of data inside the cells, like long texts, big binaries/jsons, or geographical data.".to_owned(),
                performance_table: "Performance Table".to_owned(),
                close_connection: String::from("Close Connection"),
                delete_connection: String::from("Delete Connection"),
                edit_connection: String::from("Edit Connection"),
                reload_tables: String::from("Reload Tables"),
            },

            sqlite: I18nSqlite {
                btn_add_connection: "Open File".to_owned(),
                btn_table_data_generator: "Data Generation".to_owned(),
                btn_clean_table: "Clean Table".to_owned(),
                btn_table_data_insertion: "Insert Row".to_owned(),
                info_performance_table: "Deletion is forbidden for performance table.\nSelect this for massive quantity of cells (rows x columns), order of 1e5, or when massive amount of data inside the cells, like long varchar, big binaries o geographical data.".to_owned(),
                performance_table: "Performance Table".to_owned(),
                connection_btn_help: String::from("To connect, click connection definition button"),
                close_connection: String::from("Close Connection"),
                delete_connection: String::from("Delete Connection"),
                reload_tables: String::from("Reload Tables"),
                edit_connection: String::from("Edit Connection Name"),
            }
        }
    }
}
