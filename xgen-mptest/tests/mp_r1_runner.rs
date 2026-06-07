// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! MP-R1 C1 proof — the generic [`run_scenario`] orchestrator (`#[ignore]`).
//!
//! Two smokes prove the C1 machinery end-to-end against the real binaries:
//!
//! 1. **Single-node** — the committed Round-0 `MP-C-02` scenario driven through
//!    `run_scenario` instead of the hand-wired `c5_mp_c_02.rs`. Same result:
//!    invite & join converge (alice owner + bob member on both views). Proves the
//!    runner reproduces the hand-wired flow over the manifest/batch surface.
//!    Real clock, no federation → runs on a default node build.
//!
//! 2. **Two-node federation** — a fresh A↔B scenario (written to a tempdir)
//!    exercising the G-6 bootstrap helper (MP-R1-D1a): seed both directions →
//!    actors register (identities replicate **both** ways) → owner creates the
//!    Space → re-seed naming it → `initiate`. Asserts the *machinery*: bob's
//!    identity reaches A (alice's invite of bob succeeds), and alice's Space
//!    replicates A→B (B's transcript carries it). The full true-cross-node
//!    membership convergence (bob joining on B) is the C4/T1 deliverable that
//!    builds on this. Mock clock → requires `--features harness-control`.
//!
//! ```text
//! cargo build -p xgen-node -p xgen-client && cargo build -p xgen-node --features harness-control
//! cargo test -p xgen-mptest --test mp_r1_runner -- --ignored --nocapture
//! ```

use std::path::Path;

use xgen_mptest::dial::{ClockMode, RoundDial};
use xgen_mptest::manifest::Scenario;
use xgen_mptest::runner::run_scenario;

const PORT_A: u16 = 8482;
const PORT_B: u16 = 8483;

/// Smoke 1 — the committed single-node MP-C-02, re-expressed through the runner.
#[tokio::test]
#[ignore = "heavy: spawns real xgen-node + 2 xgen-client residents; run with --ignored"]
async fn mp_c_02_single_node_through_run_scenario() {
    let scenario_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("docs/tests/multiparty_scenarios/MP-C-02");
    let scenario = Scenario::load(&scenario_dir).expect("load committed MP-C-02 scenario");

    let dial = RoundDial {
        clock: ClockMode::Real,
        ..Default::default()
    };
    let outcome = run_scenario(&scenario, &dial)
        .await
        .expect("run_scenario(MP-C-02)");

    for run in &outcome.actor_runs {
        assert!(
            run.all_ok(),
            "actor `{}` had a failed command: {:?}",
            run.actor,
            run.replies
        );
    }

    eprintln!("MP-C-02 (via run_scenario) oracle: {}", outcome.verdict.detail);
    assert!(
        outcome.verdict.pass,
        "MP-C-02 convergence FAILED through run_scenario: {}",
        outcome.verdict.detail
    );
    assert!(
        outcome.projections.len() >= 2,
        "expected ≥2 actor views, got {}",
        outcome.projections.len()
    );
    for p in &outcome.projections {
        assert_eq!(
            p.members.len(),
            2,
            "view `{}` should have 2 members (alice owner + bob member): {:?}",
            p.node,
            p.members
        );
    }
    eprintln!("MP-C-02 PASS through run_scenario: both views converge on 2 members");
}

/// Smoke 2 — the 2-node federation machinery: G-6 bootstrap + cross-node
/// identity replication + A→B Space replication.
#[tokio::test]
#[ignore = "heavy: spawns two real harness-control xgen-node + 2 clients; run with --ignored"]
async fn two_node_federation_machinery_through_run_scenario() {
    let tmp = tempfile::tempdir().expect("tempdir");
    write_fed_scenario(tmp.path());
    let scenario = Scenario::load(tmp.path()).expect("load 2-node federation scenario");

    let dial = RoundDial {
        clock: ClockMode::Mock,
        ..Default::default()
    };
    let outcome = run_scenario(&scenario, &dial)
        .await
        .expect("run_scenario(2-node federation)");

    // Alice's batch must complete — crucially her `invite` of bob, which requires
    // bob's identity to have replicated B→A (proves bidirectional identity
    // replication via the G-6 seed).
    let alice = outcome
        .actor_runs
        .iter()
        .find(|r| r.actor == "alice")
        .expect("alice run");
    assert!(
        alice.all_ok(),
        "alice had a failed command (invite needs bob's id on A): {:?}",
        alice.replies
    );
    let bob = outcome
        .actor_runs
        .iter()
        .find(|r| r.actor == "bob")
        .expect("bob run");
    assert!(bob.all_ok(), "bob register failed: {:?}", bob.replies);

    // The owner's Space must replicate A→B (the federation push after initiate).
    let space = outcome
        .space_id
        .clone()
        .expect("alice exported a space_id");
    let tb = outcome
        .transcripts
        .iter()
        .find(|t| t.node == "b")
        .expect("node b transcript");
    assert!(
        !tb.event_ids_for_space(&space).is_empty(),
        "Space {space} did NOT replicate A→B — B's transcript has no events for it: {:?}",
        tb.events
    );
    eprintln!(
        "2-node federation PASS: bidirectional identity replication (alice invited bob) + \
         Space {space} replicated A→B (machinery proven; true cross-node convergence is C4)"
    );
}

/// Write a minimal A↔B federation scenario into `dir`.
fn write_fed_scenario(dir: &Path) {
    let manifest = format!(
        r#"
scenario = "MP-R1-FED-SMOKE"
description = "C1 2-node federation machinery proof"

[[nodes]]
label = "a"
port = {PORT_A}

[[nodes]]
label = "b"
port = {PORT_B}

[[federation]]
from = "a"
to = "b"

[[actors]]
name = "alice"
node = "a"
batch = "alice.jsonl"

[[actors]]
name = "bob"
node = "b"
batch = "bob.jsonl"

[[exports]]
actor = "bob"
command = "b1"
field = "identity_id"
key = "bob_identity_id"

[[exports]]
actor = "alice"
command = "a2"
field = "space_id"
key = "space_id"
"#
    );
    // alice: register, create the shared Space + a room, invite bob (member).
    // The invite blocks on bob's exported identity (cross-node), proving B→A
    // identity replication when it succeeds.
    let alice = "\
{\"cmd\":\"register\",\"args\":{\"name\":\"alice\"},\"id\":\"a1\"}\n\
{\"cmd\":\"create-space\",\"args\":{\"name\":\"FED-S\"},\"id\":\"a2\",\"bind\":\"s\"}\n\
{\"cmd\":\"create-room\",\"args\":{\"space\":\"$s\",\"name\":\"general\"},\"id\":\"a3\",\"bind\":\"r\"}\n\
{\"cmd\":\"invite\",\"args\":{\"space\":\"$s\",\"identity\":\"{{bob_identity_id}}\",\"role\":\"member\"},\"id\":\"a4\"}\n";
    // bob: register only (his identity replicates A←B; the cross-node join is C4).
    let bob = "{\"cmd\":\"register\",\"args\":{\"name\":\"bob\"},\"id\":\"b1\"}\n";

    std::fs::write(dir.join("manifest.toml"), manifest).expect("write manifest");
    std::fs::write(dir.join("alice.jsonl"), alice).expect("write alice batch");
    std::fs::write(dir.join("bob.jsonl"), bob).expect("write bob batch");
}
