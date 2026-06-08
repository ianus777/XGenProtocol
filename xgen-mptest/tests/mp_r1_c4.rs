// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! MP-R1 Tranche 1 (C4) — cross-node cooperative core (`#[ignore]`).
//!
//! The riskiest path, first: true A↔B convergence through `run_scenario` + the
//! G-6 bootstrap. Three committed scenarios under
//! `docs/tests/multiparty_scenarios/`:
//! - **MP-C-02** invite & join (alice@A invites; bob@B joins; membership + content converge).
//! - **MP-C-03** concurrent send (both members post; both messages retained + converge).
//! - **MP-C-07** DM private space (create-dm-space; both exchange messages; 2-party converge).
//!
//! All are federated ⇒ require a `--features harness-control` node build. The
//! convergence oracle excludes the asymmetric `state.federation_add` bootstrap
//! event (MP-R1-D7); membership convergence is the positive federation-formed
//! assertion.
//!
//! ```text
//! cargo build -p xgen-node --features harness-control && cargo build -p xgen-client
//! cargo test -p xgen-mptest --test mp_r1_c4 -- --ignored --nocapture
//! ```

use std::path::{Path, PathBuf};

use xgen_mptest::batch::ActorRun;
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
/// converged into both transcripts).
fn event_id_of<'a>(runs: &'a [ActorRun], actor: &str, command: &str) -> &'a str {
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

/// MP-C-02 — invite & join converges true cross-node A↔B.
#[tokio::test]
#[ignore = "heavy: spawns two harness-control xgen-node + 2 clients; run with --ignored"]
async fn mp_c_02_invite_join_converges_cross_node() {
    let o = run("MP-C-02").await;
    assert_all_ok(&o);
    assert!(o.verdict.pass, "MP-C-02 did not converge: {}", o.verdict.detail);
    assert!(o.projections.len() >= 2, "expected ≥2 actor views");
    for p in &o.projections {
        assert_eq!(
            p.members.len(),
            2,
            "view `{}` should have 2 members (alice owner + bob member): {:?}",
            p.node,
            p.members
        );
    }
    eprintln!("MP-C-02 PASS (cross-node A↔B): {}", o.verdict.detail);
}

/// MP-C-03 — both members post concurrently; both messages retained + converge.
#[tokio::test]
#[ignore = "heavy: spawns two harness-control xgen-node + 2 clients; run with --ignored"]
async fn mp_c_03_concurrent_send_both_retained() {
    let o = run("MP-C-03").await;
    assert_all_ok(&o);
    assert!(o.verdict.pass, "MP-C-03 did not converge: {}", o.verdict.detail);
    let space = o.space_id.clone().expect("space_id exported");
    let alice_msg = event_id_of(&o.actor_runs, "alice", "a5");
    let bob_msg = event_id_of(&o.actor_runs, "bob", "b4");
    assert_event_on_all_nodes(&o.transcripts, &space, alice_msg, "alice's message");
    assert_event_on_all_nodes(&o.transcripts, &space, bob_msg, "bob's message");
    eprintln!("MP-C-03 PASS: both messages retained + converge on both nodes");
}

/// MP-C-07 — DM private space across nodes.
///
/// **KNOWN FAIL → routed finding (MP-R1-D6).** This is the committed repro of a
/// real DM-cross-node gap, NOT a convergence proof — it asserts convergence and
/// fails on purpose. Two facets (see the matrix MP-C-07 row + `MP_findings.md`):
/// - **(1) convergence** — Bob's `membership.join` applies on B but never
///   propagates B→A (alice's view stays `{alice:owner}`); DM-specific (MP-C-02
///   propagates B→A under the identical federation).
/// - **(2) observability, open)** — DM `message.text` events are created (send
///   returns an `event_id`) but absent from both nodes' `.events`.
///
/// Routed, not patched (a binary change → out of scope). Run it to reproduce the
/// gap; it stays RED until the fix-arc lands.
///
/// MP-F1a note: facet-2's home-node delivery half is fixed by the client
/// send-confirm retrofit and witnessed by the single-node `MP-C-07-LOCAL`
/// (`mp_r1_c5`). This federated repro stays KNOWN-FAIL — facet-1 (cross-node DM
/// convergence) is MP-F1b. Do not flip it here.
#[tokio::test]
#[ignore = "KNOWN FAIL → routed finding (MP-R1-D6): DM cross-node does not converge; see matrix MP-C-07 / MP_findings.md. Repro only."]
async fn mp_c_07_dm_across_nodes_converges() {
    let o = run("MP-C-07").await;
    assert_all_ok(&o);
    assert!(o.verdict.pass, "MP-C-07 did not converge: {}", o.verdict.detail);
    let space = o.space_id.clone().expect("dm space_id exported");
    let alice_msg = event_id_of(&o.actor_runs, "alice", "a3");
    let bob_msg = event_id_of(&o.actor_runs, "bob", "b4");
    assert_event_on_all_nodes(&o.transcripts, &space, alice_msg, "alice's DM message");
    assert_event_on_all_nodes(&o.transcripts, &space, bob_msg, "bob's DM message");
    eprintln!("MP-C-07 PASS (DM A↔B): 2 parties converge on both messages");
}
