# Multiparty Test Operation — Intro for Clair
> **Status**: ACTIVE  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-16  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## What this file is

This is the **entry point for the Multiparty test operation**. Before you start work on any individual Multiparty test file, read this document. It explains what the operation is, how the five files fit together, and the conventions that apply to all of them.

This file does NOT replace CLAUDE.md. CLAUDE.md is the project-wide briefing — behaviour rules, architecture rules, file placement, build commands, the Multiparty operation summary in context. Read CLAUDE.md first if you haven't yet. This file is the operation-specific briefing that picks up where CLAUDE.md leaves off.

---

## Reading order for a fresh session

1. **`CLAUDE.md`** — project briefing. Behaviour rules 1–7, current state, architecture rules, file placement, error code convention, build commands. **Mandatory first read.**
2. **This file (`MULTIPARTY_S0_intro.md`)** — operation briefing. The five-file suite, the conventions that apply across all tests, the dependency chain.
3. **The current Multiparty file** — the specific test you're working on. Identify it by scanning the five `MULTIPARTY_S{1,2,3,4,5}_*.md` headers and finding the first one with status `PENDING` or `ACTIVE`.
4. **Any findings files referenced** — if previous tests in the suite have already produced findings, those findings may affect the current test (e.g. S3 may have raised a spec gap; S4's M0.3 cross-references S3's findings).

The current Multiparty file's own M0 ("Preparation") step has its own spec cross-check list. Read it before you execute anything.

---

## What is the Multiparty operation

The Multiparty operation is a **sequenced suite of five test files** that exercise XGen's behaviour beyond the single-node, single-client paths covered by the Phase 1 smoke test and Phase 2 integration test. The previous tests proved the protocol works. The Multiparty suite proves the protocol scales correctness across multiple dimensions:

- **Local fan-out** — multiple clients on one Node receiving each other's events.
- **Concurrent writes** — clients on different Nodes sending at the same wall-clock moment.
- **Federation topology** — events propagating across 3+ Nodes including transitive (non-direct) paths.
- **Realistic chat-room** — all of the above composed: multiple Nodes, multiple Clients per Node, sustained chat.
- **Identity portability** — a Client re-homing from one Node to another, keeping the same Identity.

Each scenario gets its own file. Each file follows the same shape: P1 (smoke / warm-up) and P2 (stress / sustained run). They run in a fixed order. Each file must reach `COMPLETED` before the next begins.

---

## The five files, in execution order

| # | File | Theme |
|---|---|---|
| 1 | `MULTIPARTY_S1_multiclient_one_node.md` | Multiple clients on one Node — local fan-out |
| 2 | `MULTIPARTY_S2_concurrent_send.md` | DAG under concurrent writes (2 Nodes) |
| 3 | `MULTIPARTY_S3_federation_topology.md` | 3 Nodes, chain + mesh, transitive propagation |
| 4 | `MULTIPARTY_S4_n_clients_n_nodes.md` | 4 Nodes, 6 Clients, real chat-room |
| 5 | `MULTIPARTY_S5_client_rebind.md` | Identity portability — Client re-homes to a different Node |

**The order is locked.** Each file's Prerequisites section lists what must be COMPLETED before it can begin. Do not run them out of order. If S2 fails, do not proceed to S3 — fix S2 first.

---

## File set per scenario — what gets created

For each scenario Sn, the artifacts are:

| File | Created by | Lifecycle |
|---|---|---|
| `docs/tests/MULTIPARTY_Sn_*.md` | Already written (Chat Claude) | PENDING → ACTIVE → COMPLETED |
| `docs/tests/MULTIPARTY_Sn_findings.md` | You, at the scenario's M0.1 step | ACTIVE during execution → COMPLETED after |
| `docs/tests/scripts/multiparty_sn_*.xgb` | You, at the scenario's M0.4 (or equivalent) | Created when needed, kept for re-runs |

The instruction file specifies the **exact contents** of every `.xgb` script in its Appendix B and C. Create the script files verbatim from those appendices. Do not improvise the script contents.

If the instruction file uses `@last_space` or `@last_room` placeholders in `.xgb` scripts and the batch dispatcher doesn't support those backreferences (verify against `BATCH_FLAG_ph2.md`), you must split the script into two passes: first pass captures IDs, second pass uses literal IDs. Each scenario's Appendix B / C notes this caveat.

