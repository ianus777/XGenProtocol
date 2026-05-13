// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

// Append-only in-memory Event store (spec 3.2.5).
// Phase 1: no persistence — store lives in process memory only.
// Phase 2: replace with an indexed on-disk store when the smoke test is done.

use std::collections::HashMap;

use thiserror::Error;

use crate::wire::types::Event;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum StoreError {
    #[error("event '{0}' already exists in store")]
    DuplicateEventId(String),
    #[error("event is missing event_id — cannot insert unsigned event")]
    MissingEventId,
}

/// Append-only in-memory store keyed by event_id.
pub struct EventStore {
    events: HashMap<String, Event>,
}

impl EventStore {
    pub fn new() -> Self {
        Self { events: HashMap::new() }
    }

    /// Insert an event. Returns an error if the event has no event_id or if the id already exists.
    pub fn insert(&mut self, event: Event) -> Result<(), StoreError> {
        let id = event.event_id.clone().ok_or(StoreError::MissingEventId)?;
        if self.events.contains_key(&id) {
            return Err(StoreError::DuplicateEventId(id));
        }
        self.events.insert(id, event);
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&Event> {
        self.events.get(id)
    }

    pub fn contains(&self, id: &str) -> bool {
        self.events.contains_key(id)
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

impl Default for EventStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wire::types::{Event, EventType};
    use serde_json::json;

    fn make_event(id: &str) -> Event {
        let mut ev = Event::new(
            EventType::StateRoomCreate,
            "xgen://pubkey/ed25519:sender".to_string(),
            "xgen://hash/sha256:room".to_string(),
            "xgen://hash/sha256:space".to_string(),
            vec![],
            "2026-04-27T12:00:00Z".to_string(),
            json!({}),
        );
        ev.event_id = Some(id.to_string());
        ev
    }

    #[test]
    fn insert_and_retrieve() {
        let mut store = EventStore::new();
        store.insert(make_event("xgen://hash/sha256:aaa")).unwrap();
        assert!(store.contains("xgen://hash/sha256:aaa"));
        assert!(store.get("xgen://hash/sha256:aaa").is_some());
    }

    #[test]
    fn duplicate_id_rejected() {
        let mut store = EventStore::new();
        store.insert(make_event("xgen://hash/sha256:aaa")).unwrap();
        assert!(matches!(
            store.insert(make_event("xgen://hash/sha256:aaa")),
            Err(StoreError::DuplicateEventId(_))
        ));
    }

    #[test]
    fn missing_event_id_rejected() {
        let mut store = EventStore::new();
        let ev = Event::new(
            EventType::MessageText,
            "s".to_string(), "r".to_string(), "sp".to_string(),
            vec![], "2026-04-27T12:00:00Z".to_string(), json!({}),
        );
        assert!(matches!(store.insert(ev), Err(StoreError::MissingEventId)));
    }

    #[test]
    fn len_and_empty() {
        let mut store = EventStore::new();
        assert!(store.is_empty());
        store.insert(make_event("xgen://hash/sha256:a1")).unwrap();
        store.insert(make_event("xgen://hash/sha256:a2")).unwrap();
        assert_eq!(store.len(), 2);
    }

    #[test]
    fn unknown_id_returns_none() {
        let store = EventStore::new();
        assert!(store.get("xgen://hash/sha256:nope").is_none());
        assert!(!store.contains("xgen://hash/sha256:nope"));
    }
}
