# MP-R1 — Multiparty-tests Round 1 (deterministic correctness floor): D-071 Phase-0 Audit
> **Status**: COMPLETED  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-10  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. What this is + where it sits

The **Multiparty-tests** milestone (unnumbered, strategic) **runs** the M9 `xgen-mptest`
harness through an escalating three-round ladder on a finalized binary. Per the Joe-locked
structure (2026-06-07), the milestone decomposes into three numbered sub-passes of
**monotonically increasing weight**:

- **MP-R1 — deterministic correctness floor** (this audit). Light, MockClock, fixed seeds, the
  fix→rerun loop. Proves every logic/wire scenario converges or rejects **correctly** before any
  load is added. Runnable on the current box now.
- **MP-R2 — scale + real-clock, moderate-heavy.** Continuation, not a fresh start: begins where R1
  ends and climbs. Needs a freed-up box.
- **MP-R3 — capstone.** Maximum the box bears (~1,562-process ceiling, `M9_findings.md` §5), the
  chaos overlay stacked. One-shot, full capture.

This is a D-071 Phase-0 audit for **MP-R1 only**: it grounds what R1 depends on against live
`main`, enumerates the R1 scenario set, and frames the forks for the MP-R1 design phase. The
**sweep contract** (§5) is recorded here even though R1 is mostly fixed-N, because R1 locks the
contract R2/R3 inherit.

"Finalized binary" = the **convergence / federation / MLS protocol core** (M1–M9.2, shipped),
which is exactly what the matrix exercises. M10 (auth module) and M11 (`self`) layer on top
without touching ordering/resolution, so the surface under test is finalized for these runs.

---

## 2. What MP-R1 depends on (the crossing)

R1 runs the **existing** `xgen-mptest` harness (built M9, C1–C5) against the **existing** F2/F3/F4
fenced seams (shipped M9.2). Both are in place. What is **not** in place is the glue that turns a
scenario manifest into a driven multi-node run. M9 Round-0 proved the *machinery* by hand-wiring
two smokes (`c5_mp_c_02`, `c5_mp_a_05`); R1 needs a **general scenario runner** that consumes a
`Scenario` (manifest + batches) end-to-end. That runner is R1's primary build.

Seam invocation surfaces (grounded, M9.2, require a `--features harness-control` node build):
- **F2** — `federation add-peer {node_id, url, spaces}` (upserts a `FederationRelationship`) then
  `federation initiate {peer_node_id}` (dials + replicates). Confirmed live in `m9_2_f2_add_peer`.
- **F3** — `clock advance {duration}` / `clock set {timestamp}`, each returning `data.now_utc`.
  Confirmed live in `m9_2_f3_clock`.
- **F4** — test-only raw malformed-frame client in `xgen-mptest::injector` (no fence, no
  production surface). Confirmed live in `m9_2_f4_malformed_frame`.

---

## 3. Grounding findings against live `main`

**G-1 — No end-to-end `run_scenario` orchestrator exists.** The harness exposes `run_actor`,
`run_microbench`, `run_init` — but nothing that consumes a parsed `Scenario` and drives a full
run. The manifest schema (`manifest.rs`) declares `[[federation]]` links (`from`/`to` node
labels, "links to establish before driving actors") and M9.2 shipped the `add-peer`/`initiate`
verbs — but **the runner never calls them**; there is no glue between the manifest field and the
seam. Round-0's two smokes were hand-coded per test. *R1's primary build is this runner:* spawn
topology → establish federation links (the F2 bootstrap ordering, see G-4, generalized) → spawn
actors → drive batches (with `exports`/`waits`/`barriers` already in `resolve.rs`/`manifest.rs`)
→ run the oracle (`oracle.rs`) → emit the matrix result.

**G-2 — `dial.rs` is stale (pre-M9.2).** `ClockMode::Mock` is declared **not operable** and
`RoundDial::validate()` **rejects** it, with a module note saying a clock-advance surface "does
not exist yet." M9.2 F3 made it operable. R1 is the deterministic round = MockClock, so R1 must
(a) make `ClockMode::Mock` valid when the node is a `harness-control` build, (b) wire the dial's
clock mode to the `clock advance`/`set` verbs, and (c) sweep the now-stale doc comments (there is
also a stale "`federation initiate` control surface that does not exist yet" comment in the src).

