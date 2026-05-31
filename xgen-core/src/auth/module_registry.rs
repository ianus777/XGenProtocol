// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

// Registry of trusted Auth Modules (auth-module-registry D-071 arc, AMR-D1).
//
// An Auth Module is a tier-verification service: it verifies and attests an
// Identity's Trust Assertion tier. The protocol already has tier verification
// LOGIC (`auth/tiers.rs`) and a forward-designed `AuthModuleUntrusted` / 3006
// rejection, but it has NO registry of which Auth Modules a Node trusts. This
// store is that registry — record + store + (in later commits) the 5
// `auth-module` admin verbs.
//
// AMR-D1 STANDALONE: this whole arc ships the record + store + verbs WITHOUT
// wiring the registry into any runtime consumer. The registration-pipeline
// consultation (steps 5–7), the Trust-Assertion-accept consultation, and the
// `AuthModuleUntrusted` / 3006 raise are an EXPLICIT DEFERRAL to their own
// future arc — the consumers are themselves unbuilt (`TrustAssertion` ctor
// lands at Pass 2). Store-before-consumer.
//
// PRIME INVARIANT (AMR-D1): an empty registry leaves the Node byte-for-byte as
// today. There is no runtime consumer this arc, so the invariant is trivially
// held — asserted by the existing suite staying green throughout.
//
// SIBLING in shape to `federation/federation_policy.rs`: a standalone
// JSON-persisted store keyed by a typed principal XGID, reusing
// `RegistryError`, persisted at the D-035-convention path
// (`xgen-node_auth_modules.json`, wired at Commit 3).

use std::{collections::HashMap, path::Path};

use serde::{Deserialize, Serialize};
use xgen_common::xgid::AuthModuleXgid;

use crate::auth::tiers::AuthTier;
use crate::federation::registry::RegistryError;

/// A registered Auth Module trusted by this Node (AMR-D1).
///
/// The `module_id` (AMR-D2, the seventh XGID flavour) is the single source of
/// truth for the module's identity AND its key: the Ed25519 verifying key is
/// recovered via `module_id.pubkey()`, so there is NO separate `public_key`
/// field (AMR-D3 derive-don't-store — mirrors Node/Identity, avoids dual
/// source-of-truth drift).
///
/// Timestamps are RFC 3339 UTC strings, sibling to `pending_queue.rs`'s
/// `received_at` (xgen-core stays time-free; the admin verb that constructs a
/// record supplies the timestamp). `revoked` is block-only (A2-D1): a revoked
/// module is retained, marked untrusted, not removed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthModuleRecord {
    /// Principal-flavour XGID identifying the module by its Ed25519 key
    /// (AMR-D2 / AMR-D3). Key recoverable via `module_id.pubkey()`.
    pub module_id: AuthModuleXgid,
    /// Where the module is reached (the `auth-module test` probe target, C4).
    pub endpoint_url: String,
    /// The Auth Tiers this Node accepts attestations for from this module.
    pub accepted_tiers: Vec<AuthTier>,
    /// RFC 3339 UTC timestamp of registration (supplied by the verb).
    pub registered_at: String,
    /// Block-only revocation marker (A2-D1). A revoked module stays in the
    /// registry (still listed) but is untrusted.
    #[serde(default)]
    pub revoked: bool,
    /// RFC 3339 UTC timestamp of revocation, when `revoked`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revoked_at: Option<String>,
}

/// Persistent registry of trusted Auth Modules, keyed by `module_id` (AMR-D2).
///
/// JSON file shape: `{ "modules": { "<module_id_uri>": {...} } }`. A standalone
/// store (not a field on any relationship), sibling to
/// `FederationPolicyStore` — see the module doc-comment.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AuthModuleRegistry {
    #[serde(default)]
    modules: HashMap<AuthModuleXgid, AuthModuleRecord>,
}

