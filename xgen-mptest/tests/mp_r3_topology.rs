// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! MP-R3 capstone — MP-C-14 star → star+mesh topology (`#[ignore]`).
//!
//! MP-R3 §6.3: a 4-node Space under the `StarPlusMesh` federation pattern (the
//! star n0→ni PLUS the leaf cross-links ni→nj — a full mesh; the generator's link
//! emission is unit-proven in `sweep.rs`). The owner (a0@n0) creates the Space +
//! room; each leaf (a1@n1, a2@n2, a3@n3) joins cross-node and posts. Delivery +
//! convergence must hold under the wider topology (the F-5/D-089 pairwise model;
//! MP-A-13 anti-transitivity is the guard).
//!
//! ```text
//! cargo build -p xgen-node --features harness-control && cargo build -p xgen-client
//! cargo test -p xgen-mptest --test mp_r3_topology -- --ignored --nocapture
//! ```

use std::sync::Arc;

use xgen_mptest::dial::{ClockMode, RoundDial};
use xgen_mptest::runner::run_scenario;
use xgen_mptest::sweep::{
    ActorGenCtx, FederationPattern, GenExport, GeneratedTemplate, ScenarioTemplate,
};

/// Substitute the cross-actor placeholders AFTER `format!` (the tokens carry no
/// `{{ }}` braces so the macro never sees them).
fn subst(raw: String) -> String {
    raw.replace("SPACE_PH", "{{space_id}}")
        .replace("ROOM_PH", "{{room_id}}")
}

/// MP-C-14 generated template — a0 owns + posts; a1.. join the Space cross-node
/// and post. Federation = StarPlusMesh (full mesh among the topology nodes).
fn mp_c_14_template() -> ScenarioTemplate {
    ScenarioTemplate::Generated(GeneratedTemplate {
        scenario_id: "MP-C-14".into(),
        base_port: 8507,
        federation: FederationPattern::StarPlusMesh,
        actor_batch: Arc::new(|ctx: &ActorGenCtx| {
            if ctx.is_owner {
                let mut b = String::new();
                b.push_str("{\"cmd\":\"register\",\"args\":{\"name\":\"a0\"},\"id\":\"r0\"}\n");
                b.push_str("{\"cmd\":\"create-space\",\"args\":{\"name\":\"MP-C-14\"},\"id\":\"s\",\"bind\":\"sp\"}\n");
                b.push_str("{\"cmd\":\"create-room\",\"args\":{\"space\":\"$sp\",\"name\":\"general\"},\"id\":\"r\",\"bind\":\"rm\"}\n");
                b.push_str("{\"cmd\":\"send\",\"args\":{\"space\":\"$sp\",\"room\":\"$rm\",\"text\":\"a0-hello\"},\"id\":\"p0\",\"after_ms\":40}\n");
                return b;
            }
            let i = ctx.index;
            let mut b = String::new();
            b.push_str(&format!(
                "{{\"cmd\":\"register\",\"args\":{{\"name\":\"a{i}\"}},\"id\":\"r{i}\"}}\n"
            ));
            b.push_str(&subst(
                "{\"cmd\":\"join\",\"args\":{\"space\":\"SPACE_PH\"},\"id\":\"j\"}\n".to_string(),
            ));
            b.push_str(&subst(format!(
                "{{\"cmd\":\"send\",\"args\":{{\"space\":\"SPACE_PH\",\"room\":\"ROOM_PH\",\"text\":\"a{i}-hello\"}},\"after_ms\":40}}\n"
            )));
            b
        }),
        exports: vec![
            GenExport { actor_index: 0, command: "s".into(), field: "space_id".into(), key: "space_id".into() },
            GenExport { actor_index: 0, command: "r".into(), field: "room_id".into(), key: "room_id".into() },
        ],
    })
}

/// MP-C-14 — 4 nodes star+mesh; the Space converges across all nodes.
#[tokio::test]
#[ignore = "heavy: spawns a harness-control xgen-node ×4 + 4 clients; run with --ignored"]
async fn mp_c_14_star_plus_mesh_converges() {
    let template = mp_c_14_template();
    let dial = RoundDial {
        nodes: 4,
        clients: 4,
        clock: ClockMode::Mock,
        settle_max_secs: Some(45),
        ..Default::default()
    };
    let gen = template.generate(&dial).expect("generate MP-C-14 star+mesh");
    let o = run_scenario(&gen.scenario, &dial)
        .await
        .unwrap_or_else(|e| panic!("run_scenario(MP-C-14): {e:#}"));
    assert!(
        o.verdict.pass,
        "MP-C-14: the Space did not converge across the 4-node star+mesh topology — {}",
        o.verdict.detail
    );
    eprintln!(
        "MP-C-14 PASS: 4-node star+mesh converged ({} node transcripts) — {}",
        o.transcripts.len(),
        o.verdict.detail
    );
}
