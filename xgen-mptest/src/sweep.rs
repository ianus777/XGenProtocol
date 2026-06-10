// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! The sweep contract (MP-R1-D2 / C2) — locked now so R2/R3 inherit it.
//!
//! [`crate::dial::RoundDial`] is a single point; a [`Sweep`] is a thin layer that
//! yields a **sequence** of dials along one [`SweepAxis`]. The run result is a
//! **curve + break-point** ([`SweepResult`]), not a bool: each rung records its
//! dial, its [`crate::oracle::OracleVerdict`], its peak [`ResourceSample`], and a
//! [`RungClass`].
//!
//! ## The mandatory distinction (D-065) is the stop condition
//! [`classify_rung`] separates a **logic** fault from a **hardware** ceiling — the
//! distinction the audit (§5) requires so the sweep never mislabels "ran out of
//! RAM" as "the protocol broke":
//! - [`RungClass::Green`]  — oracle passed → climb.
//! - [`RungClass::LogicFault`] — oracle failed **and** resources look healthy
//!   (non-convergence / lost admitted event / wrong rejection) → stop + route a
//!   finding (MP-R1-D6).
//! - [`RungClass::Ceiling`] — oracle failed **and** resources show exhaustion
//!   (RSS wall / thread-thrash) → stop, recorded as a **hardware** break-point,
//!   *not* a protocol FAIL.
//!
//! The classifier consults the [`ResourceSample`] before labelling any non-GREEN
//! rung. With no resource evidence (`None`), the conservative call is
//! `LogicFault` (no ceiling evidence found) — flagged, not silently a ceiling.
//!
//! ## R1's use (and the honest boundary)
//! R1 runs a **degenerate single-rung** sweep through the same [`SweepResult`]
//! type (so R2/R3 inherit the contract with no retrofit). C2 builds the type +
//! the single-rung path + the classifier; the multi-rung *climb* mechanics are
//! exercised by R2/R3, which also **calibrate** ceiling detection against the
//! bench-derived box ceiling. Today's ceiling signal is **per-process** (the peak
//! sample's RSS-wall / thread-thrash); aggregate-RSS-vs-box-RAM and
//! OOM-death-by-exit-code are R2/R3 enrichments (recorded, not built here).
//!
//! Topology note: in R1 the scenario manifest is authoritative for topology, so a
//! single-rung sweep's axis value does not change [`crate::runner::run_scenario`]
//! behaviour — the engine is exercised, the climb is not stressed (MP-R1 scope).

use std::sync::Arc;

use crate::dial::RoundDial;
use crate::manifest::Scenario;
use crate::oracle::OracleVerdict;
use crate::resource::ResourceSample;
use crate::runner::run_scenario;
use crate::Result;
use anyhow::Context;

/// Per-process RSS above which the peak process is treated as a runaway / wall
/// (a healthy node is ~15–35 MB; 1.5 GB is ~50× that). Coarse first-pass floor,
/// retuned against the bench box-ceiling in R2/R3.
pub const RSS_WALL_BYTES: u64 = 1_500 * 1024 * 1024;

/// Thread count above which the process is treated as scheduler-thrashing (the
/// binaries pin 1–2 tokio worker threads; 64 is far past any healthy steady
/// state). Coarse first-pass floor.
pub const THREAD_THRASH_COUNT: u32 = 64;

/// Which dial knob a sweep steps. Only the axes with a home in [`RoundDial`]
/// today are modelled; further axes (e.g. a message rate) land when the dial
/// grows the field in R2/R3 (the enum is the open set the design names).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SweepAxis {
    /// Real node processes (topology width).
    Nodes,
    /// Real client processes.
    Clients,
    /// Logical participants multiplexed per client/AI-resident process.
    ResidentsPerProcess,
}

impl SweepAxis {
    /// Apply a rung `value` to `base`, producing the rung's concrete dial.
    pub fn apply(self, base: &RoundDial, value: usize) -> RoundDial {
        let mut d = base.clone();
        match self {
            SweepAxis::Nodes => d.nodes = value,
            SweepAxis::Clients => d.clients = value,
            SweepAxis::ResidentsPerProcess => d.residents_per_process = value,
        }
        d
    }

