// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

// In-memory Event store — the vanilla default backend behind the `EventStore`
// trait (spec 3.2.5; EventStore milestone, D-080). Append-only. Durability is
// provided by the xgen-node file-persistence layer (atomic write +
// honest-fail/quarantine, D-084); a future engine module (SQLite/redb) is an
// alternative backend behind the same trait, NOT a replacement of this one.

use std::collections::HashMap;

use thiserror::Error;
use xgen_common::module::Descriptor;
use xgen_common::xgid::{EventXgid, Xgid};

use crate::wire::types::Event;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum StoreError {
    #[error("event '{0}' already exists in store")]
    DuplicateEventId(String),
    #[error("event is missing event_id — cannot insert unsigned event")]
    MissingEventId,
}

/// The Event store seam (EventStore milestone, ES-D1; realises D-080).
///
/// The trait abstracts the per-Space **store index** — append, point lookup,
/// append-sequence range, membership, count. It is the swap boundary (ES-D5):
/// consumer functions take `&dyn EventStore` so a future engine module
/// (SQLite/redb, a later milestone) can be substituted without touching them.
/// Owners (`NodeRuntime.stores`, `RoomDag`) hold the concrete backend and may
/// use its inherent convenience methods directly; the trait is what crosses
/// the consumer boundary.
///
/// Contract notes:
/// - **Owned returns** (`get`, `range` clone) keep the trait engine-agnostic —
///   an on-disk engine cannot hand out a borrow into its storage. The vanilla
///   in-memory backend clones.
/// - **`range` is by append sequence (ES-D1 R1):** a monotonic per-store
///   counter assigns each appended event the next sequence number; `range`
///   returns every event from `since_seq` onward, in append order. This is the
///   primitive a future engine backend's incremental fetch is built on. It is
///   *not* causal/topological order — the sync path (`collect_sync_history` +
///   topo-sort) composes causal order against a peer's frontier above this
///   layer and does not go through `range`.
pub trait EventStore {
    /// Append an event. Errors if it has no event_id or the id already exists.
    fn append(&mut self, event: Event) -> Result<(), StoreError>;

    /// Point lookup by event_id. Owned (clones on the in-memory backend).
    fn get(&self, id: &EventXgid) -> Result<Option<Event>, StoreError>;

    /// All events with append-sequence `>= since_seq`, in append order.
    /// `since_seq` past the end yields an empty vec.
    fn range(&self, since_seq: u64) -> Result<Vec<Event>, StoreError>;

    /// True if an event with this id is present.
    fn contains(&self, id: &EventXgid) -> bool;

    /// Number of events stored.
    fn len(&self) -> usize;

    /// True if the store holds no events.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Opaque settings handed to a [`StorageEngine`] at `open` time (SE-D5).
///
/// **Opaque passthrough.** The host reads the `[storage.<engine>]` config
/// sub-table as a [`toml::Value`] and wraps it here; it never interprets the
/// contents. The engine owns and validates *its own* schema via
/// [`EngineSettings::deserialize`], and is **loud on failure** (a bad sub-table
/// yields [`EngineError::InvalidSettings`] and the Node refuses to start).
#[derive(Debug, Clone)]
pub struct EngineSettings(toml::Value);

impl EngineSettings {
    /// Wrap a `[storage.<engine>]` sub-table. An empty table is the natural
    /// "no settings provided" value.
    pub fn new(value: toml::Value) -> Self {
        Self(value)
    }

    /// An empty settings table (no `[storage.<engine>]` section present).
    pub fn empty() -> Self {
        Self(toml::Value::Table(toml::map::Map::new()))
    }

    /// The raw underlying TOML value, for engines that want to inspect it
    /// directly rather than deserialize into a struct.
    pub fn value(&self) -> &toml::Value {
        &self.0
    }

