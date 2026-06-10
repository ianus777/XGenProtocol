// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! MP-R2 C3 proof — the late-federation / catch-up director capability (`#[ignore]`).
//!
//! The `[[federation]] after` parse is unit-tested in `manifest.rs` without
//! spawning. These smokes prove the *director* path (MP-R2-D5): a link with an
//! `after` gate is **not** pre-seeded — the director establishes it (both
//! directions, naming the already-built/aged Space) only after that key is
//! published, so a node federates *after* the Space has history and catches up
//! via the existing sync path.
//!
//! ## Box-gated (RUN gate, M-R2.3) + honest boundary
//! Heavy — spawns real `--features harness-control` `xgen-node` + clients. These
//! run at the box-gated RUN, not in the fast suite. The scenario arg surface
//! (`invite` TTL arg, `join`, the membership/transcript projections) is grounded
//! against the matrix §2/§5 + the R1 batches; **any drift is confirmed at the
//! first box-gated RUN** (Rule 2 — a spawn/connect timeout is a flake, re-run
//! isolated; a real arg mismatch is a finding to route, not patch).
//!
//! ```text
//! cargo build -p xgen-node --features harness-control && cargo build -p xgen-client
//! cargo test -p xgen-mptest --test mp_r2_catchup -- --ignored --nocapture
//! ```

use std::path::Path;

use xgen_mptest::dial::{ClockMode, RoundDial};
use xgen_mptest::manifest::Scenario;
use xgen_mptest::runner::run_scenario;

// ── Smoke 1: the late-federation machinery (2-node catch-up) ─────────────────

const S1_PORT_A: u16 = 8486;
const S1_PORT_B: u16 = 8487;

/// A↔B where **B federates late**: alice@A builds a Space (create + room + a
/// post) and exports `history_ready`; the A→B link carries `after =
/// "history_ready"`, so the director establishes it only after the history
/// exists. Asserts B's transcript catches up the Space (it carries the Space's
/// events) — the late-federation director path, end-to-end.
#[tokio::test]
#[ignore = "heavy: spawns two real harness-control xgen-node + 2 clients; box-gated RUN"]
async fn late_federation_catch_up_converges() {
    let tmp = tempfile::tempdir().expect("tempdir");
    write_late_fed_scenario(tmp.path());
    let scenario = Scenario::load(tmp.path()).expect("load late-federation scenario");

    let dial = RoundDial {
        clock: ClockMode::Mock,
        ..Default::default()
    };
    let outcome = run_scenario(&scenario, &dial)
        .await
        .expect("run_scenario(late-federation) — node built --features harness-control?");

    let alice = outcome
        .actor_runs
        .iter()
        .find(|r| r.actor == "alice")
        .expect("alice run");
    assert!(alice.all_ok(), "alice batch failed: {:?}", alice.replies);

    let space = outcome.space_id.clone().expect("alice exported a space_id");
    let tb = outcome
        .transcripts
        .iter()
        .find(|t| t.node == "b")
        .expect("node b transcript");
    assert!(
        !tb.event_ids_for_space(&space).is_empty(),
        "Space {space} did NOT catch up onto the LATE-federated B — B's transcript has no events for it: {:?}",
        tb.events
    );
    eprintln!(
        "C3 late-federation PASS: A→B established AFTER history_ready (not pre-seeded); \
         Space {space} caught up onto the late-federated B"
    );
}

