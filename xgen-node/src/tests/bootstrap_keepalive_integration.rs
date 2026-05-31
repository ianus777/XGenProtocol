// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

// Bootstrap keepalive scheduler + re-advertise integration tests (A3, C4).
//
//  1. scheduler tick fires a keepalive for a due registration and pushes its
//     TTL forward on the verified ack (against an in-process stub responder).
//  2. an empty store makes a tick a no-op (prime invariant — no network).
//  3. best-effort re-advertise (A3-D2): a fan-out to a dead URL is swallowed —
//     the local registration state stays intact, the verb-side write stands.

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::Duration;

    use chrono::{SecondsFormat, Utc};
    use ed25519_dalek::SigningKey;
    use tempfile::tempdir;
    use tokio::sync::Mutex;

    use crate::bootstrap_keepalive::{bootstrap_keepalive_tick, readvertise_all};
    use crate::transport::server::Server;
    use xgen_common::xgid::NodeXgid;
    use xgen_core::bootstrap::registration_store::{
        BootstrapRegistration, BootstrapRegistrationStore, BootstrapSelfInfo,
    };
    use xgen_core::bootstrap::signing::sign_bootstrap;
    use xgen_core::transport::connection::Inbound;
    use xgen_core::wire::types::BootstrapMessage;

    fn nid(key: &SigningKey) -> NodeXgid {
        NodeXgid::from_pubkey(&key.verifying_key())
    }

    /// Accept ONE connection and reply to a Keepalive (→ KeepaliveAck) or a
    /// Register (→ RegisterAck), both signed by the bootstrap key.
    async fn run_stub(mut server: Server, ack_key: SigningKey, ack_node_id: String) {
        let mut conn = match server.accept().await {
            Ok(c) => c,
            Err(_) => return,
        };
        match conn.recv().await {
            Ok(Inbound::Bootstrap(BootstrapMessage::Keepalive { .. })) => {
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
            Ok(Inbound::Bootstrap(BootstrapMessage::Register { .. })) => {
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
            _ => {}
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    fn registration(bootstrap_id: &NodeXgid, url: &str, expires_at: Option<String>) -> BootstrapRegistration {
        BootstrapRegistration {
            bootstrap_id: bootstrap_id.clone(),
            url: url.to_string(),
            directory_url: "https://bootstrap.example.com/xgen-directory".to_string(),
            registered_at: "2026-05-31T00:00:00.000Z".to_string(),
            expires_at,
        }
    }

    #[tokio::test]
    async fn tick_keepalives_a_due_registration_and_refreshes_ttl() {
        let bootstrap_key = SigningKey::from_bytes(&[0xB0; 32]);
        let bootstrap_id = nid(&bootstrap_key);
        let self_key = SigningKey::from_bytes(&[0x5E; 32]);
        let self_id = nid(&self_key);

        let server = Server::bind("127.0.0.1:0".parse().unwrap()).await.unwrap();
        let url = format!("ws://{}/", server.local_addr());
        let stub = tokio::spawn(run_stub(server, bootstrap_key, bootstrap_id.as_str().to_string()));

        // A registration already expired → due for keepalive this tick.
        let past = (Utc::now() - chrono::Duration::hours(1)).to_rfc3339_opts(SecondsFormat::Millis, true);
        let dir = tempdir().unwrap();
        let path = dir.path().join("xgen-node_bootstrap.json");
        let store = {
            let mut s = BootstrapRegistrationStore::new();
            s.add(registration(&bootstrap_id, &url, Some(past)));
            Arc::new(Mutex::new(s))
        };

        bootstrap_keepalive_tick(Arc::clone(&store), path.clone(), Arc::new(self_key), self_id).await;

        // The detached attempt pushes expires_at well into the future (now + 7d).
        let mut refreshed = false;
        for _ in 0..40 {
            {
                let s = store.lock().await;
                if let Some(r) = s.get(&bootstrap_id) {
                    if let Some(exp) = &r.expires_at {
                        if chrono::DateTime::parse_from_rfc3339(exp)
                            .map(|e| e.with_timezone(&Utc) > Utc::now() + chrono::Duration::days(6))
                            .unwrap_or(false)
                        {
                            refreshed = true;
                            break;
                        }
                    }
                }
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        assert!(refreshed, "keepalive should refresh the registration TTL ~7 days out");

        let _ = stub.await;
    }

    #[tokio::test]
    async fn empty_store_tick_is_a_noop() {
        // Prime invariant: an empty store makes the tick a no-op (no network).
        let dir = tempdir().unwrap();
        let path = dir.path().join("xgen-node_bootstrap.json");
        let store = Arc::new(Mutex::new(BootstrapRegistrationStore::new()));
        let self_key = SigningKey::from_bytes(&[0x5E; 32]);
        let self_id = nid(&self_key);
        // Returns promptly without touching the network; nothing to assert beyond
        // "does not hang / panic", and the store stays empty.
        bootstrap_keepalive_tick(Arc::clone(&store), path, Arc::new(self_key), self_id).await;
        assert!(store.lock().await.is_empty());
    }

    #[tokio::test]
    async fn readvertise_failure_is_best_effort_local_state_intact() {
        // A3-D2: a registration pointing at a dead URL → re-advertise fails and
        // is swallowed; the registration is retained (local state intact).
        let bootstrap_key = SigningKey::from_bytes(&[0xB0; 32]);
        let bootstrap_id = nid(&bootstrap_key);
        let self_key = SigningKey::from_bytes(&[0x5E; 32]);
        let self_id = nid(&self_key);

        let dir = tempdir().unwrap();
        let path = dir.path().join("xgen-node_bootstrap.json");
        // Port 1 on loopback — nothing listening, connect fails fast.
        let dead_url = "ws://127.0.0.1:1/";
        let store = {
            let mut s = BootstrapRegistrationStore::new();
            s.add(registration(&bootstrap_id, dead_url, None));
            Arc::new(Mutex::new(s))
        };

        let new_info = BootstrapSelfInfo {
            endpoint: "wss://self.example.com/xgen".to_string(),
            region: "EU".to_string(),
            capabilities: vec![],
            auth_tiers_served: vec![],
        };
        // Must complete (best-effort) despite the dead URL.
        readvertise_all(Arc::clone(&store), path, &self_key, &self_id, &new_info).await;

        // The registration is still present (re-advertise failure didn't drop it).
        let s = store.lock().await;
        assert_eq!(s.len(), 1);
        assert!(s.get(&bootstrap_id).is_some());
    }
}
