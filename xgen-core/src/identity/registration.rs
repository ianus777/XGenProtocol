// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

// Identity registration acceptance pipeline (spec 3.6.3–3.6.5).
//
// Implements the 8-step Node acceptance criteria for identity.register.
// In Local Node mode, steps 4–7 (trust assertion checks) are skipped.
//
// Also provides:
//   - sign_register()   — client-side: sign identity.register
//   - sign_update()     — client-side: sign identity.update
//   - verify_register() — node-side:   verify incoming identity.register signature
//   - build_register()  — client-side: construct an unsigned identity.register

use chrono::{SecondsFormat, Utc};
use ed25519_dalek::SigningKey;
use thiserror::Error;

use crate::{
    crypto::{encoding, signing},
    identity::registry::{DeviceRecord, IdentityRecord},
    wire::{
        canonical::canonical_object_json,
        types::IdentityMessage,
    },
};

// ── Maximum display name length (Phase 1 decision; recorded in DECISIONS.md) ──
const MAX_DISPLAY_NAME_LEN: usize = 128;

// ── Canonical field orders (signature excluded) ───────────────────────────────

const REGISTER_FIELDS: &[&str] = &[
    "protocol_version", "type", "identity_id", "display_name", "trust_assertion", "timestamp",
];

const UPDATE_FIELDS: &[&str] = &[
    "protocol_version", "type", "identity_id", "update_version", "changes", "timestamp",
];

// ── Error type ────────────────────────────────────────────────────────────────

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RegistrationError {
    #[error("identity_id does not match authenticated transport identity (code 3001)")]
    IdentityMismatch,
    #[error("registration request signature invalid (code 3002)")]
    SignatureInvalid,
    #[error("identity already registered on this node (code 3007)")]
    AlreadyRegistered,
    #[error("trust assertion required but not provided (code 3003)")]
    TrustAssertionRequired,
    #[error("trust assertion signature invalid (code 3004)")]
    AssertionSignatureInvalid,
    #[error("trust assertion has expired (code 3005)")]
    AssertionExpired,
    #[error("auth module is not trusted by this node (code 3006)")]
    AuthModuleUntrusted,
    #[error("node capacity exceeded (code 3008)")]
    NodeCapacityExceeded,
    #[error("display name invalid — empty or too long (code 3009)")]
    DisplayNameInvalid,
    #[error("unexpected message type: expected identity.register")]
    WrongMessageType,
}

impl RegistrationError {
    pub fn to_registration_code(&self) -> (u32, &'static str) {
        match self {
            Self::IdentityMismatch => (3001, "identity_mismatch"),
            Self::SignatureInvalid => (3002, "signature_invalid"),
            Self::AlreadyRegistered => (3007, "already_registered"),
            Self::TrustAssertionRequired => (3003, "trust_assertion_required"),
            Self::AssertionSignatureInvalid => (3004, "assertion_signature_invalid"),
            Self::AssertionExpired => (3005, "assertion_expired"),
            Self::AuthModuleUntrusted => (3006, "auth_module_untrusted"),
            Self::NodeCapacityExceeded => (3008, "node_capacity_exceeded"),
            Self::DisplayNameInvalid => (3009, "display_name_invalid"),
            Self::WrongMessageType => (3002, "signature_invalid"),
        }
    }
}

// ── Client-side helpers ───────────────────────────────────────────────────────

/// Derive an identity_id URI from a signing key.
pub fn identity_id_from_key(key: &SigningKey) -> String {
    format!(
        "xgen://pubkey/ed25519:{}",
        encoding::encode(key.verifying_key().as_bytes())
    )
}

/// Build an unsigned `identity.register` message ready for signing.
pub fn build_register(
    key: &SigningKey,
    display_name: Option<String>,
) -> IdentityMessage {
    let identity_id = identity_id_from_key(key);
    let ts = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    IdentityMessage::Register {
        protocol_version: "0.1".to_string(),
        identity_id,
        display_name,
        trust_assertion: None, // Local Node mode — no assertion
        timestamp: ts,
        signature: None,
    }
}

