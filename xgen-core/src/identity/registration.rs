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

use std::collections::{HashMap, HashSet};

use chrono::{DateTime, SecondsFormat, Utc};
use ed25519_dalek::SigningKey;
use thiserror::Error;
use xgen_common::xgid::{IdentityXgid, NodeXgid, Xgid};
use xgen_common::TrustAssertion;

use crate::{
    auth::tiers::{verify_tier_assertion, AuthTier},
    crypto::{encoding, signing},
    identity::registry::{DeviceRecord, IdentityRecord},
    wire::{
        canonical::canonical_object_json,
        types::{AiCapabilities, IdentityMessage, IdentityReplicateMessage},
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
    "re_registration",
    "timestamp",
];

// Canonical field order for identity.home_changed signature (spec 3.13.8).
// Signature excluded; field order matches the §3.13.8 literal JSON.
const HOME_CHANGED_FIELDS: &[&str] = &[
    "protocol_version",
    "type",
    "identity_id",
    "old_home_node_id",
    "new_home_node_id",
    "new_home_node_url",
    "update_version",
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
    #[error("trust assertion identity_id does not match the registering identity (code 3010)")]
    AssertionIdentityMismatch,
    #[error("trust assertion claims insufficient for this node's policy (code 3011)")]
    AssertionClaimsInsufficient,
    #[error("trust assertion tier below this node's required registration tier (code 3030)")]
    AssertionTierInsufficient,
    #[error("issuer is not authorized to attest this tier (code 3032)")]
    AssertionTierUnauthorized,
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
            Self::AssertionIdentityMismatch => (3010, "assertion_identity_mismatch"),
            Self::AssertionClaimsInsufficient => (3011, "assertion_claims_insufficient"),
            Self::AssertionTierInsufficient => (3030, "tier_mismatch"),
            Self::AssertionTierUnauthorized => (3032, "assertion_tier_unauthorized"),
            Self::NodeCapacityExceeded => (3008, "node_capacity_exceeded"),
            Self::DisplayNameInvalid => (3009, "display_name_invalid"),
            Self::AiDeclarationInvalid => (3040, "ai_declaration_invalid"),
            Self::AiFlagImmutable => (3041, "ai_role_violation"),
            Self::WrongMessageType => (3002, "signature_invalid"),
        }
    }
}

/// RC-F-01 / M10.1-D1 — `kyc_verification_pending` re-homed to **3031**.
///
/// ch3 §3.11.7 originally double-defined 3010/3011 (higher-tier Auth Module) on top
/// of the live Arc-E §3.6.5 codes (`assertion_identity_mismatch` /
/// `assertion_claims_insufficient`, emitted by [`RegistrationError::to_registration_code`]
/// above and test-asserted). The reconcile keeps Arc-E's 3010/3011, folds the old
/// §3.11.7 `auth_tier_insufficient` into the **live 3030 `tier_mismatch`** (the same
/// tier gate, already emitted here at check 4 and at the PG-13 join gate via
/// `auth::tiers`), and re-homes `kyc_verification_pending` to **3031** — adjacent to
/// 3030 because tier and KYC are one domain.
///
/// **Reserved / dormant:** no emitter this arc. No Tier-3/4 KYC gate exists yet, so
/// there is no `RegistrationError` variant for it (a variant would imply an emit path
/// that does not exist). The `to_registration_code` map above is **unchanged** —
/// M10.1 changes zero emitted codes. The eventual KYC gate (M10.3 mock / D3 consumer)
/// emits `ASSERTION_KYC_VERIFICATION_PENDING`.
pub const ASSERTION_KYC_VERIFICATION_PENDING: (u32, &str) = (3031, "kyc_verification_pending");

// ── Trust Assertion validation (Arc E PG-03, ch3 §3.8.5) ──────────────────────

/// A Node's Trust-Assertion acceptance policy (AE-D3 / CP-2). Sourced from the
/// `[node]` config at startup and held on `NodeRuntime`; passed by reference into
/// [`accept_registration`]. `xgen-core` owns this type so it never depends on the
/// `xgen-node` `NodeConfig` (the CP-2 constraint — mirrors the M7-standalone
/// config→runtime-handle precedent without threading a node-layer type into core).
///
/// **Empty by default** — a fresh Node trusts no Auth Module, so in production
/// mode every assertion fails step 1 (issuer untrusted) until the operator adds a
/// trusted issuer. Local Node mode bypasses validation entirely (§3.8.8), so the
/// empty default is the honest no-op posture in today's deployments (AE-A9).
#[derive(Debug, Clone)]
pub struct AssertionPolicy {
    /// Trusted Auth Module issuer pubkey URIs (`xgen://pubkey/ed25519:…`).
    /// Step 1 (§3.8.5) requires the assertion's `issuer` to be in this set.
    pub trusted_issuers: HashSet<String>,
    /// Contact-verification claim keys this Node requires (§3.8.5 step 7).
    /// Empty by default — no contact claims demanded.
    pub required_claims: Vec<String>,
    /// Minimum Tier required to register on this Node (§3.8.5 step 4). Defaults to
    /// 1 — any valid Tier-1+ assertion satisfies registration; per-Space tier
    /// gating is the join-time concern (PG-13), not registration.
    pub required_tier: u32,
    /// M10.3 (M10.3-D1) — per-issuer authorized tiers: `issuer URI → accepted_tiers`.
    /// Derived live at the gate from the `AuthModuleRegistry` (beside
    /// `trusted_issuers`), so `AuthModuleRecord.accepted_tiers` becomes
    /// enforcement-bearing. The C2 check (`validate_assertion` step 1.5) requires
    /// `assertion.tier ∈ accepted_tiers[issuer]`. **Restrictive-only (M10.3-D2):**
    /// an empty or absent tier list ⇒ the issuer may attest any tier — so the
    /// check is invisible at the empty/T1 baseline (every M10.2 issuer), preserving
    /// the empty-baseline invariant byte-for-byte. Distinct from `required_tier`
    /// (node-wide floor): per-issuer set-membership vs node-wide `≥`.
    pub accepted_tiers_by_issuer: HashMap<String, Vec<AuthTier>>,
}

