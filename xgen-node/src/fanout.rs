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
use std::sync::{Arc, OnceLock};

use tokio::sync::{mpsc, Mutex};
use crate::node::runtime::{topological_sort, NodeRuntime};
use crate::space::state::SpaceState;
use crate::wire::types::{Event, EventType};
use xgen_common::aicontrol::{matches, Filter};
use xgen_common::conn::ConnId;
use xgen_common::xgid::{EventXgid, IdentityXgid, NodeXgid, SpaceXgid, Xgid};

/// Outbound message the fan-out path pushes into a connected client's handler.
///
/// `transport.sync_complete` (F-6, spec 3.3.6) replaces the 500ms quiet-time
/// heuristic with an explicit end-of-batch signal. After a sync_request, the
/// dispatcher sends `HistoryBatch { events }` then `SyncComplete { .. }` in
/// that order; the WebSocket drain arm in `app.rs` translates the latter into
/// a `TransportMessage::SyncComplete` on the wire.
// PG-09 / FC-D1: `EventType::Unknown(String)` grew `EventType` (and thus the
// embedded `Event`) by ~24 bytes, tipping the pre-existing `Event(Event)`-vs-
// rest size gap over clippy's `large_enum_variant` threshold. Boxing `Event`
// here is a fanout hot-path refactor (every construction + match site) and an
// optimization, not a correctness fix — out of scope for the forward-compat
// arc. Allowed deliberately; sibling to the J-095 `result_large_err` precedent.
#[allow(clippy::large_enum_variant)]
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
    /// M12.3-D1 — inject a federated blob fetch request into a peer's live
    /// federation session. The client↔node miss handler (a different task)
    /// pushes this through `FederationPeerSenders[peer]`; the federation loop's
    /// outbound arm sends it on the wire as `TransportMessage::BlobFetchRequest`.
    /// Clone-able (no `oneshot` here — the waiter lives in
    /// [`PendingFederationFetches`], V5) so `OutboundMsg` stays `Clone`.
    BlobFetchRequest {
        blob_ref: String,
        space_id: Option<String>,
    },
}

/// Per-connection outbound channels keyed by authenticated `identity_id`.
///
/// M7-events arc (EV-D2) — the value is a `Vec<(ConnId, Sender)>`: one Identity
/// may hold **multiple** concurrent connections (the primary client WS plus a
/// second same-identity `.events` WS), and each gets its own channel. Register
/// pushes `(conn_id, tx)` and creates the key if absent; remove drops the
/// matching `conn_id` and prunes the key when its Vec empties (so "is this
/// identity connected?" stays honest). Fan-out delivers to *every* connection
/// of each recipient.
///
/// Inner is a `Vec`, not `HashMap<ConnId,_>`: N-per-identity is tiny (1–2),
/// iteration is the hot path, and remove is O(n) on a trivially small n. The
/// registry is connection-kind-agnostic (EV-D2) — `ConnId` carries no kind
/// tag; events-pipe specialness lives at its own consumer.
///
/// Prime invariant: with exactly one connection per identity, a Vec-of-one
/// fans out byte-for-byte identically to the pre-arc single-sender map.
///
/// Pass 3 (Surface #4 Q4.2) — HashMap key retyped to `IdentityXgid`.
/// xgen-node-internal type (channels never escape to xgen-client); Joe-lock
/// Q-E at design walk.
pub type ClientSenders =
    Arc<Mutex<HashMap<IdentityXgid, Vec<(ConnId, mpsc::Sender<OutboundMsg>)>>>>;

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

/// M12.3-D1/D4 (V5/P2) — in-flight federated blob fetches, keyed by **peer**
/// `NodeXgid` (serialize-one-fetch-per-peer, P2). The client↔node miss handler
/// registers a [`FetchSlot`] before injecting `OutboundMsg::BlobFetchRequest`
/// into the peer's session, then awaits `waker`. The federation loop's collect
/// arms append inbound `BlobChunk`s to the single in-flight slot for *its* peer
/// (`BlobChunk` carries no `blob_ref`, so serialize-per-peer makes the stream
/// unambiguous — P2, no wire change) and fire `waker` on `BlobFetchEnd` /
/// blob-band `Error`. Sibling Arc to [`FederationPeerSenders`]; the session-end
/// cleanup clears the peer's slot (fails any waiter → 10003).
pub type PendingFederationFetches = Arc<Mutex<HashMap<NodeXgid, FetchSlot>>>;

/// One in-flight federated blob fetch (V5). `blob_ref` is what was requested
/// (the `BlobFetchEnd` arm verifies the peer's end-marker matches); `buf`
/// accumulates the streamed ciphertext; `waker` delivers the result (or `Err`
/// on miss/timeout/session-end) back to the awaiting client↔node handler.
pub struct FetchSlot {
    pub blob_ref: String,
    pub buf: Vec<u8>,
    pub waker: tokio::sync::oneshot::Sender<Result<Vec<u8>, ()>>,
}

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

/// Resolve the Space ID a given Event addresses.
///
/// M7-events C3 (EV-D4 v1.1) — converged onto the canonical
/// `Event::effective_space_id()` in `xgen-common::wire` so there is one
/// source of truth for create-event Space resolution (the subscription-filter
/// `spaces` arm uses the same helper). Thin alias retained for the existing
/// call sites (Pass 3 Surface #4 Q4.1 retyped this to `Option<SpaceXgid>`).
pub fn event_space_id(event: &Event) -> Option<SpaceXgid> {
    event.effective_space_id()
}

/// The node-observer registry value (EV-D3 / EV-D6): a list of `.events`-pipe
/// subscribers, each a `(ConnId, Filter, Sender)`. Distinct from
/// `ClientSenders` — observers are node-level (operator/AI watching all hosted
/// Spaces' fan-out), keyed by connection, filtered per subscription. The
/// `.events` pipe server (C4) pushes/prunes entries; `apply_fanout` reads them
/// after the member loop; the command-pipe `state` verb reads the count.
pub type NodeObservers = Arc<Mutex<Vec<(ConnId, Filter, mpsc::Sender<OutboundMsg>)>>>;

static NODE_OBSERVERS: OnceLock<NodeObservers> = OnceLock::new();

/// The process-global node-observer registry (EV-D3 + EV-D6).
///
/// **Shape β (J-166 protocol-audit precedent).** A process has one fan-out hub,
/// so one observer registry — held in a global rather than threaded through the
/// ~8 hot async signatures `client_senders` already rides. The `.events` pipe
/// server (C4) and `apply_fanout` and the `state` verb all reach it here; the
/// registry is the EV-D6 single source of truth for `event_subscriptions`.
/// Lazily initialised empty: an uninitialised/empty registry ⇒ no observer
/// sends ⇒ fan-out is byte-for-byte today (the C3 prime invariant).
pub fn node_observers() -> &'static NodeObservers {
    NODE_OBSERVERS.get_or_init(|| Arc::new(Mutex::new(Vec::new())))
}

/// The set of nodes an event "involves," for the EV-D4 v1.1 `nodes` filter
/// dimension — the runtime-sourced part the pure `matches` predicate cannot see.
///
/// Five sources (EV-D4 v1.1 lock; source 4 narrow per the C3 lock; source 5
/// added by M7C C1, J-222):
/// 1. the Space's `home_node` (always);
/// 2. the Space's `federation_nodes` (the peers);
/// 3. `content["node_id"]` when present (`state.federation_add` names the peer);
/// 4. the `sender` for **verified node-signed** events — `node_eject` /
///    `node_unban` (sender == `home_node`, confirmed `space/state.rs`; already
///    covered by (1) but recorded explicitly) and `federation_add` (its
///    authoring node may differ from `home_node` / `federation_nodes` across
///    vantages, so it is added);
/// 5. `content["ordered_nodes"]` when present — `state.node_priority` carries its
///    node refs there (a list of node URIs), NOT in any of (1)–(4). C3 documented
///    this gap and deferred it; M7C C1 closes it by folding `ordered_nodes` in
///    here (presence-based, like source 3) so a `nodes` filter sees node_priority
///    events. This is a `derive_event_nodes` widening only — the pure `matches`
///    predicate (3-param, caller-supplied `event_nodes`, EV-D4 v1.1) is unchanged.
fn derive_event_nodes(event: &Event, space: &SpaceState) -> Vec<NodeXgid> {
    let mut nodes: Vec<NodeXgid> = Vec::with_capacity(space.federation_nodes.len() + 2);
    nodes.push(space.home_node.clone());
    nodes.extend(space.federation_nodes.iter().cloned());
    if let Some(nid) = event.content.get("node_id").and_then(|v| v.as_str()) {
        nodes.push(NodeXgid::from_xgid(Xgid::new(nid.to_string())));
    }
    if matches!(
        event.event_type,
        EventType::MembershipNodeEject
            | EventType::MembershipNodeUnban
            | EventType::StateFederationAdd
    ) {
        nodes.push(NodeXgid::from_xgid(Xgid::new(event.sender.as_str().to_string())));
    }
    // Source 5 (M7C C1): state.node_priority's node refs live in
    // content["ordered_nodes"]. Presence-based, mirroring source 3.
    if let Some(arr) = event.content.get("ordered_nodes").and_then(|v| v.as_array()) {
        for entry in arr {
            if let Some(uri) = entry.as_str() {
                nodes.push(NodeXgid::from_xgid(Xgid::new(uri.to_string())));
            }
        }
    }
    nodes
}

