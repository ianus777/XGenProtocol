// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! C4 raw-wire injector live test (M9, `#[ignore]` — spawns a real node).
//!
//! Confirms the forged-signature attack against a real `xgen-node`: the injector
//! connects over the real `ws://` transport, authenticates, and submits an event
//! whose signature does not match its claimed `sender`. PASS = the forged event
//! is **absent** from the node's `.events` (not applied) AND the injector got
//! past WS/auth (so the absence is a *validation* rejection, not an auth bounce).
//!
//! Scope note (Checkpoint #3): this isolates "reached validation + not applied"
//! against a **synthetic** target. The step-12 *signature* isolation (claimed
//! sender is a real member, prev known, so steps 9–11 pass and only the sig is
//! wrong) lands in the C5 MP-A-05 scenario against Alice's real Space.
//!
//! ```text
//! cargo build -p xgen-node
//! cargo test -p xgen-mptest --test c4_injector -- --ignored --nocapture
//! ```

use std::time::Duration;

use xgen_mptest::binloc;
use xgen_mptest::events::{EventCollector, Filter};
use xgen_mptest::injector::{build_forged_signature_event, fresh_key, inject_event};
use xgen_mptest::oracle::{rejection_verdict, Transcript};
use xgen_mptest::process::{events_pipe, instance_label, Kind, ManagedProcess};

#[tokio::test]
#[ignore = "heavy: spawns the real xgen-node binary; run with --ignored"]
async fn c4_forged_signature_is_not_applied() {
    let bins = binloc::locate().expect("locate built binaries");

    let node_label = instance_label("C4", "node");
    let node = ManagedProcess::init_and_spawn_node(&bins, &node_label, 8460, true, None)
        .expect("spawn node");

    let collector = EventCollector::start(
        "node",
        &events_pipe(Kind::Node, &node_label),
        Filter::all(),
    )
    .await
    .expect("attach events collector");

    // Forge: claim `victim`, sign with `attacker` (≠ victim).
    let victim = fresh_key();
    let attacker = fresh_key();
    let forged = build_forged_signature_event(
        &victim,
        &attacker,
        "xgen://hash/sha256:SYNTH_SPACE",
        "xgen://hash/sha256:SYNTH_ROOM",
    );
    let forged_id = forged
        .event_id
        .as_ref()
        .map(|e| e.as_str().to_string())
        .expect("forged event has an id");

    let node_url = "ws://127.0.0.1:8460/xgen";
    let result = inject_event(node_url, &attacker, &forged)
        .await
        .expect("inject forged event");

    // Reached the session (got past WS/auth) — so a later absence is a
    // validation rejection, not an auth bounce.
    assert!(
        result.authed_as.is_some(),
        "injector failed to authenticate — cannot conclude validation rejection"
    );
    eprintln!(
        "injector authed_as={:?}; node Error reply={:?}",
        result.authed_as, result.error_reply
    );

    // Let the node process (and, if it were going to, fan out) the event.
    tokio::time::sleep(Duration::from_millis(800)).await;

    let transcript = Transcript::from_values("node", &collector.snapshot().await);
    let verdict = rejection_verdict(&[transcript], &forged_id);
    assert!(verdict.pass, "forged event was APPLIED (finding!): {}", verdict.detail);
    eprintln!("REJECTION OK: {}", verdict.detail);

    drop(collector);
    drop(node);
    tokio::time::sleep(Duration::from_millis(50)).await;
}
