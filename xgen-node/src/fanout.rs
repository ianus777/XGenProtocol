// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

// Local fan-out for the Node — broadcast accepted Events from one connected
// client to every other client that is a member of the Event's Space, and
// push history to brand-new joiners.
//
// The actual WebSocket I/O lives in `main.rs::handle_connection`. This module
// owns the data shapes and the routing decisions so they are unit-testable
// without spawning real sockets.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use tokio::sync::{mpsc, Mutex};
use crate::node::runtime::NodeRuntime;
use crate::wire::types::Event;
use xgen_common::xgid::{EventXgid, IdentityXgid, NodeXgid, SpaceXgid, Xgid};

/// Outbound message the fan-out path pushes into a connected client's handler.
///
/// `transport.sync_complete` (F-6, spec 3.3.6) replaces the 500ms quiet-time
/// heuristic with an explicit end-of-batch signal. After a sync_request, the
/// dispatcher sends `HistoryBatch { events }` then `SyncComplete { .. }` in
/// that order; the WebSocket drain arm in `app.rs` translates the latter into
/// a `TransportMessage::SyncComplete` on the wire.
#[derive(Debug, Clone)]
pub enum OutboundMsg {
    /// Deliver an Event to the client (Inbound from the client's perspective).
    Event(Event),
    /// Stream of historical events in response to a `transport.sync_request`
    /// or to a fresh `membership.join`.
    HistoryBatch { events: Vec<Event> },
    /// Explicit end-of-batch signal for a `transport.sync_request` response
    /// (F-6 / F-7). The drain arm wires this into `TransportMessage::SyncComplete`.
    SyncComplete {
        since: String,
        new_tip: String,
        continue_from: Option<String>,
    },
}

/// Per-connection outbound channels keyed by authenticated `identity_id`.
/// Phase 1 simplification: one device per Identity, so one channel per Identity.
/// On disconnect the entry is removed; on reconnect a new entry is installed.
///
/// Pass 3 (Surface #4 Q4.2) — HashMap key retyped to `IdentityXgid`.
/// xgen-node-internal type (channels never escape to xgen-client); Joe-lock
/// Q-E at design walk.
pub type ClientSenders = Arc<Mutex<HashMap<IdentityXgid, mpsc::Sender<OutboundMsg>>>>;

/// Active federation peer sessions, keyed by peer `node_id` URI (Phase 4,
/// runbook §3.4.1 Q2 lock). Each entry is the outbound mpsc Sender into a
/// live `handle_federation_incoming` (or future Phase 5 initiator-side
/// session) task — `apply_federation_push` drains `OutboundMsg::Event`
/// through these senders to push locally-accepted events to federated peers.
///
/// Mirrors `ClientSenders` shape: single source of truth for active-session
/// presence. Space-membership lookup ("which peers are federated for Space S")
/// stays on `SpaceState.federation_nodes` — the registry does NOT cache that
/// to avoid the two-sources-of-truth drift surface that Q2 Option B would
/// have introduced.
///
/// F-2a (one WS per pair bidirectional) justifies single-sender-per-peer.
/// On peer disconnect the entry is removed; on next handshake-to-ACTIVE a
/// fresh entry is installed.
///
/// Pass 3 (Surface #4 Q4.3) — HashMap key retyped to `NodeXgid`.
pub type FederationPeerSenders = Arc<Mutex<HashMap<NodeXgid, mpsc::Sender<OutboundMsg>>>>;

/// Result of processing an inbound message — describes what the fan-out path
/// should broadcast to other connected clients. Returning a description
/// (instead of doing the broadcast inside the runtime lock) keeps lock scope
/// short and lets the fan-out path lock `ClientSenders` separately.
pub struct FanoutRequest {
    /// The accepted Event to broadcast to all members of its Space (the author
    /// is filtered out at fan-out time).
    pub event: Option<Event>,
    /// `Some(joiner_id)` when the inbound was a fresh `membership.join` that
    /// added a new Space member. The fan-out path pushes the Space's full
    /// event history (in causal order, excluding the join event itself) to
    /// the joiner, so the new client sees prior `state.*` and `membership.*`
    /// events. Phase 1 reuses this for both join-time history and reconnect.
    ///
    /// Pass 3 (Surface #4 Q4.5) — retyped to `Option<IdentityXgid>`.
    pub new_joiner: Option<IdentityXgid>,
}

impl FanoutRequest {
    pub fn none() -> Self {
        Self { event: None, new_joiner: None }
    }
}

/// Resolve the Space ID a given Event addresses: `state.space_create` and
/// `state.dm_space_create` carry an empty `space_id` and use their own event_id;
/// every other Event carries the Space ID explicitly.
///
/// Pass 3 (Surface #4 Q4.1) — return retyped to `Option<SpaceXgid>` (owned)
/// per Joe-lock Q-C at design walk. State-create branch constructs SpaceXgid
/// from EventXgid (flavour change); non-state-create branch clones
/// `event.space_id` (already SpaceXgid). General rule recorded at §4.4:
/// parameters favour borrowed, returns favour owned when any branch must
/// construct.
pub fn event_space_id(event: &Event) -> Option<SpaceXgid> {
    if event.space_id.as_str().is_empty() {
        event
            .event_id
            .as_ref()
            .map(|e| SpaceXgid::from_xgid(Xgid::new(e.as_str().to_string())))
    } else {
        Some(event.space_id.clone())
    }
}

/// Broadcast a `FanoutRequest` to the relevant connected clients.
///
/// Locks the runtime briefly to fetch the Space's member list and (when
/// applicable) the Space's event history, then drops the runtime lock before
/// acquiring the `ClientSenders` mutex. This keeps the critical sections short
/// and prevents the fan-out path from blocking other handlers.
///
/// Pass 3 (Surface #4 Q4.4) — `author_id` retyped to `&IdentityXgid`.
pub async fn apply_fanout(
    req: FanoutRequest,
    author_id: &IdentityXgid,
    runtime: &Arc<Mutex<NodeRuntime>>,
    client_senders: &ClientSenders,
) {
    let event = match req.event {
        Some(ev) => ev,
        None => return,
    };
    let space_id = match event_space_id(&event) {
        Some(s) => s,
        None => return,
    };
    let event_id = event.event_id.clone();

    // Pass 3 (Surface #1 + Surface #4 Q4.9) — recipients collected from
    // SpaceState.members (HashMap<IdentityXgid, _>) natively yields IdentityXgid.
    let (recipients, history_for_joiner): (Vec<IdentityXgid>, Option<Vec<Event>>) = {
        let rt = runtime.lock().await;
        let space = match rt.spaces.get(&space_id) {
            Some(s) => s,
            None => return,
        };
        let recipients = space.members.keys().cloned().collect::<Vec<_>>();
        let history = if req.new_joiner.is_some() {
            rt.stores.get(&space_id).map(|store| {
                let all: Vec<Event> = store.values().cloned().collect();
                let sorted = topological_sort_events(all);
                sorted
                    .into_iter()
                    .filter(|e| e.event_id != event_id)
                    .collect()
            })
        } else {
            None
        };
        (recipients, history)
    };

    let senders = client_senders.lock().await;
    let event_id_for_log = event
        .event_id
        .as_ref()
        .map(|e| e.as_str().to_string())
        .unwrap_or_else(|| "(none)".to_string());

    for rid in &recipients {
        if rid == author_id {
            continue;
        }
        if let Some(tx) = senders.get(rid) {
            // Phase 9 G3: stable trace events on the fan-out delivery path.
            // Pairs with Scenario 1's honesty check #2 (fanout_delivered must
            // observe on the destination Node for federated events) and
            // Scenario 2's destination-side absence assertion (no
            // fanout_delivered for E on a non-federated peer's local clients).
            match tx.try_send(OutboundMsg::Event(event.clone())) {
                Ok(()) => {
                    tracing::debug!(
                        event = "fanout_delivered",
                        client_id = %rid.as_str(),
                        event_id = %event_id_for_log,
                        "local fan-out: event delivered to client"
                    );
                }
                Err(_) => {
                    tracing::warn!(
                        event = "fanout_dropped_channel_full",
                        client_id = %rid.as_str(),
                        event_id = %event_id_for_log,
                        "local fan-out: client channel full, event dropped"
                    );
                }
            }
        }
    }

    if let (Some(joiner_id), Some(history)) = (req.new_joiner.as_ref(), history_for_joiner) {
        if !history.is_empty() {
            if let Some(tx) = senders.get(joiner_id) {
                let _ = tx.try_send(OutboundMsg::HistoryBatch { events: history });
            }
        }
    }
}

