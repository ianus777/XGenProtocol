// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! The generic scenario runner (MP-R1-D1 / C1).
//!
//! [`run_scenario`] is the top orchestrator the Round-0 smokes hand-wired (see
//! `tests/c5_mp_c_02.rs`): it generalizes that single flow over **N nodes / N
//! actors + the federation step**, so every R1 scenario is a manifest + batch
//! set rather than bespoke test code. The per-actor [`crate::batch::run_actor`]
//! is reused unchanged; the new code here is the orchestration around it.
//!
//! ## Canonical sequence (design §2 / D1)
//! 1. **Spawn nodes** — one [`ManagedProcess`] per `manifest.nodes`; connect each
//!    node's `.aicontrol`; read its `node_id`; attach an [`EventCollector`]
//!    *before* driving (events are live-only — attach-at-start).
//! 2. **Spawn actors** — one client per batch actor; connect its `.aicontrol`.
//!    (An `injector`-kind actor routes elsewhere — wired in C7.)
//! 3. **Seed federation** — for each `[[federation]]` link, `add-peer` both
//!    directions with **empty** shared-spaces, *before* any identity registers
//!    (so `push_identity_to_peers` replicates a registering identity to the peer).
//! 4. **Drive + direct concurrently** — every batch actor's `run_actor` runs on a
//!    **shared [`Registry`]** (concurrency is required — cross-actor `{{exports}}`
//!    / `[[waits]]` only resolve if producers and consumers run together). A
//!    sibling *director* runs the G-6 federation bootstrap (MP-R1-D1a: wait for
//!    the owner's `space_id`, re-`add-peer` naming the Space, `federation
//!    initiate`) then the `[[clock]]` steps (MP-R1-D3: each gated on its `after`
//!    key, driving the node's `MockClock` over the F3 verbs).
//! 5. **Settle** — a bounded poll-until-stable window for replication to quiesce.
//! 6. **Oracle** — query `members` per actor + read each node's transcript; run
//!    the Space-scoped convergence verdict (MP-R1-D4).
//!
//! ## G-6 federation bootstrap (MP-R1-D1a)
//! Encoded once, in this runner (grounded on `tests/m9_2_f2_add_peer.rs`): seed
//! both directions empty → actors register + the owner creates the Space → re-seed
//! the `from` side naming the Space → `initiate` from `from`. The seam verbs
//! (`federation add-peer` / `federation initiate`) are M9.2 **fenced** — a
//! federated scenario therefore requires a `--features harness-control` node
//! build; the first `add-peer` is the loud build-probe (and a `Mock` dial probes
//! `clock advance 0s` up front).
//!
//! ## Topology authority (R1 vs the sweep)
//! In R1 the **manifest** is authoritative for the explicit topology — the runner
//! spawns one node per `manifest.nodes`, ignoring the dial's `nodes`/`clients`
//! scale fields (those drive the R2/R3 sweep's *generated* dials). The dial here
//! contributes `validate()` + the clock mode (Mock ⇒ harness-control probe).
//!
//! ## As-built note (D-065)
//! The runbook §3 sketches `ScenarioOutcome { verdict, actor_runs, resource }`.
//! As built it also carries `projections` / `transcripts` / `space_id` (the raw
//! materials the C6/C7 adversarial tranches need to compute a *rejection* verdict
//! without re-querying), and `resource` is `Option<ResourceSample>` (sampling is
//! best-effort — it shells out to `Get-Process` and is Windows-only). The peak
//! (max-RSS) live process is reported (the OOM frontier the C2 sweep's CEILING
//! classifier wants). These are additive to the runbook's named fields.

use std::time::Duration;

use futures_util::future::join_all;
use serde_json::json;

use crate::aicontrol::{AicontrolClient, DEFAULT_CONNECT_TIMEOUT};
use crate::batch::{parse_batch_lines, run_actor, ActorRun, BatchLine};
use crate::binloc;
use crate::dial::{ClockMode, RoundDial};
use crate::events::{EventCollector, Filter};
use crate::injector_actor::{run_injector_actor, InjectorRun};
use crate::churn::{event_flood, run_storm, slow_loris, StormPlan};
use crate::liveness::{run_liveness_probe, LivenessReport};
use crate::manifest::{ActorKind, ActorSpec, ChaosKind, ClockOp, Scenario};
use crate::oracle::{convergence_verdict, MembershipProjection, OracleVerdict, Transcript};
use crate::process::{instance_label, ManagedProcess};
use crate::resolve::Registry;
use crate::resource::{sample_process, ResourceSample};
use crate::wire::{Command, Reply};
use crate::Result;
use anyhow::{anyhow, Context};

/// The well-known export key naming a scenario's primary (shared) Space. A
/// federated scenario exports its Space under this key (the §4 authoring
/// contract); the federation director waits on it before naming the Space in the
/// re-seed. Multi-distinct-Space federation is out of MP-R1 scope.
const PRIMARY_SPACE_KEY: &str = "space_id";

/// How long a cross-actor `{{key}}` / federation wait may block.
const RESOLVE_TIMEOUT: Duration = Duration::from_secs(45);

/// The outcome of one scenario run (MP-R1-D1).
///
/// `verdict`/`actor_runs`/`resource` are the runbook's named fields;
/// `projections`/`transcripts`/`space_id` are the additive raw materials the
/// adversarial tranches consume (see the module as-built note).
#[derive(Debug)]
pub struct ScenarioOutcome {
    /// The Space-scoped convergence verdict (best-effort — see `projections`).
    /// Cooperative smokes assert `verdict.pass`; adversarial smokes ignore it and
    /// compute a rejection verdict from `actor_runs` / `transcripts`.
    pub verdict: OracleVerdict,
    /// Per (batch) actor command/reply log — rejection codes live here.
    pub actor_runs: Vec<ActorRun>,
    /// Per injector actor (raw-wire, `kind="injector"`) attack log — the crafted
    /// event ids + the node's `Error` frames (the C7 wire-path captures, D-9).
    pub injector_runs: Vec<InjectorRun>,
    /// Per-actor membership projection of the primary Space (the convergence key).
    pub projections: Vec<MembershipProjection>,
    /// Per-node `.events` transcript (arrival order).
    pub transcripts: Vec<Transcript>,
    /// The scenario's primary Space id, if it was exported.
    pub space_id: Option<String>,
    /// Peak (max-RSS) spawned-process resource sample, best-effort.
    pub resource: Option<ResourceSample>,
    /// MP-R3-D5a — aggregate RSS across **all** spawned processes (sum), best-
    /// effort; the capstone wall is total memory vs the box budget.
    pub aggregate_rss_bytes: Option<u64>,
    /// MP-R3-D5b — `true` if any spawned process exited unexpectedly (OOM /
    /// non-zero) mid-run (the clearest hardware-wall signal).
    pub process_died: bool,
    /// MP-R3-D4b — the during-chaos liveness report, when the dial requested the
    /// probe (`dial.liveness_probe`); `None` otherwise (R1/R2).
    pub liveness: Option<LivenessReport>,
    /// MP-C-16 / M10.5-D1 — per-node hosted-Space ids (each node's
    /// `space list-hosted`, i.e. `home_node == that node`). Lets a migration
    /// witness assert **home_node-flip-on-both** (the D3 / J-370 shape): after an
    /// A→B migration the Space appears in B's list (home flipped to B) and is
    /// absent from A's (home flipped away from A). Best-effort — a node that
    /// answers no `space list-hosted` contributes an empty list.
    pub hosted_by_node: Vec<(String, Vec<String>)>,
}

