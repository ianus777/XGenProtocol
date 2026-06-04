// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Trust Assertion — the signed statement an Auth Module issues certifying that
//! an Identity has been verified to a declared Tier (ch3 §3.8.4 / §3.8.5).
//!
//! The Trust Assertion is the third [`SignedPrimitive`] in the protocol — the
//! other two are the `Event` and the `node_announcement`. Like them, it is
//! self-certifying: any verifier reproduces the canonical signed bytes from the
//! struct alone and checks the Ed25519 signature against the `issuer` public key.
//!
//! ## Arc E (PG-03) — primitive completion
//!
//! Before Arc E this was scaffolding only — the [`crate::xgid::TrustAssertionXgid`]
//! flavour wrapper and the `TrustAssertionRequired` error variant existed, but the
//! struct did not, so registration steps 5–7 (ch3 §3.8.5) were dead code. This
//! module lands the struct, its canonical form, and its sign/verify so the
//! registration pipeline can validate assertions for real (AE-D1, AE-D2, AE-D4).
//!
//! ## Canonical form (AE-D2 / ch3 §3.8.5)
//!
//! The signed bytes follow the same rules as Event canonicalisation (3.2.4) —
//! no whitespace, object keys sorted, UTF-8 — with the **Trust-Assertion field
//! order** `type, tier, issuer, identity_id, issued_at, valid_until, claims`.
//! `signature` is excluded by omission from that order. The machinery is
//! [`crate::canonical::canonical_object_json`] (NOT `canonical_event_bytes`,
//! which carries the Event-envelope order) — the same helper the Node
//! announcement and `identity.register` already canonicalise through.
//!
//! ## Scope (AE-D4)
//!
//! Struct + canonical/sign/verify + a synthetic test issuer are in scope. The
//! live Auth Module service (Tier 2–4 institutional) is out of scope — the
//! verification logic here is real Ed25519, exercised by a test keypair acting
//! as the issuer.

use std::collections::BTreeMap;

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::canonical::canonical_object_json;
use crate::xgid::{AuthModuleXgid, Xgid};

/// Canonical field order for a Trust Assertion's signed form (ch3 §3.8.5).
/// `signature` is excluded by omission. `type` is first and participates in the
/// signed bytes, so a forged `type` breaks the signature — a verifier
/// reproduces it from [`trust_assertion_type`] via the struct's serde default.
const TRUST_ASSERTION_FIELDS: &[&str] = &[
    "type",
    "tier",
    "issuer",
    "identity_id",
    "issued_at",
    "valid_until",
    "claims",
];

/// The fixed `type` discriminator value for a Trust Assertion (ch3 §3.8.4).
fn trust_assertion_type() -> String {
    "trust_assertion".to_string()
}

/// Errors from [`TrustAssertion::verify`] — signature-shape and Ed25519 failures
/// only. Trust (issuer), expiry, and claims are checked by the Node's
/// `validate_assertion` (xgen-core), not here.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum TrustAssertionVerifyError {
    #[error("trust assertion signature field is absent")]
    SignatureMissing,
    #[error("trust assertion signature string is malformed")]
    SignatureMalformed,
    #[error("trust assertion issuer is not a decodable pubkey URI: {0}")]
    IssuerKey(String),
    #[error("trust assertion signature verification failed")]
    SignatureInvalid,
}

