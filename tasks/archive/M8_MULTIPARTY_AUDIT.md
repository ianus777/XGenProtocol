# M8 — Multiparty Strong-Test — Phase-0 Audit
> **Status**: COMPLETED  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-05  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. Purpose & scope

M8 is the **next-active** milestone after the Round-2 gate closed COMPLETE (J-267), first
in the locked post-gate chain **M8 → M9 (Multiparty Redesign) → Multiparty tests → M10 →
UI**. This is its D-071 Phase-0 audit: ground the multiparty suite and the binary it runs
against **before** any design lock. No design decisions, no code.

**Scope reframed by Joe (2026-06-05):** M8 is **a proper, strong multiparty test covering
all functional aspects** of the protocol as it stands today — not a narrow S1 re-run. The
"A/B metrics" framing in the ROADMAP placeholder (L776) **demotes to a sub-goal**: where a
prior "A" pass exists, compare against it; everywhere else M8 *establishes* the baseline.

This audit's central job (D-078, production-grounded enumeration) is therefore: map the
**shipped** functional surface to multiparty scenarios, and honestly mark the gap between
the May-2026 S1–S5 skeleton and "all aspects."

---

## 2. Grounding — the multiparty suite as it stands

The suite is a sequenced five-file operation (`docs/tests/MULTIPARTY_S0_intro.md`, ACTIVE),
order locked, each file P1 (smoke) + P2 (stress).

| # | File | Theme | Status |
|---|---|---|---|
| S0 | `MULTIPARTY_S0_intro.md` | operation entry point | ACTIVE |
| S1 | `MULTIPARTY_S1_multiclient_one_node.md` | N clients / 1 Node — local fan-out | **COMPLETED** |
| S2 | `MULTIPARTY_S2_concurrent_send.md` | DAG under concurrent writes (2 Nodes) | PENDING |
| S3 | `MULTIPARTY_S3_federation_topology.md` | 3 Nodes, chain + mesh, transitive propagation | PENDING |
| S4 | `MULTIPARTY_S4_n_clients_n_nodes.md` | 4 Nodes, 6 Clients, real chat-room | PENDING |
| S5 | `MULTIPARTY_S5_client_rebind.md` | identity portability — Client re-homes | PENDING |

**Only S1 has ever executed.** S2–S5 are specced but never run → **no "A" baseline exists
for them.** Running them is a *first* pass, not an *improved* pass.

**The S1 "A" (`MULTIPARTY_S1_findings.md`, COMPLETED):** ran against commit `7e06896`
(2026-05-16). P1 smoke cell-perfect (9-row pairing table, zero misses, zero content leak);
P2 stress **294/300 = 98%**, with a **6/300 silent loss** between client WS write and Node
`event_trace` receive (no error/timeout/duplicate/orphan). Four bugs found+fixed in-session
(F-001 missing fan-out, F-002 first-message drop, F-003/F-004 cross-Space tip leak ×2).
Three follow-ups left open:
1. unify the two `get_dag_tips` copies;
2. characterize the 2% loss (WS write-and-close race);
3. add a **long-lived-client `--batch` mode** for true throughput metrics (S1 measured
   per-send-connection ~600 ms, explicitly *not* a throughput test).

---

## 3. Grounding — the binary drift (A vs B)

- **A** = commit `7e06896` (2026-05-16).
- **B** = HEAD `676b9c1` (2026-06-05), working tree clean.

The B binary is **enormously drifted** from A. Everything below shipped *after* the S1 pass,
so the S1 "A" predates almost the entire current protocol surface:

M6 admin write-path · the full **M7 `--aicontrol` family** · **Durable EventStore** + the
plugin framework (`xgen-store-sqlite`) · **Arc C** state-resolution convergence
(`derive_resolved` wired onto the apply path) · **Arc D** privilege-model (PG-13 tier-gate
on join, PG-12-min per-Room overrides) · **Arc E** primitives (PG-03 TrustAssertion, PG-08
Thread) · **Arc F** Space migration (`home_node` flip across Nodes) · **Arc G** jurisdiction
federation policy · **Arc H** E2E encryption (content-blindness proof, KeyPackage lifecycle,
epoch-advance) · **Arc I/D-088** GDPR erasure architecture · the **R2-F01** A-pure client
convergence rewrite.

