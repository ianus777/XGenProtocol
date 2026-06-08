// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! MP-R1 Tranche 2 (C5) — membership-lifecycle cooperative (`#[ignore]`).
//!
//! Two of the six Tranche-2 scenarios are authorable on the client-verb rails
//! and shipped here; the other four (MP-C-06 / MP-C-08 / MP-C-09 / MP-C-13) are
//! recorded BLOCKED in `docs/tests/MULTIPARTY_TEST_MATRIX.md` (no client
//! authoring verb / harness-capability gap — out of MP-R1 scope, design §8):
//! - **MP-C-01** multi-client local single-node fan-out (alice + carol on Node A;
//!   both members; both posts seen by both — per-client `state` + `.events`).
//! - **MP-C-10** leave & rejoin, cross-node A↔B (bob joins, leaves, is
//!   re-invited, rejoins; membership converges to `{alice:owner, bob:member}`).
//!
//! Both use the Mock-clock dial ⇒ require a `--features harness-control` node
//! build. The convergence oracle excludes the asymmetric `state.federation_add`
//! bootstrap event (MP-R1-D7); membership convergence is the positive
//! federation-formed assertion.
//!
//! ```text
//! cargo build -p xgen-node --features harness-control && cargo build -p xgen-client
//! cargo test -p xgen-mptest --test mp_r1_c5 -- --ignored --nocapture
//! ```

use std::path::{Path, PathBuf};

use xgen_mptest::dial::{ClockMode, RoundDial};
use xgen_mptest::manifest::Scenario;
use xgen_mptest::oracle::Transcript;
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

fn assert_all_ok(outcome: &ScenarioOutcome) {
    for r in &outcome.actor_runs {
        assert!(
            r.all_ok(),
            "actor `{}` had a failed command: {:?}",
            r.actor,
            r.replies
        );
    }
}

/// The `event_id` a given actor's command produced (for asserting a message
/// converged into a transcript).
fn event_id_of<'a>(
    runs: &'a [xgen_mptest::batch::ActorRun],
    actor: &str,
    command: &str,
) -> &'a str {
    runs.iter()
        .find(|r| r.actor == actor)
        .unwrap_or_else(|| panic!("no run for actor {actor}"))
        .reply_for(command)
        .unwrap_or_else(|| panic!("no reply for {actor}.{command}"))
        .data_str("event_id")
        .unwrap_or_else(|| panic!("{actor}.{command} reply has no event_id"))
}

/// Assert `event_id` is in every node's cooperative event set for the Space.
fn assert_event_on_all_nodes(transcripts: &[Transcript], space: &str, event_id: &str, what: &str) {
    for t in transcripts {
        assert!(
            t.cooperative_event_ids_for_space(space).contains(event_id),
            "{what} ({event_id}) missing from node `{}` cooperative set: {:?}",
            t.node,
            t.cooperative_event_ids_for_space(space)
        );
    }
}

/// MP-C-01 — multi-client local single-node fan-out.
///
/// Single node, two clients. The convergence claim is per-client: both client
/// views of S agree on `{alice:owner, carol:member}`, and each client sees the
/// other's message (both posts in Node A's cooperative event set). With one node
/// the cross-node transcript set-equality is N/A; the signal is the two
/// projections agreeing + both messages present.
#[tokio::test]
#[ignore = "heavy: spawns a harness-control xgen-node + 2 clients; run with --ignored"]
async fn mp_c_01_local_fanout_converges() {
    let o = run("MP-C-01").await;
    assert_all_ok(&o);
    assert!(o.verdict.pass, "MP-C-01 did not converge: {}", o.verdict.detail);
    assert!(o.projections.len() >= 2, "expected ≥2 client views");
    for p in &o.projections {
        assert_eq!(
            p.members.len(),
            2,
            "view `{}` should have 2 members (alice owner + carol member): {:?}",
            p.node,
            p.members
        );
    }
    let space = o.space_id.clone().expect("space_id exported");
    let alice_msg = event_id_of(&o.actor_runs, "alice", "a5");
    let carol_msg = event_id_of(&o.actor_runs, "carol", "c4");
    assert_event_on_all_nodes(&o.transcripts, &space, alice_msg, "alice's message");
    assert_event_on_all_nodes(&o.transcripts, &space, carol_msg, "carol's message");
    eprintln!("MP-C-01 PASS (single-node local fan-out): {}", o.verdict.detail);
}

/// MP-C-10 — leave & rejoin converges true cross-node A↔B.
///
/// bob joins, leaves, is re-invited, and rejoins — each act originating on B and
/// propagating B→A. The final resolved membership must converge on both nodes to
/// `{alice:owner, bob:member}`, and the cooperative event-id set must match.
#[tokio::test]
#[ignore = "heavy: spawns two harness-control xgen-node + 2 clients; run with --ignored"]
async fn mp_c_10_leave_and_rejoin_converges() {
    let o = run("MP-C-10").await;
    assert_all_ok(&o);
    assert!(o.verdict.pass, "MP-C-10 did not converge: {}", o.verdict.detail);
    assert!(o.projections.len() >= 2, "expected ≥2 node views");
    for p in &o.projections {
        assert_eq!(
            p.members.len(),
            2,
            "after rejoin, view `{}` should have 2 members (alice owner + bob member): {:?}",
            p.node,
            p.members
        );
    }
    eprintln!("MP-C-10 PASS (leave & rejoin, cross-node A↔B): {}", o.verdict.detail);
}
