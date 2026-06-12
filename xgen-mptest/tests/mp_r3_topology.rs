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
//! **MP-F14-D5 leaf-content coverage enrichment.** The scenario exercises all
//! three post-classes so a GREEN proves *every member sees every post regardless
//! of join order* — not just the happy path where a race didn't bite:
//! - **(a) pre-join creator post** `p0` — authored BEFORE the leaves join (the
//!   load-bearing MP-F14 gap; kept early — do **not** relax into a no-race shape);
//! - **(b) post-join leaf post** `p{i}` — each leaf posts gated on its OWN join
//!   landing (its exported `a{i}_joined` event_id), so the post is genuine
//!   post-join content, not a post racing its own join;
//! - **(c) post-join creator post** `pf` — a0 posts again gated on ALL leaf joins
//!   (the steady-state control).
//!
//! The oracle asserts each post's `event_id` is present on EVERY node's
//! transcript (so a mutual-absence "set-equality" green fails).
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
                // (a) pre-join creator post — authored BEFORE the leaves join
                // (the load-bearing MP-F14 gap; kept early, do not relax it).
                b.push_str("{\"cmd\":\"send\",\"args\":{\"space\":\"$sp\",\"room\":\"$rm\",\"text\":\"a0-hello\"},\"id\":\"p0\",\"after_ms\":40}\n");
                // (c) post-join creator post — a0 posts AGAIN gated on ALL leaf
                // joins (the inert `gate` field carries the `{{a{i}_joined}}`
                // tokens the harness blocks on; the binary tolerates the unknown
                // top-level field). The steady-state control (D5).
                b.push_str("{\"cmd\":\"send\",\"args\":{\"space\":\"$sp\",\"room\":\"$rm\",\"text\":\"a0-post-join\"},\"id\":\"pf\",\"gate\":\"{{a1_joined}} {{a2_joined}} {{a3_joined}}\"}\n");
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
            // (b) post-join leaf post — gated on THIS leaf's own join landing (its
            // exported `a{i}_joined` event_id), so the post is genuine post-join
            // content, not a post racing its own join (D5). `gate` is an inert
            // top-level field the binary tolerates; the harness blocks on its
            // `{{…}}` token. (`{{{{a{i}_joined}}}}` → the literal `{{a{i}_joined}}`.)
            let self_join_gate = format!("{{{{a{i}_joined}}}}");
            b.push_str(
                &subst(format!(
                    "{{\"cmd\":\"send\",\"args\":{{\"space\":\"SPACE_PH\",\"room\":\"ROOM_PH\",\"text\":\"a{i}-hello\"}},\"id\":\"p{i}\",\"gate\":\"GATE_PH\",\"after_ms\":40}}\n"
                ))
                .replace("GATE_PH", &self_join_gate),
            );
            b
        }),
        exports: vec![
            GenExport { actor_index: 0, command: "s".into(), field: "space_id".into(), key: "space_id".into() },
            GenExport { actor_index: 0, command: "r".into(), field: "room_id".into(), key: "room_id".into() },
            // D5: each leaf's join publishes `a{i}_joined` (its join `event_id`) —
            // the gate (b) each leaf post + (c) the post-join creator post wait on.
            GenExport { actor_index: 1, command: "j".into(), field: "event_id".into(), key: "a1_joined".into() },
            GenExport { actor_index: 2, command: "j".into(), field: "event_id".into(), key: "a2_joined".into() },
            GenExport { actor_index: 3, command: "j".into(), field: "event_id".into(), key: "a3_joined".into() },
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

    // MP-F14-D5 — the 3-class coverage assertion. The verdict above proves the
    // cooperative event-id SETS are equal across nodes; that alone admits a hollow
    // "mutually-absent" green (a post rejected/lost everywhere leaves the sets
    // equal). So additionally require every authored post (a/b/c) to be present on
    // EVERY node's transcript, keyed by the post's own `event_id` (from its send
    // reply). A leaf/creator post that never converged is absent from some
    // transcript → fails here.
    let posts: Vec<(String, String)> = {
        let post_eid = |actor: &str, id: &str| -> String {
            o.actor_runs
                .iter()
                .find(|r| r.actor == actor)
                .and_then(|r| r.reply_for(id))
                .and_then(|rep| rep.data_str("event_id"))
                .map(str::to_string)
                .unwrap_or_else(|| {
                    panic!(
                        "MP-C-14: actor `{actor}` post `{id}` produced no event_id reply \
                         (was the post rejected?)"
                    )
                })
        };
        let mut v = vec![
            ("(a) pre-join creator post p0".to_string(), post_eid("a0", "p0")),
            ("(c) post-join creator post pf".to_string(), post_eid("a0", "pf")),
        ];
        for i in 1..4 {
            v.push((
                format!("(b) post-join leaf post a{i}/p{i}"),
                post_eid(&format!("a{i}"), &format!("p{i}")),
            ));
        }
        v
    };
    for t in &o.transcripts {
        for (label, eid) in &posts {
            assert!(
                t.contains_event(eid),
                "MP-C-14: node `{}` transcript is MISSING {label} ({eid}) — the 3-class \
                 enrichment did not fully converge across the star+mesh topology",
                t.node
            );
        }
    }

    eprintln!(
        "MP-C-14 PASS: 4-node star+mesh converged ({} node transcripts); 3-class coverage \
         OK — (a) pre-join creator + 3×(b) leaf + (c) post-join creator present on every node — {}",
        o.transcripts.len(),
        o.verdict.detail
    );
}

