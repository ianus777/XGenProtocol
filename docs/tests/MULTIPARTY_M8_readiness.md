# Multiparty M8 — Wave 1 / C1 Readiness Note
> **Status**: COMPLETED  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-05  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## What this file is

The **C1 readiness artifact** for M8 (runbook `tasks/M8_MULTIPARTY_IMPL.md` §3 C1). It records the
five pickup checks (CP-1…CP-5, D-078) run against the **B** binary **before** any scenario
results, so the suite is runnable and the scenario authors (C2+) inherit grounded decisions.
No scenario results here — those live in `MULTIPARTY_Sn_findings.md`.

**Entry chain (Rule 0):** CLAUDE.md PLAY → JOURNAL J-268 → `tasks/M8_MULTIPARTY_DESIGN.md`
§2–§3+§5 → `tasks/M8_MULTIPARTY_IMPL.md` §2–§3 → this note.

---

## B build stamp (CP-3)

The "B" target per M8-D3 is HEAD `676b9c1`. The actual current HEAD is `8b14aa8`; the two
intervening commits (`9a8c5e1` M8-open, `8b14aa8` runbook) are **doc-only** — verified
`git diff --name-only 676b9c1..HEAD -- '*.rs' '*.toml'` returns **zero source changes**. So a
binary built from current HEAD is code-identical to the M8-D3 B target.

```
$ xgen-node.exe version
xgen-node 0.10.3.260605-1454
Commit:   8b14aa8
Node ID:  (no keypair — run 'xgen-node init')

$ xgen-client.exe version
xgen-client 0.10.3.260605-1454
Commit:  8b14aa8
```

Built `cargo build --release -p xgen-node -p xgen-client` (target `C:/cargo-targets/XGenProtocol`,
per `.cargo/config`). **Every `MULTIPARTY_Sn_findings.md` "B" column uses:
`0.10.3.260605-1454`, commit `8b14aa8` (≡ code of `676b9c1`).**

**Harness builds clean (C1 DoD):** `cargo test --workspace` → **1156 passed / 0 failed /
2 ignored**, 0 build errors (full-capture sum over all `test result:` lines). Matches the
J-268 baseline. The 2 ignored are the pre-existing documented ignores.

---

## CP-1 — `.xgb` verb-surface drift

**Verdict: ZERO verb-delta on the 13 on-disk S1 scripts. No script patching required.**

The 13 scripts (`docs/tests/scripts/multiparty_s1_*.xgb`) use exactly six verbs. Each was
checked against the **real B binary** (`xgen-client <verb> --help`), not just source:

| Script verb (S1) | B-binary usage (clap) | Match |
|---|---|---|
| `register --name <n>` | `register [OPTIONS] --name <NAME>` | ✅ |
| `create-space --name <n>` | `create-space [OPTIONS] --name <NAME>` | ✅ |
| `create-room --space <s> --name <n>` | `create-room [OPTIONS] --space <SPACE> --name <NAME>` | ✅ |
| `send --space <s> --room <r> --text <t>` | `send [OPTIONS] --space <SPACE> --room <ROOM> --text <TEXT>` | ✅ |
| `join --space <s> [--room <r>]` | `join [OPTIONS] --space <SPACE>` (`--room` optional) | ✅ |
| `status` | `status [OPTIONS]` | ✅ |

**Why this holds despite M6/M7 drift:** M6/M7 were *additive* on the client surface
(`create-dm-space`, `leave`, `members`, `ai …` were added; nothing the S1 scripts use was
moved or renamed) and `--batch` was untouched (D-066). The `--batch` executor parses through
the **same** canonical `Cli` clap parser as the direct CLI (`xgen-client/src/batch.rs:166`
and `app.rs:821`), so clap is the single drift gate — a moved/renamed verb would fail there.

**`connect` is NOT a verb on B.** The node is set via the global `--node` flag or config; the
13 S1 scripts already reflect this connect-less shape (they never use `connect`).

