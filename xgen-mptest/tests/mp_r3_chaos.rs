// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! MP-R3 capstone — the chaos overlay (`#[ignore]`).
//!
//! MP-R3-D4a two-seam composition: a cooperative chat runs UNDER a stacked chaos
//! task — a relationship-level partition (director / node-conn) with an event
//! flood (raw-WS / parallel task) running during the partition window, then a
//! heal. The director chaos-steps and the parallel chaos task run concurrently on
//! the shared registry, gated through the timeline keys
//! (partition → flood-during-partition → heal). The cooperative Space must
//! converge after the chaos window (rides MP-F11 at heal).
//!
//! The during-chaos **liveness probe** (R3-D4b) runs concurrently and asserts the
//! node stays live throughout the chaos window; convergence is checked via the
//! per-node verdict (R3-D4d).
//!
//! ```text
//! cargo build -p xgen-node --features harness-control && cargo build -p xgen-client
//! cargo test -p xgen-mptest --test mp_r3_chaos -- --ignored --nocapture
//! ```

use std::path::{Path, PathBuf};

use xgen_mptest::dial::{ClockMode, RoundDial};
use xgen_mptest::manifest::Scenario;
use xgen_mptest::oracle::node_convergence_verdict;
use xgen_mptest::runner::{run_scenario, ScenarioOutcome};

fn scenario_dir(id: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("docs/tests/multiparty_scenarios")
        .join(id)
}

async fn run(id: &str) -> ScenarioOutcome {
    let scenario = Scenario::load(scenario_dir(id)).unwrap_or_else(|e| panic!("load {id}: {e:#}"));
    let dial = RoundDial {
        clock: ClockMode::Mock,
        settle_max_secs: Some(60),
        // R3-D4b — run the during-chaos liveness probe.
        liveness_probe: true,
        ..Default::default()
    };
    run_scenario(&scenario, &dial)
        .await
        .unwrap_or_else(|e| panic!("run_scenario({id}): {e:#}"))
}

/// The chaos overlay: node stays live (R3-D4b) AND the chat converges (R3-D4d
/// per-node verdict) after a composed partition + flood + heal window.
#[tokio::test]
#[ignore = "heavy: spawns a harness-control xgen-node ×2 + 2 clients; run with --ignored"]
async fn chaos_overlay_liveness_and_convergence() {
    let o = run("MP-R3-CHAOS").await;

    // R3-D4b — the node stayed live throughout the composed chaos window.
    let liveness = o
        .liveness
        .as_ref()
        .expect("liveness probe requested but no report");
    assert!(
        liveness.all_responsive(),
        "MP-R3-CHAOS: a node went unresponsive under the chaos overlay: {:?}",
        liveness.unresponsive()
    );

    // R3-D4d — the chat converged after the chaos window (per-node verdict).
    let space = o.space_id.as_deref().expect("no primary Space exported");
    let v = node_convergence_verdict(&o.projections, &o.transcripts, space);
    assert!(
        v.pass,
        "MP-R3-CHAOS: the chat did not converge after the composed chaos window \
         (partition + flood + heal) — {}",
        v.detail
    );
    eprintln!(
        "MP-R3-CHAOS PASS: node stayed live under the overlay + chat converged — {}",
        v.detail
    );
}
