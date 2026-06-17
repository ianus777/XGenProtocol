# M8 — Multiparty Strong-Test — Implementation Runbook
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-05  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. Purpose & how to use this runbook

Implementation plan for M8, below the design lock (`tasks/M8_MULTIPARTY_DESIGN.md` v1.1).
Clair works this; Chat Claude holds cross-file edits while Clair is on a file (one writer per
file per atom). The arc is three waves (C1–C7) + close; **each wave boundary is a Joe-lock
checkpoint** — Clair stops, reports, and waits for Joe before the next wave.

**Discipline reminders (baked in):**
- DoD checklists **never** list "commit pushed" — the `Status: COMPLETED` header on the
  scenario/findings file is the real shipped signal (task-file convention).
- M8 is the **diagnostic that scopes M9** (M8-D4): a scenario that surfaces a real weakness is
  a **success** (record it, feed M9) — do **not** redesign in-arc. Only a *clean, contained*
  bug gets an S1-style in-session fix; anything structural is a finding, not a fix.
- Binary-level scenarios run **real** `xgen-node.exe` / `xgen-client.exe` under `test_runs/`,
  driven by `.xgb` scripts in `docs/tests/scripts/`, with results in `MULTIPARTY_Sn_findings.md`
  (S0 convention). Build output is `C:/cargo-targets/XGenProtocol` (`CARGO_TARGET_DIR`).
- Every scenario records the four metrics (§7): M1 delivery, M2 convergence-correctness
  (headline), M3 integrity, M4 latency (informational). Throughput is **not** measured.
- A/B per M8-D3: S1 carries the historical-A delta; S2–S8 establish their own baseline.

---

## 2. Pickup checks — run FIRST, at the top of C1 (D-078)

- **CP-1 `.xgb` verb-surface drift.** Reconcile every command in the 13 existing S1 scripts
  (`docs/tests/scripts/*.xgb`) against the **current** client CLI (`xgen-client/src/main.rs`
  clap surface) + node CLI. The S1 scripts assume the `7e06896` surface; M6/M7 moved verbs.
  Produce a verb-delta note before running anything.
- **CP-2 harness ceiling.** Grounded: the federation harness already reaches **3 Nodes**
  (`xgen-node/src/tests/phase9_three_node_anti_transitivity.rs`,
  `phase9_m8_convergence_smoke.rs`). **3-Node = available; only S4's 4-Node is genuine
  scale-up.** Decide at C1 whether S4 needs the 4th Node or composes at 3 (M8-D6 / Joe-lock).
- **CP-3 B-record stamp.** Capture `xgen-node version` + `xgen-client version` + commit into
  every `MULTIPARTY_Sn_findings.md` "B" column (the S1 findings M0.2 pattern).
- **CP-4 per-scenario placement.** For each convergence/E2E correctness check, decide
  binary-level (real processes) vs workspace integration test (M8-D6). Default binary-level
  for anything an operator/UI would do; integration test only where real processes add no
  signal (e.g. a pure permutation-convergence assertion already provable in-process).
- **CP-5 AI-participant viability.** Confirm `--aicontrol` can drive a client that **holds a
  room membership** across a session. `--service` (headless resident) is a stub (audit §5). If
  a live membership is impossible without the resident mode, **fold S8 into a scripted S4
  variant** and record the limitation — do **NOT** build resident mode (M8-D2 non-goal).

---

## 3. Wave 1 — readiness + convergence

### C1 — readiness (M8-D5)
**Goal:** make the suite runnable against B; no scenario results yet.
**Steps:** run CP-1…CP-3; patch/regenerate any S1 scripts whose verbs drifted (keep originals
if still valid); confirm a clean `test_runs/` layout convention for M8 (suggest
`test_runs/m8_<scenario>/`); stamp the B build.
**Artifacts:** a short `docs/tests/scripts/` refresh + a readiness note (fold into
`MULTIPARTY_S0_intro.md` as an M8 addendum, or a brief `MULTIPARTY_M8_readiness.md`).
**DoD:** CP-1 verb-delta recorded · scripts run end-to-end against B without verb errors ·
CP-2 4-Node decision recorded · B-stamp captured · harness builds clean.

### C2 — S2 concurrent state-events → convergence-under-conflict (Arc C headline)
**Goal:** extend S2 from "concurrent message sends" to **concurrent state events** that
genuinely conflict, and prove M2 (byte-identical convergence) across Nodes + clients.
**Conflict cases (minimum):** concurrent ban vs join (same target); concurrent role change
(two admins); concurrent key-rotation. Two Nodes, ≥2 clients each.
**Steps:** author `MULTIPARTY_S2_*` extension + `.xgb` scripts (Appendix-style, verbatim
contents) → run all arrival permutations → assert each Node's resolved `SpaceState`
byte-identical + each client projection == its home Node (G-ALIGN) → record M1–M4.
**Honest-residue watch (M8-D4):** if a permutation diverges, that is a **finding** for M9
(record root layer 3/5a/5b vs 5c per the R2-F01 probe), not an in-arc fix.
**DoD:** `MULTIPARTY_S2_findings.md` COMPLETED with the conflict matrix · M2 verdict
(converged / divergence-finding) · G-ALIGN verdict · M1/M3 zero-or-characterized · M4 recorded.

**>>> Joe-lock checkpoint #1 (end of Wave 1).** Report C1+C2; Joe confirms before Wave 2.

---

## 4. Wave 2 — federation composite

