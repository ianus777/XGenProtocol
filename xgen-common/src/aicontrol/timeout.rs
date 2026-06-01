// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! AC-D3a per-command timeout tiers.
//!
//! A 3-tier class rule: read/local 5 s, write/network 30 s, federation
//! interaction 180 s. Tier is derived from a verb's READ/WRITE classification
//! plus a federation-interaction flag — new verbs inherit their tier, there is
//! no per-verb table. Standing invariant: the tier default is always **≥ the
//! verb's own internal timeout**, else the local guard masks legitimate slow
//! completion as a false `TIMEOUT`.
//!
//! **Substrate-home note (D-078, runbook §10 / checkpoint #1).** The three
//! tier values mirror the shipped constants but are **re-stated** here rather
//! than imported: `xgen-common` sits beneath both consumer crates, and the
//! constants live above it —
//! `xgen_core::dag::pending::PENDING_TIMEOUT_SECS` (30) and
//! `FEDERATION_RELATIONSHIP_TIMEOUT_SECS` (180) are `pub` in `xgen-core`, while
//! `AUTH_MODULE_PROBE_TIMEOUT_SECS` (5) is private to `xgen-node`. The AC-D3a
//! `tier ≥ verb-internal` invariant is locked by a drift-check test in each
//! consuming binary (C2 client / C4 node), where those constants are reachable.

use super::codes::{ControlCode, ControlError};

/// Read/local default (seconds). Mirrors the shipped read-path guard
/// `xgen-node::admin_ops::AUTH_MODULE_PROBE_TIMEOUT_SECS` (5).
pub const READ_TIMEOUT_SECS: u64 = 5;

/// Write/network default (seconds). Mirrors
/// `xgen_core::dag::pending::PENDING_TIMEOUT_SECS` (30).
pub const WRITE_TIMEOUT_SECS: u64 = 30;

/// Federation-interaction default (seconds). Mirrors
/// `xgen_core::dag::pending::FEDERATION_RELATIONSHIP_TIMEOUT_SECS` (180).
pub const FEDERATION_TIMEOUT_SECS: u64 = 180;

/// The 3-tier verb class (AC-D3a).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeoutTier {
    /// Local read (5 s): `state`, `whoami`, `status`, `*-list`/`show`/`query`.
    Read,
    /// Home-Node round-trip / write (30 s): `send`, `create-*`, `invite`,
    /// `join`, `register`, node writes.
    Write,
    /// Cross-Node handshake (180 s): `federation initiate` / `accept`.
    Federation,
}

impl TimeoutTier {
    /// The tier default in seconds.
    pub fn default_secs(self) -> u64 {
        match self {
            TimeoutTier::Read => READ_TIMEOUT_SECS,
            TimeoutTier::Write => WRITE_TIMEOUT_SECS,
            TimeoutTier::Federation => FEDERATION_TIMEOUT_SECS,
        }
    }

    /// The tier default in milliseconds.
    pub fn default_ms(self) -> u64 {
        self.default_secs() * 1000
    }
}

/// Resolve the effective per-command timeout in milliseconds.
///
/// The driver's optional `timeout_ms` (§10, taken from `args`) is honored
/// **as-is** — no clamp-up to the tier default, the driver owns the trade-off
/// — but **floor-validated**: it must be a positive integer, else
/// [`ControlCode::BadArgument`]. Absent → the tier default.
pub fn resolve_timeout_ms(
    tier: TimeoutTier,
    override_val: Option<&serde_json::Value>,
) -> Result<u64, ControlError> {
    match override_val {
        None => Ok(tier.default_ms()),
        Some(v) => match v.as_u64() {
            Some(ms) if ms > 0 => Ok(ms),
            _ => Err(ControlError::new(
                ControlCode::BadArgument,
                "`timeout_ms` must be a positive integer (milliseconds)",
            )),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn tier_defaults_match_the_three_tiers() {
        assert_eq!(TimeoutTier::Read.default_secs(), 5);
        assert_eq!(TimeoutTier::Write.default_secs(), 30);
        assert_eq!(TimeoutTier::Federation.default_secs(), 180);
        assert_eq!(TimeoutTier::Read.default_ms(), 5_000);
        assert_eq!(TimeoutTier::Federation.default_ms(), 180_000);
    }

    #[test]
    fn no_override_uses_tier_default() {
        assert_eq!(resolve_timeout_ms(TimeoutTier::Write, None).unwrap(), 30_000);
    }

    #[test]
    fn positive_override_is_honored_as_is_no_clamp_up() {
        // A read-tier verb (5 s) with an explicit 120 s override is honored —
        // the driver owns the trade-off, no clamp to the tier default.
        let ms = resolve_timeout_ms(TimeoutTier::Read, Some(&json!(120_000))).unwrap();
        assert_eq!(ms, 120_000);
    }

    #[test]
    fn smaller_override_than_tier_is_also_honored() {
        let ms = resolve_timeout_ms(TimeoutTier::Federation, Some(&json!(250))).unwrap();
        assert_eq!(ms, 250);
    }

    #[test]
    fn zero_override_is_bad_argument() {
        let e = resolve_timeout_ms(TimeoutTier::Read, Some(&json!(0))).unwrap_err();
        assert_eq!(e.code, ControlCode::BadArgument);
    }

    #[test]
    fn negative_override_is_bad_argument() {
        let e = resolve_timeout_ms(TimeoutTier::Read, Some(&json!(-5))).unwrap_err();
        assert_eq!(e.code, ControlCode::BadArgument);
    }

    #[test]
    fn non_numeric_override_is_bad_argument() {
        let e = resolve_timeout_ms(TimeoutTier::Read, Some(&json!("soon"))).unwrap_err();
        assert_eq!(e.code, ControlCode::BadArgument);
    }

    #[test]
    fn fractional_override_is_bad_argument() {
        // §10 floor-validation: timeout_ms is integer milliseconds.
        let e = resolve_timeout_ms(TimeoutTier::Read, Some(&json!(5.5))).unwrap_err();
        assert_eq!(e.code, ControlCode::BadArgument);
    }
}
