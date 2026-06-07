# M9 — Strategic Multiparty Test Harness — Phase-0 Audit
> **Status**: ACTIVE  
> Version: 1.2  
> Date: Jun 2026  
> **Last updated**: 2026-06-07  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. Purpose

The D-071 Phase-0 audit that opens **M9** — the numbered operative milestone that **builds**
the strategic multiparty test harness. M9 ships the harness (infrastructure); the unnumbered
**Multiparty-tests** milestone then **runs** it on a finalized binary; a multiparty *redesign*
is a contingent follow-on, not M9 scope (J-303). This document grounds what test scaffolding
exists today, names the in-process → real-binary seam M9 must cross, and frames the forks the
design phase will lock. Doc-only; no code, no DECISIONS change, no milestone-state change
beyond the Phase-0 open recorded at close.

---

## 2. Charter (Joe, 2026-06-07)

The harness must exercise **all aspects of both binaries** (`xgen-node` + `xgen-client`)
through the **`--aicontrol`** interface, by **simulating mass use** of many real running nodes
and clients, covering **both**:

1. **Cooperative / realistic** — many genuine concurrent participants in realistic situations
   (chat at scale, federation topology, identity re-home), AI-resident-driven.
2. **Adversarial** — deliberate **attempts to break the system** (malformed / forged /
   out-of-policy / flooding / partition / skew traffic).

Two consequences locked by the charter (recorded here, formalised in design):
- **Drive surface = `--aicontrol`** (Joe-stated; the D-066 AI-driver surface). `--batch` stays
  frozen for human use (D-066) and is **not** the harness driver.
- **The adversarial leg needs a hostile-driver capability** the cooperative AI-resident does
  not provide. This is M9's central new design question (M9-F-F below).

---

## 3. Gates satisfied (D-071)

- **M8.5 finalization CLOSED** (J-278) — INV bootstrap, F-5 anti-transitivity (Option 1
  pairwise, D-089), S5 identity-rebind surfaces shipped.
- **M8.6 / M8.7 CLOSED** (J-294 / J-302) — Clock-injection seam + four federation-stress
  compounds; `mls.commit` concurrent-commit resolution. The last sequenced arcs before M9.
- **Round-2 (UI-gate) audit: GO** (J-258, CONDITIONAL→GO) — codebase coherent.
- **INV-EXP CLOSED** (J-298) — invite-expiry replay gate (admission-only, F-5/D-089 trust).
- Suite **1212/0/2**; clippy clean (default + `--all-features`).

---

## 4. Grounding — what scaffolding exists today

### 4.1 The in-process harness (M9-A1 — the seam)
`xgen-node/src/tests/phase9_harness.rs` spawns a Node as a `tokio::spawn`-ed accept loop that
**mirrors** `run_node`'s post-config body. It **deliberately skips**: keypair-file load,
startup banner, tracing init, **replay-from-disk**, **reconnect scheduler**, state-writer
task, pending-buffer timeout sweep, **named-pipe server**, Tauri, Ctrl+C handler. It keeps:
runtime construction, shared-state wiring, `Server::bind` on `127.0.0.1:0`, accept loop,
`handle_connection`. `federate()` drives A→B via the production `attempt_reconnect`; identity
replication is the test's responsibility.

**This is exactly the surface M9 must cross.** The in-process harness proves protocol logic in
one address space; it cannot exercise the real-process startup, the named-pipe `--aicontrol`
command surface, on-disk keypairs/storage, or true cross-process transport. **M9 = build the
harness that drives the real `xgen-node.exe` / `xgen-client.exe` as separate processes.**

### 4.2 The `--aicontrol` surface (M9-A2)
Shipped in the M7 family (`--aicontrol` v1 J-205, events J-212): client + node `.aicontrol`
command pipes over `xgen-common::aicontrol`, adapter-wrapping `ops::*` / `admin_ops::*`
(D-065/D-066). The node surface is `admin_ops::*` + `state`. `.events` pipes (J-210/J-211)
give live event observation on both binaries (`event_subscriptions` counted live). **This is
the harness's drive + observe surface.** Grounding the exact present command set and pipe
paths is a design-phase task.

