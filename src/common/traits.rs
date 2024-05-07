// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use eframe::egui;
use tokio::runtime::Runtime;

use crate::app_state::AppState;

use super::internationalization::I18n;

pub trait ShowVec {
    fn to_string_vec(&self) -> Vec<String>;
}

pub trait Show {
    fn to_string(&self) -> String;
}

pub trait Runner<T> {
    fn run() -> T;
}

pub trait Sidenav<T> {
    fn show_sidenav(&mut self, rt: &Runtime, ctx: &egui::Context, app_st: &mut T, i18n: &I18n);
}

pub trait Create {
    fn create(config: &str) -> Self;
}
