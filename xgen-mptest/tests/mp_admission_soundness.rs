// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! M-ADMISSION-SOUNDNESS Leg 1 — the four wire situations (`#[ignore]`, box-gated).
//!
//! 🛑 **THIS FILE DOES NOT TEST. IT OBSERVES.** *A test asserts what we already
//! decided; a simulation shows us what we never asked.* Every instinct in a
//! `tests/` directory pulls toward `assert!` — resisted here deliberately.
//! Assertions appear ONLY where a failure would mean the RIG is broken (a setup
//! verb failed, a precondition never held). The findings are **printed**.
//!
//! Concretely, and this is the whole point: `S-2` prints bob's role before and
//! after his rejoin and asserts NEITHER. `state.rs:1273-1276` says it comes back
//! `Role::Member`. If it did not, an `assert_eq!(role, "member")` would report
//! that as *this leg failing* rather than as *a contradiction of `D-154`①* —
//! which is far bigger than a red test. Likewise `S-8` prints three refusals
//! adjacent and asserts nothing about their equality: if they are identical the
//! transcript shows it as text, and if they differ an assertion would have
//! buried the finding.
//!
//! ```text
//! cargo build -p xgen-node && cargo build -p xgen-client
//! cargo test -p xgen-mptest --test mp_admission_soundness -- --ignored \
//!     --test-threads=1 --nocapture
//! ```
//!
//! ## The deliverable is the `READING` line
//!
//! Each situation prints a header, its observations, a footer, and one
//! plain-English `READING` line saying what a person in bob's seat would and
//! would not know. If that line cannot be written honestly from what was
//! printed, the situation did not observe enough — which is itself a finding.
//!
//! ## Scope (`Q-1` ruled **B** at J-780)
//!
//! - **In:** `S-1` (leave/return), `S-2` (the silent demotion), `S-7`
//!   (node-eject → rejoin → unban → rejoin), `S-8` (stranger vs spent invite
//!   vs live invite).
//! - **Out:** `S-3`/`S-4` are Leg 2 (they need a live rig and Joe's eyes).
//!   `S-5`/`S-6`/`S-10` are **not drivable at all** — `membership.kick` and
//!   `state.space_admission` have a complete receiving half and **no emitter on
//!   any shipped surface**. They are `M-ADMISSION-SURFACE`'s.
//!
//! 🛑 Those two are deliberately NOT staged by constructing their events
//! fixture-style. Building the event directly is exactly what let the gap hide
//! for the whole of `M-SPACE-ADMISSION`: every leg gate built its own events, so
//! no leg ever needed a verb, so no leg ever noticed there wasn't one.
//!
//! ## Bounds, stated rather than discovered
//!
//! - The harness sees **the wire, not the screen**. It reaches the payload; what
//!   a person is *shown* is not observable here.
//! - A spawn/connect timeout is a **flake**, not a failure — re-run isolated and
//!   state how many runs the recorded result took.
//! - `#[ignore]` always ⇒ this file contributes **ZERO** to the `cargo` floor.

use serde_json::json;

use xgen_mptest::aicontrol::{AicontrolClient, DEFAULT_CONNECT_TIMEOUT};
use xgen_mptest::binloc;
use xgen_mptest::process::{instance_label, ManagedProcess};
use xgen_mptest::wire::{Command, Reply};

// Ports are per-run and per-situation. Swept across `xgen-mptest/tests/` and
// `src/` at `ed8f789`: the highest port otherwise in use is 8591
// (`mp_g4_rejoin_e2e`, which holds 8590 and 8591). 8592-8595 are free — the
// next file to need one should start at 8596.
const PORT_S1: u16 = 8592;
const PORT_S2: u16 = 8593;
const PORT_S7: u16 = 8594;
const PORT_S8: u16 = 8595;

/// A deliberate pause. `S-8` needs one and says why at its call site.
async fn pause(ms: u64) {
    tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
}

/// Send one `--aicontrol` verb and return the reply, whatever it is.
///
/// 🔑 **This is this leg's default.** A refusal is the OBSERVATION here, not a
/// failure — so nothing in this helper judges the reply.
async fn call(c: &mut AicontrolClient, cmd: &str, args: serde_json::Value) -> Reply {
    let mut m = Command::new(cmd);
    if let serde_json::Value::Object(o) = args {
        m.args = o;
    }
    c.send(&m).await.unwrap_or_else(|e| panic!("aicontrol `{cmd}` transport failure: {e}"))
}

