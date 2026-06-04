// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Arc H (PG-05, AH-D5) — the content-blindness proof.
//!
//! The arc's headline deliverable test (the analogue of Arc F's two-node e2e):
//! a real in-process Node hosts an E2E Space, a member sends one encrypted
//! `message.text` through the **live validation/ingest path** (`submit_locally`
//! → `dispatch_event` → 13-step validation → store), and we assert the Node
//! cannot recover the plaintext while a member (holding the epoch key) can.
//!
//! Five assertions (AH-D5; assertion 5 = the AH-D1 erasability invariant, one
//! invariant stated once):
//!
//! 1. (a) The Node stores the `enc:` blob **byte-identical** — `message.*`
//!    content is never parsed (`validate_event` is content-blind), and the DS
//!    blind-store primitive `handle_encrypted_content` is a pure pass-through.
//! 2. (b) The plaintext **never appears** in the Node-visible event. Note the
//!    guarantee is actually *stronger* than the design's "`enc:` blind-
//!    substitution": `event_trace` (`trace_event`/`trace_local`) omits the
//!    `content` field **entirely**, for every event type — there is no
//!    content-substitution branch to bypass. We assert the structural form: the
//!    plaintext marker is absent from the Node's stored event.
//! 3. (c) DS content routing is opaque — `handle_encrypted_content` pass-through
//!    and `is_encrypted_content` detection on the live stored content. Opaque
//!    routing of `mls.welcome`/`mls.commit` control messages rides C2.
//! 4. (d) A Node-side read of the content field yields **only ciphertext**; a
//!    member unwraps `CK` under the epoch key and decrypts to the original
//!    plaintext. The Node holds no epoch key, so it has no path to either.
//! 5. (e) **Erasability invariant (threat-defended):** destroying the wrapped
//!    `CK` leaves content unrecoverable *even by a party who still holds the
//!    epoch secret* — proved by shredding the wrapped `CK` and failing to
//!    decrypt with the correct epoch key. The Node, holding no epoch secret at
//!    all, is strictly weaker than that adversary.
//!
//! **Fence — content-blind ≠ blind (do not over-claim).** This is a
//! *content*-blindness proof and nothing more. It does **NOT** claim metadata or
//! traffic-analysis blindness: who-talked-to-whom, when, message volume/timing,
//! and which-Space all remain fully legible to the Node (Layer-1 metadata,
//! required for routing / signature validation / audit) — and are frequently the
//! *more* sensitive signal. Metadata-blindness is far beyond Arc H and is not
//! delivered and not claimed.
//!
//! **Honest C1 boundary (D-065).** The production client `ops::send` does not yet
//! hold a per-Room `ClientMlsGroup`, so it cannot live-encrypt until the
//! KeyPackage/Welcome/epoch lifecycle lands (C2). This proof constructs the
//! member group in-process — exactly the "in-process" form AH-D5 specifies — and
//! drives the real Node ingest, which is what makes the server-blindness
//! guarantee true and checkable on the wire at C1.

#![cfg(test)]

#[cfg(test)]
mod tests {
    use crate::tests::phase9_harness::{
        event_id_str, pubkey_uri, rdx, spawn_in_process_node,
    };
    use crate::{
        identity::keypair,
        message::exchange::build_message_text_event,
        node::runtime::DispatchOutcome,
        space::state::{
            build_mls_group_init_event, build_room_create_event, build_space_create_event,
            sign_event,
        },
    };
    use xgen_core::encryption::client_mls::{
        decrypt_message_envelope, encrypt_message_envelope, envelope_with_destroyed_key,
        ClientMlsGroup, EncryptedContent,
    };
    use xgen_core::encryption::delivery_service::{handle_encrypted_content, is_encrypted_content};

    // A distinctive plaintext marker — its absence from the Node-visible event is
    // the content-blindness witness (it only ever exists inside the ciphertext).
    const PLAINTEXT: &[u8] = br#"{"text":"SECRET-PLAINTEXT-MARKER-9f3a2b"}"#;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn node_hosting_e2e_space_cannot_recover_message_plaintext() {
        let node = spawn_in_process_node().await;
        let alice = keypair::generate();
        let alice_id = pubkey_uri(&alice);
        node.register_identity(&alice).await;

