// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

// Auth Module Tier 2–4 interface definitions (spec 3.11.1–3.11.5).
//
// Scope: interface definitions and slot contract enforcement only.
// Tiers 2–4 are built in institutional collaboration with qualified organisations.
// This module defines the contracts those organisations build against.
// No verification logic is implemented here — that is the Auth Module's domain.
//
// The Node verifies the Trust Assertion signature. If valid, the claim fields
// are trusted as-is. The Node does not re-verify legal names, ISO certifications,
// or security clearances.

use serde::{Deserialize, Serialize};
use thiserror::Error;

// ── TTL constants (WD-09, WD-10, WD-11) ──────────────────────────────────────

pub const TIER2_TTL_DAYS: u64 = 365;
pub const TIER3_TTL_DAYS: u64 = 180;
pub const TIER4_TTL_DAYS: u64 = 90;

// ── Auth tier model ───────────────────────────────────────────────────────────

/// The four authentication tiers defined by the XGen Protocol (spec 3.11.1).
///
/// Tier 1: cryptographic identity only (Phase 1 reference implementation).
/// Tiers 2–4: progressively stronger identity verification, implemented by
/// qualified external Auth Modules in institutional collaboration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthTier {
    Tier1 = 1,
    Tier2 = 2,
    Tier3 = 3,
    Tier4 = 4,
}

impl AuthTier {
    pub fn from_u32(v: u32) -> Option<Self> {
        match v {
            1 => Some(Self::Tier1),
            2 => Some(Self::Tier2),
            3 => Some(Self::Tier3),
            4 => Some(Self::Tier4),
            _ => None,
        }
    }

    pub fn as_u32(self) -> u32 {
        self as u32
    }

    pub fn ttl_days(self) -> Option<u64> {
        match self {
            Self::Tier1 => None,
            Self::Tier2 => Some(TIER2_TTL_DAYS),
            Self::Tier3 => Some(TIER3_TTL_DAYS),
            Self::Tier4 => Some(TIER4_TTL_DAYS),
        }
    }
}

// ── Claim structs ─────────────────────────────────────────────────────────────

/// Tier 2 Trust Assertion claims (spec 3.11.2).
/// Organisational identity verification — verified by a qualified Auth Module.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tier2Claims {
    pub tier_verified: u32,
    pub legal_name_verified: bool,
    pub organisation_verified: bool,
    pub organisation_domain: String,
    pub iso27001_operator: bool,
}

/// Tier 3 Trust Assertion claims (spec 3.11.3).
/// All Tier 2 fields plus regulated-entity compliance verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tier3Claims {
    // Tier 2 fields
    pub tier_verified: u32,
    pub legal_name_verified: bool,
    pub organisation_verified: bool,
    pub organisation_domain: String,
    pub iso27001_operator: bool,
    // Tier 3 additions
    pub aml_kyc_cleared: bool,
    pub corporate_role_verified: bool,
    pub audit_trail_maintained: bool,
    pub regulatory_compliance: String,
}

/// Tier 4 Trust Assertion claims (spec 3.11.4).
/// All Tier 3 fields plus government/high-security clearance verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tier4Claims {
    // Tier 2 fields
    pub tier_verified: u32,
    pub legal_name_verified: bool,
    pub organisation_verified: bool,
    pub organisation_domain: String,
    pub iso27001_operator: bool,
    // Tier 3 additions
    pub aml_kyc_cleared: bool,
    pub corporate_role_verified: bool,
    pub audit_trail_maintained: bool,
    pub regulatory_compliance: String,
    // Tier 4 additions
    pub security_clearance_level: String,
    pub jurisdiction: String,
    pub hardware_token_bound: bool,
    pub biometric_verified: bool,
}

// ── Slot contract enforcement ─────────────────────────────────────────────────

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AuthError {
    /// Trust Assertion tier is below the Space's required auth_tier (error 3030).
    #[error("tier mismatch: assertion tier {assertion_tier} is below required tier {required_tier}")]
    TierMismatch { assertion_tier: u32, required_tier: u32 },
    /// Trust Assertion has expired.
    #[error("trust assertion expired: issued {issued_at_secs} secs ago, ttl {ttl_secs} secs")]
    AssertionExpired { issued_at_secs: u64, ttl_secs: u64 },
    /// Unknown tier value in assertion.
    #[error("unknown auth tier: {0}")]
    UnknownTier(u32),
}

