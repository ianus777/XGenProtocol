// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

// State file types for xgen-node_state.json and xgen-client_state.json (D-026).
//
// Written by a running Node/client every 5 seconds. Read by observability CLI
// commands (status, connections, spaces, peers, whoami).  No secret material
// ever enters these files — private keys and plaintext signatures stay in memory.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{IdentityXgid, NodeXgid, RoomXgid, SpaceXgid};

// ── Node state ────────────────────────────────────────────────────────────────

/// Live status snapshot written to xgen-node_state.json (D-026).
///
/// XGID Retrofit Pass 1 Commit 3 retyped the identifier-bearing fields on
/// the Node-side observability structs (`NodeState`, `ConnectedClient`,
/// `FederatedPeer`, `HostedSpace`, `HostedRoom`) to flavour-typed XGIDs.
/// `session_id` stays `String` per D-072 "what XGID is not". Client-side
/// state types (`ClientState`, `KnownSpace`, `KnownRoom`) stay `String` per
/// Pass 1 scope discipline — they belong to Pass 4 (xgen-client retypes)
/// even though they happen to live in xgen-common.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NodeState {
    pub node_id: NodeXgid,
    pub version: String,
    pub build: String,
    /// RFC 3339 timestamp of Node startup.
    pub started_at: String,
    /// "local" or "production".
    pub mode: String,
    pub endpoint: String,
    /// RFC 3339 timestamp of last write.
    pub updated_at: String,
    pub clients: Vec<ConnectedClient>,
    pub peers: Vec<FederatedPeer>,
    pub spaces: Vec<HostedSpace>,
    /// Phase 6 / F-10 observability surface (runbook §3.6.1 Lock C2).
    /// Sum across all Spaces' `PendingBuffer`s of events currently held
    /// pending Identity-record arrival. Lets operators detect when
    /// Identity replication is the bottleneck for federation event
    /// ingestion (F-10 §13.7 use case). `#[serde(default)]` keeps
    /// pre-Phase-6 state.json files parsing.
    #[serde(default)]
    pub pending_identity_replication: usize,
    /// Phase 7.5 §8.2 observability surface — sibling to
    /// `pending_identity_replication`. Sum across all Spaces' `PendingBuffer`s
    /// of events currently held pending federation-relationship arrival
    /// (third HeldPending trigger, P7.5-B). Lets operators detect when
    /// `state.federation_add` ingestion is the bottleneck for federation
    /// bootstrap. `#[serde(default)]` keeps pre-Phase-7.5 state.json
    /// files parsing.
    #[serde(default)]
    pub pending_federation_relationship: usize,
    /// SE-D8 — the active storage backend advert (operator-visible). Reports the
    /// engine actually backing per-Space storage ("vanilla" when no engine is
    /// selected) + its durability class + the asserted tier. `#[serde(default)]`
    /// keeps pre-storage-engine state.json files parsing.
    #[serde(default)]
    pub storage: StorageAdvert,
}

