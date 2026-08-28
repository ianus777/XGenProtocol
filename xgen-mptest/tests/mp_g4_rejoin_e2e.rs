// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! M-SPACE-ADMISSION Leg G-4 §5b — the **system gate** (`#[ignore]`, box-gated).
//!
//! 🛑 §5 of the runbook is an IMPLEMENTATION gate — *does the function return
//! what the runbook says*. This is a SYSTEM gate: *does the composition* —
//! serialisation, transport routing, the client's own data directory, the node's
//! store, and the order things actually happen in. **No test in §5 can fail the
//! way this file can fail, and that is the whole reason it exists.**
//!
//! Nothing in Legs G-1…G-3 ever ran against a live node, a wire, or a second
//! identity; `3048` has existed in the node for two legs and **no client has
//! ever received it**. Driven directly (not via `run_scenario`) because `V-9a`
//! mutates bob's client state between two commands and `V-9b` spawns a second
//! client instance mid-run — neither is expressible in the declarative runner.
//!
//! ```text
//! cargo build -p xgen-node && cargo build -p xgen-client
//! cargo test -p xgen-mptest --test mp_g4_rejoin_e2e -- --ignored --nocapture
//! ```
//!
//! ## `V-9c` — `3048` on a wire, as a PROCEDURE
//!
//! `V-9c` is the negative control and is not a permanently-green test: it is
//! `V-9a` run against a **reverted build**. Disarm Leg G-4 in
//! `xgen-client/src/batch.rs` by making `get_invite_bootstrap`'s `resolve`
//! helper ignore the selection —
//!
//! ```text
//! (Some(key), _) => select_rejoin_anchor(served, key),   // ← replace with
//! (Some(_), _) => vec![],                                // ← this
//! ```
//!
//! — then `cargo build -p xgen-client` and re-run `v9a_...`. The rejoin reply is
//! printed verbatim by both tests before they assert, so the refusal is captured
//! whether the assertion passes or fails. Restore by file copy afterwards
//! (**not** `git checkout --`, which restores from the index and discards
//! unstaged work).
//!
//! (`batch.rs:419` at `39cf7d3`; the anchor is `resolve`'s match on
//! `(rejoin_key, invite_id)` — the line is only a convenience, `D-152`
//! clause 1.)
//!
//! 🛑 **RE-POINTED AT LEG G-5 (2026-08-27), BECAUSE IT WAS
//! UNFOLLOWABLE.** This procedure previously named
//! `(None, Some(key)) => select_rejoin_anchor(served, key)` — an arm that
//! does not exist and never shipped. The invite-first PRECEDENCE it was
//! written against was DELETED at runbook v1.2 (2026-08-26, stated in capitals
//! above `select_rejoin_anchor`'s doc in `batch.rs`), and the disarm was left
//! pointing at the deleted shape. `V-9c` is the negative control for the whole
//! leg, and a procedure that cannot be executed is not a control. A leg that
//! edits a file invalidates its own citations into that file — which is why
//! the match, not the line, is the anchor above.
//!
//! ## 📌 HISTORY — status at the Leg G-4 hand-back (SUPERSEDED 2026-08-26)
//!
//! 🛑 **THIS BLOCK IS A DATED RECORD, NOT THE CURRENT STATE. BOTH
//! SCENARIOS PASS** at `39cf7d3` — `2 passed; 0 failed`, re-driven
//! independently by both seats (J-778). It is KEPT rather than deleted
//! because the refutation is the record: it diagnosed a real RED state, and
//! that diagnosis became `D-156`.
//!
//! Two of its claims are false as of 2026-08-26:
//!
//! - *"BOTH SCENARIOS FAIL"* — both passed once the precedence was deleted.
//! - *"§3's precedence is Joe-locked"* — Joe DELETED that precedence at
//!   runbook v1.2 after it failed on a live wire. `D-156`: two objects share
//!   one word — the invite ENTITLEMENT (`PendingInvite`, CONSUMED by
//!   `apply_join`) and the invite RECORD (the `membership.invite` event,
//!   PERMANENT). The client asked a HISTORY question, *is there an invite
//!   event naming me?* — answer yes, forever — and read it as a STATE
//!   question, which only `pending_invites` can answer and no client can read.
//!
//! The block's own diagnosis of the CAUSE was correct and is why the fix was
//! three lines. The original text follows verbatim.
//!
//! These tests assert the INTENDED behaviour and are RED against the shipped
//! (runbook-locked) implementation. They are deliberately **not** rewritten to
//! assert the current behaviour — baking a defect in as expected is how a
//! system gate stops being one.
//!
//! Both fail with `3048 rejoin_not_anchored`. Cause, measured by A/B on one
//! variable rather than argued:
//!
//! - Leg G-3 route 2 serves a departed member *the membership events naming
//!   her* — which **includes the invite she already consumed**. `apply_join`
//!   removed it from `pending_invites`, but the EVENT is still in the store and
//!   `bootstrap_event_names_requester` matches `MembershipInvite` on
//!   `content.target_identity`.
//! - `get_invite_bootstrap`'s invite scan matches on the served EVENT, not on
//!   `pending_invites`, so it cannot tell a LIVE invite from a CONSUMED one.
//! - Runbook §3's 🔒 *an invite naming her still wins* therefore anchors the
//!   rejoin on a **stale** invite — which her own first join already descends
//!   from — leaving the rejoin concurrent with her leave. Hence `3048`.
//!
//! **A/B (discarded probe, restored sha256-identical):** preferring the
//! selection over the invite makes BOTH scenarios pass, `Accepted` + membership
//! restored. Nothing else changed.
//!
//! 🔑 `select_rejoin_anchor` already handles the stale invite correctly **by
//! construction**: the consumed invite is referenced by her own join, so §3
//! step 3 subtracts it. And for a PURE invitee the two sources CONVERGE —
//! selection = `[invite_id]` — so `V-1`'s *an invitee is byte-identical* is
//! satisfied by the selection alone. The runbook's REQUIREMENT survives; its
//! chosen MECHANISM (precedence) is what does not.
//!
//! 🛑 Not patched here: §3's precedence is 🔒 Joe-locked, and the harness rule
//! is that a real defect surfaced by a system gate is a finding routed to a
//! ruling, never patched under the leg's own banner (Rule 6).
//!
//! ## Bounds, stated rather than discovered
//! - A spawn/connect timeout is a **flake**, not a failure — re-run isolated,
//!   and state how many runs the recorded result took.
//! - The harness **drives and observes**; it never patches. A real defect
//!   surfaced here is a finding routed to a fix-arc, never patched under G-4.
//! - `#[ignore]` always ⇒ this file contributes **ZERO** to the `cargo` floor.

