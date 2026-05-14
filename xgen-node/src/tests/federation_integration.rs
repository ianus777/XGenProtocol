// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

// Federation integration tests — require both Server (xgen-node) and xgen-core handshake logic.

#[cfg(test)]
mod tests {
    use crate::{
        federation::handshake::{run_initiating, run_receiving},
        identity::keypair,
        transport::{client, server::Server},
        wire::types::FederationCapabilities,
    };

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
            None,
        )
        .await
        .unwrap();

        let server_session = server_task.await.unwrap();

        assert_eq!(client_session.session_id, server_session.session_id);
        assert_ne!(client_session.peer_node_id, server_session.peer_node_id);
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
            None,
        )
        .await
        .unwrap();

        let server_session = server_task.await.unwrap();

        assert_eq!(client_session.shared_spaces, spaces_clone);
        assert_eq!(server_session.shared_spaces, spaces_clone);
    }
}