impl ScenarioOutcome {
    /// MP-C-16 (M10.5-D1) — does `node_label` home `space_id`, per its
    /// `space list-hosted` (`home_node == node_label`)?
    pub fn node_hosts_space(&self, node_label: &str, space_id: &str) -> bool {
        self.hosted_by_node
            .iter()
            .any(|(l, spaces)| l == node_label && spaces.iter().any(|s| s == space_id))
    }
}

/// A live node: its process, control connection, address, and observer.
struct NodeHandle {
    /// Topology label (e.g. `"a"`).
    label: String,
    proc: ManagedProcess,
    ctl: AicontrolClient,
    url: String,
    node_id: String,
    collector: EventCollector,
}

/// A live batch actor: its process, control connection, spec, and parsed batch.
struct ActorHandle {
    spec: ActorSpec,
    proc: ManagedProcess,
    ctl: AicontrolClient,
    lines: Vec<BatchLine>,
}

/// An injector actor (MP-R1-D1 / C7): no client process — its target node(s) as
/// `(label, ws_url)` pairs + parsed attack-directive batch, driven by
/// [`run_injector_actor`] on the shared `Registry` alongside the batch actors.
/// MP-R3-D3: a multi-target injector (`nodes = […]`) lists ≥2 targets for
/// equivocation (MP-A-06); a single-target injector has one.
struct InjectorHandle {
    name: String,
    targets: Vec<(String, String)>,
    lines: Vec<BatchLine>,
}

/// Resolve an injector spec's target `(label, ws_url)` set (MP-R3-D3): the
/// `nodes` list if non-empty, else the single `node`. `label_urls` is the
/// topology's `(label, ws_url)` projection. Pure so it is unit-tested without
/// spawning. Errors on an unknown label.
fn injector_targets(
    spec_node: &str,
    spec_nodes: &[String],
    label_urls: &[(String, String)],
) -> Result<Vec<(String, String)>> {
    let labels: Vec<&str> = if spec_nodes.is_empty() {
        vec![spec_node]
    } else {
        spec_nodes.iter().map(String::as_str).collect()
    };
    labels
        .into_iter()
        .map(|label| {
            label_urls
                .iter()
                .find(|(l, _)| l == label)
                .map(|(l, url)| (l.clone(), url.clone()))
                .ok_or_else(|| anyhow!("injector targets unknown node label `{label}`"))
        })
        .collect()
}

/// One federation link reduced to what the director needs (owned, so it does not
/// re-borrow the node table while it mutates the `from` node's connection).
struct LinkPlan {
    from_idx: usize,
    to_idx: usize,
    /// `(node_id, url)` of the `to` node — the add-peer target on `from`.
    to_peer: (String, String),
    /// `(node_id, url)` of the `from` node — the add-peer target on `to` (the
    /// reverse direction, established by the director for a late link that was
    /// not pre-seeded).
    from_peer: (String, String),
    /// MP-R2-D5: if set, this is a **late** link — not pre-seeded; the director
    /// establishes it (both directions, naming the Space) after `after` fires.
    after: Option<String>,
}

/// One clock-control step reduced to a target node index + the F3 verb (MP-R1-D3).
struct ClockPlan {
    node_idx: usize,
    op: ClockOp,
    value: String,
    after: Option<String>,
    publishes: Option<String>,
}

/// One Space-migration step reduced to node indices (MP-R2 C6c). The director
/// resolves the Space id from the exported `space_key` + the destination node's
/// id/url from `to_idx`, then fires `migration initiate` on `from_idx`.
struct MigrationPlan {
    from_idx: usize,
    to_idx: usize,
    space_key: String,
    after: Option<String>,
}

/// One **director** chaos step (MP-R3-D4a) — a relationship-level
/// `Partition`/`Heal` between two endpoints, reduced to node indices + peer
/// `(node_id, url)` pairs (so the action does not re-borrow the node table while
/// it mutates an endpoint's connection). `Partition` = `federation defederate`
/// both directions; `Heal` = `add-peer` (naming the Space) + `initiate`.
struct ChaosPlan {
    kind: ChaosKind,
    a_idx: usize,
    b_idx: usize,
    a_peer: (String, String),
    b_peer: (String, String),
    after: Option<String>,
    publishes: Option<String>,
}

/// One **raw-WS** chaos step (MP-R3-D4a) — a `Flood`/`Storm`/`SlowLoris` load
/// driven by the parallel chaos task against a single target node's `ws://`
/// (no node-conn borrow). `count`/`hold` are the load knobs.
struct RawWsChaos {
    kind: ChaosKind,
    url: String,
    after: Option<String>,
    publishes: Option<String>,
    count: usize,
    hold: Duration,
}

/// Partition a manifest chaos list into (director-step indices, raw-WS indices)
/// by kind (MP-R3-D4a). Pure — unit-tested without spawning. `Partition`/`Heal`
/// are director (node-conn) actions; `Flood`/`Storm`/`SlowLoris` are raw-WS.
fn partition_chaos_steps(steps: &[crate::manifest::ChaosStep]) -> (Vec<usize>, Vec<usize>) {
    let mut director = Vec::new();
    let mut raw_ws = Vec::new();
    for (i, s) in steps.iter().enumerate() {
        if s.kind.is_director() {
            director.push(i);
        } else {
            raw_ws.push(i);
        }
    }
    (director, raw_ws)
}

