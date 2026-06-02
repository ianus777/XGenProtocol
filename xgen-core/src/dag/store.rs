// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

// Append-only in-memory Event store (spec 3.2.5).
// Phase 1: no persistence — store lives in process memory only.
// Phase 2: replace with an indexed on-disk store when the smoke test is done.

use std::collections::HashMap;

use thiserror::Error;
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
}
