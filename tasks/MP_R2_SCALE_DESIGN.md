# MP-R2 — Multiparty-tests Round 2 (scale + real-clock): Design
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-10  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. What this is

The design phase for **MP-R2**, the scale + real-clock round of the Multiparty-tests milestone.
Executes the Joe-LOCKED resolution of the six Phase-0 forks (`tasks/MP_R2_SCALE_AUDIT.md` §5) into
**MP-R2-D1..D6** (arc-local, D-069). Design only — no code, no run. The box (RUN gate, M-R2.3)
stays held; design + runbook proceed without it.

R2 **continues where R1 ended and climbs the volume axis**. R1's deliverable was the general
`run_scenario` runner + the 22 correctness scenarios, all green-to-criterion under no load. R2's
deliverable is the **break-point per volume axis** (oracle-checked per rung, GREEN/LOGIC-FAULT/
CEILING) against the bench-calibrated box ceiling, plus the never-run new-capability cross-node /
topology / durability scenarios the matrix tags R2.

**Locked-fork inputs (Joe, by recommendation):** F-1 defer the residents multiplexer to R3
(R2 = one process per logical participant; test-crate-only). F-2 pre-generate sustained-window
batches. F-3 runner consumes `dial.nodes`/`dial.clients` for sweep rows only. F-4 CEILING
hardening = caveats 1 + 5 only. F-5 build late-federation/catch-up machinery as R2 infra. F-6
tranche by the (a)-scale-sweep / (b)-new-capability split.

---

## 2. The design falsification (surfaced, not papered — D-065)

**Grounding F-1's premise against live `main` falsified its axis enumeration.** F-1 (and the
audit §4) said the (a)-sweep rows "scale on clients/nodes/**rate/connections**." Closer grounding
shows **only clients/nodes are dial-spawn axes; rate and connection-churn are neither dial fields
nor existing harness capabilities:**

