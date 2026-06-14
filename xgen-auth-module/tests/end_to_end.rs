// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! M10.2 end-to-end witnesses (RED-on-revert) — the binary's issuance chained to
//! the Node's validation against a **registry-trusted** issuer.
//!
//! Witness 1 (end-to-end accept): module issues → identity registers with the
//! assertion → node validates against the registry-trusted issuer → accepted.
//! Witness 2 (live revoke): the issuer is `revoke`d → the same registration is
//! rejected (3006), proving the trust set is **live-read** from the registry
//! (M10.2-D2), not a startup snapshot.
//!
//! Validation is exercised via `accept_registration` against a policy whose
//! `trusted_issuers` is derived from the registry (`AuthModuleRegistry::
//! trusted_issuers`) — the exact composition the node gate runs. This mirrors the
//! `identity_integration.rs` precedent (registration tested through
//! `accept_registration`, not a live WS connection).

use std::collections::HashSet;

use xgen_auth_module::{issue, issue_tier1, module_xgid};
use xgen_core::auth::module_registry::{AuthModuleRecord, AuthModuleRegistry};
use xgen_core::auth::tiers::AuthTier;
use xgen_core::identity::keypair;
use xgen_core::identity::registration::{
    accept_registration, identity_id_from_key, sign_register, AssertionPolicy, RegistrationError,
};
use xgen_core::wire::types::IdentityMessage;

const HOME: &str = "xgen://pubkey/ed25519:HOMENODE";
const FUTURE: &str = "2030-01-01T00:00:00.000Z";
const REGISTERED_AT: &str = "2026-06-13T12:00:00.000Z";

/// A registry record trusting `module` (the operator-CRUD shape).
fn record_for(module: &ed25519_dalek::SigningKey) -> AuthModuleRecord {
    AuthModuleRecord {
        module_id: module_xgid(module),
        endpoint_url: String::new(),
        accepted_tiers: vec![],
        registered_at: REGISTERED_AT.to_string(),
        revoked: false,
        revoked_at: None,
    }
}

/// Build a signed `identity.register` carrying the module-issued assertion.
fn signed_register(
    identity_key: &ed25519_dalek::SigningKey,
    assertion: &xgen_common::TrustAssertion,
) -> (IdentityMessage, String) {
    let identity_id = identity_id_from_key(identity_key);
    let msg = IdentityMessage::Register {
        protocol_version: "0.1".to_string(),
        identity_id: identity_id.clone(),
        display_name: None,
        is_ai: false,
        ai_capabilities: None,
        trust_assertion: Some(serde_json::to_value(assertion).unwrap()),
        re_registration: false,
        timestamp: REGISTERED_AT.to_string(),
        signature: None,
    };
    (sign_register(msg, identity_key), identity_id)
}

/// The gate's live-read composition: snapshot policy posture (required_claims /
/// required_tier) + `trusted_issuers` derived live from the registry.
fn gate_policy(reg: &AuthModuleRegistry) -> AssertionPolicy {
    AssertionPolicy {
        trusted_issuers: reg.trusted_issuers(),
        // M10.3-D1 — derive per-issuer accepted tiers live from the registry,
        // exactly as the node gate does (so the C2 check is enforcement-bearing
        // in these end-to-end witnesses).
        accepted_tiers_by_issuer: reg.accepted_tiers_by_issuer(),
        required_claims: Vec::new(),
        required_tier: 1,
    }
}

#[test]
fn witness1_end_to_end_accept_against_registry_trusted_issuer() {
    let module = keypair::generate();
    let identity = keypair::generate();
    let assertion = issue_tier1(&module, &identity_id_from_key(&identity), FUTURE);
    let (msg, id) = signed_register(&identity, &assertion);

    // Trusted via the registry → accepted, and the validated tier is persisted.
    let mut reg = AuthModuleRegistry::new();
    reg.register(record_for(&module));
    let record = accept_registration(&msg, &id, false, false, HOME, REGISTERED_AT, &gate_policy(&reg))
        .expect("registry-trusted module → accepted");
    assert_eq!(record.trust_assertion.as_ref().unwrap()["tier"].as_u64(), Some(1));

    // RED: an empty registry (issuer untrusted) → step 1 rejects (3006).
    let empty = AuthModuleRegistry::new();
    let err = accept_registration(&msg, &id, false, false, HOME, REGISTERED_AT, &gate_policy(&empty))
        .unwrap_err();
    assert!(matches!(err, RegistrationError::AuthModuleUntrusted));
}

#[test]
fn witness2_live_revoke_rejects_without_restart() {
    let module = keypair::generate();
    let identity = keypair::generate();
    let assertion = issue_tier1(&module, &identity_id_from_key(&identity), FUTURE);
    let (msg, id) = signed_register(&identity, &assertion);

    let mut reg = AuthModuleRegistry::new();
    reg.register(record_for(&module));
    // Trusted → accepted.
    assert!(accept_registration(&msg, &id, false, false, HOME, REGISTERED_AT, &gate_policy(&reg)).is_ok());

    // Revoke the issuer in the SAME live registry — no restart, no re-seed.
    reg.revoke(&module_xgid(&module), "2026-06-13T13:00:00.000Z".to_string());
    // The gate re-derives `trusted_issuers` → the revoked issuer is gone →
    // the same registration now rejects (3006). RED if the derivation were a
    // snapshot (the revoke would not bite without a restart).
    let err = accept_registration(&msg, &id, false, false, HOME, REGISTERED_AT, &gate_policy(&reg))
        .unwrap_err();
    assert!(matches!(err, RegistrationError::AuthModuleUntrusted));

    // Sanity: the revoked issuer really is out of the live trust set.
    assert_eq!(reg.trusted_issuers(), HashSet::new());
}

/// Witness 2 (M10.3 C2 `accepted_tiers` scope): the mock issues a T3 assertion,
/// but the operator scoped this issuer to T2 only → rejected with **3012**
/// `assertion_tier_unauthorized` (distinct from a Space-floor 3030). Its T2
/// assertion, in scope, is accepted. RED on revert of the C2 check.
#[test]
fn witness2_c2_accepted_tiers_scope_rejects_out_of_scope_tier() {
    let module = keypair::generate();
    let identity = keypair::generate();
    let id = identity_id_from_key(&identity);

    // Operator trusts this issuer but scopes it to Tier-2 only.
    let mut reg = AuthModuleRegistry::new();
    let mut record = record_for(&module);
    record.accepted_tiers = vec![AuthTier::Tier2];
    reg.register(record);

    // The mock issues a T3 assertion → out of the issuer's scope → 3012.
    let ta3 = issue(&module, &id, AuthTier::Tier3, FUTURE);
    let (msg3, _) = signed_register(&identity, &ta3);
    let err = accept_registration(&msg3, &id, false, false, HOME, REGISTERED_AT, &gate_policy(&reg))
        .unwrap_err();
    assert!(matches!(err, RegistrationError::AssertionTierUnauthorized));
    assert_eq!(err.to_registration_code(), (3012, "assertion_tier_unauthorized"));

    // A T2 assertion from the same issuer is in scope → accepted.
    let ta2 = issue(&module, &id, AuthTier::Tier2, FUTURE);
    let (msg2, _) = signed_register(&identity, &ta2);
    let record = accept_registration(&msg2, &id, false, false, HOME, REGISTERED_AT, &gate_policy(&reg))
        .expect("in-scope T2 accepted");
    assert_eq!(record.trust_assertion.as_ref().unwrap()["tier"].as_u64(), Some(2));
}