/// Same, asserting success — **for rig setup only**, where a failure means the
/// fixture is broken rather than that something was learned.
async fn ok(c: &mut AicontrolClient, cmd: &str, args: serde_json::Value) -> Reply {
    let r = call(c, cmd, args).await;
    assert!(r.is_ok(), "setup verb `{cmd}` failed — the rig is broken, not the protocol: {r:?}");
    r
}

/// The membership oracle, read ONCE per observation.
///
/// `members` re-derives the roster through the node's own `derive_resolved` and
/// projects only `is_present()` members (`D-154`), so it answers *did this
/// SURVIVE RESOLUTION*, not merely *did the node say Accepted* — the distinction
/// `3048` exists to draw.
///
/// 📌 Presence and role come out of **one** snapshot on purpose. Two separate
/// `members` calls could straddle a change, and a printed
/// `role = admin, present = false` pair assembled from two instants would be a
/// statement about no moment that ever existed.
///
/// 🛑 Driven from an observer who is still a member: a departed member cannot
/// drain (`collect_sync_history` gates on `is_member`), so bob cannot be his own
/// oracle here — that is a bound on what he can see, not a convenience.
async fn roster(observer: &mut AicontrolClient, space: &str) -> Vec<serde_json::Value> {
    let r = ok(observer, "members", json!({ "space": space })).await;
    r.data()
        .and_then(|d| d.get("members"))
        .and_then(|m| m.as_array())
        .cloned()
        .unwrap_or_default()
}

/// `(present, role)` for one identity, from a single roster read. `role` is
/// `None` when the identity is not in the projected roster at all — which is
/// itself the answer, and is printed as such rather than defaulted.
async fn presence_and_role(
    observer: &mut AicontrolClient,
    space: &str,
    who: &str,
) -> (bool, Option<String>) {
    let rows = roster(observer, space).await;
    let row = rows
        .iter()
        .find(|m| m.get("identity_id").and_then(|v| v.as_str()) == Some(who));
    match row {
        Some(m) => (
            true,
            m.get("role").and_then(|v| v.as_str()).map(str::to_string),
        ),
        None => (false, None),
    }
}

/// bob's own view of his rooms — a read of **his** `xgen-client_state.json`
/// (`ops::rooms` is synchronous and never touches the node). Labelled as his
/// belief in the transcript, because that is what it is: what a person in his
/// seat would see, which need not match the node.
async fn own_rooms(c: &mut AicontrolClient, space: &str) -> String {
    let r = call(c, "rooms", json!({ "space": space })).await;
    match r.data().and_then(|d| d.get("rooms")).and_then(|v| v.as_array()) {
        Some(rooms) => {
            let names: Vec<String> = rooms
                .iter()
                .map(|x| {
                    x.get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("<unnamed>")
                        .to_string()
                })
                .collect();
            format!("[{}]", names.join(", "))
        }
        None => format!("<no rooms readable: {r:?}>"),
    }
}

/// A reply rendered for a person: the outcome, then the code when there is one.
/// The full `Debug` is printed separately and verbatim — this is the legible
/// summary that sits beside it, never a replacement for it.
fn verdict(r: &Reply) -> String {
    match r.error() {
        None => "ACCEPTED".to_string(),
        Some(e) => {
            let reject = e
                .reject_code
                .map(|c| format!(", node reject_code={c}"))
                .unwrap_or_default();
            format!("REFUSED [{} / {}{}] {}", e.code, e.category, reject, e.message)
        }
    }
}

// ── transcript ────────────────────────────────────────────────────────────────

fn hdr(id: &str, title: &str) {
    eprintln!("\n── {id} · {title} {}", "─".repeat(60usize.saturating_sub(title.len())));
}
fn ftr(id: &str) {
    eprintln!("── end {id} {}\n", "─".repeat(64));
}
fn line(label: &str, text: impl std::fmt::Display) {
    eprintln!("   {label:<10} {text}");
}

