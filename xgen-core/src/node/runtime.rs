// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

// NodeRuntime — wires all stateful components together (spec 3.7.11).
//
// Holds the per-Node state that the smoke test drives:
//   - Ed25519 node keypair and node_id
//   - IdentityRegistry (registered Identities on this Node)
//   - Per-Space: SpaceState, EventStore, DagGraph, PendingBuffer
//
// Two insertion modes:
//   ingest_event  — direct DAG insert + SpaceState.apply_event, no 13-step validation.
//                   Used for: history sync, locally produced setup events, replayed events.
//   accept_message — full 13-step pipeline (validate_steps_8_13 + store).
//                   Used for: message.text events from authenticated clients.
//                   Out-of-order events (unknown prev_events) are buffered in PendingBuffer
//                   and re-processed when their predecessors arrive (spec 3.2.5).

use std::collections::HashMap;

use ed25519_dalek::SigningKey;

use crate::{
    crypto::encoding,
    dag::{graph::DagGraph, pending::PendingBuffer, store::EventStore},
    identity::{
        registry::{IdentityRecord, IdentityRegistry, RegistryError},
        replication::ReplicaRegistry,
    },
    message::exchange::{accept_event, ExchangeError},
    space::{dm_promotion::DmProposal, state::SpaceState},
    wire::types::{Event, EventType},
};

pub struct NodeRuntime {
    pub node_keypair: SigningKey,
    pub node_id: String,
    pub identity_registry: IdentityRegistry,
    /// SpaceState per space_id.
    pub spaces: HashMap<String, SpaceState>,
    /// EventStore per space_id.
    pub stores: HashMap<String, EventStore>,
    /// DagGraph per space_id.
    pub graphs: HashMap<String, DagGraph>,
    /// PendingBuffer per space_id — holds events whose prev_events are not yet known.
    pub pending: HashMap<String, PendingBuffer>,
    /// In-flight DM Space promotion proposals — keyed by space_id.
    /// Not persisted; discarded on Node restart or when proposal resolves.
    pub dm_proposals: HashMap<String, DmProposal>,
    /// Tracks which peer nodes hold replicas of Identities owned by this Node.
    /// Not persisted — rebuilt from local state on restart (Phase 2 simplification).
    pub replica_registry: ReplicaRegistry,
}

impl NodeRuntime {
    pub fn new(keypair: SigningKey) -> Self {
        let node_id = format!(
            "xgen://pubkey/ed25519:{}",
            encoding::encode(keypair.verifying_key().as_bytes())
        );
        Self {
            node_keypair: keypair,
            node_id,
            identity_registry: IdentityRegistry::new(),
            spaces: HashMap::new(),
            stores: HashMap::new(),
            graphs: HashMap::new(),
            pending: HashMap::new(),
            dm_proposals: HashMap::new(),
            replica_registry: ReplicaRegistry::new(),
        }
    }

    pub fn register_identity(&mut self, record: IdentityRecord) -> Result<(), RegistryError> {
        self.identity_registry.register(record)
    }

    /// Insert an Event directly into the DAG and apply it to SpaceState.
    /// No 13-step validation — caller is responsible for event correctness.
    pub fn ingest_event(&mut self, event: Event) {
        let space_id = if event.space_id.is_empty() {
            // state.space_create and state.dm_space_create have empty space_id;
            // the event_id becomes the space_id.
            match event.event_id.as_ref() {
                Some(id) => id.clone(),
                None => return, // unsigned event — reject silently
            }
        } else {
            event.space_id.clone()
        };

        self.stores.entry(space_id.clone()).or_insert_with(EventStore::new);
        self.graphs.entry(space_id.clone()).or_insert_with(DagGraph::new);

        let NodeRuntime { spaces, stores, graphs, .. } = self;
        let store = stores.get_mut(&space_id).unwrap();
        let graph = graphs.get_mut(&space_id).unwrap();

        // Insert into DAG (ignore structural errors — e.g., duplicate or out-of-order).
        let _ = graph.add_event(&event, store);
        // Insert into store (ignore duplicate).
        let _ = store.insert(event.clone());

        // Apply to SpaceState.
        match &event.event_type {
            EventType::StateSpaceCreate => {
                if let Ok(mut state) = SpaceState::from_space_create(&event) {
                    // Replay any events already in the store that arrived out of order
                    // (e.g. state.room_create received before state.space_create).
                    let stored: Vec<Event> = store.values().cloned().collect();
                    for ev in topological_sort(stored) {
                        if ev.event_id.as_deref() != event.event_id.as_deref() {
                            let _ = state.apply_event(&ev);
                        }
                    }
                    spaces.insert(state.space_id.clone(), state);
                }
            }
            _ => {
                if let Some(state) = spaces.get_mut(&space_id) {
                    let _ = state.apply_event(&event);
                }
            }
        }
    }