### 4.3 The AI-resident (M9-A3 — the cooperative participant mechanism)
M8 §3 routed to M9 the **AI-resident load-test harness friction list** (captured in
`docs/tests/MULTIPARTY_S8_findings.md`). Rationale on the record: load-bearing multiparty
faults only surface with **many genuine concurrent participants**, and the AI resident
(`--ai-mode --service`, M4 — holds a live membership, A-pure G-ALIGN apply path) supplies that
concurrency without a human crowd. **C9 fixed-in-M8** (`a51b556`): the AI resident now honors
`--node`. The strategic suite is AI-resident-driven *by design* (M8 §3, first articulation —
recorded, not yet promoted). **`MULTIPARTY_S8_findings.md` is direct M9 input.**

### 4.4 The S1–S5 scenario specs (M9-A4)
`docs/tests/MULTIPARTY_S{1..5}_*.md` exist as rich scenario specs — S1 local fan-out · S2
concurrent send · S3 federation topology (transitive) · S4 n×n sustained chat · S5 identity
re-home. **They are DEPRECATED as runbooks** (written for manual Tauri `--batch`, descoped
2026-05-17) but reusable as **scenario intent** for the cooperative family. Per-scenario M8
diagnostic results live in `MULTIPARTY_S{1..8}_findings.md`. These define the realistic-
situation catalogue the Multiparty-tests milestone will draw on; M9 builds the machinery, not
the scenarios.

### 4.5 Reusable seams (M9-A5)
- **M8.6 Clock seam** — `Clock`/`RealClock`/`MockClock` (xgen-common, `mock-clock`), reconnect
  connect-timeout, federation channel-capacity seam. Promotion-watch: **likely promotes to a
  D-NNN if M9's harness reuses it.**
- **M8.7 two-`NodeRuntime` convergence pattern** + the `RoomState` `Eq` oracle — the in-process
  analogue of the cross-process convergence oracle M9 needs (M9-F-D).
- **phase9 helpers** — `federate()`, `register_identity()`, the two/three-node smokes,
  `phase9_drop_and_recover`, the C1/C2/C4/C6/C8/C10 compounds. Real-binary analogues of these
  are the harness's first proof targets.

