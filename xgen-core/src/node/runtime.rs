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
    message::exchange::{
        accept_event, check_ai_capability, check_ai_operator_targets_pub, check_permission_pub,
        validate_event, ExchangeError, ValidationOutcome,
    },
    space::{dm_promotion::DmProposal, state::SpaceState},
    wire::types::{Event, EventType},
};

/// Outcome of `NodeRuntime::dispatch_event` — the F-4 unified post-validation
/// pipeline. Replaces the pre-F-4 three-way path branching in `process_inbound`
/// (audit §3.2, design doc §7).
///
/// - `Accepted` — event validated, semantic checks passed, ingested into DAG +
///   SpaceState. `new_joiner` is `Some(identity_id)` when this event was a
///   `membership.join` that added a new Space member (the caller pushes the
///   Space's history to the joiner).
/// - `HeldPending` — event buffered with missing predecessors; will be
///   re-dispatched when those events arrive, or discarded after F-4a's 30 s
///   timeout (Ch3 §3.9.6, error 4002).
/// - `Rejected` — event failed structural / semantic validation. Caller logs
///   and drops. M6 (new) Phase 2 wires the wire-layer rejection signal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DispatchOutcome {
    Accepted { new_joiner: Option<String> },
    HeldPending,
    Rejected(String),
}

