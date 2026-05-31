// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

// Per-Space node-policy store (NP-D1–D6, the node-policy D-071 arc).
//
// A `NodePolicy` is the Node-operator's standing posture for one hosted Space's
// own host behavior — the smallest schema that fills the §3.7.13.6 *actionable*
// auto-moderation gap (distinct from the owner's *display* threshold). It is
// **Node-operator authority** (principal #1, the `force-eject` signer), per
// hosted Space, **non-propagating**: it touches no Space-DAG governance state
// (owner, #3) and no AI-operator delegation (#2) — the Node/Space line is the
// propagate/don't-propagate line (NP-D1). Unlike `force-eject` (an intervention
// that mints a DAG event and propagates), node-policy is local-only config
// (NP-D5).
//
// This is a SIBLING store to `SpaceState`, not a field on it — sibling-shape to
// `FederationPolicyStore` (operator-lifecycle state, kept out of the
// protocol-derived state object). It lives in `space/` because it is
// `SpaceXgid`-keyed (`federation_policy.rs` lives in `federation/` only because
// it is `NodeXgid`-keyed). Persisted JSON at the D-035-convention path
// `xgen-node_node_policy.json`.
//
// FORK X (NP-D3): the store is INERT this arc — **nothing in the running Node
// reads it**. Enforcement (an actionable auto-moderation reader) is deferred to
// the temperature-plugin arc; the two verbs (`space set-node-policy` /
// `show-node-policy`) are the sole consumer. The schema is Y-shaped (built to
// drive a future reader) but X-delivered (no reader yet). PRIME INVARIANT: an
// empty store + `absent == disabled` (NP-D2) = today byte-for-byte; held
// trivially because no consumer reads it.

use std::{collections::HashMap, path::Path};

use serde::{Deserialize, Serialize};
use xgen_common::xgid::SpaceXgid;

use crate::federation::registry::RegistryError;

/// A Node-operator's standing auto-moderation posture for one hosted Space
/// (NP-D2). Smallest v1 schema: a master switch + an optional actionable
/// threshold.
///
/// `auto_moderation` is the master switch; `action_threshold` is the temperature
/// value at or above which the (future, plugin-arc) reader would act — meaningful
/// only when `auto_moderation` is `true`, and constrained to `[0.0, 1.0]`
/// (validated at the verb boundary, `SPACE_8005`). Excluded from v1 (D-065):
/// `cooldown_override`, `rate_cap`, `storage_quota` (no consumer / collide with
/// `max_event_size`).
///
/// `Default` → `{ false, None }` = today byte-for-byte. **Absent == disabled**:
/// a missing store entry and `{ false, None }` are indistinguishable, so `show`
/// on an unset hosted Space returns the default (sibling to the federation-policy
/// prime-invariant-as-a-value).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct NodePolicy {
    /// Master switch for Node-side automated moderation of this Space. `false`
    /// (the default) = disabled = today.
    #[serde(default)]
    pub auto_moderation: bool,
    /// Actionable temperature threshold in `[0.0, 1.0]` — the value at or above
    /// which the future plugin-arc reader would act. `None` = no threshold set.
    /// Only meaningful when `auto_moderation` is `true`.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub action_threshold: Option<f64>,
}

/// Persistent per-Space node-policy store, keyed by Space id (NP-D4: per-Space
/// only, no Node-wide default in v1 — there is no consumer to exercise a second
/// resolution path yet).
///
/// JSON file shape: `{ "policies": { "<space_id>": {...} } }`. Kept a sibling to
/// `SpaceState` (separate file, separate type) because a policy is
/// operator-lifecycle state independent of the protocol-derived Space state —
/// see the module doc-comment.
///
/// Default-absent semantics (no policy → disabled) live in the verb / `Default`
/// (NP-D2), NOT in the store: the store reports presence/absence faithfully and
/// never invents a default record (sibling to `FederationPolicyStore`).
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct NodePolicyStore {
    #[serde(default)]
    policies: HashMap<SpaceXgid, NodePolicy>,
}

