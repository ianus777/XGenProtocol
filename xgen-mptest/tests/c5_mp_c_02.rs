// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Round-0 cooperative smoke — **MP-C-02** (invite & join), single-node.
//!
//! Joe-locked Option 1: run on ONE node (alice + bob both on A). This proves the
//! full harness machinery end-to-end against the real binaries —
//! spawn → `.aicontrol` drive → cross-actor `{{}}` (`bob_identity_id` /
//! `space_id` / `room_id`) + the `[[waits]]` happens-after edge (Bob's join
//! follows Alice's invite) → membership-convergence oracle → capture — with
//! **real protocol convergence** (the M8.5-B INV bootstrap + membership
//! resolution on node A), not a harness-induced relay.
//!
//! The TRUE cross-node form (alice@A, bob@B, federated) is gated on a fresh-peer
//! `federation initiate` control surface that does not exist yet (the external
//! surfaces federate only KNOWN peers, FED_3006) — a flagged Multiparty-tests
//! prerequisite, recorded in the manifest + the matrix.
//!
//! ```text
//! cargo build -p xgen-node -p xgen-client
//! cargo test -p xgen-mptest --test c5_mp_c_02 -- --ignored --nocapture
//! ```

use std::path::Path;
use std::time::Duration;

use xgen_mptest::aicontrol::{AicontrolClient, DEFAULT_CONNECT_TIMEOUT};
use xgen_mptest::batch::{parse_batch_lines, run_actor, ActorRun};
use xgen_mptest::binloc;
use xgen_mptest::capture::Capture;
use xgen_mptest::events::{EventCollector, Filter};
use xgen_mptest::manifest::Scenario;
use xgen_mptest::oracle::{convergence_verdict, MembershipProjection, Transcript};
use xgen_mptest::process::{events_pipe, instance_label, Kind, ManagedProcess};
use xgen_mptest::resolve::Registry;
use xgen_mptest::wire::Command;

const RESOLVE_TIMEOUT: Duration = Duration::from_secs(45);

