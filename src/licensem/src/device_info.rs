// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use common::run_shell_comand;

#[cfg(not(target_os = "macos"))]
pub fn device_info() -> (String, String, String) {
    let mac = match mac_address::get_mac_address() {
        Ok(Some(ma)) => format!("{ma}"),
        _ => "60:05:40:03:20:01".to_string(),
    };

    (host, mac, platform.to_owned())
}

#[cfg(target_os = "macos")]
pub fn device_info() -> (String, String, String) {
    let host = whoami::hostname();
    let platform = std::env::consts::OS;
    let mid_result = get_mid_result().unwrap();
    let additional_data = additional_data();

    (
        host,
        format!("{mid_result:} // {additional_data:?}"),
        platform.to_owned(),
    )
}

#[cfg(target_os = "macos")]
pub fn get_mid_result() -> Result<String, String> {
    use common::run_shell_comand;

    let system_profiler_output = run_shell_comand(
        "sh",
        [
            "-c",
            r#"system_profiler SPHardwareDataType SPSecureElementDataType"#,
        ],
    )?;

    let targets = [
        "Model Number",
        "Serial Number",
        "Hardware UUID",
        "Provisioning UDID",
        "SEID",
    ];

    let combined_string = process_output(&system_profiler_output, &targets);

    if combined_string.is_empty() {
        return Err("Combined identifier is empty.".to_string());
    }

    Ok(combined_string)
}

#[cfg(target_os = "macos")]
fn process_output(output_str: &str, targets: &[&str]) -> String {
    let mut result = Vec::new();

    for target in targets {
        for line in output_str.to_lowercase().lines() {
            if line.contains(&target.to_lowercase()) {
                let parts: Vec<&str> = line.split(":").collect();

                if parts.len() == 2 {
                    let value = parts[1].trim().to_string();

                    if !value.is_empty() {
                        result.push(value);
                    }
                }
            }
        }
    }

    result.join("|")
}

#[cfg(target_os = "macos")]
fn additional_data() -> String {
    let sysctl_data = sysctl_data().unwrap();
    let sysctl_lines: Vec<&str> = sysctl_data.trim().split('\n').collect();

    let username = sysctl_lines.get(0).unwrap().to_string();
    let hostname = sysctl_lines.get(1).unwrap().to_string();
    let chip = sysctl_lines.get(2).unwrap().to_string();
    let cpu_core_count: i8 = sysctl_lines.get(3).unwrap().parse().unwrap();

    format!("{username:} {hostname:} {chip:} {cpu_core_count}")
}

fn sysctl_data() -> Result<String, String> {
    let sysctl_output = run_shell_comand(
        "sh",
        [
            "-c",
            r#"whoami && sysctl -n kern.hostname machdep.cpu.brand_string hw.ncpu"#,
        ],
    )?;

    Ok(sysctl_output)
}
