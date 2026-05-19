// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Federation session post-handshake orchestration.
//!
//! Phase 3 introduced `stream_federation_delta` (F-1a tip exchange — bilateral
//! delta delivery after handshake reaches ACTIVE).
//!
//! Phase 4 adds `apply_federation_push` (F-1 / F-1b / F-5 — push locally-
//! accepted events to federated peers; drop-on-peer-down with no outbound
//! queue; F-5 anti-transitivity guard via `EventOrigin`).
//!
//! Cross-references:
//! - Runbook `tasks/FEDERATION_PROPAGATION_COMPLETION.md` §3.3 Locked wire shape
//!   (bilateral tips on Hello + Capabilities).
//! - Runbook §3.3.1 Lock 2 (a-i symmetry rule for `state.federation_add` trigger).
//! - Runbook §3.3.1 Lock 4 (sibling helper `compute_federation_delta_for_space`
//!   in `fanout`, sibling to `collect_sync_history` rather than a generalisation).
//! - Runbook §3.3.1 Lock 5 (`SyncComplete.new_tip` informational semantic).
//! - Runbook §3.3.1 Lock 6 (sorted-by-`space_id` cross-Space ordering).
//! - Runbook §3.4.1 Q1 lock (`EventOrigin` runtime parameter).
//! - Runbook §3.4.1 Q2 lock (`FederationPeerSenders` as the sibling of
//!   `ClientSenders`; `SpaceState.federation_nodes` stays the single source of
//!   truth for federation membership).
//! - Runbook §3.4.1 R13 (try_send not send — drop on channel-full per F-1b).
//! - Runbook §3.4.1 R14 (drop-on-peer-down log line).

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;

use ed25519_dalek::SigningKey;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use xgen_common::event_trace::{trace_event, trace_local, EventDirection, LocalAction, SessionContext};

use xgen_core::{
    node::runtime::{EventOrigin, NodeRuntime},
    space::state::{build_federation_add_event, sign_event},
    transport::connection::{Connection, TransportError},
    wire::types::{Event, TransportMessage},
};

use crate::app::persist_event;
use crate::fanout::{compute_federation_delta_for_space, FederationPeerSenders, OutboundMsg};

/// F-1a bilateral delta delivery (runbook §3.3 Locked wire shape + §3.3.1
/// Locks 2, 4, 5, 6).
///
/// For each Space in `shared_spaces` (iterated in sorted order per §3.3.1
/// Lock 6 for determinism):
/// 1. Compute the per-Space delta from the peer's tip (absent → full history).
/// 2. Apply the a-i symmetry rule (§3.3.1 Lock 2): if the peer's `tips[S]` is
///    absent AND this Node has events for `S`, build & ingest & persist
///    `state.federation_add(S)` and append it to the delta (DAG audit trail
///    of relationship establishment for this Space). Idempotent across replays
///    by the wire-shape: only fires when peer is genuinely new for the Space.
/// 3. Send every event in the delta in topological order over the connection.
///
/// After all Spaces: send ONE `TransportMessage::SyncComplete` carrying the
/// cross-Space whole-batch terminator (§3.3.1 Lock 5). `new_tip` is best-effort
/// — the last `event_id` sent across all Spaces, or empty if the delta was
/// fully empty. Receivers MUST NOT compare it to a single-Space tip; they
/// track per-Space tips through event ingestion.
///
/// `our_node_id` is needed because state.federation_add carries the home
/// Node's sender; this Node IS the home for Spaces it has events for under
/// the a-i rule.
pub async fn stream_federation_delta(
    conn: &mut Connection<TcpStream>,
    runtime: &Arc<Mutex<NodeRuntime>>,
    shared_spaces: &[String],
    peer_tips: &BTreeMap<String, String>,
    peer_node_id: &str,
    session_id: &str,
    negotiated_version: &str,
    negotiated_serialisation: &str,
    node_keypair: &SigningKey,
    spaces_dir: &Path,
) -> Result<(), TransportError> {
    // §3.3.1 Lock 6: sort by space_id for cross-Space delivery determinism.
    let mut spaces_sorted: Vec<&String> = shared_spaces.iter().collect();
    spaces_sorted.sort();

    let mut last_event_id_sent: String = String::new();

    for space_id in spaces_sorted {
        let peer_tip_opt: Option<&str> = peer_tips.get(space_id).map(|s| s.as_str());
        let peer_absent = peer_tip_opt.map(|s| s.is_empty()).unwrap_or(true);

        // Snapshot our local tips for this Space — used both as the
        // we-have-events check (a-i symmetry rule) and as prev_events for
        // state.federation_add when the rule fires.
        let our_local_tips: Vec<String> = {
            let rt = runtime.lock().await;
            rt.dag_tips(space_id)
        };
        let we_have_events = !our_local_tips.is_empty();

        let mut delta =
            compute_federation_delta_for_space(runtime, space_id, peer_tip_opt).await;

        // §3.3.1 Lock 2 — a-i symmetry rule: this side builds state.federation_add
        // for `space_id` exactly when the peer's tips map shows that Space absent
        // AND we have events for it. Deterministic from wire-visible tips maps;
        // both sides compute the same answer from the same data.
        if peer_absent && we_have_events {
            let fed_add_ev = sign_event(
                build_federation_add_event(
                    node_keypair,
                    space_id,
                    our_local_tips,
                    peer_node_id,
                    session_id,
                    negotiated_version,
                    negotiated_serialisation,
                ),
                node_keypair,
            );
            let fed_add_id = fed_add_ev.event_id.as_deref().unwrap_or("(none)").to_string();
            let fed_add_type = fed_add_ev.event_type.to_string();
            trace_local(
                LocalAction::CreateEvent,
                &fed_add_id,
                Some(&fed_add_type),
                Some(space_id),
                None,
            );
            {
                let mut rt = runtime.lock().await;
                rt.ingest_event(fed_add_ev.clone());
            }
            persist_event(spaces_dir, space_id, &fed_add_ev);
            trace_local(LocalAction::StoreEvent, &fed_add_id, None, Some(space_id), None);
            trace_local(LocalAction::ApplyEvent, &fed_add_id, None, Some(space_id), None);
            delta.push(fed_add_ev);
        }

        let fed_session_ctx = SessionContext {
            identity_id: Some(peer_node_id.to_string()),
            role: Some(xgen_common::event_trace::SpaceRole::Owner),
            space_id: Some(space_id.to_string()),
        };
        for ev in &delta {
            trace_event(ev, EventDirection::Out, &fed_session_ctx);
            conn.send_event(ev).await?;
            if let Some(id) = &ev.event_id {
                last_event_id_sent = id.clone();
            }
        }
    }

    // §3.3.1 Lock 5: cross-Space whole-batch terminator. `new_tip` is
    // informational — receivers track per-Space tips through event ingestion,
    // not through this field. `since` is empty because there's no sync_request
    // cursor to echo (this is a handshake-driven delta, not a pull response).
    let complete = TransportMessage::SyncComplete {
        protocol_version: "0.1".to_string(),
        since: String::new(),
        new_tip: last_event_id_sent,
        continue_from: None,
    };
    conn.send_transport(&complete).await?;
    Ok(())
}

