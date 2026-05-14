// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

// Post-transfer integrity verification (spec 3.12.6).
//
// The destination Node performs these checks after receiving
// migration.transfer_complete:
//   1. Event count match — total Events in destination DAG equals total_events
//   2. DAG tip match — destination DAG tips match the expected_tips array exactly

use thiserror::Error;

// ── Errors ────────────────────────────────────────────────────────────────────

#[derive(Debug, PartialEq, Eq, Error)]
pub enum VerificationError {
    #[error(
        "6010 event_count_mismatch: expected {expected} events, destination has {actual}"
    )]
    EventCountMismatch { expected: usize, actual: usize },
    #[error("6011 tips_mismatch: destination DAG tips do not match source tips")]
    TipsMismatch { expected: Vec<String>, actual: Vec<String> },
}

impl VerificationError {
    pub fn error_code(&self) -> u32 {
        match self {
            Self::EventCountMismatch { .. } => 6010,
            Self::TipsMismatch { .. } => 6011,
        }
    }
}

// ── Verification ──────────────────────────────────────────────────────────────

/// Verify that the destination's received transfer matches the source's declaration.
///
/// `actual_event_count` — number of Events in the destination's DAG for this Space.
/// `actual_tips` — DAG tip event_ids on the destination Node.
/// `expected_total_events` — `total_events` field from `migration.transfer_complete`.
/// `expected_tips` — `dag_tips` field from `migration.transfer_complete`.
pub fn verify_transfer(
    actual_event_count: usize,
    actual_tips: &[String],
    expected_total_events: usize,
    expected_tips: &[String],
) -> Result<(), VerificationError> {
    if actual_event_count != expected_total_events {
        return Err(VerificationError::EventCountMismatch {
            expected: expected_total_events,
            actual: actual_event_count,
        });
    }

    let mut sorted_expected: Vec<String> = expected_tips.to_vec();
    sorted_expected.sort();
    let mut sorted_actual: Vec<String> = actual_tips.to_vec();
    sorted_actual.sort();

    if sorted_actual != sorted_expected {
        return Err(VerificationError::TipsMismatch {
            expected: sorted_expected,
            actual: sorted_actual,
        });
    }

    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn tips(ids: &[&str]) -> Vec<String> {
        ids.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn verification_passes_on_valid_transfer() {
        let result = verify_transfer(10, &tips(&["tip1", "tip2"]), 10, &tips(&["tip1", "tip2"]));
        assert!(result.is_ok());
    }

    #[test]
    fn verification_tips_order_independent() {
        // Tips should match regardless of order.
        let result = verify_transfer(5, &tips(&["b", "a"]), 5, &tips(&["a", "b"]));
        assert!(result.is_ok());
    }

    #[test]
    fn verification_fails_on_count_mismatch() {
        let err = verify_transfer(9, &tips(&["tip1"]), 10, &tips(&["tip1"])).unwrap_err();
        assert_eq!(err.error_code(), 6010);
        assert!(matches!(err, VerificationError::EventCountMismatch { expected: 10, actual: 9 }));
    }

    #[test]
    fn verification_fails_on_tips_mismatch() {
        let err =
            verify_transfer(5, &tips(&["tip1"]), 5, &tips(&["tip2"])).unwrap_err();
        assert_eq!(err.error_code(), 6011);
        assert!(matches!(err, VerificationError::TipsMismatch { .. }));
    }
}
