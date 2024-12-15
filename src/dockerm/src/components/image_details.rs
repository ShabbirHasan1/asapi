// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use eframe::egui;
use tokio::runtime::Runtime;

use common::I18nDocker;

use crate::{domain::DockerLocalState, view::DockerView};

impl DockerView {
    pub fn show(
        &mut self,
        _rt: &Runtime,
        _ctx: &egui::Context,
        _i18n: &I18nDocker,
        _local_st: &DockerLocalState,
    ) {
        // DockerImagePresenter::get_image_info(self.connection, image_name, image_info)
    }
}
