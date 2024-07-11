// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::io::{Error as IOError, ErrorKind};
use sqlm::mysqlm::state::MySqlAppState;
use sqlm::pgm::state::PgAppState;
use redism::state::{RedisAppState, RedisConnectionDefinition};
use sqlm::sqlitem::state::{SQLiteAppState, SQLiteConnectionDefinition};
use sqlm::sqlx_common::state::SqlConnectionDefinition;
use tokio::fs as async_fs;

use clickhousem::domain::ClickHouseConnectionDefinition;
use clickhousem::state::ClickHouseAppState;
use common::internationalization::I18nOptions;
use httpm::methods::HttpMethod;
use httpm::request::Request;
use httpm::state::HttpAppState;
use httpm::workspace::Workspace;
use kafkam::state::{Cluster, KafkaAppState};
use mongom::state::{MongoAppState, MongoConnectionDefinition};

#[derive(Clone, Deserialize, Serialize, Copy, PartialEq, Debug, Default)]
pub enum ViewType {
    #[default]
    Http,
    Pg,
    MySql,
    SQLite,
    Redis,
    Mongo,
    Kafka,
    ClickHouse
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct AppConfig {
    pub version: u8,
    pub dark_theme: bool,
    pub language: I18nOptions,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct AppState {
    pub app_config: AppConfig,
    pub selected_view: ViewType,
    pub show_settings: bool,
    pub http: HttpAppState,
    pub pg: PgAppState,
    pub mysql: MySqlAppState,
    pub sqlite: SQLiteAppState,
    pub redis: RedisAppState,
    pub mongo: MongoAppState,
    pub kafka: KafkaAppState,
    pub clickhouse: ClickHouseAppState,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            app_config: Default::default(),
            selected_view: Default::default(),
            show_settings: false,
            http: Default::default(),
            // HttpAppState {
            //     show_sidebar: true,
            //     workspaces: vec![Workspace::default()],
            //     current_workspace_idx: 0,
            // },
            pg: Default::default(),
            mysql: Default::default(),
            sqlite: Default::default(),
            redis: Default::default(),
            mongo: Default::default(),
            kafka: Default::default(),
            clickhouse: Default::default(),
        }
    }
}

pub fn read_state_and_adapt(file_name: &str) -> AppState {
    let string_data = fs::read_to_string(file_name);
    if string_data.is_err() {
        return AppState::default();
    }

    let json_value = serde_json::from_str::<Value>(&string_data.unwrap());
    if json_value.is_err() {
        return AppState::default();
    }
    let j = json_value.unwrap();
    let app_config = read_app_config(j.get("app_config"));

    AppState {
        app_config,
        selected_view: read_selected_view(j.get("selected_view")),
        show_settings: extract_bool(&j, "show_settings"),
        http: read_http_app_state(j.get("http")),
        pg: read_pg_app_state(j.get("pg")),
        mysql: read_mysql_app_state(j.get("mysql")),
        sqlite: read_sqlite_app_state(j.get("sqlite")),
        redis: read_redis_app_state(j.get("redis")),
        mongo: read_mongo_app_state(j.get("mongo")),
        kafka: read_kafka_app_state(j.get("kafka")),
        clickhouse: read_clickhouse_app_state(j.get("clickhouse")),
    }
}


fn read_clickhouse_app_state(m: Option<&Value>) -> ClickHouseAppState {
    match m {
        Some(p) => {
            let show_sidebar = extract_bool(p, "show_sidebar");
            let performance_table = extract_bool(p, "performance_table");
            let connections = p
                .get("connections")
                .and_then(|ws| {
                    ws.as_array().map(|arr| {
                        arr.iter()
                            .map(read_clickhouse_connection_definition)
                            .collect::<Vec<_>>()
                    })
                })
                .unwrap_or_default();

           ClickHouseAppState {
                show_sidebar,
                performance_table,
                connections,
            }
        }
        None => ClickHouseAppState::default(),
    }
}

fn read_clickhouse_connection_definition(c: &Value) -> ClickHouseConnectionDefinition {
    ClickHouseConnectionDefinition {
        name: extract_string(c, "name"),
        host: extract_string(c, "host"),
        port: extract_string(c, "port"),
        user: extract_string(c, "user"),
        password: extract_string(c, "password"),
        protocol: Default::default(),
        options: Default::default()
    }
}

