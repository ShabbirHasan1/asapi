// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use eframe::egui;
use egui_extras::{Column, TableBuilder};
use rdkafka::Statistics;
use std::sync::Arc;
use tokio::runtime::Runtime;

use crate::{
    kafkam::{
        presenter::KafkaConsumer,
        state::{Cluster, KafkaConsumerMessage, KafkaPanel},
        view::KafkaView,
    },
    qk_error,
};

pub fn show_stats(ui: &mut egui::Ui, stats: &[Statistics]) {}
