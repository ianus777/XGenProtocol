// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! MP-R2 C6e — MP-C-12 E2E node-content-blindness (injector-path, `#[ignore]`).
//!
//! The **black-box, real-binary** analogue of the in-process Arc H proof
//! (`xgen-node/src/tests/arc_h_content_blindness.rs`): a member constructs the
//! epoch key + per-message `enc:` envelope **test-crate-side** (the node never
//! holds the key), submits the encrypted `message.text` over the real wire via
//! [`WireActor`], and we assert the node's observed `.events` carry the
//! **ciphertext only** — the plaintext marker never appears node-side.
//!
//! ## Scope — node-blindness CORE only (Joe-locked C6e, honestly scoped)
//! This is **not** a full E2E witness. It proves the **node-content-blindness**
//! property MP-C-12 most cares about (the node stores/fans-out opaque ciphertext;
//! it holds no key, so it has no path to plaintext). The **client-DECRYPT half**
//! (a member unwrapping `CK` under the epoch key back to plaintext) stays
//! **D3-gated** (Arc H / J-257 — the production client e2e path is unbuilt). That
//! decrypt half is exercised *in-process* by the Arc H test; here it is a named
//! **boundary** (sibling to MP-C-07's harness-green-with-boundary), NOT asserted.
//!
//! No client-verb e2e drive: the client `--aicontrol`/`ops` surface has no e2e
//! verb, and exposing one is production work (D3) — that path is STOP / route, not
//! patch. This smoke uses only the **test-crate raw-wire path** (xgen-core builders
//! + WireActor), so it needs **no production-crate change** (the C6e grounding verdict).
//!
//! ## Box-gated (RUN gate, M-R2.3)
//! Heavy — spawns a real node. The content-blindness assertion (ciphertext
//! present, plaintext absent in the node transcript) runs at the box-gated RUN.
//! Rule 2: a spawn/connect timeout is a flake, re-run isolated.
//!
//! ```text
//! cargo build -p xgen-node && cargo build -p xgen-client   # single-node, real clock: no harness-control
//! cargo test -p xgen-mptest --test mp_r2_e2e -- --ignored --nocapture
//! ```

use std::time::Duration;

use xgen_core::encryption::client_mls::{encrypt_message_envelope, ClientMlsGroup};
use xgen_core::space::state::{
    build_mls_group_init_event, build_room_create_event, build_space_create_event, sign_event,
};
use xgen_core::wire::types::Event;

use xgen_mptest::binloc;
use xgen_mptest::events::{EventCollector, Filter};
use xgen_mptest::injector::build_member_message;
use xgen_mptest::process::{events_pipe, instance_label, Kind, ManagedProcess};
use xgen_mptest::wireactor::WireActor;

const PORT: u16 = 8525;
/// The plaintext (an inner `message.text` payload, mirroring the Arc H proof).
/// Its marker must NEVER appear in the node's observed events.
const PLAINTEXT: &[u8] = br#"{"text":"MP-C-12-SECRET-PLAINTEXT-MARKER-9f3a2b"}"#;
const MARKER: &str = "MP-C-12-SECRET-PLAINTEXT-MARKER-9f3a2b";

fn event_id_of(ev: &Event) -> String {
    ev.event_id
        .as_ref()
        .map(|e| e.as_str().to_string())
        .expect("signed event has an event_id")
}

#[tokio::test]
#[ignore = "heavy: spawns a real xgen-node + submits an encrypted message; box-gated RUN"]
async fn mp_c_12_node_content_blindness_injector_path() {
    let bins = binloc::locate().expect("locate binaries");
    let label = instance_label("MP-C-12", "node");
    // Held for kill-on-drop (the node lives until end of scope).
    let _node = ManagedProcess::init_and_spawn_node(&bins, &label, PORT, true, None)
        .expect("spawn node");

    // Observe the node's `.events` from the start (live-only; attach-at-start).
    let collector = EventCollector::start("a", &events_pipe(Kind::Node, &label), Filter::all())
        .await
        .expect("attach .events collector");

    let url = format!("ws://127.0.0.1:{PORT}/xgen");
    let mut wa = WireActor::connect(&url).await.expect("wireactor connect");
    wa.register("alice").await.expect("register");

    // ── E2E Space (e2e_encryption ON) + room + mls_group_init genesis ────────
    let space_ev = sign_event(
        build_space_create_event(wa.key(), "MP-C-12", None, 1, &url, None, true),
        wa.key(),
    );
    let space_id = event_id_of(&space_ev);
    wa.submit(&space_ev).await.expect("submit e2e space create");

    let room_ev = sign_event(
        build_room_create_event(wa.key(), &space_id, "general", None),
        wa.key(),
    );
    let room_id = event_id_of(&room_ev);
    wa.submit(&room_ev).await.expect("submit room create");

    let init_ev = sign_event(
        build_mls_group_init_event(wa.key(), &space_id, &room_id, &room_id),
        wa.key(),
    );
    wa.submit(&init_ev).await.expect("submit mls_group_init");

    // ── Construct the epoch key + per-message envelope TEST-CRATE-SIDE ───────
    // The epoch secret lives only here (the member); the node never holds it.
    let group = ClientMlsGroup::new(&room_id, wa.identity_id(), [7u8; 32]);
    let epoch_key = group.current_epoch_key();
    let enc = encrypt_message_envelope(&epoch_key, group.epoch, PLAINTEXT);
    let enc_blob = enc.0.clone();
    assert!(enc_blob.starts_with("enc:"), "envelope must carry the enc: prefix");
    assert!(
        !enc_blob.contains(MARKER),
        "sanity: the ciphertext envelope must not contain the plaintext marker"
    );

    // ── Submit the encrypted message.text over the real wire ─────────────────
    let msg_ev = sign_event(
        build_member_message(wa.key(), &space_id, &room_id, vec![&space_id], &enc_blob),
        wa.key(),
    );
    let msg_id = event_id_of(&msg_ev);
    wa.submit(&msg_ev).await.expect("submit encrypted message");

    // ── Settle: poll the observer until the encrypted message is observed ────
    let mut snapshot = Vec::new();
    let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
    while tokio::time::Instant::now() < deadline {
        tokio::time::sleep(Duration::from_millis(300)).await;
        snapshot = collector.snapshot().await;
        let snap_json = serde_json::to_string(&snapshot).unwrap();
        if snap_json.contains(&enc_blob) {
            break;
        }
    }
    let snap_json = serde_json::to_string(&snapshot).unwrap();

    // ── (core) the node carries the CIPHERTEXT, never the PLAINTEXT ──────────
    assert!(
        snap_json.contains(&enc_blob),
        "node `.events` did not carry the encrypted message {msg_id} (enc: blob absent): {snap_json}"
    );
    assert!(
        !snap_json.contains(MARKER),
        "CONTENT-BLINDNESS VIOLATED: the plaintext marker leaked into the node's observed events"
    );

    // D3 BOUNDARY (named, not asserted): the client-DECRYPT half — a member
    // unwrapping CK under the epoch key back to PLAINTEXT — is exercised in-process
    // by the Arc H proof, and stays D3-gated here (no production client e2e path).
    eprintln!(
        "C6e MP-C-12 PASS (node-blindness core): node carried the enc: ciphertext for {msg_id} \
         + the plaintext marker never appeared node-side. Client-decrypt half = D3 boundary."
    );

    drop(wa);
    drop(collector);
}
