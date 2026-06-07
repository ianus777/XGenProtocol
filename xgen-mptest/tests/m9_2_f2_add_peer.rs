// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! M9.2 F2 smoke — FENCED `federation add-peer` full bootstrap (`#[ignore]`).
//!
//! The cross-node bootstrap that was impossible before M9.2 (the external
//! surfaces federate only KNOWN peers — FED_3006). Two fresh harness-control
//! nodes A/B; `add-peer` each direction naming the shared Space (M9.2-D2′:
//! `add-peer` upserts a `FederationRelationship` the existing `federation
//! initiate` consumes) → `federation initiate` from A → the Space replicates
//! A→B (observed as B's hosted-space count rising from 0 to ≥1).
//!
//! **REQUIRES a harness-control node build** (the verb is fenced, M9.2-D1):
//! ```text
//! cargo build -p xgen-node -p xgen-client && cargo build -p xgen-node --features harness-control
//! cargo test -p xgen-mptest --test m9_2_f2_add_peer -- --ignored --nocapture
//! ```
//! (Build the default client + the harness-control node; binloc resolves both
//! from the same target dir, the node overwritten with the fenced build.)

use std::time::Duration;

use xgen_mptest::aicontrol::{AicontrolClient, DEFAULT_CONNECT_TIMEOUT};
use xgen_mptest::binloc;
use xgen_mptest::process::{aicontrol_pipe, instance_label, Kind, ManagedProcess};
use xgen_mptest::wire::{Command, Reply};

const PORT_A: u16 = 8480;
const PORT_B: u16 = 8481;

