// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

// Federation module — handshake state machine and relationship registry (spec 3.4).

pub mod handshake;
pub mod registry;

#[cfg(test)]
mod tests {
    use super::{
        handshake::{run_initiating, run_receiving},
        registry::{FederationRegistry, FederationRelationship},
    };
    use crate::wire::types::FederationCapabilities;
    use crate::{identity::keypair, transport::server::Server, transport::client};
    use chrono::{SecondsFormat, Utc};

    /// Full handshake between two in-process Nodes:
    ///   1. Bind a server.
    ///   2. Spawn a server task: accept transport connection, authenticate, run
    ///      receiving side of federation handshake.
    ///   3. Client: connect, authenticate, run initiating side.
    ///   4. Both sides must reach ACTIVE state with matching session_id.
    #[tokio::test]
    async fn full_handshake_reaches_active_both_session_ids_match() {
        let mut server = Server::bind("127.0.0.1:0".parse().unwrap())
            .await
            .unwrap();
        let addr = server.local_addr();

        // Each Node has its own keypair.
        let server_key = keypair::generate();
        let client_key = keypair::generate();

        let server_task = tokio::spawn(async move {
            let mut conn = server.accept().await.unwrap();
            conn.server_authenticate().await.unwrap();
            run_receiving(&mut conn, &server_key, FederationCapabilities::default())
                .await
                .unwrap()
        });

        let mut client_conn = client::connect(addr).await.unwrap();
        client_conn.client_authenticate(&client_key).await.unwrap();

        let client_session = run_initiating(
            &mut client_conn,
            &client_key,
            FederationCapabilities::default(),
            vec![],
        )
        .await
        .unwrap();

        let server_session = server_task.await.unwrap();

        // Both sides must agree on the session_id.
        assert_eq!(client_session.session_id, server_session.session_id);

        // Each side's peer_node_id is the other node's ID — they must be different
        // because they were generated from distinct keypairs.
        assert_ne!(client_session.peer_node_id, server_session.peer_node_id);

        // Negotiated values must match.
        assert_eq!(client_session.negotiated_serialisation, "json");
        assert_eq!(server_session.negotiated_serialisation, "json");
        assert_eq!(client_session.negotiated_version, "0.1");
        assert_eq!(server_session.negotiated_version, "0.1");
    }

    /// shared_spaces declared by the initiating Node are echoed back in the session.
    #[tokio::test]
    async fn shared_spaces_propagate_through_handshake() {
        let mut server = Server::bind("127.0.0.1:0".parse().unwrap())
            .await
            .unwrap();
        let addr = server.local_addr();

        let server_key = keypair::generate();
        let client_key = keypair::generate();
        let spaces = vec![
            "xgen://hash/sha256:space001".to_string(),
            "xgen://hash/sha256:space002".to_string(),
        ];
        let spaces_clone = spaces.clone();

        let server_task = tokio::spawn(async move {
            let mut conn = server.accept().await.unwrap();
            conn.server_authenticate().await.unwrap();
            run_receiving(&mut conn, &server_key, FederationCapabilities::default())
                .await
                .unwrap()
        });

        let mut client_conn = client::connect(addr).await.unwrap();
        client_conn.client_authenticate(&client_key).await.unwrap();

        let client_session = run_initiating(
            &mut client_conn,
            &client_key,
            FederationCapabilities::default(),
            spaces,
        )
        .await
        .unwrap();

        let server_session = server_task.await.unwrap();

        assert_eq!(client_session.shared_spaces, spaces_clone);
        assert_eq!(server_session.shared_spaces, spaces_clone);
    }

    /// Verify that the registry correctly stores and retrieves a session built
    /// from a completed FederationSession.
    #[test]
    fn registry_stores_session_and_round_trips() {
        use super::handshake::FederationSession;

        let session = FederationSession {
            peer_node_id: "xgen://pubkey/ed25519:AAAA".to_string(),
            session_id: "xgen://hash/sha256:sess".to_string(),
            negotiated_serialisation: "json".to_string(),
            negotiated_version: "0.1".to_string(),
            shared_spaces: vec!["xgen://hash/sha256:space1".to_string()],
        };

        let ts = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
        let rel = FederationRelationship::from_session(&session, ts);

        let mut reg = FederationRegistry::new();
        reg.upsert(rel);

        let stored = reg.get("xgen://pubkey/ed25519:AAAA").unwrap();
        assert_eq!(stored.session_id, "xgen://hash/sha256:sess");
        assert_eq!(stored.negotiated_serialisation, "json");
    }
}
