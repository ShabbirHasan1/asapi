// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use bollard::secret::{ImageSummary, Volume};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use crate::domain::{ContainerInfo, NetworkInfo};


#[derive(Default)]
pub struct DockerLocalState {
    pub images: Arc<Mutex<Vec<ImageSummary>>>,
    pub containers: Arc<Mutex<Vec<ContainerInfo>>>,
    pub volumes: Arc<Mutex<Vec<Volume>>>,
    pub networks: Arc<Mutex<Vec<NetworkInfo>>>,
}

#[derive(Default, Serialize, Clone, Debug, Deserialize)]
pub struct DockerAppState {
    pub show_sidebar: bool,
}