fn read_kafka_app_state(m: Option<&Value>) -> KafkaAppState {
    match m {
        Some(m) => {
            let show_sidebar = extract_bool(m, "show_sidebar");
            let clusters = m
                .get("connections")
                .and_then(|ws| {
                    ws.as_array().map(|arr| {
                        arr.iter()
                            .map(read_kafka_cluster_definition)
                            .collect::<Vec<_>>()
                    })
                })
                .unwrap_or_default();

            KafkaAppState {
                show_sidebar,
                clusters,
            }
        }
        None => KafkaAppState::default(),
    }
}

fn read_kafka_cluster_definition(c: &Value) -> Cluster {
    Cluster {
        name: extract_string(c, "name"),
        host: extract_string(c, "host"),
        port: extract_string(c, "port"),
    }
}

fn read_mongo_app_state(m: Option<&Value>) -> MongoAppState {
    match m {
        Some(m) => {
            let show_sidebar = extract_bool(m, "show_sidebar");
            let performance_table = extract_bool(m, "performance_table");
            let connections = m
                .get("connections")
                .and_then(|ws| {
                    ws.as_array().map(|arr| {
                        arr.iter()
                            .map(read_mongo_connection_definition)
                            .collect::<Vec<_>>()
                    })
                })
                .unwrap_or_default();

            MongoAppState {
                show_sidebar,
                performance_table,
                connections,
            }
        }
        None => MongoAppState::default(),
    }
}

fn read_mongo_connection_definition(c: &Value) -> MongoConnectionDefinition {
    MongoConnectionDefinition {
        name: extract_string(c, "name"),
        host: extract_string(c, "host"),
        port: extract_string(c, "port"),
        user: extract_string(c, "user"),
        password: extract_string(c, "password"),
        is_srv: extract_bool(c, "is_srv"),
    }
}

fn read_redis_app_state(f: Option<&Value>) -> RedisAppState {
    match f {
        Some(f) => {
            let show_sidebar = extract_bool(f, "show_sidebar");
            let connections = f
                .get("connections")
                .and_then(|ws| {
                    ws.as_array().map(|arr| {
                        arr.iter()
                            .map(read_redis_connection_definition)
                            .collect::<Vec<_>>()
                    })
                })
                .unwrap_or_default();

            RedisAppState {
                show_sidebar,
                connections,
            }
        }
        None => RedisAppState::default(),
    }
}

fn read_redis_connection_definition(c: &Value) -> RedisConnectionDefinition {
    RedisConnectionDefinition {
        host: extract_string(c, "host"),
        port: extract_string(c, "port"),
    }
}

fn read_sqlite_app_state(s: Option<&Value>) -> SQLiteAppState {
    match s {
        Some(p) => {
            let show_sidebar = extract_bool(p, "show_sidebar");
            let performance_table = extract_bool(p, "performance_table");
            let connections = p
                .get("connections")
                .and_then(|ws| {
                    ws.as_array().map(|arr| {
                        arr.iter()
                            .map(read_sqlite_connection_definition)
                            .collect::<Vec<_>>()
                    })
                })
                .unwrap_or_default();

            SQLiteAppState {
                show_sidebar,
                performance_table,
                connections,
            }
        }
        None => SQLiteAppState::default(),
    }
}

fn read_pg_app_state(p: Option<&Value>) -> PgAppState {
    match p {
        Some(p) => {
            let show_sidebar = extract_bool(p, "show_sidebar");
            let performance_table = extract_bool(p, "performance_table");
            let connections = p
                .get("connections")
                .and_then(|ws| {
                    ws.as_array().map(|arr| {
                        arr.iter()
                            .map(read_sql_connection_definition)
                            .collect::<Vec<_>>()
                    })
                })
                .unwrap_or_default();

            PgAppState {
                show_sidebar,
                performance_table,
                connections,
            }
        }
        None => PgAppState::default(),
    }
}

