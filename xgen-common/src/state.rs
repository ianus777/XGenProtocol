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

// ── Node state ────────────────────────────────────────────────────────────────

/// Live status snapshot written to xgen-node_state.json (D-026).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NodeState {
    pub node_id: String,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectedClient {
    pub identity_id: String,
    pub display_name: String,
    /// RFC 3339.
    pub connected_at: String,
    pub events_sent: u64,
    pub events_received: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FederatedPeer {
    pub node_id: String,
    #[serde(default)]
    pub endpoint: String,
    /// "ACTIVE", "DISCONNECTED", etc.
    #[serde(default)]
    pub state: String,
    #[serde(default)]
    pub session_id: String,
    /// Protocol version string, e.g. "0.1".
    #[serde(default)]
    pub version: String,
    /// Negotiated serialisation format, e.g. "json".
    #[serde(default)]
    pub protocol: String,
    #[serde(default)]
    pub shared_spaces: Vec<String>,
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
    pub space_id: String,
    pub name: String,
    pub member_count: usize,
    pub event_count: u64,
    pub rooms: Vec<HostedRoom>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostedRoom {
    pub room_id: String,
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