- **No rate axis, no pacing.** `grep -i rate|delay|throttle|inter_send|per_second` over
  `xgen-mptest/src` hits only doc-comments (`sweep.rs:63` names a *future* `MessageRate`) and
  incidental poll-loop sleeps. `RoundDial` ([`dial.rs:68-83`](../xgen-mptest/src/dial.rs#L68)) has
  no rate field; `SweepAxis` ([`sweep.rs:65-73`](../xgen-mptest/src/sweep.rs#L65)) is
  `{Nodes, Clients, ResidentsPerProcess}` only. Critically, **`run_actor` fires lines
  back-to-back** — resolve → substitute → send → next, no inter-send delay
  ([`batch.rs:107-159`](../xgen-mptest/src/batch.rs#L107)); the injector loop is likewise
  back-to-back ([`injector_actor.rs:116`](../xgen-mptest/src/injector_actor.rs#L116)). So a "1000-
  message sustained window" completes in milliseconds, and flood "rate" is uncontrolled (back-to-
  back is the only rate). **Rate/intensity is net-new** — it is a *batch-generation + pacing*
  property, not a dial-spawn axis.
- **No connection-churn primitive.** `grep -i churn|disconnect|reconnect|interval` finds nothing
  in the harness for connection management. The orchestrator opens exactly one `.aicontrol`
  connection per actor + one WS per injector; there is no "open N connections, hold/drop them"
  driver. So **connection-churn (MP-A-18, MP-A-19) is net-new orchestrator machinery** that fits
  neither the dial-spawn model nor the batch model.

**Resolution (within Joe's locks; the literal F-1 enumeration refined, its intent preserved).**
R2's break-points come off three *distinct* mechanisms, not one:

1. **Spawn-scale** — `dial.nodes` + `dial.clients` → `run_scenario` spawn count (F-3, exactly as
   locked). Rows: MP-C-05 (participants), MP-C-11 (churning members).
2. **Intensity** — pre-generated **paced** batches (F-2 generalized: pacing is the rate knob).
   Rows: MP-A-07 (injector flood pace), and MP-C-05's within-window posting rate (C-05 carries
   *both* mechanisms 1 + 2).
3. **Connection-churn** — net-new orchestrator connection driver (R2 infra, sibling to F-5's
   catch-up machinery). Rows: MP-A-18, MP-A-19.

Plus **MP-A-11 (oversized payload) is fixed-N** (one big event, "bounded or rejected; no OOM") —
**not a sweep at all** → moves to tranche (b). This refines F-1's intent (R2 climbs the non-
residents axes) and **does not re-open Joe's F-3 lock** (the dial-spawn bridge is still nodes +
clients only). It is flagged here for Joe's blessing at the MP-R2-D# lock: **the R2 sweep has two
climb mechanisms (spawn-scale via the dial, intensity via paced batch-gen) + one orthogonal infra
item (connection-churn), not a single dial-axis climb.**

---

## 3. MP-R2-D1 (F-1 + F-6) — axis taxonomy, the (a)/(b) tranche split, residents → R3

**Decision: residents multiplexing DEFERRED to R3; R2 runs one process per logical participant
against the bench-calibrated ceiling; R2 is test-crate-only (no production-crate touch).** The
`residents_per_process` axis and the F-1(a) `xgen-client` AI-resident multiplexer become **R3
infra** — density-per-process is the R3 capstone concern (~1,562-process ceiling), and every R2
break-point comes off a non-residents mechanism (§2). Keeping residents out of R2 keeps the round
**test-crate-only** (the multiplexer would have forced an `xgen-client` production change).

**Axis table (canonical; `ResidentsPerProcess` explicitly R3 so the record implies no R2 residents
climb — carry-in #2):**

| Axis / mechanism | R2 status | Realized by | Rows |
|---|---|---|---|
| **Nodes** (`SweepAxis::Nodes`) | R2 | dial-spawn bridge (D3) | topology-width rungs (MP-C-14 climb) |
| **Clients** (`SweepAxis::Clients`) | R2 | dial-spawn bridge (D3) | MP-C-05, MP-C-11 |
| **Intensity / rate** | R2 | paced batch-generation (D2) | MP-A-07, MP-C-05 (within-window) |
| **Connection-churn** | R2 (infra) | net-new orchestrator driver (D5 sibling) | MP-A-18, MP-A-19 |
| **ResidentsPerProcess** (`SweepAxis::ResidentsPerProcess`) | **R3** (deferred, F-1) | AI-resident multiplexer (R3 infra) | density rungs (R3) |

**Tranche split (F-6 — by scale-sweep / new-capability, not coop/adversarial), so each tranche's
dependency on the D3 dial-bridge is explicit:**

- **Tranche (a) — scale/intensity sweep** (gated on D2 + D3): **MP-C-05, MP-C-11** (spawn-scale),
  **MP-A-07** (intensity). These exercise the dial→runner bridge + paced batch-gen + the CEILING
  classifier — the round's headline curve+break-point contract.
- **Tranche (b) — new-capability fixed-N** (independent of the dial-bridge; ride `run_scenario`
  as-is once their capability exists): **MP-C-04** (3-node), **MP-C-12** (E2E), **MP-C-14**
  (star+mesh — fixed-N per rung, topology-width via the Nodes axis), **MP-C-15** (restart+replay),
  **MP-C-16** (migration); **MP-A-06** (equivocation), **MP-A-11** (oversized payload, fixed-N),
  **MP-A-13** (anti-transitivity), **MP-A-21** (MLS rollback).
- **Tranche (c) — infra:** the **late-federation/catch-up** machinery (D5; MP-C-15/16 + MP-A-01(ii))
  and the **connection-churn** driver (MP-A-18, MP-A-19).

Arc-local (D-069).

---

## 4. MP-R2-D2 (F-2) — sustained-window + intensity via pre-generated PACED batches

**Decision: pre-generate batches (verbatim-feed invariant preserved); add inter-send pacing to the
batch driver; the window lives in the batch, not in `settle()`.** A sustained-window scenario is a
**paced pre-generated batch** spanning the window duration; intensity/rate is the pacing parameter.

- **Pre-generate (F-2).** A test-crate batch generator (`template + params → concrete .jsonl
  lines`) emits the saved batches the harness already feeds **verbatim** (no inline generation in
  the hot path — preserves the matrix §2 invariant + `batch.rs`'s "send as written"). The generator
  is offline/at-scenario-load; the drive path is unchanged-shape.
- **Pacing is net-new (the §2 falsification consequence).** Because `run_actor` fires back-to-back
  ([`batch.rs:107-159`](../xgen-mptest/src/batch.rs#L107)), a window/rate needs an inter-send
  cadence. **Lock: a per-line optional `after_ms` honored by `run_actor`** (the smallest faithful
  addition — the batch stays verbatim + self-describing; the generator stamps the cadence; a line
  with no `after_ms` sends immediately, so R1 batches are byte-unchanged). The same mechanism on
  the injector loop gives MP-A-07 its flood pace. A *rate sweep* = batches regenerated at
  decreasing `after_ms` (or increasing burst) per rung.
- **`settle()` survives unchanged — confirmed, not amended (F-2's "confirm or amend").** The long
  part of a sustained scenario is the **drive** (the paced batch over the window); `settle()`
  ([`runner.rs:437`](../xgen-mptest/src/runner.rs#L437)) runs **after** the drive and only waits for
  the final fan-out to quiesce (poll-until-stable, 15 s ceiling). The window does not live in
  `settle()`, so its 15 s cap is adequate post-drive quiescence regardless of window length. **No
  windowed/timed-drive model is needed** — the window is the paced batch's length. (Recorded
  explicitly so the runbook does not re-litigate it.)

Arc-local (D-069).

---

## 5. MP-R2-D3 (F-3 + G-6) — the dial → `run_scenario` spawn bridge (Nodes + Clients, sweep-rows-only)

**Decision: `run_scenario` consumes `dial.nodes` + `dial.clients` for sweep rows only; the manifest
stays authoritative for fixed-N rows; `worker_threads` is consumed from the dial (G-6).** This is
the structural heart (the §2 finding: the sweep axis is inert until the dial reaches the spawn loop).

- **Sweep-rows-only consume (F-3, exactly as locked).** The topology-authority note
  ([`runner.rs:46-49`](../xgen-mptest/src/runner.rs#L46)) is amended **for the sweep path only**:
  fixed-N (b)-tranche rows spawn per `manifest.nodes`/`manifest.actors` (unchanged); (a)-tranche
  sweep rows scale the spawn count from `dial.nodes`/`dial.clients`. With residents deferred (D1),
  the bridge needs **nodes + clients only** — not residents.
- **The F-2 ↔ F-3 coupling (grounded refinement).** You cannot spawn `dial.clients` processes
  without saying what each *does*: the manifest names specific actors with specific batch files. So
  a sweep row is a **templated actor** the runner instantiates ×`dial.clients`, each with a
  **generated batch** (D2) — `dial.clients` drives spawn count **and** batch generation together.
  (This is why F-2 and F-3 are one mechanism for the (a)-rows, not two independent ones.) Locked
  shape: a sweep scenario carries a base actor/node template; per rung the runner generates the
  concrete N actors + their paced batches, spawns them, runs `run_scenario`. Fixed-N rows carry an
  explicit manifest as today.
- **G-6 `worker_threads` — consume from the dial (no dead knob).** `RoundDial.worker_threads`
  ([`dial.rs:82`](../xgen-mptest/src/dial.rs#L82)) is decorative today (spawn reads
  `XGEN_MPTEST_WORKER_THREADS`, [`process.rs:83`](../xgen-mptest/src/process.rs#L83)). **Lock:
  consume it** — the spawn path reads `dial.worker_threads` (env var as override/fallback), making
  the dial the single source for spawn-scale knobs. (Retiring the field was the alternative; consume
  is chosen so the dial is complete + the env override is preserved.)

Arc-local (D-069).

---

## 6. MP-R2-D4 (F-4) — CEILING hardening = caveats 1 + 5 only

**Decision: R2 hardens the CEILING classifier on caveats 1 + 5; caveats 2/3/4 defer to R3.**
The classifier is already wired + fed (audit G-5); R2 calibrates it + closes the one real mislabel
path.

- **Caveat 1 — bench-calibrated floors.** Wire the `bench.rs` `BoxCeilingReport`
  ([`bench.rs:91`](../xgen-mptest/src/bench.rs#L91) `estimated_ceiling`,
  [`bench.rs:82`](../xgen-mptest/src/bench.rs#L82) `reference_mean_rss_bytes`) output into
  `RSS_WALL_BYTES` / `THREAD_THRASH_COUNT` (today the coarse first-pass 1.5 GB / 64,
  [`sweep.rs:54-60`](../xgen-mptest/src/sweep.rs#L54)). The floors become bench-derived (e.g.
  RSS-wall = a multiple of the measured mean RSS; thread-thrash off the measured steady-state),
  not hardwired. The **first RUN step is the bench** (`XGEN_MPTEST_BENCH_TIERS=10,50,100`) → its
  report calibrates the floors **before** any sweep runs (§9 RUN gate).
- **Caveat 5 — sampler-failure-under-a-failed-rung = ceiling-suspect (closes the one real mislabel
  path).** Today `classify_rung` returns the conservative `LogicFault` when a failed rung has
  `resource == None` ([`sweep.rs:161-164`](../xgen-mptest/src/sweep.rs#L161)). But `Get-Process`
  sampling is most likely to fail **under memory pressure** — exactly when a true `Ceiling` should
  fire — so "None → LogicFault" is the one path the HANDOFF's feared "OOM mislabels as protocol
  broke" can take. **Lock: a failed rung with no resource sample is classified `Ceiling`
  (ceiling-suspect), recorded `resource-sample-unavailable: ceiling-suspect`, not silent
  `LogicFault`.** This **reverses the R1 single-rung default** — recorded as a deliberate R2 change
  (the R1 conservative-LogicFault was right when sampling was reliable on a single small rung; at
  R2 scale, absence-of-sample on a fail is ceiling-evidence). The runbook decides the exact shape
  (a `Ceiling`-suspect variant/flag vs the None-branch flip); the contract is: **never silently
  LogicFault a failed rung whose sampler died.**
- **Defer to R3:** caveat 2 (continuous/peak-during-drive sampling), caveat 3 (aggregate-RSS-vs-
  box-RAM + OOM-by-exit-code), caveat 4 (injector/harness-process sampling). Recorded as R3 CEILING
  enrichments.

Arc-local (D-069).

---

## 7. MP-R2-D5 (F-5) — late-federation/catch-up machinery as R2 infra

**Decision: build the late-federation/catch-up machinery as R2 infra, shared across MP-C-15
(restart → catch up), MP-C-16 (migration cutover), and MP-A-01(ii) (aged-Space replay); MP-A-01(ii)
rides it rather than staying a standalone PENDING.** The G-6 bootstrap establishes federation
**early** (`runner.rs` seeds add-peer before any identity registers); the catch-up machinery is the
inverse — a node that federates (or restarts, or receives a migrated Space) **after** the Space
already has history, then catches up via the existing sync path.

- **The machinery (runner extension, test-crate-only).** A director capability to run a
  **federate/join-after-history** sequence: the owner's batch builds (and, for MP-A-01(ii), the
  `[[clock]]` ages) the Space first; *then* the late node's `add-peer` + `federation initiate` fire
  (or a killed node restarts, or a migration cutover completes), and the runner asserts the late
  node converges to the aged Space via catch-up. This is sequencing + lifecycle control over the
  existing federation/sync seams — not a new protocol surface.
- **MP-A-01(ii) rides it.** The R1-recorded PENDING ("needs a cross-node catch-up where B federates
  AFTER the clock ages the Space; the fixed G-6 bootstrap establishes federation early") is exactly
  this machinery. With it, A-01(ii) becomes runnable as an R2 row (membership preserved on the
  catching-up peer; admission-only gate not re-rejecting on replay — the J-298 property, now real-
  binary witnessed). No longer a standalone harness-timing gap.
- **MP-C-15 (restart) + MP-C-16 (migration)** consume the same kill/restart + cutover + catch-up
  primitives. (Connection-churn for MP-A-18/19 is a *separate* infra item — D1 tranche (c) — not
  this catch-up machinery.)

Arc-local (D-069).

---

## 8. MP-R2-D6 — scope, scenario set, defect policy, honest boundary

**The R2 scenario set (corrected count — carry-in #1).** **14 R2 rows = 7 cooperative + 7
adversarial**, plus **MP-A-01(ii)** as a carried infra-borne row, with **MP-A-08 reassigned to R3**
(matrix §4 authoritative; the audit §6 "8+7=15" header was a transcription slip its own count-note
already corrected to 7 cooperative — fixed cleanly to 14 here).

- **Cooperative (7):** MP-C-04, MP-C-05, MP-C-11, MP-C-12, MP-C-14, MP-C-15, MP-C-16.
  *(MP-C-05/11/14 carry an R3 entry-rung continuation.)*
- **Adversarial (7):** MP-A-06, MP-A-07, MP-A-11, MP-A-13, MP-A-18, MP-A-19, MP-A-21.
  *(MP-A-07/18 carry an R3 continuation.)*
- **Infra-borne:** MP-A-01(ii) (rides D5).
- **Reassigned out of R2 → R3:** MP-A-08 (partition + reconnect storm).

Grouped by the D1 tranche split: **(a)** MP-C-05, MP-C-11, MP-A-07 · **(b)** MP-C-04, MP-C-12,
MP-C-14, MP-C-15, MP-C-16, MP-A-06, MP-A-11, MP-A-13, MP-A-21 · **(c) infra** late-federation/
catch-up (+ MP-A-01(ii), MP-C-15, MP-C-16) and connection-churn (MP-A-18, MP-A-19).

**Change surface (all in `xgen-mptest` + scenario dirs — test-crate-only, F-1 defer holds this).**
The dial-spawn bridge + templated-actor instantiation (D3); the paced batch generator + `run_actor`
`after_ms` pacing (D2); the bench→floors calibration + the ceiling-suspect classifier change (D4);
the late-federation/catch-up director capability (D5); the connection-churn orchestrator driver
(D1(c)); the 14 R2 scenario dirs. **No production crate is touched** (the residents multiplexer —
the one item that would have — is deferred to R3 per F-1).

**Defect policy — surface-and-route (D-065/D-084), never patch-in-place.** A row that surfaces a
genuine protocol defect under load (a lost admitted event, a convergence failure under churn, a
reconnect deadlock, an MLS epoch regression) routes to `tasks/MP_findings.md` + its own fix-arc;
it does **not** block the R2 break-point record. The deliverable is the **curve + break-point per
volume axis**, oracle-checked per rung, each non-GREEN rung classified GREEN/LOGIC-FAULT/CEILING —
a CEILING is a hardware finding, **not** a protocol FAIL.

**Honest boundary.** R2 proves the protocol holds under **moderate-heavy load + real time** and
finds the break-points off the nodes / clients / intensity / connection-churn mechanisms; it is
**not** the chaos capstone (R3) and **not** density-per-process (R3 residents). A green R2 is a
scale floor on the freed box, read against the bench-calibrated ceiling — not an unbounded-scale
claim.

**RUN gate (held).** The freed box is the M-R2.3 RUN gate, not the design/runbook gate. **First RUN
step: the `bench.rs` box-ceiling micro-benchmark** (`XGEN_MPTEST_BENCH_TIERS=10,50,100`) — its
report calibrates the D4 floors **and** fixes the real R2 (and R3) participant/node numbers, before
any sweep. Operational fences carry from the audit §7 (binary-clobber: workspace check before the
harness-control build; spawn-timeout flakes classified distinctly from protocol RED, Rule 2).

Arc-local (D-069).

---

## 9. Next

**Joe-lock MP-R2-D1..D6** (with the §2 falsification blessed — the two-climb-mechanism + connection-
churn-infra refinement of F-1's axis enumeration) → the MP-R2 runbook
(`tasks/MP_R2_SCALE_IMPL.md`): the dial-bridge + batch-gen/pacing commit, the CEILING calibration
commit, the late-federation/catch-up + connection-churn infra commits, then the (a)/(b) scenario
tranches → Clair. Commit order: Clair's code/arc-doc FIRST, Chat's doc-bridge separate, Joe pushes
both. No self-close.

Per D-065 + D-069 + D-071 + D-074 + D-084 + MP-R1-D2 (the sweep contract R2 inherits) + MP-R1-D8
(honest boundary) + MP-R1-D10 (surface-and-route).
