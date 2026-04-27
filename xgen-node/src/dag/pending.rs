// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

// Pending buffer for Events whose prev_events are not yet known (spec 3.2.5).
//
// When a Node receives an Event that references a predecessor it hasn't seen yet,
// it must not reject the Event outright — the predecessor may arrive shortly from
// another peer.  The Event is held here until all its predecessors are present
// in the store, at which point it is returned to the caller for processing.
//
// An Event may be waiting for multiple missing predecessors simultaneously.
// It is only released when ALL of them have been resolved.

use std::collections::{HashMap, HashSet};

use crate::wire::types::Event;

use super::store::EventStore;

/// Holds Events that are waiting for one or more missing predecessors.
pub struct PendingBuffer {
    /// All pending Events, keyed by their own event_id.
    events: HashMap<String, Event>,
    /// For each missing predecessor ID, the set of pending event_ids waiting for it.
    waiting_for: HashMap<String, HashSet<String>>,
}

impl PendingBuffer {
    pub fn new() -> Self {
        Self {
            events: HashMap::new(),
            waiting_for: HashMap::new(),
        }
    }

    /// Add an Event to the pending buffer.
    /// `missing_ids` is the subset of the Event's prev_events that are not yet in the store.
    pub fn add(&mut self, event: Event, missing_ids: &[String]) {
        let eid = match &event.event_id {
            Some(id) => id.clone(),
            None => return, // cannot buffer an event with no ID
        };
        for mid in missing_ids {
            self.waiting_for
                .entry(mid.clone())
                .or_default()
                .insert(eid.clone());
        }
        self.events.insert(eid, event);
    }

    /// Notify the buffer that `resolved_id` has been added to `store`.
    /// Returns all Events that are now fully unblocked (every prev_event is in the store).
    /// Removes those Events from the buffer.
    pub fn resolve(&mut self, resolved_id: &str, store: &EventStore) -> Vec<Event> {
        let candidates = match self.waiting_for.remove(resolved_id) {
            Some(c) => c,
            None => return vec![],
        };

        let mut ready = Vec::new();
        for cid in candidates {
            if let Some(ev) = self.events.get(&cid) {
                let all_known = ev.prev_events.iter().all(|pid| store.contains(pid));
                if all_known {
                    if let Some(ev) = self.events.remove(&cid) {
                        // Clean this event out of any other waiting_for entries.
                        for prev_id in &ev.prev_events {
                            if let Some(waiters) = self.waiting_for.get_mut(prev_id) {
                                waiters.remove(&cid);
                            }
                        }
                        ready.push(ev);
                    }
                }
                // If not all known yet, leave in buffer — it will be released
                // when the remaining predecessor(s) arrive.
            }
        }
        ready
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// True if an Event with this event_id is currently buffered.
    pub fn contains(&self, id: &str) -> bool {
        self.events.contains_key(id)
    }
}

impl Default for PendingBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wire::types::{Event, EventType};
    use serde_json::json;

    fn make_event(id: &str, prev: Vec<&str>) -> Event {
        let mut ev = Event::new(
            EventType::MessageText,
            "xgen://pubkey/ed25519:sender".to_string(),
            "xgen://hash/sha256:room".to_string(),
            "xgen://hash/sha256:space".to_string(),
            prev.iter().map(|s| s.to_string()).collect(),
            "2026-04-27T12:00:00Z".to_string(),
            json!({}),
        );
        ev.event_id = Some(id.to_string());
        ev
    }

    fn store_with(ids: &[&str]) -> EventStore {
        let mut s = EventStore::new();
        for id in ids {
            let mut ev = Event::new(
                EventType::StateRoomCreate,
                "s".to_string(), "r".to_string(), "sp".to_string(),
                vec![], "2026-04-27T12:00:00Z".to_string(), json!({}),
            );
            ev.event_id = Some(id.to_string());
            s.insert(ev).unwrap();
        }
        s
    }

    #[test]
    fn event_released_when_predecessor_arrives() {
        let mut buf = PendingBuffer::new();
        let mut store = EventStore::new();

        // E1 references E0, but E0 is not in store yet.
        let e1 = make_event("id:e1", vec!["id:e0"]);
        buf.add(e1, &["id:e0".to_string()]);
        assert_eq!(buf.len(), 1);

        // Now E0 arrives and is inserted into the store.
        let mut e0 = Event::new(
            EventType::StateRoomCreate,
            "s".to_string(), "r".to_string(), "sp".to_string(),
            vec![], "2026-04-27T12:00:00Z".to_string(), json!({}),
        );
        e0.event_id = Some("id:e0".to_string());
        store.insert(e0).unwrap();

        let ready = buf.resolve("id:e0", &store);
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].event_id.as_deref(), Some("id:e1"));
        assert!(buf.is_empty());
    }

    #[test]
    fn event_with_two_missing_predecessors_waits_for_both() {
        let mut buf = PendingBuffer::new();

        // E2 waits for both E0 and E1.
        let e2 = make_event("id:e2", vec!["id:e0", "id:e1"]);
        buf.add(e2, &["id:e0".to_string(), "id:e1".to_string()]);

        // E0 arrives — E2 still waits for E1.
        let store_with_e0 = store_with(&["id:e0"]);
        let ready = buf.resolve("id:e0", &store_with_e0);
        assert!(ready.is_empty());
        assert_eq!(buf.len(), 1);

        // E1 arrives — E2 is now fully unblocked.
        let store_with_both = store_with(&["id:e0", "id:e1"]);
        let ready = buf.resolve("id:e1", &store_with_both);
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].event_id.as_deref(), Some("id:e2"));
        assert!(buf.is_empty());
    }

    #[test]
    fn multiple_events_waiting_for_same_predecessor() {
        let mut buf = PendingBuffer::new();

        // E1 and E2 both wait for E0.
        let e1 = make_event("id:e1", vec!["id:e0"]);
        let e2 = make_event("id:e2", vec!["id:e0"]);
        buf.add(e1, &["id:e0".to_string()]);
        buf.add(e2, &["id:e0".to_string()]);
        assert_eq!(buf.len(), 2);

        let store = store_with(&["id:e0"]);
        let ready = buf.resolve("id:e0", &store);
        assert_eq!(ready.len(), 2);
        assert!(buf.is_empty());
    }

    #[test]
    fn resolve_unknown_id_returns_empty() {
        let mut buf = PendingBuffer::new();
        let store = EventStore::new();
        let ready = buf.resolve("id:nobody", &store);
        assert!(ready.is_empty());
    }

    #[test]
    fn contains_returns_correct_state() {
        let mut buf = PendingBuffer::new();
        let e1 = make_event("id:e1", vec!["id:e0"]);
        buf.add(e1, &["id:e0".to_string()]);
        assert!(buf.contains("id:e1"));
        assert!(!buf.contains("id:e0"));
    }
}
