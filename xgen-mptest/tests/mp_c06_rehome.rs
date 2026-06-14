// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! MP-C-06 (M10.5 C1c) — the identity re-home **rails** smoke (`#[ignore]`, box-gated).
//!
//! Validates the re-home harness rails (D6): **keypair relocation** (one identity
//! across two node connections) + **per-phase node retarget**. alice registers on
//! node A; the `[[rehome]]` step then spawns a NEW client **reusing alice's
//! keypair** pointed at node C and runs `register --re-registration` there. The
//! rail property asserted here: the re-home registers on C with the **same
//! `identity_id`** (key continuity) — proving the keypair was relocated and the
//! client retargeted to the new home.
//!
//! Scope: **rails only** (J-374-independent). The MP-C-06 replicate-convergence
//! WITNESS — alice posts from C → reaches a member on B; B re-points alice's
//! replica to C (`push_identity_to_peers`, A7); identity/membership continuity —
//! lands separately on these rails after the M10.5 re-lock doc-bridge (J-374):
//! **close-on-replicate, no `home_changed` emit** (the emit was dropped as a
//! structural no-op in this topology — single-hop to the new home, which already
//! knows; version-stale; never reaches the peer that needs it).
//!
//! ```text
//! cargo build -p xgen-node --features harness-control && cargo build -p xgen-client
//! cargo test -p xgen-mptest --test mp_c06_rehome -- --ignored --nocapture
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

#[tokio::test]
#[ignore = "heavy: spawns 2 real harness-control nodes + a re-home client; box-gated RUN"]
async fn mp_c06_rehome_preserves_identity_on_new_home() {
    let tmp = tempfile::tempdir().expect("tempdir");
    write_mp_c06(tmp.path());
    let scenario = Scenario::load(tmp.path()).expect("load MP-C-06 rails smoke");
    let outcome = run_scenario(&scenario, &mock_dial())
        .await
        .expect("run_scenario(MP-C-06 rehome) — node built --features harness-control?");

    // alice's original identity_id (her register on A).
    let alice = outcome
        .actor_runs
        .iter()
        .find(|r| r.actor == "alice")
        .expect("alice run present");
    let orig_id = alice
        .reply_for("a1")
        .and_then(|r| r.data_str("identity_id"))
        .expect("alice register reply carries identity_id")
        .to_string();

    // The re-home run: `register --re-registration` on C, reusing alice's keypair
    // (driven by the [[rehome]] phase as actor "alice-rehome").
    let rehome = outcome
        .actor_runs
        .iter()
        .find(|r| r.actor == "alice-rehome")
        .expect("alice-rehome run present (the re-home phase drove it)");
    assert!(
        rehome.all_ok(),
        "re-home register did not succeed: {:?}",
        rehome.replies
    );
    let rehomed_id = rehome
        .reply_for("r1")
        .and_then(|r| r.data_str("identity_id"))
        .expect("re-home register reply carries identity_id")
        .to_string();

    assert_eq!(
        rehomed_id, orig_id,
        "key continuity: the re-home on C must present the SAME identity_id as the original on A \
         (keypair relocation rail)"
    );
    eprintln!(
        "MP-C-06 rails PASS: alice re-homed A→C with continuous identity_id {orig_id} \
         (convergence witness rides J-374 re-sync)"
    );
}

/// A homes alice; the `[[rehome]]` step re-homes her identity to C (a bare node).
/// A↔C federated early so alice's record replicates to C before the re-home (a
/// faithful `re_home`). Minimal rails scenario — no Space/post (the convergence
/// witness adds those after J-374).
fn write_mp_c06(dir: &Path) {
    let manifest = r#"
scenario = "MP-C-06"
description = "identity re-home rails smoke: alice A->C, key continuity"

[[nodes]]
label = "a"
port = 8530
[[nodes]]
label = "c"
port = 8532

[[federation]]
from = "a"
to = "c"

[[actors]]
name = "alice"
node = "a"
batch = "alice.jsonl"

[[rehome]]
actor = "alice"
to = "c"
batch = "alice_rehome.jsonl"

[[exports]]
actor = "alice"
command = "a2"
field = "space_id"
key = "space_id"
"#;
    // alice creates a Space so the A↔C federation has a shared Space to name (the
    // director resolves it from the `space_id` export) + so her record replicates
    // to C before the re-home (a faithful `re_home`).
    let alice = "\
{\"cmd\":\"register\",\"args\":{\"name\":\"alice\"},\"id\":\"a1\"}\n\
{\"cmd\":\"create-space\",\"args\":{\"name\":\"MP-C-06\"},\"id\":\"a2\"}\n";
    let alice_rehome =
        "{\"cmd\":\"register\",\"args\":{\"name\":\"alice\",\"re_registration\":true},\"id\":\"r1\"}\n";
    std::fs::write(dir.join("manifest.toml"), manifest).expect("write manifest");
    std::fs::write(dir.join("alice.jsonl"), alice).expect("write alice batch");
    std::fs::write(dir.join("alice_rehome.jsonl"), alice_rehome).expect("write rehome batch");
}