#[tokio::test]
#[ignore = "heavy: spawns real xgen-node + 2 xgen-client residents; run with --ignored"]
async fn mp_c_02_invite_and_join_converges_single_node() {
    let bins = binloc::locate().expect("locate built binaries");

    // ── Load the scenario (manifest + batches) ───────────────────────────────
    let scenario_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("docs/tests/multiparty_scenarios/MP-C-02");
    let scenario = Scenario::load(&scenario_dir).expect("load MP-C-02 scenario");
    let m = &scenario.manifest;
    assert_eq!(m.nodes.len(), 1, "Round-0 MP-C-02 is single-node");
    let node_spec = &m.nodes[0];
    let node_url = format!("ws://127.0.0.1:{}/xgen", node_spec.port);

    // ── Spawn node + observer ────────────────────────────────────────────────
    let node_label = instance_label("MP-C-02", "node");
    let node = ManagedProcess::init_and_spawn_node(&bins, &node_label, node_spec.port, node_spec.local)
        .expect("spawn node");
    let collector = EventCollector::start("a", &events_pipe(Kind::Node, &node_label), Filter::all())
        .await
        .expect("attach collector");

    // ── Spawn both clients (single-node: both → node A) ──────────────────────
    let alice_spec = m.actor("alice").expect("alice in manifest");
    let bob_spec = m.actor("bob").expect("bob in manifest");
    let alice_label = instance_label("MP-C-02", "alice");
    let bob_label = instance_label("MP-C-02", "bob");
    let _alice_proc = ManagedProcess::init_and_spawn_client(&bins, &alice_label, &node_url, false)
        .expect("spawn alice client");
    let _bob_proc = ManagedProcess::init_and_spawn_client(&bins, &bob_label, &node_url, false)
        .expect("spawn bob client");

    let alice_pipe = xgen_mptest::process::aicontrol_pipe(Kind::Client, &alice_label);
    let bob_pipe = xgen_mptest::process::aicontrol_pipe(Kind::Client, &bob_label);
    let mut alice_ctl = AicontrolClient::connect(&alice_pipe, DEFAULT_CONNECT_TIMEOUT)
        .await
        .expect("connect alice aicontrol");
    let mut bob_ctl = AicontrolClient::connect(&bob_pipe, DEFAULT_CONNECT_TIMEOUT)
        .await
        .expect("connect bob aicontrol");

    // ── Drive both batches concurrently against the shared registry ──────────
    let alice_lines = parse_batch_lines(
        &std::fs::read_to_string(scenario.batch_path(alice_spec)).unwrap(),
    )
    .unwrap();
    let bob_lines =
        parse_batch_lines(&std::fs::read_to_string(scenario.batch_path(bob_spec)).unwrap()).unwrap();

    let registry = Registry::new();
    let (alice_res, bob_res) = tokio::join!(
        run_actor("alice", &alice_lines, &mut alice_ctl, m, &registry, RESOLVE_TIMEOUT),
        run_actor("bob", &bob_lines, &mut bob_ctl, m, &registry, RESOLVE_TIMEOUT),
    );
    let alice_run: ActorRun = alice_res.expect("alice batch ran");
    let bob_run: ActorRun = bob_res.expect("bob batch ran");
    assert!(alice_run.all_ok(), "alice had a failed command: {:?}", alice_run.replies);
    assert!(bob_run.all_ok(), "bob had a failed command: {:?}", bob_run.replies);

    // ── Quiesce, then read the membership projection from each client ────────
    tokio::time::sleep(Duration::from_millis(1200)).await;
    let space_id = registry.get("space_id").await.expect("space_id was exported");

    let mut members_cmd = Command::new("members");
    members_cmd.args.insert("space".into(), serde_json::json!(space_id));
    let alice_members = alice_ctl.send(&members_cmd).await.expect("alice members");
    let bob_members = bob_ctl.send(&members_cmd).await.expect("bob members");
    assert!(alice_members.is_ok(), "alice members failed: {alice_members:?}");
    assert!(bob_members.is_ok(), "bob members failed: {bob_members:?}");

    let alice_proj = MembershipProjection::from_members_data("alice-view", alice_members.data().unwrap())
        .expect("alice projection");
    let bob_proj = MembershipProjection::from_members_data("bob-view", bob_members.data().unwrap())
        .expect("bob projection");

    // ── Oracle: membership converges; Bob is a member on both views ──────────
    let transcript = Transcript::from_values("a", &collector.snapshot().await);
    let verdict = convergence_verdict(
        &[alice_proj.clone(), bob_proj.clone()],
        std::slice::from_ref(&transcript),
        &space_id,
    );
    eprintln!("MP-C-02 oracle: {}", verdict.detail);
    assert!(verdict.pass, "MP-C-02 convergence FAILED: {}", verdict.detail);

    let bob_id = registry.get("bob_identity_id").await.expect("bob_identity_id exported");
    assert!(
        alice_proj.members.contains_key(&bob_id),
        "Bob not a member in alice's view: {:?}",
        alice_proj.members
    );
    assert!(
        bob_proj.members.contains_key(&bob_id),
        "Bob not a member in bob's view: {:?}",
        bob_proj.members
    );
    eprintln!("MP-C-02 PASS: Bob is a member on both views; membership converges");

    // ── Capture ──────────────────────────────────────────────────────────────
    let tmp = tempfile::tempdir().unwrap();
    let cap = Capture::new(tmp.path(), "MP-C-02").unwrap();
    cap.write_resolved_manifest(&std::fs::read_to_string(scenario_dir.join("manifest.toml")).unwrap())
        .unwrap();
    cap.write_actor_log(&alice_run).unwrap();
    cap.write_actor_log(&bob_run).unwrap();
    cap.write_transcript("a", &collector.snapshot().await).unwrap();
    cap.write_verdict(&verdict).unwrap();

    drop(alice_ctl);
    drop(bob_ctl);
    drop(collector);
    drop(_alice_proc);
    drop(_bob_proc);
    drop(node);
    tokio::time::sleep(Duration::from_millis(50)).await;
}
