// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! M7-events `.events` integration test — **node seam** (EIT-D1, C2).
//!
//! The join the arc never exercised end-to-end. The fan-out → observer-registry
//! → matching/filtering path is covered (`fanout::tests::observer_receives_…`),
//! and the pipe handler → JSONL-drain → prune path is covered
//! (`events_pipe::tests::subscribe_then_stream_then_prune_on_close`), but only
//! against a **manually** injected channel. This test joins them: an observer
//! registered by the **real `.events` pipe handler** receives an event from a
//! **real `apply_fanout`** and emerges as bare `Event` JSONL out of the pipe.
//!
//! `apply_fanout` early-returns unless the event's Space is hosted (its
//! `event_nodes` are derived from the looked-up `SpaceState`), so the fixture
//! seats a creator-only Space in a real `NodeRuntime` (lighter than the
//! three-member fan-out fixture — observers are membership-independent, EV-D5).
//! Drive over `tokio::io::duplex` (the pipe is `#[cfg(windows)]`; the handler is
//! stream-generic). Serial on the process-global `node_observers` registry.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use ed25519_dalek::SigningKey;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::Mutex;
use tokio::time::timeout;

use xgen_common::wire::Event;
use xgen_common::xgid::{IdentityXgid, SpaceXgid, Xgid};
use xgen_core::identity::keypair;
use xgen_core::space::state::{build_room_create_event, build_space_create_event, sign_event};

use crate::message::exchange::build_message_text_event;
use crate::events_pipe::handle_events_connection;
use crate::fanout::{apply_fanout, ClientSenders, FanoutRequest};
use crate::node::runtime::NodeRuntime;
use crate::tests::phase9_harness::{make_identity_record, pubkey_uri};

const T: Duration = Duration::from_secs(5);
const HOME: &str = "xgen://pubkey/ed25519:NODE";

fn event_id_str(ev: &Event) -> String {
    ev.event_id
        .as_ref()
        .expect("event must have event_id")
        .as_str()
        .to_string()
}

/// A real `NodeRuntime` hosting one creator-only Space (space + room ingested),
/// enough for `apply_fanout` to resolve the Space and reach the observer block.
fn setup_creator_space() -> (Arc<Mutex<NodeRuntime>>, String, String, SigningKey) {
    let node_key = keypair::generate();
    let mut rt = NodeRuntime::new(node_key);
    let alice = keypair::generate();
    rt.register_identity(make_identity_record(&alice, HOME)).unwrap();

    let space_ev = sign_event(build_space_create_event(&alice, "Test", None, 1, HOME, None, false), &alice);
    let space_id = event_id_str(&space_ev);
    rt.ingest_event(space_ev);

    let room_ev = sign_event(build_room_create_event(&alice, &space_id, "general", None), &alice);
    let room_id = event_id_str(&room_ev);
    rt.ingest_event(room_ev);

    (Arc::new(Mutex::new(rt)), space_id, room_id, alice)
}

async fn subscribe_and_ack(
    space_id: &str,
) -> (
    tokio::task::JoinHandle<()>,
    BufReader<tokio::io::ReadHalf<tokio::io::DuplexStream>>,
    tokio::io::WriteHalf<tokio::io::DuplexStream>,
) {
    let (client, server_half) = tokio::io::duplex(8192);
    let handler = tokio::spawn(async move { handle_events_connection(server_half).await });

    let (cr, mut cw) = tokio::io::split(client);
    let mut creader = BufReader::new(cr);

    let sub = format!(
        "{{\"cmd\":\"subscribe\",\"args\":{{\"spaces\":[\"{space_id}\"],\"event_types\":[\"message.text\"]}}}}\n"
    );
    cw.write_all(sub.as_bytes()).await.unwrap();

    // The node handler registers the observer BEFORE writing the ack, so once we
    // read the ack the observer is live and apply_fanout will reach it.
    let mut ack = String::new();
    timeout(T, creader.read_line(&mut ack))
        .await
        .expect("ack within timeout")
        .unwrap();
    assert!(ack.contains("\"status\":\"ok\""), "expected subscribe ack, got: {ack}");

    (handler, creader, cw)
}

#[tokio::test]
#[serial_test::serial(node_observers)]
async fn node_observer_receives_apply_fanout_event_via_pipe() {
    let (runtime, space_id, room_id, alice) = setup_creator_space();
    let author = IdentityXgid::from_xgid(Xgid::new(pubkey_uri(&alice)));
    let senders: ClientSenders = Arc::new(Mutex::new(HashMap::new()));

    let (handler, mut creader, cw) = subscribe_and_ack(&space_id).await;

    let tip = runtime
        .lock()
        .await
        .dag_tips(&SpaceXgid::from_xgid(Xgid::new(space_id.clone())))[0]
        .clone();
    let msg = sign_event(
        build_message_text_event(&alice, &space_id, &room_id, vec![tip], "watch me"),
        &alice,
    );
    apply_fanout(
        FanoutRequest { event: Some(msg.clone()), new_joiner: None },
        &author,
        &runtime,
        &senders,
    )
    .await;

    let mut line = String::new();
    timeout(T, creader.read_line(&mut line))
        .await
        .expect("event within timeout")
        .unwrap();
    let got: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
    assert_eq!(got["type"], "message.text");
    assert_eq!(got["event_id"].as_str().unwrap(), event_id_str(&msg));

    drop(cw);
    drop(creader);
    let _ = timeout(T, handler).await;
}

#[tokio::test]
#[serial_test::serial(node_observers)]
async fn node_observer_filters_out_nonmatching_event_via_pipe() {
    let (runtime, space_id, room_id, alice) = setup_creator_space();
    let author = IdentityXgid::from_xgid(Xgid::new(pubkey_uri(&alice)));
    let senders: ClientSenders = Arc::new(Mutex::new(HashMap::new()));

    let (handler, mut creader, cw) = subscribe_and_ack(&space_id).await;

    // A state.room_create (wrong type) is fanned first: apply_fanout's observer
    // filter excludes it (matches() == false → no try_send), so it never reaches
    // the pipe. Then a message.text matches. The first (and only) JSONL line must
    // be the message.text — proving the room_create was filtered before the pipe.
    let room2 = sign_event(build_room_create_event(&alice, &space_id, "general2", None), &alice);
    apply_fanout(
        FanoutRequest { event: Some(room2), new_joiner: None },
        &author,
        &runtime,
        &senders,
    )
    .await;

    let tip = runtime
        .lock()
        .await
        .dag_tips(&SpaceXgid::from_xgid(Xgid::new(space_id.clone())))[0]
        .clone();
    let msg = sign_event(
        build_message_text_event(&alice, &space_id, &room_id, vec![tip], "watch me"),
        &alice,
    );
    apply_fanout(
        FanoutRequest { event: Some(msg.clone()), new_joiner: None },
        &author,
        &runtime,
        &senders,
    )
    .await;

    let mut line = String::new();
    timeout(T, creader.read_line(&mut line))
        .await
        .expect("event within timeout")
        .unwrap();
    let got: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
    assert_eq!(
        got["type"], "message.text",
        "only the filter-matching event should reach the pipe; the room_create must be dropped"
    );

    drop(cw);
    drop(creader);
    let _ = timeout(T, handler).await;
}