/// Run a scenario end-to-end against freshly-spawned real binaries, returning the
/// convergence verdict + the raw oracle materials.
///
/// **Heavy** — spawns real `xgen-node`/`xgen-client` processes. Callers are
/// `#[ignore]` / out-of-band. A scenario with `[[federation]]` links requires a
/// `--features harness-control` node build (the seam verbs are fenced).
pub async fn run_scenario(scenario: &Scenario, dial: &RoundDial) -> Result<ScenarioOutcome> {
    dial.validate()?;
    let m = &scenario.manifest;
    let bins = binloc::locate()?;

    // ── 1. Spawn nodes, connect control, read node_id, attach observer ───────
    let mut nodes: Vec<NodeHandle> = Vec::with_capacity(m.nodes.len());
    for spec in &m.nodes {
        let label = instance_label(&m.scenario, &format!("node-{}", spec.label));
        let proc =
            ManagedProcess::init_and_spawn_node(&bins, &label, spec.port, spec.local, dial.worker_threads)
                .with_context(|| format!("spawning node `{}`", spec.label))?;
        let mut ctl = AicontrolClient::connect(&proc.aicontrol_pipe, DEFAULT_CONNECT_TIMEOUT)
            .await
            .with_context(|| format!("connecting node `{}` aicontrol", spec.label))?;
        let state = ctl
            .send(&Command::new("state"))
            .await
            .with_context(|| format!("node `{}` state", spec.label))?;
        let node_id = state
            .data_str("node_id")
            .ok_or_else(|| anyhow!("node `{}` state had no node_id: {state:?}", spec.label))?
            .to_string();
        let collector = EventCollector::start(
            &spec.label,
            &format!("{}.events", proc.aicontrol_pipe),
            Filter::all(),
        )
        .await
        .with_context(|| format!("attaching collector to node `{}`", spec.label))?;
        nodes.push(NodeHandle {
            label: spec.label.clone(),
            url: format!("ws://127.0.0.1:{}/xgen", spec.port),
            node_id,
            proc,
            ctl,
            collector,
        });
    }

    // The harness-control build-probe for a Mock dial (federated scenarios also
    // probe implicitly via the first fenced `add-peer`).
    if dial.clock == ClockMode::Mock {
        if let Some(first) = nodes.first_mut() {
            probe_harness_control(&mut first.ctl).await?;
        }
    }

    // ── 2. Spawn batch actors (clients) + collect injector actors (raw-wire) ──
    // An `injector`-kind actor (MP-R1-D1 / C7) spawns no client process — it
    // speaks the transport directly and recvs the node's `Error` frame. It is
    // driven concurrently with the batch actors on the shared `Registry` (so it
    // can import a batch actor's exported value, e.g. MP-A-16's target Space).
    let mut actors: Vec<ActorHandle> = Vec::new();
    let mut injectors: Vec<InjectorHandle> = Vec::new();
    for spec in &m.actors {
        let lines = parse_batch_lines(
            &std::fs::read_to_string(scenario.batch_path(spec))
                .with_context(|| format!("reading batch for `{}`", spec.name))?,
        )?;
        if spec.kind == ActorKind::Injector {
            // MP-R3-D3: resolve the injector's target set (multi-target if
            // `nodes = […]`, else the single `node`).
            let label_urls: Vec<(String, String)> =
                nodes.iter().map(|n| (n.label.clone(), n.url.clone())).collect();
            let targets = injector_targets(&spec.node, &spec.nodes, &label_urls)
                .with_context(|| format!("injector `{}` targets", spec.name))?;
            injectors.push(InjectorHandle {
                name: spec.name.clone(),
                targets,
                lines,
            });
            continue;
        }
        let node = node_by_label(&nodes, &spec.node)?;
        let label = instance_label(&m.scenario, &spec.name);
        let proc = ManagedProcess::init_and_spawn_client(
            &bins,
            &label,
            &node.url,
            spec.ai_mode,
            dial.worker_threads,
        )
        .with_context(|| format!("spawning client `{}`", spec.name))?;
        let ctl = AicontrolClient::connect(&proc.aicontrol_pipe, DEFAULT_CONNECT_TIMEOUT)
            .await
            .with_context(|| format!("connecting client `{}` aicontrol", spec.name))?;
        actors.push(ActorHandle {
            spec: spec.clone(),
            proc,
            ctl,
            lines,
        });
    }

    // ── 3. Seed federation (both directions, empty spaces) BEFORE the drive ──
    let node_addrs: Vec<(String, String)> = nodes
        .iter()
        .map(|n| (n.node_id.clone(), n.url.clone()))
        .collect();
    let mut link_plans: Vec<LinkPlan> = Vec::with_capacity(m.federation.len());
    for link in &m.federation {
        let from_idx = node_idx(&nodes, &link.from)?;
        let to_idx = node_idx(&nodes, &link.to)?;
        let to_peer = node_addrs[to_idx].clone();
        let from_peer = node_addrs[from_idx].clone();
        // MP-R2-D5: a **late** link (`after` set) is NOT pre-seeded here — the
        // director establishes it after its gate fires, so the node federates
        // *after* the Space has history / has been clock-aged. An early link
        // (the G-6 bootstrap, every R1 scenario) seeds both directions now.
        if link.after.is_none() {
            node_add_peer(&mut nodes[from_idx].ctl, &to_peer.0, &to_peer.1, &[])
                .await
                .with_context(|| format!("seed add-peer {} → {}", link.from, link.to))?;
            node_add_peer(&mut nodes[to_idx].ctl, &from_peer.0, &from_peer.1, &[])
                .await
                .with_context(|| format!("seed add-peer {} → {}", link.to, link.from))?;
        }
        link_plans.push(LinkPlan {
            from_idx,
            to_idx,
            to_peer,
            from_peer,
            after: link.after.clone(),
        });
    }

    // Clock-control steps (MP-R1-D3) reduced to node indices.
    let mut clock_plans: Vec<ClockPlan> = Vec::with_capacity(m.clock.len());
    for step in &m.clock {
        clock_plans.push(ClockPlan {
            node_idx: node_idx(&nodes, &step.node)?,
            op: step.op,
            value: step.value.clone(),
            after: step.after.clone(),
            publishes: step.publishes.clone(),
        });
    }

    // Migration steps (MP-R2 C6c) reduced to node indices.
    let mut migration_plans: Vec<MigrationPlan> = Vec::with_capacity(m.migration.len());
    for step in &m.migration {
        migration_plans.push(MigrationPlan {
            from_idx: node_idx(&nodes, &step.from)?,
            to_idx: node_idx(&nodes, &step.to)?,
            space_key: step.space_key.clone(),
            after: step.after.clone(),
        });
    }

    // Chaos steps (MP-R3-D4a) partitioned into director (node-conn) + raw-WS sets.
    let (chaos_director_idx, chaos_rawws_idx) = partition_chaos_steps(&m.chaos);
    let mut chaos_plans: Vec<ChaosPlan> = Vec::with_capacity(chaos_director_idx.len());
    for &i in &chaos_director_idx {
        let step = &m.chaos[i];
        // Validated arity (manifest): a director chaos step has 2 endpoints.
        let a_idx = node_idx(&nodes, &step.nodes[0])?;
        let b_idx = node_idx(&nodes, &step.nodes[1])?;
        chaos_plans.push(ChaosPlan {
            kind: step.kind,
            a_idx,
            b_idx,
            a_peer: node_addrs[a_idx].clone(),
            b_peer: node_addrs[b_idx].clone(),
            after: step.after.clone(),
            publishes: step.publishes.clone(),
        });
    }
    let mut rawws_chaos: Vec<RawWsChaos> = Vec::with_capacity(chaos_rawws_idx.len());
    for &i in &chaos_rawws_idx {
        let step = &m.chaos[i];
        let target_idx = node_idx(&nodes, &step.nodes[0])?;
        rawws_chaos.push(RawWsChaos {
            kind: step.kind,
            url: nodes[target_idx].url.clone(),
            after: step.after.clone(),
            publishes: step.publishes.clone(),
            count: step.count.unwrap_or(100),
            hold: Duration::from_millis(step.hold_ms.unwrap_or(0)),
        });
    }

    // Node `(label, aicontrol_pipe)` pairs for the liveness probe (R3-D4b) —
    // captured before the director's `&mut nodes` borrow.
    let node_pipes: Vec<(String, String)> = nodes
        .iter()
        .map(|n| (n.label.clone(), n.proc.aicontrol_pipe.clone()))
        .collect();

    // ── 4. Drive batch actors + injector actors concurrently + the director ──
    let registry = Registry::new();
    let batch_names: Vec<String> = actors.iter().map(|a| a.spec.name.clone()).collect();
    let drive = join_all(actors.iter_mut().map(|a| {
        run_actor(
            &a.spec.name,
            &a.lines,
            &mut a.ctl,
            m,
            &registry,
            RESOLVE_TIMEOUT,
        )
    }));
    let inj_drive = join_all(injectors.iter().map(|inj| {
        run_injector_actor(&inj.name, &inj.targets, &inj.lines, &registry, RESOLVE_TIMEOUT)
    }));
    let direct = run_director(
        &mut nodes,
        &link_plans,
        &clock_plans,
        &migration_plans,
        &chaos_plans,
        &registry,
        RESOLVE_TIMEOUT,
    );
    // MP-R3-D4a — the parallel chaos task (raw-WS load) runs alongside the drive +
    // director on the shared registry; it uses no node connection, so it composes
    // with the director's single-owner `&mut nodes`.
    let chaos_drive = run_chaos(&rawws_chaos, &registry, RESOLVE_TIMEOUT);
    // MP-R3-D4b — the during-chaos liveness probe (opens its OWN aicontrol
    // connections per node — does not borrow the director's `&mut nodes`).
    // Bounded by a fixed sample budget so the `join!` returns; a no-op (yields
    // `None`) when the dial did not request it.
    const LIVENESS_SAMPLES: usize = 12;
    const LIVENESS_INTERVAL: Duration = Duration::from_millis(500);
    let probe = async {
        if dial.liveness_probe {
            Some(run_liveness_probe(&node_pipes, LIVENESS_SAMPLES, LIVENESS_INTERVAL).await)
        } else {
            None
        }
    };
    let (drive_results, inj_results, direct_result, chaos_result, liveness) =
        tokio::join!(drive, inj_drive, direct, chaos_drive, probe);
    direct_result.context("scenario director")?;
    chaos_result.context("scenario chaos task")?;
    let mut actor_runs: Vec<ActorRun> = Vec::with_capacity(drive_results.len());
    for (name, res) in batch_names.iter().zip(drive_results) {
        actor_runs.push(res.with_context(|| format!("driving actor `{}`", name))?);
    }
    let mut injector_runs: Vec<InjectorRun> = Vec::with_capacity(inj_results.len());
    for (inj, res) in injectors.iter().zip(inj_results) {
        injector_runs.push(res.with_context(|| format!("driving injector `{}`", inj.name))?);
    }

    // ── 5. Settle (bounded poll-until-stable; elastic ceiling, R3-D4c) ───────
    settle(&nodes, dial.settle_max()).await;

    // ── 6. Oracle: per-actor membership projection + per-node transcript ─────
    let space_id = registry.get(PRIMARY_SPACE_KEY).await;
    let mut projections: Vec<MembershipProjection> = Vec::new();
    if let Some(space) = space_id.as_deref() {
        for a in actors.iter_mut() {
            let mut cmd = Command::new("members");
            cmd.args.insert("space".into(), json!(space));
            // Best-effort: an actor that never joined may not project a view.
            if let Ok(reply) = a.ctl.send(&cmd).await {
                if let Some(data) = reply.data() {
                    if let Some(proj) = MembershipProjection::from_members_data(
                        &format!("{}-view", a.spec.name),
                        data,
                    )
                    // MP-R3-D4d — tag with the topology node so node_convergence_verdict
                    // can collapse per node (churn-at-scale: a mid-leave actor's absence
                    // can't break ≥2 when a stable reader covers its node).
                    .map(|p| p.with_node_label(a.spec.node.clone()))
                    {
                        projections.push(proj);
                    }
                }
            }
        }
    }

    let mut transcripts: Vec<Transcript> = Vec::with_capacity(nodes.len());
    for n in &nodes {
        transcripts.push(Transcript::from_values(&n.label, &n.collector.snapshot().await));
    }

    let verdict = match space_id.as_deref() {
        Some(space) => convergence_verdict(&projections, &transcripts, space),
        None => OracleVerdict::fail(format!(
            "no `{PRIMARY_SPACE_KEY}` exported — Space-scoped oracle not applicable"
        )),
    };

    // MP-C-16 (M10.5-D1) — per-node hosted-Space query for the flip-on-both
    // witness. `space list-hosted` returns the Spaces a node homes
    // (`home_node == self`, admin_ops.rs); a migration cutover flips a Space's
    // `home_node`, so it appears in the destination's list and leaves the
    // source's. Best-effort: a node that errors / answers nothing contributes an
    // empty list. Mirrors the `members` query pattern above.
    let mut hosted_by_node: Vec<(String, Vec<String>)> = Vec::with_capacity(nodes.len());
    for n in nodes.iter_mut() {
        let mut hosted: Vec<String> = Vec::new();
        if let Ok(reply) = n.ctl.send(&Command::new("space list-hosted")).await {
            if let Some(arr) = reply
                .data()
                .and_then(|d| d.get("spaces"))
                .and_then(|v| v.as_array())
            {
                for s in arr {
                    if let Some(id) = s.get("space_id").and_then(|v| v.as_str()) {
                        hosted.push(id.to_string());
                    }
                }
            }
        }
        hosted_by_node.push((n.label.clone(), hosted));
    }

    // MP-R3-D5b — did any spawned process exit unexpectedly (OOM / non-zero)?
    // Checked before the immutable resource sample (it needs `&mut`).
    let process_died = any_process_died(&mut nodes, &mut actors);
    // MP-R3-D5a — peak + aggregate RSS across all spawned processes (best-effort).
    let (resource, aggregate_rss_bytes) = aggregate_resource(&nodes, &actors);

    Ok(ScenarioOutcome {
        verdict,
        actor_runs,
        injector_runs,
        projections,
        transcripts,
        space_id,
        resource,
        aggregate_rss_bytes,
        process_died,
        liveness,
        hosted_by_node,
    })
}

