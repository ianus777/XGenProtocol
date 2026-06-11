// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! MP-R3 capstone — MP-A-08 relationship-level partition + heal (`#[ignore]`).
//!
//! MP-R3-D2 + R3-D4a: a `federation defederate` severs A↔B while each side admits
//! a distinct post, then a Heal (`add-peer` + `initiate`) re-establishes
//! federation; after the elastic settle the two nodes must converge.
//!
//! Oracle: `convergence_verdict` = convergence-after-heal + cooperative
//! transcript-set equality (no lost admitted events — both partition-window posts
//! present on both nodes after heal).
//!
//! **Rides MP-F11 (C5):** the heal is a late-establish catch-up onto the
//! re-federated peer; without the regular-Space populate-on-establish + F-3 drain
//! the partition-window posts do not catch up → GREEN only after C5.
//!
//! **HONEST BOUNDARY (R3-D2):** this witnesses convergence-after-heal +
//! no-lost-events. The "no reconnect deadlock" property is the transport-level
//! reconnect scheduler — routed separately, NOT exercised here. MP-A-08 is
//! therefore harness-green-with-boundary (sibling to MP-C-07 / MP-A-01(ii)).
//!
//! ```text
//! cargo build -p xgen-node --features harness-control && cargo build -p xgen-client
//! cargo test -p xgen-mptest --test mp_r3_partition -- --ignored --nocapture
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
        // R3-D4c — a capstone heal + catch-up wants a longer settle ceiling than
        // the 15s default.
        settle_max_secs: Some(60),
        ..Default::default()
    };
    run_scenario(&scenario, &dial)
        .await
        .unwrap_or_else(|e| panic!("run_scenario({id}): {e:#}"))
}

/// MP-A-08 — relationship-level partition converges after heal (no lost events).
#[tokio::test]
#[ignore = "heavy: spawns a harness-control xgen-node ×2 + 2 clients; run with --ignored"]
async fn mp_a_08_relationship_partition_converges_after_heal() {
    let o = run("MP-A-08").await;
    assert!(
        o.verdict.pass,
        "MP-A-08: the two nodes did not converge after the partition heal \
         (a partition-window post was lost, or the re-federated peer did not catch \
         up — needs MP-F11/C5) — {}",
        o.verdict.detail
    );
    eprintln!(
        "MP-A-08 PASS (harness-green-with-boundary): convergence-after-heal + \
         no-lost-admitted-events — {}. (The no-reconnect-deadlock half is the \
         transport-level seam, routed.)",
        o.verdict.detail
    );
}