**Concrete drift signal (grounded):** `get_dag_tips` — the function at the centre of S1's
F-003/F-004 bugs — is **gone from `xgen-client/src/main.rs`** (0 refs); only one copy remains
(`xgen-client/src/batch.rs:87`). The duplicate that S1 follow-up #1 flagged is already
resolved, and the read path S1 exercised no longer exists in that form. **Implication: the
S1 "A" is a weak A/B baseline** — protocol, client convergence, storage, and fan-out have all
changed underneath it. Any S1 A/B must either re-run S1 on an A-equivalent build or be framed
honestly as "A historical, B measured, deltas explained."

---

## 4. Functional-surface enumeration (D-078) — "all aspects"

The five S1–S5 dimensions cover **transport/propagation correctness**. They do **not** cover
the capabilities that shipped after the skeleton was written. A strong test of "all aspects"
must cover both. Below: shipped surface → multiparty relevance → existing-scenario coverage.

### 4.1 Covered (or partially scaffolded) by the S1–S5 skeleton
| Aspect | Multiparty relevance | Skeleton coverage |
|---|---|---|
| Local fan-out | N clients/1 Node see each other's events | **S1 (RAN, PASS)** |
| Concurrent writes | DAG integrity under simultaneous sends | S2 spec (PENDING; predates `derive_resolved`) |
| Federation topology | transitive propagation across 3+ Nodes | S3 spec (PENDING) |
| Realistic N×N chat | all of the above composed | S4 spec (PENDING) |
| Identity portability | client re-homes, keeps identity | S5 spec (PENDING) |

### 4.2 Shipped after the skeleton — **no scenario covers these today**
| Aspect | Multiparty test it implies | Source |
|---|---|---|
| **Convergence under conflict** | concurrent ban/join, role conflict, key-rotation → all Nodes elect the same winner (byte-identical) | Arc C |
| **Client/node convergence alignment** | N client projections re-derive to the node's resolved view under concurrency/skew | R2-F01 (A-pure) |
| **Privilege enforcement** | tier-gated join refused multiparty; per-Room override ("Mods can't post in #announcements") observed by all members | Arc D |
| **Threads / trust assertions** | thread resolve-vs-archive converges; assertion-gated participation | Arc E |
| **Space migration** | `home_node` flips on *both* Nodes mid-session; clients keep converging | Arc F |
| **Jurisdiction policy** | federation rejects cross-jurisdiction Space per `allowed_jurisdictions` | Arc G |
| **E2E content-blindness** | encrypted fan-out to N members; Node never sees plaintext; KeyPackage pool / epoch-advance under multiple joiners | Arc H |
| **EventStore durability/replay** | Node restart mid-session → replay → clients resync; no orphans | Durable EventStore + plugin |
| **AI-driven participants** | `--aicontrol`-driven client as a first-class room member alongside humans | M7 |

**The gap is the headline finding:** "all aspects" is roughly **2× the skeleton's surface.**
S2–S5 need either substantial extension or new sibling scenarios (S6+) to reach it.

### 4.3 Out of multiparty scope (named, not tested here)
Real RFC 9420/openmls (D3) · GDPR erasure *implementation* (PG-02, D3-gated) · active data
residency (operator/Tier-2+ infra) · the multi-device seam (R2-F09, pulled, D3). These are
interface-locked or downstream; M8 tests the *shipped* interfaces, not the D3 upgrades.

---

## 5. Tooling & harness grounding

- **`--batch` is still per-send short-lived.** `xgen-client --batch <file>` runs the
  in-process batch executor and exits (`main.rs` dispatch comment L14); each `send` opens its
  own WS connection (the S1 ~600 ms/round-trip cost). **`--service` (headless resident) is a
  stub** ("until 2b/M3", `main.rs` L15). **So S1 follow-up #3 (long-lived client mode for
  throughput) is still unbuilt** — this directly constrains what "metrics" can mean (see Q2).
- **`.xgb` script viability is unverified against B.** The S1 scripts were written for the
  `7e06896` CLI surface; M6/M7 verb changes may have moved them. Re-validation is a design /
  early-implementation task, not assumed.