use std::path::Path;

use serde_json::json;

use xgen_mptest::aicontrol::{AicontrolClient, DEFAULT_CONNECT_TIMEOUT};
use xgen_mptest::binloc;
use xgen_mptest::process::{instance_label, ManagedProcess};
use xgen_mptest::wire::{Command, Reply};

/// Ports are per-run; pick ones the sibling tests do not use.
const PORT_A: u16 = 8590;
const PORT_B: u16 = 8591;

/// Send one `--aicontrol` verb and return the reply (ok or error — the caller
/// decides what it means).
async fn call(c: &mut AicontrolClient, cmd: &str, args: serde_json::Value) -> Reply {
    let mut m = Command::new(cmd);
    if let serde_json::Value::Object(o) = args {
        m.args = o;
    }
    c.send(&m).await.unwrap_or_else(|e| panic!("aicontrol `{cmd}` transport failure: {e}"))
}

/// Same, asserting success — for the setup steps whose failure is a rig fault,
/// not a finding.
async fn ok(c: &mut AicontrolClient, cmd: &str, args: serde_json::Value) -> Reply {
    let r = call(c, cmd, args).await;
    assert!(r.is_ok(), "setup verb `{cmd}` failed: {r:?}");
    r
}

/// The membership oracle: `members` re-derives the roster through the node's own
/// `derive_resolved` and projects only `is_present()` members (`D-154`). So this
/// answers *did the rejoin SURVIVE RESOLUTION*, not merely *did the node say
/// Accepted* — which is exactly the distinction `3048` exists to draw. Driven
/// from a client that is still a member: a departed member cannot drain
/// (`collect_sync_history` gates on `is_member`).
async fn is_present(observer: &mut AicontrolClient, space: &str, who: &str) -> bool {
    let r = ok(observer, "members", json!({ "space": space })).await;
    r.data()
        .and_then(|d| d.get("members"))
        .and_then(|m| m.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|m| m.get("identity_id").and_then(|v| v.as_str()))
                .any(|id| id == who)
        })
        .unwrap_or(false)
}