// ── MP-C-06 — the replicate-convergence WITNESS (3c, J-374) ───────────────────

/// MP-C-06 (M10.5 C1c, J-374) — the replicate-convergence WITNESS on the rails.
///
/// Full-mesh A↔B↔C. alice homes on A (creates the Space + a Room + a pre-home
/// post); bob co-members on B (open-join). The `[[rehome]]` step re-homes alice
/// A→C (`register --re_registration`, keypair relocated) and she posts again from
/// C. Per the J-374 CP-4 re-lock there is **no `home_changed` emit** — MP-C-06
/// closes on **replicate-convergence (A7)**: C's re-registration
/// `push_identity_to_peers` re-points alice's replica on the peers to C. Asserts
/// the real mechanism, not observability:
///   1. **re-point** — B's stored `home_node` for alice equals C's (C homes
///      alice ⇒ its own record's home is C's node_id); B re-pointed there.
///      RED-on-revert: neuter `push_identity_to_peers` ⇒ B keeps home=A ≠ C ⇒ fail.
///   2. **identity continuity** — the re-home presents the SAME `identity_id`.
///      RED-on-revert: neuter the rails' keypair reuse ⇒ a different `identity_id`.
///   3. **membership preserved** — B's resolved Space owner is still alice.
///   4. **functional convergence** — alice's post FROM C reaches the Space on B
///      (its `event_id` lands in B's transcript). RED-on-revert: a broken keypair
///      ⇒ alice-rehome is a stranger ⇒ the owner post never lands.
///
/// Grounding note (J-374, D-065): the re-point is witnessed *directly* (1) — the
/// functional post (4) does NOT itself depend on the re-point (B validates
/// alice's signature by the pubkey it already holds), so it proves operate-from-
/// new-home, while (1) proves the registry re-point. Both halves of A7's DoD.
#[tokio::test]
#[ignore = "heavy: spawns 3 real harness-control nodes + a re-home client; box-gated RUN"]
async fn mp_c06_rehome_converges_on_replicate() {
    let tmp = tempfile::tempdir().expect("tempdir");
    write_mp_c06_convergence(tmp.path());
    let scenario = Scenario::load(tmp.path()).expect("load MP-C-06 convergence witness");
    let outcome = run_scenario(&scenario, &mock_dial())
        .await
        .expect("run_scenario(MP-C-06 convergence) — node built --features harness-control?");

    let space = outcome.space_id.clone().expect("alice exported the Space id");

    // alice's continuous identity (her register on A).
    let alice = outcome
        .actor_runs
        .iter()
        .find(|r| r.actor == "alice")
        .expect("alice run present");
    let alice_id = alice
        .reply_for("a1")
        .and_then(|r| r.data_str("identity_id"))
        .expect("alice register carries identity_id")
        .to_string();

    // (2) identity continuity — the re-home all-ok + the SAME identity_id.
    let rehome = outcome
        .actor_runs
        .iter()
        .find(|r| r.actor == "alice-rehome")
        .expect("alice-rehome run present");
    assert!(
        rehome.all_ok(),
        "re-home batch did not all succeed: {:?}",
        rehome.replies
    );
    let rehomed_id = rehome
        .reply_for("r1")
        .and_then(|r| r.data_str("identity_id"))
        .expect("re-home register carries identity_id")
        .to_string();
    assert_eq!(
        rehomed_id, alice_id,
        "key continuity: the re-home on C must present alice's original identity_id"
    );

    // (1) re-point (A7) — B re-pointed alice's replica to C. C homes alice, so
    // C's own record reports home = C's node_id; B's record must equal it. (No
    // literal node_id needed: comparing the two captured values is the witness —
    // and it bites: neutering push_identity_to_peers leaves B at home=A ≠ C.)
    let c_home = outcome
        .node_identity_home("c", &alice_id)
        .expect("C holds alice's record (her new home)");
    let b_home = outcome
        .node_identity_home("b", &alice_id)
        .expect("B holds a replica of alice's record");
    assert_eq!(
        b_home, c_home,
        "replicate-convergence: B did not re-point alice's replica to C \
         (B home_node={b_home}, C home_node={c_home}) — push_identity_to_peers (A7)"
    );

    // (3) membership preserved — B's resolved view still has alice as Space owner.
    let bob_view = outcome
        .projections
        .iter()
        .find(|p| p.node_label.as_deref() == Some("b"))
        .expect("bob projects B's view of the Space");
    assert_eq!(
        bob_view.owner_id, alice_id,
        "membership preserved: B's resolved Space owner ({}) != alice ({alice_id})",
        bob_view.owner_id
    );

    // (4) functional convergence — alice's post FROM C reached the Space on B.
    let post_id = rehome
        .reply_for("r2")
        .and_then(|r| r.data_str("event_id"))
        .expect("re-home send carries event_id")
        .to_string();
    let tb = outcome
        .transcripts
        .iter()
        .find(|t| t.node == "b")
        .expect("b transcript present");
    assert!(
        tb.event_ids_for_space(&space).contains(&post_id),
        "functional convergence: alice's post-re-home message {post_id} from C did not reach \
         the Space on B: {:?}",
        tb.event_ids_for_space(&space)
    );

    eprintln!(
        "MP-C-06 convergence PASS: B re-pointed alice→C (home={b_home}); identity continuous \
         ({alice_id}); membership preserved (owner=alice); post-from-C {post_id} reached B"
    );
}

