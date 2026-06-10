// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! MP-R1 Tranche 4 (C7) — wire/injector (`#[ignore]`).
//!
//! Six injector scenarios driven through the new `kind="injector"` dispatch
//! (`run_scenario` → `run_injector_actor`): the injector speaks the transport
//! directly and **recvs the node's `Error` frame** (MP-R1-D9 — the wire-path
//! capability the fire-and-forget batch client lacks). MP-A-01 (expired-invite
//! federation replay) is batch+clock, not injector, and is handled separately.
//!
//! Per-row oracle (C7-specific grounding, D-065):
//! - where an `Error` frame is the expected rejection signal (forged-signature →
//!   step-12 / clock-skew → wire-3046) the smoke asserts it arrived with the
//!   expected code;
//! - where the expected outcome is HeldPending / dedup the signal is **absence**
//!   (the crafted event never lands) — a held event emits no `Error` frame;
//! - MP-A-12 (malformed frame) asserts the node closed it cleanly AND still
//!   serves legitimate traffic afterwards (liveness).
//!
//! ```text
//! cargo build -p xgen-node --features harness-control && cargo build -p xgen-client
//! cargo test -p xgen-mptest --test mp_r1_c7 -- --ignored --nocapture
//! ```

use std::path::{Path, PathBuf};

use xgen_mptest::dial::{ClockMode, RoundDial};
use xgen_mptest::injector_actor::InjectionRecord;
use xgen_mptest::oracle::{rejection_verdict, MembershipProjection, Transcript};
use xgen_mptest::runner::{run_scenario, ScenarioOutcome};
use xgen_mptest::manifest::Scenario;

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

/// The injector's captured record for a directive id.
fn inj_record<'a>(o: &'a ScenarioOutcome, actor: &str, id: &str) -> &'a InjectionRecord {
    o.injector_runs
        .iter()
        .find(|r| r.actor == actor)
        .unwrap_or_else(|| panic!("no injector run for `{actor}`"))
        .record_for(id)
        .unwrap_or_else(|| panic!("no injector record for `{actor}.{id}`"))
}

/// `true` if `event_id` is present in any node transcript.
fn present(transcripts: &[Transcript], event_id: &str) -> bool {
    transcripts.iter().any(|t| t.contains_event(event_id))
}

fn alice_view(o: &ScenarioOutcome) -> &MembershipProjection {
    o.projections
        .iter()
        .find(|p| p.node == "alice-view")
        .unwrap_or_else(|| {
            panic!(
                "no `alice-view` projection (views: {:?})",
                o.projections.iter().map(|p| &p.node).collect::<Vec<_>>()
            )
        })
}

fn event_id_of(rec: &InjectionRecord) -> &str {
    rec.event_id
        .as_deref()
        .unwrap_or_else(|| panic!("injector record `{:?}` has no event_id", rec.id))
}

/// A batch actor's reply `data` field (for the batch+clock MP-A-01, which has no
/// injector — bob's join / register replies live in `actor_runs`).
fn reply_field<'a>(o: &'a ScenarioOutcome, actor: &str, command: &str, field: &str) -> &'a str {
    o.actor_runs
        .iter()
        .find(|r| r.actor == actor)
        .unwrap_or_else(|| panic!("no batch run for `{actor}`"))
        .reply_for(command)
        .unwrap_or_else(|| panic!("no reply for `{actor}.{command}`"))
        .data_str(field)
        .unwrap_or_else(|| panic!("`{actor}.{command}` reply has no `{field}`"))
}

/// MP-A-05 — signature forgery. The injector forges its own (real-member)
/// identity (sign with attacker key, claim the registered member) so steps 9–11
/// pass and only step 12 fails. Rejected + Error recv'd; forged event absent.
#[tokio::test]
#[ignore = "heavy: spawns a harness-control xgen-node; run with --ignored"]
async fn mp_a_05_forged_signature_rejected() {
    let o = run("MP-A-05").await;
    let rec = inj_record(&o, "mallory", "x3");
    let forged = event_id_of(rec);
    let v = rejection_verdict(&o.transcripts, forged);
    assert!(v.pass, "MP-A-05: forged event was applied — {}", v.detail);
    let (code, msg) = rec
        .error_reply
        .clone()
        .expect("MP-A-05: expected an Error frame on the wire (step-12 rejection)");
    assert_eq!(code, 4000, "MP-A-05: expected step-12 generic-4000 band, got {code}: {msg}");
    eprintln!("MP-A-05 PASS: forgery rejected (absent) + Error recv'd on wire: {code} {msg}");
}