/// One director step, identified by kind + index into the respective plan slice
/// (MP-F10 / F10-D1). `Copy` (just indices) so the ordered worklist is cheap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DirectorStep {
    Link(usize),
    Clock(usize),
    Migration(usize),
    /// MP-R3-D4a — a relationship-level partition/heal chaos step (node-conn).
    Chaos(usize),
}

/// F10-D1 — order the director steps so any step whose `after` gate is a key that
/// another director step **publishes** (only clock steps publish) runs AFTER its
/// publisher. This is the fix for the MP-F10 deadlock: the fixed
/// federation→clock→migration phase order blocks a federation link gated on a key
/// a *later* clock step publishes (`clock_advanced`), so the federation phase
/// waits forever on a key the clock phase never reaches to publish.
///
/// **External** `after` keys (published by the concurrent actor drive — e.g.
/// `history_ready`, `bob_join_ready` — not by any director step) carry no internal
/// edge; they are waited on at runtime, unordered. The ordering is **stable and
/// phase-biased**: with no publish→wait edge it returns the original phase order
/// (links → clocks → migrations, manifest order within each), so cooperative
/// scenarios are unperturbed.
///
/// Pure (no async, no node table) so it is unit-tested directly. Returns an error
/// on a dependency cycle (a clock-published key waited on circularly).
fn order_director_steps(
    links: &[LinkPlan],
    clocks: &[ClockPlan],
    migrations: &[MigrationPlan],
    chaos: &[ChaosPlan],
) -> Result<Vec<DirectorStep>> {
    // Default phase-ordered worklist: links, clocks, migrations, then chaos.
    let all: Vec<DirectorStep> = (0..links.len())
        .map(DirectorStep::Link)
        .chain((0..clocks.len()).map(DirectorStep::Clock))
        .chain((0..migrations.len()).map(DirectorStep::Migration))
        .chain((0..chaos.len()).map(DirectorStep::Chaos))
        .collect();

    let step_after = |s: DirectorStep| -> Option<&str> {
        match s {
            DirectorStep::Link(i) => links[i].after.as_deref(),
            DirectorStep::Clock(i) => clocks[i].after.as_deref(),
            DirectorStep::Migration(i) => migrations[i].after.as_deref(),
            DirectorStep::Chaos(i) => chaos[i].after.as_deref(),
        }
    };

    // Map each internally-published key → its position in `all`. Clock steps and
    // chaos steps (R3-D4a) publish; a step waiting on a published key is ordered
    // after its publisher (the F10-D1 edge — e.g. a Heal after the Partition it
    // depends on, or a fed link after the clock that ages the Space).
    let mut published_at: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for (pos, s) in all.iter().enumerate() {
        match s {
            DirectorStep::Clock(i) => {
                if let Some(k) = clocks[*i].publishes.as_deref() {
                    published_at.insert(k, pos);
                }
            }
            DirectorStep::Chaos(i) => {
                if let Some(k) = chaos[*i].publishes.as_deref() {
                    published_at.insert(k, pos);
                }
            }
            _ => {}
        }
    }

    // Stable, phase-biased topological order: emit the first not-yet-emitted step
    // (in `all` order) whose internal predecessor (if any) is already emitted.
    let mut emitted = vec![false; all.len()];
    let mut order: Vec<DirectorStep> = Vec::with_capacity(all.len());
    while order.len() < all.len() {
        let mut progress = false;
        for (pos, s) in all.iter().enumerate() {
            if emitted[pos] {
                continue;
            }
            let ready = match step_after(*s) {
                None => true,
                // External key (no director step publishes it) → ready; the runtime
                // wait_for blocks on the concurrent actor drive.
                Some(k) => match published_at.get(k) {
                    None => true,
                    Some(&pub_pos) => emitted[pub_pos],
                },
            };
            if ready {
                order.push(*s);
                emitted[pos] = true;
                progress = true;
            }
        }
        if !progress {
            return Err(anyhow!(
                "director step dependency cycle: a clock-published key is waited on circularly"
            ));
        }
    }
    Ok(order)
}

