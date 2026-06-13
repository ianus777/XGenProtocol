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

// ── AI-D8 module-policy descriptor + mock self-label (M10.1-D3 / D4) ───────────
//
// Both ride `TrustClaims.extra` (a `BTreeMap<String, Value>`), so they are part of
// the signed canonical bytes: `TrustAssertion::canonical_bytes` includes the whole
// `claims` object, and `canonical::canonical_value` sorts every nested object key
// *recursively* — so a member at any depth under `claims.extra` is signed by the
// issuing Auth Module. A *new top-level `TrustAssertion` field* would be wrong
// (AE-D5: `TRUST_ASSERTION_FIELDS` is a fixed canonical set). This arc lands the
// descriptor as *expression* only — there is no enforcement consumer (no
// default-by-tier resolution, no tier-gate read); interpreting absence is the
// D3-gated consumer's call (Arc-I AI-D4/AI-D8).

/// An Auth Module's self-declared kind (M10.1-D4 / AI-A7). **Expression, not
/// trust** — a Node honours a module only via its explicit `trusted_auth_modules`
/// gate (M10-A-04); this label never grants trust. `reference` is the project's
/// reference build; `mock` is the parameterised T2–T4 demonstrator (M10.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModuleKind {
    Reference,
    Mock,
}

/// The AI-D8 module-policy descriptor (M10.1-D3): an Auth Module's declared policy,
/// carried under `claims.extra["module_policy"]`. **Forward-extensible by design** —
/// `erasability` is its *first* member, not its only one; unknown members are
/// preserved verbatim (the §8 open-doors principle, realised structurally by the
/// flattened `extra`). No protocol change is needed when a module brings a future
/// policy member.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ModulePolicy {
    /// The module's declared erasure/retention policy (Arc-I AI-D8 first member).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub erasability: Option<Erasability>,
    /// Unknown module-policy members, preserved round-trip (forward compat).
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// The erasability sub-descriptor (Arc-I AI-D8 / AI-D4). Itself forward-extensible:
/// `retention` is the declared posture; unknown members are preserved verbatim.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Erasability {
    /// The declared retention posture — one of AI-D4's protocol-fixed endpoints.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retention: Option<Retention>,
    /// Unknown erasability members, preserved round-trip (forward compat).
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Arc-I AI-D4's protocol-fixed erasability endpoints. T1 = max-erasable,
/// T4 = no record destruction; a T2/T3 module declares one of these, bounded by the
/// gradient. M10.1 carries the value; the *enforcement* of the gradient (and the
/// default for an absent descriptor) is the deferred D3-gated consumer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Retention {
    /// Content / identity-binding may be erased (T1 endpoint; module-declarable).
    Erasable,
    /// No record destruction — legal-hold retention (T4 endpoint; module-declarable).
    Retained,
}

impl TrustClaims {
    /// The `claims.extra` key carrying the mock self-label (M10.1-D4).
    pub const MODULE_KIND_KEY: &'static str = "module_kind";
    /// The `claims.extra` key carrying the AI-D8 module-policy descriptor (M10.1-D3).
    pub const MODULE_POLICY_KEY: &'static str = "module_policy";

    /// Read the signed module self-label (M10.1-D4). Absent — or present but
    /// unparseable — resolves to [`ModuleKind::Reference`], the confirmed default
    /// (the reference build is the unlabelled baseline; legacy assertions with no
    /// `module_kind` are reference by definition).
    pub fn module_kind(&self) -> ModuleKind {
        self.extra
            .get(Self::MODULE_KIND_KEY)
            .and_then(|v| serde_json::from_value::<ModuleKind>(v.clone()).ok())
            .unwrap_or(ModuleKind::Reference)
    }

    /// Set the signed module self-label (M10.1-D4). The value joins the canonical
    /// bytes, so it must be set before signing.
    pub fn set_module_kind(&mut self, kind: ModuleKind) {
        self.extra.insert(
            Self::MODULE_KIND_KEY.to_string(),
            serde_json::to_value(kind).expect("ModuleKind derives Serialize"),
        );
    }

    /// Read the AI-D8 module-policy descriptor (M10.1-D3). `None` = absent. A
    /// present-but-malformed value also resolves to `None` — a deliberate lenient
    /// read for this expression-only arc; the D3-gated consumer defines strictness.
    pub fn module_policy(&self) -> Option<ModulePolicy> {
        self.extra
            .get(Self::MODULE_POLICY_KEY)
            .and_then(|v| serde_json::from_value::<ModulePolicy>(v.clone()).ok())
    }

