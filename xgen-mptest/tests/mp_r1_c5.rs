// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! MP-R1 Tranche 2 (C5) — membership-lifecycle cooperative (`#[ignore]`).
//!
//! Five of the six Tranche-2 scenarios are authorable on the client-verb rails
//! and shipped here; the last (MP-C-06, re-home) is recorded BLOCKED in
//! `docs/tests/MULTIPARTY_TEST_MATRIX.md` (deferred to M10 — D10). MP-C-13 joined
//! at the `thread` arc (arc 4); MP-C-08 at `room_update` (arc 3); MP-C-09 at
//! `ban` (arc 2):
//! - **MP-C-01** multi-client local single-node fan-out (alice + carol on Node A;
//!   both members; both posts seen by both — per-client `state` + `.events`).
//! - **MP-C-10** leave & rejoin, cross-node A↔B (bob joins, leaves, is
//!   re-invited, rejoins; membership converges to `{alice:owner, bob:member}`).
//! - **MP-C-09** ban → converge → post-rejected (the `ban` arc; banned member's
//!   post rejected step-11 + reject-surfaced per MP-F5; bob excluded from membership).
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
use xgen_mptest::oracle::{rejection_verdict, Transcript};
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

/// Assert a given actor command produced an `Ok` reply.
fn assert_cmd_ok(runs: &[xgen_mptest::batch::ActorRun], actor: &str, command: &str) {
    let r = runs
        .iter()
        .find(|r| r.actor == actor)
        .unwrap_or_else(|| panic!("no run for actor {actor}"));
    let reply = r
        .reply_for(command)
        .unwrap_or_else(|| panic!("no reply for {actor}.{command}"));
    assert!(
        reply.is_ok(),
        "{actor}.{command} should be Ok: {:?}",
        r.replies
    );
}

/// A `data` field of a given actor command's reply (panics if absent / not Ok).
fn data_of<'a>(
    runs: &'a [xgen_mptest::batch::ActorRun],
    actor: &str,
    command: &str,
    field: &str,
) -> &'a str {
    runs.iter()
        .find(|r| r.actor == actor)
        .unwrap_or_else(|| panic!("no run for actor {actor}"))
        .reply_for(command)
        .unwrap_or_else(|| panic!("no reply for {actor}.{command}"))
        .data_str(field)
        .unwrap_or_else(|| panic!("{actor}.{command} reply has no {field}"))
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

/// MP-C-09 — ban → converge → post-rejected (the `ban` thin-verb's cooperative
/// witness; inherits the MP-F5 assert-the-reject oracle). Single-node (BAN-D1).
///
/// alice bans bob (a room member); `apply_ban` cascades the removal. bob's
/// subsequent post is rejected at `validate_event` step-11 (sender no longer a
/// member) — and post-MP-F5 that reject surfaces structurally. Asserts:
/// 1. the ban (a5) + bob's pre-ban room-join (b3) were accepted;
/// 2. bob's post (b4) reply is an Error with `reject_code` (≈4000, pinned
///    empirically) + `event_id` — assert-the-reject, proving the rewritten oracle
///    holds for ban, not just an effect-absence pass;
/// 3. the post event is absent from the node transcript (`rejection_verdict`);
/// 4. the resolved membership excludes bob on every view (ban applied + converged).
#[tokio::test]
#[ignore = "heavy: spawns a harness-control xgen-node + 2 clients; run with --ignored"]
async fn mp_c_09_ban_then_post_rejected() {
    let o = run("MP-C-09").await;
    // Setup accepted: ban (a5) + bob's pre-ban room-join (b3).
    assert_cmd_ok(&o.actor_runs, "alice", "a5");
    assert_cmd_ok(&o.actor_runs, "bob", "b3");

    // (2) Assert-the-reject on bob's post-ban send (b4).
    let reply = o
        .actor_runs
        .iter()
        .find(|r| r.actor == "bob")
        .expect("no run for bob")
        .reply_for("b4")
        .expect("no reply for bob.b4");
    let err = reply
        .error()
        .unwrap_or_else(|| panic!("MP-C-09: bob.b4 reply was Ok — expected a node reject. Reply: {reply:?}"));
    // Empirically wire 4000 (step-11 non-member; unmapped variant, MP-F2-followon).
    assert_eq!(
        err.reject_code,
        Some(4000),
        "MP-C-09: expected wire reject_code 4000 (step-11 non-member); got {:?} (message: {})",
        err.reject_code,
        err.message
    );
    let offending = err
        .event_id
        .as_deref()
        .unwrap_or_else(|| panic!("MP-C-09: reject reply carries no event_id: {err:?}"));

    // (3) the post is absent from every node's transcript.
    let v = rejection_verdict(&o.transcripts, offending);
    assert!(v.pass, "MP-C-09: banned member's post was applied — {}", v.detail);

    // (4) the resolved membership excludes bob on every view (ban converged).
    let bob_id = data_of(&o.actor_runs, "bob", "b1", "identity_id");
    assert!(!o.projections.is_empty(), "expected ≥1 membership view");
    for p in &o.projections {
        assert!(
            !p.members.contains_key(bob_id),
            "MP-C-09: bob still a member on view `{}` (ban not applied): {:?}",
            p.node,
            p.members
        );
    }
    eprintln!(
        "MP-C-09 PASS: ban applied; bob's post reject_code={:?}, {offending} absent on all nodes; bob excluded from membership",
        err.reject_code
    );
}

