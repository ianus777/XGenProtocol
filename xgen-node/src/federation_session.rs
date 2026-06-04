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
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::Mutex;

use xgen_common::event_trace::{trace_event, trace_local, EventDirection, LocalAction, SessionContext};
use xgen_common::xgid::{NodeXgid, SpaceXgid, Xgid};

use xgen_core::{
    federation::federation_policy::{jurisdiction_permits, policy_permits, FederationPolicyStore},
    node::runtime::{space_id_of, EventOrigin, NodeRuntime},
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
///
/// Pass 3 (Surface #3 Q3.1+Q3.3) — `shared_spaces` retyped to `&[SpaceXgid]`
/// (source is `runtime.spaces.keys()` which post-Surface-#1 yields `&SpaceXgid`);
/// `peer_node_id` retyped to `&NodeXgid`. `peer_tips: &BTreeMap<String, String>`
/// stays per Q3.2 (wire-derived from TransportMessage::Hello/Capabilities;
/// §4.3 format-boundary preservation). `session_id` + `negotiated_version` +
/// `negotiated_serialisation` stay `&str` per Q3.4 (descriptive-string slots).
#[allow(clippy::too_many_arguments)]
pub async fn stream_federation_delta<S>(
    conn: &mut Connection<S>,
    runtime: &Arc<Mutex<NodeRuntime>>,
    shared_spaces: &[SpaceXgid],
    peer_tips: &BTreeMap<String, String>,
    peer_node_id: &NodeXgid,
    session_id: &str,
    negotiated_version: &str,
    negotiated_serialisation: &str,
    node_keypair: &SigningKey,
    spaces_dir: &Path,
) -> Result<(), TransportError>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    // §3.3.1 Lock 6: sort by space_id for cross-Space delivery determinism.
    let mut spaces_sorted: Vec<&SpaceXgid> = shared_spaces.iter().collect();
    spaces_sorted.sort_by(|a, b| a.as_str().cmp(b.as_str()));

    let mut last_event_id_sent: String = String::new();

    for space_id in spaces_sorted {
        // Pass 3 (Surface #3 Q3.2) — `peer_tips` is wire-format BTreeMap<String,
        // String>; lookup via String key projection.
        let peer_tip_opt: Option<&str> = peer_tips.get(space_id.as_str()).map(|s| s.as_str());
        let peer_absent = peer_tip_opt.map(|s| s.is_empty()).unwrap_or(true);

        // Snapshot our local tips for this Space — used both as the
        // we-have-events check (a-i symmetry rule) and as prev_events for
        // state.federation_add when the rule fires.
        let our_local_tips: Vec<String> = {
            let rt = runtime.lock().await;
            rt.dag_tips(space_id)
        };
        let we_have_events = !our_local_tips.is_empty();

        // Pass 3 (Surface #4 Q4.6) — `compute_federation_delta_for_space` now
        // takes `&SpaceXgid` + `Option<&EventXgid>`. Project from the wire-format
        // String tip to EventXgid at this xgen-node-vs-typed-helper boundary.
        let peer_tip_typed = peer_tip_opt
            .filter(|s| !s.is_empty())
            .map(|s| xgen_common::xgid::EventXgid::from_xgid(Xgid::new(s.to_string())));
        let mut delta = compute_federation_delta_for_space(
            runtime,
            space_id,
            peer_tip_typed.as_ref(),
        )
        .await;

        // §3.3.1 Lock 2 — a-i symmetry rule: this side builds state.federation_add
        // for `space_id` exactly when the peer's tips map shows that Space absent
        // AND we have events for it. Deterministic from wire-visible tips maps;
        // both sides compute the same answer from the same data.
        if peer_absent && we_have_events {
            // xgen-core wire-format builders take &str at the
            // event-construction boundary (§4.3 format-boundary preservation);
            // project from typed at the call-site.
            let fed_add_ev = sign_event(
                build_federation_add_event(
                    node_keypair,
                    space_id.as_str(),
                    our_local_tips,
                    peer_node_id.as_str(),
                    session_id,
                    negotiated_version,
                    negotiated_serialisation,
                ),
                node_keypair,
            );
            let fed_add_id = fed_add_ev
                .event_id
                .as_ref()
                .map(|e| e.as_str().to_string())
                .unwrap_or_else(|| "(none)".to_string());
            let fed_add_type = fed_add_ev.event_type.to_string();
            trace_local(
                LocalAction::CreateEvent,
                &fed_add_id,
                Some(&fed_add_type),
                Some(space_id.as_str()),
                None,
            );
            {
                let mut rt = runtime.lock().await;
                rt.ingest_event(fed_add_ev.clone());
            }
            if let Err(e) = persist_event(spaces_dir, space_id.as_str(), &fed_add_ev) {
                tracing::error!(space_id = %space_id.as_str(), error = %e, "failed to persist federation_add event to disk (ES-D4)");
            }
            trace_local(
                LocalAction::StoreEvent,
                &fed_add_id,
                None,
                Some(space_id.as_str()),
                None,
            );
            trace_local(
                LocalAction::ApplyEvent,
                &fed_add_id,
                None,
                Some(space_id.as_str()),
                None,
            );
            delta.push(fed_add_ev);
        }

        let fed_session_ctx = SessionContext {
            identity_id: Some(peer_node_id.as_str().to_string()),
            role: Some(xgen_common::event_trace::SpaceRole::Owner),
            space_id: Some(space_id.as_str().to_string()),
        };
        for ev in &delta {
            trace_event(ev, EventDirection::Out, &fed_session_ctx);
            conn.send_event(ev).await?;
            if let Some(id) = &ev.event_id {
                last_event_id_sent = id.as_str().to_string();
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
///
/// Pass 3 (Surface #3 Q3.5) — `local_node_id` retyped to `&NodeXgid`.
/// Source is `home_node_id` from runtime (in-memory `NodeXgid` post-Surface-#1).
pub async fn apply_federation_push(
    event: &Event,
    origin: EventOrigin,
    runtime: &Arc<Mutex<NodeRuntime>>,
    federation_peer_senders: &FederationPeerSenders,
    local_node_id: &NodeXgid,
    // 2b FAC-D3 outbound enforcement. `None` → no policy store (permit all —
    // tests + the node-authored admin push path pass None, preserving today's
    // behaviour). `Some` → filter the push targets by the operator's per-peer
    // policy via the pure `policy_permits` helper, the symmetric outbound to
    // the inbound consult in `process_inbound`.
    policy_store: Option<&Arc<Mutex<FederationPolicyStore>>>,
) {
    let event_id_for_log = event
        .event_id
        .as_ref()
        .map(|e| e.as_str().to_string())
        .unwrap_or_else(|| "(none)".to_string());

    // F-5 §8.5 anti-transitivity guard. The first action in the function;
    // any future maintainer reading this function sees the gate at the top.
    if matches!(origin, EventOrigin::ReceivedViaFederation) {
        // Phase 9 G2: stable trace event for the F-5 guard firing. Source-side
        // honesty assertion target (findings §2.2): a Node that received E
        // via federation must emit this and skip the outbound iteration.
        tracing::debug!(
            event = "federation_push_skipped_origin",
            local_node_id = %local_node_id.as_str(),
            event_id = %event_id_for_log,
            "F-5 anti-transitivity: skipping federation push for event received via federation"
        );
        return;
    }

    // Resolve the Space this event belongs to via the shared `space_id_of`
    // resolver (no-drift per D-067; same resolution dispatch_event uses).
    // State-create events carry empty space_id and anchor on their own
    // event_id.
    let space_id: SpaceXgid = match space_id_of(event) {
        Some(s) => s,
        None => return,
    };

    // Pass 3 (Surface #3 J-134 Finding B / D-079 closure) — drop the prior
    // `Vec<String>` annotation; `SpaceState.federation_nodes` is already
    // `Vec<NodeXgid>` post-Pass 1 Commit 4 (state.rs:132). Type-inference
    // accepts the typed source natively.
    // Arc G PG-04 (CP-1) — read the Space's declared jurisdiction in the SAME
    // runtime read (no extra lock): the Space exists here (we are pushing its
    // events), so the derived `jurisdiction` is authoritative. AND-composed
    // into the per-peer filter below via `jurisdiction_permits`.
    let (federation_nodes, space_jurisdiction): (Vec<NodeXgid>, Option<String>) = {
        let rt = runtime.lock().await;
        match rt.spaces.get(&space_id) {
            Some(s) => (s.federation_nodes.clone(), s.jurisdiction.clone()),
            None => (Vec::new(), None),
        }
    };
    if federation_nodes.is_empty() {
        return;
    }

    // 2b FAC-D3 outbound enforcement — filter the push targets by the
    // operator's per-peer policy before sending. A `Deny` peer or a peer whose
    // `allowed_spaces` excludes this Space is dropped from the push set (no
    // leak outbound); absent policy permits (prime invariant). The policy lock
    // is taken once here and released before the senders lock below.
    let federation_nodes: Vec<NodeXgid> = if let Some(ps) = policy_store {
        let store = ps.lock().await;
        federation_nodes
            .into_iter()
            .filter(|peer| {
                // AND-compose the FAC-D3 space/mode policy with the Arc G PG-04
                // jurisdiction dimension (AG-D5). Absent policy / no jurisdiction
                // allow-list ⇒ permits (prime invariant byte-for-byte).
                let policy = store.get(peer);
                let permit = policy_permits(policy, &space_id)
                    && jurisdiction_permits(policy, space_jurisdiction.as_deref());
                if !permit {
                    tracing::debug!(
                        event = "federation_push_skipped_policy",
                        local_node_id = %local_node_id.as_str(),
                        peer_node_id = %peer.as_str(),
                        space_id = %space_id.as_str(),
                        event_id = %event_id_for_log,
                        "2b FAC-D3: outbound federation push skipped by operator policy"
                    );
                }
                permit
            })
            .collect()
    } else {
        federation_nodes
    };
    if federation_nodes.is_empty() {
        return;
    }

    let senders = federation_peer_senders.lock().await;

    for peer_id in &federation_nodes {
        match senders.get(peer_id) {
            Some(tx) => {
                // R13: try_send (non-blocking, drop on channel-full per F-1b).
                match tx.try_send(OutboundMsg::Event(event.clone())) {
                    Ok(()) => {
                        // Phase 9 G2: stable trace event for successful F-1 push.
                        // Source-side honesty assertion target (findings §2.1).
                        tracing::debug!(
                            event = "federation_push_sent",
                            local_node_id = %local_node_id.as_str(),
                            peer_node_id = %peer_id.as_str(),
                            space_id = %space_id.as_str(),
                            event_id = %event_id_for_log,
                            "F-1 federation push enqueued for peer"
                        );
                    }
                    Err(e) => {
                        // R14: drop-on-peer-down log line (channel-full branch).
                        tracing::warn!(
                            event = "federation_push_dropped_full",
                            local_node_id = %local_node_id.as_str(),
                            peer_node_id = %peer_id.as_str(),
                            space_id = %space_id.as_str(),
                            event_id = %event_id_for_log,
                            reason = %e,
                            "F-1b drop-on-peer-down: federation push dropped (channel full; recovery via tip-exchange on next handshake)"
                        );
                    }
                }
            }
            None => {
                // R14: drop-on-peer-down log line (peer-not-registered branch).
                tracing::warn!(
                    event = "federation_push_dropped_unregistered",
                    local_node_id = %local_node_id.as_str(),
                    peer_node_id = %peer_id.as_str(),
                    space_id = %space_id.as_str(),
                    event_id = %event_id_for_log,
                    "F-1b drop-on-peer-down: federation push dropped (peer unreachable; recovery via tip-exchange on next handshake)"
                );
            }
        }
    }
}

// ── Pass 3 Commit 2a per-surface test T5 (runbook §4.7) ──────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use xgen_common::xgid::Xgid;

    // T5 (Surface #3) — verify the federation_session.rs handler identifier slots
    // retype cleanly while the wire-format String boundary stays preserved per
    // design doc §4.3 v1.1 Q3.2. The load-bearing assertion is at the type level:
    // `apply_federation_push` accepts `&NodeXgid` for `local_node_id` (Q3.5
    // retype) while `peer_tips` (caller-side wire-format BTreeMap<String,String>)
    // stays at the §4.3 wire-format-preservation boundary.
    //
    // This is a compile-time test of the surface contract — if the signatures
    // drift the test won't compile. The runtime path is exercised end-to-end
    // by the existing phase9_two_node_smoke + federation_push_integration
    // module tests.
    #[test]
    fn federation_session_handler_identifier_slots_retyped_at_boundary() {
        // Construct typed XGID values at the typed-side of the boundary.
        let local_node_id: NodeXgid = NodeXgid::from_xgid(Xgid::new(
            "xgen://pubkey/ed25519:local".to_string(),
        ));
        let peer_node_id: NodeXgid = NodeXgid::from_xgid(Xgid::new(
            "xgen://pubkey/ed25519:peer".to_string(),
        ));
        let shared_spaces: Vec<SpaceXgid> = vec![SpaceXgid::from_xgid(Xgid::new(
            "xgen://hash/sha256:space".to_string(),
        ))];
        let peer_tips: std::collections::BTreeMap<String, String> = Default::default();

        // Assert the typed slots compile at their expected types — the
        // load-bearing contract for Surface #3 Q3.1+Q3.3+Q3.5 + the wire-format
        // BTreeMap<String, String> stays per Q3.2.
        let _: &NodeXgid = &local_node_id;
        let _: &NodeXgid = &peer_node_id;
        let _: &[SpaceXgid] = &shared_spaces;
        let _: &std::collections::BTreeMap<String, String> = &peer_tips;

        // Confirm flavour PartialEq via inner Xgid works as expected at the
        // typed comparison boundary (used at federation_session.rs:248-261
        // for the peer iteration site).
        let other_peer = NodeXgid::from_xgid(Xgid::new(
            "xgen://pubkey/ed25519:peer".to_string(),
        ));
        assert_eq!(peer_node_id, other_peer);
        assert_ne!(peer_node_id, local_node_id);
    }
}
