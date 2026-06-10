// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! MP-R2 C6b — node restart + replay (`#[ignore]`).
//!
//! Proves the `ManagedProcess::restart` primitive (MP-R2-D5 / C6b) + MP-C-15: a
//! node killed mid-chat and re-spawned **without re-`init`** (same instance label
//! ⇒ same data dir) replays its persisted Spaces from disk. Driven directly (not
//! via `run_scenario`) because the restart needs the `ManagedProcess` handle.
//!
//! ## Box-gated (RUN gate, M-R2.3) + boundary
//! Heavy — spawns a real node. Asserts (1) the restart primitive preserves the
//! instance identity (label / data dir / pipe) + the node serves `state` again,
//! and (2) the durable property at the **hosted-space-count** level (count rises
//! on create, does NOT drop across restart = replayed). Asserting the *specific*
//! `space_id` survived (and cross-node rejoin-and-converge, composing with C3's
//! late-federation) is the box-gated RUN enrichment — needs a spaces-list verb /
//! a second node. Rule 2: a spawn/connect timeout is a flake, re-run isolated.
//!
//! ```text
//! cargo build -p xgen-node && cargo build -p xgen-client   # single-node, real clock: no harness-control
//! cargo test -p xgen-mptest --test mp_r2_restart -- --ignored --nocapture
//! ```

use xgen_mptest::aicontrol::{AicontrolClient, DEFAULT_CONNECT_TIMEOUT};
use xgen_mptest::binloc;
use xgen_mptest::process::{instance_label, ManagedProcess};
use xgen_mptest::wire::Reply;
use xgen_mptest::wireactor::WireActor;

const PORT: u16 = 8520;

/// The `state` reply's hosted-space count (mirrors the m9_2_f2 helper).
fn hosted_spaces(state: &Reply) -> u64 {
    state
        .data()
        .and_then(|d| d.get("hosted_spaces"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0)
}

#[tokio::test]
#[ignore = "heavy: spawns a real xgen-node, creates a Space, restarts it; box-gated RUN"]
async fn mp_c_15_restart_replay_preserves_space() {
    let bins = binloc::locate().expect("locate binaries");
    let label = instance_label("MP-C-15", "node");
    let mut node = ManagedProcess::init_and_spawn_node(&bins, &label, PORT, true, None)
        .expect("spawn node");

    // Capture the restart-primitive invariants (must survive the restart).
    let label_before = node.label.clone();
    let data_dir_before = node.data_dir.clone();
    let pipe_before = node.aicontrol_pipe.clone();

    let url = format!("ws://127.0.0.1:{PORT}/xgen");

    // Baseline hosted-space count, then create a fresh Space (persisted to disk).
    let mut ctl = AicontrolClient::connect(&node.aicontrol_pipe, DEFAULT_CONNECT_TIMEOUT)
        .await
        .expect("connect aicontrol (node up)");
    let before = hosted_spaces(&ctl.send_verb("state").await.expect("state pre-create"));

    let mut wa = WireActor::connect(&url).await.expect("wireactor connect");
    wa.register("alice").await.expect("register");
    let space = wa.create_space("MP-C-15").await.expect("create space");
    drop(wa);

    // The new Space must show up in the count before we restart.
    let mut after_create = 0;
    for _ in 0..20 {
        after_create = hosted_spaces(&ctl.send_verb("state").await.expect("state post-create"));
        if after_create > before {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }
    assert!(
        after_create > before,
        "Space {space} not hosted before restart (before {before}, after {after_create})"
    );
    drop(ctl);

    // ── Restart (no re-init; data dir persists ⇒ replay-from-disk) ───────────
    node.restart().expect("restart node");

    // Restart-primitive invariant: identity preserved.
    assert_eq!(node.label, label_before, "restart changed the instance label");
    assert_eq!(node.data_dir, data_dir_before, "restart changed the data dir");
    assert_eq!(node.aicontrol_pipe, pipe_before, "restart changed the .aicontrol pipe");

    // Reconnect (the old pipe server died with the old child) + assert the count
    // did not drop — the persisted Space replayed.
    let mut ctl2 = AicontrolClient::connect(&node.aicontrol_pipe, DEFAULT_CONNECT_TIMEOUT)
        .await
        .expect("reconnect aicontrol after restart");
    let after_restart = hosted_spaces(&ctl2.send_verb("state").await.expect("state post-restart"));
    assert!(
        after_restart >= after_create,
        "restart LOST hosted Spaces (had {after_create}, replayed only {after_restart}) — replay-from-disk failed"
    );
    eprintln!(
        "C6b MP-C-15 PASS: restart preserved instance identity + replayed Spaces from disk \
         (hosted {before} -> {after_create} -> {after_restart} across create + restart)"
    );
}