/// MP-C-08 — multi-room space + per-room overrides (the `room_update` thin-verb's
/// witness; PG-12). Single-node (RU-D3). alice sets a `(Moderator, SendMessages)
/// → Deny` override on room2 (announcements) and invites bob as moderator. bob
/// posts in room1 (no override → permitted, converges) and room2 (denied).
/// Asserts BOTH halves (RU-D2):
/// 1. POSITIVE / per-room independence — bob's room1 post is in the node's
///    cooperative event set (converges);
/// 2. ENFORCEMENT (assert-the-reject, inherits MP-F5) — bob's room2 post reply is
///    an Error with `reject_code` (PermissionDenied → 4000, pinned empirically)
///    + `event_id`, and the post is absent everywhere.
/// Same role, two rooms, opposite outcomes ⇒ per-room overrides honored
/// independently (the override's presence in room2 state is proven by its effect).
#[tokio::test]
#[ignore = "heavy: spawns a harness-control xgen-node + 2 clients; run with --ignored"]
async fn mp_c_08_per_room_override() {
    let o = run("MP-C-08").await;
    // Setup accepted: room_update (a5), bob's room2 join (b4), bob's room1 post (b5).
    assert_cmd_ok(&o.actor_runs, "alice", "a5");
    assert_cmd_ok(&o.actor_runs, "bob", "b4");
    assert_cmd_ok(&o.actor_runs, "bob", "b5");

    let space = o.space_id.clone().expect("space_id exported");

    // (1) positive / per-room independence: room1 post (no override) converges.
    let room1_post = event_id_of(&o.actor_runs, "bob", "b5");
    assert_event_on_all_nodes(&o.transcripts, &space, room1_post, "bob's room1 post (permitted)");

    // (2) enforcement (assert-the-reject): room2 post denied + absent.
    let reply = o
        .actor_runs
        .iter()
        .find(|r| r.actor == "bob")
        .expect("no run for bob")
        .reply_for("b6")
        .expect("no reply for bob.b6");
    let err = reply
        .error()
        .unwrap_or_else(|| panic!("MP-C-08: bob.b6 (room2 post) reply was Ok — expected a Deny reject. Reply: {reply:?}"));
    // PermissionDenied → wire 4000 (unmapped variant; MP-A-20 precedent, MP-F2-followon).
    assert_eq!(
        err.reject_code,
        Some(4000),
        "MP-C-08: expected wire reject_code 4000 (PermissionDenied); got {:?} (message: {})",
        err.reject_code,
        err.message
    );
    let offending = err
        .event_id
        .as_deref()
        .unwrap_or_else(|| panic!("MP-C-08: reject reply carries no event_id: {err:?}"));
    let v = rejection_verdict(&o.transcripts, offending);
    assert!(v.pass, "MP-C-08: denied room2 post was applied — {}", v.detail);

    eprintln!(
        "MP-C-08 PASS: room1 post {room1_post} converged (no override); room2 post reject_code={:?}, {offending} absent (Deny override honored per-room)",
        err.reject_code
    );
}