/// Spawn a node + alice + bob and connect to both client pipes. The node process
/// is returned so a situation can also drive the NODE's own admin pipe (`S-7`),
/// and the URL so a situation can add a third identity (`S-8`).
///
/// Follows `mp_g4_rejoin_e2e::rig` — a second shape for the same job would be a
/// drift surface. (Each `tests/*.rs` is its own crate, so this is a copy of the
/// pattern rather than a shared import; the shared surface lives in
/// `xgen-mptest/src/`, and nothing here needed to move into it.)
async fn rig(
    tag: &str,
    port: u16,
) -> (ManagedProcess, ManagedProcess, ManagedProcess, AicontrolClient, AicontrolClient, String) {
    let bins = binloc::locate()
        .expect("locate binaries — `cargo build -p xgen-node -p xgen-client` first?");
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

/// alice registers, creates a Space and a room, and invites bob. Returns
/// `(space_id, room_id, bob_identity_id)`. Every step is `ok` — a failure here
/// is a broken fixture, not an observation.
async fn setup(
    a: &mut AicontrolClient,
    b: &mut AicontrolClient,
    space_name: &str,
    bob_role: &str,
    valid_for_days: Option<u32>,
) -> (String, String, String) {
    ok(a, "register", json!({ "name": "alice" })).await;
    let bob_reg = ok(b, "register", json!({ "name": "bob" })).await;
    let bob_id = bob_reg.data_str("identity_id").expect("bob identity_id").to_string();

    let sp = ok(a, "create-space", json!({ "name": space_name })).await;
    let space = sp.data_str("space_id").expect("space_id").to_string();
    let rm = ok(a, "create-room", json!({ "space": &space, "name": "general" })).await;
    let room = rm.data_str("room_id").expect("room_id").to_string();

    let mut inv = json!({ "space": &space, "identity": &bob_id, "role": bob_role });
    if let Some(d) = valid_for_days {
        inv["valid_for_days"] = json!(d);
    }
    ok(a, "invite", inv).await;
    (space, room, bob_id)
}

// ── S-1 — she leaves in the morning and comes back in the evening ─────────────

/// The ordinary shape, and the control the other three are read against.
#[tokio::test]
#[ignore = "heavy: spawns a real xgen-node + 2 real client residents; box-gated RUN"]
async fn s1_member_leaves_and_comes_back() {
    let (_node, _alice, _bobp, mut a, mut b, _url) = rig("MP-AS-S1", PORT_S1).await;
    hdr("S-1", "a member leaves and comes back");

    let (space, room, bob_id) = setup(&mut a, &mut b, "AS-S1", "member", None).await;
    line("setup", format!("alice created {space}, invited bob as member"));

    ok(&mut b, "join", json!({ "space": &space })).await;
    ok(&mut b, "join", json!({ "space": &space, "room": &room })).await;
    let sent = call(&mut b, "send", json!({ "space": &space, "room": &room, "text": "morning" })).await;
    line("action", format!("bob joined space + room, sent one message -> {}", verdict(&sent)));

    let (p0, r0) = presence_and_role(&mut a, &space, &bob_id).await;
    let rooms0 = own_rooms(&mut b, &space).await;
    line("before", format!("present = {p0}, role = {r0:?}, bob's own rooms = {rooms0}"));

    let left = ok(&mut b, "leave", json!({ "space": &space })).await;
    line("action", format!("bob leave (whole Space) -> {}", verdict(&left)));
    let (p1, _) = presence_and_role(&mut a, &space, &bob_id).await;
    line("gap", format!("present = {p1}"));

    // THE REJOIN — printed verbatim BEFORE any assertion, so a refusal is
    // captured even when the assertion below then fails.
    let rejoin = call(&mut b, "join", json!({ "space": &space })).await;
    line("reply", format!("rejoin -> {}", verdict(&rejoin)));
    eprintln!("   verbatim   {rejoin:?}");

    let (p2, r2) = presence_and_role(&mut a, &space, &bob_id).await;
    let rooms2 = own_rooms(&mut b, &space).await;
    let hist = call(&mut b, "history", json!({ "space": &space, "room": &room })).await;
    let hist_n = hist
        .data()
        .and_then(|d| d.get("messages"))
        .and_then(|v| v.as_array())
        .map(|a| a.len() as i64)
        .unwrap_or(-1);
    line("after", format!("present = {p2}, role = {r2:?}, bob's own rooms = {rooms2}"));
    line("after", format!("bob's history for the room -> {} (messages read: {hist_n})", verdict(&hist)));

    // The only assertions in this situation. `D-154`① presence half, already
    // proven by `V-9a` — here they are the rig's own sanity, not the finding.
    assert!(rejoin.is_ok(), "the ordinary rejoin was refused — the rig or the protocol is broken: {rejoin:?}");
    assert!(p2, "bob must be present again after an accepted rejoin (it has to survive `derive_resolved`)");

    // Corrected from the observed run: the provisional line predicted that losing
    // room membership would cost him his reach. It did not — he read the history.
    line(
        "READING",
        "bob came back with the same role and the same reach: he still reads the \
         room's history. But his own client holds no record of the Space at all \
         (`rooms` fails identically before and after), so nothing he can see \
         locally reflects a membership the node confirms.",
    );
    ftr("S-1");
}

// ── S-2 — the silent demotion ─────────────────────────────────────────────────

/// 🔑 **The situation this leg exists for.** An admin leaves and rejoins on her
/// own anchor. `state.rs:1273-1276` re-derives `(role, invited_by)` from
/// `pending_invites`, which her FIRST join consumed — absent ⇒ `Role::Member`.
/// *Presence, never position.* The protocol is behaving exactly as `D-154`①
/// rules; the question this situation asks is **whether anything tells her**.
///
/// 🛑 Nothing here asserts the role in either direction. If it comes back
/// `admin`, that contradicts `D-154`① and is a far larger finding than a red
/// test — an assertion would have reported it as *this leg failing* instead.
#[tokio::test]
#[ignore = "heavy: spawns a real xgen-node + 2 real client residents; box-gated RUN"]
async fn s2_admin_leaves_and_returns_silently_demoted() {
    let (_node, _alice, _bobp, mut a, mut b, _url) = rig("MP-AS-S2", PORT_S2).await;
    hdr("S-2", "an admin leaves and comes back");

    let (space, room, bob_id) = setup(&mut a, &mut b, "AS-S2", "admin", None).await;
    line("setup", format!("alice created {space}, invited bob as ADMIN"));

    ok(&mut b, "join", json!({ "space": &space })).await;
    ok(&mut b, "join", json!({ "space": &space, "room": &room })).await;

    // Confirmed BEFORE he leaves — otherwise "he was an admin" is an assumption
    // about the fixture rather than something this run observed.
    let (p0, r0) = presence_and_role(&mut a, &space, &bob_id).await;
    let rooms0 = own_rooms(&mut b, &space).await;
    line("before", format!("bob role = {r0:?}, present = {p0}, bob's own rooms = {rooms0}"));
    assert_eq!(
        r0.as_deref(),
        Some("admin"),
        "PRECONDITION: bob must actually hold admin before he leaves, or this \
         situation observes nothing. This asserts the FIXTURE, not the finding."
    );

    let left = ok(&mut b, "leave", json!({ "space": &space })).await;
    line("action", format!("bob leave -> {}", verdict(&left)));

    // NO NEW INVITE IS ISSUED. That is the situation.
    let rejoin = call(&mut b, "join", json!({ "space": &space })).await;
    line("action", "bob join (no new invite; anchored on his own events)");
    line("reply", verdict(&rejoin));
    eprintln!("   verbatim   {rejoin:?}");

    // "Anything at all in the reply that mentions a role" — demonstrated
    // mechanically rather than claimed, so the SILENCE is a measurement.
    let raw = format!("{rejoin:?}");
    let mentions: Vec<&str> = ["role", "admin", "member", "demot", "privile"]
        .into_iter()
        .filter(|w| raw.to_lowercase().contains(w))
        .collect();
    line("role-words", format!("terms present in the rejoin reply: {mentions:?}"));

    let (p2, r2) = presence_and_role(&mut a, &space, &bob_id).await;
    let rooms2 = own_rooms(&mut b, &space).await;
    line("after", format!("bob role = {r2:?}, present = {p2}, bob's own rooms = {rooms2}"));
    line("compare", format!("role before = {r0:?}  ->  role after = {r2:?}"));

    assert!(rejoin.is_ok(), "the rejoin was refused, so the role question was never reached: {rejoin:?}");

    line(
        "READING",
        "bob was an admin, left, and came back a member. The reply that readmitted \
         him is shaped exactly like an ordinary member's and carries no role term \
         at all, so nothing on this surface tells him his standing changed and \
         nothing he receives would let him find out.",
    );
    ftr("S-2");
}

// ── S-7 — node-eject, a rejoin attempt, un-ban, another attempt ───────────────

/// The node operator removes bob, bob tries to come back, the operator reverses
/// it, bob tries again. `apply_node_eject` marks him departed AND bans him
/// (`state.rs:1410-1412`); `apply_node_unban` lifts the ban only
/// (`state.rs:1432`) — it does not restore membership.
///
/// 📌 `D-154`⑥'s carried cost is the thing to look at: retention makes the
/// ejection a durable federated record while `node_eject` is itself reversible,
/// so a reversed ejection still leaves the record saying it happened. **Print,
/// do not judge.**
#[tokio::test]
#[ignore = "heavy: spawns a real xgen-node + 2 real client residents; box-gated RUN"]
async fn s7_node_eject_rejoin_unban_rejoin() {
    let (node, _alice, _bobp, mut a, mut b, _url) = rig("MP-AS-S7", PORT_S7).await;
    hdr("S-7", "node-eject, rejoin, un-ban, rejoin");

    let (space, room, bob_id) = setup(&mut a, &mut b, "AS-S7", "member", None).await;
    ok(&mut b, "join", json!({ "space": &space })).await;
    ok(&mut b, "join", json!({ "space": &space, "room": &room })).await;
    let (p0, r0) = presence_and_role(&mut a, &space, &bob_id).await;
    line("setup", format!("bob joined {space}: present = {p0}, role = {r0:?}"));

    // The THIRD pipe — the NODE's own admin surface, not a client's.
    let mut n = AicontrolClient::connect(&node.aicontrol_pipe, DEFAULT_CONNECT_TIMEOUT)
        .await
        .expect("connect the NODE aicontrol pipe");

    // Node verb args marshal by serde field name (AC-D1): `space_id` /
    // `identity_id`, not the client's `--space` / `--identity`.
    let eject = call(&mut n, "space force-eject",
        json!({ "space_id": &space, "identity_id": &bob_id, "reason": "MP-AS-S7 simulation" })).await;
    line("action", format!("node: space force-eject bob -> {}", verdict(&eject)));
    line("event", format!("eject event_id = {:?}", eject.data_str("event_id")));
    eprintln!("   verbatim   {eject:?}");

    let (p1, _) = presence_and_role(&mut a, &space, &bob_id).await;
    line("after", format!("present = {p1}"));

    let try1 = call(&mut b, "join", json!({ "space": &space })).await;
    line("attempt-1", format!("bob rejoin while banned -> {}", verdict(&try1)));
    eprintln!("   verbatim   {try1:?}");
    let (p2, _) = presence_and_role(&mut a, &space, &bob_id).await;
    line("after", format!("present = {p2}"));

    let unban = call(&mut n, "space unban",
        json!({ "space_id": &space, "identity_id": &bob_id, "reason": "MP-AS-S7 reversal" })).await;
    line("action", format!("node: space unban bob -> {}", verdict(&unban)));
    line("event", format!("unban event_id = {:?}", unban.data_str("event_id")));

    let (p3, _) = presence_and_role(&mut a, &space, &bob_id).await;
    line("after", format!("present after un-ban (before any rejoin) = {p3}"));

    let try2 = call(&mut b, "join", json!({ "space": &space })).await;
    line("attempt-2", format!("bob rejoin after un-ban -> {}", verdict(&try2)));
    eprintln!("   verbatim   {try2:?}");

    let (p4, r4) = presence_and_role(&mut a, &space, &bob_id).await;
    // Feeds S-2's reading: if he is back, on what terms?
    line("after", format!("present = {p4}, role = {r4:?}"));
    line("compare", format!("role before eject = {r0:?}  ->  role after un-ban = {r4:?}"));

    line(
        "READING",
        "the ban door speaks plainly - bob is told he is banned, naming him and the \
         Space. Un-banning alone does not put him back; he stays absent until he \
         asks again, and when he does he is readmitted as a plain member, with \
         nothing said about the ejection that stays in the record.",
    );
    ftr("S-7");
}

// ── S-8 — a stranger, a spent invite, and a live one ──────────────────────────

/// Three approaches to the SAME Space, printed adjacent so that whether they
/// read alike is visible as text rather than described.
///
/// 🔑 `1011` was ruled ONE WORD FOR EVERY REFUSAL at J-779 on the
/// membership-oracle argument (`fanout.rs:824`). This is that ruling being
/// LOOKED at on a wire. 🛑 Nothing here asserts the three strings are equal: if
/// they are identical the transcript shows it, and if they differ an assertion
/// would have hidden the finding as a red test.
#[tokio::test]
#[ignore = "heavy: spawns a real xgen-node + 3 real client residents; box-gated RUN"]
async fn s8_stranger_spent_invite_and_live_invite() {
    let bins = binloc::locate().expect("locate binaries");
    let (_node, _alice, _bobp, mut a, mut b, url) = rig("MP-AS-S8", PORT_S8).await;
    hdr("S-8", "a stranger, a spent invite, and a live one");

    // bob is invited with a validity of ZERO days. `ops.rs:1182-1188` has NO
    // lower bound: `valid_until = now + 0 days`, i.e. the instant of the invite.
    let (space, _room, bob_id) = setup(&mut a, &mut b, "AS-S8", "member", Some(0)).await;
    line("setup", format!("alice created {space}; bob invited with --valid-for-days 0"));

    // carol: a real registered identity who was never invited and never a member.
    let carol_p = ManagedProcess::init_and_spawn_client(&bins, &instance_label("MP-AS-S8", "carol"), &url, false, None)
        .expect("spawn carol");
    let mut c = AicontrolClient::connect(&carol_p.aicontrol_pipe, DEFAULT_CONNECT_TIMEOUT)
        .await
        .expect("connect carol aicontrol");
    let carol_reg = ok(&mut c, "register", json!({ "name": "carol" })).await;
    let carol_id = carol_reg.data_str("identity_id").expect("carol identity_id").to_string();
    line("setup", format!("carol registered ({carol_id}); never invited, never a member"));

    // ① the stranger.
    let r1 = call(&mut c, "join", json!({ "space": &space })).await;

    // ② the spent invite. The gate is `now > valid_until`, STRICTLY greater
    // (`runtime.rs:1826-1828`) — so at `valid_until == now` the invite is NOT
    // yet expired. The sleep is what moves the clock unambiguously past the
    // stamp; without it this observation would be a race with sub-second
    // execution, and a pass would prove nothing about which side of the
    // boundary was tested.
    pause(2_000).await;
    let r2 = call(&mut b, "join", json!({ "space": &space })).await;

    // ③ the control — a normal invite, proving the rig admits anyone at all.
    // `apply_invite` REPLACES the pending invite for a target, so this
    // supersedes the spent one rather than sitting beside it.
    ok(&mut a, "invite", json!({ "space": &space, "identity": &bob_id, "role": "member" })).await;
    let r3 = call(&mut b, "join", json!({ "space": &space })).await;

    // Adjacent, under one header — the point of the situation.
    eprintln!("   ── the three replies, verbatim and adjacent ──");
    eprintln!("   (1) stranger      {}", verdict(&r1));
    eprintln!("       verbatim      {r1:?}");
    eprintln!("   (2) spent invite  {}", verdict(&r2));
    eprintln!("       verbatim      {r2:?}");
    eprintln!("   (3) live invite   {}", verdict(&r3));
    eprintln!("       verbatim      {r3:?}");

    let (pc, rc) = presence_and_role(&mut a, &space, &carol_id).await;
    let (pb, rb) = presence_and_role(&mut a, &space, &bob_id).await;
    line("after", format!("carol present = {pc}, role = {rc:?}"));
    line("after", format!("bob   present = {pb}, role = {rb:?}"));

    line(
        "READING",
        "carol, who was never invited, walked in; bob, who was invited and whose \
         invite had lapsed, was turned away and told the exact deadline he missed. \
         Holding a spent invite left him worse off than having no relationship to \
         the Space at all.",
    );
    ftr("S-8");
}