/// alice@A: register → create Space → create room → post; exports `space_id`
/// (the primary key) + `history_ready` (the late-link gate, off her post). bob@B
/// registers (a presence so B has a client; the catch-up is node-level). The
/// A→B link waits on `history_ready`.
fn write_late_fed_scenario(dir: &Path) {
    let manifest = format!(
        r#"
scenario = "MP-R2-LATEFED-SMOKE"
description = "C3 late-federation / catch-up machinery proof"

[[nodes]]
label = "a"
port = {S1_PORT_A}

[[nodes]]
label = "b"
port = {S1_PORT_B}

[[federation]]
from = "a"
to = "b"
after = "history_ready"

[[actors]]
name = "alice"
node = "a"
batch = "alice.jsonl"

[[actors]]
name = "bob"
node = "b"
batch = "bob.jsonl"

[[exports]]
actor = "alice"
command = "a2"
field = "space_id"
key = "space_id"

[[exports]]
actor = "alice"
command = "a4"
field = "event_id"
key = "history_ready"
"#
    );
    // alice builds the Space + a post BEFORE B federates; her post's export
    // (`history_ready`) is the late-link gate.
    let alice = "\
{\"cmd\":\"register\",\"args\":{\"name\":\"alice\"},\"id\":\"a1\"}\n\
{\"cmd\":\"create-space\",\"args\":{\"name\":\"LATEFED-S\"},\"id\":\"a2\",\"bind\":\"s\"}\n\
{\"cmd\":\"create-room\",\"args\":{\"space\":\"$s\",\"name\":\"general\"},\"id\":\"a3\",\"bind\":\"r\"}\n\
{\"cmd\":\"send\",\"args\":{\"space\":\"$s\",\"room\":\"$r\",\"text\":\"pre-federation history\"},\"id\":\"a4\"}\n";
    let bob = "{\"cmd\":\"register\",\"args\":{\"name\":\"bob\"},\"id\":\"b1\"}\n";

    std::fs::write(dir.join("manifest.toml"), manifest).expect("write manifest");
    std::fs::write(dir.join("alice.jsonl"), alice).expect("write alice batch");
    std::fs::write(dir.join("bob.jsonl"), bob).expect("write bob batch");
}

// ── Smoke 2: MP-A-01(ii) — aged-invite replay preserves membership ───────────

const S2_PORT_A: u16 = 8488;
const S2_PORT_B: u16 = 8489;
const S2_PORT_C: u16 = 8490;

/// MP-A-01(ii): an aged-Space catch-up does **not** re-reject the historical
/// invited-join (INV-EXP / J-298 — the 3044 invite-expiry gate is admission-only,
/// skipped on the federation-receive path). Shape: A↔B federate **early**;
/// alice@A invites bob@B (short TTL) and bob joins (the invite + join become
/// historical Space-DAG events on A+B). The clock then advances past the TTL
/// (publishing `clock_advanced`). A **third** node C federates **late** (the A→C
/// link waits on `clock_advanced`) and catches up the now-aged Space: the
/// historical invited-join replays onto C and is **preserved**, not re-rejected
/// as expired.
///
/// Assertion (transcript-level, the harness-observability boundary — sibling to
/// MP-C-13 / MP-C-07): bob's join event id replays into C's transcript for the
/// Space (the catch-up admitted it; it was not dropped as expired). A node-side
/// membership-projection rail for a non-member observer is not available, so
/// transcript presence + absence-of-rejection is the witness (the J-298 property
/// itself is unit-proven in-process; this is its real-binary catch-up repro).
///
/// **Box-gated; scenario arg surface (invite TTL, join) confirmed at the first
/// RUN.**
#[tokio::test]
#[ignore = "heavy: spawns three real harness-control xgen-node + clients; box-gated RUN"]
async fn mp_a_01_ii_aged_invite_replay_preserves_membership() {
    let tmp = tempfile::tempdir().expect("tempdir");
    write_aged_replay_scenario(tmp.path());
    let scenario = Scenario::load(tmp.path()).expect("load aged-replay scenario");

    let dial = RoundDial {
        clock: ClockMode::Mock,
        ..Default::default()
    };
    let outcome = run_scenario(&scenario, &dial)
        .await
        .expect("run_scenario(aged-replay) — node built --features harness-control?");

    let space = outcome.space_id.clone().expect("alice exported a space_id");
    let bob = outcome
        .actor_runs
        .iter()
        .find(|r| r.actor == "bob")
        .expect("bob run");
    let bob_join = bob
        .reply_for("b2")
        .and_then(|r| r.data_str("event_id"))
        .expect("bob's join exported an event_id");

    let tc = outcome
        .transcripts
        .iter()
        .find(|t| t.node == "c")
        .expect("node c transcript");
    let on_c = tc.event_ids_for_space(&space);
    assert!(
        on_c.iter().any(|id| id == bob_join),
        "bob's historical invited-join {bob_join} did NOT replay onto the late catch-up node C \
         (aged-invite re-rejected on replay?) — C's Space events: {on_c:?}"
    );
    eprintln!(
        "C3 MP-A-01(ii) PASS: aged-Space catch-up onto late-federated C preserved bob's \
         historical invited-join {bob_join} (admission-only gate, not re-rejected)"
    );
}