    /// Accept a message Event through the full 13-step validation pipeline.
    /// Stores it in the DAG on success. Does NOT update SpaceState
    /// (message events do not modify Space/Room membership).
    ///
    /// If prev_events reference unknown predecessors (out-of-order federation delivery),
    /// the event is buffered in PendingBuffer and `Err(HeldPending)` is returned.
    /// When the missing predecessors subsequently arrive and are accepted, the buffered
    /// event is automatically re-processed.
    pub fn accept_message(
        &mut self,
        space_id: &str,
        event: Event,
    ) -> Result<(), ExchangeError> {
        self.stores.entry(space_id.to_string()).or_insert_with(EventStore::new);
        self.graphs.entry(space_id.to_string()).or_insert_with(DagGraph::new);

        let event_id = event.event_id.clone();

        let result = {
            let NodeRuntime { spaces, stores, graphs, identity_registry, .. } = self;
            let space = spaces
                .get(space_id)
                .ok_or_else(|| ExchangeError::DagError("space not found".to_string()))?;
            let store = stores.get_mut(space_id).unwrap();
            let graph = graphs.get_mut(space_id).unwrap();
            accept_event(event.clone(), space, identity_registry, store, graph)
        };

        match result {
            Ok(()) => {
                if let Some(eid) = event_id.as_deref() {
                    self.drain_pending_messages(space_id, eid);
                }
                Ok(())
            }
            Err(ExchangeError::HeldPending(missing)) => {
                self.pending
                    .entry(space_id.to_string())
                    .or_default()
                    .add(event, &missing);
                Err(ExchangeError::HeldPending(missing))
            }
            Err(e) => Err(e),
        }
    }

    /// Drain events from the pending buffer that were waiting for `resolved_id`.
    /// Each newly accepted event may unblock further pending events (recursive).
    fn drain_pending_messages(&mut self, space_id: &str, resolved_id: &str) {
        let ready = {
            let store = match self.stores.get(space_id) {
                Some(s) => s,
                None => return,
            };
            match self.pending.get_mut(space_id) {
                Some(buf) => buf.resolve(resolved_id, store),
                None => return,
            }
        };

        for ev in ready {
            let ev_id = ev.event_id.clone();
            let accepted = {
                let NodeRuntime { spaces, stores, graphs, identity_registry, .. } = self;
                if let Some(space) = spaces.get(space_id) {
                    let store = stores.get_mut(space_id).unwrap();
                    let graph = graphs.get_mut(space_id).unwrap();
                    accept_event(ev, space, identity_registry, store, graph).is_ok()
                } else {
                    false
                }
            };
            if accepted {
                if let Some(eid) = ev_id.as_deref() {
                    self.drain_pending_messages(space_id, eid);
                }
            }
        }
    }

    /// Return all events for a Space in topological (causal) order.
    /// Roots (empty prev_events) first; every event follows all its predecessors.
    pub fn all_events(&self, space_id: &str) -> Vec<Event> {
        let store = match self.stores.get(space_id) {
            Some(s) => s,
            None => return vec![],
        };
        topological_sort(store.values().cloned().collect())
    }

    /// Return current DAG tips for a Space.
    pub fn dag_tips(&self, space_id: &str) -> Vec<String> {
        self.graphs
            .get(space_id)
            .map(|g| g.current_tips())
            .unwrap_or_default()
    }
}

/// Kahn's topological sort: returns events in causal order (roots first).
/// Events whose predecessors are not in the set are treated as roots.
fn topological_sort(events: Vec<Event>) -> Vec<Event> {
    use std::collections::{HashMap, VecDeque};

    let by_id: HashMap<String, Event> = events
        .into_iter()
        .filter_map(|e| e.event_id.clone().map(|id| (id, e)))
        .collect();

    // Count predecessors that are within this set (in-degree).
    let mut in_degree: HashMap<&str, usize> = HashMap::new();
    let mut successors: HashMap<&str, Vec<&str>> = HashMap::new();

    for (id, ev) in &by_id {
        in_degree.entry(id.as_str()).or_insert(0);
        for prev in &ev.prev_events {
            if by_id.contains_key(prev.as_str()) {
                *in_degree.entry(id.as_str()).or_insert(0) += 1;
                successors.entry(prev.as_str()).or_default().push(id.as_str());
            }
        }
    }

    let mut queue: VecDeque<&str> = in_degree
        .iter()
        .filter(|(_, &d)| d == 0)
        .map(|(id, _)| *id)
        .collect();
    // Stable ordering within the same level.
    let mut queue_vec: Vec<&str> = queue.drain(..).collect();
    queue_vec.sort();
    queue.extend(queue_vec);

    let mut result = Vec::new();
    while let Some(id) = queue.pop_front() {
        result.push(by_id[id].clone());
        if let Some(deps) = successors.get(id) {
            let mut next: Vec<&str> = deps
                .iter()
                .copied()
                .filter(|dep| {
                    let d = in_degree.get_mut(dep).unwrap();
                    *d -= 1;
                    *d == 0
                })
                .collect();
            next.sort();
            queue.extend(next);
        }
    }

    result
}