/// The `claims` object of a Trust Assertion (ch3 §3.8.4). `tier_verified` is the
/// only mandatory claim; the rest are optional and reflect what the Auth Module
/// actually verified. Unknown claim keys are preserved round-trip (open-namespace
/// forward compat — mirrors [`crate::wire::AiCapabilities`]'s `extra`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TrustClaims {
    /// MANDATORY — the Auth Module certifies this Identity meets the Tier standard.
    pub tier_verified: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email_verified: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phone_verified: Option<bool>,
    /// Salted SHA-256 hash of the email (Option A privacy — only the hash
    /// propagates; ch3 §3.8.4). Plaintext contact details are never permitted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phone_hash: Option<String>,
    /// Unknown claim keys, preserved round-trip for forward compatibility.
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl TrustClaims {
    /// True if a named contact claim is present and asserts a positive
    /// verification (ch3 §3.8.5 check 7 — the Node-policy required-claims gate).
    /// Known boolean claims read their field; a hash claim is satisfied by
    /// presence; unknown keys consult `extra` (truthy when `true` or a
    /// non-empty string).
    pub fn has_claim(&self, key: &str) -> bool {
        match key {
            "tier_verified" => self.tier_verified,
            "email_verified" => self.email_verified == Some(true),
            "phone_verified" => self.phone_verified == Some(true),
            "email_hash" => self.email_hash.is_some(),
            "phone_hash" => self.phone_hash.is_some(),
            other => match self.extra.get(other) {
                Some(Value::Bool(b)) => *b,
                Some(Value::String(s)) => !s.is_empty(),
                Some(Value::Null) | None => false,
                Some(_) => true,
            },
        }
    }
}

/// A signed Trust Assertion (ch3 §3.8.4). Fields are exactly the wire schema;
/// `valid_until` (not `expires_at`) is wire-authoritative per AE-D1.
///
/// `jurisdiction` is deliberately omitted (AE-D5 reversal) — a SignedPrimitive's
/// canonical field set is locked by §3.8.5, so any added field either breaks the
/// signature contract or becomes an unsigned side-field (wrong for a verifiable
/// artefact). Jurisdictional namespacing belongs to PG-04 / arc G.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TrustAssertion {
    /// Canonical discriminator — always `"trust_assertion"` (ch3 §3.8.4). Part of
    /// the signed bytes; defaulted so a deserialised assertion that omits it still
    /// reproduces the signed form.
    #[serde(rename = "type", default = "trust_assertion_type")]
    pub kind: String,
    /// Auth Tier this assertion certifies (1–4).
    pub tier: u32,
    /// Auth Module that issued this assertion — a `xgen://pubkey/ed25519:` URI.
    pub issuer: String,
    /// Identity this assertion is for — a `xgen://pubkey/ed25519:` URI.
    pub identity_id: String,
    /// RFC 3339 UTC timestamp the assertion was issued.
    pub issued_at: String,
    /// RFC 3339 UTC timestamp the assertion expires (AE-D1 — `valid_until`, the
    /// ch3 §3.8.4 wire name; NOT `expires_at`).
    pub valid_until: String,
    /// Verification claims (ch3 §3.8.4).
    pub claims: TrustClaims,
    /// Auth Module signature over the canonical form
    /// (`ed25519:<b64url-key>:<b64url-sig>`). Excluded from the signed bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

impl TrustAssertion {
    /// The canonical signed bytes (ch3 §3.8.5): no whitespace, claims keys sorted,
    /// `signature` excluded, field order `type → tier → … → claims`. Reproducible
    /// by any verifier from the struct alone. Independent of whether `signature`
    /// is set, since the field order excludes it.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let value = serde_json::to_value(self).expect("TrustAssertion derives Serialize");
        canonical_object_json(&value, TRUST_ASSERTION_FIELDS).into_bytes()
    }

    /// Sign this assertion with the issuer's Ed25519 key, returning it with
    /// `signature` populated. The format mirrors `xgen-core::crypto::signing`
    /// (`ed25519:<base64url-pubkey>:<base64url-sig>`) — a wire-stable protocol
    /// constant. The caller is responsible for `issuer` naming `key`'s public key;
    /// [`TrustAssertion::verify`] re-derives the verifying key from `issuer`, so a
    /// mismatch fails verification.
    pub fn sign(mut self, key: &SigningKey) -> Self {
        // canonical_bytes excludes `signature` by field order, so clearing it
        // first is unnecessary — but keep the signed shape unambiguous.
        self.signature = None;
        let bytes = self.canonical_bytes();
        let sig = key.sign(&bytes);
        self.signature = Some(format!(
            "ed25519:{}:{}",
            URL_SAFE_NO_PAD.encode(key.verifying_key().as_bytes()),
            URL_SAFE_NO_PAD.encode(sig.to_bytes()),
        ));
        self
    }

    /// Verify the signature against the **`issuer`** public key over the canonical
    /// bytes (ch3 §3.8.5 check 2). Binds the signature to the `issuer` field — the
    /// key embedded in the signature string is ignored, so an assertion that names
    /// one issuer but is signed by another fails. Does NOT check trust, expiry, or
    /// claims — that is the Node's `validate_assertion`.
    pub fn verify(&self) -> Result<(), TrustAssertionVerifyError> {
        let sig_str = self
            .signature
            .as_deref()
            .ok_or(TrustAssertionVerifyError::SignatureMissing)?;
        let issuer_vk = AuthModuleXgid::from_xgid(Xgid::new(self.issuer.clone()))
            .pubkey()
            .map_err(|e| TrustAssertionVerifyError::IssuerKey(e.to_string()))?;
        let sig = parse_signature(sig_str)?;
        issuer_vk
            .verify(&self.canonical_bytes(), &sig)
            .map_err(|_| TrustAssertionVerifyError::SignatureInvalid)
    }
}

