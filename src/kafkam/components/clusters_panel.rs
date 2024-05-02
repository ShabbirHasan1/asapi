// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use eframe::egui;
use rdkafka::metadata::Metadata;

use crate::kafkam::view::KafkaView;

impl KafkaView {}

pub fn show_clusters_metadata_info(ui: &mut egui::Ui, metadata: &Metadata) {
    ui.label("Cluster information:".to_string());
    ui.label(format!("  Broker count: {}", metadata.brokers().len()));
    ui.label(format!("  Topics count: {}", metadata.topics().len()));
    ui.label(format!(
        "  Metadata broker name: {}",
        metadata.orig_broker_name()
    ));
    ui.label(format!(
        "  Metadata broker id: {}\n",
        metadata.orig_broker_id()
    ));

    ui.label("Brokers:");
    for broker in metadata.brokers() {
        ui.label(format!(
            "  Id: {}  Host: {}:{}  ",
            broker.id(),
            broker.host(),
            broker.port(),
        ));
    }
}
