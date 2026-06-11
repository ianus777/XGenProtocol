// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! MP-R3 capstone — MP-A-06 equivocation via the multi-target injector (`#[ignore]`).
//!
//! The MP-R3-D3 deliverable: one hostile identity reaches BOTH nodes
//! (`kind="injector"`, `nodes=["a","b"]`) and presents two CONFLICTING events at
//! one frontier — fork-A → node a, fork-B → node b. Both forks are individually
//! valid (one hostile key); M8 resolution elects a single winner →
//! **convergence-on-winner** (NOT rejection — MP-A-06). The oracle is the
//! EXISTING `convergence_verdict` (no net-new oracle): if the equivocation forced
//! a permanent fork, the two nodes' membership / transcript sets would diverge and
//! the verdict would fail; convergence-on-winner ⇒ they agree ⇒ verdict passes.
//!
//! ## Payload pinned by observation (R3-D3 — the runbook mechanic)
//! The candidate fork (in `MP-A-06/mallory.jsonl`) is a self-membership leave/join:
//! fork-A `membership.leave` → a, fork-B `membership.join` → b, anchored to the
//! Space root (concurrent on `membership:{space}:{mallory}`; resolution
//! deterministically elects leave, Layer-1 leave>join, proven MP-F7/J-348). The
//! RUN confirms: (i) both forks land (equivocation is not a rejection), (ii)
//! `convergence_verdict` passes (both nodes resolve the same winner), (iii) no
//! permanent fork. **If the RUN falsifies the candidate** (e.g. a non-member leave
//! is rejected on `a` so the forks don't both land / don't both federate), re-pin
//! the conflicting pair in `mallory.jsonl` (other candidates: conflicting
//! `thread.status`, or owner-seated conflicting `state.room_update`) and record the
//! observed winner here. The mechanism (multi-target submit from one key) is fixed;
//! only the payload is observation-pinned.
//!
//! ```text
//! cargo build -p xgen-node --features harness-control && cargo build -p xgen-client
//! cargo test -p xgen-mptest --test mp_r3_equivocation -- --ignored --nocapture
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
        ..Default::default()
    };
    run_scenario(&scenario, &dial)
        .await
        .unwrap_or_else(|e| panic!("run_scenario({id}): {e:#}"))
}

/// MP-A-06 — equivocation converges on a single winner (no permanent fork).
///
/// The multi-target injector presents fork-A → a and fork-B → b at one frontier;
/// after federation + settle the two cooperative readers (alice@a, bob@b) must
/// agree on the resolved membership AND carry the same cooperative event-id set
/// (the `convergence_verdict` — exactly the convergence-on-winner property). A
/// permanent fork makes the two nodes' sets diverge → the verdict fails.
#[tokio::test]
#[ignore = "heavy: spawns a harness-control xgen-node ×2 + 2 clients; run with --ignored"]
async fn mp_a_06_equivocation_converges_on_winner() {
    let o = run("MP-A-06").await;
    assert!(
        o.verdict.pass,
        "MP-A-06: the two nodes did not converge after the equivocation \
         (a permanent fork, or the forks did not both propagate) — {}",
        o.verdict.detail
    );
    eprintln!(
        "MP-A-06 PASS: equivocation converged on a single winner (no permanent fork) — {}",
        o.verdict.detail
    );
}
