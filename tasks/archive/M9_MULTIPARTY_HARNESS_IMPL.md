# M9 — Strategic Multiparty Test Harness — Implementation Runbook
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

The implementation runbook for M9, executing the J-305 Joe-LOCKED design
(`tasks/M9_MULTIPARTY_HARNESS_DESIGN.md` v1.0, M9-D1…M9-D9). Builds the strategic multiparty
test harness + the round dial + capture + the two Round-0 smokes (M9-D9). **Clair executes**;
Chat Claude authored. Three checkpoints (C1/C2/C4). No DECISIONS change (M9-D# arc-local). The
DoD has **no "commit pushed" line** — `Status: COMPLETED` is the shipped signal; Joe pushes.

---

## 2. Grounded surfaces (so Clair does not re-discover)

- **Workspace members:** `xgen-common`, `xgen-core`, `xgen-node`, `xgen-client`,
  `xgen-store-sqlite` (root `Cargo.toml`). **C1 adds `xgen-mptest`.**
- **Binaries:** built to `C:/cargo-targets/XGenProtocol/<profile>/`. Node `xgen-node.exe`,
  client `xgen-client.exe`. The orchestrator locates them from the target dir (honor
  `CARGO_TARGET_DIR`; fall back to the known path), or `cargo build -p xgen-node -p xgen-client`
  first.
- **Node launch:** `--instance <label>` (pipe namespacing), `--service` (headless `run_node`),
  local_mode (config default true, or `--local`), `--node <id>` (C9, honored). `--init` creates
  the keypair. Under `--service` the **`.aicontrol` sister server auto-starts** (app.rs:1248
  `start_aicontrol_server`).
- **Pipe names:** `pipe::pipe_name(instance)` → `None ⇒ \\.\pipe\xgen-node`; `Some("n1") ⇒
  \\.\pipe\xgen-node-n1`; client analogously `xgen-client-<label>`. `aicontrol_pipe_name(base)`
  (xgen-node aicontrol.rs:90) derives the `.aicontrol` pipe; `.events` pipe likewise. The
  orchestrator computes these from the labels it assigns.
- **aicontrol wire:** persistent JSONL — write one `Command` per line
  (`{"cmd","args","id","bind"}`), read one `Reply` envelope per line (flat `code`/`category`/
  `message`/`instance_state`).
- **Transport (for the injector):** `ws://{addr}/`; `xgen-core::transport::client::connect_url`
  + the `TransportMessage::Challenge`/`Auth` handshake, then `Connection::send_event`. Crafted
  `Event`s built with `xgen-core` builders, then corrupted.
- **Clock:** the `mock-clock` feature + `RealClock`/`MockClock` (M8.6 seam) — built into the
  binaries; MockClock advance is driven via the harness for R1.

---

## 3. Commit plan (C1…C6)

### C1 — orchestrator crate + process lifecycle + batch runner + manifest
- Add `xgen-mptest` workspace member (test-only; lib + a `cargo run` entry and/or `#[ignore]`
  integration tests — heavy, spawns real processes, **out-of-band from the fast unit suite**).
- **Process lifecycle:** temp data dir + `--init` keypair per node/client (RAII teardown via
  `tempfile` + kill-on-drop); spawn node(s) (`--service --instance <l> [--node <id>] [--local]`,
  worker-threads pinned to 1–2 via env/flag); establish federation relationships; spawn
  client(s) / AI-resident(s) (`--ai-mode --service` for the cooperative crowd).
- **aicontrol client:** connect to each actor's `.aicontrol` pipe; write JSONL lines, read Reply
  envelopes, retain `id`-correlated results + named `bind` values.
- **Batch runner:** read `<actor>.jsonl`, feed line-by-line, capture replies.
- **Manifest:** parse `docs/tests/multiparty_scenarios/<ID>/manifest.toml` (actors → node
  assignment, batch file, ordering/barriers, `exports` = reply fields published, `imports` =
  `{{key}}` placeholders consumed). Resolve `{{key}}` from a prior actor's exported reply field
  before sending; enforce ordering via the manifest barriers.
- **⛳ Checkpoint #1 (Joe-lock, light):** confirm the exe-location strategy, the pipe-connect
  mechanism, the `manifest.toml` schema, and the cross-actor `{{}}` resolution shape — before
  the oracle is built on top. Surface, do not work around, any pipe/launch friction (the S8
  list).

### C2 — convergence oracle + capture-by-default
- **Oracle (M9-D4):** per node, read the `state` verb projection + collect the `.events`
  transcript; assert equality across all nodes hosting the Space for convergence scenarios;
  assert offending-event **absence** + the expected envelope `code`/`category` for rejection
  scenarios.
- **Capture:** per-run artifact dir — per-actor command/reply logs, per-node `.events`
  transcripts, per-process RSS + thread-count samples, the oracle verdict, the resolved manifest.
- **⛳ Checkpoint #2 (Joe-lock, light):** confirm the equality definition (which `state` fields
  must match; how `.events` order is compared) — the cross-process analogue of the in-process
  `RoomState` `Eq` oracle.

