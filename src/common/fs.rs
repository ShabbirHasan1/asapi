// -------------------------------------------------------------------------
// Copyright (C) 2023 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use std::collections::HashSet;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::io::{Error as IOError, ErrorKind};
use std::path::{Path, PathBuf};
use tokio::fs as async_fs;

use crate::app_state::AppState;

pub fn file_exists(fp: &str) -> bool {
    Path::exists(Path::new(fp))
}

pub async fn async_save_state(
    state: &AppState,
    file_name: &str,
    save_bak: bool,
) -> Result<(), IOError> {
    let json_string = serde_json::to_string_pretty(state).map_err(|err| {
        IOError::new(
            ErrorKind::InvalidData,
            format!("Failed to serialize data: {}", err),
        )
    })?;
    if save_bak {
        async_fs::copy(file_name, format!("{file_name}.bak")).await?;
    }
    async_fs::write(file_name, json_string).await?;
    Ok(())
}

pub fn save_state(state: &AppState, file_name: &str, save_bak: bool) -> Result<(), IOError> {
    let json_string = serde_json::to_string_pretty(state).map_err(|err| {
        IOError::new(
            ErrorKind::InvalidData,
            format!("Failed to serialize data: {}", err),
        )
    })?;
    if save_bak {
        fs::copy(file_name, format!("{file_name}.bak"))?;
    }
    Ok(())
}

pub fn load_state(file_name: &str) -> Result<AppState, IOError> {
    let json_data = fs::read_to_string(file_name)?;
    let state: AppState = serde_json::from_str(&json_data).map_err(|err| {
        IOError::new(
            ErrorKind::InvalidData,
            format!("Failed to deserialize data: {}", err),
        )
    })?;

    Ok(state)
}

pub fn append_to_file(file_path: &str, text: &str) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        // .write(true) // Innecesario por `append`.
        .append(true)
        .open(file_path)?;

    writeln!(file, "{}", text)?;
    Ok(())
}

fn path_extension_validation(file: &Path, extensions: &HashSet<&str>) -> bool {
    extensions.contains(
        file.extension()
            .and_then(|e| e.to_str())
            .unwrap_or_default(),
    )
}

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
