// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! M12.3 C3 — federated fetch-blob-by-hash **e2e** (`#[ignore]`, box-gated).
//!
//! The headline W1 (M12.3-A-01): a peer home **B** that holds a federated
//! `message.file` descriptor but not the ciphertext fetches the blob **across
//! homes** from the home that has it. `xgen-mptest` drives the **real binaries**
//! over `.aicontrol` across two federated nodes (the F2/F9 harness-control
//! `add-peer`/`initiate` seam, via `[[federation]]`; sibling to MP-C-02/03):
//!
//!   alice@A: register → create-space (homed at A) → create-room → invite bob →
//!            `send --attach <file>` (uploads the blob to A; the descriptor
//!            federates to B via the existing eager push)
//!   bob@B  : register → join (gated AFTER alice's send, so the descriptor has
//!            federated to B) → join room → `fetch --out-dir <dir>`
//!   assert : the file bob fetched on B is **byte-identical** to A's original —
//!            B held only the descriptor, missed locally, and federation-fetched
//!            the ciphertext from A (M12.3-D1/D4).
//!
//! Scope (honest, D-065): this is the box-gated real-binary headline; the
//! **deterministic** spine gate is the in-suite in-process witness
//! (`xgen-node` `m12_3_federation_fetch::federated_blob_fetch_round_trip`,
//! RED-on-revert). Federation propagation here is real-clock: bob's join is
//! gated on alice's send (`alice_sent`) so the descriptor has federated to B
//! well before bob fetches (a wide margin — register/join/room-join round-trips
//! intervene); if a RUN ever races, re-run (the in-suite witness is the
//! correctness lock). The typed-`10003`/never-leak/self-free properties are
//! witnessed in-suite (C1 + C2 W2/W4/W5).
//!
//! ```text
//! cargo build -p xgen-node --features harness-control && cargo build -p xgen-client
//! cargo test -p xgen-mptest --test m12_3_federation_fetch_e2e -- --ignored --nocapture
//! ```

use std::path::Path;

use xgen_mptest::dial::{ClockMode, RoundDial};
use xgen_mptest::manifest::Scenario;
use xgen_mptest::runner::run_scenario;

fn mock_dial() -> RoundDial {
    RoundDial {
        clock: ClockMode::Mock,
        ..Default::default()
    }
}

/// Deterministic payload spanning multiple `BLOB_CHUNK_BYTES` (128 KiB) chunks,
/// so the federated transfer also exercises chunked reassembly across homes.
fn payload(n: usize, seed: u8) -> Vec<u8> {
    (0..n).map(|i| ((i + seed as usize) % 251) as u8).collect()
}

#[tokio::test]
#[ignore = "heavy: spawns two harness-control xgen-node + 2 clients across homes; box-gated RUN"]
async fn w1_federated_blob_fetch_roundtrip_byte_identical() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let src = tmp.path().join("payload.bin");
    let bytes = payload(300 * 1024, 13); // > 128 KiB → multi-chunk across homes
    std::fs::write(&src, &bytes).expect("write payload");
    let out_dir = tmp.path().join("fetched");

    write_scenario(tmp.path(), &src, &out_dir, 8580);
    let scenario = Scenario::load(tmp.path()).expect("load M12.3 federation-fetch e2e scenario");
    let outcome = run_scenario(&scenario, &mock_dial())
        .await
        .expect("run_scenario(M12.3 e2e) — both nodes built --features harness-control?");

    // alice authored the attachment; bob fetched it across homes — both batches ok.
    let alice = outcome
        .actor_runs
        .iter()
        .find(|r| r.actor == "alice")
        .expect("alice run present");
    assert!(alice.all_ok(), "alice batch failed: {:?}", alice.replies);
    let bob = outcome
        .actor_runs
        .iter()
        .find(|r| r.actor == "bob")
        .expect("bob run present");
    assert!(bob.all_ok(), "bob (fetch) batch failed: {:?}", bob.replies);

    // Headline (W1): the file bob fetched on B is byte-identical to A's original.
    // B held only the federated descriptor; the bytes came from A via the
    // federated fetch (M12.3-D1/D4).
    let fetched = out_dir.join("payload.bin");
    let got = std::fs::read(&fetched)
        .unwrap_or_else(|e| panic!("bob's fetched file {fetched:?} not written: {e}"));
    assert_eq!(
        got, bytes,
        "W1: B's federation-fetched attachment is byte-identical to A's original"
    );

    eprintln!(
        "M12.3 e2e PASS: B fetched a federated blob byte-identical across homes from A"
    );
}

