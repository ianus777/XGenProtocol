// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

// Message exchange — pipeline steps 8–13 and event acceptance (spec 3.2.6).
//
// Steps 1–7 (structural) are in wire/validation.rs.
// This module adds the crypto and state-dependent steps:
//
//   8.  event_id matches hash of canonical content
//   9.  all prev_events are known to this Node (→ hold pending if not)
//   10. no DAG structural violation (self-reference, root-type rules, fanin limit)
//   11. sender is a registered Identity and a Space/Room member
//   12. signature verifies against sender's public key
//   13. sender has permission to produce this EventType in this Room

use chrono::{SecondsFormat, Utc};
use ed25519_dalek::SigningKey;
use serde_json::json;
use thiserror::Error;
use xgen_common::xgid::{EventXgid, IdentityXgid, RoomXgid, SpaceXgid, Xgid};

use crate::{
    crypto::{encoding, hashing},
    dag::{
        graph::{is_dag_root_type, DagGraph, MAX_PREV_EVENTS},
        store::EventStore,
    },
    identity::registry::IdentityRegistry,
    space::{
        membership::{
            can_ban, can_change_space_info, can_delegate_ai_operator, can_invite, can_kick,
        },
        state::{verify_event_signature, SpaceState},
    },
    wire::{
        canonical::canonical_event_bytes,
        types::{Event, EventType},
    },
};

// ── Errors ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum ExchangeError {
    #[error("step 8: event_id does not match canonical content hash")]
    EventIdMismatch,

    #[error("step 9: unknown prev_events — event held pending")]
    HeldPending(Vec<EventXgid>),

    #[error("step 10: DAG structural violation — {0}")]
    DagError(String),

    #[error("step 11: sender is not a registered Identity")]
    UnknownSender,

    #[error("step 11: sender is not a Space member")]
    NotASpaceMember,

    #[error("step 11: sender is not a member of room '{0}'")]
    NotARoomMember(RoomXgid),

    #[error("step 12: signature verification failed")]
    SignatureFailure,

    #[error("step 13: permission denied for {0}")]
    PermissionDenied(String),

    /// Spec 3.6.10.4 — an AI Identity's Event violates a declared capability
    /// restriction. The `String` payload is the human-readable rule description
    /// (e.g. "dm_initiate disallowed"). Wire error code is 3042.
    #[error("ai capability violation: {0}")]
    AiCapabilityViolation(String),

    /// Spec 3.6.10.6 (M3) — an AI Identity violates a structural role rule, OR
    /// a role-related event has an invalid target (AI not actually an AI member,
    /// target not a member, wrong signer for delegate/revoke). The `String`
    /// payload is the rule description. Wire error code is 3041.
    #[error("ai role violation: {0}")]
    AiRoleViolation(String),

    /// M6 A4-D1 — a `membership.node_eject` / `membership.node_unban` whose
    /// `sender` is not the Space's home Node. Node-administrator force-eject
    /// authority is signature + `sender == space.home_node`, a first-class
    /// authority distinct from member-role permission (D-070). Wire code 3043.
    #[error("node-eject authority: sender is not the Space's home Node")]
    NodeEjectAuthority,

    #[error("event is missing event_id")]
    MissingEventId,
}

impl ExchangeError {
    /// Map this error to the protocol wire (code, name) tuple per the relevant
    /// spec section. Only errors that have a defined wire code are mapped;
    /// internal-only variants fall back to a generic transport error.
    pub fn to_wire_code(&self) -> Option<(u32, &'static str)> {
        match self {
            Self::AiCapabilityViolation(_) => Some((3042, "ai_capability_violation")),
            Self::AiRoleViolation(_) => Some((3041, "ai_role_violation")),
            Self::NodeEjectAuthority => Some((3043, "node_eject_authority")),
            _ => None,
        }
    }
}

// ── Validation pipeline steps 8–13 ───────────────────────────────────────────

/// Run pipeline steps 8–13 on an Event that has already passed steps 1–7.
///
/// Does NOT mutate `store` or `graph` — the caller must call `accept_event`
/// (or insert manually) after this returns `Ok`.
///
/// Returns `HeldPending(missing_ids)` when prev_events reference unknown Events;
/// the caller should buffer the event and request the missing predecessors.
///
/// **DEPRECATED at XGID Retrofit Pass 2 (J-125, design doc §4.2 Q5.b).** This is
/// the pre-F-4 legacy 13-step pipeline. Production code paths flow through
/// `validate_event` (the F-4 unified validation core); the only remaining callers
/// are `accept_event` (also deprecated) and xgen-core test fixtures. Removal is
/// scheduled as its own future audit-design-impl arc per D-071 — that arc audits
/// the test-fixture migration cost, locks the removal sequence, and lands the
/// removal alongside `accept_event` and (per Surface #5 Q5.4) candidate
/// `accept_message`. Pass 2 retypes this function minimally for call-site
/// compatibility; the function body's internal logic is unchanged.
#[deprecated(
    note = "Deprecated at XGID Retrofit Pass 2 (J-125). Use validate_event (F-4 unified core). Removal scheduled as own audit-design-impl arc per D-071."
)]
pub fn validate_steps_8_13(
    event: &Event,
    space: &SpaceState,
    id_registry: &IdentityRegistry,
    store: &dyn EventStore,
) -> Result<(), ExchangeError> {
    // Step 8 — event_id matches canonical content hash. event_id is
    // Option<EventXgid>; bind as typed reference, project to &str via .as_str()
    // for the hash comparison.
    let event_id = event
        .event_id
        .as_ref()
        .ok_or(ExchangeError::MissingEventId)?;
    let v = serde_json::to_value(event).expect("Event is always serialisable");
    let canonical = canonical_event_bytes(&v);
    let expected_id = hashing::hash_uri(&canonical);
    if event_id.as_str() != expected_id {
        return Err(ExchangeError::EventIdMismatch);
    }

    // Step 9 — all prev_events known to this Node. Collect missing EventXgids
    // as typed clones; HeldPending payload is Vec<EventXgid> post-Pass-2.
    let unknown: Vec<EventXgid> = event
        .prev_events
        .iter()
        .filter(|id| !store.contains(id))
        .cloned()
        .collect();
    if !unknown.is_empty() {
        return Err(ExchangeError::HeldPending(unknown));
    }

    // Step 10 — DAG structural rules (no mutation).
    validate_dag_structure(event)?;

    // Step 11 — sender is a registered Identity and a Space/Room member.
    // event.sender is IdentityXgid; bind as typed reference. IdentityRegistry
    // methods take &IdentityXgid post-Pass-2 (Surface #4); SpaceState methods
    // still take &str (defer to Pass 3, design doc §5.1), so we project via
    // .as_str() at the call-site boundary.
    let sender = &event.sender;
    if !id_registry.contains(sender) {
        return Err(ExchangeError::UnknownSender);
    }
    if !space.is_member(sender.as_str()) {
        return Err(ExchangeError::NotASpaceMember);
    }
    if !event.room_id.as_str().is_empty()
        && !space.is_room_member(sender.as_str(), event.room_id.as_str())
    {
        return Err(ExchangeError::NotARoomMember(event.room_id.clone()));
    }

    // Step 12 — signature verifies against the sender's embedded public key.
    if !verify_event_signature(event) {
        return Err(ExchangeError::SignatureFailure);
    }

    // Spec 3.6.10.4 — AI capability enforcement. After signature validation,
    // before permission validation. Applies only to Events from AI Identities.
    check_ai_capability(event, id_registry)?;

    // Spec 3.6.10.6 (M3) — target validation for AI operator events. Signer-role
    // checks live in `check_permission`; this verifies named targets exist as
    // Space members and the `ai_identity_id` is genuinely an AI Identity.
    check_ai_operator_targets(event, space, id_registry)?;

    // Step 13 — sender has permission to produce this EventType in this Room.
    check_permission(event, space)?;

    Ok(())
}

/// Enforce AI-specific event constraints (spec 3.6.10.4 capability flags +
/// spec 3.6.10.6 structural role rule). For human Identities (is_ai = false)
/// the function is a no-op.
///
/// Order matters: the M3 structural rule (AI cannot own a Space) is checked
/// first because it supersedes the D-059 capability-flag rule. The remaining
/// `dm_initiate` capability code is unreachable for AI senders of
/// `state.dm_space_create` after M3 — kept in place so the 3042 wire code
/// stays meaningful as a framework for any future re-enablement.
///
/// Exposed (`pub`) so the Node's `state.space_create` / `state.dm_space_create`
/// reception path can gate-keep these bootstrap events before constructing
/// the SpaceState, since the standard pipeline (`validate_steps_8_13`) requires
/// a pre-existing SpaceState that does not exist for a creation event.
pub fn check_ai_capability(
    event: &Event,
    id_registry: &IdentityRegistry,
) -> Result<(), ExchangeError> {
    // event.sender is IdentityXgid; pass typed reference per Surface #4 Q4.1.
    let record = match id_registry.get(&event.sender) {
        Some(r) => r,
        None => return Ok(()), // sender unknown — handled earlier as UnknownSender
    };
    if !record.is_ai {
        return Ok(());
    }

    // M3 (spec 3.6.10.6) — AI Identities MUST NOT be Space owners.
    // Reject any AI sender of state.space_create or state.dm_space_create.
    if matches!(
        event.event_type,
        EventType::StateSpaceCreate | EventType::StateDmSpaceCreate
    ) {
        return Err(ExchangeError::AiRoleViolation(
            "ai cannot create a space (ai-owned spaces are prohibited)".to_string(),
        ));
    }

    // D-059 capability-flag enforcement. After M3 this is unreachable for
    // `state.dm_space_create` because the structural check above always fires
    // first; preserved for forward-compat with any future role-model change.
    let caps = match record.ai_capabilities.as_ref() {
        Some(c) => c,
        // An AI record without capabilities is structurally invalid (step 8 at
        // registration prevents it), but if such a record somehow exists,
        // err on the side of restriction: deny capabilities that default to false.
        None => {
            if matches!(event.event_type, EventType::StateDmSpaceCreate) {
                return Err(ExchangeError::AiCapabilityViolation(
                    "dm_initiate disallowed".to_string(),
                ));
            }
            return Ok(());
        }
    };
    if matches!(event.event_type, EventType::StateDmSpaceCreate) && !caps.dm_initiate {
        return Err(ExchangeError::AiCapabilityViolation(
            "dm_initiate disallowed".to_string(),
        ));
    }
    // `spontaneous_post` is NOT Node-validated in Phase 2 (spec 3.6.10.4).
    Ok(())
}

