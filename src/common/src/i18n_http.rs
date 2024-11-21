// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use super::icon_moon::IconMoon;

#[derive(Clone)]
pub struct I18nHttp {
    pub http_btn_edit_ws_name: String,
    pub http_btn_delete_ws: String,
    pub http_btn_update_request: String,
    pub http_btn_send_request: String,
    pub http_send_to_http_performance: String,
    pub http_select_file: String,
    pub http_select_folder: String,
    pub http_clean_file_folder_selection: String,
    pub http_selected_files_prefix: String,
    pub http_multipart_help: String,
    pub http_context_menu_rename: String,
    pub http_context_menu_delete: String,
    pub http_context_menu_update: String,
    pub http_body_add_files: String,
    pub http_edit_request_name: String,
    pub http_save_request: String,
    pub http_import_swagger: String,
    pub http_swagger_json_limitation: String,
    pub http_request_method: String,
}

impl I18nHttp {
    pub fn new_es() -> Self {
        I18nHttp {
                http_btn_edit_ws_name: "Editar nombre del espacio de trabajo".to_owned(),
                http_btn_delete_ws: "Borrar espacio de trabajo".to_owned(),
                http_btn_update_request: "Actualizar".to_owned(),
                http_btn_send_request: "Lanzar Petición".to_owned(),
                http_send_to_http_performance: "Rendimiento de la Petición".to_owned(),
                http_select_file: format!("{} Subir Archivo", IconMoon::File.as_str()),
                http_select_folder: format!("{} Subir Carpeta", IconMoon::FolderOpen.as_str()),
                http_clean_file_folder_selection: String::from("Limpiar Selección"),
                http_selected_files_prefix: String::from("archivos seleccionados"),
                http_multipart_help: String::from("Seleccinar para enviar petición como form/multipart, pertmitiendo subida de archivos"),
                http_context_menu_rename: String::from("Renombrar"),
                http_context_menu_delete: String::from("Borrar"),
                http_context_menu_update: String::from("Actualizar"),
                http_body_add_files: String::from("Añadir Archivo(s)"),
                http_edit_request_name: String::from("Editar Nombre de la Petición"),
                http_save_request: String::from("Guardar Petición"),
                http_import_swagger: format!("Importar OpenAPI {}", IconMoon::Letteri),
                http_swagger_json_limitation: String::from("Solo JSON"),
                http_request_method: String::from("Método")
        }
    }

    pub fn new_en() -> Self {
        I18nHttp {
            http_btn_edit_ws_name: "Edit workspace name".to_owned(),
            http_btn_delete_ws: "Delete workspace".to_owned(),
            http_btn_update_request: "Update".to_owned(),
            http_btn_send_request: "Send Request".to_owned(),
            http_send_to_http_performance: "Request Performance".to_owned(),
            http_select_folder: format!("{} Upload Folder", IconMoon::FolderOpen.as_str()),
            http_select_file: format!("{} Upload File", IconMoon::File.as_str()),
            http_clean_file_folder_selection: String::from("Clean Selection"),
            http_selected_files_prefix: String::from("selected files"),
            http_multipart_help: String::from(
                "Selecting sends request as multipart, uploading files if selected",
            ),
            http_context_menu_rename: String::from("Rename"),
            http_context_menu_delete: String::from("Delete"),
            http_context_menu_update: String::from("Update"),
            http_body_add_files: String::from("Add File(s)"),
            http_edit_request_name: String::from("Edit Request Name"),
            http_save_request: String::from("Save Request"),
            http_import_swagger: format!("Import OpenAPI {}", IconMoon::Letteri),
            http_swagger_json_limitation: String::from("Only JSON allowed"),
            http_request_method: String::from("Method"),
        }
    }
}