**G-3 — No clock-control step in the scenario format.** MP-A-01 (expired-invite federation
replay) requires advancing the clock *between* commands ("invite … clock advances past
`valid_until` … replay"). The manifest has no field and the batch JSONL has no verb for a
scenario-level clock action. This is net-new: either a manifest `[[clock]]` step list (ordered
against exports/barriers) or a batch-embeddable clock directive. Design-phase fork (F-C).

**G-4 — State-isolation hazard (shared `spaces_dir`).** Spawned nodes default `spaces_dir` to a
**shared** `<exe_dir>/spaces` (`NodeConfig::default` → exe_dir, pre-existing binary behaviour,
out of scope to change here). So a node may host spaces accumulated by prior runs, polluting any
**absolute**-state oracle. Both Round-0 smokes dodged this by asserting on a unique fresh Space
(F2: count *increase*; MP-C-02: its own Space id-set). R1 must pick a policy (fork F-D): set a
per-instance `spaces_dir` from the harness, or mandate every oracle be **Space-scoped** (the
robust Round-0 pattern). Until decided, R1 scenarios must not assert on absolute node state.

**G-5 — `residents_per_process` multiplexing is unbridged.** The dial declares the cheap volume
axis (many logical participants per process), but the runner spawns **one client process per
actor** (manifest `[[actors]]`). Irrelevant to R1 (light, one participant per actor), but recorded
here as an **R2/R3 prerequisite** so it is on the milestone record, not rediscovered later.

**G-6 — Federation bootstrap ordering is load-bearing (carried from J-315).** The F2 smoke
established that a cross-node Space needs a specific order: seed peer → register identity (so
`push_identity_to_peers` replicates it before any signed event) → create Space → re-seed the
relationship with the Space id → `initiate`. The `run_scenario` runner (G-1) must encode this
ordering as the canonical federation-establishment sequence, not leave it to each scenario. This
is harness ordering (legitimate), not a binary change.

---

## 4. Forks for the MP-R1 design phase (Joe-lock)

- **F-A — scenario-runner shape.** One generic `run_scenario(Scenario, RoundDial)` covering both
  families, vs per-family runners. *Lean lean: one generic runner;* adversarial wire-attacks
  branch into the injector path inside it (the matrix already separates batch-expressible logic
  attacks from raw-wire attacks).
- **F-B — the sweep contract.** Where `start / step / max / stop-on-fail` lives. The dial today is
  a **single point**; the sweep is a new layer (a sequence of `RoundDial`s, oracle-checked per
  rung, result = curve + break-point). R1 is mostly fixed-N, but the contract is locked here so
  R2/R3 inherit it. *Rec: a `Sweep` wrapper iterating `RoundDial`, with the per-rung oracle as the
  stop condition.*
- **F-C — scenario clock-control surface (G-3).** Manifest `[[clock]]` step vs batch clock verb.
  *Rec: a manifest-level ordered `[[clock]]` step keyed to exports/barriers — keeps the per-actor
  JSONL purely actor-driven and the clock a scenario-level director action.*
- **F-D — state isolation (G-4).** Per-instance `spaces_dir` vs Space-scoped oracle only. *Rec:
  Space-scoped oracle as the contract (matches the proven Round-0 pattern, no binary change), and
  investigate a per-instance `spaces_dir` override as a hardening follow-on if feasible without
  touching the binary.*
- **F-E — the R1 scenario set (§6).** Confirm which PENDING scenarios are R1.
- **F-F — defect policy (restated).** Surface-and-route, not patch (M9 D-065/D-084 discipline): a
  FAIL becomes a routed finding in a findings file and does **not** block MP-R1 close; the run is
  the deliverable. *Confirm it holds.*

---

## 5. The round ladder + sweep contract (locked shape, recorded for R2/R3)

Weight is a **continuous sweep within each round** and a **monotonic climb across rounds** — the
dial sweeps, it does not step between three fixed presets.

- **Within a round:** the volume parameter walks a sequence (e.g. N = 10 → 25 → 50 → 100 …),
  oracle-checked at each rung, climbing until the round ceiling or first failure. The break point
  **is** the deliverable ("converged clean to N, degraded at N′"), not a bare PASS/FAIL.
- **Across rounds:** each round starts where the prior ended and climbs higher — one continuous
  ascent R1 → R2 → R3 toward the ~1,562-process box wall.
- **R1's place on the ladder:** the floor. Smallest viable N, MockClock, fixed seeds — weight is
  *not* the goal; unambiguous determinism is. R1 establishes the correctness baseline every
  heavier rung is measured against.

**Mandatory oracle distinction (D-065).** A sweep finds *this box's* ceiling, not the protocol's.
The oracle MUST distinguish a **logic fault** (non-convergence, lost admitted event, wrong
rejection) from a **box-ceiling artifact** (OOM, scheduler thrash, RSS wall) — or the curve will
mislabel "ran out of RAM" as "protocol broke." This distinction is a hard requirement on the
sweep result schema.

---

## 6. The R1 scenario set (from the matrix, Round = R1)

Deterministic logic + wire scenarios whose first round is R1. **22 scenarios**; 2 already PASS at
Round-0 (single-node) and are promoted to their true cross-node form here.

**Cooperative (9):** MP-C-01 local fan-out · **MP-C-02** invite & join (✅ Round-0 single-node →
true cross-node A↔B now F2 is live) · MP-C-03 concurrent send under conflict (cross-node) ·
MP-C-06 identity re-home · MP-C-07 DM space · MP-C-08 multi-room + per-room overrides · MP-C-09
ban → converge → post-rejected · MP-C-10 leave & rejoin · MP-C-13 thread create/resolve/archive.

**Adversarial (13):** MP-A-01 expired-invite federation replay (needs F3 clock) · MP-A-02
over-ceiling/expired invite at submission · MP-A-03 tier-gate join refusal · MP-A-04
unauthorized/non-member send · **MP-A-05** signature forgery (✅ Round-0) · MP-A-09
duplicate-event_id dedup · MP-A-10 causal gap / missing-parent · MP-A-12 malformed frame (needs
F4) · MP-A-14 ban-evasion via new identity · MP-A-15 clock-skew timestamp (M9.1-resolved at the
core — re-confirm rejection 3046 at the binary boundary) · MP-A-16 forged invite · MP-A-17
wrong-space_id confusion · MP-A-20 privilege escalation.

The remaining 13 (MP-C-04/05/11/12/14/15/16, MP-A-06/07/08/11/18/19/21) carry an R2/R3 first
round (scale, real-clock, volume, chaos, MLS-epoch) and are **out of MP-R1 scope** — deferred to
MP-R2/MP-R3.

---

## 7. Scope, honest boundary, defect policy

**MP-R1 scope:** the general `run_scenario` runner (G-1) + the dial/clock un-stale + clock-control
surface (G-2/G-3) + the state-isolation policy (G-4) + the federation-bootstrap sequence (G-6) +
authoring the R1 batch set (§6) + running R1 to a recorded result per scenario in the matrix +
routing any defect as a finding. **Out of scope:** any binary/protocol change (findings route
out); the sweep *mechanics* beyond locking the contract (R2/R3 build them); `residents_per_process`
multiplexing (G-5, R2/R3).

**Honest boundary (D-065):** R1 proves *correctness* on the protocol core under **no load**. It
does not prove scale (R2/R3), and it tests *this* protocol surface, not a future one. A
green R1 is a correctness floor, not a coverage or robustness claim.

**Defect policy (D-065 / D-084):** the run **surfaces** defects; it does **not** patch the
binaries. Each defect is a routed finding (a new `tasks/MP_findings.md`, mirroring
`M9_findings.md`); a FAIL does not block MP-R1 close. The recorded result — PASS, or FAIL→routed
finding — is the deliverable.

**Suite:** 1271/0/11 (no code this phase). No DECISIONS change (MP-R1-D# arc-local, D-069).

Per D-065 + D-069 + D-071 + D-084.
