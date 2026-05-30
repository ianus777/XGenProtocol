// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

// Pending federation request queue (FAC-D1 / FAC-D1a, sub-arc 2a).
//
// When `federation.require_approval` is enabled, an inbound federation
// handshake from a peer that is not already an `Active` relationship is NOT
// auto-established. Instead the handshake-derived facts are recorded here as
// a `PendingFederationRequest` (and the peer is sent `Reject` 2003 — the
// pause-point lands in Commit 3), to await an operator `federation accept`
// or `federation reject`.
//
// This is a SIBLING store to `FederationRegistry`, not a field on it: a
// pending request is PRE-relationship — there is no `session_id` yet (the
// session is minted when the approved peer reconnects and runs a fresh
// handshake). Persisted JSON at the D-035-convention path, sibling to the
// federation registry file.

use std::{collections::HashMap, path::Path};

use serde::{Deserialize, Serialize};
use xgen_common::xgid::{NodeXgid, SpaceXgid};

use super::registry::RegistryError;

/// A queued inbound federation request awaiting operator approval (FAC-D1a).
///
/// Captures the handshake-negotiated facts needed to complete the
/// relationship on `federation accept`. There is deliberately no `session_id`:
/// a pending request has not completed a session — the `session_id` is minted
/// when the approved peer reconnects and runs a fresh handshake.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingFederationRequest {
    pub peer_node_id: NodeXgid,
    /// WebSocket endpoint URL of the peer (advisory; from hello node_endpoint).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub peer_url: Option<String>,
    /// RFC 3339 UTC timestamp the request was received / enqueued.
    pub received_at: String,
    pub shared_spaces: Vec<SpaceXgid>,
    pub negotiated_version: String,
    pub negotiated_serialisation: String,
}

/// Persistent queue of pending federation requests, keyed by peer node_id.
///
/// JSON file shape: `{ "requests": { "<peer_node_id>": {...} } }`. Kept a
/// sibling to `FederationRegistry` (separate file, separate type) because the
/// queue is pre-relationship — see the module doc-comment.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PendingFederationQueue {
    #[serde(default)]
    requests: HashMap<NodeXgid, PendingFederationRequest>,
}

impl PendingFederationQueue {
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert or replace a pending request (keyed by `peer_node_id`).
    /// Idempotent on the key: a peer that retries while still pending updates
    /// its existing entry rather than creating a duplicate, so the queue never
    /// double-enqueues the same peer.
    pub fn add(&mut self, request: PendingFederationRequest) {
        self.requests.insert(request.peer_node_id.clone(), request);
    }

    /// Remove a pending request (called on `accept` / `reject`). Returns it if
    /// present so the caller can use the captured handshake facts.
    pub fn remove(&mut self, peer_node_id: &NodeXgid) -> Option<PendingFederationRequest> {
        self.requests.remove(peer_node_id)
    }

    pub fn get(&self, peer_node_id: &NodeXgid) -> Option<&PendingFederationRequest> {
        self.requests.get(peer_node_id)
    }

    pub fn all(&self) -> Vec<&PendingFederationRequest> {
        self.requests.values().collect()
    }

    pub fn len(&self) -> usize {
        self.requests.len()
    }

    pub fn is_empty(&self) -> bool {
        self.requests.is_empty()
    }

    pub fn save(&self, path: &Path) -> Result<(), RegistryError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn load(path: &Path) -> Result<Self, RegistryError> {
        let json = std::fs::read_to_string(path)?;
        let queue: Self = serde_json::from_str(&json)?;
        Ok(queue)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use xgen_common::xgid::Xgid;

    fn node_key(s: &str) -> NodeXgid {
        NodeXgid::from_xgid(Xgid::new(s.to_string()))
    }

    fn sample_request(peer_id: &str) -> PendingFederationRequest {
        PendingFederationRequest {
            peer_node_id: node_key(peer_id),
            peer_url: Some("ws://127.0.0.1:8081/xgen".to_string()),
            received_at: "2026-05-30T12:00:00.000Z".to_string(),
            shared_spaces: vec![SpaceXgid::from_xgid(Xgid::new(
                "xgen://hash/sha256:space1".to_string(),
            ))],
            negotiated_version: "0.1".to_string(),
            negotiated_serialisation: "json".to_string(),
        }
    }

    #[test]
    fn add_get_remove_all() {
        let mut q = PendingFederationQueue::new();
        assert!(q.is_empty());
        q.add(sample_request("xgen://pubkey/ed25519:AAAA"));
        q.add(sample_request("xgen://pubkey/ed25519:BBBB"));
        assert_eq!(q.len(), 2);
        assert_eq!(q.all().len(), 2);

        let got = q.get(&node_key("xgen://pubkey/ed25519:AAAA")).unwrap();
        assert_eq!(got.negotiated_serialisation, "json");

        let removed = q.remove(&node_key("xgen://pubkey/ed25519:AAAA"));
        assert!(removed.is_some());
        assert!(q.get(&node_key("xgen://pubkey/ed25519:AAAA")).is_none());
        assert_eq!(q.len(), 1);
    }

    #[test]
    fn add_is_idempotent_on_peer_key() {
        // A retrying-while-pending peer must not double-enqueue (FAC-D1a).
        let mut q = PendingFederationQueue::new();
        q.add(sample_request("xgen://pubkey/ed25519:AAAA"));
        q.add(sample_request("xgen://pubkey/ed25519:AAAA"));
        assert_eq!(q.len(), 1);
    }

    #[test]
    fn save_load_round_trip() {
        let mut q = PendingFederationQueue::new();
        q.add(sample_request("xgen://pubkey/ed25519:AAAA"));
        q.add(sample_request("xgen://pubkey/ed25519:BBBB"));

        let tmp = NamedTempFile::new().unwrap();
        q.save(tmp.path()).unwrap();

        let loaded = PendingFederationQueue::load(tmp.path()).unwrap();
        assert_eq!(loaded.len(), 2);
        let r = loaded.get(&node_key("xgen://pubkey/ed25519:BBBB")).unwrap();
        assert_eq!(r.received_at, "2026-05-30T12:00:00.000Z");
        assert_eq!(r.shared_spaces.len(), 1);
        assert_eq!(r.peer_url.as_deref(), Some("ws://127.0.0.1:8081/xgen"));
    }

    #[test]
    fn empty_queue_saves_and_loads() {
        let q = PendingFederationQueue::new();
        let tmp = NamedTempFile::new().unwrap();
        q.save(tmp.path()).unwrap();
        let loaded = PendingFederationQueue::load(tmp.path()).unwrap();
        assert!(loaded.is_empty());
    }
}
