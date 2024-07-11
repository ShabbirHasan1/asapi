// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use eframe::egui;

use super::state::SqlState;

pub trait Show {
    fn show(ui: &mut egui::Ui, state: &mut SqlState);
}

// Para definir cómo se genera la url de conexión a partir
// de una definición de una conexión.
pub trait ToUrl {
    fn to_url(&self) -> String;
}

pub trait Presenter {
    fn should_be_wrapped(&self, col_type: &str) -> bool;

    fn should_be_added_to_delete_stmt(&self, col_type: &str) -> bool;
}

pub trait ShowVec {
    fn to_string_vec(&self) -> Vec<String>;
}
