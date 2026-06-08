// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! MP-R1 Tranche 3 (C6) — logic-adversarial, batch-expressible (`#[ignore]`).
//!
//! Four authorable batch scenarios (MP-A-02/04/17/20). Of the seven Tranche-3
//! rows: two are recorded BLOCKED in the matrix (MP-A-03 no Space-`auth_tier`
//! verb; MP-A-14 the member-ban verb gap), and MP-A-16 was reclassified to C7 —
//! its "join references a never-issued invite" attack is injector-only (a batch
//! `join` with no invite referenced is a legitimate open-join that succeeds by
//! design — open-join model, runtime.rs:1244 / J-275 — so the batch form is
//! mis-premised; the real attack crafts a join whose `prev_events` reference a
//! fabricated invite, which only the raw-wire injector can do).
//!
//! ## The C6 oracle (Option A — paired rejection)
//! The client `invite`/`join`/`send` ops are **fire-and-forget**: `send_event`
//! writes the frame and never `recv`s the Node's accept/reject (connection.rs:120),
//! so a protocol rejection (3045 / `PermissionDenied` / 4000 / non-member) does
//! **not** reach the aicontrol reply — `run_actor` captures `{status:ok,
//! event_id}` regardless. The category-level "why" is therefore **not**
//! batch-observable; it lives on the C7 wire path (a `WireActor` recvs the Node's
//! `Error` frame, as MP-A-05 Round-0 did). So each scenario asserts the **paired**
//! rejection property, which IS batch-observable:
//! 1. the offending `event_id` (returned by the fire-and-forget op) is **absent**
//!    from every node's transcript (`rejection_verdict` — never applied anywhere);
//! 2. the **protected state is unchanged** (the scenario-specific invariant —
//!    target never a member / no message / no cross-space leak).
//!
//! Absence + state-unchanged together are the sound "the attack had no effect"
//! proof for the R1 floor (absence alone is too weak — benign reasons exist for
//! an absent event). It proves the action was stopped, not *which* rule stopped it
//! (that is C7).
//!
//! ```text
//! cargo build -p xgen-node --features harness-control && cargo build -p xgen-client
//! cargo test -p xgen-mptest --test mp_r1_c6 -- --ignored --nocapture
//! ```

use std::path::{Path, PathBuf};

use xgen_mptest::dial::{ClockMode, RoundDial};
use xgen_mptest::manifest::Scenario;
use xgen_mptest::oracle::{rejection_verdict, MembershipProjection};
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

/// A reply `data` field for a given actor's command (e.g. the offending op's
/// `event_id`, or a registrant's `identity_id`).
fn reply_field<'a>(o: &'a ScenarioOutcome, actor: &str, command: &str, field: &str) -> &'a str {
    o.actor_runs
        .iter()
        .find(|r| r.actor == actor)
        .unwrap_or_else(|| panic!("no run for actor {actor}"))
        .reply_for(command)
        .unwrap_or_else(|| panic!("no reply for {actor}.{command}"))
        .data_str(field)
        .unwrap_or_else(|| panic!("{actor}.{command} reply has no `{field}`"))
}

/// The owner's membership projection of the primary Space (`alice` is the owner
/// in every C6 scenario, so her view is the authoritative resolved membership).
fn alice_view(o: &ScenarioOutcome) -> &MembershipProjection {
    o.projections
        .iter()
        .find(|p| p.node == "alice-view")
        .unwrap_or_else(|| {
            panic!(
                "no `alice-view` projection (views present: {:?})",
                o.projections.iter().map(|p| &p.node).collect::<Vec<_>>()
            )
        })
}

