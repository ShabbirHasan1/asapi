// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use eframe::egui;
use rdkafka::metadata::Metadata;

use crate::{common::internationalization::I18n, kafkam::view::KafkaView};

impl KafkaView {}

pub fn show_clusters_metadata_info(ui: &mut egui::Ui, metadata: &Metadata, i18n: &I18n) {
    ui.label(
        egui::RichText::new(&i18n.kafka_cluster_info)
            .heading()
            .strong(),
    );
    ui.label(format!("  #Broker: {}", metadata.brokers().len()));
    ui.label(format!("  #Topics: {}", metadata.topics().len()));
    ui.label(format!(
        "  Metadata Broker {}: {}",
        &i18n.kafka_name,
        metadata.orig_broker_name()
    ));
    ui.label(format!(
        "  Metadata Broker Id: {}\n",
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
