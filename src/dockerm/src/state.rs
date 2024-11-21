// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use bollard::secret::ImageSummary;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Default)]
pub struct DockerLocalState {
    pub images: Arc<Mutex<Vec<ImageSummary>>>,
}

#[derive(Default, Serialize, Clone, Debug, Deserialize)]
pub struct DockerAppState {
    pub show_sidebar: bool,
}
