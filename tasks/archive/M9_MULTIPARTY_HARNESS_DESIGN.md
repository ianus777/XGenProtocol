# M9 — Strategic Multiparty Test Harness — Design
> **Status**: COMPLETED  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-07  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. Purpose

The M9 design phase, Joe-LOCKED 2026-06-07. Executes the J-304 Phase-0 audit
(`tasks/M9_MULTIPARTY_HARNESS_AUDIT.md` v1.2) and the charter. Locks the seven forks as
**M9-D1…M9-D9** (arc-local per D-069 — no DECISIONS change at design time). Doc-only; no code.
Deliverable feeding the runbook → Clair. Companion: the scenario catalogue
`docs/tests/MULTIPARTY_TEST_MATRIX.md` (expanded to v1.1 this session).

---

## 2. Locked decisions (M9-D1…M9-D9)

- **M9-D1 (F-A) — Drive surface = `--aicontrol`.** The persistent JSONL command pipe is the sole
  harness driver. `--batch` stays frozen for human use (D-066). The node surface (`admin_ops::*`
  + `state`) and client surface (`register`/`create-space`/`create-room`/`create-dm-space`/
  `invite`/`join`/`send`/`members`/`leave`) are both reached this way.
- **M9-D2 (F-B) — Two-number participant model.** Scale = real OS *processes* (topology width,
  HW-bound) × AI-resident-multiplexed *logical participants* (load depth, cheap). Cooperative
  load is AI-resident-driven (`--ai-mode --service`); adversarial wire-attacks use the hostile
  driver (M9-D6).
