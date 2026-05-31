// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

// Local bootstrap-client store (bootstrap-client D-071 arc, BC-D1/BC-D2).
//
// A3 is CLIENT-ONLY (A3-D1): this Node registers *itself* with Bootstrap Nodes
// and manages its own advertisement. This store is the runtime-mutable side of
// that — the registrations record (which Bootstrap Nodes this Node is currently
// registered with) plus the self-info record (the endpoint / region /
// capabilities / advertised tiers this Node publishes). It is NOT the
// server-side directory/reputation machinery (`bootstrap/directory.rs`,
// `reputation.rs`), which is the orthogonal Bootstrap-Node-*server* role.
//
// BC-D1 / BC-D2 — config seeds, store is truth. The `[bootstrap]` TOML section
// holds operator seed intent only; the runtime-mutable fields live here so the
// `register` / `deregister` / `set-info` / `set-tiers` admin verbs can mutate
// them without rewriting the operator's config file. Sibling discipline to the
// three prior D-071 arcs (`federation/pending_queue.rs`,
// `federation/federation_policy.rs`, `auth/module_registry.rs`).
//
// BC-D1(b) — ONE combined store file `xgen-node_bootstrap.json` holds both the
// registrations map and the self-info record.
//
// PRIME INVARIANT: a Node with no `[bootstrap]` config and an empty
// registrations store registers with nobody and behaves byte-for-byte like
// today. No runtime consumer exists this commit (first consumer = C3), so the
// invariant is trivially held — the existing suite stays green throughout.

use std::{collections::HashMap, path::Path};

use serde::{Deserialize, Serialize};
use xgen_common::xgid::NodeXgid;

use crate::federation::registry::RegistryError;

/// A record of one Bootstrap Node this Node is registered with (BC-D2).
///
/// Keyed in the store by `bootstrap_id` (the Bootstrap Node's principal XGID,
/// whose Ed25519 key — recoverable via `bootstrap_id.pubkey()` — verifies the
/// `register_ack` / `keepalive_ack` signatures in C2). `directory_url` is
/// returned by the Bootstrap Node in its `RegisterAck`. Timestamps are RFC 3339
/// UTC strings, supplied by the verb (xgen-core stays time-free, sibling to
/// `pending_queue.rs`'s `received_at`). `expires_at` is the TTL after which the
/// directory entry lapses without a keepalive; it is `None` until a TTL is
/// assigned (the keepalive scheduler lands in C4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BootstrapRegistration {
    /// Principal-flavour XGID identifying the Bootstrap Node by its Ed25519 key.
    /// Key recoverable via `bootstrap_id.pubkey()` (verifies acks in C2).
    pub bootstrap_id: NodeXgid,
    /// The Bootstrap Node's connect URL (where `register`/`keepalive`/`deregister`
    /// frames are sent — the framed transport, BC-D3; NOT the directory HTTP URL).
    pub url: String,
    /// HTTPS directory URL returned in the Bootstrap Node's `RegisterAck`.
    pub directory_url: String,
    /// RFC 3339 UTC timestamp of registration (supplied by the verb).
    pub registered_at: String,
    /// RFC 3339 UTC TTL expiry, refreshed on `keepalive_ack`. `None` until a TTL
    /// is assigned (keepalive scheduler, C4).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
}

/// The self-advertisement this Node publishes to Bootstrap Nodes (BC-D2).
///
/// `endpoint` / `region` / `capabilities` map to the `Register` / `Keepalive`
/// wire frames and are re-advertised by `set-info` (A3-D2, C4). `auth_tiers_served`
/// (modular Tier 1–4 set) has **no field in the wire frames** — so `set-tiers`
/// writes it here for `show` to display, but re-advertise is a documented no-op
/// (Checkpoint #1(d), Option A; a wire extension to propagate tiers is a deferred
/// protocol-design arc, OUT of A3).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BootstrapSelfInfo {
    /// This Node's advertised endpoint (the `endpoint` field of `Register`).
    #[serde(default)]
    pub endpoint: String,
    /// Operator-declared region (the `region` field of `Register`).
    #[serde(default)]
    pub region: String,
    /// Advertised `xgen.*` capability tokens (the `capabilities` field of `Register`).
    #[serde(default)]
    pub capabilities: Vec<String>,
    /// Advertised Auth Tiers served (1–4). Local-display only — no wire field
    /// carries it (Checkpoint #1(d), Option A).
    #[serde(default)]
    pub auth_tiers_served: Vec<u8>,
}