/// SE-D8 capability advert — the active storage backend (a local conformance
/// property; federation/wire advert is deferred). `engine` is the descriptor
/// name ("vanilla" for the default in-memory + JSON backend), `assurance` its
/// durability class, `asserts_tier` the node's gate-resolved tier.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct StorageAdvert {
    pub engine: String,
    pub assurance: String,
    pub asserts_tier: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectedClient {
    pub identity_id: IdentityXgid,
    pub display_name: String,
    /// RFC 3339.
    pub connected_at: String,
    pub events_sent: u64,
    pub events_received: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FederatedPeer {
    pub node_id: NodeXgid,
    #[serde(default)]
    pub endpoint: String,
    /// "ACTIVE", "DISCONNECTED", etc.
    #[serde(default)]
    pub state: String,
    /// Session IDs are NOT XGIDs (D-072 "what XGID is not") — they're derived
    /// from a sort+hash of the two Node URIs plus a timestamp, not a
    /// principal-flavour pubkey or hash-anchored canonical-bytes digest.
    /// Stays `String` at Pass 1.
    #[serde(default)]
    pub session_id: String,
    /// Protocol version string, e.g. "0.1".
    #[serde(default)]
    pub version: String,
    /// Negotiated serialisation format, e.g. "json".
    #[serde(default)]
    pub protocol: String,
    #[serde(default)]
    pub shared_spaces: Vec<SpaceXgid>,
    /// RFC 3339.
    #[serde(default)]
    pub connected_at: String,
    /// RFC 3339.
    #[serde(default)]
    pub last_seen_at: String,

    // ── F-1c operational record fields (Phase 9 G1 observability) ────────────
    // Sourced from `FederationRegistry::PeerOperationalRecord` (runbook
    // §3.5.1 Lock A). These let observers — operators and Phase 9 integration
    // tests — read peer-level operational state from state.json instead of
    // parsing transient log lines. `#[serde(default)]` keeps pre-Phase-9
    // state.json files parsing.
    /// True if the last federation session for this peer ended without a
    /// successful reconnect; mirrors `PeerOperationalRecord.lost_connection`.
    #[serde(default)]
    pub lost_connection: bool,
    /// RFC 3339 — last time a handshake completed to ACTIVE state. `None`
    /// when this peer has never had a successful session in this Node's
    /// recorded history.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_successful_session: Option<String>,
    /// RFC 3339 — when the reconnect scheduler will next attempt to reach a
    /// lost peer. `None` when the peer is currently active or has no
    /// reconnect scheduled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_reconnect_attempt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostedSpace {
    pub space_id: SpaceXgid,
    pub name: String,
    pub member_count: usize,
    pub event_count: u64,
    pub rooms: Vec<HostedRoom>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostedRoom {
    pub room_id: RoomXgid,
    pub name: String,
    pub event_count: u64,
    /// RFC 3339, or empty string if no messages yet.
    pub last_activity: String,
}

// ── Client state ──────────────────────────────────────────────────────────────

/// Status snapshot written to xgen-client_state.json (D-026).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClientState {
    pub identity_id: String,
    pub display_name: String,
    pub version: String,
    pub build: String,
    pub home_node: String,
    /// RFC 3339 timestamp of last write.
    pub updated_at: String,
    pub spaces: Vec<KnownSpace>,
    /// MP-F7 — per-Space last local event id (space_id → event_id), causal-DAG
    /// bookkeeping for the leave→rejoin anchor. `ops::leave` writes it; `ops::join`
    /// reads it on the `get_dag_tips`-empty fallback so an open rejoin causally
    /// descends from the member's own leave (linear `j→lv→rj`) instead of the
    /// create root (concurrent → Layer-1 `leave>join` drops it). Best-effort:
    /// absence degrades to today's root fallback, never an error. Kept off
    /// `KnownSpace` (not membership state) → no joined-list consumer sweep.
    #[serde(default)]
    pub last_local_events: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnownSpace {
    pub space_id: String,
    pub name: String,
    pub node_endpoint: String,
    /// "owner", "admin", "moderator", "member".
    pub role: String,
    pub rooms: Vec<KnownRoom>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnownRoom {
    pub room_id: String,
    pub name: String,
    pub joined: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Xgid;

    fn node(uri: &str) -> NodeXgid {
        NodeXgid::from_xgid(Xgid::new(uri.to_string()))
    }
    fn identity(uri: &str) -> IdentityXgid {
        IdentityXgid::from_xgid(Xgid::new(uri.to_string()))
    }
    fn space(uri: &str) -> SpaceXgid {
        SpaceXgid::from_xgid(Xgid::new(uri.to_string()))
    }
    fn room(uri: &str) -> RoomXgid {
        RoomXgid::from_xgid(Xgid::new(uri.to_string()))
    }

    /// Test C from XGID Retrofit Pass 1 §sub-question 4. Construct a NodeState
    /// with typed-XGID fields populated through every Pass-1-scoped sub-struct
    /// (`ConnectedClient.identity_id`, `FederatedPeer.node_id`/`shared_spaces`,
    /// `HostedSpace.space_id`, `HostedRoom.room_id`); serialise to the JSON
    /// shape that `xgen-node_state.json` carries; assert every XGID-bearing
    /// field is a plain JSON string; deserialise back; assert byte-equal.
    /// Locks the observability-layer invariance so operators reading
    /// `xgen-node_state.json` see XGIDs in the same shape across the Pass 1
    /// transition.
    #[test]
    fn node_state_observability_xgid_roundtrip() {
        let ns = NodeState {
            node_id: node("xgen://pubkey/ed25519:nodeA"),
            version: "0.10.3".to_string(),
            build: "test".to_string(),
            started_at: "2026-05-25T10:00:00.000Z".to_string(),
            mode: "production".to_string(),
            endpoint: "127.0.0.1:8080".to_string(),
            updated_at: "2026-05-25T10:00:05.000Z".to_string(),
            clients: vec![ConnectedClient {
                identity_id: identity("xgen://pubkey/ed25519:alice"),
                display_name: "Alice".to_string(),
                connected_at: "2026-05-25T10:00:01.000Z".to_string(),
                events_sent: 5,
                events_received: 3,
            }],
            peers: vec![FederatedPeer {
                node_id: node("xgen://pubkey/ed25519:nodeB"),
                endpoint: "127.0.0.1:8081".to_string(),
                state: "ACTIVE".to_string(),
                session_id: "session-abc-123".to_string(),
                version: "0.1".to_string(),
                protocol: "json".to_string(),
                shared_spaces: vec![
                    space("xgen://hash/sha256:space1"),
                    space("xgen://hash/sha256:space2"),
                ],
                connected_at: "2026-05-25T10:00:02.000Z".to_string(),
                last_seen_at: "2026-05-25T10:00:04.000Z".to_string(),
                lost_connection: false,
                last_successful_session: Some("2026-05-25T10:00:02.000Z".to_string()),
                next_reconnect_attempt: None,
            }],
            spaces: vec![HostedSpace {
                space_id: space("xgen://hash/sha256:space1"),
                name: "Alpha".to_string(),
                member_count: 3,
                event_count: 42,
                rooms: vec![HostedRoom {
                    room_id: room("xgen://hash/sha256:room1"),
                    name: "general".to_string(),
                    event_count: 30,
                    last_activity: "2026-05-25T10:00:03.000Z".to_string(),
                }],
            }],
            pending_identity_replication: 0,
            pending_federation_relationship: 0,
            storage: StorageAdvert {
                engine: "vanilla".to_string(),
                assurance: "best_effort".to_string(),
                asserts_tier: 1,
            },
        };

        let json = serde_json::to_string(&ns).expect("serialise NodeState");

        // Every XGID-bearing field must appear as a plain JSON string —
        // value-side `"xgen://..."`, never object-wrapped.
        assert!(
            json.contains(r#""node_id":"xgen://pubkey/ed25519:nodeA""#),
            "NodeState.node_id must be plain string, got: {}",
            json
        );
        assert!(
            json.contains(r#""identity_id":"xgen://pubkey/ed25519:alice""#),
            "ConnectedClient.identity_id must be plain string, got: {}",
            json
        );
        assert!(
            json.contains(r#""node_id":"xgen://pubkey/ed25519:nodeB""#),
            "FederatedPeer.node_id must be plain string, got: {}",
            json
        );
        // session_id stays String per D-072 — not retyped at Pass 1.
        assert!(
            json.contains(r#""session_id":"session-abc-123""#),
            "FederatedPeer.session_id stays String, got: {}",
            json
        );
        assert!(
            json.contains(r#""shared_spaces":["xgen://hash/sha256:space1","xgen://hash/sha256:space2"]"#),
            "FederatedPeer.shared_spaces must be a plain string array, got: {}",
            json
        );
        assert!(
            json.contains(r#""space_id":"xgen://hash/sha256:space1""#),
            "HostedSpace.space_id must be plain string, got: {}",
            json
        );
        assert!(
            json.contains(r#""room_id":"xgen://hash/sha256:room1""#),
            "HostedRoom.room_id must be plain string, got: {}",
            json
        );

        // No XGID-bearing field appears as an object — no `:{` adjacent to
        // any of the typed-XGID field names.
        for forbidden in [
            r#""node_id":{"#,
            r#""identity_id":{"#,
            r#""space_id":{"#,
            r#""room_id":{"#,
        ] {
            assert!(
                !json.contains(forbidden),
                "Found object-wrapped XGID field {} in JSON: {}",
                forbidden,
                json
            );
        }

        // Roundtrip-through-serde — byte-equal NodeState recovered.
        let round: NodeState = serde_json::from_str(&json).expect("deserialise NodeState");
        // Field-by-field equality through the XGID fields.
        assert_eq!(round.node_id, ns.node_id);
        assert_eq!(round.clients[0].identity_id, ns.clients[0].identity_id);
        assert_eq!(round.peers[0].node_id, ns.peers[0].node_id);
        assert_eq!(round.peers[0].session_id, ns.peers[0].session_id);
        assert_eq!(round.peers[0].shared_spaces, ns.peers[0].shared_spaces);
        assert_eq!(round.spaces[0].space_id, ns.spaces[0].space_id);
        assert_eq!(round.spaces[0].rooms[0].room_id, ns.spaces[0].rooms[0].room_id);
    }
}
