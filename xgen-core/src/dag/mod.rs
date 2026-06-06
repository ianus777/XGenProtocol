// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

// DAG module — Event storage, graph tracking, pending buffer (spec 3.2.5).

pub mod graph;
pub mod pending;
pub mod store;

use thiserror::Error;
use xgen_common::xgid::{EventXgid, Xgid};

use crate::identity::registry::IdentityRegistry;
use crate::node::runtime::EventOrigin;
use crate::wire::types::Event;

use graph::{DagGraph, GraphError};
use pending::PendingBuffer;
use store::{EventStore, InMemoryEventStore, StoreError};

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
    // SE-D6 (Storage-Engine milestone): boxed so the vanilla
    // `InMemoryEventStore` can be swapped for an engine module behind the same
    // `EventStore` trait. Behaviour-neutral — the box holds the vanilla backend.
    // `+ Send + Sync` mirrors `NodeRuntime.stores` (the concrete backend already
    // satisfies them; keeps `RoomDag` usable across async boundaries).
    store: Box<dyn EventStore + Send + Sync>,
    graph: DagGraph,
    pending: PendingBuffer,
}

impl RoomDag {
    pub fn new() -> Self {
        Self {
            store: Box::new(InMemoryEventStore::new()),
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
        // Identify which prev_events are missing. event.prev_events is
        // Vec<EventXgid>; collect typed clones for the post-Pass-2 PendingBuffer::add
        // signature (Surface #3 Q3.1).
        let missing: Vec<EventXgid> = event
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
            //
            // M8.6 (clock seam) — RoomDag is OUTSIDE the federation-stress seam
            // fence (it holds no `Clock`), so it supplies a real monotonic stamp
            // directly. Behaviour-identical to the pre-seam internal
            // `received_at: Instant::now()`.
            //
            // INV-EXP (D-1) — RoomDag is a structural-only layer that never
            // re-dispatches through the F-4 origin-gated pipeline (it drains via
            // `accept_event`, which is origin-blind), so the stored origin is
            // never consumed here. `LocallySubmitted` is the neutral default.
            self.pending
                .add(event, EventOrigin::LocallySubmitted, &missing, None, None, std::time::Instant::now());
            return Err(DagError::Pending(count));
        }

        // Validate DAG rules and update graph. SE-D6: reborrow the boxed store
        // as `&dyn EventStore` (the consumer's param type) — `&Box` does not
        // coerce implicitly.
        self.graph.add_event(&event, &*self.store)?;

        // Capture the typed event_id before consuming event (Surface #3 Q3.2 —
        // drain_pending takes &EventXgid). If event lacks an event_id the drain
        // step is a no-op (no buffered entry could reference a missing id),
        // sibling-shape to the pre-Pass-2 empty-string semantics.
        let event_id = event.event_id.clone();

        // Insert into store (SE-D6: trait `append` through the box).
        self.store.append(event.clone())?;

        // Release any pending events that were waiting for this one.
        let mut accepted = vec![event];
        if let Some(ref eid) = event_id {
            self.drain_pending(eid, &mut accepted);
        }

        Ok(accepted)
    }

    /// Retrieve an Event by ID.
    ///
    /// SE-D6: the boxed `EventStore::get` is engine-agnostic and returns an
    /// owned `Event` (an on-disk engine cannot hand out a borrow); `RoomDag` is
    /// a structural-only layer so the owned clone is fine.
    pub fn get(&self, id: &str) -> Option<Event> {
        self.store
            .get(&EventXgid::from_xgid(Xgid::new(id.to_string())))
            .ok()
            .flatten()
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
    fn drain_pending(&mut self, resolved_id: &EventXgid, accepted: &mut Vec<Event>) {
        // RoomDag-level buffer entries all have `missing_identity: None`
        // (see insert() above), so the empty registry passed here is
        // unused by `try_release`. Kept explicit so the dependency on
        // `IdentityRegistry` for the resolve() signature is visible.
        let empty_registry = IdentityRegistry::new();
        let ready = self.pending.resolve(resolved_id, &*self.store, &empty_registry);
        // INV-EXP (D-1) — resolve now yields (Event, EventOrigin); RoomDag is
        // origin-blind (structural-only), so discard the origin here.
        for (ev, _origin) in ready {
            if self.graph.add_event(&ev, &*self.store).is_ok() {
                // Pass 2 (Surface #3 Q3.2) — drain_pending now takes &EventXgid;
                // capture typed event_id and recurse without &str projection.
                let next_id = ev.event_id.clone();
                if self.store.append(ev.clone()).is_ok() {
                    accepted.push(ev);
                    if let Some(ref nid) = next_id {
                        self.drain_pending(nid, accepted);
                    }
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
    use xgen_common::xgid::{EventXgid, IdentityXgid, RoomXgid, SpaceXgid, Xgid};

    fn make_event(id: &str, event_type: EventType, prev: Vec<&str>) -> Event {
        let mut ev = Event::new(
            event_type,
            IdentityXgid::from_xgid(Xgid::new("xgen://pubkey/ed25519:sender".to_string())),
            RoomXgid::from_xgid(Xgid::new("xgen://hash/sha256:room".to_string())),
            SpaceXgid::from_xgid(Xgid::new("xgen://hash/sha256:space".to_string())),
            prev.iter()
                .map(|s| EventXgid::from_xgid(Xgid::new(s.to_string())))
                .collect(),
            "2026-04-27T12:00:00Z".to_string(),
            json!({}),
        );
        ev.event_id = Some(EventXgid::from_xgid(Xgid::new(id.to_string())));
        ev
    }

    #[test]
    fn linear_chain() {
        let mut dag = RoomDag::new();
        dag.insert(make_event("id:e0", EventType::StateSpaceCreate, vec![])).unwrap();
        dag.insert(make_event("id:e1", EventType::MessageText, vec!["id:e0"])).unwrap();
        dag.insert(make_event("id:e2", EventType::MessageText, vec!["id:e1"])).unwrap();
        assert_eq!(dag.event_count(), 3);
        assert_eq!(dag.current_tips(), vec!["id:e2"]);
    }

    #[test]
    fn fork_and_merge() {
        let mut dag = RoomDag::new();

        // E0 (root)
        dag.insert(make_event("id:e0", EventType::StateSpaceCreate, vec![])).unwrap();

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
        dag.insert(make_event("id:e0", EventType::StateSpaceCreate, vec![])).unwrap();

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

        dag.insert(make_event("id:e0", EventType::StateSpaceCreate, vec![])).unwrap();

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
        dag.insert(make_event("id:e0", EventType::StateSpaceCreate, vec![])).unwrap();
        assert!(dag.get("id:e0").is_some());
        assert!(dag.get("id:unknown").is_none());
    }

    #[test]
    fn duplicate_event_rejected() {
        let mut dag = RoomDag::new();
        dag.insert(make_event("id:e0", EventType::StateSpaceCreate, vec![])).unwrap();
        assert!(dag.insert(make_event("id:e0", EventType::StateSpaceCreate, vec![])).is_err());
    }
}