### 4.6 Adversarial coverage already in-process (M9-A6)
Much "break-the-system" behaviour is already proven **in-process** and must be **re-proven at
the real-binary boundary** (plus net-new attacks the in-process tests can't stage):
- Invite-expiry replay (INV-EXP, J-298) · over-ceiling / expired invite gates (3044/3045) ·
  tier-gate join refusal (PG-13) · federation anti-transitivity / pairwise trust (F-5/D-089) ·
  back-pressure / channel saturation + no-hang (M8.6 C8) · partition + reconnect ladder
  (M8.6) · clock skew (M8.6 Clock) · concurrent-commit race (M8.7).
- **Net-new at the binary boundary:** signature/identity forgery on the wire, equivocation /
  fork attempts across real transport, flooding / DoS, malformed-frame injection, unauthorized
  join attempts that never pass client-side validation. **These need the hostile driver
  (M9-F-F).**

### 4.7 Build target (M9-A7)
Binaries build to `C:/cargo-targets/XGenProtocol`. The harness must locate or build the real
`.exe`s and manage their lifecycle (spawn, drive, observe, teardown) — Windows process model.

---

## 5. Forks for the design phase

| Fork | Question | Lean (to confirm in design) |
|------|----------|------------------------------|
| **M9-F-A** | Drive surface | **LOCKED by charter** — `--aicontrol`. `--batch` stays human-frozen (D-066). |
| **M9-F-B** | Participant model | **Two-number model** (§6.1): real OS *processes* (topology width, HW-bound) x AI-resident-multiplexed *logical participants* (load depth, cheap). Cooperative = AI-resident; adversarial = a separate **hostile driver**. |
| **M9-F-C** | Orchestrator location | Rust integration crate (`std::process` spawning real `.exe`s, driving the `.aicontrol` pipes) over an external PowerShell/script harness — keeps it in `cargo test`, contributor-legible, reuses builders. |
| **M9-F-D** | Cross-process convergence oracle | Black-box: `.events` transcripts + `state` query (and optional on-disk store compare) — the cross-process analogue of the in-process `RoomState` `Eq` oracle. |
| **M9-F-E** | Clock | **Both modes** — MockClock (deterministic) for the small reproducible round; real-clock for the scale/chaos rounds. **Triggers the `Clock` D-NNN promotion if reused.** |
| **M9-F-F** | **Adversarial injection mechanism** | **The central new question.** How the harness emits malformed / forged / out-of-policy / flooding traffic. Options: (1) a raw-wire injector below the client (crafts frames directly to a Node's transport), (2) a "hostile aicontrol driver" with crafted/abusive args, (3) a mutating proxy between honest binaries. Lean unframed — grounded in design against the real transport + validation boundary. |
| **M9-F-G** | Scale model | **The round dial** (§6.2): the harness takes `N processes x M residents/process x ramp profile` as a **parameter**, never a hardwired constant. M9's own smoke is effectively "Round 0." |

---

## 6. Scale & run strategy

### 6.1 "Mass" is two numbers (the cost model)
The AI-resident multiplexes, so participant scale decouples from process scale:
- **Process count (topology width)** — real `xgen-node.exe` / `xgen-client.exe` OS processes.
  **HW-bound.** Estimated per-process cost (Rust + tokio, lean store, tuned): ~15–35 MB RSS;
  tokio worker threads must be pinned to 1–2 for harness builds (default = #cores would thrash
  the scheduler at hundreds of processes); ports/pipes/handles are not the wall. **Memory is
  usually the binding resource.**
- **Logical participant count (load depth)** — identities/connections driven *per* process by
  the AI-resident. Events + state, an order of magnitude cheaper than processes.

**Estimated process ceilings (tuned, real processes):** ~200–400 comfortable on 16 GB/8-core;
~800–1,200 on 32 GB/16-core; ~2,000–3,000 on 64 GB/24-core+. So **low thousands of processes is
reachable on a 32–64 GB box**; **hundreds are safe everywhere**; **thousands of logical
participants** come cheap via multiplexing. Exact numbers are pending Joe's box selection; the
first M9 build task is a micro-benchmark (spawn 10/50/100, measure RSS + thread count, derive
the box's real ceiling).

### 6.2 The three-round run plan (Joe, 2026-06-07 + recommendations)
A **Multiparty-tests run plan** (the unnumbered milestone) — recorded here because it imposes
the M9-F-G round-dial requirement on the harness. Small rounds are cheap → iterate there; the
big round is expensive → instrument heavily, single pass.

| Round | Processes (node / client) | Logical participants | Clock | Purpose |
|-------|---------------------------|----------------------|-------|---------|
| **R1 — small** | 3–10 / 10–30 | ~50–100 | **MockClock (deterministic)** | Catch most bugs; reproducible; fix→rerun loop |
| **R2 — bigger** | ~20–50 / 100–300 | ~500–1,000 | Real-clock | Scale-dependent faults (back-pressure, reconnect storms, partition) |
| **R3 — biggest** | hundreds / box ceiling | low thousands | Real-clock, chaos | One-shot capstone — observe, record, do not fix-in-loop |

**Recommendations (Chat Claude, 2026-06-07 — recorded):**
1. **R1 deterministic, R3 chaotic.** Reuse the M8.6 `Clock` seam in R1 so any bug is replayable;
   run R3 real-clock *because* emergent timing behaviour is the point (this is why F-E keeps
   both modes).
2. **Adversarial scales differently from cooperative.** Logic-attacks (forged sig, expired
   invite, tier-gate bypass, equivocation) are rejected regardless of crowd size → **prove them
   in R1**. Only volume-attacks (flooding, DoS, back-pressure saturation) need R2/R3. The
   adversarial battery is **not** uniformly "scale up" — most of it lives in the cheap round.
3. **Instrument the one-shot.** R3 "just how it goes" is only useful if it leaves artifacts —
   full `.events` transcripts, per-process RSS/thread curves, the convergence-oracle verdict.
   The harness must **capture-by-default**; an uninstrumented one-shot is a wasted run.

### 6.3 Companion artifacts — test matrix + saved aicontrol batches
The **what-we-test + result** catalogue lives in `docs/tests/MULTIPARTY_TEST_MATRIX.md` (ACTIVE)
— actor-narrative scenarios (`MP-C-##` cooperative, `MP-A-##` adversarial), each with Expected /
Oracle / Round / Batch / Result; results fill in at run time; it supersedes the DEPRECATED
`MULTIPARTY_S1..S5_*.md` specs as the live scenario source. **The harness drives `--aicontrol`
from saved, versioned JSONL batch files** (Joe, 2026-06-07) under
`docs/tests/multiparty_scenarios/<ID>/`, one `.jsonl` per actor (one client process = one pipe =
one batch); the harness reads + feeds them line-by-line — no ad-hoc inline command generation.
Grounded format: one `Command` envelope per line (cmd / args / id / bind), bind-and-substitute
chaining per connection; verbs `register`→`identity_id`, `create-space`→`space_id`,
`invite`/`join`→`space_id`, `send`→`event_id`. **Two design seams flagged (not locked):**
(1) **cross-actor values** — bind/substitution is per-connection, so cross-actor values use a
harness placeholder (`{{...}}`) the orchestrator fills from a prior actor's reply (exact syntax
+ ordering/sync = design); (2) **wire-malformation attacks** are not valid envelopes → they run
via the F-F raw injector, while logic-attacks stay batch-expressible (the batch sends and the
Result asserts the rejection code/category). Worked example seeded on disk as the format pin:
`docs/tests/multiparty_scenarios/MP-C-02/{alice,bob}.jsonl`.

---

## 7. Scope boundary (charter discipline)

- **M9 builds** the harness + the **round dial + capture-by-default** + **one cooperative smoke
  + one adversarial smoke** ("Round 0") proving the machinery works end-to-end against the real
  binaries. M9 does **not** author or run the full cooperative or adversarial batteries.
- **The unnumbered Multiparty-tests milestone** authors + runs both batteries across R1/R2/R3 on
  a **finalized binary** (clean-table principle, M8 §5); it fixes the actual round numbers
  against the selected box.
- **Redesign** (if convergence/topology gaps surface) is contingent on what those runs reveal.
- **Honest boundary (D-065):** the cooperative leg leans on the AI-resident as a *stand-in* for
  human participants — named in the design coverage ledger (real human-crowd behaviour is not
  exercised; the AI-resident is the concurrency mechanism, M8 §6).

---

## 8. Next-active

**M9 design phase** — ground the present `--aicontrol` command set + pipe paths (4.2), the
AI-resident invocation + the S8 friction list (4.3), and the real transport/validation
boundary for the hostile driver (M9-F-F); lock M9-F-A…F-G (M9-F-A pre-locked by charter;
F-B/F-E/F-G shaped by §6); run the box micro-benchmark to ground §6.1; author the design →
runbook → Clair builds → close.

**Entry point (Rule 0):** CLAUDE PLAY → JOURNAL J-304 → this audit §2 + §4 + §5 + §6 →
`docs/tests/MULTIPARTY_S8_findings.md` (the AI-resident friction list) +
`docs/tests/MULTIPARTY_TEST_MATRIX.md` (the scenario catalogue + the saved-aicontrol-batch
convention) before framing the design.

Per D-065 + D-069 + D-071 + D-074 + D-078 + the two-round audit principle.