fn read_mysql_app_state(m: Option<&Value>) -> MySqlAppState {
    match m {
        Some(p) => {
            let show_sidebar = extract_bool(p, "show_sidebar");
            let performance_table = extract_bool(p, "performance_table");
            let connections = p
                .get("connections")
                .and_then(|ws| {
                    ws.as_array().map(|arr| {
                        arr.iter()
                            .map(read_sql_connection_definition)
                            .collect::<Vec<_>>()
                    })
                })
                .unwrap_or_default();

            MySqlAppState {
                show_sidebar,
                performance_table,
                connections,
            }
        }
        None => MySqlAppState::default(),
    }
}

fn read_sql_connection_definition(c: &Value) -> SqlConnectionDefinition {
    SqlConnectionDefinition {
        name: extract_string(c, "name"),
        host: extract_string(c, "host"),
        port: extract_string(c, "port"),
        user: extract_string(c, "user"),
        password: extract_string(c, "password"),
        dbname: extract_string(c, "dbname"),
    }
}

fn read_sqlite_connection_definition(c: &Value) -> SQLiteConnectionDefinition {
    SQLiteConnectionDefinition {
        name: extract_string(c, "name"),
        path: extract_string(c, "path"),
    }
}

fn read_http_app_state(s: Option<&Value>) -> HttpAppState {
    match s {
        Some(s) => {
            let show_sidebar = s
                .get("show_sidebar")
                .and_then(|b| b.as_bool())
                .unwrap_or_default();
            let ws_idx = s
                .get("current_workspace_idx")
                .and_then(|b| b.as_u64().map(|v| v as usize))
                .unwrap_or_default();
            let workspaces = s
                .get("workspaces")
                .and_then(|ws| {
                    ws.as_array()
                        .map(|arr| arr.iter().map(read_workspace).collect::<Vec<_>>())
                })
                .unwrap_or_default();

            HttpAppState {
                show_sidebar,
                workspaces,
                current_workspace_idx: ws_idx,
            }
        }
        None => HttpAppState::default(),
    }
}

fn read_workspace(w: &Value) -> Workspace {
    let id = w.get("id").and_then(|v| v.as_u64().map(|v| v as usize));
    let name = w.get("name").and_then(|n| n.as_str()).unwrap_or_default();
    let options = w.get("show_options").and_then(|v| v.as_bool());
    let requests = w.get("requests").and_then(|rs| {
        rs.as_array()
            .map(|arr| arr.iter().map(read_request).collect::<Vec<_>>())
    });

    // Hago `unwrap_or_default` aquí para que el código me quedo más compacto.
    Workspace {
        id: id.unwrap_or_default(),
        name: name.to_string(),
        requests: requests.unwrap_or_default(),
        show_options: options.unwrap_or_default(),
    }
}

fn read_request(v: &Value) -> Request {
    let name = v.get("name").and_then(|v| v.as_str()).unwrap_or_default();
    let method = v
        .get("method")
        .and_then(|v| v.as_str())
        .and_then(|m| HttpMethod::from_str(m))
        .unwrap_or_default();
    let url = v.get("url").and_then(|v| v.as_str()).unwrap_or_default();
    let multipart = v
        .get("multipart")
        .and_then(|v| v.as_bool())
        .unwrap_or_default();
    let body_params = v
        .get("body_params")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|param| {
                    if let Some(param_array) = param.as_array() {
                        if param_array.len() == 3 {
                            let key = param_array[0].as_str().unwrap_or_default().to_string();
                            let value = param_array[1].as_str().unwrap_or_default().to_string();
                            let has_files = param_array[2].as_bool().unwrap_or_default();
                            return Some((key, value, has_files));
                        }
                    }
                    None
                })
                .collect::<Vec<(String, String, bool)>>()
        })
        .unwrap_or_default();

    let headers_params = v
        .get("headers_params")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|param| {
                    if let Some(param_array) = param.as_array() {
                        if param_array.len() == 2 {
                            let key = param_array[0].as_str().unwrap_or_default().to_string();
                            let value = param_array[1].as_str().unwrap_or_default().to_string();
                            return Some((key, value));
                        }
                    }
                    None
                })
                .collect::<Vec<(String, String)>>()
        })
        .unwrap_or_default();

    Request {
        name: name.to_owned(),
        method,
        url: url.to_owned(),
        multipart,
        body_params,
        headers_params,
    }
}

// fn read_show_settings(s: Option<&Value>) -> bool {
//     s.and_then(|b| b.as_bool()).unwrap_or_default()
// }

