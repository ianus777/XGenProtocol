// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! M9.2 F4 smoke — raw malformed-frame injection (MP-A-12, `#[ignore]`).
//!
//! The test-crate-only raw client (M9.2-D4) opens its OWN `tokio-tungstenite`
//! socket and writes a truncated transport frame at the node's frame parser.
//! PASS = the node rejects it at parse and closes the connection cleanly
//! (no hang) AND the node is still alive afterwards (no panic — proven by a
//! fresh `--aicontrol` `state` query succeeding).
//!
//! F4 needs **no** `harness-control` feature and **no** production change — the
//! frame parser is in the normal build and a test-only crate is un-shippable.
//!
//! ```text
//! cargo build -p xgen-node
//! cargo test -p xgen-mptest --test m9_2_f4_malformed_frame -- --ignored --nocapture
//! ```

use std::time::Duration;

use xgen_mptest::aicontrol::{AicontrolClient, DEFAULT_CONNECT_TIMEOUT};
use xgen_mptest::binloc;
use xgen_mptest::injector::inject_malformed_frame;
use xgen_mptest::process::{aicontrol_pipe, instance_label, Kind, ManagedProcess};
use xgen_mptest::wire::Command;

#[tokio::test]
#[ignore = "heavy: spawns the real xgen-node binary; run with --ignored"]
async fn f4_malformed_frame_rejected_at_parse_node_survives() {
    let bins = binloc::locate().expect("locate built binaries");

    let node_label = instance_label("M9-2-F4", "node");
    let node = ManagedProcess::init_and_spawn_node(&bins, &node_label, 8472, true)
        .expect("spawn node");

    // Give the node's WS listener a moment to come up.
    tokio::time::sleep(Duration::from_millis(600)).await;

    // ── Raw malformed frame at the parser ────────────────────────────────────
    let node_url = "ws://127.0.0.1:8472/xgen";
    let outcome = inject_malformed_frame(node_url)
        .await
        .expect("raw malformed-frame injection");
    assert!(outcome.connected, "raw injector failed to reach the node transport");
    assert!(
        outcome.closed,
        "node did not close after the malformed frame (hang?): {}",
        outcome.note
    );
    eprintln!("F4 parse rejection: {}", outcome.note);

    // ── Liveness: the node did not panic — a fresh control session works ─────
    let pipe = aicontrol_pipe(Kind::Node, &node_label);
    let mut ctl = AicontrolClient::connect(&pipe, DEFAULT_CONNECT_TIMEOUT)
        .await
        .expect("connect node aicontrol after attack");
    let state = ctl.send(&Command::new("state")).await.expect("state after attack");
    assert!(
        state.is_ok(),
        "node state query failed after malformed-frame attack (panic?): {state:?}"
    );
    eprintln!("F4 PASS: malformed frame rejected, connection closed, node alive (MP-A-12)");

    drop(ctl);
    drop(node);
    tokio::time::sleep(Duration::from_millis(50)).await;
}
