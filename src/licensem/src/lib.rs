use secp256k1::Secp256k1;
use serde::{Deserialize, Serialize};
use serde_json;
use sha2::Sha256;
use std::error::Error;
use secp256k1::{SecretKey, PublicKey};
use k256::ecdsa::{SigningKey, VerifyingKey};
use k256::elliptic_curve::generic_array::GenericArray;
use std::num::NonZeroU32;
use aes_gcm::{Aes256Gcm, Key, Nonce};
use hex::{self};
use aes_gcm::aead::{Aead, KeyInit};
use pbkdf2::pbkdf2_hmac;


#[derive(Serialize, Deserialize, Debug)]
struct License {
    computer_name: String,
    platform: String,
    user_license: String,
    features: Option<Vec<String>>,
    expired_at: String,
    version: String,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
struct EncryptedLicense {
    iv: String,
    data: String,
    tag: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct EncryptedSignedLicense {
    license: EncryptedLicense,
    signature: String,
}

fn create_signature(license: &EncryptedLicense, salt: &str, shared_key: &str) -> String {
    fn str_to_int(s: &str) -> u32 {
        s.bytes().map(|b| b as u32).sum()
    }

    let seed: String = license.data.chars().rev().collect();
    let mut output = [0u8; 32];
    let n = str_to_int(shared_key);
    let s = str_to_int(salt);
    let n_times = n % s;

    pbkdf2_hmac::<Sha256>(seed.as_bytes(), salt.as_bytes(), n_times as u32, &mut output);
    output.iter().map(|b| format!("{:02x}", b)).collect()
}

fn generate_key_pair_from_seed(seed: &str, salt: &[u8]) -> Result<(String, String), Box<dyn std::error::Error>> {
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

fn decrypt(
    shared_key_hex: &str,
    encrypted: &EncryptedLicense,
) -> Result<License, Box<dyn Error>> {
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
        Err(e) => {
            // eprintln!("Error decrypting license: {:?}", e);
            Err(format!("Error decrypting license: {:?}", e).into())
        }
    }
}


fn derive_shared_key(public_key_hex: &str, private_key_hex: &str) -> Result<String, Box<dyn std::error::Error>> {
    let private_key_bytes = hex::decode(private_key_hex)?;
    let public_key_bytes = hex::decode(public_key_hex)?;

    // println!("public_key_bytes {public_key_bytes:?}");
    // println!("private_key_bytes {private_key_bytes:?}");

    let private_key = SecretKey::from_slice(&private_key_bytes)?;
    let public_key = PublicKey::from_slice(&public_key_bytes)?;

    let secp = Secp256k1::new();

    let shared_point = public_key.mul_tweak(&secp, &private_key.into())?;

    let shared_secret = shared_point.serialize();
    let shared_key_hex = hex::encode(&shared_secret[1..33]); // Omitir el primer byte (identificador de compresión) y tomar los siguientes 32 bytes

    Ok(shared_key_hex[0..64].to_string())
}

fn verify_license(license: &EncryptedSignedLicense, salt: &str, shared_key: &str) -> bool {
    let signature = create_signature(&license.license, salt, shared_key);
    if signature != license.signature {
        return false;
    }

    let str_datetime = license.expired_at;
    let dt = chrono::DateTime::parse_from_rfc3339(&str_datetime);
    if dt.is_err() {
        return false;
    }
    let datetime = dt.unwrap();
    let now = chrono::Local::now();
}
