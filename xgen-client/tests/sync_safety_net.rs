// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! F-6b safety-net timeout integration test for the Federation Event
//! Propagation completion milestone (Phase 1).
//!
//! Scenario: a server completes the auth handshake but then deliberately
//! ignores the subsequent `transport.sync_request` — never sends
//! `transport.sync_complete`. The client-side `get_dag_tips` must surface
//! a "safety-net timeout" error rather than silently proceed with
//! incomplete data (D-065: honest behaviour over polite behaviour).

use std::time::Duration;

use tokio::net::TcpListener;

use xgen_client_lib::batch::get_dag_tips;
use xgen_core::{
    identity::keypair,
    transport::{client::connect_url, connection::Connection},
};

/// Minimal WS server: bind to an ephemeral port, report the bound address
/// over a oneshot, accept one connection, run the server-side auth
/// handshake, then **block silently forever** without responding to any
/// further inbound message. Exercises the requester's safety-net timeout.
async fn spawn_silent_server() -> (std::net::SocketAddr, tokio::task::JoinHandle<()>) {
    let (tx, rx) = tokio::sync::oneshot::channel::<std::net::SocketAddr>();
    let handle = tokio::spawn(async move {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let _ = tx.send(addr);
        let (stream, _) = listener.accept().await.unwrap();
        let ws = tokio_tungstenite::accept_async(stream).await.unwrap();
        let mut conn = Connection::new(ws);
        // Complete auth handshake so the client can issue sync_request.
        let _ = conn.server_authenticate("xgen://pubkey/ed25519:TESTNODE").await;
        // Drain inbound forever, never reply. The client's sync_request
        // will sit unanswered until the safety-net timeout fires.
        loop {
            if conn.recv().await.is_err() {
                break;
            }
        }
    });
    let addr = rx.await.expect("server reported its address");
    (addr, handle)
}

#[tokio::test]
async fn get_dag_tips_safety_net_timeout_fires_when_peer_silent() {
    let (addr, _server_handle) = spawn_silent_server().await;
    let url = format!("ws://{}/xgen", addr);

    // Client connects + authenticates.
    let key = keypair::generate();
    let mut conn = connect_url(&url).await.expect("client connect");
    conn.client_authenticate(&key)
        .await
        .expect("client auth");

    // get_dag_tips with a short 200 ms safety-net. The server will accept
    // the sync_request but never emit sync_complete — the helper must
    // surface an error citing the timeout, not return Ok(empty).
    let result = get_dag_tips(
        &mut conn,
        "xgen://hash/sha256:nonexistent_space",
        Duration::from_millis(200),
    )
    .await;

    match result {
        Ok(_) => panic!(
            "expected safety-net timeout error; got Ok — F-6b regression \
             (silent proceed instead of honest error)"
        ),
        Err(e) => {
            let msg = format!("{:#}", e);
            assert!(
                msg.contains("safety-net timeout") || msg.contains("sync_complete"),
                "error should cite the safety net / missing sync_complete, got: {}",
                msg
            );
        }
    }
}