        // ── E2E Space (AH-D2) ────────────────────────────────────────────────
        let space_ev = sign_event(
            build_space_create_event(&alice, "Secure Space", None, 1, &node.node_id, None, true),
            &alice,
        );
        let space_id = event_id_str(&space_ev);
        node.ingest(space_ev).await;
        assert!(
            node.space_state(&space_id).await.unwrap().e2e_encryption,
            "the Node knows the Space is E2E"
        );

        // ── Room (Alice the owner is its member) ─────────────────────────────
        let room_ev =
            sign_event(build_room_create_event(&alice, &space_id, "general", None), &alice);
        let room_id = event_id_str(&room_ev);
        node.ingest(room_ev).await;

        // ── state.mls_group_init: Node-readable genesis anchor (AH-D3) ───────
        let init_ev = sign_event(
            build_mls_group_init_event(&alice, &space_id, &room_id, &room_id),
            &alice,
        );
        node.ingest(init_ev).await;
        assert_eq!(
            node.space_state(&space_id)
                .await
                .unwrap()
                .rooms
                .get(&rdx(&room_id))
                .unwrap()
                .mls_epoch,
            Some(0),
            "the Node tracks the genesis epoch (an opaque counter — no key material)"
        );

        // ── Alice's client MLS group + per-message envelope encrypt (AH-D1) ──
        // The epoch secret lives only with the member; the Node never holds it.
        let alice_group = ClientMlsGroup::new(&room_id, &alice_id, [7u8; 32]);
        let epoch_key = alice_group.current_epoch_key();
        let enc = encrypt_message_envelope(&epoch_key, alice_group.epoch, PLAINTEXT);
        let enc_blob = enc.0.clone();
        assert!(enc_blob.starts_with("enc:"));

        // The `enc:` envelope rides in `content["text"]` (the built convention;
        // is_encrypted_content / event_trace key off the `enc:` prefix).
        let tips = node.dag_tips(&space_id).await;
        let msg_ev = sign_event(
            build_message_text_event(&alice, &space_id, &room_id, tips, &enc_blob),
            &alice,
        );
        let msg_id = event_id_str(&msg_ev);

        // ── Live path: the Node validates the SIGNED ENVELOPE without the ───
        // plaintext (signature separation, §3.10.7) and stores it.
        let outcome = node.submit_locally(msg_ev).await;
        assert!(
            matches!(outcome, DispatchOutcome::Accepted { .. }),
            "the Node accepts the encrypted message over its signed envelope"
        );
        assert!(node.has_event(&space_id, &msg_id).await);

        // ── (a) byte-identical opaque store + DS pass-through ────────────────
        let stored = node.stored_event(&space_id, &msg_id).await.unwrap();
        let stored_text = stored.content["text"].as_str().unwrap();
        assert_eq!(stored_text, enc_blob, "the Node stored the enc: blob unchanged");
        assert_eq!(
            handle_encrypted_content(stored_text.as_bytes().to_vec()).unwrap(),
            stored_text.as_bytes(),
            "handle_encrypted_content is a pure opaque pass-through (DS blind-store)"
        );
        assert!(is_encrypted_content(stored_text), "the Node detects encrypted content");

        // ── (b) the plaintext is absent from the entire Node-visible event ───
        // event_trace omits content for every event type, so there is no trace
        // path to plaintext either; the stored event is the strongest surface.
        let stored_json = serde_json::to_string(&stored).unwrap();
        assert!(
            !stored_json.contains("SECRET-PLAINTEXT-MARKER-9f3a2b"),
            "plaintext must never appear in the Node-stored event"
        );

        // ── (d) Node read = ciphertext only; the member recovers plaintext ───
        let recovered =
            decrypt_message_envelope(&epoch_key, &EncryptedContent(stored_text.to_string()))
                .unwrap();
        assert_eq!(recovered, PLAINTEXT, "a member holding the epoch key reads the plaintext");

        // ── (e) erasability invariant — defeats even an epoch-secret holder ──
        let shredded =
            envelope_with_destroyed_key(&EncryptedContent(stored_text.to_string())).unwrap();
        assert!(
            decrypt_message_envelope(&epoch_key, &shredded).is_err(),
            "after the wrapped CK is destroyed, even a holder of the epoch secret \
             cannot recover the content (AH-D1 / AH-D5 assertion 5)"
        );

        node.shutdown().await;
    }
}
