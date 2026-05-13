// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

// Federation relationship registry (spec 3.4.5).
//
// Persistent record of active federation relationships. Consulted on startup to
// re-establish connections without requiring a new handshake sequence.

use std::{collections::HashMap, path::Path};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::handshake::FederationSession;

// ── Types ─────────────────────────────────────────────────────────────────────

/// A single recorded federation relationship.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationRelationship {
    pub peer_node_id: String,
    pub shared_spaces: Vec<String>,
    pub negotiated_version: String,
    pub negotiated_serialisation: String,
    pub session_id: String,
    /// RFC 3339 timestamp of the last successful connection.
    pub last_connected: String,
}

impl FederationRelationship {
    pub fn from_session(session: &FederationSession, last_connected: String) -> Self {
        Self {
            peer_node_id: session.peer_node_id.clone(),
            shared_spaces: session.shared_spaces.clone(),
            negotiated_version: session.negotiated_version.clone(),
            negotiated_serialisation: session.negotiated_serialisation.clone(),
            session_id: session.session_id.clone(),
            last_connected,
        }
    }
}

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

// ── Registry ──────────────────────────────────────────────────────────────────

/// Persistent federation relationship registry, keyed by peer node_id.
#[derive(Debug, Default)]
pub struct FederationRegistry {
    relationships: HashMap<String, FederationRelationship>,
}

impl FederationRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert or update a relationship. The last_connected timestamp is supplied
    /// by the caller so the registry stays deterministic in tests.
    pub fn upsert(&mut self, rel: FederationRelationship) {
        self.relationships.insert(rel.peer_node_id.clone(), rel);
    }

    /// Remove a relationship (called when `federation.goodbye` is received).
    pub fn remove(&mut self, peer_node_id: &str) -> Option<FederationRelationship> {
        self.relationships.remove(peer_node_id)
    }

    pub fn get(&self, peer_node_id: &str) -> Option<&FederationRelationship> {
        self.relationships.get(peer_node_id)
    }

    pub fn all(&self) -> Vec<&FederationRelationship> {
        self.relationships.values().collect()
    }

    pub fn len(&self) -> usize {
        self.relationships.len()
    }

    pub fn is_empty(&self) -> bool {
        self.relationships.is_empty()
    }

    // ── Persistence ───────────────────────────────────────────────────────────

    pub fn save(&self, path: &Path) -> Result<(), RegistryError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let list: Vec<&FederationRelationship> = self.relationships.values().collect();
        let json = serde_json::to_string_pretty(&list)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn load(path: &Path) -> Result<Self, RegistryError> {
        let json = std::fs::read_to_string(path)?;
        let list: Vec<FederationRelationship> = serde_json::from_str(&json)?;
        let relationships = list
            .into_iter()
            .map(|r| (r.peer_node_id.clone(), r))
            .collect();
        Ok(Self { relationships })
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn sample_rel(peer_id: &str) -> FederationRelationship {
        FederationRelationship {
            peer_node_id: peer_id.to_string(),
            shared_spaces: vec!["xgen://hash/sha256:space1".to_string()],
            negotiated_version: "0.1".to_string(),
            negotiated_serialisation: "json".to_string(),
            session_id: "xgen://hash/sha256:session1".to_string(),
            last_connected: "2026-04-27T12:00:00.000Z".to_string(),
        }
    }

    #[test]
    fn upsert_and_get() {
        let mut reg = FederationRegistry::new();
        let rel = sample_rel("xgen://pubkey/ed25519:AAAA");
        reg.upsert(rel);
        assert!(reg.get("xgen://pubkey/ed25519:AAAA").is_some());
        assert_eq!(reg.len(), 1);
    }

    #[test]
    fn upsert_updates_existing() {
        let mut reg = FederationRegistry::new();
        reg.upsert(sample_rel("xgen://pubkey/ed25519:AAAA"));
        let updated = FederationRelationship {
            last_connected: "2026-05-01T00:00:00.000Z".to_string(),
            session_id: "xgen://hash/sha256:session2".to_string(),
            ..sample_rel("xgen://pubkey/ed25519:AAAA")
        };
        reg.upsert(updated);
        let stored = reg.get("xgen://pubkey/ed25519:AAAA").unwrap();
        assert_eq!(stored.session_id, "xgen://hash/sha256:session2");
        assert_eq!(reg.len(), 1);
    }

    #[test]
    fn remove_returns_and_deletes() {
        let mut reg = FederationRegistry::new();
        reg.upsert(sample_rel("xgen://pubkey/ed25519:AAAA"));
        let removed = reg.remove("xgen://pubkey/ed25519:AAAA");
        assert!(removed.is_some());
        assert!(reg.is_empty());
    }

    #[test]
    fn all_returns_all_entries() {
        let mut reg = FederationRegistry::new();
        reg.upsert(sample_rel("xgen://pubkey/ed25519:AAAA"));
        reg.upsert(sample_rel("xgen://pubkey/ed25519:BBBB"));
        assert_eq!(reg.all().len(), 2);
    }

    #[test]
    fn save_load_round_trip() {
        let mut reg = FederationRegistry::new();
        reg.upsert(sample_rel("xgen://pubkey/ed25519:AAAA"));
        reg.upsert(sample_rel("xgen://pubkey/ed25519:BBBB"));

        let tmp = NamedTempFile::new().unwrap();
        reg.save(tmp.path()).unwrap();

        let loaded = FederationRegistry::load(tmp.path()).unwrap();
        assert_eq!(loaded.len(), 2);
        assert!(loaded.get("xgen://pubkey/ed25519:AAAA").is_some());
        assert!(loaded.get("xgen://pubkey/ed25519:BBBB").is_some());
    }

    #[test]
    fn empty_registry_saves_and_loads() {
        let reg = FederationRegistry::new();
        let tmp = NamedTempFile::new().unwrap();
        reg.save(tmp.path()).unwrap();
        let loaded = FederationRegistry::load(tmp.path()).unwrap();
        assert!(loaded.is_empty());
    }
}
