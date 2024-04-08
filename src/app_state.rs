// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use serde::{Deserialize, Serialize};

use crate::httpm::workspace::Workspace;
use crate::{common::internationalization::I18nOptions, httpm::state::HttpAppState};

#[derive(Clone, Deserialize, Serialize, Copy, PartialEq, Debug)]
pub enum ViewType {
    Http,
}

impl Default for ViewType {
    fn default() -> Self {
        ViewType::Http
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
    pub http: HttpAppState,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            app_config: AppConfig::default(),
            selected_view: ViewType::default(),
            show_settings: false,
            http: HttpAppState {
                show_sidebar: true,
                workspaces: vec![Workspace::default()],
                current_workspace_idx: 0,
            },
        }
    }
}