**Stale templates flagged (not on-disk scripts):** the S2–S5 instruction-file Appendix
templates (`MULTIPARTY_S2…S5_*.md`, never instantiated as files) reference a `connect ws://…`
verb and `register --name X --passphrase Y`. **Both are stale** — there is no `connect` verb,
and `register` takes only `--name` (passphrase moved to `init`). **Any C2+ script authoring
must follow the S1 on-disk shape**, not the S2–S5 templates: no `connect`; node via
`--node`/config; passphrase via `init`; literal IDs via two-pass capture (the `--batch`
dispatcher has no `@last_space`/`@last_room` backreference support — confirmed in S1 findings
M0.4 and unchanged on B).

**Live end-to-end proof (not just clap-in-isolation):** ran the self-contained pass1 script
through the real B `--batch` path against a fresh local-mode node on `ws://127.0.0.1:8090`:

```
$ xgen-client --instance m8c1a --node ws://127.0.0.1:8090/xgen \
    --batch docs/tests/scripts/multiparty_s1_smoke_clientA_pass1.xgb
Identity registered successfully.  ... alice
Space created:  Space ID: xgen://hash/sha256:873ba6e9...   (fresh B-run id)
xgen-client status  ... Spaces joined: 1
Batch complete: 3 commands executed, all succeeded.
```

register → create-space → status, **3/3, zero verb errors, real protocol execution**. The
fresh Space ID differs from the A-run's hardcoded ID (expected — content-hash of a new
keypair's events), confirming the two-pass capture-IDs dance is inherent to the binary harness,
not a drift.

---

## CP-2 — federation-harness ceiling & the 3-vs-4-Node decision

**There is no node ceiling in the workspace harness.** `phase9_harness::spawn_in_process_node()`
returns one in-process Node; `federate(a, b, shared_spaces)` connects any pair via the
production `attempt_reconnect`. N Nodes = spawn N + federate pairwise. The 3-Node case is
already proven (`phase9_three_node_anti_transitivity.rs` = 3 spawns + 2 `federate` calls;
`phase9_m8_convergence_smoke.rs` = 2-Node convergence oracle via `InProcessNode::space_state`).

**The only genuine scale-up is S4's 4 Nodes at the BINARY level** (4 real `xgen-node.exe`
on 4 ports + 6 real `xgen-client` instances) — OS-process orchestration cost, not a harness
limit. Single-node binary orchestration is proven working this session (CP-1 live pass1).

**Decision (recorded; reconfirm at S4 / Wave 2 Joe-lock):**
- **Convergence correctness (M2)** is proven at the **workspace** level — it needs ≤ 3 Nodes
  (two independent Nodes fed opposite arrival orders is the faithful realisation of
  arrival-order independence; see `phase9_m8_convergence_smoke.rs` rationale). No 4th Node
  needed for any convergence proof.
- **S4 (N×N real chat)** targets the design's **4 Nodes / 6 Clients at binary level**; it
  **composes at 3 Nodes** if 4-process orchestration proves flaky in practice (record the
  reduction in `MULTIPARTY_S4_findings.md` if taken — M8-D6 / Joe-lock).

---

## CP-4 — per-scenario placement (binary-level vs workspace integration test, M8-D6)

Default: **binary-level for anything an operator/UI would do**; **workspace integration test
only where real processes add no signal** (a deterministic correctness proof already provable
in-process). Per-scenario calls (reconfirm per scenario):

