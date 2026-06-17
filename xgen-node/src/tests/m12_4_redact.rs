// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! M12.4 — redact erasure integration witnesses (WE1/WE3/WE4) against a **real**
//! in-process node's runtime + a **real** on-disk `BlobStore`, driven through the
//! real `app::apply_redact_erasure` hook (the exact code `process_inbound`'s
//! `Accepted` arm runs for a `message.redact`).
//!
//! Honest boundary (D-065, sibling to M12.1's W-witnesses). These exercise the
//! **load-bearing** erasure mechanism end-to-end: the `resolve_redact_erasure`
//! decision (F2b retention read + target → `blob_ref` resolution, xgen-core) + the
//! fs delete + the Retained-refusal decision, on real runtime state + a real store
//! on disk. The **full WS redact-admission path** (a member sending a
//! `message.redact` over the wire → `process_inbound`) is the **C3 box-gated
//! real-binary e2e** (`m12_4_self_thread_redact_e2e`). WE2 (Retained refusal,
//! RED-on-revert D2) + WE5 (shared-fate) are xgen-core unit tests
//! (`node::runtime::m12_4_redact_erasure_tests`).
//!
//! **WE4 (federated erasure).** `apply_redact_erasure` is origin-agnostic: the
//! `process_inbound` call site (the `if event.event_type == MessageRedact` block
//! in the `Accepted` arm) has **no origin guard**, so it runs for both
//! `LocallySubmitted` and `ReceivedViaFederation`. So a redact arriving at a peer
//! that **cached** a blob (M12.3 — `store.put` on the fetch-miss path, app.rs:1945)
//! erases that cache. The blob in `we4_federated_redact_deletes_cached_copy` is put
//! via that same `BlobStore::put` path B uses, modeling B's cache; the redact
//! mechanism deletes it. RED-on-revert: guarding the hook call on
//! `origin == LocallySubmitted` would leave federated caches.

use xgen_common::xgid::{IdentityXgid, NodeXgid, SpaceXgid, Xgid};

use serde_json::json;

use xgen_core::blob_store::BlobStore;
use xgen_core::crypto::hashing::hash_uri;
use xgen_core::encryption::blob::encrypt_blob;
use xgen_core::identity::keypair;
use xgen_core::identity::registry::{DeviceRecord, IdentityRecord};
use xgen_core::message::exchange::{
    build_message_file_event, build_message_redact_event, Descriptor,
};
use xgen_core::node::runtime::RedactErasure;
use xgen_core::space::state::sign_event;
use xgen_core::wire::types::Event;

use super::phase9_harness::{spawn_in_process_node, InProcessNode};

const SID: &str =
    "xgen://space/sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const RID: &str =
    "xgen://room/sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

fn blob_path(blobs_dir: &std::path::Path, blob_ref: &str) -> std::path::PathBuf {
    let hex = blob_ref.strip_prefix("xgen://hash/sha256:").unwrap();
    blobs_dir.join(format!("{hex}.blob"))
}

fn author_record(author: &IdentityXgid, retention: Option<&str>) -> IdentityRecord {
    let trust_assertion = retention.map(|r| {
        json!({
            "claims": {
                "tier_verified": true,
                "module_policy": { "erasability": { "retention": r } }
            }
        })
    });
    IdentityRecord {
        identity_id: author.clone(),
        display_name: None,
        is_ai: false,
        ai_capabilities: None,
        registered_at: "2026-06-17T00:00:00.000Z".to_string(),
        trust_assertion,
        devices: vec![DeviceRecord {
            device_id: author.as_str().to_string(),
            device_name: None,
            authorised_at: "2026-06-17T00:00:00.000Z".to_string(),
        }],
        home_node: NodeXgid::from_xgid(Xgid::new("xgen://pubkey/ed25519:NODE".to_string())),
        update_version: 0,
        revoked: false,
        revoked_at: None,
        revocation_reason: None,
    }
}

/// Put a real ciphertext blob on disk (the same `BlobStore::put` the upload /
/// federated-cache paths use), store a signed `message.file` referencing it in the
/// node's runtime, register its author with `retention`, and return
/// `(blobs_dir, blob_ref, redact_targeting_the_file)`.
async fn setup(node: &InProcessNode, plaintext: &[u8], retention: Option<&str>) -> (std::path::PathBuf, String, Event) {
    let blobs_dir = node.spaces_dir.join("blobs");
    let (_k, ciphertext) = encrypt_blob(plaintext);
    let blob_ref = hash_uri(&ciphertext);
    BlobStore::new(&blobs_dir).put(&ciphertext).expect("put blob");

    let author = keypair::generate();
    let desc = Descriptor {
        blob_ref: blob_ref.clone(),
        plaintext_hash: hash_uri(plaintext),
        key: "BASE64-PER-BLOB-KEY".to_string(),
        filename: "attachment.bin".to_string(),
        mime: "application/octet-stream".to_string(),
        size: plaintext.len() as u64,
    };
    let file_ev = sign_event(
        build_message_file_event(&author, SID, RID, vec![], &[desc]),
        &author,
    );
    let author_id = file_ev.sender.clone();
    let file_id = file_ev.event_id.as_ref().unwrap().as_str().to_string();
    let space = SpaceXgid::from_xgid(Xgid::new(SID.to_string()));
    {
        let mut rt = node.runtime.lock().await;
        rt.ensure_store(&space).unwrap();
        rt.stores.get_mut(&space).unwrap().append(file_ev).unwrap();
        rt.identity_registry
            .register(author_record(&author_id, retention))
            .unwrap();
    }

    // The redactor is an arbitrary key (NOT the author) — D2 reads the author's
    // retention, not the redactor's.
    let redactor = keypair::generate();
    let redact = sign_event(
        build_message_redact_event(&redactor, SID, RID, vec![], &file_id),
        &redactor,
    );
    (blobs_dir, blob_ref, redact)
}

