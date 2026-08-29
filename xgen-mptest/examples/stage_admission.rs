// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! M-ADMISSION-SOUNDNESS Leg 2 — **the staging harness for the screen halves.**
//!
//! ```text
//! cargo run -p xgen-mptest --example stage_admission
//! ```
//!
//! 🛑 **THIS IS NOT A TEST AND IT MUST NOT BECOME ONE.** A `cargo test` tears its
//! rig down at the end and leaves nothing to attach to. `S-3` and `S-4` are the
//! two situations whose ruled behaviour **is an experience**, and the harness
//! sees the wire and never the screen — so this drives a world into the staged
//! state, **leaves the node alive**, and blocks until a person has looked.
//!
//! ⚠️ **An EXAMPLE, not a `[[bin]]`.** `xgen-mptest`'s own manifest header says
//! *"Not a shipped artifact; never depended on by a binary"*, and a `[[bin]]`
//! would contradict that in the crate that exists to be black-box. An example
//! adds no shipped surface and is **built but never run** by `cargo test`, so
//! the workspace floor cannot move because of it (`V-1`).
//!
//! ## What it stages — ONE world that serves both screen halves
//!
//! alice creates a Space with **three rooms** (`S-4`'s shape), invites bob; bob
//! joins the Space and all three rooms and says one thing. alice says one thing
//! **while he is present** — the discriminator without which *his history is
//! empty* and *his history withholds the gap* read alike. bob leaves. In the
//! gap, carol and dave arrive, **carol is removed**, and alice says three more
//! things (`S-3`'s shape). bob rejoins. Then it stops and waits.
//!
//! ## Who owns the instance dir — the constraint that shapes the handover
//!
//! 🛑 **The Tauri shell starts its own `.aicontrol` server on the same derived
//! pipe name** (`desktop.rs`, the `aicontrol_pipe_name(&pipe_name)` spawn), and
//! it holds the same `xgen-client_state.json`. A `--service` resident and a GUI
//! on one `--instance` label would therefore contend for both. ⇒ **this harness
//! KILLS alice's and bob's residents before it hands over**, keeping their data
//! dirs, so that exactly one process owns each dir at a time. The node stays up;
//! it is what the GUIs connect to.
//!
//! 📌 Not a line from the runbook — a constraint found by reading `desktop.rs`
//! while writing this, and recorded here rather than left to be discovered when
//! two processes quietly fought over one state file.
//!
//! ## What this harness does NOT solve, and will not pretend to
//!
//! 🛑 **The built `xgen-client.exe` has no CDP port.** `additionalBrowserArgs`
//! lives only in `xgen-client/cdp.dev.conf.json`, a **dev-only overlay** applied
//! by `cargo tauri dev --config cdp.dev.conf.json` (`D-104`) — the base
//! `tauri.conf.json` has none. ⇒ the launch line printed below **shows the
//! screen and exposes no `9222`.**
//!
//! 🛑 **And the debug launcher cannot be pointed at a staged instance.**
//! `run-client.ps1 -Debug` takes `-Mode`/`-Debug` and nothing else — no
//! `--data-dir`, no `--instance` — and pins Vite to **5173 strictPort** and CDP
//! to **9222**, so it also cannot run twice at once.
//!
//! ⇒ **Two GUIs side by side is not reachable through the shipped launcher, and
//! captures are SEQUENTIAL: one identity, look, close, the next.** This harness
//! is built for exactly that — it blocks until Enter, so both data dirs and the
//! node stay alive across as many launches as the viewing needs.
//!
//! **Reported, not worked around** (Rule 6). Whether the CDP route is
//! `cargo tauri dev --config cdp.dev.conf.json -- -- --data-dir <dir>
//! --instance <label>` is **untested by this seat** and belongs to whoever
//! drives the viewing session.

use serde_json::json;

use xgen_mptest::aicontrol::{AicontrolClient, DEFAULT_CONNECT_TIMEOUT};
use xgen_mptest::binloc;
use xgen_mptest::process::{exe_dir, instance_label, ManagedProcess};
use xgen_mptest::wire::{Command, Reply};

/// 🛑 **8596 is this harness's own.** `mp_admission_soundness.rs` holds
/// 8592-8595 (Leg 1) and 8597-8598 (Leg 2's wire halves), and deliberately
/// skips this one: a test and this example can be alive on one box at the same
/// time, and a port held by a live process is held regardless of which target
/// spawned it. **The next file to need one starts at 8599.**
const PORT: u16 = 8596;

/// The marker the driving session polls for. 🛑 **Polled for, never waited out
/// on elapsed time** — `N-206`'s family: a fixed sleep reports *ready* whether
/// or not anything started.
const READY_MARKER: &str = "STAGE-ADMISSION-READY";

// ── drive ─────────────────────────────────────────────────────────────────────

