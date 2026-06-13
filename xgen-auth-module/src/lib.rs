// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! `xgen-auth-module` — the Tier-1 **reference** Auth Module (M10.2).
//!
//! A pure **offline signer** (M10.2-D1 / D5): it holds its own Ed25519 keypair
//! and issues Tier-1 [`TrustAssertion`]s signed *as itself*, so an Identity can
//! present a real (non-synthetic) module-signed assertion at registration and
//! drive the live 7-check `validate_assertion` against a registry-trusted issuer.
//! The Node never calls the module — it verifies the presented signature offline
//! against the issuer pubkey (= the module's [`AuthModuleXgid`]). No live
//! endpoint; issuance is out-of-band (ch3 §3.8.7).
//!
//! **What Tier-1 attests (honest, audit A5):** proof-of-key-possession only —
//! `tier_verified: true` at tier 1, no contact claims. The reference module is
//! the demonstrator that the Tier-1 floor works mechanically (Fork 1, over the
//! hardcoded baseline), not a KYC service (that is Tier 2–4, M10.3+).
//!
//! Library-first: this lib holds the issuance logic; `main.rs` is a thin CLI.

use std::collections::BTreeMap;

use chrono::{SecondsFormat, Utc};
use ed25519_dalek::SigningKey;
use xgen_common::{
    AuthModuleXgid, Erasability, ModuleKind, ModulePolicy, Retention, TrustAssertion, TrustClaims,
};

/// The module's principal identity — its [`AuthModuleXgid`], derived from its
/// signing key. Its `to_string()` is the `xgen://pubkey/ed25519:` URI that names
/// this module as the `issuer` of every assertion it signs, and the URI an
/// operator hands to `auth-module register` to trust it.
pub fn module_xgid(module_key: &SigningKey) -> AuthModuleXgid {
    AuthModuleXgid::from_pubkey(&module_key.verifying_key())
}

/// Issue a signed Tier-1 [`TrustAssertion`] for `identity_id`, valid until
/// `valid_until` (RFC 3339 UTC) — the exact shape `validate_assertion` accepts
/// (M10.2-D1). The `issuer` field names this module (`module_xgid`), and the
/// assertion is signed with `module_key`, so any verifier re-derives the issuer
/// key from `issuer` and checks the signature offline.
///
/// Populates the M10.1 descriptor on the signed claims (M10.2-D5): `module_kind:
/// reference` + `module_policy.erasability.retention: erasable` (Tier-1 is the
/// max-erasable endpoint, Arc-I AI-D4). Both ride `claims.extra` and are set
/// **before** signing, so they are covered by the assertion signature.
pub fn issue_tier1(module_key: &SigningKey, identity_id: &str, valid_until: &str) -> TrustAssertion {
    let mut claims = TrustClaims {
        // Tier-1 = proof-of-key-possession; the module certifies the tier only.
        tier_verified: true,
        email_verified: None,
        phone_verified: None,
        email_hash: None,
        phone_hash: None,
        extra: BTreeMap::new(),
    };
    // M10.1 descriptor (set before sign — joins the canonical bytes).
    claims.set_module_kind(ModuleKind::Reference);
    claims.set_module_policy(&ModulePolicy {
        erasability: Some(Erasability {
            retention: Some(Retention::Erasable),
            extra: BTreeMap::new(),
        }),
        extra: BTreeMap::new(),
    });

    TrustAssertion {
        kind: "trust_assertion".to_string(),
        tier: 1,
        issuer: module_xgid(module_key).to_string(),
        identity_id: identity_id.to_string(),
        issued_at: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
        valid_until: valid_until.to_string(),
        claims,
        signature: None,
    }
    .sign(module_key)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FUTURE: &str = "2030-01-01T00:00:00.000Z";

    fn module_key(seed: u8) -> SigningKey {
        SigningKey::from_bytes(&[seed; 32])
    }

    /// The binary genuinely issues a valid, self-signed Tier-1 assertion that
    /// names itself as issuer and carries the M10.1 descriptor — RED if the
    /// issuance drops the signature, mis-names the issuer, or skips the
    /// descriptor (which would ride no signed bytes).
    #[test]
    fn issue_tier1_produces_valid_self_signed_descriptor_assertion() {
        let key = module_key(0xA1);
        let id = "xgen://pubkey/ed25519:CLIENT";
        let ta = issue_tier1(&key, id, FUTURE);

        // Signature verifies against the named issuer (offline, self-certifying).
        ta.verify().expect("issued assertion verifies");
        // Issuer names this module; identity + tier as issued.
        assert_eq!(ta.issuer, module_xgid(&key).to_string());
        assert_eq!(ta.identity_id, id);
        assert_eq!(ta.tier, 1);
        assert!(ta.claims.tier_verified);
        // M10.1 descriptor populated + signature-covered.
        assert_eq!(ta.claims.module_kind(), ModuleKind::Reference);
        let policy = ta.claims.module_policy().expect("module_policy present");
        assert_eq!(
            policy.erasability.and_then(|e| e.retention),
            Some(Retention::Erasable)
        );
    }

    /// Tampering a signed descriptor member after issuance breaks the signature
    /// — confirms the descriptor is inside the signed bytes (not a side-channel).
    #[test]
    fn issued_descriptor_is_signature_covered() {
        let key = module_key(0xA2);
        let mut ta = issue_tier1(&key, "xgen://pubkey/ed25519:CLIENT", FUTURE);
        ta.claims.set_module_kind(ModuleKind::Mock);
        assert!(ta.verify().is_err(), "tampered module_kind must break the signature");
    }
}
