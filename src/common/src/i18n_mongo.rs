// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

#[derive(Clone)]
pub struct I18nMongo {
    pub mongo_connection_name: String,
    pub mongo_connection_host: String,
    pub mongo_connection_port: String,
    pub mongo_connection_user: String,
    pub mongo_connection_password: String,
    pub mongo_connection_srv: String,
    pub mongo_connection_timeout: String,
    pub mongo_actions: String,
    pub mongo_insert_one_error: String,
    pub mongo_invalid_doc_to_insert: String,
    pub mongo_clean_filter: String,
    pub mongo_clean_parent: String,
    pub mongo_previsualize_filter: String,
    pub mongo_connections: String,
    pub mongo_databases: String,
    pub mongo_collections: String,
    pub mongo_close_connection: String,
    pub mongo_delete_connection: String,
    pub mongo_copy_database_info: String,
    pub mongo_copy_collection_info: String,
    pub mongo_error_client_uninitialized: String,
    pub mongo_delete_filter: String,
    pub mongo_wrong_action: String,
    pub mongo_filter_heading: String,
    pub mongo_new_document_heading: String,
    pub mongo_doc_menu_copy: String,
    pub mongo_doc_menu_delete_by_id: String,
    pub mongo_edit_connection: String,
    pub add_connection: String,
    pub cancel: String,
    pub save: String,
}

impl I18nMongo {
    pub fn new_en() -> Self {
        I18nMongo {
            mongo_connection_name: String::from("Name"),
            mongo_connection_host: "Host".to_owned(),
            mongo_connection_port: "Port".to_owned(),
            mongo_connection_user: "User".to_owned(),
            mongo_connection_password: "Password".to_owned(),
            mongo_connection_srv: "SRV".to_owned(),
            mongo_connection_timeout: String::from("Databases listing Operation Timeout"),
            mongo_actions: "Select Action".to_owned(),
            mongo_insert_one_error: String::from("InsertOne only accepts one element"),
            mongo_invalid_doc_to_insert: String::from("Invalid document"),
            mongo_clean_filter: String::from("Clean Filter"),
            mongo_clean_parent: String::from("Delete Current Parent"),
            mongo_previsualize_filter: String::from("Filter Previsualization"),
            mongo_connections: String::from("Connections"),
            mongo_databases: String::from("Databases"),
            mongo_collections: String::from("Colecciones"),
            mongo_close_connection: String::from("Close Connection"),
            mongo_delete_connection: String::from("Delete Connection"),
            mongo_copy_database_info: String::from("Copy Database Info"),
            mongo_copy_collection_info: String::from("Copy Collection Info"),
            mongo_error_client_uninitialized: String::from("Client does not Exists"),
            mongo_delete_filter: String::from("Delete Filter"),
            mongo_wrong_action: String::from("Wrong Action"),
            mongo_filter_heading: String::from("Filter"),
            mongo_new_document_heading: String::from("New Document"),
            mongo_doc_menu_copy: String::from("Copy Document"),
            mongo_doc_menu_delete_by_id: String::from("Delete using _id"),
            mongo_edit_connection: String::from("Edit Connection"),
            add_connection: String::from("Add Connection"),
            cancel: String::from("Cancel"),
            save: String::from("Save"),
        }
    }

    pub fn new_es() -> Self {
        I18nMongo {
            mongo_connection_name: String::from("Nombre"),
            mongo_connection_host: "Host".to_owned(),
            mongo_connection_port: "Puerto".to_owned(),
            mongo_connection_user: "Usuario".to_owned(),
            mongo_connection_password: "Password".to_owned(),
            mongo_connection_srv: "SRV".to_owned(),
            mongo_connection_timeout: String::from(
                "La operación de listar bases de datos excedió el tiempo límite",
            ),
            mongo_actions: "Seleccionar Acción".to_owned(),
            mongo_insert_one_error: String::from("InsertOne acepta un único elemento"),
            mongo_invalid_doc_to_insert: String::from(
                "El Documento que se está intentando isnertar no es válido",
            ),
            mongo_clean_filter: String::from("Limpiar Filtro"),
            mongo_clean_parent: String::from("Borrar Padre Actual"),
            mongo_previsualize_filter: String::from("Previsualizar Filtro"),
            mongo_connections: String::from("Conexiones"),
            mongo_databases: String::from("Bases de Datos"),
            mongo_collections: String::from("Colecciones"),
            mongo_close_connection: String::from("Cerrar Conexión"),
            mongo_delete_connection: String::from("Borrar Conexión"),
            mongo_copy_database_info: String::from("Copiar Información de la Base de Datos"),
            mongo_copy_collection_info: String::from("Copiar Información de la Colección"),
            mongo_error_client_uninitialized: String::from("Cliente no Inicializado"),
            mongo_delete_filter: String::from("Borrar Filtro"),
            mongo_wrong_action: String::from("Acción Incorrecta"),
            mongo_filter_heading: String::from("Filtro"),
            mongo_new_document_heading: String::from("Nuevo Documento"),
            mongo_doc_menu_copy: String::from("Copiar Documento"),
            mongo_doc_menu_delete_by_id: String::from("Borrar usando _id"),
            mongo_edit_connection: String::from("Editar Conexión"),
            add_connection: String::from("Añadir Conexión"),
            cancel: String::from("Cancelar"),
            save: String::from("Guardar"),
        }
    }
}
