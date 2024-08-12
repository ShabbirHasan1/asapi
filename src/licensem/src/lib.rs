mod check;

use directories::ProjectDirs;
use log::{error, log};
use std::fs;
use std::path::PathBuf;
use crate::check::{EncryptedSignedLicense, device_info, private_check_license};

pub enum LicenseResult {
    None, // Hay definida carpeta de configuración de usuario pero no está creada. Se crea y ya. El usuario tendrá que activar.
    Wrong, // Licencia es errónea.
    Ok,   // Licencia es correcta. Solo este estado permite que se use la aplicación.
    Error(String), // No hay definida carpeta de configuración de usuario.
}

pub struct LicenceActivationInfo {
    user_license: String,
    device_name: String,
    platform: String,
    id: String, // Clave pública
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
                return LicenseResult::Wrong;
            }
            let encrypted_license =
                serde_json::from_str::<EncryptedSignedLicense>(&json_data.unwrap());
            if encrypted_license.is_err() {
                return LicenseResult::Wrong;
            }

            let (host, mac, platform) = device_info();
            let is_valid = private_check_license(&encrypted_license.unwrap(), "saltggg198sd7urf", &format!("{host}__{mac}__{platform}"));

            if is_valid {
                return LicenseResult::Ok;
            } else {
                return LicenseResult::Wrong;
            }
        }
    } else {
        let msg = "No se pudo determinar el directorio de configuración.".to_string();
        error!("{msg}");
        return LicenseResult::Error(msg);
    }
}
