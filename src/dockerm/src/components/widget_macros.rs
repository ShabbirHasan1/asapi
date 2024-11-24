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
macro_rules! image_item {
    ($ui:expr, $repo_tags:expr, $image:expr, $i18n:expr) => {{
        let parts: Vec<&str> = $repo_tags.split(':').collect();
        let (image_name, image_tag) = match &parts[..] {
            [name] => (*name, ""), // No tag present
            [name, tag] => (*name, *tag),
            _ => return, // continue antes de ser macro, // Formato inválido
        };

        egui::CollapsingHeader::new(egui::RichText::new(image_name).monospace().size(12.0)).show(
            $ui,
            |ui| {
                let job = wrap_dark_gray_text(format!("ID: {:}", $image.id));
                ui.label(job);
                ui.label(format!("Tag: {}", image_tag));
                ui.label(format!("{}: {} MB", $i18n.size, $image.size / 1048576));
            },
        );
    }};
}

#[macro_export]
macro_rules! container_item {
    ($ui:expr, $container:expr, $image_title:expr) => {{
        let id = $ui.make_persistent_id(&$container.name);
        let color = if $container.state == "running" {
            eframe::epaint::Color32::DARK_GREEN
        } else {
            eframe::epaint::Color32::DARK_GRAY
        };
        // No hace falta tanto, con `CollapsingHeader` es suficiente, lo dejo como ejemplo
        // porque es mucho más flexible.
        egui::collapsing_header::CollapsingState::load_with_default_open($ui.ctx(), id, true)
            .show_header($ui, |ui| {
                ui.label(
                    egui::RichText::new(&$container.name)
                        .color(color)
                        .monospace()
                        .size(12.0),
                );
            })
            .body(|ui| {
                let job = wrap_dark_gray_text(format!("ID: {:}", $container.id));
                ui.label(job);
                ui.label(format!("{}: {}", $image_title, $container.image));
            });
    }};
}

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

        egui::CollapsingHeader::new(
            egui::RichText::new(volume_name).monospace().size(12.0),
        )
            .show($ui, |ui| {
                ui.label(format!(
                    "{}: {}",
                    $i18n.size,
                    $volume.usage_data.as_ref().map_or(0, |e| e.size)
                ));
            });
    }};
}
