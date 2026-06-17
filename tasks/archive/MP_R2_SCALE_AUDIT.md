# MP-R2 — Multiparty-tests Round 2 (scale + real-clock): D-071 Phase-0 Audit
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jun 2026  
> **Last updated**: 2026-06-10  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. What this is + where it sits

The **Multiparty-tests** milestone **runs** the M9 `xgen-mptest` harness through an escalating
three-round ladder on a finalized binary. Per the Joe-locked structure (2026-06-07), the milestone
decomposes into three numbered sub-passes of monotonically increasing weight:

- **MP-R1 — deterministic correctness floor.** ✅ CLOSED (J-340, HEAD `a9fbd98`): all-green-to-
  criterion, the protocol core proven correct under **no load**. Light, MockClock, fixed seeds.
- **MP-R2 — scale + real-clock, moderate-heavy** (this audit). **Continuation, not a fresh start:**
  begins where R1 ends and climbs the volume axis. The `Sweep`/`SweepResult` contract (locked at
  MP-R1-D2, single-rung-proven) runs **multi-rung for the first time**; the deliverable is the
  **break-point per volume axis** (oracle-checked per rung), not a bare pass/fail.
- **MP-R3 — capstone.** Maximum the box bears (~1,562-process estimate, `M9_findings.md` §5 / the
  bench), chaos overlay stacked. One-shot, full capture.

This is a D-071 Phase-0 audit for **MP-R2 only**: it grounds what R2 depends on against live `main`
to a **verdict** (wired / net-new / stubbed) per the HANDOFF §3 asks, reconciles the R2/R3 row
split against the canonical matrix §4 (HANDOFF §4), and frames the forks for the MP-R2 design
phase. **No code authored here; no heavy run** — the freed box is the RUN gate (M-R2.3), not the
Phase-0 gate (HANDOFF §1).

"Finalized binary" = the convergence/federation/MLS protocol core (M1–M9.2, shipped) — the surface
MP-R1 certified correct. R2 stresses that same surface; it does **not** re-litigate correctness.

---

## 2. What MP-R2 depends on (the crossing)