impl Default for AssertionPolicy {
    fn default() -> Self {
        Self {
            trusted_issuers: HashSet::new(),
            required_claims: Vec::new(),
            required_tier: 1,
            accepted_tiers_by_issuer: HashMap::new(),
        }
    }
}

/// Validate a Trust Assertion against this Node's policy — the full seven-check
/// §3.8.5 sequence (AE-D3). Activates registration steps 5–7, which were dead
/// code before Arc E (`accept_registration` bound-and-dropped the assertion and
/// the `assertion_signature_invalid` / `assertion_expired` variants were never
/// returned).
///
/// Checks, in §3.8.5 order:
/// 1. `issuer` ∈ `policy.trusted_issuers` — else [`RegistrationError::AuthModuleUntrusted`] (3006)
/// 2. signature verifies against `issuer` — else [`RegistrationError::AssertionSignatureInvalid`] (3004)
/// 3. `identity_id` == registering identity — else [`RegistrationError::AssertionIdentityMismatch`] (3010)
/// 4. `tier` ≥ `policy.required_tier` — else [`RegistrationError::AssertionTierInsufficient`] (3030)
/// 5. `valid_until` is in the future vs `now` — else [`RegistrationError::AssertionExpired`] (3005)
/// 6. `claims.tier_verified == true` — else [`RegistrationError::AssertionClaimsInsufficient`] (3011)
/// 7. all `policy.required_claims` present — else [`RegistrationError::AssertionClaimsInsufficient`] (3011)
///
/// Steps 2/3/4/5/6 are pure-local and always run once an assertion is present;
/// steps 1 + 7 consult `policy`. The tier check (4) reuses
/// [`crate::auth::tiers::verify_tier_assertion`] (no-drift, D-067).
///
/// Wire-code note (Arc E, recorded honestly per D-065): the design guessed "new"
/// codes 3006/3007/3008, but those are already allocated
/// (`auth_module_untrusted` / `already_registered` / `node_capacity_exceeded`).
/// Grounding reuses 3004/3005/3006 (the assertion family, previously dead) and
/// allocates **3010** + **3011** for the two genuinely new failures — the same
/// guessed-code-superseded-at-implementation pattern as the AUTHMOD/BOOT arcs.
pub fn validate_assertion(
    assertion: &TrustAssertion,
    registering_identity_id: &str,
    policy: &AssertionPolicy,
    now: DateTime<Utc>,
) -> Result<(), RegistrationError> {
    // Step 1 — issuer is a trusted Auth Module on this Node.
    if !policy.trusted_issuers.contains(&assertion.issuer) {
        return Err(RegistrationError::AuthModuleUntrusted);
    }
    // Step 1.5 (C2, M10.3-D1/D2) — the trusted issuer is authorized to attest
    // THIS tier (per-issuer scope). Restrictive-only: an empty/absent tier list
    // means unrestricted, so this is invisible at the empty/T1 baseline (every
    // M10.2 issuer). Distinct from Step 4 (node-wide floor): set-membership, not
    // `≥`. A T2-scoped issuer attesting T3 fails here with 3032, even though
    // T3 ≥ any floor.
    if let Some(tiers) = policy.accepted_tiers_by_issuer.get(&assertion.issuer) {
        if !tiers.is_empty() && !tiers.iter().any(|t| t.as_u32() == assertion.tier) {
            return Err(RegistrationError::AssertionTierUnauthorized);
        }
    }
    // Step 2 — signature verifies against the issuer key.
    assertion
        .verify()
        .map_err(|_| RegistrationError::AssertionSignatureInvalid)?;
    // Step 3 — identity_id matches the registering Identity.
    if assertion.identity_id != registering_identity_id {
        return Err(RegistrationError::AssertionIdentityMismatch);
    }
    // Step 4 — tier ≥ the Node's required registration tier (reuse tiers.rs).
    verify_tier_assertion(assertion.tier, policy.required_tier)
        .map_err(|_| RegistrationError::AssertionTierInsufficient)?;
    // Step 5 — valid_until is in the future. A malformed timestamp is treated as
    // expired: an unparseable expiry cannot be proven to be in the future.
    let valid_until = DateTime::parse_from_rfc3339(&assertion.valid_until)
        .map(|d| d.with_timezone(&Utc))
        .map_err(|_| RegistrationError::AssertionExpired)?;
    if valid_until <= now {
        return Err(RegistrationError::AssertionExpired);
    }
    // Step 6 — claims certify the tier.
    if !assertion.claims.tier_verified {
        return Err(RegistrationError::AssertionClaimsInsufficient);
    }
    // Step 7 — Node-policy required contact claims are present.
    for required in &policy.required_claims {
        if !assertion.claims.has_claim(required) {
            return Err(RegistrationError::AssertionClaimsInsufficient);
        }
    }
    Ok(())
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
        re_registration: false,
        timestamp: ts,
        signature: None,
    }
}

