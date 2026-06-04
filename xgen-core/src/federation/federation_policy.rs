// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

// Per-peer federation policy store (FAC-D3 / FAC-D4, sub-arc 2b).
//
// A `FederationPolicy` is an operator-set, granular control sitting BETWEEN
// "auto-establish + propagate everything" (no policy) and "stop everything"
// (`defederate` / `reject`). It can DENY a peer outright (relationship
// survives, but its events neither apply inbound nor push outbound) or
// RESTRICT which shared Spaces federate with it (`allowed_spaces`).
//
// This is a SIBLING store to `FederationRegistry`, not a field on it (FAC-D4
// lock, mirroring 2a's `pending_queue.rs`): the relationship is
// handshake-derived, whereas a policy is operator-lifecycle state that can
// exist BEFORE any handshake — an operator may pre-deny a peer that has never
// federated. Persisted JSON at the D-035-convention path, sibling to the
// federation registry + queue files.
//
// Enforcement (Commit 2) reads policies through the pure `policy_permits`
// helper at two sites — outbound `apply_federation_push` and the inbound
// federated-event ingest gate (FAC-D3, both directions). The PRIME INVARIANT
// (2b): a peer with NO stored policy permits everything, byte-for-byte as
// today (sibling to 2a's `require_approval = false`). That default-permit
// lives in the helper, not the store — see Commit 2.

use std::{collections::HashMap, path::Path};

use serde::{Deserialize, Serialize};
use xgen_common::xgid::{NodeXgid, SpaceXgid};

use super::registry::RegistryError;

/// Federation policy mode for a peer (FAC-D4).
///
/// `Allow` (the default) lets the peer federate, optionally narrowed by
/// `FederationPolicy::allowed_spaces`. `Deny` blocks the peer entirely without
/// tearing down the relationship — distinct from `defederate`/`reject`, which
/// remove or tombstone it. A `Deny` bites inbound (the peer's events are
/// dropped pre-apply) and short-circuits outbound (no push to the peer).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PolicyMode {
    #[default]
    Allow,
    Deny,
}

/// An operator-set federation policy for a single peer (FAC-D4).
///
/// Minimal v1 shape: `mode` + optional `allowed_spaces`. `rate_limit` is
/// DEFERRED (it couples to the still-unbuilt federation-under-load
/// measurement). `allowed_spaces` is RESTRICTIVE-ONLY: a policy narrows, never
/// widens — the effective federated set is `shared_spaces ∩ allowed_spaces`,
/// and the protocol-derived `shared_spaces` stays authoritative. `None` means
/// "all shared Spaces" (no restriction); `Some(set)` restricts to that set.
///
/// `Default` → `{ Allow, None }` = permit-all, the prime invariant expressed
/// as a value.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FederationPolicy {
    #[serde(default)]
    pub mode: PolicyMode,
    /// Restrictive-only allow-list of shared Spaces. `None` → all shared
    /// Spaces; `Some(set)` → only Spaces in the set (intersected with the
    /// protocol-derived `shared_spaces`).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub allowed_spaces: Option<Vec<SpaceXgid>>,
    /// Restrictive-only allow-list of Space jurisdictions (Arc G PG-04, AG-D5).
    /// `None` → no jurisdiction restriction (permit-all; prime invariant);
    /// `Some(set)` → federate only Spaces whose declared `jurisdiction` ∈ set.
    /// A Space that declares **no** jurisdiction is denied under a restrictive
    /// set (strict undeclared-denied, locked — mirrors `allowed_spaces`). Open
    /// strings, matched verbatim. Additive serde (mirrors `allowed_spaces`):
    /// persists + round-trips through the existing store; `Default` stays
    /// permit-all. AND-composed with `policy_permits` via `jurisdiction_permits`
    /// at both enforcement sites.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub allowed_jurisdictions: Option<Vec<String>>,
}

/// Persistent per-peer federation policy store, keyed by peer node_id.
///
/// JSON file shape: `{ "policies": { "<peer_node_id>": {...} } }`. Kept a
/// sibling to `FederationRegistry` (separate file, separate type) because a
/// policy is operator-lifecycle state independent of the relationship — see
/// the module doc-comment.
///
/// Default-absent semantics (no policy → permit) live in the enforcement
/// helper `policy_permits` (Commit 2), NOT in the store: the store reports
/// presence/absence faithfully and never invents a default record.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct FederationPolicyStore {
    #[serde(default)]
    policies: HashMap<NodeXgid, FederationPolicy>,
}