#[tokio::test]
async fn we1_redact_erases_blob_bytes() {
    // WE1 (headline) — an Erasable author's attachment is redacted → the ciphertext
    // file is gone from blobs_dir, and the store no longer serves it.
    // RED-on-revert: skip the BlobStore::delete in apply_redact_erasure → the blob
    // stays present.
    let node = spawn_in_process_node().await;
    let (blobs_dir, blob_ref, redact) =
        setup(&node, b"M12.4 erasable attachment payload", Some("erasable")).await;

    assert!(blob_path(&blobs_dir, &blob_ref).exists(), "blob present before redact");

    let outcome =
        crate::app::apply_redact_erasure(&redact, &node.runtime, &blobs_dir).await;
    assert_eq!(outcome, RedactErasure::Delete(vec![blob_ref.clone()]));

    // The bytes are erased — both on disk and through the store.
    assert!(!blob_path(&blobs_dir, &blob_ref).exists(), "WE1: blob bytes erased");
    assert_eq!(
        BlobStore::new(&blobs_dir).get(&blob_ref).unwrap(),
        None,
        "WE1: the store no longer serves the redacted blob"
    );

    node.shutdown().await;
}

#[tokio::test]
async fn we3_retained_redact_keeps_bytes_event_unaffected() {
    // WE3 (convergence / side-effect-gate, D3) — a Retained author's content is
    // redacted: the erasure side-effect is REFUSED (10004) and the bytes SURVIVE
    // (the legal-hold floor). The redact event itself is untouched — the gate is on
    // the side-effect, not admission (the hook runs in the Accepted arm, downstream
    // of admission; apply_redact_erasure returns a decision, never un-admits). So
    // the redact event converges on every node regardless of this outcome.
    // RED-on-revert: gating ADMISSION on retention (rejecting the redact) would
    // diverge across nodes.
    let node = spawn_in_process_node().await;
    let (blobs_dir, blob_ref, redact) =
        setup(&node, b"M12.4 retained attachment payload", Some("retained")).await;

    let outcome =
        crate::app::apply_redact_erasure(&redact, &node.runtime, &blobs_dir).await;
    assert_eq!(
        outcome,
        RedactErasure::RefusedRetained,
        "WE3: Retained content's erasure is refused"
    );
    assert!(
        blob_path(&blobs_dir, &blob_ref).exists(),
        "WE3: the legal-hold floor keeps the bytes"
    );
    // The target message.file event is still in the store — the redact never
    // un-admits it (D3: the side-effect, not admission, is gated).
    let space = SpaceXgid::from_xgid(Xgid::new(SID.to_string()));
    let target_id = redact.content["target_event_id"].as_str().unwrap();
    let target_xid =
        xgen_common::xgid::EventXgid::from_xgid(Xgid::new(target_id.to_string()));
    let present = {
        let rt = node.runtime.lock().await;
        rt.stores.get(&space).unwrap().contains(&target_xid)
    };
    assert!(present, "WE3: the target event remains stored (admission untouched)");

    node.shutdown().await;
}

#[tokio::test]
async fn we4_federated_redact_deletes_cached_copy() {
    // WE4 (federated erasure) — the blob models a peer's M12.3 cache (put via the
    // same BlobStore::put path B uses at the federation fetch-miss, app.rs:1945).
    // apply_redact_erasure is origin-agnostic (the process_inbound call site has no
    // origin guard), so the same mechanism that erases a local copy erases a
    // federated cache. RED-on-revert: guarding the hook on origin==LocallySubmitted
    // would leave federated caches intact.
    let node = spawn_in_process_node().await;
    let (blobs_dir, blob_ref, redact) =
        setup(&node, b"M12.4 federated-cached attachment", Some("erasable")).await;

    assert!(blob_path(&blobs_dir, &blob_ref).exists(), "cached blob present on the peer");

    let outcome =
        crate::app::apply_redact_erasure(&redact, &node.runtime, &blobs_dir).await;
    assert_eq!(outcome, RedactErasure::Delete(vec![blob_ref.clone()]));
    assert!(
        !blob_path(&blobs_dir, &blob_ref).exists(),
        "WE4: the federated cached copy is erased"
    );

    node.shutdown().await;
}