| Scenario | Placement | Rationale |
|---|---|---|
| **S1** re-run | binary-level | apples-to-apples with the historical-A (which was binary-level local fan-out) |
| **S2** convergence (C2) | **hybrid** | M2 byte-identical + G-ALIGN convergence = **workspace** (deterministic permutation proof; real processes add no signal); operator-realistic concurrent federation send + DAG sibling/order coherence = **binary-level** |
| **S3** topology/jurisdiction/migration | binary-level | real federation across Nodes is exactly what real processes prove; jurisdiction-reject + migration `home_node` flip need the real drivers |
| **S4** N×N + durability | binary-level | operator-realistic chat + restart-replay resync |
| **S5** rebind | binary-level | identity portability across real Nodes |
| **S6** E2E blindness | hybrid | content-blindness invariant = workspace (the Arc-H proof already is one); multiparty encrypted fan-out + KeyPackage pool/epoch-advance = binary-level |
| **S7** privilege | binary-level | tier-gate refusal + per-Room override observed by all members; workspace backup for pure enforcement assertions |
| **S8** AI participant | binary-level | `--ai-mode --service` residents (see CP-5) |

---

## CP-5 — `--aicontrol` live-membership viability & the AI-participant (S8) path

**`--aicontrol` (and `--batch`) CANNOT hold a live room membership.** Both are **one-shot**:
each command opens a WS, sends, and disconnects (`ops::join` ends with
`conn.goodbye("client_disconnect")`, `xgen-client/src/ops.rs:803`; sessions are one-shot,
`session.rs:14-20`). No held WS → no live fan-out reception. The `--aicontrol` pipe wraps the
same one-shot `ops::*` as `--batch` (`xgen-client/src/aicontrol.rs`, sister to `batch.rs`).

**The human `--service` resident is a stub for ingest** — it holds a WS but **drains and
discards** inbound events (`xgen-client/src/service.rs:19-20,114-115`: "events ignored at this
layer for M1 MVP — real ingest wiring is M3 work"); `main.rs:15` "stub until 2b/M3". This is
the audit §5 stub.

**The AI resident `--ai-mode --service` DOES hold a live membership and receives fan-out** —
it is the real M4 resident (`xgen-client/src/ai_service.rs`): persistent WS
(`run_ai_loop`), per-Space `SpaceState` reconstructed from inbound via the node-style ingest
gate (R2-F01 C2, `ai_service.rs` `apply_or_rebuild`), and reply emission under pacing/mute.

**S8 path (recorded; reconfirm at C7 / Wave 3):** drive an **`--ai-mode --service` AI
resident** as the first-class member — it already holds live membership + receives fan-out;
join it via a one-shot `--batch`/`--aicontrol` `join` to its own data dir before/while the
resident runs, then the resident observes fan-out and replies. **No resident mode is built**
(M8-D2 non-goal; M4 already shipped the AI resident). The CP-5 "fold S8 into a scripted S4
variant" fallback is **not needed** — the live AI-membership path exists. (If the AI-resident's
join + observe loop proves unable to hold membership across a session at binary level when
exercised in C7, record the limitation there and fall back to the scripted S4 variant — do
not build resident mode.)

---

## C2 conflict-set viability (grounding for the next commit)

C2 extends S2 from concurrent **message** sends to concurrent **state** events that genuinely
conflict, proving M2 (byte-identical resolved `SpaceState` across all Nodes + every client
projection / G-ALIGN, every arrival permutation). The seven-layer `resolve()`
(`xgen-core/src/resolution/algorithm.rs`) + `state_key_for_event`
(`…/resolution/state_key.rs`) were grounded. The runbook's three minimum cases map to
distinct resolution layers; all chosen pairs are **live + buildable on B** (builder + applier
both present):

| Case | Conflict pair | Resolution layer | Builder / applier (B) |
|---|---|---|---|
| ban-vs-join | `MembershipBan(target)` vs `MembershipJoin(target)` (shared `membership:space:target`) | **Layer 1** — ban wins (removal precedence) | `build_membership_event` + membership appliers ✅ |
| role conflict | two concurrent same-type state events by senders of different roles (e.g. owner vs admin `state.room_update` on one room, OR owner vs admin `MembershipInvite` of one target) | **Layer 4** — higher role wins | `build_room_update_event`/`build_membership_event` + `apply_room_update`/membership appliers ✅ |
| key-rotation → **substituted** | `thread.resolved` vs `thread.archived` on one thread (shared `thread.status:thread`) | **Layer 5c** — lexicographic backstop | `build_thread_resolved_event`/`build_thread_archived_event` + `apply_thread_status` ✅ |

