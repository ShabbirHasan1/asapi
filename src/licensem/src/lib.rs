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



#[derive(Serialize, Deserialize, Debug)]
struct License {
    computerName: String,
    platform: String,
    userLicense: String,
    features: Option<Vec<String>>,
    expiredAt: String,
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
    signature == license.signature
}

#[cfg(test)]
mod tests {
    use super::*;

    fn license() -> EncryptedSignedLicense {
        let encrypted_license = EncryptedLicense {
            iv: "fb38fc467a69b7a10119b390".to_string(),
            data: String::from("6c7464e685a5b9fe2cc9fa6c6d222a2f986b0ee077e7e3825a1d0ce0aaeb338a8dbf7941be8c0f87e5c8daf9b7411e2f58753bf65ace045301d7b5b4d2507cd4f9a3e0aa40fcf0fd052d597544b78bf9e53c03e355e7330b20d9667d560e12d1c851351d137ce897ed8f37fb165407bbf925534e6ce9d4ca38fbe6b1c46696ecf04fb77db3bcad626b6fb93bdfbcbe5e6ef5d2cbb97348b98170e10f184ae301b2"),
            tag: String::from("f8b4210286df0e83eeb6e1cb4c8c61cd")
        };
        let signature = "34afe37d2b6ed07c56459b49384e19010189b141268ef5d490eafe34128ba444".to_string();

        EncryptedSignedLicense {
            license: encrypted_license.clone(),
            signature,
        }
    }

    fn old_license() -> EncryptedSignedLicense {
        let encrypted_license = EncryptedLicense {
            iv: "f8ac2c0c0856d687f839cc12".to_string(),
            data: String::from("982c4b978cfaa02547cc6303895614cacd580304abdec1c001bb2f6e0bab44ff96f4bc46abbc14a3829e348f20dedaf17442831168e362f2883d93b6a63b9b2c703b05632eb383e767dfec605549d041cb44df4662d94f8f0d7866c58559341cdb51395f0672d98a3647e29f2302e5c7e12258c0179b70c72361a5e3f71aabade777e1650373dfcaa94cae5a9ffafd496b0f4cd9a0b94d3123a331fc9170f99ccb"),
            tag: String::from("23d6476e5fd04d21df6945257ad43c0d")
        };
        let signature = "e150ec4643221f666ff67ed612035f3ad126b8dbe2d9cc433dca7640e959d8c2".to_string();

        EncryptedSignedLicense {
            license: encrypted_license.clone(),
            signature,
        }
    }

    fn wrong_license() -> EncryptedSignedLicense {
        let encrypted_license = EncryptedLicense {
            iv: "fb38fc467a69b7a10119b390".to_string(),
            data: String::from("6c7464e685a5b9fe2cc9fa6c6d222a2f986b0ee077e7e3825a1d0ce0aaeb338a8dbf7941be8c0f87e5c8daf9b7411e2f58753bf65ace045301d7b5b4d2507cd4f9a3e0aa40fcf0fd052d597544b78bf9e53c03e355e7330b20d9667d560e12d1c851351d137ce897ed8f37fb165407bbf925534e6ce9d4ca38fbe6b1c46696ecf04fb77db3bcad626b6fb93bdfbcbe5e6ef5d2cbb97348b98170e10f184ae301b2"),
            tag: String::from("f8b4210286df0e83eeb6e1cb4c8c61cd")
        };
        let signature = "34afe37d2b6ed07c56459b49384e19010189b141268ef5d490eafe34128ba445".to_string();

        EncryptedSignedLicense {
            license: encrypted_license.clone(),
            signature,
        }
    }

    #[test]
    fn test_verify_license_with_valid_signature() {
        // Definición constantes ///////////////////////////
        let salt = "saltggg198sd7urf";
        let client_seed = "client_seed";
        let server_seed = "server SEED";

        let encrypted_signed_license = license();
        ///////////////////////////////////////////////////

        let c_keys = generate_key_pair_from_seed(&client_seed, salt.as_bytes());
        let s_keys = generate_key_pair_from_seed(&server_seed, salt.as_bytes());

        match (s_keys, c_keys) {
            (Ok((s_public, s_private)), Ok((c_public, c_private))) => {
                match (derive_shared_key(&s_public, &c_private), derive_shared_key(&c_public, &s_private)) {
                    (Ok(client_skey), Ok(server_skey)) => {
                        assert_eq!(client_skey, server_skey);
                        let is_valid = verify_license(&encrypted_signed_license, salt, &client_skey);
                        assert!(is_valid);
                    }
                    _ => assert!(false)
                }
            }
            _ => assert!(false)
        }
    }

    #[test]
    fn test_verify_license_with_wrong_signature() {
        // Definición constantes ///////////////////////////
        let salt = "saltggg198sd7urf";
        let client_seed = "client_seed";
        let server_seed = "server SEED";

        let encrypted_signed_license = wrong_license();
        ///////////////////////////////////////////////////

        let c_keys = generate_key_pair_from_seed(&client_seed, salt.as_bytes());
        let s_keys = generate_key_pair_from_seed(&server_seed, salt.as_bytes());

        match (s_keys, c_keys) {
            (Ok((s_public, s_private)), Ok((c_public, c_private))) => {
                match (derive_shared_key(&s_public, &c_private), derive_shared_key(&c_public, &s_private)) {
                    (Ok(client_skey), Ok(server_skey)) => {
                        assert_eq!(client_skey, server_skey);
                        let is_valid = verify_license(&encrypted_signed_license, salt, &client_skey);
                        assert!(!is_valid);
                    }
                    _ => assert!(false)
                }
            }
            _ => assert!(false)
        }
    }

