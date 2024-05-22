// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use eframe::egui;
use std::collections::HashMap;
use tokio::runtime::Runtime;

use super::internationalization::I18n;

pub trait ShowVec {
    fn to_string_vec(&self) -> Vec<String>;
}

pub trait Show {
    fn to_string(&self) -> String;
}

pub trait Tree<K, V> {
    fn to_tree(&self) -> HashMap<K, V>;
}

pub trait Runner<T> {
    fn run() -> T;
}

// Para definir cómo se genera la url de conexión a partir
// de una definición de una conexión.
pub trait ToUrl {
    fn to_url(&self) -> String;
}

pub trait Sidenav<T> {
    fn show_sidenav(&mut self, rt: &Runtime, ctx: &egui::Context, app_st: &mut T, i18n: &I18n);
}

pub trait Create {
    fn create(config: &str) -> Self;
}