**Substitution recorded (M8-D4 finding, NOT a blocker).** `system.key_rotation` has a
`state_key_for_event` arm (`state_key.rs:111`, keyed on sender) **but no builder and no
`apply_event` arm** — it is a dormant forward-ready EventType (J-166 / Arc-H lineage). A
concurrent key-rotation conflict is therefore **not buildable on B without new wire surface**,
which M8 must not add. It is replaced by `thread.status` resolved-vs-archived, which exercises
the **same resolution layer** (5c lexicographic) the key-rotation case would have. The
unbuilt key-rotation path is an **M9-scoping input** (feeds the multiparty redesign / Arc-H
real-crypto), not an in-arc fix (M8-D4: a surfaced gap is a success).

These three layers (1 / 4 / 5c) match the breadth the design intends and the existing
convergence proofs (`resolution::algorithm::tests`, `phase9_m8_convergence_smoke.rs`).

---

## test_runs / workspace convention (S0 §"Test data directories")

- Binary-level scenarios run real `.exe` under **`test_runs/m8_<scenario>/`** (M8 convention;
  S0 used `test_runs/multiparty_s<N>_run<k>/`). Prior data archived to
  `test_runs/multiparty_s<N>_<timestamp>_pre/` then deleted; archive path recorded in findings.
- Build output is `C:/cargo-targets/XGenProtocol` (`.cargo/config`); the C1 sandbox used
  `…/release/instances/<label>` (outside the repo tree) and is disposable.
- Instance labels per S0 §"Instance labels" (`m2nA`/`m2a`, etc.); content-leak `findstr` per
  scenario; latency informational (M4); **throughput NOT measured** (M8-D2).

---

## Historical-A baseline reference (M8-D3)

S1 "A" = commit `7e06896` (2026-05-16), `MULTIPARTY_S1_findings.md` (COMPLETED): P1 smoke
cell-perfect (9-row pairing table, zero miss, zero content leak); P2 stress **294/300 = 98%**,
**6/300 silent loss** (client WS write → Node receive race; no error/timeout/duplicate/orphan).
Three follow-ups: (1) unify `get_dag_tips` — **done on B** (single canonical
`xgen-client/src/batch.rs:87`); (2) characterize the 2% loss — open; (3) long-lived client
mode for throughput — **unbuilt** (the M8-D2 throughput non-goal blocker). Per M8-D3: S1
records "A historical / B measured / deltas explained" (no `7e06896` rebuild); S2–S8 establish
their own baseline.

---

## C1 Definition of Done

- [x] **CP-1** verb-delta recorded — zero delta on the 13 on-disk S1 scripts (real-binary
  clap proof + live pass1 `--batch` run); S2–S5 template staleness flagged; no script patch needed.
- [x] **CP-2** 3-vs-4-Node decision recorded — no harness ceiling; S4 = binary-level 4 Nodes
  (composes at 3); convergence proofs need ≤ 3.
- [x] **CP-3** B-stamp captured — `0.10.3.260605-1454` / `8b14aa8` (≡ `676b9c1` code).
- [x] **CP-4** per-scenario placement defaults recorded.
- [x] **CP-5** `--aicontrol`-membership finding recorded — one-shot, cannot hold membership;
  `--ai-mode --service` AI resident is the live S8 path; no resident build.
- [x] Scripts run end-to-end against B without verb errors (live pass1 3/3).
- [x] Harness builds clean — `cargo test --workspace` 1156/0/2.
- [x] C2 conflict-set grounded (Layers 1/4/5c live; key-rotation substituted + recorded).

---

*End of MULTIPARTY_M8_readiness.md — C1 complete. Next: C2 (S2 concurrent state-event
convergence). STOP at end of Wave 1 (C1+C2) = Joe-lock checkpoint #1.*