/// `V-9a`'s fresh-install condition, surgically: drop the Space's entry from
/// `last_local_events`. `rejoin_anchor_or_root` (`ops.rs:142-147`, measured at
/// `fa0f8ad`) reads exactly that map, and its absence is what sent her to the
/// create root. `ClientState.last_local_events` is `#[serde(default)]`, so
/// removing the key is a LEGAL state, not a corruption.
///
/// Safe between two `--aicontrol` commands: every `ops::*` state writer is
/// read-modify-write per call and there is no periodic writer.
fn clear_leave_anchor(data_dir: &Path, space: &str) {
    let p = data_dir.join("xgen-client_state.json");
    let raw = std::fs::read_to_string(&p)
        .unwrap_or_else(|e| panic!("reading {}: {e}", p.display()));
    let mut v: serde_json::Value = serde_json::from_str(&raw).expect("client state is JSON");
    let had = v
        .get_mut("last_local_events")
        .and_then(|m| m.as_object_mut())
        .map(|m| m.remove(space).is_some())
        .unwrap_or(false);
    assert!(
        had,
        "PRECONDITION: bob's leave anchor for {space} must exist before it is cleared \
         — `ops::leave` persists it (MP-F7-D2/D4). Its absence means the fixture \
         degraded and this test would pass for the wrong reason."
    );
    std::fs::write(&p, serde_json::to_string_pretty(&v).expect("serialize"))
        .unwrap_or_else(|e| panic!("writing {}: {e}", p.display()));
}

/// Spawn a node + two client residents and connect to both pipes.
async fn rig(
    tag: &str,
    port: u16,
) -> (ManagedProcess, ManagedProcess, ManagedProcess, AicontrolClient, AicontrolClient, String) {
    let bins = binloc::locate().expect("locate binaries — `cargo build -p xgen-node -p xgen-client`?");
    let node = ManagedProcess::init_and_spawn_node(&bins, &instance_label(tag, "node"), port, true, None)
        .expect("spawn node");
    let url = format!("ws://127.0.0.1:{port}/xgen");
    let alice = ManagedProcess::init_and_spawn_client(&bins, &instance_label(tag, "alice"), &url, false, None)
        .expect("spawn alice");
    let bob = ManagedProcess::init_and_spawn_client(&bins, &instance_label(tag, "bob"), &url, false, None)
        .expect("spawn bob");
    let a = AicontrolClient::connect(&alice.aicontrol_pipe, DEFAULT_CONNECT_TIMEOUT)
        .await
        .expect("connect alice aicontrol");
    let b = AicontrolClient::connect(&bob.aicontrol_pipe, DEFAULT_CONNECT_TIMEOUT)
        .await
        .expect("connect bob aicontrol");
    (node, alice, bob, a, b, url)
}

// ── V-9a — the leg's subject ──────────────────────────────────────────────────

/// `V-9a` — a member who was INVITED, joined, left, and lost her local anchor
/// comes back. This is the ordinary shape: almost nobody joins a Space without
/// having been invited to it first.
#[tokio::test]
#[ignore = "heavy: spawns a real xgen-node + 2 real client residents; box-gated RUN"]
async fn v9a_invited_member_rejoins_after_losing_her_local_anchor() {
    let (_node, _alice, bob, mut a, mut b, _url) = rig("MP-G4-A", PORT_A).await;

    let alice_id = ok(&mut a, "register", json!({ "name": "alice" })).await;
    let _ = alice_id;
    let bob_reg = ok(&mut b, "register", json!({ "name": "bob" })).await;
    let bob_id = bob_reg.data_str("identity_id").expect("bob identity_id").to_string();

    let sp = ok(&mut a, "create-space", json!({ "name": "G4-A" })).await;
    let space = sp.data_str("space_id").expect("space_id").to_string();
    ok(&mut a, "create-room", json!({ "space": &space, "name": "general" })).await;
    ok(&mut a, "invite", json!({ "space": &space, "identity": &bob_id, "role": "member" })).await;

    ok(&mut b, "join", json!({ "space": &space })).await;
    assert!(is_present(&mut a, &space, &bob_id).await, "bob must be a member after his first join");

    ok(&mut b, "leave", json!({ "space": &space })).await;
    assert!(!is_present(&mut a, &space, &bob_id).await, "bob must not be present after leaving");

    // The fresh-install condition.
    clear_leave_anchor(&bob.data_dir, &space);

    // THE REJOIN. Printed verbatim BEFORE the assertion, so `V-9c`'s reverted
    // run captures the refusal even though the assertion then fails.
    let rejoin = call(&mut b, "join", json!({ "space": &space })).await;
    eprintln!("V-9a REJOIN REPLY (verbatim): {rejoin:?}");
    assert!(rejoin.is_ok(), "the rejoin was refused: {rejoin:?}");

    assert!(
        is_present(&mut a, &space, &bob_id).await,
        "bob must be a member again — `Accepted` alone is not enough, the rejoin \
         has to SURVIVE `derive_resolved` (that is what `3048` is about)"
    );
    eprintln!("V-9a PASS: an invited, departed member rejoined from a cleared anchor");
}

