// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! M12.2a C4 — the full self-thread attachment **e2e** (`#[ignore]`, box-gated).
//!
//! Discharges the M12.1 honest boundary: "the thin `xgen-client` ops glue
//! (`ops::send --attach` / `ops::fetch_attachments`) has no full ops-level /
//! self-thread e2e … the real-binary fetch path needs the M12.2 fetch CLI verb."
//! With the fetch verb now a routable `ClientCommand` (C1), `xgen-mptest` drives
//! the **real binaries** over `.aicontrol`:
//!
//!   client A: `register` → `self` (open the personal thread) → `send --attach`
//!   client B (a SECOND client reusing A's keypair = same identity, via the
//!            `[[rehome]] to = "a"` rails): `fetch --out-dir <dir>`
//!   assert: the fetched file on B's disk is **byte-identical** to A's original.
//!
//! This adds the ops/CLI-layer coverage C5's raw-WS witness could not reach
//! (it drove `Connection::upload_blob/fetch_blob` directly, bypassing the verb).
//!
//! Scope (honest, D-065): two witnesses ship here — `w_e2e` (single-file
//! self-thread round-trip, multi-chunk over the wire) + `w_multi` (multi-file +
//! the S-2 `reconstruct_argv` array arm end-to-end). The **F6 size gate** is
//! witnessed **in-suite** by `xgen-node` `m12_blob_roundtrip::w_toolarge_*`
//! (RED-on-revert); a real-binary too-large witness would need a per-node
//! ceiling knob the scenario manifest does not expose (and the default 16 MiB
//! client pre-check fires before the node gate) — deferred, low marginal value
//! over the in-suite RED-on-revert.
//!
//! ```text
//! cargo build -p xgen-node --features harness-control && cargo build -p xgen-client
//! cargo test -p xgen-mptest --test m12_2a_self_thread_e2e -- --ignored --nocapture
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

/// Deterministic payload that spans multiple `BLOB_CHUNK_BYTES` (128 KiB) chunks,
/// so the round-trip also exercises chunked transfer over the real binary.
fn payload(n: usize, seed: u8) -> Vec<u8> {
    (0..n).map(|i| ((i + seed as usize) % 251) as u8).collect()
}

#[tokio::test]
#[ignore = "heavy: spawns a real harness-control node + 2 same-identity clients; box-gated RUN"]
async fn w_e2e_self_thread_attachment_roundtrip_byte_identical() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let src = tmp.path().join("payload.bin");
    let bytes = payload(300 * 1024, 7); // > 128 KiB → multi-chunk
    std::fs::write(&src, &bytes).expect("write payload");
    let out_dir = tmp.path().join("fetched");

    write_e2e_scenario(tmp.path(), std::slice::from_ref(&src), &out_dir, 8560);
    let scenario = Scenario::load(tmp.path()).expect("load M12.2a e2e scenario");
    let outcome = run_scenario(&scenario, &mock_dial())
        .await
        .expect("run_scenario(M12.2a e2e) — node built --features harness-control?");

    // A's send (a3) and B's fetch (f1) both succeeded.
    let alice = outcome
        .actor_runs
        .iter()
        .find(|r| r.actor == "alice")
        .expect("alice run present");
    assert!(alice.all_ok(), "alice batch failed: {:?}", alice.replies);
    let bob = outcome
        .actor_runs
        .iter()
        .find(|r| r.actor == "alice-rehome")
        .expect("fetch client (alice-rehome) run present");
    assert!(bob.all_ok(), "fetch batch failed: {:?}", bob.replies);

    // Headline: the fetched file on B's disk is byte-identical to A's original.
    let fetched = out_dir.join("payload.bin");
    let got = std::fs::read(&fetched)
        .unwrap_or_else(|e| panic!("fetched file {fetched:?} not written: {e}"));
    assert_eq!(got, bytes, "e2e: fetched attachment is byte-identical to the original");

    eprintln!("M12.2a e2e PASS: self-thread attachment round-tripped byte-identical via real binaries");
}