impl AuthModuleRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert or replace a module record (backs `auth-module register`). Named
    /// `register` for the verb; keyed by the record's `module_id`.
    pub fn register(&mut self, record: AuthModuleRecord) {
        self.modules.insert(record.module_id.clone(), record);
    }

    /// Mark a module revoked + retain it (A2-D1 block-only). Sets `revoked =
    /// true` and `revoked_at`; the record stays in the registry so it still
    /// lists. Returns `true` if the module existed (the verb maps `false` to an
    /// unknown-module error in Commit 3). `revoked_at` is caller-supplied
    /// (xgen-core stays time-free).
    pub fn revoke(&mut self, module_id: &AuthModuleXgid, revoked_at: String) -> bool {
        match self.modules.get_mut(module_id) {
            Some(record) => {
                record.revoked = true;
                record.revoked_at = Some(revoked_at);
                true
            }
            None => false,
        }
    }

    /// Replace a module's accepted-tier set (backs `auth-module set-tiers`).
    /// Returns `true` if the module existed.
    pub fn set_tiers(&mut self, module_id: &AuthModuleXgid, tiers: Vec<AuthTier>) -> bool {
        match self.modules.get_mut(module_id) {
            Some(record) => {
                record.accepted_tiers = tiers;
                true
            }
            None => false,
        }
    }

    pub fn get(&self, module_id: &AuthModuleXgid) -> Option<&AuthModuleRecord> {
        self.modules.get(module_id)
    }

    /// All records (backs `auth-module list`). The key is redundant with
    /// `record.module_id`, so only the records are returned.
    pub fn all(&self) -> Vec<&AuthModuleRecord> {
        self.modules.values().collect()
    }

    pub fn len(&self) -> usize {
        self.modules.len()
    }

    pub fn is_empty(&self) -> bool {
        self.modules.is_empty()
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
        let registry: Self = serde_json::from_str(&json)?;
        Ok(registry)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;
    use tempfile::NamedTempFile;

    /// Deterministic module id from a seed — `from_pubkey` so the URI is always
    /// a valid principal XGID (AMR-D2/D3: a malformed id is impossible).
    fn module_id(seed: u8) -> AuthModuleXgid {
        let sk = SigningKey::from_bytes(&[seed; 32]);
        AuthModuleXgid::from_pubkey(&sk.verifying_key())
    }

    fn sample_record(seed: u8) -> AuthModuleRecord {
        AuthModuleRecord {
            module_id: module_id(seed),
            endpoint_url: "https://auth.example.com/verify".to_string(),
            accepted_tiers: vec![AuthTier::Tier2, AuthTier::Tier3],
            registered_at: "2026-05-31T12:00:00.000Z".to_string(),
            revoked: false,
            revoked_at: None,
        }
    }

    #[test]
    fn register_then_get_and_list() {
        let mut reg = AuthModuleRegistry::new();
        assert!(reg.is_empty());

        reg.register(sample_record(0x11));
        reg.register(sample_record(0x22));
        assert_eq!(reg.len(), 2);
        assert_eq!(reg.all().len(), 2);

        let got = reg.get(&module_id(0x11)).unwrap();
        assert_eq!(got.endpoint_url, "https://auth.example.com/verify");
        assert_eq!(got.accepted_tiers, vec![AuthTier::Tier2, AuthTier::Tier3]);
        assert!(!got.revoked);
    }

    #[test]
    fn register_is_insert_or_replace() {
        let mut reg = AuthModuleRegistry::new();
        reg.register(sample_record(0x11));

        let mut updated = sample_record(0x11);
        updated.endpoint_url = "https://auth.example.com/v2".to_string();
        reg.register(updated);

        assert_eq!(reg.len(), 1);
        assert_eq!(
            reg.get(&module_id(0x11)).unwrap().endpoint_url,
            "https://auth.example.com/v2"
        );
    }

    #[test]
    fn revoke_marks_untrusted_and_retains() {
        // A2-D1 block-only: revoke sets the marker + timestamp but keeps the
        // record listed (it does NOT remove it).
        let mut reg = AuthModuleRegistry::new();
        reg.register(sample_record(0x11));

        let found = reg.revoke(&module_id(0x11), "2026-05-31T13:00:00.000Z".to_string());
        assert!(found);
        assert_eq!(reg.len(), 1); // retained, not removed
        let r = reg.get(&module_id(0x11)).unwrap();
        assert!(r.revoked);
        assert_eq!(r.revoked_at.as_deref(), Some("2026-05-31T13:00:00.000Z"));

        // Revoking an unknown module reports not-found (the verb → error in C3).
        assert!(!reg.revoke(&module_id(0x99), "2026-05-31T13:00:00.000Z".to_string()));
    }

    #[test]
    fn set_tiers_replaces_and_reports_unknown() {
        let mut reg = AuthModuleRegistry::new();
        reg.register(sample_record(0x11));

        let found = reg.set_tiers(&module_id(0x11), vec![AuthTier::Tier4]);
        assert!(found);
        assert_eq!(reg.get(&module_id(0x11)).unwrap().accepted_tiers, vec![AuthTier::Tier4]);

        assert!(!reg.set_tiers(&module_id(0x99), vec![AuthTier::Tier1]));
    }

    #[test]
    fn serde_round_trip() {
        let mut reg = AuthModuleRegistry::new();
        reg.register(sample_record(0x11));
        reg.revoke(&module_id(0x11), "2026-05-31T13:00:00.000Z".to_string());

        let json = serde_json::to_string(&reg).unwrap();
        // accepted_tiers serialise snake_case; module keyed by its principal URI.
        assert!(json.contains("xgen://pubkey/ed25519:"));

        let back: AuthModuleRegistry = serde_json::from_str(&json).unwrap();
        assert_eq!(back.len(), 1);
        let r = back.get(&module_id(0x11)).unwrap();
        assert!(r.revoked);
        assert_eq!(r.revoked_at.as_deref(), Some("2026-05-31T13:00:00.000Z"));
    }

    #[test]
    fn module_id_pubkey_recoverable_no_stored_key() {
        // AMR-D3: the key is derived from module_id, not stored — round-trips.
        let sk = SigningKey::from_bytes(&[0x11; 32]);
        let rec = sample_record(0x11);
        let recovered = rec.module_id.pubkey().expect("decode");
        assert_eq!(recovered.as_bytes(), sk.verifying_key().as_bytes());
    }

    #[test]
    fn save_load_round_trip() {
        let mut reg = AuthModuleRegistry::new();
        reg.register(sample_record(0x11));
        reg.register(sample_record(0x22));

        let tmp = NamedTempFile::new().unwrap();
        reg.save(tmp.path()).unwrap();

        let loaded = AuthModuleRegistry::load(tmp.path()).unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(
            loaded.get(&module_id(0x22)).unwrap().accepted_tiers,
            vec![AuthTier::Tier2, AuthTier::Tier3]
        );
    }

    #[test]
    fn empty_registry_saves_and_loads() {
        let reg = AuthModuleRegistry::new();
        let tmp = NamedTempFile::new().unwrap();
        reg.save(tmp.path()).unwrap();
        let loaded = AuthModuleRegistry::load(tmp.path()).unwrap();
        assert!(loaded.is_empty());
    }
}