// ── V-9b — the true fresh install ─────────────────────────────────────────────

/// `V-9b` — a clean data directory holding bob's **same identity**, then rejoin.
///
/// ✅ **Reachability MEASURED, not assumed** (the runbook left it explicitly
/// unmeasured): `ManagedProcess::spawn_client_reusing_keypair` copies **only**
/// `xgen-client_keypair.enc` and `xgen-client_config.toml` into a NEW instance
/// data dir — it does not copy `xgen-client_state.json` and does not re-`init`
/// (which would mint a different `identity_id`). Same identity, empty state:
/// that is the true fresh-install condition exactly, with nothing surgical.
#[tokio::test]
#[ignore = "heavy: spawns a real xgen-node + 3 real client residents; box-gated RUN"]
async fn v9b_same_identity_on_a_clean_data_dir_rejoins() {
    let bins = binloc::locate().expect("locate binaries");
    let (_node, _alice, bob, mut a, mut b, url) = rig("MP-G4-B", PORT_B).await;

    ok(&mut a, "register", json!({ "name": "alice" })).await;
    let bob_reg = ok(&mut b, "register", json!({ "name": "bob" })).await;
    let bob_id = bob_reg.data_str("identity_id").expect("bob identity_id").to_string();

    let sp = ok(&mut a, "create-space", json!({ "name": "G4-B" })).await;
    let space = sp.data_str("space_id").expect("space_id").to_string();
    ok(&mut a, "create-room", json!({ "space": &space, "name": "general" })).await;
    ok(&mut a, "invite", json!({ "space": &space, "identity": &bob_id, "role": "member" })).await;

    ok(&mut b, "join", json!({ "space": &space })).await;
    assert!(is_present(&mut a, &space, &bob_id).await, "bob must be a member after his first join");
    ok(&mut b, "leave", json!({ "space": &space })).await;
    assert!(!is_present(&mut a, &space, &bob_id).await, "bob must not be present after leaving");

    // The reinstall: a brand-new instance, bob's keypair, NO state.json.
    let fresh = ManagedProcess::spawn_client_reusing_keypair(
        &bins,
        &instance_label("MP-G4-B", "bob2"),
        &bob.data_dir,
        &url,
        None,
    )
    .expect("spawn bob's reinstalled client reusing his keypair");
    assert!(
        !fresh.data_dir.join("xgen-client_state.json").exists(),
        "PRECONDITION: the reinstall must start with NO client state — otherwise \
         this is not a fresh install and the test would pass for the wrong reason"
    );
    let mut b2 = AicontrolClient::connect(&fresh.aicontrol_pipe, DEFAULT_CONNECT_TIMEOUT)
        .await
        .expect("connect the reinstalled client");

    // 📌 MEASURED, not assumed: a truly-fresh data dir has NO
    // `xgen-client_state.json`, and every verb refuses with *state file not
    // found — run `init` and `register` first*. So a real reinstall needs
    // `register --re-registration` before anything else — the documented
    // re-home shape (`mp_c06_rehome`). That is a genuine BOUND on the
    // "she reinstalls and comes back" story, and it is recorded here rather
    // than papered over: the keypair alone is not enough to act.
    ok(&mut b2, "register", json!({ "name": "bob", "re_registration": true })).await;

    let who = ok(&mut b2, "whoami", json!({})).await;
    assert_eq!(
        who.data_str("identity_id"),
        Some(bob_id.as_str()),
        "the reinstall must present the SAME identity — otherwise this is a new \
         person joining, not a member coming back"
    );

    let rejoin = call(&mut b2, "join", json!({ "space": &space })).await;
    eprintln!("V-9b REJOIN REPLY (verbatim): {rejoin:?}");
    assert!(rejoin.is_ok(), "the fresh-install rejoin was refused: {rejoin:?}");

    assert!(
        is_present(&mut a, &space, &bob_id).await,
        "bob must be a member again after reinstalling"
    );
    eprintln!("V-9b PASS: the same identity rejoined from a clean data directory");
}
