mod check;

pub use crate::check::{get_license_info_for_device_registration, device_info};
use crate::check::{private_check_license, EncryptedSignedLicense};
use check::DeviceInfo;
use directories::ProjectDirs;
use httpm::request::api_request;
use log::error;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, PartialEq)]
pub enum LicenseResult {
    None, // Hay definida carpeta de configuración de usuario pero no está creada. Se crea y ya. El usuario tendrá que activar.
    Wrong(String), // Licencia es errónea.
    Ok,   // Licencia es correcta. Solo este estado permite que se use la aplicación.
    Error(String), // No hay definida carpeta de configuración de usuario.
}

pub struct LicenseActivationInfo {
    pub user_license: String,
    pub device_info: DeviceInfo,
}


pub fn check_license_file() -> LicenseResult {
    if let Some(proj_dirs) = ProjectDirs::from("es", "qoback", "Asapi") {
        proj_dirs.config_dir();
        // Lin: /home/alice/.config/barapp
        // Win: C:\Users\Alice\AppData\Roaming\Foo Corp\Bar App\config
        // Mac: /Users/Alice/Library/Application Support/com.Foo-Corp.Bar-App
        // Obtener el directorio de configuración específico para esta aplicación
        let config_dir: PathBuf = proj_dirs.config_dir().to_path_buf();

        // Crear el directorio si no existe
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)
                .expect("No se pudo crear el directorio de configuración");
            LicenseResult::None
        } else {
            let config_file = config_dir.join("license.json");
            let json_data = fs::read_to_string(config_file);
            if json_data.is_err() {
                return LicenseResult::Wrong("Invalid License. No JSON.".to_string());
            }
            let encrypted_license =
                serde_json::from_str::<EncryptedSignedLicense>(&json_data.unwrap());
            if encrypted_license.is_err() {
                return LicenseResult::Wrong("Invalid License. Wrong JSON format.".to_string());
            }

            let (host, mac, platform) = device_info();
            let is_valid = private_check_license(
                &encrypted_license.unwrap(),
                "saltggg198sd7urf",
                &format!("{host}__{mac}__{platform}"),
            );

            if is_valid {
                return LicenseResult::Ok;
            } else {
                return LicenseResult::Wrong("Invalid License.".to_string());
            }
        }
    } else {
        let msg = "No se pudo determinar el directorio de configuración.".to_string();
        error!("{msg}");
        return LicenseResult::Error(msg);
    }
}

pub async fn post_license(license: LicenseActivationInfo) {
    let result = api_request(
        httpm::methods::HttpMethod::Post,
        "https://asapi.qoback.es/api/v1/license/create-device-license",
        &vec![
            ("user_license".to_string(), license.user_license, false),
            ("device_name".to_string(), license.device_info.0, false),
            ("platform".to_string(), license.device_info.1, false),
            ("id".to_string(), license.device_info.2.clone(), false),
        ],
        &vec![("id".to_string(), license.device_info.2)],
    )
    .await;

    match result {
        Ok((response, _headers)) => {
            log::info!("{response}");
        }
        Err(error) => {
            log::error!("{error}");
        }
    }
}
