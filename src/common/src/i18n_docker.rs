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
}

impl I18nDocker {
    pub fn new_en() -> Self {
        I18nDocker {
            btn_connect: "Connect".to_string(),
        }
    }

    pub fn new_es() -> Self {
        I18nDocker {
            btn_connect: "Conectar".to_string(),
        }
    }
}
