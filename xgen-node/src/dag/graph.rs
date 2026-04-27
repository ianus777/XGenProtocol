// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

// DAG tip tracking and structural validation (spec 3.2.5).
//
// Responsibilities:
//   - Maintain the set of current DAG tips (Events with no successors)
//   - Validate prev_events rules before a new Event enters the store
//   - Detect self-reference (the only cycle possible when inserting a new Event)
//
// Note: full cycle detection ("no descendant in prev_events") is trivially satisfied
// for any newly inserted Event — it has no descendants yet. Only self-reference needs
// an explicit check.

use std::collections::{HashMap, HashSet};

use thiserror::Error;

use crate::wire::types::{Event, EventType};

use super::store::EventStore;

/// Maximum prev_events entries (spec 3.2.5 — Phase 1 limit).
pub const MAX_PREV_EVENTS: usize = 10;

/// Event types whose prev_events MUST be an empty array (DAG roots).
fn is_dag_root_type(et: &EventType) -> bool {
    matches!(
        et,
        EventType::StateSpaceCreate
            | EventType::StateDmSpaceCreate
            | EventType::StateRoomCreate
    )
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum GraphError {
    #[error("event is missing event_id")]
    MissingEventId,
    #[error("root event type '{0}' must have empty prev_events")]
    RootEventHasPrevEvents(&'static str),
    #[error("non-root event must have at least one prev_event")]
    EmptyPrevEvents,
    #[error("prev_events has {0} entries; maximum is {}", MAX_PREV_EVENTS)]
    TooManyPrevEvents(usize),
    #[error("event references itself in prev_events")]
    SelfReference,
    #[error("prev_event '{0}' is not in the store")]
    UnknownPrevEvent(String),
}

/// Tracks DAG tips and predecessor/successor relationships for one Room's Event log.
pub struct DagGraph {
    /// event_ids that currently have no successors.
    tips: HashSet<String>,
    /// event_id → set of event_ids that reference it as a predecessor.
    successors: HashMap<String, HashSet<String>>,
}

impl DagGraph {
    pub fn new() -> Self {
        Self {
            tips: HashSet::new(),
            successors: HashMap::new(),
        }
    }

    /// Validate and register a new Event in the DAG.
    ///
    /// Precondition: all prev_events must already be in `store`.
    /// This method does NOT insert into the store — that is the caller's responsibility.
    pub fn add_event(&mut self, event: &Event, store: &EventStore) -> Result<(), GraphError> {
        let id = event.event_id.as_deref().ok_or(GraphError::MissingEventId)?;

        let is_root = is_dag_root_type(&event.event_type);

        // Root events must have empty prev_events; non-root must have at least one.
        if is_root && !event.prev_events.is_empty() {
            return Err(GraphError::RootEventHasPrevEvents(event.event_type.as_str()));
        }
        if !is_root && event.prev_events.is_empty() {
            return Err(GraphError::EmptyPrevEvents);
        }

        // Phase 1 limit on prev_events fanin.
        if event.prev_events.len() > MAX_PREV_EVENTS {
            return Err(GraphError::TooManyPrevEvents(event.prev_events.len()));
        }

        // Self-reference check.
        if event.prev_events.iter().any(|p| p == id) {
            return Err(GraphError::SelfReference);
        }

        // All prev_events must be in the store.
        for prev_id in &event.prev_events {
            if !store.contains(prev_id) {
                return Err(GraphError::UnknownPrevEvent(prev_id.clone()));
            }
        }

        // Update tips: predecessors are no longer tips; this event becomes a tip.
        for prev_id in &event.prev_events {
            self.tips.remove(prev_id);
            self.successors
                .entry(prev_id.clone())
                .or_default()
                .insert(id.to_string());
        }
        self.tips.insert(id.to_string());

        Ok(())
    }

    /// Current DAG tips — event_ids with no successors.
    pub fn current_tips(&self) -> Vec<String> {
        let mut tips: Vec<String> = self.tips.iter().cloned().collect();
        tips.sort(); // stable order for deterministic tests
        tips
    }

    pub fn tip_count(&self) -> usize {
        self.tips.len()
    }

    /// True if `id` is currently a tip.
    pub fn is_tip(&self, id: &str) -> bool {
        self.tips.contains(id)
    }
}

impl Default for DagGraph {
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

    fn insert_both(store: &mut EventStore, graph: &mut DagGraph, event: Event) {
        graph.add_event(&event, store).unwrap();
        store.insert(event).unwrap();
    }

    #[test]
    fn root_event_becomes_only_tip() {
        let mut store = EventStore::new();
        let mut graph = DagGraph::new();
        let e0 = make_event("id:e0", EventType::StateRoomCreate, vec![]);
        insert_both(&mut store, &mut graph, e0);
        assert_eq!(graph.tip_count(), 1);
        assert!(graph.is_tip("id:e0"));
    }

    #[test]
    fn linear_chain_has_single_tip() {
        let mut store = EventStore::new();
        let mut graph = DagGraph::new();

        insert_both(&mut store, &mut graph, make_event("id:e0", EventType::StateRoomCreate, vec![]));
        insert_both(&mut store, &mut graph, make_event("id:e1", EventType::MessageText, vec!["id:e0"]));
        insert_both(&mut store, &mut graph, make_event("id:e2", EventType::MessageText, vec!["id:e1"]));

        assert_eq!(graph.tip_count(), 1);
        assert!(graph.is_tip("id:e2"));
        assert!(!graph.is_tip("id:e0"));
        assert!(!graph.is_tip("id:e1"));
    }

    #[test]
    fn fork_produces_two_tips() {
        let mut store = EventStore::new();
        let mut graph = DagGraph::new();

        // E0 ← E1
        //     ↖ E2  (fork: both E1 and E2 reference E0)
        insert_both(&mut store, &mut graph, make_event("id:e0", EventType::StateRoomCreate, vec![]));
        insert_both(&mut store, &mut graph, make_event("id:e1", EventType::MessageText, vec!["id:e0"]));
        insert_both(&mut store, &mut graph, make_event("id:e2", EventType::MessageText, vec!["id:e0"]));

        assert_eq!(graph.tip_count(), 2);
        assert!(graph.is_tip("id:e1"));
        assert!(graph.is_tip("id:e2"));
        assert!(!graph.is_tip("id:e0"));
    }

    #[test]
    fn merge_collapses_fork_to_single_tip() {
        let mut store = EventStore::new();
        let mut graph = DagGraph::new();

        // E0 ← E1 ←┐
        //     ↖ E2 ←┤
        //             E3 (merge: prev_events = [E1, E2])
        insert_both(&mut store, &mut graph, make_event("id:e0", EventType::StateRoomCreate, vec![]));
        insert_both(&mut store, &mut graph, make_event("id:e1", EventType::MessageText, vec!["id:e0"]));
        insert_both(&mut store, &mut graph, make_event("id:e2", EventType::MessageText, vec!["id:e0"]));
        insert_both(&mut store, &mut graph, make_event("id:e3", EventType::MessageText, vec!["id:e1", "id:e2"]));

        assert_eq!(graph.tip_count(), 1);
        assert!(graph.is_tip("id:e3"));
    }

    #[test]
    fn self_reference_rejected() {
        let store = EventStore::new();
        let mut graph = DagGraph::new();
        let ev = make_event("id:e0", EventType::MessageText, vec!["id:e0"]);
        assert!(matches!(graph.add_event(&ev, &store), Err(GraphError::SelfReference)));
    }

    #[test]
    fn unknown_prev_event_rejected() {
        let store = EventStore::new();
        let mut graph = DagGraph::new();
        let ev = make_event("id:e1", EventType::MessageText, vec!["id:e0"]);
        assert!(matches!(
            graph.add_event(&ev, &store),
            Err(GraphError::UnknownPrevEvent(_))
        ));
    }

    #[test]
    fn root_event_with_prev_events_rejected() {
        let mut store = EventStore::new();
        let mut graph = DagGraph::new();
        insert_both(&mut store, &mut graph, make_event("id:e0", EventType::StateRoomCreate, vec![]));
        let bad = make_event("id:e1", EventType::StateRoomCreate, vec!["id:e0"]);
        assert!(matches!(
            graph.add_event(&bad, &store),
            Err(GraphError::RootEventHasPrevEvents(_))
        ));
    }

    #[test]
    fn non_root_without_prev_events_rejected() {
        let store = EventStore::new();
        let mut graph = DagGraph::new();
        let ev = make_event("id:e0", EventType::MessageText, vec![]);
        assert!(matches!(graph.add_event(&ev, &store), Err(GraphError::EmptyPrevEvents)));
    }

    #[test]
    fn too_many_prev_events_rejected() {
        let store = EventStore::new();
        let mut graph = DagGraph::new();
        // The limit check fires before the store-existence check, so no events needed.
        let prev: Vec<&str> = vec![
            "id:p0", "id:p1", "id:p2", "id:p3", "id:p4",
            "id:p5", "id:p6", "id:p7", "id:p8", "id:p9", "id:p10",
        ];
        let ev = make_event("id:e0", EventType::MessageText, prev);
        assert!(matches!(
            graph.add_event(&ev, &store),
            Err(GraphError::TooManyPrevEvents(11))
        ));
    }

    #[test]
    fn missing_event_id_rejected() {
        let store = EventStore::new();
        let mut graph = DagGraph::new();
        let ev = Event::new(
            EventType::StateRoomCreate,
            "s".to_string(), "r".to_string(), "sp".to_string(),
            vec![], "2026-04-27T12:00:00Z".to_string(), serde_json::json!({}),
        );
        assert!(matches!(graph.add_event(&ev, &store), Err(GraphError::MissingEventId)));
    }
}