/// Sign an `identity.register` message with the Identity keypair.
pub fn sign_register(msg: IdentityMessage, key: &SigningKey) -> IdentityMessage {
    let canonical = canonical_json_for_register(&msg);
    let sig = signing::sign(key, canonical.as_bytes());
    set_register_signature(msg, sig)
}

/// Sign an `identity.update` message with the Identity keypair.
pub fn sign_update(msg: IdentityMessage, key: &SigningKey) -> IdentityMessage {
    let canonical = canonical_json_for_update(&msg);
    let sig = signing::sign(key, canonical.as_bytes());
    set_update_signature(msg, sig)
}

// ── Node-side verification ────────────────────────────────────────────────────

/// Verify the signature on an incoming `identity.register` message.
pub fn verify_register(msg: &IdentityMessage) -> Result<(), RegistrationError> {
    let (identity_id, sig_str) = extract_register_sig(msg)?;
    let vk = parse_vk(identity_id)?;
    let canonical = canonical_json_for_register(msg);
    signing::verify(&vk, canonical.as_bytes(), sig_str)
        .map_err(|_| RegistrationError::SignatureInvalid)
}

/// Verify the signature on an incoming `identity.update` message.
pub fn verify_update(msg: &IdentityMessage) -> Result<(), RegistrationError> {
    let (identity_id, sig_str) = extract_update_sig(msg)?;
    let vk = parse_vk(identity_id)?;
    let canonical = canonical_json_for_update(msg);
    signing::verify(&vk, canonical.as_bytes(), sig_str)
        .map_err(|_| RegistrationError::SignatureInvalid)
}

// ── 8-step acceptance pipeline (spec 3.6.4) ──────────────────────────────────

