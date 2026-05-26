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
use xgen_common::xgid::{IdentityXgid, NodeXgid, Xgid};

use crate::{
    crypto::{encoding, signing},
    identity::registry::{DeviceRecord, IdentityRecord},
    wire::{
        canonical::canonical_object_json,
        types::{AiCapabilities, IdentityMessage},
    },
};

// ── Maximum display name length (Phase 1 decision; recorded in DECISIONS.md) ──
const MAX_DISPLAY_NAME_LEN: usize = 128;

// ── Canonical field orders (signature excluded) ───────────────────────────────

// Canonical field order for identity.register signature (spec 3.6.3).
// `is_ai` and `ai_capabilities` (3.6.10) are placed between `display_name`
// and `trust_assertion` to match the spec table order. Both are absent from
// the canonical form when the registrant is human (is_ai = false), preserving
// pre-3.6.10 signature compatibility.
const REGISTER_FIELDS: &[&str] = &[
    "protocol_version",
    "type",
    "identity_id",
    "display_name",
    "is_ai",
    "ai_capabilities",
    "trust_assertion",
    "timestamp",
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
    #[error("AI declaration shape invalid — is_ai / ai_capabilities inconsistent or required capability missing (code 3040)")]
    AiDeclarationInvalid,
    #[error("is_ai is immutable after registration (code 3041)")]
    AiFlagImmutable,
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
            Self::AiDeclarationInvalid => (3040, "ai_declaration_invalid"),
            Self::AiFlagImmutable => (3041, "ai_role_violation"),
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

/// Build an unsigned `identity.register` message for a human Identity.
/// Equivalent to `build_register_with_ai(key, display_name, false, None)`.
pub fn build_register(
    key: &SigningKey,
    display_name: Option<String>,
) -> IdentityMessage {
    build_register_with_ai(key, display_name, false, None)
}

/// Build an unsigned `identity.register` message with explicit AI declaration.
///
/// For human Identities pass `is_ai = false` and `capabilities = None`.
/// For AI Identities pass `is_ai = true` and a fully-populated `AiCapabilities`
/// (spec 3.6.10.3 — all Phase 2 required capability keys must be present).
pub fn build_register_with_ai(
    key: &SigningKey,
    display_name: Option<String>,
    is_ai: bool,
    capabilities: Option<AiCapabilities>,
) -> IdentityMessage {
    let identity_id = identity_id_from_key(key);
    let ts = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    IdentityMessage::Register {
        protocol_version: "0.1".to_string(),
        identity_id,
        display_name,
        is_ai,
        ai_capabilities: capabilities,
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
    let (identity_id, display_name, is_ai, ai_capabilities, trust_assertion) = match msg {
        IdentityMessage::Register {
            identity_id,
            display_name,
            is_ai,
            ai_capabilities,
            trust_assertion,
            ..
        } => (
            identity_id.as_str(),
            display_name.as_deref(),
            *is_ai,
            ai_capabilities.as_ref(),
            trust_assertion.as_ref(),
        ),
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

    // Display name validation (independent of step numbering; checked always).
    if let Some(name) = display_name {
        if name.is_empty() || name.len() > MAX_DISPLAY_NAME_LEN {
            return Err(RegistrationError::DisplayNameInvalid);
        }
        if name.chars().any(|c| c.is_control()) {
            return Err(RegistrationError::DisplayNameInvalid);
        }
    }

    // Step 8 — is_ai / ai_capabilities shape consistency (spec 3.6.4, 3.6.10.1).
    validate_ai_declaration(is_ai, ai_capabilities)?;

    // Step 9 — capacity check (not enforced in Phase 1; deferred).

    // Build and return the Identity record. Pass 2 widens callers to typed XGIDs;
    // the wraps at the IdentityRecord-construction boundary collapse then.
    Ok(IdentityRecord {
        identity_id: IdentityXgid::from_xgid(Xgid::new(identity_id.to_string())),
        display_name: display_name.map(str::to_string),
        is_ai,
        ai_capabilities: ai_capabilities.cloned(),
        registered_at: registered_at.to_string(),
        trust_assertion: trust_assertion.cloned(),
        devices: vec![DeviceRecord {
            device_id: identity_id.to_string(), // Phase 1: identity_id == device_id
            device_name: None,
            authorised_at: registered_at.to_string(),
        }],
        home_node: NodeXgid::from_xgid(Xgid::new(home_node_id.to_string())),
        update_version: 0,
    })
}

/// Validate that `is_ai` and `ai_capabilities` agree per spec 3.6.10.1.
///
/// - `is_ai = true` → `ai_capabilities` MUST be Some and contain all required keys
/// - `is_ai = false` → `ai_capabilities` MUST be None
///
/// Required Phase 2 keys are encoded as struct fields on `AiCapabilities`, so
/// successful deserialisation already proves they are present. Future required
/// keys SHOULD be added as struct fields (causing serde to reject older
/// registrations missing them), not as values in `extra`.
pub fn validate_ai_declaration(
    is_ai: bool,
    capabilities: Option<&AiCapabilities>,
) -> Result<(), RegistrationError> {
    match (is_ai, capabilities) {
        (true, Some(_)) => Ok(()),
        (false, None) => Ok(()),
        // is_ai = true without capabilities, or is_ai = false with capabilities.
        _ => Err(RegistrationError::AiDeclarationInvalid),
    }
}

/// Validate an `identity.update` `changes` object. Per spec 3.6.10.2, the
/// `is_ai` flag is immutable: any update whose changes include the `is_ai`
/// key MUST be rejected with error 3041.
///
/// `changes` is the value of `IdentityMessage::Update.changes` — typically a
/// JSON object. Non-object values are passed through (other validators handle
/// shape; this fn only enforces the is_ai immutability rule).
pub fn validate_update_changes(changes: &serde_json::Value) -> Result<(), RegistrationError> {
    if let Some(obj) = changes.as_object() {
        if obj.contains_key("is_ai") {
            return Err(RegistrationError::AiFlagImmutable);
        }
    }
    Ok(())
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
            protocol_version,
            identity_id,
            display_name,
            is_ai,
            ai_capabilities,
            trust_assertion,
            timestamp,
            ..
        } => IdentityMessage::Register {
            protocol_version,
            identity_id,
            display_name,
            is_ai,
            ai_capabilities,
            trust_assertion,
            timestamp,
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
                protocol_version,
                identity_id,
                is_ai,
                ai_capabilities,
                trust_assertion,
                timestamp,
                signature,
                ..
            } => IdentityMessage::Register {
                protocol_version,
                identity_id,
                is_ai,
                ai_capabilities,
                trust_assertion,
                timestamp,
                signature,
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
        assert_eq!(record.identity_id.as_str(), id);
        assert_eq!(record.display_name.as_deref(), Some("Alice"));
        assert_eq!(record.home_node.as_str(), HOME);
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

    // ── AI Identity extension (spec 3.6.10) ──────────────────────────────────

    fn ai_caps_default() -> AiCapabilities {
        AiCapabilities {
            dm_initiate: false,
            spontaneous_post: false,
            extra: Default::default(),
        }
    }

    #[test]
    fn ai_registration_with_full_capabilities_accepted() {
        let key = keypair::generate();
        let id = identity_id_from_key(&key);
        let msg = sign_register(
            build_register_with_ai(&key, Some("BotAlice".to_string()), true, Some(ai_caps_default())),
            &key,
        );
        let ts = "2026-04-27T12:00:00.000Z";
        let record = accept_registration(&msg, &id, false, true, HOME, ts).unwrap();
        assert!(record.is_ai);
        let caps = record.ai_capabilities.expect("AI record must carry capabilities");
        assert!(!caps.dm_initiate);
        assert!(!caps.spontaneous_post);
    }

    #[test]
    fn ai_true_without_capabilities_rejected() {
        // is_ai = true but ai_capabilities omitted → 3040
        let key = keypair::generate();
        let id = identity_id_from_key(&key);
        let msg = sign_register(build_register_with_ai(&key, None, true, None), &key);
        let err = accept_registration(&msg, &id, false, true, HOME, "ts").unwrap_err();
        assert_eq!(err, RegistrationError::AiDeclarationInvalid);
        assert_eq!(err.to_registration_code(), (3040, "ai_declaration_invalid"));
    }

    #[test]
    fn ai_false_with_capabilities_rejected() {
        // is_ai = false but ai_capabilities is Some → 3040
        let key = keypair::generate();
        let id = identity_id_from_key(&key);
        let msg = sign_register(
            build_register_with_ai(&key, None, false, Some(ai_caps_default())),
            &key,
        );
        let err = accept_registration(&msg, &id, false, true, HOME, "ts").unwrap_err();
        assert_eq!(err, RegistrationError::AiDeclarationInvalid);
    }

    #[test]
    fn human_registration_record_carries_default_ai_fields() {
        // is_ai = false, ai_capabilities = None — legacy human registration shape.
        let key = keypair::generate();
        let id = identity_id_from_key(&key);
        let msg = make_signed_register(&key, Some("Alice"));
        let record = accept_registration(&msg, &id, false, true, HOME, "ts").unwrap();
        assert!(!record.is_ai);
        assert!(record.ai_capabilities.is_none());
    }

    #[test]
    fn ai_capabilities_with_extra_keys_preserved() {
        // Open-enum forward compat: extra unknown keys must survive accept_registration.
        let key = keypair::generate();
        let id = identity_id_from_key(&key);
        let mut caps = ai_caps_default();
        caps.extra.insert(
            "com.example.experimental_flag".to_string(),
            serde_json::json!(true),
        );
        let msg = sign_register(
            build_register_with_ai(&key, None, true, Some(caps)),
            &key,
        );
        let record = accept_registration(&msg, &id, false, true, HOME, "ts").unwrap();
        let stored = record.ai_capabilities.expect("AI record must carry capabilities");
        assert_eq!(
            stored.extra.get("com.example.experimental_flag"),
            Some(&serde_json::json!(true)),
        );
    }

    #[test]
    fn update_changing_is_ai_rejected_3041() {
        // M3 widened the 3041 wire name from `ai_flag_immutable` to the
        // umbrella `ai_role_violation`. is_ai immutability is one specific
        // role violation (you cannot change your AI role); the umbrella
        // also covers state.space_create/dm_space_create from an AI,
        // wrong-signer/target failures on delegate/revoke.
        let changes = serde_json::json!({"is_ai": true});
        let err = validate_update_changes(&changes).unwrap_err();
        assert_eq!(err, RegistrationError::AiFlagImmutable);
        assert_eq!(err.to_registration_code(), (3041, "ai_role_violation"));
    }

    #[test]
    fn update_changing_other_fields_allowed() {
        // Capability updates and display-name updates are allowed (spec 3.6.10.5).
        let changes = serde_json::json!({"display_name": "Alice v3"});
        assert!(validate_update_changes(&changes).is_ok());
        let cap_changes = serde_json::json!({
            "ai_capabilities": {"dm_initiate": true, "spontaneous_post": false}
        });
        assert!(validate_update_changes(&cap_changes).is_ok());
    }

    #[test]
    fn ai_registration_signature_includes_ai_fields_in_canonical_form() {
        // If is_ai/ai_capabilities are excluded from canonical form, an attacker
        // could strip them after signing without breaking the signature. Verify
        // that stripping is_ai breaks verification.
        let key = keypair::generate();
        let signed = sign_register(
            build_register_with_ai(&key, None, true, Some(ai_caps_default())),
            &key,
        );
        let tampered = match signed {
            IdentityMessage::Register {
                protocol_version,
                identity_id,
                display_name,
                ai_capabilities,
                trust_assertion,
                timestamp,
                signature,
                ..
            } => IdentityMessage::Register {
                protocol_version,
                identity_id,
                display_name,
                is_ai: false, // flipped
                ai_capabilities,
                trust_assertion,
                timestamp,
                signature,
            },
            _ => unreachable!(),
        };
        assert!(verify_register(&tampered).is_err());
    }

    #[test]
    fn human_registration_canonical_form_unchanged_by_3_6_10() {
        // A human (is_ai=false, ai_capabilities=None) registration must produce
        // the same canonical bytes as if the 3.6.10 fields were never added —
        // both fields are skip_serializing when default.
        let key = keypair::generate();
        let msg = build_register(&key, Some("Alice".to_string()));
        let v = serde_json::to_value(&msg).unwrap();
        assert!(v.as_object().unwrap().get("is_ai").is_none(),
            "is_ai = false must not appear in serialised form");
        assert!(v.as_object().unwrap().get("ai_capabilities").is_none(),
            "None ai_capabilities must not appear in serialised form");
    }
}