/// The paired rejection oracle: (1) the offending event is absent everywhere;
/// (2) the named target identity never gained membership of the Space.
fn assert_rejected_no_membership(
    o: &ScenarioOutcome,
    attacker: &str,
    attack_cmd: &str,
    target_id: &str,
    what: &str,
) {
    let offending = reply_field(o, attacker, attack_cmd, "event_id");
    let v = rejection_verdict(&o.transcripts, offending);
    assert!(v.pass, "{what}: offending event was applied — {}", v.detail);
    let members = &alice_view(o).members;
    assert!(
        !members.contains_key(target_id),
        "{what}: target `{target_id}` gained membership (state changed): {members:?}"
    );
    eprintln!("{what} PASS: offending {offending} absent on all nodes; target not a member");
}

/// MP-A-02 — over-ceiling invite at submission. The 9999-day invite exceeds the
/// Tier-1 14d ceiling; the Node rejects it at ingest (wire 3045). The invite
/// never lands and bob never becomes a member.
#[tokio::test]
#[ignore = "heavy: spawns a harness-control xgen-node + 2 clients; run with --ignored"]
async fn mp_a_02_over_ceiling_invite_rejected() {
    let o = run("MP-A-02").await;
    let bob_id = reply_field(&o, "bob", "b1", "identity_id");
    assert_rejected_no_membership(&o, "alice", "a4", bob_id, "MP-A-02 over-ceiling invite");
}

/// MP-A-04 — unauthorized non-member send. carol (never a member) posts into S;
/// the Node rejects (step-11 sender-membership). The message lands nowhere and
/// carol never becomes a member.
#[tokio::test]
#[ignore = "heavy: spawns a harness-control xgen-node + 2 clients; run with --ignored"]
async fn mp_a_04_non_member_send_rejected() {
    let o = run("MP-A-04").await;
    let carol_id = reply_field(&o, "carol", "c1", "identity_id");
    assert_rejected_no_membership(&o, "carol", "c2", carol_id, "MP-A-04 non-member send");
}

// MP-A-16 (never-issued invite) was reclassified to C7 (injector) — see the
// module header. A batch `join` cannot reference a fabricated invite predecessor
// (the verb takes no invite arg; the node-side bootstrap does an open-join), and
// an uninvited open-join legitimately succeeds (runtime.rs:1244 / J-275), so the
// batch form is mis-premised. The injector form lands in C7.

/// MP-A-17 — wrong-space_id confusion. carol sends to a non-existent space; the
/// Node rejects (4000). The event lands nowhere and does not leak into the real
/// Space S (S's membership stays exactly `{alice:owner}`).
#[tokio::test]
#[ignore = "heavy: spawns a harness-control xgen-node + 2 clients; run with --ignored"]
async fn mp_a_17_wrong_space_id_no_leak() {
    let o = run("MP-A-17").await;
    let offending = reply_field(&o, "carol", "c2", "event_id");
    let v = rejection_verdict(&o.transcripts, offending);
    assert!(v.pass, "MP-A-17: offending event was applied — {}", v.detail);
    // No cross-space leak: the real Space S has exactly the owner, no new member.
    let members = &alice_view(&o).members;
    assert_eq!(
        members.len(),
        1,
        "MP-A-17: real Space S membership changed (cross-space leak?): {members:?}"
    );
    let carol_id = reply_field(&o, "carol", "c1", "identity_id");
    assert!(
        !members.contains_key(carol_id),
        "MP-A-17: carol leaked into S: {members:?}"
    );
    eprintln!("MP-A-17 PASS: bogus-space event {offending} absent on all nodes; S = {{alice:owner}}, no leak");
}

/// MP-A-20 — privilege escalation (reframed). Member bob attempts the
/// owner/admin-gated `invite` of carol; `can_invite` denies. The escalation-invite
/// never lands and carol never becomes a member (effect-absence — the `permission`
/// category itself is a C7 wire-path assertion, unreachable on batch).
#[tokio::test]
#[ignore = "heavy: spawns a harness-control xgen-node + 3 clients; run with --ignored"]
async fn mp_a_20_member_invite_refused() {
    let o = run("MP-A-20").await;
    let carol_id = reply_field(&o, "carol", "c1", "identity_id");
    assert_rejected_no_membership(&o, "bob", "b3", carol_id, "MP-A-20 privilege escalation");
}
