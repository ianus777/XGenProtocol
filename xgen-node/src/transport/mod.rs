// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

// Transport module — WebSocket connections, authentication, keepalive (spec 3.3).
// server.rs is Node-specific and lives here; auth/client/connection live in xgen-core.

pub mod server;

// Re-export shared transport from xgen-core so that crate::transport::{auth, client, connection}
// continues to work throughout xgen-node (main.rs, tests, etc.).
pub use xgen_core::transport::auth;
pub use xgen_core::transport::client;
pub use xgen_core::transport::connection;

#[cfg(test)]
mod tests {
    use super::{client, server::Server};
    use crate::identity::keypair;

    /// Full connection lifecycle test:
    ///   1. Bind a server on a random port.
    ///   2. Spawn a server task that accepts one connection and authenticates it.
    ///   3. Connect a client and authenticate.
    ///   4. Client sends a WebSocket ping; server receives it.
    ///   5. Client sends transport.goodbye; server sees Closed.
    #[tokio::test]
    async fn connect_authenticate_ping_goodbye() {
        let mut server = Server::bind("127.0.0.1:0".parse().unwrap())
            .await
            .unwrap();
        let server_addr = server.local_addr();

        // Generate client Identity keypair.
        let client_key = keypair::generate();
        let expected_id = super::auth::identity_id_from_key(&client_key.verifying_key());

        // Server task.
        let server_task = tokio::spawn(async move {
            let mut conn = server.accept().await.unwrap();
            let identity_id = conn.server_authenticate("xgen://pubkey/ed25519:TESTNODE").await.unwrap();
            (identity_id, conn)
        });

        // Client connects and authenticates.
        let mut client_conn = client::connect(server_addr).await.unwrap();
        let returned_id = client_conn.client_authenticate(&client_key).await.unwrap().identity_id;

        // Server task completes.
        let (server_identity_id, mut server_conn) = server_task.await.unwrap();

        // Both sides agree on the identity.
        assert_eq!(server_identity_id, expected_id);
        // `AuthOutcome.identity_id` is now `IdentityXgid` (M-RP-XGID-SLOT-RETYPE
        // Leg C); compare against the `String` expected id via `.as_str()`.
        assert_eq!(returned_id.as_str(), expected_id);

        // Client sends a WebSocket ping; server should receive it as Inbound::Ping.
        client_conn.ping().await.unwrap();
        match server_conn.recv().await.unwrap() {
            crate::transport::connection::Inbound::Ping(_) => {}
            other => panic!("expected Ping, got {other:?}"),
        }

        // Client sends goodbye; server side sees the connection close.
        client_conn.goodbye("client_disconnect").await.unwrap();
        match server_conn.recv().await.unwrap() {
            crate::transport::connection::Inbound::Transport(
                crate::wire::types::TransportMessage::Goodbye { reason, .. },
            ) => {
                assert_eq!(reason, "client_disconnect");
            }
            other => panic!("expected Goodbye, got {other:?}"),
        }
    }

    /// M10.4 C1 witness — `AuthOk` echoes the Node's `node_id` (Shape B, MP-F13).
    /// RED on revert of the populate in `server_authenticate`: `node_id` → None.
    #[tokio::test]
    async fn server_authenticate_echoes_node_id() {
        use crate::wire::types::TransportMessage;
        use crate::transport::connection::Inbound;

        let mut server = Server::bind("127.0.0.1:0".parse().unwrap())
            .await
            .unwrap();
        let addr = server.local_addr();
        let client_key = keypair::generate();
        const NODE_ID: &str = "xgen://pubkey/ed25519:WITNESSNODE";

        let server_task = tokio::spawn(async move {
            let mut conn = server.accept().await.unwrap();
            conn.server_authenticate(NODE_ID).await.unwrap();
        });

        // Manual client handshake so we can read the raw AuthOk.
        let mut conn = client::connect(addr).await.unwrap();
        let nonce = match conn.recv().await.unwrap() {
            Inbound::Transport(TransportMessage::Challenge { nonce, .. }) => nonce,
            other => panic!("expected Challenge, got {other:?}"),
        };
        let auth = super::auth::build_auth_response(&nonce, &client_key);
        conn.send_transport(&auth).await.unwrap();
        match conn.recv().await.unwrap() {
            Inbound::Transport(TransportMessage::AuthOk { node_id, .. }) => {
                assert_eq!(node_id.as_deref(), Some(NODE_ID));
            }
            other => panic!("expected AuthOk, got {other:?}"),
        }
        server_task.await.unwrap();
    }