### C3 — S3 topology + jurisdiction + migration (3 Nodes)
**Goal:** transitive propagation across 3 Nodes (chain + mesh), **plus** the two federation
capabilities that postdate the skeleton: jurisdiction policy + Space migration.
**Cases:** transitive event reaches a non-adjacent Node (S3 baseline); a cross-jurisdiction
Space is **rejected** at a peer per `allowed_jurisdictions` (Arc G); a `home_node` migration
flips on **both** Nodes mid-topology and clients keep converging (Arc F).
**DoD:** `MULTIPARTY_S3_findings.md` COMPLETED · transitive-delivery M1 · jurisdiction-reject
observed · migration `home_node` flip verified both sides · M2/M3 verdicts.

### C4 — S4 N×N chat + durability/replay; S5 rebind
**Goal:** composite real chat-room (target 4 Nodes / 6 Clients per S4 spec, or 3-Node per the
CP-2 decision) with the **G-DURABILITY** gate, plus the S5 identity-portability run.
**Cases:** sustained multi-client chat; **restart a Node mid-run** → replay from EventStore →
clients resync, zero orphans (G-DURABILITY); S5 client re-homes to a different Node keeping
identity (distinct from C3 migration).
**DoD:** `MULTIPARTY_S4_findings.md` + `MULTIPARTY_S5_findings.md` COMPLETED · restart-replay
resync verified · S5 identity preserved across rebind · M1–M4 recorded.

**>>> Joe-lock checkpoint #2 (end of Wave 2).** Report C3+C4; Joe confirms before Wave 3.

---

## 5. Wave 3 — orthogonal capabilities

### C5 — S6 E2E content-blindness (Arc H)
**Goal:** encrypted fan-out to N members at binary level; the Node never sees plaintext.
**Cases:** N members in an E2E-on Space exchange `enc:` v2 messages; assert zero plaintext in
Node logs/store (M3 content-blindness); KeyPackage pool consumed + replenished on multi-join;
epoch-advance on a `mls.commit`. (Single-committer happy path; commit-race is D3-fenced.)
**DoD:** `MULTIPARTY_S6_findings.md` COMPLETED · content-blindness asserted · KeyPackage
pool + epoch-advance observed under multiple joiners.

### C6 — S7 privilege enforcement (Arc D)
**Goal:** multiparty authority behaviour (NOT the auth matrix — M8-A7).
**Cases:** a tier-gated join is **refused** and all members observe the refusal/absence; a
per-Room override ("Mods can't post in #announcements") is enforced and observed by every
member. Synthetic Tier-1/Local-Node setup; no Auth Module ref set.
**DoD:** `MULTIPARTY_S7_findings.md` COMPLETED · tier-gate refusal multiparty-visible ·
per-Room override enforced + converged.

### C7 — S8 AI-driven participant (M7)
**Goal:** an `--aicontrol`-driven client as a first-class room member alongside humans.
**Cases (per CP-5):** if a live membership holds → S8 as its own scenario; if not → a scripted
S4 variant with the limitation recorded. Assert the AI participant's events fan out + converge
like any member; G-ALIGN holds for its projection.
**DoD:** `MULTIPARTY_S8_findings.md` (or S4-variant note) COMPLETED · AI participant
membership + convergence verified, or the CP-5 limitation recorded.

**>>> Joe-lock checkpoint #3 (end of Wave 3).** Report Wave 3; Joe confirms before close.

---

## 6. Close

Consolidate every scenario's findings into an M8 close record (`MULTIPARTY_M8_findings.md` or
the close section of S0), summarizing: per-scenario M1–M4, the headline M2 convergence verdict
across the suite, every divergence/weakness as an **M9-scoping input** (M8-D4), and the S1
A/B delta (M8-D3). Then: ROADMAP M8 🟡 OPEN → ⚫ CLOSED + version bump; CLAUDE.md PLAY flip;
JOURNAL close entry; M8-D# eval (arc-local unless a cross-arc invariant emerged); scenario
docs → COMPLETED. Same-commit atomicity (D-074). **Output: the diagnostic that scopes M9.**

---

## 7. Verification rigour & conventions

- **Metrics recording (every scenario):** M1 delivery completeness (100% or characterized
  bounded loss) · M2 convergence-correctness (byte-identical resolved `SpaceState` across all
  Nodes AND all client projections, every permutation) · M3 integrity (zero orphans /
  duplicates / unexpected pending-timeouts; content-blindness where E2E on) · M4 latency
  (informational, per-connection round-trip). **No throughput** (M8-D2).
- **Binary-level run discipline:** clean `test_runs/m8_*` per scenario; capture binary
  version+commit (CP-3); content-leak grep on Node logs (S1 M1 pattern); keep `.xgb` scripts
  for re-runs.
- **Role split:** Clair owns the implementation files for the active commit; Chat Claude holds
  edits on shared canonical files while Clair is working. Clair never pushes; Joe pushes.
- **Git pattern:** explicit `git add <file>` per file → `git status` → `git commit -m … -m …`
  (one `-m` per paragraph) → Joe pushes.
- **Workspace-test additions (M8-D6, where chosen):** standard `cargo test --workspace`
  green + clippy clean both feature sets; serialize any count-asserting shared-state tests.

---

## 8. Status & next-active

Runbook **ACTIVE v1.0**, authored below the design lock. No code yet. On Joe approval, Clair
picks up **Wave 1 / C1** (the CP-1…CP-5 pickup checks + readiness), then C2 (S2 convergence).

**Entry point:** CLAUDE.md PLAY → JOURNAL J-268 → `tasks/M8_MULTIPARTY_DESIGN.md` §2–§3 + §5
→ this runbook §2–§3 per Rule 0.

Per Rule 0 + D-065 + D-069 + D-071 + D-074 + D-078 + the two-round audit principle.