    /// Read this axis's current value out of a dial.
    pub fn value_of(self, dial: &RoundDial) -> usize {
        match self {
            SweepAxis::Nodes => dial.nodes,
            SweepAxis::Clients => dial.clients,
            SweepAxis::ResidentsPerProcess => dial.residents_per_process,
        }
    }
}

/// A sweep along one axis: rungs are `start, start+step, …, ≤ max`. R1 uses a
/// degenerate single rung (`step == 0`, via [`Sweep::single`]).
#[derive(Debug, Clone)]
pub struct Sweep {
    pub axis: SweepAxis,
    pub start: usize,
    pub step: usize,
    pub max: usize,
    /// Stop the climb on the first `LogicFault` rung. (`Ceiling` always stops —
    /// the hardware wall is reached.)
    pub stop_on_fail: bool,
}

impl Sweep {
    /// A degenerate single-rung sweep at one axis value (R1).
    pub fn single(axis: SweepAxis, value: usize) -> Sweep {
        Sweep {
            axis,
            start: value,
            step: value,
            max: value,
            stop_on_fail: true,
        }
    }

    /// The ordered axis values this sweep visits. A zero step (or `start ≥ max`)
    /// yields a single rung at `start`.
    pub fn rung_values(&self) -> Vec<usize> {
        if self.step == 0 || self.start >= self.max {
            return vec![self.start];
        }
        let mut out = Vec::new();
        let mut x = self.start;
        while x <= self.max {
            out.push(x);
            x += self.step;
        }
        out
    }
}

/// One rung's classification (the D-065 distinction).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RungClass {
    Green,
    LogicFault,
    Ceiling,
}

/// Whether a peak resource sample shows hardware exhaustion (RSS wall or thread
/// thrash). Per-process today (the peak sample); aggregate/OOM detection is an
/// R2/R3 enrichment.
pub fn is_resource_exhausted(sample: &ResourceSample) -> bool {
    sample.rss_bytes >= RSS_WALL_BYTES || sample.thread_count >= THREAD_THRASH_COUNT
}

/// Classify a rung from its oracle verdict and peak resource sample — the
/// pure, unit-tested heart of the sweep (no process spawn).
pub fn classify_rung(verdict: &OracleVerdict, resource: Option<&ResourceSample>) -> RungClass {
    if verdict.pass {
        return RungClass::Green;
    }
    // Non-GREEN: consult resources before labelling (D-065). A wall ⇒ Ceiling;
    // otherwise (healthy resources, or no evidence) ⇒ LogicFault.
    match resource {
        Some(r) if is_resource_exhausted(r) => RungClass::Ceiling,
        _ => RungClass::LogicFault,
    }
}

/// One evaluated rung.
#[derive(Debug)]
pub struct SweepRung {
    pub dial: RoundDial,
    pub verdict: OracleVerdict,
    pub resource: Option<ResourceSample>,
    pub class: RungClass,
}

/// Where (and why) the climb stopped.
#[derive(Debug)]
pub struct BreakPoint {
    pub rung_index: usize,
    pub dial: RoundDial,
    /// `LogicFault` or `Ceiling` (the stop reason).
    pub class: RungClass,
    pub detail: String,
}

/// The result of a sweep: the curve of rungs + an optional break-point.
#[derive(Debug)]
pub struct SweepResult {
    pub rungs: Vec<SweepRung>,
    pub break_point: Option<BreakPoint>,
}

impl SweepResult {
    /// `true` if every rung was GREEN (no break-point).
    pub fn all_green(&self) -> bool {
        self.break_point.is_none()
    }
}

/// Context handed to a generated actor's batch builder ([`ActorBatchFn`]).
pub struct ActorGenCtx {
    /// 0-based actor index (`a0`, `a1`, …). `a0` is the scenario owner.
    pub index: usize,
    /// Total actors this rung generates (= `dial.clients`).
    pub total: usize,
    /// The topology node label this actor is assigned to (`n0`, `n1`, …).
    pub node_label: String,
    /// `true` for the owner actor (`index == 0`) — the one that creates the
    /// shared Space and (in a real chat template) exports it.
    pub is_owner: bool,
}

