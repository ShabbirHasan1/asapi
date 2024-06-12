// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use eframe::egui;
use tokio::sync::mpsc::{Sender, Receiver};

use super::domain::ClickHouseMessage;

pub struct ClickHouseView {
    sidenav: ClickHouseSidenav,
    state: ClickHouseState,
    tx: Sender<ClickHouseMessage>,
    rx: Receiver<ClickHouseMessage>,
}
