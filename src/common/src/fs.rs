// -------------------------------------------------------------------------
// Copyright (C) 2023 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

// use std::collections::HashSet;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

pub fn file_exists(fp: &str) -> bool {
    Path::exists(Path::new(fp))
}

#[derive(serde::Deserialize, Debug, Default)]
pub struct Version {
    pub version: u16,
}

#[derive(serde::Deserialize, Debug, Default)]
pub struct BaseAppConfig {
    pub app_config: Version,
}

pub fn append_to_file(file_path: &str, text: &str) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        // .write(true) // Innecesario por `append`.
        .append(true)
        .open(file_path)?;

    writeln!(file, "{}", text)?;
    Ok(())
}

// fn path_extension_validation(file: &Path, extensions: &HashSet<&str>) -> bool {
//     extensions.contains(
//         file.extension()
//             .and_then(|e| e.to_str())
//             .unwrap_or_default(),
//     )
// }

pub fn list_files_in_directory(dir: &Path) -> Vec<PathBuf> {
    fs::read_dir(dir).map_or_else(
        |_| vec![],
        |entries| {
            entries
                .flatten()
                .map(|p| p.path())
                .filter(|p| p.is_file())
                .collect::<Vec<PathBuf>>()
        },
    )
}
