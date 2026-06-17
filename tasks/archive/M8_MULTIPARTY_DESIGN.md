# M8 — Multiparty Strong-Test — Phase-1 Design
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jun 2026  
> **Last updated**: 2026-06-05  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. Purpose

Phase-1 design for M8, building on the Phase-0 audit (`tasks/M8_MULTIPARTY_AUDIT.md` v1.0).
M8 = **a proper, strong multiparty test covering all functional aspects** of the current
binary; A/B metrics are a sub-goal. **All design decisions (M8-D1 through M8-D6) were
Joe-locked 2026-06-05**, together with the §3 scenario set (S1–S8) and the §6 wave structure.
No code, no DECISIONS.md change (M8-D# arc-local, D-069).

---

## 2. Locked scope decisions

- **M8-D1 (Q1 — coverage depth = HYBRID, LOCKED).** Extend the federation-native scenarios,
  add new scenarios for orthogonal capabilities, fold convergence-alignment and
  durability/replay as **cross-cutting verification gates** rather than standalone files.
  (§3 is the concrete map.)
- **M8-D2 (Q2 — metric set = FOUR, throughput a NON-GOAL, LOCKED).** The pass metrics are
  delivery completeness, convergence correctness, integrity, and latency (informational).
  **Sustained throughput is an explicit M8 non-goal** — it is blocked on the unbuilt
  long-lived client mode (audit M8-A4); building that mode inside M8 is scope creep. Revisit
  only if M9's redesign touches the connection model. (§4 defines the four.)

---

## 3. Per-scenario coverage matrix (M8-D1 realized, LOCKED)

Legend: **E** = extend existing spec · **R** = run existing spec as-is · **N** = new
scenario. P1 = smoke/warm-up, P2 = stress/sustained, as in S0.

| Scenario | Action | Dimension | New capability folded in |
|---|---|---|---|
| **S1** multiclient/1-Node | R (re-run on B) | local fan-out | client/node alignment gate; B-vs-historical-A delta |
| **S2** concurrent-send (2 Nodes) | **E** | DAG under concurrent writes | **convergence-under-conflict** (concurrent ban/join, role conflict, key-rotation → byte-identical winner across Nodes) — Arc C headline |
| **S3** federation topology (3 Nodes) | **E** | transitive propagation | **jurisdiction policy** (cross-jurisdiction Space rejected per `allowed_jurisdictions`); **migration** (`home_node` flip mid-topology, both Nodes) |
| **S4** N×N real chat (4 Nodes/6 Clients) | **E** | composite chat-room | **durability/replay gate** (restart a Node mid-run → replay → clients resync, no orphans) |
| **S5** client rebind | R | identity portability | (distinct from migration: client re-homes, keeps identity) |
| **S6** E2E blindness | **N** | encrypted multiparty | **content-blindness** at binary level (encrypted fan-out to N members; Node never sees plaintext; KeyPackage pool + epoch-advance on multi-join) — Arc H |
| **S7** privilege enforcement | **N** | authority under N clients | **tier-gated join refusal**; **per-Room override** ("Mods can't post in #announcements") observed by all members — Arc D |
| **S8** AI-driven participant | **N** | mixed human/AI room | `--aicontrol`-driven client as a first-class member (composes with S4; see CP-5) — M7 |

**Cross-cutting gates (every scenario, not standalone files):**
- **G-ALIGN** — every client's projection equals its home Node's resolved view (R2-F01 A-pure
  must hold under live multiparty load, not just unit tests).
- **G-INTEGRITY** — zero DAG orphans, zero duplicate `event_id`, zero unexpected
  pending-timeouts; where E2E is on, zero plaintext in Node logs/store.
- **G-DURABILITY** — folded where a restart is natural (S1, S4); not a separate scenario.

**M8-A7 honored (scope separation):** S7 tests the *multiparty behaviour* of the tier-gate
and Room overrides — NOT the auth-tier matrix (that is M10's own battery). A synthetic
Tier-1/Local-Node setup is sufficient; no Auth Module reference set is required or built here.

---

## 4. The four metrics (M8-D2 realized, LOCKED)

- **M1 — Delivery completeness.** Every event accepted into a Space DAG reaches every member
  (real-time fan-out or sync pull). **Target:** 100% of accepted events observed by every
  member. The S1 "A" showed 294/300 (6 silent losses); M8 determines whether that persists on
  B and, if so, characterizes it (S1 follow-up #2). Pass = 100% or a characterized, bounded,
  non-structural loss with a recorded root cause.
- **M2 — Convergence correctness (headline; new since S1).** For every conflict scenario, all
  Nodes' resolved `SpaceState` is **byte-identical**, AND every client projection equals its
  home Node's resolved view (G-ALIGN). Pass = byte-identical across all Nodes + clients under
  every arrival permutation tested.
- **M3 — Integrity.** Zero DAG orphans, zero duplicate `event_id`, zero unexpected
  pending-timeouts, no `ERROR`/unexpected `WARN` lines; where E2E on, content-blindness holds
  (no plaintext in Node-visible surfaces). Pass = all zero.
- **M4 — Latency (informational, not pass/fail).** Per-connection round-trip, recorded as in
  S1. **Throughput is NOT measured** (M8-D2 non-goal).

---

## 5. Design decisions — Q3–Q6 (LOCKED 2026-06-05)

- **M8-D3 (Q3 — A/B framing, LOCKED).** S1: record "A historical (`7e06896`) / B measured
  (`676b9c1`) / deltas explained" — do **not** rebuild the A-equivalent binary (audit M8-A3).
  S2–S8: M8 **establishes** the baseline (no A).
- **M8-D4 (Q4 — M8 as M9 diagnostic, LOCKED).** M8 is the **diagnostic that scopes M9**; the
  post-M9 "Multiparty tests" gate is the *real* A/B, across the redesign (audit M8-A6).
  Consequence: M8 optimizes for **breadth + honest baseline capture**, not for polishing any
  single metric. A scenario that surfaces a real weakness is a *success* (it feeds M9), not a
  failure to fix in-arc — unless it is a clean, contained bug (S1-style in-session fix).
- **M8-D5 (Q5 — readiness as first commit, LOCKED).** `.xgb` re-validation against the B CLI
  surface + federation-harness scale-up to 3–4 Nodes are an **M8-internal prerequisite
  (Wave 1, C1)**, not a separate arc.
- **M8-D6 (Q6 — suite home = hybrid, LOCKED).** End-to-end scenarios stay binary-level
  (`MULTIPARTY_S*` + `.xgb` + `test_runs/`); pure convergence/E2E *correctness* checks may live
  as workspace integration tests where spinning real processes adds no signal (per-scenario
  call, CP-4). Lean binary-level for anything a real operator/UI would do.

---

## 6. Wave / commit structure (runbook seed, LOCKED)

- **Phase 0 — audit** ✅ (`M8_MULTIPARTY_AUDIT.md`)
- **Phase 1 — design** ✅ (this doc, Joe-locked 2026-06-05)
- **Wave 1 — readiness + convergence**
  - C1 `.xgb`/CLI re-validation against B + federation harness scale-up (M8-D5)
  - C2 **S2** concurrent state-events → convergence-under-conflict + G-ALIGN
- **Wave 2 — federation composite**
  - C3 **S3** topology + jurisdiction + migration
  - C4 **S4** N×N chat + G-DURABILITY restart-replay; **S5** rebind run
- **Wave 3 — orthogonal capabilities**
  - C5 **S6** E2E content-blindness
  - C6 **S7** privilege enforcement
  - C7 **S8** AI-driven participant
- **Close** — consolidate findings into an M8 findings record → **hand the diagnostic to M9
  scope**. Each scenario produces its own `MULTIPARTY_Sn_findings.md` (S0 convention).

Wave boundaries are natural Joe-lock checkpoints. Waves are sequential (S0 order discipline);
within a wave, scenarios are independent enough to reorder if a blocker appears.

---

## 7. Confirm-at-pickup for implementation (D-078)

- **CP-1** `.xgb` verb surface — reconcile every script command against the current B CLI
  (M6/M7 drift); the S1 scripts assume the `7e06896` surface.
- **CP-2** federation harness ceiling — confirm `phase9_harness` scales to 3–4 Nodes or
  identify the work to get there.
- **CP-3** B-record stamp — capture `xgen-node/-client version` + commit for the B column of
  every findings file.
- **CP-4** per-scenario placement — for each convergence/E2E check, decide binary-level vs
  workspace integration test (M8-D6).
- **CP-5** AI-participant viability — confirm `--aicontrol` can drive a client that *holds a
  room membership* across a session; the `--service` resident is a stub (audit §5), so S8 may
  need the command-pipe driving a foreground client rather than a headless resident. If S8
  cannot hold a live membership without the resident mode, fold S8 into a scripted variant of
  S4 and record the limitation (do not build resident mode — M8-D2 non-goal spirit).

---

## 8. Status & next-active

**Design phase Joe-locked 2026-06-05.** All of M8-D1…D6 + the §3 scenario set (S1–S8) + the
§6 wave structure are locked. No code, no DECISIONS.md change (M8-D# arc-local, D-069). Suite
unchanged 1156/0/2. M8 flips 🟡 pending → 🟡 OPEN (design-locked) in the ROADMAP.

The canonical-record open ships same-commit (D-074): JOURNAL J-268 "M8 OPENED" + CLAUDE.md
PLAY flip + ROADMAP bump, alongside this design lock.

**Next-active: the runbook** (`tasks/M8_MULTIPARTY_IMPL.md`, Joe-approved) → Clair implements
Wave 1 (C1 readiness → C2 S2).

**Entry point:** CLAUDE.md PLAY → JOURNAL J-268 → `tasks/M8_MULTIPARTY_DESIGN.md` §2–§3 + §5
per Rule 0.

Per Rule 0 + D-065 + D-069 + D-071 + D-074 + D-078 + the two-round audit principle.