/// Persistent local bootstrap-client store (BC-D1) — registrations map keyed by
/// `bootstrap_id` plus the single self-info record. ONE combined file
/// `xgen-node_bootstrap.json` (BC-D1(b)).
///
/// JSON file shape:
/// `{ "registrations": { "<bootstrap_id_uri>": {...} }, "self_info": {...} }`.
/// A standalone store (not a field on any relationship), sibling to
/// `AuthModuleRegistry` / `FederationPolicyStore` — see the module doc-comment.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct BootstrapRegistrationStore {
    #[serde(default)]
    registrations: HashMap<NodeXgid, BootstrapRegistration>,
    #[serde(default)]
    self_info: BootstrapSelfInfo,
}

impl BootstrapRegistrationStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert or replace a registration (backs `bootstrap register`). Keyed by
    /// the record's `bootstrap_id`.
    pub fn add(&mut self, registration: BootstrapRegistration) {
        self.registrations
            .insert(registration.bootstrap_id.clone(), registration);
    }

    /// Remove a registration (backs `bootstrap deregister`). Returns `true` if
    /// the registration existed (the verb maps `false` to an unknown-bootstrap-node
    /// error in C3).
    pub fn remove(&mut self, bootstrap_id: &NodeXgid) -> bool {
        self.registrations.remove(bootstrap_id).is_some()
    }

    pub fn get(&self, bootstrap_id: &NodeXgid) -> Option<&BootstrapRegistration> {
        self.registrations.get(bootstrap_id)
    }

    /// Mutable access to a registration (backs the keepalive scheduler's TTL
    /// refresh in C4). Returns `None` for an unknown bootstrap node.
    pub fn get_mut(&mut self, bootstrap_id: &NodeXgid) -> Option<&mut BootstrapRegistration> {
        self.registrations.get_mut(bootstrap_id)
    }

    /// All registrations (backs `bootstrap show`). The key is redundant with
    /// `record.bootstrap_id`, so only the records are returned.
    pub fn all(&self) -> Vec<&BootstrapRegistration> {
        self.registrations.values().collect()
    }

    pub fn len(&self) -> usize {
        self.registrations.len()
    }

    pub fn is_empty(&self) -> bool {
        self.registrations.is_empty()
    }

    /// The advertised self-info record (backs `bootstrap show` + the C4
    /// re-advertise fan-out).
    pub fn self_info(&self) -> &BootstrapSelfInfo {
        &self.self_info
    }

    /// Set the wire-advertised self-info fields (backs `bootstrap set-info`).
    /// `endpoint` / `region` / `capabilities` map to the `Register` wire frame;
    /// re-advertise is wired in C4 (A3-D2). Leaves `auth_tiers_served` untouched.
    pub fn set_info(&mut self, endpoint: String, region: String, capabilities: Vec<String>) {
        self.self_info.endpoint = endpoint;
        self.self_info.region = region;
        self.self_info.capabilities = capabilities;
    }

    /// Set the advertised Auth Tiers (backs `bootstrap set-tiers`). Local-display
    /// only — no wire field carries it, so there is no re-advertise (Checkpoint
    /// #1(d), Option A). Leaves the wire-advertised fields untouched.
    pub fn set_tiers(&mut self, auth_tiers_served: Vec<u8>) {
        self.self_info.auth_tiers_served = auth_tiers_served;
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
        let store: Self = serde_json::from_str(&json)?;
        Ok(store)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;
    use tempfile::NamedTempFile;

    /// Deterministic bootstrap id from a seed — `from_pubkey` so the URI is
    /// always a valid principal XGID.
    fn bootstrap_id(seed: u8) -> NodeXgid {
        let sk = SigningKey::from_bytes(&[seed; 32]);
        NodeXgid::from_pubkey(&sk.verifying_key())
    }

    fn sample_registration(seed: u8) -> BootstrapRegistration {
        BootstrapRegistration {
            bootstrap_id: bootstrap_id(seed),
            url: "wss://bootstrap.example.com/xgen".to_string(),
            directory_url: "https://bootstrap.example.com/xgen-directory".to_string(),
            registered_at: "2026-05-31T12:00:00.000Z".to_string(),
            expires_at: None,
        }
    }

    #[test]
    fn add_then_get_and_list() {
        let mut store = BootstrapRegistrationStore::new();
        assert!(store.is_empty());

        store.add(sample_registration(0x11));
        store.add(sample_registration(0x22));
        assert_eq!(store.len(), 2);
        assert_eq!(store.all().len(), 2);

        let got = store.get(&bootstrap_id(0x11)).unwrap();
        assert_eq!(got.url, "wss://bootstrap.example.com/xgen");
        assert_eq!(got.directory_url, "https://bootstrap.example.com/xgen-directory");
        assert!(got.expires_at.is_none());
    }

    #[test]
    fn add_is_insert_or_replace() {
        let mut store = BootstrapRegistrationStore::new();
        store.add(sample_registration(0x11));

        let mut updated = sample_registration(0x11);
        updated.url = "wss://bootstrap.example.com/xgen-v2".to_string();
        store.add(updated);

        assert_eq!(store.len(), 1);
        assert_eq!(
            store.get(&bootstrap_id(0x11)).unwrap().url,
            "wss://bootstrap.example.com/xgen-v2"
        );
    }

    #[test]
    fn remove_reports_existence() {
        let mut store = BootstrapRegistrationStore::new();
        store.add(sample_registration(0x11));

        assert!(store.remove(&bootstrap_id(0x11)));
        assert!(store.is_empty());
        // Removing an unknown bootstrap node reports not-found (the verb → error in C3).
        assert!(!store.remove(&bootstrap_id(0x99)));
    }

    #[test]
    fn get_mut_refreshes_ttl() {
        // Backs the C4 keepalive scheduler's TTL refresh.
        let mut store = BootstrapRegistrationStore::new();
        store.add(sample_registration(0x11));

        let reg = store.get_mut(&bootstrap_id(0x11)).unwrap();
        reg.expires_at = Some("2026-06-30T00:00:00.000Z".to_string());

        assert_eq!(
            store.get(&bootstrap_id(0x11)).unwrap().expires_at.as_deref(),
            Some("2026-06-30T00:00:00.000Z")
        );
        assert!(store.get_mut(&bootstrap_id(0x99)).is_none());
    }

    #[test]
    fn self_info_defaults_empty() {
        let store = BootstrapRegistrationStore::new();
        let info = store.self_info();
        assert!(info.endpoint.is_empty());
        assert!(info.region.is_empty());
        assert!(info.capabilities.is_empty());
        assert!(info.auth_tiers_served.is_empty());
    }

    #[test]
    fn set_info_edits_wire_fields_only() {
        let mut store = BootstrapRegistrationStore::new();
        store.set_tiers(vec![2, 3]);
        store.set_info(
            "wss://self.example.com/xgen".to_string(),
            "EU".to_string(),
            vec!["xgen.federation".to_string()],
        );

        let info = store.self_info();
        assert_eq!(info.endpoint, "wss://self.example.com/xgen");
        assert_eq!(info.region, "EU");
        assert_eq!(info.capabilities, vec!["xgen.federation".to_string()]);
        // set-info leaves the tiers untouched.
        assert_eq!(info.auth_tiers_served, vec![2, 3]);
    }

    #[test]
    fn set_tiers_edits_tiers_only() {
        let mut store = BootstrapRegistrationStore::new();
        store.set_info(
            "wss://self.example.com/xgen".to_string(),
            "EU".to_string(),
            vec!["xgen.federation".to_string()],
        );
        store.set_tiers(vec![1, 4]);

        let info = store.self_info();
        assert_eq!(info.auth_tiers_served, vec![1, 4]);
        // set-tiers leaves the wire-advertised fields untouched.
        assert_eq!(info.endpoint, "wss://self.example.com/xgen");
        assert_eq!(info.region, "EU");
        assert_eq!(info.capabilities, vec!["xgen.federation".to_string()]);
    }

    #[test]
    fn serde_round_trip_carries_registrations_and_self_info() {
        let mut store = BootstrapRegistrationStore::new();
        store.add(sample_registration(0x11));
        store.set_info("wss://self.example.com/xgen".to_string(), "EU".to_string(), vec![]);
        store.set_tiers(vec![2]);

        let json = serde_json::to_string(&store).unwrap();
        // bootstrap node keyed by its principal URI.
        assert!(json.contains("xgen://pubkey/ed25519:"));

        let back: BootstrapRegistrationStore = serde_json::from_str(&json).unwrap();
        assert_eq!(back.len(), 1);
        assert_eq!(back.self_info().region, "EU");
        assert_eq!(back.self_info().auth_tiers_served, vec![2]);
    }

    #[test]
    fn save_load_round_trip() {
        let mut store = BootstrapRegistrationStore::new();
        store.add(sample_registration(0x11));
        store.add(sample_registration(0x22));
        store.set_info("wss://self.example.com/xgen".to_string(), "US".to_string(), vec![]);

        let tmp = NamedTempFile::new().unwrap();
        store.save(tmp.path()).unwrap();

        let loaded = BootstrapRegistrationStore::load(tmp.path()).unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded.self_info().region, "US");
        assert_eq!(
            loaded.get(&bootstrap_id(0x22)).unwrap().directory_url,
            "https://bootstrap.example.com/xgen-directory"
        );
    }

    #[test]
    fn empty_store_saves_and_loads() {
        // Prime-invariant shape: an empty store round-trips (registers with nobody).
        let store = BootstrapRegistrationStore::new();
        let tmp = NamedTempFile::new().unwrap();
        store.save(tmp.path()).unwrap();
        let loaded = BootstrapRegistrationStore::load(tmp.path()).unwrap();
        assert!(loaded.is_empty());
        assert_eq!(loaded.self_info(), &BootstrapSelfInfo::default());
    }
}
