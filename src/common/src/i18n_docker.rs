// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

#[derive(Clone)]
pub struct I18nDocker {
    pub btn_connect: String,
    pub name: String,
    pub images: String,
    pub image: String,
    pub image_id: String,
    pub ports: String,
    pub containers: String,
    pub volumes: String,
    pub networks: String,
    pub size: String,
    pub author: String,
    pub architecture: String,
    pub image_info: String,
    pub container_info: String,
    pub parent: String,
    pub created: String,
    pub logs: String,
    pub stats: String
}

impl I18nDocker {
    pub fn new_en() -> Self {
        I18nDocker {
            name: "Name".to_string(),
            btn_connect: "Connect".to_string(),
            image: "Image".to_string(),
            image_id: "Image ID".to_string(),
            images: "Images".to_string(),
            ports: "Ports".to_string(),
            containers: "Containers".to_string(),
            volumes: "Volumes".to_string(),
            networks: "Networks".to_string(),
            size: String::from("Size"),
            author: String::from("Author"),
            architecture: String::from("Architecture"),
            image_info: String::from("Basic Information"),
            container_info: String::from("Container Information"),
            parent: String::from("Parent"),
            created: String::from("Creado"),
            logs: String::from("Logs"),
            stats: String::from("Stats"),
        }
    }

    pub fn new_es() -> Self {
        I18nDocker {
            name: "Nombre".to_string(),
            btn_connect: "Conectar".to_string(),
            image: "Imagen".to_string(),
            image_id: "ID Imagen".to_string(),
            ports: "Puertos".to_string(),
            images: "Imágenes".to_string(),
            containers: "Contenedores".to_string(),
            volumes: "Volúmenes".to_string(),
            networks: "Redes".to_string(),
            size: String::from("Tamaño"),
            author: String::from("Autor"),
            architecture: String::from("Arquitectura"),
            image_info: String::from("Información Básica"),
            container_info: String::from("Información de Contenedor"),
            parent: String::from("Padre"),
            created: String::from("Creado"),
            logs: String::from("Logs"),
            stats: String::from("Estadísticas"),
        }
    }
}
