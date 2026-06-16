// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! M12.1 — blob attachment witnesses (W1–W5) against the **real** in-process
//! node (`spawn_in_process_node` → real `handle_connection` blob arms + real
//! `BlobStore` on disk), driven by a **real WS client** `Connection`
//! (`connect_url` + `client_authenticate`).
//!
//! Witness scope (honest boundary, D-065). These exercise the **load-bearing**
//! M12.1 mechanism end-to-end: per-blob encryption (C2) + chunked-base64 WS
//! transfer (C3) + the content-blind content-addressed store (C1), through the
//! real node WS loop + real on-disk `blobs_dir`. The thin `xgen-client` ops glue
//! (`ops::send --attach` / `ops::fetch_attachments`, C4) is component-tested +
//! clap-parse-tested; a full **ops-level / self-thread** round-trip would need a
//! crate that links both `xgen-client` and `xgen-node` (none exists — xgen-mptest
//! drives the *binaries*, and the real-binary fetch path needs the M12.2 fetch
//! CLI verb). The self-DM federation wall itself is M11/D-021 (already witnessed),
//! inherited unchanged — M12.1 adds no federation surface (W5).

use std::time::Duration;

use xgen_core::crypto::hashing::hash_uri;
use xgen_core::encryption::blob::{decrypt_blob, encrypt_blob};
use xgen_core::identity::keypair;
use xgen_core::transport::client::connect_url;
use xgen_core::wire::types::BLOB_CHUNK_BYTES;

use super::phase9_harness::{spawn_in_process_node, spawn_in_process_node_with_max_blob_bytes};

const T: Duration = Duration::from_secs(5);

/// On-disk path of a stored blob under `blobs_dir` (`<sha256-hex>.blob`).
fn blob_path(blobs_dir: &std::path::Path, blob_ref: &str) -> std::path::PathBuf {
    let hex = blob_ref.strip_prefix("xgen://hash/sha256:").unwrap();
    blobs_dir.join(format!("{hex}.blob"))
}

#[tokio::test]
async fn w1_w2_w5_blob_roundtrip_content_blind_no_federation() {
    let node = spawn_in_process_node().await;
    let blobs_dir = node.spaces_dir.join("blobs");

    let key = keypair::generate();
    let mut conn = connect_url(&node.endpoint).await.expect("connect");
    let _ = conn.client_authenticate(&key).await.expect("authenticate");

    // Encrypt a plaintext attachment under a fresh per-blob key (M12-D5).
    let plaintext = b"M12.1 attachment plaintext PLAINTEXT-MARKER payload".to_vec();
    let (blob_key, ciphertext) = encrypt_blob(&plaintext);
    let blob_ref = hash_uri(&ciphertext);

    // Upload the CIPHERTEXT to the node's content-blind store (client 1).
    let confirmed = conn.upload_blob(&blob_ref, &ciphertext, T).await.expect("upload");
    assert_eq!(confirmed, blob_ref, "node confirms the content-address");

    // Fetch it back on a SECOND same-identity client connection (the W1 framing:
    // a second device of the same user pulls the attachment from the home node).
    let mut conn2 = connect_url(&node.endpoint).await.expect("connect (client 2)");
    let _ = conn2.client_authenticate(&key).await.expect("authenticate (client 2)");
    let fetched = conn2.fetch_blob(&blob_ref, T).await.expect("fetch");

    // W1 — round-trip: ciphertext byte-identical, and decrypt → original plaintext.
    assert_eq!(fetched, ciphertext, "W1: ciphertext round-trips byte-identical");
    let decrypted = decrypt_blob(&blob_key, &fetched).expect("decrypt");
    assert_eq!(decrypted, plaintext, "W1: decrypts to the original plaintext");

    // W2 — content-blindness at rest: the file in blobs_dir is the ciphertext;
    // the plaintext (incl. its marker) never appears in the store.
    // RED-on-revert: if the node stored plaintext, on_disk == plaintext.
    let on_disk = std::fs::read(blob_path(&blobs_dir, &blob_ref)).expect("blob on disk");
    assert_eq!(on_disk, ciphertext, "W2: bytes at rest are ciphertext");
    assert!(
        !on_disk.windows(b"PLAINTEXT-MARKER".len()).any(|w| w == b"PLAINTEXT-MARKER"),
        "W2: the plaintext marker never appears in the store"
    );

    // W5 — never-federates: blob transfer is a local TransportMessage, not an
    // Event; it touches no federation surface. The node has no federation peers.
    assert!(
        node.federation_peer_senders.lock().await.is_empty(),
        "W5: blob transfer triggers no federation"
    );

    node.shutdown().await;
}