fn read_selected_view(view: Option<&Value>) -> ViewType {
    match view {
        Some(v) => match v.as_str().unwrap_or_default() {
            "Http" => ViewType::Http,
            "Pg" => ViewType::Pg,
            "MySql" => ViewType::MySql,
            "SQLite" => ViewType::SQLite,
            "Redis" => ViewType::Redis,
            "Mongo" => ViewType::Mongo,
            "Kafka" => ViewType::Kafka,
            _ => ViewType::default(),
        },
        None => ViewType::default(),
    }
}

fn read_app_config(config: Option<&Value>) -> AppConfig {
    match config {
        Some(c) => {
            let version = c
                .get("version")
                .and_then(|v| v.as_u64().map(|v| v as u8))
                .unwrap_or_default();
            let language = c
                .get("language")
                .and_then(|v| v.as_str())
                .map(|l| {
                    if l == "ES" {
                        I18nOptions::ES
                    } else if l == "EN" {
                        I18nOptions::EN
                    } else {
                        I18nOptions::EN
                    }
                })
                .unwrap_or_default();
            AppConfig {
                version,
                dark_theme: extract_bool(c, "dark_theme"),
                language,
            }
        }
        None => AppConfig::default(),
    }
}

fn extract_bool(v: &Value, field: &str) -> bool {
    v.get(field).and_then(|n| n.as_bool()).unwrap_or_default()
}

fn extract_string(v: &Value, field: &str) -> String {
    v.get(field)
        .and_then(|n| n.as_str())
        .unwrap_or_default()
        .to_string()
}

pub fn load_state(file_name: &str) -> Result<AppState, IOError> {
    let json_data = fs::read_to_string(file_name)?;
    let state: AppState = serde_json::from_str(&json_data).map_err(|err| {
        IOError::new(
            ErrorKind::InvalidData,
            format!("Failed to deserialize data: {}", err),
        )
    })?;

    Ok(state)
}

pub async fn _async_save_state(
    state: &AppState,
    file_name: &str,
    save_bak: bool,
) -> Result<(), IOError> {
    let json_string = serde_json::to_string_pretty(state).map_err(|err| {
        IOError::new(
            ErrorKind::InvalidData,
            format!("Failed to serialize data: {}", err),
        )
    })?;
    if save_bak {
        async_fs::copy(file_name, format!("{file_name}.bak")).await?;
    }
    async_fs::write(file_name, json_string).await?;
    Ok(())
}