/// The scenario director — a sibling of the actor drive that owns the node
/// connections during the concurrent phase, executing the federation / clock /
/// migration steps over the shared registry.
///
/// **F10-D1 (dependency-ordered, single-owner):** steps run in
/// [`order_director_steps`] order — the original federation → clock → migration
/// phase order EXCEPT a step gated (`after`) on a key another step publishes (only
/// clock steps publish) runs after its publisher. A single sequential director
/// still owns `&mut nodes` (no borrow refactor). Each step still `wait_for`s its
/// `after` at runtime (the ordering makes an internally-published gate resolvable;
/// an external gate blocks on the concurrent actor drive). All step kinds are
/// no-ops when empty.
async fn run_director(
    nodes: &mut [NodeHandle],
    links: &[LinkPlan],
    clocks: &[ClockPlan],
    migrations: &[MigrationPlan],
    chaos: &[ChaosPlan],
    registry: &Registry,
    timeout: Duration,
) -> Result<()> {
    let order = order_director_steps(links, clocks, migrations, chaos)?;

    for s in order {
        match s {
            DirectorStep::Link(i) => {
                let plan = &links[i];
                let space = registry
                    .wait_for(PRIMARY_SPACE_KEY, timeout)
                    .await
                    .context("director waiting for the exported Space id")?;
                match &plan.after {
                    // MP-R2-D5 late link: wait for its gate (e.g. the clock-aged
                    // key — now resolvable because F10-D1 ordered the publishing
                    // clock step before this link), then establish BOTH directions
                    // naming the Space + initiate (this link was NOT pre-seeded).
                    Some(after) => {
                        registry
                            .wait_for(after, timeout)
                            .await
                            .with_context(|| format!("late-federation link waiting on `{{{{{after}}}}}`"))?;
                        node_add_peer(&mut nodes[plan.from_idx].ctl, &plan.to_peer.0, &plan.to_peer.1, &[space.as_str()])
                            .await
                            .context("late-fed add-peer from→to (named)")?;
                        node_add_peer(&mut nodes[plan.to_idx].ctl, &plan.from_peer.0, &plan.from_peer.1, &[space.as_str()])
                            .await
                            .context("late-fed add-peer to→from (named)")?;
                        node_initiate(&mut nodes[plan.from_idx].ctl, &plan.to_peer.0)
                            .await
                            .context("late-fed federation initiate")?;
                    }
                    // Normal early link (pre-seeded empty both directions): re-seed
                    // `from` naming the Space + initiate — the G-6 bootstrap tail.
                    None => {
                        node_add_peer(&mut nodes[plan.from_idx].ctl, &plan.to_peer.0, &plan.to_peer.1, &[space.as_str()])
                            .await
                            .context("re-seed add-peer naming the Space")?;
                        node_initiate(&mut nodes[plan.from_idx].ctl, &plan.to_peer.0)
                            .await
                            .context("federation initiate")?;
                    }
                }
            }
            DirectorStep::Clock(i) => {
                let step = &clocks[i];
                if let Some(after) = &step.after {
                    registry
                        .wait_for(after, timeout)
                        .await
                        .with_context(|| format!("clock step waiting on `{{{{{after}}}}}`"))?;
                }
                node_clock(&mut nodes[step.node_idx].ctl, step.op, &step.value)
                    .await
                    .with_context(|| format!("clock {:?} {}", step.op, step.value))?;
                // Publish the clock-completion key (if named) so a later director
                // step (F10-D1) OR an actor can resolve its `after`/`[[waits]]`.
                if let Some(key) = &step.publishes {
                    registry.publish(key.clone(), step.value.clone()).await;
                }
            }
            DirectorStep::Migration(i) => {
                let m = &migrations[i];
                if let Some(after) = &m.after {
                    registry
                        .wait_for(after, timeout)
                        .await
                        .with_context(|| format!("migration step waiting on `{{{{{after}}}}}`"))?;
                }
                let space_id = registry
                    .wait_for(&m.space_key, timeout)
                    .await
                    .with_context(|| format!("migration resolving Space `{{{{{}}}}}`", m.space_key))?;
                // Resolve the destination id/url before borrowing the `from` ctl mutably.
                let dest_id = nodes[m.to_idx].node_id.clone();
                let dest_url = nodes[m.to_idx].url.clone();
                node_migrate(&mut nodes[m.from_idx].ctl, &space_id, &dest_id, &dest_url)
                    .await
                    .context("migration initiate")?;
            }
            DirectorStep::Chaos(i) => {
                let plan = &chaos[i];
                if let Some(after) = &plan.after {
                    registry
                        .wait_for(after, timeout)
                        .await
                        .with_context(|| format!("chaos step waiting on `{{{{{after}}}}}`"))?;
                }
                match plan.kind {
                    // R3-D2 relationship-level partition: defederate both
                    // directions (each endpoint drops the other peer).
                    ChaosKind::Partition => {
                        node_defederate(&mut nodes[plan.a_idx].ctl, &plan.b_peer.0)
                            .await
                            .context("chaos partition defederate a→b")?;
                        node_defederate(&mut nodes[plan.b_idx].ctl, &plan.a_peer.0)
                            .await
                            .context("chaos partition defederate b→a")?;
                    }
                    // Heal: re-establish federation (the late-establish catch-up
                    // that rides MP-F11) — add-peer naming the Space both ways +
                    // initiate from a.
                    ChaosKind::Heal => {
                        let space = registry
                            .wait_for(PRIMARY_SPACE_KEY, timeout)
                            .await
                            .context("chaos heal resolving the Space id")?;
                        node_add_peer(&mut nodes[plan.a_idx].ctl, &plan.b_peer.0, &plan.b_peer.1, &[space.as_str()])
                            .await
                            .context("chaos heal add-peer a→b")?;
                        node_add_peer(&mut nodes[plan.b_idx].ctl, &plan.a_peer.0, &plan.a_peer.1, &[space.as_str()])
                            .await
                            .context("chaos heal add-peer b→a")?;
                        node_initiate(&mut nodes[plan.a_idx].ctl, &plan.b_peer.0)
                            .await
                            .context("chaos heal initiate")?;
                    }
                    other => {
                        return Err(anyhow!(
                            "non-director chaos kind {other:?} routed to the director (a bug)"
                        ))
                    }
                }
                if let Some(key) = &plan.publishes {
                    registry.publish(key.clone(), "done").await;
                }
            }
        }
    }
    Ok(())
}

