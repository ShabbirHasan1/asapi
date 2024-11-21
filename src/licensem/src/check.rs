use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use hex::{self};
use k256::ecdsa::{SigningKey, VerifyingKey};
use k256::elliptic_curve::generic_array::GenericArray;
use log::{error, info};
use pbkdf2::pbkdf2_hmac;
use secp256k1::Secp256k1;
use secp256k1::{PublicKey, SecretKey};
use serde::{Deserialize, Serialize};
use serde_json;
use sha2::Sha256;
use std::error::Error;
use std::num::NonZeroU32;

#[derive(Serialize, Deserialize, Debug, Default)]
struct License {
    device_name: String,
    platform: String,
    user_license: String,
    features: Option<Vec<String>>,
    expired_at: String, // podría usar un tipo de chrono pero prefiero hacerlo a mano y no dejar a serde.
    version: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct EncryptedLicense {
    iv: String,
    data: String,
    tag: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct EncryptedSignedLicense {
    license: EncryptedLicense,
    extra: (String, String),
}

fn create_signature(license: &EncryptedLicense, salt: &str, shared_key: &str) -> String {
    fn str_to_int(s: &str) -> u32 {
        s.bytes().map(|b| b as u32).sum()
    }

    let seed: String = license.data.chars().rev().collect();
    let mut output = [0u8; 32];
    let n = str_to_int(shared_key);
    let s = str_to_int(salt);

    log::info!("{n:} % {s:}");
    let n_times = n % s;

    pbkdf2_hmac::<Sha256>(
        seed.as_bytes(),
        salt.as_bytes(),
        n_times as u32,
        &mut output,
    );
    log::info!("{output:?}");
    output.iter().map(|b| format!("{:02x}", b)).collect()
}

fn generate_key_pair_from_seed(
    seed: &str,
    salt: &[u8],
) -> Result<(String, String), Box<dyn std::error::Error>> {
    // Definir constantes para PBKDF2
    let iterations = NonZeroU32::new(100_000).unwrap();
    // let saltd = b"salt";
    let mut hash = [0u8; 32];

    // Derivar la clave usando PBKDF2
    ring::pbkdf2::derive(
        ring::pbkdf2::PBKDF2_HMAC_SHA256,
        iterations,
        salt,
        seed.as_bytes(),
        &mut hash,
    );

    // Crear la clave privada desde el hash derivado
    // let signing_key = SigningKey::from_bytes(&hash)?;
    // Convertir el array de bytes a GenericArray<u8, U32>
    let key_bytes = GenericArray::clone_from_slice(&hash);

    // Crear la clave privada desde el hash derivado
    let signing_key = SigningKey::from_bytes(&key_bytes)?;
    let private_key_hex = hex::encode(signing_key.to_bytes());

    // Obtener la clave pública correspondiente
    let verifying_key = VerifyingKey::from(&signing_key);
    let public_key = verifying_key.to_encoded_point(false);
    let public_key_hex = hex::encode(public_key.as_bytes());

    Ok((public_key_hex, private_key_hex))
}

fn decrypt(shared_key_hex: &str, encrypted: &EncryptedLicense) -> Result<License, Box<dyn Error>> {
    let shared_key = hex::decode(shared_key_hex)?;
    let iv = hex::decode(&encrypted.iv)?;
    let encrypted_data = hex::decode(&encrypted.data)?;
    let tag = hex::decode(&encrypted.tag)?;

    // Combinar los datos encriptados y el tag
    let mut ciphertext_with_tag = encrypted_data;
    ciphertext_with_tag.extend_from_slice(&tag);

    let key = Key::<Aes256Gcm>::from_slice(&shared_key[0..32]);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&iv);

    match cipher.decrypt(nonce, ciphertext_with_tag.as_ref()) {
        Ok(decrypted) => {
            let decrypted_str = String::from_utf8(decrypted)?;
            let license: License = serde_json::from_str(&decrypted_str)?;

            Ok(license)
        }
        Err(e) => Err(format!("Error decrypting license: {:?}", e).into()),
    }
}

fn derive_shared_key(
    public_key_hex: &str,
    private_key_hex: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let private_key_bytes = hex::decode(private_key_hex)?;
    let public_key_bytes = hex::decode(public_key_hex)?;

    let private_key = SecretKey::from_slice(&private_key_bytes)?;
    let public_key = PublicKey::from_slice(&public_key_bytes)?;

    let secp = Secp256k1::new();

    let shared_point = public_key.mul_tweak(&secp, &private_key.into())?;

    let shared_secret = shared_point.serialize();
    let shared_key_hex = hex::encode(&shared_secret[1..33]); // Omitir el primer byte (identificador de compresión) y tomar los siguientes 32 bytes

    Ok(shared_key_hex[0..64].to_string())
}

fn verify_signature(
    license: &EncryptedLicense,
    server_signature: &str,
    salt: &str,
    shared_key: &str,
) -> bool {
    let client_signature = create_signature(&license, salt, shared_key);
    log::info!("{server_signature:} -- {client_signature:}");
    client_signature == server_signature
}

fn verify_expiration_date(license: &License) -> bool {
    let str_datetime = &license.expired_at;
    let dt = chrono::DateTime::parse_from_rfc3339(str_datetime);
    if dt.is_err() {
        return false;
    }
    let datetime = dt.unwrap();
    let now = chrono::Local::now();

    datetime.naive_utc().and_utc().timestamp_millis()
        >= now.naive_utc().and_utc().timestamp_millis()
}

pub fn device_info() -> (String, String, String) {
    let platform = std::env::consts::OS;
    let host = whoami::hostname();
    let mac = match mac_address::get_mac_address() {
        Ok(Some(ma)) => format!("{ma}"),
        _ => "60:05:40:03:20:01".to_string(),
    };

    (host, mac, platform.to_owned())
}

#[derive(Debug)]
//                    device      platform    public-key
pub struct DeviceInfo(pub String, pub String, pub String);

pub fn get_license_info_for_device_registration() -> Result<DeviceInfo, String> {
    let (host, mac, platform) = device_info();
    let seed = format!("{host}__{mac}__{platform}");
    match generate_key_pair_from_seed(&seed, b"saltggg198sd7urf") {
        Ok((public, _)) => Ok(DeviceInfo(host, platform, public)),
        Err(err) => Err(format!("{err:?}")),
    }
}

pub fn private_check_license(encrypted: &EncryptedSignedLicense, salt: &str, seed: &str) -> bool {
    // Creamos al vuelo claves y derivada.
    let (host, mac, platform) = device_info();
    log::info!("{host:}, {mac:}, {platform:}");
    // Calculamos compartida o creamos una errónea que fallará la primera comprobación.
    // let (client_public_key, client_private_key) = generate_key_pair_from_seed(seed, salt.as_bytes()).unwrap();

    let shared_key = generate_key_pair_from_seed(seed, salt.as_bytes())
        .and_then(|(_, client_private_key)| {
            log::info!("{client_private_key:}");
            derive_shared_key(&encrypted.extra.0, &client_private_key)
        })
        .map_or("wrong_shared_key".to_string(), |k| k);

    log::info!("shared_key: {shared_key:}");
    if !verify_signature(&encrypted.license, &encrypted.extra.1, salt, &shared_key) {
        return false;
    }

    match decrypt(&shared_key, &encrypted.license) {
        Ok(license) => {
            info!("License: {license:?}");
            verify_expiration_date(&license)
                && license.device_name == host
                && license.platform == platform
        }
        Err(err) => {
            error!("{err:?}");
            false
        }
    }
}
