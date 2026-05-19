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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederatedPeer {
    pub node_id: String,
    pub endpoint: String,
    /// "ACTIVE", "DISCONNECTED", etc.
    pub state: String,
    pub session_id: String,
    /// Protocol version string, e.g. "0.1".
    pub version: String,
    /// Negotiated serialisation format, e.g. "json".
    pub protocol: String,
    pub shared_spaces: Vec<String>,
    /// RFC 3339.
    pub connected_at: String,
    /// RFC 3339.
    pub last_seen_at: String,
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
