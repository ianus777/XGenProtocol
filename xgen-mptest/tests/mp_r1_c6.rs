// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! MP-R1 Tranche 3 (C6) — logic-adversarial, batch-expressible (`#[ignore]`).
//!
//! Five authorable batch scenarios (MP-A-02/03/04/17/20). MP-A-03 joined this
//! tranche at MP-F5 (the `create-space --auth-tier` verb shipped, J-334 arc 1).
//! Of the remaining Tranche-3 rows: MP-A-14 is BLOCKED (the member-ban verb gap),
//! and MP-A-16 was reclassified to C7 — its "join references a never-issued
//! invite" attack is injector-only (a batch `join` with no invite referenced is a
//! legitimate open-join that succeeds by design — open-join model, runtime.rs:1244
//! / J-275 — so the batch form is mis-premised; the real attack crafts a join
//! whose `prev_events` reference a fabricated invite, which only the raw-wire
//! injector can do).
//!
//! ## The C6 oracle (assert-the-reject — MP-F5, supersedes the pre-MP-F2 read)
//! **History (the stale premise).** Pre-MP-F1a/MP-F2 the client ops were
//! fire-and-forget (no recv of the node's accept/reject), so a rejection did not
//! reach the aicontrol reply — the oracle read `{status:ok, event_id}` and
//! asserted only effect-absence. MP-F1a (await-confirm) + MP-F2 (`reject_signal`,
//! node sends code + event_id) + **MP-F5** (the client surfaces them structurally)
//! changed that: a locally-submitted single-event reject now returns an `Error`
//! reply carrying `reject_code` (the wire code, e.g. 3030/3045) + `event_id`. The
//! reject IS batch-observable (MP-R1-D9 amended, favorably).
//!
//! So each scenario now **asserts the reject directly** (`assert_rejected_no_membership`):
//! 1. the offending op's reply is an `Error` with a structured `reject_code`
//!    (== the expected wire code when pinned, else present) + an `event_id`;
//! 2. the offending event is **absent** from every node's transcript (via that
//!    `event_id` — `rejection_verdict`, never applied anywhere);
//! 3. the **protected state is unchanged** (target never a member / no message /
//!    no cross-space leak).
//!
//! Reject-surfaced + absence + state-unchanged together are the sound "the attack
//! was stopped, by this rule, and had no effect" proof for the R1 floor. (The C7
//! wire path remains the place that recvs the raw `Error` frame directly.)
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

/// The assert-the-reject oracle (MP-F5-D3, post-MP-F5 reject-surfacing —
/// supersedes the pre-MP-F1a/MP-F2 fire-and-forget read). A rejected single-event
/// op now surfaces an `Error` reply carrying `reject_code` + `event_id`. Asserts:
/// (1) the offending op's reply is an `Error` with a structured `reject_code`
///     (== `expect_code` when given, else just present) + an `event_id`;
/// (2) the offending event is absent from every node's transcript (via that
///     `event_id`);
/// (3) the named target identity never gained membership of the Space.
fn assert_rejected_no_membership(
    o: &ScenarioOutcome,
    attacker: &str,
    attack_cmd: &str,
    target_id: &str,
    expect_code: Option<u32>,
    what: &str,
) {
    let reply = o
        .actor_runs
        .iter()
        .find(|r| r.actor == attacker)
        .unwrap_or_else(|| panic!("no run for actor {attacker}"))
        .reply_for(attack_cmd)
        .unwrap_or_else(|| panic!("no reply for {attacker}.{attack_cmd}"));
    let err = reply.error().unwrap_or_else(|| {
        panic!("{what}: {attacker}.{attack_cmd} reply was Ok — expected a node reject (Error). Reply: {reply:?}")
    });
    let offending = err.event_id.as_deref().unwrap_or_else(|| {
        panic!("{what}: reject reply carries no event_id (MP-F5 surfacing missing): {err:?}")
    });
    match expect_code {
        Some(code) => assert_eq!(
            err.reject_code,
            Some(code),
            "{what}: expected wire reject_code {code}; got {:?} (message: {})",
            err.reject_code,
            err.message
        ),
        None => assert!(
            err.reject_code.is_some(),
            "{what}: expected a structured wire reject_code; got none (message: {})",
            err.message
        ),
    }
    let v = rejection_verdict(&o.transcripts, offending);
    assert!(v.pass, "{what}: offending event was applied — {}", v.detail);
    let members = &alice_view(o).members;
    assert!(
        !members.contains_key(target_id),
        "{what}: target `{target_id}` gained membership (state changed): {members:?}"
    );
    eprintln!(
        "{what} PASS: reject_code={:?}; offending {offending} absent on all nodes; target not a member",
        err.reject_code
    );
}