/// Topological sort of a set of Events by `prev_events`. Events whose
/// predecessors are all already emitted come first. The DAG is acyclic by
/// construction (self-references rejected at insertion time), so this
/// terminates. Used to order history-push so the receiver sees parents
/// before children.
///
/// Pass 3 (Surface #4 inheritance) — emitted set retyped to `HashSet<EventXgid>`;
/// `event_id` slots are `Option<EventXgid>` post-Pass-1; comparison uses typed
/// PartialEq via inner Xgid.
pub fn topological_sort_events(mut events: Vec<Event>) -> Vec<Event> {
    let mut emitted: HashSet<EventXgid> = HashSet::new();
    let mut out: Vec<Event> = Vec::with_capacity(events.len());
    let mut changed = true;
    while !events.is_empty() && changed {
        changed = false;

        // D-076 wire-order determinism (locked at topological-sort
        // design-phase close 2026-05-22; sibling-distinct from D-067
        // at code-organisation layer and D-075 at event-model layer;
        // all four lock no-drift-surface properties explicitly across
        // four protocol layers — D-067 + D-070 + D-075 + D-076).
        //
        // Sort ready siblings by event_id lexicographically. event_id is
        // content-hash-derived per Appendix J (xgen_appendix_j_en.md), so
        // the sort key is byte-stable across senders with identical Space
        // history, which is exactly what D-076's "two senders with
        // identical state produce byte-identical federation deltas"
        // contract obligates.
        //
        // Pass 3 (Surface #4 Q4.8) — sort works through Option's Ord using
        // EventXgid's Ord via inner Xgid's Ord; no retype needed at the
        // sort line itself.
        events.sort_by(|a, b| a.event_id.cmp(&b.event_id));

        let mut i = 0;
        while i < events.len() {
            let ready = events[i].prev_events.iter().all(|p| {
                emitted.contains(p) || !events.iter().any(|e| e.event_id.as_ref() == Some(p))
            });
            if ready {
                let ev = events.remove(i);
                if let Some(id) = &ev.event_id {
                    emitted.insert(id.clone());
                }
                out.push(ev);
                changed = true;
            } else {
                i += 1;
            }
        }
    }
    // Append any stragglers (cyclic or with unknown predecessors — neither
    // should occur in practice, but guarantee the function preserves all input).
    out.extend(events);
    out
}

/// Collect events to return in response to `transport.sync_request` (spec 3.3.6 + F-7).
/// Returns events from every Space the requester is a member of. If `since` is
/// non-empty, returns only events whose position follows `since` in the
/// whole-batch ordering (HashMap iteration across Spaces, topo-sort within
/// each Space). At most `limit` events are returned per call (F-7a default 1000
/// applied at the caller).
///
/// Returns `(events, continue_from)`:
/// - `events` — up to `limit` events for this page.
/// - `continue_from` — `Some(event_id)` of the last event in the page when more
///   events remain after `limit`; `None` when this page consumed every
///   remaining event (catch-up complete from the responder's side).
///
/// **Whole-batch pagination model (Clair-locked at Phase 1, F-6/F-7).** The
/// runbook left cross-Space behaviour as Clair's latitude — per-Space
/// `SyncComplete` (one per Space with that Space's tip) or whole-batch (one
/// `SyncComplete` per page across the union event stream). Whole-batch lands
/// because it matches the existing flat-Vec model in this function: events
/// are already concatenated in HashMap iteration order, and the pagination
/// cursor is a single event_id in that flattened sequence. Per-Space tip
/// enrichment is future work.
///
/// Pagination cursor stability: HashMap iteration order is stable within a
/// Rust process when no mutations happen. If a Space is added between
/// paginated requests, the order can change and the next page may miss events
/// from the new Space — recovery via F-1a tip-exchange on the next handshake.
/// Acceptable for Phase 1 / 2 scale; revisit if profiling shows the corner
/// case matters.
///
/// Pass 3 (Surface #4 Q4.7) — `requester_id` retyped to `&IdentityXgid`
/// (in-memory typed slot); `since` stays `&str` and `continue_from: Option<String>`
/// stays String — both wire-format pagination cursors per §4.3 format-boundary
/// preservation (TransportMessage::SyncRequest::since / SyncComplete::continue_from).
pub async fn collect_sync_history(
    runtime: &Arc<Mutex<NodeRuntime>>,
    requester_id: &IdentityXgid,
    since: &str,
    limit: usize,
) -> (Vec<Event>, Option<String>) {
    let rt = runtime.lock().await;
    // Build the candidate sequence (all member-Space events in whole-batch order).
    let mut candidate: Vec<Event> = Vec::new();
    for (space_id, space) in &rt.spaces {
        if !space.is_member(requester_id.as_str()) {
            continue;
        }
        if let Some(store) = rt.stores.get(space_id) {
            let all: Vec<Event> = store.values().cloned().collect();
            candidate.extend(topological_sort_events(all));
        }
    }
    drop(rt);

    // Resume past the `since` cursor when non-empty.
    let start = if since.is_empty() {
        0
    } else {
        match candidate
            .iter()
            .position(|e| e.event_id.as_ref().map(|x| x.as_str()) == Some(since))
        {
            Some(i) => i + 1,
            None => return (Vec::new(), None),
        }
    };
    let tail = &candidate[start..];

    // Pagination cap.
    let take = tail.len().min(limit);
    let page: Vec<Event> = tail[..take].to_vec();
    let continue_from = if take < tail.len() {
        page.last()
            .and_then(|e| e.event_id.as_ref().map(|x| x.as_str().to_string()))
    } else {
        None
    };
    (page, continue_from)
}

