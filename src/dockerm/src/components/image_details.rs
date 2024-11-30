// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use std::sync::Arc;

use eframe::egui;
use tokio::runtime::Runtime;

use common::I18nDocker;
use components::widgets::wrap_dark_gray_text;

use crate::presenter::{self, DockerImagePresenter, DockerPresenter};
use crate::{network_item, volume_item};
use crate::{
    domain::{DockerAppState, DockerLocalState},
    view::DockerView,
};

impl DockerView {
    pub fn show(
        &mut self,
        rt: &Runtime,
        ctx: &egui::Context,
        i18n: &I18nDocker,
        local_st: &DockerLocalState,
    ) {
        // DockerImagePresenter::get_image_info(self.connection, image_name, image_info)
    }
}
