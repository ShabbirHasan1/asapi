// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

pub struct I18nKafka {
    pub kafka_accept: String,
    pub kafka_cancel: String,
    pub kafka_btn_add_connection: String,
    pub kafka_edit_cluster: String,
    pub kafka_edit_cluster_name_label: String,
    pub kafka_edit_cluster_host_label: String,
    pub kafka_edit_cluster_port_label: String,
    pub kafka_edit_cluster_save: String,
    pub kafka_edit_cluster_cancel: String,
    pub kafka_btn_connect: String,
    pub kafka_btn_connected: String,
    pub kafka_btn_show_brokers: String,
    pub kafka_btn_show_subscription: String,
    pub kafka_btn_show_stats: String,
    pub kafka_btn_show_topics: String,
    pub kafka_cluster_info: String,
    pub kafka_name: String,
    pub kafka_n_messages_in_topic: String,
    pub kafka_n_partitions_in_topic: String,
    pub kafka_replication_factor: String,
    pub kafka_create_topic: String,
    pub kafka_delete_topic: String,
    pub kafka_new_topic_name_hint: String,
    pub kafka_new_topic_config: String,
    pub kafka_new_topic_n_partitions_hint: String,
    pub kafka_partitions_info: String,
    pub kafka_topics_info: String,
    pub kafka_topic_replication: String,
    pub kafka_last_update: String,
}

impl I18nKafka {
    pub fn new_en() -> Self {
        I18nKafka {
            kafka_accept: String::from("Aceptar"),
            kafka_cancel: String::from("Cancelar"),
            kafka_btn_add_connection: "Añadir Clúster".to_owned(),
            kafka_edit_cluster: String::from("Editar Clúster"),
            kafka_edit_cluster_name_label: "Nombre del Clúster".to_owned(),
            kafka_edit_cluster_host_label: "Host".to_owned(),
            kafka_edit_cluster_port_label: "Puerto".to_owned(),
            kafka_edit_cluster_save: "Guardar".to_owned(),
            kafka_edit_cluster_cancel: "Cancelar".to_owned(),
            kafka_btn_connect: "Conectar".to_owned(),
            kafka_btn_connected: "Conectado".to_owned(),
            kafka_btn_show_brokers: "Brokers".to_owned(),
            kafka_btn_show_subscription: "Subscripción".to_owned(),
            kafka_btn_show_stats: "Estadísticas".to_string(),
            kafka_btn_show_topics: "Topics".to_owned(),
            kafka_cluster_info: String::from("Información del Clúster"),
            kafka_name: String::from("Nombre"),
            kafka_n_messages_in_topic: String::from("Número de Mensajes"),
            kafka_n_partitions_in_topic: String::from("Número de Particiones"),
            kafka_replication_factor: String::from("Factor de Replicación"),
            kafka_create_topic: String::from("Crear Topics"),
            kafka_partitions_info: String::from("Información de Particiones"),
            kafka_topics_info: String::from("Resumen"),
            kafka_delete_topic: String::from("Borrar Topic"),
            kafka_new_topic_name_hint: String::from("Nombre"),
            kafka_new_topic_config: String::from("Configuración"),
            kafka_new_topic_n_partitions_hint: String::from("Particiones"),
            kafka_topic_replication: String::from(
                "Configuración de Replicación para un nuevo Topic",
            ),
            kafka_last_update: String::from("Última Actualización"),
        }
    }

    pub fn new_es() -> Self {
        I18nKafka {
            kafka_accept: String::from("Accept"),
            kafka_cancel: String::from("Cancel"),
            kafka_btn_add_connection: "Add Cluster".to_owned(),
            kafka_edit_cluster: String::from("Edit Cluster"),
            kafka_edit_cluster_name_label: "Cluster Name".to_owned(),
            kafka_edit_cluster_host_label: "Host".to_owned(),
            kafka_edit_cluster_port_label: "Port".to_owned(),
            kafka_edit_cluster_save: "Save".to_owned(),
            kafka_edit_cluster_cancel: "Cancel".to_owned(),
            kafka_btn_connect: "Connect".to_owned(),
            kafka_btn_connected: "Connected".to_owned(),
            kafka_btn_show_brokers: "Brokers".to_owned(),
            kafka_btn_show_subscription: "Subscription".to_owned(),
            kafka_btn_show_stats: "Stats".to_string(),
            kafka_btn_show_topics: "Topics".to_owned(),
            kafka_cluster_info: String::from("Cluster Info"),
            kafka_name: String::from("Name"),
            kafka_n_messages_in_topic: String::from("Number of Messages"),
            kafka_n_partitions_in_topic: String::from("Number of Partitions"),
            kafka_replication_factor: String::from("Replication Factor"),
            kafka_create_topic: String::from("Create Topics"),
            kafka_partitions_info: String::from("Partitions Information"),
            kafka_topics_info: String::from("Topic Info"),
            kafka_delete_topic: String::from("Delete Topic"),
            kafka_new_topic_name_hint: String::from("Name"),
            kafka_new_topic_config: String::from("Configuration"),
            kafka_new_topic_n_partitions_hint: String::from("Partitions"),
            kafka_topic_replication: String::from("Replication configuration for a new topic"),
            kafka_last_update: String::from("Last Update"),
        }
    }
}