---

## Conventions that apply to every Multiparty file

### Honesty (CLAUDE.md Rules 1–7)

Every Multiparty file repeats this rule. Re-read CLAUDE.md Rules 1–7 if needed.

- **Never fabricate results.** If a command fails, report the failure with the actual output.
- **Show actual output, not a description.** Verification steps require quoting real terminal output.
- **Stop and report when a tool fails.** Do not work around silently.
- **Write the findings file as you go, not after the fact.**
- **Never invent numbers.** Event counts, latency, pairing matches — all from actual command output.
- **When in doubt, do less and ask.** Ambiguity is escalated to Joe, not resolved silently.
- **Definition of Done is a checklist, not a formality.** Each item independently verified.

### Status flow

Each instruction file follows this lifecycle:

```
PENDING  →  ACTIVE  →  COMPLETED
    ↑          ↓
    └─── BLOCKED ───┐
                    ↓
              (resume when blocker cleared)
```

- **PENDING** — the file is written but you have not started executing it yet. This is the state of all five files right now.
- **ACTIVE** — you have begun executing M0. Flip the header to `ACTIVE` at the moment you create the findings file.
- **COMPLETED** — every checklist item in the Definition of Done sections is ticked and the verdict is PASS. Flip the header to `COMPLETED` only after the final commit.
- **BLOCKED** — used for capability gates (especially S5) where a prerequisite is missing from the current build. The test is paused; a separate task file in `tasks/` describes what needs to be implemented before resuming.

**Header status is updated by editing the instruction file's header.** The Status line is the canonical source of truth for which test is currently in flight.

### Findings file is the runtime write surface

The instruction file is **read-only during execution** except for the final Definition of Done checklist (you tick items as you complete them). All observations, raw output, anomalies, and re-run data go to the findings file. This keeps the instruction file stable across re-runs.

If a test runs multiple times (because of a bug fix and re-run), each run gets a new row in the findings file's "Run history" table. Findings accumulate; the instruction file does not get re-edited.

### Scripts (`.xgb`) location and naming

All `.xgb` scripts for the Multiparty suite live at `docs/tests/scripts/`. Naming convention:

```
multiparty_sN_<phase>_<client>_<role>.xgb
```

Where:

- `N` is the scenario number (1..5).
- `<phase>` is `smoke` (for P1) or `stress` (for P2).
- `<client>` identifies the client instance (e.g. `m1a`, `clientA1`, `m5alice`).
- `<role>` describes the purpose (`setup`, `send`, `bootstrap`, `pre_rehome`, `post_rehome`, etc.).

Example: `docs/tests/scripts/multiparty_s4_stress_m4b1_send.xgb`.

The scenario file's appendices specify the exact path for each script.

### Instance labels

Every Multiparty test uses instance labels prefixed with `m` (for Multiparty) and the scenario number:

| Scenario | Node labels | Client labels |
|---|---|---|
| S1 | `m1node` | `m1a`, `m1b`, `m1c` |
| S2 | `m2nA`, `m2nB` | `m2a`, `m2b` |
| S3 | `m3nA`, `m3nB`, `m3nC` | `m3a`, `m3b`, `m3c` |
| S4 | `m4nA`, `m4nB`, `m4nC`, `m4nD` | `m4a1`, `m4a2`, `m4b1`, `m4b2`, `m4c1`, `m4d1` |
| S5 | `m5nA`, `m5nB`, `m5nC` | `m5alice` |

These labels become parts of named pipe paths (`\\.\pipe\xgen-client-<label>`), data directory names, and log filenames. Reuse of labels between scenarios is permitted because workspaces are cleaned between scenarios — but never reuse a label **within** a scenario.

### Test data directories and workspace hygiene

Every scenario's M0.3 (or equivalent) step requires a **clean workspace** for each instance before the test starts. If prior data exists, archive it to:

```
test_runs/multiparty_s<N>_<timestamp>_pre/
```

Then delete the live data directory. Record the archive path in the findings file. This makes re-runs reproducible — earlier runs are preserved for comparison if needed.

### Pairing tables — the standard verification mechanism

