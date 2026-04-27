// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

// Ed25519 signing and verification (spec 3.2.4)
// Signature string format: "ed25519:<base64url-pubkey>:<base64url-sig>"

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};

use crate::crypto::encoding::{self, EncodingError};

#[derive(Debug, thiserror::Error)]
pub enum SigningError {
    #[error("signature verification failed")]
    VerificationFailed,
    #[error("malformed signature string: {0}")]
    Malformed(String),
    #[error("encoding error: {0}")]
    Encoding(#[from] EncodingError),
    #[error("invalid public key bytes")]
    InvalidKey,
}

/// Format a signature as "ed25519:<base64url-pubkey>:<base64url-sig>"
pub fn format_signature(verifying_key: &VerifyingKey, signature: &Signature) -> String {
    let key_b64 = encoding::encode(verifying_key.as_bytes());
    let sig_b64 = encoding::encode(&signature.to_bytes());
    format!("ed25519:{}:{}", key_b64, sig_b64)
}

/// Sign a message and return the formatted signature string
pub fn sign(signing_key: &SigningKey, message: &[u8]) -> String {
    let sig = signing_key.sign(message);
    format_signature(&signing_key.verifying_key(), &sig)
}

/// Verify a formatted signature string against a message using the key embedded in the string
pub fn verify_with_embedded_key(message: &[u8], sig_string: &str) -> Result<(), SigningError> {
    let key = verifying_key_from_sig_string(sig_string)?;
    let sig = sig_bytes_from_sig_string(sig_string)?;
    key.verify(message, &sig).map_err(|_| SigningError::VerificationFailed)
}

/// Verify using a known verifying key — ignores the keyid embedded in the signature string
pub fn verify(verifying_key: &VerifyingKey, message: &[u8], sig_string: &str) -> Result<(), SigningError> {
    let sig = sig_bytes_from_sig_string(sig_string)?;
    verifying_key.verify(message, &sig).map_err(|_| SigningError::VerificationFailed)
}

pub fn verifying_key_from_sig_string(sig_string: &str) -> Result<VerifyingKey, SigningError> {
    let parts = split_sig_string(sig_string)?;
    let key_bytes = encoding::decode(parts[1])?;
    let arr: [u8; 32] = key_bytes
        .try_into()
        .map_err(|_| SigningError::Malformed("public key must be 32 bytes".into()))?;
    VerifyingKey::from_bytes(&arr).map_err(|_| SigningError::InvalidKey)
}

fn sig_bytes_from_sig_string(sig_string: &str) -> Result<Signature, SigningError> {
    let parts = split_sig_string(sig_string)?;
    let sig_bytes = encoding::decode(parts[2])?;
    let arr: [u8; 64] = sig_bytes
        .try_into()
        .map_err(|_| SigningError::Malformed("signature must be 64 bytes".into()))?;
    Ok(Signature::from_bytes(&arr))
}

fn split_sig_string(s: &str) -> Result<Vec<&str>, SigningError> {
    let parts: Vec<&str> = s.splitn(3, ':').collect();
    if parts.len() != 3 {
        return Err(SigningError::Malformed(s.into()));
    }
    if parts[0] != "ed25519" {
        return Err(SigningError::Malformed(format!("unknown algorithm: {}", parts[0])));
    }
    Ok(parts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::OsRng;

    fn keypair() -> SigningKey {
        use rand::RngCore;
        let mut secret = [0u8; 32];
        OsRng.fill_bytes(&mut secret);
        SigningKey::from_bytes(&secret)
    }

    #[test]
    fn sign_verify_round_trip() {
        let key = keypair();
        let msg = b"xgen protocol test message";
        let sig = sign(&key, msg);
        assert!(verify(&key.verifying_key(), msg, &sig).is_ok());
    }

    #[test]
    fn verify_with_embedded_key_round_trip() {
        let key = keypair();
        let msg = b"test";
        let sig = sign(&key, msg);
        assert!(verify_with_embedded_key(msg, &sig).is_ok());
    }

    #[test]
    fn wrong_message_fails() {
        let key = keypair();
        let sig = sign(&key, b"correct message");
        assert!(verify(&key.verifying_key(), b"wrong message", &sig).is_err());
    }

    #[test]
    fn wrong_key_fails() {
        let key1 = keypair();
        let key2 = keypair();
        let sig = sign(&key1, b"message");
        assert!(verify(&key2.verifying_key(), b"message", &sig).is_err());
    }

    #[test]
    fn signature_format() {
        let key = keypair();
        let sig = sign(&key, b"test");
        let parts: Vec<&str> = sig.splitn(3, ':').collect();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0], "ed25519");
        // pubkey is 32 bytes → 43 base64url chars
        assert_eq!(parts[1].len(), 43);
        // sig is 64 bytes → 86 base64url chars
        assert_eq!(parts[2].len(), 86);
    }
}
