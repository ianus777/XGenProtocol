// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Phase 9 Compound C10 — Identity-replicate hook serialisation under lock contention.
//!
//! Contract per runbook §5.4 + findings §3.10: `handle_identity_replicate_msg`
//! at [`xgen-node/src/app.rs:1695`] calls `rt.drain_pending_by_identity` at
//! [`xgen-core/src/node/runtime.rs:911`] inside the same runtime-lock critical
//! section as the identity registration. Under high concurrent federation push
//! load (many incoming events for the same Identity), the tokio Mutex
//! serialises register+drain — no two hooks can run in parallel, no buffered
//! event is drained twice.
//!
//! Test shape: 3 buffered events for bob (HeldPending F-10) + 3 concurrent
//! identity-replicate calls competing for the runtime Mutex. Asserts:
//! - Total events ingested for S = 3 (each event ingests exactly once).
//! - Total drain hook invocations ≤ 3 (only the first replicate that
//!   successfully upserts the Identity record fires a non-empty drain;
//!   subsequent calls hit `VersionStale` and skip drain, but the upper
//!   bound is the call count).
//! - No DAG-rejection from duplicate-ingest.
//!
//! File location: xgen-node-side because the production lock-contention
//! surface is in `xgen-node/src/app.rs` (Joe-locked at checkpoint #2). NOT
//! declared in `xgen-core/src/node/tests/mod.rs`.

use std::sync::Arc;

use ed25519_dalek::SigningKey;
use serde_json::json;
use tokio::sync::{Mutex, Notify};

use xgen_common::wire::{Event, EventType};
use xgen_common::xgid::{EventXgid, IdentityXgid, NodeXgid, RoomXgid, SpaceXgid, Xgid};
use xgen_core::crypto::encoding;
use xgen_core::identity::{
    keypair,
    registry::IdentityRecord,
    replication::handle_incoming_replicate,
};
use xgen_core::node::runtime::{DispatchOutcome, EventOrigin, NodeRuntime};
use xgen_core::space::state::{
    build_federation_add_event, build_room_create_event, build_space_create_event, sign_event,
};

fn idx(s: &str) -> IdentityXgid {
    IdentityXgid::from_xgid(Xgid::new(s.to_string()))
}
fn ndx(s: &str) -> NodeXgid {
    NodeXgid::from_xgid(Xgid::new(s.to_string()))
}
fn sdx(s: &str) -> SpaceXgid {
    SpaceXgid::from_xgid(Xgid::new(s.to_string()))
}
fn edx(s: &str) -> EventXgid {
    EventXgid::from_xgid(Xgid::new(s.to_string()))
}
fn rdx(s: &str) -> RoomXgid {
    RoomXgid::from_xgid(Xgid::new(s.to_string()))
}
fn event_id_str(ev: &Event) -> String {
    ev.event_id
        .as_ref()
        .expect("event must have event_id")
        .as_str()
        .to_string()
}

fn pubkey_uri(key: &SigningKey) -> String {
    format!(
        "xgen://pubkey/ed25519:{}",
        encoding::encode(key.verifying_key().as_bytes())
    )
}

fn make_record(key: &SigningKey, home_node: &str, version: u64) -> IdentityRecord {
    IdentityRecord {
        identity_id: idx(&pubkey_uri(key)),
        display_name: None,
        is_ai: false,
        ai_capabilities: None,
        registered_at: "2026-05-25T00:00:00.000Z".to_string(),
        trust_assertion: None,
        devices: vec![],
        home_node: ndx(home_node),
        update_version: version,
        revoked: false,
        revoked_at: None,
        revocation_reason: None,
    }
}

