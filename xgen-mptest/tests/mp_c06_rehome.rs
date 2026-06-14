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