/// The parallel chaos task (MP-R3-D4a) — drives the **raw-WS** load steps
/// (`Flood`/`Storm`/`SlowLoris`) concurrently with the actor drive + the director,
/// gating each on its `after` key and publishing its `publishes` key on
/// completion (the chaos timeline; e.g. a flood-during-partition that the director
/// Heal gates on). Uses [`crate::churn`] (raw `connect_url`) — **no node-conn
/// borrow**, so it composes with the director's single-owner `&mut nodes`.
async fn run_chaos(steps: &[RawWsChaos], registry: &Registry, timeout: Duration) -> Result<()> {
    for s in steps {
        if let Some(after) = &s.after {
            registry
                .wait_for(after, timeout)
                .await
                .with_context(|| format!("chaos load waiting on `{{{{{after}}}}}`"))?;
        }
        match s.kind {
            ChaosKind::Flood => {
                event_flood(&s.url, s.count, s.hold)
                    .await
                    .context("chaos flood")?;
            }
            ChaosKind::Storm => {
                run_storm(
                    &s.url,
                    StormPlan {
                        cycles: s.count,
                        conns_per_cycle: 8,
                    },
                )
                .await
                .context("chaos storm")?;
            }
            ChaosKind::SlowLoris => {
                let held = slow_loris(&s.url, s.count, s.hold)
                    .await
                    .context("chaos slow-loris")?;
                drop(held); // release the held connections after the hold window
            }
            other => {
                return Err(anyhow!(
                    "director chaos kind {other:?} routed to the raw-WS task (a bug)"
                ))
            }
        }
        if let Some(key) = &s.publishes {
            registry.publish(key.clone(), "done").await;
        }
    }
    Ok(())
}

/// Bounded poll-until-stable settle: wait for the total observed-event count to
/// stop changing across two consecutive intervals (replication quiescence),
/// capped at `max` so a stuck run still returns. Single-node scenarios settle in
/// ~1s. MP-R3-D4c — `max` is the elastic ceiling (`dial.settle_max()`); the
/// stable-for-2 termination is unchanged, so a quiesced run returns early.
async fn settle(nodes: &[NodeHandle], max: Duration) {
    const INTERVAL: Duration = Duration::from_millis(400);
    // A small grace so the last fan-out of the drive lands before the first poll.
    tokio::time::sleep(Duration::from_millis(600)).await;
    let deadline = tokio::time::Instant::now() + max;
    let mut prev = total_events(nodes).await;
    let mut stable_rounds = 0u8;
    while tokio::time::Instant::now() < deadline {
        tokio::time::sleep(INTERVAL).await;
        let now = total_events(nodes).await;
        if now == prev {
            stable_rounds += 1;
            if stable_rounds >= 2 {
                return;
            }
        } else {
            stable_rounds = 0;
            prev = now;
        }
    }
}

async fn total_events(nodes: &[NodeHandle]) -> usize {
    let mut total = 0;
    for n in nodes {
        total += n.collector.len().await;
    }
    total
}

/// Sample every spawned process's RSS/threads (best-effort) → the **peak**
/// (max-RSS, the OOM frontier) + the **aggregate** RSS sum (MP-R3-D5a — the
/// capstone wall is total memory vs the box budget). `(None, None)` if nothing
/// sampled.
fn aggregate_resource(
    nodes: &[NodeHandle],
    actors: &[ActorHandle],
) -> (Option<ResourceSample>, Option<u64>) {
    let pids = nodes
        .iter()
        .map(|n| n.proc.pid())
        .chain(actors.iter().map(|a| a.proc.pid()));
    let samples: Vec<ResourceSample> = pids.filter_map(|pid| sample_process(pid).ok()).collect();
    if samples.is_empty() {
        return (None, None);
    }
    let total: u64 = samples.iter().map(|s| s.rss_bytes).sum();
    let peak = samples.into_iter().max_by_key(|s| s.rss_bytes);
    (peak, Some(total))
}