#[tokio::test]
#[ignore = "heavy: spawns a real harness-control node + 2 same-identity clients; box-gated RUN"]
async fn w_multi_self_thread_multifile_roundtrip() {
    // W-multi — D3 multi-file `--attach` + the S-2 reconstruct_argv array arm,
    // end-to-end over the real binary: A sends TWO files in one `send`; B fetches
    // both byte-identical.
    let tmp = tempfile::tempdir().expect("tempdir");
    let a = tmp.path().join("alpha.bin");
    let b = tmp.path().join("beta.bin");
    let abytes = payload(64 * 1024, 11);
    let bbytes = payload(200 * 1024, 29);
    std::fs::write(&a, &abytes).expect("write alpha");
    std::fs::write(&b, &bbytes).expect("write beta");
    let out_dir = tmp.path().join("fetched");

    write_e2e_scenario(tmp.path(), &[a, b], &out_dir, 8562);
    let scenario = Scenario::load(tmp.path()).expect("load M12.2a multi-file scenario");
    let outcome = run_scenario(&scenario, &mock_dial())
        .await
        .expect("run_scenario(M12.2a multi) — node built --features harness-control?");

    let fetcher = outcome
        .actor_runs
        .iter()
        .find(|r| r.actor == "alice-rehome")
        .expect("fetch client run present");
    assert!(fetcher.all_ok(), "fetch batch failed: {:?}", fetcher.replies);

    let got_a = std::fs::read(out_dir.join("alpha.bin")).expect("alpha fetched");
    let got_b = std::fs::read(out_dir.join("beta.bin")).expect("beta fetched");
    assert_eq!(got_a, abytes, "multi-file: alpha byte-identical");
    assert_eq!(got_b, bbytes, "multi-file: beta byte-identical");

    eprintln!("M12.2a multi-file e2e PASS: both attachments round-tripped byte-identical");
}

/// One node, one identity. alice registers, opens her `self` thread, and sends a
/// `message.file` carrying `attach_paths`. The `[[rehome]] to = "a"` step spawns a
/// SECOND client reusing alice's keypair (same identity, same home node) that runs
/// `fetch` into `out_dir`. The self thread's `space_id`/`room_id` flow from `self`
/// (a2) to both A's `send` and B's `fetch` via `{{…}}` exports.
fn write_e2e_scenario(dir: &Path, attach_paths: &[std::path::PathBuf], out_dir: &Path, port: u16) {
    // Each test fn uses a distinct port (the mp_c06 convention): `--ignored` runs
    // both tests in parallel, so a shared port would collide on bind.
    let manifest = format!(
        r#"
scenario = "M12-2a-e2e"
description = "self-thread attachment e2e: A send --attach, B (same identity) fetch byte-identical"

[[nodes]]
label = "a"
port = {port}

[[actors]]
name = "alice"
node = "a"
batch = "alice.jsonl"

[[rehome]]
actor = "alice"
to = "a"
batch = "alice_fetch.jsonl"

[[exports]]
actor = "alice"
command = "a2"
field = "space_id"
key = "space_id"

[[exports]]
actor = "alice"
command = "a2"
field = "room_id"
key = "room_id"
"#
    );

    // Build batch lines via serde_json so absolute paths (Windows `\`) escape
    // correctly; the `{{space_id}}`/`{{room_id}}` placeholders are literal string
    // values the registry resolves at drive time.
    let attach: Vec<&str> = attach_paths.iter().map(|p| p.to_str().expect("utf8 path")).collect();
    let reg = serde_json::json!({"cmd":"register","args":{"name":"alice"},"id":"a1"});
    let self_open = serde_json::json!({"cmd":"self","args":{},"id":"a2"});
    let send = serde_json::json!({
        "cmd":"send",
        "args":{"space":"{{space_id}}","room":"{{room_id}}","attach": attach},
        "id":"a3"
    });
    let alice = format!("{reg}\n{self_open}\n{send}\n");

    let fetch = serde_json::json!({
        "cmd":"fetch",
        "args":{"space":"{{space_id}}","room":"{{room_id}}","out_dir": out_dir.to_str().expect("utf8 out_dir")},
        "id":"f1"
    });
    let fetch_batch = format!("{fetch}\n");

    std::fs::write(dir.join("manifest.toml"), manifest).expect("write manifest");
    std::fs::write(dir.join("alice.jsonl"), alice).expect("write alice batch");
    std::fs::write(dir.join("alice_fetch.jsonl"), fetch_batch).expect("write fetch batch");
}
