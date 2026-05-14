// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

// Node Reputation Format (spec 3.15.1–3.15.4).
//
// Bootstrap Nodes maintain a reputation record per known Node. The record is a
// set of weighted signal components that combine into a final score (0.0–1.0).
// Scores are exposed in directory documents; component breakdowns are internal.

use std::collections::HashMap;

/// Default reputation propagation interval in hours (WD-29).
pub const REPUTATION_PROPAGATION_INTERVAL_HOURS: u64 = 6;

// ── Component weights (spec 3.15.1, WD-27) ───────────────────────────────────

const WEIGHT_UPTIME_RATIO: f64 = 0.35;
const WEIGHT_ANNOUNCEMENT_FRESHNESS: f64 = 0.25;
const WEIGHT_DEFEDERATION_COUNT: f64 = 0.20;
const WEIGHT_SUCCESSFUL_FEDERATIONS: f64 = 0.10;
const WEIGHT_FAILED_FEDERATIONS: f64 = 0.10;

// ── Reputation record ─────────────────────────────────────────────────────────

/// Per-component reputation signals for a single Node (spec 3.15.1).
/// All values are normalised to [0.0, 1.0] before score computation,
/// except `defederation_count`, `successful_federations`, and `failed_federations`
/// which are raw counts that must be normalised by the caller.
///
/// Normalisation convention used here:
/// - `uptime_ratio` and `announcement_freshness`: already in [0.0, 1.0]
/// - `defederation_count`: penalises score; normalised as `1.0 / (1.0 + count)`
/// - `successful_federations`: positive; normalised as `count / (count + 1)`
/// - `failed_federations`: negative; normalised as `1.0 - (fails / (fails + success + 1))`
#[derive(Debug, Clone, PartialEq)]
pub struct ReputationComponents {
    /// Fraction of keepalive pings responded to (0.0–1.0).
    pub uptime_ratio: f64,
    /// Freshness of last announcement: 1.0 = within 24h, decays to 0.0 at 90 days.
    pub announcement_freshness: f64,
    /// Number of times other Nodes sent a defederation signal against this Node.
    pub defederation_count: u64,
    /// Count of successful federation handshakes.
    pub successful_federations: u64,
    /// Count of failed federation handshakes.
    pub failed_federations: u64,
    /// Count of confirmed protocol violations.
    pub protocol_violations: u64,
}

impl Default for ReputationComponents {
    fn default() -> Self {
        Self {
            uptime_ratio: 1.0,
            announcement_freshness: 1.0,
            defederation_count: 0,
            successful_federations: 0,
            failed_federations: 0,
            protocol_violations: 0,
        }
    }
}

/// Compute the reputation score for a set of components.
///
/// Returns a value in [0.0, 1.0]. The formula normalises each component and
/// applies the weight table from spec 3.15.1 (WD-27).
pub fn compute_score(c: &ReputationComponents) -> f64 {
    // Normalise each component to [0.0, 1.0].
    let uptime = c.uptime_ratio.clamp(0.0, 1.0);
    let freshness = c.announcement_freshness.clamp(0.0, 1.0);
    // Higher defederation_count → lower component score.
    let defed = 1.0 / (1.0 + c.defederation_count as f64);
    let success = c.successful_federations as f64
        / (c.successful_federations as f64 + 1.0);
    let fail_penalty = 1.0
        - (c.failed_federations as f64
            / (c.failed_federations as f64 + c.successful_federations as f64 + 1.0));

    let score = uptime * WEIGHT_UPTIME_RATIO
        + freshness * WEIGHT_ANNOUNCEMENT_FRESHNESS
        + defed * WEIGHT_DEFEDERATION_COUNT
        + success * WEIGHT_SUCCESSFUL_FEDERATIONS
        + fail_penalty * WEIGHT_FAILED_FEDERATIONS;

    score.clamp(0.0, 1.0)
}

/// Merge a remote reputation record into the local record using the 60/40 weighted average
/// rule from spec 3.15.2 (WD-28).
///
/// Each float component: `merged = (local × 0.6) + (remote × 0.4)`.
/// Integer counts: rounded weighted average.
pub fn merge_components(local: &ReputationComponents, remote: &ReputationComponents) -> ReputationComponents {
    ReputationComponents {
        uptime_ratio: local.uptime_ratio * 0.6 + remote.uptime_ratio * 0.4,
        announcement_freshness: local.announcement_freshness * 0.6
            + remote.announcement_freshness * 0.4,
        defederation_count: ((local.defederation_count as f64 * 0.6)
            + (remote.defederation_count as f64 * 0.4))
            .round() as u64,
        successful_federations: ((local.successful_federations as f64 * 0.6)
            + (remote.successful_federations as f64 * 0.4))
            .round() as u64,
        failed_federations: ((local.failed_federations as f64 * 0.6)
            + (remote.failed_federations as f64 * 0.4))
            .round() as u64,
        protocol_violations: ((local.protocol_violations as f64 * 0.6)
            + (remote.protocol_violations as f64 * 0.4))
            .round() as u64,
    }
}

/// Compute announcement freshness from the announcement timestamp and now.
///
/// Returns 1.0 if `age_hours` ≤ 24; decays linearly to 0.0 at 90 days (2160 hours).
pub fn announcement_freshness(age_hours: f64) -> f64 {
    if age_hours <= 24.0 {
        1.0
    } else {
        let decay_hours = 2160.0 - 24.0; // 90 days minus 24h grace period
        (1.0 - (age_hours - 24.0) / decay_hours).clamp(0.0, 1.0)
    }
}

