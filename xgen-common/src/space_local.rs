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

use crate::{NodeXgid, SpaceXgid};

/// Local-only provenance metadata about a Space.
///
/// Persisted to a dedicated Node-local JSON store (`xgen-node_space_local_metadata.json`);
/// not federated, not in the event log, not in `SpaceState`. The store is
/// keyed by `space_id`; duplicate inserts for the same `space_id` are a no-op.
///
/// XGID Retrofit Pass 1 Commit 3 — `space_id` retypes from `String` to
/// `SpaceXgid`. `introducer_node_id` was retyped to `Option<NodeXgid>` at
/// XGID Adoption v1 Commit 2 (J-095). Both fields now express the protocol-
/// object kind in the type system; wire/disk shape stays byte-equal thanks
/// to serde-transparency on both flavour wrappers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpaceLocalMetadata {
    pub space_id: SpaceXgid,
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
    pub fn new_local(space_id: SpaceXgid, introduced_at: String) -> Self {
        Self {
            space_id,
            introducer_node_id: None,
            introduced_at,
        }
    }

    pub fn new_via_federation(
        space_id: SpaceXgid,
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

    fn test_space_xgid(uri: &str) -> SpaceXgid {
        SpaceXgid::from_xgid(Xgid::new(uri.to_string()))
    }

    #[test]
    fn new_local_leaves_introducer_none() {
        let m = SpaceLocalMetadata::new_local(
            test_space_xgid("xgen://hash/sha256:abc"),
            "2026-05-20T12:00:00.000Z".to_string(),
        );
        assert!(m.introducer_node_id.is_none());
    }

    #[test]
    fn new_via_federation_sets_introducer() {
        let peer = test_node_xgid("xgen://pubkey/ed25519:peerA");
        let m = SpaceLocalMetadata::new_via_federation(
            test_space_xgid("xgen://hash/sha256:abc"),
            peer.clone(),
            "2026-05-20T12:00:00.000Z".to_string(),
        );
        assert_eq!(m.introducer_node_id, Some(peer));
    }

    /// Test B from XGID Retrofit Pass 1 §sub-question 4. Extends the v1
    /// `serde_roundtrip_with_introducer` lock to cover the new `space_id:
    /// SpaceXgid` retype alongside the existing `introducer_node_id:
    /// Option<NodeXgid>` retype. Both fields must serialise as plain JSON
    /// strings (Appendix J §J.5 invariance 2), the legacy pre-XGID JSON
    /// shape must continue to deserialise (forward-compat), and the
    /// standard roundtrip-through-self must hold.
    #[test]
    fn space_local_metadata_full_xgid_roundtrip() {
        let peer = test_node_xgid("xgen://pubkey/ed25519:peerA");
        let m = SpaceLocalMetadata::new_via_federation(
            test_space_xgid("xgen://hash/sha256:abc"),
            peer,
            "2026-05-20T12:00:00.000Z".to_string(),
        );
        let json = serde_json::to_string(&m).unwrap();

        // Wire-format invariance: both retyped fields serialise as plain
        // strings. No object wrapping, no flavour tag.
        assert!(
            json.contains(r#""space_id":"xgen://hash/sha256:abc""#),
            "space_id must serialise as plain string, got: {}",
            json
        );
        assert!(
            json.contains(r#""introducer_node_id":"xgen://pubkey/ed25519:peerA""#),
            "introducer_node_id must serialise as plain string, got: {}",
            json
        );
        assert!(
            !json.contains(r#""space_id":{"#),
            "space_id must NOT serialise as object, got: {}",
            json
        );
        assert!(
            !json.contains(r#""introducer_node_id":{"#),
            "introducer_node_id must NOT serialise as object, got: {}",
            json
        );

        // Forward-compat: the pre-XGID JSON shape (both space_id and
        // introducer_node_id as plain strings) deserialises into the post-
        // XGID typed shape. This guarantees existing on-disk
        // xgen-node_space_local_metadata.json files load after the upgrade.
        let legacy_shape_json = r#"{"space_id":"xgen://hash/sha256:abc","introducer_node_id":"xgen://pubkey/ed25519:peerA","introduced_at":"2026-05-20T12:00:00.000Z"}"#;
        let from_legacy: SpaceLocalMetadata = serde_json::from_str(legacy_shape_json).unwrap();
        assert_eq!(from_legacy, m);

        // Standard roundtrip-through-self.
        let round: SpaceLocalMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(m, round);
    }

    #[test]
    fn serde_roundtrip_local_skips_introducer_field() {
        let m = SpaceLocalMetadata::new_local(
            test_space_xgid("xgen://hash/sha256:abc"),
            "2026-05-20T12:00:00.000Z".to_string(),
        );
        let json = serde_json::to_string(&m).unwrap();
        // skip_serializing_if = Option::is_none means the field is absent on the wire.
        assert!(!json.contains("introducer_node_id"));
        // space_id remains; it's required (no skip_serializing_if).
        assert!(json.contains(r#""space_id":"xgen://hash/sha256:abc""#));
        let round: SpaceLocalMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(m, round);
    }
}
