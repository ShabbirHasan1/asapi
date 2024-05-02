// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use rdkafka::metadata::Metadata as ClusterMetadata;
use serde::{Deserialize, Serialize};

use super::presenter::KafkaMessage;

#[derive(Eq, PartialEq, Debug, Default)]
pub enum KafkaPanel {
    #[default]
    Brokers,
    Topics,
    Subscribe,
}

#[derive(Default, Serialize, Clone, Debug, Deserialize)]
pub struct Cluster {
    pub name: String,
    pub host: String,
    pub port: String,
}

#[derive(Default, Serialize, Clone, Debug, Deserialize)]
pub struct KafkaAppState {
    pub show_sidebar: bool,
    pub clusters: Vec<Cluster>,
}

// #[derive(Default)]
pub struct KafkaLocalState {
    pub tmp_cluster_config: Cluster,
    pub current_view: KafkaPanel,
    pub current_cluster_idx: usize,
    pub clusters_metadata: Vec<Option<ClusterMetadata>>,
    pub is_first_update: bool,
    pub tx: tokio::sync::mpsc::Sender<KafkaMessage>,
    pub rx: tokio::sync::mpsc::Receiver<KafkaMessage>,
    pub selected_cluster_to_edit_idx: Option<usize>,
}

impl Default for KafkaLocalState {
    fn default() -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(8);

        Self {
            tmp_cluster_config: Default::default(),
            current_view: KafkaPanel::default(),
            current_cluster_idx: usize::MAX,
            clusters_metadata: Vec::new(),
            is_first_update: true,
            selected_cluster_to_edit_idx: Default::default(),
            tx,
            rx,
        }
    }
}