    /// Deserialize the opaque settings into the engine's own schema type. This
    /// is the SE-D5 "plugin validates its own schema, loud on failure" path —
    /// any mismatch becomes [`EngineError::InvalidSettings`].
    pub fn deserialize<T: serde::de::DeserializeOwned>(&self) -> Result<T, EngineError> {
        self.0
            .clone()
            .try_into()
            .map_err(|e: toml::de::Error| EngineError::InvalidSettings(e.to_string()))
    }
}

/// Failure modes when bringing up a [`StorageEngine`] (SE-D5).
#[derive(Debug, Error)]
pub enum EngineError {
    /// The `[storage.<engine>]` settings did not match the engine's schema.
    #[error("invalid storage-engine settings: {0}")]
    InvalidSettings(String),
    /// The engine's backend could not be opened (I/O, corruption, etc.).
    #[error("failed to open storage engine: {0}")]
    Open(String),
}

/// A pluggable storage backend (Storage-Engine / Plugin-Framework milestone,
/// SE-D1) — the **engine contract** sitting a rung above the [`EventStore`]
/// data seam.
///
/// An engine *is* an `EventStore` (the supertrait bound) plus two lifecycle
/// associated functions the data seam has no place for: how to construct one
/// from opaque settings, and how it describes itself. Vanilla
/// [`InMemoryEventStore`] is the default backend when no engine is selected and
/// is deliberately **not** a `StorageEngine` — it needs no settings and fills
/// no slot; an engine module (e.g. `xgen-store-sqlite`) is the alternative,
/// never a replacement of vanilla (D-080).
///
/// ## Sync, v1 — a forward-constraining lock (SE-D1)
///
/// `open` and the `EventStore` methods are synchronous. This fits a sync
/// backend (`rusqlite`) cleanly but commits the engine trait to **sync now**;
/// an async engine would be a breaking trait change, deferred to its own arc.
///
/// ## Durable append-seq is a contract item (SE-D1)
///
/// Vanilla rebuilds its append-sequence counter in RAM on replay. An engine
/// **MUST persist its own append-seq**, or `EventStore::range(since_seq)`
/// breaks across restart. A `since_seq` cursor is only valid within one backend
/// identity — swapping backends renumbers — so v1 makes an engine swap
/// restart-required plus peer re-sync. (Named in the SE-D7 conformance spec.)
///
/// ## Object-safety — intentional, do not "fix" (SE-D1 §3)
///
/// `open(...) -> Result<Self>` and the receiver-less `descriptor()` make
/// `StorageEngine` **not `dyn`-compatible by design**. It is a *static
/// (monomorphised)* trait: the type-erasure to `Box<dyn EventStore>` happens at
/// the per-engine registration site (`register::<E>`, SE-D3), which stores
/// `E::descriptor()` plus a factory closing over `E::open`. The registry holds
/// `Descriptor` + a boxing factory, **never `dyn StorageEngine`**, and owners
/// hold `Box<dyn EventStore>`. Do not add `&self` or attempt to box
/// `dyn StorageEngine` to make the signatures "look" object-safe.
pub trait StorageEngine: EventStore {
    /// Construct the engine from its opaque `[storage.<engine>]` settings.
    /// Validates the engine's own schema; loud on failure (`EngineError`).
    fn open(settings: &EngineSettings) -> Result<Self, EngineError>
    where
        Self: Sized;

    /// The engine's self-description — a const in the engine's own code. The
    /// `kind_id` names the slot it fills; the registry rejects an unknown
    /// `kind_id` loudly (SE-D3).
    fn descriptor() -> Descriptor;
}

/// Append-only in-memory store keyed by event_id (the vanilla default backend,
/// ES-D2). `events` is the index; `order` records insertion sequence so
/// `range(since_seq)` is an O(1) suffix slice over a contiguous, monotonic
/// counter (ES-D1 R1).
pub struct InMemoryEventStore {
    events: HashMap<EventXgid, Event>,
    /// Append-sequence index: `order[seq]` is the event_id appended at that
    /// sequence. Contiguous because the store is append-only (no removal).
    order: Vec<EventXgid>,
}

impl InMemoryEventStore {
    pub fn new() -> Self {
        Self { events: HashMap::new(), order: Vec::new() }
    }

    /// Insert an event. Returns an error if the event has no event_id or if the
    /// id already exists. Maintains the append-sequence `order` index.
    ///
    /// Inherent owner-convenience method: `RoomDag`/`NodeRuntime` hold the
    /// concrete backend and call this directly. The trait's `append` delegates
    /// here, so every insertion path maintains the append-seq counter.
    pub fn insert(&mut self, event: Event) -> Result<(), StoreError> {
        let id = event.event_id.clone().ok_or(StoreError::MissingEventId)?;
        if self.events.contains_key(&id) {
            return Err(StoreError::DuplicateEventId(id.as_str().to_string()));
        }
        self.order.push(id.clone());
        self.events.insert(id, event);
        Ok(())
    }

    /// Borrowing point lookup (owner-convenience; the trait's `get` is owned).
    pub fn get(&self, id: &str) -> Option<&Event> {
        // Pass 2 widens this method to take `&EventXgid`; the wrap collapses then.
        self.events
            .get(&EventXgid::from_xgid(Xgid::new(id.to_string())))
    }

    /// Borrowing membership check (owner-convenience; the trait's `contains`
    /// takes `&EventXgid`).
    pub fn contains(&self, id: &str) -> bool {
        // Pass 2 widens this method to take `&EventXgid`; the wrap collapses then.
        self.events
            .contains_key(&EventXgid::from_xgid(Xgid::new(id.to_string())))
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Iterate over all stored events (order not guaranteed).
    pub fn values(&self) -> impl Iterator<Item = &Event> {
        self.events.values()
    }
}

impl Default for InMemoryEventStore {
    fn default() -> Self {
        Self::new()
    }
}

impl EventStore for InMemoryEventStore {
    fn append(&mut self, event: Event) -> Result<(), StoreError> {
        self.insert(event)
    }