/// MP-R3-D5b — did any spawned process exit unexpectedly (OOM / non-zero) before
/// teardown? A still-running process reports no exit (not died); a process that
/// exited non-`success()` is the clearest hardware-wall signal.
fn any_process_died(nodes: &mut [NodeHandle], actors: &mut [ActorHandle]) -> bool {
    let node_died = nodes
        .iter_mut()
        .any(|n| matches!(n.proc.try_exit_status(), Some(s) if !s.success()));
    let actor_died = actors
        .iter_mut()
        .any(|a| matches!(a.proc.try_exit_status(), Some(s) if !s.success()));
    node_died || actor_died
}

/// Send `federation add-peer` and require an OK reply (loud on a non-harness
/// build, mirroring the F2 smoke).
async fn node_add_peer(
    ctl: &mut AicontrolClient,
    peer_id: &str,
    peer_url: &str,
    spaces: &[&str],
) -> Result<()> {
    let mut c = Command::new("federation add-peer");
    c.args.insert("node_id".into(), json!(peer_id));
    c.args.insert("url".into(), json!(peer_url));
    c.args.insert("spaces".into(), json!(spaces));
    require_ok(ctl.send(&c).await?, "federation add-peer")
}

/// Send `federation initiate` and require an OK reply.
async fn node_initiate(ctl: &mut AicontrolClient, peer_id: &str) -> Result<()> {
    let mut c = Command::new("federation initiate");
    c.args.insert("peer_node_id".into(), json!(peer_id));
    require_ok(ctl.send(&c).await?, "federation initiate")
}

/// Send `federation defederate` and require an OK reply (MP-R3-D4a — the
/// relationship-level partition primitive; the verb is unfenced, aicontrol.rs).
async fn node_defederate(ctl: &mut AicontrolClient, peer_id: &str) -> Result<()> {
    let mut c = Command::new("federation defederate");
    c.args.insert("peer_node_id".into(), json!(peer_id));
    require_ok(ctl.send(&c).await?, "federation defederate")
}

/// Drive a node's injected `MockClock` (MP-R1-D3 / F3) — `clock advance
/// <duration>` or `clock set <rfc3339>`. Fenced (requires `--features
/// harness-control`); loud on a default build.
async fn node_clock(ctl: &mut AicontrolClient, op: ClockOp, value: &str) -> Result<()> {
    let (verb, arg) = match op {
        ClockOp::Advance => ("clock advance", "duration"),
        ClockOp::Set => ("clock set", "timestamp"),
    };
    let mut c = Command::new(verb);
    c.args.insert(arg.into(), json!(value));
    require_ok(ctl.send(&c).await?, verb)
}

/// Fire `migration initiate <space_id> --destination-id --destination-url` on the
/// source node (MP-R2 C6c / Arc F AF-D7). The migration runs detached on the node
/// (propose → transfer → verify → cutover); this returns once the verb is accepted.
async fn node_migrate(
    ctl: &mut AicontrolClient,
    space_id: &str,
    dest_id: &str,
    dest_url: &str,
) -> Result<()> {
    let mut c = Command::new("migration initiate");
    c.args.insert("space_id".into(), json!(space_id));
    c.args.insert("destination_id".into(), json!(dest_id));
    c.args.insert("destination_url".into(), json!(dest_url));
    require_ok(ctl.send(&c).await?, "migration initiate")
}

/// Probe the node build for `--features harness-control` via a no-op
/// `clock advance 0s` (a Mock dial requires the fenced clock seam).
async fn probe_harness_control(ctl: &mut AicontrolClient) -> Result<()> {
    let mut c = Command::new("clock advance");
    c.args.insert("duration".into(), json!("0s"));
    require_ok(ctl.send(&c).await?, "clock advance (harness-control probe)")
}

/// Turn a non-OK fenced-verb reply into a loud error with the harness-control
/// hint (matching the F2/F3 smoke assertion message).
fn require_ok(reply: Reply, what: &str) -> Result<()> {
    if reply.is_ok() {
        Ok(())
    } else {
        Err(anyhow!(
            "{what} failed — did you build the node `--features harness-control`? reply={reply:?}"
        ))
    }
}

fn node_idx(nodes: &[NodeHandle], label: &str) -> Result<usize> {
    nodes
        .iter()
        .position(|n| n.label == label)
        .ok_or_else(|| anyhow!("federation link references unknown node `{label}`"))
}

fn node_by_label<'a>(nodes: &'a [NodeHandle], label: &str) -> Result<&'a NodeHandle> {
    nodes
        .iter()
        .find(|n| n.label == label)
        .ok_or_else(|| anyhow!("actor references unknown node `{label}`"))
}

// ── F10-D1 director-ordering unit tests (MP-F10) ─────────────────────────────
#[cfg(test)]
mod director_order_tests {
    use super::{order_director_steps, ChaosPlan, ClockPlan, DirectorStep, LinkPlan, MigrationPlan};
    use crate::manifest::{ChaosKind, ClockOp};

    fn link(from_idx: usize, to_idx: usize, after: Option<&str>) -> LinkPlan {
        LinkPlan {
            from_idx,
            to_idx,
            to_peer: ("peer".into(), "ws://to".into()),
            from_peer: ("peer".into(), "ws://from".into()),
            after: after.map(String::from),
        }
    }
    fn clock(node_idx: usize, after: Option<&str>, publishes: Option<&str>) -> ClockPlan {
        ClockPlan {
            node_idx,
            op: ClockOp::Advance,
            value: "2d".into(),
            after: after.map(String::from),
            publishes: publishes.map(String::from),
        }
    }
    fn chaos(kind: ChaosKind, after: Option<&str>, publishes: Option<&str>) -> ChaosPlan {
        ChaosPlan {
            kind,
            a_idx: 0,
            b_idx: 1,
            a_peer: ("a".into(), "ws://a".into()),
            b_peer: ("b".into(), "ws://b".into()),
            after: after.map(String::from),
            publishes: publishes.map(String::from),
        }
    }

    fn pos(order: &[DirectorStep], step: DirectorStep) -> usize {
        order.iter().position(|s| *s == step).expect("step in order")
    }