// ── Registry ──────────────────────────────────────────────────────────────────

/// Per-Bootstrap-Node registry: tracks reputation components for each known Node.
#[derive(Debug, Default)]
pub struct ReputationRegistry {
    records: HashMap<String, ReputationComponents>,
}

impl ReputationRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Return the components for `node_id`, or defaults if not yet tracked.
    pub fn get(&self, node_id: &str) -> ReputationComponents {
        self.records.get(node_id).cloned().unwrap_or_default()
    }

    /// Upsert the components for `node_id`.
    pub fn set(&mut self, node_id: &str, components: ReputationComponents) {
        self.records.insert(node_id.to_string(), components);
    }

    /// Compute and return the current score for `node_id`.
    pub fn score(&self, node_id: &str) -> f64 {
        compute_score(&self.get(node_id))
    }

    /// Handle a defederation signal — increment `defederation_count` for `node_id`.
    /// Returns the new score after the increment.
    pub fn handle_defederation_signal(&mut self, reported_node_id: &str) -> Option<f64> {
        if !self.records.contains_key(reported_node_id) {
            return None; // Reporting a Node not in the directory — reject.
        }
        let rec = self.records.get_mut(reported_node_id).unwrap();
        rec.defederation_count += 1;
        Some(compute_score(rec))
    }

    /// Merge a remote reputation record into the local one.
    pub fn merge_remote(&mut self, node_id: &str, remote: &ReputationComponents) {
        let local = self.get(node_id);
        let merged = merge_components(&local, remote);
        self.set(node_id, merged);
    }

    pub fn is_known(&self, node_id: &str) -> bool {
        self.records.contains_key(node_id)
    }

    pub fn all_scores(&self) -> Vec<(String, f64)> {
        self.records
            .iter()
            .map(|(id, c)| (id.clone(), compute_score(c)))
            .collect()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn perfect() -> ReputationComponents {
        ReputationComponents {
            uptime_ratio: 1.0,
            announcement_freshness: 1.0,
            defederation_count: 0,
            successful_federations: 10,
            failed_federations: 0,
            protocol_violations: 0,
        }
    }

    #[test]
    fn reputation_score_computed_correctly() {
        let c = perfect();
        let score = compute_score(&c);
        // With perfect inputs the score should be high (close to 1.0).
        assert!(score > 0.9, "expected score > 0.9, got {score}");
        assert!(score <= 1.0);

        // A node with zero uptime should score much lower.
        let c_bad = ReputationComponents {
            uptime_ratio: 0.0,
            announcement_freshness: 0.0,
            defederation_count: 100,
            successful_federations: 0,
            failed_federations: 100,
            protocol_violations: 10,
        };
        let bad_score = compute_score(&c_bad);
        assert!(bad_score < 0.2, "expected bad_score < 0.2, got {bad_score}");
    }

    #[test]
    fn defederation_signal_increments_count() {
        let node_id = "xgen://pubkey/ed25519:NODE_A";
        let mut reg = ReputationRegistry::new();
        reg.set(node_id, perfect());

        let score_before = reg.score(node_id);
        let new_score = reg.handle_defederation_signal(node_id).unwrap();

        // Score must decrease after defederation signal.
        assert!(new_score < score_before, "score should decrease: {new_score} < {score_before}");
        // Count must have incremented.
        assert_eq!(reg.get(node_id).defederation_count, 1);
    }

    #[test]
    fn defederation_signal_rejects_unknown_node() {
        let mut reg = ReputationRegistry::new();
        let result = reg.handle_defederation_signal("xgen://pubkey/ed25519:UNKNOWN");
        assert!(result.is_none());
    }

    #[test]
    fn reputation_merge_applies_weights() {
        let local = ReputationComponents {
            uptime_ratio: 1.0,
            announcement_freshness: 1.0,
            defederation_count: 0,
            successful_federations: 10,
            failed_federations: 0,
            protocol_violations: 0,
        };
        let remote = ReputationComponents {
            uptime_ratio: 0.0,
            announcement_freshness: 0.0,
            defederation_count: 10,
            successful_federations: 0,
            failed_federations: 10,
            protocol_violations: 5,
        };

        let merged = merge_components(&local, &remote);

        // uptime_ratio: 1.0 * 0.6 + 0.0 * 0.4 = 0.6
        assert!((merged.uptime_ratio - 0.6).abs() < 0.001);
        // announcement_freshness: same
        assert!((merged.announcement_freshness - 0.6).abs() < 0.001);
        // defederation_count: 0 * 0.6 + 10 * 0.4 = 4 (rounded)
        assert_eq!(merged.defederation_count, 4);
    }

    #[test]
    fn stale_announcement_reduces_freshness() {
        // 0 hours → 1.0
        let f0 = announcement_freshness(0.0);
        assert!((f0 - 1.0).abs() < 0.001);

        // 24 hours → 1.0 (within grace period)
        let f24 = announcement_freshness(24.0);
        assert!((f24 - 1.0).abs() < 0.001);

        // 1080 hours (45 days) → approx 0.5
        let f45d = announcement_freshness(1080.0);
        assert!(f45d > 0.45 && f45d < 0.55, "expected ~0.5, got {f45d}");

        // 2160 hours (90 days) → 0.0
        let f90d = announcement_freshness(2160.0);
        assert!((f90d).abs() < 0.001, "expected 0.0, got {f90d}");

        // Beyond 90 days → clamped to 0.0
        let f100d = announcement_freshness(2400.0);
        assert!((f100d).abs() < 0.001);
    }
}
