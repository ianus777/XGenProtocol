// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! MP-R3 capstone — MP-A-07 flooding intensity *curve* (`#[ignore]`).
//!
//! MP-R3 §6.3: the R2 flood was a single paced liveness witness; R3 sweeps the
//! intensity into a **curve** — a descending paced sequence (`flood_rate_curve`)
//! down to the **firehose** floor (no-drain submit), probing honest-traffic
//! liveness after each rung. The deliverable is the curve + the break-point-per-
//! rate (the rate at which the node stops serving honest traffic), not a single
//! pass/fail. The firehose (`event_flood_firehose`) is the no-drain top of the
//! curve.
//!
//! ```text
//! cargo build -p xgen-node && cargo build -p xgen-client   # single-node, real clock
//! cargo test -p xgen-mptest --test mp_r3_flood -- --ignored --nocapture
//! ```

use std::time::Duration;

use xgen_mptest::aicontrol::{AicontrolClient, DEFAULT_CONNECT_TIMEOUT};
use xgen_mptest::binloc;
use xgen_mptest::churn::{event_flood_firehose, event_flood_mode, flood_rate_curve, FloodMode};
use xgen_mptest::process::{instance_label, ManagedProcess};

const PORT: u16 = 8505;

/// MP-A-07 intensity curve — sweep the flood pace down to the firehose floor;
/// the node must keep serving honest `.aicontrol` traffic at each rung (or the
/// rung is the break-point). Records the curve.
#[tokio::test]
#[ignore = "heavy: spawns a real xgen-node + floods its event-ingest path across a rate curve; box-gated RUN"]
async fn mp_a_07_intensity_curve_liveness() {
    let bins = binloc::locate().expect("locate binaries");
    let label = instance_label("MP-A-07-curve", "node");
    let node = ManagedProcess::init_and_spawn_node(&bins, &label, PORT, true, None)
        .expect("spawn node");
    let mut ctl = AicontrolClient::connect(&node.aicontrol_pipe, DEFAULT_CONNECT_TIMEOUT)
        .await
        .expect("connect aicontrol (node up)");
    let url = format!("ws://127.0.0.1:{PORT}/xgen");

    // The intensity curve: pace 40ms → 0ms (firehose) by 10ms, 5 rungs.
    let curve = flood_rate_curve(40, 10, 5);
    let mut break_point: Option<Duration> = None;
    for (rung, pace) in curve.iter().enumerate() {
        let sent = event_flood_mode(&url, 150, FloodMode::Paced(*pace))
            .await
            .expect("paced flood rung");
        let reply = ctl.send_verb("state").await;
        let live = reply.map(|r| r.is_ok()).unwrap_or(false);
        eprintln!(
            "MP-A-07 curve rung {rung}: pace={pace:?}, sent={sent}, node_live={live}"
        );
        if !live && break_point.is_none() {
            break_point = Some(*pace);
        }
    }

    // The firehose floor (no-drain) — the top of the curve.
    let fh = event_flood_firehose(&url, 300).await.expect("firehose flood");
    let reply = ctl.send_verb("state").await;
    let live = reply.map(|r| r.is_ok()).unwrap_or(false);
    eprintln!("MP-A-07 firehose: sent={fh}, node_live={live}");

    eprintln!(
        "MP-A-07 intensity curve: break_point_pace={break_point:?} (None ⇒ node stayed live across the whole curve)"
    );
    // The curve is the deliverable; the floor assertion is that a paced (gentle)
    // flood keeps the node live — a regression that wedges the node even at the
    // gentlest rung is the real finding.
    let gentle = event_flood_mode(&url, 50, FloodMode::Paced(Duration::from_millis(40)))
        .await
        .expect("gentle flood");
    let reply = ctl.send_verb("state").await.expect("state after gentle flood");
    assert!(
        reply.is_ok(),
        "node did NOT stay live after a gentle paced flood ({gentle} events) — a real finding"
    );
    eprintln!("MP-A-07 PASS: intensity curve recorded; node serves honest traffic at the gentle floor");
}