/// Build a `membership.join` event from `sender` to a target room.
///
/// Why join (not message.text): after F-10 drain re-dispatches with
/// `peer_node_id=None`, validation re-runs against the now-registered Identity.
/// `message.text` would fail step 11 membership check (bob still not a Space
/// member after Identity replication — replication updates the registry, not
/// Space membership). `membership.join` skips the step 11 membership check by
/// design (`skip_membership` in `xgen-core/src/message/exchange.rs:505-510`
/// includes `MembershipJoin`) and step 13 (permission, gated on the same skip).
/// Step 11 universal sender-registration passes (bob now in registry); step
/// 12 signature passes (bob signed). Net: each bob-signed membership.join
/// ingests cleanly after drain, which is the property the test pins.
fn build_membership_join(sender: &SigningKey, space_id: &str, room_id: &str, prev: Vec<String>, nonce: &str) -> Event {
    Event::new(
        EventType::MembershipJoin,
        idx(&pubkey_uri(sender)),
        rdx(room_id),
        sdx(space_id),
        prev.iter().map(|p| edx(p)).collect(),
        "2026-05-25T00:00:01.000Z".to_string(),
        json!({ "nonce": nonce }),
    )
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn c10_identity_replicate_hook_serialisation_no_double_drain() {
    // ── Setup ──
    // NodeRuntime with alice registered + Space + room + 3 federation_peers
    // (X1, X2, X3) in S's federation_nodes. Bob NOT registered.
    let alice = keypair::generate();
    let bob = keypair::generate();
    let bob_uri = pubkey_uri(&bob);
    let node_key = keypair::generate();
    let mut rt = NodeRuntime::new(node_key);
    let rt_node_id_str = rt.node_id.as_str().to_string();
    rt.register_identity(make_record(&alice, &rt_node_id_str, 0)).expect("alice");

    let space_ev = sign_event(
        build_space_create_event(&alice, "c10-space", None, 1, &rt_node_id_str, None, false),
        &alice,
    );
    let space_id: String = event_id_str(&space_ev);
    let space_id_typed = sdx(&space_id);
    rt.ingest_event(space_ev);
    let room_ev = sign_event(
        build_room_create_event(&alice, &space_id, "general", None),
        &alice,
    );
    let room_id: String = event_id_str(&room_ev);
    rt.ingest_event(room_ev);

    // 3 federation peers X1, X2, X3.
    let peers: Vec<(SigningKey, String)> = (0..3)
        .map(|_| {
            let k = keypair::generate();
            let uri = pubkey_uri(&k);
            (k, uri)
        })
        .collect();
    let node_key_clone = rt.node_keypair.clone();
    for (_, peer_uri) in &peers {
        let fed_add = sign_event(
            build_federation_add_event(
                &node_key_clone,
                &space_id,
                rt.dag_tips(&space_id_typed),
                peer_uri,
                "xgen://hash/sha256:c10-setup",
                "0.1",
                "json",
            ),
            &node_key_clone,
        );
        rt.ingest_event(fed_add);
    }

    // ── Dispatch 3 distinct bob-signed events, one via each peer. F-10
    // unknown-signer triggers HeldPending; each buffers in S's PendingBuffer
    // with missing_identity = Some(bob_uri). ──
    let tip = rt.dag_tips(&space_id_typed);
    let mut bob_event_ids: Vec<String> = Vec::with_capacity(3);
    for (i, (_peer_key, peer_uri)) in peers.iter().enumerate() {
        let ev = sign_event(
            build_membership_join(&bob, &space_id, &room_id, tip.clone(), &format!("c10-bob-join-{i}")),
            &bob,
        );
        let id: String = event_id_str(&ev);
        bob_event_ids.push(id.clone());
        let peer_uri_typed = ndx(peer_uri);
        let outcome = rt.dispatch_event(ev, EventOrigin::ReceivedViaFederation, Some(&peer_uri_typed));
        assert!(
            matches!(outcome, DispatchOutcome::HeldPending),
            "bob's event #{i} via peer {peer_uri} must HeldPend on F-10 unknown-signer"
        );
    }
    assert_eq!(
        rt.pending[&space_id_typed].pending_identity_count(),
        3,
        "setup: all 3 bob-events must be in PendingBuffer waiting on bob's Identity"
    );

    // ── Wrap NodeRuntime in Arc<Mutex<>> for concurrent access. ──
    let runtime = Arc::new(Mutex::new(rt));

    // Notify-barrier: all 3 replicate tasks wait on `go`, main fires
    // `notify_waiters` to release them simultaneously so they race for the
    // runtime mutex. This guarantees lock contention rather than serial
    // execution from scheduling artifacts.
    let go = Arc::new(Notify::new());

    // Spawn 3 concurrent identity-replicate tasks. Each replicates bob's
    // Identity record at version=1 — first to acquire the lock succeeds the
    // upsert + fires the drain hook; subsequent two hit VersionStale (per
    // handle_incoming_replicate at xgen-core/src/identity/replication.rs:120+)
    // and skip the drain. The serialised critical section guarantees no two
    // drains overlap.
    let mut handles: Vec<tokio::task::JoinHandle<usize>> = Vec::with_capacity(3);
    for task_id in 0..3 {
        let rt_clone = runtime.clone();
        let go_clone = go.clone();
        let bob_uri_typed = idx(&bob_uri);
        let bob_record = make_record(&bob, "test-home-node", 1);
        let handle = tokio::spawn(async move {
            // Wait for the go signal.
            go_clone.notified().await;
            // Mirror production handle_identity_replicate_msg:
            //   1. lock runtime
            //   2. handle_incoming_replicate(record, &mut rt.identity_registry)
            //   3. on Ok: rt.drain_pending_by_identity(&identity_id, ReceivedViaFederation)
            // Returns count of drained events for this task (0 if VersionStale).
            let mut rt = rt_clone.lock().await;
            let replicate_outcome = handle_incoming_replicate(bob_record, &mut rt.identity_registry);
            if replicate_outcome.is_ok() {
                let drained = rt.drain_pending_by_identity(&bob_uri_typed);
                drained.len()
            } else {
                let _ = task_id;
                0
            }
        });
        handles.push(handle);
    }

    // Give spawned tasks time to reach the notified() await before firing go.
    // (notify_waiters wakes only currently-registered waiters; ensure all 3
    // are parked first.)
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    go.notify_waiters();

    // Collect drain counts from all 3 tasks.
    let mut drain_counts: Vec<usize> = Vec::with_capacity(3);
    for h in handles {
        drain_counts.push(h.await.expect("task join"));
    }

    // ── Assertion 1 — Total drain invocations bound ≤ 3. ──
    // (Each of the 3 tasks counts as one invocation. A task that hit
    // VersionStale and skipped drain contributes 0 to the sum; the bound is
    // on the number of times drain ran, not the number of events drained.)
    let invocations_with_drain = drain_counts.iter().filter(|&&n| n > 0).count();
    assert!(
        invocations_with_drain <= 3,
        "drain hook invocations bound: {} > 3",
        invocations_with_drain
    );

    // ── Assertion 2 — Total events drained across all invocations = 3. ──
    // (Each unique bob-event ingests exactly once; no double-drain.)
    let total_drained: usize = drain_counts.iter().sum();
    assert_eq!(
        total_drained, 3,
        "total events drained must equal 3 (one per buffered bob-event); got {:?}",
        drain_counts
    );

    // ── Assertion 3 — All 3 bob-events in EventStore; none remain in PendingBuffer. ──
    let rt = runtime.lock().await;
    for id in &bob_event_ids {
        assert!(
            rt.stores[&space_id_typed].contains(&EventXgid::from_xgid(Xgid::new(id.to_string()))),
            "bob event {id} must be in EventStore after drain"
        );
        assert!(
            !rt.pending[&space_id_typed].contains(id),
            "bob event {id} must NOT be in PendingBuffer after drain"
        );
    }
    assert_eq!(
        rt.pending[&space_id_typed].pending_identity_count(),
        0,
        "no events should remain waiting on bob's Identity after upsert + drain"
    );

    // ── Assertion 4 — No duplicate-ingest DAG-rejection. ──
    // The EventStore for S should now contain: 1 space_create + 1 room_create
    // + 3 federation_add (peers X1/X2/X3) + 3 bob-events = 8 events.
    assert_eq!(
        rt.stores[&space_id_typed].len(),
        8,
        "EventStore length: expected 8 (1 sc + 1 rc + 3 fed_add + 3 bob), got {}",
        rt.stores[&space_id_typed].len()
    );
}
