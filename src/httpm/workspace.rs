// -------------------------------------------------------------------------
// Copyright (C) 2023 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use serde::{Deserialize, Serialize};

use super::methods::HttpMethod;

#[derive(Clone, Default, Serialize, Deserialize, Debug)]
pub struct Request {
    pub name: String,
    pub method: HttpMethod,
    pub url: String,
    pub body_params: Vec<(String, String)>,
    pub headers_params: Vec<(String, String)>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Workspace {
    pub id: usize,
    pub name: String,
    pub requests: Vec<Request>,
    pub show_options: bool,
}

impl Default for Workspace {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::from("Workspace 1"),
            requests: Vec::new(),
            show_options: false,
        }
    }
}
