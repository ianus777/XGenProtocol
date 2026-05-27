// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

// Identity record store (spec 3.6.6–3.6.7).
//
// Persistent registry of Identity records registered on this Node.
// Keyed by identity_id (pubkey_uri). Phase 1: JSON file on disk.

use std::{collections::HashMap, path::Path};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use xgen_common::{
    wire::AiCapabilities,
    xgid::{IdentityXgid, NodeXgid},
};

// ── Record types ──────────────────────────────────────────────────────────────

/// A device associated with an Identity (spec 3.6.6).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceRecord {
    pub device_id: String,
    pub device_name: Option<String>,
    pub authorised_at: String,
}

/// Full Identity record stored on the Node (spec 3.6.6).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityRecord {
    pub identity_id: IdentityXgid,
    pub display_name: Option<String>,
    /// AI declaration (spec 3.6.10). Immutable after registration.
    /// Default `false` (human); skipped from serialised output when `false` so
    /// canonical forms of human-only records remain identical to pre-3.6.10 ones.
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_ai: bool,
    /// Capability flag set for AI Identities (spec 3.6.10.3). Required when
    /// `is_ai = true`; MUST be `None` when `is_ai = false`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ai_capabilities: Option<AiCapabilities>,
    pub registered_at: String,
    pub trust_assertion: Option<serde_json::Value>,
    pub devices: Vec<DeviceRecord>,
    pub home_node: NodeXgid,
    /// Monotonic counter for update propagation (spec 3.6.8).
    pub update_version: u64,
}

#[inline]
fn is_false(b: &bool) -> bool {
    !*b
}

// ── Errors ────────────────────────────────────────────────────────────────────

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RegistryError {
    #[error("identity already registered")]
    AlreadyRegistered,
    #[error("identity not found")]
    NotFound,
    #[error("update version not higher than stored version")]
    StaleUpdate,
    #[error("I/O error: {0}")]
    Io(String),
    #[error("JSON error: {0}")]
    Json(String),
}

// ── Registry ──────────────────────────────────────────────────────────────────

/// Persistent registry of Identity records on this Node.
#[derive(Debug, Default)]
pub struct IdentityRegistry {
    records: HashMap<IdentityXgid, IdentityRecord>,
}

