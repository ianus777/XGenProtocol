// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! MP-R2 C6a — tranche (b) topology/static scenarios (`#[ignore]`).
//!
//! The fixed-N rows that ride the **existing** `run_scenario` over a multi-node
//! `[[federation]]` topology — no new mechanism (the restart/migration/injector/
//! E2E rows are C6b–C6e). Each writes its scenario **inline to a tempdir** (the
//! `mp_r1_runner.rs` precedent for box-gated smokes — promotable to a committed
//! `multiparty_scenarios/<ID>/` dir once the RUN validates it), then asserts a
//! transcript-level delivery/anti-delivery property.
//!
//! ## Box-gated (RUN gate, M-R2.3) + boundary
//! Heavy — spawns 3–4 real `--features harness-control` nodes (federated ⇒ Mock
//! clock + harness-control). The topology semantics (transitive delivery,
//! anti-transitivity) are **validated at the first box-gated RUN**; these assert
//! content-delivery (alice's Space reaching / not reaching a leaf), not full
//! membership-on-all-nodes (a RUN enrichment). Rule 2: spawn timeout = flake.
//!
//! ```text
//! cargo build -p xgen-node --features harness-control && cargo build -p xgen-client
//! cargo test -p xgen-mptest --test mp_r2_fixed -- --ignored --nocapture
//! ```

use std::path::Path;

use xgen_mptest::dial::{ClockMode, RoundDial};
use xgen_mptest::manifest::Scenario;
use xgen_mptest::runner::run_scenario;

fn mock_dial() -> RoundDial {
    RoundDial {
        clock: ClockMode::Mock,
        ..Default::default()
    }
}

// ── MP-C-04 — 3-node transitive path: A's Space reaches BOTH leaves ──────────

#[tokio::test]
#[ignore = "heavy: spawns 3 real harness-control nodes + clients; box-gated RUN"]
async fn mp_c_04_three_node_transitive_converges() {
    let tmp = tempfile::tempdir().expect("tempdir");
    write_mp_c_04(tmp.path());
    let scenario = Scenario::load(tmp.path()).expect("load MP-C-04");
    let outcome = run_scenario(&scenario, &mock_dial())
        .await
        .expect("run_scenario(MP-C-04) — node built --features harness-control?");

    let space = outcome.space_id.clone().expect("alice exported a space_id");
    for leaf in ["b", "c"] {
        let t = outcome
            .transcripts
            .iter()
            .find(|t| t.node == leaf)
            .unwrap_or_else(|| panic!("node {leaf} transcript"));
        assert!(
            !t.event_ids_for_space(&space).is_empty(),
            "Space {space} did NOT reach leaf {leaf} (transitive delivery failed): {:?}",
            t.events
        );
    }
    eprintln!("C6a MP-C-04 PASS: alice's Space {space} reached both leaves B + C");
}

/// A central, B + C leaves (star: a→b, a→c). alice builds + posts; b/c register.
fn write_mp_c_04(dir: &Path) {
    let manifest = r#"
scenario = "MP-C-04"
description = "3-node transitive path: A's Space reaches both leaves"

[[nodes]]
label = "a"
port = 8510
[[nodes]]
label = "b"
port = 8511
[[nodes]]
label = "c"
port = 8512

[[federation]]
from = "a"
to = "b"
[[federation]]
from = "a"
to = "c"

[[actors]]
name = "alice"
node = "a"
batch = "alice.jsonl"
[[actors]]
name = "bob"
node = "b"
batch = "bob.jsonl"
[[actors]]
name = "carol"
node = "c"
batch = "carol.jsonl"

[[exports]]
actor = "alice"
command = "a2"
field = "space_id"
key = "space_id"
"#;
    let alice = "\
{\"cmd\":\"register\",\"args\":{\"name\":\"alice\"},\"id\":\"a1\"}\n\
{\"cmd\":\"create-space\",\"args\":{\"name\":\"MP-C-04\"},\"id\":\"a2\",\"bind\":\"s\"}\n\
{\"cmd\":\"create-room\",\"args\":{\"space\":\"$s\",\"name\":\"general\"},\"id\":\"a3\",\"bind\":\"r\"}\n\
{\"cmd\":\"send\",\"args\":{\"space\":\"$s\",\"room\":\"$r\",\"text\":\"hello all\"},\"id\":\"a4\"}\n";
    write_scenario(dir, manifest, &[("alice.jsonl", alice), ("bob.jsonl", REGISTER_BOB), ("carol.jsonl", REGISTER_CAROL)]);
}

// ── MP-C-14 — 4-node star: A's Space reaches every leaf ──────────────────────

