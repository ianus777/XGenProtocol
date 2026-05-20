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

use chrono::{SecondsFormat, Utc};
use ed25519_dalek::SigningKey;
use xgen_common::space_local::SpaceLocalMetadata;
use xgen_common::{NodeXgid, Xgid};

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
    /// Phase 7.5 §5.3 + §5.6 — local-only per-Space provenance metadata.
    /// Sibling to SpaceState (NOT a field on it — preserves SpaceState's
    /// "all content derived from federated events" invariant). Populated
    /// ONCE at Space-create ingestion (federation: introducer = peer;
    /// local: introducer = None); idempotent on duplicate Space-create
    /// arrivals (HashMap::entry-or-insert semantics). Persisted by
    /// xgen-node to `xgen-node_space_local_metadata.json`.
    pub space_local_metadata: HashMap<String, SpaceLocalMetadata>,
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
            space_local_metadata: HashMap::new(),
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

        self.stores.entry(space_id.clone()).or_default();
        self.graphs.entry(space_id.clone()).or_default();

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
        self.stores.entry(space_id.to_string()).or_default();
        self.graphs.entry(space_id.to_string()).or_default();

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
                // Legacy `accept_message` path (test-only-reachable post-F-4
                // per runbook §3.6.1 Step 1 verification — only smoke.rs
                // calls accept_message). Phase 6 preserves None for
                // `missing_identity` here: the legacy ExchangeError shape
                // doesn't carry identity-missing semantics, and updating
                // it would propagate into many test fixtures for no
                // production benefit.
                self.pending
                    .entry(space_id.to_string())
                    .or_default()
                    .add(event, &missing, None, None);
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
            let NodeRuntime { pending, identity_registry, .. } = self;
            match pending.get_mut(space_id) {
                Some(buf) => buf.resolve(resolved_id, store, identity_registry),
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
    pub fn dispatch_event(
        &mut self,
        event: Event,
        origin: EventOrigin,
        peer_node_id: Option<&str>,
    ) -> DispatchOutcome {
        // `origin` flows through for caller-visible signature transparency
        // (Phase 4 Q1 lock). Phase 7's F-3 federation-relationship check at
        // step 2 below consults `peer_node_id` (Phase 7 Lock C1, runbook
        // §3.7.1) — federation-channel events arrive with `Some(peer)`,
        // locally-submitted events arrive with `None`.
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
        //
        // Phase 7.5 §5 — F-4 step 1 skip for Space-create EventTypes.
        // state.space_create and state.dm_space_create create the Space they
        // reference; the Space-exists check cannot apply to them. The skip
        // is narrower than the F-3 skip below — it does NOT extend to
        // state.federation_add, which still requires the target Space to
        // exist locally (the federation_add-arrives-before-space_create case
        // is handled by HeldPending in Phase 7.5 §6 — the third trigger
        // added on top of F-4a's predecessor trigger and F-10's Identity
        // trigger).
        if !is_space_creation && !self.spaces.contains_key(&space_id) {
            return DispatchOutcome::Rejected(format!("space not found: {space_id}"));
        }

        // Step 2 — Federation-relationship check (F-3 second check, Phase 7
        // runbook §3.7.1 Lock A1 + Lock B1). Runs only for federation-
        // channel events (peer_node_id is Some); locally-submitted events
        // skip this check. The lookup consults `SpaceState.federation_nodes`
        // — same source Phase 4's `apply_federation_push` uses on the
        // outbound side (Phase 4 §3.4.1 Q2 "single source of truth"). F-3
        // is the inbound symmetric check; both directions must read the
        // same source.
        if let Some(peer) = peer_node_id {
            // Lock B1 (runbook §3.7.1) — state.federation_add arriving over a federation
            // session is itself the relationship-establishing event. Skipping the F-3
            // check is what lets the relationship bootstrap; the session-level handshake
            // auth (peer Node-keypair) + the event-level signature (same keypair) cover
            // the relevant authority claims. Not narrowing to "sender == wire-authenticated
            // peer == federation_add.adds_node" — that's B2, explicitly NOT done here.
            // If a future threat model justifies B2, it layers on top of B1 cleanly.
            //
            // Phase 7.5 §5 — F-3 skip extension for Space-create EventTypes.
            // state.space_create and state.dm_space_create by structural necessity
            // bring the Space into existence; SpaceState.federation_nodes[space]
            // cannot exist yet (no SpaceState yet). Sibling to Lock B1 above.
            // Signature verification is NOT skipped — only the structural
            // federation-relationship check is skipped; unknown-signer case is
            // covered by F-10 HeldPending. Skip is narrow: state.room_create
            // (also a DAG root, but referencing an existing Space) is NOT
            // included — if the parent Space doesn't exist locally, room_create
            // SHOULD be rejected (the discriminator is "creates the Space it
            // references", not "DAG root").
            let skip_f3 = matches!(
                event.event_type,
                EventType::StateFederationAdd
                    | EventType::StateSpaceCreate
                    | EventType::StateDmSpaceCreate
            );
            if !skip_f3 {
                let relationship_ok = self
                    .spaces
                    .get(&space_id)
                    .map(|s| s.federation_nodes.iter().any(|n| n == peer))
                    .unwrap_or(false);
                if !relationship_ok {
                    let event_id_for_log =
                        event.event_id.as_deref().unwrap_or("(none)").to_string();
                    // Phase 7.5 §6 — Held-not-bypassed posture. When the
                    // peer is not yet in SpaceState.federation_nodes for the
                    // target Space, the event is deferred via HeldPending on
                    // the federation-relationship trigger (third trigger,
                    // sibling to F-4a predecessor and F-10 Identity). Resolved
                    // by an idempotent state.federation_add arrival hook
                    // (`drain_pending_by_federation_relationship` below); on
                    // timeout (default 180s — `[sync].federation_relationship_timeout_seconds`,
                    // Phase 7.5 §7), the timeout sweep emits 4007
                    // federation_relationship_timeout per §6.3 precedence.
                    //
                    // F-3 is not weakened — it is deferred until its data
                    // source (federation_nodes) is populated. The buffer is
                    // a holding cell, not a back-channel: the event is not
                    // accepted into storage, not fanned out, not visible
                    // downstream until F-3 passes on re-validation.
                    //
                    // Phase 9 G2: stable trace event for F-3 reject. Phase
                    // 7.5 §8.2 retains the name `f3_reject` and adds a
                    // disposition field (`held_pending` for this Phase 7.5
                    // path; the historical `rejected` value is reserved for
                    // potential future permanent-reject paths and is not
                    // emitted under Phase 7.5 v1).
                    tracing::warn!(
                        event = "f3_reject",
                        peer_node_id = %peer,
                        space_id = %space_id,
                        event_id = %event_id_for_log,
                        reason = "federation_relationship_missing",
                        disposition = "held_pending",
                        "F-3 federation-relationship gate deferred inbound event via HeldPending"
                    );
                    self.pending
                        .entry(space_id.clone())
                        .or_default()
                        .add(
                            event,
                            &[],
                            None,
                            Some((peer.to_string(), space_id.clone())),
                        );
                    return DispatchOutcome::HeldPending;
                }
            }
        }

        // Step 3 — Validation core (uniform across all event families).
        self.stores
            .entry(space_id.clone())
            .or_default();
        self.graphs
            .entry(space_id.clone())
            .or_default();

        // Phase 7 B3 (locked 2026-05-20) — federation_add events arriving via
        // a federation channel skip step 9 (predecessor presence), step 11
        // (sender registration + sender membership), and step 13 (sender
        // permission). The flag is set only when the channel is federation
        // (peer_node_id.is_some()) AND the event type is StateFederationAdd;
        // locally-submitted federation_add retains full validation. See B3
        // amendment §4.1 + §4.2 for the full lock text and reasoning.
        let fed_add_via_federation = peer_node_id.is_some()
            && matches!(event.event_type, EventType::StateFederationAdd);

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
            validate_event(&event, space, identity_registry, store, fed_add_via_federation)
        };

        match outcome {
            ValidationOutcome::Rejected(err) => {
                let event_id_for_log =
                    event.event_id.as_deref().unwrap_or("(none)").to_string();
                // Phase 9 G2: stable trace event for F-4 validation rejection.
                // Distinct from `event_rejected` (the wrapper at app.rs) and
                // `f3_reject` (federation-relationship gate above) so tests
                // can target validation-core failures specifically.
                tracing::warn!(
                    event = "validation_reject",
                    space_id = %space_id,
                    event_id = %event_id_for_log,
                    reason = %err,
                    "F-4 validation core rejected event"
                );
                return DispatchOutcome::Rejected(err.to_string());
            }
            ValidationOutcome::HeldPending { missing_predecessors, missing_identity } => {
                self.pending
                    .entry(space_id)
                    .or_default()
                    .add(
                        event,
                        &missing_predecessors,
                        missing_identity.as_deref(),
                        None,
                    );
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

        // Phase 7.5 §5.3 + §5.6 — capture local-only Space provenance once,
        // before ingest. `entry().or_insert_with()` makes this idempotent:
        // duplicate state.space_create / state.dm_space_create events for
        // the same effective space_id leave the first introducer intact.
        // The field is populated only when origin == ReceivedViaFederation
        // AND peer_node_id is Some (the wire-authenticated federation peer);
        // locally-submitted Space-creates and federation drains with
        // peer_node_id == None leave introducer = None.
        if is_space_creation {
            let introduced_at = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
            let metadata = match (origin, peer_node_id) {
                (EventOrigin::ReceivedViaFederation, Some(peer)) => {
                    // XGID Adoption v1 Commit 2 — wrap the wire-authenticated
                    // federation peer ID (currently flowing through
                    // dispatch_event as `Option<&str>`) into the v1 typed
                    // NodeXgid flavour at the type-boundary entry point.
                    // Retrofit Pass 3 (xgen-node retype) will widen
                    // dispatch_event's `peer_node_id` parameter from
                    // `Option<&str>` to `Option<&NodeXgid>`, at which point
                    // this wrap collapses into a borrow.
                    let introducer = NodeXgid::from_xgid(Xgid::new(peer.to_string()));
                    SpaceLocalMetadata::new_via_federation(
                        space_id.clone(),
                        introducer,
                        introduced_at,
                    )
                }
                _ => SpaceLocalMetadata::new_local(space_id.clone(), introduced_at),
            };
            self.space_local_metadata
                .entry(space_id.clone())
                .or_insert(metadata);
        }

        let event_id = event.event_id.clone();
        // Phase 7.5 §6 — capture federation_add metadata before ingest_event
        // consumes the event. On successful state.federation_add ingestion,
        // fire the federation-relationship arrival hook (drain dependent
        // events HeldPending on the third trigger). Inside dispatch_event
        // so every caller — production process_inbound, test direct
        // dispatch, future M6 admin write-path — fires the hook uniformly.
        // Mirror of Phase 6's Identity-arrival hook architecture but lifted
        // from xgen-node::app into the dispatcher so the lock is intrinsic.
        let fed_add_drain_pair: Option<(String, String)> =
            if matches!(event.event_type, EventType::StateFederationAdd) {
                event
                    .content
                    .get("node_id")
                    .and_then(|v| v.as_str())
                    .map(|peer| (peer.to_string(), space_id.clone()))
            } else {
                None
            };

        self.ingest_event(event);

        // Step 6 — Drain pending events whose missing predecessor just
        // arrived. F-4: pending now contains events of any family, not
        // just messages.
        if let Some(eid) = event_id.as_deref() {
            self.drain_pending_uniform(&space_id, eid, origin);
        }

        // Step 7 — Phase 7.5 §6 federation-relationship arrival hook.
        // Idempotent: fires on every successful federation_add ingestion;
        // no-op when no entries are buffered on the (peer, space) pair.
        if let Some((peer, sp)) = fed_add_drain_pair {
            self.drain_pending_by_federation_relationship(&peer, &sp, origin);
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
            let NodeRuntime { pending, identity_registry, .. } = self;
            match pending.get_mut(space_id) {
                Some(buf) => buf.resolve(resolved_id, store, identity_registry),
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
            //
            // Phase 7 (F-3): peer_node_id is passed as None at drain time —
            // PendingBuffer does not store the originating peer_node_id per
            // buffered event (same approximation as Phase 4's origin field).
            // F-3 re-check is therefore skipped on drain; a buffered
            // federation event whose peer relationship was torn down within
            // the 30 s HeldPending window slips through. Hazard is narrow
            // (operator-driven defederation rare, window short); future
            // tightening is the same shape Phase 4 anticipated for origin
            // tracking — BufferedEntry gains a peer_node_id field per entry.
            let _ = self.dispatch_event(ev, origin, None);
        }
    }

    /// Phase 6 / F-10 — drain events buffered pending Identity-record
    /// arrival. Called from `xgen-node/src/app.rs::handle_identity_replicate_msg`
    /// after a successful `handle_incoming_replicate` (the only production
    /// path by which an unknown signer becomes known to this Node — per the
    /// Phase 6 survey, `accept_registration` writes locally-hosted
    /// identities synchronously so no event could be waiting on them).
    ///
    /// Per runbook §3.6.1 Lock A2: cross-Space fan-out is small at
    /// deployment scale (~1-10 Spaces per Node), so the arrival hook
    /// iterates all Spaces' `PendingBuffer`s and asks each to resolve the
    /// arrived identity. Released events re-enter `dispatch_event` through
    /// the same shape as predecessor-arrival drain.
    pub fn drain_pending_by_identity(&mut self, identity_id: &str, origin: EventOrigin) {
        // Collect (space_id, ready_events) under the buffer lock domain
        // first so we can re-dispatch outside it without re-entrant
        // borrows on self.pending.
        let space_ids: Vec<String> = self.pending.keys().cloned().collect();
        let mut all_ready: Vec<Event> = Vec::new();
        for space_id in &space_ids {
            // Each Space has its own store and shares the Node-wide
            // identity_registry. The arrival hook just landed
            // identity_id in id_registry, so the registry passed below
            // already contains it — `try_release` for events with
            // `missing_identity == Some(identity_id)` will see
            // identity_known == true.
            let ready_for_space = {
                let store = match self.stores.get(space_id) {
                    Some(s) => s,
                    None => continue,
                };
                let NodeRuntime { pending, identity_registry, .. } = self;
                match pending.get_mut(space_id) {
                    Some(buf) => buf.resolve_identity(identity_id, store, identity_registry),
                    None => continue,
                }
            };
            all_ready.extend(ready_for_space);
        }
        for ev in all_ready {
            // Same drain approximation as `drain_pending_uniform` — F-3
            // peer_node_id not stored per buffered entry; passing None
            // skips the F-3 re-check on drain.
            let _ = self.dispatch_event(ev, origin, None);
        }
    }

    /// Phase 7.5 §6 — drain events buffered pending federation-relationship
    /// arrival. Called from `xgen-node::app::process_inbound` after a
    /// `state.federation_add` for (peer, space) successfully ingests
    /// locally. Idempotent: fires on every successful ingestion, not only
    /// the first; subsequent fires for the same pair are no-ops because
    /// the secondary index has already been drained (mirror of F-10's
    /// Identity-arrival hook semantics).
    ///
    /// Cross-Space fan-out via iteration over all `pending` keys, same
    /// pattern as `drain_pending_by_identity` (Phase 6 Lock A2): the
    /// resolved (peer, space) pair fan-out is small at deployment scale.
    /// Released events re-enter `dispatch_event` through the same shape
    /// as predecessor-arrival drain — passing `peer_node_id = None` skips
    /// the F-3 re-check on drain (same narrow hazard as predecessor/Identity
    /// drains; bounded by the federation-relationship timeout window).
    pub fn drain_pending_by_federation_relationship(
        &mut self,
        peer_node_id: &str,
        resolved_space_id: &str,
        origin: EventOrigin,
    ) {
        let space_ids: Vec<String> = self.pending.keys().cloned().collect();
        let mut all_ready: Vec<Event> = Vec::new();
        for space_id in &space_ids {
            let ready_for_space = {
                let store = match self.stores.get(space_id) {
                    Some(s) => s,
                    None => continue,
                };
                let NodeRuntime {
                    pending,
                    identity_registry,
                    ..
                } = self;
                match pending.get_mut(space_id) {
                    Some(buf) => buf.resolve_federation_relationship(
                        peer_node_id,
                        resolved_space_id,
                        store,
                        identity_registry,
                    ),
                    None => continue,
                }
            };
            all_ready.extend(ready_for_space);
        }
        for ev in all_ready {
            // Drain approximation: same shape as the other two drain helpers.
            // The drained event passed F-3 in the new world (its (peer, space)
            // is now in federation_nodes by definition — federation_add just
            // ingested) so re-check would pass; we still pass None to keep
            // the drain-path symmetry with the other two hooks.
            let _ = self.dispatch_event(ev, origin, None);
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

#[cfg(test)]
mod phase_7_5_tests {
    //! Phase 7.5 §5 — F-3 + F-4 step 1 skip rules for Space-create EventTypes
    //! and SpaceLocalMetadata population at federation ingestion.
    //!
    //! Sibling unit tests to Phase 7's
    //! `xgen-node/src/tests/federation_relationship_integration.rs`. These
    //! live in `xgen-core` (next to the dispatcher) because the skip rules
    //! and metadata population are dispatcher-internal logic — no transport
    //! scaffolding needed.
    use chrono::{SecondsFormat, Utc};
    use serde_json::json;
    use xgen_common::space_local::SpaceLocalMetadata as _SpaceLocalMetadata;

    use super::{DispatchOutcome, EventOrigin, NodeRuntime};
    use crate::{
        crypto::encoding,
        identity::{keypair, registry::IdentityRecord},
        space::state::{
            build_dm_space_create_event, build_room_create_event, build_space_create_event,
            sign_event,
        },
        wire::types::{Event, EventType},
    };

    fn pubkey_uri(key: &ed25519_dalek::SigningKey) -> String {
        format!(
            "xgen://pubkey/ed25519:{}",
            encoding::encode(key.verifying_key().as_bytes())
        )
    }

    fn make_record(key: &ed25519_dalek::SigningKey, home_node: &str) -> IdentityRecord {
        IdentityRecord {
            identity_id: pubkey_uri(key),
            display_name: None,
            is_ai: false,
            ai_capabilities: None,
            registered_at: "2026-05-20T00:00:00.000Z".to_string(),
            trust_assertion: None,
            devices: vec![],
            home_node: home_node.to_string(),
            update_version: 0,
        }
    }

    fn cold_node_with_registered(alice: &ed25519_dalek::SigningKey) -> NodeRuntime {
        let node_key = keypair::generate();
        let mut node = NodeRuntime::new(node_key);
        node.register_identity(make_record(alice, &node.node_id))
            .unwrap();
        node
    }

    /// F-3 skip: brand-new Node receiving state.space_create from a federation
    /// peer with no prior relationship → not rejected by F-3.
    #[test]
    fn f3_skips_state_space_create_from_federation() {
        let alice = keypair::generate();
        let mut node = cold_node_with_registered(&alice);

        let space_ev = sign_event(
            build_space_create_event(&alice, "test-space", None, 1, &node.node_id),
            &alice,
        );

        let peer_key = keypair::generate();
        let peer_id = pubkey_uri(&peer_key);

        let outcome = node.dispatch_event(space_ev, EventOrigin::ReceivedViaFederation, Some(&peer_id));
        if let DispatchOutcome::Rejected(reason) = &outcome {
            assert!(
                !reason.contains("federation_relationship_missing"),
                "F-3 should skip state.space_create — got rejection: {reason}"
            );
        }
        assert!(
            matches!(outcome, DispatchOutcome::Accepted { .. }),
            "expected Accepted, got {:?}",
            outcome
        );
    }

    /// F-3 skip: same for state.dm_space_create.
    #[test]
    fn f3_skips_state_dm_space_create_from_federation() {
        let alice = keypair::generate();
        let mut node = cold_node_with_registered(&alice);

        let invitee = keypair::generate();
        let invitee_id = pubkey_uri(&invitee);

        let dm_ev = sign_event(
            build_dm_space_create_event(&alice, &invitee_id, &node.node_id),
            &alice,
        );

        let peer_key = keypair::generate();
        let peer_id = pubkey_uri(&peer_key);

        let outcome = node.dispatch_event(dm_ev, EventOrigin::ReceivedViaFederation, Some(&peer_id));
        if let DispatchOutcome::Rejected(reason) = &outcome {
            assert!(
                !reason.contains("federation_relationship_missing"),
                "F-3 should skip state.dm_space_create — got rejection: {reason}"
            );
        }
        assert!(
            matches!(outcome, DispatchOutcome::Accepted { .. }),
            "expected Accepted, got {:?}",
            outcome
        );
    }

    /// Narrowness regression: F-3 still applies to state.room_create from an
    /// unfederated peer. The Phase 7.5 §5 skip is narrow ("creates the Space
    /// it references") and MUST NOT extend to room_create, which references
    /// a parent Space.
    ///
    /// Post-Phase-7.5, an F-3 fail no longer permanent-rejects; it defers
    /// the event via HeldPending on the federation-relationship trigger
    /// (P7.5-B held-not-bypassed posture). The narrowness assertion: the
    /// outcome is HeldPending and the event lands on the federation-
    /// relationship secondary index for (peer, space). It is NOT Accepted
    /// (which would be the wrong outcome — F-3 did its job by deferring).
    #[test]
    fn f3_does_not_skip_state_room_create() {
        let alice = keypair::generate();
        let mut node = cold_node_with_registered(&alice);

        // Pre-ingest a Space so room_create has a valid parent.
        let space_ev = sign_event(
            build_space_create_event(&alice, "test-space", None, 1, &node.node_id),
            &alice,
        );
        let space_id = space_ev.event_id.clone().unwrap();
        node.ingest_event(space_ev);

        let room_ev = sign_event(
            build_room_create_event(&alice, &space_id, "general", None),
            &alice,
        );
        let room_event_id = room_ev.event_id.clone().unwrap();

        let peer_key = keypair::generate();
        let peer_id = pubkey_uri(&peer_key);

        let outcome = node.dispatch_event(room_ev, EventOrigin::ReceivedViaFederation, Some(&peer_id));
        assert!(
            matches!(outcome, DispatchOutcome::HeldPending),
            "expected HeldPending for room_create against unfederated peer; got {:?}",
            outcome
        );
        let buf = node
            .pending
            .get(&space_id)
            .expect("pending buffer must exist for the Space");
        assert!(
            buf.contains(&room_event_id),
            "room_create must be buffered on the federation-relationship trigger"
        );
        assert_eq!(buf.pending_federation_relationship_count(), 1);
    }

    /// F-4 step 1 skip: brand-new Node, no Space yet — state.space_create
    /// arrives and is NOT rejected with "space not found". (The F-4 step 1
    /// skip predates Phase 7.5; this test pins the behavior because the
    /// Phase 7.5 §5 verbatim comment block names it as load-bearing.)
    #[test]
    fn f4_step1_skips_state_space_create_unknown_space() {
        let alice = keypair::generate();
        let mut node = cold_node_with_registered(&alice);

        let space_ev = sign_event(
            build_space_create_event(&alice, "test-space", None, 1, &node.node_id),
            &alice,
        );

        let outcome = node.dispatch_event(space_ev, EventOrigin::LocallySubmitted, None);
        if let DispatchOutcome::Rejected(reason) = &outcome {
            assert!(
                !reason.contains("space not found"),
                "F-4 step 1 should skip state.space_create — got rejection: {reason}"
            );
        }
        assert!(
            matches!(outcome, DispatchOutcome::Accepted { .. }),
            "expected Accepted, got {:?}",
            outcome
        );
    }

    /// F-4 step 1 skip: same for state.dm_space_create.
    #[test]
    fn f4_step1_skips_state_dm_space_create_unknown_space() {
        let alice = keypair::generate();
        let mut node = cold_node_with_registered(&alice);

        let invitee = keypair::generate();
        let invitee_id = pubkey_uri(&invitee);

        let dm_ev = sign_event(
            build_dm_space_create_event(&alice, &invitee_id, &node.node_id),
            &alice,
        );

        let outcome = node.dispatch_event(dm_ev, EventOrigin::LocallySubmitted, None);
        if let DispatchOutcome::Rejected(reason) = &outcome {
            assert!(
                !reason.contains("space not found"),
                "F-4 step 1 should skip state.dm_space_create — got rejection: {reason}"
            );
        }
        assert!(
            matches!(outcome, DispatchOutcome::Accepted { .. }),
            "expected Accepted, got {:?}",
            outcome
        );
    }

    /// Negative: F-4 step 1 still rejects state.federation_add when the
    /// target Space doesn't exist locally. (The federation_add-before-
    /// space_create case is Phase 7.5 §6's HeldPending territory in
    /// Commit 3, not a step-1 skip.)
    #[test]
    fn f4_step1_does_not_skip_state_federation_add_unknown_space() {
        let alice = keypair::generate();
        let mut node = cold_node_with_registered(&alice);

        let unknown_space = "xgen://hash/sha256:unknown_space".to_string();
        let peer_key = keypair::generate();
        let peer_id = pubkey_uri(&peer_key);

        let node_key_clone = node.node_keypair.clone();
        let fed_add = sign_event(
            Event::new(
                EventType::StateFederationAdd,
                pubkey_uri(&node_key_clone),
                String::new(),
                unknown_space.clone(),
                vec![unknown_space.clone()],
                Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
                json!({
                    "node_id": peer_id,
                    "session_id": "xgen://hash/sha256:s",
                    "negotiated_version": "0.1",
                    "negotiated_serialisation": "json",
                }),
            ),
            &node_key_clone,
        );

        let outcome = node.dispatch_event(fed_add, EventOrigin::ReceivedViaFederation, Some(&peer_id));
        match outcome {
            DispatchOutcome::Rejected(reason) => assert!(
                reason.contains("space not found"),
                "expected F-4 step 1 'space not found' for federation_add against unknown Space; got: {reason}"
            ),
            other => panic!("expected Rejected, got {:?}", other),
        }
    }

    /// SpaceLocalMetadata: introducer is populated with the peer Node ID
    /// when state.space_create arrives via federation.
    #[test]
    fn space_local_metadata_populated_on_federation_space_create() {
        let alice = keypair::generate();
        let mut node = cold_node_with_registered(&alice);

        let space_ev = sign_event(
            build_space_create_event(&alice, "fed-space", None, 1, &node.node_id),
            &alice,
        );
        let space_id = space_ev.event_id.clone().unwrap();

        let peer_key = keypair::generate();
        let peer_id = pubkey_uri(&peer_key);

        let outcome = node.dispatch_event(space_ev, EventOrigin::ReceivedViaFederation, Some(&peer_id));
        assert!(matches!(outcome, DispatchOutcome::Accepted { .. }));

        let meta: &_SpaceLocalMetadata = node
            .space_local_metadata
            .get(&space_id)
            .expect("metadata must be present");
        assert_eq!(meta.space_id, space_id);
        // XGID Adoption v1 — read the typed NodeXgid back through its inner
        // URI string for comparison against the &str `peer_id`. Once
        // Retrofit Pass 3 retypes peer-ID call sites onto NodeXgid, the
        // assertion can drop the `.as_str()` projection.
        assert_eq!(
            meta.introducer_node_id.as_ref().map(|n| n.as_str()),
            Some(peer_id.as_str())
        );
        assert!(!meta.introduced_at.is_empty());
    }

    /// SpaceLocalMetadata: introducer is None for locally-submitted Space-create.
    #[test]
    fn space_local_metadata_introducer_none_on_local_space_create() {
        let alice = keypair::generate();
        let mut node = cold_node_with_registered(&alice);

        let space_ev = sign_event(
            build_space_create_event(&alice, "local-space", None, 1, &node.node_id),
            &alice,
        );
        let space_id = space_ev.event_id.clone().unwrap();

        let outcome = node.dispatch_event(space_ev, EventOrigin::LocallySubmitted, None);
        assert!(matches!(outcome, DispatchOutcome::Accepted { .. }));

        let meta = node
            .space_local_metadata
            .get(&space_id)
            .expect("metadata must be present");
        assert!(meta.introducer_node_id.is_none());
    }

    /// SpaceLocalMetadata: a second state.space_create for the same space_id
    /// does NOT update the introducer (idempotent at the ingestion layer).
    /// Models the case where a duplicate Space-create arrives via a different
    /// path after the first one was ingested.
    #[test]
    fn space_local_metadata_immutable_after_create() {
        let alice = keypair::generate();
        let mut node = cold_node_with_registered(&alice);

        let space_ev = sign_event(
            build_space_create_event(&alice, "twice-space", None, 1, &node.node_id),
            &alice,
        );
        let space_id = space_ev.event_id.clone().unwrap();

        let peer_a = keypair::generate();
        let peer_a_id = pubkey_uri(&peer_a);
        let peer_b = keypair::generate();
        let peer_b_id = pubkey_uri(&peer_b);

        let first =
            node.dispatch_event(space_ev.clone(), EventOrigin::ReceivedViaFederation, Some(&peer_a_id));
        assert!(matches!(first, DispatchOutcome::Accepted { .. }));

        // Same event via a different peer — `entry().or_insert()` preserves
        // the first introducer. (`ingest_event`'s existing duplicate-event
        // guard also makes the second dispatch a state-level no-op.)
        let _ = node.dispatch_event(space_ev, EventOrigin::ReceivedViaFederation, Some(&peer_b_id));

        let meta = node.space_local_metadata.get(&space_id).unwrap();
        assert_eq!(
            meta.introducer_node_id.as_ref().map(|n| n.as_str()),
            Some(peer_a_id.as_str()),
            "second arrival via different peer must not overwrite first introducer"
        );
    }

    /// Phase 7.5 §6 — F-3 fail produces HeldPending (held-not-bypassed
    /// posture) instead of permanent Rejected. The event lands on the
    /// federation-relationship secondary index for the (peer, space) pair.
    #[test]
    fn f3_fail_buffers_event_on_federation_relationship_trigger() {
        use crate::space::state::build_room_create_event;
        let alice = keypair::generate();
        let mut node = cold_node_with_registered(&alice);

        // Pre-ingest a Space.
        let space_ev = sign_event(
            build_space_create_event(&alice, "fed-space", None, 1, &node.node_id),
            &alice,
        );
        let space_id = space_ev.event_id.clone().unwrap();
        node.ingest_event(space_ev);

        let room_ev = sign_event(
            build_room_create_event(&alice, &space_id, "general", None),
            &alice,
        );
        let room_event_id = room_ev.event_id.clone().unwrap();

        let peer = keypair::generate();
        let peer_id = pubkey_uri(&peer);

        let outcome =
            node.dispatch_event(room_ev, EventOrigin::ReceivedViaFederation, Some(&peer_id));
        assert!(matches!(outcome, DispatchOutcome::HeldPending));

        let buf = node.pending.get(&space_id).expect("buffer must exist");
        assert!(buf.contains(&room_event_id));
        assert_eq!(buf.pending_federation_relationship_count(), 1);
    }

    /// drain_pending_by_federation_relationship: after a federation_add for
    /// (peer, space) lands, buffered events waiting on that pair re-dispatch
    /// through the unified pipeline. Here we exercise the helper directly
    /// (the production path in xgen-node fires the hook from process_inbound).
    #[test]
    fn drain_pending_by_federation_relationship_drains_buffered_events() {
        use crate::space::state::{build_federation_add_event, build_room_create_event};
        let alice = keypair::generate();
        let mut node = cold_node_with_registered(&alice);

        // Set up a Space owned by Alice.
        let space_ev = sign_event(
            build_space_create_event(&alice, "boot-space", None, 1, &node.node_id),
            &alice,
        );
        let space_id = space_ev.event_id.clone().unwrap();
        node.ingest_event(space_ev);

        // A federation peer pushes a room_create. F-3 buffers it.
        let peer = keypair::generate();
        let peer_id = pubkey_uri(&peer);
        let room_ev = sign_event(
            build_room_create_event(&alice, &space_id, "general", None),
            &alice,
        );
        let room_event_id = room_ev.event_id.clone().unwrap();
        let outcome = node.dispatch_event(
            room_ev,
            EventOrigin::ReceivedViaFederation,
            Some(&peer_id),
        );
        assert!(matches!(outcome, DispatchOutcome::HeldPending));
        assert!(node.pending[&space_id].contains(&room_event_id));

        // Ingest the federation_add directly into the DAG (test-shortcut
        // via ingest_event — bypasses validate_event, same approach Phase 7
        // tests use to set up federation_nodes). The signature/validation
        // path is exercised by the integration tests in Commit 4 at
        // NodeRuntime + dispatch_event level.
        let node_key = node.node_keypair.clone();
        let fed_add = sign_event(
            build_federation_add_event(
                &node_key,
                &space_id,
                node.dag_tips(&space_id),
                &peer_id,
                "xgen://hash/sha256:s",
                "0.1",
                "json",
            ),
            &node_key,
        );
        node.ingest_event(fed_add);
        assert!(
            node.spaces[&space_id]
                .federation_nodes
                .iter()
                .any(|n| n == &peer_id),
            "setup: federation_nodes must include the peer"
        );

        // Fire the arrival hook.
        node.drain_pending_by_federation_relationship(
            &peer_id,
            &space_id,
            EventOrigin::ReceivedViaFederation,
        );

        // Buffered event should now be gone (either accepted on re-dispatch
        // or rejected by downstream validation — both remove from buffer).
        assert!(!node.pending[&space_id].contains(&room_event_id));
    }

    /// drain_pending_by_federation_relationship: idempotent — second fire
    /// for the same pair is a no-op.
    #[test]
    fn drain_pending_by_federation_relationship_idempotent() {
        let alice = keypair::generate();
        let mut node = cold_node_with_registered(&alice);
        let peer = keypair::generate();
        let peer_id = pubkey_uri(&peer);

        // No buffer entries — both calls are no-ops; the second must not panic.
        node.drain_pending_by_federation_relationship(
            &peer_id,
            "xgen://hash/sha256:nothing",
            EventOrigin::ReceivedViaFederation,
        );
        node.drain_pending_by_federation_relationship(
            &peer_id,
            "xgen://hash/sha256:nothing",
            EventOrigin::ReceivedViaFederation,
        );
    }

    // ── Phase 7 B3 amendment tests (locked 2026-05-20) ────────────────────

    use crate::space::state::build_federation_add_event as build_fed_add;

    /// B3: a federation_add arriving via federation channel against an
    /// unknown predecessor (i.e., the predecessor is in HeldPending, not
    /// in the store) is Accepted directly. Without B3 this would HeldPending
    /// on missing predecessor (the predecessor-chain deadlock — B3 §3.1).
    #[test]
    fn b3_federation_add_via_federation_skips_step_9_predecessor() {
        let alice = keypair::generate();
        let mut node = cold_node_with_registered(&alice);

        // Set up a Space.
        let space_ev = sign_event(
            build_space_create_event(&alice, "b3-space", None, 1, &node.node_id),
            &alice,
        );
        let space_id = space_ev.event_id.clone().unwrap();
        node.ingest_event(space_ev);

        // Reference a predecessor that does NOT exist in the store.
        let bogus_predecessor = "xgen://hash/sha256:not_in_store".to_string();

        let peer = keypair::generate();
        let peer_id = pubkey_uri(&peer);
        let node_key = node.node_keypair.clone();
        let fed_add = sign_event(
            build_fed_add(
                &node_key,
                &space_id,
                vec![bogus_predecessor.clone()],
                &peer_id,
                "xgen://hash/sha256:s",
                "0.1",
                "json",
            ),
            &node_key,
        );

        let outcome = node.dispatch_event(
            fed_add,
            EventOrigin::ReceivedViaFederation,
            Some(&peer_id),
        );
        assert!(
            matches!(outcome, DispatchOutcome::Accepted { .. }),
            "B3 should accept federation_add even with unknown predecessor; got {:?}",
            outcome
        );
        // Side-effect: federation_nodes for the Space now includes peer.
        assert!(node.spaces[&space_id]
            .federation_nodes
            .iter()
            .any(|n| n == &peer_id));
    }

    /// B3: a federation_add arriving via federation channel signed by a Node
    /// keypair that is NOT in the IdentityRegistry is Accepted directly.
    /// Without B3 this would HeldPending on missing Identity (Q3-overload
    /// trap — Node URIs are never registered as Identities).
    #[test]
    fn b3_federation_add_via_federation_skips_step_11_first_half() {
        let alice = keypair::generate();
        // node_b's identity_registry contains only Alice; peer's Node URI
        // is NOT registered as an Identity (and there's no production path
        // to ever register it).
        let mut node = cold_node_with_registered(&alice);

        let space_ev = sign_event(
            build_space_create_event(&alice, "b3-space", None, 1, &node.node_id),
            &alice,
        );
        let space_id = space_ev.event_id.clone().unwrap();
        node.ingest_event(space_ev);

        let peer = keypair::generate();
        let peer_id = pubkey_uri(&peer);
        // Use the PEER's keypair (NOT this Node's) to sign the federation_add
        // so the sender field is the peer's Node URI, which is unknown to
        // our identity_registry.
        let fed_add = sign_event(
            build_fed_add(
                &peer,
                &space_id,
                node.dag_tips(&space_id),
                &peer_id,
                "xgen://hash/sha256:s",
                "0.1",
                "json",
            ),
            &peer,
        );

        let outcome = node.dispatch_event(
            fed_add,
            EventOrigin::ReceivedViaFederation,
            Some(&peer_id),
        );
        assert!(
            matches!(outcome, DispatchOutcome::Accepted { .. }),
            "B3 should accept federation_add with unknown signer Identity (Q3-overload); got {:?}",
            outcome
        );
    }

    /// B3: a federation_add arriving via federation channel signed by a
    /// non-member is Accepted. Without B3 this would Reject(NotASpaceMember).
    #[test]
    fn b3_federation_add_via_federation_skips_step_11_membership() {
        let alice = keypair::generate();
        let mut node = cold_node_with_registered(&alice);

        // Pre-ingest a Space with Alice as the only member.
        let space_ev = sign_event(
            build_space_create_event(&alice, "b3-space", None, 1, &node.node_id),
            &alice,
        );
        let space_id = space_ev.event_id.clone().unwrap();
        node.ingest_event(space_ev);

        // federation_add signed by this Node's keypair — sender is this
        // Node's URI, which is NOT a Space member.
        let node_key = node.node_keypair.clone();
        let peer = keypair::generate();
        let peer_id = pubkey_uri(&peer);
        let fed_add = sign_event(
            build_fed_add(
                &node_key,
                &space_id,
                node.dag_tips(&space_id),
                &peer_id,
                "xgen://hash/sha256:s",
                "0.1",
                "json",
            ),
            &node_key,
        );

        let outcome = node.dispatch_event(
            fed_add,
            EventOrigin::ReceivedViaFederation,
            Some(&peer_id),
        );
        assert!(
            matches!(outcome, DispatchOutcome::Accepted { .. }),
            "B3 should accept federation_add with non-member signer; got {:?}",
            outcome
        );
    }

    /// B3 step 12 signature verification IS preserved. A federation_add
    /// arriving via federation channel with a tampered signature is rejected.
    #[test]
    fn b3_federation_add_via_federation_still_verifies_signature() {
        let alice = keypair::generate();
        let mut node = cold_node_with_registered(&alice);

        let space_ev = sign_event(
            build_space_create_event(&alice, "b3-space", None, 1, &node.node_id),
            &alice,
        );
        let space_id = space_ev.event_id.clone().unwrap();
        node.ingest_event(space_ev);

        // Construct federation_add with a corrupted signature.
        let node_key = node.node_keypair.clone();
        let peer = keypair::generate();
        let peer_id = pubkey_uri(&peer);
        let mut fed_add = sign_event(
            build_fed_add(
                &node_key,
                &space_id,
                node.dag_tips(&space_id),
                &peer_id,
                "xgen://hash/sha256:s",
                "0.1",
                "json",
            ),
            &node_key,
        );
        // Mutate the content AFTER signing so canonical-form hash matches
        // the event_id but the signature does not verify. Actually simpler:
        // overwrite signature with a bogus one of the same shape.
        fed_add.signature = Some(
            "ed25519:fakekey:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
                .to_string(),
        );

        let outcome = node.dispatch_event(
            fed_add,
            EventOrigin::ReceivedViaFederation,
            Some(&peer_id),
        );
        // step 12 signature check fires; the exact rejection branch
        // depends on whether the corrupted-signature shape parses as a
        // valid Ed25519 signature first. Either way it is NOT Accepted.
        assert!(
            !matches!(outcome, DispatchOutcome::Accepted { .. }),
            "B3 must NOT accept federation_add with invalid signature; got {:?}",
            outcome
        );
    }

    /// B3 narrowness: a federation_add arriving as LocallySubmitted (M6
    /// admin write-path shape, future) does NOT trip the skip. Full
    /// validation applies.
    #[test]
    fn b3_locally_submitted_federation_add_retains_full_validation() {
        let alice = keypair::generate();
        let mut node = cold_node_with_registered(&alice);

        let space_ev = sign_event(
            build_space_create_event(&alice, "b3-space", None, 1, &node.node_id),
            &alice,
        );
        let space_id = space_ev.event_id.clone().unwrap();
        node.ingest_event(space_ev);

        let node_key = node.node_keypair.clone();
        let peer = keypair::generate();
        let peer_id = pubkey_uri(&peer);
        let fed_add = sign_event(
            build_fed_add(
                &node_key,
                &space_id,
                node.dag_tips(&space_id),
                &peer_id,
                "xgen://hash/sha256:s",
                "0.1",
                "json",
            ),
            &node_key,
        );

        // peer_node_id = None → LocallySubmitted. Full validation runs.
        // The Node URI is not registered as an Identity → step 11 first-half
        // F-10 HeldPending fires (because B3 is narrowly scoped to federation
        // channel and does NOT apply here).
        let outcome = node.dispatch_event(fed_add, EventOrigin::LocallySubmitted, None);
        // Outcome should be HeldPending or Rejected — anything except an
        // unconditional Accept that would imply B3 fired for the local path.
        assert!(
            !matches!(outcome, DispatchOutcome::Accepted { .. }),
            "B3 must NOT apply to LocallySubmitted federation_add; got {:?}",
            outcome
        );
    }
}