/// Full-mesh A↔B↔C. alice homes on A (Space + Room + pre-home post), bob
/// co-members on B (open-join). The `[[rehome]]` step re-homes alice A→C and she
/// posts again from C. A↔C carries her record to C before the re-home (faithful
/// `re_home`); C↔B is what lets C's `push_identity_to_peers` re-point B + lets
/// the post federate C→B.
fn write_mp_c06_convergence(dir: &Path) {
    let manifest = r#"
scenario = "MP-C-06-conv"
description = "identity re-home A->C replicate-convergence (full-mesh A<->B<->C)"

[[nodes]]
label = "a"
port = 8540
[[nodes]]
label = "b"
port = 8541
[[nodes]]
label = "c"
port = 8542

[[federation]]
from = "a"
to = "b"
[[federation]]
from = "a"
to = "c"
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

[[rehome]]
actor = "alice"
to = "c"
batch = "alice_rehome.jsonl"

[[exports]]
actor = "alice"
command = "a2"
field = "space_id"
key = "space_id"
[[exports]]
actor = "alice"
command = "a3"
field = "room_id"
key = "room_id"
"#;
    let alice = "\
{\"cmd\":\"register\",\"args\":{\"name\":\"alice\"},\"id\":\"a1\"}\n\
{\"cmd\":\"create-space\",\"args\":{\"name\":\"MP-C-06\"},\"id\":\"a2\",\"bind\":\"s\"}\n\
{\"cmd\":\"create-room\",\"args\":{\"space\":\"$s\",\"name\":\"general\"},\"id\":\"a3\",\"bind\":\"r\"}\n\
{\"cmd\":\"send\",\"args\":{\"space\":\"$s\",\"room\":\"$r\",\"text\":\"pre-re-home\"},\"id\":\"a4\"}\n";
    let bob = "\
{\"cmd\":\"register\",\"args\":{\"name\":\"bob\"},\"id\":\"b1\"}\n\
{\"cmd\":\"join\",\"args\":{\"space\":\"{{space_id}}\"},\"id\":\"b2\"}\n";
    let alice_rehome = "\
{\"cmd\":\"register\",\"args\":{\"name\":\"alice\",\"re_registration\":true},\"id\":\"r1\"}\n\
{\"cmd\":\"send\",\"args\":{\"space\":\"{{space_id}}\",\"room\":\"{{room_id}}\",\"text\":\"post-from-new-home\"},\"id\":\"r2\"}\n";
    std::fs::write(dir.join("manifest.toml"), manifest).expect("write manifest");
    std::fs::write(dir.join("alice.jsonl"), alice).expect("write alice batch");
    std::fs::write(dir.join("bob.jsonl"), bob).expect("write bob batch");
    std::fs::write(dir.join("alice_rehome.jsonl"), alice_rehome).expect("write rehome batch");
}