    /// Authentication with a bad signature must return AuthFailed on the client.
    #[tokio::test]
    async fn bad_signature_rejected() {
        use crate::wire::types::TransportMessage;
        use crate::transport::connection::Inbound;

        let mut server = Server::bind("127.0.0.1:0".parse().unwrap())
            .await
            .unwrap();
        let addr = server.local_addr();

        // Server task: run auth, expect failure.
        let server_task = tokio::spawn(async move {
            let mut conn = server.accept().await.unwrap();
            // server_authenticate will send auth_fail and return Err.
            conn.server_authenticate("xgen://pubkey/ed25519:TESTNODE").await.is_err()
        });

        // Client: tamper with the signature.
        let mut conn = client::connect(addr).await.unwrap();

        // Receive the challenge.
        let nonce = match conn.recv().await.unwrap() {
            Inbound::Transport(TransportMessage::Challenge { nonce, .. }) => nonce,
            other => panic!("expected Challenge, got {other:?}"),
        };

        // Send a forged auth with a wrong key.
        let wrong_key = keypair::generate();
        let real_key = keypair::generate();
        // Sign with wrong_key but claim real_key's identity.
        use crate::crypto::{encoding, signing};
        let nonce_bytes = encoding::decode(&nonce).unwrap();
        let sig = signing::sign(&wrong_key, &nonce_bytes);
        let pubkey_b64 = encoding::encode(real_key.verifying_key().as_bytes());
        let forged_auth = TransportMessage::Auth {
            protocol_version: "0.1".to_string(),
            identity_id: format!("xgen://pubkey/ed25519:{}", pubkey_b64),
            nonce,
            signature: sig,
        };
        conn.send_transport(&forged_auth).await.unwrap();

        // Client should receive auth_fail.
        match conn.recv().await.unwrap() {
            Inbound::Transport(TransportMessage::AuthFail { error_code, .. }) => {
                assert_eq!(error_code, 1001); // auth_signature_invalid
            }
            other => panic!("expected AuthFail, got {other:?}"),
        }

        let server_got_err = server_task.await.unwrap();
        assert!(server_got_err);
    }

    /// An Event can be sent from client to server after authentication.
    #[tokio::test]
    async fn event_exchange_after_auth() {
        use crate::wire::types::{Event, EventType};
        use crate::transport::connection::Inbound;
        use serde_json::json;
        use xgen_common::xgid::{EventXgid, IdentityXgid, RoomXgid, SpaceXgid, Xgid};

        let mut server = Server::bind("127.0.0.1:0".parse().unwrap())
            .await
            .unwrap();
        let addr = server.local_addr();
        let client_key = keypair::generate();

        let server_task = tokio::spawn(async move {
            let mut conn = server.accept().await.unwrap();
            conn.server_authenticate("xgen://pubkey/ed25519:TESTNODE").await.unwrap();
            // Receive one event.
            match conn.recv().await.unwrap() {
                Inbound::Event(ev) => ev,
                other => panic!("expected Event, got {other:?}"),
            }
        });

        let mut conn = client::connect(addr).await.unwrap();
        conn.client_authenticate(&client_key).await.unwrap();

        let mut ev = Event::new(
            EventType::MessageText,
            IdentityXgid::from_xgid(Xgid::new("xgen://pubkey/ed25519:sender".to_string())),
            RoomXgid::from_xgid(Xgid::new("xgen://hash/sha256:room".to_string())),
            SpaceXgid::from_xgid(Xgid::new("xgen://hash/sha256:space".to_string())),
            vec![],
            "2026-04-27T12:00:00Z".to_string(),
            json!({"text": "hello"}),
        );
        ev.event_id = Some(EventXgid::from_xgid(Xgid::new("xgen://hash/sha256:evt001".to_string())));

        conn.send_event(&ev).await.unwrap();

        let received = server_task.await.unwrap();
        assert_eq!(
            received.event_id.as_ref().map(|x| x.as_str()),
            Some("xgen://hash/sha256:evt001")
        );
    }
}