    /// THE F10-D1 witness (RED-on-revert = the fixed-order deadlock): a federation
    /// link gated on a key a clock step publishes is ordered AFTER that clock step
    /// — so at runtime its `wait_for(clock_advanced)` is resolvable (the fixed
    /// federation→clock→migration order would run the link first → deadlock).
    /// Mirrors MP-A-01(ii): A↔B early, A→C late gated on `clock_advanced`.
    #[test]
    fn director_orders_fed_link_after_its_clock_gate() {
        let links = vec![
            link(0, 1, None),                  // early A→B
            link(0, 2, Some("clock_advanced")), // late A→C, gated on the clock
        ];
        let clocks = vec![
            // clock on A: gated on the EXTERNAL bob_join_ready, publishes clock_advanced.
            clock(0, Some("bob_join_ready"), Some("clock_advanced")),
        ];
        let order = order_director_steps(&links, &clocks, &[], &[]).expect("orders");

        // Positive order assertion: the publishing clock precedes the waiting link.
        assert!(
            pos(&order, DirectorStep::Clock(0)) < pos(&order, DirectorStep::Link(1)),
            "clock publishing `clock_advanced` must be ordered BEFORE the link waiting on it; got {order:?}"
        );
        // The early, ungated link keeps its front position (no-edge stays put).
        assert_eq!(order[0], DirectorStep::Link(0), "ungated early link stays first");
    }

    /// MP-R3-D4a — the F10-D1 ordering extends to `DirectorStep::Chaos`: a Heal
    /// gated on a key a Partition publishes is ordered AFTER the Partition (so at
    /// runtime its `wait_for` is resolvable), and a Partition gated on an external
    /// (actor/clock) key keeps its place. Mirrors the chaos timeline
    /// partition→…→heal.
    #[test]
    fn director_orders_chaos_step_after_its_publishing_predecessor() {
        let chaos = vec![
            // Heal (chaos[0]) gated on `partitioned` — declared BEFORE the
            // Partition in manifest order, to prove the edge reorders it.
            chaos(ChaosKind::Heal, Some("partitioned"), None),
            // Partition (chaos[1]) publishes `partitioned`.
            chaos(ChaosKind::Partition, Some("space_built"), Some("partitioned")),
        ];
        let order = order_director_steps(&[], &[], &[], &chaos).expect("orders");
        assert!(
            pos(&order, DirectorStep::Chaos(1)) < pos(&order, DirectorStep::Chaos(0)),
            "the Partition publishing `partitioned` must precede the Heal waiting on it; got {order:?}"
        );
    }

    /// A chaos step gated on an EXTERNAL key (published by the parallel chaos task
    /// or the actor drive, not any director step) carries no internal edge → it
    /// keeps phase order (chaos after links/clocks/migrations).
    #[test]
    fn director_chaos_external_gate_keeps_phase_order() {
        let links = vec![link(0, 1, None)];
        let chaos = vec![chaos(ChaosKind::Heal, Some("flood_done"), None)]; // flood_done = external
        let order = order_director_steps(&links, &[], &[], &chaos).expect("orders");
        assert_eq!(
            order,
            vec![DirectorStep::Link(0), DirectorStep::Chaos(0)],
            "external-gated chaos keeps phase order (after links)"
        );
    }

    /// No-edge case is order-unchanged: with no publish→wait coupling the order is
    /// the original phase order (links → clocks → migrations, manifest order) — the
    /// cooperative/topology director paths are unperturbed.
    #[test]
    fn director_no_edge_keeps_phase_order() {
        let links = vec![link(0, 1, None), link(0, 2, Some("history_ready"))]; // history_ready = external
        let clocks = vec![clock(0, None, None)];
        let migrations = vec![MigrationPlan {
            from_idx: 0,
            to_idx: 1,
            space_key: "space_id".into(),
            after: None,
        }];
        let order = order_director_steps(&links, &clocks, &migrations, &[]).expect("orders");
        assert_eq!(
            order,
            vec![
                DirectorStep::Link(0),
                DirectorStep::Link(1),
                DirectorStep::Clock(0),
                DirectorStep::Migration(0),
            ],
            "no publish→wait edge → original phase order preserved"
        );
    }
}

// ── MP-R3-D4a chaos-step partition (kind → seam) unit tests ──────────────────
#[cfg(test)]
mod chaos_partition_tests {
    use super::partition_chaos_steps;
    use crate::manifest::{ChaosKind, ChaosStep};

    fn step(kind: ChaosKind) -> ChaosStep {
        ChaosStep {
            kind,
            nodes: vec![],
            after: None,
            publishes: None,
            count: None,
            hold_ms: None,
        }
    }

    #[test]
    fn chaos_specs_partition_into_director_and_raw_ws() {
        // Partition/Heal → director (node-conn); Flood/Storm/SlowLoris → raw-WS.
        let steps = vec![
            step(ChaosKind::Partition), // 0 director
            step(ChaosKind::Flood),     // 1 raw-ws
            step(ChaosKind::Heal),      // 2 director
            step(ChaosKind::Storm),     // 3 raw-ws
            step(ChaosKind::SlowLoris), // 4 raw-ws
        ];
        let (director, raw_ws) = partition_chaos_steps(&steps);
        assert_eq!(director, vec![0, 2]);
        assert_eq!(raw_ws, vec![1, 3, 4]);
    }

    #[test]
    fn empty_chaos_partitions_to_empty() {
        let (d, r) = partition_chaos_steps(&[]);
        assert!(d.is_empty() && r.is_empty());
    }
}

// ── MP-R3-D3 injector-target resolution unit tests ───────────────────────────
#[cfg(test)]
mod injector_target_tests {
    use super::injector_targets;

    fn topology() -> Vec<(String, String)> {
        vec![
            ("a".into(), "ws://127.0.0.1:9001/xgen".into()),
            ("b".into(), "ws://127.0.0.1:9002/xgen".into()),
        ]
    }

    #[test]
    fn injector_targets_default_to_single_node() {
        // MP-R3-D3 backward-compat: a `node`-only injector (empty `nodes` list)
        // resolves to exactly one target — the R1/R2 single-target shape.
        let t = injector_targets("a", &[], &topology()).expect("resolves");
        assert_eq!(t.len(), 1);
        assert_eq!(t[0].0, "a");
        assert_eq!(t[0].1, "ws://127.0.0.1:9001/xgen");
    }

    #[test]
    fn injector_targets_multi_resolves_all_labels_in_order() {
        // A `nodes = ["a","b"]` injector resolves both targets, in list order
        // (fork-a → a, fork-b → b).
        let nodes = vec!["a".to_string(), "b".to_string()];
        let t = injector_targets("a", &nodes, &topology()).expect("resolves");
        assert_eq!(t.len(), 2);
        assert_eq!(t[0].0, "a");
        assert_eq!(t[1].0, "b");
        assert_eq!(t[1].1, "ws://127.0.0.1:9002/xgen");
    }

    #[test]
    fn injector_targets_unknown_label_errors() {
        let nodes = vec!["a".to_string(), "ghost".to_string()];
        let r = injector_targets("a", &nodes, &topology());
        assert!(r.is_err());
        assert!(format!("{:#}", r.unwrap_err()).contains("unknown node label"));
    }
}