impl AuthError {
    /// Map this error to the protocol wire (code, name) tuple (spec 3.11 / §3030).
    ///
    /// `TierMismatch` and `UnknownTier` are both join-admission failures — the
    /// joiner's Trust-Assertion tier does not satisfy the Space's slot contract
    /// (PG-13, Arc D PM-D1) — and map to wire **3030** `tier_mismatch`.
    /// `AssertionExpired` is a TTL concern outside the join tier-gate and has no
    /// wire code here. Mirrors `ExchangeError::to_wire_code` (no-drift, D-067).
    pub fn to_wire_code(&self) -> Option<(u32, &'static str)> {
        match self {
            Self::TierMismatch { .. } | Self::UnknownTier(_) => Some((3030, "tier_mismatch")),
            Self::AssertionExpired { .. } => None,
        }
    }
}

/// Verify that a Trust Assertion's tier satisfies a Space's slot contract.
///
/// `assertion_tier` — the `tier_verified` field from the Trust Assertion claims.
/// `space_auth_tier` — the `auth_tier` field from the Space state.
///
/// Returns `Ok(())` if the assertion tier >= required tier.
/// Returns `Err(AuthError::TierMismatch)` if the assertion tier < required tier.
/// Returns `Err(AuthError::UnknownTier)` if the tier value is not 1–4.
pub fn verify_tier_assertion(assertion_tier: u32, space_auth_tier: u32) -> Result<(), AuthError> {
    if AuthTier::from_u32(assertion_tier).is_none() {
        return Err(AuthError::UnknownTier(assertion_tier));
    }
    if assertion_tier < space_auth_tier {
        return Err(AuthError::TierMismatch { assertion_tier, required_tier: space_auth_tier });
    }
    Ok(())
}