### C3 — round dial + micro-benchmark
- Parameterize `N nodes × M clients × R residents/process × ramp profile × clock-mode`
  (MockClock | real-clock).
- **Micro-benchmark:** spawn 10 / 50 / 100 processes, sample RSS + thread count, write a
  box-ceiling report into the artifact dir (grounds the audit §6.1 numbers for the 32 GB/20-core
  box before R2/R3 numbers are fixed).

### C4 — raw-wire injector (F-F, M9-D6)
- Minimal test-only wire-client in `xgen-mptest`: `connect_url(ws://…)` + auth, then
  `send_event` of a crafted `Event`. Attacks (one invariant violated each): forged signature
  (wrong key), malformed/truncated frame, duplicate `event_id`, equivocation (conflicting events
  to two nodes), clock-skew timestamp, forged-invite reference.
- **⛳ Checkpoint #3 (Joe-lock):** confirm each attack actually reaches the F-4 13-step `validate_event`
  (exchange.rs; `ingest_event` at runtime.rs:481 is the **no-validation** direct-insert, NOT the
  boundary) and is rejected there — not bounced earlier at the WS/auth layer in a way that
  does not exercise validation. Record the rejection point per attack (a finding if any attack is
  silently accepted — surface per D-065/D-084).

### C5 — Round-0 smokes (M9-D9)
- **Cooperative MP-C-02:** author `docs/tests/multiparty_scenarios/MP-C-02/manifest.toml` (the
  `alice.jsonl` + `bob.jsonl` batches already seeded); run end-to-end; assert Bob is a member on
  **both** nodes + S converges (oracle). Exercises spawn → drive → `{{}}` → oracle.
- **Adversarial MP-A-05:** forged-signature injection via C4; assert rejection at `validate_event` (F-4 step 12)
  + absence from every node's converged state.
- Both update the matrix Result fields (PENDING → PASS + run ref) for MP-C-02 / MP-A-05 only.

### C6 — close (doc-only)
- Audit + design + this runbook → `Status: COMPLETED`. **Matrix stays ACTIVE** (only MP-C-02 /
  MP-A-05 flip to PASS; the rest remain PENDING for the Multiparty-tests runs).
- **Evaluate the `Clock`-trait DECISIONS promotion** (M9-D5): M8.6 → INV-EXP → M9 = 3 distinct
  reuses; record whether it reaches the four-recurrence-durable bar (likely stays a
  promotion-watch unless Joe promotes).
- JOURNAL close entry; ROADMAP M9 → ✅ CLOSED + version bump; CLAUDE PLAY close block.
- **Next-active after close:** the unnumbered **Multiparty-tests** milestone (run R1 → R2 → R3
  on a finalized binary), or Joe selects.

---

## 4. Verify (each code commit)