/// Set the `re_registration` flag on an unsigned `identity.register` message
/// (S5-D1/D2, spec 3.13.8) before signing, so the flag is part of the canonical
/// signed form when `true`. No-op on non-Register variants.
pub fn set_re_registration(msg: IdentityMessage, flag: bool) -> IdentityMessage {
    match msg {
        IdentityMessage::Register {
            protocol_version,
            identity_id,
            display_name,
            is_ai,
            ai_capabilities,
            trust_assertion,
            timestamp,
            signature,
            ..
        } => IdentityMessage::Register {
            protocol_version,
            identity_id,
            display_name,
            is_ai,
            ai_capabilities,
            trust_assertion,
            re_registration: flag,
            timestamp,
            signature,
        },
        other => other,
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

// ── identity.home_changed (S5-D3, spec 3.13.8) ───────────────────────────────

/// Build an unsigned `identity.home_changed` notification (spec 3.13.8 step 5).
/// Sign with `sign_home_changed` before sending.
pub fn build_home_changed(
    identity_id: &str,
    old_home_node_id: &str,
    new_home_node_id: &str,
    new_home_node_url: &str,
    update_version: u64,
) -> IdentityReplicateMessage {
    IdentityReplicateMessage::HomeChanged {
        protocol_version: "0.1".to_string(),
        identity_id: identity_id.to_string(),
        old_home_node_id: old_home_node_id.to_string(),
        new_home_node_id: new_home_node_id.to_string(),
        new_home_node_url: new_home_node_url.to_string(),
        update_version,
        timestamp: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
        signature: None,
    }
}

/// Sign an `identity.home_changed` message with the Identity keypair (the same
/// signing path as `identity.register`).
pub fn sign_home_changed(
    msg: IdentityReplicateMessage,
    key: &SigningKey,
) -> IdentityReplicateMessage {
    let canonical = canonical_json_for_home_changed(&msg);
    let sig = signing::sign(key, canonical.as_bytes());
    set_home_changed_signature(msg, sig)
}

/// Verify the signature on an incoming `identity.home_changed` message against
/// the Identity's own pubkey (the `identity_id`). Mirrors `verify_register`.
pub fn verify_home_changed(msg: &IdentityReplicateMessage) -> Result<(), RegistrationError> {
    let (identity_id, sig_str) = extract_home_changed_sig(msg)?;
    let vk = parse_vk(identity_id)?;
    let canonical = canonical_json_for_home_changed(msg);
    signing::verify(&vk, canonical.as_bytes(), sig_str)
        .map_err(|_| RegistrationError::SignatureInvalid)
}

// ── 8-step acceptance pipeline (spec 3.6.4) ──────────────────────────────────

/// Run the Node-side acceptance pipeline for an incoming `identity.register`.
///
/// `authenticated_id` is the identity verified during transport authentication.
/// `already_registered` is checked from the registry before calling this.
/// `local_node` = true skips trust assertion checks (steps 4–7, §3.8.8).
/// `home_node_id` is this Node's own node_id URI.
/// `policy` is the Node's Trust-Assertion acceptance policy (Arc E PG-03);
/// it is consulted only in the `!local_node` branch and is `AssertionPolicy::default()`
/// (empty trusted-issuer set, required_tier 1) for Local Node / test paths.
///
/// Returns the `IdentityRecord` to store on success.
pub fn accept_registration(
    msg: &IdentityMessage,
    authenticated_id: &str,
    already_registered: bool,
    local_node: bool,
    home_node_id: &str,
    registered_at: &str,
    policy: &AssertionPolicy,
) -> Result<IdentityRecord, RegistrationError> {
    // Extract fields — must be identity.register
    let (identity_id, display_name, is_ai, ai_capabilities, trust_assertion, re_registration) =
        match msg {
            IdentityMessage::Register {
                identity_id,
                display_name,
                is_ai,
                ai_capabilities,
                trust_assertion,
                re_registration,
                ..
            } => (
                identity_id.as_str(),
                display_name.as_deref(),
                *is_ai,
                ai_capabilities.as_ref(),
                trust_assertion.as_ref(),
                *re_registration,
            ),
            _ => return Err(RegistrationError::WrongMessageType),
        };

    // Step 1 — identity_id matches transport auth
    if identity_id != authenticated_id {
        return Err(RegistrationError::IdentityMismatch);
    }

    // Step 2 — signature verifies
    verify_register(msg)?;

    // Step 3 — not already registered, UNLESS this is an orphan-recovery
    // re-registration (S5-D1/D2, spec 3.13.8): `re_registration:true` permits
    // re-homing an already-known identity_id. Ownership is still proven by
    // Step 1 (3001 identity_mismatch) + Step 2 (signature) above, which fire
    // before this branch — so a `re_registration` flag set on an id the caller
    // does not own is already rejected (3022 stays dormant per design §4.3).
    // On the re-home path the handler stores via `upsert` + bumps
    // `update_version` from the prior record (the registry's own `register`
    // is a second duplicate gate — see app.rs handle_identity_msg).
    if already_registered && !re_registration {
        return Err(RegistrationError::AlreadyRegistered);
    }

    // Steps 4–7 — trust assertion (skipped entirely in Local Node mode, §3.8.8).
    if !local_node {
        // Step 4 — a trust_assertion must be present.
        let raw = trust_assertion.ok_or(RegistrationError::TrustAssertionRequired)?;
        // Parse the wire JSON into the typed assertion (tolerant of unknown claim
        // keys — open-namespace forward compat). A malformed body is a
        // signature-shape failure.
        let assertion: TrustAssertion = serde_json::from_value(raw.clone())
            .map_err(|_| RegistrationError::AssertionSignatureInvalid)?;
        // Steps 1–7 (ch3 §3.8.5) — Arc E PG-03 activates what was dead code.
        // `registered_at` is the caller-supplied "now"; fall back to wall-clock
        // only if it is not parseable (it always is in production).
        let now = DateTime::parse_from_rfc3339(registered_at)
            .map(|d| d.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
        validate_assertion(&assertion, identity_id, policy, now)?;
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
        // A5: a freshly accepted registration is active (never revoked).
        revoked: false,
        revoked_at: None,
        revocation_reason: None,
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

fn canonical_json_for_home_changed(msg: &IdentityReplicateMessage) -> String {
    let v = serde_json::to_value(msg).expect("IdentityReplicateMessage is always serialisable");
    canonical_object_json(&v, HOME_CHANGED_FIELDS)
}

fn set_home_changed_signature(
    msg: IdentityReplicateMessage,
    sig: String,
) -> IdentityReplicateMessage {
    match msg {
        IdentityReplicateMessage::HomeChanged {
            protocol_version,
            identity_id,
            old_home_node_id,
            new_home_node_id,
            new_home_node_url,
            update_version,
            timestamp,
            ..
        } => IdentityReplicateMessage::HomeChanged {
            protocol_version,
            identity_id,
            old_home_node_id,
            new_home_node_id,
            new_home_node_url,
            update_version,
            timestamp,
            signature: Some(sig),
        },
        other => other,
    }
}

fn extract_home_changed_sig(
    msg: &IdentityReplicateMessage,
) -> Result<(&str, &str), RegistrationError> {
    match msg {
        IdentityReplicateMessage::HomeChanged { identity_id, signature, .. } => {
            let sig = signature.as_deref().ok_or(RegistrationError::SignatureInvalid)?;
            Ok((identity_id.as_str(), sig))
        }
        _ => Err(RegistrationError::WrongMessageType),
    }
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
            re_registration,
            timestamp,
            ..
        } => IdentityMessage::Register {
            protocol_version,
            identity_id,
            display_name,
            is_ai,
            ai_capabilities,
            trust_assertion,
            re_registration,
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
                re_registration: false,
                timestamp,
                signature,
                display_name: Some("Eve".to_string()),
            },
            _ => unreachable!(),
        };
        assert!(verify_register(&tampered).is_err());
    }

    // ── identity.home_changed sign/verify (S5-D3) ─────────────────────────────

    #[test]
    fn sign_verify_home_changed_round_trip() {
        let key = keypair::generate();
        let id = identity_id_from_key(&key);
        let msg = build_home_changed(
            &id,
            "xgen://pubkey/ed25519:OLD",
            "xgen://pubkey/ed25519:NEW",
            "wss://new.example.com/xgen",
            5,
        );
        let signed = sign_home_changed(msg, &key);
        assert!(verify_home_changed(&signed).is_ok());
    }

    #[test]
    fn tampered_home_changed_fails_verification() {
        let key = keypair::generate();
        let id = identity_id_from_key(&key);
        let signed = sign_home_changed(
            build_home_changed(
                &id,
                "xgen://pubkey/ed25519:OLD",
                "xgen://pubkey/ed25519:NEW",
                "wss://new.example.com/xgen",
                5,
            ),
            &key,
        );
        // Swap the new_home_node_id after signing — verification must fail.
        let tampered = match signed {
            IdentityReplicateMessage::HomeChanged {
                protocol_version,
                identity_id,
                old_home_node_id,
                new_home_node_url,
                update_version,
                timestamp,
                signature,
                ..
            } => IdentityReplicateMessage::HomeChanged {
                protocol_version,
                identity_id,
                old_home_node_id,
                new_home_node_id: "xgen://pubkey/ed25519:EVIL".to_string(),
                new_home_node_url,
                update_version,
                timestamp,
                signature,
            },
            _ => unreachable!(),
        };
        assert!(verify_home_changed(&tampered).is_err());
    }

    #[test]
    fn home_changed_unsigned_fails_verification() {
        // An unsigned home_changed (signature None) is rejected by verify.
        let key = keypair::generate();
        let id = identity_id_from_key(&key);
        let unsigned = build_home_changed(
            &id,
            "xgen://pubkey/ed25519:OLD",
            "xgen://pubkey/ed25519:NEW",
            "wss://new.example.com/xgen",
            5,
        );
        assert!(verify_home_changed(&unsigned).is_err());
    }

    #[test]
    fn local_node_accept_pipeline_succeeds() {
        let key = keypair::generate();
        let id = identity_id_from_key(&key);
        let msg = make_signed_register(&key, Some("Alice"));
        let ts = "2026-04-27T12:00:00.000Z";
        let record = accept_registration(&msg, &id, false, true, HOME, ts, &AssertionPolicy::default()).unwrap();
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
        let err = accept_registration(&msg, &other_id, false, true, HOME, "ts", &AssertionPolicy::default()).unwrap_err();
        assert_eq!(err, RegistrationError::IdentityMismatch);
    }

    #[test]
    fn already_registered_rejected() {
        let key = keypair::generate();
        let id = identity_id_from_key(&key);
        let msg = make_signed_register(&key, None);
        let err = accept_registration(&msg, &id, true, true, HOME, "ts", &AssertionPolicy::default()).unwrap_err();
        assert_eq!(err, RegistrationError::AlreadyRegistered);
    }

    // ── S5-D1/D2 re-registration (orphan recovery, spec 3.13.8) ───────────────

    fn signed_reregister(key: &SigningKey) -> IdentityMessage {
        let identity_id = identity_id_from_key(key);
        let ts = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
        let msg = IdentityMessage::Register {
            protocol_version: "0.1".to_string(),
            identity_id,
            display_name: Some("Alice".to_string()),
            is_ai: false,
            ai_capabilities: None,
            trust_assertion: None,
            re_registration: true,
            timestamp: ts,
            signature: None,
        };
        sign_register(msg, key)
    }

    #[test]
    fn reregistration_permitted_when_already_registered() {
        // re_registration:true bypasses Step 3 for an already-known identity_id;
        // accept_registration returns the re-homed record (home_node = this Node).
        // The version bump + upsert is the handler's job (Option X) — see app.rs.
        let key = keypair::generate();
        let id = identity_id_from_key(&key);
        let msg = signed_reregister(&key);
        let rec = accept_registration(
            &msg,
            &id,
            true, // already_registered
            true, // local_node (skip assertion checks)
            HOME,
            "2026-06-06T12:00:00.000Z",
            &AssertionPolicy::default(),
        )
        .unwrap();
        assert_eq!(rec.identity_id.as_str(), id);
        assert_eq!(rec.home_node.as_str(), HOME); // re-home target = this Node
    }

    #[test]
    fn reregistration_without_flag_still_rejected_3007() {
        // Without the flag, an already-known id is still a duplicate (unchanged).
        let key = keypair::generate();
        let id = identity_id_from_key(&key);
        let msg = make_signed_register(&key, Some("Alice")); // re_registration: false
        let err = accept_registration(
            &msg,
            &id,
            true,
            true,
            HOME,
            "2026-06-06T12:00:00.000Z",
            &AssertionPolicy::default(),
        )
        .unwrap_err();
        assert_eq!(err, RegistrationError::AlreadyRegistered);
        assert_eq!(err.to_registration_code().0, 3007);
    }

    #[test]
    fn reregistration_flag_on_fresh_id_is_plain_registration() {
        // re_registration:true with !already_registered is just a fresh
        // registration (the orphan-recovery Case B — new home holds no replica).
        let key = keypair::generate();
        let id = identity_id_from_key(&key);
        let msg = signed_reregister(&key);
        let rec = accept_registration(
            &msg,
            &id,
            false, // not already registered
            true,
            HOME,
            "2026-06-06T12:00:00.000Z",
            &AssertionPolicy::default(),
        )
        .unwrap();
        assert_eq!(rec.update_version, 0);
    }

    #[test]
    fn set_re_registration_sets_flag_and_signs() {
        // C2 threading helper: set the flag before signing → it is part of the
        // canonical signed form and the signature verifies.
        let key = keypair::generate();
        let msg = set_re_registration(build_register(&key, Some("Alice".to_string())), true);
        let signed = sign_register(msg, &key);
        assert!(verify_register(&signed).is_ok());
        match signed {
            IdentityMessage::Register { re_registration, .. } => assert!(re_registration),
            _ => unreachable!(),
        }
    }

    #[test]
    fn set_re_registration_false_omits_from_canonical_form() {
        // Setting false leaves re_registration omitted (no signature break for
        // normal registrations) — mirrors the is_ai precedent.
        let key = keypair::generate();
        let signed = sign_register(
            set_re_registration(build_register(&key, Some("Alice".to_string())), false),
            &key,
        );
        let v = serde_json::to_value(&signed).unwrap();
        assert!(v.as_object().unwrap().get("re_registration").is_none());
        assert!(verify_register(&signed).is_ok());
    }

    #[test]
    fn trust_assertion_required_in_non_local_mode() {
        let key = keypair::generate();
        let id = identity_id_from_key(&key);
        // Build a register request without trust_assertion (Local Node style).
        let msg = make_signed_register(&key, None);
        // Run in non-local mode — should fail at step 4.
        let err = accept_registration(&msg, &id, false, false, HOME, "ts", &AssertionPolicy::default()).unwrap_err();
        assert_eq!(err, RegistrationError::TrustAssertionRequired);
    }

    #[test]
    fn display_name_too_long_rejected() {
        let key = keypair::generate();
        let id = identity_id_from_key(&key);
        let long_name = "A".repeat(MAX_DISPLAY_NAME_LEN + 1);
        let msg = make_signed_register(&key, Some(&long_name));
        let err = accept_registration(&msg, &id, false, true, HOME, "ts", &AssertionPolicy::default()).unwrap_err();
        assert_eq!(err, RegistrationError::DisplayNameInvalid);
    }

    #[test]
    fn empty_display_name_rejected() {
        let key = keypair::generate();
        let id = identity_id_from_key(&key);
        let msg = make_signed_register(&key, Some(""));
        let err = accept_registration(&msg, &id, false, true, HOME, "ts", &AssertionPolicy::default()).unwrap_err();
        assert_eq!(err, RegistrationError::DisplayNameInvalid);
    }

    #[test]
    fn display_name_with_control_char_rejected() {
        let key = keypair::generate();
        let id = identity_id_from_key(&key);
        let msg = make_signed_register(&key, Some("Alice\x00hack"));
        let err = accept_registration(&msg, &id, false, true, HOME, "ts", &AssertionPolicy::default()).unwrap_err();
        assert_eq!(err, RegistrationError::DisplayNameInvalid);
    }

    #[test]
    fn no_display_name_accepted() {
        let key = keypair::generate();
        let id = identity_id_from_key(&key);
        let msg = make_signed_register(&key, None);
        let record = accept_registration(&msg, &id, false, true, HOME, "ts", &AssertionPolicy::default()).unwrap();
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
        let record = accept_registration(&msg, &id, false, true, HOME, ts, &AssertionPolicy::default()).unwrap();
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
        let err = accept_registration(&msg, &id, false, true, HOME, "ts", &AssertionPolicy::default()).unwrap_err();
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
        let err = accept_registration(&msg, &id, false, true, HOME, "ts", &AssertionPolicy::default()).unwrap_err();
        assert_eq!(err, RegistrationError::AiDeclarationInvalid);
    }

    #[test]
    fn human_registration_record_carries_default_ai_fields() {
        // is_ai = false, ai_capabilities = None — legacy human registration shape.
        let key = keypair::generate();
        let id = identity_id_from_key(&key);
        let msg = make_signed_register(&key, Some("Alice"));
        let record = accept_registration(&msg, &id, false, true, HOME, "ts", &AssertionPolicy::default()).unwrap();
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
        let record = accept_registration(&msg, &id, false, true, HOME, "ts", &AssertionPolicy::default()).unwrap();
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
                re_registration: false,
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

    // ── Trust Assertion validation (Arc E PG-03, ch3 §3.8.5) ──────────────────

    use std::collections::{BTreeMap, HashSet};
    use xgen_common::xgid::AuthModuleXgid;
    use xgen_common::TrustClaims;

    const FUTURE: &str = "2099-01-01T00:00:00.000Z";
    const PAST: &str = "2020-01-01T00:00:00.000Z";

    fn now() -> DateTime<Utc> {
        Utc::now()
    }

    /// Synthetic Auth Module issuer keypair — stands in for a Tier 2–4 module
    /// (none ships in Arc E; AE-D4). Deterministic seed, no `rand` needed.
    fn issuer_key(seed: u8) -> SigningKey {
        SigningKey::from_bytes(&[seed; 32])
    }

    fn issuer_uri(key: &SigningKey) -> String {
        AuthModuleXgid::from_pubkey(&key.verifying_key()).to_string()
    }

    fn make_assertion(
        issuer: &SigningKey,
        identity_id: &str,
        tier: u32,
        valid_until: &str,
        tier_verified: bool,
    ) -> TrustAssertion {
        TrustAssertion {
            kind: "trust_assertion".to_string(),
            tier,
            issuer: issuer_uri(issuer),
            identity_id: identity_id.to_string(),
            issued_at: "2026-04-26T10:06:00.000Z".to_string(),
            valid_until: valid_until.to_string(),
            claims: TrustClaims {
                tier_verified,
                email_verified: None,
                phone_verified: None,
                email_hash: None,
                phone_hash: None,
                extra: BTreeMap::new(),
            },
            signature: None,
        }
        .sign(issuer)
    }

    fn policy_trusting(issuer: &SigningKey) -> AssertionPolicy {
        AssertionPolicy {
            trusted_issuers: HashSet::from([issuer_uri(issuer)]),
            required_claims: Vec::new(),
            required_tier: 1,
            accepted_tiers_by_issuer: HashMap::new(),
        }
    }

    /// M10.3 — `policy_trusting` plus a per-issuer accepted-tier scope (C2).
    fn policy_trusting_tiers(issuer: &SigningKey, tiers: Vec<AuthTier>) -> AssertionPolicy {
        let mut p = policy_trusting(issuer);
        p.accepted_tiers_by_issuer.insert(issuer_uri(issuer), tiers);
        p
    }

    #[test]
    fn validate_assertion_accepts_valid_synthetic() {
        let issuer = issuer_key(0xA1);
        let ta = make_assertion(&issuer, "xgen://pubkey/ed25519:CLIENT", 1, FUTURE, true);
        assert!(validate_assertion(&ta, "xgen://pubkey/ed25519:CLIENT", &policy_trusting(&issuer), now()).is_ok());
    }

    #[test]
    fn validate_assertion_rejects_untrusted_issuer() {
        // Step 1 — empty trusted-issuer set (the default posture).
        let issuer = issuer_key(0xA1);
        let ta = make_assertion(&issuer, "xgen://pubkey/ed25519:CLIENT", 1, FUTURE, true);
        let err = validate_assertion(&ta, "xgen://pubkey/ed25519:CLIENT", &AssertionPolicy::default(), now()).unwrap_err();
        assert_eq!(err, RegistrationError::AuthModuleUntrusted);
        assert_eq!(err.to_registration_code(), (3006, "auth_module_untrusted"));
    }

    #[test]
    fn validate_assertion_rejects_bad_signature() {
        // Step 2 — tamper the tier after signing; the signature no longer matches.
        let issuer = issuer_key(0xA1);
        let mut ta = make_assertion(&issuer, "xgen://pubkey/ed25519:CLIENT", 1, FUTURE, true);
        ta.tier = 4;
        let err = validate_assertion(&ta, "xgen://pubkey/ed25519:CLIENT", &policy_trusting(&issuer), now()).unwrap_err();
        assert_eq!(err, RegistrationError::AssertionSignatureInvalid);
        assert_eq!(err.to_registration_code(), (3004, "assertion_signature_invalid"));
    }

    #[test]
    fn validate_assertion_rejects_identity_mismatch() {
        // Step 3 — the assertion is for a different Identity than the registrant.
        let issuer = issuer_key(0xA1);
        let ta = make_assertion(&issuer, "xgen://pubkey/ed25519:SOMEONE_ELSE", 1, FUTURE, true);
        let err = validate_assertion(&ta, "xgen://pubkey/ed25519:CLIENT", &policy_trusting(&issuer), now()).unwrap_err();
        assert_eq!(err, RegistrationError::AssertionIdentityMismatch);
        assert_eq!(err.to_registration_code(), (3010, "assertion_identity_mismatch"));
    }

    /// Witness 3 (M10.1-D1, RC-F-01 band reconcile). The Arc-E §3.6.5 codes stay
    /// exactly as emitted (3010/3011 identity-assertion, 3030 `tier_mismatch`), and
    /// `kyc_verification_pending` is reserved at **3031**, distinct from the band.
    /// Zero change to emitted codes; the spec half (§3.11.7 no longer claims
    /// 3010/3011) is verified by the ch3 edit. RED if any wire integer/string drifts.
    #[test]
    fn band_reconcile_codes_unchanged_and_kyc_reserved() {
        assert_eq!(
            RegistrationError::AssertionIdentityMismatch.to_registration_code(),
            (3010, "assertion_identity_mismatch")
        );
        assert_eq!(
            RegistrationError::AssertionClaimsInsufficient.to_registration_code(),
            (3011, "assertion_claims_insufficient")
        );
        assert_eq!(
            RegistrationError::AssertionTierInsufficient.to_registration_code(),
            (3030, "tier_mismatch")
        );
        // `auth_tier_insufficient` folded into the live 3030 (no separate code).
        // `kyc_verification_pending` re-homed to 3031, reserved/dormant.
        assert_eq!(ASSERTION_KYC_VERIFICATION_PENDING, (3031, "kyc_verification_pending"));
        // 3031 is distinct from the live Arc-E band integers.
        let band = [3010u32, 3011, 3030];
        assert!(!band.contains(&ASSERTION_KYC_VERIFICATION_PENDING.0));
    }

    #[test]
    fn validate_assertion_rejects_low_tier() {
        // Step 4 — Node requires Tier 2; a Tier-1 assertion is insufficient.
        let issuer = issuer_key(0xA1);
        let ta = make_assertion(&issuer, "xgen://pubkey/ed25519:CLIENT", 1, FUTURE, true);
        let mut policy = policy_trusting(&issuer);
        policy.required_tier = 2;
        let err = validate_assertion(&ta, "xgen://pubkey/ed25519:CLIENT", &policy, now()).unwrap_err();
        assert_eq!(err, RegistrationError::AssertionTierInsufficient);
        assert_eq!(err.to_registration_code(), (3030, "tier_mismatch"));
    }

    #[test]
    fn validate_assertion_rejects_expired() {
        // Step 5 — valid_until is in the past.
        let issuer = issuer_key(0xA1);
        let ta = make_assertion(&issuer, "xgen://pubkey/ed25519:CLIENT", 1, PAST, true);
        let err = validate_assertion(&ta, "xgen://pubkey/ed25519:CLIENT", &policy_trusting(&issuer), now()).unwrap_err();
        assert_eq!(err, RegistrationError::AssertionExpired);
        assert_eq!(err.to_registration_code(), (3005, "assertion_expired"));
    }

    #[test]
    fn validate_assertion_rejects_tier_not_verified() {
        // Step 6 — claims.tier_verified is false.
        let issuer = issuer_key(0xA1);
        let ta = make_assertion(&issuer, "xgen://pubkey/ed25519:CLIENT", 1, FUTURE, false);
        let err = validate_assertion(&ta, "xgen://pubkey/ed25519:CLIENT", &policy_trusting(&issuer), now()).unwrap_err();
        assert_eq!(err, RegistrationError::AssertionClaimsInsufficient);
        assert_eq!(err.to_registration_code(), (3011, "assertion_claims_insufficient"));
    }

    #[test]
    fn validate_assertion_rejects_missing_required_claim() {
        // Step 7 — Node policy requires email_verified; the assertion lacks it.
        let issuer = issuer_key(0xA1);
        let ta = make_assertion(&issuer, "xgen://pubkey/ed25519:CLIENT", 1, FUTURE, true);
        let mut policy = policy_trusting(&issuer);
        policy.required_claims = vec!["email_verified".to_string()];
        let err = validate_assertion(&ta, "xgen://pubkey/ed25519:CLIENT", &policy, now()).unwrap_err();
        assert_eq!(err, RegistrationError::AssertionClaimsInsufficient);
    }

    #[test]
    fn validate_assertion_accepts_present_required_claim() {
        // Step 7 positive — policy requires email_verified and it is present+true.
        let issuer = issuer_key(0xA1);
        let mut ta = make_assertion(&issuer, "xgen://pubkey/ed25519:CLIENT", 1, FUTURE, true);
        ta.claims.email_verified = Some(true);
        let ta = ta.sign(&issuer); // re-sign with the added claim
        let mut policy = policy_trusting(&issuer);
        policy.required_claims = vec!["email_verified".to_string()];
        assert!(validate_assertion(&ta, "xgen://pubkey/ed25519:CLIENT", &policy, now()).is_ok());
    }

    // ── M10.3 — C2 per-issuer accepted_tiers (witnesses 2 + 3 at validate level) ─

    /// Witness 2: an issuer scoped to T2 has its T3 assertion rejected with 3032
    /// (distinct from a node-floor 3030); its T2 assertion is accepted.
    #[test]
    fn validate_assertion_c2_rejects_tier_outside_issuer_scope() {
        let issuer = issuer_key(0xA1);
        let policy = policy_trusting_tiers(&issuer, vec![AuthTier::Tier2]);
        let id = "xgen://pubkey/ed25519:CLIENT";

        let ta3 = make_assertion(&issuer, id, 3, FUTURE, true);
        let err = validate_assertion(&ta3, id, &policy, now()).unwrap_err();
        assert!(matches!(err, RegistrationError::AssertionTierUnauthorized));
        assert_eq!(err.to_registration_code(), (3032, "assertion_tier_unauthorized"));

        let ta2 = make_assertion(&issuer, id, 2, FUTURE, true);
        assert!(validate_assertion(&ta2, id, &policy, now()).is_ok(), "T2 is in scope");
    }

    /// Witness 3 (validate level): empty/absent `accepted_tiers` ⇒ any tier accepted
    /// (M10.3-D2 restrictive-only) — the M10.2 empty-baseline invariant. RED if
    /// empty were read as deny-all.
    #[test]
    fn validate_assertion_c2_empty_scope_is_unrestricted() {
        let issuer = issuer_key(0xA1);
        let id = "xgen://pubkey/ed25519:CLIENT";
        let scoped_empty = policy_trusting_tiers(&issuer, vec![]); // issuer present, empty tiers
        let absent = policy_trusting(&issuer); // issuer absent from the tier map
        for tier in [1u32, 2, 3, 4] {
            let ta = make_assertion(&issuer, id, tier, FUTURE, true);
            assert!(
                validate_assertion(&ta, id, &scoped_empty, now()).is_ok(),
                "empty accepted_tiers must accept tier {tier}"
            );
            assert!(
                validate_assertion(&ta, id, &absent, now()).is_ok(),
                "absent tier map must accept tier {tier}"
            );
        }
    }

    // ── accept_registration end-to-end (non-local) ────────────────────────────

    fn signed_register_with_assertion(key: &SigningKey, assertion: &TrustAssertion) -> IdentityMessage {
        let identity_id = identity_id_from_key(key);
        let ts = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
        let msg = IdentityMessage::Register {
            protocol_version: "0.1".to_string(),
            identity_id,
            display_name: None,
            is_ai: false,
            ai_capabilities: None,
            trust_assertion: Some(serde_json::to_value(assertion).unwrap()),
            re_registration: false,
            timestamp: ts,
            signature: None,
        };
        sign_register(msg, key)
    }

    #[test]
    fn non_local_registration_with_valid_assertion_accepted() {
        let issuer = issuer_key(0xB1);
        let key = keypair::generate();
        let id = identity_id_from_key(&key);
        let ta = make_assertion(&issuer, &id, 2, FUTURE, true);
        let msg = signed_register_with_assertion(&key, &ta);
        let record =
            accept_registration(&msg, &id, false, false, HOME, "2026-06-04T12:00:00.000Z", &policy_trusting(&issuer))
                .unwrap();
        // The validated assertion is persisted; its tier is now authoritative.
        assert_eq!(record.trust_assertion.as_ref().unwrap()["tier"].as_u64(), Some(2));
    }

    #[test]
    fn non_local_registration_untrusted_issuer_rejected() {
        let issuer = issuer_key(0xB1);
        let key = keypair::generate();
        let id = identity_id_from_key(&key);
        let ta = make_assertion(&issuer, &id, 1, FUTURE, true);
        let msg = signed_register_with_assertion(&key, &ta);
        // Default policy trusts nobody.
        let err = accept_registration(&msg, &id, false, false, HOME, "2026-06-04T12:00:00.000Z", &AssertionPolicy::default())
            .unwrap_err();
        assert_eq!(err, RegistrationError::AuthModuleUntrusted);
    }

    #[test]
    fn non_local_registration_expired_assertion_rejected() {
        let issuer = issuer_key(0xB1);
        let key = keypair::generate();
        let id = identity_id_from_key(&key);
        let ta = make_assertion(&issuer, &id, 1, PAST, true);
        let msg = signed_register_with_assertion(&key, &ta);
        let err = accept_registration(&msg, &id, false, false, HOME, "2026-06-04T12:00:00.000Z", &policy_trusting(&issuer))
            .unwrap_err();
        assert_eq!(err, RegistrationError::AssertionExpired);
    }

    #[test]
    fn non_local_registration_malformed_assertion_rejected() {
        // A trust_assertion body that is not a valid assertion (missing required
        // fields) fails to parse → AssertionSignatureInvalid.
        let key = keypair::generate();
        let id = identity_id_from_key(&key);
        let ts = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
        let msg = sign_register(
            IdentityMessage::Register {
                protocol_version: "0.1".to_string(),
                identity_id: id.clone(),
                display_name: None,
                is_ai: false,
                ai_capabilities: None,
                trust_assertion: Some(serde_json::json!({"not": "an assertion"})),
                re_registration: false,
                timestamp: ts,
                signature: None,
            },
            &key,
        );
        let issuer = issuer_key(0xB1);
        let err = accept_registration(&msg, &id, false, false, HOME, "2026-06-04T12:00:00.000Z", &policy_trusting(&issuer))
            .unwrap_err();
        assert_eq!(err, RegistrationError::AssertionSignatureInvalid);
    }

    #[test]
    fn local_node_bypasses_assertion_validation() {
        // §3.8.8 — Local Node mode skips steps 4–7 entirely, even with an
        // expired assertion present and an empty (trust-nobody) policy.
        let issuer = issuer_key(0xB1);
        let key = keypair::generate();
        let id = identity_id_from_key(&key);
        let ta = make_assertion(&issuer, &id, 1, PAST, true);
        let msg = signed_register_with_assertion(&key, &ta);
        let record = accept_registration(&msg, &id, false, true, HOME, "2026-06-04T12:00:00.000Z", &AssertionPolicy::default())
            .unwrap();
        assert_eq!(record.identity_id.as_str(), id);
    }
}