#[tokio::test]
async fn w4_multi_chunk_blob_roundtrip_exact() {
    // W4 — transfer fidelity: a payload spanning multiple chunks reassembles
    // byte-identical through the real node + store.
    let node = spawn_in_process_node().await;
    let key = keypair::generate();
    let mut conn = connect_url(&node.endpoint).await.expect("connect");
    let _ = conn.client_authenticate(&key).await.expect("authenticate");

    let plaintext = vec![0x5Au8; BLOB_CHUNK_BYTES * 2 + 123]; // > 2 chunks
    let (blob_key, ciphertext) = encrypt_blob(&plaintext);
    assert!(ciphertext.len() > BLOB_CHUNK_BYTES, "genuinely multi-chunk");
    let blob_ref = hash_uri(&ciphertext);

    conn.upload_blob(&blob_ref, &ciphertext, T).await.expect("upload");
    let fetched = conn.fetch_blob(&blob_ref, T).await.expect("fetch");
    assert_eq!(fetched, ciphertext, "W4: multi-chunk reassembles exactly");
    assert_eq!(decrypt_blob(&blob_key, &fetched).expect("decrypt"), plaintext);

    node.shutdown().await;
}

#[tokio::test]
async fn w3_corrupted_blob_rejected() {
    // W3 — content-address integrity: a substituted/corrupted on-disk blob is
    // rejected (node BlobStore::get → 10001 → fetch_blob errors), never returned.
    // RED-on-revert: dropping the get() hash check returns the tampered bytes.
    let node = spawn_in_process_node().await;
    let blobs_dir = node.spaces_dir.join("blobs");
    let key = keypair::generate();
    let mut conn = connect_url(&node.endpoint).await.expect("connect");
    let _ = conn.client_authenticate(&key).await.expect("authenticate");

    let (_k, ciphertext) = encrypt_blob(b"original attachment");
    let blob_ref = hash_uri(&ciphertext);
    conn.upload_blob(&blob_ref, &ciphertext, T).await.expect("upload");

    // Substitute the on-disk blob (same filename, different bytes).
    std::fs::write(blob_path(&blobs_dir, &blob_ref), b"tampered-substituted-bytes")
        .expect("corrupt the stored blob");

    let r = conn.fetch_blob(&blob_ref, T).await;
    assert!(r.is_err(), "W3: a corrupted blob is rejected, not returned");

    node.shutdown().await;
}

#[tokio::test]
async fn w_toolarge_over_ceiling_rejected_with_10002() {
    // F6 (M12.2a/D4) — the node rejects an over-ceiling blob with wire 10002
    // (`blob_too_large`), without buffering it; an under-ceiling blob still
    // round-trips on the same connection (one terminal reply per upload).
    // RED-on-revert: drop the BlobUploadBegin / accumulate size gate (the
    // `blob_oversized` path in handle_connection) → the over-ceiling upload is
    // buffered + stored → upload_blob returns Ok → the `is_err()` assert fails.
    // Small ceiling + small blob keeps the witness cheap.
    let ceiling: u64 = 1024;
    let node = spawn_in_process_node_with_max_blob_bytes(ceiling).await;
    let key = keypair::generate();
    let mut conn = connect_url(&node.endpoint).await.expect("connect");
    let _ = conn.client_authenticate(&key).await.expect("authenticate");

    // Over-ceiling: 4 KiB plaintext → ciphertext well past the 1 KiB ceiling.
    let (_k, big_ct) = encrypt_blob(&vec![0xABu8; 4096]);
    assert!(big_ct.len() as u64 > ceiling, "blob genuinely over the ceiling");
    let big_ref = hash_uri(&big_ct);
    let r = conn.upload_blob(&big_ref, &big_ct, T).await;
    assert!(r.is_err(), "F6: over-ceiling upload is rejected");
    let msg = format!("{}", r.unwrap_err());
    assert!(msg.contains("10002"), "F6: reject carries wire 10002 (got: {msg})");
    // Content-blind store never received the rejected bytes.
    assert!(
        !blob_path(&node.spaces_dir.join("blobs"), &big_ref).exists(),
        "F6: an over-ceiling blob is never stored"
    );

    // Under-ceiling still round-trips on the same connection (no leftover reply).
    let (uk, small_ct) = encrypt_blob(b"tiny");
    assert!(small_ct.len() as u64 <= ceiling, "small blob under the ceiling");
    let small_ref = hash_uri(&small_ct);
    conn.upload_blob(&small_ref, &small_ct, T).await.expect("under-ceiling upload ok");
    let fetched = conn.fetch_blob(&small_ref, T).await.expect("fetch");
    assert_eq!(decrypt_blob(&uk, &fetched).expect("decrypt"), b"tiny");

    node.shutdown().await;
}
