// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

// DAG module — Event storage, graph tracking, pending buffer (spec 3.2.5).

pub mod graph;
pub mod pending;
pub mod store;

use thiserror::Error;

use crate::identity::registry::IdentityRegistry;
use crate::wire::types::Event;

use graph::{DagGraph, GraphError};
use pending::PendingBuffer;
use store::{EventStore, StoreError};

#[derive(Debug, Error)]
pub enum DagError {
    #[error("store error: {0}")]
    Store(#[from] StoreError),
    #[error("graph error: {0}")]
    Graph(#[from] GraphError),
    #[error("event held in pending buffer: missing {0} predecessor(s)")]
    Pending(usize),
}

/// A single Room's complete Event DAG — store, tips, and pending buffer combined.
///
/// Usage:
///   1. Call `insert()` for each arriving Event.
///   2. If Ok(drained) is returned, process the returned events in order (the
///      originally inserted event is always first; drained peers follow).
///   3. If Err(DagError::Pending) is returned, the event is buffered; no action needed.
///   4. On success, query `current_tips()` to know what to reference in the next Event.
pub struct RoomDag {
    store: EventStore,
    graph: DagGraph,
    pending: PendingBuffer,
}

impl RoomDag {
    pub fn new() -> Self {
        Self {
            store: EventStore::new(),
            graph: DagGraph::new(),
            pending: PendingBuffer::new(),
        }
    }

    /// Try to insert an Event into the DAG.
    ///
    /// Returns:
    /// - `Ok(Vec<Event>)` — event accepted; vec contains the inserted event plus any
    ///   pending events that became unblocked as a result (may be empty).
    /// - `Err(DagError::Pending)` — event has unknown predecessors and was buffered.
    /// - `Err(e)` — structural violation; event is rejected outright.
    pub fn insert(&mut self, event: Event) -> Result<Vec<Event>, DagError> {
        // Identify which prev_events are missing.
        let missing: Vec<String> = event
            .prev_events
            .iter()
            .filter(|id| !self.store.contains(id))
            .cloned()
            .collect();

        if !missing.is_empty() {
            let count = missing.len();
            // RoomDag is a structural-only layer below the F-4 validation
            // core; events arriving here are not identity-checked, so
            // `missing_identity` is always None. PendingBuffer's
            // `try_release` short-circuits the identity check for entries
            // added with None — the registry passed to `resolve` below is
            // unused for these entries.
            self.pending.add(event, &missing, None);
            return Err(DagError::Pending(count));
        }

        // Validate DAG rules and update graph.
        self.graph.add_event(&event, &self.store)?;

        // Capture the event_id before consuming event.
        let event_id = event.event_id.clone().unwrap_or_default();

        // Insert into store.
        self.store.insert(event.clone())?;

        // Release any pending events that were waiting for this one.
        let mut accepted = vec![event];
        self.drain_pending(&event_id, &mut accepted);

        Ok(accepted)
    }

    /// Retrieve an Event by ID.
    pub fn get(&self, id: &str) -> Option<&Event> {
        self.store.get(id)
    }

    /// Current DAG tips — event_ids with no successors.
    pub fn current_tips(&self) -> Vec<String> {
        self.graph.current_tips()
    }

    pub fn event_count(&self) -> usize {
        self.store.len()
    }

    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// Recursively drain the pending buffer after a new event was inserted.
    fn drain_pending(&mut self, resolved_id: &str, accepted: &mut Vec<Event>) {
        // RoomDag-level buffer entries all have `missing_identity: None`
        // (see insert() above), so the empty registry passed here is
        // unused by `try_release`. Kept explicit so the dependency on
        // `IdentityRegistry` for the resolve() signature is visible.
        let empty_registry = IdentityRegistry::new();
        let ready = self.pending.resolve(resolved_id, &self.store, &empty_registry);
        for ev in ready {
            if self.graph.add_event(&ev, &self.store).is_ok() {
                let next_id = ev.event_id.clone().unwrap_or_default();
                if self.store.insert(ev.clone()).is_ok() {
                    accepted.push(ev);
                    self.drain_pending(&next_id, accepted);
                }
            }
        }
    }
}

impl Default for RoomDag {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wire::types::{Event, EventType};
    use serde_json::json;

    fn make_event(id: &str, event_type: EventType, prev: Vec<&str>) -> Event {
        let mut ev = Event::new(
            event_type,
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

    #[test]
    fn linear_chain() {
        let mut dag = RoomDag::new();
        dag.insert(make_event("id:e0", EventType::StateRoomCreate, vec![])).unwrap();
        dag.insert(make_event("id:e1", EventType::MessageText, vec!["id:e0"])).unwrap();
        dag.insert(make_event("id:e2", EventType::MessageText, vec!["id:e1"])).unwrap();
        assert_eq!(dag.event_count(), 3);
        assert_eq!(dag.current_tips(), vec!["id:e2"]);
    }

    #[test]
    fn fork_and_merge() {
        let mut dag = RoomDag::new();

        // E0 (root)
        dag.insert(make_event("id:e0", EventType::StateRoomCreate, vec![])).unwrap();

        // Fork: E1 and E2 both reference E0
        dag.insert(make_event("id:e1", EventType::MessageText, vec!["id:e0"])).unwrap();
        dag.insert(make_event("id:e2", EventType::MessageText, vec!["id:e0"])).unwrap();
        let mut tips = dag.current_tips();
        tips.sort();
        assert_eq!(tips, vec!["id:e1", "id:e2"]);

        // Merge: E3 references both E1 and E2
        dag.insert(make_event("id:e3", EventType::MessageText, vec!["id:e1", "id:e2"])).unwrap();
        assert_eq!(dag.current_tips(), vec!["id:e3"]);
        assert_eq!(dag.event_count(), 4);
    }

    #[test]
    fn out_of_order_delivery_via_pending() {
        let mut dag = RoomDag::new();

        // E0 is the root.
        dag.insert(make_event("id:e0", EventType::StateRoomCreate, vec![])).unwrap();

        // E2 arrives before E1 — goes to pending.
        let result = dag.insert(make_event("id:e2", EventType::MessageText, vec!["id:e1"]));
        assert!(matches!(result, Err(DagError::Pending(1))));
        assert_eq!(dag.pending_count(), 1);
        assert_eq!(dag.event_count(), 1);

        // E1 arrives — both E1 and E2 should now be accepted.
        let accepted = dag
            .insert(make_event("id:e1", EventType::MessageText, vec!["id:e0"]))
            .unwrap();
        assert_eq!(accepted.len(), 2); // E1 + E2 drained
        assert_eq!(dag.event_count(), 3);
        assert_eq!(dag.pending_count(), 0);
        assert_eq!(dag.current_tips(), vec!["id:e2"]);
    }

    #[test]
    fn chain_of_pending_events_all_drain() {
        let mut dag = RoomDag::new();

        dag.insert(make_event("id:e0", EventType::StateRoomCreate, vec![])).unwrap();

        // Arrive in reverse: e3 → e2 → e1, then the missing root e1 arrives last.
        dag.insert(make_event("id:e3", EventType::MessageText, vec!["id:e2"])).unwrap_err();
        dag.insert(make_event("id:e2", EventType::MessageText, vec!["id:e1"])).unwrap_err();
        assert_eq!(dag.pending_count(), 2);

        // E1 arrives — cascade: E1 is inserted, drains E2, which drains E3.
        let accepted = dag
            .insert(make_event("id:e1", EventType::MessageText, vec!["id:e0"]))
            .unwrap();
        assert_eq!(accepted.len(), 3); // e1 + e2 + e3
        assert_eq!(dag.event_count(), 4);
        assert_eq!(dag.pending_count(), 0);
        assert_eq!(dag.current_tips(), vec!["id:e3"]);
    }

    #[test]
    fn retrieve_event_by_id() {
        let mut dag = RoomDag::new();
        dag.insert(make_event("id:e0", EventType::StateRoomCreate, vec![])).unwrap();
        assert!(dag.get("id:e0").is_some());
        assert!(dag.get("id:unknown").is_none());
    }

    #[test]
    fn duplicate_event_rejected() {
        let mut dag = RoomDag::new();
        dag.insert(make_event("id:e0", EventType::StateRoomCreate, vec![])).unwrap();
        assert!(dag.insert(make_event("id:e0", EventType::StateRoomCreate, vec![])).is_err());
    }
}
