// -------------------------------------------------------------------------
// Copyright (C) 2023 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use std::fs;
use std::io::{Error as IOError, ErrorKind};
use tokio::fs as async_fs;
use std::fs::OpenOptions;
use std::io::Write;

use crate::app_state::AppState;

pub async fn async_save_state(state: &AppState, file_name: &str) -> Result<(), IOError> {
    let json_string = serde_json::to_string_pretty(state).map_err(|err| {
        IOError::new(
            ErrorKind::InvalidData,
            format!("Failed to serialize data: {}", err),
        )
    })?;
    async_fs::write(file_name, json_string).await?;
    Ok(())
}

pub fn save_state(state: &AppState, file_name: &str) -> Result<(), IOError> {
    let json_string = serde_json::to_string_pretty(state).map_err(|err| {
        IOError::new(
            ErrorKind::InvalidData,
            format!("Failed to serialize data: {}", err),
        )
    })?;
    fs::write(file_name, json_string)?;
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
        .write(true)
        .append(true)
        .open(file_path)?;

    writeln!(file, "{}", text)?;
    Ok(())
}
