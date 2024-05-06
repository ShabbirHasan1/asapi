// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use rdkafka::metadata::Metadata;
use rdkafka::{error::KafkaError, metadata::Metadata as ClusterMetadata};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

type KafkaTopicsCount = HashMap<String, i64>;

// =================================
// Dominio
// =================================
pub struct KafkaConsumerMessage {
    pub key: String,
    pub topic: String,
    pub offset: String,
    pub timestamp: String,
    pub partition: String,
    pub payload: String,
}

pub enum KafkaMessage {
    Str(String),
    // ConsumerMessage(KafkaMessageBody),
    ClusterMetadata((usize, Metadata, KafkaTopicsCount)),
    Error(KafkaError),
}

#[derive(Eq, PartialEq, Debug, Default)]
pub enum KafkaPanel {
    Brokers,
    Topics,
    Subscribe,
    #[default]
    Stats,
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

#[derive(Default)]
pub struct KafkaNewTopic<'a> {
    pub show: bool,
    pub name: String,
    pub n_partitions: i32,
    pub fixed_topic_replication: i32,
    pub raw_config: String,
    pub parsed_config: Vec<(&'a str, &'a str)>,
}

pub struct KafkaLocalState {
    pub tmp_cluster_config: Cluster,
    pub current_view: KafkaPanel,
    pub current_cluster_idx: usize,
    pub current_cluster_metadata: Option<ClusterMetadata>,
    pub clusters_metadata_count: KafkaTopicsCount,
    pub is_first_update: bool,
    pub tx: tokio::sync::mpsc::Sender<KafkaMessage>,
    pub rx: tokio::sync::mpsc::Receiver<KafkaMessage>,
    pub selected_cluster_to_edit_idx: Option<usize>,
    pub new_topic: KafkaNewTopic<'static>,
    pub last_error: Option<KafkaError>,
}

impl Default for KafkaLocalState {
    fn default() -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(8);

        Self {
            tmp_cluster_config: Default::default(),
            current_view: KafkaPanel::default(),
            current_cluster_idx: usize::MAX,
            current_cluster_metadata: Default::default(),
            clusters_metadata_count: Default::default(),
            is_first_update: true,
            selected_cluster_to_edit_idx: Default::default(),
            new_topic: Default::default(),
            last_error: None,
            tx,
            rx,
        }
    }
}
