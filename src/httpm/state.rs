// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};

use super::performance::view::HttpPerformanceView;
use super::workspace::Workspace;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct HttpAppState {
    pub show_sidebar: bool,
    pub workspaces: Vec<Workspace>,
    pub current_workspace_idx: usize,
}

#[derive(Eq, PartialEq)]
pub enum HttpPanel {
    Regular,
    Performance,
}

impl Default for HttpPanel {
    fn default() -> Self {
        HttpPanel::Regular
    }
}

#[derive(Default)]
pub struct HttpLocalState {
    pub selected_request_idx: Option<usize>,
    pub has_request_some_change: bool,
    pub selected_request_action: Option<String>,
    pub response_headers: HeaderMap,
    pub show_hide_json_response: bool,
    pub has_been_updated: bool,
    pub panel: HttpPanel,
    pub performance_panel: HttpPerformanceView,
}