#[tokio::test]
#[ignore = "heavy: spawns 4 real harness-control nodes + clients; box-gated RUN"]
async fn mp_c_14_star_topology_converges() {
    let tmp = tempfile::tempdir().expect("tempdir");
    write_mp_c_14(tmp.path());
    let scenario = Scenario::load(tmp.path()).expect("load MP-C-14");
    let outcome = run_scenario(&scenario, &mock_dial())
        .await
        .expect("run_scenario(MP-C-14) — node built --features harness-control?");

    let space = outcome.space_id.clone().expect("alice exported a space_id");
    for leaf in ["b", "c", "d"] {
        let t = outcome
            .transcripts
            .iter()
            .find(|t| t.node == leaf)
            .unwrap_or_else(|| panic!("node {leaf} transcript"));
        assert!(
            !t.event_ids_for_space(&space).is_empty(),
            "Space {space} did NOT reach star leaf {leaf}: {:?}",
            t.events
        );
    }
    eprintln!("C6a MP-C-14 PASS: alice's Space {space} reached all 3 star leaves");
}

/// n0 central + 3 leaves (star: a→b, a→c, a→d). Mesh cross-links are a RUN
/// enrichment (kept a clean star here). alice builds + posts; leaves register.
fn write_mp_c_14(dir: &Path) {
    let manifest = r#"
scenario = "MP-C-14"
description = "4-node star: A's Space reaches every leaf"

[[nodes]]
label = "a"
port = 8513
[[nodes]]
label = "b"
port = 8514
[[nodes]]
label = "c"
port = 8515
[[nodes]]
label = "d"
port = 8516

[[federation]]
from = "a"
to = "b"
[[federation]]
from = "a"
to = "c"
[[federation]]
from = "a"
to = "d"

[[actors]]
name = "alice"
node = "a"
batch = "alice.jsonl"
[[actors]]
name = "bob"
node = "b"
batch = "bob.jsonl"
[[actors]]
name = "carol"
node = "c"
batch = "carol.jsonl"
[[actors]]
name = "dave"
node = "d"
batch = "dave.jsonl"

[[exports]]
actor = "alice"
command = "a2"
field = "space_id"
key = "space_id"
"#;
    let alice = "\
{\"cmd\":\"register\",\"args\":{\"name\":\"alice\"},\"id\":\"a1\"}\n\
{\"cmd\":\"create-space\",\"args\":{\"name\":\"MP-C-14\"},\"id\":\"a2\",\"bind\":\"s\"}\n\
{\"cmd\":\"create-room\",\"args\":{\"space\":\"$s\",\"name\":\"general\"},\"id\":\"a3\",\"bind\":\"r\"}\n\
{\"cmd\":\"send\",\"args\":{\"space\":\"$s\",\"room\":\"$r\",\"text\":\"star broadcast\"},\"id\":\"a4\"}\n";
    write_scenario(
        dir,
        manifest,
        &[
            ("alice.jsonl", alice),
            ("bob.jsonl", REGISTER_BOB),
            ("carol.jsonl", REGISTER_CAROL),
            ("dave.jsonl", "{\"cmd\":\"register\",\"args\":{\"name\":\"dave\"},\"id\":\"d1\"}\n"),
        ],
    );
}

// ── MP-A-13 — anti-transitivity: B does NOT re-forward A's event to C ─────────

#[tokio::test]
#[ignore = "heavy: spawns 3 real harness-control nodes + clients; box-gated RUN"]
async fn mp_a_13_anti_transitivity_c_does_not_receive_via_b() {
    let tmp = tempfile::tempdir().expect("tempdir");
    write_mp_a_13(tmp.path());
    let scenario = Scenario::load(tmp.path()).expect("load MP-A-13");
    let outcome = run_scenario(&scenario, &mock_dial())
        .await
        .expect("run_scenario(MP-A-13) — node built --features harness-control?");

    let space = outcome.space_id.clone().expect("alice exported a space_id");
    // A↔B federated ⇒ B has the Space. B↔C federated, A↔C NOT ⇒ the F-5/D-089
    // pairwise model means B does NOT re-forward A's Space to C.
    let tb = outcome.transcripts.iter().find(|t| t.node == "b").expect("b transcript");
    assert!(
        !tb.event_ids_for_space(&space).is_empty(),
        "precondition failed: A's Space did not even reach the directly-federated B"
    );
    let tc = outcome.transcripts.iter().find(|t| t.node == "c").expect("c transcript");
    assert!(
        tc.event_ids_for_space(&space).is_empty(),
        "anti-transitivity VIOLATED: C received A's Space {space} via B's relay (should be pairwise-only): {:?}",
        tc.events
    );
    eprintln!("C6a MP-A-13 PASS: B did not re-forward A's Space to C (pairwise, not transitive)");
}

