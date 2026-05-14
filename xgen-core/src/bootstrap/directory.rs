// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

// Bootstrap Node directory — in-memory store and signed document (spec 3.14.2–3.14.4).
//
// The directory is an in-memory list of known Nodes. It is exported as a signed
// JSON document served over HTTPS (the HTTP binding is in xgen-node, not here).
//
// Document signing follows the same pattern as NodeAnnouncement:
//   canonical JSON (field-sorted) → Ed25519 signature → `signature` field appended.

use std::collections::HashMap;

use ed25519_dalek::SigningKey;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::crypto::{encoding, signing};

// ── Directory entry ───────────────────────────────────────────────────────────

/// A single Node entry in the Bootstrap directory (spec 3.14.2).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryEntry {
    pub node_id: String,
    pub endpoint: String,
    pub region: String,
    pub last_seen: String,
    pub reputation_score: f64,
}

// ── Bootstrap directory ───────────────────────────────────────────────────────

/// In-memory Bootstrap Node directory (spec 3.14.2–3.14.4).
#[derive(Debug, Default)]
pub struct BootstrapDirectory {
    /// Known Nodes, keyed by node_id.
    entries: HashMap<String, DirectoryEntry>,
}

impl BootstrapDirectory {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add or update a Node in the directory (spec 3.14.3 — registration).
    pub fn register_node(&mut self, entry: DirectoryEntry) {
        self.entries.insert(entry.node_id.clone(), entry);
    }

    /// Remove a Node from the directory.
    pub fn remove_node(&mut self, node_id: &str) {
        self.entries.remove(node_id);
    }

    /// Return true if `node_id` is in the directory.
    pub fn contains(&self, node_id: &str) -> bool {
        self.entries.contains_key(node_id)
    }

