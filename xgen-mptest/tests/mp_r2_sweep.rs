// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! MP-R2 C5 — tranche (a) scale sweeps: the multi-rung clients-axis sweep
//! (`#[ignore]`).
//!
//! The generator + classifier are unit-tested in `sweep.rs` without spawning.
//! These smokes run the **first multi-rung sweep**: a `ScenarioTemplate::Generated`
//! is expanded per rung over `SweepAxis::Clients`, each rung run, oracle-checked,
//! and classified GREEN/LOGIC-FAULT/CEILING. The **break-point per axis is the
//! deliverable**, not pass/fail (MP-R2-D1(a) + the inherited MP-R1-D2 contract).
//!
//! ## Generated scenarios are CODE, not static dirs (refinement)
//! A `Generated` template is a Rust closure (`actor_batch`) + sizing, not a static
//! `docs/tests/multiparty_scenarios/<ID>/` dir (that convention is for fixed
//! scenarios). MP-C-05/11 live here as template-builder fns; the generator writes
//! the concrete manifest + paced batches to a tempdir per rung.
//!
//! ## Box-gated (RUN gate, M-R2.3) + honest boundary
//! Heavy — spawns `dial.clients` real clients per rung. The break-point RUN +
//! the matrix flip are box-gated. The scenario CONTENT (room-membership-on-join
//! semantics, the exact `send`/`join`/`leave` arg surface) is grounded against
//! matrix §2/§5 but **validated at the first box-gated RUN** — if a joiner cannot
//! post to a room after a Space `join`, that surfaces there (a finding to route,
//! not patch; Rule 2 — a spawn timeout is a flake, re-run isolated). The sweep
//! `max` is a placeholder here (8); the real ceiling comes from the bench
//! (`XGEN_MPTEST_BENCH_TIERS=10,50,100`) at RUN.
//!
//! ```text
//! cargo build -p xgen-node && cargo build -p xgen-client   # single-node, real clock: no harness-control
//! cargo test -p xgen-mptest --test mp_r2_sweep -- --ignored --nocapture
//! ```

use std::sync::Arc;

use xgen_mptest::dial::RoundDial;
use xgen_mptest::sweep::{
    run_sweep, ActorGenCtx, CeilingFloors, FederationPattern, GenExport, GeneratedTemplate,
    RungClass, ScenarioTemplate, Sweep, SweepAxis,
};

/// Posts per actor in the sustained window; the inter-send pace (ms).
const POSTS: usize = 5;
const PACE_MS: u64 = 40;

/// Substitute the cross-actor placeholders AFTER `format!` (so the macro never
/// sees the `{{ }}` braces — the tokens carry no braces).
fn subst(raw: String) -> String {
    raw.replace("SPACE_PH", "{{space_id}}")
        .replace("ROOM_PH", "{{room_id}}")
}

/// The owner (`a0`) batch shared by MP-C-05/11: register → create Space (id `s`,
/// exports `space_id`) → create room (id `r`, exports `room_id`) → paced posts.
fn owner_batch(scenario: &str) -> String {
    let mut b = String::new();
    b.push_str("{\"cmd\":\"register\",\"args\":{\"name\":\"a0\"},\"id\":\"r0\"}\n");
    b.push_str(&format!(
        "{{\"cmd\":\"create-space\",\"args\":{{\"name\":\"{scenario}\"}},\"id\":\"s\",\"bind\":\"sp\"}}\n"
    ));
    b.push_str("{\"cmd\":\"create-room\",\"args\":{\"space\":\"$sp\",\"name\":\"general\"},\"id\":\"r\",\"bind\":\"rm\"}\n");
    for p in 0..POSTS {
        b.push_str(&format!(
            "{{\"cmd\":\"send\",\"args\":{{\"space\":\"$sp\",\"room\":\"$rm\",\"text\":\"a0-m{p}\"}},\"id\":\"p{p}\",\"after_ms\":{PACE_MS}}}\n"
        ));
    }
    b
}

/// The standard owner exports (space_id + room_id) every generated chat needs.
fn chat_exports() -> Vec<GenExport> {
    vec![
        GenExport { actor_index: 0, command: "s".into(), field: "space_id".into(), key: "space_id".into() },
        GenExport { actor_index: 0, command: "r".into(), field: "room_id".into(), key: "room_id".into() },
    ]
}