impl NodePolicyStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert or replace the policy for a Space (called by `space
    /// set-node-policy`). A full set, not a partial patch (mirrors `federation
    /// set-policy`): the verb builds the complete `NodePolicy`.
    pub fn set(&mut self, space_id: SpaceXgid, policy: NodePolicy) {
        self.policies.insert(space_id, policy);
    }

    /// Remove a Space's policy (reverts it to default-disabled). Returns it if
    /// present.
    pub fn remove(&mut self, space_id: &SpaceXgid) -> Option<NodePolicy> {
        self.policies.remove(space_id)
    }

    pub fn get(&self, space_id: &SpaceXgid) -> Option<&NodePolicy> {
        self.policies.get(space_id)
    }

    pub fn all(&self) -> Vec<(&SpaceXgid, &NodePolicy)> {
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

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use xgen_common::xgid::Xgid;

    fn space(s: &str) -> SpaceXgid {
        SpaceXgid::from_xgid(Xgid::new(s.to_string()))
    }

    #[test]
    fn default_policy_is_disabled() {
        // Absent == disabled, expressed as a value: the prime invariant.
        let p = NodePolicy::default();
        assert!(!p.auto_moderation);
        assert!(p.action_threshold.is_none());
    }

    #[test]
    fn set_get_remove() {
        let mut store = NodePolicyStore::new();
        assert!(store.is_empty());

        store.set(
            space("xgen://hash/sha256:space1"),
            NodePolicy {
                auto_moderation: true,
                action_threshold: Some(0.8),
            },
        );
        store.set(
            space("xgen://hash/sha256:space2"),
            NodePolicy {
                auto_moderation: false,
                action_threshold: None,
            },
        );
        assert_eq!(store.len(), 2);
        assert_eq!(store.all().len(), 2);

        let got = store.get(&space("xgen://hash/sha256:space1")).unwrap();
        assert!(got.auto_moderation);
        assert_eq!(got.action_threshold, Some(0.8));

        // set is insert-or-replace.
        store.set(space("xgen://hash/sha256:space1"), NodePolicy::default());
        assert_eq!(store.len(), 2);
        assert!(!store.get(&space("xgen://hash/sha256:space1")).unwrap().auto_moderation);

        let removed = store.remove(&space("xgen://hash/sha256:space2"));
        assert!(removed.is_some());
        assert!(store.get(&space("xgen://hash/sha256:space2")).is_none());
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn absent_space_has_no_policy() {
        // Faithful absence: the store invents no default record (absent ==
        // disabled lives in the verb/Default, not here).
        let store = NodePolicyStore::new();
        assert!(store.get(&space("xgen://hash/sha256:nope")).is_none());
    }

    #[test]
    fn save_load_round_trip() {
        let mut store = NodePolicyStore::new();
        store.set(
            space("xgen://hash/sha256:space1"),
            NodePolicy {
                auto_moderation: true,
                action_threshold: Some(0.5),
            },
        );

        let tmp = NamedTempFile::new().unwrap();
        store.save(tmp.path()).unwrap();

        let loaded = NodePolicyStore::load(tmp.path()).unwrap();
        assert_eq!(loaded.len(), 1);
        let p = loaded.get(&space("xgen://hash/sha256:space1")).unwrap();
        assert!(p.auto_moderation);
        assert_eq!(p.action_threshold, Some(0.5));
    }

    #[test]
    fn empty_store_saves_and_loads() {
        let store = NodePolicyStore::new();
        let tmp = NamedTempFile::new().unwrap();
        store.save(tmp.path()).unwrap();
        let loaded = NodePolicyStore::load(tmp.path()).unwrap();
        assert!(loaded.is_empty());
    }

    #[test]
    fn action_threshold_omitted_when_none() {
        // skip_serializing_if keeps a disabled policy compact on disk.
        let store_default = {
            let mut s = NodePolicyStore::new();
            s.set(space("xgen://hash/sha256:s"), NodePolicy::default());
            s
        };
        let json = serde_json::to_string(&store_default).unwrap();
        assert!(!json.contains("action_threshold"));
        assert!(json.contains("auto_moderation"));
    }
}