    #[test]
    fn test_decrypt_license_without_checking_expiration_date() {
        // Definición constantes ///////////////////////////
        let salt = "saltggg198sd7urf";
        let client_seed = "client_seed";
        let server_seed = "server SEED";

        let encrypted_signed_license = license();
        ///////////////////////////////////////////////////

        let c_keys = generate_key_pair_from_seed(&client_seed, salt.as_bytes());
        let s_keys = generate_key_pair_from_seed(&server_seed, salt.as_bytes());

        match (s_keys, c_keys) {
            (Ok((s_public, s_private)), Ok((c_public, c_private))) => {
                match (derive_shared_key(&s_public, &c_private), derive_shared_key(&c_public, &s_private)) {
                    (Ok(client_skey), Ok(server_skey)) => {
                        assert_eq!(client_skey, server_skey);
                        let is_valid = verify_license(&encrypted_signed_license, salt, &client_skey);
                        assert!(is_valid);

                        match decrypt(&client_skey, &encrypted_signed_license.license) {
                            Ok(_) => assert!(true),
                           Err(_) => assert!(false)
                        }
                    }
                    _ => assert!(false)
                }
            }
            _ => assert!(false)
        }
    }

    #[test]
    fn test_decrypt_license_valid_expiration_date() {
        // Definición constantes ///////////////////////////
        let salt = "saltggg198sd7urf";
        let client_seed = "client_seed";
        let server_seed = "server SEED";

        let encrypted_signed_license = license();
        ///////////////////////////////////////////////////

        let c_keys = generate_key_pair_from_seed(&client_seed, salt.as_bytes());
        let s_keys = generate_key_pair_from_seed(&server_seed, salt.as_bytes());

        match (s_keys, c_keys) {
            (Ok((s_public, s_private)), Ok((c_public, c_private))) => {
                match (derive_shared_key(&s_public, &c_private), derive_shared_key(&c_public, &s_private)) {
                    (Ok(client_skey), Ok(server_skey)) => {
                        assert_eq!(client_skey, server_skey);
                        let is_valid = verify_license(&encrypted_signed_license, salt, &client_skey);
                        assert!(is_valid);

                        match decrypt(&client_skey, &encrypted_signed_license.license) {
                            Ok(license) => {
                                assert!(!license.computerName.is_empty());
                                assert!(!license.expiredAt.is_empty());
                                assert!(!license.platform.is_empty());

                                let str_datetime = license.expiredAt;
                                let datetime = chrono::DateTime::parse_from_rfc3339(&str_datetime);
                                if datetime.is_err() {
                                    assert!(false);
                                }
                                let dt = datetime.unwrap();
                                let now = chrono::Local::now();
                                println!("{dt:?}, {now:?}");
                                assert!(
                                    dt.naive_utc().and_utc().timestamp_millis() > now.naive_utc().and_utc().timestamp_millis()
                                );
                            }
                           Err(_) => assert!(false)
                        }
                    }
                    _ => assert!(false)
                }
            }
            _ => assert!(false)
        }
    }

    #[test]
    fn test_decrypt_license_old_expiration_date() {
        // Definición constantes ///////////////////////////
        let salt = "saltggg198sd7urf";
        let client_seed = "client_seed";
        let server_seed = "server SEED";

        let encrypted_signed_license = old_license();
        ///////////////////////////////////////////////////

        let c_keys = generate_key_pair_from_seed(&client_seed, salt.as_bytes());
        let s_keys = generate_key_pair_from_seed(&server_seed, salt.as_bytes());

        match (s_keys, c_keys) {
            (Ok((s_public, s_private)), Ok((c_public, c_private))) => {
                match (derive_shared_key(&s_public, &c_private), derive_shared_key(&c_public, &s_private)) {
                    (Ok(client_skey), Ok(server_skey)) => {
                        assert_eq!(client_skey, server_skey);
                        let is_valid = verify_license(&encrypted_signed_license, salt, &client_skey);
                        assert!(is_valid);

                        match decrypt(&client_skey, &encrypted_signed_license.license) {
                            Ok(license) => {
                                assert!(!license.computerName.is_empty());
                                assert!(!license.expiredAt.is_empty());
                                assert!(!license.platform.is_empty());

                                let str_datetime = license.expiredAt;
                                let datetime = chrono::DateTime::parse_from_rfc3339(&str_datetime);
                                if datetime.is_err() {
                                    assert!(false);
                                }
                                let dt = datetime.unwrap();
                                let now = chrono::Local::now();
                                assert!(
                                    dt.naive_utc().and_utc().timestamp_millis() < now.naive_utc().and_utc().timestamp_millis()
                                );
                            }
                           Err(_) => assert!(false)
                        }
                    }
                    _ => assert!(false)
                }
            }
            _ => assert!(false)
        }
    }
}
