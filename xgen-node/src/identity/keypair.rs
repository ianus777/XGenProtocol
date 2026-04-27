// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// Website: https://www.alchemydump.com
// Licensed under the PolyForm Noncommercial License 1.0.0
// See: https://polyformproject.org/licenses/noncommercial/1.0.0/

// Ed25519 keypair generation, encrypted storage, and loading (spec 3.5.1)
// Encryption: ChaCha20-Poly1305 with Argon2id key derivation.
// Pass empty string as passphrase for Local Node mode (Phase 1).

use std::path::Path;

use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Key, Nonce,
};
use ed25519_dalek::SigningKey;
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};

use crate::crypto::encoding::{self, EncodingError};

#[derive(Debug, thiserror::Error)]
pub enum KeypairError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("encoding error: {0}")]
    Encoding(#[from] EncodingError),
    #[error("decryption failed — wrong passphrase?")]
    DecryptionFailed,
    #[error("invalid key bytes in file")]
    InvalidKey,
    #[error("unsupported file version: {0}")]
    UnsupportedVersion(u32),
    #[error("key derivation failed")]
    KdfFailed,
}

#[derive(Serialize, Deserialize)]
struct KeypairFile {
    version: u32,
    algorithm: String,
    kdf: String,
    salt: String,       // base64url, 32 bytes
    nonce: String,      // base64url, 12 bytes
    ciphertext: String, // base64url, 32-byte key + 16-byte AEAD tag = 48 bytes
}

// Argon2id parameters — tuned for interactive use, not for high-security offline cracking.
const KDF_M_COST: u32 = 65536; // 64 MB
const KDF_T_COST: u32 = 3;
const KDF_P_COST: u32 = 1;

pub fn generate() -> SigningKey {
    let mut secret = [0u8; 32];
    OsRng.fill_bytes(&mut secret);
    SigningKey::from_bytes(&secret)
}

pub fn save(signing_key: &SigningKey, path: &Path, passphrase: &str) -> Result<(), KeypairError> {
    let mut salt = [0u8; 32];
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut salt);
    OsRng.fill_bytes(&mut nonce_bytes);

    let enc_key = derive_key(passphrase, &salt)?;
    let cipher = ChaCha20Poly1305::new(Key::from_slice(&enc_key));
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce_bytes), signing_key.as_bytes().as_ref())
        .map_err(|_| KeypairError::DecryptionFailed)?;

    let file = KeypairFile {
        version: 1,
        algorithm: "ed25519".to_string(),
        kdf: "argon2id".to_string(),
        salt: encoding::encode(&salt),
        nonce: encoding::encode(&nonce_bytes),
        ciphertext: encoding::encode(&ciphertext),
    };

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, serde_json::to_string_pretty(&file)?)?;
    Ok(())
}

pub fn load(path: &Path, passphrase: &str) -> Result<SigningKey, KeypairError> {
    let json = std::fs::read_to_string(path)?;
    let file: KeypairFile = serde_json::from_str(&json)?;

    if file.version != 1 {
        return Err(KeypairError::UnsupportedVersion(file.version));
    }

    let salt = encoding::decode(&file.salt)?;
    let nonce_bytes = encoding::decode(&file.nonce)?;
    let ciphertext = encoding::decode(&file.ciphertext)?;

    let enc_key = derive_key(passphrase, &salt)?;
    let cipher = ChaCha20Poly1305::new(Key::from_slice(&enc_key));
    let plaintext = cipher
        .decrypt(Nonce::from_slice(&nonce_bytes), ciphertext.as_ref())
        .map_err(|_| KeypairError::DecryptionFailed)?;

    let key_bytes: [u8; 32] = plaintext.try_into().map_err(|_| KeypairError::InvalidKey)?;
    Ok(SigningKey::from_bytes(&key_bytes))
}

fn derive_key(passphrase: &str, salt: &[u8]) -> Result<[u8; 32], KeypairError> {
    let params = Params::new(KDF_M_COST, KDF_T_COST, KDF_P_COST, Some(32))
        .map_err(|_| KeypairError::KdfFailed)?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = [0u8; 32];
    argon2
        .hash_password_into(passphrase.as_bytes(), salt, &mut key)
        .map_err(|_| KeypairError::KdfFailed)?;
    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn generate_is_unique() {
        let k1 = generate();
        let k2 = generate();
        assert_ne!(k1.as_bytes(), k2.as_bytes());
    }

    #[test]
    fn save_load_round_trip() {
        let key = generate();
        let tmp = NamedTempFile::new().unwrap();
        save(&key, tmp.path(), "test-passphrase").unwrap();
        let loaded = load(tmp.path(), "test-passphrase").unwrap();
        assert_eq!(key.as_bytes(), loaded.as_bytes());
    }

    #[test]
    fn wrong_passphrase_fails() {
        let key = generate();
        let tmp = NamedTempFile::new().unwrap();
        save(&key, tmp.path(), "correct").unwrap();
        assert!(load(tmp.path(), "wrong").is_err());
    }

    #[test]
    fn empty_passphrase_works() {
        let key = generate();
        let tmp = NamedTempFile::new().unwrap();
        save(&key, tmp.path(), "").unwrap();
        let loaded = load(tmp.path(), "").unwrap();
        assert_eq!(key.as_bytes(), loaded.as_bytes());
    }
}