- **M9-D3 (F-C) — Orchestrator = a dedicated test-only Rust crate.** A new workspace member
  (working name `xgen-mptest`) that **spawns the real built `.exe`s** via `std::process`
  (located through cargo's `CARGO_BIN_EXE_*` env), drives each through its `.aicontrol` pipe, and
  observes through its `.events` pipe. It does **not** link the binaries — black-box over the
  real deployment surface. Runs under `cargo test` / `cargo run -p xgen-mptest`. (Exact crate
  layout finalised in the runbook.)
- **M9-D4 (F-D) — Convergence oracle = `.events` + `state` equality across nodes.** The
  cross-process analogue of the in-process `RoomState` `Eq` oracle: for each scenario, compare
  the per-node `state` projection and the ordered `.events` transcript across all participating
  nodes; assert byte/semantic equality where convergence is claimed.
- **M9-D5 (F-E) — Clock = both modes.** MockClock (deterministic) for R1; real-clock for R2/R3.
  The harness reuses the M8.6 `Clock` seam (the binaries already carry `RealClock`/`MockClock`
  behind the `mock-clock` feature). **This is the 3rd reuse of the seam (M8.6 → INV-EXP →
  M9)** → the `Clock`-trait DECISIONS promotion is **re-evaluated at M9 close** (the four-
  recurrence-durable test; not promoted at design time).
- **M9-D6 (F-F) — Hostile driver = raw-wire injector.** Grounded against the validation boundary:
  the F-4 13-step **`validate_event`** (exchange.rs, in `dispatch_event`/`process_inbound`)
  rejects forged/malformed events (step 12 = signature); `ingest_event` (runtime.rs:481) is the
  **no-validation** direct-insert (`None => return` only), NOT the boundary; and the
  client binary's `ops::*` cannot *build* a forged event — so wire-attacks require a **minimal
  Rust wire-client** that speaks the transport directly to a Node's `Server`/`handle_connection`,
  crafting frames (forged signature, malformed bytes, equivocation to different nodes). The
  injector is test-only, lives in the orchestrator crate, and reuses `xgen-core` wire types to
  build (then deliberately corrupt) `Event`s. **Logic-attacks stay batch-expressible** (M9-D8);
  only wire-malformation/forgery/equivocation use the injector.
- **M9-D7 (F-G) — Scale = parameterized round dial.** The harness takes `N nodes × M clients ×
  R residents/process × ramp profile × clock-mode` as parameters — never a hardwired constant.
  Box grounded: **32 GB / 20-core (Intel Ultra 7 265KF)** → the ~800–1,200-process tier (stretch
  ~1,500); the big rounds want the box otherwise idle. **First build task = a micro-benchmark**
  (spawn 10/50/100, measure RSS + thread count) to derive the real ceiling before R2/R3 numbers
  are fixed. Worker-threads pinned (1–2) per spawned binary to avoid scheduler thrash.
- **M9-D8 — Scenario format = per-actor JSONL batches + a manifest.** Each scenario is a
  directory `docs/tests/multiparty_scenarios/<ID>/` containing: one `<actor>.jsonl` per actor
  (saved, versioned aicontrol command batches — `Command` envelopes, in-connection `bind`/`$`
  chaining) and a **`manifest.toml`** declaring actors → {node assignment, batch file,
  ordering/barriers, exported reply keys, imported `{{key}}` placeholders}. **Cross-actor
  values** (e.g. Bob's `join` needing Alice's `space_id`) are `{{key}}` placeholders the
  orchestrator fills from a prior actor's **exported** reply field — `bind`/`$` alone is
  per-connection and cannot cross actors. The orchestrator enforces ordering via the manifest's
  barriers.
- **M9-D9 — Scope = harness + dial + capture + Round-0 smokes.** M9 ships the orchestrator, the
  round dial, capture-by-default, and **one cooperative smoke + one adversarial smoke**
  ("Round 0") proving the machinery end-to-end. The full batteries + the real R1/R2/R3 numbers
  are the unnumbered **Multiparty-tests** milestone on a finalized binary.

---

## 3. Harness architecture

### 3.1 Lifecycle
Per scenario the orchestrator: (1) creates temp data dirs + real keypairs (`--init`) per node and
client; (2) spawns node `.exe`(s) (`--service` headless, worker-threads pinned, `--node` honored
per C9) and establishes federation relationships; (3) spawns client `.exe`(s) and/or AI-resident
`.exe`(s) (`--ai-mode --service`); (4) drives each actor's `.jsonl` batch through its `.aicontrol`
pipe in manifest order, resolving `{{...}}` placeholders from exported replies; (5) for
adversarial wire scenarios, runs the M9-D6 injector against the target node's transport; (6)
quiesces, runs the M9-D4 oracle; (7) tears down (kill processes, capture artifacts, clean temps).

### 3.2 The AI-resident as the crowd
Cooperative mass-use is supplied by AI-resident processes (M4, A-pure G-ALIGN apply path) each
multiplexing many logical participants — the M8 §6 mechanism. The `MULTIPARTY_S8_findings.md`
friction list is the input punch-list for making the resident harness-drivable (addressed in the
runbook against the live resident).

### 3.3 Convergence oracle (M9-D4)
Two readings per node: the `state` control verb (membership/room/epoch projection) and the
`.events` transcript (ordered applied events). Equality across all nodes hosting the Space is the
PASS condition for convergence scenarios; rejection scenarios assert the offending event is
**absent** from every node's state and the expected envelope `code`/`category` was returned.

### 3.4 Capture-by-default
Every run writes an artifact dir: per-actor command/reply logs, per-node `.events` transcripts,
per-process RSS + thread-count samples, the oracle verdict, and the resolved scenario manifest.
R3's one-shot is only meaningful with this (audit §6.2 rec 3).

---

## 4. The hostile driver (M9-D6, F-F)

A minimal test-only wire-client in the orchestrator crate. It connects to a target Node's
`Server` like a peer/client and sends `Event` frames it constructs with `xgen-core` builders, then
**deliberately violates** one invariant per attack: a signature over the wrong key (forgery), a
truncated/garbage frame (malformed), the same `event_id` twice (replay/dedup), conflicting events
to two nodes at one frontier (equivocation), a far-future/past timestamp (skew), or a reference to
a non-existent invite/space (forged-capability). Expected outcome is always **rejection at `validate_event` (F-4 step 12)** (or buffering-then-drop for causal gaps) with the event never reaching any node's
converged state. Volume-attacks (flooding, connect/disconnect storms, slow-loris) reuse the
injector at high rate / abnormal connection patterns and assert no-hang + local liveness (the
M8.6 C8 property at the binary boundary).

---

## 5. Scenario coverage → subsystem map

The matrix (v1.1) scenarios map onto every shipped subsystem, so the campaign is a coverage
surface, not a sampler:

- **Membership / resolution** — MP-C-01/02/03, MP-A-02/03/04 → M8 convergence, INV bootstrap,
  PG-13 tier-gate.
- **Federation / propagation** — MP-C-04/14, MP-A-01/13 → F-5/D-089 pairwise trust,
  anti-transitivity, INV-EXP replay.
- **Identity** — MP-C-06, MP-A-14/16 → S5 re-home, ban-evasion, invite forgery.
- **Rooms / threads / DM** — MP-C-07/08/13 → PG-12 overrides, Arc E Thread, DM spaces.
- **Durability / migration** — MP-C-15/16 → S4 replay, Arc F migration.
- **Encryption** — MP-C-12, MP-A-21 → Arc H content-blindness, M8.7 commit-race.
- **Resource / transport** — MP-A-05/06/09/10/11/12/15/17/18/19/20 → wire validation,
  back-pressure (M8.6), privilege enforcement.

---

## 6. Proof plan — Round-0 (M9-D9)

M9's own acceptance: the harness + dial + capture run green on **two** smokes against the real
binaries:
- **Cooperative smoke = MP-C-02** (invite & join across two real nodes; Bob a member on both,
  S converges) — exercises spawn → `.aicontrol` drive → cross-actor `{{...}}` → `.events`/`state`
  oracle end-to-end.
- **Adversarial smoke = MP-A-05** (forged-signature injection via M9-D6) — exercises the hostile
  driver and asserts rejection at `validate_event` (F-4 step 12) + absence from converged state.

Build sequence (runbook details): (C1) orchestrator crate + process lifecycle + batch runner +
manifest/`{{...}}`; (C2) the `.events`/`state` oracle + capture; (C3) the round dial + the
micro-benchmark; (C4) the M9-D6 injector; (C5) the two Round-0 smokes; (C6) close. The full
batteries are **not** built here.

---

## 7. Coverage ledger / honest boundary (D-065)

- The cooperative leg uses the AI-resident as a **stand-in** for human participants — real
  human-crowd behaviour is not exercised (M8 §6; named, not glossed).
- Round-0 proves the **machinery**, not coverage — a green M9 means the harness works, not that
  the system passed the suite. Coverage is the Multiparty-tests milestone's claim.
- The injector simulates a hostile peer at the wire; it does **not** model a compromised honest
  binary (insider) — out of scope, named.
- MLS loser-rebuild + real key schedule remain **L** (Arc H / M8.7 boundaries), so encryption
  scenarios prove envelope/epoch behaviour, not production-client crypto.

---

## 8. Next-active

**Runbook** (`tasks/M9_MULTIPARTY_HARNESS_IMPL.md`) — sequence C1…C6 above, with checkpoints at
the orchestrator-shape boundary (C1), the oracle (C2), and the injector (C4); then Clair builds.

**Entry point (Rule 0):** CLAUDE PLAY → JOURNAL J-305 → this design §2 + §3 + §6 →
`docs/tests/MULTIPARTY_TEST_MATRIX.md` (v1.1) → `docs/tests/MULTIPARTY_S8_findings.md`.

Per D-065 + D-069 + D-071 + D-074 + D-078.
