// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use serde::{Deserialize, Serialize};

use crate::{common::internationalization::I18nOptions, mongom::state::MongoAppState};

#[derive(Clone, Deserialize, Serialize, Copy, PartialEq, Debug)]
pub enum ViewType {
    Mongo,
}

impl Default for ViewType {
    fn default() -> Self {
        ViewType::Mongo
    }
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
    pub mongo: MongoAppState,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            app_config: AppConfig::default(),
            selected_view: ViewType::default(),
            show_settings: false,
            mongo: MongoAppState::default(),
        }
    }
}
