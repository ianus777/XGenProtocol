// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

// Bootstrap client send-path integration tests (bootstrap-client arc, C2).
//
// Exercise the real framed exchange (connect_url → send_bootstrap → recv) over
// an in-process stub Bootstrap Node responder bound on 127.0.0.1:0. The stub
// does NOT run transport challenge-response auth (Pin 1 — bootstrap has no
// transport handshake; spec §3.14.3 verifies the message signature only).
//
// Coverage:
//  1. register round-trip — signed RegisterAck verified against the operator
//     bootstrap_id; the returned record carries the ack's directory_url + a TTL.
//  2. ack signed by the WRONG key (but self-declaring the expected id) is
//     rejected (Pin 2 — verification is against the expected key).
//  3. keepalive round-trip — returns a refreshed TTL expiry.
//  4. deregister — fire-and-forget send succeeds (no ack defined).

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use ed25519_dalek::SigningKey;
    use tokio::sync::oneshot;

    use crate::bootstrap_client::{
        deregister_from_bootstrap, keepalive_bootstrap, register_with_bootstrap,
        BootstrapClientError,
    };
    use crate::transport::server::Server;
    use xgen_common::xgid::NodeXgid;
    use xgen_core::bootstrap::registration_store::BootstrapSelfInfo;
    use xgen_core::bootstrap::signing::sign_bootstrap;
    use xgen_core::transport::connection::Inbound;
    use xgen_core::wire::types::BootstrapMessage;

    fn nid(key: &SigningKey) -> NodeXgid {
        NodeXgid::from_pubkey(&key.verifying_key())
    }

    fn self_info() -> BootstrapSelfInfo {
        BootstrapSelfInfo {
            endpoint: "wss://self.example.com/xgen".to_string(),
            region: "EU".to_string(),
            capabilities: vec!["xgen.federation".to_string()],
            auth_tiers_served: vec![2, 3],
        }
    }

    /// Accept ONE connection and reply to whatever bootstrap frame arrives:
    ///
    /// - Register   → RegisterAck signed by `ack_key`, node_id `ack_node_id`
    /// - Keepalive  → KeepaliveAck signed by `ack_key`, node_id `ack_node_id`
    /// - Deregister → no reply (fire-and-forget)
    ///
    /// `ack_node_id` / `ack_key` are parameters so a test can forge a bad ack.
    /// `observed` signals which frame the stub saw (used by the deregister
    /// test, which has no ack to await).
    async fn run_stub(
        mut server: Server,
        ack_key: SigningKey,
        ack_node_id: String,
        observed: oneshot::Sender<&'static str>,
    ) {
        let mut conn = match server.accept().await {
            Ok(c) => c,
            Err(_) => return,
        };
        let inbound = match conn.recv().await {
            Ok(Inbound::Bootstrap(m)) => m,
            _ => return,
        };
        match inbound {
            BootstrapMessage::Register { .. } => {
                let _ = observed.send("register");
                let ack = sign_bootstrap(
                    BootstrapMessage::RegisterAck {
                        protocol_version: "0.1".to_string(),
                        node_id: ack_node_id,
                        directory_url: "https://bootstrap.example.com/xgen-directory".to_string(),
                        timestamp: "2026-05-31T12:00:01.000Z".to_string(),
                        signature: None,
                    },
                    &ack_key,
                );
                let _ = conn.send_bootstrap(&ack).await;
            }
            BootstrapMessage::Keepalive { .. } => {
                let _ = observed.send("keepalive");
                let ack = sign_bootstrap(
                    BootstrapMessage::KeepaliveAck {
                        protocol_version: "0.1".to_string(),
                        node_id: ack_node_id,
                        timestamp: "2026-05-31T12:00:01.000Z".to_string(),
                        signature: None,
                    },
                    &ack_key,
                );
                let _ = conn.send_bootstrap(&ack).await;
            }
            BootstrapMessage::Deregister { .. } => {
                let _ = observed.send("deregister");
                // No deregister_ack in the protocol — just observe.
            }
            _ => {}
        }
        // Hold the connection briefly so the client's recv completes before drop.
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    #[tokio::test]
    async fn register_round_trip_records_directory_url_and_ttl() {
        let bootstrap_key = SigningKey::from_bytes(&[0xB0; 32]);
        let bootstrap_id = nid(&bootstrap_key);
        let self_key = SigningKey::from_bytes(&[0x5E; 32]);
        let self_id = nid(&self_key);

        let server = Server::bind("127.0.0.1:0".parse().unwrap()).await.unwrap();
        let url = format!("ws://{}/", server.local_addr());

        let (tx, _rx) = oneshot::channel();
        let stub = tokio::spawn(run_stub(
            server,
            bootstrap_key.clone(),
            bootstrap_id.as_str().to_string(),
            tx,
        ));

        let reg = register_with_bootstrap(&url, &bootstrap_id, &self_id, &self_info(), &self_key)
            .await
            .expect("register should succeed");

        assert_eq!(reg.bootstrap_id, bootstrap_id);
        assert_eq!(reg.url, url);
        assert_eq!(reg.directory_url, "https://bootstrap.example.com/xgen-directory");
        assert!(reg.expires_at.is_some(), "TTL expiry recorded");
        assert!(!reg.registered_at.is_empty());

        let _ = stub.await;
    }

    #[tokio::test]
    async fn register_ack_signed_by_wrong_key_is_rejected() {
        // Pin 2: the ack self-declares the expected bootstrap_id but is signed
        // by an attacker key — verification against the expected key fails.
        let bootstrap_key = SigningKey::from_bytes(&[0xB0; 32]);
        let bootstrap_id = nid(&bootstrap_key);
        let attacker_key = SigningKey::from_bytes(&[0xAA; 32]);
        let self_key = SigningKey::from_bytes(&[0x5E; 32]);
        let self_id = nid(&self_key);

        let server = Server::bind("127.0.0.1:0".parse().unwrap()).await.unwrap();
        let url = format!("ws://{}/", server.local_addr());

        let (tx, _rx) = oneshot::channel();
        // Stub signs the ack with the attacker key but still claims bootstrap_id.
        let stub = tokio::spawn(run_stub(
            server,
            attacker_key,
            bootstrap_id.as_str().to_string(),
            tx,
        ));

        let err = register_with_bootstrap(&url, &bootstrap_id, &self_id, &self_info(), &self_key)
            .await
            .expect_err("forged ack must be rejected");
        assert!(matches!(err, BootstrapClientError::AckVerify(_)), "got {err:?}");

        let _ = stub.await;
    }

    #[tokio::test]
    async fn keepalive_round_trip_returns_refreshed_ttl() {
        let bootstrap_key = SigningKey::from_bytes(&[0xB0; 32]);
        let bootstrap_id = nid(&bootstrap_key);
        let self_key = SigningKey::from_bytes(&[0x5E; 32]);
        let self_id = nid(&self_key);

        let server = Server::bind("127.0.0.1:0".parse().unwrap()).await.unwrap();
        let url = format!("ws://{}/", server.local_addr());

        let (tx, _rx) = oneshot::channel();
        let stub = tokio::spawn(run_stub(
            server,
            bootstrap_key.clone(),
            bootstrap_id.as_str().to_string(),
            tx,
        ));

        let expires_at = keepalive_bootstrap(&url, &bootstrap_id, &self_id, &self_key)
            .await
            .expect("keepalive should succeed");
        assert!(!expires_at.is_empty(), "refreshed TTL returned");

        let _ = stub.await;
    }

    #[tokio::test]
    async fn deregister_send_succeeds_without_ack() {
        let bootstrap_key = SigningKey::from_bytes(&[0xB0; 32]);
        let bootstrap_id = nid(&bootstrap_key);
        let self_key = SigningKey::from_bytes(&[0x5E; 32]);
        let self_id = nid(&self_key);

        let server = Server::bind("127.0.0.1:0".parse().unwrap()).await.unwrap();
        let url = format!("ws://{}/", server.local_addr());

        let (tx, rx) = oneshot::channel();
        let stub = tokio::spawn(run_stub(
            server,
            bootstrap_key,
            bootstrap_id.as_str().to_string(),
            tx,
        ));

        deregister_from_bootstrap(&url, &self_id, &self_key)
            .await
            .expect("deregister send should succeed");

        // The stub observed a deregister frame (no ack is sent for it).
        let observed = tokio::time::timeout(Duration::from_secs(2), rx)
            .await
            .expect("stub should observe the frame")
            .expect("oneshot delivered");
        assert_eq!(observed, "deregister");

        let _ = stub.await;
    }
}
