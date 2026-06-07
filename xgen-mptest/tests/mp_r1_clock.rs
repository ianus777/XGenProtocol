// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! MP-R1 C3 proof — the clock-director machinery (`#[ignore]`).
//!
//! The `[[clock]]` parse + the `rejection_category_verdict` helper are unit-tested
//! in `manifest.rs` / `oracle.rs` without spawning. This smoke proves the
//! *director* path: a `[[clock]]` step, gated on a drive-published export key,
//! fires the fenced F3 `clock advance` verb against a harness-control node and
//! the run completes (the director `require_ok`s it). The full clock *scenario*
//! (MP-A-01: advance past an invite's `valid_until`, then replay) is C7.
//!
//! ```text
//! cargo build -p xgen-node --features harness-control && cargo build -p xgen-client
//! cargo test -p xgen-mptest --test mp_r1_clock -- --ignored --nocapture
//! ```

use std::path::Path;

use xgen_mptest::dial::{ClockMode, RoundDial};
use xgen_mptest::manifest::Scenario;
use xgen_mptest::runner::run_scenario;

const PORT_A: u16 = 8484;

#[tokio::test]
#[ignore = "heavy: spawns a real harness-control xgen-node + a client; run with --ignored"]
async fn clock_director_fires_advance_gated_on_export() {
    let tmp = tempfile::tempdir().expect("tempdir");
    write_clock_scenario(tmp.path());
    let scenario = Scenario::load(tmp.path()).expect("load clock scenario");

    // Mock clock ⇒ run_scenario probes harness-control up front; the [[clock]]
    // advance step then drives the same injected MockClock.
    let dial = RoundDial {
        clock: ClockMode::Mock,
        ..Default::default()
    };
    // A successful return proves the director fired `clock advance 15d` (gated on
    // alice's exported space_id) and the node accepted it (`require_ok`). A
    // default (non-harness) node would make the run error here.
    let outcome = run_scenario(&scenario, &dial)
        .await
        .expect("run_scenario(clock) — did you build the node --features harness-control?");

    let alice = outcome
        .actor_runs
        .iter()
        .find(|r| r.actor == "alice")
        .expect("alice run");
    assert!(alice.all_ok(), "alice batch failed: {:?}", alice.replies);
    assert!(
        outcome.space_id.is_some(),
        "alice should have exported a space_id (the clock step's `after` gate)"
    );
    eprintln!("C3 clock-director PASS: `clock advance 15d` fired (gated on space_id) against the harness-control node");
}

/// A single-node scenario with one `[[clock]]` advance step gated on `space_id`.
fn write_clock_scenario(dir: &Path) {
    let manifest = format!(
        r#"
scenario = "MP-R1-CLOCK-SMOKE"
description = "C3 clock-director machinery proof"

[[nodes]]
label = "a"
port = {PORT_A}

[[actors]]
name = "alice"
node = "a"
batch = "alice.jsonl"

[[exports]]
actor = "alice"
command = "a2"
field = "space_id"
key = "space_id"

[[clock]]
node = "a"
op = "advance"
value = "15d"
after = "space_id"
"#
    );
    let alice = "\
{\"cmd\":\"register\",\"args\":{\"name\":\"alice\"},\"id\":\"a1\"}\n\
{\"cmd\":\"create-space\",\"args\":{\"name\":\"CLOCK-S\"},\"id\":\"a2\",\"bind\":\"s\"}\n";

    std::fs::write(dir.join("manifest.toml"), manifest).expect("write manifest");
    std::fs::write(dir.join("alice.jsonl"), alice).expect("write alice batch");
}
