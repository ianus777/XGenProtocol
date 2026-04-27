// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

// Challenge-response authentication (spec 3.3.4, Phase 2).
//
// Protocol:
//   1. Node generates a random nonce and sends transport.challenge.
//   2. Client signs the raw nonce bytes with its Ed25519 private key.
//   3. Client sends transport.auth with identity_id, nonce (echoed), and signature.
//   4. Node verifies:
//        - nonce matches what was issued
//        - challenge is < 30 seconds old
//        - signature verifies against the public key in identity_id
//   5. Node sends transport.auth_ok or transport.auth_fail.
//
// Local Node mode: no Identity registry lookup — any valid signature is accepted.

use chrono::{DateTime, SecondsFormat, Utc};
use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::{rngs::OsRng, RngCore};
use thiserror::Error;

use crate::crypto::{encoding, signing};
use crate::wire::types::TransportMessage;

/// Challenge nonce is 30 bytes before expiry (spec 3.3.8, code 1003).
pub const CHALLENGE_EXPIRY_SECS: i64 = 30;

/// Prefix for Identity pubkey URIs (spec 3.1.6).
const PUBKEY_URI_PREFIX: &str = "xgen://pubkey/ed25519:";

// ── Error type ────────────────────────────────────────────────────────────────

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AuthError {
    #[error("wrong message type — expected transport.auth")]
    WrongMessageType,
    #[error("nonce mismatch (code 1004)")]
    NonceMismatch,
    #[error("challenge expired after {CHALLENGE_EXPIRY_SECS}s (code 1003)")]
    NonceExpired,
    #[error("invalid nonce encoding")]
    InvalidNonce,
    #[error("invalid identity_id format (code 1002)")]
    InvalidIdentityId,
    #[error("signature invalid (code 1001)")]
    SignatureInvalid,
}

impl AuthError {
    /// Map to the transport error code and string from spec 3.3.8.
    pub fn to_transport_code(&self) -> (u32, &'static str) {
        match self {
            Self::SignatureInvalid => (1001, "auth_signature_invalid"),
            Self::InvalidIdentityId => (1002, "auth_identity_unknown"),
            Self::NonceExpired => (1003, "auth_nonce_expired"),
            Self::NonceMismatch => (1004, "auth_nonce_mismatch"),
            Self::WrongMessageType | Self::InvalidNonce => (1000, "auth_error"),
        }
    }
}

// ── Server side ───────────────────────────────────────────────────────────────

/// A challenge that has been issued to a peer, held until the response arrives.
pub struct IssuedChallenge {
    pub nonce: String,        // base64url-encoded 32 random bytes
    pub issued_at: DateTime<Utc>,
}

/// Generate a fresh challenge for the server to send.
pub fn issue_challenge() -> (IssuedChallenge, TransportMessage) {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    let nonce = encoding::encode(&bytes);
    let issued_at = Utc::now();
    let timestamp = issued_at.to_rfc3339_opts(SecondsFormat::Millis, true);

    let msg = TransportMessage::Challenge {
        protocol_version: "0.1".to_string(),
        nonce: nonce.clone(),
        timestamp,
    };
    (IssuedChallenge { nonce, issued_at }, msg)
}

/// Verify a `transport.auth` message against the issued challenge.
/// Returns the `identity_id` string on success.
pub fn verify_auth_response(
    issued: &IssuedChallenge,
    auth: &TransportMessage,
) -> Result<String, AuthError> {
    let (identity_id, nonce, sig) = match auth {
        TransportMessage::Auth { identity_id, nonce, signature, .. } => {
            (identity_id, nonce, signature)
        }
        _ => return Err(AuthError::WrongMessageType),
    };

    // Nonce must match exactly.
    if nonce != &issued.nonce {
        return Err(AuthError::NonceMismatch);
    }

    // Challenge must be < 30 seconds old.
    let age = Utc::now().signed_duration_since(issued.issued_at);
    if age.num_seconds() >= CHALLENGE_EXPIRY_SECS {
        return Err(AuthError::NonceExpired);
    }

    // Decode nonce to raw bytes — the signed input (spec 3.3.4).
    let nonce_bytes = encoding::decode(nonce).map_err(|_| AuthError::InvalidNonce)?;

    // Extract the verifying key from identity_id.
    let verifying_key = parse_identity_id(identity_id)?;

    // Verify signature.
    signing::verify(&verifying_key, &nonce_bytes, sig).map_err(|_| AuthError::SignatureInvalid)?;

    Ok(identity_id.clone())
}