/// A↔B early; alice invites bob (short TTL) + bob joins (historical); clock ages
/// past the TTL; C federates late (A→C `after = clock_advanced`) and catches up.
fn write_aged_replay_scenario(dir: &Path) {
    let manifest = format!(
        r#"
scenario = "MP-A-01-ii-AGED-REPLAY"
description = "C3 aged-invite replay preserves membership on a late catch-up peer"

[[nodes]]
label = "a"
port = {S2_PORT_A}

[[nodes]]
label = "b"
port = {S2_PORT_B}

[[nodes]]
label = "c"
port = {S2_PORT_C}

[[federation]]
from = "a"
to = "b"

[[federation]]
from = "a"
to = "c"
after = "clock_advanced"

[[actors]]
name = "alice"
node = "a"
batch = "alice.jsonl"

[[actors]]
name = "bob"
node = "b"
batch = "bob.jsonl"

[[exports]]
actor = "alice"
command = "a2"
field = "space_id"
key = "space_id"

[[exports]]
actor = "alice"
command = "a4"
field = "event_id"
key = "invite_ready"

[[exports]]
actor = "bob"
command = "b1"
field = "identity_id"
key = "bob_identity_id"

[[exports]]
actor = "bob"
command = "b2"
field = "event_id"
key = "bob_join_ready"

[[waits]]
actor = "bob"
command = "b2"
key = "invite_ready"

[[clock]]
node = "a"
op = "advance"
value = "2d"
after = "bob_join_ready"
publishes = "clock_advanced"
"#
    );
    // alice: register → space → room → invite bob with a SHORT TTL (1 day).
    let alice = "\
{\"cmd\":\"register\",\"args\":{\"name\":\"alice\"},\"id\":\"a1\"}\n\
{\"cmd\":\"create-space\",\"args\":{\"name\":\"AGED-S\"},\"id\":\"a2\",\"bind\":\"s\"}\n\
{\"cmd\":\"create-room\",\"args\":{\"space\":\"$s\",\"name\":\"general\"},\"id\":\"a3\",\"bind\":\"r\"}\n\
{\"cmd\":\"invite\",\"args\":{\"space\":\"$s\",\"identity\":\"{{bob_identity_id}}\",\"role\":\"member\",\"valid_for_days\":1},\"id\":\"a4\"}\n";
    // bob: register, then join AFTER the invite (the historical invited-join);
    // the clock then ages past the 1-day TTL before C catches up.
    let bob = "\
{\"cmd\":\"register\",\"args\":{\"name\":\"bob\"},\"id\":\"b1\"}\n\
{\"cmd\":\"join\",\"args\":{\"space\":\"{{space_id}}\"},\"id\":\"b2\"}\n";

    std::fs::write(dir.join("manifest.toml"), manifest).expect("write manifest");
    std::fs::write(dir.join("alice.jsonl"), alice).expect("write alice batch");
    std::fs::write(dir.join("bob.jsonl"), bob).expect("write bob batch");
}