/// Builds one generated actor's `.jsonl` batch text from its context. Held in an
/// `Arc` (cheap to share across rungs; `Send + Sync` for the async drive). C5
/// (tranche (a)) supplies the real chat/churn closures; C1 ships the plumbing.
pub type ActorBatchFn = Arc<dyn Fn(&ActorGenCtx) -> String + Send + Sync>;

/// How a generated scenario's nodes are federated.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FederationPattern {
    /// No federation links (single-node, or independent nodes).
    None,
    /// A star: every node `n1..` federates with `n0` (the owner's node).
    StarFromFirst,
}

/// A scenario the sweep expands per rung into a concrete [`Scenario`] sized from
/// the rung's [`RoundDial`] (MP-R2-D3 / R-1). The dial's `nodes`/`clients` are
/// consumed **here**, by the generator — [`run_scenario`] stays manifest-
/// authoritative (its spawn loop is unchanged; the (b) fixed-N tranche is
/// byte-unaffected).
pub enum ScenarioTemplate {
    /// A fixed, already-loaded scenario — [`generate`](ScenarioTemplate::generate)
    /// returns it regardless of the dial (the R1 single-rung path; keeps
    /// `mp_r1_sweep.rs` green under the evolved `run_sweep` signature).
    Fixed(Scenario),
    /// A dial-sized generated scenario.
    Generated(GeneratedTemplate),
}

/// The inputs for a dial-sized generated scenario (MP-R2-D3). C1 ships the
/// node/actor/federation sizing; the per-actor batch content is the
/// [`ActorBatchFn`] the (a)-tranche (C5) supplies.
pub struct GeneratedTemplate {
    /// Scenario id (e.g. `"MP-C-05"`).
    pub scenario_id: String,
    /// WS port of `n0`; node `ni` listens on `base_port + i`.
    pub base_port: u16,
    /// Federation among the generated nodes.
    pub federation: FederationPattern,
    /// Builds each actor's batch (`a0`..`a{clients-1}`).
    pub actor_batch: ActorBatchFn,
}

/// A generated [`Scenario`] plus the tempdir backing its files. The tempdir is
/// held so the dir **outlives the run** — dropping `GeneratedScenario` deletes
/// the generated manifest + batches (RAII cleanup; `None` for `Fixed`).
pub struct GeneratedScenario {
    pub scenario: Scenario,
    _tmp: Option<tempfile::TempDir>,
}

impl ScenarioTemplate {
    /// Expand this template into a concrete [`Scenario`] at `dial`'s scale.
    /// `Fixed` ignores the dial; `Generated` writes a manifest + `dial.clients`
    /// batches across `dial.nodes` nodes into a fresh tempdir and loads it. Pure
    /// I/O (no process spawn) — unit-testable without the box.
    pub fn generate(&self, dial: &RoundDial) -> Result<GeneratedScenario> {
        match self {
            ScenarioTemplate::Fixed(s) => Ok(GeneratedScenario {
                scenario: s.clone(),
                _tmp: None,
            }),
            ScenarioTemplate::Generated(t) => {
                let nodes = dial.nodes.max(1);
                let clients = dial.clients.max(1);
                let tmp = tempfile::tempdir().context("generate scenario tempdir")?;
                let mut manifest = format!("scenario = \"{}\"\n", t.scenario_id);
                for i in 0..nodes {
                    manifest.push_str(&format!(
                        "\n[[nodes]]\nlabel = \"n{i}\"\nport = {}\n",
                        t.base_port as usize + i
                    ));
                }
                if t.federation == FederationPattern::StarFromFirst {
                    for i in 1..nodes {
                        manifest.push_str(&format!(
                            "\n[[federation]]\nfrom = \"n0\"\nto = \"n{i}\"\n"
                        ));
                    }
                }
                for a in 0..clients {
                    let node = a % nodes;
                    let ctx = ActorGenCtx {
                        index: a,
                        total: clients,
                        node_label: format!("n{node}"),
                        is_owner: a == 0,
                    };
                    let batch = (t.actor_batch)(&ctx);
                    let fname = format!("a{a}.jsonl");
                    std::fs::write(tmp.path().join(&fname), batch)
                        .with_context(|| format!("write generated batch {fname}"))?;
                    manifest.push_str(&format!(
                        "\n[[actors]]\nname = \"a{a}\"\nnode = \"n{node}\"\nbatch = \"{fname}\"\n"
                    ));
                }
                std::fs::write(tmp.path().join("manifest.toml"), &manifest)
                    .context("write generated manifest")?;
                let scenario = Scenario::load(tmp.path())?;
                Ok(GeneratedScenario {
                    scenario,
                    _tmp: Some(tmp),
                })
            }
        }
    }
}

