// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

// State resolution — public API (spec 3.9.1–3.9.7).

pub mod algorithm;
pub mod conflict;
pub mod state_key;

// Re-export the primary public surface.
pub use algorithm::resolve;
pub use conflict::{conflicts_with, find_conflicts};
pub use state_key::{state_key_for_event, StateKey};

use thiserror::Error;

/// Errors produced by the state resolution algorithm (spec 3.9.8, 4xxx range).
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ResolutionError {
    /// 4001 — conflict set is empty; resolution cannot proceed.
    #[error("4001 state_conflict_unresolvable: conflict set is empty")]
    StateConflictUnresolvable,

    /// 4002 — Event held in pending buffer exceeded the timeout window.
    /// Defined here for completeness; timeout logic is in `dag/pending.rs` (Layer 13).
    #[error("4002 predecessor_timeout: pending event exceeded timeout")]
    PredecessorTimeout,

    /// 4003 — DAG cycle detected (self-reference or transitive cycle).
    #[error("4003 dag_cycle_detected: cycle in event DAG")]
    DagCycleDetected,

    /// 4004 — Event carries a state key not valid for its EventType.
    #[error("4004 state_key_invalid: state key is invalid for this EventType")]
    StateKeyInvalid,

    /// 4005 — All five resolution layers applied without producing a winner.
    /// Should never occur because Layer 5c always resolves.
    #[error("4005 resolution_stack_exhausted: all resolution layers exhausted")]
    ResolutionStackExhausted,

    /// 4006 — Event held pending unknown-signer Identity-record arrival
    /// (Phase 6 / F-10) exceeded the timeout window. Defined for spec
    /// completeness; the timeout-sweep call site in
    /// `xgen-node::app::run_node` emits the literal numeric code rather
    /// than constructing this variant.
    #[error("4006 identity_record_timeout: pending event Identity record never arrived")]
    IdentityRecordTimeout,

    /// 4007 — Event held pending federation-relationship arrival
    /// (Phase 7.5 §6 third trigger) exceeded the configured timeout
    /// (`[sync].federation_relationship_timeout_seconds`, default 180 s).
    /// Defined for spec completeness; the timeout-sweep call site in
    /// `xgen-node::app::run_node` emits the literal numeric code per the
    /// §6.3 precedence rule (predecessor > federation-relationship > Identity).
    #[error(
        "4007 federation_relationship_timeout: pending event federation_add never arrived"
    )]
    FederationRelationshipTimeout,
}

impl ResolutionError {
    /// Numeric error code for wire transmission and display.
    pub fn error_code(&self) -> u32 {
        match self {
            Self::StateConflictUnresolvable => 4001,
            Self::PredecessorTimeout => 4002,
            Self::DagCycleDetected => 4003,
            Self::StateKeyInvalid => 4004,
            Self::ResolutionStackExhausted => 4005,
            Self::IdentityRecordTimeout => 4006,
            Self::FederationRelationshipTimeout => 4007,
        }
    }

    /// Machine-readable error string (display format: `E00{code}`).
    pub fn error_string(&self) -> &'static str {
        match self {
            Self::StateConflictUnresolvable => "state_conflict_unresolvable",
            Self::PredecessorTimeout => "predecessor_timeout",
            Self::DagCycleDetected => "dag_cycle_detected",
            Self::StateKeyInvalid => "state_key_invalid",
            Self::ResolutionStackExhausted => "resolution_stack_exhausted",
            Self::IdentityRecordTimeout => "identity_record_timeout",
            Self::FederationRelationshipTimeout => "federation_relationship_timeout",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_codes_correct() {
        assert_eq!(ResolutionError::StateConflictUnresolvable.error_code(), 4001);
        assert_eq!(ResolutionError::PredecessorTimeout.error_code(), 4002);
        assert_eq!(ResolutionError::DagCycleDetected.error_code(), 4003);
        assert_eq!(ResolutionError::StateKeyInvalid.error_code(), 4004);
        assert_eq!(ResolutionError::ResolutionStackExhausted.error_code(), 4005);
        assert_eq!(ResolutionError::IdentityRecordTimeout.error_code(), 4006);
        assert_eq!(
            ResolutionError::FederationRelationshipTimeout.error_code(),
            4007
        );
    }

    #[test]
    fn error_strings_correct() {
        assert_eq!(ResolutionError::StateConflictUnresolvable.error_string(), "state_conflict_unresolvable");
        assert_eq!(ResolutionError::PredecessorTimeout.error_string(), "predecessor_timeout");
        assert_eq!(ResolutionError::ResolutionStackExhausted.error_string(), "resolution_stack_exhausted");
        assert_eq!(
            ResolutionError::IdentityRecordTimeout.error_string(),
            "identity_record_timeout"
        );
        assert_eq!(
            ResolutionError::FederationRelationshipTimeout.error_string(),
            "federation_relationship_timeout"
        );
    }

    #[test]
    fn empty_conflict_set_returns_error() {
        let err = ResolutionError::StateConflictUnresolvable;
        assert_eq!(err.error_code(), 4001);
    }
}