Most scenarios use **pairing tables** as their primary verification artifact. A pairing table is a matrix where:

- Rows are individual events (identified by their short `event_id` prefix).
- Columns are the logs / stores where each event should appear.
- Cells are `✔` if the event appears with the expected direction (Outbound for the author, Inbound for observers), `—` if the event is not expected there, or `✘` if the event is expected but missing (a bug).

A test PASSES at the pairing level only when every `✔` is verified and zero `✘` cells are observed.

The exact pairing matrix shape varies per scenario. S1 uses a single 11-event × 4-column table (Node + 3 Clients). S4 uses two matrices: a Node-presence matrix (events × Nodes) and a Client-presence matrix (events × Clients). The scenario file specifies the matrix shape.

### Content-leak checks

After each P1 phase, run a `findstr` to verify that message text content **does not appear** in log lines outside of normal `message.text` event handling. This catches accidental plaintext leakage in cryptographic debug paths, error messages, or unintended dump locations.

The exact `findstr` command is in each scenario's "Content-leak check" section. Always run it. Always paste the result verbatim into findings (zero matches expected).

### Latency metrics

P2 phases record latency informationally (median / p95 / max). These metrics are not yet pass/fail criteria for the Multiparty suite — they're baseline measurements for future regression detection. Record them honestly; do not fabricate.

### Concurrency timing — wall-clock measurable

S2 and S4 require concurrent dispatch within a measured window. The window is specified per scenario (50 ms for S2, 2 s for S4). **Record actual dispatch timestamps to sub-millisecond resolution.** If the window is exceeded, the test's concurrency claim is not met — record the actual window and decide whether to re-run.

### Stop on first failure

Within each scenario, P1 failure halts the test — do not run P2. Within P2, partial failures count as full failures — there are no soft-pass modes. Across the suite, scenario failure halts the suite — do not proceed to the next scenario. Fix the bug, re-run the failed scenario, then continue.

This is intentional. Tests further down the suite depend on earlier tests' guarantees. Running S4 with S3 broken would produce uninterpretable results.

---

## Spec sections you will need

The five scenario files reference different parts of `docs/xgen_ch3_specification.md`. Across the whole suite, the relevant sections are:

| Section | Title | Used by |
|---|---|---|
| §3.2 | Event Specification (DAG, validation pipeline) | S2, S3, S4 |
| §3.2.6 step 9 | Pending buffer for unknown predecessors | S3 |
| §3.4 | Federation Handshake | S2, S3, S4 |
| §3.5.5 | Announcement Propagation | S3 |
| §3.6 | Identity Registration (incl. `re_registration` field, line 1668) | S5 |
| §3.7 | Space and Room Protocol | S1, S2, S3, S4 |
| §3.7.11 | Phase 1 smoke test | All (for comparison) |
| §3.9 | State Resolution Algorithm | S2, S3, S4 |
| §3.9.2 | Convergence guarantee | S3, S4 |
| §3.9.6 | Pending event timeout (30 s) | S3 |
| §3.13 | Identity Replication Parameters | S5 |
| §3.13.1 | Replication model and authority | S5 |
| §3.13.4 | `identity.replicate` wire protocol | S5 |
| §3.13.8 | Orphaned Identity recovery | S5 |

Each scenario's M0 step requires quoting specific spec sections into the findings file. This is not busywork — it's the contract between the spec (what is supposed to happen) and the observation (what actually happens). The quotes are what you compare runtime behaviour against.

---

## Spec gaps and capability gates

Two scenarios surface concerns that require care:

### S3 — spec gap (informational, not blocking)

`§3.2` lacks an explicit "forward on accept to all federated peers" canonical sentence. Transitive event propagation is **implied** by `§3.9.2`'s convergence guarantee — every Node holding the same Event set computes the same state — but the "forward on accept" rule is not written as a single normative MUST.

S3's M0.3 step requires you to record this gap in findings. If S3 PASSES (the implementation does the right thing), the test description proposes a one-sentence addition to `§3.2` for a future spec pass. **Do not silently amend the spec.** Propose only; raise it to Joe in the JOURNAL entry.

If S3 FAILS due to transitive propagation not occurring, the spec gap and the implementation bug get filed together.

### S5 — capability gate (potentially blocking)

