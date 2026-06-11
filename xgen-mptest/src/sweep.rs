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
/// (a healthy node is ~15–35 MB; 1.5 GB is ~50× that). The **coarse default**
/// floor ([`CeilingFloors::default`]); MP-R2-D4 calibrates it against the bench
/// box-ceiling via [`CeilingFloors::from_bench`].
pub const RSS_WALL_BYTES: u64 = 1_500 * 1024 * 1024;

/// Thread count above which the process is treated as scheduler-thrashing (the
/// binaries pin 1–2 tokio worker threads; 64 is far past any healthy steady
/// state). The **coarse default** floor — a steady-state property, not box-RAM-
/// bound, so [`CeilingFloors::from_bench`] leaves it at the default.
pub const THREAD_THRASH_COUNT: u32 = 64;

/// The runaway multiple: a process whose RSS exceeds `RSS_RUNAWAY_MULTIPLE × the
/// measured healthy mean RSS` is treated as a wall. The coarse default's 1.5 GB
/// ≈ 50× a ~30 MB node, so this is the same shape, now tracking the *measured*
/// footprint instead of a hardwired guess (MP-R2-D4 caveat 1).
pub const RSS_RUNAWAY_MULTIPLE: f64 = 50.0;

/// The CEILING-classifier floors (MP-R2-D4 / R-2). The classifier's RSS-wall +
/// thread-thrash thresholds are a **runtime struct** (not the compile-time
/// consts) so a bench run can calibrate them: [`CeilingFloors::default`] = the
/// coarse consts (R1 behaviour where no bench ran); [`CeilingFloors::from_bench`]
/// = the bench-derived floors. Threaded into [`run_sweep`] and the classifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CeilingFloors {
    pub rss_wall_bytes: u64,
    pub thread_thrash_count: u32,
}

impl Default for CeilingFloors {
    fn default() -> Self {
        CeilingFloors {
            rss_wall_bytes: RSS_WALL_BYTES,
            thread_thrash_count: THREAD_THRASH_COUNT,
        }
    }
}

impl CeilingFloors {
    /// Derive calibrated floors from a measured box-ceiling report (MP-R2-D4
    /// caveat 1 / R-2). The RSS wall = [`RSS_RUNAWAY_MULTIPLE`] × the measured
    /// healthy mean RSS (so the wall tracks the real node footprint on *this*
    /// box, not a hardwired guess); thread-thrash stays the steady-state default
    /// (it is not box-RAM-bound). A degenerate report (zero/absent mean) falls
    /// back to the coarse default, so calibration never yields a softer-than-
    /// default RSS wall.
    pub fn from_bench(report: &crate::bench::BoxCeilingReport) -> CeilingFloors {
        let mean = report.reference_mean_rss_bytes();
        let rss_wall_bytes = if mean > 0.0 {
            (mean * RSS_RUNAWAY_MULTIPLE) as u64
        } else {
            RSS_WALL_BYTES
        };
        CeilingFloors {
            rss_wall_bytes,
            thread_thrash_count: THREAD_THRASH_COUNT,
        }
    }
}

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
/// thrash) against the given [`CeilingFloors`]. Per-process today (the peak
/// sample); aggregate/OOM detection is an R3 enrichment (MP-R2-D4 defers 2/3/4).
pub fn is_resource_exhausted(sample: &ResourceSample, floors: &CeilingFloors) -> bool {
    sample.rss_bytes >= floors.rss_wall_bytes || sample.thread_count >= floors.thread_thrash_count
}

