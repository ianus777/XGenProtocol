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
//! The during-chaos **liveness probe** (R3-D4b) is added in C2b; this C2a smoke
//! asserts convergence-after-chaos. C2b extends it with the node-stays-live
//! assertion (the full capstone overlay witness).
//!
//! ```text
//! cargo build -p xgen-node --features harness-control && cargo build -p xgen-client
//! cargo test -p xgen-mptest --test mp_r3_chaos -- --ignored --nocapture
//! ```

use std::path::{Path, PathBuf};

use xgen_mptest::dial::{ClockMode, RoundDial};
use xgen_mptest::manifest::Scenario;
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
        ..Default::default()
    };
    run_scenario(&scenario, &dial)
        .await
        .unwrap_or_else(|e| panic!("run_scenario({id}): {e:#}"))
}

/// The chaos overlay converges after a composed partition + flood + heal window —
/// the two-seam composition (director partition/heal + parallel flood) runs
/// concurrently with the chat and the Space still converges.
#[tokio::test]
#[ignore = "heavy: spawns a harness-control xgen-node ×2 + 2 clients; run with --ignored"]
async fn chaos_overlay_converges_after_composed_chaos() {
    let o = run("MP-R3-CHAOS").await;
    assert!(
        o.verdict.pass,
        "MP-R3-CHAOS: the chat did not converge after the composed chaos window \
         (partition + flood + heal) — {}",
        o.verdict.detail
    );
    eprintln!(
        "MP-R3-CHAOS PASS: composed partition + flood + heal, chat converged — {}",
        o.verdict.detail
    );
}