/// F-5 origin gating annotation (Phase 4, runbook §3.4.1 Q1 lock).
///
/// Runtime metadata about where this Node observed an event in its in-process
/// flow. Wire-invisible — events on the wire carry no origin field; this enum
/// flows alongside the `Event` through the in-process dispatcher. Used by
/// `apply_federation_push` to short-circuit anti-transitive forwarding per
/// design doc §8.5 (events received via federation MUST NOT be re-pushed to
/// other federation peers).
///
/// Lives in `xgen-core::node::runtime` next to `DispatchOutcome` because it is
/// runtime metadata about the dispatcher's input, not wire metadata. Putting
/// it on `xgen-common::wire::Event` (with `#[serde(skip)]`) would hide the
/// runtime annotation on a wire-shape struct — the failure mode D-069 names.
///
/// Forward-compatible with future variants: `ReceivedViaAdminInjection`
/// (M6 Node admin write path) and `ReceivedViaBackfill` (hypothetical future
/// replay tooling) are anticipated but not added until they have a caller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventOrigin {
    /// Event arrived via a client connection — the home Node is the originator
    /// for federation-push purposes. Locally-submitted events are the only
    /// events that may enter `apply_federation_push`.
    LocallySubmitted,
    /// Event arrived via a federation peer session — another Node is the
    /// originator. Anti-transitivity (F-5 §8.5): this Node MUST NOT push it
    /// onward to other federation peers.
    ReceivedViaFederation,
}

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
    /// WebSocket endpoint URLs of known peer Nodes: node_id → ws[s]:// URL.
    /// Populated when a federation handshake is received with node_endpoint set.
    /// Used to push identity replication to peers after registration.
    pub peer_urls: HashMap<String, String>,
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
            peer_urls: HashMap::new(),
        }
    }

    /// Record (or update) the WebSocket endpoint URL for a known peer Node.
    pub fn record_peer_url(&mut self, node_id: &str, url: String) {
        self.peer_urls.insert(node_id.to_string(), url);
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

    // ── F-4 unified dispatcher (xgen_federation_propagation_design §7) ────

    /// Dispatch an inbound Event through the F-4 unified pipeline.
    ///
    /// Replaces the pre-F-4 three-path branching (audit §3.2) where messages
    /// went through `accept_message` (full pipeline) and membership.join /
    /// other state events went through `ingest_event` directly (skipping
    /// signature verification + HeldPending). After F-4, every event family
    /// reaches event-handling code only via this function.
    ///
    /// Pipeline shape, per design doc §7.7:
    ///   1. Structural pre-check (`space_present`). Caller asserts the
    ///      Space context exists (or the event is a Space-creation root).
    ///      This is the cheap fail-fast that avoids wasting crypto.
    ///   2. Federation-relationship check — placeholder for Phase 7 (F-3
    ///      second check); always passes today for non-federation channels.
    ///   3. Validation core (`validate_event`): signature, timestamp,
    ///      predecessor presence with HeldPending on miss, DAG structure,
    ///      sender registration + membership.
    ///   4. Semantic pre-checks: AI role violation (3041), AI operator
    ///      target/permission (3041), AI capability (3042).
    ///   5. Per-event-type handler (ingest + state-machine apply via
    ///      `ingest_event`). `new_joiner` detection for `MembershipJoin`.
    ///   6. Drain the pending buffer for events that this one just
    ///      unblocked; each unblocked event re-enters `dispatch_event`.
    ///
    /// Returns the `DispatchOutcome` so the caller (`process_inbound`) can
    /// build the `FanoutRequest` (local fan-out) and gate the federation-push
    /// side-effect (Phase 4, runbook §3.4.1 Q1 lock).
    ///
    /// `origin` is runtime metadata about where this Node observed the event
    /// (client connection vs federation peer session). The validation core is
    /// origin-uniform — `origin` is unused inside `dispatch_event` itself and
    /// flows through for signature transparency: a future contributor reading
    /// `dispatch_event(event, origin)` sees the in-process annotation as a
    /// first-class concern of the dispatcher, rather than buried at the
    /// caller's apply-federation-push site. Phase 4 uses `origin` only at
    /// `apply_federation_push`'s anti-transitivity guard (F-5 §8.5).
    pub fn dispatch_event(&mut self, event: Event, origin: EventOrigin) -> DispatchOutcome {
        // `origin` is reserved for future origin-aware validation extensions
        // (e.g. Phase 7's F-3 federation-relationship check may want to
        // consult origin when the receiver-side gate fires). Phase 4 does not
        // consume it inside validation; the parameter exists for the caller's
        // signature-transparency benefit (Q1 lock).
        let _ = origin;

        // Resolve the effective space_id. State-create events carry empty
        // space_id on the wire; their own event_id becomes the space_id.
        let space_id = if event.space_id.is_empty() {
            match event.event_id.as_deref() {
                Some(id) => id.to_string(),
                None => {
                    return DispatchOutcome::Rejected("event missing event_id".to_string());
                }
            }
        } else {
            event.space_id.clone()
        };

        let is_space_creation = matches!(
            event.event_type,
            EventType::StateSpaceCreate | EventType::StateDmSpaceCreate
        );

        // Step 1 — Structural pre-check. Non-create events targeting an
        // unknown Space fail fast (cheap HashMap lookup) before validation.
        if !is_space_creation && !self.spaces.contains_key(&space_id) {
            return DispatchOutcome::Rejected(format!("space not found: {space_id}"));
        }

        // Step 2 — Federation-relationship check (F-3 second check) lives
        // here. Phase 7 fills in the real lookup against the federation
        // registry; Phase 2 leaves a structural seam.

        // Step 3 — Validation core (uniform across all event families).
        self.stores
            .entry(space_id.clone())
            .or_insert_with(EventStore::new);
        self.graphs
            .entry(space_id.clone())
            .or_insert_with(DagGraph::new);

        let outcome = {
            let NodeRuntime {
                spaces,
                stores,
                identity_registry,
                ..
            } = self;
            let space = if is_space_creation {
                None
            } else {
                spaces.get(&space_id)
            };
            let store = stores.get(&space_id).unwrap();
            validate_event(&event, space, identity_registry, store)
        };

        match outcome {
            ValidationOutcome::Rejected(err) => {
                return DispatchOutcome::Rejected(err.to_string());
            }
            ValidationOutcome::HeldPending(missing) => {
                self.pending
                    .entry(space_id)
                    .or_default()
                    .add(event, &missing);
                return DispatchOutcome::HeldPending;
            }
            ValidationOutcome::Validated => {}
        }

        // Step 4 — Semantic pre-checks (post-validation, per design doc §7.6).
        // AI role violation: AI senders cannot create Spaces (M3, 3041).
        if is_space_creation {
            if let Some(record) = self.identity_registry.get(&event.sender) {
                if record.is_ai {
                    return DispatchOutcome::Rejected(format!(
                        "ai_role_violation: {} from AI sender",
                        event.event_type.as_str()
                    ));
                }
            }
        }
        // AI capability check (3042) — applies to validated events from AI
        // senders. For human senders the function is a no-op.
        if let Err(e) = check_ai_capability(&event, &self.identity_registry) {
            return DispatchOutcome::Rejected(e.to_string());
        }
        // AI operator target + signer check for delegate / revoke (3041).
        if matches!(
            event.event_type,
            EventType::StateAiOperatorDelegate | EventType::StateAiOperatorRevoke
        ) {
            if let Some(space) = self.spaces.get(&space_id) {
                if let Err(e) =
                    check_ai_operator_targets_pub(&event, space, &self.identity_registry)
                {
                    return DispatchOutcome::Rejected(e.to_string());
                }
                if let Err(e) = check_permission_pub(&event, space) {
                    return DispatchOutcome::Rejected(e.to_string());
                }
            }
        }

        // Step 5 — Per-event-type post-validation handler. Detect new joiner
        // before ingest (the membership.join event itself makes the joiner
        // a member, so detection has to look at pre-ingest state).
        let new_joiner = if matches!(event.event_type, EventType::MembershipJoin) {
            let already_member = self
                .spaces
                .get(&space_id)
                .map(|s| s.is_member(&event.sender))
                .unwrap_or(false);
            if !already_member {
                Some(event.sender.clone())
            } else {
                None
            }
        } else {
            None
        };

        let event_id = event.event_id.clone();
        self.ingest_event(event);

        // Step 6 — Drain pending events whose missing predecessor just
        // arrived. F-4: pending now contains events of any family, not
        // just messages.
        if let Some(eid) = event_id.as_deref() {
            self.drain_pending_uniform(&space_id, eid, origin);
        }

        DispatchOutcome::Accepted { new_joiner }
    }

    /// Drain events from the pending buffer that were waiting for
    /// `resolved_id`. Each unblocked event is re-dispatched through the
    /// full F-4 pipeline (validation + semantic + ingest), so events of
    /// any family — not just messages — recover correctly from out-of-order
    /// delivery.
    ///
    /// Recursive: each newly-ingested event may unblock further events.
    /// Bounded by the depth of the DAG (and by the 30s timeout that
    /// eventually discards stragglers per F-4a).
    ///
    /// Phase 4: drained events inherit the triggering event's `origin`. This
    /// is semantically inexact — a buffered event's true origin is whatever
    /// path it arrived on, which `PendingBuffer` does not store. Acceptable
    /// for Phase 4 because drained events do not surface to
    /// `apply_federation_push` (they're invisible to `process_inbound` —
    /// only the triggering event's outcome bubbles up). If a future phase
    /// needs accurate origin tracking on drained events, `PendingBuffer`
    /// gains an origin field per entry.
    fn drain_pending_uniform(&mut self, space_id: &str, resolved_id: &str, origin: EventOrigin) {
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
            // Re-dispatch through the full pipeline. Validation should now
            // succeed (predecessor present); semantic checks re-run since
            // they may depend on freshly-updated SpaceState.
            //
            // Outcomes other than Accepted are logged via the caller;
            // dispatch_event itself recursively handles further unblocking.
            let _ = self.dispatch_event(ev, origin);
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