/// MP-A-02 — over-ceiling invite at submission. The 9999-day invite exceeds the
/// Tier-1 14d ceiling; the Node rejects it at ingest (wire 3045). The invite
/// never lands and bob never becomes a member.
#[tokio::test]
#[ignore = "heavy: spawns a harness-control xgen-node + 2 clients; run with --ignored"]
async fn mp_a_02_over_ceiling_invite_rejected() {
    let o = run("MP-A-02").await;
    let bob_id = reply_field(&o, "bob", "b1", "identity_id");
    // Wire 3045 invite_validity_exceeds_max (MP-F5: now structurally surfaced).
    assert_rejected_no_membership(&o, "alice", "a4", bob_id, Some(3045), "MP-A-02 over-ceiling invite");
}

/// MP-A-03 — tier-gate join refusal (PG-13). alice creates a Tier-2 Space
/// (`create-space --auth-tier 2`, the J-334 arc-1 verb); bob (Tier-1, no
/// assertion → `assertion_tier_of` = 1) attempts to join. The PG-13 step-4 gate
/// refuses with wire 3030 `tier_mismatch`; the join lands nowhere and bob never
/// becomes a member. This is the auth-tier verb's batch witness — deferred from
/// arc 1 to MP-F5 because it depends on the reject-surfacing oracle this arc adds.
#[tokio::test]
#[ignore = "heavy: spawns a harness-control xgen-node + 2 clients; run with --ignored"]
async fn mp_a_03_tier_gate_join_refused() {
    let o = run("MP-A-03").await;
    let bob_id = reply_field(&o, "bob", "b1", "identity_id");
    assert_rejected_no_membership(&o, "bob", "b2", bob_id, Some(3030), "MP-A-03 tier-gate join refusal");
}

/// MP-A-04 — unauthorized non-member send. carol (never a member) posts into S;
/// the Node rejects (step-11 sender-membership). The message lands nowhere and
/// carol never becomes a member.
#[tokio::test]
#[ignore = "heavy: spawns a harness-control xgen-node + 2 clients; run with --ignored"]
async fn mp_a_04_non_member_send_rejected() {
    let o = run("MP-A-04").await;
    let carol_id = reply_field(&o, "carol", "c1", "identity_id");
    // Step-11 sender-membership reject. Empirically wire 4000 (an unmapped
    // ExchangeError variant — MP-F2-D3 boundary / MP-F2-followon will assign a
    // specific code; pinned to the observed value so a remap re-grounds it here).
    assert_rejected_no_membership(&o, "carol", "c2", carol_id, Some(4000), "MP-A-04 non-member send");
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
    // MP-F5: the bogus-space send is rejected → Error reply with event_id.
    let reply = o
        .actor_runs
        .iter()
        .find(|r| r.actor == "carol")
        .expect("no run for carol")
        .reply_for("c2")
        .expect("no reply for carol.c2");
    let err = reply
        .error()
        .unwrap_or_else(|| panic!("MP-A-17: carol.c2 reply was Ok — expected a reject. Reply: {reply:?}"));
    // Empirically wire 4000 (wrong-space_id → unmapped variant; MP-F2-followon).
    assert_eq!(
        err.reject_code,
        Some(4000),
        "MP-A-17: expected wire reject_code 4000; got {:?} (message: {})",
        err.reject_code,
        err.message
    );
    let offending = err
        .event_id
        .as_deref()
        .unwrap_or_else(|| panic!("MP-A-17: reject reply carries no event_id: {err:?}"));
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
    // PermissionDenied (can_invite). Empirically wire 4000 (unmapped variant —
    // MP-F2-followon; pinned to observed so a remap re-grounds it here).
    assert_rejected_no_membership(&o, "bob", "b3", carol_id, Some(4000), "MP-A-20 privilege escalation");
}
