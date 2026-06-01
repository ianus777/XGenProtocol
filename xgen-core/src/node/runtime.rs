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
use xgen_common::xgid::{EventXgid, IdentityXgid, NodeXgid, SpaceXgid, Xgid};

use crate::{
    dag::{graph::DagGraph, pending::PendingBuffer, store::EventStore},
    identity::{
        registry::{IdentityRecord, IdentityRegistry, RegistryError},
        replication::ReplicaRegistry,
    },
    message::exchange::{
        check_ai_capability, check_ai_operator_targets_pub, check_permission_pub,
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
///
///   `additional_persisted` carries events drained from the in-pipeline
///   pending buffer (predecessor / federation-relationship arrival hooks
///   inside `dispatch_event`) whose Accepted outcomes need persistence at
///   the caller's storage site. Phase 7.5 persistence-amendment milestone
///   Q2 (a) return-vector lock (Shape β2 — each drain helper returns
///   `Vec<Event>`; `dispatch_event` aggregates via concatenation;
///   `process_inbound` iterates and persists each one). Layer separation:
///   xgen-core stays I/O-free, the persist authority remains xgen-node's
///   storage-write site. Sibling-shape to how `new_joiner` is detected
///   inside dispatch_event and surfaced for caller-side history-push
///   — both fields carry post-dispatch side-effects that the dispatcher
///   detects but does not itself execute.
///
///   Ordering invariant: events appear in fire-order across the drain
///   helpers `dispatch_event` invokes (predecessor-drain first,
///   federation-relationship-drain second). Callers MUST treat the vec
///   as a sequence to persist in iteration order, not as a set —
///   predecessor events generally precede their successors in the vec,
///   which matters for any caller that processes the vec without going
///   through `persist_event`'s per-event duplicate-guard.
///
///   Per Phase 7.5 persistence-amendment milestone re-walk Y-lock; see
///   `tasks/HANDOFF_PERSISTENCE_AMENDMENT_REWALK.md` §2 + JOURNAL J-108 +
///   DECISIONS.md D-077 (Track 1 landing in parallel session-arc).
/// - `HeldPending` — event buffered with missing predecessors; will be
///   re-dispatched when those events arrive, or discarded after F-4a's 30 s
///   timeout (Ch3 §3.9.6, error 4002).
/// - `Rejected` — event failed structural / semantic validation. Caller logs
///   and drops. M6 (new) Phase 2 wires the wire-layer rejection signal.
// PartialEq/Eq removed under Phase 7.5 persistence-amendment Commit 2a —
// `additional_persisted: Vec<Event>` contains `Event` which does not
// derive PartialEq/Eq (intentional — Events are compared via event_id,
// not field-by-field equality). No production caller compares
// DispatchOutcome by equality; pattern-matching via `matches!` is the
// existing access shape and doesn't require these traits.
#[derive(Debug, Clone)]
pub enum DispatchOutcome {
    Accepted {
        // Pass 2 (J-125, design §2.2 Q2.1) — new_joiner carries the typed
        // IdentityXgid of the joining sender for MembershipJoin events.
        new_joiner: Option<IdentityXgid>,
        additional_persisted: Vec<Event>,
    },
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

// Pass 2 (J-125, design §4.1 Q2.8.c — partial retype rationale).
//
// Surface #2 retypes the Node-identifier surfaces (`node_id`, `peer_urls` keys)
// to typed XGIDs. The six per-Space identifier-keyed HashMaps below (`spaces`,
// `stores`, `graphs`, `pending`, `dm_proposals`, `space_local_metadata`) keep
// `String` keys at Pass 2 and defer the `SpaceXgid`-key retype to Pass 3,
// where xgen-node's call sites for these maps (federation_session, fanout,
// app handlers, replay_spaces_from_dir) are touched substantively. The
// call-site-density heuristic — small-cardinality Node-identifier maps retype
// in their own Pass, large-cardinality per-Space maps defer to the Pass that
// touches their primary call-site crate — is recorded as a candidate D-NNN
// sub-principle per design §4.1 (flagged-not-promoted; three-instance
// threshold opens at Pass 3 milestone close).
pub struct NodeRuntime {
    pub node_keypair: SigningKey,
    // Pass 2 (Surface #2 Q2.5) — Node-identifier typed at the struct boundary.
    pub node_id: NodeXgid,
    pub identity_registry: IdentityRegistry,
    /// SpaceState per space_id. Pass 3 (Surface #1 Q1.1) retypes key to SpaceXgid.
    pub spaces: HashMap<SpaceXgid, SpaceState>,
    /// EventStore per space_id. Pass 3 (Surface #1 Q1.1) retypes key to SpaceXgid.
    pub stores: HashMap<SpaceXgid, EventStore>,
    /// DagGraph per space_id. Pass 3 (Surface #1 Q1.1) retypes key to SpaceXgid.
    pub graphs: HashMap<SpaceXgid, DagGraph>,
    /// PendingBuffer per space_id — holds events whose prev_events are not yet known.
    /// Pass 3 (Surface #1 Q1.1) retypes key to SpaceXgid.
    pub pending: HashMap<SpaceXgid, PendingBuffer>,
    /// In-flight DM Space promotion proposals — keyed by space_id.
    /// Not persisted; discarded on Node restart or when proposal resolves.
    /// Pass 3 (Surface #1 Q1.1) retypes key to SpaceXgid.
    pub dm_proposals: HashMap<SpaceXgid, DmProposal>,
    /// Tracks which peer nodes hold replicas of Identities owned by this Node.
    /// Not persisted — rebuilt from local state on restart (Phase 2 simplification).
    pub replica_registry: ReplicaRegistry,
    /// WebSocket endpoint URLs of known peer Nodes: node_id → ws[s]:// URL.
    /// Populated when a federation handshake is received with node_endpoint set.
    /// Used to push identity replication to peers after registration.
    ///
    /// Pass 2 (Surface #2 Q2.6) — key retypes to `NodeXgid`; the URL value stays
    /// `String` (descriptive-string slot per design principle §3).
    pub peer_urls: HashMap<NodeXgid, String>,
    /// Phase 7.5 §5.3 + §5.6 — local-only per-Space provenance metadata.
    /// Sibling to SpaceState (NOT a field on it — preserves SpaceState's
    /// "all content derived from federated events" invariant). Populated
    /// ONCE at Space-create ingestion (federation: introducer = peer;
    /// local: introducer = None); idempotent on duplicate Space-create
    /// arrivals (HashMap::entry-or-insert semantics). Persisted by
    /// xgen-node to `xgen-node_space_local_metadata.json`.
    /// Pass 3 (Surface #1 Q1.1) retypes key to SpaceXgid.
    pub space_local_metadata: HashMap<SpaceXgid, SpaceLocalMetadata>,
}

/// Resolve an event's effective Space anchor. State-create events carry an
/// empty `space_id` on the wire and anchor on their own `event_id`; every
/// other event carries the Space directly. Returns `None` only for the
/// malformed case of an empty-space_id event that also lacks an `event_id`.
///
/// Single source of this resolution (no-drift per D-067) — `dispatch_event`
/// (F-3 + 2b policy), `apply_federation_push` (outbound policy), and
/// `process_inbound` (inbound policy) all call it rather than re-inlining the
/// empty→event_id rule.
pub fn space_id_of(event: &Event) -> Option<SpaceXgid> {
    if event.space_id.as_str().is_empty() {
        event
            .event_id
            .as_ref()
            .map(|id| SpaceXgid::from_xgid(Xgid::new(id.as_str().to_string())))
    } else {
        Some(event.space_id.clone())
    }
}

impl NodeRuntime {
    pub fn new(keypair: SigningKey) -> Self {
        // Pass 2 (Surface #2 Q2.5) — construct typed NodeXgid via the principal
        // pubkey constructor; the inner Xgid string matches the legacy format.
        let node_id = NodeXgid::from_pubkey(&keypair.verifying_key());
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
        // Pass 2 (Surface #2 Q2.6) — peer_urls keys are typed NodeXgid;
        // construct typed wrapper at the insert-site boundary. The `node_id`
        // parameter stays `&str` (xgen-node-side parallel parameter defers to
        // Pass 3 per design §5.1).
        self.peer_urls
            .insert(NodeXgid::from_xgid(Xgid::new(node_id.to_string())), url);
    }

    pub fn register_identity(&mut self, record: IdentityRecord) -> Result<(), RegistryError> {
        self.identity_registry.register(record)
    }

    /// Insert an Event directly into the DAG and apply it to SpaceState.
    /// No 13-step validation — caller is responsible for event correctness.
    pub fn ingest_event(&mut self, event: Event) {
        // Pass 3 (Surface #1 Q1.1+Q1.3) — NodeRuntime per-space maps keyed by
        // SpaceXgid; the local variable binds as typed reference.
        let space_id: SpaceXgid = if event.space_id.as_str().is_empty() {
            // state.space_create and state.dm_space_create have empty space_id;
            // the event_id becomes the space_id.
            match event.event_id.as_ref() {
                Some(id) => SpaceXgid::from_xgid(Xgid::new(id.as_str().to_string())),
                None => return, // unsigned event — reject silently
            }
        } else {
            event.space_id.clone()
        };

        self.stores.entry(space_id.clone()).or_default();
        self.graphs.entry(space_id.clone()).or_default();

        // D-075 vantage: capture local Node URI before destructuring `self`.
        // Threaded into `apply_event` so `apply_federation_add` can derive
        // the relevant peer per design §4.1.
        // Pass 2 (Surface #2 Q2.5 + Q2.7) — node_id is NodeXgid; project to
        // owned String for SpaceState::apply_event (out of Pass 2 scope per
        // design §5.1 — SpaceState methods take &str, defer to Pass 3).
        let my_node_id: String = self.node_id.as_str().to_string();

        let NodeRuntime { spaces, stores, graphs, .. } = self;
        let store = stores.get_mut(&space_id).unwrap();
        let graph = graphs.get_mut(&space_id).unwrap();

        // Q1(a).iii.α — tracing::error on graph.add_event failure, continue.
        //
        // Phase 7.5 persistence-amendment milestone (J-108). The Q1 lock at
        // design doc §3 was originally (a).iii.β (Result<(), GraphError> at
        // the signature, compiler-forced caller handling) but reverted to
        // (a).iii.α (log-level vigilance) at implementation when the
        // cross-milestone Phase 7 B3 amendment dependency surfaced:
        // state.federation_add events arriving via federation channel
        // intentionally have missing predecessors (B3 §4.1 predecessor-chain
        // deadlock; xgen-core/src/message/exchange.rs:455). The B3 amendment
        // relied on this site's silent-discard as a feature — the federation_add
        // event lands in EventStore + mutates SpaceState.federation_nodes even
        // though graph.add_event returns UnknownPrevEvent. Result-propagation
        // would have broken B3 at the SpaceState mutation layer.
        //
        // FUTURE WORK (candidate D-NNN — "ingest path invariant encoding under
        // bidirectional sustainability discipline"): this site, plus the four
        // other silent-discard sites in ingest_event (event_id-missing-return,
        // store.insert silent, two apply_event silents), plus the three drain
        // helpers' silent-discards, plus any reject paths that swallow event-
        // acceptance failures, all share the same discipline question: under
        // what circumstances may a fallible operation discard its error? The
        // audit must be bidirectional — forward-drift (future callers bypass
        // upstream validation) AND backward-coherence (current callers depend
        // on the silent as a feature). Both questions must be asked at every
        // site simultaneously, because closing any one silent in isolation can
        // break a cross-milestone semantic dependency (B3 at this milestone is
        // the worked example).
        //
        // Scope of the future walk: re-audit ingest_event's five silents +
        // the three drain helpers + the M6 reject paths + B3's apply_event
        // dependency, simultaneously, under the bidirectional sustainability
        // frame. Do NOT close any one silent in isolation. Promotion of
        // candidate D-NNN to D-NNN happens when (a) Joe locks the walk as
        // worth pursuing, OR (b) dependent work (M6 admin write path, M8
        // federation depth, future cold-start refactor) surfaces a concrete
        // drift instance log-level vigilance does not catch.
        //
        // Rungs above (a).iii.α at the design level (recorded for future-walk
        // reference; not promoted at this milestone):
        //   - (a).iii.β — Result<(), GraphError> compiler-forced handling
        //   - ValidatedEvent wrapper — type-constructor discipline
        //   - Sealed traits + visitor pattern — new-caller shape constraint
        //   - Formal verification — machine-checked invariants
        match graph.add_event(&event, store) {
            Ok(()) => {}
            Err(e) => {
                tracing::error!(
                    event = "graph_add_event_failed",
                    space_id = %space_id.as_str(),
                    event_id = %event.event_id.as_ref().map(|e| e.as_str()).unwrap_or("(none)"),
                    error = %e,
                    "graph.add_event returned error; event continues to store + apply_event \
                     per (a).iii.α + Phase 7 B3 amendment (federation_add bootstrap case)"
                );
            }
        }
        // Insert into store (ignore duplicate — out-of-scope per Q1 narrow-scope;
        // candidate D-NNN bidirectional-sustainability future-walk).
        let _ = store.insert(event.clone());

        // Apply to SpaceState.
        match &event.event_type {
            EventType::StateSpaceCreate => {
                if let Ok(mut state) = SpaceState::from_space_create(&event) {
                    // Replay any events already in the store that arrived out of order
                    // (e.g. state.room_create received before state.space_create).
                    let stored: Vec<Event> = store.values().cloned().collect();
                    for ev in topological_sort(stored) {
                        if ev.event_id.as_ref().map(|e| e.as_str())
                            != event.event_id.as_ref().map(|e| e.as_str())
                        {
                            let _ = state.apply_event(&ev, &my_node_id);
                        }
                    }
                    // Pass 3 (Surface #1 Q1.1) — spaces keyed by SpaceXgid;
                    // typed insertion with no String projection.
                    spaces.insert(state.space_id.clone(), state);
                }
            }
            // M7C-D4 / A3 — DM-init on ingest. The node never holds the creator's
            // key, so it uses the key-less `from_dm_space_create_node` (seeds
            // members = {creator} + pending_invites = {invitee} from the root's
            // content; no Room). Membership is carried by THIS root, not the
            // auto-`membership.invite` (which is a no-op-by-reject under DM
            // constraints, 3.16.1 — CP-1 trace, J-219). The separately-arriving
            // auto-`state.room_create` applies through the normal applier. Mirrors
            // the StateSpaceCreate arm, including the out-of-order replay of
            // already-stored child events (the disk-replay safety net; the
            // production 3-event send is root-first per the A3 ordering invariant).
            // Before A3 this fell to the `_` arm, where `spaces.get_mut` returned
            // None and nothing was built (the J-214 catch).
            EventType::StateDmSpaceCreate => {
                if let Ok(mut state) = SpaceState::from_dm_space_create_node(&event) {
                    let stored: Vec<Event> = store.values().cloned().collect();
                    for ev in topological_sort(stored) {
                        if ev.event_id.as_ref().map(|e| e.as_str())
                            != event.event_id.as_ref().map(|e| e.as_str())
                        {
                            let _ = state.apply_event(&ev, &my_node_id);
                        }
                    }
                    spaces.insert(state.space_id.clone(), state);
                }
            }
            _ => {
                if let Some(state) = spaces.get_mut(&space_id) {
                    let _ = state.apply_event(&event, &my_node_id);
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
    ///
    /// **Pass 2 Q5.4 candidate for deprecation-audit arc.** `accept_message` flows
    /// through `accept_event` + `validate_steps_8_13` — both deprecated at Pass 2
    /// (design §4.2 Q5.b). `accept_message` is kept active in Pass 2 because xgen-node
    /// clients call it directly (`xgen-node/src/main.rs`, smoke tests), but may join
    /// the deprecation removal arc per D-071 alongside `validate_steps_8_13` +
    /// `accept_event`. When that arc opens, this function's callers migrate to
    /// `dispatch_event` (the F-4 unified path). Signature retyped at Pass 2 per
    /// Surface #5 Q5.1: `space_id` binds as `&SpaceXgid` at the typed boundary;
    /// SpaceState/store/graph maps still keyed by String (Pass 2 Q2.8.c — defer
    /// per-space map retype to Pass 3), so we project at the HashMap lookup sites.
    pub fn accept_message(
        &mut self,
        space_id: &SpaceXgid,
        event: Event,
    ) -> Result<(), ExchangeError> {
        // Pass 3 (Surface #1 Q1.3+Q1.4) — per-space HashMap keyed by SpaceXgid;
        // typed entry construction at insertion boundary, Borrow<str> lookup
        // would also work but typed-clone keeps the call self-documenting.
        self.stores.entry(space_id.clone()).or_default();
        self.graphs.entry(space_id.clone()).or_default();

        let event_id = event.event_id.clone();

        let result = {
            let NodeRuntime { spaces, stores, graphs, identity_registry, .. } = self;
            let space = spaces
                .get(space_id)
                .ok_or_else(|| ExchangeError::DagError("space not found".to_string()))?;
            let store = stores.get_mut(space_id).unwrap();
            let graph = graphs.get_mut(space_id).unwrap();
            // Q5.3 — `accept_event` is deprecated at Pass 2; accept_message
            // propagates the deprecation as test-only-reachable scope. The
            // removal arc closes both functions together per D-071.
            #[allow(deprecated)]
            crate::message::exchange::accept_event(
                event.clone(),
                space,
                identity_registry,
                store,
                graph,
            )
        };

        match result {
            Ok(()) => {
                if let Some(eid) = event_id.as_ref() {
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
                //
                // Pass 2 (Surface #1 Q2 + Surface #3 Q3.1): `missing` is
                // Vec<EventXgid> (ExchangeError::HeldPending retyped); pass
                // directly to the typed PendingBuffer::add signature.
                self.pending
                    .entry(space_id.clone())
                    .or_default()
                    .add(event, &missing, None, None);
                Err(ExchangeError::HeldPending(missing))
            }
            Err(e) => Err(e),
        }
    }

    /// Drain events from the pending buffer that were waiting for `resolved_id`.
    /// Each newly accepted event may unblock further pending events (recursive).
    fn drain_pending_messages(&mut self, space_id: &SpaceXgid, resolved_id: &EventXgid) {
        // Pass 3 (Surface #1 Q1.4) — internal helper signature retyped to
        // &SpaceXgid / &EventXgid per Pass 2 principle (internal variables
        // bind as typed references). Called from accept_message
        // (deprecated test-only path per Surface #5 Q5.4) + recursively.
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
                    // accept_event is deprecated at Pass 2 (Surface #1 Q5);
                    // drain_pending_messages inherits the deprecation scope —
                    // closes alongside accept_message + accept_event +
                    // validate_steps_8_13 in the D-071 audit-design-impl arc.
                    #[allow(deprecated)]
                    crate::message::exchange::accept_event(
                        ev,
                        space,
                        identity_registry,
                        store,
                        graph,
                    )
                    .is_ok()
                } else {
                    false
                }
            };
            if accepted {
                if let Some(eid) = ev_id.as_ref() {
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
        peer_node_id: Option<&NodeXgid>,
    ) -> DispatchOutcome {
        // `origin` flows through for caller-visible signature transparency
        // (Phase 4 Q1 lock). Phase 7's F-3 federation-relationship check at
        // step 2 below consults `peer_node_id` (Phase 7 Lock C1, runbook
        // §3.7.1) — federation-channel events arrive with `Some(peer)`,
        // locally-submitted events arrive with `None`.
        //
        // Pass 3 (Surface #2 Q2.1) — `peer_node_id` retypes from `Option<&str>`
        // to `Option<&NodeXgid>`; borrowed boundary per Joe-lock Q-B at design
        // walk (parameter never stored — owners pass &NodeXgid they hold;
        // owned would force unnecessary clones).
        let _ = origin;

        // Resolve the effective space_id via the shared `space_id_of` resolver
        // (no-drift per D-067; same resolution used by apply_federation_push +
        // process_inbound). State-create events carry empty space_id on the
        // wire; their own event_id becomes the space_id.
        // Pass 3 (Surface #1 Q1.3) — internal variable binds as typed SpaceXgid.
        let space_id: SpaceXgid = match space_id_of(&event) {
            Some(s) => s,
            None => {
                return DispatchOutcome::Rejected("event missing event_id".to_string());
            }
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
            return DispatchOutcome::Rejected(format!("space not found: {}", space_id.as_str()));
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
                // Pass 3 (Surface #2 Q2.3) — `peer_node_id` is now `&NodeXgid`;
                // typed PartialEq comparison via inner Xgid; projection collapsed.
                let relationship_ok = self
                    .spaces
                    .get(&space_id)
                    .map(|s| s.federation_nodes.iter().any(|n| n == peer))
                    .unwrap_or(false);
                if !relationship_ok {
                    let event_id_for_log = event
                        .event_id
                        .as_ref()
                        .map(|e| e.as_str())
                        .unwrap_or("(none)")
                        .to_string();
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
                        peer_node_id = %peer.as_str(),
                        space_id = %space_id.as_str(),
                        event_id = %event_id_for_log,
                        reason = "federation_relationship_missing",
                        disposition = "held_pending",
                        "F-3 federation-relationship gate deferred inbound event via HeldPending"
                    );
                    // Pass 3 (Surface #2 Q2.3) — typed-clone construction of
                    // PendingBuffer::add fed_key tuple; the Xgid::new wrap
                    // collapsed once peer_node_id retyped to &NodeXgid.
                    let fed_key = (peer.clone(), space_id.clone());
                    self.pending
                        .entry(space_id.clone())
                        .or_default()
                        .add(event, &[], None, Some(fed_key));
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
                let event_id_for_log = event
                    .event_id
                    .as_ref()
                    .map(|e| e.as_str())
                    .unwrap_or("(none)")
                    .to_string();
                // Phase 9 G2: stable trace event for F-4 validation rejection.
                // Distinct from `event_rejected` (the wrapper at app.rs) and
                // `f3_reject` (federation-relationship gate above) so tests
                // can target validation-core failures specifically.
                tracing::warn!(
                    event = "validation_reject",
                    space_id = %space_id.as_str(),
                    event_id = %event_id_for_log,
                    reason = %err,
                    "F-4 validation core rejected event"
                );
                return DispatchOutcome::Rejected(err.to_string());
            }
            ValidationOutcome::HeldPending { missing_predecessors, missing_identity } => {
                // Pass 2 (Surface #1 Q1 + Surface #3 Q3.1) — ValidationOutcome
                // now carries Vec<EventXgid> + Option<IdentityXgid>; PendingBuffer::add
                // takes &[EventXgid] + Option<&IdentityXgid>. Bind missing_identity
                // as &IdentityXgid via .as_ref() (not .as_deref(), which would
                // project through Deref<Target = Xgid>).
                self.pending
                    .entry(space_id)
                    .or_default()
                    .add(
                        event,
                        &missing_predecessors,
                        missing_identity.as_ref(),
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
                .map(|s| s.is_member(event.sender.as_str()))
                .unwrap_or(false);
            if !already_member {
                // Pass 2 (Surface #2 Q2.1) — DispatchOutcome.new_joiner is
                // Option<IdentityXgid>; pass the typed sender clone directly.
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
        //
        // Pass 3 (Surface #2 Q2.3) — the XGID Adoption v1 wrap-at-boundary
        // pattern collapses: `peer` is now `&NodeXgid` directly; clone into
        // the owned-introducer slot. The previous `Xgid::new(peer.to_string())`
        // construction is dead at this site.
        if is_space_creation {
            let introduced_at = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
            let metadata = match (origin, peer_node_id) {
                (EventOrigin::ReceivedViaFederation, Some(peer)) => {
                    SpaceLocalMetadata::new_via_federation(
                        space_id.clone(),
                        peer.clone(),
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
        //
        // D-075 vantage-aware drain key (locked at bidirectional
        // federation_nodes design phase 2026-05-21; sibling derivation to
        // `apply_federation_add` at xgen-core/src/space/state.rs:351 — see
        // that site's verbatim D-075 comment block for the locking
        // walkthrough). Both sites ask the same question — "which peer
        // does this state.federation_add establish a relationship with,
        // from my vantage?" — and MUST derive the same answer; drift between
        // them is a bug. If I am content.node_id (B's vantage on A's
        // event), the relevant peer is event.sender (the asserter A);
        // else, the relevant peer is content.node_id.
        //
        // Load-bearing invariant: this drain key MUST match F-3's third-
        // trigger keying at Step 2 above (the buffer entry's `Some((peer,
        // space))` argument is the wire-authenticated peer = the OTHER
        // party from this Node's view). Drift between F-3's keying and
        // this drain-pair derivation leaves buffered events stranded
        // until 4007 timeout — which is exactly the bug Phase 9 Scenario
        // 1 surfaced after Commit 2's applier-only fix.
        // Pass 3 (Surface #1 Q1.4 + Surface #2 Q2.4) — D-075 vantage derivation
        // binds the drain pair as typed (NodeXgid, SpaceXgid). event.sender is
        // IdentityXgid (Identity URI bytes also serve as Node URI in the
        // principal-flavour space here); wrap into NodeXgid at the boundary
        // because the drain-helper signature consumes &NodeXgid (Q1.4).
        let fed_add_drain_pair: Option<(NodeXgid, SpaceXgid)> =
            if matches!(event.event_type, EventType::StateFederationAdd) {
                event
                    .content
                    .get("node_id")
                    .and_then(|v| v.as_str())
                    .map(|content_node_id| {
                        let peer = if content_node_id == self.node_id.as_str() {
                            NodeXgid::from_xgid(Xgid::new(event.sender.as_str().to_string()))
                        } else {
                            NodeXgid::from_xgid(Xgid::new(content_node_id.to_string()))
                        };
                        (peer, space_id.clone())
                    })
            } else {
                None
            };

        self.ingest_event(event);

        // Phase 7.5 persistence-amendment Q3 aggregation — `drain_pending_uniform`
        // returns the Vec<Event> of Accepted-drained events; collect into the
        // outgoing `additional_persisted` vector. Empty vec when no drains
        // fired (typical case — drain hook is per-resolved-event, fires only
        // on predecessor arrival). Layer separation per Q2 lock (xgen-core
        // stays I/O-free).
        let mut additional_persisted: Vec<Event> = Vec::new();

        // Step 6 — Drain pending events whose missing predecessor just
        // arrived. F-4: pending now contains events of any family, not
        // just messages.
        if let Some(eid) = event_id.as_ref() {
            additional_persisted.extend(self.drain_pending_uniform(&space_id, eid, origin));
        }

        // Step 7 — Phase 7.5 §6 federation-relationship arrival hook.
        // Idempotent: fires on every successful federation_add ingestion;
        // no-op when no entries are buffered on the (peer, space) pair.
        //
        // Phase 7.5 persistence-amendment Q3 aggregation — sibling to the
        // predecessor-drain aggregation above. Both drains write into the
        // same outgoing vector; ordering reflects fire-order (predecessor
        // first, then federation-relationship), which matches the in-pipeline
        // discovery order and is acceptable because `process_inbound`'s
        // persist loop is per-event idempotent (persist_event checks for
        // duplicates by event_id before append) AND `replay_spaces_from_dir`'s
        // Q1(a).ii sort-on-replay defensive layer at xgen-node/src/app.rs:2628
        // handles any DAG-ordering surprises at recovery time (Commit 2
        // already shipped).
        if let Some((peer, sp)) = fed_add_drain_pair {
            additional_persisted
                .extend(self.drain_pending_by_federation_relationship(&peer, &sp, origin));
        }

        DispatchOutcome::Accepted {
            new_joiner,
            additional_persisted,
        }
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
    ///
    /// Phase 7.5 persistence-amendment Q3 return-vector — this helper returns
    /// `Vec<Event>` containing drained events whose `dispatch_event` outcomes
    /// were Accepted. The caller (`dispatch_event` itself) aggregates the
    /// returned vector and surfaces it via `DispatchOutcome::Accepted {
    /// additional_persisted, .. }`. Shape β2 over Shape β1 on five grounds
    /// (runbook §4a.4):
    ///   1. Self-documenting signatures — each function's contract visible at signature
    ///   2. Easier code-review than threaded-accumulator alternative
    ///   3. Bounded recursion depth makes Vec allocation cost negligible at protocol traffic
    ///   4. Sibling-shape to the existing `drain_pending_messages` recursion pattern in this file
    ///   5. Avoids the "outer caller forgets to thread the accumulator" footgun
    ///
    /// Per Phase 7.5 persistence-amendment milestone re-walk Y-lock; see
    /// `tasks/HANDOFF_PERSISTENCE_AMENDMENT_REWALK.md` §2 + JOURNAL J-108 +
    /// DECISIONS.md D-077 (Track 1 landing in parallel session-arc).
    fn drain_pending_uniform(
        &mut self,
        space_id: &SpaceXgid,
        resolved_id: &EventXgid,
        origin: EventOrigin,
    ) -> Vec<Event> {
        // Pass 3 (Surface #1 Q1.4 + Surface #2 Q2.5) — internal helper
        // signature retyped to &SpaceXgid / &EventXgid; typed PendingBuffer
        // calls drop the previous Xgid::new wrap.
        let ready = {
            let store = match self.stores.get(space_id) {
                Some(s) => s,
                None => return Vec::new(),
            };
            let NodeRuntime { pending, identity_registry, .. } = self;
            match pending.get_mut(space_id) {
                Some(buf) => buf.resolve(resolved_id, store, identity_registry),
                None => return Vec::new(),
            }
        };
        let mut drained: Vec<Event> = Vec::new();
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
            //
            // Phase 7.5 persistence-amendment Q2/Q3 — capture the drained
            // event into the outgoing vec on Accepted outcome. The recursive
            // dispatch_event returns its own Accepted's `additional_persisted`
            // for any deeper cascade; Shape β2 flattening means each
            // recursive layer's drained events bubble up into this outer
            // vec, so the top-level caller sees the entire cascade flat.
            let ev_clone = ev.clone();
            match self.dispatch_event(ev, origin, None) {
                DispatchOutcome::Accepted {
                    new_joiner: _,
                    additional_persisted,
                } => {
                    drained.push(ev_clone);
                    drained.extend(additional_persisted);
                }
                DispatchOutcome::HeldPending | DispatchOutcome::Rejected(_) => {}
            }
        }
        drained
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
    ///
    /// Phase 7.5 persistence-amendment Q3 return-vector — returns `Vec<Event>`
    /// of drained events for caller-side persistence. Caller is
    /// `xgen-node/src/app.rs::handle_identity_replicate_msg` (NOT
    /// `dispatch_event` — F-10's arrival hook lives at xgen-node per layer
    /// separation). Shape β2 details + five grounds at `drain_pending_uniform`'s
    /// doc-comment in this file. Per Phase 7.5 persistence-amendment milestone
    /// re-walk Y-lock.
    pub fn drain_pending_by_identity(
        &mut self,
        identity_id: &IdentityXgid,
        origin: EventOrigin,
    ) -> Vec<Event> {
        // Pass 3 (Surface #1 Q1.4 + Surface #2 Q2.5) — internal helper
        // signature retyped to &IdentityXgid; typed PendingBuffer call drops
        // the previous Xgid::new wrap.
        //
        // Collect (space_id, ready_events) under the buffer lock domain
        // first so we can re-dispatch outside it without re-entrant
        // borrows on self.pending.
        let space_ids: Vec<SpaceXgid> = self.pending.keys().cloned().collect();
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
        let mut drained: Vec<Event> = Vec::new();
        for ev in all_ready {
            // Same drain approximation as `drain_pending_uniform` — F-3
            // peer_node_id not stored per buffered entry; passing None
            // skips the F-3 re-check on drain.
            //
            // Phase 7.5 persistence-amendment Q2/Q3 — capture for caller-side
            // persistence; Shape β2 cascade flattening per drain_pending_uniform's
            // doc-comment.
            let ev_clone = ev.clone();
            match self.dispatch_event(ev, origin, None) {
                DispatchOutcome::Accepted {
                    new_joiner: _,
                    additional_persisted,
                } => {
                    drained.push(ev_clone);
                    drained.extend(additional_persisted);
                }
                DispatchOutcome::HeldPending | DispatchOutcome::Rejected(_) => {}
            }
        }
        drained
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
    ///
    /// Phase 7.5 persistence-amendment Q3 return-vector — returns `Vec<Event>`
    /// of drained events for caller-side persistence. Shape β2 details + five
    /// grounds at `drain_pending_uniform`'s doc-comment in this file. Per
    /// Phase 7.5 persistence-amendment milestone re-walk Y-lock.
    pub fn drain_pending_by_federation_relationship(
        &mut self,
        peer_node_id: &NodeXgid,
        resolved_space_id: &SpaceXgid,
        origin: EventOrigin,
    ) -> Vec<Event> {
        // Pass 3 (Surface #1 Q1.4 + Surface #2 Q2.5) — internal helper
        // signature retyped to (&NodeXgid, &SpaceXgid); typed PendingBuffer
        // call drops the previous Xgid::new wraps.
        let space_ids: Vec<SpaceXgid> = self.pending.keys().cloned().collect();
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
        let mut drained: Vec<Event> = Vec::new();
        for ev in all_ready {
            // Drain approximation: same shape as the other two drain helpers.
            // The drained event passed F-3 in the new world (its (peer, space)
            // is now in federation_nodes by definition — federation_add just
            // ingested) so re-check would pass; we still pass None to keep
            // the drain-path symmetry with the other two hooks.
            //
            // Phase 7.5 persistence-amendment Q2/Q3 — capture for caller-side
            // persistence; Shape β2 cascade flattening per drain_pending_uniform's
            // doc-comment.
            let ev_clone = ev.clone();
            match self.dispatch_event(ev, origin, None) {
                DispatchOutcome::Accepted {
                    new_joiner: _,
                    additional_persisted,
                } => {
                    drained.push(ev_clone);
                    drained.extend(additional_persisted);
                }
                DispatchOutcome::HeldPending | DispatchOutcome::Rejected(_) => {}
            }
        }
        drained
    }

    /// Return all events for a Space in topological (causal) order.
    /// Roots (empty prev_events) first; every event follows all its predecessors.
    ///
    /// Pass 3 (Surface #1 Q1.5) — public API parameter retypes to `&SpaceXgid`
    /// per Joe-lock Q-A at design walk (preserves Pass-internal-consistency with
    /// Pass 2's principle; Borrow<str> means call sites holding `&str` continue
    /// to work via projection where needed).
    pub fn all_events(&self, space_id: &SpaceXgid) -> Vec<Event> {
        let store = match self.stores.get(space_id) {
            Some(s) => s,
            None => return vec![],
        };
        topological_sort(store.values().cloned().collect())
    }

    /// Return current DAG tips for a Space.
    ///
    /// Pass 3 (Surface #1 Q1.5) — public API parameter retypes to `&SpaceXgid`.
    pub fn dag_tips(&self, space_id: &SpaceXgid) -> Vec<String> {
        self.graphs
            .get(space_id)
            .map(|g| g.current_tips())
            .unwrap_or_default()
    }
}

/// Kahn's topological sort: returns events in causal order (roots first).
/// Events whose predecessors are not in the set are treated as roots.
///
/// Made `pub` at Phase 7.5 persistence-amendment milestone Commit 2 per
/// runbook §4.6 lock — `replay_spaces_from_dir` (xgen-node::app) re-uses
/// this primitive as the Q1 (a).ii defensive layer (sort events
/// topologically before passing each to `ingest_event` so on-disk
/// store-iteration order minimises spurious `graph_add_event_failed`
/// error-log spam for legitimately-ordered events written out of DAG
/// order). Single source of truth per D-067 + D-076 no-drift-surface
/// family — a sibling implementation in xgen-node would introduce the
/// drift surface D-076 was promoted to eliminate.
pub fn topological_sort(events: Vec<Event>) -> Vec<Event> {
    use std::collections::{HashMap, VecDeque};

    // Pass 1 Commit 4: event.event_id is Option<EventXgid>; project to String
    // for the local topological-sort map. Internal algorithm bookkeeping —
    // Pass 2 may widen this map's key type if it becomes useful.
    let by_id: HashMap<String, Event> = events
        .into_iter()
        .filter_map(|e| {
            e.event_id
                .as_ref()
                .map(|id| (id.as_str().to_string(), e.clone()))
        })
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
    use xgen_common::xgid::{IdentityXgid, NodeXgid, SpaceXgid, Xgid};

    use super::{DispatchOutcome, EventOrigin, NodeRuntime};
    use crate::{
        crypto::encoding,
        identity::{keypair, registry::IdentityRecord},
        space::state::{
            build_dm_space_create_event, build_membership_event, build_room_create_event,
            build_space_create_event, sign_event, SpaceState,
        },
        wire::types::{Event, EventType},
    };

    fn pubkey_uri(key: &ed25519_dalek::SigningKey) -> String {
        format!(
            "xgen://pubkey/ed25519:{}",
            encoding::encode(key.verifying_key().as_bytes())
        )
    }

    // ── Pass 1 Commit 4a test helpers — typed XGID wrappers at fixture sites ──
    fn idx(s: &str) -> IdentityXgid {
        IdentityXgid::from_xgid(Xgid::new(s.to_string()))
    }
    fn ndx(s: &str) -> NodeXgid {
        NodeXgid::from_xgid(Xgid::new(s.to_string()))
    }
    fn sdx(s: &str) -> SpaceXgid {
        SpaceXgid::from_xgid(Xgid::new(s.to_string()))
    }
    fn event_id_str(ev: &Event) -> String {
        ev.event_id
            .as_ref()
            .expect("signed event has event_id")
            .as_str()
            .to_string()
    }

    fn make_record(key: &ed25519_dalek::SigningKey, home_node: &str) -> IdentityRecord {
        IdentityRecord {
            identity_id: idx(&pubkey_uri(key)),
            display_name: None,
            is_ai: false,
            ai_capabilities: None,
            registered_at: "2026-05-20T00:00:00.000Z".to_string(),
            trust_assertion: None,
            devices: vec![],
            home_node: ndx(home_node),
            update_version: 0,
            revoked: false,
            revoked_at: None,
            revocation_reason: None,
        }
    }

    fn cold_node_with_registered(alice: &ed25519_dalek::SigningKey) -> NodeRuntime {
        let node_key = keypair::generate();
        let mut node = NodeRuntime::new(node_key);
        node.register_identity(make_record(alice, node.node_id.as_str()))
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
            build_space_create_event(&alice, "test-space", None, 1, node.node_id.as_str()),
            &alice,
        );

        let peer_key = keypair::generate();
        let peer_id = ndx(&pubkey_uri(&peer_key));

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
            build_dm_space_create_event(&alice, &invitee_id, node.node_id.as_str()),
            &alice,
        );

        let peer_key = keypair::generate();
        let peer_id = ndx(&pubkey_uri(&peer_key));

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
            build_space_create_event(&alice, "test-space", None, 1, node.node_id.as_str()),
            &alice,
        );
        let space_id = event_id_str(&space_ev);
        node.ingest_event(space_ev);

        let room_ev = sign_event(
            build_room_create_event(&alice, &space_id, "general", None),
            &alice,
        );
        let room_event_id = event_id_str(&room_ev);

        let peer_key = keypair::generate();
        let peer_id = ndx(&pubkey_uri(&peer_key));

        let outcome = node.dispatch_event(room_ev, EventOrigin::ReceivedViaFederation, Some(&peer_id));
        assert!(
            matches!(outcome, DispatchOutcome::HeldPending),
            "expected HeldPending for room_create against unfederated peer; got {:?}",
            outcome
        );
        let buf = node
            .pending
            .get(space_id.as_str())
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
            build_space_create_event(&alice, "test-space", None, 1, node.node_id.as_str()),
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
            build_dm_space_create_event(&alice, &invitee_id, node.node_id.as_str()),
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

    /// M7C-D4 / A3 — the ordered 3-event send (root → room → invite over one
    /// connection, sequential, root-first) builds correct DM state on the
    /// creator's home Node. Ordering is the correctness contract: room/invite
    /// are reject-if-space-absent (step 1), NOT pending-buffered. Mirrors what
    /// `ops::create_dm_space` produces: the invite is tip-chained to the auto-room
    /// (dm_space_create ← room_create ← invite), so it is Accepted + persisted;
    /// it is a state no-op (apply_invite rejects under DM constraints, swallowed)
    /// — membership rides the root.
    #[test]
    fn dm_init_ordered_three_event_path_builds_state() {
        let alice = keypair::generate();
        let mut node = cold_node_with_registered(&alice);
        let alice_id = pubkey_uri(&alice);

        let invitee = keypair::generate();
        let invitee_id = pubkey_uri(&invitee);

        // Build the three creator-signed events exactly as ops::create_dm_space will.
        let dm_ev = sign_event(
            build_dm_space_create_event(&alice, &invitee_id, node.node_id.as_str()),
            &alice,
        );
        let space_id_str = event_id_str(&dm_ev);
        let space_id = sdx(&space_id_str);
        // Auto-room from the constructor (correctly chained to the space).
        let (_authoring, room_ev, _constructor_invite) =
            SpaceState::from_dm_space_create(&dm_ev, &alice).unwrap();
        let room_id = event_id_str(&room_ev);
        // Rebuild the invite tip-chained to the room (A3 (iii) — the constructor's
        // bundled invite has empty prev_events, a latent bug overridden here).
        let mut invite_unsigned = build_membership_event(
            &alice,
            &space_id_str,
            &room_id,
            EventType::MembershipInvite,
            json!({ "target_identity": invitee_id, "role": "member" }),
        );
        invite_unsigned.prev_events =
            vec![xgen_common::xgid::EventXgid::from_xgid(Xgid::new(room_id.clone()))];
        let invite_ev = sign_event(invite_unsigned, &alice);

        // Dispatch in order — root first.
        assert!(
            matches!(
                node.dispatch_event(dm_ev, EventOrigin::LocallySubmitted, None),
                DispatchOutcome::Accepted { .. }
            ),
            "dm_space_create root should be Accepted"
        );
        assert!(
            matches!(
                node.dispatch_event(room_ev, EventOrigin::LocallySubmitted, None),
                DispatchOutcome::Accepted { .. }
            ),
            "auto-room should be Accepted (first DM Room)"
        );
        // The tip-chained invite is Accepted (well-formed, predecessor known) but
        // is a state no-op (apply_invite reject under DM constraints, swallowed).
        assert!(
            matches!(
                node.dispatch_event(invite_ev, EventOrigin::LocallySubmitted, None),
                DispatchOutcome::Accepted { .. }
            ),
            "tip-chained auto-invite should be Accepted"
        );

        // State built from the root + room; the invite changed nothing.
        let state = node.spaces.get(&space_id).expect("DM space built on ingest");
        assert!(state.is_dm, "DM constraints active");
        assert_eq!(state.members.len(), 1, "only the creator is a member (invite is a no-op)");
        assert!(state.members.contains_key(alice_id.as_str()), "creator is the owner-member");
        assert!(
            state.pending_invites.contains_key(invitee_id.as_str()),
            "invitee seeded as pending invite from the root content"
        );
        assert_eq!(state.rooms.len(), 1, "the auto-room applied");
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
        let peer_id = ndx(&pubkey_uri(&peer_key));

        let node_key_clone = node.node_keypair.clone();
        let fed_add = sign_event(
            Event::new(
                EventType::StateFederationAdd,
                idx(&pubkey_uri(&node_key_clone)),
                xgen_common::wire::empty_room_xgid(),
                xgen_common::xgid::SpaceXgid::from_xgid(Xgid::new(unknown_space.clone())),
                vec![xgen_common::xgid::EventXgid::from_xgid(Xgid::new(unknown_space.clone()))],
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
            build_space_create_event(&alice, "fed-space", None, 1, node.node_id.as_str()),
            &alice,
        );
        let space_id = event_id_str(&space_ev);

        let peer_key = keypair::generate();
        let peer_id = ndx(&pubkey_uri(&peer_key));

        let outcome = node.dispatch_event(space_ev, EventOrigin::ReceivedViaFederation, Some(&peer_id));
        assert!(matches!(outcome, DispatchOutcome::Accepted { .. }));

        let meta: &_SpaceLocalMetadata = node
            .space_local_metadata
            .get(space_id.as_str())
            .expect("metadata must be present");
        assert_eq!(meta.space_id.as_str(), space_id);
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
            build_space_create_event(&alice, "local-space", None, 1, node.node_id.as_str()),
            &alice,
        );
        let space_id = event_id_str(&space_ev);

        let outcome = node.dispatch_event(space_ev, EventOrigin::LocallySubmitted, None);
        assert!(matches!(outcome, DispatchOutcome::Accepted { .. }));

        let meta = node
            .space_local_metadata
            .get(space_id.as_str())
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
            build_space_create_event(&alice, "twice-space", None, 1, node.node_id.as_str()),
            &alice,
        );
        let space_id = event_id_str(&space_ev);

        let peer_a = keypair::generate();
        let peer_a_id = ndx(&pubkey_uri(&peer_a));
        let peer_b = keypair::generate();
        let peer_b_id = ndx(&pubkey_uri(&peer_b));

        let first =
            node.dispatch_event(space_ev.clone(), EventOrigin::ReceivedViaFederation, Some(&peer_a_id));
        assert!(matches!(first, DispatchOutcome::Accepted { .. }));

        // Same event via a different peer — `entry().or_insert()` preserves
        // the first introducer. (`ingest_event`'s existing duplicate-event
        // guard also makes the second dispatch a state-level no-op.)
        let _ = node.dispatch_event(space_ev, EventOrigin::ReceivedViaFederation, Some(&peer_b_id));

        let meta = node.space_local_metadata.get(space_id.as_str()).unwrap();
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
            build_space_create_event(&alice, "fed-space", None, 1, node.node_id.as_str()),
            &alice,
        );
        let space_id = event_id_str(&space_ev);
        node.ingest_event(space_ev);

        let room_ev = sign_event(
            build_room_create_event(&alice, &space_id, "general", None),
            &alice,
        );
        let room_event_id = event_id_str(&room_ev);

        let peer = keypair::generate();
        let peer_id = ndx(&pubkey_uri(&peer));

        let outcome =
            node.dispatch_event(room_ev, EventOrigin::ReceivedViaFederation, Some(&peer_id));
        assert!(matches!(outcome, DispatchOutcome::HeldPending));

        let buf = node.pending.get(space_id.as_str()).expect("buffer must exist");
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
            build_space_create_event(&alice, "boot-space", None, 1, node.node_id.as_str()),
            &alice,
        );
        let space_id = event_id_str(&space_ev);
        node.ingest_event(space_ev);

        // A federation peer pushes a room_create. F-3 buffers it.
        let peer = keypair::generate();
        let peer_id = ndx(&pubkey_uri(&peer));
        let room_ev = sign_event(
            build_room_create_event(&alice, &space_id, "general", None),
            &alice,
        );
        let room_event_id = event_id_str(&room_ev);
        let outcome = node.dispatch_event(
            room_ev,
            EventOrigin::ReceivedViaFederation,
            Some(&peer_id),
        );
        assert!(matches!(outcome, DispatchOutcome::HeldPending));
        assert!(node.pending[space_id.as_str()].contains(&room_event_id));

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
                node.dag_tips(&sdx(&space_id)),
                peer_id.as_str(),
                "xgen://hash/sha256:s",
                "0.1",
                "json",
            ),
            &node_key,
        );
        node.ingest_event(fed_add);
        assert!(
            node.spaces[space_id.as_str()]
                .federation_nodes
                .iter()
                .any(|n| n.as_str() == peer_id.as_str()),
            "setup: federation_nodes must include the peer"
        );

        // Fire the arrival hook.
        node.drain_pending_by_federation_relationship(
            &peer_id,
            &sdx(&space_id),
            EventOrigin::ReceivedViaFederation,
        );

        // Buffered event should now be gone (either accepted on re-dispatch
        // or rejected by downstream validation — both remove from buffer).
        assert!(!node.pending[space_id.as_str()].contains(&room_event_id));
    }

    /// drain_pending_by_federation_relationship: idempotent — second fire
    /// for the same pair is a no-op.
    #[test]
    fn drain_pending_by_federation_relationship_idempotent() {
        let alice = keypair::generate();
        let mut node = cold_node_with_registered(&alice);
        let peer = keypair::generate();
        let peer_id = ndx(&pubkey_uri(&peer));

        // No buffer entries — both calls are no-ops; the second must not panic.
        let nothing = sdx("xgen://hash/sha256:nothing");
        node.drain_pending_by_federation_relationship(
            &peer_id,
            &nothing,
            EventOrigin::ReceivedViaFederation,
        );
        node.drain_pending_by_federation_relationship(
            &peer_id,
            &nothing,
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
            build_space_create_event(&alice, "b3-space", None, 1, node.node_id.as_str()),
            &alice,
        );
        let space_id = event_id_str(&space_ev);
        node.ingest_event(space_ev);

        // Reference a predecessor that does NOT exist in the store.
        let bogus_predecessor = "xgen://hash/sha256:not_in_store".to_string();

        let peer = keypair::generate();
        let peer_id = ndx(&pubkey_uri(&peer));
        let node_key = node.node_keypair.clone();
        let fed_add = sign_event(
            build_fed_add(
                &node_key,
                &space_id,
                vec![bogus_predecessor.clone()],
                peer_id.as_str(),
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
        assert!(node.spaces[space_id.as_str()]
            .federation_nodes
            .iter()
            .any(|n| n.as_str() == peer_id.as_str()));
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
            build_space_create_event(&alice, "b3-space", None, 1, node.node_id.as_str()),
            &alice,
        );
        let space_id = event_id_str(&space_ev);
        node.ingest_event(space_ev);

        let peer = keypair::generate();
        let peer_id = ndx(&pubkey_uri(&peer));
        // Use the PEER's keypair (NOT this Node's) to sign the federation_add
        // so the sender field is the peer's Node URI, which is unknown to
        // our identity_registry.
        let fed_add = sign_event(
            build_fed_add(
                &peer,
                &space_id,
                node.dag_tips(&sdx(&space_id)),
                peer_id.as_str(),
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
            build_space_create_event(&alice, "b3-space", None, 1, node.node_id.as_str()),
            &alice,
        );
        let space_id = event_id_str(&space_ev);
        node.ingest_event(space_ev);

        // federation_add signed by this Node's keypair — sender is this
        // Node's URI, which is NOT a Space member.
        let node_key = node.node_keypair.clone();
        let peer = keypair::generate();
        let peer_id = ndx(&pubkey_uri(&peer));
        let fed_add = sign_event(
            build_fed_add(
                &node_key,
                &space_id,
                node.dag_tips(&sdx(&space_id)),
                peer_id.as_str(),
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
            build_space_create_event(&alice, "b3-space", None, 1, node.node_id.as_str()),
            &alice,
        );
        let space_id = event_id_str(&space_ev);
        node.ingest_event(space_ev);

        // Construct federation_add with a corrupted signature.
        let node_key = node.node_keypair.clone();
        let peer = keypair::generate();
        let peer_id = ndx(&pubkey_uri(&peer));
        let mut fed_add = sign_event(
            build_fed_add(
                &node_key,
                &space_id,
                node.dag_tips(&sdx(&space_id)),
                peer_id.as_str(),
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
            build_space_create_event(&alice, "b3-space", None, 1, node.node_id.as_str()),
            &alice,
        );
        let space_id = event_id_str(&space_ev);
        node.ingest_event(space_ev);

        let node_key = node.node_keypair.clone();
        let peer = keypair::generate();
        let peer_id = ndx(&pubkey_uri(&peer));
        let fed_add = sign_event(
            build_fed_add(
                &node_key,
                &space_id,
                node.dag_tips(&sdx(&space_id)),
                peer_id.as_str(),
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

#[cfg(test)]
mod persistence_amendment_commit_2a_tests {
    //! Phase 7.5 persistence-amendment milestone Commit 2a (Q2 return-vector
    //! + Q3 all-three-drain-helpers) — unit tests locked at Joe-lock
    //!   checkpoint-#2-Commit-2a per runbook §4a.7.
    //!
    //! Test list (5 of 5):
    //!
    //!   1. dispatch_event_returns_additional_persisted_from_drain_pending_uniform
    //!   2. dispatch_event_returns_additional_persisted_from_drain_pending_by_federation_relationship
    //!   3. drain_pending_by_identity_returns_drained_events_for_caller_persistence
    //!   4. dispatch_event_aggregates_additional_persisted_across_multiple_drains
    //!   5. recursive_drain_flattens_into_outer_additional_persisted
    use serde_json::json;
    use xgen_common::xgid::{IdentityXgid, NodeXgid, SpaceXgid, Xgid};

    use super::{DispatchOutcome, EventOrigin, NodeRuntime};
    use crate::{
        crypto::encoding,
        identity::{keypair, registry::IdentityRecord},
        message::exchange::build_message_text_event,
        space::state::{
            build_federation_add_event, build_membership_event, build_room_create_event,
            build_space_create_event, sign_event,
        },
        wire::types::{Event, EventType},
    };

    fn pubkey_uri(key: &ed25519_dalek::SigningKey) -> String {
        format!(
            "xgen://pubkey/ed25519:{}",
            encoding::encode(key.verifying_key().as_bytes())
        )
    }

    // ── Pass 1 Commit 4a test helpers — typed XGID wrappers at fixture sites ──
    fn idx(s: &str) -> IdentityXgid {
        IdentityXgid::from_xgid(Xgid::new(s.to_string()))
    }
    fn ndx(s: &str) -> NodeXgid {
        NodeXgid::from_xgid(Xgid::new(s.to_string()))
    }
    fn sdx(s: &str) -> SpaceXgid {
        SpaceXgid::from_xgid(Xgid::new(s.to_string()))
    }
    fn event_id_str(ev: &Event) -> String {
        ev.event_id
            .as_ref()
            .expect("signed event has event_id")
            .as_str()
            .to_string()
    }

    fn make_record(key: &ed25519_dalek::SigningKey, home_node: &str) -> IdentityRecord {
        IdentityRecord {
            identity_id: idx(&pubkey_uri(key)),
            display_name: None,
            is_ai: false,
            ai_capabilities: None,
            registered_at: "2026-05-23T00:00:00.000Z".to_string(),
            trust_assertion: None,
            devices: vec![],
            home_node: ndx(home_node),
            update_version: 0,
            revoked: false,
            revoked_at: None,
            revocation_reason: None,
        }
    }

    /// Build a node + alice-owned Space + Room. Returns (node, space_id,
    /// room_id, alice_keypair) — alice is the Space owner (membership rules
    /// of state.space_create auto-add the signer as owner).
    fn setup_space_with_room() -> (
        NodeRuntime,
        String,
        String,
        ed25519_dalek::SigningKey,
    ) {
        let alice = keypair::generate();
        let node_key = keypair::generate();
        let mut node = NodeRuntime::new(node_key);
        node.register_identity(make_record(&alice, node.node_id.as_str()))
            .unwrap();
        let space_ev = sign_event(
            build_space_create_event(&alice, "p7-5-amend-space", None, 1, node.node_id.as_str()),
            &alice,
        );
        let space_id = event_id_str(&space_ev);
        node.ingest_event(space_ev);
        let room_ev = sign_event(
            build_room_create_event(&alice, &space_id, "general", None),
            &alice,
        );
        let room_id = event_id_str(&room_ev);
        node.ingest_event(room_ev);
        (node, space_id, room_id, alice)
    }

    /// Test 1 — Q2(a) happy-path for the predecessor-arrival drain.
    /// Buffer msg_b (predecessor msg_a missing), then dispatch msg_a; assert
    /// msg_a's outcome carries msg_b in additional_persisted (Step 6 drain).
    #[test]
    fn dispatch_event_returns_additional_persisted_from_drain_pending_uniform() {
        let (mut node, space_id, room_id, alice) = setup_space_with_room();
        let current_tip = node.dag_tips(&sdx(&space_id)).first().cloned().unwrap();

        let msg_a = sign_event(
            build_message_text_event(&alice, &space_id, &room_id, vec![current_tip], "A"),
            &alice,
        );
        let msg_a_id = event_id_str(&msg_a);
        let msg_b = sign_event(
            build_message_text_event(&alice, &space_id, &room_id, vec![msg_a_id.clone()], "B"),
            &alice,
        );
        let msg_b_id = event_id_str(&msg_b);

        // Buffer B first.
        let out_b = node.dispatch_event(msg_b, EventOrigin::LocallySubmitted, None);
        assert!(
            matches!(out_b, DispatchOutcome::HeldPending),
            "B should HeldPending when predecessor A absent; got {:?}",
            out_b
        );

        // Dispatch A; Step 6 drain should fire and drain B.
        let out_a = node.dispatch_event(msg_a, EventOrigin::LocallySubmitted, None);
        match out_a {
            DispatchOutcome::Accepted {
                additional_persisted,
                ..
            } => {
                assert_eq!(
                    additional_persisted.len(),
                    1,
                    "additional_persisted should contain B; got {} events",
                    additional_persisted.len()
                );
                assert_eq!(
                    additional_persisted[0].event_id.as_ref().map(|e| e.as_str()),
                    Some(msg_b_id.as_str()),
                    "drained event must be B"
                );
            }
            other => panic!("expected Accepted with [B]; got {:?}", other),
        }
    }

    /// Test 2 — Q2(a) happy-path for the federation-relationship drain.
    /// Buffer an event on the F-3 trigger (no federation relationship), then
    /// dispatch state.federation_add for the (peer, space) pair; assert the
    /// fed_add's outcome carries the previously-buffered event in
    /// additional_persisted (Step 7 drain).
    #[test]
    fn dispatch_event_returns_additional_persisted_from_drain_pending_by_federation_relationship()
    {
        let (mut node, space_id, _room_id, alice) = setup_space_with_room();

        // A peer pushes a room_create — F-3 buffers it (no fed relationship yet).
        let peer = keypair::generate();
        let peer_id = ndx(&pubkey_uri(&peer));
        let buffered_room_ev = sign_event(
            build_room_create_event(&alice, &space_id, "fed-arriving-room", None),
            &alice,
        );
        let buffered_event_id = event_id_str(&buffered_room_ev);
        let out_buffered = node.dispatch_event(
            buffered_room_ev,
            EventOrigin::ReceivedViaFederation,
            Some(&peer_id),
        );
        assert!(
            matches!(out_buffered, DispatchOutcome::HeldPending),
            "F-3 should buffer on federation-relationship trigger; got {:?}",
            out_buffered
        );

        // Now dispatch the federation_add that establishes (peer, space).
        let node_key = node.node_keypair.clone();
        let fed_add = sign_event(
            build_federation_add_event(
                &node_key,
                &space_id,
                node.dag_tips(&sdx(&space_id)),
                peer_id.as_str(),
                "xgen://hash/sha256:session",
                "0.1",
                "json",
            ),
            &node_key,
        );
        let out_fed_add = node.dispatch_event(
            fed_add,
            EventOrigin::ReceivedViaFederation,
            Some(&peer_id),
        );
        match out_fed_add {
            DispatchOutcome::Accepted {
                additional_persisted,
                ..
            } => {
                assert!(
                    additional_persisted
                        .iter()
                        .any(|ev| ev.event_id.as_ref().map(|e| e.as_str()) == Some(buffered_event_id.as_str())),
                    "additional_persisted should contain the F-3-drained event; got {} events",
                    additional_persisted.len()
                );
            }
            other => panic!("expected Accepted with F-3-drained event; got {:?}", other),
        }
    }

    /// Test 3 — Q3 third-drain-helper coverage. drain_pending_by_identity is
    /// invoked from xgen-node::app::handle_identity_replicate_msg (NOT
    /// dispatch_event), so we exercise it directly. Buffer an event whose
    /// signer is unknown to id_registry, register the signer, call
    /// drain_pending_by_identity, assert returned Vec<Event> contains the
    /// drained event.
    #[test]
    fn drain_pending_by_identity_returns_drained_events_for_caller_persistence() {
        let (mut node, space_id, _room_id, alice) = setup_space_with_room();

        // Bob is NOT registered. A bob-signed state.federation_add arrives
        // via federation (skips Steps 9/11 per B3, but signer-unknown can
        // still buffer via F-10 Identity-arrival path if any other step
        // catches it). Simpler: use a Path-A message after pretending bob
        // is a Space member via direct membership ingest.
        let bob = keypair::generate();
        let bob_id = pubkey_uri(&bob);
        let invite = sign_event(
            build_membership_event(
                &alice,
                &space_id,
                "",
                EventType::MembershipInvite,
                json!({ "target_identity": bob_id, "role": "member" }),
            ),
            &alice,
        );
        node.ingest_event(invite);
        // Bob joins at Space level (room_id empty).
        let bob_space_join = sign_event(
            build_membership_event(&bob, &space_id, "", EventType::MembershipJoin, json!({})),
            &bob,
        );
        node.ingest_event(bob_space_join);
        // Bob joins at Room level (room_id non-empty) — required so Step 11b
        // membership-check passes when his post-Identity-arrival re-dispatch
        // hits a Room-context event.
        // node.spaces[space_id.as_str()].rooms is HashMap<RoomXgid, _> post-Pass-1; project
        // the key to String at the &str-API boundary.
        let room_id_for_join: String = node.spaces[space_id.as_str()]
            .rooms
            .keys()
            .next()
            .unwrap()
            .as_str()
            .to_string();
        let bob_room_join = sign_event(
            build_membership_event(
                &bob,
                &space_id,
                &room_id_for_join,
                EventType::MembershipJoin,
                json!({}),
            ),
            &bob,
        );
        node.ingest_event(bob_room_join);

        // Bob is now a member at SpaceState level (Space + Room), but his
        // Identity record is not in id_registry. Construct a bob-signed
        // message and dispatch it — validate_event Step 11a catches unknown
        // signer, buffers with missing_identity=Some(bob_id).
        let current_tip = node.dag_tips(&sdx(&space_id)).first().cloned().unwrap();
        let bob_msg = sign_event(
            build_message_text_event(
                &bob,
                &space_id,
                &room_id_for_join,
                vec![current_tip],
                "msg from bob whose identity record is missing",
            ),
            &bob,
        );
        let bob_msg_id = event_id_str(&bob_msg);
        let outcome = node.dispatch_event(bob_msg, EventOrigin::LocallySubmitted, None);
        assert!(
            matches!(outcome, DispatchOutcome::HeldPending),
            "bob's message should HeldPending pending Identity arrival; got {:?}",
            outcome
        );

        // Register bob's Identity.
        node.register_identity(make_record(&bob, node.node_id.as_str()))
            .unwrap();

        // Fire drain_pending_by_identity directly — returns Vec<Event>.
        let drained =
            node.drain_pending_by_identity(&idx(&bob_id), EventOrigin::ReceivedViaFederation);

        assert!(
            drained
                .iter()
                .any(|ev| ev.event_id.as_ref().map(|e| e.as_str()) == Some(bob_msg_id.as_str())),
            "drained Vec<Event> should contain bob's message; got {} events",
            drained.len()
        );
    }

    /// Test 4 — Q3 multi-drain aggregation. Single dispatch_event call that
    /// fires BOTH the predecessor drain AND the federation-relationship
    /// drain. The state.federation_add we dispatch has prev_events advancing
    /// the DAG (no predecessor effect at Step 6 from itself) — to surface
    /// Step 6's drain, we buffer an event whose predecessor is the fed_add's
    /// event_id. Step 7 fires for the (peer, space) pair separately.
    #[test]
    fn dispatch_event_aggregates_additional_persisted_across_multiple_drains() {
        let (mut node, space_id, room_id, alice) = setup_space_with_room();
        let peer = keypair::generate();
        let peer_id = ndx(&pubkey_uri(&peer));

        // Buffer a room_create on the F-3 trigger (no fed relationship yet).
        let f3_buffered = sign_event(
            build_room_create_event(&alice, &space_id, "f3-room", None),
            &alice,
        );
        let f3_buffered_id = event_id_str(&f3_buffered);
        let out_f3 = node.dispatch_event(
            f3_buffered,
            EventOrigin::ReceivedViaFederation,
            Some(&peer_id),
        );
        assert!(matches!(out_f3, DispatchOutcome::HeldPending));

        // Construct the federation_add. Its event_id is computed by sign_event.
        let node_key = node.node_keypair.clone();
        let fed_add = sign_event(
            build_federation_add_event(
                &node_key,
                &space_id,
                node.dag_tips(&sdx(&space_id)),
                peer_id.as_str(),
                "xgen://hash/sha256:multi-drain-session",
                "0.1",
                "json",
            ),
            &node_key,
        );
        let fed_add_id = event_id_str(&fed_add);

        // Buffer a successor-of-fed_add event on the predecessor trigger.
        // Use build_message_text_event so prev_events can be custom-set
        // BEFORE signing (post-sign mutation breaks Step 8 event_id hash).
        let pred_buffered = sign_event(
            build_message_text_event(
                &alice,
                &space_id,
                &room_id,
                vec![fed_add_id.clone()],
                "successor of fed_add",
            ),
            &alice,
        );
        let pred_buffered_id = event_id_str(&pred_buffered);
        let out_pred = node.dispatch_event(
            pred_buffered,
            EventOrigin::LocallySubmitted,
            None,
        );
        assert!(
            matches!(out_pred, DispatchOutcome::HeldPending),
            "predecessor-trigger buffer expected; got {:?}",
            out_pred
        );

        // Dispatch the federation_add. Step 6 drains pred_buffered (its
        // predecessor fed_add_id just arrived); Step 7 drains f3_buffered
        // (its (peer, space) relationship just established).
        let out_fed_add = node.dispatch_event(
            fed_add,
            EventOrigin::ReceivedViaFederation,
            Some(&peer_id),
        );
        match out_fed_add {
            DispatchOutcome::Accepted {
                additional_persisted,
                ..
            } => {
                let ids: Vec<Option<&str>> = additional_persisted
                    .iter()
                    .map(|ev| ev.event_id.as_ref().map(|e| e.as_str()))
                    .collect();
                assert!(
                    ids.contains(&Some(pred_buffered_id.as_str())),
                    "predecessor-drained event missing from additional_persisted; got ids {:?}",
                    ids
                );
                assert!(
                    ids.contains(&Some(f3_buffered_id.as_str())),
                    "F-3-drained event missing from additional_persisted; got ids {:?}",
                    ids
                );
            }
            other => panic!(
                "expected Accepted aggregating both drained events; got {:?}",
                other
            ),
        }
    }

    /// Test 5 — Shape β2 cascade regression lock. A → drains B → drains C;
    /// assert ALL of B and C land in A's additional_persisted (the β2
    /// flattening invariant; Shape β1 regression would surface as missing C).
    #[test]
    fn recursive_drain_flattens_into_outer_additional_persisted() {
        let (mut node, space_id, room_id, alice) = setup_space_with_room();
        let current_tip = node.dag_tips(&sdx(&space_id)).first().cloned().unwrap();

        let msg_a = sign_event(
            build_message_text_event(&alice, &space_id, &room_id, vec![current_tip], "A"),
            &alice,
        );
        let msg_a_id = event_id_str(&msg_a);
        let msg_b = sign_event(
            build_message_text_event(
                &alice,
                &space_id,
                &room_id,
                vec![msg_a_id.clone()],
                "B",
            ),
            &alice,
        );
        let msg_b_id = event_id_str(&msg_b);
        let msg_c = sign_event(
            build_message_text_event(
                &alice,
                &space_id,
                &room_id,
                vec![msg_b_id.clone()],
                "C",
            ),
            &alice,
        );
        let msg_c_id = event_id_str(&msg_c);

        // Buffer C first (B not present), then B (A not present).
        assert!(matches!(
            node.dispatch_event(msg_c, EventOrigin::LocallySubmitted, None),
            DispatchOutcome::HeldPending
        ));
        assert!(matches!(
            node.dispatch_event(msg_b, EventOrigin::LocallySubmitted, None),
            DispatchOutcome::HeldPending
        ));

        // Dispatch A; cascade drains B, then B's recursive dispatch drains C.
        // Shape β2 flattens both into A's outer additional_persisted.
        let out_a = node.dispatch_event(msg_a, EventOrigin::LocallySubmitted, None);
        match out_a {
            DispatchOutcome::Accepted {
                additional_persisted,
                ..
            } => {
                let ids: Vec<Option<&str>> = additional_persisted
                    .iter()
                    .map(|ev| ev.event_id.as_ref().map(|e| e.as_str()))
                    .collect();
                assert!(
                    ids.contains(&Some(msg_b_id.as_str())),
                    "B missing from cascade; got ids {:?}",
                    ids
                );
                assert!(
                    ids.contains(&Some(msg_c_id.as_str())),
                    "C missing from cascade (Shape β1 regression?); got ids {:?}",
                    ids
                );
                assert_eq!(
                    additional_persisted.len(),
                    2,
                    "cascade should yield exactly [B, C]; got {} events",
                    additional_persisted.len()
                );
            }
            other => panic!("expected Accepted with cascade [B, C]; got {:?}", other),
        }
    }

    // ── Pass 3 Commit 2a per-surface tests T1-T4 (runbook §4.7) ──────────────

    // T1 (Surface #1) — round-trip insert with typed SpaceXgid key + retrieve
    // via Borrow<str> projection + hash-consistency at boundary.
    #[test]
    fn noderuntime_per_space_map_insert_retrieve_with_typed_key() {
        let alice = keypair::generate();
        let node_key = keypair::generate();
        let mut rt = NodeRuntime::new(node_key);
        rt.register_identity(make_record(&alice, rt.node_id.as_str()))
            .expect("register");

        let space_ev = sign_event(
            build_space_create_event(&alice, "t1-space", None, 1, rt.node_id.as_str()),
            &alice,
        );
        let space_id_str = event_id_str(&space_ev);
        rt.ingest_event(space_ev);
        let space_id_typed = sdx(&space_id_str);

        // (a) Typed contains_key with &SpaceXgid succeeds (post-Surface-#1 retype).
        assert!(rt.spaces.contains_key(&space_id_typed));
        assert!(rt.stores.contains_key(&space_id_typed));
        assert!(rt.graphs.contains_key(&space_id_typed));

        // (b) Borrow<str> projection — HashMap<SpaceXgid, _>::contains_key(&str) works.
        assert!(rt.spaces.contains_key(space_id_str.as_str()));
        assert!(rt.stores.contains_key(space_id_str.as_str()));

        // (c) Hash-consistency at boundary: typed and &str lookups return same value.
        let via_typed = rt.spaces.get(&space_id_typed).map(|s| s.space_id.clone());
        let via_str = rt.spaces.get(space_id_str.as_str()).map(|s| s.space_id.clone());
        assert_eq!(via_typed, via_str);
    }

    // T2 (Surface #1) — verify all six per-space maps accept their respective
    // SpaceXgid keys without cross-flavour leak. Each map is queried with its
    // own typed key; each access returns the value inserted at ingest time.
    #[test]
    fn noderuntime_per_space_map_six_flavours_isolated() {
        let alice = keypair::generate();
        let node_key = keypair::generate();
        let mut rt = NodeRuntime::new(node_key);
        rt.register_identity(make_record(&alice, rt.node_id.as_str()))
            .expect("register");

        let space_ev = sign_event(
            build_space_create_event(&alice, "t2-space", None, 1, rt.node_id.as_str()),
            &alice,
        );
        let space_id = sdx(&event_id_str(&space_ev));
        rt.ingest_event(space_ev);

        // All six per-space maps are SpaceXgid-keyed post-Pass-3 (Q1.1).
        // Lookup succeeds via typed contains_key on all that get populated at ingest:
        assert!(rt.spaces.contains_key(&space_id), "spaces");
        assert!(rt.stores.contains_key(&space_id), "stores");
        assert!(rt.graphs.contains_key(&space_id), "graphs");
        // pending / dm_proposals / space_local_metadata are demand-populated;
        // verify the key types compile (the maps exist) by inserting and reading.
        rt.pending.entry(space_id.clone()).or_default();
        assert!(rt.pending.contains_key(&space_id), "pending");
        rt.space_local_metadata.entry(space_id.clone()).or_insert_with(|| {
            xgen_common::space_local::SpaceLocalMetadata::new_local(
                space_id.clone(),
                chrono::Utc::now().to_rfc3339(),
            )
        });
        assert!(
            rt.space_local_metadata.contains_key(&space_id),
            "space_local_metadata"
        );
    }

    // T3 (Surface #1) — verify helper method signatures expose typed keys at
    // public API boundary (all_events + dag_tips take &SpaceXgid per Q1.5).
    #[test]
    fn noderuntime_per_space_map_helper_signatures_typed_at_boundary() {
        let alice = keypair::generate();
        let node_key = keypair::generate();
        let mut rt = NodeRuntime::new(node_key);
        rt.register_identity(make_record(&alice, rt.node_id.as_str()))
            .expect("register");

        let space_ev = sign_event(
            build_space_create_event(&alice, "t3-space", None, 1, rt.node_id.as_str()),
            &alice,
        );
        let space_id = sdx(&event_id_str(&space_ev));
        rt.ingest_event(space_ev);

        // Public-API helpers (Q1.5) consume &SpaceXgid natively.
        let events = rt.all_events(&space_id);
        assert!(!events.is_empty(), "all_events must include the space_create");
        let tips = rt.dag_tips(&space_id);
        assert!(!tips.is_empty(), "dag_tips must include the space_create tip");
    }

    // T4 (Surface #2) — verify dispatch_event borrowed-NodeXgid boundary works
    // under both Some(&NodeXgid) (federation channel) and None (local).
    #[test]
    fn dispatch_event_with_borrowed_node_xgid_projects_to_str_at_callsite() {
        let alice = keypair::generate();
        let node_key = keypair::generate();
        let mut rt = NodeRuntime::new(node_key);
        rt.register_identity(make_record(&alice, rt.node_id.as_str()))
            .expect("register");

        // Local-submitted Space-create succeeds with None peer_node_id.
        let space_ev = sign_event(
            build_space_create_event(&alice, "t4-space", None, 1, rt.node_id.as_str()),
            &alice,
        );
        let outcome_local = rt.dispatch_event(
            space_ev.clone(),
            EventOrigin::LocallySubmitted,
            None,
        );
        assert!(
            matches!(outcome_local, DispatchOutcome::Accepted { .. }),
            "local Space-create must accept; got {:?}",
            outcome_local
        );

        // F-3 skip-rule for Space-creates (Lock B1 + §5 extension) means a
        // federation-channel Space-create with Some(&NodeXgid) also accepts
        // without requiring pre-existing federation_nodes entry. The borrowed
        // peer_node_id type-check is the load-bearing assertion here.
        let peer_key = keypair::generate();
        let peer = ndx(&pubkey_uri(&peer_key));
        let space2_ev = sign_event(
            build_space_create_event(&alice, "t4-space-fed", None, 1, rt.node_id.as_str()),
            &alice,
        );
        let outcome_fed = rt.dispatch_event(
            space2_ev,
            EventOrigin::ReceivedViaFederation,
            Some(&peer),
        );
        assert!(
            matches!(outcome_fed, DispatchOutcome::Accepted { .. }),
            "federation-channel Space-create with typed peer must accept (F-3 skip); got {:?}",
            outcome_fed
        );
    }
}
