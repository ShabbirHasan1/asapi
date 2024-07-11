// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use eframe::egui;
use rdkafka::metadata::Metadata;

use common::internationalization::I18n;
use components::heading_strong;

use crate::kafkam::view::KafkaView;

impl KafkaView {}

pub fn show_clusters_metadata_info(ui: &mut egui::Ui, metadata: &Metadata, i18n: &I18n) {
    ui.label(
        egui::RichText::new(&i18n.kafka_cluster_info)
            .heading()
            .strong(),
    );

    egui::Grid::new("kafka-clusters-info")
        .num_columns(2)
        .show(ui, |ui| {
            ui.monospace("#Broker");
            ui.label(metadata.brokers().len().to_string());
            ui.end_row();
            ui.monospace("#Topics");
            ui.label(metadata.topics().len().to_string());
            ui.end_row();
            ui.monospace("Metadata Broker");
            ui.end_row();
            ui.monospace(format!("\t{}", &i18n.kafka_name));
            ui.label(metadata.orig_broker_name());
            ui.end_row();
            ui.monospace("\tId");
            ui.label(metadata.orig_broker_id().to_string());
            ui.end_row();
        });

    heading_strong!(ui, "Broker");
    egui::Grid::new("kafka-clusters-broker-info")
        .num_columns(4)
        .show(ui, |ui| {
            for broker in metadata.brokers() {
                ui.monospace("\tId");
                ui.label(broker.id().to_string());
                ui.monospace("\tHost");
                ui.label(format!("\t{}:{}  ", broker.host(), broker.port(),));
                ui.end_row();
            }
        });
}

// pub fn show_cluster_configuration(ui: &mut egui::Ui, i18n: &I18n) {}
