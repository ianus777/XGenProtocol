# Multiparty Test S4 — Findings (M8 / Wave 2 / C4)
> **Status**: COMPLETED  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-05  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## What this records

The **C4** result of M8 Wave 2 (runbook `tasks/M8_MULTIPARTY_IMPL.md` §4 C4; design
`tasks/M8_MULTIPARTY_DESIGN.md` §3 S4 row + the **G-DURABILITY** cross-cutting gate). S4 is the
composite N×N chat-room with the durability/replay gate. First baseline (no historical "A";
M8-D3). B stamp: `8b14aa8` (≡ `676b9c1`).

---

## G-DURABILITY — the gate (new proof, `m8_s4_durability.rs`)

A Node restarts mid-run → replays its on-disk EventStore → resolved state returns
**byte-identical** with **zero orphans**. Proven on the production durability path:
`ingest`/`submit_locally` persist each event via `persist_event` (per-Space file-backed
vanilla store), `shutdown_keep_data` preserves the on-disk tree, and
`spawn_in_process_node_with_state` reloads it via `replay_spaces_from_dir` + registry loads —
the same helpers `run_node` uses at startup.

| Test | What it proves | Verdict |
|---|---|---|
| `s4_node_restart_replays_byte_identical_state_no_orphans` | Build a Space (create + room + member + 2 messages), snapshot resolved state, `shutdown_keep_data`, respawn from the preserved tree; the replayed `SpaceState` is **byte-identical** (`SpaceState: PartialEq`) and **every persisted event** (incl. the message DAG entries) re-loads — zero orphans, zero loss. | **PASS** |

```
$ cargo test -p xgen-node --lib m8_s4
running 1 test
test tests::m8_s4_durability::tests::s4_node_restart_replays_byte_identical_state_no_orphans ... ok
test result: ok. 1 passed; 0 failed; ...
```

**Storage engine note.** The vanilla file-backed EventStore replay path is what the gate
exercises; the sqlite engine (`xgen-store-sqlite`, `--all-features`) is a production-scale
backing, **not required** for the replay correctness proof. (The Durable EventStore milestone,
J-228/J-232, made a selected engine the durability authority; the vanilla path remains the
default replay source proven here.)

---

## N×N convergence (referenced)

The multi-actor / N-event convergence dimension of S4 is already proven by:
- **C2 / S2** (`m8_s2_convergence.rs`) — concurrent multi-actor state conflicts across Nodes,
  every arrival permutation, byte-identical resolved state + G-ALIGN (Layers 1/4/5c).
- **C3 / S3** (`m8_s3_federation.rs`) — multiparty migration convergence across 3 Nodes; and
  `phase9_three_node_anti_transitivity.rs` — N-message multi-Node fan-out delivery.

C4's genuinely-new contribution is the **restart-replay (G-DURABILITY)** dimension above; the
N×N convergence is not re-authored (M8 covers breadth without duplicating proven properties).

---

## The four metrics (M8-D2)

- **M1 — Delivery completeness.** Characterized: restart-replay loses zero events (every
  persisted event re-loads; assertion above). Live multi-client fan-out delivery is the S1-A /
  binary baseline.
- **M2 — Convergence correctness.** **CONVERGED** across restart — the replayed resolved
  `SpaceState` is byte-identical to the pre-restart state (the durability form of M2);
  multi-actor convergence referenced (C2/C3).
- **M3 — Integrity.** **Zero orphans** verified explicitly (every event present post-replay);
  zero duplicates (content-hash ids dedup); no `ERROR`/unexpected `WARN`.
- **M4 — Latency (informational; throughput NOT measured).** Restart-replay is local disk I/O;
  no network latency to report. Binary-level reference = S1-A.

---

## CP-4 placement + binary-half scope (incl. the 4-vs-3 Node decision)

Per CP-4/M8-D6, the G-DURABILITY correctness proof is workspace-homed (deterministic disk
replay; real processes add no signal). The operator-realistic **binary-level S4** — the
design's **4 Nodes / 6 Clients** real chat-room with a mid-run Node restart — is scoped to the
binary suite-execution pass (**CP-2 decision: 4 Nodes at binary level, composes at 3 if
4-process orchestration is flaky** — record the reduction in this file if taken). The
workspace durability proof needs a single Node (restart-replay is per-Node), and the multi-Node
convergence is covered by C2/C3 — so the binary 4-Node run adds operator-realism (real WS
fan-out under sustained load + a real `--stop`/restart cycle), not new correctness signal.

---

## Definition of Done — C4 (S4)

- [x] **Restart-replay resync verified** — byte-identical `SpaceState`, zero orphans (new
  test).
- [x] N×N convergence referenced (C2/C3) — not duplicated.
- [x] M1 characterized · M2 CONVERGED (durability form) · M3 zero orphans (explicit) · M4
  recorded.
- [x] CP-2 4-vs-3 Node decision recorded (binary half: 4 Nodes, composes at 3).
- [x] `cargo test --workspace` 1162/0/2; clippy clean both feature sets.
- [x] S5 (rebind) verdict recorded in `MULTIPARTY_S5_findings.md` (BLOCKED — M8-D4).

---

*End of MULTIPARTY_S4_findings.md — C4 (S4 durability) complete. See
`MULTIPARTY_S5_findings.md` for the rebind verdict.*
