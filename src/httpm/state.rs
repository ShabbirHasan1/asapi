// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use std::default;
use std::path::PathBuf;

use egui_file_dialog::{DialogMode, DialogState, FileDialog};
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

#[derive(Eq, PartialEq, Default)]
pub enum HttpPanel {
    #[default]
    Regular,
    Performance,
}

#[derive(Default)]
pub struct HttpFileState {
    pub file_dialog: FileDialog,
    // pub selected_path: Option<PathBuf>,
    // pub selected_folder: Option<PathBuf>,
    // pub selected_file: Option<PathBuf>,
    pub files_in_selected_folder: Vec<PathBuf>,
    pub selected_mode: Option<DialogMode>,
    pub current_state: Option<DialogState>,
    pub must_read: bool,
}

#[derive(Default)]
pub enum HttpRequestAction {
    #[default]
    None,
    Rename,
    Delete,
    Update,
}
#[derive(Default)]
pub struct HttpLocalState {
    pub upload_files: bool,
    pub selected_request_idx: Option<usize>,
    pub has_request_some_change: bool,
    pub selected_request_action: HttpRequestAction,
    pub response_headers: HeaderMap,
    pub show_hide_json_response: bool,
    pub has_been_updated: bool,
    pub panel: HttpPanel,
    pub performance_panel: HttpPerformanceView,
    pub files: HttpFileState,
}

impl HttpFileState {
    pub fn reset(&mut self) {
        self.selected_mode = None;
        self.must_read = false;
        self.files_in_selected_folder.clear();
    }
}