/// Two federated nodes; alice@A owns the Space + the blob, bob@B is a cross-home
/// member who fetches it. Federation is seeded by the runner's G-6 bootstrap
/// from the `[[federation]]` link. Bob's `join` is gated on `alice_sent` so the
/// `message.file` descriptor has federated to B before bob fetches.
fn write_scenario(dir: &Path, attach_path: &Path, out_dir: &Path, base_port: u16) {
    let manifest = format!(
        r#"
scenario = "M12-3-fed-fetch-e2e"
description = "federated fetch-blob-by-hash: alice@A send --attach, bob@B (cross-home) fetch byte-identical"

[[nodes]]
label = "a"
port = {base_port}

[[nodes]]
label = "b"
port = {port_b}

[[federation]]
from = "a"
to = "b"

[[actors]]
name = "alice"
node = "a"
batch = "alice.jsonl"

[[actors]]
name = "bob"
node = "b"
batch = "bob.jsonl"

# bob's identity → alice's invite
[[exports]]
actor = "bob"
command = "b1"
field = "identity_id"
key = "bob_identity_id"

[[exports]]
actor = "alice"
command = "a2"
field = "space_id"
key = "space_id"

[[exports]]
actor = "alice"
command = "a3"
field = "room_id"
key = "room_id"

# alice's send completing on A → the descriptor is authored + federating to B.
# bob's join is gated on it (a wide margin before bob's fetch).
[[exports]]
actor = "alice"
command = "a5"
field = "event_id"
key = "alice_sent"

# alice invites bob only once his identity is known.
[[waits]]
actor = "alice"
command = "a4"
key = "bob_identity_id"

# bob joins only after alice's attachment is sent (so the descriptor has
# federated to B before bob's later fetch sweeps B's room history).
[[waits]]
actor = "bob"
command = "b2"
key = "alice_sent"
"#,
        base_port = base_port,
        port_b = base_port + 1,
    );

    let attach = attach_path.to_str().expect("utf8 attach path");
    // alice: register, create Space (homed at A) + room, invite bob, send --attach.
    let alice = format!(
        concat!(
            "{reg}\n",
            "{create_space}\n",
            "{create_room}\n",
            "{invite}\n",
            "{send}\n"
        ),
        reg = serde_json::json!({"cmd":"register","args":{"name":"alice"},"id":"a1"}),
        create_space = serde_json::json!({"cmd":"create-space","args":{"name":"S"},"id":"a2","bind":"s"}),
        create_room = serde_json::json!({"cmd":"create-room","args":{"space":"$s","name":"general"},"id":"a3","bind":"r"}),
        invite = serde_json::json!({"cmd":"invite","args":{"space":"$s","identity":"{{bob_identity_id}}","role":"member"},"id":"a4"}),
        send = serde_json::json!({"cmd":"send","args":{"space":"$s","room":"$r","attach":[attach]},"id":"a5"}),
    );

    // bob: register, join Space (gated on alice_sent), join room, fetch.
    let bob = format!(
        concat!(
            "{reg}\n",
            "{join_space}\n",
            "{join_room}\n",
            "{fetch}\n"
        ),
        reg = serde_json::json!({"cmd":"register","args":{"name":"bob"},"id":"b1"}),
        join_space = serde_json::json!({"cmd":"join","args":{"space":"{{space_id}}"},"id":"b2"}),
        join_room = serde_json::json!({"cmd":"join","args":{"space":"{{space_id}}","room":"{{room_id}}"},"id":"b3"}),
        fetch = serde_json::json!({"cmd":"fetch","args":{"space":"{{space_id}}","room":"{{room_id}}","out_dir":out_dir.to_str().expect("utf8 out_dir")},"id":"b4"}),
    );

    std::fs::write(dir.join("manifest.toml"), manifest).expect("write manifest");
    std::fs::write(dir.join("alice.jsonl"), alice).expect("write alice batch");
    std::fs::write(dir.join("bob.jsonl"), bob).expect("write bob batch");
}