/// F-1 federation event push (Phase 4 — runbook §3.4 + §3.4.1 Q1/Q2 locks +
/// R13/R14 Clair-latitude items).
///
/// Sibling of `apply_fanout` — runs AFTER local fan-out as a separate
/// concern. For each federated peer Node in the event's `SpaceState.
/// federation_nodes`, drains `OutboundMsg::Event(event.clone())` into the
/// peer's active session sender (if registered in `FederationPeerSenders`).
///
/// **F-5 origin gating (§8.5).** Hard guard at the top: events with
/// `EventOrigin::ReceivedViaFederation` MUST NOT be pushed onward to
/// federation peers — anti-transitivity. The implementation marker is
/// wire-invisible: a peer cannot tell from the wire which side originated
/// an event, but this Node's `process_inbound` knew where it arrived from
/// and threaded the origin through.
///
/// **F-1b drop-on-peer-down (§4.5).** No outbound queue. Sends use
/// `try_send` (non-blocking); channel-full or peer-not-registered both
/// produce an observability log line and continue iterating other peers.
/// Recovery for dropped pushes is the peer's responsibility via F-1a
/// tip-exchange on the next handshake.
///
/// **No-op cases (silent, not log-spam):**
/// - Event not bound to a Space (`event.space_id` empty AND event_type not
///   a Space-creation root) — no peers to address.
/// - Space has no `federation_nodes` (not yet federated) — nothing to do.
/// - Origin is `ReceivedViaFederation` — F-5 guard.
pub async fn apply_federation_push(
    event: &Event,
    origin: EventOrigin,
    runtime: &Arc<Mutex<NodeRuntime>>,
    federation_peer_senders: &FederationPeerSenders,
) {
    // F-5 §8.5 anti-transitivity guard. The first action in the function;
    // any future maintainer reading this function sees the gate at the top.
    if matches!(origin, EventOrigin::ReceivedViaFederation) {
        return;
    }

    // Resolve the Space this event belongs to. State-create events carry
    // empty space_id and use their own event_id as the Space anchor
    // (matches the resolution in NodeRuntime::ingest_event / dispatch_event).
    let space_id = if event.space_id.is_empty() {
        match event.event_id.as_deref() {
            Some(id) => id.to_string(),
            None => return,
        }
    } else {
        event.space_id.clone()
    };

    // Snapshot the federated peer list and event_id under runtime lock.
    let federation_nodes: Vec<String> = {
        let rt = runtime.lock().await;
        rt.spaces
            .get(&space_id)
            .map(|s| s.federation_nodes.clone())
            .unwrap_or_default()
    };
    if federation_nodes.is_empty() {
        return;
    }

    let event_id_for_log = event.event_id.as_deref().unwrap_or("(none)").to_string();
    let senders = federation_peer_senders.lock().await;

    for peer_id in &federation_nodes {
        match senders.get(peer_id) {
            Some(tx) => {
                // R13: try_send (non-blocking, drop on channel-full per F-1b).
                if let Err(e) = tx.try_send(OutboundMsg::Event(event.clone())) {
                    // R14: drop-on-peer-down log line (channel-full branch).
                    tracing::warn!(
                        peer_node_id = %peer_id,
                        space_id = %space_id,
                        event_id = %event_id_for_log,
                        reason = %e,
                        "F-1b drop-on-peer-down: federation push dropped (channel full; recovery via tip-exchange on next handshake)"
                    );
                }
            }
            None => {
                // R14: drop-on-peer-down log line (peer-not-registered branch).
                tracing::warn!(
                    peer_node_id = %peer_id,
                    space_id = %space_id,
                    event_id = %event_id_for_log,
                    "F-1b drop-on-peer-down: federation push dropped (peer unreachable; recovery via tip-exchange on next handshake)"
                );
            }
        }
    }
}
