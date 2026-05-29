// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

// Identity replication (spec 3.13.1–3.13.6).
//
// The home Node propagates Identity records to REPLICATION_FACTOR replica Nodes
// so clients can retrieve an Identity when the home Node is unreachable.
//
// This module provides:
//   - select_replicas  — pick up to N nodes to receive a new or updated record
//   - handle_incoming_replicate — accept/reject an inbound identity.replicate
//   - ReplicaRegistry — tracks which nodes hold replicas of each home-owned Identity

use std::collections::HashMap;

use thiserror::Error;

use super::registry::{IdentityRecord, IdentityRegistry};

/// Number of replica Nodes a new Identity is propagated to on registration (WD-19).
pub const REPLICATION_FACTOR: usize = 3;

// ── Errors ────────────────────────────────────────────────────────────────────

/// Errors from inbound replication handling.
#[derive(Debug, PartialEq, Eq, Error)]
pub enum ReplicationError {
    /// 3020 — incoming record's update_version is not higher than stored.
    #[error("3020 identity_version_stale: incoming version {incoming} <= stored version {stored}")]
    VersionStale { incoming: u64, stored: u64 },
}

impl ReplicationError {
    pub fn error_code(&self) -> u32 {
        match self {
            Self::VersionStale { .. } => 3020,
        }
    }
}

// ── ReplicaRegistry ───────────────────────────────────────────────────────────

/// Tracks which Node IDs hold replicas of each identity owned by this Node.
/// Keyed by identity_id; values are the node_ids of replica holders.
/// Not persisted — rebuilt from local state on restart (Phase 2 simplification).
#[derive(Debug, Default)]
pub struct ReplicaRegistry {
    replicas: HashMap<String, Vec<String>>,
}

impl ReplicaRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record that `node_id` now holds a replica of `identity_id`.
    pub fn add_replica(&mut self, identity_id: &str, node_id: &str) {
        self.replicas
            .entry(identity_id.to_string())
            .or_default()
            .push(node_id.to_string());
    }

    /// Remove `node_id` from the replica list for `identity_id` (e.g. stale replica).
    pub fn remove_replica(&mut self, identity_id: &str, node_id: &str) {
        if let Some(list) = self.replicas.get_mut(identity_id) {
            list.retain(|n| n != node_id);
        }
    }

    /// All node IDs currently holding a replica of `identity_id`.
    pub fn get_replicas(&self, identity_id: &str) -> &[String] {
        self.replicas.get(identity_id).map(Vec::as_slice).unwrap_or(&[])
    }

    /// True if `node_id` is already registered as a replica for `identity_id`.
    pub fn has_replica(&self, identity_id: &str, node_id: &str) -> bool {
        self.replicas
            .get(identity_id)
            .map(|list| list.iter().any(|n| n == node_id))
            .unwrap_or(false)
    }

    pub fn is_empty(&self) -> bool {
        self.replicas.is_empty()
    }
}

// ── Replica selection ─────────────────────────────────────────────────────────

/// Select up to `REPLICATION_FACTOR` nodes from `candidates` to receive a replica.
///
/// Excludes any node already in `existing_replicas`. Phase 2 selection is
/// filter-then-truncate (criteria 3 and 4 from spec 3.13.3). Geographic
/// diversity and freshness ranking are deferred to a future phase when node
/// announcement metadata is richer.
///
/// Returns a Vec of node_id Strings (cloned from candidates).
pub fn select_replicas(candidates: &[String], existing_replicas: &[String]) -> Vec<String> {
    candidates
        .iter()
        .filter(|n| !existing_replicas.contains(n))
        .take(REPLICATION_FACTOR)
        .cloned()
        .collect()
}

// ── Inbound replication handler ───────────────────────────────────────────────

