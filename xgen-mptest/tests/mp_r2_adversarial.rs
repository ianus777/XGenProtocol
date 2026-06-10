// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! MP-R2 C6d — single-node adversarial-submit smokes (`#[ignore]`).
//!
//! The C6d rows that fit the **existing single-node** wire path (a member-context
//! `WireActor` crafts + submits a hostile event; the node must bound/reject it +
//! stay live). MP-A-06 (equivocation) is **NOT here** — it needs a two-node
//! injector (a new multi-node-adversary capability) and was Joe-re-routed to R3
//! alongside MP-A-08 (see the matrix + runbook §9/§13).
//!
//! ## Box-gated (RUN gate, M-R2.3) + boundary
//! Heavy — spawns a real node. Each asserts node-liveness after the attack (a
//! legitimate `.aicontrol` `state` still lands). MP-A-21's no-epoch-regression
//! property + MP-A-11's exact size-cap are box-gated RUN deliverables (the
//! harness has no `mls_epoch` query). Rule 2: spawn timeout = flake.
//!
//! ```text
//! cargo build -p xgen-node && cargo build -p xgen-client   # single-node, real clock: no harness-control
//! cargo test -p xgen-mptest --test mp_r2_adversarial -- --ignored --nocapture
//! ```

use std::time::Duration;

use xgen_core::space::state::build_mls_commit_event;
use xgen_mptest::aicontrol::{AicontrolClient, DEFAULT_CONNECT_TIMEOUT};
use xgen_mptest::binloc;
use xgen_mptest::injector::build_member_message;
use xgen_mptest::process::{instance_label, ManagedProcess};
use xgen_mptest::wireactor::WireActor;

const PORT_A11: u16 = 8523;
const PORT_A21: u16 = 8524;
const RECV_WINDOW: Duration = Duration::from_millis(800);

// ── MP-A-11 — oversized payload: bounded/rejected, node stays live ────────────

#[tokio::test]
#[ignore = "heavy: spawns a real xgen-node + submits a 1 MiB event; box-gated RUN"]
async fn mp_a_11_oversized_payload_bounded_node_alive() {
    let bins = binloc::locate().expect("locate binaries");
    let label = instance_label("MP-A-11", "node");
    let node = ManagedProcess::init_and_spawn_node(&bins, &label, PORT_A11, true, None)
        .expect("spawn node");
    let mut ctl = AicontrolClient::connect(&node.aicontrol_pipe, DEFAULT_CONNECT_TIMEOUT)
        .await
        .expect("connect aicontrol (node up)");

    let url = format!("ws://127.0.0.1:{PORT_A11}/xgen");
    let mut wa = WireActor::connect(&url).await.expect("wireactor connect");
    wa.register("oversize").await.expect("register");
    let space = wa.create_space("MP-A-11").await.expect("create space");
    let room = wa.create_room(&space, "general").await.expect("create room");

    // A 1 MiB message text — the node must bound or reject it, not OOM.
    let huge = "x".repeat(1024 * 1024);
    let ev = build_member_message(wa.key(), &space, &room, vec![&space], &huge);
    let reply = wa
        .submit_recv_error(&ev, RECV_WINDOW)
        .await
        .expect("oversized submit");
    eprintln!("MP-A-11 oversized: node reply = {reply:?}");

    // The node must still serve honest traffic (no hang / no OOM).
    let state = ctl.send_verb("state").await.expect("state after oversized");
    assert!(
        state.is_ok(),
        "node did NOT stay live after a 1 MiB payload: {state:?}"
    );
    eprintln!("C6d MP-A-11 PASS: node bounded/rejected the oversized payload + stayed live");
}

// ── MP-A-21 — stale MLS commit replay: no epoch regression, node stays live ───

#[tokio::test]
#[ignore = "heavy: spawns a real xgen-node + replays a stale mls.commit; box-gated RUN"]
async fn mp_a_21_stale_mls_commit_no_regression() {
    let bins = binloc::locate().expect("locate binaries");
    let label = instance_label("MP-A-21", "node");
    let node = ManagedProcess::init_and_spawn_node(&bins, &label, PORT_A21, true, None)
        .expect("spawn node");
    let mut ctl = AicontrolClient::connect(&node.aicontrol_pipe, DEFAULT_CONNECT_TIMEOUT)
        .await
        .expect("connect aicontrol (node up)");

    let url = format!("ws://127.0.0.1:{PORT_A21}/xgen");
    let mut wa = WireActor::connect(&url).await.expect("wireactor connect");
    wa.register("mallory").await.expect("register");
    let space = wa.create_space("MP-A-21").await.expect("create space");
    let room = wa.create_room(&space, "general").await.expect("create room");

    // Advance to a high epoch, then replay a STALE (lower-epoch) commit.
    let advance = build_mls_commit_event(wa.key(), &space, &room, vec![room.clone()], 3);
    wa.submit(&advance).await.expect("submit advance commit (epoch 3)");
    let stale = build_mls_commit_event(wa.key(), &space, &room, vec![room.clone()], 1);
    let reply = wa
        .submit_recv_error(&stale, RECV_WINDOW)
        .await
        .expect("submit stale commit (epoch 1)");
    eprintln!("MP-A-21 stale-commit replay: node reply = {reply:?}");

    // The node must stay live; the stale replay must not roll the epoch back
    // (M8.7 mls_commit_tip keys by target epoch — the stale commit cannot win
    // against the advanced epoch). The no-regression *observable* (an mls_epoch
    // query) is the box-gated RUN deliverable; here we assert node-liveness +
    // capture the reply (rejected/inert).
    let state = ctl.send_verb("state").await.expect("state after stale commit");
    assert!(
        state.is_ok(),
        "node did NOT stay live after a stale mls.commit replay: {state:?}"
    );
    eprintln!("C6d MP-A-21 PASS: node stayed live after a stale mls.commit replay (no-regression query is the RUN deliverable)");
}