/// Verify that a Trust Assertion has not expired.
///
/// `issued_at_secs` — Unix timestamp when the assertion was issued.
/// `now_secs` — current Unix timestamp.
/// `tier` — the assertion's tier (determines the TTL constant).
///
/// Tier 1 assertions have no TTL. Returns `Ok(())` for Tier 1.
pub fn verify_assertion_ttl(
    issued_at_secs: u64,
    now_secs: u64,
    tier: AuthTier,
) -> Result<(), AuthError> {
    let ttl_days = match tier.ttl_days() {
        None => return Ok(()),
        Some(d) => d,
    };
    let ttl_secs = ttl_days * 86_400;
    let age_secs = now_secs.saturating_sub(issued_at_secs);
    if age_secs > ttl_secs {
        return Err(AuthError::AssertionExpired { issued_at_secs, ttl_secs });
    }
    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn tier2_claims_parsed_correctly() {
        let raw = json!({
            "tier_verified": 2,
            "legal_name_verified": true,
            "organisation_verified": true,
            "organisation_domain": "example.com",
            "iso27001_operator": false
        });
        let claims: Tier2Claims = serde_json::from_value(raw).unwrap();
        assert_eq!(claims.tier_verified, 2);
        assert!(claims.legal_name_verified);
        assert!(claims.organisation_verified);
        assert_eq!(claims.organisation_domain, "example.com");
        assert!(!claims.iso27001_operator);
    }

    #[test]
    fn tier3_claims_parsed_correctly() {
        let raw = json!({
            "tier_verified": 3,
            "legal_name_verified": true,
            "organisation_verified": true,
            "organisation_domain": "bank.eu",
            "iso27001_operator": true,
            "aml_kyc_cleared": true,
            "corporate_role_verified": true,
            "audit_trail_maintained": true,
            "regulatory_compliance": "MiFID II"
        });
        let claims: Tier3Claims = serde_json::from_value(raw).unwrap();
        assert_eq!(claims.tier_verified, 3);
        assert!(claims.aml_kyc_cleared);
        assert_eq!(claims.regulatory_compliance, "MiFID II");
    }

    #[test]
    fn tier4_claims_parsed_correctly() {
        let raw = json!({
            "tier_verified": 4,
            "legal_name_verified": true,
            "organisation_verified": true,
            "organisation_domain": "gov.sk",
            "iso27001_operator": true,
            "aml_kyc_cleared": true,
            "corporate_role_verified": true,
            "audit_trail_maintained": true,
            "regulatory_compliance": "NIS2",
            "security_clearance_level": "SECRET",
            "jurisdiction": "SK",
            "hardware_token_bound": true,
            "biometric_verified": true
        });
        let claims: Tier4Claims = serde_json::from_value(raw).unwrap();
        assert_eq!(claims.tier_verified, 4);
        assert_eq!(claims.security_clearance_level, "SECRET");
        assert_eq!(claims.jurisdiction, "SK");
        assert!(claims.hardware_token_bound);
        assert!(claims.biometric_verified);
    }

    #[test]
    fn tier_mismatch_rejected() {
        // Tier 1 assertion cannot enter a Tier 2 Space.
        let err = verify_tier_assertion(1, 2).unwrap_err();
        assert_eq!(err, AuthError::TierMismatch { assertion_tier: 1, required_tier: 2 });

        // Tier 2 assertion cannot enter a Tier 3 Space.
        let err = verify_tier_assertion(2, 3).unwrap_err();
        assert_eq!(err, AuthError::TierMismatch { assertion_tier: 2, required_tier: 3 });
    }

    #[test]
    fn higher_tier_accepted_in_lower_space() {
        // Tier 3 assertion is accepted in a Tier 2 Space.
        assert!(verify_tier_assertion(3, 2).is_ok());
        // Tier 4 assertion is accepted in a Tier 1 Space.
        assert!(verify_tier_assertion(4, 1).is_ok());
        // Same tier is always accepted.
        assert!(verify_tier_assertion(2, 2).is_ok());
    }

    #[test]
    fn tier2_ttl_enforced() {
        let issued_at: u64 = 1_000_000;
        let ttl_secs = TIER2_TTL_DAYS * 86_400;
        // Within TTL.
        assert!(verify_assertion_ttl(issued_at, issued_at + ttl_secs - 1, AuthTier::Tier2).is_ok());
        // Exactly at expiry boundary — still valid.
        assert!(verify_assertion_ttl(issued_at, issued_at + ttl_secs, AuthTier::Tier2).is_ok());
        // One second past TTL — expired.
        let err =
            verify_assertion_ttl(issued_at, issued_at + ttl_secs + 1, AuthTier::Tier2).unwrap_err();
        assert!(matches!(err, AuthError::AssertionExpired { .. }));
    }

    #[test]
    fn tier1_has_no_ttl() {
        // Tier 1 assertions never expire — no TTL defined.
        let issued_at: u64 = 0;
        let far_future: u64 = u64::MAX / 2;
        assert!(verify_assertion_ttl(issued_at, far_future, AuthTier::Tier1).is_ok());
    }

    #[test]
    fn unknown_tier_rejected() {
        let err = verify_tier_assertion(5, 2).unwrap_err();
        assert_eq!(err, AuthError::UnknownTier(5));
    }

    #[test]
    fn auth_error_wire_codes() {
        // Both admission failures map to wire 3030 (PG-13).
        assert_eq!(
            AuthError::TierMismatch { assertion_tier: 1, required_tier: 2 }.to_wire_code(),
            Some((3030, "tier_mismatch"))
        );
        assert_eq!(AuthError::UnknownTier(7).to_wire_code(), Some((3030, "tier_mismatch")));
        // TTL expiry is not a join tier-gate concern — no wire code here.
        assert_eq!(
            AuthError::AssertionExpired { issued_at_secs: 0, ttl_secs: 1 }.to_wire_code(),
            None
        );
    }

    #[test]
    fn auth_tier_ordering() {
        assert!(AuthTier::Tier1 < AuthTier::Tier2);
        assert!(AuthTier::Tier2 < AuthTier::Tier3);
        assert!(AuthTier::Tier3 < AuthTier::Tier4);
    }

    #[test]
    fn auth_tier_from_u32_roundtrip() {
        for v in 1u32..=4 {
            let tier = AuthTier::from_u32(v).unwrap();
            assert_eq!(tier.as_u32(), v);
        }
        assert!(AuthTier::from_u32(0).is_none());
        assert!(AuthTier::from_u32(5).is_none());
    }
}