/// MP-A-09 — duplicate-event_id. The same event submitted twice.
///
/// **DAG/store/disk dedup holds — grounded three ways (D-065):** `graph.add_event`
/// is structurally idempotent (tips/successors are sets; no error, no second
/// vertex on a duplicate id), `store` ignores a duplicate insert (runtime.rs:578),
/// and `persist_event` has a per-event duplicate-guard. So "applied once" holds at
/// the DAG/state layer.
///
/// **MP-F3 RESOLVED (J-326): fanned out exactly once.** Pre-fix, `dispatch_event`
/// returned `Accepted` for a re-submitted duplicate, so `apply_fanout`
/// RE-BROADCAST it and the `.events` transcript showed a recipient receiving it
/// 2× (mild amplification, not state corruption — routed MP-F3). The fix adds a
/// dedup-at-dispatch gate (`DispatchOutcome::Duplicate` via `store.contains`,
/// runtime.rs) → no fan-out + an idempotent ack. So the falsifiable success
/// criterion (design §5, Joe-pinned) is: **no recipient transcript receives the
/// duplicate more than once.** This asserts the per-transcript max occurrence is
/// exactly 1 — delivered (present), never doubled — which retires the prior
/// re-emit-tolerance / measurement-gap note. A regression that re-broadcasts a
/// duplicate makes some transcript show it 2× and fails this.
#[tokio::test]
#[ignore = "heavy: spawns a harness-control xgen-node; run with --ignored"]
async fn mp_a_09_duplicate_fanned_out_exactly_once() {
    let o = run("MP-A-09").await;
    let rec = inj_record(&o, "mallory", "x3");
    let dup = event_id_of(rec);
    // Max occurrences of the duplicate in any single recipient transcript.
    let max_per_transcript = o
        .transcripts
        .iter()
        .map(|t| t.events.iter().filter(|e| e.event_id.as_deref() == Some(dup)).count())
        .max()
        .unwrap_or(0);
    assert_eq!(
        max_per_transcript, 1,
        "MP-A-09 (MP-F3): the duplicate must be fanned out EXACTLY ONCE per recipient. \
         max-per-transcript == {max_per_transcript} (0 ⇒ never delivered; >1 ⇒ \
         apply_fanout re-broadcast the re-submitted duplicate = MP-F3 regression)."
    );
    eprintln!(
        "MP-A-09 PASS (MP-F3 resolved): the re-submitted duplicate is applied once \
         (DAG/store/disk dedup) AND fanned out exactly once (dedup-at-dispatch → \
         DispatchOutcome::Duplicate → FanoutRequest::none()). No recipient received it twice."
    );
}

/// MP-A-10 — causal gap / missing-parent. Event referencing an absent parent is
/// held (Step-9), never applied out of causal order.
#[tokio::test]
#[ignore = "heavy: spawns a harness-control xgen-node; run with --ignored"]
async fn mp_a_10_missing_parent_held() {
    let o = run("MP-A-10").await;
    let rec = inj_record(&o, "mallory", "x3");
    let ev = event_id_of(rec);
    let v = rejection_verdict(&o.transcripts, ev);
    assert!(v.pass, "MP-A-10: missing-parent event was applied — {}", v.detail);
    eprintln!(
        "MP-A-10 PASS: missing-parent held/not-admitted (absent); wire reply={:?}",
        rec.error_reply
    );
}

/// MP-A-12 — malformed / truncated frame. Node closes the bad frame cleanly and
/// still serves legitimate traffic afterwards (liveness).
#[tokio::test]
#[ignore = "heavy: spawns a harness-control xgen-node; run with --ignored"]
async fn mp_a_12_malformed_frame_rejected_node_alive() {
    let o = run("MP-A-12").await;
    let frame = inj_record(&o, "mallory", "x1");
    assert_eq!(
        frame.frame_closed,
        Some(true),
        "MP-A-12: node did not close the malformed frame cleanly (wedged?)"
    );
    let create = inj_record(&o, "mallory", "x3");
    let sid = event_id_of(create);
    assert!(
        present(&o.transcripts, sid),
        "MP-A-12: post-attack create did not land — node not serving legitimate traffic"
    );
    eprintln!("MP-A-12 PASS: malformed frame closed cleanly + node alive (post-attack create landed)");
}

/// MP-A-15 — clock-skew timestamp. A 2099-dated event is rejected by the M9.1
/// Step-8.5 future-skew bound; absent + Error recv'd.
///
/// **MP-F2 CLOSED (this row's payoff):** the delivered `Error` frame now carries
/// the specific `to_wire_code` `3046` (`event_timestamp_out_of_bounds`), not the
/// generic 4000 band. MP-F2 widened `DispatchOutcome::Rejected` to carry the
/// structured `RejectInfo`, so `reject_signal` puts the computed code on the wire
/// (D-070's deferred reject-code half; the prior generic-4000 was the J-081 shape).
/// `assert_eq!(code, 3046)` below is the falsifiable proof. (Unmapped variants —
/// e.g. signature, MP-A-05 — still emit 4000 until MP-F2-followon.)
#[tokio::test]
#[ignore = "heavy: spawns a harness-control xgen-node; run with --ignored"]
async fn mp_a_15_clock_skew_rejected() {
    let o = run("MP-A-15").await;
    let rec = inj_record(&o, "mallory", "x3");
    let ev = event_id_of(rec);
    let v = rejection_verdict(&o.transcripts, ev);
    assert!(v.pass, "MP-A-15: skewed event was applied — {}", v.detail);
    let (code, msg) = rec
        .error_reply
        .clone()
        .expect("MP-A-15: expected an Error frame for the skewed event (M9.1 Step 8.5)");
    assert!(
        msg.to_lowercase().contains("timestamp") || msg.to_lowercase().contains("skew"),
        "MP-A-15: Error message does not indicate a timestamp/skew rejection: {code} {msg}"
    );
    // MP-F2 payoff — the specific wire code is now on the wire (was generic 4000).
    assert_eq!(
        code, 3046,
        "MP-F2: timestamp reject must deliver wire 3046 on the Error frame; got {code}: {msg}"
    );
    eprintln!(
        "MP-A-15 PASS: clock-skew rejected by the M9.1 Step-8.5 bound + absent. \
         Wire code = {code} (== 3046, MP-F2: the specific code is now on the wire) + message '{msg}'."
    );
}

