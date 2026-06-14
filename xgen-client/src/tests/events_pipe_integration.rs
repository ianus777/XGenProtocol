// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! M7-events `.events` integration test — **client seam** (EIT-D1, C1).
//!
//! Closes the C5 component-test boundary flagged at J-211: the client
//! `handle_events_connection` happy path — `subscribe` → open a second
//! same-identity WS → `client_authenticate` → tail → `forwardable` → JSONL —
//! was never exercised end-to-end because `connect_url` is concrete.
//!
//! Approach (EIT-D1/D3, design §0): drive the real handler over an in-memory
//! `tokio::io::duplex` (the pipe side), while the WS side is a **real** local
//! stub server built from `xgen-core::transport` primitives (the
//! `sync_safety_net` pattern, extended to emit events after auth). No real
//! Node and **no Space membership** are needed: the client forwards whatever
//! `Inbound::Event` arrives, filtered only by its own subscribe `Filter`;
//! entitlement/membership filtering is the Node's job (node-seam test, C2).

use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::time::timeout;

use xgen_common::state::ClientState;
use xgen_common::wire::{Event, EventType};
use xgen_common::xgid::{EventXgid, IdentityXgid, RoomXgid, SpaceXgid, Xgid};
use xgen_core::identity::keypair;
use xgen_core::transport::connection::Connection;

use crate::events_pipe::handle_events_connection;

const T: Duration = Duration::from_secs(5);

/// A minimal real WS server: bind ephemeral, accept one connection, complete
/// the server-side auth handshake, send `events` in order, then drain until the
/// peer closes. Mirrors `tests/sync_safety_net.rs::spawn_silent_server` but
/// emits events instead of going silent.
async fn spawn_event_server(events: Vec<Event>) -> (std::net::SocketAddr, tokio::task::JoinHandle<()>) {
    let (tx, rx) = tokio::sync::oneshot::channel::<std::net::SocketAddr>();
    let handle = tokio::spawn(async move {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let _ = tx.send(addr);
        let (stream, _) = listener.accept().await.unwrap();
        let ws = tokio_tungstenite::accept_async(stream).await.unwrap();
        let mut conn = Connection::new(ws);
        if conn.server_authenticate("xgen://pubkey/ed25519:TESTNODE").await.is_err() {
            return;
        }
        for ev in &events {
            if conn.send_event(ev).await.is_err() {
                return;
            }
        }
        // Keep the connection open so the client can read; drain until closed.
        loop {
            if conn.recv().await.is_err() {
                break;
            }
        }
    });
    let addr = rx.await.expect("server reported its address");
    (addr, handle)
}

/// Write the on-disk fixture the client handler reads: an encrypted keypair
/// (empty passphrase, matching `app::load_keypair`) + a `client_state.json`
/// whose `home_node` points at the stub server.
fn setup_client_dir(home_node: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let key = keypair::generate();
    keypair::save(&key, &dir.path().join("xgen-client_keypair.enc"), "")
        .expect("save keypair fixture");
    let state = ClientState {
        identity_id: "xgen://pubkey/ed25519:test".into(),
        display_name: "tester".into(),
        version: "0.0.0".into(),
        build: "test".into(),
        home_node: home_node.into(),
        updated_at: "2026-06-02T00:00:00.000Z".into(),
        spaces: vec![],
        last_local_events: Default::default(),
    };
    std::fs::write(
        dir.path().join("xgen-client_state.json"),
        serde_json::to_string_pretty(&state).unwrap(),
    )
    .unwrap();
    dir
}

fn ev(ty: EventType) -> Event {
    Event {
        protocol_version: "0.1".into(),
        event_type: ty,
        event_id: Some(EventXgid::from_xgid(Xgid::new("xgen://hash/sha256:EV".into()))),
        sender: IdentityXgid::from_xgid(Xgid::new("xgen://pubkey/ed25519:A".into())),
        room_id: RoomXgid::from_xgid(Xgid::new(String::new())),
        space_id: SpaceXgid::from_xgid(Xgid::new("xgen://hash/sha256:S".into())),
        prev_events: vec![],
        timestamp: "2026-06-01T00:00:00.000Z".into(),
        content: serde_json::json!({}),
        meta_atts: None,
        signature: Some("ed25519:S:S".into()),
    }
}

/// Drive the handler, returning the connected client-side reader/writer halves
/// after the subscribe has been acked.
async fn subscribe_and_ack(
    home_node: &str,
    subscribe_args: &str,
) -> (
    tempfile::TempDir,
    tokio::task::JoinHandle<()>,
    BufReader<tokio::io::ReadHalf<tokio::io::DuplexStream>>,
    tokio::io::WriteHalf<tokio::io::DuplexStream>,
) {
    let dir = setup_client_dir(home_node);
    let (client, server_half) = tokio::io::duplex(8192);
    let d = dir.path().to_path_buf();
    let handler = tokio::spawn(async move { handle_events_connection(server_half, d).await });

    let (cr, mut cw) = tokio::io::split(client);
    let mut creader = BufReader::new(cr);

    let line = format!("{{\"cmd\":\"subscribe\",\"args\":{subscribe_args}}}\n");
    cw.write_all(line.as_bytes()).await.unwrap();

    let mut ack = String::new();
    timeout(T, creader.read_line(&mut ack))
        .await
        .expect("ack within timeout")
        .unwrap();
    assert!(ack.contains("\"status\":\"ok\""), "expected subscribe ack, got: {ack}");

    (dir, handler, creader, cw)
}

#[tokio::test]
#[serial_test::serial(events_sessions)]
async fn client_subscribe_forwards_matching_live_event() {
    let (addr, _server) = spawn_event_server(vec![ev(EventType::MessageText)]).await;
    let url = format!("ws://{addr}/xgen");

    let (_dir, handler, mut creader, cw) =
        subscribe_and_ack(&url, r#"{"event_types":["message.text"]}"#).await;

    // The emitted message.text arrives over the second WS and is forwarded as
    // bare Event JSONL down the pipe.
    let mut line = String::new();
    timeout(T, creader.read_line(&mut line))
        .await
        .expect("event within timeout")
        .unwrap();
    let got: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
    assert_eq!(got["type"], "message.text");
    assert_eq!(got["event_id"], "xgen://hash/sha256:EV");

    drop(cw);
    drop(creader);
    let _ = timeout(T, handler).await;
}

#[tokio::test]
#[serial_test::serial(events_sessions)]
async fn client_filters_out_nonmatching_event() {
    // Server sends a non-matching event (state.room_create) FIRST, then a
    // matching one (message.text). The handler must forward ONLY the match —
    // so the first (and only) JSONL line we read is the message.text, which
    // proves the room_create was filtered out at the drain (`forwardable`).
    let (addr, _server) = spawn_event_server(vec![
        ev(EventType::StateRoomCreate),
        ev(EventType::MessageText),
    ])
    .await;
    let url = format!("ws://{addr}/xgen");

    let (_dir, handler, mut creader, cw) =
        subscribe_and_ack(&url, r#"{"event_types":["message.text"]}"#).await;

    let mut line = String::new();
    timeout(T, creader.read_line(&mut line))
        .await
        .expect("event within timeout")
        .unwrap();
    let got: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
    assert_eq!(
        got["type"], "message.text",
        "only the filter-matching event should be forwarded; the room_create must be dropped"
    );

    drop(cw);
    drop(creader);
    let _ = timeout(T, handler).await;
}