/// Public re-export of `check_ai_operator_targets` for Node-side bootstrap
/// paths that need to gate AI operator events without going through the full
/// `validate_steps_8_13` pipeline (e.g. when the event is being accepted via
/// the catch-all branch in `xgen-node::process_inbound`).
pub fn check_ai_operator_targets_pub(
    event: &Event,
    space: &SpaceState,
    id_registry: &IdentityRegistry,
) -> Result<(), ExchangeError> {
    check_ai_operator_targets(event, space, id_registry)
}

/// Public re-export of `check_permission` for the same Node-side bootstrap use.
pub fn check_permission_pub(event: &Event, space: &SpaceState) -> Result<(), ExchangeError> {
    check_permission(event, space)
}

/// Spec 3.6.10.6 (M3) — validate target-side constraints for `state.ai_operator_delegate`
/// and `state.ai_operator_revoke`. Signer-role checks live in `check_permission`;
/// this function checks the named targets exist as Space members and that the
/// `ai_identity_id` is registered with `is_ai = true`.
///
/// Returns `AiRoleViolation` (wire code 3041) on any failure.
fn check_ai_operator_targets(
    event: &Event,
    space: &SpaceState,
    id_registry: &IdentityRegistry,
) -> Result<(), ExchangeError> {
    // JSON-extracted identity URIs lifted to typed IdentityXgid bindings at the
    // boundary per design principle §3 — IdentityRegistry methods consume the
    // typed reference (Surface #4 Q4.1); SpaceState.is_member still takes &str
    // (out of Pass 2 scope per §5.1; defer to Pass 3).
    match event.event_type {
        EventType::StateAiOperatorDelegate => {
            let ai_id_str = event.content["ai_identity_id"].as_str().ok_or_else(|| {
                ExchangeError::AiRoleViolation("missing ai_identity_id".to_string())
            })?;
            let new_op_str = event.content["new_operator_identity_id"]
                .as_str()
                .ok_or_else(|| {
                    ExchangeError::AiRoleViolation(
                        "missing new_operator_identity_id".to_string(),
                    )
                })?;
            let ai_id = IdentityXgid::from_xgid(Xgid::new(ai_id_str.to_string()));
            if !space.is_member(ai_id_str) {
                return Err(ExchangeError::AiRoleViolation(
                    "ai_identity_id is not a space member".to_string(),
                ));
            }
            if !space.is_member(new_op_str) {
                return Err(ExchangeError::AiRoleViolation(
                    "new_operator_identity_id is not a space member".to_string(),
                ));
            }
            let ai_record = id_registry.get(&ai_id).ok_or_else(|| {
                ExchangeError::AiRoleViolation(
                    "ai_identity_id not registered on this node".to_string(),
                )
            })?;
            if !ai_record.is_ai {
                return Err(ExchangeError::AiRoleViolation(
                    "ai_identity_id is not an ai identity".to_string(),
                ));
            }
            Ok(())
        }
        EventType::StateAiOperatorRevoke => {
            let ai_id_str = event.content["ai_identity_id"].as_str().ok_or_else(|| {
                ExchangeError::AiRoleViolation("missing ai_identity_id".to_string())
            })?;
            let ai_id = IdentityXgid::from_xgid(Xgid::new(ai_id_str.to_string()));
            if !space.is_member(ai_id_str) {
                return Err(ExchangeError::AiRoleViolation(
                    "ai_identity_id is not a space member".to_string(),
                ));
            }
            let ai_record = id_registry.get(&ai_id).ok_or_else(|| {
                ExchangeError::AiRoleViolation(
                    "ai_identity_id not registered on this node".to_string(),
                )
            })?;
            if !ai_record.is_ai {
                return Err(ExchangeError::AiRoleViolation(
                    "ai_identity_id is not an ai identity".to_string(),
                ));
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

/// Accept an Event: run all 13 pipeline steps, then insert into the DAG and store.
///
/// Callers are responsible for propagating the event to federated Nodes after
/// this returns `Ok`.
///
/// **DEPRECATED at XGID Retrofit Pass 2 (J-125, design doc §4.2 Q5.b — symmetric
/// application alongside `validate_steps_8_13`).** Production code paths flow
/// through the F-4 unified validation core (`validate_event`) plus a separate
/// ingest step at the dispatcher. Remaining callers are xgen-core test fixtures.
/// Removal scheduled as its own future audit-design-impl arc per D-071.
#[deprecated(
    note = "Deprecated at XGID Retrofit Pass 2 (J-125). Use validate_event (F-4 unified core) + explicit ingest. Removal scheduled as own audit-design-impl arc per D-071."
)]
pub fn accept_event(
    event: Event,
    space: &SpaceState,
    id_registry: &IdentityRegistry,
    store: &mut dyn EventStore,
    graph: &mut DagGraph,
) -> Result<(), ExchangeError> {
    // `validate_steps_8_13` is also deprecated; `accept_event` is its only
    // production-shape consumer alongside test fixtures. Both removed together
    // in the D-071 audit-design-impl arc.
    #[allow(deprecated)]
    validate_steps_8_13(&event, space, id_registry, &*store)?;
    graph
        .add_event(&event, &*store)
        .map_err(|e| ExchangeError::DagError(e.to_string()))?;
    store
        .append(event)
        .map_err(|e| ExchangeError::DagError(e.to_string()))?;
    Ok(())
}

// ── F-4 unified validation core (xgen_federation_propagation_design §7) ────────

/// Outcome of running the F-4 unified validation core on an Event.
///
/// Locked at design Pass 2 (2026-05-18) per `docs/xgen_federation_propagation_design.md`
/// §7.4 (decision Option 1), extended at Phase 6 (F-10, runbook §3.6.1 Lock B1)
/// to carry an optional `missing_identity` alongside `missing_predecessors`.
/// The validation core is non-mutating; the caller decides what to do with
/// the outcome:
/// - `Validated` → run semantic pre-checks and ingest.
/// - `HeldPending { missing_predecessors, missing_identity }` → buffer the
///   event with the listed missing predecessor event_ids and/or missing
///   signer identity_id (uniform 30 s timeout per F-4a / F-10a; shared
///   `PendingBuffer` per Space). At least one of the two fields is
///   non-empty; an event is not HeldPending if nothing is missing.
/// - `Rejected(err)` → log + drop; M6 (new) Phase 2 wires the wire-layer
///   rejection signal (envelope `event_id` on `TransportMessage::Error`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationOutcome {
    Validated,
    HeldPending {
        missing_predecessors: Vec<EventXgid>,
        missing_identity: Option<IdentityXgid>,
    },
    Rejected(ExchangeError),
}

/// F-4 unified validation core (`docs/xgen_federation_propagation_design.md` §7.4).
///
/// Runs structural and crypto checks uniformly across all event families. Does
/// NOT mutate state — the caller's post-validation handler does that.
///
/// Replaces the pre-F-4 asymmetry where Path A (messages) ran the full 13-step
/// pipeline while Paths B (`MembershipJoin`) and C (other state events) skipped
/// signature verification, timestamp check, and HeldPending integration. After
/// F-4, every event family reaches event-handling code only via this function.
///
/// `space` is `None` for Space-creation events (`StateSpaceCreate`,
/// `StateDmSpaceCreate`) because the `SpaceState` does not exist yet — the
/// post-validation handler builds it during ingest. For all other events
/// `space` MUST be `Some(...)` referencing the (parent) Space.
///
/// Sub-check coverage by event family:
///
/// | Check | Messages | Joins | Create | Other state |
/// |---|---|---|---|---|
/// | Step 8 event_id hash | ✓ | ✓ | ✓ | ✓ |
/// | Step 9 predecessors known (HeldPending on miss) | ✓ | ✓ | ✓ (vacuous; empty prev_events) | ✓ |
/// | Step 10 DAG structure | ✓ | ✓ | ✓ | ✓ |
/// | Step 11 sender registered | ✓ | ✓ | ✓ | ✓ |
/// | Step 11 sender is Space member | ✓ | — (event makes them a member) | — (no Space yet) | ✓ |
/// | Step 11 sender is Room member | ✓ | — | — | ✓ |
/// | Step 12 signature | ✓ | ✓ | ✓ | ✓ |
/// | Step 13 permission | ✓ | — | — | ✓ |
///
/// AI role + AI operator target/permission checks are NOT in the validation
/// core per design doc §7.7 — they live in step 4 (semantic pre-checks) of
/// the `process_inbound` dispatcher.
pub fn validate_event(
    event: &Event,
    space: Option<&SpaceState>,
    id_registry: &IdentityRegistry,
    store: &dyn EventStore,
    fed_add_via_federation: bool,
) -> ValidationOutcome {
    // Phase 7 B3 (locked 2026-05-20) — `state.federation_add` events arriving
    // via a federation channel (caller signals via `fed_add_via_federation`)
    // skip step 9 (predecessor presence), step 11 (sender registration AND
    // sender membership — both halves), and step 13 (sender permission).
    // Steps 8 (event_id hash), 10 (DAG structure), and 12 (signature
    // verification) still fire.
    //
    // Reasoning (sibling to Phase 7 B1's F-3 skip): federation_add IS the
    // relationship-establishing event. The predecessor check would deadlock
    // because federation_add's predecessors are themselves held on the
    // Phase 7.5 §6 federation-relationship trigger (predecessor-chain
    // deadlock). The sender-registration check fails because federation_add
    // is Node-authored under Phase 4 §3.4.1 Q3 overload (sender = Node URI)
    // and Node URIs are not inserted into IdentityRegistry by either of its
    // populating paths (`IdentityMessage::Register`,
    // `handle_incoming_replicate`); without the skip federation_add F-10-
    // buffers forever. The membership + permission checks fail because Node
    // URIs are by design not Space members.
    //
    // Authority chain: session-level handshake auth + step 12 signature
    // verification (decodes the pubkey from the sender URI directly via
    // `verify_event_signature` — pure crypto, no registry lookup, Q3-
    // overload transparent at this layer). See
    // `tasks/FEDERATION_PROPAGATION_PHASE_7_B3_AMENDMENT.md` §4.1 + §4.3 for
    // the full reasoning and §3 for the captured production-gap evidence.
    //
    // Scope is narrow: caller in `dispatch_event` sets the flag only when
    // `event.event_type == StateFederationAdd && peer_node_id.is_some()`.
    // Locally-submitted federation_add (M6 admin write path, future) retains
    // full validation per B3 §4.2.

    // Step 8 — event_id matches canonical content hash. event_id is Option<EventXgid>;
    // project to &str for comparison.
    let event_id = match event.event_id.as_ref().map(|e| e.as_str()) {
        Some(id) => id,
        None => return ValidationOutcome::Rejected(ExchangeError::MissingEventId),
    };
    let v = serde_json::to_value(event).expect("Event is always serialisable");
    let canonical = canonical_event_bytes(&v);
    let expected_id = hashing::hash_uri(&canonical);
    if event_id != expected_id {
        return ValidationOutcome::Rejected(ExchangeError::EventIdMismatch);
    }

    // Step 10 — DAG structural rules. Runs before step 9's predecessor lookup
    // so events with malformed prev_events fail synchronously rather than
    // potentially HeldPending-ing on a nonsense reference.
    if let Err(e) = validate_dag_structure(event) {
        return ValidationOutcome::Rejected(e);
    }

    // Step 9 — predecessors known (HeldPending on miss). Phase 6 (F-10):
    // identity-missing case below also produces HeldPending. The two checks
    // run sequentially; if predecessors are missing AND the sender is also
    // unknown, the predecessor check fires first (single return point per
    // pass). The buffered event re-validates on either arrival and the
    // second check fires on retry.
    //
    // Phase 7 B3 — skipped for federation_add via federation channel
    // (predecessor-chain deadlock — see B3 §3.1).
    if !fed_add_via_federation {
        // event.prev_events is Vec<EventXgid> (Pass 1); collect missing entries
        // as typed EventXgid clones — no projection through &str needed at the
        // Pass 2 boundary. Borrow<str> handles the store.contains(&str) lookup.
        let unknown: Vec<EventXgid> = event
            .prev_events
            .iter()
            .filter(|id| !store.contains(id))
            .cloned()
            .collect();
        if !unknown.is_empty() {
            return ValidationOutcome::HeldPending {
                missing_predecessors: unknown,
                missing_identity: None,
            };
        }
    }

    // Step 11 — sender is a registered Identity (universal). Phase 6 (F-10
    // Lock B1) changes this from a hard rejection to HeldPending so federation
    // events for first-contact peers buffer pending Identity-record arrival
    // via replication, rather than reject-then-re-pull. See design doc §13
    // (F-10) for the architectural reasoning and §13.6 for the
    // "Identity record never arrives" recovery path (30 s timeout → discard
    // → next sync_request / F-1a tip-exchange re-delivers).
    //
    // Phase 7 B3 — skipped for federation_add via federation channel
    // (Q3-overload — Node URIs are not inserted into IdentityRegistry;
    // see B3 §4.1).
    // event.sender is IdentityXgid; bind as typed reference. `id_registry.contains`
    // takes `&IdentityXgid` post-Pass-2 (Surface #4 Q4.2); `missing_identity` carries
    // the typed IdentityXgid clone, not a String projection.
    // M6 A4-D1 — node_eject / node_unban are Node-authored: `sender` is the home
    // Node keypair, which is NOT a registered Identity (same as federation_add via
    // federation, B3 §4.1). They skip the sender-registration HeldPending; their
    // authority is the dedicated home-node gate after step 12.
    let node_authored = matches!(
        event.event_type,
        EventType::MembershipNodeEject | EventType::MembershipNodeUnban
    );
    let sender = &event.sender;
    if !fed_add_via_federation && !node_authored && !id_registry.contains(sender) {
        return ValidationOutcome::HeldPending {
            missing_predecessors: Vec::new(),
            missing_identity: Some(sender.clone()),
        };
    }

    // Step 11 — sender membership checks. Skipped for:
    //   * MembershipJoin — the event itself makes the sender a member.
    //   * StateSpaceCreate / StateDmSpaceCreate — no Space yet.
    //   * StateRoomCreate from a non-member — disallowed; falls through to
    //     the Space-member check below (room_create's sender must be in
    //     the Space). Permission check (step 13) governs whether they may
    //     additionally create rooms.
    //   * Phase 7 B3 — federation_add via federation channel. Node-authored
    //     events have no Space-membership; authority is signature + session
    //     handshake. See B3 §4.1.
    // M6 A4-D1: node_eject / node_unban are Node-authored (sender = home Node,
    // not a member), so they bypass the member-role checks — their authority is
    // the dedicated home-node gate after step 12.
    let skip_membership = matches!(
        event.event_type,
        EventType::MembershipJoin
            | EventType::StateSpaceCreate
            | EventType::StateDmSpaceCreate
            | EventType::MembershipNodeEject
            | EventType::MembershipNodeUnban
    ) || fed_add_via_federation;
    if !skip_membership {
        let space = match space {
            Some(s) => s,
            None => {
                return ValidationOutcome::Rejected(ExchangeError::DagError(
                    "space context required for non-create event".to_string(),
                ));
            }
        };
        // SpaceState helpers (is_member / is_room_member) take &str; project at
        // call-site boundary per design principle §3. SpaceState identifier surfaces
        // defer to Pass 3 (design doc §5.1) where xgen-node call sites are touched.
        if !space.is_member(sender.as_str()) {
            return ValidationOutcome::Rejected(ExchangeError::NotASpaceMember);
        }
        if !event.room_id.as_str().is_empty()
            && !space.is_room_member(sender.as_str(), event.room_id.as_str())
        {
            return ValidationOutcome::Rejected(ExchangeError::NotARoomMember(
                event.room_id.clone(),
            ));
        }
    }

    // Step 12 — signature verifies against the sender's embedded public key.
    if !verify_event_signature(event) {
        return ValidationOutcome::Rejected(ExchangeError::SignatureFailure);
    }

    // M6 A4-D1 — Node-authority gate for node_eject / node_unban. Authorized by
    // signature (just verified) + `sender == space.home_node`, NOT by member role
    // (they are in skip_membership). Self-contained: home_node is replicated Space
    // state, so federated peers validate identically. A forged eject from any
    // other (validly-signed) signer is rejected here with wire code 3043.
    if matches!(
        event.event_type,
        EventType::MembershipNodeEject | EventType::MembershipNodeUnban
    ) {
        match space {
            Some(s) if event.sender.as_str() == s.home_node.as_str() => {}
            Some(_) => return ValidationOutcome::Rejected(ExchangeError::NodeEjectAuthority),
            None => {
                return ValidationOutcome::Rejected(ExchangeError::DagError(
                    "space context required for node-authority event".to_string(),
                ));
            }
        }
    }

    // Step 13 — permission. Skipped for create + join + Phase 7 B3
    // federation_add-via-federation (skip_membership covers all four cases).
    if !skip_membership {
        if let Some(space) = space {
            if let Err(e) = check_permission(event, space) {
                return ValidationOutcome::Rejected(e);
            }
        }
    }

    ValidationOutcome::Validated
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn validate_dag_structure(event: &Event) -> Result<(), ExchangeError> {
    // event.event_id is Option<EventXgid>; project to &str for comparison.
    let id = event
        .event_id
        .as_ref()
        .map(|e| e.as_str())
        .ok_or(ExchangeError::MissingEventId)?;

    // D-067 no-drift-surface: delegate to is_dag_root_type rather than
    // re-encoding the DAG-root set inline. Under amended D-076 v1.1
    // (locked at topological-sort design-phase re-walk Step 2, 2026-05-22),
    // state.room_create is NOT a root — its sole predecessor is
    // state.space_create. The F-4 validation pipeline and DagGraph::add_event
    // share one canonical encoding of the root set; they cannot drift.
    let is_root = is_dag_root_type(&event.event_type);

    if is_root && !event.prev_events.is_empty() {
        return Err(ExchangeError::DagError(
            "root event type must have empty prev_events".to_string(),
        ));
    }
    if !is_root && event.prev_events.is_empty() {
        return Err(ExchangeError::DagError(
            "non-root event must reference at least one predecessor".to_string(),
        ));
    }
    if event.prev_events.len() > MAX_PREV_EVENTS {
        return Err(ExchangeError::DagError(format!(
            "prev_events has {} entries; maximum is {}",
            event.prev_events.len(),
            MAX_PREV_EVENTS
        )));
    }
    if event.prev_events.iter().any(|p| p.as_str() == id) {
        return Err(ExchangeError::DagError(
            "self-reference in prev_events".to_string(),
        ));
    }

    Ok(())
}

fn check_permission(event: &Event, space: &SpaceState) -> Result<(), ExchangeError> {
    // event.sender is IdentityXgid; project to &str at the &str-taking helper boundary.
    let sender = event.sender.as_str();
    match &event.event_type {
        // Message events require room membership only — verified in step 11.
        EventType::MessageText
        | EventType::MessageFile
        | EventType::MessageReaction
        | EventType::MessageRedact => Ok(()),

        // State updates require Admin or above.
        EventType::StateRoomUpdate | EventType::StateSpaceUpdate => {
            let role = space.member_role(sender);
            if role.map(can_change_space_info).unwrap_or(false) {
                Ok(())
            } else {
                Err(ExchangeError::PermissionDenied(
                    event.event_type.as_str().to_string(),
                ))
            }
        }

        // Membership operations require the appropriate role.
        EventType::MembershipInvite => {
            let role = space.member_role(sender);
            if role.map(can_invite).unwrap_or(false) {
                Ok(())
            } else {
                Err(ExchangeError::PermissionDenied(
                    event.event_type.as_str().to_string(),
                ))
            }
        }
        EventType::MembershipKick => {
            let role = space.member_role(sender);
            if role.map(can_kick).unwrap_or(false) {
                Ok(())
            } else {
                Err(ExchangeError::PermissionDenied(
                    event.event_type.as_str().to_string(),
                ))
            }
        }
        EventType::MembershipBan => {
            let role = space.member_role(sender);
            if role.map(can_ban).unwrap_or(false) {
                Ok(())
            } else {
                Err(ExchangeError::PermissionDenied(
                    event.event_type.as_str().to_string(),
                ))
            }
        }

        // M3 (spec 3.6.10.6) — AI operator delegate/revoke: owner or admin only.
        // Signer-not-admin returns 3041 (AiRoleViolation) per locked decision #4
        // (`wrong signer` is in the 3041 scope). Other validation failures (target
        // not a member, AI not actually AI) live in `check_ai_operator_targets`.
        EventType::StateAiOperatorDelegate | EventType::StateAiOperatorRevoke => {
            let role = space.member_role(sender);
            if role.map(can_delegate_ai_operator).unwrap_or(false) {
                Ok(())
            } else {
                Err(ExchangeError::AiRoleViolation(format!(
                    "{} requires owner or admin",
                    event.event_type.as_str()
                )))
            }
        }

        // All other event types: permitted for any Space member.
        _ => Ok(()),
    }
}

// ── Event builder ─────────────────────────────────────────────────────────────

/// Build an unsigned `message.text` Event.
/// Call `space::state::sign_event` to compute event_id and signature.
pub fn build_message_text_event(
    key: &SigningKey,
    space_id: &str,
    room_id: &str,
    prev_events: Vec<String>,
    text: &str,
) -> Event {
    // Pass 2 widens these projections; the wraps collapse then.
    Event::new(
        EventType::MessageText,
        IdentityXgid::from_xgid(Xgid::new(format!(
            "xgen://pubkey/ed25519:{}",
            encoding::encode(key.verifying_key().as_bytes())
        ))),
        RoomXgid::from_xgid(Xgid::new(room_id.to_string())),
        SpaceXgid::from_xgid(Xgid::new(space_id.to_string())),
        prev_events
            .into_iter()
            .map(|s| EventXgid::from_xgid(Xgid::new(s)))
            .collect(),
        Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
        json!({ "text": text }),
    )
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(deprecated)] // Test fixtures exercise accept_event before its own audit-design-impl removal arc per D-071 (Pass 2 Surface #5 Q5 lock at J-125).
mod tests {
    use super::*;
    use chrono::{SecondsFormat, Utc};
    use crate::{
        dag::{graph::DagGraph, store::InMemoryEventStore},
        identity::{
            keypair,
            registry::{IdentityRecord, IdentityRegistry},
        },
        space::state::{
            build_room_create_event, build_space_create_event, sign_event,
            SpaceState,
        },
        wire::types::{Event, EventType},
    };
    use serde_json::json;
    use xgen_common::xgid::NodeXgid;

    const HOME: &str = "xgen://pubkey/ed25519:NODE";

    // ── Test fixtures ─────────────────────────────────────────────────────────

    // ── Pass 1 Commit 4a test helpers — typed XGID wrappers at fixture sites ──
    fn idx(s: &str) -> IdentityXgid {
        IdentityXgid::from_xgid(Xgid::new(s.to_string()))
    }
    fn ndx(s: &str) -> NodeXgid {
        NodeXgid::from_xgid(Xgid::new(s.to_string()))
    }
    fn event_id_str(ev: &Event) -> String {
        ev.event_id
            .as_ref()
            .expect("signed event has event_id")
            .as_str()
            .to_string()
    }

    fn make_identity_record(key: &SigningKey, home: &str) -> IdentityRecord {
        let id = format!(
            "xgen://pubkey/ed25519:{}",
            encoding::encode(key.verifying_key().as_bytes())
        );
        IdentityRecord {
            identity_id: idx(&id),
            display_name: None,
            is_ai: false,
            ai_capabilities: None,
            registered_at: "2026-04-28T00:00:00.000Z".to_string(),
            trust_assertion: None,
            devices: vec![],
            home_node: ndx(home),
            update_version: 0,
            revoked: false,
            revoked_at: None,
            revocation_reason: None,
        }
    }

    /// Build a membership event with explicit prev_events (DAG-aware variant for tests).
    fn membership_ev_with_prev(
        key: &SigningKey,
        space_id: &str,
        room_id: &str,
        event_type: EventType,
        prev_events: Vec<String>,
        content: serde_json::Value,
    ) -> Event {
        Event::new(
            event_type,
            idx(&format!(
                "xgen://pubkey/ed25519:{}",
                encoding::encode(key.verifying_key().as_bytes())
            )),
            RoomXgid::from_xgid(Xgid::new(room_id.to_string())),
            SpaceXgid::from_xgid(Xgid::new(space_id.to_string())),
            prev_events
                .into_iter()
                .map(|s| EventXgid::from_xgid(Xgid::new(s)))
                .collect(),
            Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
            content,
        )
    }

    /// Build the canonical set of setup events for a two-user Space, returned in
    /// insertion order so any node can replay them deterministically.
    ///
    /// DAG chain (linear, single tip at the end):
    ///   state.space_create (root)
    ///   state.room_create (root, but linked into chain via invite's prev)
    ///   membership.invite  prev=[space_id, room_id]
    ///   membership.join (space)  prev=[invite_id]
    ///   membership.join (room)   prev=[join_space_id]  ← tip
    fn build_setup_events(
        alice_key: &SigningKey,
        bob_key: &SigningKey,
    ) -> (Vec<Event>, String, String) {
        let mut events = Vec::new();

        let space_ev = sign_event(
            build_space_create_event(alice_key, "Test Space", None, 1, HOME),
            alice_key,
        );
        let space_id = event_id_str(&space_ev);
        events.push(space_ev);

        let room_ev = sign_event(
            build_room_create_event(alice_key, &space_id, "general", None),
            alice_key,
        );
        let room_id = event_id_str(&room_ev);
        events.push(room_ev);

        let bob_id = format!(
            "xgen://pubkey/ed25519:{}",
            encoding::encode(bob_key.verifying_key().as_bytes())
        );
        // invite prev = [space_id, room_id] — merges the two roots into one chain.
        let invite_ev = sign_event(
            membership_ev_with_prev(
                alice_key,
                &space_id,
                "",
                EventType::MembershipInvite,
                vec![space_id.clone(), room_id.clone()],
                json!({ "target_identity": bob_id, "role": "member" }),
            ),
            alice_key,
        );
        let invite_id = event_id_str(&invite_ev);
        events.push(invite_ev);

        let join_space_ev = sign_event(
            membership_ev_with_prev(
                bob_key,
                &space_id,
                "",
                EventType::MembershipJoin,
                vec![invite_id],
                json!({}),
            ),
            bob_key,
        );
        let join_space_id = event_id_str(&join_space_ev);
        events.push(join_space_ev);

        let join_room_ev = sign_event(
            membership_ev_with_prev(
                bob_key,
                &space_id,
                &room_id,
                EventType::MembershipJoin,
                vec![join_space_id],
                json!({}),
            ),
            bob_key,
        );
        events.push(join_room_ev);

        (events, space_id, room_id)
    }

    /// Replay a set of ordered events into a store + graph, building SpaceState as we go.
    fn replay_events(
        events: &[Event],
        store: &mut InMemoryEventStore,
        graph: &mut DagGraph,
    ) -> SpaceState {
        let mut space: Option<SpaceState> = None;
        for ev in events {
            graph.add_event(ev, &*store).unwrap();
            store.insert(ev.clone()).unwrap();
            match &ev.event_type {
                EventType::StateSpaceCreate => {
                    space = Some(SpaceState::from_space_create(ev).unwrap());
                }
                _ => {
                    if let Some(ref mut s) = space {
                        let _ = s.apply_event(ev, "");
                    }
                }
            }
        }
        space.expect("space_create must be in events")
    }

    /// Seed an EventStore and DagGraph. Returns (SpaceState, IdentityRegistry, space_id, room_id, tip_id).
    fn setup_node(
        alice_key: &SigningKey,
        bob_key: &SigningKey,
        store: &mut InMemoryEventStore,
        graph: &mut DagGraph,
    ) -> (SpaceState, IdentityRegistry, String, String, String) {
        let (events, space_id, room_id) = build_setup_events(alice_key, bob_key);
        let tip_id = event_id_str(events.last().unwrap());
        let space = replay_events(&events, store, graph);

        let mut registry = IdentityRegistry::new();
        registry.register(make_identity_record(alice_key, HOME)).unwrap();
        registry.register(make_identity_record(bob_key, HOME)).unwrap();

        (space, registry, space_id, room_id, tip_id)
    }

    // ── Step 8 tests ──────────────────────────────────────────────────────────

    #[test]
    fn step8_valid_event_id_passes() {
        let alice = keypair::generate();
        let bob = keypair::generate();
        let mut store = InMemoryEventStore::new();
        let mut graph = DagGraph::new();
        let (space, registry, space_id, room_id, tip_id) =
            setup_node(&alice, &bob, &mut store, &mut graph);

        let ev = sign_event(
            build_message_text_event(&alice, &space_id, &room_id, vec![tip_id], "hello"),
            &alice,
        );
        assert!(validate_steps_8_13(&ev, &space, &registry, &store).is_ok());
    }

    #[test]
    fn step8_wrong_event_id_rejected() {
        let alice = keypair::generate();
        let bob = keypair::generate();
        let mut store = InMemoryEventStore::new();
        let mut graph = DagGraph::new();
        let (space, registry, space_id, room_id, tip_id) =
            setup_node(&alice, &bob, &mut store, &mut graph);

        let mut ev = sign_event(
            build_message_text_event(&alice, &space_id, &room_id, vec![tip_id], "hello"),
            &alice,
        );
        ev.event_id = Some(EventXgid::from_xgid(Xgid::new(
            "xgen://hash/sha256:TAMPERED".to_string(),
        )));

        assert!(matches!(
            validate_steps_8_13(&ev, &space, &registry, &store),
            Err(ExchangeError::EventIdMismatch)
        ));
    }

    // ── Step 9 tests ──────────────────────────────────────────────────────────

    #[test]
    fn step9_unknown_prev_event_held_pending() {
        let alice = keypair::generate();
        let bob = keypair::generate();
        let mut store = InMemoryEventStore::new();
        let mut graph = DagGraph::new();
        let (space, registry, space_id, room_id, _) =
            setup_node(&alice, &bob, &mut store, &mut graph);

        // Reference an event that is not in the store.
        let unknown_id = "xgen://hash/sha256:UNKNOWN_PREV";
        let ev = sign_event(
            build_message_text_event(
                &alice,
                &space_id,
                &room_id,
                vec![unknown_id.to_string()],
                "hello",
            ),
            &alice,
        );

        assert!(matches!(
            validate_steps_8_13(&ev, &space, &registry, &store),
            Err(ExchangeError::HeldPending(_))
        ));
    }

    // ── Step 11 tests ─────────────────────────────────────────────────────────

    #[test]
    fn step11_unregistered_sender_rejected() {
        let alice = keypair::generate();
        let bob = keypair::generate();
        let charlie = keypair::generate(); // not registered
        let mut store = InMemoryEventStore::new();
        let mut graph = DagGraph::new();
        let (space, registry, space_id, room_id, tip_id) =
            setup_node(&alice, &bob, &mut store, &mut graph);

        let ev = sign_event(
            build_message_text_event(&charlie, &space_id, &room_id, vec![tip_id], "hello"),
            &charlie,
        );

        assert!(matches!(
            validate_steps_8_13(&ev, &space, &registry, &store),
            Err(ExchangeError::UnknownSender)
        ));
    }

    #[test]
    fn step11_non_space_member_rejected() {
        let alice = keypair::generate();
        let bob = keypair::generate();
        let charlie = keypair::generate(); // registered but not a member
        let mut store = InMemoryEventStore::new();
        let mut graph = DagGraph::new();
        let (space, mut registry, space_id, room_id, tip_id) =
            setup_node(&alice, &bob, &mut store, &mut graph);

        // Register Charlie but don't add to the Space.
        registry.register(make_identity_record(&charlie, HOME)).unwrap();

        let ev = sign_event(
            build_message_text_event(&charlie, &space_id, &room_id, vec![tip_id], "hello"),
            &charlie,
        );

        assert!(matches!(
            validate_steps_8_13(&ev, &space, &registry, &store),
            Err(ExchangeError::NotASpaceMember)
        ));
    }

    #[test]
    fn node_eject_from_non_home_node_rejected_with_3043() {
        // M6 A4-D1 wire gate (forge prevention): a validly-signed node_eject whose
        // sender is NOT the Space's home Node is rejected with NodeEjectAuthority
        // (wire 3043). Reaches the gate by passing steps 8–12 (node_eject is in
        // skip_membership, so steps 11/13 don't fire).
        let alice = keypair::generate();
        let bob = keypair::generate();
        let mallory = keypair::generate(); // validly-signed, but not the home Node
        let mut store = InMemoryEventStore::new();
        let mut graph = DagGraph::new();
        let (space, registry, space_id, _room_id, tip_id) =
            setup_node(&alice, &bob, &mut store, &mut graph);

        let target = format!(
            "xgen://pubkey/ed25519:{}",
            encoding::encode(bob.verifying_key().as_bytes())
        );
        let eject = sign_event(
            membership_ev_with_prev(
                &mallory,
                &space_id,
                "",
                EventType::MembershipNodeEject,
                vec![tip_id],
                json!({ "target_identity": target }),
            ),
            &mallory,
        );
        let outcome = validate_event(&eject, Some(&space), &registry, &store, false);
        assert!(matches!(
            outcome,
            ValidationOutcome::Rejected(ExchangeError::NodeEjectAuthority)
        ));
        assert_eq!(
            ExchangeError::NodeEjectAuthority.to_wire_code(),
            Some((3043, "node_eject_authority"))
        );
    }

    #[test]
    fn step11_non_room_member_rejected() {
        let alice = keypair::generate();
        let bob = keypair::generate();
        let charlie = keypair::generate();
        let mut store = InMemoryEventStore::new();
        let mut graph = DagGraph::new();
        let (mut space, mut registry, space_id, room_id, tip_id) =
            setup_node(&alice, &bob, &mut store, &mut graph);

        // Register Charlie and invite/join the Space but NOT the room.
        registry.register(make_identity_record(&charlie, HOME)).unwrap();
        let charlie_id = format!(
            "xgen://pubkey/ed25519:{}",
            encoding::encode(charlie.verifying_key().as_bytes())
        );
        let invite_ev = sign_event(
            membership_ev_with_prev(
                &alice,
                &space_id,
                "",
                EventType::MembershipInvite,
                vec![tip_id.clone()],
                json!({ "target_identity": charlie_id, "role": "member" }),
            ),
            &alice,
        );
        space.apply_event(&invite_ev, "").unwrap();
        graph.add_event(&invite_ev, &store).unwrap();
        let charlie_invite_id = event_id_str(&invite_ev);
        store.insert(invite_ev).unwrap();

        let join_ev = sign_event(
            membership_ev_with_prev(
                &charlie,
                &space_id,
                "",
                EventType::MembershipJoin,
                vec![charlie_invite_id],
                json!({}),
            ),
            &charlie,
        );
        space.apply_event(&join_ev, "").unwrap();
        graph.add_event(&join_ev, &store).unwrap();
        let new_tip = event_id_str(&join_ev);
        store.insert(join_ev).unwrap();

        // Charlie tries to message in a room they haven't joined.
        let ev = sign_event(
            build_message_text_event(&charlie, &space_id, &room_id, vec![new_tip], "hello"),
            &charlie,
        );

        assert!(matches!(
            validate_steps_8_13(&ev, &space, &registry, &store),
            Err(ExchangeError::NotARoomMember(_))
        ));
    }

    // ── Step 12 tests ─────────────────────────────────────────────────────────

    #[test]
    fn step12_tampered_content_fails_signature() {
        let alice = keypair::generate();
        let bob = keypair::generate();
        let mut store = InMemoryEventStore::new();
        let mut graph = DagGraph::new();
        let (space, registry, space_id, room_id, tip_id) =
            setup_node(&alice, &bob, &mut store, &mut graph);

        let mut ev = sign_event(
            build_message_text_event(&alice, &space_id, &room_id, vec![tip_id], "hello"),
            &alice,
        );
        // Tamper with content AFTER signing — event_id stays the same so step 8 passes,
        // but the signature no longer matches.
        ev.content = json!({ "text": "TAMPERED" });
        // Recompute event_id to match the tampered content so step 8 passes.
        let v = serde_json::to_value(&ev).unwrap();
        let bytes = crate::wire::canonical::canonical_event_bytes(&v);
        ev.event_id = Some(EventXgid::from_xgid(Xgid::new(hashing::hash_uri(&bytes))));

        assert!(matches!(
            validate_steps_8_13(&ev, &space, &registry, &store),
            Err(ExchangeError::SignatureFailure)
        ));
    }

    // ── accept_event tests ────────────────────────────────────────────────────

    #[test]
    fn accept_event_stores_in_dag() {
        let alice = keypair::generate();
        let bob = keypair::generate();
        let mut store = InMemoryEventStore::new();
        let mut graph = DagGraph::new();
        let (space, registry, space_id, room_id, tip_id) =
            setup_node(&alice, &bob, &mut store, &mut graph);

        let ev = sign_event(
            build_message_text_event(&alice, &space_id, &room_id, vec![tip_id.clone()], "hello"),
            &alice,
        );
        let ev_id = event_id_str(&ev);

        accept_event(ev, &space, &registry, &mut store, &mut graph).unwrap();

        assert!(store.contains(&ev_id));
        assert!(graph.is_tip(&ev_id));
        assert!(!graph.is_tip(&tip_id)); // previous tip replaced
    }

    #[test]
    fn accept_event_duplicate_rejected() {
        let alice = keypair::generate();
        let bob = keypair::generate();
        let mut store = InMemoryEventStore::new();
        let mut graph = DagGraph::new();
        let (space, registry, space_id, room_id, tip_id) =
            setup_node(&alice, &bob, &mut store, &mut graph);

        let ev = sign_event(
            build_message_text_event(&alice, &space_id, &room_id, vec![tip_id], "hello"),
            &alice,
        );

        accept_event(ev.clone(), &space, &registry, &mut store, &mut graph).unwrap();
        // Second accept of the same event must fail.
        assert!(accept_event(ev, &space, &registry, &mut store, &mut graph).is_err());
    }

    // ── Integration: two-node propagation ────────────────────────────────────

    /// Alice sends "Hello Bob" on Node A. We simulate propagation by calling
    /// accept_event on Node B with the same event. Verify the event appears in
    /// Node B's store with correct event_id, valid signature, and correct prev_events.
    #[test]
    fn message_propagates_from_node_a_to_node_b() {
        let alice = keypair::generate();
        let bob = keypair::generate();

        // Build the shared setup events once — both nodes replay the identical events
        // so they arrive at the same event_ids and tips.
        let (setup_events, space_id, room_id) = build_setup_events(&alice, &bob);
        let tip_id = event_id_str(setup_events.last().unwrap());

        let mut registry = IdentityRegistry::new();
        registry.register(make_identity_record(&alice, HOME)).unwrap();
        registry.register(make_identity_record(&bob, HOME)).unwrap();

        // Node A
        let mut store_a = InMemoryEventStore::new();
        let mut graph_a = DagGraph::new();
        let space_a = replay_events(&setup_events, &mut store_a, &mut graph_a);

        // Node B — seeded with the exact same events (deterministic replay).
        let mut store_b = InMemoryEventStore::new();
        let mut graph_b = DagGraph::new();
        let space_b = replay_events(&setup_events, &mut store_b, &mut graph_b);

        // Alice sends a message on Node A.
        let msg_ev = sign_event(
            build_message_text_event(&alice, &space_id, &room_id, vec![tip_id.clone()], "Hello Bob"),
            &alice,
        );
        let msg_id = event_id_str(&msg_ev);
        let msg_prev = msg_ev.prev_events.clone();

        accept_event(msg_ev.clone(), &space_a, &registry, &mut store_a, &mut graph_a).unwrap();

        // Propagate to Node B (simulate forwarding the event over the wire).
        accept_event(msg_ev, &space_b, &registry, &mut store_b, &mut graph_b).unwrap();

        // Verify event is in Node B's store.
        let stored = store_b.get(&msg_id).expect("event must be in Node B's store");
        assert_eq!(
            stored.event_id.as_ref().map(|e| e.as_str()),
            Some(msg_id.as_str())
        );
        assert_eq!(stored.event_type, EventType::MessageText);
        assert_eq!(stored.prev_events, msg_prev);
        assert!(stored.signature.is_some());
        // Signature is still valid on the stored copy.
        assert!(verify_event_signature(stored));
        // Event is the current DAG tip on Node B.
        assert!(graph_b.is_tip(&msg_id));
    }

    // ── AI capability enforcement (spec 3.6.10.4) ───────────────────────────

    fn ai_record(key: &SigningKey, dm_initiate: bool) -> IdentityRecord {
        let id = format!(
            "xgen://pubkey/ed25519:{}",
            encoding::encode(key.verifying_key().as_bytes())
        );
        IdentityRecord {
            identity_id: idx(&id),
            display_name: Some("Bot".to_string()),
            is_ai: true,
            ai_capabilities: Some(xgen_common::wire::AiCapabilities {
                dm_initiate,
                spontaneous_post: false,
                extra: Default::default(),
            }),
            registered_at: "2026-04-28T00:00:00.000Z".to_string(),
            trust_assertion: None,
            devices: vec![],
            home_node: ndx(HOME),
            update_version: 0,
            revoked: false,
            revoked_at: None,
            revocation_reason: None,
        }
    }

    fn dm_space_create_event(creator: &SigningKey, invitee_id: &str) -> Event {
        sign_event(
            crate::space::state::build_dm_space_create_event(creator, invitee_id, HOME),
            creator,
        )
    }

    #[test]
    fn ai_dm_space_create_rejected_regardless_of_dm_initiate() {
        // M3 (spec 3.6.10.6): AI cannot own a Space — state.dm_space_create from
        // any AI sender is rejected with AiRoleViolation (3041), superseding the
        // earlier D-059 capability-flag rejection path. Verified for both
        // dm_initiate values to confirm the role rule wins regardless.
        for dm_initiate in [false, true] {
            let bot = keypair::generate();
            let bob = keypair::generate();
            let bob_id = format!(
                "xgen://pubkey/ed25519:{}",
                encoding::encode(bob.verifying_key().as_bytes())
            );

            let mut registry = IdentityRegistry::new();
            registry.register(ai_record(&bot, dm_initiate)).unwrap();

            let ev = dm_space_create_event(&bot, &bob_id);
            let err = check_ai_capability(&ev, &registry).unwrap_err();
            match err {
                ExchangeError::AiRoleViolation(msg) => {
                    assert!(
                        msg.contains("ai cannot create a space"),
                        "msg: {msg}"
                    );
                }
                other => panic!(
                    "expected AiRoleViolation (dm_initiate={dm_initiate}), got {other:?}"
                ),
            }
        }
    }

    #[test]
    fn ai_space_create_rejected_regardless_of_capabilities() {
        // M3 (spec 3.6.10.6): symmetrically, state.space_create from an AI is
        // also rejected with AiRoleViolation — AI cannot be a Space owner.
        let bot = keypair::generate();
        let mut registry = IdentityRegistry::new();
        registry.register(ai_record(&bot, /* dm_initiate */ true)).unwrap();

        let ev = sign_event(
            crate::space::state::build_space_create_event(&bot, "Bot Space", None, 1, HOME),
            &bot,
        );
        let err = check_ai_capability(&ev, &registry).unwrap_err();
        match err {
            ExchangeError::AiRoleViolation(msg) => {
                assert!(msg.contains("ai cannot create a space"), "msg: {msg}");
            }
            other => panic!("expected AiRoleViolation, got {other:?}"),
        }
    }

    #[test]
    fn ai_role_violation_wire_code_is_3041() {
        let err = ExchangeError::AiRoleViolation("test".to_string());
        assert_eq!(err.to_wire_code(), Some((3041, "ai_role_violation")));
    }

    #[test]
    fn ai_capability_violation_wire_code_is_3042() {
        let err = ExchangeError::AiCapabilityViolation("dm_initiate disallowed".to_string());
        assert_eq!(err.to_wire_code(), Some((3042, "ai_capability_violation")));
    }

    #[test]
    fn human_dm_space_create_passes_capability_check() {
        let alice = keypair::generate();
        let bob = keypair::generate();
        let bob_id = format!(
            "xgen://pubkey/ed25519:{}",
            encoding::encode(bob.verifying_key().as_bytes())
        );

        let mut registry = IdentityRegistry::new();
        registry.register(make_identity_record(&alice, HOME)).unwrap();

        let ev = dm_space_create_event(&alice, &bob_id);
        assert!(check_ai_capability(&ev, &registry).is_ok());
    }

    #[test]
    fn ai_message_text_passes_capability_check() {
        // Only state.dm_space_create is gated; other event types from an AI
        // with dm_initiate = false must still pass the capability check.
        let bot = keypair::generate();
        let bob = keypair::generate();
        let mut store = InMemoryEventStore::new();
        let mut graph = DagGraph::new();
        let (space, mut registry, space_id, room_id, tip_id) =
            setup_node(&bot, &bob, &mut store, &mut graph);

        // Replace bot's record with one marked as AI (dm_initiate = false).
        registry.upsert(ai_record(&bot, false));

        let ev = sign_event(
            build_message_text_event(&bot, &space_id, &room_id, vec![tip_id], "hello"),
            &bot,
        );
        assert!(check_ai_capability(&ev, &registry).is_ok());
        // And full pipeline succeeds.
        assert!(validate_steps_8_13(&ev, &space, &registry, &store).is_ok());
    }

    /// Two concurrent messages from different senders produce two DAG tips.
    #[test]
    fn concurrent_messages_produce_two_tips() {
        let alice = keypair::generate();
        let bob = keypair::generate();
        let mut store = InMemoryEventStore::new();
        let mut graph = DagGraph::new();
        let (space, registry, space_id, room_id, tip_id) =
            setup_node(&alice, &bob, &mut store, &mut graph);

        let ev_alice = sign_event(
            build_message_text_event(
                &alice,
                &space_id,
                &room_id,
                vec![tip_id.clone()],
                "Hi from Alice",
            ),
            &alice,
        );
        let ev_bob = sign_event(
            build_message_text_event(
                &bob,
                &space_id,
                &room_id,
                vec![tip_id.clone()],
                "Hi from Bob",
            ),
            &bob,
        );

        accept_event(ev_alice, &space, &registry, &mut store, &mut graph).unwrap();
        accept_event(ev_bob, &space, &registry, &mut store, &mut graph).unwrap();

        // Both messages reference the same predecessor → fork → two tips.
        assert_eq!(graph.tip_count(), 2);
    }

    // ── M3 (spec 3.6.10.6) — AI operator delegate / revoke validation ─────────

    /// Fixture: owner alice, admin bob, AI member dave (`is_ai=true`), plain
    /// member carol. Returns (space_state, identity_registry, space_id,
    /// alice_key, bob_key, dave_key (AI), carol_key, tip_event_id).
    #[allow(clippy::type_complexity)]
    fn setup_ai_space() -> (
        SpaceState,
        IdentityRegistry,
        String,
        SigningKey, // alice (owner)
        SigningKey, // bob (admin)
        SigningKey, // dave (AI)
        SigningKey, // carol (member)
        String,     // tip event id
    ) {
        use crate::space::state::{
            build_room_create_event, build_space_create_event, sign_event, SpaceState,
        };

        let alice = keypair::generate();
        let bob = keypair::generate();
        let dave = keypair::generate();
        let carol = keypair::generate();
        let alice_id = format!(
            "xgen://pubkey/ed25519:{}",
            encoding::encode(alice.verifying_key().as_bytes())
        );
        let bob_id = format!(
            "xgen://pubkey/ed25519:{}",
            encoding::encode(bob.verifying_key().as_bytes())
        );
        let dave_id = format!(
            "xgen://pubkey/ed25519:{}",
            encoding::encode(dave.verifying_key().as_bytes())
        );
        let carol_id = format!(
            "xgen://pubkey/ed25519:{}",
            encoding::encode(carol.verifying_key().as_bytes())
        );
        let _ = alice_id;

        let space_ev = sign_event(
            build_space_create_event(&alice, "M3 Test", None, 1, HOME),
            &alice,
        );
        let space_id = event_id_str(&space_ev);
        let room_ev = sign_event(
            build_room_create_event(&alice, &space_id, "general", None),
            &alice,
        );
        let room_id = event_id_str(&room_ev);
        let _ = room_id;

        let mut space = SpaceState::from_space_create(&space_ev).unwrap();
        space.apply_event(&room_ev, "").unwrap();

        let mut last_id = event_id_str(&room_ev);

        // Invite + join for each of bob (admin), dave (member), carol (member).
        for (target_id, role, joiner_key) in [
            (&bob_id, "admin", &bob),
            (&dave_id, "member", &dave),
            (&carol_id, "member", &carol),
        ] {
            let invite = sign_event(
                membership_ev_with_prev(
                    &alice,
                    &space_id,
                    "",
                    EventType::MembershipInvite,
                    vec![last_id.clone()],
                    json!({ "target_identity": target_id, "role": role }),
                ),
                &alice,
            );
            space.apply_event(&invite, "").unwrap();
            last_id = event_id_str(&invite);
            let join = sign_event(
                membership_ev_with_prev(
                    joiner_key,
                    &space_id,
                    "",
                    EventType::MembershipJoin,
                    vec![last_id.clone()],
                    json!({}),
                ),
                joiner_key,
            );
            space.apply_event(&join, "").unwrap();
            last_id = event_id_str(&join);
        }

        let mut registry = IdentityRegistry::new();
        registry.register(make_identity_record(&alice, HOME)).unwrap();
        registry.register(make_identity_record(&bob, HOME)).unwrap();
        registry.register(ai_record(&dave, /* dm_initiate */ false)).unwrap();
        registry.register(make_identity_record(&carol, HOME)).unwrap();

        (space, registry, space_id, alice, bob, dave, carol, last_id)
    }

    /// Lightweight tip-aware delegate event constructor that wires the right prev_events.
    fn signed_delegate(
        signer: &SigningKey,
        space_id: &str,
        prev: Vec<String>,
        ai_id: &str,
        new_op_id: &str,
    ) -> Event {
        crate::space::state::sign_event(
            crate::space::state::build_state_ai_operator_delegate_event(
                signer,
                space_id,
                prev,
                ai_id,
                new_op_id,
            ),
            signer,
        )
    }

    fn signed_revoke(
        signer: &SigningKey,
        space_id: &str,
        prev: Vec<String>,
        ai_id: &str,
    ) -> Event {
        crate::space::state::sign_event(
            crate::space::state::build_state_ai_operator_revoke_event(signer, space_id, prev, ai_id),
            signer,
        )
    }

    fn id_of_key(k: &SigningKey) -> String {
        format!(
            "xgen://pubkey/ed25519:{}",
            encoding::encode(k.verifying_key().as_bytes())
        )
    }

    #[test]
    fn m3_delegate_happy_path() {
        let (space, registry, sid, _alice, bob, dave, carol, tip) = setup_ai_space();
        let mut store = InMemoryEventStore::new();
        let mut graph = DagGraph::new();
        // Seed the store/graph with a stand-in tip so prev_events validates.
        let space_ev = crate::space::state::sign_event(
            crate::space::state::build_space_create_event(&_alice, "stub", None, 1, HOME),
            &_alice,
        );
        graph.add_event(&space_ev, &store).unwrap();
        store.insert(space_ev).unwrap();
        // Insert the actual tip as a synthetic root-bearing event by reusing the
        // event chain from setup_ai_space — easier to just use accept_event on the
        // delegate without DAG tip checks, since we test the validation function
        // directly (not the DAG). Build a thinner test: skip the store/graph and
        // call validate_steps_8_13 only after pre-loading the tip.
        // Easier: just exercise check_ai_operator_targets + check_permission
        // directly here, since those are the M3-specific validators.
        let _ = (&store, &graph, &tip);

        // Admin bob delegates dave's operator role to carol.
        let ev = signed_delegate(
            &bob,
            &sid,
            vec![tip.clone()],
            &id_of_key(&dave),
            &id_of_key(&carol),
        );
        // Target validation must pass.
        assert!(check_ai_operator_targets(&ev, &space, &registry).is_ok());
        // Permission check must pass (bob is admin).
        assert!(check_permission(&ev, &space).is_ok());
    }

    #[test]
    fn m3_delegate_rejected_when_signer_is_member() {
        let (space, _registry, sid, _alice, _bob, dave, carol, tip) = setup_ai_space();
        // carol is a plain member — cannot delegate.
        let ev = signed_delegate(
            &carol,
            &sid,
            vec![tip],
            &id_of_key(&dave),
            &id_of_key(&_bob),
        );
        let err = check_permission(&ev, &space).unwrap_err();
        match err {
            ExchangeError::AiRoleViolation(msg) => {
                assert!(msg.contains("requires owner or admin"), "msg: {msg}");
            }
            other => panic!("expected AiRoleViolation, got {other:?}"),
        }
    }

    #[test]
    fn m3_delegate_rejected_when_ai_target_not_a_member() {
        let (space, registry, sid, _alice, bob, _dave, carol, tip) = setup_ai_space();
        // Non-member target.
        let ev = signed_delegate(
            &bob,
            &sid,
            vec![tip],
            "xgen://pubkey/ed25519:STRANGER",
            &id_of_key(&carol),
        );
        let err = check_ai_operator_targets(&ev, &space, &registry).unwrap_err();
        match err {
            ExchangeError::AiRoleViolation(msg) => {
                assert!(msg.contains("ai_identity_id is not a space member"), "msg: {msg}");
            }
            other => panic!("expected AiRoleViolation, got {other:?}"),
        }
    }

    #[test]
    fn m3_delegate_rejected_when_ai_target_is_not_ai() {
        let (space, registry, sid, _alice, bob, _dave, carol, tip) = setup_ai_space();
        // Use carol (a human member) in the ai_identity_id slot.
        let ev = signed_delegate(&bob, &sid, vec![tip], &id_of_key(&carol), &id_of_key(&bob));
        let err = check_ai_operator_targets(&ev, &space, &registry).unwrap_err();
        match err {
            ExchangeError::AiRoleViolation(msg) => {
                assert!(msg.contains("not an ai identity"), "msg: {msg}");
            }
            other => panic!("expected AiRoleViolation, got {other:?}"),
        }
    }

    #[test]
    fn m3_delegate_rejected_when_new_operator_not_a_member() {
        let (space, registry, sid, _alice, bob, dave, _carol, tip) = setup_ai_space();
        let ev = signed_delegate(
            &bob,
            &sid,
            vec![tip],
            &id_of_key(&dave),
            "xgen://pubkey/ed25519:GHOST",
        );
        let err = check_ai_operator_targets(&ev, &space, &registry).unwrap_err();
        match err {
            ExchangeError::AiRoleViolation(msg) => {
                assert!(
                    msg.contains("new_operator_identity_id is not a space member"),
                    "msg: {msg}"
                );
            }
            other => panic!("expected AiRoleViolation, got {other:?}"),
        }
    }

    #[test]
    fn m3_revoke_happy_path() {
        let (space, registry, sid, _alice, bob, dave, _carol, tip) = setup_ai_space();
        let ev = signed_revoke(&bob, &sid, vec![tip], &id_of_key(&dave));
        assert!(check_ai_operator_targets(&ev, &space, &registry).is_ok());
        assert!(check_permission(&ev, &space).is_ok());
    }

    #[test]
    fn m3_revoke_rejected_when_signer_is_member() {
        let (space, _registry, sid, _alice, _bob, dave, carol, tip) = setup_ai_space();
        let ev = signed_revoke(&carol, &sid, vec![tip], &id_of_key(&dave));
        let err = check_permission(&ev, &space).unwrap_err();
        assert!(matches!(err, ExchangeError::AiRoleViolation(_)));
    }

    /// M3 federation smoke (decision #6). Two Nodes (A and B), one AI Identity,
    /// delegate event, revoke event, and a delegatee-leaves case — all three
    /// cross-Node scenarios verified end-to-end. Propagation is simulated by
    /// calling `accept_event` on Node B with the events Node A accepted.
    /// (Same pattern as `message_propagates_from_node_a_to_node_b`.)
    #[test]
    fn m3_two_node_federation_smoke() {
        use crate::space::state::{
            build_room_create_event, build_space_create_event, sign_event, SpaceState,
        };

        // ── Fixture: alice (owner, on A), bob (AI on B), carol (member on B) ──
        let alice = keypair::generate(); // owner, drives Node A
        let bob = keypair::generate(); // AI member
        let carol = keypair::generate(); // plain member, candidate operator
        let alice_id = format!(
            "xgen://pubkey/ed25519:{}",
            encoding::encode(alice.verifying_key().as_bytes())
        );
        let bob_id = format!(
            "xgen://pubkey/ed25519:{}",
            encoding::encode(bob.verifying_key().as_bytes())
        );
        let carol_id = format!(
            "xgen://pubkey/ed25519:{}",
            encoding::encode(carol.verifying_key().as_bytes())
        );

        // Identity registry — shared between Nodes (federation replicates is_ai).
        let mut registry = IdentityRegistry::new();
        registry.register(make_identity_record(&alice, HOME)).unwrap();
        registry.register(ai_record(&bob, /* dm_initiate */ false)).unwrap();
        registry.register(make_identity_record(&carol, HOME)).unwrap();

        // ── Build the setup chain (space_create + room + 2 invites + 2 joins) ──
        let mut chain: Vec<Event> = Vec::new();
        let space_ev = sign_event(
            build_space_create_event(&alice, "Federated AI test", None, 1, HOME),
            &alice,
        );
        let space_id = event_id_str(&space_ev);
        chain.push(space_ev.clone());

        let room_ev = sign_event(
            build_room_create_event(&alice, &space_id, "general", None),
            &alice,
        );
        chain.push(room_ev.clone());
        let mut last_id = event_id_str(&room_ev);

        for (target, joiner) in [(&bob_id, &bob), (&carol_id, &carol)] {
            let invite = sign_event(
                membership_ev_with_prev(
                    &alice,
                    &space_id,
                    "",
                    EventType::MembershipInvite,
                    vec![last_id.clone()],
                    json!({ "target_identity": target, "role": "member" }),
                ),
                &alice,
            );
            last_id = event_id_str(&invite);
            chain.push(invite);
            let join = sign_event(
                membership_ev_with_prev(
                    joiner,
                    &space_id,
                    "",
                    EventType::MembershipJoin,
                    vec![last_id.clone()],
                    json!({}),
                ),
                joiner,
            );
            last_id = event_id_str(&join);
            chain.push(join);
        }

        // Helper: replay a slice of events into a (store, graph, space) tuple.
        fn replay_chain(
            events: &[Event],
            store: &mut InMemoryEventStore,
            graph: &mut DagGraph,
        ) -> SpaceState {
            let mut space: Option<SpaceState> = None;
            for ev in events {
                graph.add_event(ev, &*store).unwrap();
                store.insert(ev.clone()).unwrap();
                match &ev.event_type {
                    EventType::StateSpaceCreate => {
                        space = Some(SpaceState::from_space_create(ev).unwrap());
                    }
                    _ => {
                        if let Some(ref mut s) = space {
                            let _ = s.apply_event(ev, "");
                        }
                    }
                }
            }
            space.expect("space_create at head of chain")
        }

        // Both Nodes replay the deterministic setup chain → identical state.
        let mut store_a = InMemoryEventStore::new();
        let mut graph_a = DagGraph::new();
        let mut space_a = replay_chain(&chain, &mut store_a, &mut graph_a);
        let mut store_b = InMemoryEventStore::new();
        let mut graph_b = DagGraph::new();
        let mut space_b = replay_chain(&chain, &mut store_b, &mut graph_b);

        // Sanity: pre-delegate resolution falls back to inviter alice on both.
        assert_eq!(
            space_a.resolve_operator(&bob_id).as_deref(),
            Some(alice_id.as_str()),
            "pre-delegate: Node A resolves alice (inviter) as operator"
        );
        assert_eq!(
            space_b.resolve_operator(&bob_id).as_deref(),
            Some(alice_id.as_str()),
            "pre-delegate: Node B resolves alice (inviter) as operator"
        );

        // Helper: alice signs an event referencing the current tip, accepted on
        // Node A then propagated to Node B. Returns the event_id (new tip).
        let mut current_tip = last_id.clone();

        // ── Scenario 1: cross-Node delegate ───────────────────────────────────
        let delegate_ev = sign_event(
            crate::space::state::build_state_ai_operator_delegate_event(
                &alice,
                &space_id,
                vec![current_tip.clone()],
                &bob_id,
                &carol_id,
            ),
            &alice,
        );
        let delegate_id = event_id_str(&delegate_ev);
        accept_event(
            delegate_ev.clone(),
            &space_a,
            &registry,
            &mut store_a,
            &mut graph_a,
        )
        .expect("Node A accepts delegate");
        space_a.apply_event(&delegate_ev, "").expect("Node A applies delegate");

        accept_event(
            delegate_ev.clone(),
            &space_b,
            &registry,
            &mut store_b,
            &mut graph_b,
        )
        .expect("Node B accepts delegate (propagation)");
        space_b.apply_event(&delegate_ev, "").expect("Node B applies delegate");
        current_tip = delegate_id;

        assert_eq!(
            space_a.resolve_operator(&bob_id).as_deref(),
            Some(carol_id.as_str()),
            "Scenario 1 — Node A resolves carol after delegate"
        );
        assert_eq!(
            space_b.resolve_operator(&bob_id).as_deref(),
            Some(carol_id.as_str()),
            "Scenario 1 — Node B resolves carol after cross-Node propagation"
        );

        // ── Scenario 2: cross-Node revoke ─────────────────────────────────────
        let revoke_ev = sign_event(
            crate::space::state::build_state_ai_operator_revoke_event(
                &alice,
                &space_id,
                vec![current_tip.clone()],
                &bob_id,
            ),
            &alice,
        );
        let revoke_id = event_id_str(&revoke_ev);
        accept_event(
            revoke_ev.clone(),
            &space_a,
            &registry,
            &mut store_a,
            &mut graph_a,
        )
        .expect("Node A accepts revoke");
        space_a.apply_event(&revoke_ev, "").expect("Node A applies revoke");
        accept_event(
            revoke_ev.clone(),
            &space_b,
            &registry,
            &mut store_b,
            &mut graph_b,
        )
        .expect("Node B accepts revoke (propagation)");
        space_b.apply_event(&revoke_ev, "").expect("Node B applies revoke");
        current_tip = revoke_id;

        assert_eq!(
            space_a.resolve_operator(&bob_id).as_deref(),
            Some(alice_id.as_str()),
            "Scenario 2 — Node A resolves alice (inviter fallback) after revoke"
        );
        assert_eq!(
            space_b.resolve_operator(&bob_id).as_deref(),
            Some(alice_id.as_str()),
            "Scenario 2 — Node B resolves alice (inviter fallback) after revoke"
        );

        // ── Scenario 3: fall-upward across federation (re-delegate + kick) ────
        let redelegate_ev = sign_event(
            crate::space::state::build_state_ai_operator_delegate_event(
                &alice,
                &space_id,
                vec![current_tip.clone()],
                &bob_id,
                &carol_id,
            ),
            &alice,
        );
        let redelegate_id = event_id_str(&redelegate_ev);
        accept_event(
            redelegate_ev.clone(),
            &space_a,
            &registry,
            &mut store_a,
            &mut graph_a,
        )
        .expect("Node A accepts re-delegate");
        space_a.apply_event(&redelegate_ev, "").expect("Node A applies re-delegate");
        accept_event(
            redelegate_ev.clone(),
            &space_b,
            &registry,
            &mut store_b,
            &mut graph_b,
        )
        .expect("Node B accepts re-delegate (propagation)");
        space_b.apply_event(&redelegate_ev, "").expect("Node B applies re-delegate");
        current_tip = redelegate_id;

        // Kick carol — alice is owner, can kick.
        let kick_ev = sign_event(
            membership_ev_with_prev(
                &alice,
                &space_id,
                "",
                EventType::MembershipKick,
                vec![current_tip.clone()],
                json!({ "target_identity": carol_id }),
            ),
            &alice,
        );
        accept_event(
            kick_ev.clone(),
            &space_a,
            &registry,
            &mut store_a,
            &mut graph_a,
        )
        .expect("Node A accepts kick of carol");
        space_a.apply_event(&kick_ev, "").expect("Node A applies kick");
        accept_event(
            kick_ev.clone(),
            &space_b,
            &registry,
            &mut store_b,
            &mut graph_b,
        )
        .expect("Node B accepts kick (propagation)");
        space_b.apply_event(&kick_ev, "").expect("Node B applies kick");

        // The stored delegation still names carol, but she's no longer a
        // member. Resolution must transparently fall through to alice without
        // any explicit revoke being signed.
        assert!(
            space_a.ai_operator_delegations.contains_key(bob_id.as_str()),
            "Scenario 3 — stored delegation still present on Node A"
        );
        assert!(
            space_b.ai_operator_delegations.contains_key(bob_id.as_str()),
            "Scenario 3 — stored delegation still present on Node B"
        );
        assert!(
            !space_a.is_member(carol_id.as_str()),
            "Scenario 3 — carol is no longer a Space A member"
        );
        assert!(
            !space_b.is_member(carol_id.as_str()),
            "Scenario 3 — carol is no longer a Space B member"
        );
        assert_eq!(
            space_a.resolve_operator(&bob_id).as_deref(),
            Some(alice_id.as_str()),
            "Scenario 3 — Node A falls upward to alice (inviter) without revoke"
        );
        assert_eq!(
            space_b.resolve_operator(&bob_id).as_deref(),
            Some(alice_id.as_str()),
            "Scenario 3 — Node B falls upward to alice (inviter) without revoke"
        );
    }

    #[test]
    fn m3_revoke_rejected_when_ai_target_is_not_ai() {
        let (space, registry, sid, _alice, bob, _dave, carol, tip) = setup_ai_space();
        let ev = signed_revoke(&bob, &sid, vec![tip], &id_of_key(&carol));
        let err = check_ai_operator_targets(&ev, &space, &registry).unwrap_err();
        match err {
            ExchangeError::AiRoleViolation(msg) => {
                assert!(msg.contains("not an ai identity"), "msg: {msg}");
            }
            other => panic!("expected AiRoleViolation, got {other:?}"),
        }
    }
}