/// Broadcast a `FanoutRequest` to the relevant connected clients, then to any
/// node-level `.events` observers (EV-D3).
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
    let (recipients, history_for_joiner, event_nodes): (
        Vec<IdentityXgid>,
        Option<Vec<Event>>,
        Vec<NodeXgid>,
    ) = {
        let rt = runtime.lock().await;
        let space = match rt.spaces.get(&space_id) {
            Some(s) => s,
            None => return,
        };
        // `C-5` / `D-154` — PRESENT members only. A departed member is retained in
        // `members` now, and a bare `.keys()` would keep delivering every event in
        // this Space to someone who left it.
        let recipients = space
            .members
            .iter()
            .filter(|(_, m)| m.is_present())
            .map(|(id, _)| id.clone())
            .collect::<Vec<_>>();
        // EV-D4 v1.1 — derive the event's node set while the Space is in hand
        // (the `nodes` filter dimension the pure `matches` can't see). Cheap;
        // consumed only by the observer loop below.
        let event_nodes = derive_event_nodes(&event, space);
        let history = if let Some(joiner) = req.new_joiner.as_ref() {
            rt.stores.get(&space_id).map(|store| {
                // SE-D6: trait `range(0)` (all events, append order) replaces
                // the inherent `values()`; sorted below, so order in is fine.
                let all: Vec<Event> = store.range(0).unwrap_or_default();
                // `D-154`④ (E2-2) — the presence-interval filter. Computed
                // BEFORE the delivery sort: the SET comes from core's fold
                // order, the ORDER stays `topological_sort_events`'. For a
                // first-time joiner this is the whole log ⇒ byte-identical
                // payload, which is the property that keeps E-2 from being a
                // regression.
                let permitted = permitted_event_ids(&all, joiner);
                let sorted = topological_sort_events(all);
                sorted
                    .into_iter()
                    .filter(|e| e.event_id != event_id)
                    .filter(|e| e.event_id.as_ref().is_some_and(|id| permitted.contains(id)))
                    .collect()
            })
        } else {
            None
        };
        (recipients, history, event_nodes)
    };

    let senders = client_senders.lock().await;
    let event_id_for_log = event
        .event_id
        .as_ref()
        .map(|e| e.as_str().to_string())
        .unwrap_or_else(|| "(none)".to_string());

    for rid in &recipients {
        if rid == author_id {
            // EV-D2 — author exclusion stays by *identity*, not conn_id: an
            // author's own connections (including a future events-pipe WS) do
            // not see their own posted event echoed.
            continue;
        }
        if let Some(conns) = senders.get(rid) {
            // EV-D2 — deliver to every connection of this recipient. Phase 9
            // G3: stable trace events per (rid, conn_id) on the delivery path.
            // Pairs with Scenario 1's honesty check #2 (fanout_delivered must
            // observe on the destination Node for federated events) and
            // Scenario 2's destination-side absence assertion (no
            // fanout_delivered for E on a non-federated peer's local clients).
            for (conn_id, tx) in conns {
                match tx.try_send(OutboundMsg::Event(event.clone())) {
                    Ok(()) => {
                        tracing::debug!(
                            event = "fanout_delivered",
                            client_id = %rid.as_str(),
                            conn_id = %conn_id,
                            event_id = %event_id_for_log,
                            "local fan-out: event delivered to client"
                        );
                    }
                    Err(_) => {
                        tracing::warn!(
                            event = "fanout_dropped_channel_full",
                            client_id = %rid.as_str(),
                            conn_id = %conn_id,
                            event_id = %event_id_for_log,
                            "local fan-out: client channel full, event dropped"
                        );
                    }
                }
            }
        }
    }

    if let (Some(joiner_id), Some(history)) = (req.new_joiner.as_ref(), history_for_joiner) {
        if !history.is_empty() {
            if let Some(conns) = senders.get(joiner_id) {
                // EV-D2 — history goes to all of the joiner's connections.
                for (_conn_id, tx) in conns {
                    let _ = tx.try_send(OutboundMsg::HistoryBatch {
                        events: history.clone(),
                    });
                }
            }
        }
    }
    drop(senders);

    // EV-D3 / EV-D6 — node observer fan-out (Shape β). The process-global
    // registry is written by the `.events` pipe server (C4); empty/uninit ⇒
    // this loop is a no-op ⇒ fan-out is byte-for-byte today (prime invariant).
    // Filter-before-send (EV-D4 A): only matching events enter the bounded
    // observer channel, so a narrow subscription cannot flood it. Observers see
    // ALL fanned events regardless of membership — including federation-received
    // and author-originated — since `apply_fanout` is the superset chokepoint
    // (EV-D5). No author exclusion: an observer is an operator/AI surface, not a
    // member, and wants the complete view.
    let observers = node_observers().lock().await;
    for (conn_id, filter, tx) in observers.iter() {
        if matches(filter, &event, &event_nodes) {
            match tx.try_send(OutboundMsg::Event(event.clone())) {
                Ok(()) => {
                    tracing::debug!(
                        event = "observer_delivered",
                        conn_id = %conn_id,
                        event_id = %event_id_for_log,
                        "node observer: event delivered"
                    );
                }
                Err(_) => {
                    tracing::warn!(
                        event = "observer_dropped_channel_full",
                        conn_id = %conn_id,
                        event_id = %event_id_for_log,
                        "node observer: channel full, event dropped"
                    );
                }
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
    // `E2-3` — the SAME whole-batch order before the clause-④ filter, plus the
    // union of what she may receive. Kept for exactly one purpose: resolving a
    // `since` cursor the filter has removed (the fallback below). Nothing else
    // reads them.
    let mut unfiltered_order: Vec<EventXgid> = Vec::new();
    let mut permitted_all: HashSet<EventXgid> = HashSet::new();
    for (space_id, space) in &rt.spaces {
        if !space.is_member(requester_id.as_str()) {
            continue;
        }
        if let Some(store) = rt.stores.get(space_id) {
            // SE-D6: trait `range(0)` replaces inherent `values()` (sorted below).
            let all: Vec<Event> = store.range(0).unwrap_or_default();
            // `D-154`④ (E2-3) — the presence-interval filter, PER SPACE, inside
            // the loop. `is_member` above is the PRESENT-TENSE accessor Leg E-1
            // gated, so a REJOINER passes it — gating it correctly is exactly
            // what makes this door reachable by the person clause ④ bounds
            // (Leg E-2 Phase-0 §3). Applied per Space here rather than after
            // pagination, so `continue_from` is always computed over events she
            // may actually receive — filtering a page after the limit was taken
            // would return short pages and a cursor into the wrong sequence.
            // ⚠️ The `since` lookup below therefore runs over the FILTERED list,
            // which is what makes the fallback there necessary; the reasoning is
            // at that site.
            let permitted = permitted_event_ids(&all, requester_id);
            for e in topological_sort_events(all) {
                // An id-less event cannot come out of a store — `EventStore::
                // insert` refuses `MissingEventId` — so this skip is unreachable
                // and is here to keep the cursor arithmetic below exact rather
                // than to change any behaviour.
                let Some(id) = e.event_id.clone() else { continue };
                unfiltered_order.push(id.clone());
                if permitted.contains(&id) {
                    permitted_all.insert(id);
                    candidate.push(e);
                }
            }
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
            None => {
                // `E2-6`.7 — the cursor is not in her FILTERED list. Before
                // clause ④ every member-Space event was, so a miss meant a
                // genuinely unknown cursor and `(vec![], None)` was truthful.
                // The filter can now remove a cursor that resolves perfectly
                // well, and an empty page with no `continue_from` is
                // byte-identical to *caught up* — `collect_sync_history_
                // empty_when_caught_up` pins exactly that shape — so the client
                // would silently believe it had everything (`D-065`).
                //
                // 🛑 **DEVIATION FROM `E2-3`'s STATED MECHANISM, REPORTED AND
                // NOT ABSORBED (Rule 6).** `E2-3` prescribes filter-then-cursor
                // and says that ordering *avoids* this; measured, that ordering
                // PRODUCES it, and `E2-6`.7 forbids it. Resolving the cursor in
                // the unfiltered order and resuming at the first permitted event
                // after it satisfies both. A cursor unknown in BOTH orders keeps
                // today's refusal, so an unknown cursor is still refused and only
                // a WITHHELD one is rescued.
                //
                // `candidate` is the permitted subsequence of `unfiltered_order`
                // in the same order, so the count of permitted ids at or before
                // the cursor IS the index of the first permitted event after it.
                match unfiltered_order.iter().position(|id| id.as_str() == since) {
                    Some(k) => unfiltered_order[..=k]
                        .iter()
                        .filter(|id| permitted_all.contains(*id))
                        .count(),
                    None => return (Vec::new(), None),
                }
            }
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

/// M8.5-B (INV-D1, CP-3) — the **structural** event types served by the scoped
/// invite-bootstrap fetch. The set is the Space/Room creates plus the membership
/// chain (admission structure: invite/join/leave/kick/ban/node_eject/node_unban)
/// — everything an invitee needs to discover the invite naming it and to know
/// its standing at admission (a banned identity must not bootstrap as if clean).
///
/// **Deliberate exclusions (the INV-D1 privacy line):** all message/thread/MLS
/// content; pacing/temperature/federation/AI/migration state; **and
/// `MembershipMute`** — mute is an AI-pacing/moderation signal, not admission
/// structure, and serving it would leak moderation posture to a not-yet-member
/// (it syncs normally once they are a full member).
///
/// This set is a *discovery* payload, not an authoritative DAG: the invitee
/// reads the invite's `event_id` to **name** it (INV-D2); the Node re-validates
/// the subsequent `membership.join` server-side against its full DAG, so there is
/// no ancestry-completeness obligation on what is served here.
fn is_structural_bootstrap_type(event_type: &EventType) -> bool {
    matches!(
        event_type,
        EventType::StateSpaceCreate
            | EventType::StateDmSpaceCreate
            | EventType::StateRoomCreate
            | EventType::MembershipInvite
            | EventType::MembershipJoin
            | EventType::MembershipLeave
            | EventType::MembershipKick
            | EventType::MembershipBan
            | EventType::MembershipNodeEject
            | EventType::MembershipNodeUnban
    )
}

/// `D-154`④ (Joe, 2026-08-22) — **the presence-interval filter**, as CLARIFIED
/// 2026-08-23 (J-769): *structure is not content*.
///
/// Returns the set of `event_id`s `identity` may receive from this Space's log:
/// **everything up to each of her departures, everything from each rejoin
/// forward, and — for the periods she was absent — the membership structure but
/// not the conversation.**
///
/// **Ordered by [`topological_sort`] (xgen-core), NOT by
/// [`topological_sort_events`]** — Leg E-2 Phase-0 §4b, option (B). That is the
/// same function `resolution::derive` folds the state with, so the boundary this
/// walk computes agrees with `SpaceMember::left_at` **by construction rather
/// than by coincidence**. The two sorts are different linear extensions of one
/// DAG and are measurably NOT order-equal on a DAG with concurrency (see
/// `two_sorts_preserve_the_event_set_and_causal_order`) — which is precisely why
/// the SET is decided here while the ORDER stays the delivery sort's at the door.
///
/// **The walk, one `present` flag:**
///
/// - opens `present` at **index 0**, not at her first join. ④ says *everything
///   up to her departure*, and a first-time joiner receives the whole store
///   today ⇒ her payload must stay byte-identical.
/// - **closes** on a departure naming her. 🛑 **Both shapes, or the walk is
///   wrong** (`N-197`): a `membership.leave` names the departed as `sender`,
///   while `kick` / `ban` / `node_eject` name the *actor* as sender and her in
///   `content["target_identity"]`. A walk reading only `sender` yields a
///   plausible, non-empty, WRONG slice — and every `leave`-based test passes.
/// - **reopens** on a `membership.join` she sent.
/// - while absent, [`is_structural_bootstrap_type`] ⇒ **admit**. That predicate
///   is already the *"membership structure is the least-protected class"* set
///   the clarification was ruled on (`D-154`④ ②), so the admitted-while-absent
///   set and the invite-bootstrap set cannot drift apart.
/// - her own departure and rejoin events are structural ⇒ admitted either way;
///   no off-by-one guard is needed and none is written.
///
/// N leave/rejoin cycles ⇒ N+1 intervals. A single-boundary implementation is
/// wrong by construction (Leg E Phase-0 §5d(D) was refused for that reason).
///
/// 🛑 **`room_id` MUST be empty on `leave` and `join`, and on `kick` — this
/// mirrors the appliers and is NOT decoration.** `apply_leave` and `apply_kick`
/// each return early on a room-level event without touching `left_at`
/// (`state.rs`), so a walk that closed on a room-level leave would open a gap
/// the fold never opened — the exact walk-disagrees-with-`left_at` failure
/// option (B) was chosen to eliminate. `apply_ban` / `apply_node_eject` have no
/// room-level branch, so they close regardless of `room_id`, and this matches.
///
/// 📌 **The set is built by SUBTRACTION, not by collecting the sort's output.**
/// [`topological_sort`] is lossy where [`topological_sort_events`] explicitly
/// *"guarantees the function preserves all input"*: it drops events with no
/// `event_id` (`filter_map`) and never emits a cycle member (Kahn). Both are
/// unreachable through a store today — `EventStore::insert` refuses a
/// `MissingEventId`, and a hash-linked cycle is not constructible — but
/// collecting positively from the sort would make the no-op-for-a-first-time-
/// joiner property depend on that infeasibility argument instead of on the code.
/// Subtracting keeps it true by construction, and fails toward *delivered as
/// today* rather than toward *silently withheld from everyone*.
pub(crate) fn permitted_event_ids(all: &[Event], identity: &IdentityXgid) -> HashSet<EventXgid> {
    // Start from the whole log; remove only what falls inside an absence.
    let mut permitted: HashSet<EventXgid> =
        all.iter().filter_map(|e| e.event_id.clone()).collect();

    let mut present = true;
    for e in topological_sort(all.to_vec()) {
        // Classify under the state in effect when the event arrives, then move
        // the flag. (For the boundary events themselves the order is immaterial:
        // they are structural and are admitted either way.)
        if !present && !is_structural_bootstrap_type(&e.event_type) {
            if let Some(id) = &e.event_id {
                permitted.remove(id);
            }
        }

        let space_level = e.room_id.as_str().is_empty();
        let targets_her = e.content["target_identity"].as_str() == Some(identity.as_str());
        let closes = match e.event_type {
            // `apply_leave` / `apply_kick`: room-level returns early, no `left_at`.
            EventType::MembershipLeave => space_level && e.sender == *identity,
            EventType::MembershipKick => space_level && targets_her,
            // `apply_ban` / `apply_node_eject`: no room-level branch.
            EventType::MembershipBan | EventType::MembershipNodeEject => targets_her,
            _ => false,
        };
        let reopens =
            matches!(e.event_type, EventType::MembershipJoin) && space_level && e.sender == *identity;

        // Closing an already-closed interval is a no-op, which is the boolean
        // form of `mark_departed`'s first-wins rule (`state.rs`): a second
        // departure never moves the boundary.
        if closes {
            present = false;
        } else if reopens {
            present = true;
        }
    }

    permitted
}

/// `M-SPACE-ADMISSION` Leg G-3 — **does this event NAME the requester?**
///
/// The per-type field test behind the door's served set (§3, option ②). It is
/// deliberately **not** a `sender == her || target == her` union, and that is
/// the whole point: a `kick` **she issued** while she was a member carries her
/// as `sender` and a THIRD PARTY as `target_identity`, so a union would serve
/// her someone else's removal — the exact disclosure ② withholds.
///
/// So the field is read per type, matching the appliers (`state.rs`) and
/// [`permitted_event_ids`]'s own classification:
///
/// - `join` / `leave` — the subject SIGNS them ⇒ read `sender`.
/// - `invite` / `kick` / `ban` / `node_eject` / `node_unban` — someone else acts
///   ON the subject and is the `sender` ⇒ read `content["target_identity"]`.
///   ✅ All five verified at their appliers to use that one field name.
///
/// 📌 **No `room_id` condition, and its absence is a decision.**
/// [`permitted_event_ids`] tests `space_level` because it computes a presence
/// BOUNDARY that must agree with `left_at`, and the appliers return early on a
/// room-level event. This is a DISCLOSURE test, not a boundary: a room-level
/// event naming her still tells her only about herself.
fn bootstrap_event_names_requester(event: &Event, requester_id: &IdentityXgid) -> bool {
    match event.event_type {
        EventType::MembershipJoin | EventType::MembershipLeave => event.sender == *requester_id,
        EventType::MembershipInvite
        | EventType::MembershipKick
        | EventType::MembershipBan
        | EventType::MembershipNodeEject
        | EventType::MembershipNodeUnban => {
            event.content["target_identity"].as_str() == Some(requester_id.as_str())
        }
        _ => false,
    }
}

/// M8.5-B (INV-D1/INV-D2, CP-2/CP-3) — the scoped structural invite-bootstrap
/// fetch. Serves someone **entitled to enter** (not yet a member) the structural
/// event set of `space_id` so they can chain a causally-correct `membership.join`.
///
/// **Two entitlement routes** (`M-SPACE-ADMISSION` Leg G-3, Joe 2026-08-26):
///
/// 1. **A pending invitee** holding an **unexpired** `pending_invite` — M8.5-B's
///    original case. It reads the invite naming it and chains off that id.
/// 2. **A retained departed member who is not banned** — `D-154`①'s rejoiner.
///    She needs no invite (Leg G-1 admits her at the join gate without one), so
///    she has none to read; she anchors on her own last membership event.
///
/// 🛑 **THE BAN TERM IS LOAD-BEARING AND IS NOT DUPLICATION.** Before Leg G-3
/// the single line `pending_invites.get(..).ok_or(REFUSED)?` was doing TWO jobs:
/// proving entitlement AND, as a side effect, excluding the banned — a banned
/// identity holds no pending invite (`apply_ban` / `apply_node_eject` both
/// `pending_invites.remove`), so it was refused for the wrong reason. Widening
/// that line replaces only the first job. **`left_at.is_some()` is true for a
/// banned and for a node-ejected identity too**, so without the explicit
/// `space.banned` test route 2 would hand the Space's membership chain to
/// someone it has permanently excluded.
///
/// ⚠️ **This is the INVERSE of Leg G-1's gate, and the difference is measured,
/// not stylistic.** There, a ban clause would have been a second source of truth
/// because the dispatch-level pre-check (`runtime.rs`, MP-F6) runs upstream in
/// the same function. **Here nothing runs upstream at all**: neither this
/// function nor the node's `InviteBootstrapRequest` arm (`app.rs`, whose own
/// comment says authorization lives here) reads `banned`, and the dispatch-level
/// pre-check guards event SUBMISSION, not transport requests. `banned` had zero
/// occurrences in this file before this leg.
///
/// **The served set is route-dependent (§3, ruled ②).** An invitee gets the
/// whole membership chain, as it always has. A former member standing outside
/// gets the creates plus **only the membership events naming her** — she learns
/// nothing new about anyone else until she is actually back in, at which point
/// `D-154`④ governs as it already does. 🛑 That means **the payload now depends
/// on who knocks**, which is a real complexity cost, named rather than traded
/// away, and documented at `wire/types.rs` and ch3 §3.3.11 alongside this.
///
/// 🔑 The `D-154`④ presence-interval filter ([`permitted_event_ids`]) is NOT
/// applied here and would be a **no-op** if it were: its job is to withhold
/// CONTENT during an absence, and this path serves no content by construction.
///
/// Sibling to `collect_sync_history`, which stays **member-only** and untouched
/// (its `is_member` gate is present-tense, so a former member is refused there
/// and stays refused). The validity read-gate (INV-D6) still lives here, still
/// inside the invite route — an absent or expired invite is refused **at the
/// request**, not served-then-rejected-later. Refusal carries transport wire
/// `1011 invite_bootstrap_refused` (a `transport.*` refusal belongs in the
/// 1xxx transport band; 3044 is the join-acceptance gate, a separate band).
///
/// Returns the structural events in topological order on success, or
/// `Err((1011, "invite_bootstrap_refused"))` when the Space is unknown, or the
/// requester neither holds an unexpired pending invite nor is a retained
/// departed member who is not banned.
pub async fn collect_invite_bootstrap(
    runtime: &Arc<Mutex<NodeRuntime>>,
    requester_id: &IdentityXgid,
    space_id: &str,
) -> Result<Vec<Event>, (u32, &'static str)> {
    const REFUSED: (u32, &str) = (1011, "invite_bootstrap_refused");
    let rt = runtime.lock().await;
    let space = rt.spaces.get(space_id).ok_or(REFUSED)?;
    // Leg G-3 route 2 — a RETAINED DEPARTED member who is not banned.
    //
    // `!m.is_present()` is Leg G-1's and Leg G-2's term, RE-READ not re-spelled:
    // `SpaceMember::is_present()` is `D-067`'s one fact in one place. Third site,
    // same spelling. `space.banned` is read DIRECTLY — see the ban paragraph
    // above; there is no upstream check on this path to defer to.
    //
    // 🛑 One `banned` test covers BOTH permanent exclusions: `apply_ban` and
    // `apply_node_eject` each `banned.insert`, while `apply_kick` does NOT — so
    // a kicked member stays eligible to fetch her anchor, which is exactly
    // `D-154`②③ (a kick is remembered; she may return).
    let is_former_member = space
        .members
        .get(requester_id)
        .is_some_and(|m| !m.is_present())
        && !space.banned.contains(requester_id);

    // Authorization: an unexpired pending invite, OR route 2.
    match space.pending_invites.get(requester_id) {
        Some(pending) => {
            // Read-gate (INV-D6): an expired invite is a dead read capability.
            // Mirrors the join-acceptance gate's fail-closed-for-non-DM rule
            // (C2): on a regular Space an absent/unparseable `valid_until` is
            // malformed/legacy → refuse; DM Spaces are exempt by design
            // (`dm_constraints_active`) — DMs don't use this path, but the
            // exemption stays consistent with the join gate.
            //
            // ⚠️ NAMED, AND NOW RULED — the invite route SHADOWS route 2. A
            // former member who was re-invited, whose invite then expired, is
            // refused here even though route 2 would admit her with no invite
            // at all. Reachable: neither `apply_leave` nor `apply_kick` clears
            // `pending_invites`, and `apply_invite` checks the actor's role and
            // `banned` but nothing about the target's membership, so a departed
            // member CAN be re-invited.
            //
            // 🔒 JOE RULED THE SHADOW CORRECT (2026-08-26, J-777): she is
            // refused, and must obtain a fresh invite or stay out.
            //
            // The argument that lost is kept, rewritten not removed, because the
            // next reader hits the same fork and this is the only place it is
            // visible: *a dead capability she is not relying on defeats the
            // route that exists precisely so she needs none, and whether someone
            // happened to re-invite her is arbitrary.* Two facts answer it:
            //
            //  1. `D-154`① — THE INVITE IS THE CARRIER OF THE ROLE, not merely
            //     permission to enter. `apply_join` takes `(role, invited_by)`
            //     from `pending_invites.remove`, defaulting to `Role::Member`.
            //     Under a true OR, someone re-invited as MODERATOR whose invite
            //     expired would fall through and be admitted a plain member —
            //     the elevated grant silently dropped, invisible to her and to
            //     whoever issued it. The refusal forces the inviter to re-affirm
            //     the role. So the trigger is not arbitrary: it is a role grant,
            //     and expiry has to mean something for the grant as well as for
            //     the entry.
            //  2. DECISIVE — THE OR WOULD OPEN A DOOR ONTO A LOCKED GATE. The
            //     `3044 invite_expired` gate at event submission
            //     is `if let Some(pi) = space.pending_invites.get(&event.sender)`
            //     — `runtime.rs`, the ONLY `pending_invites.get(&event.sender)`
            //     site in the file (`:1819` at `951b758`; it read `:1804` at
            //     `fa0f8ad` and Leg G-4's own comment edit to that file moved it,
            //     which is why the SYMBOL is the anchor and the line is only a
            //     convenience — `D-152` clause 1)
            //     — NOT conditioned on the rejoin flag. She would fetch her
            //     anchor here, build a correct join, and be refused `3044` at
            //     submission anyway. Refusing at the door is the CONSISTENT
            //     behaviour; this was never a G-3 quirk.
            //
            // 🔓 What survives is a STRING, not a predicate: this refusal is
            // `1011` and is indistinguishable from a stranger's, so nothing
            // tells her to ask for a fresh invite. Filed for `G-5`.
            match pending.valid_until.as_deref() {
                Some(vu_str) => {
                    let past = chrono::DateTime::parse_from_rfc3339(vu_str)
                        .map(|vu| chrono::Utc::now() > vu.with_timezone(&chrono::Utc))
                        .unwrap_or(true); // unparseable ⇒ fail-closed
                    if past {
                        return Err(REFUSED);
                    }
                }
                None if !space.dm_constraints_active => return Err(REFUSED),
                None => {}
            }
        }
        // No invite to expire, so no expiry to check — and no substitute
        // deadline is invented for her.
        None if is_former_member => {}
        None => return Err(REFUSED),
    }
    // Serve the structural-only set (CP-3), in topological order.
    //
    // §3 ruled ②: a former member gets the creates plus only the membership
    // events NAMING her; an INVITEE's payload is unchanged (`V-6`) — ① was
    // *"the same set an invitee gets"*, so ② narrows exactly one side. The
    // creates are unconditional: without them the batch is unparseable.
    //
    // 📌 `is_former_member` carries `&& !banned`, but at THIS point the two
    // coincide: a banned former member was already refused above, so the flag
    // reads here as plain *she is a departed member*. Someone entitled by BOTH
    // routes (departed, and re-invited with a live invite) is narrowed — she is
    // still a person outside the room, and the invite naming her survives the
    // filter, so the M8.5-B chain (INV-D2/D3) still works for her.
    //
    // 🔒 `is_structural_bootstrap_type`'s TYPE SET is untouched. Under ② the
    // narrowing is a filter on INSTANCES, not on which types are structural —
    // which is what keeps this set and `permitted_event_ids`'s
    // admitted-while-absent set from drifting apart (`D-154`④ ②).
    let events: Vec<Event> = match rt.stores.get(space_id) {
        Some(store) => store
            .range(0)
            .unwrap_or_default()
            .into_iter()
            .filter(|e| is_structural_bootstrap_type(&e.event_type))
            .filter(|e| {
                !is_former_member
                    || matches!(
                        e.event_type,
                        EventType::StateSpaceCreate
                            | EventType::StateDmSpaceCreate
                            | EventType::StateRoomCreate
                    )
                    || bootstrap_event_names_requester(e, requester_id)
            })
            .collect(),
        None => Vec::new(),
    };
    drop(rt);
    Ok(topological_sort_events(events))
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
    // SE-D6: trait `range(0)` replaces inherent `values()`; explicitly sorted below.
    let mut all: Vec<Event> = store.range(0).unwrap_or_default();
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
            build_space_create_event(&alice, "Test", None, 1, HOME, None, false),
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

        // Bob is invited and joins. M8 C2: chain prev_events causally
        // (invite → join, each off the running tip) so they are NOT treated as
        // concurrent same-key events by the resolving apply path. Real clients
        // always set prev_events; raw ingest_event skips validation so the
        // fixture supplies the linkage. Without it, invite(bob) and join(bob)
        // share a membership key with no causal ordering → the SR-D1 gate
        // resolves them and drops the join → bob never becomes a member.
        let mut invite = build_membership_event(
            &alice,
            &space_id,
            "",
            EventType::MembershipInvite,
            json!({ "target_identity": bob_id, "role": "member" }),
        );
        invite.prev_events = vec![edx(&room_id)];
        let invite = sign_event(invite, &alice);
        let invite_id = event_id_str(&invite);
        rt.ingest_event(invite);
        let mut bob_join =
            build_membership_event(&bob, &space_id, "", EventType::MembershipJoin, json!({}));
        bob_join.prev_events = vec![edx(&invite_id)];
        let bob_join = sign_event(bob_join, &bob);
        let bob_join_id = event_id_str(&bob_join);
        rt.ingest_event(bob_join);

        // Carol is invited and joins, chained off bob's join.
        let mut invite_c = build_membership_event(
            &alice,
            &space_id,
            "",
            EventType::MembershipInvite,
            json!({ "target_identity": carol_id, "role": "member" }),
        );
        invite_c.prev_events = vec![edx(&bob_join_id)];
        let invite_c = sign_event(invite_c, &alice);
        let invite_c_id = event_id_str(&invite_c);
        rt.ingest_event(invite_c);
        let mut carol_join =
            build_membership_event(&carol, &space_id, "", EventType::MembershipJoin, json!({}));
        carol_join.prev_events = vec![edx(&invite_c_id)];
        let carol_join = sign_event(carol_join, &carol);
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
            senders_clone
                .lock()
                .await
                .insert(id, vec![(ConnId::mint(), tx)]);
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
        senders.lock().await.insert(idx(&alice_id), vec![(ConnId::mint(), tx_a)]);
        senders.lock().await.insert(idx(&bob_id), vec![(ConnId::mint(), tx_b)]);
        senders.lock().await.insert(idx(&carol_id), vec![(ConnId::mint(), tx_c)]);

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

    /// `C-5` / `D-154` — a DEPARTED member is RETAINED in `SpaceState::members`
    /// and must drop out of the fan-out recipient list. Without the presence
    /// filter at the recipient site she keeps receiving every event in a Space she
    /// left, which is the privacy half of this leg.
    #[tokio::test]
    async fn fanout_excludes_a_departed_member() {
        let (mut rt, space_id, room_id, alice, bob, carol) = setup_three_member_space();
        let alice_id = pubkey_uri(&alice);
        let bob_id = pubkey_uri(&bob);
        let carol_id = pubkey_uri(&carol);
        let sx = sdx(&space_id);

        // Carol leaves. Chained off the running tip: `ingest_event` skips
        // validation, but an unchained non-root membership event is resolved as
        // concurrent and dropped — the fixture above says so at length.
        let tip = rt.dag_tips(&sx)[0].clone();
        let mut leave =
            build_membership_event(&carol, &space_id, "", EventType::MembershipLeave, json!({}));
        leave.prev_events = vec![edx(&tip)];
        rt.ingest_event(sign_event(leave, &carol));

        // Precondition, two-sided: retained in the map, absent from the answer.
        assert!(
            rt.spaces[&sx].members.contains_key(carol_id.as_str()),
            "D-154 - carol's record is RETAINED, which is what makes this test necessary"
        );
        assert!(!rt.spaces[&sx].is_member(carol_id.as_str()), "and she is not present");

        let runtime = Arc::new(Mutex::new(rt));
        let senders: ClientSenders = Arc::new(Mutex::new(HashMap::new()));
        let (tx_a, mut rx_a) = mpsc::channel::<OutboundMsg>(64);
        let (tx_b, mut rx_b) = mpsc::channel::<OutboundMsg>(64);
        let (tx_c, mut rx_c) = mpsc::channel::<OutboundMsg>(64);
        senders.lock().await.insert(idx(&alice_id), vec![(ConnId::mint(), tx_a)]);
        senders.lock().await.insert(idx(&bob_id), vec![(ConnId::mint(), tx_b)]);
        senders.lock().await.insert(idx(&carol_id), vec![(ConnId::mint(), tx_c)]);

        let tip2 = runtime.lock().await.dag_tips(&sx)[0].clone();
        let msg = sign_event(
            build_message_text_event(&alice, &space_id, &room_id, vec![tip2], "hello"),
            &alice,
        );
        let req = FanoutRequest { event: Some(msg.clone()), new_joiner: None };
        apply_fanout(req, &idx(&alice_id), &runtime, &senders).await;

        // Bob is the POSITIVE CONTROL: he is still present and DOES receive, so an
        // empty channel for carol is a real exclusion and not a dead fan-out.
        match rx_b.recv().await.expect("bob (present) receives") {
            OutboundMsg::Event(ev) => assert_eq!(ev.event_id, msg.event_id),
            _ => panic!("expected Event"),
        }
        assert!(rx_c.try_recv().is_err(), "C-5: the DEPARTED member receives nothing");
        assert!(rx_a.try_recv().is_err(), "and the author is still excluded");
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
            sign_event(build_space_create_event(&alice, "Test", None, 1, HOME, None, false), &alice);
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
        senders.lock().await.insert(idx(&alice_id), vec![(ConnId::mint(), tx_a)]);
        senders.lock().await.insert(idx(&bob_id), vec![(ConnId::mint(), tx_b)]);
        senders.lock().await.insert(idx(&carol_id), vec![(ConnId::mint(), tx_c)]);

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
        senders.lock().await.insert(idx(&carol_id), vec![(ConnId::mint(), tx_c)]);

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

    // ── M7-events arc — EV-D2 multi-connection fan-out regression locks ────

    /// Prime invariant (C1): with exactly one connection per identity, the
    /// retyped `Vec<(ConnId, Sender)>` registry fans out byte-for-byte
    /// identically to the pre-arc single-sender map — each non-author member
    /// receives exactly one copy of the event, the author receives none.
    #[tokio::test]
    async fn single_connection_fanout_unchanged() {
        let (rt, space_id, room_id, alice, _bob, carol) = setup_three_member_space();
        let alice_id = pubkey_uri(&alice);
        let carol_id = pubkey_uri(&carol);
        let runtime = Arc::new(Mutex::new(rt));
        let senders: ClientSenders = Arc::new(Mutex::new(HashMap::new()));

        // Vec-of-one per identity — exactly today's shape.
        let (tx_a, mut rx_a) = mpsc::channel::<OutboundMsg>(64);
        let (tx_c, mut rx_c) = mpsc::channel::<OutboundMsg>(64);
        senders.lock().await.insert(idx(&alice_id), vec![(ConnId::mint(), tx_a)]);
        senders.lock().await.insert(idx(&carol_id), vec![(ConnId::mint(), tx_c)]);

        let space_id_typed = sdx(&space_id);
        let tip = runtime.lock().await.dag_tips(&space_id_typed)[0].clone();
        let msg = sign_event(
            build_message_text_event(&alice, &space_id, &room_id, vec![tip], "solo"),
            &alice,
        );
        let req = FanoutRequest { event: Some(msg.clone()), new_joiner: None };
        let alice_id_typed = idx(&alice_id);
        apply_fanout(req, &alice_id_typed, &runtime, &senders).await;

        // Carol receives exactly one copy.
        match rx_c.recv().await.expect("carol receives") {
            OutboundMsg::Event(ev) => assert_eq!(ev.event_id, msg.event_id),
            _ => panic!("expected Event"),
        }
        assert!(rx_c.try_recv().is_err(), "exactly one delivery to the single connection");
        // Author excluded by identity.
        assert!(rx_a.try_recv().is_err(), "author must not receive its own event");
        let _ = carol;
    }

    /// New capability (EV-D2): an Identity holding two concurrent connections
    /// (the primary client WS + a future same-identity `.events` WS) receives
    /// the fanned event on **both** channels. This is the whole point of the
    /// retype — a second same-identity connection no longer clobbers the first.
    #[tokio::test]
    async fn two_connections_same_identity_both_receive() {
        let (rt, space_id, room_id, alice, bob, _carol) = setup_three_member_space();
        let alice_id = pubkey_uri(&alice);
        let bob_id = pubkey_uri(&bob);
        let runtime = Arc::new(Mutex::new(rt));
        let senders: ClientSenders = Arc::new(Mutex::new(HashMap::new()));

        // Bob holds two connections under one identity key.
        let conn1 = ConnId::mint();
        let conn2 = ConnId::mint();
        assert_ne!(conn1, conn2);
        let (tx_b1, mut rx_b1) = mpsc::channel::<OutboundMsg>(64);
        let (tx_b2, mut rx_b2) = mpsc::channel::<OutboundMsg>(64);
        senders
            .lock()
            .await
            .insert(idx(&bob_id), vec![(conn1, tx_b1), (conn2, tx_b2)]);

        let space_id_typed = sdx(&space_id);
        let tip = runtime.lock().await.dag_tips(&space_id_typed)[0].clone();
        let msg = sign_event(
            build_message_text_event(&alice, &space_id, &room_id, vec![tip], "multi"),
            &alice,
        );
        let req = FanoutRequest { event: Some(msg.clone()), new_joiner: None };
        let alice_id_typed = idx(&alice_id);
        apply_fanout(req, &alice_id_typed, &runtime, &senders).await;

        // Both of Bob's connections receive the same event.
        match rx_b1.recv().await.expect("bob conn1 receives") {
            OutboundMsg::Event(ev) => assert_eq!(ev.event_id, msg.event_id),
            _ => panic!("expected Event on conn1"),
        }
        match rx_b2.recv().await.expect("bob conn2 receives") {
            OutboundMsg::Event(ev) => assert_eq!(ev.event_id, msg.event_id),
            _ => panic!("expected Event on conn2"),
        }
    }

    /// EV-D2 consequence 1 — author exclusion is by *identity*, not connection:
    /// an author holding multiple live connections receives her own posted
    /// event on **none** of them, while a separate recipient still receives it
    /// on all of its connections. Covered-by-construction (the author key is
    /// simply absent from the recipient set), but C1 is its natural home — the
    /// recipient case is locked by `two_connections_same_identity_both_receive`;
    /// this locks the symmetric author case.
    #[tokio::test]
    async fn author_multi_connection_excluded_across_all() {
        let (rt, space_id, room_id, alice, _bob, carol) = setup_three_member_space();
        let alice_id = pubkey_uri(&alice);
        let carol_id = pubkey_uri(&carol);
        let runtime = Arc::new(Mutex::new(rt));
        let senders: ClientSenders = Arc::new(Mutex::new(HashMap::new()));

        // Author (alice) holds two connections; recipient (carol) holds two.
        let (tx_a1, mut rx_a1) = mpsc::channel::<OutboundMsg>(64);
        let (tx_a2, mut rx_a2) = mpsc::channel::<OutboundMsg>(64);
        senders
            .lock()
            .await
            .insert(idx(&alice_id), vec![(ConnId::mint(), tx_a1), (ConnId::mint(), tx_a2)]);
        let (tx_c1, mut rx_c1) = mpsc::channel::<OutboundMsg>(64);
        let (tx_c2, mut rx_c2) = mpsc::channel::<OutboundMsg>(64);
        senders
            .lock()
            .await
            .insert(idx(&carol_id), vec![(ConnId::mint(), tx_c1), (ConnId::mint(), tx_c2)]);

        let space_id_typed = sdx(&space_id);
        let tip = runtime.lock().await.dag_tips(&space_id_typed)[0].clone();
        let msg = sign_event(
            build_message_text_event(&alice, &space_id, &room_id, vec![tip], "author-multi"),
            &alice,
        );
        let req = FanoutRequest { event: Some(msg.clone()), new_joiner: None };
        let alice_id_typed = idx(&alice_id);
        apply_fanout(req, &alice_id_typed, &runtime, &senders).await;

        // Both recipient connections receive it.
        match rx_c1.recv().await.expect("carol conn1 receives") {
            OutboundMsg::Event(ev) => assert_eq!(ev.event_id, msg.event_id),
            _ => panic!("expected Event on carol conn1"),
        }
        match rx_c2.recv().await.expect("carol conn2 receives") {
            OutboundMsg::Event(ev) => assert_eq!(ev.event_id, msg.event_id),
            _ => panic!("expected Event on carol conn2"),
        }
        // Neither of the author's connections receives her own event.
        assert!(rx_a1.try_recv().is_err(), "author conn1 must not receive own event");
        assert!(rx_a2.try_recv().is_err(), "author conn2 must not receive own event");
    }

    // ── M7-events C3 — node observer fan-out (EV-D3/EV-D4/EV-D6) ───────────

    /// A node `.events` observer receives a fanned event its filter matches and
    /// does NOT receive one it filters out. Uses the process-global registry
    /// (Shape β); the filter is scoped to this test's unique Space so a
    /// concurrent test's fan-out cannot leak into the observer channel.
    /// Serial-grouped with the `state` count assertions on the same global.
    #[tokio::test]
    #[serial_test::serial(node_observers)]
    async fn observer_receives_matching_event_and_not_filtered_out() {
        use xgen_common::aicontrol::parse;

        let (rt, space_id, room_id, alice, _bob, _carol) = setup_three_member_space();
        let alice_id = pubkey_uri(&alice);
        let runtime = Arc::new(Mutex::new(rt));
        let senders: ClientSenders = Arc::new(Mutex::new(HashMap::new()));
        let alice_id_typed = idx(&alice_id);
        let space_id_typed = sdx(&space_id);

        // Observer scoped to THIS Space + message.text only.
        let filter = parse(json!({
            "spaces": [space_id],
            "event_types": ["message.text"],
        }))
        .unwrap();
        let conn = ConnId::mint();
        let (obs_tx, mut obs_rx) = mpsc::channel::<OutboundMsg>(16);
        node_observers().lock().await.push((conn, filter, obs_tx));

        // Matching: a message.text in this Space → the observer receives it.
        let tip = runtime.lock().await.dag_tips(&space_id_typed)[0].clone();
        let msg = sign_event(
            build_message_text_event(&alice, &space_id, &room_id, vec![tip], "watch me"),
            &alice,
        );
        apply_fanout(
            FanoutRequest { event: Some(msg.clone()), new_joiner: None },
            &alice_id_typed,
            &runtime,
            &senders,
        )
        .await;
        match obs_rx.recv().await.expect("observer receives matching event") {
            OutboundMsg::Event(ev) => assert_eq!(ev.event_id, msg.event_id),
            _ => panic!("expected Event"),
        }

        // Non-matching: a state.room_create in this Space (wrong type) → excluded.
        let room2 = sign_event(
            build_room_create_event(&alice, &space_id, "general2", None),
            &alice,
        );
        apply_fanout(
            FanoutRequest { event: Some(room2.clone()), new_joiner: None },
            &alice_id_typed,
            &runtime,
            &senders,
        )
        .await;
        assert!(
            obs_rx.try_recv().is_err(),
            "observer must NOT receive a filtered-out (wrong-type) event"
        );

        // Prune so the registry returns to empty for sibling serial tests.
        node_observers().lock().await.retain(|(c, _, _)| *c != conn);
    }

    /// `derive_event_nodes` honors the EV-D4 v1.1 sources 1–4 (home_node +
    /// federation_nodes + content["node_id"] + sender-if-node-signed). Source 5
    /// (`content["ordered_nodes"]`, M7C C1) is covered separately below.
    #[tokio::test]
    async fn derive_event_nodes_covers_the_four_sources() {
        let (mut rt, space_id, room_id, alice, _bob, _carol) = setup_three_member_space();
        let space_id_typed = sdx(&space_id);
        // Add a federation peer to the Space so source 2 has content.
        let peer = ndx("xgen://pubkey/ed25519:PEER");
        rt.spaces
            .get_mut(&space_id_typed)
            .unwrap()
            .federation_nodes
            .push(peer.clone());
        let space = rt.spaces.get(&space_id_typed).unwrap();
        let home = space.home_node.clone();

        // A plain message → home_node + federation_nodes only (sources 1+2).
        let tip = rt.dag_tips(&space_id_typed)[0].clone();
        let msg = sign_event(
            build_message_text_event(&alice, &space_id, &room_id, vec![tip], "hi"),
            &alice,
        );
        let nodes = derive_event_nodes(&msg, space);
        assert!(nodes.contains(&home), "source 1: home_node");
        assert!(nodes.contains(&peer), "source 2: federation peer");
        // sender (alice, a member identity) is NOT added for a non-node-signed type.
        let alice_as_node = ndx(&pubkey_uri(&alice));
        assert!(!nodes.contains(&alice_as_node), "source 4 must not fire for message.text");
    }

    /// M7C C1 — source 5: `state.node_priority` carries its node refs in
    /// `content["ordered_nodes"]`. `derive_event_nodes` folds them in, and the
    /// (unchanged, 3-param) `matches` honors a `nodes` filter via the resulting
    /// `event_nodes`. Closes the EV-D4 `nodes`-dimension gap C3 documented.
    #[tokio::test]
    async fn derive_event_nodes_includes_ordered_nodes_source_5() {
        use xgen_common::aicontrol::{matches, parse};
        use xgen_common::xgid::RoomXgid;
        let (rt, space_id, _room_id, alice, _bob, _carol) = setup_three_member_space();
        let space = rt.spaces.get(&sdx(&space_id)).unwrap();

        let n1 = "xgen://pubkey/ed25519:PRIOR1";
        let n2 = "xgen://pubkey/ed25519:PRIOR2";
        let ev = Event::new(
            EventType::StateNodePriority,
            idx(&pubkey_uri(&alice)),
            RoomXgid::from_xgid(Xgid::new(String::new())),
            sdx(&space_id),
            vec![],
            "2026-06-01T00:00:00.000Z".to_string(),
            json!({ "ordered_nodes": [n1, n2] }),
        );

        let event_nodes = derive_event_nodes(&ev, space);
        assert!(event_nodes.contains(&ndx(n1)), "source 5: ordered_nodes[0] folded in");
        assert!(event_nodes.contains(&ndx(n2)), "source 5: ordered_nodes[1] folded in");

        // `matches` is unchanged (3-param, caller-supplied event_nodes).
        let f_match = parse(json!({ "nodes": [n1] })).unwrap();
        assert!(matches(&f_match, &ev, &event_nodes), "ordered_nodes membership matches");
        let f_nomatch = parse(json!({ "nodes": ["xgen://pubkey/ed25519:OTHER"] })).unwrap();
        assert!(!matches(&f_nomatch, &ev, &event_nodes), "node not in ordered_nodes → no match");
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
            build_space_create_event(&alice, "A", None, 1, HOME, None, false),
            &alice,
        );
        let space_a_id: String = event_id_str(&space_a);
        rt.ingest_event(space_a);
        let space_b = sign_event(
            build_space_create_event(&bob, "B", None, 1, HOME, None, false),
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

    #[tokio::test]
    async fn collect_sync_history_serves_self_dm_to_the_user() {
        // M11 (D-021) W4: a self-DM (invitee == creator) is reachable by any
        // client authenticated as the user, via member-gated sync — proving
        // M11-D2's "Node-resident, not device-local" reach. A second device is
        // modeled as a second same-identity sync_request.
        use crate::space::state::{build_dm_space_create_event, SpaceState};
        let node_key = keypair::generate();
        let mut rt = NodeRuntime::new(node_key);
        let alice = keypair::generate();
        let alice_id = pubkey_uri(&alice);
        rt.register_identity(make_identity_record(&alice_id)).unwrap();

        // Ingest the self-DM create chain (root → auto-room), invitee = alice.
        let dm_ev = sign_event(build_dm_space_create_event(&alice, &alice_id, HOME), &alice);
        let dm_space_id: String = event_id_str(&dm_ev);
        rt.ingest_event(dm_ev.clone());
        let (_state, room_ev, _invite) =
            SpaceState::from_dm_space_create(&dm_ev, &alice).unwrap();
        rt.ingest_event(room_ev);

        let runtime = Arc::new(Mutex::new(rt));
        let alice_typed = idx(&alice_id);

        // First client (the user) syncs → the self-DM is served (alice is its Owner).
        let (events, _cursor) = collect_sync_history(&runtime, &alice_typed, "", 1000).await;
        assert!(
            events
                .iter()
                .any(|e| e.event_id.as_ref().map(|x| x.as_str()) == Some(dm_space_id.as_str())),
            "the self-DM create event must be reachable via the user's own sync"
        );

        // A second client authenticated as the same user (a second device) sees
        // the same thread — Node-resident, not device-local (M11-D2).
        let (events2, _c2) = collect_sync_history(&runtime, &alice_typed, "", 1000).await;
        assert!(
            events2
                .iter()
                .any(|e| e.event_id.as_ref().map(|x| x.as_str()) == Some(dm_space_id.as_str())),
            "a second client authenticated as the user sees the same self-DM (M11-D2)"
        );
    }

    // ── M8.5-B (INV-D1/D6) — scoped invite-bootstrap fetch ─────────────────
    //
    // Build a Space (alice owner) with a Room, a message (content, must NOT be
    // served), and a PENDING invite naming `invitee` with `valid_until` — the
    // invitee has NOT joined. Returns (runtime, space_id, message_id).
    fn setup_pending_invitee_space(
        invitee_uri: &str,
        valid_until: Option<&str>,
    ) -> (Arc<Mutex<NodeRuntime>>, String, String) {
        use crate::message::exchange::build_message_text_event;
        let node_key = keypair::generate();
        let mut rt = NodeRuntime::new(node_key);
        let alice = keypair::generate();
        let alice_id = pubkey_uri(&alice);
        rt.register_identity(make_identity_record(&alice_id)).unwrap();
        rt.register_identity(make_identity_record(invitee_uri)).unwrap();

        let space_ev =
            sign_event(build_space_create_event(&alice, "Boot", None, 1, HOME, None, false), &alice);
        let space_id: String = event_id_str(&space_ev);
        rt.ingest_event(space_ev);
        let room_ev =
            sign_event(build_room_create_event(&alice, &space_id, "general", None), &alice);
        let room_id: String = event_id_str(&room_ev);
        rt.ingest_event(room_ev.clone());

        // A message (content) — chained off the room. Must be excluded (CP-3).
        let msg = sign_event(
            build_message_text_event(&alice, &space_id, &room_id, vec![room_id.clone()], "secret"),
            &alice,
        );
        let msg_id: String = event_id_str(&msg);
        rt.ingest_event(msg);

        // Invite naming the invitee, chained off the message tip. Pending only.
        let mut content = json!({ "target_identity": invitee_uri, "role": "member" });
        if let Some(vu) = valid_until {
            content["valid_until"] = json!(vu);
        }
        let mut invite =
            build_membership_event(&alice, &space_id, "", EventType::MembershipInvite, content);
        invite.prev_events = vec![edx(&msg_id)];
        rt.ingest_event(sign_event(invite, &alice));

        (Arc::new(Mutex::new(rt)), space_id, msg_id)
    }

    #[tokio::test]
    async fn collect_invite_bootstrap_serves_structural_only_to_pending_invitee() {
        let bob = keypair::generate();
        let bob_id = pubkey_uri(&bob);
        let future =
            (chrono::Utc::now() + chrono::Duration::hours(1)).to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        let (runtime, space_id, msg_id) = setup_pending_invitee_space(&bob_id, Some(&future));

        let served = collect_invite_bootstrap(&runtime, &idx(&bob_id), &space_id)
            .await
            .expect("pending invitee with unexpired invite must be served");

        // Structural events present: space_create, room_create, the invite.
        assert!(
            served.iter().any(|e| e.event_id.as_ref().map(|x| x.as_str()) == Some(space_id.as_str())),
            "must include the Space create"
        );
        assert!(
            served.iter().any(|e| matches!(e.event_type, EventType::StateRoomCreate)),
            "must include the Room create"
        );
        let invite = served
            .iter()
            .find(|e| matches!(e.event_type, EventType::MembershipInvite))
            .expect("must include the invite naming the requester");
        assert_eq!(
            invite.content["target_identity"].as_str(),
            Some(bob_id.as_str()),
            "served invite must name the requester (so it can read invite_id)"
        );
        // CP-3 privacy line: NO message content served.
        assert!(
            !served.iter().any(|e| matches!(e.event_type, EventType::MessageText)),
            "structural fetch must NOT serve message content"
        );
        assert!(
            served.iter().all(|e| e.event_id.as_ref().map(|x| x.as_str()) != Some(msg_id.as_str())),
            "the message event must be excluded"
        );
    }

    #[tokio::test]
    async fn collect_invite_bootstrap_refuses_non_invitee_1011() {
        let bob = keypair::generate();
        let bob_id = pubkey_uri(&bob);
        let future =
            (chrono::Utc::now() + chrono::Duration::hours(1)).to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        let (runtime, space_id, _msg) = setup_pending_invitee_space(&bob_id, Some(&future));

        // Carol holds no pending invite → refused.
        let carol = keypair::generate();
        let carol_id = pubkey_uri(&carol);
        let err = collect_invite_bootstrap(&runtime, &idx(&carol_id), &space_id)
            .await
            .expect_err("a non-invitee must be refused");
        assert_eq!(err, (1011, "invite_bootstrap_refused"));
    }

    #[tokio::test]
    async fn collect_invite_bootstrap_refuses_expired_invite_1011() {
        let bob = keypair::generate();
        let bob_id = pubkey_uri(&bob);
        let past =
            (chrono::Utc::now() - chrono::Duration::hours(1)).to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        let (runtime, space_id, _msg) = setup_pending_invitee_space(&bob_id, Some(&past));

        let err = collect_invite_bootstrap(&runtime, &idx(&bob_id), &space_id)
            .await
            .expect_err("an expired invite is a dead read capability");
        assert_eq!(err, (1011, "invite_bootstrap_refused"));
    }

    /// M8.5-B C1 end-to-end (node side): the production-shaped bootstrap. Bob
    /// **sources** the invite `event_id` from the scoped fetch (not a fixture
    /// hand-chain), chains its `membership.join` off it (INV-D3), and is admitted.
    /// This dissolves M85-A3: the join is causally *after* the invite, so it is
    /// not concurrent on the `membership:{space}:{bob}` key. The client wire glue
    /// (`ops::join` sourcing the fetch) lands in C2; here the node-side flow is
    /// proven without any hand-chained linkage.
    #[tokio::test]
    async fn invite_bootstrap_join_makes_member_via_sourced_invite_id() {
        use crate::node::runtime::{DispatchOutcome, EventOrigin};
        let bob = keypair::generate();
        let bob_id = pubkey_uri(&bob);
        let future = (chrono::Utc::now() + chrono::Duration::hours(1))
            .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        let (runtime, space_id, _msg) = setup_pending_invitee_space(&bob_id, Some(&future));

        // 1. Bob bootstraps and discovers the invite naming him.
        let served = collect_invite_bootstrap(&runtime, &idx(&bob_id), &space_id)
            .await
            .expect("served");
        let invite_id = served
            .iter()
            .find(|e| {
                matches!(e.event_type, EventType::MembershipInvite)
                    && e.content["target_identity"].as_str() == Some(bob_id.as_str())
            })
            .and_then(|e| e.event_id.as_ref().map(|x| x.as_str().to_string()))
            .expect("bob must discover the invite naming him");

        // 2. Bob chains its join off the sourced invite_id (INV-D3) and dispatches.
        let mut join =
            build_membership_event(&bob, &space_id, "", EventType::MembershipJoin, json!({}));
        join.prev_events = vec![edx(&invite_id)];
        let join = sign_event(join, &bob);
        let outcome = {
            let mut rt = runtime.lock().await;
            rt.dispatch_event(join, EventOrigin::LocallySubmitted, None)
        };
        assert!(
            matches!(outcome, DispatchOutcome::Accepted { new_joiner: Some(_), .. }),
            "bootstrap join (chained off the sourced invite) must be Accepted; got {:?}",
            outcome
        );
        assert!(
            runtime.lock().await.spaces[space_id.as_str()].is_member(&bob_id),
            "bob must be a member after the bootstrap join"
        );
    }

    // ── M-SPACE-ADMISSION Leg G-3 — the door ───────────────────────────────
    //
    // `collect_invite_bootstrap`'s authorization widens: a RETAINED DEPARTED
    // member who is NOT banned may fetch her own anchor without an invite.
    //
    // 🛑 The ban term is the control this leg exists to get right. Before G-3
    // the pending-invite line was doing two jobs — proving entitlement AND
    // excluding the banned as a side effect (a banned identity's invite is
    // removed by `apply_ban` / `apply_node_eject`). Widening replaces only the
    // first, and `left_at.is_some()` is true for a banned and an ejected
    // identity too, so without `space.banned` the widening would hand the
    // membership chain to someone the Space permanently excluded.

    #[derive(Clone, Copy)]
    enum DepartedBy {
        Leave,
        Kick,
        Ban,
        NodeEject,
    }

    struct DepartedFixture {
        runtime: Arc<Mutex<NodeRuntime>>,
        space_id: String,
        carol_id: String,
        /// Carol's own invite, join and departure — what `G-4` anchors on.
        carol_invite: String,
        carol_join: String,
        carol_departure: String,
        /// A THIRD PARTY's structure, before her departure and DURING her
        /// absence. `V-7`'s subject is the second one.
        bob_join: String,
        bob_leave_during_her_absence: String,
        /// Content — must never be served (CP-3).
        msg: String,
    }

    /// ```text
    ///   space_create → room_create
    ///   → invite(carol) → join(carol)      her own chain
    ///   → invite(bob)   → join(bob)        third-party structure, she was present
    ///   → "content"                        CP-3
    ///   → leave|kick|ban|node_eject(carol) the boundary — her anchor
    ///   → leave(bob)                       THIRD-PARTY departure during her absence
    /// ```
    /// `home_node` is the runtime's REAL node key so `membership.node_eject`
    /// (Node authority: `sender == home_node`) is constructible here.
    fn setup_departed_member_space(departure: DepartedBy) -> DepartedFixture {
        use crate::message::exchange::build_message_text_event;
        let node_key = keypair::generate();
        let node_uri = pubkey_uri(&node_key);
        let mut rt = NodeRuntime::new(node_key.clone());
        let alice = keypair::generate();
        let bob = keypair::generate();
        let carol = keypair::generate();
        let alice_id = pubkey_uri(&alice);
        let bob_id = pubkey_uri(&bob);
        let carol_id = pubkey_uri(&carol);
        rt.register_identity(make_identity_record(&alice_id)).unwrap();
        rt.register_identity(make_identity_record(&bob_id)).unwrap();
        rt.register_identity(make_identity_record(&carol_id)).unwrap();

        let space_ev = sign_event(
            build_space_create_event(&alice, "Door", None, 1, &node_uri, None, false),
            &alice,
        );
        let space_id: String = event_id_str(&space_ev);
        rt.ingest_event(space_ev);
        let sx = sdx(&space_id);

        let room_ev =
            sign_event(build_room_create_event(&alice, &space_id, "general", None), &alice);
        let room_id: String = event_id_str(&room_ev);
        rt.ingest_event(room_ev);

        let carol_invite = chain_ingest(
            &mut rt,
            &sx,
            build_membership_event(
                &alice,
                &space_id,
                "",
                EventType::MembershipInvite,
                json!({ "target_identity": carol_id, "role": "member" }),
            ),
            &alice,
        );
        let carol_join = chain_ingest(
            &mut rt,
            &sx,
            build_membership_event(&carol, &space_id, "", EventType::MembershipJoin, json!({})),
            &carol,
        );

        // A third party, admitted while carol was present.
        chain_ingest(
            &mut rt,
            &sx,
            build_membership_event(
                &alice,
                &space_id,
                "",
                EventType::MembershipInvite,
                json!({ "target_identity": bob_id, "role": "member" }),
            ),
            &alice,
        );
        let bob_join = chain_ingest(
            &mut rt,
            &sx,
            build_membership_event(&bob, &space_id, "", EventType::MembershipJoin, json!({})),
            &bob,
        );

        // Content (CP-3 — never served on this path).
        let msg = chain_ingest(
            &mut rt,
            &sx,
            build_message_text_event(&alice, &space_id, &room_id, vec![], "secret"),
            &alice,
        );

        // Carol departs — four shapes, one `banned` test between them.
        let carol_departure = match departure {
            DepartedBy::Leave => chain_ingest(
                &mut rt,
                &sx,
                build_membership_event(&carol, &space_id, "", EventType::MembershipLeave, json!({})),
                &carol,
            ),
            DepartedBy::Kick => chain_ingest(
                &mut rt,
                &sx,
                build_membership_event(
                    &alice,
                    &space_id,
                    "",
                    EventType::MembershipKick,
                    json!({ "target_identity": carol_id }),
                ),
                &alice,
            ),
            DepartedBy::Ban => chain_ingest(
                &mut rt,
                &sx,
                build_membership_event(
                    &alice,
                    &space_id,
                    "",
                    EventType::MembershipBan,
                    json!({ "target_identity": carol_id }),
                ),
                &alice,
            ),
            // Node authority: `apply_node_eject` requires `sender == home_node`.
            DepartedBy::NodeEject => chain_ingest(
                &mut rt,
                &sx,
                build_membership_event(
                    &node_key,
                    &space_id,
                    "",
                    EventType::MembershipNodeEject,
                    json!({ "target_identity": carol_id }),
                ),
                &node_key,
            ),
        };

        // `V-7`'s subject: a THIRD PARTY's departure, during her absence.
        let bob_leave_during_her_absence = chain_ingest(
            &mut rt,
            &sx,
            build_membership_event(&bob, &space_id, "", EventType::MembershipLeave, json!({})),
            &bob,
        );

        DepartedFixture {
            runtime: Arc::new(Mutex::new(rt)),
            space_id,
            carol_id,
            carol_invite,
            carol_join,
            carol_departure,
            bob_join,
            bob_leave_during_her_absence,
            msg,
        }
    }

    /// Assert the fixture actually reached the state each test depends on —
    /// a departed carol, and (for the two permanent exclusions) a banned one.
    /// Without this a silently-degraded fixture would make `V-2`/`V-3` pass for
    /// the wrong reason: refused-because-never-a-member reads exactly like
    /// refused-because-banned at the `Err` boundary.
    async fn assert_fixture_state(f: &DepartedFixture, expect_banned: bool) {
        let rt = f.runtime.lock().await;
        let space = &rt.spaces[f.space_id.as_str()];
        let m = space
            .members
            .get(&idx(&f.carol_id))
            .expect("carol's membership record must be RETAINED (`D-154`①②③⑥)");
        assert!(!m.is_present(), "carol must be marked departed");
        assert_eq!(
            space.banned.contains(&idx(&f.carol_id)),
            expect_banned,
            "fixture must reach the expected `banned` state"
        );
    }

    /// `V-1` — **THE SUBJECT.** A departed member holding NO invite is served,
    /// and the batch carries her own last membership event: the thing `G-4`
    /// will anchor her rejoin on.
    ///
    /// 🛑 Before this leg she was refused `1011` — not because she was
    /// unwelcome, but because `pending_invites.get(..).ok_or(REFUSED)?` was the
    /// only entitlement route and her invite was consumed by her FIRST join.
    #[tokio::test]
    async fn invite_bootstrap_serves_a_departed_member_who_holds_no_invite() {
        let f = setup_departed_member_space(DepartedBy::Leave);
        assert_fixture_state(&f, false).await;

        // Precondition: she really holds no pending invite — otherwise this
        // test would be exercising the ORIGINAL route, not the new one.
        {
            let rt = f.runtime.lock().await;
            assert!(
                !rt.spaces[f.space_id.as_str()]
                    .pending_invites
                    .contains_key(&idx(&f.carol_id)),
                "her invite was consumed at her first join; the new route is what admits her"
            );
        }

        let served = collect_invite_bootstrap(&f.runtime, &idx(&f.carol_id), &f.space_id)
            .await
            .expect("a retained departed member who is not banned must be served");

        let ids: Vec<String> = served
            .iter()
            .filter_map(|e| e.event_id.as_ref().map(|x| x.as_str().to_string()))
            .collect();
        assert!(
            ids.contains(&f.carol_departure),
            "her own departure — the anchor `G-4` selects — must be served"
        );
        assert!(ids.contains(&f.carol_join), "her own join must be served");
        assert!(ids.contains(&f.carol_invite), "her own invite must be served");
        assert!(
            ids.contains(&f.space_id),
            "the Space create must be served — without it the batch is unparseable"
        );
        assert!(
            served.iter().any(|e| matches!(e.event_type, EventType::StateRoomCreate)),
            "the Room create must be served"
        );
        // CP-3 privacy line, unchanged by this leg.
        assert!(!ids.contains(&f.msg), "content must NEVER be served on this path");
    }

    /// `V-2` — 🔒 **THE BAN CONTROL. THE ONE THIS LEG EXISTS TO GET RIGHT.**
    ///
    /// A banned former member satisfies `!is_present()` exactly as a leaver
    /// does. Without the explicit `space.banned` term the widening would serve
    /// her the Space's membership structure.
    #[tokio::test]
    async fn invite_bootstrap_refuses_a_banned_former_member_1011() {
        let f = setup_departed_member_space(DepartedBy::Ban);
        assert_fixture_state(&f, true).await;

        let err = collect_invite_bootstrap(&f.runtime, &idx(&f.carol_id), &f.space_id)
            .await
            .expect_err("a BANNED former member must be refused, not served her anchor");
        assert_eq!(err, (1011, "invite_bootstrap_refused"));
    }

    /// `V-3` — **THE EJECTION CONTROL.** `apply_node_eject` reaches `apply_ban`'s
    /// end state by a different authority (`D-154`⑥) — it BANS two lines below
    /// its `mark_departed` — so ONE `banned` test covers both permanent
    /// exclusions and no second predicate is written for the Node-authority path.
    #[tokio::test]
    async fn invite_bootstrap_refuses_a_node_ejected_former_member_1011() {
        let f = setup_departed_member_space(DepartedBy::NodeEject);
        assert_fixture_state(&f, true).await;

        let err = collect_invite_bootstrap(&f.runtime, &idx(&f.carol_id), &f.space_id)
            .await
            .expect_err("a node-ejected former member must be refused");
        assert_eq!(err, (1011, "invite_bootstrap_refused"));
    }

    /// `V-4` — **THE KICK CONTROL, and it is the one that proves the ban term is
    /// a BAN test rather than a departure test.** `apply_kick` marks departed and
    /// does NOT ban (`D-154`②③) ⇒ she is eligible to return, so she is eligible
    /// to fetch her anchor. A term reading *"was removed by someone else"*
    /// instead of *"is banned"* would turn this red.
    #[tokio::test]
    async fn invite_bootstrap_serves_a_kicked_member_who_is_not_banned() {
        let f = setup_departed_member_space(DepartedBy::Kick);
        assert_fixture_state(&f, false).await;

        let served = collect_invite_bootstrap(&f.runtime, &idx(&f.carol_id), &f.space_id)
            .await
            .expect("a kicked (not banned) member may still fetch her anchor");
        let ids: Vec<String> = served
            .iter()
            .filter_map(|e| e.event_id.as_ref().map(|x| x.as_str().to_string()))
            .collect();
        assert!(
            ids.contains(&f.carol_departure),
            "the kick NAMING her is her last membership event and must be served"
        );
    }

    /// `V-5` — **THE STRANGER CONTROL.** Never a member, no invite. Unchanged
    /// behaviour, asserted against the SAME fixture so the widening is shown to
    /// admit exactly one new class and not simply to weaken the door.
    #[tokio::test]
    async fn invite_bootstrap_still_refuses_a_stranger_1011() {
        let f = setup_departed_member_space(DepartedBy::Leave);
        let dave = keypair::generate();
        let dave_id = pubkey_uri(&dave);

        let err = collect_invite_bootstrap(&f.runtime, &idx(&dave_id), &f.space_id)
            .await
            .expect_err("someone who was never a member and holds no invite must be refused");
        assert_eq!(err, (1011, "invite_bootstrap_refused"));
    }

    /// `V-7` — 🔒 **THE DISCLOSURE CONTROL. MANDATORY: §3 ruled ②, and without
    /// this ② is an intention rather than a behaviour.**
    ///
    /// She is standing OUTSIDE. A third party's departure during her absence is
    /// in the store and must NOT be in her batch: `D-154`④-as-clarified ruled
    /// what a RETURNING member receives, and serving the chain here would widen
    /// that from *after readmission* to *on request, while still outside*.
    ///
    /// 🛑 Asserted for BOTH third-party shapes, because they fail differently:
    /// bob's JOIN carries him as `sender`, his LEAVE carries him as `sender`
    /// too — but a `kick` would carry the ACTOR as sender and the subject in
    /// `content`. The per-type field test is what keeps a `kick` SHE issued
    /// (her as `sender`, a third party as target) out of her batch.
    #[tokio::test]
    async fn invite_bootstrap_withholds_third_party_membership_from_a_departed_member() {
        let f = setup_departed_member_space(DepartedBy::Leave);

        // The events are genuinely in the store — otherwise this asserts nothing.
        {
            let rt = f.runtime.lock().await;
            let stored: Vec<String> = rt.stores[&sdx(&f.space_id)]
                .range(0)
                .unwrap_or_default()
                .iter()
                .filter_map(|e| e.event_id.as_ref().map(|x| x.as_str().to_string()))
                .collect();
            assert!(
                stored.contains(&f.bob_leave_during_her_absence),
                "precondition: the third party's departure must BE in the store"
            );
            assert!(stored.contains(&f.bob_join), "precondition: bob's join is in the store");
        }

        let served = collect_invite_bootstrap(&f.runtime, &idx(&f.carol_id), &f.space_id)
            .await
            .expect("served");
        let ids: Vec<String> = served
            .iter()
            .filter_map(|e| e.event_id.as_ref().map(|x| x.as_str().to_string()))
            .collect();

        assert!(
            !ids.contains(&f.bob_leave_during_her_absence),
            "§3 ②: she must NOT learn that a third party left while she was away"
        );
        assert!(
            !ids.contains(&f.bob_join),
            "§3 ②: nor a third party's admission — she gets only what names her"
        );
        // ...while her own chain is intact. A filter that served her nothing
        // would satisfy the two assertions above and fail the leg.
        assert!(ids.contains(&f.carol_departure), "her own chain must survive the filter");
        assert!(ids.contains(&f.carol_join), "her own chain must survive the filter");
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
            build_space_create_event(&alice, "P", None, 1, HOME, None, false),
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
            store.contains(&EventXgid::from_xgid(Xgid::new(msg_b_id.to_string()))),
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
        // M8.5-B (C2) — a regular-Space invite stamps `valid_until` (a real
        // client always does post-C2; the join-acceptance gate is fail-closed
        // for non-DM Spaces). Aligning the fixture to production, not gaming the
        // gate.
        let dave_valid_until = (chrono::Utc::now() + chrono::Duration::days(14))
            .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        let mut invite = build_membership_event(
            &alice,
            &space_id,
            "",
            EventType::MembershipInvite,
            json!({ "target_identity": dave_id, "role": "member", "valid_until": dave_valid_until }),
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
            store.contains(&EventXgid::from_xgid(Xgid::new(invite_2_id.to_string()))),
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
                let reason = reason.reason;
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
            build_space_create_event(&alice, "Test", None, 1, HOME, None, false),
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
            build_space_create_event(&key, "t6-space", None, 1, "xgen://pubkey/ed25519:home", None, false),
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

    // ── M-SPACE-ADMISSION Leg E-2 — `D-154`④, the gap ─────────────────────
    //
    // `E2-6`. Per the runbook's §4 structural binding, the WALK
    // (`permitted_event_ids`) and the DOORS (`apply_fanout`,
    // `collect_sync_history`) are barred from sharing a test: a filter proven
    // only at a door cannot show the walk is right, and a walk proven only in
    // isolation cannot show the door calls it. The kick control below belongs
    // to the walk; the sync-history tests belong to door ②.

    /// Which shape ends carol's membership in the fixture. `leave` names the
    /// departed as `sender`; `kick` names the *actor* as sender and carol in
    /// `content["target_identity"]` — `N-197`'s whole subject.
    #[derive(Clone, Copy, PartialEq)]
    enum Departure {
        Leave,
        Kick,
    }

    /// A Space carrying one completed absence for carol.
    struct GapFixture {
        rt: NodeRuntime,
        space_id: String,
        room_id: String,
        alice: ed25519_dalek::SigningKey,
        carol: ed25519_dalek::SigningKey,
        alice_id: String,
        carol_id: String,
        pre_msg: String,
        gap_msg: String,
        post_msg: String,
        bob_join: String,
        departure: String,
        rejoin: String,
    }

    /// Chain `ev` off the Space's running DAG tip, sign it, ingest it, return
    /// its id. **`ingest_event` skips validation, but an unchained non-root
    /// membership event resolves as concurrent and is silently dropped** — the
    /// `setup_three_member_space` fixture says so at length, and E-1's
    /// `fanout_excludes_a_departed_member` chains for the same reason.
    fn chain_ingest(
        rt: &mut NodeRuntime,
        sx: &SpaceXgid,
        mut ev: Event,
        key: &ed25519_dalek::SigningKey,
    ) -> String {
        let tip = rt.dag_tips(sx)[0].clone();
        ev.prev_events = vec![edx(&tip)];
        let ev = sign_event(ev, key);
        let id = event_id_str(&ev);
        rt.ingest_event(ev);
        id
    }

    /// The Leg E-2 fixture:
    ///
    /// ```text
    ///   space_create → room_create → invite(carol) → join(carol)
    ///   → "pre"                 alice speaks while carol is PRESENT
    ///   → leave|kick(carol)     the boundary
    ///   → invite(bob) → join(bob)   STRUCTURE during the gap
    ///   → "gap"                 CONTENT during the gap
    ///   → join(carol)           the rejoin (no invite — `D-154`①)
    ///   → "post"                alice speaks while carol is PRESENT again
    /// ```
    fn setup_gap_space(departure: Departure) -> GapFixture {
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
            sign_event(build_space_create_event(&alice, "Gap", None, 1, HOME, None, false), &alice);
        let space_id: String = event_id_str(&space_ev);
        rt.ingest_event(space_ev);
        let sx = sdx(&space_id);

        let room_ev =
            sign_event(build_room_create_event(&alice, &space_id, "general", None), &alice);
        let room_id: String = event_id_str(&room_ev);
        rt.ingest_event(room_ev);

        // Carol is invited and joins.
        chain_ingest(
            &mut rt,
            &sx,
            build_membership_event(
                &alice,
                &space_id,
                "",
                EventType::MembershipInvite,
                json!({ "target_identity": carol_id, "role": "member" }),
            ),
            &alice,
        );
        chain_ingest(
            &mut rt,
            &sx,
            build_membership_event(&carol, &space_id, "", EventType::MembershipJoin, json!({})),
            &carol,
        );

        // Alice speaks while carol is present.
        let pre_msg = chain_ingest(
            &mut rt,
            &sx,
            build_message_text_event(&alice, &space_id, &room_id, vec![], "pre"),
            &alice,
        );

        // Carol departs — by her own hand, or at alice's.
        let departure_id = match departure {
            Departure::Leave => chain_ingest(
                &mut rt,
                &sx,
                build_membership_event(&carol, &space_id, "", EventType::MembershipLeave, json!({})),
                &carol,
            ),
            Departure::Kick => chain_ingest(
                &mut rt,
                &sx,
                build_membership_event(
                    &alice,
                    &space_id,
                    "",
                    EventType::MembershipKick,
                    json!({ "target_identity": carol_id }),
                ),
                &alice,
            ),
        };

        // Structure during the gap: bob is invited and joins.
        chain_ingest(
            &mut rt,
            &sx,
            build_membership_event(
                &alice,
                &space_id,
                "",
                EventType::MembershipInvite,
                json!({ "target_identity": bob_id, "role": "member" }),
            ),
            &alice,
        );
        let bob_join = chain_ingest(
            &mut rt,
            &sx,
            build_membership_event(&bob, &space_id, "", EventType::MembershipJoin, json!({})),
            &bob,
        );

        // Content during the gap.
        let gap_msg = chain_ingest(
            &mut rt,
            &sx,
            build_message_text_event(&alice, &space_id, &room_id, vec![], "gap"),
            &alice,
        );

        // Carol rejoins. `D-154`① — no invite is required and none is issued.
        let rejoin = chain_ingest(
            &mut rt,
            &sx,
            build_membership_event(&carol, &space_id, "", EventType::MembershipJoin, json!({})),
            &carol,
        );

        // Content after the rejoin.
        let post_msg = chain_ingest(
            &mut rt,
            &sx,
            build_message_text_event(&alice, &space_id, &room_id, vec![], "post"),
            &alice,
        );

        GapFixture {
            rt,
            space_id,
            room_id,
            alice,
            carol,
            alice_id,
            carol_id,
            pre_msg,
            gap_msg,
            post_msg,
            bob_join,
            departure: departure_id,
            rejoin,
        }
    }

    /// All events currently in the Space's store.
    fn store_events(rt: &NodeRuntime, space_id: &str) -> Vec<Event> {
        rt.stores
            .get(&sdx(space_id))
            .expect("store")
            .range(0)
            .unwrap_or_default()
    }

    /// Drain a receiver and return the first `HistoryBatch` it carries.
    fn take_history(rx: &mut mpsc::Receiver<OutboundMsg>) -> Vec<Event> {
        while let Ok(msg) = rx.try_recv() {
            if let OutboundMsg::HistoryBatch { events } = msg {
                return events;
            }
        }
        panic!("expected a HistoryBatch");
    }

    // ── the WALK (`E2-1`) ──────────────────────────────────────────────────

    /// `E2-6`.5 — **THE KICK CONTROL, and it belongs to the WALK** (runbook §4).
    ///
    /// 🛑 `N-197`: a walk reading only `event.sender` classifies a `leave`
    /// correctly and a `kick` not at all — for a kicked, banned or ejected
    /// member it produces a plausible, non-empty, WRONG slice, and every
    /// `leave`-based test still passes. This asserts the kick shape at the walk
    /// directly, so `W-3c` (disarm the walk to read only `sender`) turns THIS
    /// red while leaving the `leave`-based door test green. **That exact split
    /// is the proof; if both go red the control is not isolating what it claims.**
    #[test]
    fn walk_closes_the_gap_when_the_departure_is_a_kick_not_a_leave() {
        let f = setup_gap_space(Departure::Kick);
        let all = store_events(&f.rt, &f.space_id);
        let permitted = permitted_event_ids(&all, &idx(&f.carol_id));

        assert!(
            !permitted.contains(&edx(&f.gap_msg)),
            "N-197: a KICKED member's gap must be closed — a walk reading only \
             `event.sender` never sees this departure and admits the gap"
        );
        assert!(permitted.contains(&edx(&f.pre_msg)), "pre-departure content is hers");
        assert!(permitted.contains(&edx(&f.post_msg)), "post-rejoin content is hers");
        assert!(
            permitted.contains(&edx(&f.bob_join)),
            "D-154 clause 4 as clarified: STRUCTURE inside the gap still passes"
        );
    }

    /// `E2-6`.4 — **two cycles, two gaps.** N leave/rejoin cycles yield N+1
    /// intervals; a single-boundary implementation is wrong by construction
    /// (Leg E Phase-0 §5d(D)), and only a second cycle can show it.
    #[test]
    fn walk_closes_both_gaps_across_two_leave_rejoin_cycles() {
        let f = setup_gap_space(Departure::Leave);
        let mut rt = f.rt;
        let sx = sdx(&f.space_id);

        // Second cycle: leave, gap content, rejoin, post content.
        chain_ingest(
            &mut rt,
            &sx,
            build_membership_event(&f.carol, &f.space_id, "", EventType::MembershipLeave, json!({})),
            &f.carol,
        );
        let gap2 = chain_ingest(
            &mut rt,
            &sx,
            build_message_text_event(&f.alice, &f.space_id, &f.room_id, vec![], "gap2"),
            &f.alice,
        );
        chain_ingest(
            &mut rt,
            &sx,
            build_membership_event(&f.carol, &f.space_id, "", EventType::MembershipJoin, json!({})),
            &f.carol,
        );
        let post2 = chain_ingest(
            &mut rt,
            &sx,
            build_message_text_event(&f.alice, &f.space_id, &f.room_id, vec![], "post2"),
            &f.alice,
        );

        let all = store_events(&rt, &f.space_id);
        let permitted = permitted_event_ids(&all, &idx(&f.carol_id));

        assert!(!permitted.contains(&edx(&f.gap_msg)), "first gap closed");
        assert!(!permitted.contains(&edx(&gap2)), "SECOND gap closed");
        assert!(permitted.contains(&edx(&f.pre_msg)), "before the first departure");
        assert!(
            permitted.contains(&edx(&f.post_msg)),
            "between the two absences she was present — that interval is hers"
        );
        assert!(permitted.contains(&edx(&post2)), "after the second rejoin");
    }

    /// **Chat's addition beyond the specified nine, and it is REPORTED as one
    /// (Rule 6).** The runbook's §3 close conditions attach *"`room_id` is
    /// empty"* to the REOPEN and not to the closes. Measured against the
    /// appliers: `apply_leave` and `apply_kick` each **return early on a
    /// room-level event without touching `left_at`** (`state.rs`), so a walk
    /// that closed on a room-level leave would open a gap the fold never
    /// opened — the exact walk-disagrees-with-`left_at` failure Phase-0 §4b
    /// chose option (B) to eliminate. `apply_ban` / `apply_node_eject` have no
    /// room-level branch and so close regardless; the walk matches both.
    ///
    /// Without this test the correct condition is one "simplification" away
    /// from a green suite — `F-3`'s species from Leg E-1.
    #[test]
    fn walk_ignores_a_room_level_leave_because_the_applier_does() {
        let node_key = keypair::generate();
        let mut rt = NodeRuntime::new(node_key);
        let alice = keypair::generate();
        let carol = keypair::generate();
        let alice_id = pubkey_uri(&alice);
        let carol_id = pubkey_uri(&carol);
        rt.register_identity(make_identity_record(&alice_id)).unwrap();
        rt.register_identity(make_identity_record(&carol_id)).unwrap();

        let space_ev =
            sign_event(build_space_create_event(&alice, "RL", None, 1, HOME, None, false), &alice);
        let space_id: String = event_id_str(&space_ev);
        rt.ingest_event(space_ev);
        let sx = sdx(&space_id);
        let room_ev =
            sign_event(build_room_create_event(&alice, &space_id, "general", None), &alice);
        let room_id: String = event_id_str(&room_ev);
        rt.ingest_event(room_ev);

        chain_ingest(
            &mut rt,
            &sx,
            build_membership_event(
                &alice,
                &space_id,
                "",
                EventType::MembershipInvite,
                json!({ "target_identity": carol_id, "role": "member" }),
            ),
            &alice,
        );
        chain_ingest(
            &mut rt,
            &sx,
            build_membership_event(&carol, &space_id, "", EventType::MembershipJoin, json!({})),
            &carol,
        );
        // A ROOM-level leave: room_id is NON-empty.
        chain_ingest(
            &mut rt,
            &sx,
            build_membership_event(
                &carol,
                &space_id,
                &room_id,
                EventType::MembershipLeave,
                json!({}),
            ),
            &carol,
        );
        let after = chain_ingest(
            &mut rt,
            &sx,
            build_message_text_event(&alice, &space_id, &room_id, vec![], "after"),
            &alice,
        );

        // The fold agrees: she never left the Space.
        assert!(
            rt.spaces[&sx].is_member(&carol_id),
            "precondition — a room-level leave must not mark her departed"
        );

        let all = store_events(&rt, &space_id);
        let permitted = permitted_event_ids(&all, &idx(&carol_id));
        assert!(
            permitted.contains(&edx(&after)),
            "a room-level leave must not open a Space-level gap — the walk must \
             match apply_leave's early return"
        );
        assert_eq!(
            permitted.len(),
            all.len(),
            "nothing at all is withheld from a member who never left the Space"
        );
    }

    /// `E2-6`.8 — **SUBSTITUTED, AND THE SUBSTITUTION IS THE FINDING (Rule 6).**
    ///
    /// The runbook specifies *"`topological_sort` and `topological_sort_events`
    /// **agree** on a fixture DAG with concurrency."* 🛑 **Measured: they do
    /// not.** On roots `{a, z}` with `a → b` and `b` sorting before `z`,
    /// `topological_sort_events` yields `[a, b, z]` while `topological_sort`
    /// yields `[a, z, b]`; on a diamond plus an independent chain, `[a,b,c,d,m,n]`
    /// against `[a,m,b,c,n,d]`. **Both are valid topological orders** — the
    /// delivery sort re-sorts the whole remaining set each round and emits every
    /// ready event (depth-favouring), while core's is Kahn with a FIFO queue
    /// (breadth-favouring). An order-equality assertion is therefore not
    /// writable, and could be made true only by unifying them — Phase-0 §4(C),
    /// explicitly out of scope for this leg.
    ///
    /// ✅ **The divergence does not touch clause 4's correctness, and that is
    /// exactly what option (B) bought:** the SET is decided by core's order —
    /// the same order the fold used to compute `left_at` — and the delivery
    /// ORDER is left to the delivery sort. Two functions, two jobs, neither
    /// asked to do the other's.
    ///
    /// 🔒 **So this pins what E-2 actually depends on, which order-equality
    /// never did:** both sorts return the **same SET** (core's is lossy where
    /// the delivery sort explicitly preserves all input — `filter_map` on
    /// `event_id`, and Kahn never emits a cycle member) and both emit every
    /// in-set predecessor before its successor. **If core's sort ever starts
    /// losing events this goes red** — and that is the drift that would
    /// silently withhold events from everyone.
    #[test]
    fn two_sorts_preserve_the_event_set_and_causal_order() {
        use std::collections::HashSet as StdHashSet;

        // Concurrency at every shape that matters: two roots, a fork, a
        // diamond join, and an independent chain.
        let evs = vec![
            mk_event("a", &[]),
            mk_event("m", &[]),
            mk_event("b", &["a"]),
            mk_event("c", &["a"]),
            mk_event("d", &["b", "c"]),
            mk_event("n", &["m"]),
        ];

        let via_events = topological_sort_events(evs.clone());
        let via_core = topological_sort(evs.clone());

        let set_in: StdHashSet<String> =
            evs.iter().map(|e| e.event_id.as_ref().unwrap().as_str().to_string()).collect();
        let set_events: StdHashSet<String> =
            via_events.iter().map(|e| e.event_id.as_ref().unwrap().as_str().to_string()).collect();
        let set_core: StdHashSet<String> =
            via_core.iter().map(|e| e.event_id.as_ref().unwrap().as_str().to_string()).collect();

        assert_eq!(set_events, set_in, "the delivery sort must preserve all input");
        assert_eq!(
            set_core, set_in,
            "core's sort must preserve all input — E2-1 derives the permitted SET \
             from this walk, so an event lost here is an event withheld from everyone"
        );

        // Both orders must be causally valid.
        for (label, sorted) in [("events", &via_events), ("core", &via_core)] {
            let mut seen: StdHashSet<String> = StdHashSet::new();
            for e in sorted.iter() {
                for p in &e.prev_events {
                    if set_in.contains(p.as_str()) {
                        assert!(
                            seen.contains(p.as_str()),
                            "{}: {} emitted before its predecessor {}",
                            label,
                            e.event_id.as_ref().unwrap().as_str(),
                            p.as_str()
                        );
                    }
                }
                seen.insert(e.event_id.as_ref().unwrap().as_str().to_string());
            }
        }
    }

    // ── DOOR ① — the joiner push (`E2-2`) ──────────────────────────────────

    /// `E2-6`.1 / `W-4` — **NO-OP FOR A FIRST-TIME JOINER, asserted
    /// BYTE-IDENTICALLY rather than by a count.** This is the property that
    /// keeps E-2 from being a regression: a joiner with no departures has one
    /// interval covering the whole log, so her payload must be exactly what it
    /// was before the filter existed.
    ///
    /// 🛑 **THE FIXTURE IS LOAD-BEARING AND THE FIRST ONE CHOSEN WAS NOT.**
    /// Written first against `setup_three_member_space`, this test PASSED under
    /// `W-3d` (open the first interval at her first join instead of at index 0)
    /// — because that fixture holds **only structural events**, every one of
    /// which the structural clause re-admits, so the disarm was invisible to it.
    /// It passed for a reason unrelated to what it claims: `F-3`'s species from
    /// Leg E-1. **Bob is the subject here instead** — he joins during carol's
    /// absence and never departs, and `pre_msg` is real CONTENT sitting before
    /// his join, so `W-3d` can actually bite. The precondition below is what
    /// stops the fixture silently degrading back.
    #[tokio::test]
    async fn first_time_joiner_history_is_byte_identical_to_the_unfiltered_push() {
        let f = setup_gap_space(Departure::Leave);
        let all = store_events(&f.rt, &f.space_id);
        let bob_join_id = edx(&f.bob_join);

        // What door 1 served BEFORE the filter existed: the whole store in
        // delivery order, minus the triggering event.
        let expected: Vec<String> = topological_sort_events(all.clone())
            .into_iter()
            .filter(|e| e.event_id.as_ref() != Some(&bob_join_id))
            .map(|e| event_id_str(&e))
            .collect();
        assert!(
            expected.contains(&f.pre_msg),
            "precondition — the payload must contain NON-STRUCTURAL content from              before this joiner's join, or W-3d cannot turn this test red"
        );

        let bob_join_ev = all
            .into_iter()
            .find(|e| e.event_id.as_ref() == Some(&bob_join_id))
            .expect("bob's join event");
        let bob_id = bob_join_ev.sender.clone();

        let runtime = Arc::new(Mutex::new(f.rt));
        let senders: ClientSenders = Arc::new(Mutex::new(HashMap::new()));
        let (tx_a, _rx_a) = mpsc::channel::<OutboundMsg>(64);
        let (tx_b, mut rx_b) = mpsc::channel::<OutboundMsg>(64);
        senders.lock().await.insert(idx(&f.alice_id), vec![(ConnId::mint(), tx_a)]);
        senders.lock().await.insert(bob_id.clone(), vec![(ConnId::mint(), tx_b)]);

        apply_fanout(
            FanoutRequest { event: Some(bob_join_ev), new_joiner: Some(bob_id.clone()) },
            &bob_id,
            &runtime,
            &senders,
        )
        .await;

        let got: Vec<String> = take_history(&mut rx_b).iter().map(event_id_str).collect();
        assert_eq!(
            got, expected,
            "W-4: a first-time joiner's push must be byte-identical to the              unfiltered payload — same events, same order"
        );
        assert!(!got.is_empty(), "positive control: the push is not trivially empty");
    }

    /// `E2-6`.2 — **the leg's subject.** A returning member does NOT receive the
    /// conversation held while she was away. `W-3a` (drop the filter at door ①)
    /// turns this red; `W-3c` (walk reads only `sender`) must leave it GREEN,
    /// because this fixture's departure is a `leave`.
    #[tokio::test]
    async fn rejoiner_push_withholds_the_gap_conversation() {
        let f = setup_gap_space(Departure::Leave);
        let sx = sdx(&f.space_id);
        let rejoin_ev = store_events(&f.rt, &f.space_id)
            .into_iter()
            .find(|e| e.event_id.as_ref() == Some(&edx(&f.rejoin)))
            .expect("the rejoin event");

        // Precondition: she is present again, so door ① is genuinely reachable.
        assert!(f.rt.spaces[&sx].is_member(&f.carol_id), "she rejoined");

        let runtime = Arc::new(Mutex::new(f.rt));
        let senders: ClientSenders = Arc::new(Mutex::new(HashMap::new()));
        let (tx_a, _rx_a) = mpsc::channel::<OutboundMsg>(64);
        let (tx_c, mut rx_c) = mpsc::channel::<OutboundMsg>(64);
        senders.lock().await.insert(idx(&f.alice_id), vec![(ConnId::mint(), tx_a)]);
        senders.lock().await.insert(idx(&f.carol_id), vec![(ConnId::mint(), tx_c)]);

        let carol_typed = idx(&f.carol_id);
        apply_fanout(
            FanoutRequest { event: Some(rejoin_ev), new_joiner: Some(carol_typed.clone()) },
            &carol_typed,
            &runtime,
            &senders,
        )
        .await;

        let got: Vec<String> = take_history(&mut rx_c).iter().map(event_id_str).collect();
        assert!(
            got.contains(&f.pre_msg),
            "positive control: she still receives what was said while she was here"
        );
        assert!(
            !got.contains(&f.gap_msg),
            "D-154 clause 4: the conversation held during her absence is withheld"
        );
        assert!(
            got.contains(&f.post_msg),
            "and everything from the rejoin forward is hers"
        );
    }

    /// `E2-6`.3 — **structure passes.** `D-154`④ as clarified 2026-08-23: the
    /// gap is closed to CONTENT and open to MEMBERSHIP STRUCTURE. She learns
    /// that bob joined while she was away; she does not learn what he said.
    /// `W-3e` (admit nothing while absent) turns this red and leaves
    /// `E2-6`.2 green.
    #[tokio::test]
    async fn rejoiner_push_still_carries_the_gap_membership_structure() {
        let f = setup_gap_space(Departure::Leave);
        let rejoin_ev = store_events(&f.rt, &f.space_id)
            .into_iter()
            .find(|e| e.event_id.as_ref() == Some(&edx(&f.rejoin)))
            .expect("the rejoin event");

        let runtime = Arc::new(Mutex::new(f.rt));
        let senders: ClientSenders = Arc::new(Mutex::new(HashMap::new()));
        let (tx_c, mut rx_c) = mpsc::channel::<OutboundMsg>(64);
        senders.lock().await.insert(idx(&f.carol_id), vec![(ConnId::mint(), tx_c)]);

        let carol_typed = idx(&f.carol_id);
        apply_fanout(
            FanoutRequest { event: Some(rejoin_ev), new_joiner: Some(carol_typed.clone()) },
            &carol_typed,
            &runtime,
            &senders,
        )
        .await;

        let got: Vec<String> = take_history(&mut rx_c).iter().map(event_id_str).collect();
        assert!(
            got.contains(&f.bob_join),
            "structure is not content — she receives bob's join from the gap"
        );
        assert!(
            got.contains(&f.departure),
            "and her own departure event, which is structural too"
        );
        assert!(
            !got.contains(&f.gap_msg),
            "while the conversation from the same window stays withheld"
        );
    }

    // ── DOOR ② — collect_sync_history (`E2-3`) ─────────────────────────────

    /// `E2-6`.6 / `E2-5`.2 (`C-5b`) — **the door the Phase-0 found.**
    /// `collect_sync_history`'s gate is `space.is_member(requester)`, the
    /// PRESENT-TENSE accessor Leg E-1 gated — so a rejoiner passes it and, before
    /// this leg, was served the entire store. `C-5b` was filed as *"self-closes
    /// under (i)"*: true for a DEPARTED member and false for a RETURNED one,
    /// which is why this test exists. `W-3b` (drop the filter at door ②) turns
    /// it red on its own — door ①'s tests stay green.
    #[tokio::test]
    async fn sync_history_withholds_the_gap_for_a_returned_member() {
        let f = setup_gap_space(Departure::Leave);
        let sx = sdx(&f.space_id);
        assert!(
            f.rt.spaces[&sx].is_member(&f.carol_id),
            "precondition — she passes the member gate, which is the point"
        );

        let runtime = Arc::new(Mutex::new(f.rt));
        let (page, _cursor) =
            collect_sync_history(&runtime, &idx(&f.carol_id), "", 1000).await;
        let ids: Vec<String> = page.iter().map(event_id_str).collect();

        assert!(ids.contains(&f.pre_msg), "positive control: pre-departure content is served");
        assert!(
            !ids.contains(&f.gap_msg),
            "D-154 clause 4 at door 2 — the pull is filtered exactly as the push is"
        );
        assert!(ids.contains(&f.post_msg), "post-rejoin content is served");
        assert!(ids.contains(&f.bob_join), "gap STRUCTURE still passes here too");
    }

    /// `E2-6`.9 — **the positive control that stops every probe here answering
    /// "withheld" for everyone.** Alice never departed; both doors must serve
    /// her the complete log, gap events included.
    #[tokio::test]
    async fn a_never_departed_member_is_unaffected_at_both_doors() {
        let f = setup_gap_space(Departure::Leave);
        let all_ids: Vec<String> = store_events(&f.rt, &f.space_id).iter().map(event_id_str).collect();

        // The walk, directly.
        let all = store_events(&f.rt, &f.space_id);
        let permitted = permitted_event_ids(&all, &idx(&f.alice_id));
        assert_eq!(
            permitted.len(),
            all.len(),
            "the walk withholds nothing from someone who never left"
        );

        // Door 2.
        let runtime = Arc::new(Mutex::new(f.rt));
        let (page, _cursor) =
            collect_sync_history(&runtime, &idx(&f.alice_id), "", 1000).await;
        let got: Vec<String> = page.iter().map(event_id_str).collect();
        assert_eq!(got.len(), all_ids.len(), "door 2 serves alice the complete log");
        assert!(
            got.contains(&f.gap_msg),
            "including the events carol may not see — otherwise this suite would \
             pass with a filter that withholds from everybody"
        );
    }

    /// `E2-6`.7 — **a cursor pointing at an event INSIDE a gap must not produce
    /// a silent empty sync.**
    ///
    /// 🛑 **Why this can bite at all, and why it is E-2's to answer:** before
    /// clause ④ every event of a member-Space was in the requester's candidate
    /// list, so a `position()` miss meant a genuinely unknown cursor and
    /// `(vec![], None)` was truthful. **The filter can now remove a cursor that
    /// resolves perfectly well** — and an empty page with no `continue_from` is
    /// byte-identical to "caught up" (`collect_sync_history_empty_when_caught_up`
    /// asserts exactly that shape). A client would silently believe it had
    /// everything. `D-065`: honest over polite.
    #[tokio::test]
    async fn sync_history_cursor_inside_a_gap_does_not_silently_empty() {
        let f = setup_gap_space(Departure::Leave);
        let runtime = Arc::new(Mutex::new(f.rt));

        // The cursor names an event she may not receive.
        let (page, _cursor) =
            collect_sync_history(&runtime, &idx(&f.carol_id), &f.gap_msg, 1000).await;
        let ids: Vec<String> = page.iter().map(event_id_str).collect();

        assert!(
            !page.is_empty(),
            "a cursor inside a gap must resume, not return an empty page that is \
             indistinguishable from being caught up"
        );
        assert!(
            ids.contains(&f.post_msg),
            "resumption continues at the first permitted event after the cursor"
        );
        assert!(
            !ids.contains(&f.gap_msg) && !ids.contains(&f.pre_msg),
            "and it does not rewind, nor un-withhold the gap"
        );
    }

    /// `E2-6`.7's realistic sibling — **the cursor a returning client actually
    /// holds.** She synced up to her last pre-departure event, then left. On
    /// rejoin her client sends that id as `since`. It is permitted, so it
    /// resolves in the filtered list directly, and what follows is the
    /// post-rejoin conversation with the gap removed.
    #[tokio::test]
    async fn sync_history_resumes_from_a_pre_departure_cursor_skipping_the_gap() {
        let f = setup_gap_space(Departure::Leave);
        let runtime = Arc::new(Mutex::new(f.rt));

        let (page, _cursor) =
            collect_sync_history(&runtime, &idx(&f.carol_id), &f.pre_msg, 1000).await;
        let ids: Vec<String> = page.iter().map(event_id_str).collect();

        assert!(!ids.contains(&f.pre_msg), "the cursor event itself is not re-sent");
        assert!(!ids.contains(&f.gap_msg), "the gap stays closed across a resume");
        assert!(ids.contains(&f.post_msg), "and she catches up on what came after");
        assert!(
            ids.contains(&f.bob_join),
            "structure from the gap still reaches her on this path too"
        );
    }

    /// `E2-5`.1 (`C-3`) — **`V-4`'s first half, inherited from Leg E-1 and
    /// discharged here by MEASUREMENT rather than by reading.**
    ///
    /// `runtime.rs`'s new-joiner detection is `!is_member(sender)`, and
    /// `is_member` is the present-tense accessor Leg E-1 gated. A departed
    /// record is retained but not present ⇒ a **rejoin dispatches as
    /// `Accepted { new_joiner: Some(_) }`**, which is what makes door ① fire for
    /// her at all — and therefore what makes `E2-2` load-bearing rather than
    /// decorative. Leg E-1 established this by code-reading; `V-4` exists
    /// precisely to refuse reading.
    #[test]
    fn a_rejoiner_dispatches_as_a_new_joiner() {
        use crate::node::runtime::{DispatchOutcome, EventOrigin};

        let node_key = keypair::generate();
        let mut rt = NodeRuntime::new(node_key);
        let alice = keypair::generate();
        let carol = keypair::generate();
        let alice_id = pubkey_uri(&alice);
        let carol_id = pubkey_uri(&carol);
        rt.register_identity(make_identity_record(&alice_id)).unwrap();
        rt.register_identity(make_identity_record(&carol_id)).unwrap();

        let space_ev =
            sign_event(build_space_create_event(&alice, "C3", None, 1, HOME, None, false), &alice);
        let space_id: String = event_id_str(&space_ev);
        rt.ingest_event(space_ev);
        let sx = sdx(&space_id);
        let room_ev =
            sign_event(build_room_create_event(&alice, &space_id, "general", None), &alice);
        rt.ingest_event(room_ev);

        chain_ingest(
            &mut rt,
            &sx,
            build_membership_event(
                &alice,
                &space_id,
                "",
                EventType::MembershipInvite,
                json!({ "target_identity": carol_id, "role": "member" }),
            ),
            &alice,
        );
        chain_ingest(
            &mut rt,
            &sx,
            build_membership_event(&carol, &space_id, "", EventType::MembershipJoin, json!({})),
            &carol,
        );
        chain_ingest(
            &mut rt,
            &sx,
            build_membership_event(&carol, &space_id, "", EventType::MembershipLeave, json!({})),
            &carol,
        );

        // Two-sided precondition: RETAINED, and NOT present.
        assert!(
            rt.spaces[&sx].members.contains_key(carol_id.as_str()),
            "D-154 - the record survives the departure"
        );
        assert!(!rt.spaces[&sx].is_member(&carol_id), "and she is not present");

        // Now dispatch the rejoin through the real path.
        let tip = rt.dag_tips(&sx)[0].clone();
        let mut rejoin =
            build_membership_event(&carol, &space_id, "", EventType::MembershipJoin, json!({}));
        rejoin.prev_events = vec![edx(&tip)];
        let rejoin = sign_event(rejoin, &carol);

        let outcome = rt.dispatch_event(rejoin, EventOrigin::LocallySubmitted, None);
        match outcome {
            DispatchOutcome::Accepted { new_joiner, .. } => assert_eq!(
                new_joiner.as_ref().map(|i| i.as_str()),
                Some(carol_id.as_str()),
                "C-3: a rejoiner IS a new joiner at dispatch — this is what makes \
                 door 1 fire for her, and therefore what E2-2 exists to filter"
            ),
            other => panic!("expected Accepted, got {:?}", other),
        }
    }
}