// ── Client side ───────────────────────────────────────────────────────────────

/// Build a `transport.auth` response to a received challenge.
/// Signs the raw nonce bytes (not the base64url string) with the client's key.
pub fn build_auth_response(nonce: &str, signing_key: &SigningKey) -> TransportMessage {
    let nonce_bytes = encoding::decode(nonce).expect("challenge nonce is valid base64url");
    let sig = signing::sign(signing_key, &nonce_bytes);
    let pubkey_b64 = encoding::encode(signing_key.verifying_key().as_bytes());
    let identity_id = format!("{}{}", PUBKEY_URI_PREFIX, pubkey_b64);

    TransportMessage::Auth {
        protocol_version: "0.1".to_string(),
        identity_id,
        nonce: nonce.to_string(),
        signature: sig,
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Parse an identity_id URI into an Ed25519 `VerifyingKey`.
fn parse_identity_id(identity_id: &str) -> Result<VerifyingKey, AuthError> {
    let b64 = identity_id
        .strip_prefix(PUBKEY_URI_PREFIX)
        .ok_or(AuthError::InvalidIdentityId)?;
    let bytes = encoding::decode(b64).map_err(|_| AuthError::InvalidIdentityId)?;
    let arr: [u8; 32] = bytes.try_into().map_err(|_| AuthError::InvalidIdentityId)?;
    VerifyingKey::from_bytes(&arr).map_err(|_| AuthError::InvalidIdentityId)
}

/// Derive an identity_id URI from a `VerifyingKey`.
pub fn identity_id_from_key(key: &VerifyingKey) -> String {
    format!("{}{}", PUBKEY_URI_PREFIX, encoding::encode(key.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::keypair;

    fn make_key() -> SigningKey {
        keypair::generate()
    }

    #[test]
    fn full_auth_round_trip() {
        let key = make_key();
        let (issued, _challenge_msg) = issue_challenge();
        let auth_msg = build_auth_response(&issued.nonce, &key);
        let identity_id = verify_auth_response(&issued, &auth_msg).unwrap();
        let expected_id = identity_id_from_key(&key.verifying_key());
        assert_eq!(identity_id, expected_id);
    }

    #[test]
    fn wrong_nonce_rejected() {
        let key = make_key();
        let (issued, _) = issue_challenge();
        // Build auth with a different nonce.
        let (other_issued, _) = issue_challenge();
        let auth_msg = build_auth_response(&other_issued.nonce, &key);
        assert_eq!(
            verify_auth_response(&issued, &auth_msg),
            Err(AuthError::NonceMismatch)
        );
    }

    #[test]
    fn wrong_key_rejected() {
        let key1 = make_key();
        let key2 = make_key();
        let (issued, _) = issue_challenge();
        // Sign with key1 but claim key2's identity.
        let nonce_bytes = encoding::decode(&issued.nonce).unwrap();
        let sig = signing::sign(&key1, &nonce_bytes);
        let auth_msg = TransportMessage::Auth {
            protocol_version: "0.1".to_string(),
            identity_id: identity_id_from_key(&key2.verifying_key()),
            nonce: issued.nonce.clone(),
            signature: sig,
        };
        assert_eq!(
            verify_auth_response(&issued, &auth_msg),
            Err(AuthError::SignatureInvalid)
        );
    }

    #[test]
    fn wrong_message_type_rejected() {
        let (issued, challenge_msg) = issue_challenge();
        // Pass the challenge message back instead of an auth message.
        assert_eq!(
            verify_auth_response(&issued, &challenge_msg),
            Err(AuthError::WrongMessageType)
        );
    }

    #[test]
    fn identity_id_round_trip() {
        let key = make_key();
        let id = identity_id_from_key(&key.verifying_key());
        assert!(id.starts_with("xgen://pubkey/ed25519:"));
        let vk = parse_identity_id(&id).unwrap();
        assert_eq!(vk.as_bytes(), key.verifying_key().as_bytes());
    }

    #[test]
    fn error_codes_are_correct() {
        assert_eq!(AuthError::SignatureInvalid.to_transport_code().0, 1001);
        assert_eq!(AuthError::InvalidIdentityId.to_transport_code().0, 1002);
        assert_eq!(AuthError::NonceExpired.to_transport_code().0, 1003);
        assert_eq!(AuthError::NonceMismatch.to_transport_code().0, 1004);
    }
}
