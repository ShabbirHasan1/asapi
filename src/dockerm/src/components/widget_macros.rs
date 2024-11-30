// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

// Importar aquí no sirve de nada, se necesita que esté importado en el punto donde se usa.
// use components::widgets::wrap_black_text;

#[macro_export]
macro_rules! network_item {
    ($ui:expr, $network:expr, $i18n:expr) => {{
        egui::CollapsingHeader::new(egui::RichText::new(&$network.name).monospace().size(12.0))
            .show($ui, |ui| {
                let job = wrap_dark_gray_text(format!("ID: {:}", $network.id));
                ui.label(job);
                ui.label(format!("#{}: {}", $i18n.containers, $network.n_containers));
            });
    }};
}

#[macro_export]
macro_rules! volume_item {
    ($ui:expr, $volume:expr, $line_width:expr, $i18n:expr) => {{
        let volume_name = if $volume.name.len() > $line_width {
            &format!("{:}...", &$volume.name[..$line_width])
        } else {
            &$volume.name
        };

        egui::CollapsingHeader::new(egui::RichText::new(volume_name).monospace().size(12.0)).show(
            $ui,
            |ui| {
                ui.label(format!(
                    "{}: {}",
                    $i18n.size,
                    $volume.usage_data.as_ref().map_or(0, |e| e.size)
                ));
            },
        );
    }};
}

#[macro_export]
macro_rules! info_table_row {
    ($body:expr, $field:expr, $value:expr) => {{
        $body.row(24.0, |mut row| {
            row.col(|ui| {
                ui.strong($field);
            });
            row.col(|ui| {
                ui.label($value);
            });
        })
    }};
}
