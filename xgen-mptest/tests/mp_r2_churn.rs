// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! MP-R2 C4 proof — the connection-churn driver (`#[ignore]`).
//!
//! The storm **plan** is unit-tested in `churn.rs` without sockets. These smokes
//! prove the driver against a live node: a connect/disconnect storm (MP-A-18) +
//! held slow-loris connections (MP-A-19) must not take the node down — a
//! legitimate `.aicontrol` command still lands afterward (the M8.6 C4 attempt-
//! gauge property, observed at the binary as node liveness).
//!
//! The churn hits the **WS transport**; the `.aicontrol` named pipe is a separate
//! surface, so the post-churn `state` probe is an honest cross-surface liveness
//! check. **Box-gated (RUN gate, M-R2.3).** Rule 2: a connect-timeout under load
//! is a flake (re-run isolated) — a node that stops serving `.aicontrol` is the
//! real finding.
//!
//! ```text
//! cargo build -p xgen-node --features harness-control && cargo build -p xgen-client
//! cargo test -p xgen-mptest --test mp_r2_churn -- --ignored --nocapture
//! ```

use std::time::Duration;

use xgen_mptest::aicontrol::{AicontrolClient, DEFAULT_CONNECT_TIMEOUT};
use xgen_mptest::binloc;
use xgen_mptest::churn::{self, StormPlan};
use xgen_mptest::process::{instance_label, ManagedProcess};

const PORT_A18: u16 = 8491;
const PORT_A19: u16 = 8492;

#[tokio::test]
#[ignore = "heavy: spawns a real xgen-node + storms its WS transport; box-gated RUN"]
async fn mp_a_18_connect_disconnect_storm_node_stays_live() {
    let bins = binloc::locate().expect("locate binaries");
    let label = instance_label("MP-A-18", "node");
    let node = ManagedProcess::init_and_spawn_node(&bins, &label, PORT_A18, true, None)
        .expect("spawn node");

    // Connect the .aicontrol pipe first (retries until the node is up) — proves
    // the node is serving BEFORE the storm, so a post-storm failure is the storm's.
    let mut ctl = AicontrolClient::connect(&node.aicontrol_pipe, DEFAULT_CONNECT_TIMEOUT)
        .await
        .expect("connect aicontrol (node up)");

    let url = format!("ws://127.0.0.1:{PORT_A18}/xgen");
    let plan = StormPlan {
        cycles: 5,
        conns_per_cycle: 20,
    };
    let opened = churn::run_storm(&url, plan).await.expect("run storm");
    eprintln!("MP-A-18 storm: {opened} opens across {} cycles", plan.cycles);

    // The node must still serve a legitimate command over .aicontrol.
    let reply = ctl.send_verb("state").await.expect("state after storm");
    assert!(
        reply.is_ok(),
        "node did NOT stay live after the connect/disconnect storm: {reply:?}"
    );
    eprintln!("C4 MP-A-18 PASS: node stayed live after {opened} churned connections");
}

#[tokio::test]
#[ignore = "heavy: spawns a real xgen-node + holds slow-loris connections; box-gated RUN"]
async fn mp_a_19_slow_loris_does_not_exhaust_node() {
    let bins = binloc::locate().expect("locate binaries");
    let label = instance_label("MP-A-19", "node");
    let node = ManagedProcess::init_and_spawn_node(&bins, &label, PORT_A19, true, None)
        .expect("spawn node");

    let mut ctl = AicontrolClient::connect(&node.aicontrol_pipe, DEFAULT_CONNECT_TIMEOUT)
        .await
        .expect("connect aicontrol (node up)");

    let url = format!("ws://127.0.0.1:{PORT_A19}/xgen");
    // Open many connections and HOLD them (idle, no traffic) while we probe.
    let held = churn::slow_loris(&url, 50, Duration::from_secs(2))
        .await
        .expect("slow-loris hold");
    eprintln!("MP-A-19 slow-loris: holding {} connections", held.len());

    // While the connections are still held, the node must serve honest traffic.
    let reply = ctl.send_verb("state").await.expect("state under slow-loris");
    assert!(
        reply.is_ok(),
        "node was exhausted by held slow-loris connections (honest traffic refused): {reply:?}"
    );
    drop(held); // release the held connections
    eprintln!("C4 MP-A-19 PASS: honest traffic served while held connections were open");
}