/// Classify a rung from its oracle verdict + peak resource sample against the
/// calibrated [`CeilingFloors`] — the pure, unit-tested heart of the sweep (no
/// process spawn).
pub fn classify_rung(
    verdict: &OracleVerdict,
    resource: Option<&ResourceSample>,
    floors: &CeilingFloors,
) -> RungClass {
    if verdict.pass {
        return RungClass::Green;
    }
    // Non-GREEN: consult resources before labelling (D-065).
    match resource {
        // A sampled wall ⇒ hardware ceiling.
        Some(r) if is_resource_exhausted(r, floors) => RungClass::Ceiling,
        // Sampled + healthy ⇒ a genuine logic fault.
        Some(_) => RungClass::LogicFault,
        // MP-R2-D4 caveat 5 (the deliberate R1-default reversal): a failed rung
        // with **no** resource sample is a CEILING-SUSPECT, not a silent
        // LogicFault. `Get-Process` sampling fails most under memory pressure —
        // exactly when a true ceiling should fire — so absence-of-sample on a
        // fail is treated as ceiling evidence (R1's conservative None→LogicFault
        // was right for a single small reliable rung; at R2 scale it inverts).
        // The "ceiling-suspect" provenance is recorded in `BreakPoint.detail`.
        None => RungClass::Ceiling,
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
    /// MP-R3 §6.3 (MP-C-14) — a star **plus** leaf cross-links: the star
    /// (`n0 → ni`) plus a mesh among the leaves (`ni → nj`, `1 ≤ i < j`). The
    /// generated link set is every node pair (a full mesh), so delivery +
    /// convergence are exercised under the wider topology (pairwise-trust model,
    /// MP-A-13 anti-transitivity is the guard).
    StarPlusMesh,
}

/// A scenario the sweep expands per rung into a concrete [`Scenario`] sized from
/// the rung's [`RoundDial`] (MP-R2-D3 / R-1). The dial's `nodes`/`clients` are
/// consumed **here**, by the generator — [`run_scenario`] stays manifest-
/// authoritative (its spawn loop is unchanged; the (b) fixed-N tranche is
/// byte-unaffected).
pub enum ScenarioTemplate {
    /// A fixed, already-loaded scenario — [`generate`](ScenarioTemplate::generate)
    /// returns it regardless of the dial (the R1 single-rung path; keeps
    /// `mp_r1_sweep.rs` green under the evolved `run_sweep` signature). Boxed —
    /// `Scenario` is far larger than `GeneratedTemplate` (clippy
    /// large-enum-variant), more so after the MP-R3 manifest grew `Manifest`.
    Fixed(Box<Scenario>),
    /// A dial-sized generated scenario.
    Generated(GeneratedTemplate),
}

/// One cross-actor export the generated manifest emits (MP-R2 C5 extension).
/// A federated / multi-client generated scenario needs these so a joiner's
/// `{{space_id}}` / `{{room_id}}` resolves + the federation director sees the
/// primary Space (C1 flagged exports/waits generation as a C5 extension).
#[derive(Debug, Clone)]
pub struct GenExport {
    /// 0-based actor index whose reply is exported (`a{index}`).
    pub actor_index: usize,
    /// The producing command's id (must match the actor-batch closure's line id).
    pub command: String,
    /// The reply field to publish (e.g. `"space_id"`).
    pub field: String,
    /// The `{{key}}` consumers reference (e.g. `"space_id"` = the primary key).
    pub key: String,
}

/// The inputs for a dial-sized generated scenario (MP-R2-D3). C1 ships the
/// node/actor/federation sizing; the per-actor batch content is the
/// [`ActorBatchFn`] the (a)-tranche (C5) supplies, and `exports` wires the
/// cross-actor `{{key}}`s the generated batches reference (C5 extension).
pub struct GeneratedTemplate {
    /// Scenario id (e.g. `"MP-C-05"`).
    pub scenario_id: String,
    /// WS port of `n0`; node `ni` listens on `base_port + i`.
    pub base_port: u16,
    /// Federation among the generated nodes.
    pub federation: FederationPattern,
    /// Builds each actor's batch (`a0`..`a{clients-1}`).
    pub actor_batch: ActorBatchFn,
    /// Cross-actor exports the generated manifest emits (C5; empty ⇒ none).
    pub exports: Vec<GenExport>,
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
                scenario: (**s).clone(),
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
                match t.federation {
                    FederationPattern::None => {}
                    FederationPattern::StarFromFirst => {
                        for i in 1..nodes {
                            manifest.push_str(&format!(
                                "\n[[federation]]\nfrom = \"n0\"\nto = \"n{i}\"\n"
                            ));
                        }
                    }
                    FederationPattern::StarPlusMesh => {
                        // Star: n0 → ni (the central node to each leaf).
                        for i in 1..nodes {
                            manifest.push_str(&format!(
                                "\n[[federation]]\nfrom = \"n0\"\nto = \"n{i}\"\n"
                            ));
                        }
                        // Mesh: ni → nj leaf cross-links (i < j).
                        for i in 1..nodes {
                            for j in (i + 1)..nodes {
                                manifest.push_str(&format!(
                                    "\n[[federation]]\nfrom = \"n{i}\"\nto = \"n{j}\"\n"
                                ));
                            }
                        }
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
                // C5: cross-actor exports so joiners' `{{space_id}}`/`{{room_id}}`
                // resolve + the federation director sees the primary Space.
                for e in &t.exports {
                    manifest.push_str(&format!(
                        "\n[[exports]]\nactor = \"a{}\"\ncommand = \"{}\"\nfield = \"{}\"\nkey = \"{}\"\n",
                        e.actor_index, e.command, e.field, e.key
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
    floors: &CeilingFloors,
) -> Result<SweepResult> {
    let mut rungs: Vec<SweepRung> = Vec::new();
    let mut break_point: Option<BreakPoint> = None;

    for (index, value) in sweep.rung_values().into_iter().enumerate() {
        let dial = sweep.axis.apply(base, value);
        // R-1: the generator consumes dial.nodes/clients to emit a concrete
        // Scenario for this rung; run_scenario stays manifest-authoritative.
        let generated = template.generate(&dial)?;
        let outcome = run_scenario(&generated.scenario, &dial).await?;
        let class = classify_rung(&outcome.verdict, outcome.resource.as_ref(), floors);
        // MP-R2-D4 caveat 5: record the ceiling-suspect provenance when a failed
        // rung had no resource sample (the None→Ceiling reversal), so the
        // break-point is honest about *why* it is a ceiling.
        let detail = match (class, outcome.resource.as_ref()) {
            (RungClass::Ceiling, None) => format!(
                "resource-sample-unavailable: ceiling-suspect ({})",
                outcome.verdict.detail
            ),
            _ => outcome.verdict.detail.clone(),
        };
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
        let f = CeilingFloors::default();
        let pass = OracleVerdict::pass("converged");
        assert_eq!(classify_rung(&pass, None, &f), RungClass::Green);
        // Even a pathological resource sample is GREEN if the oracle passed.
        assert_eq!(
            classify_rung(&pass, Some(&sample(4096, 200)), &f),
            RungClass::Green
        );
    }

    #[test]
    fn logic_fault_when_fail_with_healthy_resources() {
        let fail = OracleVerdict::fail("membership diverged");
        assert_eq!(
            classify_rung(&fail, Some(&sample(20, 3)), &CeilingFloors::default()),
            RungClass::LogicFault
        );
    }

    #[test]
    fn ceiling_suspect_when_fail_with_no_resource_sample() {
        // MP-R2-D4 caveat 5 — the deliberate R1-default reversal (RED-on-revert
        // witness): a failed rung with NO resource sample is a CEILING-suspect,
        // NOT a LogicFault (R1 returned LogicFault here; revert the None branch →
        // this assertion fails). Sampling fails most under memory pressure, so
        // absence-of-sample on a fail is ceiling evidence.
        let fail = OracleVerdict::fail("lost an admitted event");
        assert_eq!(
            classify_rung(&fail, None, &CeilingFloors::default()),
            RungClass::Ceiling
        );
    }

    #[test]
    fn ceiling_when_fail_with_rss_wall() {
        let fail = OracleVerdict::fail("inconclusive");
        // 2 GB peak process ≥ the 1.5 GB default wall.
        assert_eq!(
            classify_rung(&fail, Some(&sample(2048, 4)), &CeilingFloors::default()),
            RungClass::Ceiling
        );
    }

    #[test]
    fn ceiling_when_fail_with_thread_thrash() {
        let fail = OracleVerdict::fail("inconclusive");
        assert_eq!(
            classify_rung(
                &fail,
                Some(&sample(30, THREAD_THRASH_COUNT)),
                &CeilingFloors::default()
            ),
            RungClass::Ceiling
        );
    }

    #[test]
    fn resource_exhaustion_boundaries() {
        let f = CeilingFloors::default();
        assert!(!is_resource_exhausted(&sample(1499, 63), &f));
        assert!(is_resource_exhausted(&sample(1500, 1), &f));
        assert!(is_resource_exhausted(&sample(1, THREAD_THRASH_COUNT), &f));
    }

    #[test]
    fn is_resource_exhausted_uses_provided_floors() {
        // The judgment is against the passed floors, not the consts.
        let tight = CeilingFloors {
            rss_wall_bytes: 100 * 1024 * 1024,
            thread_thrash_count: 8,
        };
        // 150 MB trips the tight 100 MB wall but not the 1.5 GB default.
        assert!(is_resource_exhausted(&sample(150, 1), &tight));
        assert!(!is_resource_exhausted(&sample(150, 1), &CeilingFloors::default()));
        // 8 threads trips the tight thread floor.
        assert!(is_resource_exhausted(&sample(1, 8), &tight));
    }

    #[test]
    fn floors_from_bench_derive_calibrated_walls() {
        // MP-R2-D4 caveat 1 / R-2: from_bench scales the RSS wall off the measured
        // mean RSS (here 20 MB → 20 MB × RSS_RUNAWAY_MULTIPLE), distinct from the
        // coarse 1.5 GB default. A single tier of 10 procs × 20 MB ⇒ mean 20 MB.
        let report = crate::bench::BoxCeilingReport {
            box_spec: crate::bench::BoxSpec::default(),
            tiers: vec![crate::bench::TierResult {
                tier: 10,
                aggregate: crate::resource::Aggregate {
                    count: 10,
                    total_rss_bytes: 10 * 20 * 1024 * 1024,
                    max_rss_bytes: 25 * 1024 * 1024,
                    total_threads: 30,
                    max_threads: 3,
                },
            }],
        };
        let floors = CeilingFloors::from_bench(&report);
        assert_eq!(
            floors.rss_wall_bytes,
            (20.0 * 1024.0 * 1024.0 * RSS_RUNAWAY_MULTIPLE) as u64
        );
        assert_ne!(
            floors.rss_wall_bytes,
            CeilingFloors::default().rss_wall_bytes,
            "calibrated wall must differ from the coarse default"
        );
        // A degenerate (empty) report falls back to the coarse default.
        let empty = crate::bench::BoxCeilingReport {
            box_spec: crate::bench::BoxSpec::default(),
            tiers: vec![],
        };
        assert_eq!(
            CeilingFloors::from_bench(&empty).rss_wall_bytes,
            RSS_WALL_BYTES
        );
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
            exports: vec![],
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
    fn star_plus_mesh_emits_leaf_cross_links() {
        // MP-R3 §6.3 (MP-C-14): StarPlusMesh emits the star (n0→ni) PLUS the leaf
        // cross-links (ni→nj) → a full mesh of N*(N-1)/2 links. For 4 nodes: star
        // 3 + leaf-mesh 3 = 6 = C(4,2). Contrast with the star-only count (N-1=3).
        let mut t = trivial_template("MP-C-14");
        t.federation = FederationPattern::StarPlusMesh;
        let generated = ScenarioTemplate::Generated(t)
            .generate(&RoundDial {
                nodes: 4,
                clients: 4,
                ..Default::default()
            })
            .expect("generate star+mesh");
        let fed = &generated.scenario.manifest.federation;
        assert_eq!(fed.len(), 6, "4-node star+mesh = C(4,2) = 6 links, got {}", fed.len());
        // The star links are present (n0 → n1..n3).
        for i in 1..4 {
            assert!(
                fed.iter().any(|l| l.from == "n0" && l.to == format!("n{i}")),
                "missing star link n0 → n{i}"
            );
        }
        // The leaf cross-links are present (n1→n2, n1→n3, n2→n3).
        assert!(fed.iter().any(|l| l.from == "n1" && l.to == "n2"), "missing leaf link n1 → n2");
        assert!(fed.iter().any(|l| l.from == "n2" && l.to == "n3"), "missing leaf link n2 → n3");

        // Star-only (the contrast): N-1 links, no leaf cross-links.
        let mut s = trivial_template("MP-STAR");
        s.federation = FederationPattern::StarFromFirst;
        let star = ScenarioTemplate::Generated(s)
            .generate(&RoundDial { nodes: 4, clients: 4, ..Default::default() })
            .expect("generate star");
        assert_eq!(star.scenario.manifest.federation.len(), 3, "star-only = N-1 links");
    }

    #[test]
    fn generated_template_emits_exports() {
        // C5 extension: a generated scenario emits its cross-actor exports so a
        // joiner's `{{space_id}}` resolves + the federation director sees it.
        let mut t = trivial_template("MP-EXPORTS-TEST");
        t.exports = vec![GenExport {
            actor_index: 0,
            command: "s".into(),
            field: "space_id".into(),
            key: "space_id".into(),
        }];
        let generated = ScenarioTemplate::Generated(t)
            .generate(&RoundDial {
                nodes: 1,
                clients: 3,
                ..Default::default()
            })
            .expect("generate with exports");
        assert_eq!(generated.scenario.manifest.exports.len(), 1);
        let exp = &generated.scenario.manifest.exports[0];
        assert_eq!(exp.actor, "a0");
        assert_eq!(exp.key, "space_id");
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
        let fixed = ScenarioTemplate::Fixed(Box::new(seed.scenario.clone()));
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
