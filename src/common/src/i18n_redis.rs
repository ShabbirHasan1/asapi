// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

#[derive(Clone)]
pub struct I18nRedis {
    pub redis_connections: String,
    pub redis_data_structures: String,
    pub redis_connection_host: String,
    pub redis_connection_port: String,
    pub redis_edit_connection_cancel: String,
    pub redis_edit_connection_save: String,
    pub redis_close_connection: String,
    pub redis_delete_connection: String,
    pub redis_channel: String,
    pub redis_channel_value: String,
    pub redis_channel_publish: String,
    pub redis_close_channel: String,
    pub redis_delete_channel: String,
    pub redis_channel_subscribe: String,
    pub redis_n_columns: String,
    pub redis_clean_messages: String,
    pub redis_close_subscription: String,
    pub redis_delete_subscription: String,
    pub redis_send_command: String,
    pub redis_delete_ds: String,
    pub redis_load: String,
    pub redis_stream_hover_info: String,
    pub redis_commands_header: String,
    pub redis_stream_reader_commands_header: String,
    pub redis_regular_commands: String,
    pub redis_read_commands: String,
    pub redis_clean_result: String,
    pub redis_stream_group_prefix: String,
    pub redis_stream_stream_prefix: String,
    pub redis_stream_block_prefix: String,
    pub add_connection: String,
}

impl I18nRedis {
    pub fn new_en() -> Self {
        I18nRedis {
            redis_connections: String::from("Connections"),
            redis_data_structures: String::from("Data Structures"),
            redis_edit_connection_cancel: String::from("Cancel"),
            redis_edit_connection_save: String::from("Save"),
            redis_connection_host: String::from("Host"),
            redis_connection_port: String::from("Port"),
            redis_close_connection: String::from("Edit Connection"),
            redis_delete_connection: String::from("Delete Connection"),
            redis_channel: String::from("Channel"),
            redis_close_channel: String::from("Close Channel"),
            redis_delete_channel: String::from("Delete Channel"),
            redis_channel_value: String::from("Value"),
            redis_channel_publish: String::from("Publish"),
            redis_channel_subscribe: String::from("Subscribe"),
            redis_n_columns: String::from("Number of Columns"),
            redis_clean_messages: String::from("Clear Messages"),
            redis_close_subscription: String::from("Close Subscription"),
            redis_delete_subscription: String::from("Delete Subscription"),
            redis_send_command: String::from("Send Command"),
            redis_delete_ds: String::from("Delete"),
            redis_load: String::from("Load"),
            redis_stream_hover_info: String::from("Click to Open Stream and enabling resend"),
            redis_commands_header: String::from("Comandos Disponibles"),
            redis_stream_reader_commands_header: String::from("XRead Commands"),
            redis_regular_commands: String::from("Regular Commands"),
            redis_read_commands: String::from("Read Commands"),
            redis_clean_result: String::from("Clean Result"),
            redis_stream_group_prefix: String::from("Group"),
            redis_stream_stream_prefix: String::from("Stream"),
            redis_stream_block_prefix: String::from("Blocking for"),
            add_connection: String::from("Add"),
        }
    }

    pub fn new_es() -> Self {
        I18nRedis {
            redis_connections: String::from("Conexiones"),
            redis_connection_host: String::from("Host"),
            redis_connection_port: String::from("Puerto"),
            redis_data_structures: String::from("Estructuras de Datos"),
            redis_edit_connection_cancel: String::from("Cancelar"),
            redis_edit_connection_save: String::from("Guardar"),
            redis_close_connection: String::from("Editar Conexión"),
            redis_delete_connection: String::from("Borrar Conexión"),
            redis_channel: String::from("Canal"),
            redis_close_channel: String::from("Cerrar Canal"),
            redis_delete_channel: String::from("Borrar Canal"),
            redis_channel_value: String::from("Valor"),
            redis_channel_publish: String::from("Publicar"),
            redis_channel_subscribe: String::from("Subscribir"),
            redis_n_columns: String::from("Número de Columnas"),
            redis_clean_messages: String::from("Limpiar Mensajes"),
            redis_close_subscription: String::from("Cerrar Subscripción"),
            redis_delete_subscription: String::from("Borrar Subscripción"),
            redis_send_command: String::from("Ejecutar"),
            redis_delete_ds: String::from("Borrar"),
            redis_load: String::from("Cargar"),
            redis_stream_hover_info: String::from("Clicka para abrir el Stream y reenviarlo"),
            redis_commands_header: String::from("Comandos Disponibles"),
            redis_stream_reader_commands_header: String::from("Comandos de Lectura"),
            redis_regular_commands: String::from("Comandos Regulares"),
            redis_read_commands: String::from("Comandos de Lectura"),
            redis_clean_result: String::from("Limpiar Resultado"),
            redis_stream_group_prefix: String::from("Grupo"),
            redis_stream_stream_prefix: String::from("Stream"),
            redis_stream_block_prefix: String::from("Bloqueado durante"),
            add_connection: String::from("Añadir"),
        }
    }
}