/// MP-A-16 — fabricated / never-issued invite (reclassified from C6). A
/// `membership.join` whose `prev_events` name a never-issued invite is held
/// (Step-9); the injector never becomes a member (distinct from a legitimate
/// open-join). Finding-candidate: if admitted / membership granted → route.
#[tokio::test]
#[ignore = "heavy: spawns a harness-control xgen-node + 1 client; run with --ignored"]
async fn mp_a_16_fabricated_invite_grants_nothing() {
    let o = run("MP-A-16").await;
    let rec = inj_record(&o, "mallory", "x1");
    let join = event_id_of(rec);
    let v = rejection_verdict(&o.transcripts, join);
    assert!(v.pass, "MP-A-16: fabricated-invite join was applied — {}", v.detail);
    let members = &alice_view(&o).members;
    assert_eq!(
        members.len(),
        1,
        "MP-A-16: an extra member appeared — fabricated-invite join granted membership: {members:?}"
    );
    eprintln!("MP-A-16 PASS: fabricated-invite join held + injector not a member; wire reply={:?}", rec.error_reply);
}

/// MP-A-01(i) — local expired-invite → 3044 rejected, no membership (batch +
/// `[[clock]]`). alice invites bob (1-day TTL); the director sets node A's clock
/// to 2099 *after* the invite (so the invite is ingested normally, then expires
/// from the node's vantage); bob then joins (gated on the clock-completion key) →
/// the 3044 invite-expiry gate fires on the LocallySubmitted join. Post-MP-F5
/// the reject is surfaced structurally — `join` returns an `Error` reply carrying
/// `reject_code` (3044) + `event_id` — so this asserts-the-reject (the J-336 C6
/// oracle migration applied to this C7 straggler), then absence + no membership.
///
/// MP-A-01(ii) — the INV-EXP/J-298 federation-replay-preserved regression — is
/// recorded PENDING in the matrix (materially harder: it needs a cross-node
/// aged-Space *catch-up* — federation established AFTER the clock advances — which
/// the fixed G-6 bootstrap timing does not orchestrate; the property is already
/// proven in-process at J-298).
#[tokio::test]
#[ignore = "heavy: spawns a harness-control xgen-node + 2 clients; run with --ignored"]
async fn mp_a_01_local_expired_invite_rejected() {
    let o = run("MP-A-01").await;
    // MP-F5 assert-the-reject (this C7 test is a straggler of the J-336 C6 oracle
    // migration): the expired-invite join is rejected at the 3044 gate
    // (LocallySubmitted) and surfaced as an `Error` reply carrying reject_code +
    // event_id — not an ok+event_id reply. Read the reject envelope, pin the wire
    // code, then assert absence + no membership.
    let reply = o
        .actor_runs
        .iter()
        .find(|r| r.actor == "bob")
        .expect("no run for bob")
        .reply_for("b2")
        .expect("no reply for bob.b2");
    let err = reply.error().unwrap_or_else(|| {
        panic!("MP-A-01(i): bob.b2 reply was Ok — expected the 3044 expired-invite reject. Reply: {reply:?}")
    });
    // Wire 3044 invite_expired (the J-298 INV-EXP gate; MP-F5: now structurally surfaced).
    assert_eq!(
        err.reject_code,
        Some(3044),
        "MP-A-01(i): expected wire reject_code 3044 (invite_expired); got {:?} (message: {})",
        err.reject_code,
        err.message
    );
    let join = err.event_id.as_deref().unwrap_or_else(|| {
        panic!("MP-A-01(i): reject reply carries no event_id (MP-F5 surfacing missing): {err:?}")
    });
    let v = rejection_verdict(&o.transcripts, join);
    assert!(v.pass, "MP-A-01(i): expired-invite join was applied — {}", v.detail);
    let bob_id = reply_field(&o, "bob", "b1", "identity_id");
    let members = &alice_view(&o).members;
    assert!(
        !members.contains_key(bob_id),
        "MP-A-01(i): bob gained membership despite the expired invite (3044 not enforced): {members:?}"
    );
    eprintln!("MP-A-01(i) PASS: local expired-invite join rejected (3044, surfaced) — absent + bob not a member");
}
