// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use eframe::egui;
use tokio::sync::mpsc::{Receiver, Sender};

use super::{
    components::sidenav::ClickHouseSideNav, domain::ClickHouseMessage, state::ClickHouseState,
};

pub struct ClickHouseView {
    sidenav: ClickHouseSideNav,
    state: ClickHouseState,
    tx: Sender<ClickHouseMessage>,
    rx: Receiver<ClickHouseMessage>,

    tx_sync: std::sync::mpsc::Sender<ClickHouseMessage>,
    rx_sync: std::sync::mpsc::Receiver<ClickHouseMessage>,
}