    fn get(&self, id: &EventXgid) -> Result<Option<Event>, StoreError> {
        Ok(self.events.get(id).cloned())
    }

    fn range(&self, since_seq: u64) -> Result<Vec<Event>, StoreError> {
        let start = since_seq as usize;
        if start >= self.order.len() {
            return Ok(Vec::new());
        }
        Ok(self.order[start..]
            .iter()
            .map(|id| self.events[id].clone())
            .collect())
    }

    fn contains(&self, id: &EventXgid) -> bool {
        self.events.contains_key(id)
    }

    fn len(&self) -> usize {
        self.events.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wire::types::{Event, EventType};
    use serde_json::json;

    use xgen_common::xgid::{IdentityXgid, RoomXgid, SpaceXgid};

    fn make_event(id: &str) -> Event {
        let mut ev = Event::new(
            EventType::StateSpaceCreate,
            IdentityXgid::from_xgid(Xgid::new("xgen://pubkey/ed25519:sender".to_string())),
            RoomXgid::from_xgid(Xgid::new("xgen://hash/sha256:room".to_string())),
            SpaceXgid::from_xgid(Xgid::new("xgen://hash/sha256:space".to_string())),
            vec![],
            "2026-04-27T12:00:00Z".to_string(),
            json!({}),
        );
        ev.event_id = Some(EventXgid::from_xgid(Xgid::new(id.to_string())));
        ev
    }

    fn xid(id: &str) -> EventXgid {
        EventXgid::from_xgid(Xgid::new(id.to_string()))
    }

    #[test]
    fn insert_and_retrieve() {
        let mut store = InMemoryEventStore::new();
        store.insert(make_event("xgen://hash/sha256:aaa")).unwrap();
        assert!(store.contains("xgen://hash/sha256:aaa"));
        assert!(store.get("xgen://hash/sha256:aaa").is_some());
    }

    #[test]
    fn duplicate_id_rejected() {
        let mut store = InMemoryEventStore::new();
        store.insert(make_event("xgen://hash/sha256:aaa")).unwrap();
        assert!(matches!(
            store.insert(make_event("xgen://hash/sha256:aaa")),
            Err(StoreError::DuplicateEventId(_))
        ));
    }

    #[test]
    fn missing_event_id_rejected() {
        let mut store = InMemoryEventStore::new();
        let ev = Event::new(
            EventType::MessageText,
            IdentityXgid::from_xgid(Xgid::new("s".to_string())),
            RoomXgid::from_xgid(Xgid::new("r".to_string())),
            SpaceXgid::from_xgid(Xgid::new("sp".to_string())),
            vec![],
            "2026-04-27T12:00:00Z".to_string(),
            json!({}),
        );
        assert!(matches!(store.insert(ev), Err(StoreError::MissingEventId)));
    }

    #[test]
    fn len_and_empty() {
        let mut store = InMemoryEventStore::new();
        assert!(store.is_empty());
        store.insert(make_event("xgen://hash/sha256:a1")).unwrap();
        store.insert(make_event("xgen://hash/sha256:a2")).unwrap();
        assert_eq!(store.len(), 2);
    }

    #[test]
    fn unknown_id_returns_none() {
        let store = InMemoryEventStore::new();
        assert!(store.get("xgen://hash/sha256:nope").is_none());
        assert!(!store.contains("xgen://hash/sha256:nope"));
    }

    // ── EventStore trait surface (ES-D1) ──────────────────────────────────────
    // Exercised through `&dyn` / `&mut dyn` so trait methods resolve (inherent
    // `get(&str)`/`contains(&str)` would otherwise shadow the typed trait ones).

    #[test]
    fn trait_append_and_dedup() {
        let mut store = InMemoryEventStore::new();
        let s: &mut dyn EventStore = &mut store;
        s.append(make_event("xgen://hash/sha256:t1")).unwrap();
        assert!(matches!(
            s.append(make_event("xgen://hash/sha256:t1")),
            Err(StoreError::DuplicateEventId(_))
        ));
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn trait_get_returns_owned() {
        let mut store = InMemoryEventStore::new();
        store.insert(make_event("xgen://hash/sha256:g1")).unwrap();
        let s: &dyn EventStore = &store;
        let got: Option<Event> = s.get(&xid("xgen://hash/sha256:g1")).unwrap();
        assert!(got.is_some());
        assert!(s.get(&xid("xgen://hash/sha256:nope")).unwrap().is_none());
        assert!(s.contains(&xid("xgen://hash/sha256:g1")));
        assert!(!s.contains(&xid("xgen://hash/sha256:nope")));
    }

    #[test]
    fn trait_range_is_append_seq_suffix() {
        let mut store = InMemoryEventStore::new();
        store.insert(make_event("xgen://hash/sha256:r0")).unwrap();
        store.insert(make_event("xgen://hash/sha256:r1")).unwrap();
        store.insert(make_event("xgen://hash/sha256:r2")).unwrap();
        let s: &dyn EventStore = &store;

        // from 0 → all, in append order.
        let all = s.range(0).unwrap();
        let ids: Vec<&str> = all.iter().map(|e| e.event_id.as_ref().unwrap().as_str()).collect();
        assert_eq!(
            ids,
            vec![
                "xgen://hash/sha256:r0",
                "xgen://hash/sha256:r1",
                "xgen://hash/sha256:r2"
            ]
        );

        // suffix from seq 2 → just the last one.
        let tail = s.range(2).unwrap();
        assert_eq!(tail.len(), 1);
        assert_eq!(tail[0].event_id.as_ref().unwrap().as_str(), "xgen://hash/sha256:r2");

        // since_seq at the end and past the end → empty.
        assert!(s.range(3).unwrap().is_empty());
        assert!(s.range(99).unwrap().is_empty());
    }

    // ── StorageEngine trait + register-site type-erasure (SE-D1 / SE-D3) ──────
    // Proves the static engine trait composes and erases to Box<dyn EventStore>
    // at a register-style factory, *without* StorageEngine itself being
    // dyn-compatible (the §3 object-safety story).

    use xgen_common::module::{AssuranceClass, Descriptor, ModuleImplId, ModuleKindId};

    #[derive(serde::Deserialize)]
    struct TestEngineSettings {
        label: String,
    }

    struct TestEngine {
        inner: InMemoryEventStore,
        #[allow(dead_code)]
        label: String,
    }

    impl EventStore for TestEngine {
        fn append(&mut self, event: Event) -> Result<(), StoreError> {
            self.inner.append(event)
        }
        fn get(&self, id: &EventXgid) -> Result<Option<Event>, StoreError> {
            EventStore::get(&self.inner, id)
        }
        fn range(&self, since_seq: u64) -> Result<Vec<Event>, StoreError> {
            self.inner.range(since_seq)
        }
        fn contains(&self, id: &EventXgid) -> bool {
            EventStore::contains(&self.inner, id)
        }
        fn len(&self) -> usize {
            EventStore::len(&self.inner)
        }
    }

    const TEST_KIND: ModuleKindId = ModuleKindId::from_u128(0xabcd);

    impl StorageEngine for TestEngine {
        fn open(settings: &EngineSettings) -> Result<Self, EngineError> {
            let s: TestEngineSettings = settings.deserialize()?;
            Ok(Self { inner: InMemoryEventStore::new(), label: s.label })
        }
        fn descriptor() -> Descriptor {
            Descriptor {
                kind_id: TEST_KIND,
                impl_id: ModuleImplId::from_u128(0x1234),
                name: "test-engine",
                assurance: AssuranceClass::Durable,
            }
        }
    }

    /// Mirrors the SE-D3 register site: erase a concrete `StorageEngine` to a
    /// `fn(&EngineSettings) -> Result<Box<dyn EventStore>, EngineError>` factory
    /// (a plain fn pointer — the leaning `EngineTable` value type for C3).
    fn factory_for<E: StorageEngine + 'static>(
    ) -> fn(&EngineSettings) -> Result<Box<dyn EventStore>, EngineError> {
        |settings| Ok(Box::new(E::open(settings)?))
    }

    #[test]
    fn storage_engine_opens_validates_and_type_erases() {
        let mut table = toml::map::Map::new();
        table.insert("label".into(), toml::Value::String("primary".into()));
        let settings = EngineSettings::new(toml::Value::Table(table));

        // Erase the concrete engine to Box<dyn EventStore> at the factory.
        let make = factory_for::<TestEngine>();
        let mut store: Box<dyn EventStore> = make(&settings).unwrap();
        store.append(make_event("xgen://hash/sha256:e1")).unwrap();
        assert_eq!(store.len(), 1);

        // descriptor() is reachable without an instance (associated fn).
        assert_eq!(TestEngine::descriptor().name, "test-engine");
        assert_eq!(TestEngine::descriptor().kind_id, TEST_KIND);
        assert_eq!(TestEngine::descriptor().assurance, AssuranceClass::Durable);
    }

    #[test]
    fn storage_engine_rejects_bad_settings_loudly() {
        // Empty settings → required `label` missing → InvalidSettings, never a
        // silent default (SE-D5 loud-on-failure).
        let make = factory_for::<TestEngine>();
        assert!(matches!(
            make(&EngineSettings::empty()),
            Err(EngineError::InvalidSettings(_))
        ));
    }
}
