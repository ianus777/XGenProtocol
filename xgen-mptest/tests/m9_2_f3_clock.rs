// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! M9.2 F3 smoke — FENCED `clock advance` / `clock set` (`#[ignore]`).
//!
//! Drives the node's injected `MockClock` over `--aicontrol` and observes its
//! `now_utc()` move deterministically via the verb's own reply (`data.now_utc`).
//! This proves the F3 seam end-to-end across the real process boundary: the
//! `clock advance`/`set` verbs reach the *same* `MockClock` instance installed at
//! startup under `--features harness-control` (M9.2-D3).
//!
//! **REQUIRES a harness-control node build.** A default build does not contain
//! the clock verbs (the fence, M9.2-D1) — they return `UNKNOWN_COMMAND` and this
//! smoke fails loudly with that signature.
//!
//! ```text
//! cargo build -p xgen-node --features harness-control
//! cargo test -p xgen-mptest --test m9_2_f3_clock -- --ignored --nocapture
//! ```

use std::time::Duration;

use xgen_mptest::aicontrol::{AicontrolClient, DEFAULT_CONNECT_TIMEOUT};
use xgen_mptest::binloc;
use xgen_mptest::process::{aicontrol_pipe, instance_label, Kind, ManagedProcess};
use xgen_mptest::wire::Command;

#[tokio::test]
#[ignore = "heavy: spawns a real harness-control xgen-node; run with --ignored"]
async fn f3_clock_advance_and_set_move_now_utc_deterministically() {
    let bins = binloc::locate().expect("locate built binaries");

    let node_label = instance_label("M9-2-F3", "node");
    let node = ManagedProcess::init_and_spawn_node(&bins, &node_label, 8471, true, None)
        .expect("spawn node");

    let pipe = aicontrol_pipe(Kind::Node, &node_label);
    let mut ctl = AicontrolClient::connect(&pipe, DEFAULT_CONNECT_TIMEOUT)
        .await
        .expect("connect node aicontrol");

    // ── clock advance: two equal advances → now_utc steps by the same delta ──
    let mut adv = Command::new("clock advance");
    adv.args.insert("duration".into(), serde_json::json!("1d"));
    let r1 = ctl.send(&adv).await.expect("clock advance #1");
    assert!(
        r1.is_ok(),
        "clock advance not available — did you build --features harness-control? reply={r1:?}"
    );
    let t1 = parse_utc(r1.data_str("now_utc").expect("now_utc in advance #1 reply"));

    let r2 = ctl.send(&adv).await.expect("clock advance #2");
    assert!(r2.is_ok(), "clock advance #2 failed: {r2:?}");
    let t2 = parse_utc(r2.data_str("now_utc").expect("now_utc in advance #2 reply"));

    let stepped = (t2 - t1).num_seconds();
    assert!(
        (stepped - 86_400).abs() < 5,
        "second advance should move now_utc ~1 day (86400s); moved {stepped}s"
    );

    // ── clock set: absolute set lands exactly ──
    let mut set = Command::new("clock set");
    set.args
        .insert("timestamp".into(), serde_json::json!("2099-01-01T00:00:00.000Z"));
    let r3 = ctl.send(&set).await.expect("clock set");
    assert!(r3.is_ok(), "clock set failed: {r3:?}");
    let now = r3.data_str("now_utc").expect("now_utc in set reply");
    assert!(
        now.starts_with("2099-01-01T00:00:00"),
        "clock set should pin now_utc to the absolute instant; got {now}"
    );

    eprintln!("F3 PASS: advance stepped {stepped}s; set pinned now_utc={now}");

    drop(ctl);
    drop(node);
    tokio::time::sleep(Duration::from_millis(50)).await;
}

fn parse_utc(s: &str) -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::parse_from_rfc3339(s)
        .unwrap_or_else(|e| panic!("now_utc {s:?} is not RFC 3339: {e}"))
        .with_timezone(&chrono::Utc)
}