/// A↔B and B↔C federated; A↔C NOT. alice@A builds + posts; the property is that
/// C (federated only with B) does not receive A's Space via a B relay.
fn write_mp_a_13(dir: &Path) {
    let manifest = r#"
scenario = "MP-A-13"
description = "anti-transitivity: B does not re-forward A's event to C"

[[nodes]]
label = "a"
port = 8517
[[nodes]]
label = "b"
port = 8518
[[nodes]]
label = "c"
port = 8519

[[federation]]
from = "a"
to = "b"
[[federation]]
from = "b"
to = "c"

[[actors]]
name = "alice"
node = "a"
batch = "alice.jsonl"
[[actors]]
name = "bob"
node = "b"
batch = "bob.jsonl"
[[actors]]
name = "carol"
node = "c"
batch = "carol.jsonl"

[[exports]]
actor = "alice"
command = "a2"
field = "space_id"
key = "space_id"
"#;
    let alice = "\
{\"cmd\":\"register\",\"args\":{\"name\":\"alice\"},\"id\":\"a1\"}\n\
{\"cmd\":\"create-space\",\"args\":{\"name\":\"MP-A-13\"},\"id\":\"a2\",\"bind\":\"s\"}\n\
{\"cmd\":\"create-room\",\"args\":{\"space\":\"$s\",\"name\":\"general\"},\"id\":\"a3\",\"bind\":\"r\"}\n\
{\"cmd\":\"send\",\"args\":{\"space\":\"$s\",\"room\":\"$r\",\"text\":\"only B should see this\"},\"id\":\"a4\"}\n";
    write_scenario(dir, manifest, &[("alice.jsonl", alice), ("bob.jsonl", REGISTER_BOB), ("carol.jsonl", REGISTER_CAROL)]);
}

// ── MP-C-16 — live Space migration during chat (Arc F) ───────────────────────

#[tokio::test]
#[ignore = "heavy: spawns 2 real harness-control nodes + a client; box-gated RUN"]
async fn mp_c_16_live_migration_space_rehomes() {
    let tmp = tempfile::tempdir().expect("tempdir");
    write_mp_c_16(tmp.path());
    let scenario = Scenario::load(tmp.path()).expect("load MP-C-16");
    // A green run means the director `require_ok`'d `migration initiate` — i.e.
    // the verb + args are correct, the Space is homed on A, and B is reachable.
    let outcome = run_scenario(&scenario, &mock_dial())
        .await
        .expect("run_scenario(MP-C-16) — node built --features harness-control?");

    let space = outcome.space_id.clone().expect("alice exported a space_id");
    let tb = outcome.transcripts.iter().find(|t| t.node == "b").expect("b transcript");
    assert!(
        !tb.event_ids_for_space(&space).is_empty(),
        "migrated Space {space} not present on destination B: {:?}",
        tb.events
    );
    eprintln!(
        "C6c MP-C-16 PASS: migration initiate accepted + Space {space} present on destination B \
         (home_node-flip-on-both is the box-gated RUN enrichment — needs a per-Space home query)"
    );
}

/// A homes a Space; the `[[migration]]` step moves it A→B after it is built. b is
/// the bare destination node (no actor). alice exports the Space + a build gate.
fn write_mp_c_16(dir: &Path) {
    let manifest = r#"
scenario = "MP-C-16"
description = "live Space migration A->B during chat"

[[nodes]]
label = "a"
port = 8521
[[nodes]]
label = "b"
port = 8522

[[federation]]
from = "a"
to = "b"

[[actors]]
name = "alice"
node = "a"
batch = "alice.jsonl"

[[exports]]
actor = "alice"
command = "a2"
field = "space_id"
key = "space_id"
[[exports]]
actor = "alice"
command = "a4"
field = "event_id"
key = "space_built"

[[migration]]
space_key = "space_id"
from = "a"
to = "b"
after = "space_built"
"#;
    let alice = "\
{\"cmd\":\"register\",\"args\":{\"name\":\"alice\"},\"id\":\"a1\"}\n\
{\"cmd\":\"create-space\",\"args\":{\"name\":\"MP-C-16\"},\"id\":\"a2\",\"bind\":\"s\"}\n\
{\"cmd\":\"create-room\",\"args\":{\"space\":\"$s\",\"name\":\"general\"},\"id\":\"a3\",\"bind\":\"r\"}\n\
{\"cmd\":\"send\",\"args\":{\"space\":\"$s\",\"room\":\"$r\",\"text\":\"pre-migration\"},\"id\":\"a4\"}\n";
    write_scenario(dir, manifest, &[("alice.jsonl", alice)]);
}

// ── shared helpers ───────────────────────────────────────────────────────────

const REGISTER_BOB: &str = "{\"cmd\":\"register\",\"args\":{\"name\":\"bob\"},\"id\":\"b1\"}\n";
const REGISTER_CAROL: &str = "{\"cmd\":\"register\",\"args\":{\"name\":\"carol\"},\"id\":\"c1\"}\n";

fn write_scenario(dir: &Path, manifest: &str, batches: &[(&str, &str)]) {
    std::fs::write(dir.join("manifest.toml"), manifest).expect("write manifest");
    for (name, body) in batches {
        std::fs::write(dir.join(name), body).expect("write batch");
    }
}