pub fn save_state(state: &AppState, file_name: &str, save_bak: bool) -> Result<(), IOError> {
    let json_string = serde_json::to_string_pretty(state).map_err(|err| {
        IOError::new(
            ErrorKind::InvalidData,
            format!("Failed to serialize data: {}", err),
        )
    })?;
    if save_bak {
        fs::copy(file_name, format!("{file_name}.bak"))?;
    }
    fs::write(file_name, json_string)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_read_state_and_adapt() {
        // JSON de prueba
        let test_json = json!({
            "app_config": {
                "version": 1,
                "dark_theme": true,
                "language": "EN"
            },
            "selected_view": "Http",
            "show_settings": true,
            "http": {
                "show_sidebar": true,
                "current_workspace_idx": 0,
                "workspaces": [
                    {
                        "id": 1,
                        "name": "Workspace 1",
                        "show_options": true,
                        "requests": [
                            {
                                "name": "Request 1",
                                "method": "POST",
                                "url": "https://example.com/api",
                                "multipart": true,
                                "body_params": [
                                    ["param1", "value1", true],
                                    ["param2", "value2", false]
                                ],
                                "headers_params": [
                                    ["Content-Type", "application/json"],
                                    ["Authorization", "Bearer token"]
                                ]
                            }
                        ]
                    }
                ]
            },
            "pg": {
                "show_sidebar": false,
                "performance_table": true,
                "connections": [
                    {
                        "name": "Pg Connection",
                        "host": "localhost",
                        "port": "5432",
                        "user": "user",
                        "password": "password",
                        "dbname": "database"
                    }
                ]
            },
            "mysql": {
                "show_sidebar": true,
                "performance_table": false,
                "connections": [
                    {
                        "name": "MySql Connection",
                        "host": "localhost",
                        "port": "3306",
                        "user": "user",
                        "password": "password",
                        "dbname": "database"
                    }
                ]
            },
            "sqlite": {
                "show_sidebar": true,
                "performance_table": false,
                "connections": [
                    {
                        "name": "SQLite Connection",
                        "path": "/path/to/db"
                    }
                ]
            },
            "redis": {
                "show_sidebar": true,
                "connections": [
                    {
                        "host": "localhost",
                        "port": "6379"
                    }
                ]
            },
            "mongo": {
                "show_sidebar": true,
                "performance_table": true,
                "connections": [
                    {
                        "name": "Mongo Connection",
                        "host": "localhost",
                        "port": "27017",
                        "user": "user",
                        "password": "password",
                        "is_srv": false
                    }
                ]
            },
            "kafka": {
                "show_sidebar": true,
                "connections": [
                    {
                        "name": "Kafka Cluster",
                        "host": "localhost",
                        "port": "9092"
                    }
                ]
            }
        });

        // Escribir el JSON a un archivo temporal
        let file_name = "test_state.json";
        std::fs::write(file_name, test_json.to_string()).expect("Unable to write test file");

        // Leer y adaptar el estado desde el archivo
        let app_state = read_state_and_adapt(file_name);

        // Verificar el contenido del estado deserializado
        assert_eq!(app_state.app_config.version, 1);
        assert_eq!(app_state.app_config.dark_theme, true);
        assert_eq!(app_state.app_config.language, I18nOptions::EN);
        assert_eq!(app_state.selected_view, ViewType::Http);
        assert!(app_state.show_settings);

        // Verificar el estado HTTP
        assert!(app_state.http.show_sidebar);
        assert_eq!(app_state.http.current_workspace_idx, 0);
        assert_eq!(app_state.http.workspaces.len(), 1);
        assert_eq!(app_state.http.workspaces[0].name, "Workspace 1");
        assert!(app_state.http.workspaces[0].show_options);
        assert_eq!(app_state.http.workspaces[0].requests.len(), 1);
        assert_eq!(app_state.http.workspaces[0].requests[0].name, "Request 1");
        assert_eq!(
            app_state.http.workspaces[0].requests[0].method,
            HttpMethod::Post
        );
        assert_eq!(
            app_state.http.workspaces[0].requests[0].url,
            "https://example.com/api"
        );
        assert!(app_state.http.workspaces[0].requests[0].multipart);
        assert_eq!(
            app_state.http.workspaces[0].requests[0].body_params.len(),
            2
        );
        assert_eq!(
            app_state.http.workspaces[0].requests[0].body_params[0],
            ("param1".to_string(), "value1".to_string(), true)
        );
        assert_eq!(
            app_state.http.workspaces[0].requests[0].body_params[1],
            ("param2".to_string(), "value2".to_string(), false)
        );
        assert_eq!(
            app_state.http.workspaces[0].requests[0]
                .headers_params
                .len(),
            2
        );
        assert_eq!(
            app_state.http.workspaces[0].requests[0].headers_params[0],
            ("Content-Type".to_string(), "application/json".to_string())
        );
        assert_eq!(
            app_state.http.workspaces[0].requests[0].headers_params[1],
            ("Authorization".to_string(), "Bearer token".to_string())
        );

        // Verificar el estado Pg
        assert!(!app_state.pg.show_sidebar);
        assert!(app_state.pg.performance_table);
        assert_eq!(app_state.pg.connections.len(), 1);
        assert_eq!(app_state.pg.connections[0].name, "Pg Connection");
        assert_eq!(app_state.pg.connections[0].host, "localhost");
        assert_eq!(app_state.pg.connections[0].port, "5432");
        assert_eq!(app_state.pg.connections[0].user, "user");
        assert_eq!(app_state.pg.connections[0].password, "password");
        assert_eq!(app_state.pg.connections[0].dbname, "database");

        // Verificar el estado MySql
        assert!(app_state.mysql.show_sidebar);
        assert!(!app_state.mysql.performance_table);
        assert_eq!(app_state.mysql.connections.len(), 1);
        assert_eq!(app_state.mysql.connections[0].name, "MySql Connection");
        assert_eq!(app_state.mysql.connections[0].host, "localhost");
        assert_eq!(app_state.mysql.connections[0].port, "3306");
        assert_eq!(app_state.mysql.connections[0].user, "user");
        assert_eq!(app_state.mysql.connections[0].password, "password");
        assert_eq!(app_state.mysql.connections[0].dbname, "database");

        // Verificar el estado SQLite
        assert!(app_state.sqlite.show_sidebar);
        assert!(!app_state.sqlite.performance_table);
        assert_eq!(app_state.sqlite.connections.len(), 1);
        assert_eq!(app_state.sqlite.connections[0].name, "SQLite Connection");
        assert_eq!(app_state.sqlite.connections[0].path, "/path/to/db");

        // Verificar el estado Redis
        assert!(app_state.redis.show_sidebar);
        assert_eq!(app_state.redis.connections.len(), 1);
        assert_eq!(app_state.redis.connections[0].host, "localhost");
        assert_eq!(app_state.redis.connections[0].port, "6379");

        // Verificar el estado Mongo
        assert!(app_state.mongo.show_sidebar);
        assert!(app_state.mongo.performance_table);
        assert_eq!(app_state.mongo.connections.len(), 1);
        assert_eq!(app_state.mongo.connections[0].name, "Mongo Connection");
        assert_eq!(app_state.mongo.connections[0].host, "localhost");
        assert_eq!(app_state.mongo.connections[0].port, "27017");
        assert_eq!(app_state.mongo.connections[0].user, "user");
        assert_eq!(app_state.mongo.connections[0].password, "password");
        assert!(!app_state.mongo.connections[0].is_srv);

        // Verificar el estado Kafka
        assert!(app_state.kafka.show_sidebar);
        assert_eq!(app_state.kafka.clusters.len(), 1);
        assert_eq!(app_state.kafka.clusters[0].name, "Kafka Cluster");
        assert_eq!(app_state.kafka.clusters[0].host, "localhost");
        assert_eq!(app_state.kafka.clusters[0].port, "9092");

        // Eliminar el archivo de prueba
        std::fs::remove_file(file_name).expect("Unable to delete test file");
    }

    #[test]
    fn test_read_state_with_missing_fields() {
        // JSON de prueba con campos faltantes
        let test_json = json!({
            "app_config": {
                "version": 1,
                "language": "EN"
                // Falta el campo dark_theme
            },
            // Falta el campo selected_view
            "show_settings": true,
            "http": {
                "show_sidebar": true,
                "workspaces": [
                    {
                        "name": "Workspace 1",
                        "show_options": true,
                        // Falta el campo id y requests
                    }
                ]
            },
            // Faltan todos los otros estados
        });

        // Escribir el JSON a un archivo temporal
        let file_name = "test_state_missing_fields.json";
        std::fs::write(file_name, test_json.to_string()).expect("Unable to write test file");

        // Leer y adaptar el estado desde el archivo
        let app_state = read_state_and_adapt(file_name);

        // Verificar que se usan los valores por defecto donde faltan los campos
        assert_eq!(app_state.app_config.version, 1);
        assert_eq!(app_state.app_config.dark_theme, false); // Valor por defecto
        assert_eq!(app_state.app_config.language, I18nOptions::EN);
        assert_eq!(app_state.selected_view, ViewType::default()); // Valor por defecto
        assert!(app_state.show_settings);

        // Verificar el estado HTTP
        assert!(app_state.http.show_sidebar);
        assert_eq!(app_state.http.current_workspace_idx, 0); // Valor por defecto
        assert_eq!(app_state.http.workspaces.len(), 1);
        assert_eq!(app_state.http.workspaces[0].name, "Workspace 1");
        assert!(app_state.http.workspaces[0].show_options);
        assert_eq!(app_state.http.workspaces[0].id, 0); // Valor por defecto
        assert!(app_state.http.workspaces[0].requests.is_empty()); // Valor por defecto

        // Verificar otros estados que deben estar en su valor por defecto
        assert_eq!(app_state.pg, PgAppState::default());
        assert_eq!(app_state.mysql, MySqlAppState::default());
        assert_eq!(app_state.sqlite, SQLiteAppState::default());
        assert_eq!(app_state.redis, RedisAppState::default());
        assert_eq!(app_state.mongo, MongoAppState::default());
        assert_eq!(app_state.kafka, KafkaAppState::default());

        // Eliminar el archivo de prueba
        std::fs::remove_file(file_name).expect("Unable to delete test file");
    }
}