- `cargo build --workspace --all-targets` → 0
- `cargo clippy --workspace --lib --tests -- -D warnings` → clean (default + `--all-features`)
- `cargo test --workspace` → **the fast unit suite stays green** (pre-M9 1212/0/2; M9 added pure-logic helpers → final **1262/0/8**, incl. 6 `#[ignore]` real-binary smokes) (the harness smokes are
  `#[ignore]`/out-of-band — they spawn real processes); run them explicitly
  (`cargo test -p xgen-mptest -- --ignored` or `cargo run -p xgen-mptest -- <scenario>`) and
  record the Round-0 results in the artifact dir + the matrix.

---

## 5. Scope guards

- **M9 builds machinery + Round-0 only.** Do NOT author the full MP-C / MP-A batteries (that is
  the Multiparty-tests milestone). The 37 matrix scenarios stay PENDING except the two smokes.
- **No protocol/code changes to the binaries** beyond what the harness needs to *drive/observe*
  them (the binaries are the system-under-test; if a real defect surfaces, surface it as a
  finding routed to a fix-arc — do not patch under the M9 banner). The S8 friction list may
  justify small *ergonomic* aicontrol additions — Joe-lock any such change.
- **Injector is test-only**, lives in `xgen-mptest`, never ships in a binary.

---

## 6. DoD checklist

- [ ] C1 `xgen-mptest` + lifecycle + aicontrol client + batch runner + manifest/`{{}}`;
      Checkpoint #1 closed
- [ ] C2 oracle (`state` + `.events` equality / absence) + capture-by-default; Checkpoint #2
      closed
- [ ] C3 round dial + micro-benchmark (box-ceiling report written)
- [ ] C4 raw-wire injector; Checkpoint #3 closed (per-attack rejection point recorded)
- [ ] C5 Round-0 smokes green (MP-C-02 cooperative + MP-A-05 adversarial); matrix Result updated
- [ ] C6 close — audit/design/runbook → COMPLETED; Clock-promotion evaluated; JOURNAL + ROADMAP
      + CLAUDE PLAY; matrix stays ACTIVE
- [ ] build 0 · clippy clean both feature sets · fast suite green (final 1262/0/8) · Round-0 results captured

---

## 7. As-built (close, J-307)

Closed J-307. C1–C5 shipped (commits 06cabb7 / 54fa102 / b318a21 / 99b9a71 / 6d08859); C6 doc-only.
- **Round-0:** MP-C-02 ✅ PASS (single-node, Option 1 Joe-lock — true cross-node gated on the F2 fresh-peer initiate surface) · MP-A-05 ✅ PASS (`validate_event` step-12 signature rejection).
- **Suite:** final **1262/0/8** (pre-M9 1212/0/2 + 50 pure-logic helpers + 6 `#[ignore]` real-binary smokes); build 0; clippy clean both feature sets.
- **Findings → `tasks/M9_findings.md`** (routed, NOT patched under M9): F1 no timestamp-bound validation (gap G6); F2 fresh-peer federation-initiate surface; F3 MockClock inoperable across the process boundary; F4 malformed-frame raw-send seam; + the resolved `ingest_event`→`validate_event` citation.
- **`Clock` promotion (M9-D5):** NOT promoted — M9 did not operably reuse MockClock (F3), so it is not a true 3rd runtime reuse; stays a promotion-watch.
- **Box micro-benchmark:** ~18.4 MB RSS/node, 3 threads, est. ceiling ~1,562.

**Next-active:** the unnumbered **Multiparty-tests** milestone (R1 → R2 → R3 on a finalized binary), gated on F2/F3; or Joe selects.

**Entry point for Clair (Rule 0):** CLAUDE PLAY → JOURNAL J-305 →
`tasks/M9_MULTIPARTY_HARNESS_DESIGN.md` §2 + §3 + §6 → this runbook §2 + §3 →
`docs/tests/MULTIPARTY_TEST_MATRIX.md` (MP-C-02 + MP-A-05) →
`docs/tests/MULTIPARTY_S8_findings.md` (aicontrol friction).

Per D-065 + D-069 + D-071 + D-074 + D-078 + D-084.