/// F-1a per-Space delta computation for federation handshake tip-exchange
/// (runbook §3.3 Locked wire shape + §3.3.1 Lock 4).
///
/// Returns events for `space_id` in topological order that follow the peer's
/// tip in the DAG. `peer_tip` semantics:
/// - `None` or `Some("")` → peer has no tip for this Space (brand-new or
///   never-yet-replicated) → return the full topological history.
/// - `Some(event_id)` not present in the local DAG → peer's tip is unknown to
///   us → return empty (we can't compute a delta without the cursor; recovery
///   is the peer's job via a future handshake or pull). Returns empty rather
///   than full history to avoid spurious re-delivery.
/// - `Some(event_id)` present → return events whose position follows the cursor
///   in the topo-sorted sequence.
///
/// Sibling to `collect_sync_history` per the R2 lock: that function is
/// Identity-membership-shaped (one cursor, all member Spaces concatenated);
/// this one is per-peer-per-Space-tip-shaped. Two helpers, two callers, no
/// drift surface — collect_sync_history serves client `sync_request` flows,
/// this one serves federation handshake delta delivery.
///
/// Pass 3 (Surface #4 Q4.6) — `space_id` retyped to `&SpaceXgid`; `peer_tip`
/// retyped to `Option<&EventXgid>` (both in-memory Rust slots; wire→typed
/// conversion happens at the boundary in `stream_federation_delta`).
pub async fn compute_federation_delta_for_space(
    runtime: &Arc<Mutex<NodeRuntime>>,
    space_id: &SpaceXgid,
    peer_tip: Option<&EventXgid>,
) -> Vec<Event> {
    let rt = runtime.lock().await;
    let store = match rt.stores.get(space_id) {
        Some(s) => s,
        None => return Vec::new(),
    };
    // D-076 belt-and-braces: sort the HashMap-iteration vector by event_id
    // before passing to topological_sort_events. The primitive itself
    // canonicalises ready-sibling order (line ~193); this sort ensures the
    // feed into the primitive is also canonical so the end-to-end
    // federation-delta computation is Q3.ii-compliant from store-read to
    // wire-emit. Per design task file §4.1 (Q2 middle's letter: "primitive
    // fixed + feed canonical").
    let mut all: Vec<Event> = store.values().cloned().collect();
    all.sort_by(|a, b| a.event_id.cmp(&b.event_id));
    drop(rt);

    let sorted = topological_sort_events(all);
    let tip_str = peer_tip.map(|t| t.as_str()).unwrap_or("");
    if tip_str.is_empty() {
        return sorted;
    }
    match sorted
        .iter()
        .position(|e| e.event_id.as_ref().map(|x| x.as_str()) == Some(tip_str))
    {
        Some(i) => sorted.into_iter().skip(i + 1).collect(),
        None => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use crate::crypto::encoding;
    use crate::identity::{
        keypair,
        registry::{DeviceRecord, IdentityRecord},
    };
    use crate::message::exchange::build_message_text_event;
    use crate::space::state::{
        build_membership_event, build_room_create_event, build_space_create_event, sign_event,
    };
    use crate::wire::types::EventType;
    use xgen_common::xgid::{EventXgid, IdentityXgid, NodeXgid, SpaceXgid, Xgid};

    // Pass 3 Commit 2a test-fixture helpers.
    fn idx(s: &str) -> IdentityXgid {
        IdentityXgid::from_xgid(Xgid::new(s.to_string()))
    }
    fn ndx(s: &str) -> NodeXgid {
        NodeXgid::from_xgid(Xgid::new(s.to_string()))
    }
    fn sdx(s: &str) -> SpaceXgid {
        SpaceXgid::from_xgid(Xgid::new(s.to_string()))
    }
    #[allow(dead_code)]
    fn edx(s: &str) -> EventXgid {
        EventXgid::from_xgid(Xgid::new(s.to_string()))
    }
    fn event_id_str(ev: &Event) -> String {
        ev.event_id
            .as_ref()
            .expect("event must have event_id")
            .as_str()
            .to_string()
    }

    const HOME: &str = "xgen://pubkey/ed25519:NODE";

    fn pubkey_uri(key: &ed25519_dalek::SigningKey) -> String {
        format!(
            "xgen://pubkey/ed25519:{}",
            encoding::encode(key.verifying_key().as_bytes())
        )
    }

    fn make_identity_record(id: &str) -> IdentityRecord {
        IdentityRecord {
            identity_id: idx(id),
            display_name: None,
            is_ai: false,
            ai_capabilities: None,
            registered_at: "2026-05-16T00:00:00.000Z".to_string(),
            trust_assertion: None,
            devices: vec![DeviceRecord {
                device_id: id.to_string(),
                device_name: None,
                authorised_at: "2026-05-16T00:00:00.000Z".to_string(),
            }],
            home_node: ndx(HOME),
            update_version: 0,
            revoked: false,
            revoked_at: None,
            revocation_reason: None,
        }
    }

    /// Build a NodeRuntime with three identities (alice, bob, carol), a Space
    /// owned by alice, and a Room. Returns runtime + space_id + room_id +
    /// signing keys + each identity_id.
    fn setup_three_member_space() -> (
        NodeRuntime,
        String,
        String,
        ed25519_dalek::SigningKey,
        ed25519_dalek::SigningKey,
        ed25519_dalek::SigningKey,
    ) {
        let node_key = keypair::generate();
        let mut rt = NodeRuntime::new(node_key);
        let alice = keypair::generate();
        let bob = keypair::generate();
        let carol = keypair::generate();
        let alice_id = pubkey_uri(&alice);
        let bob_id = pubkey_uri(&bob);
        let carol_id = pubkey_uri(&carol);
        rt.register_identity(make_identity_record(&alice_id)).unwrap();
        rt.register_identity(make_identity_record(&bob_id)).unwrap();
        rt.register_identity(make_identity_record(&carol_id)).unwrap();

        // Space + Room created by alice
        let space_ev = sign_event(
            build_space_create_event(&alice, "Test", None, 1, HOME),
            &alice,
        );
        let space_id: String = event_id_str(&space_ev);
        rt.ingest_event(space_ev);

        let room_ev = sign_event(
            build_room_create_event(&alice, &space_id, "general", None),
            &alice,
        );
        let room_id: String = event_id_str(&room_ev);
        rt.ingest_event(room_ev);

        // Bob is invited and joins.
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
        rt.ingest_event(invite);
        let bob_join = sign_event(
            build_membership_event(&bob, &space_id, "", EventType::MembershipJoin, json!({})),
            &bob,
        );
        rt.ingest_event(bob_join);

        // Carol is invited and joins.
        let invite_c = sign_event(
            build_membership_event(
                &alice,
                &space_id,
                "",
                EventType::MembershipInvite,
                json!({ "target_identity": carol_id, "role": "member" }),
            ),
            &alice,
        );
        rt.ingest_event(invite_c);
        let carol_join = sign_event(
            build_membership_event(&carol, &space_id, "", EventType::MembershipJoin, json!({})),
            &carol,
        );
        rt.ingest_event(carol_join);

        (rt, space_id, room_id, alice, bob, carol)
    }

    // Test helper currently used by no test in this file — kept available
    // for future fanout tests that need a directly-installed sender (Phase 7
    // shipped without needing it; subsequent fanout work may pick it up).
    #[allow(dead_code)]
    fn install_sender(senders: &ClientSenders, identity_id: &str) -> mpsc::Receiver<OutboundMsg> {
        let (tx, rx) = mpsc::channel::<OutboundMsg>(256);
        let senders_clone = senders.clone();
        let id = idx(identity_id);
        let handle = tokio::runtime::Handle::current();
        handle.block_on(async move {
            senders_clone.lock().await.insert(id, tx);
        });
        rx
    }

    #[tokio::test]
    async fn message_fans_out_to_other_members_and_excludes_author() {
        let (rt, space_id, room_id, alice, bob, carol) = setup_three_member_space();
        let alice_id = pubkey_uri(&alice);
        let bob_id = pubkey_uri(&bob);
        let carol_id = pubkey_uri(&carol);
        let runtime = Arc::new(Mutex::new(rt));
        let senders: ClientSenders = Arc::new(Mutex::new(HashMap::new()));

        let (tx_a, mut rx_a) = mpsc::channel::<OutboundMsg>(64);
        let (tx_b, mut rx_b) = mpsc::channel::<OutboundMsg>(64);
        let (tx_c, mut rx_c) = mpsc::channel::<OutboundMsg>(64);
        senders.lock().await.insert(idx(&alice_id), tx_a);
        senders.lock().await.insert(idx(&bob_id), tx_b);
        senders.lock().await.insert(idx(&carol_id), tx_c);

        // Get DAG tip for alice's outbound message.
        let space_id_typed = sdx(&space_id);
        let tip = runtime.lock().await.dag_tips(&space_id_typed)[0].clone();
        let msg = sign_event(
            build_message_text_event(&alice, &space_id, &room_id, vec![tip], "hello"),
            &alice,
        );

        let req = FanoutRequest { event: Some(msg.clone()), new_joiner: None };
        let alice_id_typed = idx(&alice_id);
        apply_fanout(req, &alice_id_typed, &runtime, &senders).await;

        // Bob and Carol must receive the event; Alice (author) must not.
        let recv_b = rx_b.recv().await.expect("bob receives");
        let recv_c = rx_c.recv().await.expect("carol receives");
        match recv_b {
            OutboundMsg::Event(ev) => assert_eq!(ev.event_id, msg.event_id),
            _ => panic!("expected Event"),
        }
        match recv_c {
            OutboundMsg::Event(ev) => assert_eq!(ev.event_id, msg.event_id),
            _ => panic!("expected Event"),
        }
        // Alice's channel must be empty (the author is excluded).
        assert!(rx_a.try_recv().is_err());
        let _ = bob;
        let _ = carol;
    }

    #[tokio::test]
    async fn new_joiner_receives_full_history_push() {
        // Bootstrap a Space with alice + bob and several messages. Then carol
        // joins fresh; she must receive the full prior history (Space, Room,
        // both prior joins, plus the messages).
        let node_key = keypair::generate();
        let mut rt = NodeRuntime::new(node_key);
        let alice = keypair::generate();
        let bob = keypair::generate();
        let carol = keypair::generate();
        let alice_id = pubkey_uri(&alice);
        let bob_id = pubkey_uri(&bob);
        let carol_id = pubkey_uri(&carol);
        rt.register_identity(make_identity_record(&alice_id)).unwrap();
        rt.register_identity(make_identity_record(&bob_id)).unwrap();
        rt.register_identity(make_identity_record(&carol_id)).unwrap();

        let space_ev =
            sign_event(build_space_create_event(&alice, "Test", None, 1, HOME), &alice);
        let space_id: String = event_id_str(&space_ev);
        rt.ingest_event(space_ev);
        let room_ev = sign_event(
            build_room_create_event(&alice, &space_id, "general", None),
            &alice,
        );
        let room_id: String = event_id_str(&room_ev);
        rt.ingest_event(room_ev);
        // Bob joins.
        rt.ingest_event(sign_event(
            build_membership_event(
                &alice,
                &space_id,
                "",
                EventType::MembershipInvite,
                json!({ "target_identity": bob_id, "role": "member" }),
            ),
            &alice,
        ));
        rt.ingest_event(sign_event(
            build_membership_event(&bob, &space_id, "", EventType::MembershipJoin, json!({})),
            &bob,
        ));
        // Alice posts a message.
        let space_id_typed = sdx(&space_id);
        let tip = rt.dag_tips(&space_id_typed)[0].clone();
        let alice_msg = sign_event(
            build_message_text_event(&alice, &space_id, &room_id, vec![tip], "first"),
            &alice,
        );
        rt.ingest_event(alice_msg);

        // Carol now joins. Bob's join is the relevant historical event row 7
        // analogue from the S1 pairing table — carol must see it.
        let carol_invite = sign_event(
            build_membership_event(
                &alice,
                &space_id,
                "",
                EventType::MembershipInvite,
                json!({ "target_identity": carol_id, "role": "member" }),
            ),
            &alice,
        );
        rt.ingest_event(carol_invite);
        let carol_join = sign_event(
            build_membership_event(&carol, &space_id, "", EventType::MembershipJoin, json!({})),
            &carol,
        );
        let carol_join_id: String = event_id_str(&carol_join);
        rt.ingest_event(carol_join.clone());

        let runtime = Arc::new(Mutex::new(rt));
        let senders: ClientSenders = Arc::new(Mutex::new(HashMap::new()));
        let (tx_a, _rx_a) = mpsc::channel::<OutboundMsg>(64);
        let (tx_b, _rx_b) = mpsc::channel::<OutboundMsg>(64);
        let (tx_c, mut rx_c) = mpsc::channel::<OutboundMsg>(64);
        senders.lock().await.insert(idx(&alice_id), tx_a);
        senders.lock().await.insert(idx(&bob_id), tx_b);
        senders.lock().await.insert(idx(&carol_id), tx_c);

        let carol_id_typed = idx(&carol_id);
        let req = FanoutRequest {
            event: Some(carol_join.clone()),
            new_joiner: Some(carol_id_typed.clone()),
        };
        apply_fanout(req, &carol_id_typed, &runtime, &senders).await;

        // Carol receives one HistoryBatch with prior events (Space, Room,
        // Bob invite, Bob join, Alice message, Carol invite). The join event
        // itself is excluded (Carol's client already has its own outbound copy).
        let mut got_history: Option<Vec<Event>> = None;
        while let Ok(msg) = rx_c.try_recv() {
            if let OutboundMsg::HistoryBatch { events } = msg {
                got_history = Some(events);
                break;
            }
        }
        let history = got_history.expect("Carol must receive HistoryBatch");
        // Must NOT contain Carol's own join.
        assert!(
            history.iter().all(|e| e.event_id.as_ref().map(|x| x.as_str()) != Some(carol_join_id.as_str())),
            "history must exclude the join event itself"
        );
        // Must contain Bob's prior join (row 7 analogue).
        let bob_join_present = history.iter().any(|e| {
            matches!(e.event_type, EventType::MembershipJoin) && e.sender.as_str() == bob_id
        });
        assert!(bob_join_present, "carol must see Bob's prior membership.join");
        // Must contain the prior message.text from Alice.
        let prior_msg_present = history
            .iter()
            .any(|e| matches!(e.event_type, EventType::MessageText) && e.sender.as_str() == alice_id);
        assert!(prior_msg_present, "carol must see Alice's prior message");
    }

    #[tokio::test]
    async fn fanout_skips_disconnected_recipients() {
        // Bob is a member but has no sender registered (not connected).
        // The fan-out must not panic and must still deliver to Carol.
        let (rt, space_id, room_id, alice, _bob, carol) = setup_three_member_space();
        let alice_id = pubkey_uri(&alice);
        let carol_id = pubkey_uri(&carol);
        let runtime = Arc::new(Mutex::new(rt));
        let senders: ClientSenders = Arc::new(Mutex::new(HashMap::new()));
        let (tx_c, mut rx_c) = mpsc::channel::<OutboundMsg>(64);
        senders.lock().await.insert(idx(&carol_id), tx_c);

        let space_id_typed = sdx(&space_id);
        let tip = runtime.lock().await.dag_tips(&space_id_typed)[0].clone();
        let msg = sign_event(
            build_message_text_event(&alice, &space_id, &room_id, vec![tip], "hi"),
            &alice,
        );
        let req = FanoutRequest { event: Some(msg.clone()), new_joiner: None };
        let alice_id_typed = idx(&alice_id);
        apply_fanout(req, &alice_id_typed, &runtime, &senders).await;

        match rx_c.recv().await.unwrap() {
            OutboundMsg::Event(ev) => assert_eq!(ev.event_id, msg.event_id),
            _ => panic!("expected Event"),
        }
        let _ = carol;
    }

    #[tokio::test]
    async fn collect_sync_history_returns_only_member_spaces() {
        // Alice is in Space A; Bob is in Space B (not in A). A sync_request
        // from Bob must return only Space B's events.
        let node_key = keypair::generate();
        let mut rt = NodeRuntime::new(node_key);
        let alice = keypair::generate();
        let bob = keypair::generate();
        let alice_id = pubkey_uri(&alice);
        let bob_id = pubkey_uri(&bob);
        rt.register_identity(make_identity_record(&alice_id)).unwrap();
        rt.register_identity(make_identity_record(&bob_id)).unwrap();

        let space_a = sign_event(
            build_space_create_event(&alice, "A", None, 1, HOME),
            &alice,
        );
        let space_a_id: String = event_id_str(&space_a);
        rt.ingest_event(space_a);
        let space_b = sign_event(
            build_space_create_event(&bob, "B", None, 1, HOME),
            &bob,
        );
        let space_b_id: String = event_id_str(&space_b);
        rt.ingest_event(space_b);

        let runtime = Arc::new(Mutex::new(rt));
        // F-7: pagination signature — usize limit + (Vec, Option<cursor>) return.
        let bob_id_typed = idx(&bob_id);
        let (events_for_bob, cursor) =
            collect_sync_history(&runtime, &bob_id_typed, "", 1000).await;
        // Bob is a member only of Space B; sync_history must contain only its
        // space_create (no Space A leak).
        assert!(
            events_for_bob
                .iter()
                .all(|e| event_space_id(e).map(|s| s.as_str().to_string()) == Some(space_b_id.clone())),
            "Bob's sync history must be limited to spaces he is a member of"
        );
        assert!(
            events_for_bob
                .iter()
                .any(|e| e.event_id.as_ref().map(|x| x.as_str()) == Some(space_b_id.as_str())),
            "Bob's sync history must include Space B's create event"
        );
        // Space B has one event; one page exhausts it; continue_from None.
        assert!(cursor.is_none(), "single-event Space fits in one page");
        let _ = space_a_id;
    }

    // ── F-7 pagination tests ──────────────────────────────────────────────

    /// Helper: build a Space with N message events from alice, return (rt, space_id, alice_id, event_ids).
    fn setup_space_with_n_messages(n: usize) -> (NodeRuntime, String, String, Vec<String>) {
        use crate::message::exchange::build_message_text_event;
        let node_key = keypair::generate();
        let mut rt = NodeRuntime::new(node_key);
        let alice = keypair::generate();
        let alice_id = pubkey_uri(&alice);
        rt.register_identity(make_identity_record(&alice_id)).unwrap();
        let space_ev = sign_event(
            build_space_create_event(&alice, "P", None, 1, HOME),
            &alice,
        );
        let space_id: String = event_id_str(&space_ev);
        rt.ingest_event(space_ev.clone());
        let room_ev = sign_event(
            build_room_create_event(&alice, &space_id, "general", None),
            &alice,
        );
        let room_id: String = event_id_str(&room_ev);
        rt.ingest_event(room_ev);
        let mut prev = vec![space_id.clone()];
        let mut ids = Vec::with_capacity(n);
        for i in 0..n {
            let body = format!("m{}", i);
            let ev = sign_event(
                build_message_text_event(
                    &alice,
                    &space_id,
                    &room_id,
                    prev.clone(),
                    &body,
                ),
                &alice,
            );
            let id: String = event_id_str(&ev);
            prev = vec![id.clone()];
            ids.push(id);
            rt.ingest_event(ev);
        }
        (rt, space_id, alice_id, ids)
    }

    #[tokio::test]
    async fn collect_sync_history_limits_page_and_returns_cursor() {
        // 15 events, limit 10 → page has 10 events, continue_from points at the 10th.
        let (rt, _space_id, alice_id, ids) = setup_space_with_n_messages(15);
        let runtime = Arc::new(Mutex::new(rt));
        let alice_id_typed = idx(&alice_id);
        let (page, cursor) = collect_sync_history(&runtime, &alice_id_typed, "", 10).await;
        // 1 space_create + 1 room_create + 10 messages = 12 candidate events,
        // capped at 10. The cursor is the event_id of the 10th delivered.
        assert_eq!(page.len(), 10);
        let last = event_id_str(page.last().unwrap());
        assert_eq!(cursor.as_deref(), Some(last.as_str()));
        let _ = ids;
    }

    #[tokio::test]
    async fn collect_sync_history_resumes_past_cursor_to_completion() {
        // 15 events, limit 10. First page → 10 events + cursor. Second page
        // starting at cursor → remaining 7 events + None.
        let (rt, _space_id, alice_id, _ids) = setup_space_with_n_messages(15);
        let runtime = Arc::new(Mutex::new(rt));
        let alice_id_typed = idx(&alice_id);
        let (page1, cursor1) = collect_sync_history(&runtime, &alice_id_typed, "", 10).await;
        assert_eq!(page1.len(), 10);
        let c1 = cursor1.expect("first page should leave a cursor");

        let (page2, cursor2) = collect_sync_history(&runtime, &alice_id_typed, &c1, 10).await;
        // 17 total candidate events (1 sc + 1 rc + 15 msg), 10 consumed,
        // 7 remaining → all fit in the 10-cap, no more cursor.
        assert_eq!(page2.len(), 7);
        assert!(cursor2.is_none(), "second page completes catch-up");
    }

    #[tokio::test]
    async fn collect_sync_history_empty_when_caught_up() {
        // Caller passes the cursor of the last event in the whole-batch
        // ordering as `since` → no further events. The "last" event here is
        // the tail of the candidate sequence returned by an exhaustive
        // first call; pinning it via that path avoids depending on HashMap
        // iteration order of the topological sort.
        let (rt, _space_id, alice_id, _ids) = setup_space_with_n_messages(3);
        let runtime = Arc::new(Mutex::new(rt));
        let alice_id_typed = idx(&alice_id);
        let (full, _) = collect_sync_history(&runtime, &alice_id_typed, "", 1000).await;
        let tail_id = event_id_str(full.last().unwrap());
        let (page, cursor) = collect_sync_history(&runtime, &alice_id_typed, &tail_id, 1000).await;
        assert!(page.is_empty(), "no events after the tail");
        assert!(cursor.is_none());
    }

    // ── F-6 wire shape roundtrip ──────────────────────────────────────────

    #[test]
    fn sync_complete_wire_roundtrip_with_continue_from() {
        // continue_from: Some(...) serialises as a non-null field.
        use crate::wire::types::TransportMessage;
        let msg = TransportMessage::SyncComplete {
            protocol_version: "0.1".to_string(),
            since: "xgen://hash/sha256:aa".to_string(),
            new_tip: "xgen://hash/sha256:bb".to_string(),
            continue_from: Some("xgen://hash/sha256:bb".to_string()),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"transport.sync_complete\""));
        assert!(json.contains("\"continue_from\":\"xgen://hash/sha256:bb\""));
        // Deserialise back to verify symmetry.
        let parsed: TransportMessage = serde_json::from_str(&json).unwrap();
        match parsed {
            TransportMessage::SyncComplete { continue_from, .. } => {
                assert_eq!(continue_from.as_deref(), Some("xgen://hash/sha256:bb"));
            }
            other => panic!("expected SyncComplete, got {:?}", other),
        }
    }

    #[test]
    fn sync_complete_wire_roundtrip_no_continue_from() {
        // continue_from: None → field is omitted on the wire (D-068 backwards
        // compat — pre-F-7 receivers ignore the optional field gracefully).
        use crate::wire::types::TransportMessage;
        let msg = TransportMessage::SyncComplete {
            protocol_version: "0.1".to_string(),
            since: "xgen://hash/sha256:aa".to_string(),
            new_tip: "xgen://hash/sha256:zz".to_string(),
            continue_from: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(!json.contains("continue_from"), "None should be omitted");
        let parsed: TransportMessage = serde_json::from_str(&json).unwrap();
        match parsed {
            TransportMessage::SyncComplete { continue_from, .. } => {
                assert!(continue_from.is_none());
            }
            _ => panic!("expected SyncComplete"),
        }
    }

    #[test]
    fn sync_request_with_limit_wire_roundtrip() {
        use crate::wire::types::TransportMessage;
        let msg = TransportMessage::SyncRequest {
            protocol_version: "0.1".to_string(),
            since: "".to_string(),
            limit: Some(500),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"limit\":500"));
        let parsed: TransportMessage = serde_json::from_str(&json).unwrap();
        match parsed {
            TransportMessage::SyncRequest { limit, .. } => {
                assert_eq!(limit, Some(500));
            }
            _ => panic!("expected SyncRequest"),
        }
    }

    // ── F-4 dispatch_event Scenario-A tests (Phase 2) ────────────────────
    //
    // Per `tasks/FEDERATION_PROPAGATION_COMPLETION.md` §3.2 DoD, three tests
    // cover the three paths the audit (J-081 §3.3) flagged as asymmetric:
    //   * Path A (messages) regression — must still work post-refactor.
    //   * Path B (membership.join) — out-of-order delivery now HeldPending
    //     (previously: dropped silently).
    //   * Path C (other state events) — out-of-order delivery now HeldPending
    //     (previously: ingested with no validation).

    use crate::node::runtime::{DispatchOutcome, EventOrigin};

    #[tokio::test]
    async fn f4_path_a_message_unknown_predecessor_held_pending_then_drains() {
        // Two messages from alice (room owner — implicit room member). msg_a
        // chains from current tip; msg_b chains from msg_a. Deliver msg_b
        // first: F-4 validation core returns HeldPending; event lands in
        // pending buffer. Delivering msg_a then accepts it and drains
        // msg_b in the same call.
        let (mut rt, space_id, room_id, alice, _bob, _carol) =
            setup_three_member_space();

        let space_id_typed = sdx(&space_id);
        let current_tip = rt.dag_tips(&space_id_typed).first().cloned().unwrap();
        let msg_a = sign_event(
            build_message_text_event(
                &alice,
                &space_id,
                &room_id,
                vec![current_tip],
                "msg_a",
            ),
            &alice,
        );
        let msg_a_id: String = event_id_str(&msg_a);
        let msg_b = sign_event(
            build_message_text_event(
                &alice,
                &space_id,
                &room_id,
                vec![msg_a_id.clone()],
                "msg_b",
            ),
            &alice,
        );
        let msg_b_id: String = event_id_str(&msg_b);

        let out_b = rt.dispatch_event(msg_b, EventOrigin::LocallySubmitted, None);
        assert!(
            matches!(out_b, DispatchOutcome::HeldPending),
            "Path A: msg_b with unknown predecessor must HeldPending, got {:?}",
            out_b
        );
        assert!(
            rt.pending.get(&space_id_typed).map(|b| b.len()).unwrap_or(0) > 0,
            "Path A: pending buffer must hold msg_b"
        );

        let out_a = rt.dispatch_event(msg_a, EventOrigin::LocallySubmitted, None);
        assert!(
            matches!(out_a, DispatchOutcome::Accepted { new_joiner: None, .. }),
            "Path A: msg_a must be Accepted, got {:?}",
            out_a
        );
        assert_eq!(
            rt.pending.get(&space_id_typed).map(|b| b.len()).unwrap_or(0),
            0,
            "Path A: pending buffer must drain after predecessor arrival"
        );
        let store = rt.stores.get(&space_id_typed).unwrap();
        assert!(
            store.contains(&msg_b_id),
            "Path A: msg_b must be in the DAG after drain"
        );
    }

    #[tokio::test]
    async fn f4_path_b_join_unknown_predecessor_held_pending_then_drains() {
        // dave is a 4th identity; alice invites dave; dave's join references
        // the invite event. Deliver join first — pre-F-4 this would silently
        // bypass validation and ingest dave into the Space with no membership.invite
        // anchor (audit Scenario-A non-message). Post-F-4, HeldPending; drains
        // when the invite arrives.
        let (mut rt, space_id, _room_id, alice, _bob, _carol) =
            setup_three_member_space();

        let dave = keypair::generate();
        let dave_id = pubkey_uri(&dave);
        rt.register_identity(make_identity_record(&dave_id)).unwrap();

        let space_id_typed = sdx(&space_id);
        let current_tip = rt.dag_tips(&space_id_typed).first().cloned().unwrap();
        let mut invite = build_membership_event(
            &alice,
            &space_id,
            "",
            EventType::MembershipInvite,
            json!({ "target_identity": dave_id, "role": "member" }),
        );
        invite.prev_events = vec![edx(&current_tip)];
        let invite = sign_event(invite, &alice);
        let invite_id: String = event_id_str(&invite);

        let mut dave_join = build_membership_event(
            &dave,
            &space_id,
            "",
            EventType::MembershipJoin,
            json!({}),
        );
        dave_join.prev_events = vec![edx(&invite_id)];
        let dave_join = sign_event(dave_join, &dave);
        let dave_join_id: String = event_id_str(&dave_join);

        let out_join = rt.dispatch_event(dave_join, EventOrigin::LocallySubmitted, None);
        assert!(
            matches!(out_join, DispatchOutcome::HeldPending),
            "Path B: join with unknown predecessor must HeldPending (was silent-ingest pre-F-4), got {:?}",
            out_join
        );
        assert!(
            !rt.spaces
                .get(&space_id_typed)
                .map(|s| s.is_member(&dave_id))
                .unwrap_or(false),
            "Path B: dave must NOT be a member yet — join is held"
        );

        let out_invite = rt.dispatch_event(invite, EventOrigin::LocallySubmitted, None);
        assert!(
            matches!(out_invite, DispatchOutcome::Accepted { new_joiner: None, .. }),
            "Path B: invite must be Accepted, got {:?}",
            out_invite
        );
        // Drain re-dispatched the join — dave becomes a member.
        assert!(
            rt.spaces.get(&space_id_typed).unwrap().is_member(&dave_id),
            "Path B: dave must be a Space member after drain"
        );
        assert_eq!(
            rt.pending.get(&space_id_typed).map(|b| b.len()).unwrap_or(0),
            0
        );
        let _ = dave_join_id;
    }

    #[tokio::test]
    async fn f4_path_c_state_unknown_predecessor_held_pending_then_drains() {
        // Path C — non-message non-join state events. Pre-F-4 these were
        // ingested directly with no validation and no HeldPending. Now
        // they go through the unified validation core.
        //
        // Test shape: two membership.invite events from alice, chained.
        // Delivering invite_2 first (predecessor unknown) must HeldPending;
        // delivering invite_1 then drains invite_2.
        let (mut rt, space_id, _room_id, alice, _bob, _carol) =
            setup_three_member_space();

        // Two non-existent target identities — invite validates the sender
        // not the target, so fictional target_identity is fine.
        let target_1 = "xgen://pubkey/ed25519:DAVETARGETXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX";
        let target_2 = "xgen://pubkey/ed25519:EVETARGETXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX";

        let space_id_typed = sdx(&space_id);
        let current_tip = rt.dag_tips(&space_id_typed).first().cloned().unwrap();
        let mut invite_1 = build_membership_event(
            &alice,
            &space_id,
            "",
            EventType::MembershipInvite,
            json!({ "target_identity": target_1, "role": "member" }),
        );
        invite_1.prev_events = vec![edx(&current_tip)];
        let invite_1 = sign_event(invite_1, &alice);
        let invite_1_id: String = event_id_str(&invite_1);

        let mut invite_2 = build_membership_event(
            &alice,
            &space_id,
            "",
            EventType::MembershipInvite,
            json!({ "target_identity": target_2, "role": "member" }),
        );
        invite_2.prev_events = vec![edx(&invite_1_id)];
        let invite_2 = sign_event(invite_2, &alice);
        let invite_2_id: String = event_id_str(&invite_2);

        let out_2 = rt.dispatch_event(invite_2, EventOrigin::LocallySubmitted, None);
        assert!(
            matches!(out_2, DispatchOutcome::HeldPending),
            "Path C: state event with unknown predecessor must HeldPending (was silent-ingest pre-F-4), got {:?}",
            out_2
        );

        let out_1 = rt.dispatch_event(invite_1, EventOrigin::LocallySubmitted, None);
        assert!(
            matches!(out_1, DispatchOutcome::Accepted { new_joiner: None, .. }),
            "Path C: invite_1 must be Accepted, got {:?}",
            out_1
        );
        // Drain processed invite_2 — both invites in the DAG now.
        let store = rt.stores.get(&space_id_typed).unwrap();
        assert!(
            store.contains(&invite_2_id),
            "Path C: invite_2 must be in the DAG after drain"
        );
        assert_eq!(
            rt.pending.get(&space_id_typed).map(|b| b.len()).unwrap_or(0),
            0
        );
    }

    #[tokio::test]
    async fn f4_rejects_bad_signature_on_membership_join() {
        // Pre-F-4: Path B skipped signature verification — a forged
        // membership.join would silently land in the DAG. Post-F-4 the
        // validation core catches it.
        let (mut rt, space_id, _room_id, _alice, _bob, _carol) =
            setup_three_member_space();

        let dave = keypair::generate();
        let dave_id = pubkey_uri(&dave);
        rt.register_identity(make_identity_record(&dave_id)).unwrap();

        let space_id_typed = sdx(&space_id);
        let current_tip = rt.dag_tips(&space_id_typed).first().cloned().unwrap();
        let mut dave_join = build_membership_event(
            &dave,
            &space_id,
            "",
            EventType::MembershipJoin,
            json!({}),
        );
        dave_join.prev_events = vec![edx(&current_tip)];
        // Sign correctly first, then tamper with the signature so the
        // event_id still matches the canonical hash but verify_event_signature
        // returns false. (Tampering after sign_event also changes the
        // canonical hash; we tamper the signature only.)
        let mut dave_join = sign_event(dave_join, &dave);
        if let Some(sig) = dave_join.signature.as_mut() {
            // Replace last 4 base64url chars to corrupt the signature
            // without changing event_id (event_id is computed before signing).
            let len = sig.len();
            if len > 4 {
                sig.replace_range(len - 4..len, "AAAA");
            }
        }

        let out = rt.dispatch_event(dave_join, EventOrigin::LocallySubmitted, None);
        match out {
            DispatchOutcome::Rejected(reason) => {
                assert!(
                    reason.contains("signature") || reason.contains("step 12"),
                    "Path B forged signature must be rejected at step 12, got: {}",
                    reason
                );
            }
            other => panic!(
                "Path B forged signature must be Rejected (F-4 closes audit §3.2), got {:?}",
                other
            ),
        }
        assert!(
            !rt.spaces.get(&space_id_typed).unwrap().is_member(&dave_id),
            "forged join must NOT make dave a member"
        );
    }

    #[test]
    fn sync_request_without_limit_omits_field() {
        // Pre-F-7 senders construct SyncRequest with limit: None; the wire
        // shape is identical to the pre-F-7 message.
        use crate::wire::types::TransportMessage;
        let msg = TransportMessage::SyncRequest {
            protocol_version: "0.1".to_string(),
            since: "".to_string(),
            limit: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(!json.contains("limit"), "None should be omitted");
        // A pre-F-7 wire form (`{"type":"transport.sync_request","protocol_version":"0.1","since":""}`)
        // deserialises with limit defaulting to None.
        let legacy = r#"{"type":"transport.sync_request","protocol_version":"0.1","since":""}"#;
        let parsed: TransportMessage = serde_json::from_str(legacy).unwrap();
        match parsed {
            TransportMessage::SyncRequest { limit, .. } => assert!(limit.is_none()),
            _ => panic!("expected SyncRequest"),
        }
    }

    // ── D-076 topological-sort wire-order determinism regression locks ────
    //
    // Four unit-level regression locks for the wire-order non-determinism
    // closed by the topological-sort milestone (see `tasks/FEDERATION_TOPOSORT_*`
    // and DECISIONS.md D-076). Tests 1-3 exercise the primitive
    // `topological_sort_events`; Test 4 exercises the end-to-end
    // `compute_federation_delta_for_space` path including HashMap-iteration
    // variance across two `NodeRuntime` instances (sibling-in-shape to the
    // bidirectional milestone's `apply_federation_add_two_vantages_mirror`
    // unit-level lock for D-075).

    /// Build a minimal synthetic Event with the given event_id and prev_events.
    /// Fields irrelevant to `topological_sort_events` (sender, content, etc.)
    /// are filled with placeholder values; the function only reads `event_id`
    /// and `prev_events`.
    fn mk_event(event_id: &str, prev_events: &[&str]) -> Event {
        Event {
            protocol_version: "0.1".to_string(),
            event_type: EventType::MessageText,
            event_id: Some(EventXgid::from_xgid(Xgid::new(event_id.to_string()))),
            sender: idx("xgen://pubkey/ed25519:STUB"),
            room_id: xgen_common::xgid::RoomXgid::from_xgid(Xgid::new(String::new())),
            space_id: sdx("xgen://hash/sha256:STUB-SPACE"),
            prev_events: prev_events.iter().map(|s| edx(s)).collect(),
            timestamp: "2026-05-22T00:00:00.000Z".to_string(),
            content: json!({}),
            meta_atts: None,
            signature: Some("ed25519:STUB:STUB".to_string()),
        }
    }

    fn ids_of(events: &[Event]) -> Vec<&str> {
        events.iter().map(|e| e.event_id.as_ref().unwrap().as_str()).collect()
    }

    /// Generate all permutations of a slice (Heap's algorithm), inclusive of
    /// the identity permutation. Small-n only; we use n=4 (24 perms) and n=5
    /// (120 perms) in tests below.
    fn permutations<T: Clone>(items: &[T]) -> Vec<Vec<T>> {
        let mut out = Vec::new();
        let mut arr: Vec<T> = items.to_vec();
        let n = arr.len();
        fn go<T: Clone>(k: usize, arr: &mut Vec<T>, out: &mut Vec<Vec<T>>) {
            if k == 1 {
                out.push(arr.clone());
                return;
            }
            for i in 0..k {
                go(k - 1, arr, out);
                if k.is_multiple_of(2) {
                    arr.swap(i, k - 1);
                } else {
                    arr.swap(0, k - 1);
                }
            }
        }
        if n == 0 {
            return vec![Vec::new()];
        }
        go(n, &mut arr, &mut out);
        out
    }

    /// Test 1 — D-076 primitive-level Q3.ii contract: every permutation of a
    /// mixed-shape DAG (roots + dependent children) produces a byte-identical
    /// output sequence under `topological_sort_events`.
    #[test]
    fn topological_sort_events_deterministic_across_permutations() {
        // DAG:  a (root)   b (root)   c (root)
        //         \         \
        //          d         e
        // 5 events; 120 permutations.
        let a = mk_event("a", &[]);
        let b = mk_event("b", &[]);
        let c = mk_event("c", &[]);
        let d = mk_event("d", &["a"]);
        let e = mk_event("e", &["b"]);

        let perms = permutations(&[a, b, c, d, e]);
        assert_eq!(perms.len(), 120, "expected 5! permutations");

        let reference: Vec<String> = topological_sort_events(perms[0].clone())
            .into_iter()
            .map(|ev| ev.event_id.unwrap().as_str().to_string())
            .collect();

        for (i, perm) in perms.iter().enumerate() {
            let sorted = topological_sort_events(perm.clone());
            let ids: Vec<String> = sorted
                .into_iter()
                .map(|ev| ev.event_id.unwrap().as_str().to_string())
                .collect();
            assert_eq!(
                ids, reference,
                "permutation {} produced divergent output — D-076 violated",
                i
            );
        }
        // Sanity: causality preserved (a before d, b before e) AND
        // lex order on tied siblings (a, b, c emit before d, e in lex order).
        assert_eq!(reference, vec!["a", "b", "c", "d", "e"]);
    }

    /// Test 2 — D-076 regression lock for the specific surfaced bug shape:
    /// two DAG roots with empty `prev_events` tie at the top of the topo sort;
    /// pre-fix the primitive preserved input order; post-fix the primitive
    /// emits them in lex event_id order regardless of input order.
    #[test]
    fn topological_sort_events_stable_tiebreak_with_empty_prev_events() {
        let a = mk_event("event_A", &[]);
        let b = mk_event("event_B", &[]);

        // Reverse-lex input: [B, A]. Post-fix output: [A, B].
        let out = topological_sort_events(vec![b.clone(), a.clone()]);
        assert_eq!(
            ids_of(&out),
            vec!["event_A", "event_B"],
            "reverse-lex input must canonicalise to lex output (D-076)"
        );

        // Lex input is unchanged.
        let out2 = topological_sort_events(vec![a, b]);
        assert_eq!(ids_of(&out2), vec!["event_A", "event_B"]);
    }

    /// Test 3 — D-076 fixed-point property: input already in canonical
    /// (lex-by-event_id, causality-respected) order passes through
    /// `topological_sort_events` byte-identical. Closes the contract from the
    /// other direction — the sort canonicalises non-canonical input but does
    /// not perturb canonical input.
    #[test]
    fn topological_sort_events_noop_for_canonically_ordered_input() {
        // DAG already in canonical order: roots in lex order, then children
        // in lex order between siblings.
        let canonical = vec![
            mk_event("a", &[]),
            mk_event("b", &[]),
            mk_event("c", &["a"]),
            mk_event("d", &["b"]),
        ];
        let expected_ids = ids_of(&canonical).iter().map(|s| s.to_string()).collect::<Vec<_>>();
        let out = topological_sort_events(canonical);
        let out_ids: Vec<String> = out
            .into_iter()
            .map(|ev| ev.event_id.unwrap().as_str().to_string())
            .collect();
        assert_eq!(out_ids, expected_ids, "canonical input must pass through unchanged");
    }

    /// Test 4 — **D-076 wire-order-determinism witness (load-bearing).** Two
    /// `NodeRuntime` instances with identical Space history (same set of
    /// pre-signed events ingested in the same order) MUST produce
    /// byte-identical federation deltas (modulo signature-bearing fields,
    /// which here are trivially equal since both runtimes consume the same
    /// pre-signed events). Each runtime has its own `HashMap<String, Event>`
    /// EventStore with its own `RandomState`, so `HashMap.values()` iteration
    /// order differs between A and B — the fix's job is to canonicalise the
    /// output regardless. Sibling-in-shape to bidirectional milestone's
    /// `apply_federation_add_two_vantages_mirror`.
    #[tokio::test]
    async fn compute_federation_delta_byte_identical_across_two_senders() {
        // Build a shared event sequence once with shared keypairs. Both
        // runtimes will ingest these exact Event structs.
        let alice = keypair::generate();
        let bob = keypair::generate();
        let alice_id = pubkey_uri(&alice);
        let bob_id = pubkey_uri(&bob);

        let space_ev = sign_event(
            build_space_create_event(&alice, "Test", None, 1, HOME),
            &alice,
        );
        let space_id: String = event_id_str(&space_ev);
        let room_ev = sign_event(
            build_room_create_event(&alice, &space_id, "general", None),
            &alice,
        );
        let invite_ev = sign_event(
            build_membership_event(
                &alice,
                &space_id,
                "",
                EventType::MembershipInvite,
                json!({ "target_identity": bob_id, "role": "member" }),
            ),
            &alice,
        );
        let join_ev = sign_event(
            build_membership_event(&bob, &space_id, "", EventType::MembershipJoin, json!({})),
            &bob,
        );

        // Two runtimes, two independent HashMaps with independent RandomState.
        // Both get the SAME set of events ingested in the SAME logical order.
        let build_runtime = || -> NodeRuntime {
            let node_key = keypair::generate();
            let mut rt = NodeRuntime::new(node_key);
            rt.register_identity(make_identity_record(&alice_id)).unwrap();
            rt.register_identity(make_identity_record(&bob_id)).unwrap();
            rt.ingest_event(space_ev.clone());
            rt.ingest_event(room_ev.clone());
            rt.ingest_event(invite_ev.clone());
            rt.ingest_event(join_ev.clone());
            rt
        };

        let rt_a = Arc::new(Mutex::new(build_runtime()));
        let rt_b = Arc::new(Mutex::new(build_runtime()));

        let space_id_typed = sdx(&space_id);
        let delta_a = compute_federation_delta_for_space(&rt_a, &space_id_typed, None).await;
        let delta_b = compute_federation_delta_for_space(&rt_b, &space_id_typed, None).await;

        let ids_a: Vec<String> = delta_a.iter().map(event_id_str).collect();
        let ids_b: Vec<String> = delta_b.iter().map(event_id_str).collect();

        assert_eq!(
            ids_a, ids_b,
            "two senders with identical Space state MUST produce byte-identical \
             federation-delta event_id sequences (D-076 contract)"
        );
        // Sanity: all four events present.
        assert_eq!(ids_a.len(), 4, "expected four events in the delta");
    }

    // ── Pass 3 Commit 2a per-surface test T6 (runbook §4.7) ──────────────

    // T6 (Surface #4) — sentinel regression that Pass 1's Option<EventXgid>
    // retype still projects cleanly under Pass 3 surrounding retypes. Per
    // design Q4.8: the sort uses event_id.cmp() through Option's Ord using
    // EventXgid's Ord via inner Xgid's Ord (no separate identifier slot
    // post-Pass-1; the retype is inherited).
    #[test]
    fn fanout_topological_sort_event_xgid_slot_pass_1_intact() {
        use xgen_core::space::state::{build_space_create_event, build_room_create_event, sign_event};

        let key = keypair::generate();
        let space_ev = sign_event(
            build_space_create_event(&key, "t6-space", None, 1, "xgen://pubkey/ed25519:home"),
            &key,
        );
        let space_id = space_ev.event_id.clone().expect("space event_id").as_str().to_string();
        let room_ev = sign_event(
            build_room_create_event(&key, &space_id, "general", None),
            &key,
        );

        // D-076 v1.1 causal-order: room_create has space_create as predecessor
        // → space_create sorts first regardless of input order. Pass 1's
        // Option<EventXgid> retype is the load-bearing slot here.
        let sorted = topological_sort_events(vec![room_ev.clone(), space_ev.clone()]);
        assert_eq!(
            sorted[0].event_id, space_ev.event_id,
            "space_create must sort first (causal-order DAG ancestor)"
        );
        assert_eq!(
            sorted[1].event_id, room_ev.event_id,
            "room_create must sort second"
        );

        // EventXgid Ord delegates to inner Xgid Ord (lex on the URI string):
        // two events with byte-identical event_id compare Equal.
        let ev_clone = sorted[0].clone();
        assert_eq!(sorted[0].event_id, ev_clone.event_id);
        assert_eq!(
            sorted[0].event_id.cmp(&ev_clone.event_id),
            std::cmp::Ordering::Equal
        );
    }
}
