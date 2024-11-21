// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

#[derive(Clone)]
pub struct I18nConfig {
    pub insert_license: String,
    pub activate_license_button: String,
    pub license_info_error: String,

    // intern: I18nOptions,
    pub debug_json_string: String,

    // Barra superior
    pub config_experimental_features: String,
    pub top_menu_config: String,
    pub top_http_toggle_sidebar: String,
    pub top_redis_toggle_sidebar: String,
    pub top_pg_toggle_sidebar_connections: String,
    pub top_pg_toggle_sidebar_tables: String,
    pub top_mongo_toggle_sidebar_connections: String,
    pub top_kafka_toggle_sidebar_cluster: String,
    pub top_sqlite_toggle_sidebar_connections: String,
    pub top_mysql_toggle_sidebar_connections: String,
    pub top_clickhouse_toggle_sidebar_connections: String,
    pub top_rabbitmq_toggle_sidebar_connections: String,
    pub top_nats_toggle_sidebar_connections: String,
    pub top_docker_toggle_sidebar_connections: String,
    pub top_import_json_state: String,
    pub top_export_json_state: String,
    pub top_export_warning: String,
}

impl I18nConfig {
    pub fn new_es() -> Self {
        I18nConfig {
            insert_license: String::from("Por favor, introduzca su licencia"),
            activate_license_button: String::from("Registrar Dispositivo"),
            license_info_error: String::from("Error al obtener información para el registro"),
            debug_json_string: "JSON Exportado".to_owned(),

            config_experimental_features: String::from("Módulos Experimentales"),
            top_menu_config: "Configuración".to_owned(),
            top_http_toggle_sidebar: "Colapsar/Mostrar Menú Lateral".to_owned(),
            top_redis_toggle_sidebar: "Colapsar/Mostrar Menú Lateral".to_owned(),
            top_pg_toggle_sidebar_connections: "Colapsar/Mostrar Menú Lateral".to_owned(),
            top_pg_toggle_sidebar_tables: "Colapsar/Mostrar Tablas".to_owned(),
            top_sqlite_toggle_sidebar_connections: "Colapsar/Mostrar Menú Lateral".to_owned(),
            top_mysql_toggle_sidebar_connections: "Colapsar/Mostrar Menú Lateral".to_owned(),
            top_mongo_toggle_sidebar_connections: "Colapsar/Mostrar Menú Lateral".to_owned(),
            top_kafka_toggle_sidebar_cluster: "Colapsar/Mostar Clústers".to_owned(),
            top_clickhouse_toggle_sidebar_connections: "Colapsar/Mostar Menú Lateral".to_owned(),
            top_rabbitmq_toggle_sidebar_connections: String::from("Colapsar/Mostrar Menú Lateral"),
            top_nats_toggle_sidebar_connections: String::from("Colapsar/Mostrar Menú Lateral"),
            top_docker_toggle_sidebar_connections: String::from("Colapsar/Mostrar Menú Lateral"),
            top_import_json_state: "Importar estado desde JSON".to_owned(),
            top_export_json_state: "Exportar a JSON".to_owned(),
            top_export_warning: "Exportar sobreescribirá los datos que tenga guardados actualmente"
                .to_owned(),
        }
    }

    pub fn new_en() -> Self {
        I18nConfig {
            insert_license: String::from("Please, insert your license"),
            activate_license_button: String::from("Register Device"),
            license_info_error: String::from("Error trying to obtain registration info"),
            debug_json_string: "Exported JSON".to_owned(),

            // Barra superior
            config_experimental_features: String::from("Experimental Features"),
            top_menu_config: "Config".to_owned(),
            top_http_toggle_sidebar: "Collapse/Show Sidebar".to_owned(),
            top_redis_toggle_sidebar: "Collapse/Show Sidebar".to_owned(),
            top_pg_toggle_sidebar_connections: "Collapse/Show Sidebar".to_owned(),
            top_pg_toggle_sidebar_tables: "Collapse/Show Tables".to_owned(),
            top_sqlite_toggle_sidebar_connections: "Collapse/Show Sidebar".to_owned(),
            top_mysql_toggle_sidebar_connections: "Collapse/Show Sidebar".to_owned(),
            top_mongo_toggle_sidebar_connections: "Collapse/Show Sidebar".to_owned(),
            top_kafka_toggle_sidebar_cluster: "Collapse/Show Clusters".to_owned(),
            top_clickhouse_toggle_sidebar_connections: "Collapse/Show Sidebar".to_owned(),
            top_rabbitmq_toggle_sidebar_connections: String::from("Collapse/Show Sidebar"),
            top_nats_toggle_sidebar_connections: String::from("Collapse/Show Sidebar"),
            top_docker_toggle_sidebar_connections: String::from("Collapse/Show Sidebar"),
            top_import_json_state: "Import state from JSON".to_owned(),
            top_export_json_state: "Export to JSON".to_owned(),
            top_export_warning: "Export will override data currently stored".to_owned(),
        }
    }
}