S5 depends on three protocol surfaces that may or may not be wired through the CLI:

1. **`re_registration` flag on `identity.register`** (`§3.6`, line 1668).
2. **`identity.replicate` push from home Node to replicas** (`§3.13.4`).
3. **`identity.home_changed` event observability** (`§3.13.8`).

S5's M0.3 step verifies all three. If (1) or (2) is missing, **mark S5 as BLOCKED** and file a separate task at `tasks/MULTIPARTY_S5_BLOCKER_re_registration.md` (or similar) describing the missing CLI surface. Do not invent workarounds. Do not fabricate the test results.

This is honest about a known-uncertain dependency. Phase 2 implementation is reportedly complete at the protocol level; the CLI surface for orphan recovery may need a small extension. Verify before committing.

---

## When you are done with the suite

After S5's `COMPLETED` status is set, run the **final wrap-up checklist** at the bottom of `MULTIPARTY_S5_client_rebind.md` ("Definition of Done — Entire MULTIPARTY operation"). It covers:

- All five instruction files set to `COMPLETED`.
- All five findings files set to `COMPLETED`.
- A consolidated JOURNAL.md entry summarising the whole operation.
- Any DECISIONS.md entries arising from findings (e.g. the §3.2 "forward on accept" proposal from S3).
- CLAUDE.md updated to reference the Multiparty suite as a permanent regression artifact.
- Any FIXES files created during the suite linked from the operation summary.

---

## How to begin

When you start a fresh session to work on the Multiparty suite:

1. Read `CLAUDE.md`.
2. Read this file (`MULTIPARTY_S0_intro.md`).
3. Open `docs/tests/` and find the first `MULTIPARTY_S{1,2,3,4,5}_*.md` whose status is `PENDING`.
4. Read that file in full.
5. Read any findings files from previous scenarios in the suite (their numbers will be lower than the current scenario's number).
6. Write a short summary in chat: which file you're on, what M0 requires, any spec cross-checks the file demands, any flags it raises.
7. Wait for Joe's confirmation before flipping the file to `ACTIVE` and beginning execution.

If you're picking up mid-scenario (the file is already `ACTIVE`), read its findings file to see exactly how far the previous session got. Resume from the first unticked Definition of Done item. Do not re-execute already-ticked steps.

---

## Anti-patterns — things that have gone wrong before and must not recur

Drawing from JOURNAL entries and FIXES files across the project, here are failure modes to avoid:

- **Fabricated test results.** Rule 1. If a step does not produce expected output, paste the actual output and stop. Never describe what the output "should have been".
- **Reading the wrong file from a cancelled path.** The project path is `E:\Projects\XGenProtocol`, not `G:\My Drive\Projects\XGenProtocol`. The G:\ path is deprecated. If you see it referenced anywhere, silently use E:\ instead.
- **"Commit pushed" as a Definition-of-Done item.** This is unflippable inside the commit that performs the push (chicken-and-egg). The `Status: COMPLETED` header is the real signal. Per project convention, do not add such checklist items to new task or findings files.
- **Premature failure declarations.** S3 requires waiting at least 60 seconds after the final send (≥ 2× the §3.9.6 pending timeout) before concluding "the event did not arrive". Premature declarations are test bugs, not protocol bugs.
- **Topology assumptions.** S3 requires verifying that A↔C is NOT federated by inspecting each Node's federation registry — not assuming it because the harness "should not have" federated them. Stale registry entries from prior runs can break the test.
- **Silent script content drift.** Each scenario's `.xgb` script contents are specified verbatim in Appendix B / C. If you edit a script for any reason, record the edit in findings and explain why. Do not silently change script behaviour.
- **Working around a missing capability.** If S5's M0.3 capability gate fails because the CLI doesn't expose `--re-registration`, file a blocker task. Do not write a workaround using manual JSON construction or low-level wire commands — the test exists to verify the CLI path works end-to-end.

---

## Questions?

If anything in this file or in a Multiparty scenario file is ambiguous, **stop and ask Joe**. CLAUDE.md Rule 6 applies. The Multiparty suite is the most thorough integration test the project has — better to spend a session clarifying than a week chasing a fabricated PASS.

---

*End of MULTIPARTY_S0_intro.md*