/// MP-C-05 — sustained n×n chat. Single-node, real clock; the **clients** axis is
/// the participant count. a0 owns + posts; a1.. join the Space and post (paced).
fn mp_c_05_template() -> ScenarioTemplate {
    ScenarioTemplate::Generated(GeneratedTemplate {
        scenario_id: "MP-C-05".into(),
        base_port: 8493,
        federation: FederationPattern::None,
        actor_batch: Arc::new(|ctx: &ActorGenCtx| {
            if ctx.is_owner {
                return owner_batch("MP-C-05");
            }
            let i = ctx.index;
            let mut b = String::new();
            b.push_str(&format!(
                "{{\"cmd\":\"register\",\"args\":{{\"name\":\"a{i}\"}},\"id\":\"r{i}\"}}\n"
            ));
            b.push_str(&subst(
                "{\"cmd\":\"join\",\"args\":{\"space\":\"SPACE_PH\"},\"id\":\"j\"}\n".to_string(),
            ));
            for p in 0..POSTS {
                b.push_str(&subst(format!(
                    "{{\"cmd\":\"send\",\"args\":{{\"space\":\"SPACE_PH\",\"room\":\"ROOM_PH\",\"text\":\"a{i}-m{p}\"}},\"after_ms\":{PACE_MS}}}\n"
                )));
            }
            b
        }),
        exports: chat_exports(),
    })
}

/// MP-C-11 — membership churn under load. a1.. join → post → leave → rejoin
/// (paced), interleaved with a0's sustained posting; the clients axis scales the
/// churning population.
fn mp_c_11_template() -> ScenarioTemplate {
    ScenarioTemplate::Generated(GeneratedTemplate {
        scenario_id: "MP-C-11".into(),
        base_port: 8503,
        federation: FederationPattern::None,
        actor_batch: Arc::new(|ctx: &ActorGenCtx| {
            if ctx.is_owner {
                return owner_batch("MP-C-11");
            }
            let i = ctx.index;
            let mut b = String::new();
            b.push_str(&format!(
                "{{\"cmd\":\"register\",\"args\":{{\"name\":\"a{i}\"}},\"id\":\"r{i}\"}}\n"
            ));
            // join → post → leave → rejoin (the churn), paced.
            b.push_str(&subst(
                "{\"cmd\":\"join\",\"args\":{\"space\":\"SPACE_PH\"},\"id\":\"j\"}\n".to_string(),
            ));
            b.push_str(&subst(format!(
                "{{\"cmd\":\"send\",\"args\":{{\"space\":\"SPACE_PH\",\"room\":\"ROOM_PH\",\"text\":\"a{i}-pre\"}},\"after_ms\":{PACE_MS}}}\n"
            )));
            b.push_str(&subst(format!(
                "{{\"cmd\":\"leave\",\"args\":{{\"space\":\"SPACE_PH\"}},\"id\":\"lv\",\"after_ms\":{PACE_MS}}}\n"
            )));
            b.push_str(&subst(format!(
                "{{\"cmd\":\"join\",\"args\":{{\"space\":\"SPACE_PH\"}},\"id\":\"rj\",\"after_ms\":{PACE_MS}}}\n"
            )));
            b
        }),
        exports: chat_exports(),
    })
}

/// A small placeholder clients sweep (2→8 by 2). The real `max` comes from the
/// bench box-ceiling at the RUN; this proves the multi-rung engine + records a
/// break-point.
fn clients_sweep() -> Sweep {
    Sweep {
        axis: SweepAxis::Clients,
        start: 2,
        step: 2,
        max: 8,
        stop_on_fail: true,
    }
}

#[tokio::test]
#[ignore = "heavy: spawns up to `max` real clients per rung; box-gated RUN"]
async fn mp_c_05_sustained_chat_clients_sweep_curve() {
    let template = mp_c_05_template();
    // Single-node, real clock (no federation, no clock steps ⇒ no harness-control).
    let base = RoundDial {
        nodes: 1,
        clients: 2,
        ..Default::default()
    };
    let result = run_sweep(&template, &clients_sweep(), &base, &CeilingFloors::default())
        .await
        .expect("run_sweep(MP-C-05 clients)");

    // The deliverable is the curve + break-point, not pass/fail.
    eprintln!(
        "MP-C-05 clients-sweep: {} rungs; break_point={:?}",
        result.rungs.len(),
        result.break_point.as_ref().map(|b| (b.rung_index, b.class, &b.detail))
    );
    // Floor sanity: the smallest rung (2 clients) must converge.
    assert!(
        matches!(result.rungs.first().map(|r| r.class), Some(RungClass::Green)),
        "smallest clients rung should converge GREEN: {:?}",
        result.rungs.first().map(|r| &r.verdict.detail)
    );
}

#[tokio::test]
#[ignore = "heavy: spawns up to `max` real clients per rung; box-gated RUN"]
async fn mp_c_11_membership_churn_clients_sweep_curve() {
    let template = mp_c_11_template();
    let base = RoundDial {
        nodes: 1,
        clients: 2,
        ..Default::default()
    };
    let result = run_sweep(&template, &clients_sweep(), &base, &CeilingFloors::default())
        .await
        .expect("run_sweep(MP-C-11 churn)");

    eprintln!(
        "MP-C-11 churn-sweep: {} rungs; break_point={:?}",
        result.rungs.len(),
        result.break_point.as_ref().map(|b| (b.rung_index, b.class, &b.detail))
    );
    assert!(
        matches!(result.rungs.first().map(|r| r.class), Some(RungClass::Green)),
        "smallest churn rung should converge GREEN: {:?}",
        result.rungs.first().map(|r| &r.verdict.detail)
    );
}