impl FederationPolicyStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert or replace the policy for a peer (called by `federation
    /// set-policy`). Named `set` (not the queue's `add`) for the verb it backs;
    /// a peer with no relationship yet may still have a policy set (the
    /// pre-deny design intent — operator lifecycle precedes protocol lifecycle).
    pub fn set(&mut self, peer_node_id: NodeXgid, policy: FederationPolicy) {
        self.policies.insert(peer_node_id, policy);
    }

    /// Remove a peer's policy (reverts the peer to default-permit). Returns it
    /// if present.
    pub fn remove(&mut self, peer_node_id: &NodeXgid) -> Option<FederationPolicy> {
        self.policies.remove(peer_node_id)
    }

    pub fn get(&self, peer_node_id: &NodeXgid) -> Option<&FederationPolicy> {
        self.policies.get(peer_node_id)
    }

    pub fn all(&self) -> Vec<(&NodeXgid, &FederationPolicy)> {
        self.policies.iter().collect()
    }

    pub fn len(&self) -> usize {
        self.policies.len()
    }

    pub fn is_empty(&self) -> bool {
        self.policies.is_empty()
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

/// FAC-D3 enforcement decision — does the operator's policy permit federating
/// `space_id` with this peer? Pure (no I/O, no drift — D-067); called at BOTH
/// enforcement sites (outbound `apply_federation_push`, inbound ingest in
/// `process_inbound`).
///
/// - `None` (no stored policy) → `true` — the PRIME INVARIANT: a peer with no
///   policy permits everything, byte-for-byte as today (sibling to 2a's
///   `require_approval = false`).
/// - `Some { Deny }` → `false` — block the peer entirely.
/// - `Some { Allow, allowed_spaces: None }` → `true` — permit all shared Spaces.
/// - `Some { Allow, allowed_spaces: Some(set) }` → `true` iff `space_id ∈ set`
///   (restrictive-only — the policy narrows the protocol-derived shared set,
///   never widens it).
pub fn policy_permits(policy: Option<&FederationPolicy>, space_id: &SpaceXgid) -> bool {
    match policy {
        None => true,
        Some(p) => match p.mode {
            PolicyMode::Deny => false,
            PolicyMode::Allow => match &p.allowed_spaces {
                None => true,
                Some(spaces) => spaces.iter().any(|s| s == space_id),
            },
        },
    }
}

/// Arc G PG-04 jurisdictional-containment decision (AG-D5) — does the operator's
/// `allowed_jurisdictions` permit federating a Space declaring
/// `space_jurisdiction` with this peer? Pure (no I/O, no drift — D-067);
/// **AND-composed** with [`policy_permits`] at BOTH enforcement sites (outbound
/// `apply_federation_push`, inbound ingest in `process_inbound`). A single-
/// responsibility sibling to `policy_permits` — `policy_permits` owns
/// mode/`allowed_spaces`; this owns the jurisdiction dimension.
///
/// - `None` policy, or `allowed_jurisdictions: None` → `true` — no jurisdiction
///   restriction (the PRIME INVARIANT; a peer with no policy permits everything
///   byte-for-byte, and a policy that sets no jurisdiction allow-list is a
///   no-op on this dimension).
/// - `Some(set)` → `true` iff the Space **declares** a jurisdiction ∈ `set`.
/// - An **undeclared** Space (`space_jurisdiction == None`) under a restrictive
///   `Some(set)` → `false` — strict undeclared-denied (locked), the same shape
///   as a Space absent from a restrictive `allowed_spaces`.
///
/// Note: `Deny` mode is owned by `policy_permits`; the AND-compose means a
/// `Deny` peer is already blocked regardless of jurisdiction, so this fn does
/// not re-check mode.
pub fn jurisdiction_permits(
    policy: Option<&FederationPolicy>,
    space_jurisdiction: Option<&str>,
) -> bool {
    match policy.and_then(|p| p.allowed_jurisdictions.as_ref()) {
        None => true,
        Some(set) => match space_jurisdiction {
            Some(j) => set.iter().any(|allowed| allowed == j),
            None => false, // strict: undeclared Space fails a restrictive set
        },
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

    fn space(s: &str) -> SpaceXgid {
        SpaceXgid::from_xgid(Xgid::new(s.to_string()))
    }

    #[test]
    fn default_policy_is_permit_all() {
        // The prime invariant expressed as a value: Allow + no space restriction.
        let p = FederationPolicy::default();
        assert_eq!(p.mode, PolicyMode::Allow);
        assert!(p.allowed_spaces.is_none());
    }

    #[test]
    fn set_get_remove() {
        let mut store = FederationPolicyStore::new();
        assert!(store.is_empty());

        store.set(
            node_key("xgen://pubkey/ed25519:AAAA"),
            FederationPolicy {
                allowed_jurisdictions: None,
                mode: PolicyMode::Deny,
                allowed_spaces: None,
            },
        );
        store.set(
            node_key("xgen://pubkey/ed25519:BBBB"),
            FederationPolicy {
                allowed_jurisdictions: None,
                mode: PolicyMode::Allow,
                allowed_spaces: Some(vec![space("xgen://hash/sha256:space1")]),
            },
        );
        assert_eq!(store.len(), 2);
        assert_eq!(store.all().len(), 2);

        let got = store.get(&node_key("xgen://pubkey/ed25519:AAAA")).unwrap();
        assert_eq!(got.mode, PolicyMode::Deny);

        // set is insert-or-replace.
        store.set(
            node_key("xgen://pubkey/ed25519:AAAA"),
            FederationPolicy::default(),
        );
        assert_eq!(store.len(), 2);
        assert_eq!(
            store.get(&node_key("xgen://pubkey/ed25519:AAAA")).unwrap().mode,
            PolicyMode::Allow
        );

        let removed = store.remove(&node_key("xgen://pubkey/ed25519:BBBB"));
        assert!(removed.is_some());
        assert!(store.get(&node_key("xgen://pubkey/ed25519:BBBB")).is_none());
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn absent_peer_has_no_policy() {
        // Faithful absence: the store invents no default record (default-permit
        // lives in the Commit-2 helper, not here).
        let store = FederationPolicyStore::new();
        assert!(store.get(&node_key("xgen://pubkey/ed25519:ZZZZ")).is_none());
    }

    #[test]
    fn serde_round_trip_deny_and_allowed_spaces() {
        let mut store = FederationPolicyStore::new();
        store.set(
            node_key("xgen://pubkey/ed25519:AAAA"),
            FederationPolicy {
                allowed_jurisdictions: None,
                mode: PolicyMode::Deny,
                allowed_spaces: None,
            },
        );
        store.set(
            node_key("xgen://pubkey/ed25519:BBBB"),
            FederationPolicy {
                allowed_jurisdictions: None,
                mode: PolicyMode::Allow,
                allowed_spaces: Some(vec![
                    space("xgen://hash/sha256:space1"),
                    space("xgen://hash/sha256:space2"),
                ]),
            },
        );

        let json = serde_json::to_string(&store).unwrap();
        // mode serialises lowercase (rename_all).
        assert!(json.contains("\"deny\""));
        assert!(json.contains("\"allow\""));

        let back: FederationPolicyStore = serde_json::from_str(&json).unwrap();
        assert_eq!(back.len(), 2);
        let b = back.get(&node_key("xgen://pubkey/ed25519:BBBB")).unwrap();
        assert_eq!(b.mode, PolicyMode::Allow);
        assert_eq!(b.allowed_spaces.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn save_load_round_trip() {
        let mut store = FederationPolicyStore::new();
        store.set(
            node_key("xgen://pubkey/ed25519:AAAA"),
            FederationPolicy {
                allowed_jurisdictions: None,
                mode: PolicyMode::Deny,
                allowed_spaces: None,
            },
        );

        let tmp = NamedTempFile::new().unwrap();
        store.save(tmp.path()).unwrap();

        let loaded = FederationPolicyStore::load(tmp.path()).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(
            loaded.get(&node_key("xgen://pubkey/ed25519:AAAA")).unwrap().mode,
            PolicyMode::Deny
        );
    }

    #[test]
    fn empty_store_saves_and_loads() {
        let store = FederationPolicyStore::new();
        let tmp = NamedTempFile::new().unwrap();
        store.save(tmp.path()).unwrap();
        let loaded = FederationPolicyStore::load(tmp.path()).unwrap();
        assert!(loaded.is_empty());
    }

    // ── policy_permits truth table (FAC-D3 enforcement decision) ────────────────

    #[test]
    fn permits_when_no_policy() {
        // Prime invariant: absent policy permits any space.
        let s = space("xgen://hash/sha256:space1");
        assert!(policy_permits(None, &s));
    }

    #[test]
    fn denies_when_mode_deny() {
        let p = FederationPolicy {
            allowed_jurisdictions: None,
            mode: PolicyMode::Deny,
            allowed_spaces: None,
        };
        assert!(!policy_permits(Some(&p), &space("xgen://hash/sha256:space1")));
        // Deny ignores allowed_spaces entirely.
        let p2 = FederationPolicy {
            allowed_jurisdictions: None,
            mode: PolicyMode::Deny,
            allowed_spaces: Some(vec![space("xgen://hash/sha256:space1")]),
        };
        assert!(!policy_permits(Some(&p2), &space("xgen://hash/sha256:space1")));
    }

    #[test]
    fn permits_all_when_allow_and_no_space_restriction() {
        let p = FederationPolicy {
            allowed_jurisdictions: None,
            mode: PolicyMode::Allow,
            allowed_spaces: None,
        };
        assert!(policy_permits(Some(&p), &space("xgen://hash/sha256:anything")));
    }

    #[test]
    fn allow_with_allowed_spaces_is_restrictive() {
        // Restrictive-only: in-list permitted, out-of-list denied.
        let p = FederationPolicy {
            allowed_jurisdictions: None,
            mode: PolicyMode::Allow,
            allowed_spaces: Some(vec![
                space("xgen://hash/sha256:space1"),
                space("xgen://hash/sha256:space2"),
            ]),
        };
        assert!(policy_permits(Some(&p), &space("xgen://hash/sha256:space1")));
        assert!(policy_permits(Some(&p), &space("xgen://hash/sha256:space2")));
        assert!(!policy_permits(Some(&p), &space("xgen://hash/sha256:space3")));
        // Empty allow-list permits nothing (narrows to the empty intersection).
        let empty = FederationPolicy {
            allowed_jurisdictions: None,
            mode: PolicyMode::Allow,
            allowed_spaces: Some(vec![]),
        };
        assert!(!policy_permits(Some(&empty), &space("xgen://hash/sha256:space1")));
    }

    // ── jurisdiction_permits truth table (Arc G PG-04 / AG-D5) ──────────────────

    fn juris(set: Option<&[&str]>) -> FederationPolicy {
        FederationPolicy {
            mode: PolicyMode::Allow,
            allowed_spaces: None,
            allowed_jurisdictions: set.map(|s| s.iter().map(|x| x.to_string()).collect()),
        }
    }

    #[test]
    fn jurisdiction_permits_no_policy_is_permit_all() {
        // Prime invariant: absent policy ⇒ no jurisdiction restriction.
        assert!(jurisdiction_permits(None, Some("SK")));
        assert!(jurisdiction_permits(None, None));
    }

    #[test]
    fn jurisdiction_permits_no_allow_list_is_no_op() {
        // A policy that sets no allowed_jurisdictions is a no-op on this
        // dimension (dormant until an operator declares a set).
        let p = juris(None);
        assert!(jurisdiction_permits(Some(&p), Some("SK")));
        assert!(jurisdiction_permits(Some(&p), Some("RU")));
        assert!(jurisdiction_permits(Some(&p), None));
    }

    #[test]
    fn jurisdiction_permits_restrictive_set_includes_and_excludes() {
        let p = juris(Some(&["SK", "EU"]));
        assert!(jurisdiction_permits(Some(&p), Some("SK")), "in-set permitted");
        assert!(jurisdiction_permits(Some(&p), Some("EU")), "in-set permitted");
        assert!(!jurisdiction_permits(Some(&p), Some("RU")), "out-of-set denied");
    }

    #[test]
    fn jurisdiction_permits_undeclared_denied_under_restrictive_set() {
        // Strict undeclared-denied (locked) — an undeclared Space fails a
        // restrictive set, the same shape as a Space absent from allowed_spaces.
        let p = juris(Some(&["SK"]));
        assert!(!jurisdiction_permits(Some(&p), None));
        // Empty allow-list permits nothing, declared or not.
        let empty = juris(Some(&[]));
        assert!(!jurisdiction_permits(Some(&empty), Some("SK")));
        assert!(!jurisdiction_permits(Some(&empty), None));
    }

    #[test]
    fn jurisdiction_serde_round_trips_through_store() {
        // Additive serde: allowed_jurisdictions persists + round-trips, and a
        // policy without it omits the key (skip_serializing_if) — backward-compat
        // with pre-Arc-G policy JSON.
        let mut store = FederationPolicyStore::new();
        store.set(node_key("xgen://pubkey/ed25519:JJJJ"), juris(Some(&["SK", "EU"])));
        let json = serde_json::to_string(&store).unwrap();
        assert!(json.contains("allowed_jurisdictions"));
        assert!(json.contains("\"SK\""));
        let back: FederationPolicyStore = serde_json::from_str(&json).unwrap();
        let p = back.get(&node_key("xgen://pubkey/ed25519:JJJJ")).unwrap();
        assert_eq!(p.allowed_jurisdictions.as_ref().unwrap(), &["SK", "EU"]);

        // A default policy omits the key entirely.
        let mut store2 = FederationPolicyStore::new();
        store2.set(node_key("xgen://pubkey/ed25519:KKKK"), FederationPolicy::default());
        let json2 = serde_json::to_string(&store2).unwrap();
        assert!(!json2.contains("allowed_jurisdictions"));
    }
}
