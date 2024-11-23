// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use eframe::egui;
use tokio::runtime::Runtime;

use bollard::Docker;
use common::I18nDocker;

use crate::state::{DockerAppState, DockerLocalState};

#[derive(Default)]
pub struct DockerView {
    pub state: DockerLocalState,
    pub flag: bool,
    pub connection: Option<Docker>,
}

impl DockerView {
    pub fn update(
        &mut self,
        ctx: &egui::Context,
        rt: &Runtime,
        app_st: &mut DockerAppState,
        i18n: &I18nDocker,
    ) {
        // =======================================
        // Preparación de cada ciclo
        // =======================================

        // =======================================
        // Panel Lateral
        // =======================================

        if app_st.show_sidebar {
            self.show_sidenav(rt, ctx, i18n);
        }

        // =======================================
        // Panel Central
        // =======================================
        // self.show_sidenav(rt, ctx, app_st, i18n.docker);
        self.flag = true;

        // =======================================
        // Preparación de cada ciclo
        // =======================================
    }
}