/// Run a [`ScenarioTemplate`] once per rung, classifying each, until a stop
/// condition (`LogicFault` with `stop_on_fail`, any `Ceiling`, or `max`).
/// **Heavy** — each rung generates a concrete [`Scenario`] from the rung's dial
/// (R-1) and spawns real binaries via [`run_scenario`]; callers are `#[ignore]`
/// / out-of-band. R1 passes a [`ScenarioTemplate::Fixed`] single-rung sweep; R2
/// passes a [`ScenarioTemplate::Generated`] multi-rung sweep.
pub async fn run_sweep(
    template: &ScenarioTemplate,
    sweep: &Sweep,
    base: &RoundDial,
) -> Result<SweepResult> {
    let mut rungs: Vec<SweepRung> = Vec::new();
    let mut break_point: Option<BreakPoint> = None;

    for (index, value) in sweep.rung_values().into_iter().enumerate() {
        let dial = sweep.axis.apply(base, value);
        // R-1: the generator consumes dial.nodes/clients to emit a concrete
        // Scenario for this rung; run_scenario stays manifest-authoritative.
        let generated = template.generate(&dial)?;
        let outcome = run_scenario(&generated.scenario, &dial).await?;
        let class = classify_rung(&outcome.verdict, outcome.resource.as_ref());
        let detail = outcome.verdict.detail.clone();
        rungs.push(SweepRung {
            dial: dial.clone(),
            verdict: outcome.verdict,
            resource: outcome.resource,
            class,
        });
        match class {
            RungClass::Green => {}
            RungClass::LogicFault => {
                break_point = Some(BreakPoint {
                    rung_index: index,
                    dial,
                    class,
                    detail,
                });
                if sweep.stop_on_fail {
                    break;
                }
            }
            RungClass::Ceiling => {
                break_point = Some(BreakPoint {
                    rung_index: index,
                    dial,
                    class,
                    detail,
                });
                break;
            }
        }
    }

    Ok(SweepResult { rungs, break_point })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(rss_mb: u64, threads: u32) -> ResourceSample {
        ResourceSample {
            pid: 1,
            rss_bytes: rss_mb * 1024 * 1024,
            thread_count: threads,
        }
    }

    #[test]
    fn green_when_verdict_passes_regardless_of_resource() {
        let pass = OracleVerdict::pass("converged");
        assert_eq!(classify_rung(&pass, None), RungClass::Green);
        // Even a pathological resource sample is GREEN if the oracle passed.
        assert_eq!(
            classify_rung(&pass, Some(&sample(4096, 200))),
            RungClass::Green
        );
    }

    #[test]
    fn logic_fault_when_fail_with_healthy_resources() {
        let fail = OracleVerdict::fail("membership diverged");
        assert_eq!(
            classify_rung(&fail, Some(&sample(20, 3))),
            RungClass::LogicFault
        );
    }

    #[test]
    fn logic_fault_when_fail_with_no_resource_evidence() {
        // No sample ⇒ no ceiling evidence ⇒ conservative LogicFault, not Ceiling.
        let fail = OracleVerdict::fail("lost an admitted event");
        assert_eq!(classify_rung(&fail, None), RungClass::LogicFault);
    }

    #[test]
    fn ceiling_when_fail_with_rss_wall() {
        let fail = OracleVerdict::fail("inconclusive");
        // 2 GB peak process ≥ the 1.5 GB wall.
        assert_eq!(classify_rung(&fail, Some(&sample(2048, 4))), RungClass::Ceiling);
    }

    #[test]
    fn ceiling_when_fail_with_thread_thrash() {
        let fail = OracleVerdict::fail("inconclusive");
        assert_eq!(
            classify_rung(&fail, Some(&sample(30, THREAD_THRASH_COUNT))),
            RungClass::Ceiling
        );
    }

    #[test]
    fn resource_exhaustion_boundaries() {
        assert!(!is_resource_exhausted(&sample(1499, 63)));
        assert!(is_resource_exhausted(&sample(1500, 1)));
        assert!(is_resource_exhausted(&sample(1, THREAD_THRASH_COUNT)));
    }

    #[test]
    fn single_sweep_has_one_rung() {
        let s = Sweep::single(SweepAxis::Clients, 2);
        assert_eq!(s.rung_values(), vec![2]);
    }

    #[test]
    fn multi_rung_values_climb_by_step_inclusive() {
        let s = Sweep {
            axis: SweepAxis::Clients,
            start: 2,
            step: 2,
            max: 8,
            stop_on_fail: true,
        };
        assert_eq!(s.rung_values(), vec![2, 4, 6, 8]);
    }

    #[test]
    fn zero_step_is_single_rung() {
        let s = Sweep {
            axis: SweepAxis::Nodes,
            start: 3,
            step: 0,
            max: 10,
            stop_on_fail: true,
        };
        assert_eq!(s.rung_values(), vec![3]);
    }

    #[test]
    fn axis_apply_sets_the_right_field() {
        let base = RoundDial::default();
        let d = SweepAxis::Nodes.apply(&base, 5);
        assert_eq!(d.nodes, 5);
        assert_eq!(d.clients, base.clients);
        assert_eq!(SweepAxis::Clients.apply(&base, 7).clients, 7);
        assert_eq!(
            SweepAxis::ResidentsPerProcess.apply(&base, 250).residents_per_process,
            250
        );
        assert_eq!(SweepAxis::Nodes.value_of(&d), 5);
    }

    /// A minimal generated template (each actor just registers) — enough to prove
    /// the generator sizes the manifest from the dial. C5 supplies real closures.
    fn trivial_template(id: &str) -> GeneratedTemplate {
        GeneratedTemplate {
            scenario_id: id.to_string(),
            base_port: 9600,
            federation: FederationPattern::None,
            actor_batch: Arc::new(|ctx: &ActorGenCtx| {
                format!(
                    "{{\"cmd\":\"register\",\"args\":{{\"name\":\"a{}\"}},\"id\":\"r{}\"}}\n",
                    ctx.index, ctx.index
                )
            }),
        }
    }

    #[test]
    fn template_generate_emits_dial_sized_manifest() {
        // MP-R2-D3 / R-1: the generator consumes dial.nodes/clients → a concrete
        // Scenario with that many nodes/actors + a batch file per actor.
        let t = ScenarioTemplate::Generated(trivial_template("MP-GEN-TEST"));
        let dial = RoundDial {
            nodes: 1,
            clients: 4,
            ..Default::default()
        };
        let generated = t.generate(&dial).expect("generate dial-sized scenario");
        assert_eq!(generated.scenario.manifest.actors.len(), 4);
        assert_eq!(generated.scenario.manifest.nodes.len(), 1);
        for a in 0..4 {
            assert!(
                generated.scenario.dir.join(format!("a{a}.jsonl")).exists(),
                "generated batch a{a}.jsonl must exist on disk"
            );
        }
    }

    #[test]
    fn fixed_template_ignores_dial() {
        // R1 single-rung compat: Fixed returns its scenario regardless of dial.
        let seed = ScenarioTemplate::Generated(trivial_template("MP-FIXED-TEST"))
            .generate(&RoundDial {
                nodes: 1,
                clients: 2,
                ..Default::default()
            })
            .expect("seed generate");
        let fixed = ScenarioTemplate::Fixed(seed.scenario.clone());
        let out = fixed
            .generate(&RoundDial {
                nodes: 5,
                clients: 99,
                ..Default::default()
            })
            .expect("fixed generate");
        assert_eq!(
            out.scenario.manifest.actors.len(),
            2,
            "Fixed must ignore the dial (R1 single-rung path)"
        );
    }
}