/// Run the Node-side acceptance pipeline for an incoming `identity.register`.
///
/// `authenticated_id` is the identity verified during transport authentication.
/// `already_registered` is checked from the registry before calling this.
/// `local_node` = true skips trust assertion checks (steps 4–7).
/// `home_node_id` is this Node's own node_id URI.
///
/// Returns the `IdentityRecord` to store on success.
pub fn accept_registration(
    msg: &IdentityMessage,
    authenticated_id: &str,
    already_registered: bool,
    local_node: bool,
    home_node_id: &str,
    registered_at: &str,
) -> Result<IdentityRecord, RegistrationError> {
    // Extract fields — must be identity.register
    let (identity_id, display_name, trust_assertion) = match msg {
        IdentityMessage::Register { identity_id, display_name, trust_assertion, .. } => {
            (identity_id.as_str(), display_name.as_deref(), trust_assertion.as_ref())
        }
        _ => return Err(RegistrationError::WrongMessageType),
    };

    // Step 1 — identity_id matches transport auth
    if identity_id != authenticated_id {
        return Err(RegistrationError::IdentityMismatch);
    }

    // Step 2 — signature verifies
    verify_register(msg)?;

    // Step 3 — not already registered
    if already_registered {
        return Err(RegistrationError::AlreadyRegistered);
    }

    // Steps 4–7 — trust assertion (skipped in Local Node mode)
    if !local_node {
        // Step 4 — trust_assertion present
        let _assertion = trust_assertion.ok_or(RegistrationError::TrustAssertionRequired)?;
        // Steps 5–7 deferred to Phase 2 (Auth Module implementation)
        // Phase 1 Local Node mode always skips these.
    }

    // Step 8 — display name validation (checked even in Local Node mode)
    if let Some(name) = display_name {
        if name.is_empty() || name.len() > MAX_DISPLAY_NAME_LEN {
            return Err(RegistrationError::DisplayNameInvalid);
        }
        if name.chars().any(|c| c.is_control()) {
            return Err(RegistrationError::DisplayNameInvalid);
        }
    }

    // Build and return the Identity record.
    Ok(IdentityRecord {
        identity_id: identity_id.to_string(),
        display_name: display_name.map(str::to_string),
        registered_at: registered_at.to_string(),
        trust_assertion: trust_assertion.cloned(),
        devices: vec![DeviceRecord {
            device_id: identity_id.to_string(), // Phase 1: identity_id == device_id
            device_name: None,
            authorised_at: registered_at.to_string(),
        }],
        home_node: home_node_id.to_string(),
        update_version: 0,
    })
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn canonical_json_for_register(msg: &IdentityMessage) -> String {
    let v = serde_json::to_value(msg).expect("IdentityMessage is always serialisable");
    canonical_object_json(&v, REGISTER_FIELDS)
}

fn canonical_json_for_update(msg: &IdentityMessage) -> String {
    let v = serde_json::to_value(msg).expect("IdentityMessage is always serialisable");
    canonical_object_json(&v, UPDATE_FIELDS)
}

fn set_register_signature(msg: IdentityMessage, sig: String) -> IdentityMessage {
    match msg {
        IdentityMessage::Register {
            protocol_version, identity_id, display_name, trust_assertion, timestamp, ..
        } => IdentityMessage::Register {
            protocol_version, identity_id, display_name, trust_assertion, timestamp,
            signature: Some(sig),
        },
        other => other,
    }
}

fn set_update_signature(msg: IdentityMessage, sig: String) -> IdentityMessage {
    match msg {
        IdentityMessage::Update {
            protocol_version, identity_id, update_version, changes, timestamp, ..
        } => IdentityMessage::Update {
            protocol_version, identity_id, update_version, changes, timestamp,
            signature: Some(sig),
        },
        other => other,
    }
}

fn extract_register_sig(msg: &IdentityMessage) -> Result<(&str, &str), RegistrationError> {
    match msg {
        IdentityMessage::Register { identity_id, signature, .. } => {
            let sig = signature.as_deref().ok_or(RegistrationError::SignatureInvalid)?;
            Ok((identity_id.as_str(), sig))
        }
        _ => Err(RegistrationError::WrongMessageType),
    }
}

fn extract_update_sig(msg: &IdentityMessage) -> Result<(&str, &str), RegistrationError> {
    match msg {
        IdentityMessage::Update { identity_id, signature, .. } => {
            let sig = signature.as_deref().ok_or(RegistrationError::SignatureInvalid)?;
            Ok((identity_id.as_str(), sig))
        }
        _ => Err(RegistrationError::WrongMessageType),
    }
}

fn parse_vk(identity_id: &str) -> Result<ed25519_dalek::VerifyingKey, RegistrationError> {
    let b64 = identity_id
        .strip_prefix("xgen://pubkey/ed25519:")
        .ok_or(RegistrationError::SignatureInvalid)?;
    let bytes = encoding::decode(b64).map_err(|_| RegistrationError::SignatureInvalid)?;
    let arr: [u8; 32] = bytes
        .try_into()
        .map_err(|_| RegistrationError::SignatureInvalid)?;
    ed25519_dalek::VerifyingKey::from_bytes(&arr).map_err(|_| RegistrationError::SignatureInvalid)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::keypair;

    fn make_signed_register(key: &SigningKey, name: Option<&str>) -> IdentityMessage {
        let msg = build_register(key, name.map(str::to_string));
        sign_register(msg, key)
    }

    const HOME: &str = "xgen://pubkey/ed25519:NODE";

    #[test]
    fn sign_verify_register_round_trip() {
        let key = keypair::generate();
        let msg = make_signed_register(&key, Some("Alice"));
        assert!(verify_register(&msg).is_ok());
    }

    #[test]
    fn tampered_display_name_fails_verification() {
        let key = keypair::generate();
        let signed = make_signed_register(&key, Some("Alice"));
        // Swap out the display name after signing.
        let tampered = match signed {
            IdentityMessage::Register {
                protocol_version, identity_id, trust_assertion, timestamp, signature, ..
            } => IdentityMessage::Register {
                protocol_version, identity_id, trust_assertion, timestamp, signature,
                display_name: Some("Eve".to_string()),
            },
            _ => unreachable!(),
        };
        assert!(verify_register(&tampered).is_err());
    }

    #[test]
    fn local_node_accept_pipeline_succeeds() {
        let key = keypair::generate();
        let id = identity_id_from_key(&key);
        let msg = make_signed_register(&key, Some("Alice"));
        let ts = "2026-04-27T12:00:00.000Z";
        let record = accept_registration(&msg, &id, false, true, HOME, ts).unwrap();
        assert_eq!(record.identity_id, id);
        assert_eq!(record.display_name.as_deref(), Some("Alice"));
        assert_eq!(record.home_node, HOME);
        assert_eq!(record.update_version, 0);
        assert_eq!(record.devices.len(), 1);
        assert_eq!(record.devices[0].device_id, id);
    }

    #[test]
    fn identity_mismatch_rejected() {
        let key = keypair::generate();
        let other_key = keypair::generate();
        let msg = make_signed_register(&key, None);
        let other_id = identity_id_from_key(&other_key);
        let err = accept_registration(&msg, &other_id, false, true, HOME, "ts").unwrap_err();
        assert_eq!(err, RegistrationError::IdentityMismatch);
    }

    #[test]
    fn already_registered_rejected() {
        let key = keypair::generate();
        let id = identity_id_from_key(&key);
        let msg = make_signed_register(&key, None);
        let err = accept_registration(&msg, &id, true, true, HOME, "ts").unwrap_err();
        assert_eq!(err, RegistrationError::AlreadyRegistered);
    }

    #[test]
    fn trust_assertion_required_in_non_local_mode() {
        let key = keypair::generate();
        let id = identity_id_from_key(&key);
        // Build a register request without trust_assertion (Local Node style).
        let msg = make_signed_register(&key, None);
        // Run in non-local mode — should fail at step 4.
        let err = accept_registration(&msg, &id, false, false, HOME, "ts").unwrap_err();
        assert_eq!(err, RegistrationError::TrustAssertionRequired);
    }

    #[test]
    fn display_name_too_long_rejected() {
        let key = keypair::generate();
        let id = identity_id_from_key(&key);
        let long_name = "A".repeat(MAX_DISPLAY_NAME_LEN + 1);
        let msg = make_signed_register(&key, Some(&long_name));
        let err = accept_registration(&msg, &id, false, true, HOME, "ts").unwrap_err();
        assert_eq!(err, RegistrationError::DisplayNameInvalid);
    }

    #[test]
    fn empty_display_name_rejected() {
        let key = keypair::generate();
        let id = identity_id_from_key(&key);
        let msg = make_signed_register(&key, Some(""));
        let err = accept_registration(&msg, &id, false, true, HOME, "ts").unwrap_err();
        assert_eq!(err, RegistrationError::DisplayNameInvalid);
    }

    #[test]
    fn display_name_with_control_char_rejected() {
        let key = keypair::generate();
        let id = identity_id_from_key(&key);
        let msg = make_signed_register(&key, Some("Alice\x00hack"));
        let err = accept_registration(&msg, &id, false, true, HOME, "ts").unwrap_err();
        assert_eq!(err, RegistrationError::DisplayNameInvalid);
    }

    #[test]
    fn no_display_name_accepted() {
        let key = keypair::generate();
        let id = identity_id_from_key(&key);
        let msg = make_signed_register(&key, None);
        let record = accept_registration(&msg, &id, false, true, HOME, "ts").unwrap();
        assert!(record.display_name.is_none());
    }

    #[test]
    fn sign_verify_update_round_trip() {
        let key = keypair::generate();
        let id = identity_id_from_key(&key);
        let ts = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
        let msg = IdentityMessage::Update {
            protocol_version: "0.1".to_string(),
            identity_id: id,
            update_version: 1,
            changes: serde_json::json!({"display_name": "Alice v2"}),
            timestamp: ts,
            signature: None,
        };
        let signed = sign_update(msg, &key);
        assert!(verify_update(&signed).is_ok());
    }
}
