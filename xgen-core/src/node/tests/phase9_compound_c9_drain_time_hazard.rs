// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

//! Phase 9 Compound C9 — F-3 drain-time approximation hazard.
//!
//! Contract per runbook §5.3 + findings §3.9: a federation event from peer X
//! buffers in PendingBuffer on missing predecessor. While buffered, X is
//! removed from S's `federation_nodes` (defederation). The predecessor
//! arrives and `drain_pending_uniform` re-dispatches the buffered event with
//! `peer_node_id: None` (production approximation — `BufferedEntry` does not
//! store originating peer per entry; F-3 is not re-checked on drain).
//!
//! **The test documents the accepting approximation**; it does NOT test
//! rejection. Production explicitly accepts this hazard at
//! `xgen-core/src/node/runtime.rs:864-865` (doc-comment in `drain_pending_uniform`):
//!
//! > "a buffered federation event whose peer relationship was torn down
//! >  within the 30 s HeldPending window slips through. Hazard is narrow
//! >  (operator-driven defederation rare, window short)"
//!
//! The bound assertion proves the test stays within the F-4a 30s HeldPending
//! window (otherwise the timeout sweep would have dropped the event before
//! drain). Test wall-clock is normally <100ms; the assertion is a sanity
//! upper-bound, not a perf measurement.

use crate::{
    crypto::encoding,
    identity::{keypair, registry::IdentityRecord},
    node::runtime::{DispatchOutcome, EventOrigin, NodeRuntime},
    space::state::{
        build_federation_add_event, build_room_create_event, build_space_create_event, sign_event,
    },
    wire::types::{Event, EventType},
};
use ed25519_dalek::SigningKey;
use serde_json::json;
use std::time::{Duration, Instant};
use xgen_common::xgid::{EventXgid, IdentityXgid, NodeXgid, RoomXgid, SpaceXgid, Xgid};

const F4A_WINDOW: Duration = Duration::from_secs(30);

fn pubkey_uri(key: &SigningKey) -> String {
    format!(
        "xgen://pubkey/ed25519:{}",
        encoding::encode(key.verifying_key().as_bytes())
    )
}

// ── Pass 1 Commit 4a typed XGID test helpers ─────────────────────────────────
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

fn make_record(key: &SigningKey, home_node: &str) -> IdentityRecord {
    IdentityRecord {
        identity_id: idx(&pubkey_uri(key)),
        display_name: None,
        is_ai: false,
        ai_capabilities: None,
        registered_at: "2026-05-25T00:00:00.000Z".to_string(),
        trust_assertion: None,
        devices: vec![],
        home_node: ndx(home_node),
        update_version: 0,
        revoked: false,
        revoked_at: None,
        revocation_reason: None,
    }
}

fn build_message_text(sender: &SigningKey, space_id: &str, room_id: &str, prev: Vec<String>, body: &str) -> Event {
    Event::new(
        EventType::MessageText,
        idx(&pubkey_uri(sender)),
        RoomXgid::from_xgid(Xgid::new(room_id.to_string())),
        SpaceXgid::from_xgid(Xgid::new(space_id.to_string())),
        prev.into_iter()
            .map(|s| EventXgid::from_xgid(Xgid::new(s)))
            .collect(),
        "2026-05-25T00:00:01.000Z".to_string(),
        json!({ "body": body }),
    )
}