/// MP-F14-D5 (fast, no spawn) — structurally lock the 3-class coverage wiring so a
/// regression that drops a post-class / its gate fails the fast suite (the box
/// witness only runs out-of-band). `generate` is pure I/O (no process spawn), so
/// this reads the generated batches + manifest directly.
#[test]
fn mp_c_14_template_wires_3_class_coverage() {
    let dial = RoundDial {
        nodes: 4,
        clients: 4,
        clock: ClockMode::Mock,
        ..Default::default()
    };
    let gen = mp_c_14_template()
        .generate(&dial)
        .expect("generate MP-C-14 star+mesh");

    // (b)+(c) gate on the leaf-join exports — all three must be declared.
    let keys: Vec<&str> = gen
        .scenario
        .manifest
        .exports
        .iter()
        .map(|e| e.key.as_str())
        .collect();
    for k in ["a1_joined", "a2_joined", "a3_joined"] {
        assert!(keys.contains(&k), "missing leaf-join export `{k}`: {keys:?}");
    }

    let read = |actor: &str| -> String {
        let spec = gen
            .scenario
            .manifest
            .actor(actor)
            .unwrap_or_else(|| panic!("actor `{actor}` not generated"));
        std::fs::read_to_string(gen.scenario.batch_path(spec))
            .unwrap_or_else(|e| panic!("read batch for `{actor}`: {e}"))
    };

    // a0: (a) p0 authored BEFORE (c) pf, and pf gates on ALL three leaf joins.
    let a0 = read("a0");
    let p0_at = a0.find("\"id\":\"p0\"").expect("a0 has the pre-join post p0");
    let pf_at = a0
        .find("\"id\":\"pf\"")
        .expect("a0 has the post-join creator post pf");
    assert!(
        p0_at < pf_at,
        "(a) p0 must be authored before (c) pf (the load-bearing p0-before-joins shape)"
    );
    for k in ["{{a1_joined}}", "{{a2_joined}}", "{{a3_joined}}"] {
        assert!(a0.contains(k), "pf must gate on every leaf join; missing `{k}`");
    }

    // Each leaf: (b) its post is gated on its OWN join landing.
    for i in 1..4 {
        let leaf = read(&format!("a{i}"));
        assert!(
            leaf.contains(&format!("\"id\":\"p{i}\"")),
            "a{i} missing the post-join leaf post p{i}"
        );
        assert!(
            leaf.contains(&format!("{{{{a{i}_joined}}}}")),
            "a{i}'s leaf post must gate on its own join export `{{{{a{i}_joined}}}}`"
        );
    }
}