    /// Return all entries ordered by `reputation_score` descending (spec 3.14.2, 3.14.4).
    pub fn sorted_by_reputation(&self) -> Vec<&DirectoryEntry> {
        let mut entries: Vec<&DirectoryEntry> = self.entries.values().collect();
        entries.sort_by(|a, b| {
            b.reputation_score
                .partial_cmp(&a.reputation_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        entries
    }

    /// Lookup Nodes, excluding those in `exclude_ids`.
    pub fn lookup(&self, exclude_ids: &[String]) -> Vec<&DirectoryEntry> {
        self.sorted_by_reputation()
            .into_iter()
            .filter(|e| !exclude_ids.contains(&e.node_id))
            .collect()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

// ── Signed directory document ─────────────────────────────────────────────────

/// Build and sign a directory document for HTTPS serving (spec 3.14.2).
///
/// The document is a JSON object with a `signature` field appended after signing.
/// The canonical form for signing uses lexicographic key order (all fields except
/// `signature`), matching the project's canonical JSON convention.
pub fn sign_directory(
    bootstrap_key: &SigningKey,
    entries: &[&DirectoryEntry],
    generated_at: &str,
) -> Value {
    let bootstrap_node_id = format!(
        "xgen://pubkey/ed25519:{}",
        encoding::encode(bootstrap_key.verifying_key().as_bytes())
    );

    let nodes: Vec<Value> = entries
        .iter()
        .map(|e| {
            json!({
                "endpoint": e.endpoint,
                "last_seen": e.last_seen,
                "node_id": e.node_id,
                "region": e.region,
                "reputation_score": e.reputation_score,
            })
        })
        .collect();

    // Build the document body (no signature yet).
    let body = json!({
        "bootstrap_node_id": bootstrap_node_id,
        "generated_at": generated_at,
        "nodes": nodes,
        "protocol_version": "0.1",
    });

    // Sign the canonical (key-sorted) form of the body.
    let canonical = serde_json::to_string(&body).expect("JSON is always serialisable");
    let sig = signing::sign(bootstrap_key, canonical.as_bytes());

    // Append signature.
    let mut doc = body;
    doc["signature"] = json!(sig);
    doc
}

/// Verify a signed directory document.
///
/// Returns true if the `signature` field verifies against the `bootstrap_node_id`
/// public key. The `bootstrap_node_id` is the pubkey_uri of the Bootstrap Node.
pub fn verify_directory(doc: &Value) -> bool {
    let bootstrap_node_id = match doc["bootstrap_node_id"].as_str() {
        Some(id) => id,
        None => return false,
    };
    let sig_str = match doc["signature"].as_str() {
        Some(s) => s,
        None => return false,
    };

    // Extract verifying key from node_id.
    let pubkey_b64 = match bootstrap_node_id.strip_prefix("xgen://pubkey/ed25519:") {
        Some(b) => b,
        None => return false,
    };
    let pubkey_bytes = match crate::crypto::encoding::decode(pubkey_b64) {
        Ok(b) => b,
        Err(_) => return false,
    };
    let vk = match ed25519_dalek::VerifyingKey::from_bytes(
        pubkey_bytes.as_slice().try_into().unwrap_or(&[0u8; 32]),
    ) {
        Ok(k) => k,
        Err(_) => return false,
    };

    // Reconstruct the canonical body (without signature).
    let mut body = doc.clone();
    body.as_object_mut().map(|o| o.remove("signature"));
    let canonical = serde_json::to_string(&body).expect("JSON is serialisable");

    signing::verify(&vk, canonical.as_bytes(), sig_str).is_ok()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::keypair;

    fn sample_entry(node_id: &str, score: f64) -> DirectoryEntry {
        DirectoryEntry {
            node_id: node_id.to_string(),
            endpoint: format!("wss://{node_id}.example.com/xgen"),
            region: "EU".to_string(),
            last_seen: "2026-05-14T10:00:00.000Z".to_string(),
            reputation_score: score,
        }
    }

    #[test]
    fn node_register_adds_to_directory() {
        let mut dir = BootstrapDirectory::new();
        assert!(!dir.contains("xgen://pubkey/ed25519:NODE_A"));

        dir.register_node(sample_entry("xgen://pubkey/ed25519:NODE_A", 0.9));
        assert!(dir.contains("xgen://pubkey/ed25519:NODE_A"));
        assert_eq!(dir.len(), 1);
    }

    #[test]
    fn lookup_returns_nodes_ordered_by_reputation() {
        let mut dir = BootstrapDirectory::new();
        dir.register_node(sample_entry("xgen://pubkey/ed25519:NODE_A", 0.5));
        dir.register_node(sample_entry("xgen://pubkey/ed25519:NODE_B", 0.9));
        dir.register_node(sample_entry("xgen://pubkey/ed25519:NODE_C", 0.7));

        let ordered = dir.sorted_by_reputation();
        assert_eq!(ordered.len(), 3);
        // Highest reputation first.
        assert_eq!(ordered[0].node_id, "xgen://pubkey/ed25519:NODE_B");
        assert_eq!(ordered[1].node_id, "xgen://pubkey/ed25519:NODE_C");
        assert_eq!(ordered[2].node_id, "xgen://pubkey/ed25519:NODE_A");
    }

    #[test]
    fn lookup_excludes_specified_nodes() {
        let mut dir = BootstrapDirectory::new();
        dir.register_node(sample_entry("xgen://pubkey/ed25519:NODE_A", 0.9));
        dir.register_node(sample_entry("xgen://pubkey/ed25519:NODE_B", 0.7));

        let result = dir.lookup(&["xgen://pubkey/ed25519:NODE_A".to_string()]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].node_id, "xgen://pubkey/ed25519:NODE_B");
    }

    #[test]
    fn directory_signed_by_bootstrap_node() {
        let key = keypair::generate();
        let entries = vec![
            sample_entry("xgen://pubkey/ed25519:NODE_A", 0.9),
            sample_entry("xgen://pubkey/ed25519:NODE_B", 0.7),
        ];
        let refs: Vec<&DirectoryEntry> = entries.iter().collect();

        let doc = sign_directory(&key, &refs, "2026-05-14T10:00:00.000Z");

        // Document must have a signature field.
        assert!(doc["signature"].as_str().is_some());

        // Signature must verify.
        assert!(verify_directory(&doc));
    }

    #[test]
    fn tampered_directory_fails_verification() {
        let key = keypair::generate();
        let entries = vec![sample_entry("xgen://pubkey/ed25519:NODE_A", 0.9)];
        let refs: Vec<&DirectoryEntry> = entries.iter().collect();

        let mut doc = sign_directory(&key, &refs, "2026-05-14T10:00:00.000Z");

        // Tamper with the generated_at timestamp.
        doc["generated_at"] = serde_json::json!("2099-01-01T00:00:00.000Z");

        assert!(!verify_directory(&doc));
    }
}
