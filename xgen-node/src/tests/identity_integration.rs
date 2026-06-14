// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

// Identity integration tests — require both Server (xgen-node) and xgen-core identity logic.

#[cfg(test)]
mod tests {
    use chrono::{SecondsFormat, Utc};

    use crate::{
        identity::{
            keypair,
            registry::IdentityRegistry,
            registration::{
                accept_registration, build_register, identity_id_from_key, sign_register,
                AssertionPolicy,
            },
        },
        transport::{client, connection::Inbound, server::Server},
        wire::types::IdentityMessage,
    };

    /// Full registration flow over transport:
    ///   1. Server binds and authenticates the client.
    ///   2. Client sends a signed identity.register (Local Node mode — no trust assertion).
    ///   3. Server runs acceptance pipeline.
    ///   4. Server sends identity.register_ok.
    ///   5. Client receives identity.register_ok and confirms identity_id matches.
    #[tokio::test]
    async fn local_node_registration_end_to_end() {
        let mut server = Server::bind("127.0.0.1:0".parse().unwrap())
            .await
            .unwrap();
        let addr = server.local_addr();

        let client_key = keypair::generate();
        let client_id = identity_id_from_key(&client_key);
        let home_node = "xgen://pubkey/ed25519:NODE_PUB";
        let client_id_clone = client_id.clone();

        let server_task = tokio::spawn(async move {
            let mut conn = server.accept().await.unwrap();
            let authenticated_id = conn.server_authenticate("xgen://pubkey/ed25519:TESTNODE").await.unwrap();

            let inbound = conn.recv().await.unwrap();
            let reg_msg = match inbound {
                Inbound::Identity(msg @ IdentityMessage::Register { .. }) => msg,
                other => panic!("expected identity.register, got {other:?}"),
            };

            let mut registry = IdentityRegistry::new();
            let ts = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
            let record = accept_registration(
                &reg_msg,
                &authenticated_id,
                registry.contains(&xgen_common::xgid::IdentityXgid::from_xgid(
                    xgen_common::xgid::Xgid::new(authenticated_id.clone()),
                )),
                true,
                home_node,
                &ts,
                &AssertionPolicy::default(),
            )
            .unwrap();

            registry.register(record).unwrap();

            let ok = IdentityMessage::RegisterOk {
                protocol_version: "0.1".to_string(),
                identity_id: authenticated_id.clone(),
                registered_at: ts,
            };
            conn.send_identity(&ok).await.unwrap();

            authenticated_id
        });

        let mut client_conn = client::connect(addr).await.unwrap();
        client_conn.client_authenticate(&client_key).await.unwrap();

        let reg_msg = sign_register(build_register(&client_key, Some("Alice".to_string())), &client_key);
        client_conn.send_identity(&reg_msg).await.unwrap();

        let inbound = client_conn.recv().await.unwrap();
        match inbound {
            Inbound::Identity(IdentityMessage::RegisterOk { identity_id, .. }) => {
                assert_eq!(identity_id, client_id_clone);
            }
            other => panic!("expected identity.register_ok, got {other:?}"),
        }

        let server_id = server_task.await.unwrap();
        assert_eq!(server_id, client_id_clone);
    }

    /// A second registration attempt from the same identity must be rejected.
    #[tokio::test]
    async fn duplicate_registration_returns_fail() {
        use crate::identity::registration::RegistrationError;

        let mut server = Server::bind("127.0.0.1:0".parse().unwrap())
            .await
            .unwrap();
        let addr = server.local_addr();

        let client_key = keypair::generate();
        let home_node = "xgen://pubkey/ed25519:NODE_PUB";

        let server_task = tokio::spawn(async move {
            let mut conn = server.accept().await.unwrap();
            let authenticated_id = conn.server_authenticate("xgen://pubkey/ed25519:TESTNODE").await.unwrap();
            let mut registry = IdentityRegistry::new();

            let inbound = conn.recv().await.unwrap();
            let reg_msg = match inbound {
                Inbound::Identity(msg @ IdentityMessage::Register { .. }) => msg,
                other => panic!("expected Register, got {other:?}"),
            };
            let ts = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
            let record = accept_registration(
                &reg_msg, &authenticated_id,
                registry.contains(&xgen_common::xgid::IdentityXgid::from_xgid(
                    xgen_common::xgid::Xgid::new(authenticated_id.clone()),
                )),
                true, home_node, &ts, &AssertionPolicy::default(),
            )
            .unwrap();
            registry.register(record).unwrap();
            conn.send_identity(&IdentityMessage::RegisterOk {
                protocol_version: "0.1".to_string(),
                identity_id: authenticated_id.clone(),
                registered_at: ts,
            })
            .await
            .unwrap();

            let inbound2 = conn.recv().await.unwrap();
            let reg_msg2 = match inbound2 {
                Inbound::Identity(msg @ IdentityMessage::Register { .. }) => msg,
                other => panic!("expected Register, got {other:?}"),
            };
            let ts2 = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
            let err = accept_registration(
                &reg_msg2, &authenticated_id,
                registry.contains(&xgen_common::xgid::IdentityXgid::from_xgid(
                    xgen_common::xgid::Xgid::new(authenticated_id.clone()),
                )),
                true, home_node, &ts2, &AssertionPolicy::default(),
            )
            .unwrap_err();

            assert_eq!(err, RegistrationError::AlreadyRegistered);

            let (code, string) = err.to_registration_code();
            conn.send_identity(&IdentityMessage::RegisterFail {
                protocol_version: "0.1".to_string(),
                error_code: code,
                error_string: string.to_string(),
                timestamp: ts2,
            })
            .await
            .unwrap();
        });

        let mut client_conn = client::connect(addr).await.unwrap();
        client_conn.client_authenticate(&client_key).await.unwrap();

        let reg = sign_register(build_register(&client_key, None), &client_key);
        client_conn.send_identity(&reg).await.unwrap();
        match client_conn.recv().await.unwrap() {
            Inbound::Identity(IdentityMessage::RegisterOk { .. }) => {}
            other => panic!("expected RegisterOk, got {other:?}"),
        }

        let reg2 = sign_register(build_register(&client_key, None), &client_key);
        client_conn.send_identity(&reg2).await.unwrap();
        match client_conn.recv().await.unwrap() {
            Inbound::Identity(IdentityMessage::RegisterFail { error_code, .. }) => {
                assert_eq!(error_code, 3007);
            }
            other => panic!("expected RegisterFail, got {other:?}"),
        }

        server_task.await.unwrap();
    }
}
