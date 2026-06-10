// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! MP-R1 C2 proof — the single-rung sweep engine (`#[ignore]`).
//!
//! The classifier (GREEN/LOGIC-FAULT/CEILING) is unit-tested in `sweep.rs`
//! without spawning. This smoke proves the *engine* path: `run_sweep` wraps
//! `run_scenario` for a degenerate single-rung [`Sweep`] and assembles a
//! [`SweepResult`] — one GREEN rung, no break-point — against the committed
//! MP-C-02 (true A↔B since C4). The multi-rung climb is exercised by R2/R3.
//!
//! ```text
//! cargo build -p xgen-node --features harness-control && cargo build -p xgen-client
//! cargo test -p xgen-mptest --test mp_r1_sweep -- --ignored --nocapture
//! ```

use std::path::Path;

use xgen_mptest::dial::{ClockMode, RoundDial};
use xgen_mptest::manifest::Scenario;
use xgen_mptest::sweep::{run_sweep, CeilingFloors, RungClass, ScenarioTemplate, Sweep, SweepAxis};

#[tokio::test]
#[ignore = "heavy: spawns real xgen-node + 2 xgen-client residents; run with --ignored"]
async fn single_rung_sweep_over_mp_c_02_is_one_green_rung() {
    let scenario_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("docs/tests/multiparty_scenarios/MP-C-02");
    let scenario = Scenario::load(&scenario_dir).expect("load committed MP-C-02 scenario");

    let base = RoundDial {
        // MP-C-02 is federated (A↔B) since C4 → harness-control build required.
        clock: ClockMode::Mock,
        ..Default::default()
    };
    // Degenerate single rung — R1's use of the sweep contract. Under the MP-R2
    // C1 evolution `run_sweep` takes a `ScenarioTemplate`; R1's fixed scenario
    // wraps in `Fixed` (the dial is ignored — single-rung, manifest-authoritative).
    let template = ScenarioTemplate::Fixed(scenario);
    let sweep = Sweep::single(SweepAxis::Clients, 2);

    // R1 uses the coarse default floors (no bench calibration this round).
    let result = run_sweep(&template, &sweep, &base, &CeilingFloors::default())
        .await
        .expect("run_sweep(MP-C-02, single rung)");

    assert_eq!(result.rungs.len(), 1, "single-rung sweep must have exactly 1 rung");
    assert_eq!(
        result.rungs[0].class,
        RungClass::Green,
        "MP-C-02 rung should classify GREEN: verdict={}",
        result.rungs[0].verdict.detail
    );
    assert!(
        result.all_green(),
        "single GREEN rung must leave no break-point: {:?}",
        result.break_point
    );
    eprintln!(
        "C2 single-rung sweep PASS: 1 GREEN rung, no break-point ({})",
        result.rungs[0].verdict.detail
    );
}
