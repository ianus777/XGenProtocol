// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Round-0 adversarial smoke — **MP-A-05** (signature / identity forgery, F-F).
//!
//! Single-node (the forgery is rejected at every node; for Round-0 that is node
//! A). The harness wire-driver registers a real member (Alice) and builds her
//! Space + Room, then the injector submits an event **claiming Alice** but
//! **signed by the attacker**. Because Alice really is a member with a known
//! predecessor, validation steps 9–11 pass and the forgery is isolated to
//! **step 12** (`verify_event_signature`). A correctly-signed control message
//! from Alice is accepted in the same Space — proving only the forged signature
//! is the cause.
//!
//! Oracle (M9-D4 rejection leg): the forged `event_id` is **absent** from the
//! node's `.events`; the control message **is present**.
//!
//! ```text
//! cargo build -p xgen-node
//! cargo test -p xgen-mptest --test c5_mp_a_05 -- --ignored --nocapture
//! ```

use std::time::Duration;

use ed25519_dalek::SigningKey;
use xgen_common::wire::{Event, EventType};
use xgen_common::xgid::{EventXgid, IdentityXgid, RoomXgid, SpaceXgid, Xgid};
use xgen_core::space::state::sign_event;

use xgen_mptest::binloc;
use xgen_mptest::capture::Capture;
use xgen_mptest::events::{EventCollector, Filter};
use xgen_mptest::injector::{fresh_key, inject_event};
use xgen_mptest::oracle::{rejection_verdict, Transcript};
use xgen_mptest::process::{events_pipe, instance_label, Kind, ManagedProcess};
use xgen_mptest::wireactor::WireActor;

fn now_rfc3339() -> String {
    use chrono::SecondsFormat;
    chrono::Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

/// A correctly-signed message from `sender_key` into (space, room) on top of `prev`.
fn legit_message(sender_key: &SigningKey, space: &str, room: &str, prev: &str, text: &str) -> Event {
    let sender = IdentityXgid::from_pubkey(&sender_key.verifying_key());
    sign_event(
        Event::new(
            EventType::MessageText,
            sender,
            RoomXgid::from_xgid(Xgid::new(room.to_string())),
            SpaceXgid::from_xgid(Xgid::new(space.to_string())),
            vec![EventXgid::from_xgid(Xgid::new(prev.to_string()))],
            now_rfc3339(),
            serde_json::json!({ "text": text }),
        ),
        sender_key,
    )
}

#[tokio::test]
#[ignore = "heavy: spawns the real xgen-node binary; run with --ignored"]
async fn mp_a_05_forged_signature_rejected_at_step_12() {
    let bins = binloc::locate().expect("locate built binaries");

    // ── Node + observer ────────────────────────────────────────────────────
    let node_label = instance_label("MP-A-05", "node");
    let node = ManagedProcess::init_and_spawn_node(&bins, &node_label, 8470, true, None)
        .expect("spawn node");
    let collector = EventCollector::start("A", &events_pipe(Kind::Node, &node_label), Filter::all())
        .await
        .expect("attach collector");

    let node_url = "ws://127.0.0.1:8470/xgen";

    // ── Cooperative setup: Alice (harness-keyed) builds a real Space + Room ──
    let alice_key = fresh_key();
    let mut alice = WireActor::connect_with_key(node_url, alice_key.clone())
        .await
        .expect("alice connect");
    alice.register("alice").await.expect("alice register");
    let space_id = alice.create_space("S").await.expect("create space");
    let room_id = alice.create_room(&space_id, "general").await.expect("create room");

    // Control: a correctly-signed message from Alice — must be ACCEPTED.
    let control = legit_message(&alice_key, &space_id, &room_id, &room_id, "honest hello");
    let control_id = control.event_id.as_ref().unwrap().as_str().to_string();
    alice.submit(&control).await.expect("submit control message");

    // ── The attack: an event claiming Alice, signed by the attacker ─────────
    let attacker_key = fresh_key();
    let alice_sender = IdentityXgid::from_pubkey(&alice_key.verifying_key());
    let forged = sign_event(
        Event::new(
            EventType::MessageText,
            alice_sender, // claims Alice (a real member)
            RoomXgid::from_xgid(Xgid::new(room_id.clone())),
            SpaceXgid::from_xgid(Xgid::new(space_id.clone())),
            vec![EventXgid::from_xgid(Xgid::new(room_id.clone()))], // known predecessor
            now_rfc3339(),
            serde_json::json!({ "text": "forged as alice (MP-A-05)" }),
        ),
        &attacker_key, // signed by the WRONG key → step 12 fails
    );
    let forged_id = forged.event_id.as_ref().unwrap().as_str().to_string();

    let result = inject_event(node_url, &attacker_key, &forged)
        .await
        .expect("inject forged event");
    assert!(
        result.authed_as.is_some(),
        "injector failed to authenticate — cannot conclude validation rejection"
    );
    eprintln!(
        "MP-A-05: injector authed={:?}; node Error reply={:?}",
        result.authed_as, result.error_reply
    );

    // Let the node process + fan out.
    tokio::time::sleep(Duration::from_millis(1000)).await;

    // ── Oracle ──────────────────────────────────────────────────────────────
    let events = collector.snapshot().await;
    let transcript = Transcript::from_values("A", &events);

    // Rejection leg: forged event absent everywhere.
    let verdict = rejection_verdict(std::slice::from_ref(&transcript), &forged_id);
    assert!(verdict.pass, "MP-A-05 FAILED (forgery applied!): {}", verdict.detail);

    // Control leg: the correctly-signed message WAS applied — proves steps 9–11
    // pass for Alice and that only the forged signature is rejected (step 12).
    assert!(
        transcript.contains_event(&control_id),
        "control message {control_id} was not applied — setup invalid, cannot isolate step 12"
    );

    eprintln!("MP-A-05 PASS: {} | control message applied", verdict.detail);

    // Capture.
    let tmp = tempfile::tempdir().unwrap();
    let cap = Capture::new(tmp.path(), "MP-A-05").unwrap();
    cap.write_transcript("A", &events).unwrap();
    cap.write_verdict(&verdict).unwrap();

    drop(alice);
    drop(collector);
    drop(node);
    tokio::time::sleep(Duration::from_millis(50)).await;
}
