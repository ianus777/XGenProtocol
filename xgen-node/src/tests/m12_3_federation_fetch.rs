// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! M12.3 C2 — federated fetch-blob-by-hash, in-process spine witness (NOT
//! box-gated). The headline W1/W3 round-trip + the W4/W5 scoping negative,
//! driven through the real two-`NodeRuntime` phase9 federation harness
//! (`federate()` → real `attempt_reconnect` / `handle_federation_incoming` →
//! real `run_federation_session_post_handshake` loops on both sides).
//!
//! The gap (M12.3-A-01): a `message.file` descriptor federates eagerly, but the
//! ciphertext blob it references has no node↔node transfer path; a peer home B
//! that holds the descriptor but not the bytes hits a local miss. M12.3-D1/D4:
//! B lazily fetches the ciphertext from a Space-federated holder over the
//! established federation session (multiplexed, P2 serialize-per-peer), serving
//! the typed `10003` (M12.3-D5) only on a genuine unreachable miss.
//!
//! The spine these witnesses gate is the node-internal
//! [`crate::app::federation_fetch_blob`] + the federation-loop serve/collect/
//! inject arms — driven directly here (the full client↔node leg + real binaries
//! is the box-gated C3 e2e).
//!
//! **RED-on-revert (W1):** neuter the federation serve arm (A stops streaming) or
//! the collect arm (B stops accumulating) in `run_federation_session_post_handshake`
//! → B's `federation_fetch_blob` times out → `Err(())` → the round-trip assertion
//! fails.

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use xgen_core::blob_store::BlobStore;

    use crate::tests::phase9_harness::{
        event_id_str, federate, spawn_in_process_node, InProcessNode,
    };
    use crate::{
        identity::keypair,
        space::state::{build_room_create_event, build_space_create_event, sign_event},
    };

    /// Build alice's Space (homed at `node`) + a room, ingested directly
    /// (pre-federation setup, the `phase9_two_node_smoke` / late-fed pattern).
    /// Returns `(space_id, room_id)`.
    async fn build_space(
        node: &InProcessNode,
        alice_key: &ed25519_dalek::SigningKey,
    ) -> (String, String) {
        let space_ev = sign_event(
            build_space_create_event(alice_key, "m12-3-space", None, 1, &node.node_id, None, false),
            alice_key,
        );
        let space_id = event_id_str(&space_ev);
        node.ingest(space_ev).await;

        let room_ev = sign_event(
            build_room_create_event(alice_key, &space_id, "general", None),
            alice_key,
        );
        let room_id = event_id_str(&room_ev);
        node.ingest(room_ev).await;

        (space_id, room_id)
    }

    /// W1 (headline) + W3 (content-blind across homes): A holds a Space + a
    /// ciphertext blob; the Space federates to B; B has the descriptor's
    /// `blob_ref` but not the bytes → B fetches across homes from A → serves
    /// **byte-identical** ciphertext. The blob is opaque random bytes (never
    /// plaintext) and is multi-chunk (> `BLOB_CHUNK_BYTES`) so the chunked
    /// transfer + P2 reassembly is exercised; W3 = the bytes are ciphertext by
    /// construction (the store + transfer never see plaintext).
    #[serial_test::serial]
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn federated_blob_fetch_round_trip() {
        let node_a = spawn_in_process_node().await;
        let node_b = spawn_in_process_node().await;

        // alice on BOTH nodes (cross-Node identity replication is its own
        // subsystem; the smoke registers the signer on both — see
        // phase9_two_node_smoke — so A's federated Space-create applies on B).
        let alice_key = keypair::generate();
        node_a.register_identity(&alice_key).await;
        node_b.register_identity(&alice_key).await;

        let (space_id, _room_id) = build_space(&node_a, &alice_key).await;

        // A holds a multi-chunk ciphertext blob (opaque; never plaintext — W3).
        // 300 KiB > BLOB_CHUNK_BYTES (128 KiB) → multi-chunk transfer.
        let ciphertext: Vec<u8> = (0..300 * 1024).map(|i| (i * 31 + 7) as u8).collect();
        let blob_ref = {
            let store = BlobStore::new(node_a.spaces_dir.join("blobs"));
            store.put(&ciphertext).expect("A stores the blob")
        };
        // B does NOT have it.
        assert!(
            !BlobStore::new(node_b.spaces_dir.join("blobs")).contains(&blob_ref),
            "precondition: B must not already hold the blob"
        );

        // A initiates → streams A's Space history to B (B builds the Space,
        // home_node = A) and both sessions register each other + stay open.
        federate(&node_a, &node_b, vec![space_id.clone()]).await;
        assert!(
            node_b.wait_for_space(&space_id, Duration::from_secs(10)).await,
            "B did not catch up the federated Space"
        );

        // B fetches the blob across homes (the spine). Inner timeout 5s (P3).
        let fetched = crate::app::federation_fetch_blob(
            &blob_ref,
            &space_id,
            &node_b.runtime,
            &node_b.federation_peer_senders,
            &node_b.pending_fetches,
            Duration::from_secs(5),
        )
        .await;

        assert_eq!(
            fetched,
            Ok(ciphertext),
            "W1: B must fetch the byte-identical ciphertext across homes from A"
        );
    }

    /// W4/W5 scoping negative: a fetch for a Space B does not host / is not
    /// federated for has no Space-federated holder → `federation_fetch_blob`
    /// returns `Err(())` **without querying any peer** (the resolution is scoped
    /// to the Space's federation set; a non-Space home is never asked). Drives
    /// the same property a self-thread blob relies on (empty federation set →
    /// local-only → never federates).
    #[serial_test::serial]
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn federation_fetch_unknown_space_is_unavailable() {
        let node_b = spawn_in_process_node().await;

        // An arbitrary blob_ref + a Space B has never heard of: no SpaceState →
        // no candidate holders → Err without any federation traffic.
        let blob_ref = xgen_core::crypto::hashing::hash_uri(b"never-stored-anywhere");
        let unknown_space = format!("xgen://hash/sha256:{}", "a".repeat(64));

        let result = crate::app::federation_fetch_blob(
            &blob_ref,
            &unknown_space,
            &node_b.runtime,
            &node_b.federation_peer_senders,
            &node_b.pending_fetches,
            Duration::from_secs(2),
        )
        .await;

        assert_eq!(
            result,
            Err(()),
            "W4/W5: an unknown/non-federated Space yields Unavailable, never a cross-home query"
        );
        // And no in-flight fetch slot was left behind.
        assert!(
            node_b.pending_fetches.lock().await.is_empty(),
            "no pending-fetch slot should linger after an unresolved fetch"
        );
    }
}
