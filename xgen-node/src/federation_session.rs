// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Federation session post-handshake orchestration (F-1a tip exchange, Phase 3).
//!
//! Houses `stream_federation_delta` — the bilateral delta-delivery helper that
//! both `handle_federation_incoming` (receiver side) and any future initiator-
//! side caller (Phase 5 reconnect scheduler, plus the Phase 3 integration tests
//! that double as initiator-side regression coverage per §3.3.1 Lock 7) invoke
//! after the handshake state machine reaches ACTIVE.
//!
//! Cross-references:
//! - Runbook `tasks/FEDERATION_PROPAGATION_COMPLETION.md` §3.3 Locked wire shape
//!   (bilateral tips on Hello + Capabilities).
//! - Runbook §3.3.1 Lock 2 (a-i symmetry rule for `state.federation_add` trigger).
//! - Runbook §3.3.1 Lock 4 (sibling helper `compute_federation_delta_for_space`
//!   in `fanout`, sibling to `collect_sync_history` rather than a generalisation).
//! - Runbook §3.3.1 Lock 5 (`SyncComplete.new_tip` informational semantic).
//! - Runbook §3.3.1 Lock 6 (sorted-by-`space_id` cross-Space ordering).

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;

use ed25519_dalek::SigningKey;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use xgen_common::event_trace::{trace_event, trace_local, EventDirection, LocalAction, SessionContext};

use xgen_core::{
    node::runtime::NodeRuntime,
    space::state::{build_federation_add_event, sign_event},
    transport::connection::{Connection, TransportError},
    wire::types::TransportMessage,
};

use crate::app::persist_event;
use crate::fanout::compute_federation_delta_for_space;

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
