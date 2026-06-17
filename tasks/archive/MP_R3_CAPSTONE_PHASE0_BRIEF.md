# MP-R3 (capstone) — Phase-0 brief (frame for Clair's grounding)
> **Status**: COMPLETED  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-11  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. What this is

The round-open frame for **MP-R3**, the capstone of the Multiparty-tests milestone. Chat-authored
architecture/scope; it is **not** the grounded audit. It hands Clair the round definition, the
dependency cluster map, an ordered grounding checklist, and the one open Joe-lock to resolve at
Phase-0 close. Clair's first Phase-0 commit is the grounded `tasks/MP_R3_CAPSTONE_AUDIT.md`
(vs live `main`, to file:line) that executes this checklist. No code until the runbook is Joe-locked.

State at open: MP-R1 ✅ CLOSED (J-340), MP-R2 ✅ CLOSED (J-348). The Multiparty milestone stays
🟢 PLAY — R3 is the last sub-pass. R3 inherits the **loop-to-green BOUNDED-gate rerun character**
(R1 J-322 → R2 J-344 → R3): a box-gated RUN surfaces findings → a scope-frozen gate → rerun → close.

## 1. Round definition

Capstone: **max the box bears, chaos overlay stacked.** The handoff cites a ~1,562-process ceiling;
that figure is **stale-suspect** (R2's bench measured ~1288 processes, memory-bound). Per R2's D4
lesson, the first RUN step re-benches the box — **no scale number is inherited as gospel.**

## 2. The four named inbound deps — they CLUSTER, not list

The deps pull in three directions; treating them as a flat list is the first scoping error to avoid.

- **Cluster A — catch-up / federation-depth (protocol):** **MP-F11** (F-3 late-third-node, regular-
  Space generalization of MP-F1b/Design-Z — D-091 invariant E + the repopulate hook +
  `drain_pending_by_federation_relationship`) **+ MP-A-08** reconnect. Same underlying problem:
  catch up events **and** the identities registered during the gap. F9-D2 was already built
  "R3/MP-A-08-free" with this in view. MP-F11 = the protocol fix; MP-A-08 = its adversarial stress.
  MP-A-01(ii) is MP-F11's first witness row.
- **Cluster B — multi-node adversary injector (capability):** **MP-A-08 + MP-A-06.** Both need
  net-new two-node / multi-target injection, link/partition control, and a convergence-on-winner
  oracle. MP-A-06 (equivocation) was re-routed R2→R3 precisely because it is the MP-A-08 class.
- **Cluster C — identity-discovery (protocol, deepest):** **MP-F13** alone — the J-278
  `home_node` = WS-URL-not-pubkey-id NodeXgid contract violation; same root as the long-horizoned
  F1B-D5 "production identity→home-node discovery." MP-C-16 (migration) is its first witness row.
  **This is the dep that can balloon the round.**

Also in R3 scope (from the R2 record): MP-A-07 flooding intensity **curve** (the curve, not the
liveness witness); **residents-multiplexing** (deferred to R3 at the R2 design lock — the net-new
orchestrator infra for >1 logical client per process).

## 3. Grounding checklist (ordered — Clair's audit executes this)

1. **Scale spine / re-bench.** Re-run the box-ceiling benchmark (R2's `bench.rs` pattern). Ground
   whether capstone scale **requires residents-multiplexing** (>1 logical client/process) or runs
   one-process-per-actor at the real box wall. This is the load-bearing net-new infra and the
   biggest risk. Do not assume 1,562 or 1288 — measure.
2. **Dep clustering + per-dep requirements.** Ground each dep's real surface to file:line; confirm
   the A/B/C cluster map; for each, state what "fix" actually costs. Output feeds the fix-vs-route
   Joe-lock (§4).
3. **Chaos overlay.** Ground what "stacked" means concretely: can the orchestrator **compose**
   fault-injection (partition + equivocation + flood-curve) on top of the scale dial, vs isolated
   rows? What is the oracle under chaos (convergence-after-heal, liveness-under-churn)?
4. **R3 row enumeration (D-078, production-grounded).** Enumerate the R3 scenario set against the
   matrix PENDING/R3 rows + the four deps' witness rows (MP-A-01(ii)→F11, MP-C-16→F13, MP-A-07
   curve, MP-A-08, MP-A-06), grounded against live code, not inferred.

## 4. THE open Joe-lock (resolve at Phase-0 close, NOT pre-locked)

**Fix-vs-route for {MP-F11, MP-F13}, decided BEFORE the RUN** — so the loop-to-green bounded gate is
reserved for newly-surfaced findings, not the known-carried deps.

- Chat's recommendation (recorded, **not** locked): **MP-F11 fixed in-round** (it is the catch-up
  infra many R3 rows ride — the role R2's D5 played) · **MP-F13 carried as a named non-green**,
  routed to the F1B-D5 home arc (MP-C-16 stays red-with-reason — the R1 MP-C-06 / R2 MP-C-16
  precedent), so R3 stays closeable and the capstone does not balloon into "solve identity
  discovery."
- This is a Phase-0-close decision: Clair grounds §3.2, Chat frames the forks, **Joe locks.**

## 5. Standing — do NOT pull into R3 (named homes)

MP-C-06 re-home → M10. MP-F6 (swallowed apply-error) → M10. MP-F12 (departed-signer re-dispatch) →
own home (peer/identity-discovery space). Production identity→home-node discovery (F1B-D5, now
joined by MP-F13) → own arc.

## 6. Discipline + next step

- Surface-and-route (D-065 / D-084); **pin-by-observation BEFORE routing** (the MP-R2 bar — three
  falsifications earned it).
- Two-commit: Clair's audit/design arc-docs commit FIRST (pushed), then Chat's Phase-0-close
  doc-bridge (the J-NNN that flips CLAUDE.md PLAY + ROADMAP). Joe pushes. Chat never pushes.
- **Next step: Clair authors `tasks/MP_R3_CAPSTONE_AUDIT.md`** — grounds §3's four items vs live
  `main` to file:line; outputs the cluster-map confirmation + the re-bench + the fix-vs-route
  recommendation. Then `tasks/MP_R3_CAPSTONE_DESIGN.md` (R3-D1..Dn) → Joe-lock → runbook → RUN.

## 7. Entry point (Rule 0)

CLAUDE.md PLAY (J-348 MP-R2-CLOSED head) → JOURNAL J-348 → `tasks/HANDOFF_MP_R3.md` → this brief →
`tasks/MP_findings.md` (R3 named deps MP-F11/F13; fix-phase note CLOSED) →
`docs/tests/MULTIPARTY_TEST_MATRIX.md` §6 → `docs/ROADMAP.md` Multiparty node.