/// MP-C-13 — thread create / resolve / archive (the `thread` thin-verb group's
/// witness; PG-08). Single-node (TH-D3). Both-halves (TH-D2):
/// 1. POSITIVE — owner alice's create + resolve + archive events all land in the
///    node's cooperative event set (the lifecycle converges; Layer-5c winner-
///    selection is unit-proven; the harness exposes no ThreadState projection so
///    convergence is asserted via the transcript, TH-D4);
/// 2. ENFORCEMENT (assert-the-reject, inherits MP-F5) — member bob's `thread
///    resolve` is refused by the ChangeInfo gate (Admin+): reply is an Error with
///    `reject_code` (PermissionDenied → 4000, pinned empirically) + `event_id`,
///    the op absent everywhere.
#[tokio::test]
#[ignore = "heavy: spawns a harness-control xgen-node + 2 clients; run with --ignored"]
async fn mp_c_13_thread_lifecycle() {
    let o = run("MP-C-13").await;
    // positive: the owner's three thread ops were accepted.
    assert_cmd_ok(&o.actor_runs, "alice", "a4");
    assert_cmd_ok(&o.actor_runs, "alice", "a5");
    assert_cmd_ok(&o.actor_runs, "alice", "a6");

    let space = o.space_id.clone().expect("space_id exported");
    for (cmd, what) in [("a4", "thread.create"), ("a5", "thread.resolved"), ("a6", "thread.archived")] {
        let ev = event_id_of(&o.actor_runs, "alice", cmd);
        assert_event_on_all_nodes(&o.transcripts, &space, ev, what);
    }

    // enforcement: member bob's resolve refused (ChangeInfo) + absent.
    let reply = o
        .actor_runs
        .iter()
        .find(|r| r.actor == "bob")
        .expect("no run for bob")
        .reply_for("b4")
        .expect("no reply for bob.b4");
    let err = reply
        .error()
        .unwrap_or_else(|| panic!("MP-C-13: bob.b4 (member resolve) reply was Ok — expected a ChangeInfo reject. Reply: {reply:?}"));
    assert_eq!(
        err.reject_code,
        Some(4000),
        "MP-C-13: expected wire reject_code 4000 (ChangeInfo PermissionDenied); got {:?} (message: {})",
        err.reject_code,
        err.message
    );
    let offending = err
        .event_id
        .as_deref()
        .unwrap_or_else(|| panic!("MP-C-13: reject reply carries no event_id: {err:?}"));
    let v = rejection_verdict(&o.transcripts, offending);
    assert!(v.pass, "MP-C-13: member's resolve was applied — {}", v.detail);

    eprintln!(
        "MP-C-13 PASS: thread create/resolve/archive converged on node; member bob's resolve reject_code={:?}, {offending} absent (ChangeInfo gate)",
        err.reject_code
    );
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

/// MP-C-07-LOCAL — DM private space, single node — **2-party message convergence**.
///
/// The single-node DM end-to-end witness for the MP-F1a + MP-F4 fixes combined.
/// alice@A `create-dm-space` with bob (3-event causal chain dm_space_create root →
/// auto-room → invite); alice posts; bob space-joins, room-joins, and posts; both
/// messages converge on Node A. NO `[[federation]]` ⇒ no facet-1; the federated
/// MP-C-07 (`mp_r1_c4`) is now harness-green-with-boundary (MP-F1b — DM federation
/// forms when members' home nodes resolve; production discovery deferred, F1B-D5).
///
/// **What it proves, layer by layer:**
/// - **MP-F1a (delivery):** `create-dm-space` awaits each event's `EventAccepted`
///   before the next send, so the whole 3-event chain lands (pre-F1a the verb
///   sent fire-and-forget then `goodbye`d, RST-dropping events 2-3 — bob could not
///   even space-join). The chain landing is asserted (dm root + auto-room).
/// - **MP-F4 (A1 + frontier anchor):** bob resolves as a **room** member, so his
///   `message.text` is accepted and converges. Pre-F4, the DM invitee's room-join
///   was dropped by node-side membership resolution: `state_key_for_event` keyed a
///   join room-agnostically (`membership:{space}:{sender}`), so bob's space-join
///   and room-join collapsed onto one key; with the room-join anchored to a
///   *concurrent* leaf (`get_dag_tips` returned the single topo-last event, which
///   could be alice's earlier message rather than bob's space-join), the two were
///   concurrent siblings and `derive_resolved` dropped one → bob Space-member-not-
///   room → `message.text` step-11 `NotARoomMember`. **A1** room-scopes the
///   membership key (space vs room facts no longer collide); the **frontier**
///   `get_dag_tips` makes the room-join causally descend from the space-join.
///   MP-C-01 (regular Space) never hit this because its message is gated *after*
///   the room-join, so no competing leaf existed.
///
/// **F1b cross-link (flag, don't act):** MP-F4 lives in `state_key` /
/// `get_dag_tips`; (iii)/MP-F1b lives in `federation_nodes` population — different
/// code, weighed together at F1b Phase-0, not merged.
#[tokio::test]
#[ignore = "heavy: spawns a harness-control xgen-node + 2 clients; run with --ignored"]
async fn mp_c_07_local_dm_2party_message_convergence() {
    let o = run("MP-C-07-LOCAL").await;

    // alice's confirmed create-dm chain returned both ids ⇒ all 3 events were
    // node-acked (the F1A-D4 chain policy returns Ok only after each EventAccepted).
    assert_cmd_ok(&o.actor_runs, "alice", "a2");
    let space = o.space_id.clone().expect("dm space_id exported");
    let dm_room = data_of(&o.actor_runs, "alice", "a2", "room_id").to_string();

    // bob can space-join (⇒ the invite landed — he sources it via the bootstrap)
    // and room-join the SAME auto-room (⇒ the auto-room landed). Both were
    // impossible pre-F1a; their success IS the facet-2 delivery proof.
    assert_cmd_ok(&o.actor_runs, "bob", "b2");
    assert_cmd_ok(&o.actor_runs, "bob", "b3");
    assert_eq!(
        data_of(&o.actor_runs, "bob", "b3", "room_id"),
        dm_room,
        "bob room-joined the create-dm auto-room"
    );

    // The chain physically persisted on Node A: the dm root (its event_id IS the
    // space_id) + the auto-room are both in Node A's cooperative event set.
    assert_event_on_all_nodes(&o.transcripts, &space, &space, "dm_space_create root");
    assert_event_on_all_nodes(&o.transcripts, &space, &dm_room, "dm auto-room");

    // MP-F4 — 2-party message convergence. bob now resolves as a ROOM member (his
    // room-join descends from his space-join via the frontier `get_dag_tips`
    // anchor; A1 keeps space/room membership in distinct conflict domains), so his
    // send is accepted. BOTH messages land in Node A's cooperative event set —
    // alice's a3 (sent before bob joined; confirms it lands post-MP-F1a) and bob's
    // b4. Genuinely RED before MP-F4 (b4 was step-11 NotARoomMember).
    assert_cmd_ok(&o.actor_runs, "bob", "b4");
    let alice_a3 = data_of(&o.actor_runs, "alice", "a3", "event_id").to_string();
    let bob_b4 = data_of(&o.actor_runs, "bob", "b4", "event_id").to_string();
    assert_event_on_all_nodes(&o.transcripts, &space, &alice_a3, "alice message a3");
    assert_event_on_all_nodes(&o.transcripts, &space, &bob_b4, "bob message b4");
    eprintln!("MP-C-07-LOCAL PASS (2-party DM message convergence: a3 + b4 on Node A)");
}