impl IdentityRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new Identity. Fails if the identity_id already exists.
    pub fn register(&mut self, record: IdentityRecord) -> Result<(), RegistryError> {
        if self.records.contains_key(&record.identity_id) {
            return Err(RegistryError::AlreadyRegistered);
        }
        self.records.insert(record.identity_id.clone(), record);
        Ok(())
    }

    pub fn get(&self, identity_id: &IdentityXgid) -> Option<&IdentityRecord> {
        // Pass 2 (J-125, design §2.4 Q4.1) — signature retypes to &IdentityXgid;
        // internal HashMap key already typed at Pass 1 (Q4.7).
        self.records.get(identity_id)
    }

    pub fn contains(&self, identity_id: &IdentityXgid) -> bool {
        // Pass 2 (J-125, design §2.4 Q4.2) — signature retypes to &IdentityXgid.
        self.records.contains_key(identity_id)
    }

    /// Apply a display name update. `update_version` must be strictly higher than stored.
    ///
    /// **Pass 2 (J-125) deliberately left `&str`** — not enumerated in design doc §2.4
    /// Q4.1–Q4.7. Q4.5 covers "FederationRegistry parallel methods"; `apply_update`
    /// is a sibling method on `IdentityRegistry` itself rather than a parallel
    /// method on a different registry, so the rule does not auto-apply. Per checkpoint
    /// #2 Lock (J-125, Joe-lock Option (a)): the asymmetry between
    /// `get`+`contains` (typed) and `apply_update` (`&str`) is real but is a
    /// design-scope question, not an implementation-scope question — Pass-2 scope
    /// discipline + D-065 honest framing favour the narrow design-doc reading.
    /// Promote in own audit-design-impl arc per D-071 if a future surface (e.g.
    /// xgen-node call-site retype at Pass 3, or a stronger no-drift discipline)
    /// makes the asymmetry concrete drift.
    pub fn apply_update(
        &mut self,
        identity_id: &str,
        display_name: Option<String>,
        update_version: u64,
    ) -> Result<(), RegistryError> {
        // Pass 2 (J-125, design §2.4 Q4.6) — bridge wrap dropped inside the
        // body via Borrow<str> shortcut on the typed-keyed HashMap (Pass 1's
        // additive Borrow<str> impl on IdentityXgid makes the &str lookup work
        // without per-call wrapper allocation).
        let record = self
            .records
            .get_mut(identity_id)
            .ok_or(RegistryError::NotFound)?;
        if update_version <= record.update_version {
            return Err(RegistryError::StaleUpdate);
        }
        record.display_name = display_name;
        record.update_version = update_version;
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    pub fn all(&self) -> Vec<&IdentityRecord> {
        self.records.values().collect()
    }

    /// Insert or overwrite an Identity record unconditionally.
    /// Used by replication — a replica Node stores identities it does not own.
    /// For home-Node registration use `register()` instead.
    pub fn upsert(&mut self, record: IdentityRecord) {
        self.records.insert(record.identity_id.clone(), record);
    }

    // ── Persistence ───────────────────────────────────────────────────────────

    pub fn save(&self, path: &Path) -> Result<(), RegistryError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| RegistryError::Io(e.to_string()))?;
        }
        let list: Vec<&IdentityRecord> = self.records.values().collect();
        let json = serde_json::to_string_pretty(&list)
            .map_err(|e| RegistryError::Json(e.to_string()))?;
        std::fs::write(path, json).map_err(|e| RegistryError::Io(e.to_string()))
    }

    pub fn load(path: &Path) -> Result<Self, RegistryError> {
        let json = std::fs::read_to_string(path)
            .map_err(|e| RegistryError::Io(e.to_string()))?;
        let list: Vec<IdentityRecord> = serde_json::from_str(&json)
            .map_err(|e| RegistryError::Json(e.to_string()))?;
        let records = list.into_iter().map(|r| (r.identity_id.clone(), r)).collect();
        Ok(Self { records })
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use xgen_common::xgid::Xgid;

    /// Tests use this helper to construct typed `IdentityXgid` keys for the
    /// post-Pass-2 `get`/`contains` signatures (Q4.1/Q4.2).
    fn ix(id: &str) -> IdentityXgid {
        IdentityXgid::from_xgid(Xgid::new(id.to_string()))
    }

    fn sample_record(id: &str) -> IdentityRecord {
        IdentityRecord {
            identity_id: IdentityXgid::from_xgid(Xgid::new(id.to_string())),
            display_name: Some("Test User".to_string()),
            is_ai: false,
            ai_capabilities: None,
            registered_at: "2026-04-27T12:00:00.000Z".to_string(),
            trust_assertion: None,
            devices: vec![DeviceRecord {
                device_id: id.to_string(),
                device_name: Some("Laptop".to_string()),
                authorised_at: "2026-04-27T12:00:00.000Z".to_string(),
            }],
            home_node: NodeXgid::from_xgid(Xgid::new(
                "xgen://pubkey/ed25519:NODE".to_string(),
            )),
            update_version: 0,
        }
    }

    #[test]
    fn register_and_get() {
        let mut reg = IdentityRegistry::new();
        reg.register(sample_record("xgen://pubkey/ed25519:AAAA")).unwrap();
        assert!(reg.get(&ix("xgen://pubkey/ed25519:AAAA")).is_some());
        assert_eq!(reg.len(), 1);
    }

    #[test]
    fn duplicate_registration_rejected() {
        let mut reg = IdentityRegistry::new();
        reg.register(sample_record("xgen://pubkey/ed25519:AAAA")).unwrap();
        let err = reg.register(sample_record("xgen://pubkey/ed25519:AAAA")).unwrap_err();
        assert_eq!(err, RegistryError::AlreadyRegistered);
    }

    #[test]
    fn contains_returns_false_for_unknown() {
        let reg = IdentityRegistry::new();
        assert!(!reg.contains(&ix("xgen://pubkey/ed25519:AAAA")));
    }

    #[test]
    fn apply_update_higher_version_succeeds() {
        let mut reg = IdentityRegistry::new();
        reg.register(sample_record("xgen://pubkey/ed25519:AAAA")).unwrap();
        reg.apply_update(
            "xgen://pubkey/ed25519:AAAA",
            Some("New Name".to_string()),
            1,
        )
        .unwrap();
        let rec = reg.get(&ix("xgen://pubkey/ed25519:AAAA")).unwrap();
        assert_eq!(rec.display_name.as_deref(), Some("New Name"));
        assert_eq!(rec.update_version, 1);
    }

    #[test]
    fn apply_update_same_version_rejected() {
        let mut reg = IdentityRegistry::new();
        reg.register(sample_record("xgen://pubkey/ed25519:AAAA")).unwrap();
        reg.apply_update("xgen://pubkey/ed25519:AAAA", Some("Name".to_string()), 1)
            .unwrap();
        let err = reg
            .apply_update("xgen://pubkey/ed25519:AAAA", Some("Replay".to_string()), 1)
            .unwrap_err();
        assert_eq!(err, RegistryError::StaleUpdate);
    }

    #[test]
    fn apply_update_to_unknown_identity_fails() {
        let mut reg = IdentityRegistry::new();
        let err = reg
            .apply_update("xgen://pubkey/ed25519:XXXX", None, 1)
            .unwrap_err();
        assert_eq!(err, RegistryError::NotFound);
    }

    #[test]
    fn save_load_round_trip() {
        let mut reg = IdentityRegistry::new();
        reg.register(sample_record("xgen://pubkey/ed25519:AAAA")).unwrap();
        reg.register(sample_record("xgen://pubkey/ed25519:BBBB")).unwrap();

        let tmp = NamedTempFile::new().unwrap();
        reg.save(tmp.path()).unwrap();

        let loaded = IdentityRegistry::load(tmp.path()).unwrap();
        assert_eq!(loaded.len(), 2);
        assert!(loaded.get(&ix("xgen://pubkey/ed25519:AAAA")).is_some());
        assert!(loaded.get(&ix("xgen://pubkey/ed25519:BBBB")).is_some());
    }

    // Per-surface unit test for Surface #4 Q4.1 + Q4.2 retypes — the typed-API
    // surface compiles and behaves correctly with &IdentityXgid arguments
    // (post-Pass-2 boundary). Sibling-shape to runbook §4.7 framing.
    #[test]
    fn get_and_contains_accept_typed_identity_xgid() {
        let mut reg = IdentityRegistry::new();
        let id = ix("xgen://pubkey/ed25519:CCCC");
        reg.register(sample_record("xgen://pubkey/ed25519:CCCC"))
            .unwrap();
        assert!(reg.contains(&id));
        assert!(reg.get(&id).is_some());
        let missing = ix("xgen://pubkey/ed25519:DDDD");
        assert!(!reg.contains(&missing));
        assert!(reg.get(&missing).is_none());
    }

    #[test]
    fn empty_registry_saves_and_loads() {
        let reg = IdentityRegistry::new();
        let tmp = NamedTempFile::new().unwrap();
        reg.save(tmp.path()).unwrap();
        let loaded = IdentityRegistry::load(tmp.path()).unwrap();
        assert!(loaded.is_empty());
    }
}