#[tokio::test(flavor = "current_thread")]
async fn c9_f3_drain_time_approximation_within_30s_window() {
    // ── Setup ──
    let alice = keypair::generate();
    let node_key = keypair::generate();
    let mut rt = NodeRuntime::new(node_key);
    rt.register_identity(make_record(&alice, rt.node_id.as_str())).expect("alice");

    let space_ev = sign_event(
        build_space_create_event(&alice, "c9-space", None, 1, rt.node_id.as_str()),
        &alice,
    );
    let space_id = event_id_str(&space_ev);
    rt.ingest_event(space_ev);

    let room_ev = sign_event(
        build_room_create_event(&alice, &space_id, "general", None),
        &alice,
    );
    let room_id = event_id_str(&room_ev);
    rt.ingest_event(room_ev);

    // Federation peer X in S's federation_nodes (so F-3 passes for X's
    // initial dispatch of E).
    let peer_key = keypair::generate();
    let peer_id = pubkey_uri(&peer_key);
    let node_key_clone = rt.node_keypair.clone();
    let fed_add = sign_event(
        build_federation_add_event(
            &node_key_clone,
            &space_id,
            rt.dag_tips(&sdx(&space_id)),
            &peer_id,
            "xgen://hash/sha256:c9-setup",
            "0.1",
            "json",
        ),
        &node_key_clone,
    );
    rt.ingest_event(fed_add);

    // ── Step 1 — Build predecessor P (alice's message). DO NOT dispatch yet. ──
    let tip = rt.dag_tips(&sdx(&space_id));
    let p = sign_event(build_message_text(&alice, &space_id, &room_id, tip, "predecessor"), &alice);
    let p_id = event_id_str(&p);

    // ── Step 2 — Build E (alice's message) with prev_events = [P.event_id]. ──
    let e = sign_event(build_message_text(&alice, &space_id, &room_id, vec![p_id.clone()], "child"), &alice);
    let e_id = event_id_str(&e);

    // ── Step 3 — Dispatch E via X (federation channel). F-3 passes (X is in
    // federation_nodes). F-4 step 9 catches missing predecessor → HeldPending. ──
    let start = Instant::now();
    let peer_id_typed = ndx(&peer_id);
    let outcome = rt.dispatch_event(e, EventOrigin::ReceivedViaFederation, Some(&peer_id_typed));
    assert!(
        matches!(outcome, DispatchOutcome::HeldPending),
        "step 3: expected HeldPending on missing predecessor, got {outcome:?}"
    );
    assert!(
        rt.pending[space_id.as_str()].contains(e_id.as_str()),
        "step 3: E must be in PendingBuffer after F-4a HeldPending"
    );

    // ── Step 4 — Defederate X (remove from S's federation_nodes). ──
    // No production state.federation_remove event exists; we mutate the
    // SpaceState directly to simulate operator-driven defederation.
    let space = rt.spaces.get_mut(space_id.as_str()).expect("SpaceState must exist");
    space.federation_nodes.retain(|n| n.as_str() != peer_id.as_str());
    assert!(
        !rt.spaces[space_id.as_str()].federation_nodes.iter().any(|n| n.as_str() == peer_id.as_str()),
        "step 4: X must no longer be in federation_nodes after defederation"
    );

    // ── Step 5 — Dispatch P locally. Step 9 passes (P has no missing
    // predecessors). P ingests. dispatch_event's post-ingest hook calls
    // drain_pending_uniform for P.event_id → finds E waiting on P → re-dispatches
    // E with peer_node_id=None (production approximation per runtime.rs:864-865). ──
    let outcome = rt.dispatch_event(p, EventOrigin::LocallySubmitted, None);
    let elapsed = start.elapsed();
    assert!(
        matches!(outcome, DispatchOutcome::Accepted { .. }),
        "step 5: P must ingest, got {outcome:?}"
    );

    // ── Assertion — E ingests via drain even though X has been defederated.
    // This is the accepting approximation the production code explicitly
    // documents. F-3 is not re-checked because drain passes peer_node_id=None. ──
    // SE-D6: `rt.stores[..]` is `Box<dyn EventStore>`; the inherent
    // `contains(&str)` is gone, use the typed trait `contains(&EventXgid)`.
    assert!(
        rt.stores[space_id.as_str()].contains(&EventXgid::from_xgid(Xgid::new(e_id.clone()))),
        "C9 hazard: E must have ingested via drain (production accepting approximation per runtime.rs:864-865 — F-3 not re-checked on drain)"
    );
    assert!(
        rt.stores[space_id.as_str()].contains(&EventXgid::from_xgid(Xgid::new(p_id.clone()))),
        "P must have ingested when dispatched locally"
    );
    assert!(
        !rt.pending[space_id.as_str()].contains(e_id.as_str()),
        "E must have been drained from PendingBuffer after P arrival"
    );

    // ── Bound assertion — total elapsed wall-clock from buffering to drain
    // must be within the F-4a HeldPending window (30s). Tests normally run
    // in <100ms; the bound is a sanity ceiling, not a perf measurement. If
    // this assertion ever fires, either the test is mis-instrumented or
    // production F-4a window changed and the test needs re-baselining. ──
    assert!(
        elapsed < F4A_WINDOW,
        "bound assertion: drain happened in {elapsed:?}; must be within F-4a window {F4A_WINDOW:?}"
    );
}
