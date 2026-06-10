// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! C2 oracle + capture live smoke (M9, `#[ignore]` — spawns real binaries).
//!
//! Validates the C2 stack against the real binaries end-to-end:
//! node + client spawn → attach an `.events` [`EventCollector`] *before* driving
//! → drive the client over `.aicontrol` (`register` → `create-space`) → read the
//! membership projection via `members` → confirm the collector captured the
//! create event for the Space → build oracle types from the real data → write
//! the capture artifacts. The full two-node MP-C-02 convergence run is C5.
//!
//! ```text
//! cargo build -p xgen-node -p xgen-client
//! cargo test -p xgen-mptest --test c2_oracle -- --ignored --nocapture
//! ```

use std::time::Duration;

use xgen_mptest::aicontrol::{AicontrolClient, DEFAULT_CONNECT_TIMEOUT};
use xgen_mptest::binloc;
use xgen_mptest::capture::Capture;
use xgen_mptest::events::{EventCollector, Filter};
use xgen_mptest::oracle::{MembershipProjection, Transcript};
use xgen_mptest::process::{events_pipe, instance_label, Kind, ManagedProcess};
use xgen_mptest::wire::Command;

#[tokio::test]
#[ignore = "heavy: spawns real xgen-node + xgen-client; run with --ignored"]
async fn c2_collector_and_members_projection_from_real_node() {
    let bins = binloc::locate().expect("locate built binaries");

    // 1. Spawn the node.
    let node_label = instance_label("C2", "node");
    let node = ManagedProcess::init_and_spawn_node(&bins, &node_label, 8456, true, None)
        .expect("spawn node");

    // 2. Attach the .events collector BEFORE anything is driven (live-only).
    let collector = EventCollector::start(
        "node",
        &events_pipe(Kind::Node, &node_label),
        Filter::all(),
    )
    .await
    .expect("attach events collector");

    // 3. Spawn the client resident pointed at the node.
    let node_url = "ws://127.0.0.1:8456/xgen";
    let client_label = instance_label("C2", "alice");
    let client = ManagedProcess::init_and_spawn_client(&bins, &client_label, node_url, false, None)
        .expect("spawn client");

    // 4. Drive the client over its .aicontrol pipe.
    let mut ctl = AicontrolClient::connect(&client.aicontrol_pipe, DEFAULT_CONNECT_TIMEOUT)
        .await
        .expect("connect client aicontrol");

    let mut reg = Command::new("register");
    reg.args.insert("name".into(), serde_json::json!("alice"));
    let reg_reply = ctl.send(&reg).await.expect("register");
    assert!(reg_reply.is_ok(), "register failed: {reg_reply:?}");
    let alice_id = reg_reply
        .data_str("identity_id")
        .expect("register → identity_id")
        .to_string();

    let mut cs = Command::new("create-space");
    cs.args.insert("name".into(), serde_json::json!("S"));
    let cs_reply = ctl.send(&cs).await.expect("create-space");
    assert!(cs_reply.is_ok(), "create-space failed: {cs_reply:?}");
    let space_id = cs_reply
        .data_str("space_id")
        .expect("create-space → space_id")
        .to_string();

    // 5. Give the node a beat to fan the create event out to the observer.
    tokio::time::sleep(Duration::from_millis(800)).await;

    // 6. Membership projection via `members` (the RoomState-Eq analogue source).
    let mut mem = Command::new("members");
    mem.args.insert("space".into(), serde_json::json!(space_id));
    let mem_reply = ctl.send(&mem).await.expect("members");
    assert!(mem_reply.is_ok(), "members failed: {mem_reply:?}");
    let proj = MembershipProjection::from_members_data("node", mem_reply.data().unwrap())
        .expect("members → projection");
    assert_eq!(proj.owner_id, alice_id, "owner should be alice");
    assert!(
        proj.members.contains_key(&alice_id),
        "alice should be a member: {:?}",
        proj.members
    );

    // 7. The collector should have captured ≥1 event for the Space (the
    //    create event reached the node observer via apply_fanout).
    let events = collector.snapshot().await;
    let transcript = Transcript::from_values("node", &events);
    let space_events = transcript.event_ids_for_space(&space_id);
    assert!(
        !transcript.events.is_empty(),
        "collector captured no events at all"
    );
    eprintln!(
        "collector captured {} events; {} for space {space_id}",
        transcript.events.len(),
        space_events.len()
    );
    // The create event must attribute to its own Space (effective_space_id:
    // empty space_id field, event_id == space_id).
    assert!(
        !space_events.is_empty(),
        "collector captured no events for space {space_id} (effective-space-id attribution?): {:?}",
        transcript.events
    );

    // 8. Capture artifacts to a temp dir.
    let tmp = tempfile::tempdir().expect("tempdir");
    let cap = Capture::new(tmp.path(), "C2-smoke").expect("capture");
    cap.write_transcript("node", &events).expect("write transcript");
    assert!(cap.dir().join("node.events.jsonl").exists());

    drop(ctl);
    drop(collector);
    drop(client);
    drop(node);
    tokio::time::sleep(Duration::from_millis(50)).await;
}
