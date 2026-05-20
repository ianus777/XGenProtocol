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
//     Some(peer)) → `introducer_node_id = Some(NodeXgid::from_xgid(...))`.
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
// XGID Adoption v1 Commit 2 (D-072 + D-073) — `introducer_node_id` retypes
// from `Option<String>` to `Option<NodeXgid>` as the v1 inaugural production
// use of a typed XGID flavour. The field name carries the role (the Node
// that introduced this Space to us); the type carries the contract (an
// XGID identifying a Node — see Appendix J §J.2 NodeXgid). Wire/disk
// format is unchanged thanks to `NodeXgid`'s `#[serde(transparent)]` impl
// — see `serde_roundtrip_with_introducer` below for the byte-shape lock.

use serde::{Deserialize, Serialize};

use crate::NodeXgid;

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
    /// Spaces. Carries the v1 typed-XGID contract (D-072 + D-073).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub introducer_node_id: Option<NodeXgid>,
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
        introducer_node_id: NodeXgid,
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
    use crate::Xgid;

    /// Build a representative `NodeXgid` from a literal URI string. The
    /// production caller in xgen-core::node::runtime constructs the same
    /// shape via `NodeXgid::from_xgid(Xgid::new(peer.to_string()))` at the
    /// federation-peer-ID boundary.
    fn test_node_xgid(uri: &str) -> NodeXgid {
        NodeXgid::from_xgid(Xgid::new(uri.to_string()))
    }

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
        let peer = test_node_xgid("xgen://pubkey/ed25519:peerA");
        let m = SpaceLocalMetadata::new_via_federation(
            "xgen://hash/sha256:abc".to_string(),
            peer.clone(),
            "2026-05-20T12:00:00.000Z".to_string(),
        );
        assert_eq!(m.introducer_node_id, Some(peer));
    }

    #[test]
    fn serde_roundtrip_with_introducer() {
        // XGID Adoption v1 wire-format invariance lock: serde-transparency on
        // NodeXgid means the on-disk JSON shape is byte-equal to the pre-XGID
        // shape — `"introducer_node_id":"xgen://pubkey/ed25519:peerA"` with
        // the value as a plain string, NOT an object wrapping. This test is
        // the per-call-site witness for Appendix J §J.5 invariance 2 at the
        // `SpaceLocalMetadata` use site.
        let peer = test_node_xgid("xgen://pubkey/ed25519:peerA");
        let m = SpaceLocalMetadata::new_via_federation(
            "xgen://hash/sha256:abc".to_string(),
            peer,
            "2026-05-20T12:00:00.000Z".to_string(),
        );
        let json = serde_json::to_string(&m).unwrap();

        // Wire-format invariance: the field value is a plain string, no
        // object wrapping, no flavour tag. Byte-equal to the pre-XGID shape.
        assert!(
            json.contains(r#""introducer_node_id":"xgen://pubkey/ed25519:peerA""#),
            "introducer_node_id must serialise as a plain string, got: {}",
            json
        );
        assert!(
            !json.contains(r#""introducer_node_id":{"#),
            "introducer_node_id must NOT serialise as an object, got: {}",
            json
        );

        // Forward-compat: a pre-XGID JSON shape (introducer as plain string)
        // deserialises into the post-XGID shape (introducer as NodeXgid).
        // This is what guarantees existing on-disk
        // xgen-node_space_local_metadata.json files load correctly after the
        // upgrade.
        let legacy_shape_json = r#"{"space_id":"xgen://hash/sha256:abc","introducer_node_id":"xgen://pubkey/ed25519:peerA","introduced_at":"2026-05-20T12:00:00.000Z"}"#;
        let from_legacy: SpaceLocalMetadata = serde_json::from_str(legacy_shape_json).unwrap();
        assert_eq!(from_legacy, m);

        // Standard roundtrip-through-self also holds.
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
