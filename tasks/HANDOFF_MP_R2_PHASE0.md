# XGen Protocol — HANDOFF: MP-R2 Phase-0 (Chat → Code Claude)
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-10  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. Purpose

Chat-authored handoff to open the **MP-R2 (scale + real-clock) D-071 Phase-0**, following MP-R1 ✅ CLOSED (J-340, HEAD `a9fbd98`). Claire (Code Claude) consumes this to author the Phase-0 audit grounded against live `main`, surfacing forks for Joe-lock. This doc → COMPLETED when consumed. Chat-only record ahead of code (J-334 / J-323 exception class); CLAUDE PLAY / JOURNAL / ROADMAP bridge lands at the Phase-0 **lock**, not now.

## 1. Precondition (read carefully)

The **freed-up box is the RUN gate (M-R2.3), not the Phase-0 gate.** Phase-0 is read-only grounding against live code and proceeds now. No heavy multi-binary run starts until Joe confirms the box is free. Hold for Joe's go before driving any heavy sweep.

## 2. Deliverable

Author `tasks/MP_R2_SCALE_AUDIT.md` (proposed name, parallel to `MP_R1_DETERMINISTIC_AUDIT.md`; Joe-lockable). D-071 Phase-0: ground each ask below to a **verdict** (wired / net-new / stubbed), not an assumption. Surface forks; lock with Joe before design.

## 3. Grounding asks (verdict-pending)

1. **`residents_per_process` multiplexing** — declared in the dial, unbridged in R1 (one process per actor). Verdict: wired vs net-new. The R2 scale prerequisite; if net-new it likely dominates R2 build cost.
2. **Real-clock path** — R1 pinned MockClock. Confirm `RoundDial` + dial-validation route real-clock through the same runner/sweep, or it is a fork.
3. **Sweep multi-rung climb** — R1 built `Sweep`/`SweepResult` but ran single-rung only. Ground implemented-vs-stubbed; the **break-point per volume axis** (oracle-checked per rung) is the deliverable, not a bare pass/fail.
4. **R2-vs-R3 row split** — reconcile (see §4); confirm which matrix rows are R2 vs R3.
5. **CEILING-vs-LOGIC-FAULT classifier** — confirm `resource.rs` RSS/thread `ResourceSample` is actually consulted at the non-GREEN-rung decision, or "OOM" mislabels as "protocol broke."

## 4. Named fork (Chat-caught) — R2/R3 row reconciliation

The carried R2 set and the canonical matrix tags **disagree**. Resolve as a Phase-0 fork for Joe-lock; do not paper over.

| Row | Matrix §4 tag | Carried-set placement | Action |
|-----|---------------|-----------------------|--------|
| MP-A-08 (partition + reconnect storm) | **R3** | listed under R2 | reconcile — matrix says R3 |
| MP-A-07 (high-rate flood) | **R2 → R3** | R2 | confirm R2 entry-rung scope |
| MP-A-18 (connect/disconnect storm) | **R2 → R3** | R2 | confirm R2 entry-rung scope |
| MP-A-11/13/19/21 | **R2** | R2 | agree — confirm |
| MP-A-06 | (earlier row, unread here) | R2 | ground the tag |
| MP-C-04/05/11/12/14/15/16 | (earlier rows, unread here) | R2 | ground each tag against §4 |
| MP-A-01(ii) | carried (late-federation harness machinery) | carried | not strictly R2 scale — confirm placement |

## 5. Operational fences (carried from the R1 close, sharper at scale)

- **Binary-clobber hazard:** `cargo test --workspace` rebuilds `xgen-node` default-features over the `harness-control` binary at the pinned target dir → heavy tranches fail all-`UNKNOWN_COMMAND` (J-315 fence signal). Run the workspace check BEFORE the harness-control build, or rebuild harness-control after any workspace build.
- **Spawn-timeout flakes:** R1's MP-C-10 failed once on an aicontrol pipe-connect timeout under peak parallelism (passed isolated). At R2 scale contention grows — handle spawn-timeout flakes distinctly from protocol RED (Rule 2: confirm-before-classify, re-run isolated).

## 6. Discipline

Phase-0 → design → Joe-lock → runbook → run. Surface-and-route defects (D-065 / D-084), never patch-in-place. Commit order: Clair's code FIRST, Chat's doc-bridge separate, Joe pushes both (PS — GitHub Desktop login down). No self-close; milestone close is Joe's lock.

## 7. Exit

Phase-0 verdicts on §3 + §4 resolved → forks surfaced → Joe-lock → `tasks/MP_R2_SCALE_AUDIT.md` ACTIVE, this handoff → COMPLETED.
