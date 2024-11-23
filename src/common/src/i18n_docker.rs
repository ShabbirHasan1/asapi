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
    pub containers: String,
    pub volumes: String,
    pub networks: String,
    pub size: String,
}

impl I18nDocker {
    pub fn new_en() -> Self {
        I18nDocker {
            name: "Name".to_string(),
            btn_connect: "Connect".to_string(),
            image: "Image".to_string(),
            images: "Images".to_string(),
            containers: "Containers".to_string(),
            volumes: "Volumes".to_string(),
            networks: "Networks".to_string(),
            size: String::from("Size"),
        }
    }

    pub fn new_es() -> Self {
        I18nDocker {
            name: "Nombre".to_string(),
            btn_connect: "Conectar".to_string(),
            image: "Imagen".to_string(),
            images: "Imágenes".to_string(),
            containers: "Contenedores".to_string(),
            volumes: "Volúmenes".to_string(),
            networks: "Redes".to_string(),
            size: String::from("Tamaño"),
        }
    }
}