R1's deliverable was the **general scenario runner** (`run_scenario`, MP-R1-D1) + the four scenario
tranches. R2 reuses `run_scenario` unchanged for fixed-N rows, but R2's defining work is the
**dial → runner scale bridge**: today the runner spawns the topology the *manifest* names and
**ignores the dial's scale fields** ([`runner.rs:46-49`](../xgen-mptest/src/runner.rs#L46) topology-
authority note). R2 is the round where the dial's `nodes` / `clients` / `residents_per_process`
become load-bearing — the sweep can already *step* a dial axis ([`sweep.rs:77`](../xgen-mptest/src/sweep.rs#L77)
`SweepAxis::apply`), but the stepped value reaches nothing in `run_scenario` today. **Bridging the
dial scale fields into the spawn loop is the structural heart of R2** (it is the same gap from
three angles — Asks 1, 3, and the residents finding all converge on it).

Seam state inherited from M9.2 + R1 (all in place): the F2/F3/F4 fenced seams, the G-6 federation
bootstrap (encoded in the runner), the `[[clock]]` director, the Space-scoped oracle, the sweep
type + classifier, the `resource.rs` sampler, the `bench.rs` box-ceiling micro-benchmark.

---

## 3. Grounding findings against live `main` (the HANDOFF §3 asks)

Each ask graded **wired** (exists + exercised) / **net-new** (must be built) / **stubbed**
(structure exists, body inert/uninvoked), with the live evidence.

### G-1 — `residents_per_process` multiplexing → **NET-NEW (at three layers; dominates R2 build)**

The R2 scale prerequisite, and the HANDOFF correctly flagged it as the likely dominant cost.
Grounded:

- **Declared, not bridged.** `RoundDial.residents_per_process` ([`dial.rs:75`](../xgen-mptest/src/dial.rs#L75))
  + `logical_participants() = clients × residents_per_process` ([`dial.rs:102`](../xgen-mptest/src/dial.rs#L102))
  + `SweepAxis::ResidentsPerProcess` applies it to a dial ([`sweep.rs:82`](../xgen-mptest/src/sweep.rs#L82)).
  **Nothing reads it in the runner** — `run_scenario` spawns exactly **one client `ManagedProcess`
  per `manifest.actors` entry** ([`runner.rs:220-247`](../xgen-mptest/src/runner.rs#L220)); `grep
  residents_per_process` hits only dial.rs + sweep.rs (+ tests), never runner.rs.
- **No manifest representation.** `ActorSpec` ([`manifest.rs:116-131`](../xgen-mptest/src/manifest.rs#L116))
  has `name` / `node` / `batch` / `ai_mode` / `kind` — **no residents/multiplicity field**. One
  actor = one batch file = one spawned process = one identity.
- **No client capability either.** `ai_mode` passes `--ai-mode` to a `--service` client
  ([`process.rs:159-174`](../xgen-mptest/src/process.rs#L159)); the M4 AI resident runs **one**
  identity (EchoPlugin), not a multiplexer. The dial.rs doc's "an AI resident drives many logical
  participants" ([`dial.rs:75`](../xgen-mptest/src/dial.rs#L75)) is **aspirational** against the
  shipped M4 resident — not a wired capability.

**Verdict: net-new at the dial→runner bridge, the manifest schema, AND (for the genuine
two-number model) the client multiplexer.** This is the largest single R2 build item. See the
fork at §5 F-1 (how a logical participant is realized).

### G-2 — Real-clock path → **WIRED (structurally) / NET-NEW (sustained-window scenarios)**

- **Structurally wired.** `ClockMode::Real` is the default ([`dial.rs:48`](../xgen-mptest/src/dial.rs#L48)),
  `validate()` accepts it ([`dial.rs:114`](../xgen-mptest/src/dial.rs#L114)), and `run_scenario`
  special-cases **only** `Mock` (the harness-control probe, [`runner.rs:207-211`](../xgen-mptest/src/runner.rs#L207)).
  The clock-director phase is a **no-op when `m.clock` is empty** ([`runner.rs:415`](../xgen-mptest/src/runner.rs#L415)).
  So a real-clock scenario routes through the same `run_scenario`/`run_sweep` with **no fork** —
  `ClockMode::Real` needs no harness-control build (`requires_harness_control()` is false,
  [`dial.rs:60`](../xgen-mptest/src/dial.rs#L60)), though most R2 rows are federated and need it
  anyway (G-7).
- **Net-new behavior territory.** R1 pinned MockClock at a fixed instant; **real-clock behavior
  under a sustained window has never been harness-exercised** — reconnect backoff ladders
  (15/30/60/120 min, M8.6), federation timeouts (30 s / 180 s), pacing windows, the M8.6 clock
  seam. And the runner's drive model is **finite-batch-then-`settle()`** ([`runner.rs:437`](../xgen-mptest/src/runner.rs#L437),
  poll-until-stable capped at 15 s wall-clock) — it does not fit "sustained posting for a window"
  (MP-C-05 / MP-C-11). The matrix lists those rows' Batch as "generated per ramp" (matrix §3), but
  the harness **feeds lines verbatim, no ad-hoc inline generation** (matrix §2). So sustained/ramp
  scenarios need (a) batch-generation machinery and (b) possibly a windowed/timed drive model
  rather than run-to-batch-end.

**Verdict: the plumbing routes real-clock through the existing runner unchanged (wired); the
sustained-window scenario *shape* + batch generation is net-new.** See fork F-2.

### G-3 — Sweep multi-rung climb → **IMPLEMENTED (type + classifier + loop) / NEVER INVOKED + AXIS-INERT**

- **Implemented.** `run_sweep` iterates `sweep.rung_values()`, classifies each rung, and stops on
  `LogicFault`-with-`stop_on_fail` or any `Ceiling`, returning `SweepResult { rungs, break_point }`
  ([`sweep.rs:204-245`](../xgen-mptest/src/sweep.rs#L204)). `classify_rung` + the GREEN/LOGIC-FAULT/
  CEILING distinction is pure + unit-tested ([`sweep.rs:155-165`](../xgen-mptest/src/sweep.rs#L155),
  9 unit tests). `rung_values()` multi-rung climb is unit-tested ([`sweep.rs:316`](../xgen-mptest/src/sweep.rs#L316)).
- **Never invoked multi-rung.** The **only** `run_sweep` caller is `mp_r1_sweep.rs:44`, and it uses
  `Sweep::single(...)` → **one rung** ([`mp_r1_sweep.rs:42`](../xgen-mptest/tests/mp_r1_sweep.rs#L42)).
  The loop body runs once; the climb (step > 0, break-point across rungs) has **never run against
  real binaries** — only the pure unit tests cover it.
- **Axis-inert at the runner (the load-bearing gap).** Even a multi-rung `Sweep` over
  `Nodes`/`Clients`/`ResidentsPerProcess` would **not change the spawned topology**: `run_scenario`
  spawns per `manifest.nodes` + `manifest.actors` and **ignores `dial.nodes`/`dial.clients`**
  ([`runner.rs:46-49`](../xgen-mptest/src/runner.rs#L46)); `residents_per_process` is unbridged
  (G-1). So `SweepAxis::apply` mutates dial fields that the spawn loop reads for **nothing** — the
  climb would re-run the identical scenario every rung.

**Verdict: the curve+break-point contract is implemented and single-rung-proven; the multi-rung
climb is implemented-but-uninvoked AND functionally inert until the dial scale fields reach
`run_scenario`'s spawn (= G-1's bridge).** The break-point-per-axis deliverable is gated on that
bridge. See fork F-3.

### G-4 — R2/R3 row split → **reconciled against matrix §4** (full reconciliation at §4 below)

The named disagreement (HANDOFF §4): **MP-A-08** carried under R2 but tagged **R3** in matrix §4
([matrix line 190](../docs/tests/MULTIPARTY_TEST_MATRIX.md)). **Matrix wins: MP-A-08 is R3.** Full
row-by-row table at §4. A second finding surfaced doing the reconciliation: **R2 is two distinct
bodies of work**, not one volume climb — see §4.

### G-5 — CEILING-vs-LOGIC-FAULT classifier → **WIRED + FED (but uncalibrated, single-snapshot, per-process)**

The HANDOFF's worry ("OOM mislabels as protocol broke") is **not** the current state for a sampled
peak above the floor — the classifier **is** consulted and **is** fed real data:

- **Consulted + fed.** `classify_rung` reads `outcome.resource.as_ref()` ([`sweep.rs:211`](../xgen-mptest/src/sweep.rs#L211));
  `run_scenario` populates `resource` via `peak_resource(&nodes, &actors)` ([`runner.rs:365`](../xgen-mptest/src/runner.rs#L365),
  [`470-477`](../xgen-mptest/src/runner.rs#L470)), which samples every node + client pid through
  `Get-Process` ([`resource.rs:89`](../xgen-mptest/src/resource.rs#L89)) and returns the **max-RSS**
  process. The D-065 distinction is real: a fail with RSS ≥ wall ⇒ `Ceiling`; a fail with healthy
  resources ⇒ `LogicFault` ([`sweep.rs:159-164`](../xgen-mptest/src/sweep.rs#L159)).
- **Five caveats R2 must close (none a correctness blocker today, all sharpen at scale):**
  1. **Uncalibrated floors.** `RSS_WALL_BYTES = 1.5 GB` + `THREAD_THRASH_COUNT = 64` are
     "coarse first-pass floors, retuned against the bench box-ceiling in R2/R3"
     ([`sweep.rs:54-60`](../xgen-mptest/src/sweep.rs#L54)). `bench.rs` derives the real ceiling
     (`BoxSpec` 32 GB/20-core, `estimate_ceiling` ≈ budget/mean-RSS, [`bench.rs:58`](../xgen-mptest/src/bench.rs#L58))
     but the sweep floors are **not wired to the bench output**. R2 RUN-gate step 0 must run the
     bench (`XGEN_MPTEST_BENCH_TIERS=10,50,100`, [`bench.rs:19`](../xgen-mptest/src/bench.rs#L19))
     on the freed box and calibrate.
  2. **Single snapshot at oracle time.** `peak_resource` samples once, **after** `settle()` — a
     transient peak *during* the drive is missed. Continuous/peak-during-drive sampling is an R2
     enrichment (sweep.rs doc already names it).
  3. **Per-process only.** Aggregate-RSS-vs-box-RAM + OOM-by-exit-code are named R2/R3 enrichments
     ([`sweep.rs:36-39`](../xgen-mptest/src/sweep.rs#L36)); the wall today is one runaway process,
     not total footprint. At R2 scale total footprint is the likelier wall.
  4. **Injectors + harness unsampled.** `peak_resource` iterates only `nodes` + `actors`
     ([`runner.rs:470-477`](../xgen-mptest/src/runner.rs#L470)); injector actors (no client
     process) and the harness's own process are not sampled.
  5. **The real inversion risk.** `Get-Process` failure → `None` → conservative `LogicFault`
     ([`sweep.rs:161-164`](../xgen-mptest/src/sweep.rs#L161)). Sampling is most likely to fail
     **under memory pressure** — exactly when a true `Ceiling` should be called. So the one way the
     HANDOFF's "OOM mislabels as protocol broke" *can* happen is a sampler that dies under the very
     pressure it should detect. R2 should treat sampler-failure-under-a-failed-rung as a
     ceiling-suspect, not silently LogicFault.

**Verdict: wired + fed; calibration, continuous/aggregate sampling, and the sampler-failure
inversion are R2 hardening.** See fork F-4.

### G-6 — `worker_threads` dial field → **DECORATIVE (env-driven, not dial-driven)**

Surfaced while grounding the spawn path. `RoundDial.worker_threads: Option<u32>`
([`dial.rs:82`](../xgen-mptest/src/dial.rs#L82)) is **never read** — the spawn pins
`TOKIO_WORKER_THREADS` from the `XGEN_MPTEST_WORKER_THREADS` env var (default `2`)
([`process.rs:83-85`](../xgen-mptest/src/process.rs#L83), [`243`](../xgen-mptest/src/process.rs#L243)).
The env override works (so worker-thread pinning is wired), but the **dial field is inert** — a
third unbridged dial field alongside `nodes`/`clients`/`residents_per_process`. Minor; the
dial→runner bridge (F-3) should either consume it or the field should be retired to avoid teaching
a false knob. Recorded so the design phase decides deliberately.

### G-7 — Binary-build selection → **MANUAL prerequisite (no per-clock-mode selection); the clobber fence is live for R2**

`binloc::locate()` returns whatever binaries sit at the pinned target dir; `init_and_spawn_node`
takes no feature/build selector ([`process.rs:123`](../xgen-mptest/src/process.rs#L123)). The
`--features harness-control` build is a **manual prerequisite** (the smoke headers run
`cargo build -p xgen-node --features harness-control` first). Most R2 rows are federated (3-node
MP-C-04/14, restart MP-C-15, migration MP-C-16) → need harness-control. So the **binary-clobber
hazard is live and sharper at R2** (HANDOFF §5): `cargo test --workspace` rebuilds `xgen-node`
default-features over the harness-control binary → heavy tranches fail all-`UNKNOWN_COMMAND` (the
J-315 fence-holds signal). Operational fence carried into the runbook (§7).

---

## 4. R2/R3 row reconciliation (HANDOFF §4 — matrix §4 authoritative)

Every disputed/unread row in HANDOFF §4 grounded against the canonical matrix
([`docs/tests/MULTIPARTY_TEST_MATRIX.md`](../docs/tests/MULTIPARTY_TEST_MATRIX.md) §3 + §4). **The
matrix tag wins** where it disagrees with the carried set.

| Row | Matrix §4 tag | Carried-set | Resolution |
|-----|---------------|-------------|------------|
| **MP-A-08** partition + reconnect storm | **R3** (line 190) | R2 | **R3** — matrix wins. The named disagreement; removed from the R2 set. |
| MP-A-07 high-rate flood / DoS | R2 → R3 (187) | R2 | **R2 entry-rung**, climbs to R3. Volume sweep. |
| MP-A-18 connect/disconnect storm | R2 → R3 (227) | R2 | **R2 entry-rung**, climbs to R3. Resource sweep (C4 leak gauge at the binary). |
| MP-A-11 oversized payload | R2 (201) | R2 | **R2** — resource. |
| MP-A-13 anti-transitivity probe | R2 (208) | R2 | **R2** — federation, fixed-N (observe `.events` on C). |
| MP-A-19 slow-loris / held connections | R2 (230) | R2 | **R2** — resource. |
| MP-A-21 stale/rollback MLS commit | R2 (238) | R2 | **R2** — wire, fixed-N (M8.7 `mls_commit_tip`). |
| MP-A-06 equivocation / fork | R2 (184) | R2 | **R2** — wire, fixed-N; outcome = convergence-on-winner, not absence. |
| MP-C-04 3-node transitive path | R2 (90) | R2 | **R2** — fixed-N (3-node), gated on the existing F2 federation. |
| MP-C-05 sustained n×n chat | R2 → R3 (95) | R2 | **R2 entry-rung**, climbs to R3. **Volume sweep** (the residents/ramp axis). |
| MP-C-11 membership churn under load | R2 → R3 (125) | R2 | **R2 entry-rung**, climbs to R3. **Volume sweep**. |
| MP-C-12 E2E content-blindness | R2 (130) | R2 | **R2** — fixed-N (Arc H), new-capability. |
| MP-C-14 4–5 node star+mesh | R2 → R3 (140) | R2 | **R2 entry-rung**, climbs to R3. Topology, fixed-N-per-rung. |
| MP-C-15 node restart mid-chat + replay | R2 (145) | R2 | **R2** — durability, fixed-N (kill/restart machinery). |
| MP-C-16 live space migration | R2 (150) | R2 | **R2** — fixed-N (Arc F migration verb). |
| **MP-A-01(ii)** federation-replay-preserved | carried | carried | **Not a scale-axis row** — see below. |

**Surfaced finding (G-4 detail): R2 is two distinct bodies of work, not one volume climb.**

- **(a) The multi-rung scale/resource sweep** — the rows where "turn up the volume" *is* the test:
  **MP-C-05, MP-C-11** (cooperative volume) + **MP-A-07, MP-A-11, MP-A-18, MP-A-19** (adversarial
  volume/resource). These exercise the dial→runner bridge (G-1/G-3) + the CEILING classifier (G-5)
  — the round's headline contract.
- **(b) New-capability fixed-N cross-node/topology rows** — never-run scenarios that are *not*
  volume sweeps: **MP-C-04** (3-node), **MP-C-12** (E2E), **MP-C-14** (star+mesh topology),
  **MP-C-15** (restart+replay), **MP-C-16** (migration); **MP-A-06** (equivocation), **MP-A-13**
  (anti-transitivity), **MP-A-21** (MLS rollback). Each needs new authoring/harness capability
  (3+-node G-6 generalization, kill/restart control, migration verb, equivocation injector,
  per-node `.events` assertion on a non-recipient), but runs at fixed N — they ride `run_scenario`
  as-is once their capability exists.

This split matters for the design phase + runbook sequencing: (b) can land independent of the
dial→runner bridge; (a) is gated on it. Recommend the design phase treats them as separate tranches.

**MP-A-01(ii) placement (fork F-5).** Matrix MP-A-01 (line 163): part (i) ✅ R1; part (ii) PENDING —
"federation-replay-membership-preserved … needs late-federation/catch-up where B federates AFTER
the clock ages the Space; the fixed G-6 bootstrap establishes federation early." This is a
**federation-timing-machinery** gap, **not a volume axis**. The same late-federation/catch-up
machinery is wanted by **MP-C-15** (restart → catch up) and arguably **MP-C-16** (migration cutover).
Recommend: carry A-01(ii) as an **R2 infrastructure item** (the catch-up/late-join machinery), not
a sweep row — surfaced as fork F-5 for Joe.

---

## 5. Forks for the MP-R2 design phase (Joe-lock)

Surfaced, not decided. The design phase locks these into **MP-R2-D#** (arc-local, D-069).

- **F-1 — how is a logical participant realized? (the residents multiplexing model; G-1).**
  - **(a) AI-resident multiplexer:** one `--ai-mode` client process drives N logical identities
    (honors the two-number model — processes are the HW wall, logical participants an order
    cheaper). Net-new client capability (the M4 resident is one-identity). Largest build, truest to
    the audit §6 scale model.
  - **(b) Runner actor-expansion:** the runner expands one `ActorSpec { residents = N }` into N
    client processes. Cheap to build, but **does not reduce process count** → defeats the
    two-number model and hits the box ceiling at the *same* `logical_participants` as one-per-process.
  - **(c) Hybrid / batch-templating:** a manifest actor-template the runner instantiates ×N
    (processes) with a `{{resident_index}}` substitution into the batch.
  - *Audit lean (not a lock):* (a) is the only option that delivers what `logical_participants()`
    promises; (b) is a cheap stopgap that should be named as such if chosen. The cost of (a) is the
    reason this fork is the round's pivot.

- **F-2 — sustained-window scenario shape + batch generation (G-2).** Finite generated-batch
  (pre-expand the ramp into verbatim `.jsonl` lines the harness already feeds) vs a windowed/timed
  drive model (the runner drives for a wall-clock window, then `settle`). The matrix's "generated
  per ramp" implies a generator; the harness's verbatim-feed invariant (matrix §2) implies
  pre-generation. *Lean:* pre-generate batches (preserves the verbatim-feed invariant, no inline
  generation in the hot path) — but the design must confirm `settle()`'s 15 s cap suits a long run.

- **F-3 — the dial → `run_scenario` scale bridge (G-1/G-3; the structural heart).** How do
  `dial.nodes` / `dial.clients` / `dial.residents_per_process` reach the spawn? Options: the runner
  reads the dial when the manifest is "templated" (a base manifest the dial scales) vs the sweep
  generates a concrete manifest per rung vs an explicit `dial_overrides_manifest` flag. This fork
  also decides G-6 (`worker_threads`: consume from the dial or retire the field). *Lean:* the
  runner consumes the dial scale fields for sweep rows (manifest stays authoritative for fixed-N
  rows) — i.e. the topology-authority note ([`runner.rs:46-49`](../xgen-mptest/src/runner.rs#L46))
  is amended for the sweep path only.

- **F-4 — CEILING classifier hardening scope for R2 (G-5).** Which of the five caveats land in R2
  vs defer to R3: (1) bench-calibrated floors [recommend R2 — cheap, high-value], (2)
  continuous/peak-during-drive sampling, (3) aggregate-RSS-vs-box-RAM + OOM-exit-code, (4)
  injector/harness sampling, (5) the sampler-failure-under-pressure inversion guard [recommend R2 —
  it is the one path to the HANDOFF's feared mislabel].

- **F-5 — MP-A-01(ii) + late-federation/catch-up machinery placement (G-4).** Carry as an R2
  infrastructure item shared with MP-C-15/16, vs leave A-01(ii) as a standalone PENDING harness-
  timing gap (as R1 recorded it), vs defer to R3. *Lean:* build the catch-up/late-join machinery as
  R2 infra (MP-C-15 needs it regardless) and let A-01(ii) ride it.

- **F-6 — R2 tranche structure (G-4 two-bodies finding).** Whether the runbook tranches by the
  (a)-scale-sweep vs (b)-new-capability split (recommended), or by cooperative/adversarial as R1
  did. The split-by-kind makes the dial-bridge dependency explicit (all (a) rows gate on F-3; all
  (b) rows don't).

---

## 6. The R2 scenario set (from the matrix, Round = R2)

Per §4 reconciliation. **15 R2 rows** (8 cooperative + 7 adversarial) + 1 carried infra item;
**1 row (MP-A-08) reassigned to R3.**

**Cooperative (8):** MP-C-04 (3-node transitive) · MP-C-05 (sustained n×n, →R3) · MP-C-11 (churn
under load, →R3) · MP-C-12 (E2E content-blindness) · MP-C-14 (star+mesh, →R3) · MP-C-15 (restart +
replay) · MP-C-16 (live migration). *(MP-C-04 + MP-C-05 + MP-C-11 + MP-C-12 + MP-C-14 + MP-C-15 +
MP-C-16 = 7 distinct; MP-C-05/11/14 carry an R3 entry-rung continuation.)*

> Count note: the matrix seeds **7** cooperative R2/R2→R3 rows (C-04/05/11/12/14/15/16). The "8"
> above is corrected to **7** on this recount — recorded honestly (the design phase confirms
> against matrix §3, no padding).

**Adversarial (7):** MP-A-06 (equivocation) · MP-A-07 (flood, →R3) · MP-A-11 (oversized payload) ·
MP-A-13 (anti-transitivity) · MP-A-18 (connect/disconnect storm, →R3) · MP-A-19 (slow-loris) ·
MP-A-21 (stale MLS commit).

**Carried R2 infra (not a sweep row):** MP-A-01(ii) — late-federation/catch-up machinery (fork F-5).

**Reassigned out of R2:** MP-A-08 (partition + reconnect storm) → **R3** (matrix line 190).

Grouped by the §4 two-bodies split:
- **(a) scale/resource sweep (gated on the F-3 dial bridge):** MP-C-05, MP-C-11, MP-A-07, MP-A-11,
  MP-A-18, MP-A-19.
- **(b) new-capability fixed-N (independent of the bridge):** MP-C-04, MP-C-12, MP-C-14, MP-C-15,
  MP-C-16, MP-A-06, MP-A-13, MP-A-21.

---

## 7. Scope, honest boundary, defect policy, operational fences

**Change surface (design-phase estimate; all in `xgen-mptest` + scenario dirs).** The dial→runner
scale bridge (F-3) + the residents multiplexing mechanism (F-1, possibly an `xgen-client`
multiplexer if option (a)) + sustained-batch generation (F-2) + the CEILING hardening (F-4) + the
late-federation/catch-up machinery (F-5) + the new-capability scenario harness extensions
(kill/restart for MP-C-15, migration drive for MP-C-16, equivocation injector for MP-A-06,
per-node-non-recipient `.events` assertion for MP-A-13) + the R2 scenario dirs. **If F-1 picks the
AI-resident multiplexer (a), R2 touches a production crate (`xgen-client`)** — protocol-change
discipline applies there; the harness changes stay test-crate.

**Defect policy (D-065/D-084) — surface-and-route, never patch-in-place.** A scale row that
surfaces a genuine protocol defect (a lost admitted event at volume, a convergence failure under
churn, a reconnect deadlock) routes to a finding in `tasks/MP_findings.md` + its own fix-arc; it
does **not** block the R2 break-point record. The deliverable is the **curve + break-point per
volume axis**, oracle-checked per rung, with each non-GREEN rung classified GREEN/LOGIC-FAULT/
CEILING (the D-065 distinction) — a CEILING is a hardware finding, not a protocol FAIL.

**Honest boundary.** R2 proves the protocol holds **under moderate-heavy load + real time** and
finds the volume break-points; it is **not** the chaos capstone (R3) and **not** correctness (R1,
done). A green R2 is a scale floor on the freed box, read against the bench-calibrated ceiling — not
an unbounded-scale claim.

**Operational fences (carried, sharper at R2).**
- **Binary-clobber (G-7).** Run `cargo test --workspace` **before** the `--features harness-control`
  build, or rebuild harness-control after any workspace build, before the heavy tranches. All-
  `UNKNOWN_COMMAND` on a fenced verb (`clock`/`add-peer`) is the J-315 fence-holds signal of a
  clobbered binary — diagnose the binary, not the code.
- **Spawn-timeout flakes ≠ protocol RED.** R1's MP-C-10 failed once on an aicontrol pipe-connect
  timeout under peak parallelism (passed isolated). R2 contention is higher by construction —
  classify spawn/connect timeouts distinctly (Rule 2: confirm-before-classify, re-run isolated)
  before routing anything as a finding.
- **RUN gate.** The freed box is the M-R2.3 RUN gate, **not** the Phase-0 gate. Phase-0 (this doc)
  + design + runbook proceed now; no heavy multi-binary sweep starts until Joe confirms the box is
  free. **First RUN step: the `bench.rs` box-ceiling micro-benchmark** (`XGEN_MPTEST_BENCH_TIERS=
  10,50,100`) to calibrate the sweep floors (F-4) + fix the real R2/R3 numbers before any sweep.

**Exit.** Verdicts on the five asks (§3 G-1..G-5) + the two extras (G-6/G-7) recorded; the R2/R3
split reconciled against matrix §4 (§4); forks F-1..F-6 surfaced for Joe-lock (§5). On Joe-lock →
the MP-R2 design phase (lock MP-R2-D#) → runbook → run. Discipline: Phase-0 → design → Joe-lock →
runbook → run; Clair's code FIRST, Chat's doc-bridge separate, Joe pushes both. No self-close.

Per D-065 + D-069 + D-071 + D-074 + D-084 + MP-R1-D2 (the sweep contract R2 inherits) + MP-R1-D8
(honest boundary) + MP-R1-D10 (surface-and-route).