/// Parse the third segment of a `ed25519:<key>:<sig>` string into a `Signature`.
/// Mirrors `xgen-core::crypto::signing::sig_bytes_from_sig_string`.
fn parse_signature(sig_string: &str) -> Result<Signature, TrustAssertionVerifyError> {
    let parts: Vec<&str> = sig_string.splitn(3, ':').collect();
    if parts.len() != 3 || parts[0] != "ed25519" {
        return Err(TrustAssertionVerifyError::SignatureMalformed);
    }
    let sig_bytes = URL_SAFE_NO_PAD
        .decode(parts[2])
        .map_err(|_| TrustAssertionVerifyError::SignatureMalformed)?;
    let arr: [u8; 64] = sig_bytes
        .try_into()
        .map_err(|_| TrustAssertionVerifyError::SignatureMalformed)?;
    Ok(Signature::from_bytes(&arr))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xgid::AuthModuleXgid;
    use serde_json::json;

    /// Deterministic test signing key — avoids a dev-only `rand` dependency for
    /// pure round-trip tests. Sibling shape to flavours.rs::test_signing_key.
    fn test_signing_key(seed: u8) -> SigningKey {
        SigningKey::from_bytes(&[seed; 32])
    }

    fn sample_claims() -> TrustClaims {
        TrustClaims {
            tier_verified: true,
            email_verified: Some(true),
            phone_verified: Some(false),
            email_hash: Some("sha256:a3f9b2c1".to_string()),
            phone_hash: None,
            extra: BTreeMap::new(),
        }
    }

    /// Build a (signed) assertion whose `issuer` names the supplied key.
    fn signed_assertion(issuer_key: &SigningKey, identity_id: &str) -> TrustAssertion {
        let issuer = AuthModuleXgid::from_pubkey(&issuer_key.verifying_key()).to_string();
        TrustAssertion {
            kind: trust_assertion_type(),
            tier: 1,
            issuer,
            identity_id: identity_id.to_string(),
            issued_at: "2026-04-26T10:06:00.000Z".to_string(),
            valid_until: "2027-04-26T00:00:00.000Z".to_string(),
            claims: sample_claims(),
            signature: None,
        }
        .sign(issuer_key)
    }

    #[test]
    fn serde_round_trip_preserves_fields() {
        let key = test_signing_key(0x11);
        let ta = signed_assertion(&key, "xgen://pubkey/ed25519:CLIENT");
        let json = serde_json::to_string(&ta).unwrap();
        let back: TrustAssertion = serde_json::from_str(&json).unwrap();
        assert_eq!(ta, back);
    }

    #[test]
    fn serialised_form_uses_type_discriminator() {
        let key = test_signing_key(0x11);
        let ta = signed_assertion(&key, "xgen://pubkey/ed25519:CLIENT");
        let v = serde_json::to_value(&ta).unwrap();
        assert_eq!(v["type"], json!("trust_assertion"));
        // `kind` (the Rust field) must serialise as `type`, not `kind`.
        assert!(v.as_object().unwrap().get("kind").is_none());
    }

    #[test]
    fn canonical_bytes_match_spec_field_order() {
        let key = test_signing_key(0x11);
        let ta = signed_assertion(&key, "xgen://pubkey/ed25519:CLIENT");
        let canonical = String::from_utf8(ta.canonical_bytes()).unwrap();
        // type first, signature excluded, no whitespace.
        assert!(canonical.starts_with("{\"type\":\"trust_assertion\","));
        assert!(!canonical.contains("signature"));
        assert!(!canonical.contains(": "));
        // Field order: type < tier < issuer < identity_id < issued_at < valid_until < claims.
        let order = ["\"type\"", "\"tier\"", "\"issuer\"", "\"identity_id\"", "\"issued_at\"", "\"valid_until\"", "\"claims\""];
        let mut last = 0;
        for field in order {
            let pos = canonical.find(field).unwrap();
            assert!(pos >= last, "field {field} out of canonical order");
            last = pos;
        }
    }

    #[test]
    fn canonical_bytes_independent_of_signature_presence() {
        let key = test_signing_key(0x11);
        let unsigned = TrustAssertion {
            kind: trust_assertion_type(),
            tier: 1,
            issuer: AuthModuleXgid::from_pubkey(&key.verifying_key()).to_string(),
            identity_id: "xgen://pubkey/ed25519:CLIENT".to_string(),
            issued_at: "2026-04-26T10:06:00.000Z".to_string(),
            valid_until: "2027-04-26T00:00:00.000Z".to_string(),
            claims: sample_claims(),
            signature: None,
        };
        let signed = unsigned.clone().sign(&key);
        assert_eq!(unsigned.canonical_bytes(), signed.canonical_bytes());
    }

    #[test]
    fn sign_verify_round_trip() {
        let key = test_signing_key(0x22);
        let ta = signed_assertion(&key, "xgen://pubkey/ed25519:CLIENT");
        assert!(ta.verify().is_ok());
    }

    #[test]
    fn tampered_tier_fails_verification() {
        let key = test_signing_key(0x22);
        let mut ta = signed_assertion(&key, "xgen://pubkey/ed25519:CLIENT");
        ta.tier = 4; // bump after signing
        assert_eq!(ta.verify(), Err(TrustAssertionVerifyError::SignatureInvalid));
    }

    #[test]
    fn tampered_claims_fail_verification() {
        let key = test_signing_key(0x22);
        let mut ta = signed_assertion(&key, "xgen://pubkey/ed25519:CLIENT");
        ta.claims.tier_verified = false;
        assert_eq!(ta.verify(), Err(TrustAssertionVerifyError::SignatureInvalid));
    }

    #[test]
    fn signature_by_other_key_but_issuer_claims_first_fails() {
        // Forge: name issuer A, but sign with B. verify() derives the key from
        // `issuer` (A), so B's signature cannot verify.
        let issuer_a = test_signing_key(0x33);
        let other_b = test_signing_key(0x44);
        let mut ta = signed_assertion(&issuer_a, "xgen://pubkey/ed25519:CLIENT");
        // Re-sign canonical bytes with B but leave issuer naming A.
        let bytes = ta.canonical_bytes();
        let sig = other_b.sign(&bytes);
        ta.signature = Some(format!(
            "ed25519:{}:{}",
            URL_SAFE_NO_PAD.encode(other_b.verifying_key().as_bytes()),
            URL_SAFE_NO_PAD.encode(sig.to_bytes()),
        ));
        assert_eq!(ta.verify(), Err(TrustAssertionVerifyError::SignatureInvalid));
    }

    #[test]
    fn missing_signature_rejected() {
        let key = test_signing_key(0x22);
        let mut ta = signed_assertion(&key, "xgen://pubkey/ed25519:CLIENT");
        ta.signature = None;
        assert_eq!(ta.verify(), Err(TrustAssertionVerifyError::SignatureMissing));
    }

    #[test]
    fn malformed_signature_rejected() {
        let key = test_signing_key(0x22);
        let mut ta = signed_assertion(&key, "xgen://pubkey/ed25519:CLIENT");
        ta.signature = Some("not-a-valid-sig".to_string());
        assert_eq!(ta.verify(), Err(TrustAssertionVerifyError::SignatureMalformed));
    }

    #[test]
    fn unparseable_issuer_rejected() {
        let key = test_signing_key(0x22);
        let mut ta = signed_assertion(&key, "xgen://pubkey/ed25519:CLIENT");
        ta.issuer = "not-a-pubkey-uri".to_string();
        assert!(matches!(ta.verify(), Err(TrustAssertionVerifyError::IssuerKey(_))));
    }

    #[test]
    fn unknown_claim_keys_preserved_round_trip() {
        let raw = json!({
            "type": "trust_assertion",
            "tier": 1,
            "issuer": "xgen://pubkey/ed25519:AUTH",
            "identity_id": "xgen://pubkey/ed25519:CLIENT",
            "issued_at": "2026-04-26T10:06:00.000Z",
            "valid_until": "2027-04-26T00:00:00.000Z",
            "claims": {
                "tier_verified": true,
                "com.example.experimental": "future-value"
            }
        });
        let ta: TrustAssertion = serde_json::from_value(raw).unwrap();
        assert_eq!(
            ta.claims.extra.get("com.example.experimental"),
            Some(&json!("future-value"))
        );
        // Round-trips back through serialisation.
        let back: TrustAssertion =
            serde_json::from_value(serde_json::to_value(&ta).unwrap()).unwrap();
        assert_eq!(
            back.claims.extra.get("com.example.experimental"),
            Some(&json!("future-value"))
        );
    }

    #[test]
    fn type_omitted_on_deserialise_defaults_in() {
        // A wire payload that omits `type` still parses and canonicalises with
        // the discriminator filled by the serde default (so verify() reproduces
        // the signer's bytes).
        let raw = json!({
            "tier": 1,
            "issuer": "xgen://pubkey/ed25519:AUTH",
            "identity_id": "xgen://pubkey/ed25519:CLIENT",
            "issued_at": "2026-04-26T10:06:00.000Z",
            "valid_until": "2027-04-26T00:00:00.000Z",
            "claims": { "tier_verified": true }
        });
        let ta: TrustAssertion = serde_json::from_value(raw).unwrap();
        assert_eq!(ta.kind, "trust_assertion");
        let canonical = String::from_utf8(ta.canonical_bytes()).unwrap();
        assert!(canonical.starts_with("{\"type\":\"trust_assertion\","));
    }

    #[test]
    fn signature_format_matches_protocol_constant() {
        // ed25519:<43-char b64url pubkey>:<86-char b64url sig> — same shape as
        // xgen-core::crypto::signing::signature_format.
        let key = test_signing_key(0x55);
        let ta = signed_assertion(&key, "xgen://pubkey/ed25519:CLIENT");
        let sig = ta.signature.unwrap();
        let parts: Vec<&str> = sig.splitn(3, ':').collect();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0], "ed25519");
        assert_eq!(parts[1].len(), 43);
        assert_eq!(parts[2].len(), 86);
    }

    #[test]
    fn has_claim_reads_known_and_extra_fields() {
        let mut claims = sample_claims();
        assert!(claims.has_claim("tier_verified"));
        assert!(claims.has_claim("email_verified"));
        assert!(!claims.has_claim("phone_verified")); // Some(false)
        assert!(claims.has_claim("email_hash"));
        assert!(!claims.has_claim("phone_hash")); // None
        assert!(!claims.has_claim("org.absent.key"));
        claims
            .extra
            .insert("org.example.flag".to_string(), json!(true));
        assert!(claims.has_claim("org.example.flag"));
        claims
            .extra
            .insert("org.example.off".to_string(), json!(false));
        assert!(!claims.has_claim("org.example.off"));
    }
}
