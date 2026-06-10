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
use crate::manifest::{ActorKind, ActorSpec, ClockOp, Scenario};
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

/// An injector actor (MP-R1-D1 / C7): no client process — its node `ws://` URL +
/// parsed attack-directive batch, driven by [`run_injector_actor`] on the shared
/// `Registry` alongside the batch actors.
struct InjectorHandle {
    name: String,
    url: String,
    lines: Vec<BatchLine>,
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
            let node = node_by_label(&nodes, &spec.node)?;
            injectors.push(InjectorHandle {
                name: spec.name.clone(),
                url: node.url.clone(),
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
        run_injector_actor(&inj.name, &inj.url, &inj.lines, &registry, RESOLVE_TIMEOUT)
    }));
    let direct = run_director(
        &mut nodes,
        &link_plans,
        &clock_plans,
        &registry,
        RESOLVE_TIMEOUT,
    );
    let (drive_results, inj_results, direct_result) = tokio::join!(drive, inj_drive, direct);
    direct_result.context("scenario director")?;
    let mut actor_runs: Vec<ActorRun> = Vec::with_capacity(drive_results.len());
    for (name, res) in batch_names.iter().zip(drive_results) {
        actor_runs.push(res.with_context(|| format!("driving actor `{}`", name))?);
    }
    let mut injector_runs: Vec<InjectorRun> = Vec::with_capacity(inj_results.len());
    for (inj, res) in injectors.iter().zip(inj_results) {
        injector_runs.push(res.with_context(|| format!("driving injector `{}`", inj.name))?);
    }

    // ── 5. Settle (bounded poll-until-stable) ────────────────────────────────
    settle(&nodes).await;

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
                    ) {
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

    // Peak resource sample across all spawned processes (best-effort).
    let resource = peak_resource(&nodes, &actors);

    Ok(ScenarioOutcome {
        verdict,
        actor_runs,
        injector_runs,
        projections,
        transcripts,
        space_id,
        resource,
    })
}

/// The scenario director — a sibling of the actor drive that owns the node
/// connections during the concurrent phase. Two phases over the shared registry:
///
/// 1. **Federation** (MP-R1-D1a steps iv/v): wait for the owner's exported Space,
///    then for each link re-`add-peer` naming it and `initiate` from `from`.
/// 2. **Clock** (MP-R1-D3): fire each clock step in manifest order — block on its
///    `after` key (if any), then send the F3 verb on the target node.
///
/// R1 ordering: federation completes before the clock phase begins (a single
/// sequential director owns `&mut nodes`). R1's clock scenarios (MP-A-01) advance
/// the clock *after* setup, so this ordering is correct; a clock-before-federation
/// scenario is out of MP-R1 scope. Both phases are no-ops when empty.
async fn run_director(
    nodes: &mut [NodeHandle],
    links: &[LinkPlan],
    clocks: &[ClockPlan],
    registry: &Registry,
    timeout: Duration,
) -> Result<()> {
    // ── Federation phase ─────────────────────────────────────────────────────
    if !links.is_empty() {
        let space = registry
            .wait_for(PRIMARY_SPACE_KEY, timeout)
            .await
            .context("director waiting for the exported Space id")?;
        for plan in links {
            match &plan.after {
                // MP-R2-D5 late link: wait for its gate (e.g. the clock-aged
                // key), then establish BOTH directions naming the now-aged Space
                // + initiate (this link was NOT pre-seeded). The receiving node
                // catches up the aged Space via the existing sync path — the
                // catch-up shape MP-A-01(ii) witnesses.
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
                // `from` naming the Space + initiate — the G-6 bootstrap tail,
                // unchanged.
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
    }

    // ── Clock phase (MP-R1-D3) ───────────────────────────────────────────────
    for step in clocks {
        if let Some(after) = &step.after {
            registry
                .wait_for(after, timeout)
                .await
                .with_context(|| format!("clock step waiting on `{{{{{after}}}}}`"))?;
        }
        node_clock(&mut nodes[step.node_idx].ctl, step.op, &step.value)
            .await
            .with_context(|| format!("clock {:?} {}", step.op, step.value))?;
        // Publish the clock-completion key (if named) so an actor can `[[waits]]`
        // on the clock having advanced (MP-A-01: bob joins only after expiry).
        if let Some(key) = &step.publishes {
            registry.publish(key.clone(), step.value.clone()).await;
        }
    }
    Ok(())
}

/// Bounded poll-until-stable settle: wait for the total observed-event count to
/// stop changing across two consecutive intervals (replication quiescence),
/// capped so a stuck run still returns. Single-node scenarios settle in ~1s.
async fn settle(nodes: &[NodeHandle]) {
    const INTERVAL: Duration = Duration::from_millis(400);
    const MAX: Duration = Duration::from_secs(15);
    // A small grace so the last fan-out of the drive lands before the first poll.
    tokio::time::sleep(Duration::from_millis(600)).await;
    let deadline = tokio::time::Instant::now() + MAX;
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

/// Sample every spawned process's RSS/threads (best-effort) and return the
/// heaviest (the OOM frontier the sweep's CEILING classifier consults).
fn peak_resource(nodes: &[NodeHandle], actors: &[ActorHandle]) -> Option<ResourceSample> {
    let pids = nodes
        .iter()
        .map(|n| n.proc.pid())
        .chain(actors.iter().map(|a| a.proc.pid()));
    pids.filter_map(|pid| sample_process(pid).ok())
        .max_by_key(|s| s.rss_bytes)
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