/// Send one verb and return whatever comes back. A refusal is an observation.
async fn call(c: &mut AicontrolClient, cmd: &str, args: serde_json::Value) -> Reply {
    let mut m = Command::new(cmd);
    if let serde_json::Value::Object(o) = args {
        m.args = o;
    }
    c.send(&m)
        .await
        .unwrap_or_else(|e| panic!("aicontrol `{cmd}` transport failure: {e}"))
}

/// Send one verb and require it. **Staging only** — a failure here means the
/// world was never built, so there is nothing to look at and stopping is right.
async fn ok(c: &mut AicontrolClient, cmd: &str, args: serde_json::Value) -> Reply {
    let r = call(c, cmd, args).await;
    assert!(
        r.is_ok(),
        "staging verb `{cmd}` failed — the world was never built: {r:?}"
    );
    r
}

fn say(s: impl std::fmt::Display) {
    println!("{s}");
}

fn rule() {
    say("──────────────────────────────────────────────────────────────────────");
}

#[tokio::main]
async fn main() {
    let bins = binloc::locate().expect(
        "locate binaries — run `cargo build -p xgen-node -p xgen-client` first",
    );
    let exedir = exe_dir(&bins);
    let url = format!("ws://127.0.0.1:{PORT}/xgen");

    let alice_label = instance_label("STAGE", "alice");
    let bob_label = instance_label("STAGE", "bob");

    rule();
    say("M-ADMISSION-SOUNDNESS Leg 2 — staging the world the screens will show.");
    rule();

    // ── the rig ───────────────────────────────────────────────────────────────
    let node = ManagedProcess::init_and_spawn_node(
        &bins,
        &instance_label("STAGE", "node"),
        PORT,
        true,
        None,
    )
    .expect("spawn node");
    say(format!("node up on {url} (pid {})", node.pid()));

    let mut alice_p =
        ManagedProcess::init_and_spawn_client(&bins, &alice_label, &url, false, None)
            .expect("spawn alice");
    let mut bob_p = ManagedProcess::init_and_spawn_client(&bins, &bob_label, &url, false, None)
        .expect("spawn bob");

    let mut a = AicontrolClient::connect(&alice_p.aicontrol_pipe, DEFAULT_CONNECT_TIMEOUT)
        .await
        .expect("connect alice aicontrol");
    let mut b = AicontrolClient::connect(&bob_p.aicontrol_pipe, DEFAULT_CONNECT_TIMEOUT)
        .await
        .expect("connect bob aicontrol");

    // ── the world ─────────────────────────────────────────────────────────────
    ok(&mut a, "register", json!({ "name": "alice" })).await;
    let bob_id = ok(&mut b, "register", json!({ "name": "bob" }))
        .await
        .data_str("identity_id")
        .expect("bob identity_id")
        .to_string();

    let space = ok(&mut a, "create-space", json!({ "name": "Studio" }))
        .await
        .data_str("space_id")
        .expect("space_id")
        .to_string();
    let mut rooms = Vec::new();
    for name in ["general", "design", "random"] {
        let r = ok(&mut a, "create-room", json!({ "space": &space, "name": name })).await;
        rooms.push((
            name,
            r.data_str("room_id").expect("room_id").to_string(),
        ));
    }
    say(format!("alice created Space `Studio` ({space}) with 3 rooms"));

    ok(
        &mut a,
        "invite",
        json!({ "space": &space, "identity": &bob_id, "role": "member" }),
    )
    .await;
    ok(&mut b, "join", json!({ "space": &space })).await;
    for (_, rid) in &rooms {
        ok(&mut b, "join", json!({ "space": &space, "room": rid })).await;
    }
    let general = &rooms[0].1;
    ok(
        &mut b,
        "send",
        json!({ "space": &space, "room": general, "text": "morning all" }),
    )
    .await;

    // Said while bob is PRESENT — the discriminator the whole gap reading rests
    // on. Without it, a screen showing nothing from the gap is indistinguishable
    // from a screen showing nothing at all.
    ok(
        &mut a,
        "send",
        json!({ "space": &space, "room": general, "text": "BEFORE-THE-GAP: bob can see this one" }),
    )
    .await;
    say("bob joined the Space and all 3 rooms; one message each side of him");

    ok(&mut b, "leave", json!({ "space": &space })).await;
    say("bob left");

    // ── the gap ───────────────────────────────────────────────────────────────
    // carol and dave JOIN before carol is removed. A ban on someone who never
    // joined removes nobody, and *whether her removal is visible* would then be
    // a question about an event that never happened.
    let carol_p = ManagedProcess::init_and_spawn_client(
        &bins,
        &instance_label("STAGE", "carol"),
        &url,
        false,
        None,
    )
    .expect("spawn carol");
    let mut c = AicontrolClient::connect(&carol_p.aicontrol_pipe, DEFAULT_CONNECT_TIMEOUT)
        .await
        .expect("connect carol aicontrol");
    let carol_id = ok(&mut c, "register", json!({ "name": "carol" }))
        .await
        .data_str("identity_id")
        .expect("carol identity_id")
        .to_string();

    let dave_p = ManagedProcess::init_and_spawn_client(
        &bins,
        &instance_label("STAGE", "dave"),
        &url,
        false,
        None,
    )
    .expect("spawn dave");
    let mut d = AicontrolClient::connect(&dave_p.aicontrol_pipe, DEFAULT_CONNECT_TIMEOUT)
        .await
        .expect("connect dave aicontrol");
    let dave_id = ok(&mut d, "register", json!({ "name": "dave" }))
        .await
        .data_str("identity_id")
        .expect("dave identity_id")
        .to_string();

    ok(
        &mut a,
        "invite",
        json!({ "space": &space, "identity": &carol_id, "role": "member" }),
    )
    .await;
    ok(
        &mut a,
        "invite",
        json!({ "space": &space, "identity": &dave_id, "role": "member" }),
    )
    .await;
    ok(&mut c, "join", json!({ "space": &space })).await;
    ok(&mut d, "join", json!({ "space": &space })).await;
    let ban = call(&mut a, "ban", json!({ "space": &space, "identity": &carol_id })).await;
    say(format!("in the gap: carol and dave arrived; alice removed carol -> {}",
        if ban.is_ok() { "ACCEPTED".to_string() } else { format!("{ban:?}") }));

    for n in 1..=3 {
        ok(
            &mut a,
            "send",
            json!({ "space": &space, "room": general, "text": format!("GAP-MESSAGE-{n}: bob was away for this one") }),
        )
        .await;
    }
    say("in the gap: alice said three things");

    // ── she comes back ────────────────────────────────────────────────────────
    let rejoin = call(&mut b, "join", json!({ "space": &space })).await;
    assert!(
        rejoin.is_ok(),
        "bob's rejoin was refused — the staged world is not the one to look at: {rejoin:?}"
    );
    say("bob rejoined — the world is staged");

    // ── hand over ─────────────────────────────────────────────────────────────
    // carol and dave played their part; drop them normally so their instance
    // dirs go with them. alice and bob keep theirs — the GUIs need them.
    drop(c);
    drop(d);
    drop(carol_p);
    drop(dave_p);

    drop(a);
    drop(b);
    alice_p.keep_artifacts();
    bob_p.keep_artifacts();
    let alice_dir = alice_p.data_dir.clone();
    let bob_dir = bob_p.data_dir.clone();
    drop(alice_p);
    drop(bob_p);
    say("alice's and bob's residents stopped — their instance dirs are free for a GUI");

    rule();
    say(format!("{READY_MARKER}"));
    rule();
    say(format!("node URL      {url}"));
    say(format!("space         {space}"));
    for (name, rid) in &rooms {
        say(format!("room `{name}`   {rid}"));
    }
    say("");
    say(format!("alice data dir  {}", alice_dir.display()));
    say(format!("bob   data dir  {}", bob_dir.display()));
    say("");
    say("GUI launch lines — the ARGUMENTS each identity's client needs:");
    say(format!(
        "  alice   \"{}\" --data-dir \"{}\" --instance {}",
        bins.client.display(),
        exedir.display(),
        alice_label
    ));
    say(format!(
        "  bob     \"{}\" --data-dir \"{}\" --instance {}",
        bins.client.display(),
        exedir.display(),
        bob_label
    ));
    say("");
    say("NOTE — these lines open the Tauri shell and expose NO CDP port. The");
    say("       remote-debugging port lives only in xgen-client/cdp.dev.conf.json,");
    say("       a dev-only overlay applied by `cargo tauri dev --config ...`");
    say("       (D-104); the base tauri.conf.json has none. run-client.ps1 -Debug");
    say("       takes no --data-dir/--instance and pins Vite 5173 + CDP 9222, so");
    say("       it can neither be aimed at a staged instance nor run twice.");
    say("       => captures are SEQUENTIAL: launch one, look, close, launch the");
    say("       other. This harness stays up across all of them.");
    rule();
    say("Press Enter to tear down (node + both instance dirs). Close any GUI first.");
    rule();

    // Blocking read on a dedicated thread — never on a runtime worker.
    let _ = tokio::task::spawn_blocking(|| {
        let mut s = String::new();
        let _ = std::io::stdin().read_line(&mut s);
    })
    .await;

    // ── teardown, printed ─────────────────────────────────────────────────────
    rule();
    say("tearing down...");
    let node_pid = node.pid();
    drop(node);
    say(format!("  node (pid {node_pid}) stopped"));
    for (who, dir) in [("alice", &alice_dir), ("bob", &bob_dir)] {
        match std::fs::remove_dir_all(dir) {
            Ok(()) => say(format!("  {who} instance dir removed: {}", dir.display())),
            Err(e) => say(format!(
                "  {who} instance dir NOT removed ({e}): {} — remove it by hand",
                dir.display()
            )),
        }
    }
    say("teardown complete.");
    rule();
}