    /// Set the AI-D8 module-policy descriptor (M10.1-D3). The descriptor joins the
    /// canonical bytes, so it must be set before signing.
    pub fn set_module_policy(&mut self, policy: &ModulePolicy) {
        self.extra.insert(
            Self::MODULE_POLICY_KEY.to_string(),
            serde_json::to_value(policy).expect("ModulePolicy derives Serialize"),
        );
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

    // ── M10.1 — AI-D8 descriptor + mock self-label witnesses (RED-on-revert) ──

    /// Unsigned assertion (sample claims) so a test can set descriptor members
    /// *before* signing — the descriptor must be inside the canonical bytes.
    fn unsigned_assertion(issuer_key: &SigningKey, identity_id: &str) -> TrustAssertion {
        TrustAssertion {
            kind: trust_assertion_type(),
            tier: 1,
            issuer: AuthModuleXgid::from_pubkey(&issuer_key.verifying_key()).to_string(),
            identity_id: identity_id.to_string(),
            issued_at: "2026-04-26T10:06:00.000Z".to_string(),
            valid_until: "2027-04-26T00:00:00.000Z".to_string(),
            claims: sample_claims(),
            signature: None,
        }
    }

    /// Witness 1 — `module_kind` + `module_policy` set in `claims.extra` are covered
    /// by the assertion signature (canonical bytes recurse into `claims`). Tampering
    /// a signed descriptor member invalidates the signature. RED if the descriptor
    /// ever rides an unsigned side-channel instead of `claims`.
    #[test]
    fn module_descriptor_is_signature_covered() {
        let key = test_signing_key(0x42);
        let mut ta = unsigned_assertion(&key, "xgen://pubkey/ed25519:CLIENT");
        ta.claims.set_module_kind(ModuleKind::Mock);
        ta.claims.set_module_policy(&ModulePolicy {
            erasability: Some(Erasability {
                retention: Some(Retention::Erasable),
                extra: BTreeMap::new(),
            }),
            extra: BTreeMap::new(),
        });
        let ta = ta.sign(&key);
        ta.verify().expect("freshly signed descriptor assertion verifies");

        // canonical_bytes is recomputed from the struct, so flipping a signed
        // descriptor member after signing must break verification.
        let mut tampered = ta.clone();
        tampered.claims.set_module_kind(ModuleKind::Reference);
        assert_eq!(
            tampered.verify(),
            Err(TrustAssertionVerifyError::SignatureInvalid),
            "tampering the signed module_kind must invalidate the signature"
        );
    }

    /// Witness 2 — §8 open-doors: an unknown member at *both* descriptor depths
    /// round-trips verbatim through the wire AND the typed accessor, and does not
    /// break verification. RED if the `#[serde(flatten)] extra` on `ModulePolicy` /
    /// `Erasability` is dropped (the unknown member would be lost on read).
    #[test]
    fn module_policy_unknown_members_round_trip() {
        let key = test_signing_key(0x43);
        let mut policy = ModulePolicy {
            erasability: Some(Erasability {
                retention: Some(Retention::Retained),
                extra: BTreeMap::from([(
                    "future_erasability_member".to_string(),
                    json!("keep-me"),
                )]),
            }),
            extra: BTreeMap::from([("future_policy_member".to_string(), json!({"k": 1}))]),
        };
        policy.extra.insert("another".to_string(), json!(true));

        let mut ta = unsigned_assertion(&key, "xgen://pubkey/ed25519:CLIENT");
        ta.claims.set_module_policy(&policy);
        let ta = ta.sign(&key);

        let back: TrustAssertion =
            serde_json::from_str(&serde_json::to_string(&ta).unwrap()).unwrap();
        back.verify()
            .expect("unknown descriptor members must not break verification");

        let read = back.claims.module_policy().expect("module_policy present");
        assert_eq!(
            read.extra.get("future_policy_member"),
            Some(&json!({"k": 1})),
            "unknown module_policy member preserved verbatim"
        );
        let er = read.erasability.expect("erasability present");
        assert_eq!(er.retention, Some(Retention::Retained));
        assert_eq!(
            er.extra.get("future_erasability_member"),
            Some(&json!("keep-me")),
            "unknown erasability member preserved verbatim"
        );
    }

    /// Witness 4 — `module_kind` accessor returns the signed value; an assertion
    /// with no `module_kind` key resolves to the confirmed default `Reference`.
    /// RED if the default flips or the read ignores `extra`.
    #[test]
    fn module_kind_accessor_reads_signed_value_and_defaults() {
        let key = test_signing_key(0x44);

        let mut ta = unsigned_assertion(&key, "xgen://pubkey/ed25519:CLIENT");
        ta.claims.set_module_kind(ModuleKind::Mock);
        let ta = ta.sign(&key);
        let back: TrustAssertion =
            serde_json::from_str(&serde_json::to_string(&ta).unwrap()).unwrap();
        assert_eq!(back.claims.module_kind(), ModuleKind::Mock);

        let plain = signed_assertion(&key, "xgen://pubkey/ed25519:CLIENT");
        assert!(!plain.claims.extra.contains_key(TrustClaims::MODULE_KIND_KEY));
        assert_eq!(plain.claims.module_kind(), ModuleKind::Reference);
    }
}