- **`get_dag_tips` consolidated** to one copy (`batch.rs:87`); the main.rs duplicate is gone
  (S1 follow-up #1 effectively resolved — confirm at design).
- **Federation harness exists.** The Phase-9 federation survey shipped a multi-node harness
  (`phase9_harness`, two-node convergence smoke used at Arc C C3); S3/S4 (3–4 Nodes) need it
  scaled up — grounding the harness ceiling is a design item.
- **Test-suite scale:** workspace at **1156/0/2** (J-267). These are unit/integration tests;
  the multiparty suite is a separate **binary-level** operation (real Node + Client processes,
  `.xgb` scripts, `test_runs/` data dirs) — M8 lives at that level, not in `cargo test`.

---

## 6. Findings

- **M8-A1 (headline).** "All aspects" ≈ 2× the S1–S5 skeleton. Nine shipped capabilities
  (§4.2) have **zero multiparty coverage** because they postdate the skeleton. M8 scope is a
  *coverage-design* problem first, an *execution* problem second.
- **M8-A2.** Only S1 has an "A". A/B is literally meaningful only for S1, and even there the
  baseline is stale (§3). For everything else M8 establishes the first baseline.
- **M8-A3.** The S1 "A" baseline build predates the entire post-`7e06896` surface → a fair
  A/B needs an explicit framing decision (re-run on A-equivalent vs honest historical/measured
  delta). Not a convergence question; a measurement-honesty question.
- **M8-A4.** Throughput metrics are **blocked on an unbuilt long-lived client mode**
  (`--service` stub). "Metrics" today can mean delivery-correctness + latency-per-connection,
  not sustained throughput, unless that mode is built as an M8 prerequisite.
- **M8-A5.** `.xgb` scripts and the federation harness need re-validation/scaling against B
  before any scenario can run — a real (small) build cost folded into the runbook.
- **M8-A6 (ordering insight, not a finding to fix).** The locked chain
  **M8 → M9 (redesign) → Multiparty tests** reads naturally as: M8's strong test is the
  **diagnostic** that scopes M9; the post-M9 "Multiparty tests" gate re-runs the same strong
  suite against the *redesigned* binary — yielding a genuine A/B **across the redesign**,
  which is more meaningful than vs. the stale S1. Framing for Q4; not locked here.
- **M8-A7 (scope separation, locked upstream).** M8's tests stay **independent of the M10
  auth-module battery** (J-267 Joe clarification). M8 exercises convergence/federation under
  N clients; auth-tier exhaustiveness is M10's own surface. Honor this when enumerating
  privilege/trust-assertion scenarios (§4.2) — test the *multiparty* behaviour, not the auth
  matrix.

---

## 7. Open design questions (framed — NOT locked)

- **Q1 — Coverage depth.** Does M8 (a) extend S2–S5 to absorb the §4.2 capabilities, or
  (b) keep S2–S5 as-is and add new sibling scenarios (S6+) for the new surface, or (c) a
  hybrid (extend where natural, add where not)? This is the primary design fork.
- **Q2 — Metric definitions.** What does "metrics" mean for a strong pass — delivery rate,
  convergence-correctness (all Nodes byte-identical), latency, sustained throughput? If
  throughput is required, the long-lived client mode (M8-A4) becomes an explicit prerequisite
  arc inside M8. Define the metric set before the runbook.
- **Q3 — A/B framing.** For S1: re-run on an A-equivalent build, or record "A historical / B
  measured / deltas explained"? For S2–S5+: confirm M8 establishes the baseline (no A).
- **Q4 — Ordering vs M9.** Treat M8 as the diagnostic feeding M9 (M8-A6), so the *real* A/B
  is the post-M9 "Multiparty tests" re-run? This shapes how much M8 invests in baseline
  capture vs breadth.
- **Q5 — Harness & tooling readiness.** Confirm the federation harness scales to 3–4 Nodes;
  decide whether `.xgb` re-validation + any long-lived mode are M8-internal prerequisite
  commits or separate arcs.
- **Q6 — Suite home.** Do new/extended scenarios live as more `MULTIPARTY_S*` files (binary-
  level, `.xgb` + `test_runs/`), or partly as workspace integration tests? The skeleton is
  binary-level; new convergence/E2E checks may be cheaper as integration tests — a placement
  decision.

---

## 8. Status & next-active

Phase-0 audit **ACTIVE** (this document). No design locks, no code, no DECISIONS change.
Suite unchanged 1156/0/2 (audit-only). M8 stays 🟡 pending in the ROADMAP.

**Next-active: Phase-1 design** — resolve Q1 (coverage depth) and Q2 (metric set) first,
since they gate everything; then Q3–Q6; then Joe-lock; then runbook; then Clair implements.

**Entry point:** CLAUDE.md PLAY → JOURNAL J-267 → `tasks/ROUND_2_AUDIT.md` §6–§7 → this
audit §6–§7 per Rule 0.

Per Rule 0 + D-065 + D-069 + D-071 + D-074 + D-078 + the two-round audit principle.