/// Handle an incoming `identity.replicate` message on a replica Node.
///
/// Inserts the record if not yet known; updates if `update_version` is strictly
/// higher than the stored version. Returns Ok(()) to indicate the caller should
/// send `identity.replicate_ack`.
///
/// Returns `Err(ReplicationError::VersionStale)` if the incoming version is not
/// higher — the caller must reply with error 3020 and the stored version.
pub fn handle_incoming_replicate(
    record: IdentityRecord,
    registry: &mut IdentityRegistry,
) -> Result<(), ReplicationError> {
    let incoming_version = record.update_version;

    // Pass 2 (J-125, Surface #4 Q4.1) — pass typed reference directly.
    match registry.get(&record.identity_id) {
        Some(existing) => {
            if incoming_version <= existing.update_version {
                return Err(ReplicationError::VersionStale {
                    incoming: incoming_version,
                    stored: existing.update_version,
                });
            }
            registry.upsert(record);
        }
        None => {
            registry.upsert(record);
        }
    }

    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::registry::{DeviceRecord, IdentityRecord, IdentityRegistry};
    use xgen_common::xgid::{IdentityXgid, Xgid};

    fn id_xgid(s: &str) -> IdentityXgid {
        IdentityXgid::from_xgid(Xgid::new(s.to_string()))
    }

    fn make_record(id: &str, version: u64) -> IdentityRecord {
        IdentityRecord {
            identity_id: xgen_common::xgid::IdentityXgid::from_xgid(xgen_common::xgid::Xgid::new(
                id.to_string(),
            )),
            display_name: Some("Test".to_string()),
            is_ai: false,
            ai_capabilities: None,
            registered_at: "2026-01-01T00:00:00.000Z".to_string(),
            trust_assertion: None,
            devices: vec![DeviceRecord {
                device_id: "dev1".to_string(),
                device_name: None,
                authorised_at: "2026-01-01T00:00:00.000Z".to_string(),
            }],
            home_node: xgen_common::xgid::NodeXgid::from_xgid(xgen_common::xgid::Xgid::new(
                "xgen://pubkey/ed25519:HOME".to_string(),
            )),
            update_version: version,
            revoked: false,
            revoked_at: None,
            revocation_reason: None,
        }
    }

    fn node_ids(n: usize) -> Vec<String> {
        (0..n).map(|i| format!("xgen://pubkey/ed25519:NODE{i}")).collect()
    }

    // ── select_replicas ───────────────────────────────────────────────────────

    #[test]
    fn replicate_pushed_after_registration() {
        // Given 5 candidate nodes and no existing replicas, select_replicas
        // returns up to REPLICATION_FACTOR nodes.
        let candidates = node_ids(5);
        let selected = select_replicas(&candidates, &[]);
        assert_eq!(selected.len(), REPLICATION_FACTOR);
        // All selected must be from candidates.
        for n in &selected {
            assert!(candidates.contains(n));
        }
    }

    #[test]
    fn replication_respects_factor() {
        // Even with 10 candidates, no more than REPLICATION_FACTOR are returned.
        let candidates = node_ids(10);
        let selected = select_replicas(&candidates, &[]);
        assert!(selected.len() <= REPLICATION_FACTOR);
    }

    #[test]
    fn existing_replicas_excluded_from_selection() {
        let candidates = node_ids(5);
        let existing = candidates[..2].to_vec();
        let selected = select_replicas(&candidates, &existing);
        // No selected node should already be in existing.
        for n in &selected {
            assert!(!existing.contains(n));
        }
        assert!(selected.len() <= REPLICATION_FACTOR);
    }

    #[test]
    fn fewer_candidates_than_factor_selects_all_available() {
        let candidates = node_ids(2); // fewer than REPLICATION_FACTOR
        let selected = select_replicas(&candidates, &[]);
        assert_eq!(selected.len(), 2);
    }

    // ── handle_incoming_replicate ─────────────────────────────────────────────

    #[test]
    fn higher_update_version_accepted() {
        let mut registry = IdentityRegistry::new();
        let id = "xgen://pubkey/ed25519:ALICE";
        registry.upsert(make_record(id, 1));

        let result = handle_incoming_replicate(make_record(id, 2), &mut registry);
        assert!(result.is_ok());
        assert_eq!(registry.get(&id_xgid(id)).unwrap().update_version, 2);
    }

    #[test]
    fn lower_update_version_rejected() {
        let mut registry = IdentityRegistry::new();
        let id = "xgen://pubkey/ed25519:ALICE";
        registry.upsert(make_record(id, 5));

        let err = handle_incoming_replicate(make_record(id, 3), &mut registry).unwrap_err();
        assert_eq!(err.error_code(), 3020);
        // Stored record must not be downgraded.
        assert_eq!(registry.get(&id_xgid(id)).unwrap().update_version, 5);
    }

    #[test]
    fn replicate_ack_returned_on_success() {
        // New identity (not previously known) — upsert succeeds, Ok(()) returned.
        let mut registry = IdentityRegistry::new();
        let id = "xgen://pubkey/ed25519:BOB";
        let result = handle_incoming_replicate(make_record(id, 1), &mut registry);
        assert!(result.is_ok());
        assert!(registry.get(&id_xgid(id)).is_some());
    }

    // ── ReplicaRegistry ───────────────────────────────────────────────────────

    #[test]
    fn replica_fallback_on_identity_get() {
        let mut rr = ReplicaRegistry::new();
        let id = "xgen://pubkey/ed25519:CHARLIE";
        rr.add_replica(id, "xgen://pubkey/ed25519:NODE0");
        rr.add_replica(id, "xgen://pubkey/ed25519:NODE1");

        let replicas = rr.get_replicas(id);
        // Caller uses this list to query replicas when identity is not local.
        assert_eq!(replicas.len(), 2);
        assert!(replicas.contains(&"xgen://pubkey/ed25519:NODE0".to_string()));
    }

    #[test]
    fn re_replication_on_identity_update() {
        let mut rr = ReplicaRegistry::new();
        let id = "xgen://pubkey/ed25519:ALICE";
        rr.add_replica(id, "xgen://pubkey/ed25519:NODE0");
        rr.add_replica(id, "xgen://pubkey/ed25519:NODE1");
        rr.add_replica(id, "xgen://pubkey/ed25519:NODE2");

        // On identity update, caller gets all replicas to re-push to.
        let targets = rr.get_replicas(id);
        assert_eq!(targets.len(), REPLICATION_FACTOR);
    }
}