#[tokio::test]
#[ignore = "heavy: spawns two real harness-control xgen-node binaries + a client; run with --ignored"]
async fn f2_add_peer_then_initiate_replicates_space_a_to_b() {
    let bins = binloc::locate().expect("locate built binaries");

    // ── Spawn two fresh nodes A/B ────────────────────────────────────────────
    let a_label = instance_label("M9-2-F2", "nodeA");
    let b_label = instance_label("M9-2-F2", "nodeB");
    let node_a = ManagedProcess::init_and_spawn_node(&bins, &a_label, PORT_A, true)
        .expect("spawn node A");
    let node_b = ManagedProcess::init_and_spawn_node(&bins, &b_label, PORT_B, true)
        .expect("spawn node B");
    let a_url = format!("ws://127.0.0.1:{PORT_A}/xgen");
    let b_url = format!("ws://127.0.0.1:{PORT_B}/xgen");

    let mut a_ctl = AicontrolClient::connect(&aicontrol_pipe(Kind::Node, &a_label), DEFAULT_CONNECT_TIMEOUT)
        .await
        .expect("connect node A aicontrol");
    let mut b_ctl = AicontrolClient::connect(&aicontrol_pipe(Kind::Node, &b_label), DEFAULT_CONNECT_TIMEOUT)
        .await
        .expect("connect node B aicontrol");

    // ── Identities + B's baseline space count ────────────────────────────────
    // NB: the binary's default config points every instance's spaces_dir at a
    // SHARED `<exe_dir>/spaces` (NodeConfig::default → exe_dir, pre-existing
    // binary behaviour, out of M9.2 scope), so B may already host spaces
    // accumulated by prior runs. The shared Space here is a BRAND-NEW unique id
    // (fresh create), and `add-peer` names only it, so the federation delta to B
    // is exactly that Space — a count *increase* unambiguously detects it
    // replicating (robust to accumulated baseline; mirrors MP-C-02 asserting on
    // its own Space, not absolute counts).
    let a_state = a_ctl.send(&Command::new("state")).await.expect("A state");
    let b_state = b_ctl.send(&Command::new("state")).await.expect("B state");
    let a_id = a_state.data_str("node_id").expect("A node_id").to_string();
    let b_id = b_state.data_str("node_id").expect("B node_id").to_string();
    let b_before = hosted_spaces(&b_state);

    // ── Seed the peer EARLY so identity-replication targets B ────────────────
    // Bootstrap ordering matters: when an identity registers on A, A pushes it
    // to every peer in `peer_urls` (app.rs `push_identity_to_peers`). `add-peer`
    // calls `record_peer_url`, so seeding B *before* alice registers is what lets
    // B learn alice's identity — without which B buffers alice-signed events as
    // an unknown signer (F-10) and the Space never materialises. (Spaces named
    // empty here; updated to [S] once S exists.)
    assert_ok(&add_peer(&mut a_ctl, &b_id, &b_url, &[]).await, "A add-peer B (seed)");
    assert_ok(&add_peer(&mut b_ctl, &a_id, &a_url, &[]).await, "B add-peer A (seed)");

    // ── alice (client on A) registers (→ identity replicates to B) ───────────
    let alice_label = instance_label("M9-2-F2", "alice");
    let _alice = ManagedProcess::init_and_spawn_client(&bins, &alice_label, &a_url, false)
        .expect("spawn alice client");
    let mut alice = AicontrolClient::connect(&aicontrol_pipe(Kind::Client, &alice_label), DEFAULT_CONNECT_TIMEOUT)
        .await
        .expect("connect alice aicontrol");

    let mut reg = Command::new("register");
    reg.args.insert("name".into(), serde_json::json!("alice"));
    assert_ok(&alice.send(&reg).await.expect("alice register"), "register");
    // Let A's async identity-replication push reach B.
    tokio::time::sleep(Duration::from_millis(1500)).await;

    // ── alice creates the Space to share ─────────────────────────────────────
    let mut cs = Command::new("create-space");
    cs.args.insert("name".into(), serde_json::json!("F2-SPACE"));
    let cs_reply = alice.send(&cs).await.expect("alice create-space");
    assert_ok(&cs_reply, "create-space");
    let space_id = cs_reply.data_str("space_id").expect("space_id").to_string();
    eprintln!("F2: alice created Space {space_id} on node A ({a_id})");
    tokio::time::sleep(Duration::from_millis(500)).await;

    // ── Re-seed A's relationship with the shared Space, then initiate ─────────
    assert_ok(&add_peer(&mut a_ctl, &b_id, &b_url, &[space_id.as_str()]).await, "A add-peer B (with space)");

    // ── A initiates federation to B → replicate the shared Space ─────────────
    let mut init = Command::new("federation initiate");
    init.args.insert("peer_node_id".into(), serde_json::json!(b_id));
    let init_reply = a_ctl.send(&init).await.expect("federation initiate");
    assert_ok(&init_reply, "federation initiate");
    eprintln!("F2: federation initiate A→B dispatched");

    // ── Poll B until its hosted-space count rises (the new Space replicated) ──
    let mut b_after = b_before;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    while tokio::time::Instant::now() < deadline {
        let s = b_ctl.send(&Command::new("state")).await.expect("B state poll");
        b_after = hosted_spaces(&s);
        if b_after > b_before {
            break;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    assert!(
        b_after > b_before,
        "Space {space_id} did NOT replicate A→B within 30s (B hosted {b_before}, still {b_after}) — \
         the cross-node bootstrap failed"
    );
    eprintln!(
        "F2 PASS: add-peer ×2 → initiate → Space {space_id} replicated A→B \
         (B hosted {b_before} → {b_after})"
    );

    drop(alice);
    drop(a_ctl);
    drop(b_ctl);
    drop(_alice);
    drop(node_a);
    drop(node_b);
    tokio::time::sleep(Duration::from_millis(50)).await;
}

async fn add_peer(ctl: &mut AicontrolClient, peer_id: &str, peer_url: &str, spaces: &[&str]) -> Reply {
    let mut c = Command::new("federation add-peer");
    c.args.insert("node_id".into(), serde_json::json!(peer_id));
    c.args.insert("url".into(), serde_json::json!(peer_url));
    c.args.insert("spaces".into(), serde_json::json!(spaces));
    ctl.send(&c).await.expect("federation add-peer")
}

fn hosted_spaces(state: &Reply) -> u64 {
    state
        .data()
        .and_then(|d| d.get("hosted_spaces"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0)
}

fn assert_ok(reply: &Reply, what: &str) {
    assert!(
        reply.is_ok(),
        "{what} failed — did you build the node --features harness-control? reply={reply:?}"
    );
}
