// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! M8 — Wave 3 / C5 — S6 E2E content-blindness (Arc H PG-05). Runbook
//! `tasks/M8_MULTIPARTY_IMPL.md` §5 C5; design `tasks/M8_MULTIPARTY_DESIGN.md`
//! §3 (S6 row).
//!
//! Extends the single-message content-blindness proof (`arc_h_content_blindness.rs`)
//! to the multiparty surface S6 wants: an E2E Space with N members; the encrypted
//! message is stored **opaque** by the Node (zero plaintext, M3 content-blindness)
//! while any member (epoch-key holder) recovers it; the **KeyPackage pool** is
//! consumed + replenish-flagged on multi-join; the **epoch advances** on
//! `mls.commit` (single-committer happy path — the commit-race is D3-fenced).
//!
//! **Honest live-vs-dormant boundary (Arc H, D-065).** The `enc:` v2 envelope,
//! the Node's content-blind validate/ingest/store path, the KeyPackage store +
//! `record_key_package` ingest hook, and the `mls_group_init`/`mls.commit` epoch
//! appliers are **live**. The member group here is the in-process `ClientMlsGroup`
//! (fixed seed) exactly as AH-D5 specifies — production live-encrypt (`ops::send`
//! holding a per-Room group), Welcome/Commit DS routing, and the replenish-request
//! round-trip ride the eventual production MLS client (D3, interface-locked). M8
//! exercises the live surface; it builds none of the dormant production client.

