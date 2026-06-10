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

/// MP-C-07 — DM private space across nodes. **Harness-green-with-boundary (MP-F1b).**
///
/// DM federation forms when the parties' home nodes are **resolvable** (Design Z).
/// The harness G-6-seeds the relationship + replicates identities, so both parties
/// resolve → `repopulate_dm_federation_nodes` populates each DM's `federation_nodes`
/// from its **parties** (members ∪ pending invitees, invariant E), so the
/// counterparty's home is in the set **from create**. Effect: the bootstrap
/// `membership.join` passes the receiving F-3 gate **with no skip** (the joiner's
/// home is already present), and the creator's pre-join message (`a3`) pushes to
/// the counterparty immediately — both DM messages converge A↔B. The
/// identity-replicate hook (`repopulate_dm_federation_after_identity`) re-fires +
/// drains any F-3-held join if the counterparty's record lands late.
///
/// **Production convergence to a not-yet-known counterparty is DEFERRED** behind
/// the routed "production identity→home-node discovery" arc (F1B-D5): a fresh
/// production DM to a stranger whose `IdentityRecord` has not replicated cannot
/// resolve their home node (gate-B), so that party is omitted (F1B-D3) and the DM
/// does not federate to them until discovery lands. The un-seeded case is **not
/// expressible on current G-6 rails** (sibling to MP-A-01(ii) PENDING) → **no
/// production witness is claimed**, by design (a clean green there would be the
/// misleading-positive the test-integrity principle forbids).
///
/// RED-on-revert (demonstrated): neuter the Z population → DM `federation_nodes`
/// stays empty → bob's `membership.join` is F-3-held on A (B not in the set) and
/// never applies → membership diverges (alice-view stays `{alice}`) → RED.
#[tokio::test]
#[ignore = "heavy: spawns two harness-control xgen-node + 2 clients; run with --ignored"]
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
