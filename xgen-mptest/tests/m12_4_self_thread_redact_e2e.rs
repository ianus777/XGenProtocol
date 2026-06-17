// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! M12.4 C3 — the full self-thread **redact** e2e (`#[ignore]`, box-gated).
//!
//! Drives the **real binaries** over `.aicontrol` to witness erasure end-to-end:
//!
//!   client A: `register` → `self` → `send --attach secret.bin` (a3)
//!                                  → `send --attach public.bin` (a3b)
//!                                  → `redact --target <a3's event_id>` (a4)
//!   client B (same identity via `[[rehome]] to = "a"`): `fetch --out-dir <dir>`
//!   assert: `public.bin` is fetched (fetch works), `secret.bin` is **absent**
//!           (redacted → the node erased its blob bytes, M12.4-D4, and the client
//!           tombstone skips it, M12.4-D6) — selective, real-binary erasure.
//!
//! This is the real-binary counterpart to the in-process WE1/WE3/WE4 witnesses
//! (xgen-node `m12_4_redact`) + the WE2/WE5 decision units (xgen-core). The full
//! Retained-refusal path (10004) is witnessed in-suite (WE2); the scenario
//! manifest exposes no Trust-Assertion knob to drive a Retained author through the
//! real binary, so the box e2e covers the erasure + tombstone headline.
//!
//! ```text
//! cargo build -p xgen-node --features harness-control && cargo build -p xgen-client
//! cargo test -p xgen-mptest --test m12_4_self_thread_redact_e2e -- --ignored --nocapture
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

fn payload(n: usize, seed: u8) -> Vec<u8> {
    (0..n).map(|i| ((i + seed as usize) % 251) as u8).collect()
}

#[tokio::test]
#[ignore = "heavy: spawns a real harness-control node + 2 same-identity clients; box-gated RUN"]
async fn w_redact_erases_attachment_e2e() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let secret = tmp.path().join("secret.bin");
    let public = tmp.path().join("public.bin");
    let secret_bytes = payload(300 * 1024, 13); // multi-chunk
    let public_bytes = payload(200 * 1024, 41);
    std::fs::write(&secret, &secret_bytes).expect("write secret");
    std::fs::write(&public, &public_bytes).expect("write public");
    let out_dir = tmp.path().join("fetched");

    write_redact_scenario(tmp.path(), &secret, &public, &out_dir, 8564);
    let scenario = Scenario::load(tmp.path()).expect("load M12.4 redact e2e scenario");
    let outcome = run_scenario(&scenario, &mock_dial())
        .await
        .expect("run_scenario(M12.4 redact e2e) — node built --features harness-control?");

    // A's register/self/send/send/redact all succeeded.
    let alice = outcome
        .actor_runs
        .iter()
        .find(|r| r.actor == "alice")
        .expect("alice run present");
    assert!(alice.all_ok(), "alice batch failed: {:?}", alice.replies);

    // B's fetch succeeded.
    let fetcher = outcome
        .actor_runs
        .iter()
        .find(|r| r.actor == "alice-rehome")
        .expect("fetch client (alice-rehome) run present");
    assert!(fetcher.all_ok(), "fetch batch failed: {:?}", fetcher.replies);

    // Headline: the NON-redacted attachment is retrievable (fetch works), the
    // REDACTED one is gone (bytes erased node-side + tombstoned client-side).
    let got_public = std::fs::read(out_dir.join("public.bin"))
        .expect("public.bin fetched — fetch path works");
    assert_eq!(got_public, public_bytes, "non-redacted attachment byte-identical");
    assert!(
        !out_dir.join("secret.bin").exists(),
        "M12.4 e2e: the redacted attachment is NOT retrievable (erased + tombstoned)"
    );

    eprintln!("M12.4 redact e2e PASS: redacted attachment gone, non-redacted retrievable (real binaries)");
}

/// One node, one identity. alice registers, opens her `self` thread, sends two
/// attachments (`secret` then `public`), then redacts the `secret` send (a3). The
/// `[[rehome]] to = "a"` step spawns a second client reusing alice's keypair that
/// `fetch`es into `out_dir`. a3's `event_id` is exported as `secret_event_id` (the
/// redact target).
fn write_redact_scenario(
    dir: &Path,
    secret: &Path,
    public: &Path,
    out_dir: &Path,
    port: u16,
) {
    let manifest = format!(
        r#"
scenario = "M12-4-redact-e2e"
description = "self-thread redact e2e: A sends two attachments + redacts one; B fetches — redacted gone, other present"

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

[[exports]]
actor = "alice"
command = "a3"
field = "event_id"
key = "secret_event_id"
"#
    );

    let secret_p = secret.to_str().expect("utf8 path");
    let public_p = public.to_str().expect("utf8 path");
    let reg = serde_json::json!({"cmd":"register","args":{"name":"alice"},"id":"a1"});
    let self_open = serde_json::json!({"cmd":"self","args":{},"id":"a2"});
    let send_secret = serde_json::json!({
        "cmd":"send",
        "args":{"space":"{{space_id}}","room":"{{room_id}}","attach":[secret_p]},
        "id":"a3"
    });
    let send_public = serde_json::json!({
        "cmd":"send",
        "args":{"space":"{{space_id}}","room":"{{room_id}}","attach":[public_p]},
        "id":"a3b"
    });
    let redact = serde_json::json!({
        "cmd":"redact",
        "args":{"space":"{{space_id}}","room":"{{room_id}}","target":"{{secret_event_id}}"},
        "id":"a4"
    });
    let alice = format!("{reg}\n{self_open}\n{send_secret}\n{send_public}\n{redact}\n");

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