#![cfg(test)]

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::tests::phase9_harness::{
        event_id_str, pubkey_uri, rdx, spawn_in_process_node,
    };
    use crate::{
        identity::keypair,
        message::exchange::build_message_text_event,
        node::runtime::DispatchOutcome,
        space::state::{
            build_membership_event, build_mls_commit_event, build_mls_group_init_event,
            build_mls_key_package_event, build_room_create_event, build_space_create_event,
            sign_event,
        },
        wire::types::EventType,
    };
    use xgen_core::encryption::client_mls::{
        decrypt_message_envelope, encrypt_message_envelope, ClientMlsGroup, EncryptedContent,
    };
    use xgen_core::encryption::delivery_service::is_encrypted_content;
    use xgen_core::encryption::key_package::MIN_KEY_PACKAGE_POOL;

    const PLAINTEXT: &[u8] = br#"{"text":"S6-SECRET-MARKER-4d7e1a"}"#;

    /// Multiparty content-blindness: an E2E Space with three members; an encrypted
    /// `message.text` is stored **opaque** by the Node (no plaintext in the
    /// Node-visible event) while an epoch-key-holding member recovers the
    /// plaintext. The M3 content-blindness invariant holds in a populated
    /// multiparty Space, not just a single-author one.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn s6_e2e_space_with_n_members_keeps_node_content_blind() {
        let node = spawn_in_process_node().await;
        let alice = keypair::generate();
        let bob = keypair::generate();
        let carol = keypair::generate();
        let alice_id = pubkey_uri(&alice);
        let bob_id = pubkey_uri(&bob);
        let carol_id = pubkey_uri(&carol);
        node.register_identity(&alice).await;
        node.register_identity(&bob).await;
        node.register_identity(&carol).await;

        // E2E-on Space (AH-D2: e2e_encryption = true).
        let create = sign_event(
            build_space_create_event(&alice, "S6-secure", None, 1, &node.node_id, None, true),
            &alice,
        );
        let sid = event_id_str(&create);
        node.ingest(create).await;
        assert!(node.space_state(&sid).await.unwrap().e2e_encryption, "Space is E2E");

        let room = sign_event(build_room_create_event(&alice, &sid, "general", None), &alice);
        let rid = event_id_str(&room);
        node.ingest(room).await;

        // Bring N members in (invite + join bob and carol).
        for (k, id) in [(&bob, &bob_id), (&carol, &carol_id)] {
            let mut inv = build_membership_event(
                &alice,
                &sid,
                "",
                EventType::MembershipInvite,
                json!({ "target_identity": id, "role": "member" }),
            );
            inv.prev_events = node.dag_tips(&sid).await.iter().map(|t| crate::tests::phase9_harness::edx(t)).collect();
            let inv = sign_event(inv, &alice);
            let inv_id = event_id_str(&inv);
            node.ingest(inv).await;
            let mut j = build_membership_event(k, &sid, "", EventType::MembershipJoin, json!({}));
            j.prev_events = vec![crate::tests::phase9_harness::edx(&inv_id)];
            node.ingest(sign_event(j, k)).await;
        }
        assert_eq!(
            node.space_state(&sid).await.unwrap().members.len(),
            3,
            "three members (alice + bob + carol)"
        );

        // Genesis epoch anchor (Node-readable, no key material).
        let init = sign_event(build_mls_group_init_event(&alice, &sid, &rid, &rid), &alice);
        node.ingest(init).await;

        // Member-side envelope encrypt (epoch secret never leaves the member).
        let group = ClientMlsGroup::new(&rid, &alice_id, [7u8; 32]);
        let epoch_key = group.current_epoch_key();
        let enc = encrypt_message_envelope(&epoch_key, group.epoch, PLAINTEXT);
        let enc_blob = enc.0.clone();
        assert!(enc_blob.starts_with("enc:"));

        let tips = node.dag_tips(&sid).await;
        let msg = sign_event(build_message_text_event(&alice, &sid, &rid, tips, &enc_blob), &alice);
        let msg_id = event_id_str(&msg);
        let outcome = node.submit_locally(msg).await;
        assert!(matches!(outcome, DispatchOutcome::Accepted { .. }), "Node accepts the encrypted message");

        // Content-blindness (M3): the Node stores the enc: blob opaque; the
        // plaintext marker appears nowhere in the Node-visible event.
        let stored = node.stored_event(&sid, &msg_id).await.unwrap();
        let stored_text = stored.content["text"].as_str().unwrap();
        assert_eq!(stored_text, enc_blob, "Node stored the enc: blob byte-identical");
        assert!(is_encrypted_content(stored_text), "Node detects encrypted content");
        assert!(
            !serde_json::to_string(&stored).unwrap().contains("S6-SECRET-MARKER-4d7e1a"),
            "plaintext must NOT appear anywhere in the Node-visible event (M3)"
        );

        // A member holding the epoch key recovers the plaintext; the Node cannot.
        let recovered =
            decrypt_message_envelope(&epoch_key, &EncryptedContent(stored_text.to_string())).unwrap();
        assert_eq!(recovered, PLAINTEXT, "an epoch-key-holding member reads the plaintext");

        node.shutdown().await;
    }

    /// KeyPackage pool: seed a member's pool via `mls.key_package` events (the
    /// `record_key_package` ingest hook), consume on (modelled) joins via
    /// `request_key_package`, and observe `needs_replenish` flip true once the pool
    /// drops below `MIN_KEY_PACKAGE_POOL`. Single-use consume (§3.10.3).
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn s6_keypackage_pool_consumed_and_replenish_flagged_on_multijoin() {
        let node = spawn_in_process_node().await;
        let alice = keypair::generate();
        let bob = keypair::generate();
        let bob_id = pubkey_uri(&bob);
        node.register_identity(&alice).await;
        node.register_identity(&bob).await;

        let create = sign_event(
            build_space_create_event(&alice, "S6-kp", None, 1, &node.node_id, None, true),
            &alice,
        );
        let sid = event_id_str(&create);
        node.ingest(create).await;
        let room = sign_event(build_room_create_event(&alice, &sid, "general", None), &alice);
        let rid = event_id_str(&room);
        node.ingest(room).await;

        // Seed 4 key packages for bob/device-1 (pool > MIN, replenish not needed).
        let device = "bob-dev1";
        for n in 0..4 {
            let kp = sign_event(
                build_mls_key_package_event(
                    &bob,
                    &sid,
                    &rid,
                    node.dag_tips(&sid).await,
                    device,
                    &format!("opaque-kp-{n}"),
                    "2099-01-01T00:00:00.000Z",
                ),
                &bob,
            );
            node.ingest(kp).await;
        }
        {
            let rt = node.runtime.lock().await;
            assert_eq!(rt.key_package_store.available_count(&bob_id, device), 4, "pool seeded");
            assert!(!rt.key_package_store.needs_replenish(&bob_id, device), "pool > MIN, no replenish");
        }

        // Two joins consume two packages (single-use) → pool drops to 2 < MIN(3)
        // → replenish flagged.
        {
            let mut rt = node.runtime.lock().await;
            for _ in 0..2 {
                let got = rt.request_key_package(&bob_id, device, "2026-06-05T00:00:00.000Z");
                assert!(got.is_ok(), "a seeded, unexpired package is served");
            }
            assert_eq!(rt.key_package_store.available_count(&bob_id, device), 2, "two consumed");
            assert!(
                rt.key_package_store.needs_replenish(&bob_id, device),
                "pool {} < MIN {} → replenish flagged",
                2,
                MIN_KEY_PACKAGE_POOL
            );
        }

        node.shutdown().await;
    }

    /// Epoch-advance on `mls.commit` (AH-D4, single-committer happy path). Genesis
    /// `mls_group_init` sets epoch 0; successive `mls.commit` events advance
    /// `RoomState.mls_epoch` deterministically (the Node tracks an opaque counter,
    /// no key material). Concurrent commit-race is D3-fenced (not exercised).
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn s6_epoch_advances_on_single_committer_mls_commit() {
        let node = spawn_in_process_node().await;
        let alice = keypair::generate();
        node.register_identity(&alice).await;

        let create = sign_event(
            build_space_create_event(&alice, "S6-epoch", None, 1, &node.node_id, None, true),
            &alice,
        );
        let sid = event_id_str(&create);
        node.ingest(create).await;
        let room = sign_event(build_room_create_event(&alice, &sid, "general", None), &alice);
        let rid = event_id_str(&room);
        node.ingest(room).await;

        let init = sign_event(build_mls_group_init_event(&alice, &sid, &rid, &rid), &alice);
        node.ingest(init).await;
        let epoch_of = |st: &crate::space::state::SpaceState| st.rooms.get(&rdx(&rid)).unwrap().mls_epoch;
        assert_eq!(epoch_of(&node.space_state(&sid).await.unwrap()), Some(0), "genesis epoch 0");

        for target in 1..=2u64 {
            let commit = sign_event(
                build_mls_commit_event(&alice, &sid, &rid, node.dag_tips(&sid).await, target),
                &alice,
            );
            node.ingest(commit).await;
            assert_eq!(
                epoch_of(&node.space_state(&sid).await.unwrap()),
                Some(target),
                "epoch advanced to {target} on mls.commit"
            );
        }

        node.shutdown().await;
    }
}
