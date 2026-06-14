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
use std::sync::Arc;

use chrono::{SecondsFormat, Utc};
use ed25519_dalek::SigningKey;
use xgen_common::clock::{Clock, RealClock};
use xgen_common::space_local::SpaceLocalMetadata;
use xgen_common::state::StorageAdvert;
use xgen_common::xgid::{EventXgid, IdentityXgid, NodeXgid, SpaceXgid, Xgid};

use crate::{
    auth::{module_registry::AuthModuleRegistry, tiers::verify_tier_assertion},
    dag::{
        graph::DagGraph,
        pending::PendingBuffer,
        store::{
            vanilla_store_factory, EventStore, StoreFactory, StoreInitError, VANILLA_DESCRIPTOR,
        },
    },
    encryption::key_package::{KeyPackageError, KeyPackageStore, StoredKeyPackage},
    identity::{
        registration::AssertionPolicy,
        registry::{IdentityRecord, IdentityRegistry, RegistryError},
        replication::ReplicaRegistry,
    },
    message::exchange::{
        check_ai_capability, check_ai_operator_targets_pub, check_permission_pub,
        validate_event, ExchangeError, ValidationOutcome,
    },
    resolution::{
        derive::{conflicts_in_log, derive_resolved},
        state_key::state_key_for_event,
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
    // MP-F3-D2 — a re-submitted duplicate (event_id already in the store) is
    // applied once but must NOT be re-broadcast. `Duplicate` is the sibling of
    // `HeldPending`: a "do-not-fan-out, not-an-error" outcome. `process_inbound`
    // maps it to `FanoutRequest::none()` (kills local fan-out AND federation
    // push) while still sending an idempotent ack (F3-D3) — the event WAS
    // accepted at first ingest, so acking is truthful and stops a retrying
    // LocallySubmitted client. Adding a variant (vs changing one's payload)
    // leaves every `matches!(_, Accepted{..})` / `Rejected(_)` wildcard intact
    // — the inverse of the MP-F2 shape call.
    Duplicate,
    // MP-F2-D1 — carries the structured `RejectInfo` (was `String`) so the
    // specific wire code reaches `process_inbound` → the `Error` frame. The
    // 1-tuple shape keeps `matches!(_, Rejected(_))` wildcards + drain arms
    // unchanged (MP-F2-D1 minimal-blast-radius lock).
    Rejected(RejectInfo),
}

/// Structured rejection metadata (MP-F2-D1) — the protocol wire code and name
/// alongside the human-readable reason.
///
/// The code field is authoritative: `reject_signal` reads `info.code` and never
/// re-parses the reason string (D-067 no-drift). This closes the old
/// `Rejected(String)` flatten that dropped `ExchangeError::to_wire_code` on the
/// wire (J-081 / D-070-pending reject-code half).
///
/// MP-F2-D3 scope: `from_exchange` surfaces a code only where `to_wire_code()`
/// is `Some` (e.g. `TimestampOutOfBounds` → 3046, closing MP-A-15). Unmapped
/// variants (signature, permission, …) fall back to the generic 4000 band this
/// arc → MP-F2-followon.
#[derive(Debug, Clone)]
pub struct RejectInfo {
    pub code: u32,
    pub name: &'static str,
    pub reason: String,
}

impl RejectInfo {
    /// Generic transport rejection — no specific protocol code (Cat D:
    /// pre-validate guards / internal errors).
    pub fn generic(reason: impl Into<String>) -> Self {
        Self { code: 4000, name: "generic", reason: reason.into() }
    }

    /// A rejection whose wire `(code, name)` the producing gate already knows
    /// (Cat B/C: the tier / invite / ai-role gates).
    pub fn coded(code: u32, name: &'static str, reason: impl Into<String>) -> Self {
        Self { code, name, reason: reason.into() }
    }

    /// Derive `(code, name)` from an `ExchangeError` (Cat A). Mapped variants
    /// carry their `to_wire_code`; unmapped fall back to generic 4000. The
    /// `reason` is the error's Display (left byte-identical — MP-F2-D2).
    pub fn from_exchange(err: &ExchangeError) -> Self {
        match err.to_wire_code() {
            Some((code, name)) => Self { code, name, reason: err.to_string() },
            None => Self::generic(err.to_string()),
        }
    }
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

/// Resolve a registered Identity's auth tier from its Trust Assertion (PG-13,
/// Arc D PM-D2).
///
/// `None` (no assertion) → tier **1**, the cryptographic-identity baseline every
/// keypair-holder has. `Some(v)` → `v["tier"]` as a `u32`, defaulting to 1 when
/// the field is absent or not an integer (forward-compat / Local-Node records).
///
/// **Arc E (PG-03) upgrade landed here as a semantic, not a structural, change.**
/// The stored `record.trust_assertion` is now the value that passed the full
/// §3.8.5 `validate_assertion` at registration (`!local_node` path) — so
/// `v["tier"]` is the *validated* tier, no longer a blindly-trusted JSON poke.
/// The join tier-gate (PG-13) therefore carries a real value at Tier 2–4; Tier-1
/// and Local-Node records stay the honest no-op. The read shape is unchanged
/// because PG-13 was deliberately wired against `["tier"]` ahead of PG-03.
fn assertion_tier_of(record: &IdentityRecord) -> u32 {
    match &record.trust_assertion {
        None => 1,
        Some(v) => v.get("tier").and_then(serde_json::Value::as_u64).unwrap_or(1) as u32,
    }
}

/// M8.5-B (INV-D6) — the maximum invite validity window, in seconds, for an
/// invitee of the given Trust-Assertion tier. The ceiling **tightens as tier
/// rises** (exposure-window minimization: the most consequential credential gets
/// the tightest window). An invite whose `valid_until` exceeds
/// `invite_timestamp + ceiling` is rejected at ingest with wire `3045`
/// (`invite_validity_exceeds_max`).
///
/// **Only Tier 1 is defined now: 14 days.** Honest posture (D-065): until
/// trusted Auth Modules exist, `assertion_tier_of` resolves every Identity to
/// Tier 1, so only the T1 path is exercisable end-to-end; the tier grading above
/// T1 is wired-but-dormant (the PG-13 posture). The higher-tier ceilings are
/// deferred to the tier/Auth-Module work that owns per-tier policy.
///
/// **Forward-note (D-077).** The T1 = 14d constant is an interim protocol value
/// standing in until Tier 1 is rebuilt as a proper Auth Module; at that point
/// the T1 ceiling becomes module-derived, **bounded ≤ 14d** (14d is the inherited
/// upper bound, not a floor). The grading rule, the cap, and the enforcement are
/// unchanged across that transition — only the *source* of the number moves.
const T1_INVITE_VALIDITY_CEILING_SECS: i64 = 14 * 24 * 60 * 60; // 14 days

fn invite_validity_ceiling_secs(tier: u32) -> i64 {
    // Higher tiers are dormant until per-tier modules land; they fall back to the
    // T1 ceiling rather than a wider window (never *wider* than 14d). `tier` is
    // threaded now so the call site is tier-aware ahead of the module work.
    let _ = tier;
    T1_INVITE_VALIDITY_CEILING_SECS
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
    /// SE-D6 (Storage-Engine milestone): the value is boxed so a selected engine
    /// module can stand in for the vanilla `InMemoryEventStore`.
    /// Behaviour-neutral when no engine is selected — the box holds an
    /// `InMemoryEventStore` (the default backend, D-080).
    ///
    /// `+ Send + Sync`: `NodeRuntime` lives in `Arc<tokio::Mutex<…>>` and crosses
    /// `tokio::spawn` boundaries, so the trait object must keep the auto-traits
    /// the concrete vanilla backend already satisfies. (C4 implication: an engine
    /// wrapping a `!Sync` resource — e.g. `rusqlite::Connection` — must
    /// internally synchronise to be `Send + Sync`.)
    pub stores: HashMap<SpaceXgid, Box<dyn EventStore + Send + Sync>>,
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
    /// MP-F11 (R3-D6) — the established **regular-Space** federation relationships
    /// this Node holds: `space_id → {established peer node ids}`. The DM case
    /// derives `federation_nodes` from its members (Design-Z, F1B); a regular
    /// Space's federation peers are established out-of-band (the federation
    /// handshake / `add-peer` naming the Space), which `NodeRuntime` does not
    /// otherwise record (the `FederationRegistry` lives in xgen-node). This is the
    /// xgen-core record of "who I have federated with for S" — the F-3 authority —
    /// so a regular Space's `federation_nodes` survives a `derive_resolved` rebuild
    /// (which re-folds the log and would otherwise drop a peer whose
    /// `state.federation_add` is predecessor-held). Populated via
    /// [`NodeRuntime::establish_federation_relationship`]; not persisted (rebuilt
    /// on establish, sibling to `replica_registry`).
    pub federation_relationships: HashMap<SpaceXgid, std::collections::HashSet<NodeXgid>>,
    /// Phase 7.5 §5.3 + §5.6 — local-only per-Space provenance metadata.
    /// Sibling to SpaceState (NOT a field on it — preserves SpaceState's
    /// "all content derived from federated events" invariant). Populated
    /// ONCE at Space-create ingestion (federation: introducer = peer;
    /// local: introducer = None); idempotent on duplicate Space-create
    /// arrivals (HashMap::entry-or-insert semantics). Persisted by
    /// xgen-node to `xgen-node_space_local_metadata.json`.
    /// Pass 3 (Surface #1 Q1.1) retypes key to SpaceXgid.
    pub space_local_metadata: HashMap<SpaceXgid, SpaceLocalMetadata>,
    /// SE-SUB-D5 — per-Space store constructor. Default = the vanilla closure
    /// (behaviour-neutral); xgen-node installs an engine closure after the SE-D4
    /// gate when an engine is selected. The three store-construction sites call
    /// [`NodeRuntime::ensure_store`] → this factory instead of hard-coding
    /// `InMemoryEventStore`.
    store_factory: StoreFactory,
    /// SE-SUB-D6 — true when a selected engine owns durability. Gates the
    /// app-layer JSON persist bypass + engine-replay rehydration in xgen-node.
    /// Default false (vanilla mode = today's JSON durability, unchanged).
    pub engine_owns_durability: bool,
    /// SE-D8 — the active storage backend advert (operator-visible node-state).
    /// Defaults to the vanilla backend; xgen-node sets it from the SE-D4 gate
    /// result at startup (for both vanilla and engine selections).
    pub storage_advert: StorageAdvert,
    /// Arc E (PG-03, CP-2) — the Node's Trust-Assertion acceptance policy. Held
    /// here (not threaded through `process_inbound`) so `handle_identity_msg`
    /// reads it under the existing runtime lock and passes it to
    /// `accept_registration`. Default = empty (trust no Auth Module, required_tier
    /// 1); xgen-node installs the config-derived policy at startup via
    /// [`NodeRuntime::set_assertion_policy`]. Consulted only in production-mode
    /// registration; Local Node bypasses (§3.8.8).
    pub assertion_policy: AssertionPolicy,
    /// M10.2 (M10.2-D2) — the live trusted-Auth-Module registry, shared with the
    /// `auth-module` CRUD verbs (one `Arc<Mutex<>>` instance). The registration
    /// gate (`handle_identity_msg`) **live-reads** `trusted_issuers()` from this
    /// per registration and overrides `assertion_policy.trusted_issuers`, so a
    /// `revoke` bites immediately (no restart). Installed by xgen-node at
    /// `run_node` top-level via [`NodeRuntime::set_auth_module_registry`] — the
    /// registry's **first runtime consumer**, structurally closing AMR-D1.
    /// `None` (the `new()` default + every test/baseline path) ⇒ empty trust set
    /// ⇒ today's behaviour byte-for-byte (the empty-baseline prime invariant).
    pub auth_module_registry: Option<Arc<tokio::sync::Mutex<AuthModuleRegistry>>>,
    /// Arc H (PG-05, AH-A5) — Node-side MLS KeyPackage pool. Populated as
    /// `mls.key_package` events are ingested (the Node stores uploaded
    /// KeyPackages); served + single-use-consumed via
    /// [`NodeRuntime::request_key_package`] when a member is added (§3.10.3/.5).
    /// In-memory + rebuilt on replay (Phase-2 simplification, sibling to
    /// `replica_registry`): the `mls.key_package` events live in the EventStore,
    /// so a restart re-populates the pool by re-ingest. **Honest residue
    /// (D-065):** consumption is not durably tracked — a consumed package is
    /// re-added on replay; production single-use durability is fenced behind D3.
    pub key_package_store: KeyPackageStore,
    /// M8.6 (clock seam, design §3.2) — the injected time source. Single home
    /// for the W-domain (`now_utc`) + M-domain (`now_instant`) reads on the
    /// federation reconnect / F-10 paths. `new()` defaults to `RealClock`
    /// (behaviour-identical to the pre-seam inline reads); tests install a
    /// `MockClock` via [`NodeRuntime::set_clock`]. Threaded as `Arc<dyn Clock>`;
    /// never serialized. Consumers (the pending-buffer `add` M-site here, and the
    /// scheduler / session W-sites in xgen-node) pull it from this field.
    clock: Arc<dyn Clock>,
    /// M8.6 (C8 seam) — capacity of the per-peer outbound federation channel
    /// (`run_federation_session_post_handshake`'s `mpsc::channel`). Production
    /// default 1024 (unchanged); the C8 bidirectional-back-pressure test sets it
    /// small (2) so the channel-full path is reachable — making the test
    /// sensitive to a future blocking-`send` regression (which, unlike today's
    /// non-blocking `try_send`, would deadlock under a mutual full-channel
    /// burst). Test-only setter; no operator surface.
    federation_channel_capacity: usize,
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
            federation_relationships: HashMap::new(),
            space_local_metadata: HashMap::new(),
            // SE-SUB-D5 — default vanilla factory; behaviour-neutral, so every
            // existing constructor/test path is unchanged.
            store_factory: vanilla_store_factory(),
            engine_owns_durability: false,
            storage_advert: StorageAdvert {
                engine: VANILLA_DESCRIPTOR.name.to_string(),
                assurance: VANILLA_DESCRIPTOR.assurance.label().to_string(),
                asserts_tier: 1,
            },
            // Arc E (PG-03) — empty default: trust no Auth Module, required_tier 1.
            // xgen-node installs the config-derived policy at startup.
            assertion_policy: AssertionPolicy::default(),
            // M10.2 (M10.2-D2) — no registry by default; xgen-node installs the
            // shared live instance at run_node top-level. None ⇒ empty trust set
            // ⇒ today (empty-baseline prime invariant).
            auth_module_registry: None,
            // Arc H (PG-05) — empty KeyPackage pool; filled by mls.key_package
            // ingestion.
            key_package_store: KeyPackageStore::new(),
            // M8.6 — production default; behaviour-identical to the pre-seam
            // inline Utc::now() / Instant::now() reads.
            clock: Arc::new(RealClock),
            // M8.6 (C8 seam) — production default, unchanged from the prior
            // hardcoded `mpsc::channel(1024)`.
            federation_channel_capacity: 1024,
        }
    }

    /// M8.6 (C8 seam) — set the per-peer outbound federation channel capacity.
    /// Production never calls this (the 1024 default stands); the C8 test sets it
    /// to 2 so the channel-full back-pressure path is reachable.
    pub fn set_federation_channel_capacity(&mut self, cap: usize) {
        self.federation_channel_capacity = cap;
    }

    /// M8.6 (C8 seam) — the per-peer outbound federation channel capacity. Read
    /// by `run_federation_session_post_handshake` at channel-create time.
    pub fn federation_channel_capacity(&self) -> usize {
        self.federation_channel_capacity
    }

    /// M8.6 (clock seam) — install the injected time source. Production never
    /// calls this (the `new()` default `RealClock` stands); tests install a
    /// `MockClock` to drive the federation reconnect / F-10 windows without real
    /// waiting. Sibling to `set_assertion_policy` / `set_store_factory`.
    pub fn set_clock(&mut self, clock: Arc<dyn Clock>) {
        self.clock = clock;
    }

    /// M8.6 (clock seam) — the injected time source (cheap `Arc` clone). Used by
    /// the xgen-node scheduler / session W-sites, which hold a `&NodeRuntime`
    /// under the runtime lock and read `clock().now_utc()`.
    pub fn clock(&self) -> Arc<dyn Clock> {
        Arc::clone(&self.clock)
    }

    /// Arc E (PG-03, CP-2) — install the Node's Trust-Assertion acceptance policy.
    /// Called by xgen-node at startup from `[node].trusted_auth_modules` config.
    /// Default (unset) is the empty policy — production registration then rejects
    /// every assertion at step 1 until an issuer is trusted; Local Node bypasses.
    pub fn set_assertion_policy(&mut self, policy: AssertionPolicy) {
        self.assertion_policy = policy;
    }

    /// M10.2 (M10.2-D2) — install the shared live Auth Module registry. Called by
    /// xgen-node at `run_node` top-level with the same `Arc<Mutex<>>` handed to the
    /// `auth-module` CRUD verbs, so a `register`/`revoke` is visible to the gate
    /// immediately. Sibling to `set_assertion_policy`. Production never leaves this
    /// `None` once wired; the `None` default keeps every test/baseline path at
    /// today's behaviour.
    pub fn set_auth_module_registry(
        &mut self,
        registry: Arc<tokio::sync::Mutex<AuthModuleRegistry>>,
    ) {
        self.auth_module_registry = Some(registry);
    }

    /// SE-D8 — set the active-storage advert (operator-visible node-state). Called
    /// by xgen-node at startup from the SE-D4 gate result.
    pub fn set_storage_advert(&mut self, advert: StorageAdvert) {
        self.storage_advert = advert;
    }

    /// SE-SUB-D5 — install a per-Space store factory (the engine closure). Sets
    /// `engine_owns_durability` so the xgen-node persist/replay layer hands
    /// durability to the engine (SE-SUB-D6). Vanilla mode never calls this.
    pub fn set_store_factory(&mut self, factory: StoreFactory) {
        self.store_factory = factory;
        self.engine_owns_durability = true;
    }

    /// SE-SUB-D4/D5 — ensure the per-Space store exists, constructing it via the
    /// injected factory on first touch. **Never** silently yields a vanilla RAM
    /// store under an engine selection: an engine open failure propagates as
    /// [`StoreInitError`], mapped loudly per call site.
    pub fn ensure_store(&mut self, space_id: &SpaceXgid) -> Result<(), StoreInitError> {
        if !self.stores.contains_key(space_id) {
            // The factory borrow ends before the insert (non-overlapping fields).
            let store = (self.store_factory)(space_id)?;
            self.stores.insert(space_id.clone(), store);
        }
        Ok(())
    }

    /// SE-SUB-D6 — rebuild a Space's in-memory graph + `SpaceState` from an
    /// **already-populated** store (engine mode startup). The store is the
    /// source of truth, so events are **not** re-appended (the double-write
    /// Scope B exists to avoid). Mirrors `ingest_event`'s apply core but over
    /// the full, pre-stored event set rather than one arriving event.
    pub fn rehydrate_space_from_store(&mut self, space_id: &SpaceXgid) {
        let events: Vec<Event> = match self.stores.get(space_id) {
            Some(s) => topological_sort(s.range(0).unwrap_or_default()),
            None => return,
        };
        if events.is_empty() {
            return;
        }
        let my_node_id: String = self.node_id.as_str().to_string();

        // Rebuild the graph (all events; predecessors checked against the store).
        self.graphs.entry(space_id.clone()).or_default();
        {
            let NodeRuntime { graphs, stores, .. } = self;
            let graph = graphs.get_mut(space_id).unwrap();
            let store = stores.get(space_id).unwrap();
            for ev in &events {
                let _ = graph.add_event(ev, &**store);
            }
        }

        // M8 C2 — build the convergent SpaceState from the resolved log.
        // Previously a plain create-then-apply replay (last-write-wins on
        // arrival order); `derive_resolved` makes cold-start convergent: if the
        // restored log contains concurrent conflicting State Events, the snapshot
        // is the seven-layer resolution of them, identical on every Node that
        // holds the same log (§3.9.2 / §3.9.7). `identity_home_nodes` is sourced
        // live from the registry (CP-C, per-rebuild).
        let ihn = build_identity_home_nodes(&self.identity_registry);
        if let Some(mut state) = derive_resolved(events, &my_node_id, &ihn) {
            // MP-F1b (F1B-D1, apply site 4) — a rebuilt DM SpaceState starts with
            // empty federation_nodes; re-populate from members on cold-start.
            repopulate_dm_federation_nodes(&mut state, &self.identity_registry);
            // MP-F11 (R3-D6) — a rebuilt REGULAR Space re-populates federation_nodes
            // from the established relationships (survives the rebuild).
            repopulate_regular_federation_nodes(&mut state, &self.federation_relationships);
            self.spaces.insert(state.space_id.clone(), state);
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

        // SE-SUB-D4 — construct the store via the injected factory (engine or
        // vanilla); an engine open failure is loud-and-skip, never a silent
        // vanilla RAM store under an engine selection.
        if let Err(e) = self.ensure_store(&space_id) {
            tracing::error!(
                event = "store_init_failed",
                space_id = %space_id.as_str(),
                error = %e,
                "ingest_event: store init failed; skipping event (no vanilla fallback)"
            );
            return;
        }
        self.graphs.entry(space_id.clone()).or_default();

        // D-075 vantage: capture local Node URI before destructuring `self`.
        // Threaded into `apply_event` so `apply_federation_add` can derive
        // the relevant peer per design §4.1.
        // Pass 2 (Surface #2 Q2.5 + Q2.7) — node_id is NodeXgid; project to
        // owned String for SpaceState::apply_event (out of Pass 2 scope per
        // design §5.1 — SpaceState methods take &str, defer to Pass 3).
        let my_node_id: String = self.node_id.as_str().to_string();

        let NodeRuntime {
            spaces,
            stores,
            graphs,
            identity_registry,
            federation_relationships,
            ..
        } = self;
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
        match graph.add_event(&event, &**store) {
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
        // SE-D6: `store` is now `&mut Box<dyn EventStore>`; the trait `append`
        // is the boxed equivalent of the inherent `insert` (it delegates to it).
        let _ = store.append(event.clone());

        // Apply to SpaceState.
        // M8 C2 — route SpaceState derivation through the resolving core.
        match &event.event_type {
            // Create events build (or rebuild) the whole Space snapshot from the
            // full log via `derive_resolved`. This subsumes the pre-M8 manual
            // out-of-order replay (a `state.room_create` arriving before its
            // `state.space_create`) AND makes the create-time snapshot convergent
            // if conflicting children are already present. `derive_resolved`
            // dispatches the create constructor internally — `from_space_create`
            // for a plain Space, `from_dm_space_create_node` for a DM (the
            // key-less node-side seed: members = {creator} + pending_invites =
            // {invitee}; the auto-`membership.invite` is a no-op-by-reject under
            // DM constraints, 3.16.1 — CP-1 trace, J-219; the auto-room applies
            // through the normal applier). The event is already in the store
            // (appended above), so it is part of `range(0)`.
            EventType::StateSpaceCreate | EventType::StateDmSpaceCreate => {
                let log: Vec<Event> = store.range(0).unwrap_or_default();
                let ihn = build_identity_home_nodes(identity_registry);
                if let Some(mut state) = derive_resolved(log, &my_node_id, &ihn) {
                    // MP-F1b (F1B-D1, apply site 1) — DM create resets
                    // federation_nodes to empty; populate from members.
                    repopulate_dm_federation_nodes(&mut state, identity_registry);
                    // MP-F11 (R3-D6) — regular Space: re-populate from the
                    // established relationships (survives the create-arm rebuild).
                    repopulate_regular_federation_nodes(&mut state, federation_relationships);
                    spaces.insert(state.space_id.clone(), state);
                }
            }
            // Every other event takes the SR-D1 conflict gate. The common case —
            // a non-state-keyed event (message, etc.) or a state event with no
            // concurrent same-key event in the log — takes the fast incremental
            // `apply_event`, byte-for-byte today's behaviour. Only a genuine
            // concurrent conflict (CP-E: ancestry-aware via `conflicts_in_log`,
            // NOT direct-parent-only `conflicts_with`) triggers a full convergent
            // rebuild from the resolved log (SR-D2). The `state_key_for_event`
            // guard short-circuits before any log scan for non-keyed events, so
            // message traffic never pays the gate cost.
            _ => {
                let conflict = state_key_for_event(&event).is_some()
                    && conflicts_in_log(&event, &store.range(0).unwrap_or_default());
                if conflict {
                    let log: Vec<Event> = store.range(0).unwrap_or_default();
                    let ihn = build_identity_home_nodes(identity_registry);
                    if let Some(mut state) = derive_resolved(log, &my_node_id, &ihn) {
                        // MP-F1b (F1B-D1, apply site 2) — a concurrent-conflict
                        // rebuild resets a DM's federation_nodes; re-populate.
                        repopulate_dm_federation_nodes(&mut state, identity_registry);
                        // MP-F11 (R3-D6) — regular Space: re-populate from the
                        // established relationships (survives the conflict rebuild).
                        repopulate_regular_federation_nodes(&mut state, federation_relationships);
                        spaces.insert(space_id.clone(), state);
                    }
                } else if let Some(state) = spaces.get_mut(&space_id) {
                    let _ = state.apply_event(&event, &my_node_id);
                    // MP-F1b (F1B-D1, apply site 3) — the common DM membership-apply
                    // (join/leave) path; re-derive federation_nodes from the new
                    // member set. The re-fire here is load-bearing: identity
                    // replication can lag the DM create, so the create-arm
                    // population may be incomplete (runbook §7.4).
                    repopulate_dm_federation_nodes(state, identity_registry);
                }
            }
        }

        // Arc H (PG-05, AH-A5) — Node-side KeyPackage store hook. An
        // `mls.key_package` event is a client uploading a KeyPackage; the Node
        // (DS role) stores it for later distribution. Reached by BOTH the live
        // path (`dispatch_event` → `ingest_event`, line ~1128) and replay
        // (`replay_spaces_from_dir` → `ingest_event`), so the pool repopulates on
        // restart. Side-effect only — does not touch SpaceState.
        if event.event_type == EventType::MlsKeyPackage {
            self.record_key_package(&event);
        }
    }

    /// Arc H (PG-05, AH-A5) — store a KeyPackage carried by an `mls.key_package`
    /// event into the Node's pool (§3.10.3). The uploader is the event sender;
    /// `device_id` / `mls_key_package` / `valid_until` come from the event
    /// content (schema §3.10.3). Missing required fields ⇒ silent skip (the event
    /// is still stored in the DAG; a malformed KeyPackage simply does not enter
    /// the servable pool).
    fn record_key_package(&mut self, event: &Event) {
        let content = &event.content;
        let mls_key_package = match content["mls_key_package"].as_str() {
            Some(s) => s.to_string(),
            None => return,
        };
        // identity_id defaults to the event sender (the uploader) when content
        // omits it; device_id is required for the pool key.
        let identity_id = content["identity_id"]
            .as_str()
            .map(str::to_string)
            .unwrap_or_else(|| event.sender.as_str().to_string());
        let device_id = match content["device_id"].as_str() {
            Some(s) => s.to_string(),
            None => return,
        };
        let valid_until = content["valid_until"].as_str().unwrap_or("").to_string();
        let uploaded_at = content["uploaded_at"]
            .as_str()
            .unwrap_or(event.timestamp.as_str())
            .to_string();
        self.key_package_store.store(StoredKeyPackage {
            identity_id,
            device_id,
            mls_key_package,
            uploaded_at,
            valid_until,
        });
    }

    /// Arc H (PG-05, AH-A5 / §3.10.5) — serve + single-use-consume a KeyPackage
    /// for a member being added to an MLS group. Discards expired packages first
    /// (§3.10.3 MUST), then consumes the next valid one. Returns the §3.10.11
    /// wire codes via [`KeyPackageError`] (5001 none / 5002 only-expired).
    pub fn request_key_package(
        &mut self,
        identity_id: &str,
        device_id: &str,
        now_timestamp: &str,
    ) -> Result<StoredKeyPackage, KeyPackageError> {
        self.key_package_store.request(identity_id, device_id, now_timestamp)
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
        // SE-SUB-D4 — store via the injected factory; engine open failure maps
        // to an error (the Space cannot accept), never a silent vanilla store.
        self.ensure_store(space_id)
            .map_err(|e| ExchangeError::DagError(format!("store init failed: {e}")))?;
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
                &mut **store,
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
                let now = self.clock.now_instant();
                // INV-EXP (D-1) — legacy accept_message path (test-only-reachable
                // per runbook §3.6.1 Step 1); origin-blind drain via
                // drain_pending_messages → accept_event, so LocallySubmitted is
                // the neutral stored default.
                self.pending
                    .entry(space_id.clone())
                    .or_default()
                    .add(event, EventOrigin::LocallySubmitted, &missing, None, None, now);
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
                Some(buf) => buf.resolve(resolved_id, &**store, identity_registry),
                None => return,
            }
        };

        // INV-EXP (D-1) — resolve yields (Event, EventOrigin); this legacy
        // accept_message drain path applies via `accept_event` (origin-blind),
        // so the stored origin is discarded here.
        for (ev, _origin) in ready {
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
                        &mut **store,
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
    /// (client connection vs federation peer session). The *validation core*
    /// (`validate_event`) is origin-uniform; `origin` is consumed by the
    /// dispatcher's admission gates and metadata: Phase 4's
    /// `apply_federation_push` anti-transitivity guard (F-5 §8.5), the
    /// space-local-metadata local-vs-federation tag, and — INV-EXP (C2) — the
    /// 3044/3045 invite-admission gates, which run *only* at live local
    /// admission (`LocallySubmitted`) and are skipped on `ReceivedViaFederation`
    /// (a replica trusts the home node's admission decision; design §2).
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
        // INV-EXP (C2) — `origin` is now consumed by the 3044/3045 admission
        // gates below (the prior `let _ = origin;` no-op is removed).

        // Resolve the effective space_id via the shared `space_id_of` resolver
        // (no-drift per D-067; same resolution used by apply_federation_push +
        // process_inbound). State-create events carry empty space_id on the
        // wire; their own event_id becomes the space_id.
        // Pass 3 (Surface #1 Q1.3) — internal variable binds as typed SpaceXgid.
        let space_id: SpaceXgid = match space_id_of(&event) {
            Some(s) => s,
            None => {
                return DispatchOutcome::Rejected(RejectInfo::generic("event missing event_id"));
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
            return DispatchOutcome::Rejected(RejectInfo::generic(format!(
                "space not found: {}",
                space_id.as_str()
            )));
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
                    let now = self.clock.now_instant();
                    // INV-EXP (D-1) — store the dispatch's own `origin` so the
                    // federation-relationship drain re-dispatches with the true
                    // per-entry origin. This F-3 path is reached only for
                    // federation-channel events (peer_node_id.is_some()), so
                    // `origin` is `ReceivedViaFederation` here — passing it
                    // (vs hardcoding) keeps the caller's declared origin the
                    // single source of truth.
                    self.pending
                        .entry(space_id.clone())
                        .or_default()
                        .add(event, origin, &[], None, Some(fed_key), now);
                    return DispatchOutcome::HeldPending;
                }
            }
        }

        // Step 3 — Validation core (uniform across all event families).
        // SE-SUB-D4 — store via the injected factory; an engine open failure
        // rejects the event (the Space cannot accept it) rather than
        // RAM-shadowing it under a vanilla fallback.
        if let Err(e) = self.ensure_store(&space_id) {
            return DispatchOutcome::Rejected(RejectInfo::generic(format!("store init failed: {e}")));
        }
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

        // M9.1 (F1 / gap G6) — read the injected clock once before the disjoint
        // borrow of `self` below. `now` (DateTime<Utc>, Copy) feeds Step 8.5's
        // future-skew bound inside validate_event (D-090; admission-only, D-076).
        let now = self.clock.now_utc();
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
            validate_event(&event, space, identity_registry, &**store, fed_add_via_federation, now)
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
                return DispatchOutcome::Rejected(RejectInfo::from_exchange(&err));
            }
            ValidationOutcome::HeldPending { missing_predecessors, missing_identity } => {
                // Pass 2 (Surface #1 Q1 + Surface #3 Q3.1) — ValidationOutcome
                // now carries Vec<EventXgid> + Option<IdentityXgid>; PendingBuffer::add
                // takes &[EventXgid] + Option<&IdentityXgid>. Bind missing_identity
                // as &IdentityXgid via .as_ref() (not .as_deref(), which would
                // project through Deref<Target = Xgid>).
                let now = self.clock.now_instant();
                // INV-EXP (D-1) — store the dispatch's own `origin` so the
                // predecessor / Identity drain re-dispatches each released event
                // with its true per-entry origin (the C2 admission gates then
                // run iff that origin is LocallySubmitted).
                self.pending
                    .entry(space_id)
                    .or_default()
                    .add(
                        event,
                        origin,
                        &missing_predecessors,
                        missing_identity.as_ref(),
                        None,
                        now,
                    );
                return DispatchOutcome::HeldPending;
            }
            ValidationOutcome::Validated => {}
        }

        // MP-F3-D1 — dedup-at-dispatch, placed AFTER `validate_event` passes
        // (in-arc correction of the design's "before validate_event" ordering;
        // J-326). "Same event_id" only means "genuine duplicate" once the event
        // is confirmed valid: event_id is the SHA-256 hash of the canonical
        // content, which EXCLUDES the signature — so a forged event reusing an
        // already-stored event's content+id but swapping the signature has a
        // colliding event_id yet a bad signature (step 12). Deduping before
        // validation would mis-report that forgery as `Duplicate` and lose the
        // signature-failure signal (the validation-asymmetry security property,
        // proven by phase9_compound_c5). So validation runs first; only a
        // fully-valid, already-stored event is a true duplicate. A genuine
        // duplicate was accepted before ⇒ it re-validates cleanly (its
        // predecessors are present; the future-skew bound never rejects a past
        // timestamp), so it reaches this gate. Placed before the Step 4 semantic
        // gates: an already-stored event is already fully accepted, so gate
        // drift (e.g. an invite that has since expired) must not un-accept it.
        //
        // On a hit: `Duplicate` → `process_inbound` sends an idempotent ack +
        // `FanoutRequest::none()` (suppresses local fan-out AND federation push,
        // F3-D3/D4). `event_id == None` is unreachable here (validation requires
        // a matching event_id). Convergence-neutral by construction (D-076): the
        // log already holds the event exactly once, so dropping the duplicate vs
        // idempotently re-applying it leave an identical log — the early-return
        // loses nothing (the five skipped re-run effects are all idempotent /
        // already-fired; design §F3-D5, re-confirmed as-built).
        if let Some(eid) = event.event_id.as_ref() {
            if self.stores.get(&space_id).map(|s| s.contains(eid)).unwrap_or(false) {
                return DispatchOutcome::Duplicate;
            }
        }

        // Step 4 — Semantic pre-checks (post-validation, per design doc §7.6).
        // AI role violation: AI senders cannot create Spaces (M3, 3041).
        if is_space_creation {
            if let Some(record) = self.identity_registry.get(&event.sender) {
                if record.is_ai {
                    return DispatchOutcome::Rejected(RejectInfo::coded(
                        3041,
                        "ai_role_violation",
                        format!("ai_role_violation: {} from AI sender", event.event_type.as_str()),
                    ));
                }
            }
        }
        // AI capability check (3042) — applies to validated events from AI
        // senders. For human senders the function is a no-op.
        if let Err(e) = check_ai_capability(&event, &self.identity_registry) {
            return DispatchOutcome::Rejected(RejectInfo::from_exchange(&e));
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
                    return DispatchOutcome::Rejected(RejectInfo::from_exchange(&e));
                }
                if let Err(e) = check_permission_pub(&event, space) {
                    return DispatchOutcome::Rejected(RejectInfo::from_exchange(&e));
                }
            }
        }
        // PG-13 (Arc D, PM-D1) — tier-gate on join. A `MembershipJoin` must
        // satisfy the Space's slot contract: the joiner's Trust-Assertion tier
        // MUST be >= `space.auth_tier`. Join is in `validate_event`'s
        // step-13-skip set (the join *makes* the member), so this is a new
        // semantic pre-check here, not a `check_permission` tweak. On a tier
        // shortfall it returns `Rejected` carrying wire 3030 (`tier_mismatch`).
        //
        // Honest Tier-1 no-op (D-065): today every Space is `auth_tier=1` and
        // every joiner resolves to tier 1 (`assertion_tier_of`), so the gate
        // evaluates `verify_tier_assertion(1, 1) = Ok` — a genuine no-op. The
        // plumbing is live, not decorative: the gate bites the moment a real
        // Tier 2–4 Space (PG-03 + a higher-tier auth module, out of arc-D
        // scope) exists. The order vs the AI checks above is not load-bearing —
        // an AI joining a Space is legitimate (AI is barred only from *owning*).
        // M8.5-B (INV-D6, CP-1/3045) — invite over-ceiling reject at ingest.
        // An invite whose `valid_until` exceeds `invite_timestamp + ceiling(tier)`
        // is rejected here (wire `3045 invite_validity_exceeds_max`), where the
        // ceiling is keyed on the *invitee's* tier (`assertion_tier_of`). The
        // Node never silently clamps (D-065 honest-fail). Absent `valid_until`
        // ⇒ no check (the cascade default is filled inviter-side, C2). This is a
        // gate, convergence-neutral — it runs before apply, returns no resolved
        // value. T1=14d is the only live ceiling (honest posture).
        //
        // INV-EXP (D-2, C2) — admission-only gate. Run iff the invite is being
        // admitted live and locally (`origin == LocallySubmitted`); on
        // `ReceivedViaFederation` the whole block is SKIPPED (the invite falls
        // through to apply — NOT rejected): a federated peer applies an invite it
        // received without re-checking the ceiling, replicating the home node's
        // already-made admission decision. 3045 is replay-stable (no clock), so
        // this is the uniform admission-only rule (design §5), not a bug fix.
        if origin == EventOrigin::LocallySubmitted
            && matches!(event.event_type, EventType::MembershipInvite)
        {
            if let Some(vu_str) = event.content["valid_until"].as_str().filter(|s| !s.is_empty()) {
                match chrono::DateTime::parse_from_rfc3339(vu_str) {
                    Ok(valid_until) => {
                        let invitee_tier = event.content["target_identity"]
                            .as_str()
                            .map(|t| IdentityXgid::from_xgid(Xgid::new(t.to_string())))
                            .and_then(|id| self.identity_registry.get(&id).map(assertion_tier_of))
                            .unwrap_or(1);
                        let ceiling = invite_validity_ceiling_secs(invitee_tier);
                        let max_valid_until = chrono::DateTime::parse_from_rfc3339(
                            event.timestamp.as_str(),
                        )
                        .ok()
                        .map(|ts| ts + chrono::Duration::seconds(ceiling));
                        if let Some(max) = max_valid_until {
                            if valid_until > max {
                                return DispatchOutcome::Rejected(RejectInfo::coded(
                                    3045,
                                    "invite_validity_exceeds_max",
                                    format!(
                                        "invite_validity_exceeds_max (3045): valid_until {} exceeds tier-{} ceiling ({}s) from invite timestamp {}",
                                        vu_str, invitee_tier, ceiling, event.timestamp.as_str()
                                    ),
                                ));
                            }
                        }
                    }
                    Err(_) => {
                        return DispatchOutcome::Rejected(RejectInfo::coded(
                            3045,
                            "invite_validity_exceeds_max",
                            format!(
                                "invite_validity_exceeds_max (3045): valid_until '{}' is not a valid RFC-3339 timestamp",
                                vu_str
                            ),
                        ));
                    }
                }
            }
        }

        if matches!(event.event_type, EventType::MembershipJoin) {
            if let Some(space) = self.spaces.get(&space_id) {
                // MP-F6 (M10.5-D2/D3) — dispatch-level banned pre-check. Without
                // it, a banned identity's re-join is *accepted-but-inert*:
                // `validate_event` does not consult `banned`, so the join passes
                // validation and reaches `ingest_event`, whose
                // `let _ = state.apply_event` (runtime.rs:748) swallows
                // `apply_join`'s `Err(Banned)` (state.rs:1003) — `ingest_event`
                // returns `()`, so the dispatch reply was `Accepted` (is_ok=true)
                // for an event `derive_resolved` will drop. The end-state stayed
                // correct (resolution is a second gate), but the *reply* lied.
                // Surface the reject HERE (the reply); the apply-layer silence at
                // :748 stays — it is load-bearing for replay tolerance (a replayed
                // event resolution will drop must not crash replay; audit A4).
                // Reject = PermissionDenied-class (4000-unmapped: `to_wire_code`
                // returns None for it, exchange.rs:140), the same shape MP-C-09's
                // banned-*send* reject lands as — no new wire code, no ch3 edit
                // (M10.5-D3; a precise `join_banned` code is MP-F2-followon work).
                if space.banned.contains(&event.sender) {
                    return DispatchOutcome::Rejected(RejectInfo::from_exchange(
                        &ExchangeError::PermissionDenied(format!(
                            "membership.join: identity {} is banned from Space {}",
                            event.sender.as_str(),
                            space_id.as_str()
                        )),
                    ));
                }
                let joiner_tier = self
                    .identity_registry
                    .get(&event.sender)
                    .map(assertion_tier_of)
                    .unwrap_or(1);
                if let Err(e) = verify_tier_assertion(joiner_tier, space.auth_tier) {
                    let (code, name) = e.to_wire_code().unwrap_or((3030, "tier_mismatch"));
                    return DispatchOutcome::Rejected(RejectInfo::coded(
                        code,
                        name,
                        format!("{name} ({code}): {e}"),
                    ));
                }
                // M8.5-B (INV-D6, CP-1/3044) — invite-expiry gate at join
                // acceptance. The pending invite's `valid_until` (stored by
                // `apply_invite`) is checked against the Node's own clock; a past
                // deadline rejects the join with wire `3044 invite_expired`.
                // Convergence-neutral (a gate, like PG-13 — no clock-skew problem,
                // no `derive_resolved` surface).
                //
                // **Fail-closed for non-DM (Joe-lock, C2).** A real client always
                // stamps `valid_until` (default 14d) post-C2, so on a regular
                // Space an absent `valid_until` means malformed/legacy → reject,
                // never "treat as no-expiry" (which would be an unbounded
                // read/join capability — exactly what INV-D6 prevents).
                // **DM Spaces are exempt by design** (`dm_constraints_active`):
                // the DM creator atomically seeds the 2-party counterparty, so
                // there is no detached in-flight invite to misdirect — the absence
                // of `valid_until` is the absence of the window `valid_until`
                // guards, not an omission.
                //
                // Space-level join only (room_id empty): a Room join is gated by
                // existing Space membership, not a pending invite; an open join
                // (no pending invite at all) is untouched.
                //
                // INV-EXP (D-1/D-3, C2) — admission-only + injected clock. The
                // gate runs iff `origin == LocallySubmitted`; on
                // `ReceivedViaFederation` it is SKIPPED (the join falls through
                // to apply — NOT rejected). A peer trusts the home node's
                // already-made admission decision and does not re-adjudicate
                // invite-expiry on replication (design §2; F-5/D-089 pairwise
                // trust). This is the headline fix: an aged-Space federation
                // catch-up no longer rejects historical invited-joins against the
                // *receiver's* wall-clock, so membership stops diverging from the
                // home node. The expiry comparison reads the injected
                // `self.clock.now_utc()` (D-090) instead of raw `Utc::now()`,
                // making the home node's real-time enforcement deterministically
                // testable (the aged-Space repro advances the injected clock).
                if origin == EventOrigin::LocallySubmitted && event.room_id.as_str().is_empty() {
                    // D-090 — read the injected clock once before the
                    // pending-invite lookup. `space` borrows `self.spaces`;
                    // `self.clock` is a disjoint field (both shared borrows), and
                    // `now` is `Copy` so the closure captures it by value.
                    let now = self.clock.now_utc();
                    if let Some(pi) = space.pending_invites.get(&event.sender) {
                        match pi.valid_until.as_deref() {
                            Some(vu_str) => {
                                let past = chrono::DateTime::parse_from_rfc3339(vu_str)
                                    .map(|vu| now > vu.with_timezone(&Utc))
                                    .unwrap_or(true); // unparseable ⇒ fail-closed
                                if past {
                                    return DispatchOutcome::Rejected(RejectInfo::coded(
                                        3044,
                                        "invite_expired",
                                        format!(
                                            "invite_expired (3044): invite valid_until {} is past or malformed",
                                            vu_str
                                        ),
                                    ));
                                }
                            }
                            None if !space.dm_constraints_active => {
                                return DispatchOutcome::Rejected(RejectInfo::coded(
                                    3044,
                                    "invite_expired",
                                    "invite_expired (3044): non-DM invite carries no valid_until (malformed/legacy)",
                                ));
                            }
                            None => {} // DM-seeded invite: exempt by design.
                        }
                    }
                }
            }
        }

        // Arc E (PG-08, AE-D6/D9) — Thread tier gates on `thread.create`, sibling
        // to the PG-13 join gate (same `assertion_tier_of` + `verify_tier_assertion`
        // path, no-drift). Room membership is already enforced in `validate_event`
        // step 11 (the event carries `room_id`); these two checks add the
        // tier semantics:
        //   (1) narrow-not-widen (ch2): a Thread's `auth_tier_min` may only raise
        //       the Room's tier floor, never lower it. Rooms carry no per-Room tier
        //       today, so the floor is the Space's `auth_tier` (honest as-built —
        //       the Room inherits the Space tier). Below-floor → reject.
        //   (2) participation (AE-D9): the creator's own tier must meet the
        //       Thread's `auth_tier_min`. Honest Tier-1 no-op until a real Tier 2–4
        //       assertion exists (PG-03 gave `assertion_tier_of` its teeth).
        if matches!(event.event_type, EventType::ThreadCreate) {
            if let Some(space) = self.spaces.get(&space_id) {
                let thread_tier = event.content["auth_tier_min"].as_u64().unwrap_or(1) as u32;
                if thread_tier < space.auth_tier {
                    return DispatchOutcome::Rejected(RejectInfo::coded(
                        3030,
                        "thread_auth_tier_below_room",
                        format!(
                            "thread_auth_tier_below_room (3030): thread auth_tier_min {} < space auth_tier {}",
                            thread_tier, space.auth_tier
                        ),
                    ));
                }
                let creator_tier = self
                    .identity_registry
                    .get(&event.sender)
                    .map(assertion_tier_of)
                    .unwrap_or(1);
                if let Err(e) = verify_tier_assertion(creator_tier, thread_tier) {
                    let (code, name) = e.to_wire_code().unwrap_or((3030, "tier_mismatch"));
                    return DispatchOutcome::Rejected(RejectInfo::coded(
                        code,
                        name,
                        format!("{name} ({code}): {e}"),
                    ));
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
            additional_persisted.extend(self.drain_pending_uniform(&space_id, eid));
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
                .extend(self.drain_pending_by_federation_relationship(&peer, &sp));
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
    /// INV-EXP (D-1) — drained events re-dispatch with their *stored* per-entry
    /// `EventOrigin` (handed back by `PendingBuffer::resolve` as part of the
    /// `(Event, EventOrigin)` pair), not the triggering event's origin. This
    /// supersedes the Phase 4 batch-origin approximation: a single arrival hook
    /// can release a mix of origins, and the C2 admission gates (3044/3045) key
    /// on the per-event origin, so the true stored origin is load-bearing. The
    /// batch `origin` parameter is therefore removed from this helper.
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
                Some(buf) => buf.resolve(resolved_id, &**store, identity_registry),
                None => return Vec::new(),
            }
        };
        let mut drained: Vec<Event> = Vec::new();
        for (ev, stored_origin) in ready {
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
            // INV-EXP (D-1) — re-dispatch with the entry's stored origin.
            let ev_clone = ev.clone();
            match self.dispatch_event(ev, stored_origin, None) {
                DispatchOutcome::Accepted {
                    new_joiner: _,
                    additional_persisted,
                } => {
                    drained.push(ev_clone);
                    drained.extend(additional_persisted);
                }
                // MP-F3-D2 — a drained event was buffered (not stored), so it
                // ingests fresh → Accepted; Duplicate is a safe no-op here.
                DispatchOutcome::HeldPending
                | DispatchOutcome::Rejected(_)
                | DispatchOutcome::Duplicate => {}
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
    ) -> Vec<Event> {
        // INV-EXP (D-1) — the batch `origin` param is removed; released events
        // re-dispatch with their stored per-entry origin (resolve_identity hands
        // back (Event, EventOrigin)). A single Identity-arrival drain can release
        // a mix of local + federation joins waiting on the same signer; per-entry
        // origin is what distinguishes them at the C2 admission gates.
        //
        // Pass 3 (Surface #1 Q1.4 + Surface #2 Q2.5) — internal helper
        // signature retyped to &IdentityXgid; typed PendingBuffer call drops
        // the previous Xgid::new wrap.
        //
        // Collect (space_id, ready_events) under the buffer lock domain
        // first so we can re-dispatch outside it without re-entrant
        // borrows on self.pending.
        let space_ids: Vec<SpaceXgid> = self.pending.keys().cloned().collect();
        let mut all_ready: Vec<(Event, EventOrigin)> = Vec::new();
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
                    Some(buf) => buf.resolve_identity(identity_id, &**store, identity_registry),
                    None => continue,
                }
            };
            all_ready.extend(ready_for_space);
        }
        let mut drained: Vec<Event> = Vec::new();
        for (ev, stored_origin) in all_ready {
            // Same drain approximation as `drain_pending_uniform` — F-3
            // peer_node_id not stored per buffered entry; passing None
            // skips the F-3 re-check on drain.
            //
            // Phase 7.5 persistence-amendment Q2/Q3 — capture for caller-side
            // persistence; Shape β2 cascade flattening per drain_pending_uniform's
            // doc-comment.
            // INV-EXP (D-1) — re-dispatch with the entry's stored origin.
            let ev_clone = ev.clone();
            match self.dispatch_event(ev, stored_origin, None) {
                DispatchOutcome::Accepted {
                    new_joiner: _,
                    additional_persisted,
                } => {
                    drained.push(ev_clone);
                    drained.extend(additional_persisted);
                }
                // MP-F3-D2 — a drained event was buffered (not stored), so it
                // ingests fresh → Accepted; Duplicate is a safe no-op here.
                DispatchOutcome::HeldPending
                | DispatchOutcome::Rejected(_)
                | DispatchOutcome::Duplicate => {}
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
    ) -> Vec<Event> {
        // INV-EXP (D-1) — the batch `origin` param is removed; released events
        // re-dispatch with their stored per-entry origin (resolve_federation_relationship
        // hands back (Event, EventOrigin)).
        //
        // Pass 3 (Surface #1 Q1.4 + Surface #2 Q2.5) — internal helper
        // signature retyped to (&NodeXgid, &SpaceXgid); typed PendingBuffer
        // call drops the previous Xgid::new wraps.
        let space_ids: Vec<SpaceXgid> = self.pending.keys().cloned().collect();
        let mut all_ready: Vec<(Event, EventOrigin)> = Vec::new();
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
                        &**store,
                        identity_registry,
                    ),
                    None => continue,
                }
            };
            all_ready.extend(ready_for_space);
        }
        let mut drained: Vec<Event> = Vec::new();
        for (ev, stored_origin) in all_ready {
            // Drain approximation: same shape as the other two drain helpers.
            // The drained event passed F-3 in the new world (its (peer, space)
            // is now in federation_nodes by definition — federation_add just
            // ingested) so re-check would pass; we still pass None to keep
            // the drain-path symmetry with the other two hooks.
            //
            // Phase 7.5 persistence-amendment Q2/Q3 — capture for caller-side
            // persistence; Shape β2 cascade flattening per drain_pending_uniform's
            // doc-comment.
            // INV-EXP (D-1) — re-dispatch with the entry's stored origin.
            let ev_clone = ev.clone();
            match self.dispatch_event(ev, stored_origin, None) {
                DispatchOutcome::Accepted {
                    new_joiner: _,
                    additional_persisted,
                } => {
                    drained.push(ev_clone);
                    drained.extend(additional_persisted);
                }
                // MP-F3-D2 — a drained event was buffered (not stored), so it
                // ingests fresh → Accepted; Duplicate is a safe no-op here.
                DispatchOutcome::HeldPending
                | DispatchOutcome::Rejected(_)
                | DispatchOutcome::Duplicate => {}
            }
        }
        drained
    }

    /// MP-F1b (Design Z) — identity-replicate hook. When an `IdentityRecord`
    /// replicates to this Node (its `home_node` becomes resolvable), re-populate
    /// `federation_nodes` for every DM the identity is a party of, then drain any
    /// F-3-pending DM membership events held for that peer.
    ///
    /// Why both halves: a DM federates *late*, and the counterparty's record can
    /// arrive **after** the DM was created here (the create-arm helper then omitted
    /// it, F1B-D3). This hook closes that timing race: (1) the re-populate adds the
    /// now-resolvable home to `federation_nodes` so subsequent DM events push to
    /// it; (2) if the counterparty's `membership.join` had already arrived and was
    /// F-3-held (peer not yet in `federation_nodes`), the drain releases it.
    ///
    /// **D-076 discharged by inheritance.** The drain reuses
    /// `drain_pending_by_federation_relationship` **verbatim** — the same hook the
    /// `state.federation_add` trigger fires (runtime.rs `dispatch_event`), not a new
    /// drain. Released events re-enter `dispatch_event` (the convergence-proven
    /// pipeline) with `peer_node_id = None`, identical to the federation_add
    /// trigger. One more trigger, no new ordering decision.
    ///
    /// Returns drained events for caller-side persistence (sibling to
    /// `drain_pending_by_identity`). DM-only; regular Spaces are untouched.
    pub fn repopulate_dm_federation_after_identity(
        &mut self,
        identity_id: &IdentityXgid,
        home_node: &NodeXgid,
    ) -> Vec<Event> {
        // DM spaces where this identity is a party (member or pending invitee).
        let dm_spaces: Vec<SpaceXgid> = self
            .spaces
            .iter()
            .filter(|(_, s)| {
                s.dm_constraints_active
                    && (s.members.contains_key(identity_id)
                        || s.pending_invites.contains_key(identity_id))
            })
            .map(|(id, _)| id.clone())
            .collect();

        let mut drained: Vec<Event> = Vec::new();
        for space_id in dm_spaces {
            // (1) Re-populate now that this party's home resolves.
            {
                let NodeRuntime { spaces, identity_registry, .. } = self;
                if let Some(state) = spaces.get_mut(&space_id) {
                    repopulate_dm_federation_nodes(state, identity_registry);
                }
            }
            // (2) Release any F-3-pending DM membership events held for this peer
            //     on this DM (verbatim reuse — D-076 by inheritance).
            drained.extend(self.drain_pending_by_federation_relationship(home_node, &space_id));
        }
        drained
    }

    /// MP-F11 (R3-D6) — establish a **regular-Space** federation relationship with
    /// `peer` for `space_id`, then drain any F-3-held content for it. The
    /// regular-Space generalization of the DM Design-Z hook
    /// ([`repopulate_dm_federation_after_identity`]).
    ///
    /// A late-federating peer's content is F-3-held because
    /// `SpaceState.federation_nodes` does not yet include the pushing peer — and
    /// the `state.federation_add` that would populate it can itself be
    /// predecessor-held (it references the sender's content tips, which are the
    /// held content): a mutual hold. This breaks the deadlock by populating
    /// `federation_nodes` from the **established relationship** (the F-3 authority,
    /// out-of-band of the held event), not from the event applying:
    /// 1. record the relationship (the durable authority that survives a
    ///    `derive_resolved` rebuild — see [`Self::federation_relationships`]);
    /// 2. add `peer` to a present, non-DM Space's `federation_nodes` — a
    ///    **legitimate relationship record**: only the established peer enters, so
    ///    F-3 still blocks third parties (the J-333 hole-lesson; **not** an
    ///    unconditional skip);
    /// 3. fire the proven `drain_pending_by_federation_relationship` hook (verbatim
    ///    reuse — D-076 by inheritance).
    ///
    /// Idempotent (the relationship set + the `federation_nodes` push both dedup).
    /// DM Spaces are untouched (they derive `federation_nodes` from members).
    /// Returns drained events for caller-side persistence.
    ///
    /// **[`repopulate_dm_federation_after_identity`]:** Self::repopulate_dm_federation_after_identity
    pub fn establish_federation_relationship(
        &mut self,
        space_id: &SpaceXgid,
        peer: &NodeXgid,
    ) -> Vec<Event> {
        // (1) Record the relationship (the durable F-3 authority).
        self.federation_relationships
            .entry(space_id.clone())
            .or_default()
            .insert(peer.clone());
        // (2) Populate federation_nodes directly for a present, non-DM Space
        //     (additive + deduped — a legitimate relationship record).
        if let Some(state) = self.spaces.get_mut(space_id) {
            if !state.dm_constraints_active && !state.federation_nodes.iter().any(|n| n == peer) {
                state.federation_nodes.push(peer.clone());
            }
        }
        // (3) Release any F-3-held content for this (peer, space).
        self.drain_pending_by_federation_relationship(peer, space_id)
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
        // SE-D6: `range(0)` (trait) replaces the inherent `values()`;
        // `topological_sort` reorders, so append order in is fine.
        topological_sort(store.range(0).unwrap_or_default())
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

/// Build the `identity_id → home_node_id` map the resolution layers (3 / 5a /
/// 5b) consult, sourced live from the identity registry (M8 C2 / CP-C).
///
/// Built per call (no cache): a rebuild happens only on cold-start
/// `rehydrate_space_from_store` or a detected conflict in `ingest_event` — both
/// rare relative to message traffic — so the construction cost is negligible
/// against the rebuild it feeds, and a cache would need invalidation on every
/// identity register / replicate. The algorithm's `HashMap<String, String>`
/// parameter is unchanged (SR-D3 — Pass-2 XGID widening is its own arc).
fn build_identity_home_nodes(registry: &IdentityRegistry) -> HashMap<String, String> {
    registry
        .all()
        .into_iter()
        .map(|r| (r.identity_id.as_str().to_string(), r.home_node.as_str().to_string()))
        .collect()
}

/// MP-F1b (F1B-D1/D2/D3, Design Z) — populate a DM Space's `federation_nodes`
/// from its current resolved **parties' home nodes**, where parties = members
/// ∪ pending invitees (invariant E amended: a DM's federation set = its parties'
/// home nodes). **DM-only**; `apply_federation_add` stays intact
/// (`DmFederationNotAllowed`) — no third-party node ever receives DM content.
/// Idempotent (full replace), so it re-fires safely at every apply site in
/// `ingest_event` + `rehydrate_space_from_store` + the identity-replicate hook.
///
/// **Why parties, not just members (Design Z).** A DM federates *late* (the
/// relationship forms at membership-apply, after the federation handshake). The
/// counterparty is a **seeded pending invitee** from create (`from_dm_space_create`),
/// and `apply_invite` rejects any further DM invites, so `pending_invites` is
/// exactly the one counterparty → parties is exactly the 2-party set. Including
/// the pending invitee's home from create means: (a) the receiving node's F-3
/// gate already has the joiner's home in `federation_nodes`, so the bootstrap
/// `membership.join` passes F-3 **with no skip** (F-3 stays the guard — a
/// non-party's node is never in the set); and (b) the creator's pre-join DM
/// message pushes to the counterparty's home immediately (closes the gap-2 a3
/// case via the existing push path, no backfill).
///
/// - **F1B-D2** — the FULL parties' home-node set, **self-included** (cross-node
///   symmetric: both parties' nodes derive the identical set; the push path skips
///   self, so a self-entry is a graceful no-op in `apply_federation_push`).
/// - **F1B-D3** — a party whose record is NOT in this node's registry is
///   **omitted** (no crash, no guess, no fabricated home). That omission IS the
///   gate-B boundary: harness-seeded → resolves → federates; production stranger
///   → omitted → deferred behind the routed identity→home discovery arc (F1B-D5).
///   The identity-replicate hook re-fires the helper when a lagging record lands.
///
/// The set is sorted for determinism: the source maps are `HashMap`s
/// (non-deterministic iteration), so two derivations of the same DM must produce
/// a byte-identical `Vec<NodeXgid>` for the within-node `assert_converges` oracle.
fn repopulate_dm_federation_nodes(state: &mut SpaceState, registry: &IdentityRegistry) {
    if !state.dm_constraints_active {
        return; // DM-only — regular Spaces use apply_federation_add (untouched).
    }
    let mut nodes: Vec<NodeXgid> = Vec::new();
    // Parties = members ∪ pending invitees (invariant E). `apply_join` moves the
    // joiner from pending_invites to members in one apply, so the union is stable
    // across that transition (never momentarily drops a party).
    for id in state.members.keys().chain(state.pending_invites.keys()) {
        if let Some(rec) = registry.get(id) {
            if !nodes.contains(&rec.home_node) {
                nodes.push(rec.home_node.clone());
            }
        }
        // else: unresolvable party → omit (F1B-D3 boundary).
    }
    nodes.sort();
    state.federation_nodes = nodes;
}

/// MP-F11 (R3-D6) — re-populate a REGULAR Space's `federation_nodes` from the
/// established federation relationships, so the relationship survives a
/// `derive_resolved` rebuild (which re-folds the log and would otherwise drop a
/// peer whose `state.federation_add` is predecessor-held). **Additive** (union
/// with whatever `apply_federation_add` already produced during the fold; deduped)
/// and a **legitimate relationship record** — only established peers enter, so F-3
/// still blocks third parties. DM Spaces are untouched (they use
/// [`repopulate_dm_federation_nodes`]). Sibling to the DM helper, sourced from the
/// out-of-band relationship record instead of the member set.
fn repopulate_regular_federation_nodes(
    state: &mut SpaceState,
    relationships: &HashMap<SpaceXgid, std::collections::HashSet<NodeXgid>>,
) {
    if state.dm_constraints_active {
        return; // DM-only path is repopulate_dm_federation_nodes.
    }
    if let Some(peers) = relationships.get(&state.space_id) {
        for peer in peers {
            if !state.federation_nodes.iter().any(|n| n == peer) {
                state.federation_nodes.push(peer.clone());
            }
        }
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
    use xgen_common::xgid::{IdentityXgid, NodeXgid, RoomXgid, SpaceXgid, Xgid};

    use super::{DispatchOutcome, EventOrigin, KeyPackageError, NodeRuntime};
    use crate::{
        crypto::encoding,
        identity::{keypair, registry::IdentityRecord},
        space::state::{
            build_dm_space_create_event, build_membership_event, build_mls_commit_event,
            build_mls_group_init_event, build_mls_key_package_event, build_room_create_event,
            build_space_create_event, sign_event, SpaceState,
        },
        wire::types::{Event, EventType},
    };

    fn rdx(s: &str) -> RoomXgid {
        RoomXgid::from_xgid(Xgid::new(s.to_string()))
    }

    // ── Arc H PG-05 (C2) — KeyPackage pool + epoch advance over the live ingest ──

    #[test]
    fn mls_key_package_ingest_populates_pool_then_request_consumes() {
        let mut rt = NodeRuntime::new(keypair::generate());
        let alice = keypair::generate();
        let node_uri = rt.node_id.as_str().to_string();

        let space_ev = sign_event(
            build_space_create_event(&alice, "s", None, 1, &node_uri, None, true),
            &alice,
        );
        let sid = event_id_str(&space_ev);
        rt.ingest_event(space_ev);

        // mls.key_package upload — the ingest hook stores it in the Node pool.
        let kp_ev = sign_event(
            build_mls_key_package_event(
                &alice,
                &sid,
                "",
                vec![sid.clone()],
                "alice-dev1",
                "KP_BLOB",
                "2026-12-01T00:00:00.000Z",
            ),
            &alice,
        );
        rt.ingest_event(kp_ev);

        let alice_id = pubkey_uri(&alice);
        assert_eq!(rt.key_package_store.available_count(&alice_id, "alice-dev1"), 1);

        // §3.10.5 request consumes single-use; the empty pool then yields 5001.
        let p = rt
            .request_key_package(&alice_id, "alice-dev1", "2026-06-04T00:00:00.000Z")
            .unwrap();
        assert_eq!(p.mls_key_package, "KP_BLOB");
        assert_eq!(
            rt.request_key_package(&alice_id, "alice-dev1", "2026-06-04T00:00:00.000Z")
                .unwrap_err(),
            KeyPackageError::NotFound
        );
    }

    #[test]
    fn mls_commit_ingest_advances_room_epoch() {
        let mut rt = NodeRuntime::new(keypair::generate());
        let alice = keypair::generate();
        let node_uri = rt.node_id.as_str().to_string();

        let space_ev = sign_event(
            build_space_create_event(&alice, "s", None, 1, &node_uri, None, true),
            &alice,
        );
        let sid = event_id_str(&space_ev);
        rt.ingest_event(space_ev);
        let room_ev = sign_event(build_room_create_event(&alice, &sid, "general", None), &alice);
        let rid = event_id_str(&room_ev);
        rt.ingest_event(room_ev);
        rt.ingest_event(sign_event(
            build_mls_group_init_event(&alice, &sid, &rid, &rid),
            &alice,
        ));
        rt.ingest_event(sign_event(
            build_mls_commit_event(&alice, &sid, &rid, vec![rid.clone()], 1),
            &alice,
        ));

        let epoch = rt.spaces.get(&sdx(&sid)).unwrap().rooms.get(&rdx(&rid)).unwrap().mls_epoch;
        assert_eq!(epoch, Some(1), "mls.commit advanced the Node-tracked epoch via ingest");
    }

    // ── M8.7 (CC-D2/D3/D5) — concurrent-commit resolution ─────────────────────

    /// Build a Space + Room + `mls.group_init` and return `(rt, sid, rid, giid)`.
    /// `home_node` is fixed (the caller's choice) so the same `space_create`
    /// event can be ingested into two independent NodeRuntimes (convergence test).
    fn mls_room_with_genesis(
        alice: &ed25519_dalek::SigningKey,
        home_node: &str,
    ) -> (Event, Event, Event) {
        let space_ev = sign_event(
            build_space_create_event(alice, "s", None, 1, home_node, None, true),
            alice,
        );
        let sid = event_id_str(&space_ev);
        let room_ev = sign_event(build_room_create_event(alice, &sid, "general", None), alice);
        let rid = event_id_str(&room_ev);
        let gi_ev = sign_event(build_mls_group_init_event(alice, &sid, &rid, &rid), alice);
        (space_ev, room_ev, gi_ev)
    }

    /// Resolution unit: two members commit the same `1 → 2` advance at one
    /// frontier (siblings off the group_init, distinct nonces ⇒ distinct ids).
    /// The `(room, target_epoch)` conflict domain makes them a genuine conflict;
    /// `derive_resolved` admits only the Layer-5c lexicographic winner, so exactly
    /// one is applied and the tip equals the lexicographically-lower `event_id`.
    #[test]
    fn mls_concurrent_commit_frontier_resolves_to_one_lexicographic_winner() {
        let alice = keypair::generate();
        let mut rt = NodeRuntime::new(keypair::generate());
        let node_uri = rt.node_id.as_str().to_string();
        let (space_ev, room_ev, gi_ev) = mls_room_with_genesis(&alice, &node_uri);
        let sid = event_id_str(&space_ev);
        let rid = event_id_str(&room_ev);
        let giid = event_id_str(&gi_ev);
        rt.ingest_event(space_ev);
        rt.ingest_event(room_ev);
        rt.ingest_event(gi_ev);

        // Two concurrent commits, both 1 → 2, both off the group_init.
        let commit_a =
            sign_event(build_mls_commit_event(&alice, &sid, &rid, vec![giid.clone()], 1), &alice);
        let commit_b =
            sign_event(build_mls_commit_event(&alice, &sid, &rid, vec![giid.clone()], 1), &alice);
        let id_a = event_id_str(&commit_a);
        let id_b = event_id_str(&commit_b);
        assert_ne!(id_a, id_b, "distinct nonces ⇒ distinct event_ids");
        let winner = std::cmp::min(id_a.clone(), id_b.clone());

        rt.ingest_event(commit_a);
        rt.ingest_event(commit_b);

        let room = rt.spaces.get(&sdx(&sid)).unwrap().rooms.get(&rdx(&rid)).unwrap();
        assert_eq!(room.mls_epoch, Some(1), "both advance to epoch 1");
        assert_eq!(
            room.mls_commit_tip.as_ref().map(|e| e.as_str().to_string()),
            Some(winner),
            "tip is the lexicographic winner, not the last-folded event"
        );
    }

    /// Headline convergence repro (CC-D5): two NodeRuntimes ingest the SAME two
    /// concurrent `1 → 2` commits in OPPOSITE orders and converge — asserted via
    /// the `RoomState` `Eq` oracle on `(mls_epoch, mls_commit_tip)`. This is also
    /// the **sensitivity witness**: revert the `MlsCommit` state-key arm and the
    /// two commits become unconflicted → each node's tip is its own last-folded
    /// commit → the tuples diverge (RED). Restored ⇒ GREEN. A counter-only design
    /// (no `mls_commit_tip`) would converge on `mls_epoch = 2` either way and stay
    /// green — the vacuity CC-D5 exists to defeat.
    #[test]
    fn mls_concurrent_commit_two_nodes_converge_on_winner_tip() {
        let alice = keypair::generate();
        // Fixed home_node so both runtimes ingest byte-identical create events.
        let home = "xgen://pubkey/ed25519:HOME";
        let (space_ev, room_ev, gi_ev) = mls_room_with_genesis(&alice, home);
        let sid = event_id_str(&space_ev);
        let rid = event_id_str(&room_ev);
        let giid = event_id_str(&gi_ev);

        // Two concurrent commits, built once so both nodes see identical events.
        let commit_a =
            sign_event(build_mls_commit_event(&alice, &sid, &rid, vec![giid.clone()], 1), &alice);
        let commit_b =
            sign_event(build_mls_commit_event(&alice, &sid, &rid, vec![giid.clone()], 1), &alice);
        let id_a = event_id_str(&commit_a);
        let id_b = event_id_str(&commit_b);
        let winner = std::cmp::min(id_a.clone(), id_b.clone());

        // Node X ingests [A, B]; Node Y ingests [B, A] — distinct node_ids.
        let mut rt_x = NodeRuntime::new(keypair::generate());
        let mut rt_y = NodeRuntime::new(keypair::generate());
        for ev in [&space_ev, &room_ev, &gi_ev] {
            rt_x.ingest_event(ev.clone());
            rt_y.ingest_event(ev.clone());
        }
        rt_x.ingest_event(commit_a.clone());
        rt_x.ingest_event(commit_b.clone());
        rt_y.ingest_event(commit_b);
        rt_y.ingest_event(commit_a);

        let room_x = rt_x.spaces.get(&sdx(&sid)).unwrap().rooms.get(&rdx(&rid)).unwrap();
        let room_y = rt_y.spaces.get(&sdx(&sid)).unwrap().rooms.get(&rdx(&rid)).unwrap();

        // Convergence: both nodes agree on epoch AND canonical commit identity.
        assert_eq!(
            (&room_x.mls_epoch, &room_x.mls_commit_tip),
            (&room_y.mls_epoch, &room_y.mls_commit_tip),
            "two nodes converge on (mls_epoch, mls_commit_tip) regardless of ingest order"
        );
        assert_eq!(room_x.mls_epoch, Some(1));
        assert_eq!(
            room_x.mls_commit_tip.as_ref().map(|e| e.as_str().to_string()),
            Some(winner),
            "converged tip is the lexicographic winner"
        );
    }

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
            build_space_create_event(&alice, "test-space", None, 1, node.node_id.as_str(), None, false),
            &alice,
        );

        let peer_key = keypair::generate();
        let peer_id = ndx(&pubkey_uri(&peer_key));

        let outcome = node.dispatch_event(space_ev, EventOrigin::ReceivedViaFederation, Some(&peer_id));
        if let DispatchOutcome::Rejected(reason) = &outcome {
            let reason = &reason.reason;
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
            let reason = &reason.reason;
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
            build_space_create_event(&alice, "test-space", None, 1, node.node_id.as_str(), None, false),
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
            build_space_create_event(&alice, "test-space", None, 1, node.node_id.as_str(), None, false),
            &alice,
        );

        let outcome = node.dispatch_event(space_ev, EventOrigin::LocallySubmitted, None);
        if let DispatchOutcome::Rejected(reason) = &outcome {
            let reason = &reason.reason;
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
            let reason = &reason.reason;
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
            DispatchOutcome::Rejected(reason) => {
                let reason = reason.reason;
                assert!(
                    reason.contains("space not found"),
                    "expected F-4 step 1 'space not found' for federation_add against unknown Space; got: {reason}"
                );
            }
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
            build_space_create_event(&alice, "fed-space", None, 1, node.node_id.as_str(), None, false),
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
            build_space_create_event(&alice, "local-space", None, 1, node.node_id.as_str(), None, false),
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
            build_space_create_event(&alice, "twice-space", None, 1, node.node_id.as_str(), None, false),
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
            build_space_create_event(&alice, "fed-space", None, 1, node.node_id.as_str(), None, false),
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
            build_space_create_event(&alice, "boot-space", None, 1, node.node_id.as_str(), None, false),
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
        );
        node.drain_pending_by_federation_relationship(
            &peer_id,
            &nothing,
        );
    }

    // ── MP-F11 (R3-D6) spine: regular-Space establish populate + drain ────────

    /// MP-F11 spine #1 — establishing a regular-Space federation relationship
    /// drains the F-3-held content AND populates `federation_nodes` so subsequent
    /// content from that peer passes F-3. RED-on-revert on either half:
    /// - revert the populate (step 2 of `establish_federation_relationship`) →
    ///   (b)+(c) fail (the peer is absent from `federation_nodes`; the subsequent
    ///   event is F-3-held);
    /// - revert the drain (step 3) → (a) fails (the held content stays buffered).
    #[test]
    fn mp_f11_regular_space_populate_on_establish_drains() {
        use crate::space::state::build_room_create_event;
        let alice = keypair::generate();
        let mut node = cold_node_with_registered(&alice);

        // Alice's REGULAR Space (dm_constraints_active = false).
        let space_ev = sign_event(
            build_space_create_event(&alice, "fed-space", None, 1, node.node_id.as_str(), None, false),
            &alice,
        );
        let space_id = event_id_str(&space_ev);
        node.ingest_event(space_ev);
        assert!(
            !node.spaces[space_id.as_str()].dm_constraints_active,
            "setup: must be a regular Space"
        );

        // A late-federating peer pushes a room_create → F-3-held (peer not yet in
        // federation_nodes; the federation_add that would populate it is not here).
        let peer = keypair::generate();
        let peer_id = ndx(&pubkey_uri(&peer));
        let r1 = sign_event(build_room_create_event(&alice, &space_id, "general", None), &alice);
        let r1_id = event_id_str(&r1);
        let held = node.dispatch_event(r1, EventOrigin::ReceivedViaFederation, Some(&peer_id));
        assert!(matches!(held, DispatchOutcome::HeldPending), "content must be F-3-held pre-establish");
        assert!(node.pending[space_id.as_str()].contains(&r1_id));

        // MP-F11 — establish the relationship: record + populate + drain.
        node.establish_federation_relationship(&sdx(&space_id), &peer_id);

        // (a) the held content drained (left the buffer).
        assert!(
            !node.pending[space_id.as_str()].contains(&r1_id),
            "(a) MP-F11: establish must drain the F-3-held content"
        );
        // (b) the peer is now in federation_nodes (a legitimate relationship record).
        assert!(
            node.spaces[space_id.as_str()].federation_nodes.iter().any(|n| n.as_str() == peer_id.as_str()),
            "(b) MP-F11: establish must populate federation_nodes with the peer"
        );
        // (c) a SUBSEQUENT content event from the peer now passes F-3 (not held).
        let r2 = sign_event(build_room_create_event(&alice, &space_id, "general2", None), &alice);
        let after = node.dispatch_event(r2, EventOrigin::ReceivedViaFederation, Some(&peer_id));
        assert!(
            matches!(after, DispatchOutcome::Accepted { .. }),
            "(c) MP-F11: subsequent content from the established peer must pass F-3, got {after:?}"
        );
    }

    /// MP-F11 spine #2 (the hole-closed assertion) — establishing a relationship
    /// with peer A does NOT open F-3 for a THIRD party B: B's content stays
    /// F-3-held and B never enters `federation_nodes`. RED-on-revert: an over-broad
    /// populate (a blanket F-3 skip / adding any sender) would let B's content
    /// apply → this fails. Mirrors MP-F1b's `..._third_party_dm_join_..._blocked`.
    #[test]
    fn mp_f11_third_party_regular_space_content_blocked_by_f3() {
        use crate::space::state::build_room_create_event;
        let alice = keypair::generate();
        let mut node = cold_node_with_registered(&alice);

        let space_ev = sign_event(
            build_space_create_event(&alice, "fed-space", None, 1, node.node_id.as_str(), None, false),
            &alice,
        );
        let space_id = event_id_str(&space_ev);
        node.ingest_event(space_ev);

        // Establish federation with peer A (the legitimate relationship).
        let peer_a = keypair::generate();
        let peer_a_id = ndx(&pubkey_uri(&peer_a));
        node.establish_federation_relationship(&sdx(&space_id), &peer_a_id);

        // A THIRD party B (NOT established) pushes content → must be F-3-held.
        let peer_b = keypair::generate();
        let peer_b_id = ndx(&pubkey_uri(&peer_b));
        let r_b = sign_event(build_room_create_event(&alice, &space_id, "fromB", None), &alice);
        let outcome = node.dispatch_event(r_b, EventOrigin::ReceivedViaFederation, Some(&peer_b_id));
        assert!(
            matches!(outcome, DispatchOutcome::HeldPending),
            "MP-F11 hole-closed: third-party content must stay F-3-held, got {outcome:?}"
        );
        // B must NOT have entered federation_nodes (only the established A is in).
        assert!(
            !node.spaces[space_id.as_str()].federation_nodes.iter().any(|n| n.as_str() == peer_b_id.as_str()),
            "MP-F11 hole-closed: a non-established third party must NOT be in federation_nodes"
        );
        assert!(
            node.spaces[space_id.as_str()].federation_nodes.iter().any(|n| n.as_str() == peer_a_id.as_str()),
            "the established peer A IS in federation_nodes"
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
            build_space_create_event(&alice, "b3-space", None, 1, node.node_id.as_str(), None, false),
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
            build_space_create_event(&alice, "b3-space", None, 1, node.node_id.as_str(), None, false),
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
            build_space_create_event(&alice, "b3-space", None, 1, node.node_id.as_str(), None, false),
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
            build_space_create_event(&alice, "b3-space", None, 1, node.node_id.as_str(), None, false),
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
            build_space_create_event(&alice, "b3-space", None, 1, node.node_id.as_str(), None, false),
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
    // INV-EXP (C3) — MockClock + the `Clock` trait (for `.now_utc()`) drive the
    // aged-Space repro deterministically.
    use xgen_common::clock::{Clock, MockClock};
    use xgen_common::xgid::{EventXgid, IdentityXgid, NodeXgid, SpaceXgid, Xgid};

    use super::{DispatchOutcome, EventOrigin, NodeRuntime};
    use crate::{
        crypto::encoding,
        dag::store::StoreInitError,
        identity::{keypair, registry::IdentityRecord},
        message::exchange::build_message_text_event,
        space::state::{
            build_dm_space_create_event, build_federation_add_event, build_membership_event,
            build_room_create_event, build_space_create_event, build_thread_create_event,
            sign_event, thread_id_from_event_id,
        },
        wire::types::{Event, EventType, ThreadStatus},
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
            build_space_create_event(&alice, "p7-5-amend-space", None, 1, node.node_id.as_str(), None, false),
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

    /// MP-F6 (M10.5-D2/D3) — the dispatch-level banned pre-check makes the reply
    /// honest. A banned identity's space re-join returns
    /// `DispatchOutcome::Rejected` (PermissionDenied-class, wire 4000) instead of
    /// the pre-M10.5 *accepted-but-inert* `Accepted` (is_ok=true while
    /// `derive_resolved` silently drops the join via `apply_join`'s `banned`
    /// consult, state.rs:1003).
    ///
    /// RED-on-revert: delete the banned pre-check in `dispatch_event`'s
    /// `MembershipJoin` block (runtime.rs ~:1388) and the `Rejected` assertion
    /// fails — the code returns `Accepted { .. }` with bob absent from members
    /// (the exact MP-F6 symptom: honest end-state, dishonest reply).
    #[test]
    fn dispatch_banned_join_rejected_not_accepted_but_inert() {
        let (mut node, space_id, _room_id, alice) = setup_space_with_room();
        let sx = sdx(&space_id);
        let bob = keypair::generate();
        let bob_id = pubkey_uri(&bob);
        node.register_identity(make_record(&bob, node.node_id.as_str()))
            .unwrap();

        // bob joins (open), then alice bans him — raw ingest (skips validation) to
        // reach the banned state through the real appliers.
        let tip0 = node.dag_tips(&sx).first().cloned().unwrap();
        let mut join =
            build_membership_event(&bob, &space_id, "", EventType::MembershipJoin, json!({}));
        join.prev_events = vec![EventXgid::from_xgid(Xgid::new(tip0))];
        let join = sign_event(join, &bob);
        let join_id = event_id_str(&join);
        node.ingest_event(join);
        assert!(
            node.spaces[&sx].members.contains_key(&idx(&bob_id)),
            "precondition: bob is a member before the ban"
        );

        let mut ban = build_membership_event(
            &alice,
            &space_id,
            "",
            EventType::MembershipBan,
            json!({ "target_identity": bob_id }),
        );
        ban.prev_events = vec![EventXgid::from_xgid(Xgid::new(join_id))];
        let ban = sign_event(ban, &alice);
        node.ingest_event(ban);
        assert!(node.spaces[&sx].banned.contains(&idx(&bob_id)), "bob is banned");
        assert!(
            !node.spaces[&sx].members.contains_key(&idx(&bob_id)),
            "the ban cascade removed bob from members"
        );

        // bob re-joins → the pre-check rejects it (honest reply), not
        // accepted-but-inert.
        let current_tip = node.dag_tips(&sx).first().cloned().unwrap();
        let mut rejoin =
            build_membership_event(&bob, &space_id, "", EventType::MembershipJoin, json!({}));
        rejoin.prev_events = vec![EventXgid::from_xgid(Xgid::new(current_tip))];
        let rejoin = sign_event(rejoin, &bob);
        let outcome = node.dispatch_event(rejoin, EventOrigin::LocallySubmitted, None);
        match outcome {
            DispatchOutcome::Rejected(info) => {
                assert_eq!(
                    info.code, 4000,
                    "banned join → PermissionDenied-class 4000 (M10.5-D3, no new wire code)"
                );
            }
            other => panic!(
                "banned re-join must be Rejected (honest reply), not {other:?} \
                 (Accepted would be the MP-F6 accepted-but-inert bug)"
            ),
        }
        // Protected state unchanged: bob is still not a member.
        assert!(
            !node.spaces[&sx].members.contains_key(&idx(&bob_id)),
            "a banned bob must not become a member"
        );
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

    /// MP-F3-D6 — a re-submitted duplicate returns `DispatchOutcome::Duplicate`
    /// and leaves the store + SpaceState byte-identical (the duplicate changed
    /// nothing). Proves the dedup-at-dispatch gate (F3-D1/D2) + that the
    /// early-return is state-neutral (the D-076 discharge at the unit level).
    #[test]
    fn duplicate_event_returns_duplicate_outcome_state_unchanged() {
        let (mut node, space_id, room_id, alice) = setup_space_with_room();
        let sx = sdx(&space_id);
        let tip = node.dag_tips(&sx).first().cloned().unwrap();
        let msg = sign_event(
            build_message_text_event(&alice, &space_id, &room_id, vec![tip], "hello"),
            &alice,
        );

        let first = node.dispatch_event(msg.clone(), EventOrigin::LocallySubmitted, None);
        assert!(
            matches!(first, DispatchOutcome::Accepted { .. }),
            "first submit must be Accepted; got {:?}",
            first
        );
        let len_after_first = node.stores.get(&sx).unwrap().len();
        let state_after_first = node.spaces.get(&sx).cloned().unwrap();

        // Re-submit the identical signed event.
        let second = node.dispatch_event(msg, EventOrigin::LocallySubmitted, None);
        assert!(
            matches!(second, DispatchOutcome::Duplicate),
            "second (identical) submit must be Duplicate; got {:?}",
            second
        );
        assert_eq!(
            node.stores.get(&sx).unwrap().len(),
            len_after_first,
            "duplicate must not append to the store"
        );
        assert_eq!(
            node.spaces.get(&sx).cloned().unwrap(),
            state_after_first,
            "duplicate must leave SpaceState byte-identical (apply was a no-op)"
        );
    }

    /// MP-F3-D7 — side-effect-skip safety (the D5 build-time obligation as a
    /// test). After an event drains a buffered dependent (Step 6), re-dispatching
    /// that same event returns `Duplicate` and fires NO second drain (no
    /// `additional_persisted`, the pending buffer stays empty). Confirms the
    /// early-return loses no needed effect: the first ingest already fired the
    /// drain keyed on the event's own id.
    #[test]
    fn duplicate_dispatch_fires_no_second_drain() {
        let (mut node, space_id, room_id, alice) = setup_space_with_room();
        let sx = sdx(&space_id);
        let tip = node.dag_tips(&sx).first().cloned().unwrap();

        let msg_a = sign_event(
            build_message_text_event(&alice, &space_id, &room_id, vec![tip], "A"),
            &alice,
        );
        let msg_a_id = event_id_str(&msg_a);
        let msg_b = sign_event(
            build_message_text_event(&alice, &space_id, &room_id, vec![msg_a_id.clone()], "B"),
            &alice,
        );

        // Buffer B (predecessor A absent), then dispatch A → drains B (Step 6).
        assert!(matches!(
            node.dispatch_event(msg_b, EventOrigin::LocallySubmitted, None),
            DispatchOutcome::HeldPending
        ));
        let out_a = node.dispatch_event(msg_a.clone(), EventOrigin::LocallySubmitted, None);
        match out_a {
            DispatchOutcome::Accepted { additional_persisted, .. } => {
                assert_eq!(additional_persisted.len(), 1, "A must drain B");
            }
            other => panic!("expected Accepted draining B; got {:?}", other),
        }

        // Re-dispatch A (now a stored duplicate) → Duplicate, no second drain.
        let again = node.dispatch_event(msg_a, EventOrigin::LocallySubmitted, None);
        assert!(
            matches!(again, DispatchOutcome::Duplicate),
            "re-dispatch of the drain-trigger must be Duplicate; got {:?}",
            again
        );
        // The pending buffer is empty (B already drained at first ingest; the
        // duplicate fired no further drain).
        let pending_empty = node
            .pending
            .get(&sx)
            .map(|b| b.is_empty())
            .unwrap_or(true);
        assert!(pending_empty, "duplicate must not leave anything buffered");
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

        // M8 C2 gate: chain invite → space-join → room-join causally (each off
        // the running DAG tip). These are all the same membership state key
        // (keyed by bob); without prev_events they would be concurrent and the
        // resolving apply path (SR-D1) would treat them as a conflict and drop
        // the join. Real clients always set prev_events to the current tips —
        // this fixture now does the same (raw ingest_event skips validation, so
        // the linkage must be supplied here).
        let tip0 = node.dag_tips(&sdx(&space_id)).first().cloned().unwrap();
        let mut invite = build_membership_event(
            &alice,
            &space_id,
            "",
            EventType::MembershipInvite,
            json!({ "target_identity": bob_id, "role": "member" }),
        );
        invite.prev_events = vec![EventXgid::from_xgid(Xgid::new(tip0))];
        let invite = sign_event(invite, &alice);
        let invite_id = event_id_str(&invite);
        node.ingest_event(invite);

        // Bob joins at Space level (room_id empty), referencing the invite.
        let mut bob_space_join =
            build_membership_event(&bob, &space_id, "", EventType::MembershipJoin, json!({}));
        bob_space_join.prev_events = vec![EventXgid::from_xgid(Xgid::new(invite_id))];
        let bob_space_join = sign_event(bob_space_join, &bob);
        let bob_space_join_id = event_id_str(&bob_space_join);
        node.ingest_event(bob_space_join);

        // Bob joins at Room level (room_id non-empty), referencing the space-join
        // — required so Step 11b membership-check passes when his
        // post-Identity-arrival re-dispatch hits a Room-context event.
        // node.spaces[space_id.as_str()].rooms is HashMap<RoomXgid, _> post-Pass-1; project
        // the key to String at the &str-API boundary.
        let room_id_for_join: String = node.spaces[space_id.as_str()]
            .rooms
            .keys()
            .next()
            .unwrap()
            .as_str()
            .to_string();
        let mut bob_room_join = build_membership_event(
            &bob,
            &space_id,
            &room_id_for_join,
            EventType::MembershipJoin,
            json!({}),
        );
        bob_room_join.prev_events = vec![EventXgid::from_xgid(Xgid::new(bob_space_join_id))];
        let bob_room_join = sign_event(bob_room_join, &bob);
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
            node.drain_pending_by_identity(&idx(&bob_id));

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
            build_space_create_event(&alice, "t1-space", None, 1, rt.node_id.as_str(), None, false),
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

    #[test]
    fn ensure_store_failure_never_silently_falls_back_to_vanilla() {
        // SE-SUB-D4 — an engine open failure must NOT yield a vanilla RAM store
        // (the false-durability this milestone exists to kill).
        let mut rt = NodeRuntime::new(keypair::generate());
        rt.set_store_factory(Box::new(|_| {
            Err(StoreInitError::EngineOpen("boom".to_string()))
        }));
        assert!(rt.engine_owns_durability, "set_store_factory marks engine durability active");
        let sid = sdx("xgen://hash/sha256:fail");
        assert!(rt.ensure_store(&sid).is_err());
        assert!(
            !rt.stores.contains_key(&sid),
            "no vanilla store materialises under a failed engine open"
        );
    }

    #[test]
    fn default_store_factory_is_vanilla_and_behaviour_neutral() {
        // SE-SUB-D5 — a fresh NodeRuntime uses the infallible vanilla factory;
        // ingest works exactly as before (engine_owns_durability stays false).
        let mut rt = NodeRuntime::new(keypair::generate());
        assert!(!rt.engine_owns_durability);
        let sid = sdx("xgen://hash/sha256:v");
        assert!(rt.ensure_store(&sid).is_ok());
        assert!(rt.stores.contains_key(&sid));
    }

    #[test]
    fn storage_advert_defaults_to_vanilla_and_is_settable() {
        // SE-D8 — the advert defaults to the vanilla backend; xgen-node sets it
        // from the gate result at startup.
        use xgen_common::state::StorageAdvert;
        let mut rt = NodeRuntime::new(keypair::generate());
        assert_eq!(rt.storage_advert.engine, "vanilla");
        assert_eq!(rt.storage_advert.assurance, "best_effort");
        assert_eq!(rt.storage_advert.asserts_tier, 1);
        rt.set_storage_advert(StorageAdvert {
            engine: "sqlite".to_string(),
            assurance: "durable".to_string(),
            asserts_tier: 2,
        });
        assert_eq!(rt.storage_advert.engine, "sqlite");
        assert_eq!(rt.storage_advert.asserts_tier, 2);
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
            build_space_create_event(&alice, "t2-space", None, 1, rt.node_id.as_str(), None, false),
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
            build_space_create_event(&alice, "t3-space", None, 1, rt.node_id.as_str(), None, false),
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
            build_space_create_event(&alice, "t4-space", None, 1, rt.node_id.as_str(), None, false),
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
            build_space_create_event(&alice, "t4-space-fed", None, 1, rt.node_id.as_str(), None, false),
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

    // ── PG-13 (Arc D, C1) — tier-gate on join ──────────────────────────────
    //
    // Helper: build a bob-signed space-level MembershipJoin chained off the
    // Space's current DAG tip (so validate_event passes steps 8–12 and the
    // dispatch reaches the step-4 tier-gate).
    fn bob_space_join(node: &NodeRuntime, space_id: &str, bob: &ed25519_dalek::SigningKey) -> Event {
        let tip = node.dag_tips(&sdx(space_id)).first().cloned().unwrap();
        let mut join =
            build_membership_event(bob, space_id, "", EventType::MembershipJoin, json!({}));
        join.prev_events = vec![EventXgid::from_xgid(Xgid::new(tip))];
        sign_event(join, bob)
    }

    /// PG-13 baseline no-op: a Tier-1 joiner into a Tier-1 Space passes the
    /// gate (`verify_tier_assertion(1, 1) = Ok`). Pins the no-op so a future
    /// PG-03 change to `assertion_tier_of` cannot silently regress the
    /// baseline join path.
    #[test]
    fn pg13_tier1_join_passes_gate() {
        let (mut node, space_id, _room_id, _alice) = setup_space_with_room();
        // bob's record has trust_assertion: None → assertion_tier_of → 1.
        let bob = keypair::generate();
        node.register_identity(make_record(&bob, node.node_id.as_str()))
            .unwrap();
        assert_eq!(node.spaces[space_id.as_str()].auth_tier, 1, "fixture Space is Tier-1");

        let outcome = node.dispatch_event(
            bob_space_join(&node, &space_id, &bob),
            EventOrigin::LocallySubmitted,
            None,
        );
        assert!(
            matches!(outcome, DispatchOutcome::Accepted { new_joiner: Some(_), .. }),
            "Tier-1 join into Tier-1 Space must pass the tier-gate; got {:?}",
            outcome
        );
    }

    /// PG-13 teeth: a Tier-1 joiner into a synthetic Tier-2 Space is Rejected
    /// with wire 3030 (`tier_mismatch`). Built ahead of PG-03 — the joiner's
    /// assertion JSON is hand-set (`{"tier":1}`), no TrustAssertion struct
    /// needed. Proves the gate is live, not decorative.
    #[test]
    fn pg13_tier1_join_into_tier2_space_rejected_3030() {
        let (mut node, space_id, _room_id, _alice) = setup_space_with_room();
        // Raise the Space's required tier to 2 (no Tier-2 auth module exists
        // yet; the field is the slot contract the gate reads).
        node.spaces.get_mut(&sdx(&space_id)).unwrap().auth_tier = 2;

        // bob asserts tier 1 — below the Space's required tier 2.
        let bob = keypair::generate();
        let mut rec = make_record(&bob, node.node_id.as_str());
        rec.trust_assertion = Some(json!({ "tier": 1 }));
        node.register_identity(rec).unwrap();

        let outcome = node.dispatch_event(
            bob_space_join(&node, &space_id, &bob),
            EventOrigin::LocallySubmitted,
            None,
        );
        match outcome {
            DispatchOutcome::Rejected(reason) => {
                // MP-F2 — wire 3030 now rides the structured RejectInfo field.
                assert_eq!(reason.code, 3030, "MP-F2 tier reject must carry wire 3030; got {}", reason.code);
                let reason = reason.reason;
                assert!(
                    reason.contains("3030") && reason.contains("tier_mismatch"),
                    "rejection must carry wire 3030 tier_mismatch; got {:?}",
                    reason
                );
            }
            other => panic!("Tier-1 join into Tier-2 Space must be Rejected; got {:?}", other),
        }
        // The gate has teeth: bob is NOT a member.
        assert!(
            !node.spaces[space_id.as_str()].is_member(&pubkey_uri(&bob)),
            "rejected joiner must not have been added to the Space"
        );
    }

    /// PG-13 acceptance at Tier 2: a Tier-2 joiner into a Tier-2 Space passes
    /// the gate (`verify_tier_assertion(2, 2) = Ok`) and is admitted.
    #[test]
    fn pg13_tier2_join_into_tier2_space_accepted() {
        let (mut node, space_id, _room_id, _alice) = setup_space_with_room();
        node.spaces.get_mut(&sdx(&space_id)).unwrap().auth_tier = 2;

        let bob = keypair::generate();
        let mut rec = make_record(&bob, node.node_id.as_str());
        rec.trust_assertion = Some(json!({ "tier": 2 }));
        node.register_identity(rec).unwrap();

        let outcome = node.dispatch_event(
            bob_space_join(&node, &space_id, &bob),
            EventOrigin::LocallySubmitted,
            None,
        );
        assert!(
            matches!(outcome, DispatchOutcome::Accepted { new_joiner: Some(_), .. }),
            "Tier-2 join into Tier-2 Space must pass the tier-gate; got {:?}",
            outcome
        );
    }

    // ── M8.5-B (INV-D6) — invite validity gates (dispatch step 4) ──────────────
    //
    // Build an alice-signed (owner) `membership.invite` naming `target_uri`,
    // chained off the Space tip so validate_event passes steps 8–13 and the
    // dispatch reaches the step-4 over-ceiling gate (3045). `valid_until` is set
    // in content only when `Some`.
    fn alice_invite(
        node: &NodeRuntime,
        space_id: &str,
        alice: &ed25519_dalek::SigningKey,
        target_uri: &str,
        valid_until: Option<&str>,
    ) -> Event {
        let tip = node.dag_tips(&sdx(space_id)).first().cloned().unwrap();
        let mut content = json!({ "target_identity": target_uri, "role": "member" });
        if let Some(vu) = valid_until {
            content["valid_until"] = json!(vu);
        }
        let mut inv =
            build_membership_event(alice, space_id, "", EventType::MembershipInvite, content);
        inv.prev_events = vec![EventXgid::from_xgid(Xgid::new(tip))];
        sign_event(inv, alice)
    }

    fn rfc3339(dt: chrono::DateTime<chrono::Utc>) -> String {
        dt.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
    }

    /// INV-D6 / 3045: an invite whose `valid_until` exceeds the invitee's tier
    /// ceiling (T1 = 14d) is Rejected at ingest with wire 3045 — the Node never
    /// silently clamps (D-065). 30 days from now > 14d ceiling.
    #[test]
    fn inv_d6_invite_over_ceiling_rejected_3045() {
        let (mut node, space_id, _room_id, alice) = setup_space_with_room();
        let bob = keypair::generate();
        node.register_identity(make_record(&bob, node.node_id.as_str()))
            .unwrap();
        let vu = rfc3339(chrono::Utc::now() + chrono::Duration::days(30));
        let outcome = node.dispatch_event(
            alice_invite(&node, &space_id, &alice, &pubkey_uri(&bob), Some(&vu)),
            EventOrigin::LocallySubmitted,
            None,
        );
        match outcome {
            DispatchOutcome::Rejected(reason) => {
                let reason = reason.reason;
                assert!(
                    reason.contains("3045") && reason.contains("invite_validity_exceeds_max"),
                    "over-ceiling invite must carry wire 3045; got {:?}",
                    reason
                );
            }
            other => panic!("over-ceiling invite must be Rejected; got {:?}", other),
        }
        // Gate has teeth: bob is not a pending invitee.
        assert!(
            !node.spaces[space_id.as_str()].pending_invites.contains_key(&idx(&pubkey_uri(&bob))),
            "rejected over-ceiling invite must not seed a pending invite"
        );
    }

    /// INV-D6: an invite within the T1 ceiling (7d < 14d) is accepted and seeds
    /// the pending invite carrying its `valid_until`.
    #[test]
    fn inv_d6_invite_within_ceiling_accepted() {
        let (mut node, space_id, _room_id, alice) = setup_space_with_room();
        let bob = keypair::generate();
        node.register_identity(make_record(&bob, node.node_id.as_str()))
            .unwrap();
        let vu = rfc3339(chrono::Utc::now() + chrono::Duration::days(7));
        let outcome = node.dispatch_event(
            alice_invite(&node, &space_id, &alice, &pubkey_uri(&bob), Some(&vu)),
            EventOrigin::LocallySubmitted,
            None,
        );
        assert!(
            matches!(outcome, DispatchOutcome::Accepted { .. }),
            "within-ceiling invite must be Accepted; got {:?}",
            outcome
        );
        let pi = node.spaces[space_id.as_str()]
            .pending_invites
            .get(&idx(&pubkey_uri(&bob)))
            .expect("bob must be a pending invitee");
        assert_eq!(pi.valid_until.as_deref(), Some(vu.as_str()), "valid_until must be stored");
    }

    /// INV-D6 / 3044: a join within the invite's validity window is accepted;
    /// a join after `valid_until` is Rejected with wire 3044 (`invite_expired`)
    /// and the joiner is NOT admitted. Two invitees, one in-window, one expired.
    #[test]
    fn inv_d6_join_within_validity_accepted_after_expiry_rejected_3044() {
        let (mut node, space_id, _room_id, alice) = setup_space_with_room();

        // In-window invitee (bob): valid_until 1h in the future.
        let bob = keypair::generate();
        node.register_identity(make_record(&bob, node.node_id.as_str())).unwrap();
        let future = rfc3339(chrono::Utc::now() + chrono::Duration::hours(1));
        let inv_b = alice_invite(&node, &space_id, &alice, &pubkey_uri(&bob), Some(&future));
        assert!(matches!(
            node.dispatch_event(inv_b, EventOrigin::LocallySubmitted, None),
            DispatchOutcome::Accepted { .. }
        ));
        let outcome_b = node.dispatch_event(
            bob_space_join(&node, &space_id, &bob),
            EventOrigin::LocallySubmitted,
            None,
        );
        assert!(
            matches!(outcome_b, DispatchOutcome::Accepted { new_joiner: Some(_), .. }),
            "in-window join must be Accepted; got {:?}",
            outcome_b
        );
        assert!(node.spaces[space_id.as_str()].is_member(&pubkey_uri(&bob)));

        // Expired invitee (carol): valid_until 1h in the past.
        let carol = keypair::generate();
        node.register_identity(make_record(&carol, node.node_id.as_str())).unwrap();
        let past = rfc3339(chrono::Utc::now() - chrono::Duration::hours(1));
        let inv_c = alice_invite(&node, &space_id, &alice, &pubkey_uri(&carol), Some(&past));
        // The invite itself is in-ceiling (past < now+14d), so it ingests fine.
        assert!(matches!(
            node.dispatch_event(inv_c, EventOrigin::LocallySubmitted, None),
            DispatchOutcome::Accepted { .. }
        ));
        let outcome_c = node.dispatch_event(
            bob_space_join(&node, &space_id, &carol),
            EventOrigin::LocallySubmitted,
            None,
        );
        match outcome_c {
            DispatchOutcome::Rejected(reason) => {
                let reason = reason.reason;
                assert!(
                    reason.contains("3044") && reason.contains("invite_expired"),
                    "expired join must carry wire 3044; got {:?}",
                    reason
                );
            }
            other => panic!("expired join must be Rejected; got {:?}", other),
        }
        assert!(
            !node.spaces[space_id.as_str()].is_member(&pubkey_uri(&carol)),
            "expired join must not admit the joiner"
        );
    }

    /// INV-D6 fail-closed (C2): on a **regular** (non-DM) Space, a pending
    /// invite with **no** `valid_until` is malformed/legacy — the join is
    /// rejected 3044, never treated as no-expiry (which would be the unbounded
    /// capability INV-D6 prevents). A real client always stamps post-C2.
    #[test]
    fn inv_d6_join_non_dm_absent_valid_until_rejected_3044() {
        let (mut node, space_id, _room_id, alice) = setup_space_with_room();
        let bob = keypair::generate();
        node.register_identity(make_record(&bob, node.node_id.as_str())).unwrap();
        // Invite with no valid_until: the 3045 over-ceiling gate doesn't fire
        // (nothing to exceed), so it ingests with a valid_until-less pending record.
        let inv = alice_invite(&node, &space_id, &alice, &pubkey_uri(&bob), None);
        assert!(matches!(
            node.dispatch_event(inv, EventOrigin::LocallySubmitted, None),
            DispatchOutcome::Accepted { .. }
        ));
        let outcome = node.dispatch_event(
            bob_space_join(&node, &space_id, &bob),
            EventOrigin::LocallySubmitted,
            None,
        );
        match outcome {
            DispatchOutcome::Rejected(reason) => {
                let reason = reason.reason;
                assert!(
                    reason.contains("3044") && reason.contains("invite_expired"),
                    "non-DM absent valid_until join must carry wire 3044; got {:?}",
                    reason
                );
            }
            other => panic!("non-DM absent valid_until join must be Rejected; got {:?}", other),
        }
        assert!(!node.spaces[space_id.as_str()].is_member(&pubkey_uri(&bob)));
    }

    /// INV-D6 DM exemption (C2): a DM Space's seeded counterparty invite has no
    /// `valid_until` by construction (`dm_constraints_active`). The join-gate
    /// exempts DM Spaces — the creator atomically seeds the 2-party counterparty,
    /// so there is no detached in-flight invite to misdirect; the absence of
    /// `valid_until` is the absence of the window it guards, not an omission.
    #[test]
    fn inv_d6_join_dm_absent_valid_until_accepted_exempt() {
        let alice = keypair::generate();
        let bob = keypair::generate();
        let node_key = keypair::generate();
        let mut node = NodeRuntime::new(node_key);
        node.register_identity(make_record(&alice, node.node_id.as_str())).unwrap();
        node.register_identity(make_record(&bob, node.node_id.as_str())).unwrap();
        let bob_id = pubkey_uri(&bob);
        let dm_ev = sign_event(
            build_dm_space_create_event(&alice, &bob_id, node.node_id.as_str()),
            &alice,
        );
        let space_id = event_id_str(&dm_ev);
        node.ingest_event(dm_ev);
        assert!(
            node.spaces[space_id.as_str()].dm_constraints_active,
            "fixture must be a DM Space"
        );
        // bob (seeded pending invitee, valid_until None) joins the DM.
        let tip = node.dag_tips(&sdx(&space_id)).first().cloned().unwrap();
        let mut join =
            build_membership_event(&bob, &space_id, "", EventType::MembershipJoin, json!({}));
        join.prev_events = vec![EventXgid::from_xgid(Xgid::new(tip))];
        let join = sign_event(join, &bob);
        let outcome = node.dispatch_event(join, EventOrigin::LocallySubmitted, None);
        assert!(
            matches!(outcome, DispatchOutcome::Accepted { new_joiner: Some(_), .. }),
            "DM join with no valid_until must be Accepted (DM-exempt); got {:?}",
            outcome
        );
        assert!(node.spaces[space_id.as_str()].is_member(&bob_id));
    }

    // ── PG-08 (Arc E) — Thread tier gates on thread.create (dispatch step 4) ───
    //
    // Build an alice-signed thread.create chained off the Room (so validate_event
    // passes steps 8–13 — alice is the Room creator hence a Room member — and the
    // dispatch reaches the step-4 thread tier gate).
    fn alice_thread_create(
        node: &NodeRuntime,
        space_id: &str,
        room_id: &str,
        alice: &ed25519_dalek::SigningKey,
        auth_tier_min: u32,
    ) -> Event {
        let tip = node.dag_tips(&sdx(space_id)).first().cloned().unwrap();
        sign_event(
            build_thread_create_event(alice, space_id, room_id, vec![tip], Some("topic"), auth_tier_min),
            alice,
        )
    }

    /// Honest Tier-1 no-op: a Tier-1 creator makes a Tier-1 Thread in a Tier-1
    /// Space → accepted and the Thread is inserted Open.
    #[test]
    fn pg08_thread_create_tier1_accepts_and_inserts() {
        let (mut node, space_id, room_id, alice) = setup_space_with_room();
        let ev = alice_thread_create(&node, &space_id, &room_id, &alice, 1);
        let thread_id = thread_id_from_event_id(&event_id_str(&ev));
        let outcome = node.dispatch_event(ev, EventOrigin::LocallySubmitted, None);
        assert!(
            matches!(outcome, DispatchOutcome::Accepted { .. }),
            "Tier-1 thread.create must accept; got {:?}",
            outcome
        );
        let thread = node.spaces[space_id.as_str()]
            .threads
            .get(&thread_id)
            .expect("thread inserted");
        assert_eq!(thread.status, ThreadStatus::Open);
    }

    /// Narrow-not-widen: a Thread may not lower the Room's tier floor. Synthetic
    /// Tier-2 Space + a Tier-1 Thread → rejected before the participation gate.
    #[test]
    fn pg08_thread_auth_tier_below_room_rejected() {
        let (mut node, space_id, room_id, alice) = setup_space_with_room();
        node.spaces.get_mut(&sdx(&space_id)).unwrap().auth_tier = 2;
        let ev = alice_thread_create(&node, &space_id, &room_id, &alice, 1);
        let outcome = node.dispatch_event(ev, EventOrigin::LocallySubmitted, None);
        match outcome {
            DispatchOutcome::Rejected(reason) => {
                let reason = reason.reason;
                assert!(
                    reason.contains("thread_auth_tier_below_room"),
                    "narrow-not-widen reject expected; got {reason:?}"
                );
            }
            other => panic!("expected narrow-not-widen rejection; got {other:?}"),
        }
    }

    /// Participation gate teeth (post-PG-03): a Tier-1 creator cannot create a
    /// Tier-2 Thread (auth_tier_min 2 satisfies narrow-not-widen vs the Tier-1
    /// Space, but the creator's own tier 1 < 2). Rejected with wire 3030.
    #[test]
    fn pg08_thread_create_above_creator_tier_rejected_3030() {
        let (mut node, space_id, room_id, alice) = setup_space_with_room();
        // alice's record has trust_assertion: None → assertion_tier_of → 1.
        let ev = alice_thread_create(&node, &space_id, &room_id, &alice, 2);
        let outcome = node.dispatch_event(ev, EventOrigin::LocallySubmitted, None);
        match outcome {
            DispatchOutcome::Rejected(reason) => {
                let reason = reason.reason;
                assert!(
                    reason.contains("3030") && reason.contains("tier_mismatch"),
                    "participation gate must reject with wire 3030; got {reason:?}"
                );
            }
            other => panic!("expected participation-tier rejection; got {other:?}"),
        }
        assert!(
            node.spaces[space_id.as_str()].threads.is_empty(),
            "rejected thread.create must not insert a Thread"
        );
    }

    /// M10.3 witness 4 — the participation gate's ACCEPT side, now reachable via a
    /// real higher-tier identity: a Tier-2 creator (validated tier-2 assertion)
    /// creates a Tier-2 Thread → accepted + inserted. The dormant gate fires
    /// correctly once an identity actually carries a higher tier (what the mock
    /// supplies). RED side = `pg08_thread_create_above_creator_tier_rejected_3030`.
    #[test]
    fn pg08_thread_create_tier2_creator_accepts() {
        let (mut node, space_id, room_id, alice) = setup_space_with_room();
        // alice was registered with trust_assertion: None (tier 1). Upgrade her to
        // a validated tier-2 assertion (as a mock T2 module would have issued and
        // this Node validated at registration).
        let mut rec = make_record(&alice, node.node_id.as_str());
        rec.trust_assertion = Some(json!({ "tier": 2 }));
        node.identity_registry.upsert(rec);
        assert_eq!(node.spaces[space_id.as_str()].auth_tier, 1, "fixture Space is Tier-1");

        let ev = alice_thread_create(&node, &space_id, &room_id, &alice, 2);
        let thread_id = thread_id_from_event_id(&event_id_str(&ev));
        let outcome = node.dispatch_event(ev, EventOrigin::LocallySubmitted, None);
        assert!(
            matches!(outcome, DispatchOutcome::Accepted { .. }),
            "Tier-2 creator must pass the participation gate for a Tier-2 Thread; got {:?}",
            outcome
        );
        assert!(
            node.spaces[space_id.as_str()].threads.contains_key(&thread_id),
            "accepted thread.create must insert the Thread"
        );
    }

    /// Arc E (PG-03) — `assertion_tier_of` reads the (now validated) stored tier.
    /// Guards a regression that would ignore the persisted assertion tier and
    /// silently flatten everyone to Tier 1 (which would defeat PG-13 at Tier 2–4).
    #[test]
    fn assertion_tier_of_reads_validated_tier() {
        let bob = keypair::generate();
        let mut rec = make_record(&bob, "xgen://pubkey/ed25519:NODE");
        // No assertion → cryptographic-identity baseline Tier 1.
        assert_eq!(super::assertion_tier_of(&rec), 1);
        // A validated Tier-2 assertion (the shape accept_registration persists).
        rec.trust_assertion = Some(json!({ "tier": 2 }));
        assert_eq!(super::assertion_tier_of(&rec), 2);
        // Absent/garbage tier field falls back to 1 (forward-compat / Local Node).
        rec.trust_assertion = Some(json!({ "claims": {} }));
        assert_eq!(super::assertion_tier_of(&rec), 1);
    }

    // ── INV-EXP — invite-expiry replay-gate (C2 origin-gate + D-090 clock) ──────
    //
    // These tests live beside the INV-D6 gate tests (above) because they reuse
    // the same `setup_space_with_room` / `alice_invite` / `bob_space_join` /
    // `make_record` helpers, and the MockClock dev-dep is on xgen-core. The
    // headline two-`NodeRuntime` repro is a two-Node test in spirit sibling to
    // `phase9_*`; a dedicated file under `node/tests/` would have to duplicate
    // these private helpers.
    //
    // The fix (C2): the 3044/3045 admission gates run iff `origin ==
    // LocallySubmitted` and are SKIPPED (invite/join proceeds to apply, not
    // rejected) on `ReceivedViaFederation`. The drain path carries the per-entry
    // origin (C1), so a buffered-then-drained event re-adjudicates against its
    // OWN origin, never a batch one. The 3044 comparison reads the injected
    // `self.clock.now_utc()` (D-090), so the aged-Space repro is deterministic.

    /// 2 hours — the gap that pushes B's clock past a 1-hour invite window.
    const TWO_HOURS: std::time::Duration = std::time::Duration::from_secs(2 * 60 * 60);

    /// Headline two-Node repro. H admits invite+join within the window (real-time
    /// local enforcement); a fresh peer B catches up after the invite has expired
    /// against B's clock. Pre-fix: B re-adjudicates expiry and rejects bob's
    /// historical join (3044) → bob present on H, absent on B (divergence).
    /// Post-fix: B skips the gate on federation replay → bob converges on both.
    /// Deterministic via the injected MockClock (D-090); no real sleep.
    #[test]
    fn inv_exp_federation_replay_preserves_membership() {
        let alice = keypair::generate();
        let bob = keypair::generate();
        let bob_uri = pubkey_uri(&bob);

        // ── Home node H — admits within window (cursor at base) ──
        let mut node_h = NodeRuntime::new(keypair::generate());
        let h_clock = std::sync::Arc::new(MockClock::new());
        node_h.set_clock(h_clock.clone());
        node_h.register_identity(make_record(&alice, node_h.node_id.as_str())).unwrap();
        node_h.register_identity(make_record(&bob, node_h.node_id.as_str())).unwrap();

        let space_ev = sign_event(
            build_space_create_event(&alice, "inv-exp-space", None, 1, node_h.node_id.as_str(), None, false),
            &alice,
        );
        let space_id = event_id_str(&space_ev);
        assert!(matches!(
            node_h.dispatch_event(space_ev.clone(), EventOrigin::LocallySubmitted, None),
            DispatchOutcome::Accepted { .. }
        ));

        // valid_until 1h from H's clock — within the 14d ceiling, in-window now.
        let valid_until = rfc3339(h_clock.now_utc() + chrono::Duration::hours(1));
        let invite_ev = alice_invite(&node_h, &space_id, &alice, &bob_uri, Some(&valid_until));
        assert!(matches!(
            node_h.dispatch_event(invite_ev.clone(), EventOrigin::LocallySubmitted, None),
            DispatchOutcome::Accepted { .. }
        ));
        let join_ev = bob_space_join(&node_h, &space_id, &bob);
        assert!(matches!(
            node_h.dispatch_event(join_ev.clone(), EventOrigin::LocallySubmitted, None),
            DispatchOutcome::Accepted { new_joiner: Some(_), .. }
        ));
        assert!(
            node_h.spaces[space_id.as_str()].is_member(&bob_uri),
            "H: bob admitted within the invite window"
        );

        // ── Fresh peer B — catches up 2h later, past valid_until ──
        let mut node_b = NodeRuntime::new(keypair::generate());
        let b_clock = std::sync::Arc::new(MockClock::new());
        b_clock.advance(TWO_HOURS); // B's wall-clock is now past valid_until
        node_b.set_clock(b_clock.clone());
        node_b.register_identity(make_record(&alice, node_b.node_id.as_str())).unwrap();
        node_b.register_identity(make_record(&bob, node_b.node_id.as_str())).unwrap();

        let h_peer = node_h.node_id.clone(); // NodeXgid
        let h_id_str = h_peer.as_str().to_string();

        // state.space_create is a DAG root → skips F-3; admitted via federation.
        assert!(matches!(
            node_b.dispatch_event(space_ev, EventOrigin::ReceivedViaFederation, Some(&h_peer)),
            DispatchOutcome::Accepted { .. }
        ));

        // Establish B↔H federation relationship for the Space so the replicated
        // invite/join pass F-3 (orthogonal to the expiry gate; same setup shape
        // the F-3 drain tests use).
        let b_node_key = node_b.node_keypair.clone();
        let fed_add = sign_event(
            build_federation_add_event(
                &b_node_key,
                &space_id,
                node_b.dag_tips(&sdx(&space_id)),
                h_id_str.as_str(),
                "xgen://hash/sha256:s",
                "0.1",
                "json",
            ),
            &b_node_key,
        );
        node_b.ingest_event(fed_add);
        assert!(
            node_b.spaces[space_id.as_str()].federation_nodes.iter().any(|n| n.as_str() == h_id_str),
            "B: federation relationship with H established"
        );

        // Replay H's invite + join via federation — B's clock is 2h past
        // valid_until. The fix skips 3045/3044 on ReceivedViaFederation.
        assert!(matches!(
            node_b.dispatch_event(invite_ev, EventOrigin::ReceivedViaFederation, Some(&h_peer)),
            DispatchOutcome::Accepted { .. }
        ));
        let join_outcome =
            node_b.dispatch_event(join_ev, EventOrigin::ReceivedViaFederation, Some(&h_peer));
        assert!(
            matches!(join_outcome, DispatchOutcome::Accepted { new_joiner: Some(_), .. }),
            "B: aged-invite join must be admitted on federation replay (gate skipped); got {:?}",
            join_outcome
        );

        // Convergence: bob is a member on BOTH nodes despite B's clock being
        // past valid_until.
        assert!(
            node_b.spaces[space_id.as_str()].is_member(&bob_uri),
            "B: bob present (converges with H) — the headline fix"
        );
        assert!(
            node_h.spaces[space_id.as_str()].is_member(&bob_uri),
            "H: bob still present"
        );
    }

    /// Enforcement intact: a local, directly-dispatched join after the invite
    /// expired is still rejected 3044 (the home node enforces the window in real
    /// time). Deterministic via the injected clock — no past wall-clock literal.
    #[test]
    fn inv_exp_local_direct_expired_join_rejected() {
        let (mut node, space_id, _room_id, alice) = setup_space_with_room();
        let clock = std::sync::Arc::new(MockClock::new());
        node.set_clock(clock.clone());
        let bob = keypair::generate();
        let bob_uri = pubkey_uri(&bob);
        node.register_identity(make_record(&bob, node.node_id.as_str())).unwrap();

        let valid_until = rfc3339(clock.now_utc() + chrono::Duration::hours(1));
        let inv = alice_invite(&node, &space_id, &alice, &bob_uri, Some(&valid_until));
        assert!(matches!(
            node.dispatch_event(inv, EventOrigin::LocallySubmitted, None),
            DispatchOutcome::Accepted { .. }
        ));

        clock.advance(TWO_HOURS); // now past valid_until

        let outcome = node.dispatch_event(
            bob_space_join(&node, &space_id, &bob),
            EventOrigin::LocallySubmitted,
            None,
        );
        match outcome {
            DispatchOutcome::Rejected(reason) => {
                let reason = reason.reason;
                assert!(
                    reason.contains("3044") && reason.contains("invite_expired"),
                    "local expired join must carry wire 3044; got {:?}",
                    reason
                );
            }
            other => panic!("local expired join must be Rejected; got {:?}", other),
        }
        assert!(!node.spaces[space_id.as_str()].is_member(&bob_uri));
    }

    /// Local-buffered→drained still enforces: a local join buffered on the
    /// missing signer Identity (F-10) carries `LocallySubmitted` per-entry; when
    /// drained after the invite expired, the gate runs at drain (= its first
    /// admission) and rejects. bob is not admitted.
    #[test]
    fn inv_exp_local_buffered_drain_still_enforces() {
        let (mut node, space_id, _room_id, alice) = setup_space_with_room();
        let clock = std::sync::Arc::new(MockClock::new());
        node.set_clock(clock.clone());

        // bob is invited within the window but NOT yet registered on this Node.
        let bob = keypair::generate();
        let bob_uri = pubkey_uri(&bob);
        let valid_until = rfc3339(clock.now_utc() + chrono::Duration::hours(1));
        let inv = alice_invite(&node, &space_id, &alice, &bob_uri, Some(&valid_until));
        assert!(matches!(
            node.dispatch_event(inv, EventOrigin::LocallySubmitted, None),
            DispatchOutcome::Accepted { .. }
        ));

        clock.advance(TWO_HOURS); // now past valid_until

        // bob's join arrives locally before bob's Identity is known → HeldPending
        // (F-10), stored origin LocallySubmitted.
        let join = bob_space_join(&node, &space_id, &bob);
        assert!(matches!(
            node.dispatch_event(join, EventOrigin::LocallySubmitted, None),
            DispatchOutcome::HeldPending
        ));

        // bob's Identity arrives → drain re-dispatches with the stored
        // LocallySubmitted origin → the 3044 gate runs at drain → rejected.
        node.register_identity(make_record(&bob, node.node_id.as_str())).unwrap();
        let drained = node.drain_pending_by_identity(&idx(&bob_uri));
        assert!(
            drained.is_empty(),
            "local buffered join must be rejected on drain (gate enforced), not accepted"
        );
        assert!(
            !node.spaces[space_id.as_str()].is_member(&bob_uri),
            "local-buffered→drained expired join must NOT admit bob"
        );
    }

    /// Federation-direct skips: a join received via federation after expiry is
    /// admitted (the replica trusts the home's admission). Gate skipped.
    #[test]
    fn inv_exp_federation_direct_skips_expiry() {
        let (mut node, space_id, _room_id, alice) = setup_space_with_room();
        let clock = std::sync::Arc::new(MockClock::new());
        node.set_clock(clock.clone());
        let bob = keypair::generate();
        let bob_uri = pubkey_uri(&bob);
        node.register_identity(make_record(&bob, node.node_id.as_str())).unwrap();

        let valid_until = rfc3339(clock.now_utc() + chrono::Duration::hours(1));
        let inv = alice_invite(&node, &space_id, &alice, &bob_uri, Some(&valid_until));
        assert!(matches!(
            node.dispatch_event(inv, EventOrigin::LocallySubmitted, None),
            DispatchOutcome::Accepted { .. }
        ));

        clock.advance(TWO_HOURS); // now past valid_until

        // peer_node_id = None isolates the expiry gate from F-3 (orthogonal;
        // same shape the drain path uses). origin = ReceivedViaFederation →
        // 3044 skipped → admitted.
        let outcome = node.dispatch_event(
            bob_space_join(&node, &space_id, &bob),
            EventOrigin::ReceivedViaFederation,
            None,
        );
        assert!(
            matches!(outcome, DispatchOutcome::Accepted { new_joiner: Some(_), .. }),
            "federation-direct aged join must be admitted (gate skipped); got {:?}",
            outcome
        );
        assert!(node.spaces[space_id.as_str()].is_member(&bob_uri));
    }

    /// Federation-buffered→drained skips (the per-entry-origin path): a
    /// federation join buffered on the missing signer Identity carries
    /// `ReceivedViaFederation` per-entry; on drain it re-dispatches with that
    /// stored origin → the 3044 gate is skipped → admitted despite the expired
    /// invite. This is the path C1's per-entry origin makes correct.
    #[test]
    fn inv_exp_federation_buffered_drain_skips_expiry() {
        let (mut node, space_id, _room_id, alice) = setup_space_with_room();
        let clock = std::sync::Arc::new(MockClock::new());
        node.set_clock(clock.clone());

        let bob = keypair::generate();
        let bob_uri = pubkey_uri(&bob);
        let valid_until = rfc3339(clock.now_utc() + chrono::Duration::hours(1));
        let inv = alice_invite(&node, &space_id, &alice, &bob_uri, Some(&valid_until));
        assert!(matches!(
            node.dispatch_event(inv, EventOrigin::LocallySubmitted, None),
            DispatchOutcome::Accepted { .. }
        ));

        clock.advance(TWO_HOURS); // now past valid_until

        // bob's join arrives via federation before bob's Identity is known →
        // HeldPending, stored origin ReceivedViaFederation.
        let join = bob_space_join(&node, &space_id, &bob);
        assert!(matches!(
            node.dispatch_event(join, EventOrigin::ReceivedViaFederation, None),
            DispatchOutcome::HeldPending
        ));

        node.register_identity(make_record(&bob, node.node_id.as_str())).unwrap();
        let drained = node.drain_pending_by_identity(&idx(&bob_uri));
        assert_eq!(
            drained.len(),
            1,
            "federation buffered join must be admitted on drain (gate skipped)"
        );
        assert!(
            node.spaces[space_id.as_str()].is_member(&bob_uri),
            "federation-buffered→drained aged join must admit bob (gate skipped)"
        );
    }

    /// Mixed-origin drain (the per-entry mechanism's guard). One LOCAL join and
    /// one FEDERATION join, both waiting on the SAME signer Identity (bob), in
    /// two Spaces on one Node, both with an expired invite. A single
    /// `resolve_identity` drain releases both; each re-dispatches with its OWN
    /// stored origin → the local one's gate runs (rejected), the federation
    /// one's gate is skipped (admitted).
    ///
    /// SENSITIVITY (its whole job): a regression that re-dispatched both with a
    /// single batch origin would put bob in BOTH spaces (batch = federation) or
    /// NEITHER (batch = local) — either way one of the two asserts below fails.
    /// This is the only test guarding C1's per-entry origin against a
    /// batch-origin revert.
    #[test]
    fn inv_exp_mixed_origin_drain_preserves_per_entry_origin() {
        let alice = keypair::generate();
        let bob = keypair::generate();
        let bob_uri = pubkey_uri(&bob);

        let mut node = NodeRuntime::new(keypair::generate());
        let clock = std::sync::Arc::new(MockClock::new());
        node.set_clock(clock.clone());
        node.register_identity(make_record(&alice, node.node_id.as_str())).unwrap();

        // Two Spaces, both alice-owned, both with bob invited within the window.
        let make_space_with_expiring_invite =
            |node: &mut NodeRuntime, name: &str, valid_until: &str| -> String {
                let space_ev = sign_event(
                    build_space_create_event(&alice, name, None, 1, node.node_id.as_str(), None, false),
                    &alice,
                );
                let sid = event_id_str(&space_ev);
                node.ingest_event(space_ev);
                let inv = alice_invite(node, &sid, &alice, &bob_uri, Some(valid_until));
                assert!(matches!(
                    node.dispatch_event(inv, EventOrigin::LocallySubmitted, None),
                    DispatchOutcome::Accepted { .. }
                ));
                sid
            };

        let valid_until = rfc3339(clock.now_utc() + chrono::Duration::hours(1));
        let space_local = make_space_with_expiring_invite(&mut node, "space-local", &valid_until);
        let space_fed = make_space_with_expiring_invite(&mut node, "space-fed", &valid_until);

        clock.advance(TWO_HOURS); // now past valid_until in both Spaces

        // bob's join into space_local buffered LocallySubmitted; into space_fed
        // buffered ReceivedViaFederation. Both wait on bob's (missing) Identity.
        let join_local = bob_space_join(&node, &space_local, &bob);
        assert!(matches!(
            node.dispatch_event(join_local, EventOrigin::LocallySubmitted, None),
            DispatchOutcome::HeldPending
        ));
        let join_fed = bob_space_join(&node, &space_fed, &bob);
        assert!(matches!(
            node.dispatch_event(join_fed, EventOrigin::ReceivedViaFederation, None),
            DispatchOutcome::HeldPending
        ));

        // One drain releases both; each re-dispatches with its OWN stored origin.
        node.register_identity(make_record(&bob, node.node_id.as_str())).unwrap();
        let _drained = node.drain_pending_by_identity(&idx(&bob_uri));

        assert!(
            node.spaces[space_fed.as_str()].is_member(&bob_uri),
            "federation join must be admitted (gate skipped on its stored origin)"
        );
        assert!(
            !node.spaces[space_local.as_str()].is_member(&bob_uri),
            "local join must be rejected (gate enforced on its stored origin) — \
             per-entry origin, not a batch origin"
        );
    }

    // ── M9.1 — Step 8.5 timestamp future-skew bound at dispatch (F1 / gap G6) ────
    //
    // Dispatch-level proof of design §5 (b)/(c)/(d) via the injected MockClock.
    // Step 8.5 lives in `validate_event` and is origin-blind, so the bound runs on
    // BOTH origins (M9.1-D2) — unlike the local-only INV-EXP gates above. The
    // witness dispatches via ReceivedViaFederation to lock the federation origin.

    /// Fixed home so the SAME `state.space_create` can be ingested into independent
    /// NodeRuntimes (the M8.7 byte-identical-create pattern).
    const M9_1_HOME: &str = "xgen://pubkey/ed25519:HOME";

    /// Signed `state.space_create` from `alice` with an explicit timestamp
    /// (re-signs so event_id covers the stamp → Step 8 passes; isolates Step 8.5).
    fn space_create_with_ts(
        alice: &ed25519_dalek::SigningKey,
        home: &str,
        ts: String,
    ) -> Event {
        let mut ev = build_space_create_event(alice, "m9_1-space", None, 1, home, None, false);
        ev.timestamp = ts;
        sign_event(ev, alice)
    }

    /// §5(b) — honest-skew same-verdict. Two nodes whose clocks differ by δ = 2 s
    /// reach the SAME verdict on the SAME event: the 5-min margin dwarfs δ, so a
    /// +10-min event is rejected on both and a base-stamped event is accepted on
    /// both. (Margin protects the verdict — convergence-safe under honest skew.)
    #[test]
    fn m9_1_honest_skew_same_verdict() {
        let alice = keypair::generate();

        let mut node_a = NodeRuntime::new(keypair::generate());
        let a_clock = std::sync::Arc::new(MockClock::new());
        node_a.set_clock(a_clock.clone());
        node_a.register_identity(make_record(&alice, M9_1_HOME)).unwrap();

        let mut node_b = NodeRuntime::new(keypair::generate());
        let b_clock = std::sync::Arc::new(MockClock::new());
        b_clock.advance(std::time::Duration::from_secs(2)); // B 2 s ahead of A
        node_b.set_clock(b_clock.clone());
        node_b.register_identity(make_record(&alice, M9_1_HOME)).unwrap();

        let base = a_clock.now_utc();

        // Over-ceiling (+10 min): both reject on the timestamp bound.
        let over = space_create_with_ts(
            &alice,
            M9_1_HOME,
            rfc3339(base + chrono::Duration::minutes(10)),
        );
        for (out, who) in [
            (node_a.dispatch_event(over.clone(), EventOrigin::LocallySubmitted, None), "A"),
            (node_b.dispatch_event(over.clone(), EventOrigin::LocallySubmitted, None), "B"),
        ] {
            match out {
                DispatchOutcome::Rejected(r) => {
                    // MP-F2 — the computed wire code now rides the structured
                    // RejectInfo (TimestampOutOfBounds → 3046), not just the prose.
                    assert_eq!(r.code, 3046, "{who}: MP-F2 timestamp reject must carry wire 3046; got {}", r.code);
                    let r = r.reason;
                    assert!(
                        r.contains("timestamp out of bounds"),
                        "{who}: +10min must reject on the timestamp bound; got {r}"
                    );
                }
                o => panic!("{who}: +10min must be rejected; got {o:?}"),
            }
        }

        // Base-stamped: both accept (the margin covers the 2 s skew).
        let ok = space_create_with_ts(&alice, M9_1_HOME, rfc3339(base));
        assert!(
            matches!(
                node_a.dispatch_event(ok.clone(), EventOrigin::LocallySubmitted, None),
                DispatchOutcome::Accepted { .. }
            ),
            "A: base-stamped event must be accepted"
        );
        assert!(
            matches!(
                node_b.dispatch_event(ok, EventOrigin::LocallySubmitted, None),
                DispatchOutcome::Accepted { .. }
            ),
            "B: base-stamped event must be accepted"
        );
    }

    /// §5(c) — catch-up leniency (the monotonicity property). An aged event
    /// (timestamp ≈ base) is accepted live at A (A.now ≈ base) AND accepted at B
    /// whose clock has advanced 2 days past it — `now` only moves forward, so an
    /// event's headroom only grows; the verdict never flips accept→reject on
    /// catch-up. This is what makes both-origins convergence-safe (design §4).
    #[test]
    fn m9_1_catchup_leniency() {
        let alice = keypair::generate();

        let mut node_a = NodeRuntime::new(keypair::generate());
        let a_clock = std::sync::Arc::new(MockClock::new());
        node_a.set_clock(a_clock.clone());
        node_a.register_identity(make_record(&alice, M9_1_HOME)).unwrap();

        let base = a_clock.now_utc();
        let aged = space_create_with_ts(&alice, M9_1_HOME, rfc3339(base));
        let sid = event_id_str(&aged);

        // Live at A: now ≈ base → accepted.
        assert!(
            matches!(
                node_a.dispatch_event(aged.clone(), EventOrigin::LocallySubmitted, None),
                DispatchOutcome::Accepted { .. }
            ),
            "A: event accepted live"
        );
        assert!(node_a.spaces.contains_key(sid.as_str()), "A: space present");

        // Catch-up at B: clock advanced 2 days past base → still accepted (B via
        // federation — DAG-root create skips F-3; locks the federation origin too).
        let mut node_b = NodeRuntime::new(keypair::generate());
        let b_clock = std::sync::Arc::new(MockClock::new());
        b_clock.advance(std::time::Duration::from_secs(2 * 24 * 60 * 60));
        node_b.set_clock(b_clock.clone());
        node_b.register_identity(make_record(&alice, M9_1_HOME)).unwrap();

        let a_peer = node_a.node_id.clone();
        assert!(
            matches!(
                node_b.dispatch_event(aged, EventOrigin::ReceivedViaFederation, Some(&a_peer)),
                DispatchOutcome::Accepted { .. }
            ),
            "B: aged event accepted on catch-up (now moved past it) — monotonicity"
        );
        assert!(
            node_b.spaces.contains_key(sid.as_str()),
            "B: space present (converges with A despite a 2-day-advanced clock)"
        );
    }

    /// §5(d) — sensitivity witness. With Step 8.5 present, a far-future event
    /// (here +10 min; the MP-A-15 injector stamps 2099) arriving via FEDERATION is
    /// rejected on the timestamp bound and is ABSENT from the applied log — locking
    /// both M9.1-D2 (bound runs on the federation origin) and the absence oracle.
    /// Reverting the Step 8.5 arm flips this to admitted (the event lands in
    /// `.spaces`) → RED; restored ⇒ GREEN. The test a no-op fix would leave green.
    #[test]
    fn m9_1_sensitivity_witness() {
        let alice = keypair::generate();
        let mut node = NodeRuntime::new(keypair::generate());
        let clock = std::sync::Arc::new(MockClock::new());
        node.set_clock(clock.clone());
        node.register_identity(make_record(&alice, M9_1_HOME)).unwrap();

        let base = clock.now_utc();
        let future = space_create_with_ts(
            &alice,
            M9_1_HOME,
            rfc3339(base + chrono::Duration::minutes(10)),
        );
        let sid = event_id_str(&future);

        let peer = ndx("xgen://pubkey/ed25519:PEER");
        let outcome = node.dispatch_event(future, EventOrigin::ReceivedViaFederation, Some(&peer));
        match outcome {
            DispatchOutcome::Rejected(r) => {
                // MP-F2 — the structured RejectInfo carries wire 3046.
                assert_eq!(r.code, 3046, "MP-F2 timestamp reject must carry wire 3046; got {}", r.code);
                let r = r.reason;
                assert!(
                    r.contains("timestamp out of bounds"),
                    "future federated event must reject on the timestamp bound; got {r}"
                );
            }
            o => panic!(
                "future federated event must be rejected (M9.1-D2 both-origins); got {o:?}"
            ),
        }
        assert!(
            !node.spaces.contains_key(sid.as_str()),
            "rejected future event must be absent from the applied log (absence oracle)"
        );
    }

    // ── MP-F1b — membership-driven DM federation (repopulate_dm_federation_nodes) ──
    //
    // NodeRuntime-level proof of the (iii) population helper (F1B-D1..D3, D7-C).
    // Joins/leaves are driven through `ingest_event` (the apply chokepoint the
    // helper hooks) directly — this lets the omit-unresolvable case admit a member
    // whose IdentityRecord is absent (the production gate-B case), which the
    // 13-step `dispatch_event` path would hold pending on the unknown signer.

    /// Create a DM on node A (alice creates with bob) via `ingest_event`. bob is
    /// seeded as a **pending invitee** (not yet a member). Returns the DM space_id
    /// + bob's identity_id. Caller controls registration.
    fn mp_f1b_dm_create(
        node: &mut NodeRuntime,
        alice: &ed25519_dalek::SigningKey,
        bob: &ed25519_dalek::SigningKey,
        creator_home: &str,
    ) -> (String, String) {
        let bob_id = pubkey_uri(bob);
        let dm_ev = sign_event(build_dm_space_create_event(alice, &bob_id, creator_home), alice);
        let space_id = event_id_str(&dm_ev);
        node.ingest_event(dm_ev);
        (space_id, bob_id)
    }

    /// bob's seeded space-join via `ingest_event`, chained off the DM tip.
    fn mp_f1b_dm_join(node: &mut NodeRuntime, bob: &ed25519_dalek::SigningKey, space_id: &str) {
        let tip = node.dag_tips(&sdx(space_id)).first().cloned().unwrap();
        let mut join =
            build_membership_event(bob, space_id, "", EventType::MembershipJoin, json!({}));
        join.prev_events = vec![EventXgid::from_xgid(Xgid::new(tip))];
        node.ingest_event(sign_event(join, bob));
    }

    /// Create + seeded join, both via `ingest_event`. Returns (space_id, bob_id).
    fn mp_f1b_dm_with_join(
        node: &mut NodeRuntime,
        alice: &ed25519_dalek::SigningKey,
        bob: &ed25519_dalek::SigningKey,
        creator_home: &str,
    ) -> (String, String) {
        let (space_id, bob_id) = mp_f1b_dm_create(node, alice, bob, creator_home);
        mp_f1b_dm_join(node, bob, &space_id);
        (space_id, bob_id)
    }

    /// F1B-D2 (Design Z) — a DM's `federation_nodes` = its **parties'** (members ∪
    /// pending invitees) home-node set, self-included, sorted. The pending-inclusion
    /// is the Z change: the set is `{A, B}` from **create** (bob pending), not only
    /// after bob joins — so the creator's pre-join message pushes to B, and the
    /// receiving F-3 gate already has B (no skip needed).
    #[test]
    fn mp_f1b_dm_federation_nodes_parties_resolvable() {
        let alice = keypair::generate();
        let bob = keypair::generate();
        let mut node = NodeRuntime::new(keypair::generate());
        let a_home = node.node_id.as_str().to_string(); // alice@A (this node, self)
        let b_home = "xgen://pubkey/ed25519:node-b-home"; // bob@B (a distinct home)
        node.register_identity(make_record(&alice, &a_home)).unwrap();
        node.register_identity(make_record(&bob, b_home)).unwrap();

        let mut expected = vec![ndx(&a_home), ndx(b_home)];
        expected.sort();

        // At CREATE — bob is a pending invitee, resolvable → already in the set.
        let (space_id, bob_id) = mp_f1b_dm_create(&mut node, &alice, &bob, &a_home);
        assert!(!node.spaces[space_id.as_str()].is_member(&bob_id), "bob is pending, not a member");
        assert_eq!(
            node.spaces[space_id.as_str()].federation_nodes, expected,
            "Z pending-inclusion: federation_nodes = {{A,B}} from create (bob pending)"
        );

        // After bob JOINS — bob moves pending→member; the set is unchanged.
        mp_f1b_dm_join(&mut node, &bob, &space_id);
        let dm = &node.spaces[space_id.as_str()];
        assert!(dm.is_member(&bob_id), "bob is now a member");
        assert_eq!(
            dm.federation_nodes, expected,
            "F1B-D2: stable across pending→member (parties = members ∪ pending)"
        );
    }

    /// F1B-D3 — a party whose IdentityRecord is NOT in this node's registry is
    /// OMITTED from `federation_nodes` (no crash, no guess). The gate-B boundary,
    /// witnessed at the unit level (honest-by-construction).
    #[test]
    fn mp_f1b_dm_federation_nodes_omits_unresolvable_party() {
        let alice = keypair::generate();
        let bob = keypair::generate();
        let mut node = NodeRuntime::new(keypair::generate());
        let a_home = node.node_id.as_str().to_string();
        node.register_identity(make_record(&alice, &a_home)).unwrap();
        // bob is NOT registered — the production case: a party we cannot resolve.

        let (space_id, bob_id) = mp_f1b_dm_with_join(&mut node, &alice, &bob, &a_home);
        let dm = &node.spaces[space_id.as_str()];
        assert!(
            dm.is_member(&bob_id),
            "bob is a member (joined) even though his record is unresolvable here"
        );
        assert_eq!(
            dm.federation_nodes,
            vec![ndx(&a_home)],
            "F1B-D3: an unresolvable party is omitted from federation_nodes"
        );
    }

    /// Z-bootstrap (gap 1) — bob's `membership.join` arriving **via federation**
    /// passes the F-3 gate **with no skip**, because bob's home (B) is already in
    /// `federation_nodes` (he is the seeded pending invitee → Z pending-inclusion).
    /// This is what closes the receiving-side gap *without* loosening F-3.
    #[test]
    fn mp_f1b_dm_join_via_federation_passes_f3_no_skip() {
        let alice = keypair::generate();
        let bob = keypair::generate();
        let mut node = NodeRuntime::new(keypair::generate());
        let a_home = node.node_id.as_str().to_string();
        let b_home = "xgen://pubkey/ed25519:node-b-home";
        node.register_identity(make_record(&alice, &a_home)).unwrap();
        node.register_identity(make_record(&bob, b_home)).unwrap();

        let (space_id, bob_id) = mp_f1b_dm_create(&mut node, &alice, &bob, &a_home);
        // bob's home B is in federation_nodes from create (pending-inclusion).
        assert!(
            node.spaces[space_id.as_str()].federation_nodes.contains(&ndx(b_home)),
            "B must be in federation_nodes via pending-inclusion (precondition for F-3 to pass)"
        );

        let tip = node.dag_tips(&sdx(&space_id)).first().cloned().unwrap();
        let mut join =
            build_membership_event(&bob, &space_id, "", EventType::MembershipJoin, json!({}));
        join.prev_events = vec![EventXgid::from_xgid(Xgid::new(tip))];
        let join = sign_event(join, &bob);
        // Federation-channel dispatch (peer = bob's home B). F-3 (step 2) consults
        // federation_nodes; B is present → passes, no skip.
        let outcome = node.dispatch_event(join, EventOrigin::ReceivedViaFederation, Some(&ndx(b_home)));
        assert!(
            matches!(outcome, DispatchOutcome::Accepted { .. }),
            "federation DM join must be Accepted (F-3 passes via pending-inclusion, no skip); got {:?}",
            outcome
        );
        assert!(node.spaces[space_id.as_str()].is_member(&bob_id), "bob is admitted");
    }

    /// Hole-closed (the F1B-D8 spine, proven) — a **third party's** DM join arriving
    /// via federation is **blocked by F-3** (their node is not in `federation_nodes`
    /// — only the 2 parties are). This proves Design Z needs **no** F-3 skip: F-3
    /// stays the guard, so the `apply_join` open-join cannot admit a 3rd member into
    /// a DM over the federation path.
    #[test]
    fn mp_f1b_third_party_dm_join_via_federation_blocked_by_f3() {
        let alice = keypair::generate();
        let bob = keypair::generate();
        let carol = keypair::generate(); // a third party
        let mut node = NodeRuntime::new(keypair::generate());
        let a_home = node.node_id.as_str().to_string();
        let b_home = "xgen://pubkey/ed25519:node-b-home";
        let c_home = "xgen://pubkey/ed25519:node-c-home";
        node.register_identity(make_record(&alice, &a_home)).unwrap();
        node.register_identity(make_record(&bob, b_home)).unwrap();
        node.register_identity(make_record(&carol, c_home)).unwrap(); // registered → not an F-10 confound

        // DM {alice, bob}; carol is NOT a party.
        let (space_id, _bob_id) = mp_f1b_dm_with_join(&mut node, &alice, &bob, &a_home);
        let carol_id = pubkey_uri(&carol);
        assert!(
            !node.spaces[space_id.as_str()].federation_nodes.contains(&ndx(c_home)),
            "carol's node C must NOT be in federation_nodes (she is not a party)"
        );

        let tip = node.dag_tips(&sdx(&space_id)).first().cloned().unwrap();
        let mut join =
            build_membership_event(&carol, &space_id, "", EventType::MembershipJoin, json!({}));
        join.prev_events = vec![EventXgid::from_xgid(Xgid::new(tip))];
        let join = sign_event(join, &carol);
        // Federation-channel dispatch (peer = carol's home C). F-3 (step 2) finds C
        // absent from federation_nodes → HeldPending (NOT admitted).
        let outcome = node.dispatch_event(join, EventOrigin::ReceivedViaFederation, Some(&ndx(c_home)));
        assert!(
            matches!(outcome, DispatchOutcome::HeldPending),
            "3rd-party federation DM join must be HeldPending by F-3 (Z needs no skip); got {:?}",
            outcome
        );
        assert!(
            !node.spaces[space_id.as_str()].is_member(&carol_id),
            "F-3 must block the 3rd-party join — carol is NOT a member (apply_join open-join never reached)"
        );
    }

    /// DM-only — the helper early-returns on a regular Space, so a regular Space's
    /// `federation_nodes` (managed by `apply_federation_add`) is never touched.
    #[test]
    fn mp_f1b_regular_space_federation_nodes_untouched() {
        let alice = keypair::generate();
        let bob = keypair::generate();
        let mut node = NodeRuntime::new(keypair::generate());
        let a_home = node.node_id.as_str().to_string();
        node.register_identity(make_record(&alice, &a_home)).unwrap();
        node.register_identity(make_record(&bob, "xgen://pubkey/ed25519:node-b-home")).unwrap();

        // A plain Space (dm_constraints_active = false; open-join per J-275).
        let create = sign_event(
            build_space_create_event(&alice, "s", None, 1, &a_home, None, false),
            &alice,
        );
        let space_id = event_id_str(&create);
        node.ingest_event(create);
        let tip = node.dag_tips(&sdx(&space_id)).first().cloned().unwrap();
        let mut join =
            build_membership_event(&bob, &space_id, "", EventType::MembershipJoin, json!({}));
        join.prev_events = vec![EventXgid::from_xgid(Xgid::new(tip))];
        node.ingest_event(sign_event(join, &bob));

        let sp = &node.spaces[space_id.as_str()];
        assert!(!sp.dm_constraints_active, "fixture must be a regular Space");
        assert!(
            sp.federation_nodes.is_empty(),
            "the DM-only helper must not populate a regular Space's federation_nodes (no federation_add → empty)"
        );
    }

    /// F1B-D7-C — the helper recomputes the full set at each membership-apply, so a
    /// leave shrinks `federation_nodes` for future events.
    #[test]
    fn mp_f1b_dm_federation_nodes_shrinks_on_leave() {
        let alice = keypair::generate();
        let bob = keypair::generate();
        let mut node = NodeRuntime::new(keypair::generate());
        let a_home = node.node_id.as_str().to_string();
        let b_home = "xgen://pubkey/ed25519:node-b-home";
        node.register_identity(make_record(&alice, &a_home)).unwrap();
        node.register_identity(make_record(&bob, b_home)).unwrap();

        let (space_id, bob_id) = mp_f1b_dm_with_join(&mut node, &alice, &bob, &a_home);
        let mut full = vec![ndx(&a_home), ndx(b_home)];
        full.sort();
        assert_eq!(node.spaces[space_id.as_str()].federation_nodes, full, "full set before leave");

        // bob leaves (causally after his join).
        let tip = node.dag_tips(&sdx(&space_id)).first().cloned().unwrap();
        let mut leave =
            build_membership_event(&bob, &space_id, "", EventType::MembershipLeave, json!({}));
        leave.prev_events = vec![EventXgid::from_xgid(Xgid::new(tip))];
        node.ingest_event(sign_event(leave, &bob));

        let dm = &node.spaces[space_id.as_str()];
        assert!(!dm.is_member(&bob_id), "bob has left the DM");
        assert_eq!(
            dm.federation_nodes,
            vec![ndx(&a_home)],
            "F1B-D7-C: a leave shrinks federation_nodes (bob's home gone)"
        );
    }
}

#[cfg(test)]
mod m8_c2_wiring_tests {
    //! M8 C2 — the live node apply path (`ingest_event` conflict gate +
    //! `rehydrate_space_from_store`) routes SpaceState derivation through
    //! `derive_resolved`. The algorithm-level convergence proof lives in
    //! `resolution::derive::tests`; these are the integration-level locks
    //! (SR-D5 secondary) on the runtime seam: concurrent same-key events
    //! converge regardless of ingest order, the cold-start rebuild is
    //! convergent, and the non-conflicting fast path is unchanged.
    use serde_json::json;
    use xgen_common::xgid::{EventXgid, IdentityXgid, SpaceXgid, Xgid};

    use super::NodeRuntime;
    use crate::{
        crypto::encoding,
        identity::keypair,
        space::state::{build_membership_event, build_space_create_event, sign_event},
        wire::types::{Event, EventType},
    };

    fn pubkey_uri(k: &ed25519_dalek::SigningKey) -> String {
        format!("xgen://pubkey/ed25519:{}", encoding::encode(k.verifying_key().as_bytes()))
    }
    fn sdx(s: &str) -> SpaceXgid {
        SpaceXgid::from_xgid(Xgid::new(s.to_string()))
    }
    fn idx(s: &str) -> IdentityXgid {
        IdentityXgid::from_xgid(Xgid::new(s.to_string()))
    }
    fn edx(s: &str) -> EventXgid {
        EventXgid::from_xgid(Xgid::new(s.to_string()))
    }
    fn eid(ev: &Event) -> String {
        ev.event_id.as_ref().unwrap().as_str().to_string()
    }

    /// `(space_create, join_bob, ban_bob, bob_id)`. `join` and `ban` both
    /// reference the create root → concurrent, same membership key (target bob).
    /// Built once so both arrival orders feed byte-identical events.
    fn ban_join_scenario() -> (Event, Event, Event, String) {
        let alice = keypair::generate();
        let bob = keypair::generate();
        let bob_id = pubkey_uri(&bob);
        let create = sign_event(
            build_space_create_event(&alice, "s", None, 1, "xgen://pubkey/ed25519:HOME", None, false),
            &alice,
        );
        let sid = eid(&create);
        let mut join = build_membership_event(&bob, &sid, "", EventType::MembershipJoin, json!({}));
        join.prev_events = vec![edx(&sid)];
        let join = sign_event(join, &bob);
        let mut ban = build_membership_event(
            &alice,
            &sid,
            "",
            EventType::MembershipBan,
            json!({ "target_identity": bob_id }),
        );
        ban.prev_events = vec![edx(&sid)];
        let ban = sign_event(ban, &alice);
        (create, join, ban, bob_id)
    }

    fn ingest_all(events: Vec<Event>) -> NodeRuntime {
        let mut node = NodeRuntime::new(keypair::generate());
        for ev in events {
            node.ingest_event(ev);
        }
        node
    }

    #[test]
    fn ingest_gate_converges_regardless_of_arrival_order() {
        let (create, join, ban, bob_id) = ban_join_scenario();
        let sid = eid(&create);

        let node_a = ingest_all(vec![create.clone(), join.clone(), ban.clone()]);
        let node_b = ingest_all(vec![create, ban, join]);

        let state_a = &node_a.spaces[&sdx(&sid)];
        let state_b = &node_b.spaces[&sdx(&sid)];
        assert_eq!(
            state_a, state_b,
            "the ingest conflict gate converges to one state regardless of arrival order"
        );
        // Layer 1: ban beats join — bob is banned, not a member.
        assert!(state_a.banned.contains(&idx(&bob_id)), "ban wins at the live seam");
        assert!(!state_a.members.contains_key(&idx(&bob_id)));
    }

    #[test]
    fn rehydrate_from_store_is_convergent() {
        let (create, join, ban, bob_id) = ban_join_scenario();
        let sid = eid(&create);
        let mut node = ingest_all(vec![create, join, ban]);

        let before = node.spaces[&sdx(&sid)].clone();
        // Cold-start rebuild from the store (the SE-SUB-D6 engine-startup path).
        node.rehydrate_space_from_store(&sdx(&sid));
        let after = &node.spaces[&sdx(&sid)];

        assert_eq!(&before, after, "rehydrate rebuilds the identical convergent snapshot");
        assert!(after.banned.contains(&idx(&bob_id)), "ban wins after cold-start rebuild");
    }

    #[test]
    fn ingest_fast_path_applies_non_conflicting_event_incrementally() {
        // create + a single causal invite → no concurrent same-key event → the
        // gate takes the fast incremental path (no rebuild), unchanged behaviour.
        let alice = keypair::generate();
        let bob_id = pubkey_uri(&keypair::generate());
        let create = sign_event(
            build_space_create_event(&alice, "s", None, 1, "xgen://pubkey/ed25519:HOME", None, false),
            &alice,
        );
        let sid = eid(&create);
        let mut node = NodeRuntime::new(keypair::generate());
        node.ingest_event(create);

        let mut invite = build_membership_event(
            &alice,
            &sid,
            "",
            EventType::MembershipInvite,
            json!({ "target_identity": bob_id, "role": "member" }),
        );
        invite.prev_events = vec![edx(&sid)];
        let invite = sign_event(invite, &alice);
        node.ingest_event(invite);

        assert!(
            node.spaces[&sdx(&sid)].pending_invites.contains_key(&idx(&bob_id)),
            "non-conflicting invite applied incrementally via the fast path"
        );
    }
}
