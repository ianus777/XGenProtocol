// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

// Local-only per-Space metadata (Phase 7.5 §5.3 + §5.6).
//
// Sibling to `SpaceState`, NOT a field on it. SpaceState's invariant — every
// piece of state is derived from federated events — must remain intact, so
// receiver-local provenance information (such as "which peer introduced this
// Space to us") lives outside SpaceState and is never federated.
//
// Populated ONCE at `state.space_create` / `state.dm_space_create` ingestion:
//   * federation ingestion (origin == ReceivedViaFederation, peer_node_id =
//     Some(peer)) → `introducer_node_id = Some(peer.to_string())`.
//   * local creation (origin == LocallySubmitted, peer_node_id = None) →
//     `introducer_node_id = None`.
//
// After first write, the entry is never modified — idempotent at the
// ingestion layer (a second `state.space_create` for the same `space_id` is
// a no-op); the JSON store treats `space_id` as the unique key with the same
// belt-and-braces effect.
//
// NOT exposed in `xgen-node_state.json` per Phase 7.5 §8.2 — the state file is
// reserved for high-level health counters; per-Space provenance is queryable
// via the JSON store on disk until M6 (new) admin work provides an operator
// CLI verb.
//
// Field-name lock: `introducer_node_id` is retained through any future
// XGID-typing pass (the field name carries the role; the type carries the
// contract).

use serde::{Deserialize, Serialize};

/// Local-only provenance metadata about a Space.
///
/// Persisted to a dedicated Node-local JSON store (`xgen-node_space_local_metadata.json`);
/// not federated, not in the event log, not in `SpaceState`. The store is
/// keyed by `space_id`; duplicate inserts for the same `space_id` are a no-op.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpaceLocalMetadata {
    pub space_id: String,
    /// Peer Node ID that delivered the Space-create event to us, when the
    /// Space arrived over a federation session. `None` for locally created
    /// Spaces. The field name is locked through any future XGID-typing pass.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub introducer_node_id: Option<String>,
    /// RFC 3339 UTC timestamp captured when this Node first ingested the
    /// Space-creation event. Never modified after first write.
    pub introduced_at: String,
}

impl SpaceLocalMetadata {
    pub fn new_local(space_id: String, introduced_at: String) -> Self {
        Self {
            space_id,
            introducer_node_id: None,
            introduced_at,
        }
    }

    pub fn new_via_federation(
        space_id: String,
        introducer_node_id: String,
        introduced_at: String,
    ) -> Self {
        Self {
            space_id,
            introducer_node_id: Some(introducer_node_id),
            introduced_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_local_leaves_introducer_none() {
        let m = SpaceLocalMetadata::new_local(
            "xgen://hash/sha256:abc".to_string(),
            "2026-05-20T12:00:00.000Z".to_string(),
        );
        assert!(m.introducer_node_id.is_none());
    }

    #[test]
    fn new_via_federation_sets_introducer() {
        let m = SpaceLocalMetadata::new_via_federation(
            "xgen://hash/sha256:abc".to_string(),
            "xgen://pubkey/ed25519:peerA".to_string(),
            "2026-05-20T12:00:00.000Z".to_string(),
        );
        assert_eq!(
            m.introducer_node_id.as_deref(),
            Some("xgen://pubkey/ed25519:peerA")
        );
    }

    #[test]
    fn serde_roundtrip_with_introducer() {
        let m = SpaceLocalMetadata::new_via_federation(
            "xgen://hash/sha256:abc".to_string(),
            "xgen://pubkey/ed25519:peerA".to_string(),
            "2026-05-20T12:00:00.000Z".to_string(),
        );
        let json = serde_json::to_string(&m).unwrap();
        let round: SpaceLocalMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(m, round);
    }

    #[test]
    fn serde_roundtrip_local_skips_introducer_field() {
        let m = SpaceLocalMetadata::new_local(
            "xgen://hash/sha256:abc".to_string(),
            "2026-05-20T12:00:00.000Z".to_string(),
        );
        let json = serde_json::to_string(&m).unwrap();
        // skip_serializing_if = Option::is_none means the field is absent on the wire.
        assert!(!json.contains("introducer_node_id"));
        let round: SpaceLocalMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(m, round);
    }
}
