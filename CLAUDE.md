# XGen Protocol — Claude Code Briefing
> For: Claude Code (claude.ai/code)  
> Date: May 2026  
> **Status:** ACTIVE  
> **Last updated:** 2026-05-17 (CLI Precedence Audit SHIPPED — J-079; M6 now ACTIVE)  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  

---

## 🔴 MANDATORY — Behaviour rules (read before doing anything else)

These rules exist because fabricated results have occurred. A summary that says "done" when the work was not actually done causes real damage — wasted sessions, false confidence, incorrect state in CLAUDE.md and JOURNAL.md. Honesty about failure is always better than a fabricated success.

**Rule 1 — Never fabricate results.** If a command fails, report the failure. Do not describe what the output *should* have been. Do not write a journal entry claiming success until success is actually confirmed.

**Rule 2 — Show actual output, not a description of output.** Every verification step requires quoting real terminal output in the journal entry. Do not paraphrase. Do not summarise. Paste the actual lines. If you cannot produce the actual output, the verification step is not complete.

**Rule 3 — Stop and report when a tool fails.** If a shell command, file operation, or any tool call fails or returns an unexpected result: (1) stop immediately, (2) report exactly what failed and the error, (3) do not attempt to work around it silently, (4) do not write a success summary. Joe will decide how to proceed.

**Rule 4 — Write the journal entry last.** The JOURNAL.md entry is written *after* all work is complete and all verification steps are confirmed with real output. Order: do the work → run verification → confirm outputs → write journal entry quoting actual output → update CLAUDE.md → commit and push.

**Rule 5 — Never invent numbers.** Test counts, file counts, line counts — these must come from actual command output. If you did not run `cargo test`, you do not know the current test count — say so.

**Rule 6 — When in doubt, do less and ask.** If a task instruction is ambiguous, or completing it would require a decision not covered by the instruction file, stop and flag the ambiguity. Do not make the decision silently. Write a clear question to Joe and wait.

**Rule 7 — Definition of Done is a checklist, not a formality.** Every task file ends with a Definition of Done checklist. Each item must be independently verified before being marked complete. Mark items complete only when confirmed with actual output or observation.

| Situation | Correct behaviour |
|---|---|
| Command succeeds | Quote actual output in journal |
| Command fails | Stop, report the exact error, do not continue |
| Tool unavailable | Report it, do not fabricate the result |
| Ambiguous instruction | Ask Joe, do not assume |
| Verification step fails | Stop, report, do not write success summary |
| Unknown test count | Run `cargo test` and quote output — never invent a number |

---

## ✅ DONE — CLI Flag Precedence Audit (D-068): SHIPPED — J-079, 5 atomic commits, 463 tests, five violations closed

**Status: SHIPPED — J-079.** The CLI Precedence Audit (`tasks/CLI_PRECEDENCE_AUDIT.md`, D-068) closed on 2026-05-17 in five atomic commits. The audit surfaced and fixed **five distinct violations**, not just the originally-named `--port` defect: one flag-threading bug (`xgen-node --port` was structurally orphaned from `run_node`) plus four parallel hardcoded subscriber-init blocks (`xgen-client --service`, `--service --ai-mode`, Tauri shell; `xgen-node` Tauri shell) silently bypassing `[logging].level` and falling back to a hardcoded `"debug"` literal. Helpers `xgen_common::precedence::resolve_setting<T>` (generic flag>env>config>default) and `resolve_log_level` (XGEN_LOG-aware specialisation) shipped in commit 1. The two previously-compliant subscriber-init paths (Node `run_node`, Client short-lived CLI) were also refactored onto the canonical helper in commit 3 for consistency and regression-locking. After J-079, **every log-level resolution in the codebase routes through one function** — the drift surface that produced these violations is architecturally eliminated, same shape as M5/D-067 eliminated drift in `xgen-client-lib::ops`. Test count 435 → **463** (+10 unit precedence + 5 URL-rewrite + 6 Node integration + 7 Client integration). Doc sync: Appendix F §F.0.6 updated; DECISIONS.md D-068 gained a closing note; both `main.rs` files' doc comments aligned with §F.0.6.

**Commits:** `3e2f311` helper + tests → `f77fe25` `--port` plumbing → `32028ad` four-site convergence → `1b62fed` integration tests → `19714ad` doc sync.

**Carry-overs (out of scope per D-068, flagged for future triage):**
- `xgen-client --quiet` doesn't gate the per-subcommand `Connecting to <node>...` line (no config equivalent → D-068 N/A).
- Short-lived Client CLI log file lands in `<exe_dir>/logs/` instead of `<data_dir>/logs/` (D-035 territory, not D-068).
- `xgen-node/src/desktop.rs::maybe_write_default_config` writes a non-schema `port = N` field (init flow which D-068 explicitly excludes).

---

## 🟡 ACTIVE — M6 Multiparty baseline pass with present `--batch`: PENDING

**Entry points: `tasks/MULTIPARTY_S1_tauri_rerun.md` + `tasks/MULTIPARTY_S2_to_S5_present_pass.md`.** Read those first; everything below is supporting context.

**Unblocked: the CLI Precedence Audit shipped in J-079** (see DONE block above). Flag precedence is now reliable across both binaries, so M6's "A" baseline metrics will reflect actual ops-layer behaviour rather than flag-vs-config drift.

M6 is **no code change** — pure measurement. Run the full Multiparty test suite (S1 through S5) twice through the present `--batch` shape (now unified through `xgen-client-lib::ops::*` after M5/J-078) and capture the metric set defined in `tasks/BATCH_FLAG_review.md` §"Baseline metrics protocol". This pass fills the **"A" baseline column** of every scenario's findings file. M7 ships `--aicontrol` v1; M8 re-runs S1-S5 against that and fills the **"B" improved column**. The A/B comparison is what the metric protocol exists to make rigorous.

**Why now:** M5 just shipped the unified `ops::*` handlers. Running multiparty against drift-prone duplicates would have made the "A" baseline meaningless for comparison with M7's "B" measurements. With M5 done, the baseline measures actual ops-layer behaviour and is directly comparable to the improved-version measurements that M7 will produce.

**Reading chain for Clair before starting:**

1. `tasks/MULTIPARTY_S1_tauri_rerun.md` — S1 runbook (Tauri rerun, picks up where J-067 left the CLI-only pass).
2. `tasks/MULTIPARTY_S2_to_S5_present_pass.md` — cross-scenario runbook for S2 through S5.
3. `tasks/BATCH_FLAG_review.md` §"Baseline metrics protocol" — the metric set every scenario captures into "A" column.
4. `JOURNAL.md` J-067 for the S1 background; J-078 for the unified-`ops::*` baseline that M6 measures against.

**Recording convention:** each `MULTIPARTY_S{N}_findings.md` gains a Metrics section with two columns (Present "A" / Improved "B"). M6 fills only "A"; "B" stays empty until M8. Friction observations append to the BATCH_FLAG_review friction log.

**Before starting, ask Joe** (Rule 6) to confirm: (a) scope is purely measurement against the current `--batch`, no protocol or `--batch` changes in M6; (b) the metric set as defined in `BATCH_FLAG_review.md` is what to capture (no silent additions); (c) S1 Tauri-rerun is run first to validate the Tauri shell against today's binary before the S2-S5 cross-scenario pass.

**Carry-overs for M6 to be aware of (not blocking):**
- The `cmd_create_space` optimistic-ack UX bug (J-077, J-078). Pre-existing; not blocking baseline measurement.
- Node-side log inspection beyond the smoke client's stderr will matter in M6 (silent-drop detection) where it was acceptable to skip in M5.
- Three out-of-scope items observed during the J-079 audit: `--quiet` per-subcommand non-gating, short-lived CLI log file path, and `maybe_write_default_config` writing a non-schema field. None affect M6 baseline metrics; flagged for future triage.

---

## ✅ DONE — M5 `ops::*` refactor: SHIPPED (435 tests, 12 atomic commits, 17/17 smoke PASS, F-003/F-004 architecturally closed)

**Status: SHIPPED — J-078.** Every user-facing `xgen-client` verb (13 total) now routes through a single `xgen-client-lib::ops::<verb>` function. All three dispatchers (`main.rs` CLI arm, `app::run_batch_file` CLI batch driver, `batch::dispatch_line` pipe arm) became thin shims calling the same `ops::*` function; each dispatcher owns its own output format. New `xgen-client/src/session.rs` (`SessionState`, `ClientIdentity`, idempotent `ensure_identity` / `ensure_connected` helpers — extension fields `bindings` / `spaces` present-but-empty for M7-shape stability). New `xgen-client/src/ops.rs` (one `pub async fn <verb>(ctx, args) -> Result<<Verb>Result>` per verb; pure data extraction; the canonical `load_or_default_state` helper). The drift surface that produced F-003/F-004 in J-067 is architecturally eliminated — there is now nowhere a second `get_dag_tips` (or any other implementation duplicate) could be introduced without being noticed. 17/17 smoke PASS against two live Nodes on `:8080`/`:8081` confirms the refactor preserves wire-correct behaviour end-to-end. Test count 429→435 (+6, all from new ops/session unit tests in commits 1-4). D-067 captures the structural outcome.

**Next session entry point: M6 — see the ACTIVE section above.** Four-step roadmap continues per D-066: M5 ✅ → M6 (multiparty baseline pass) → M7 (`--aicontrol` v1) → M8 (multiparty improved pass with A/B metrics filled in).

**Carry-overs:**
- ~~`xgen-node --port <port>` did not override `xgen-node_config.toml::listen` on first invocation during M5 smoke setup; second invocation of the same command succeeded. Flag-vs-config precedence bug in `xgen-node`.~~ **Scheduled as the CLI Precedence Audit (D-068, `tasks/CLI_PRECEDENCE_AUDIT.md`) — see ACTIVE block at the top of this file. The audit now blocks M6.**
- Tauri commands for the 13 protocol verbs still don't exist; current Tauri shell is lifecycle-indicator + pipe-server only. When verb-level Tauri commands eventually land they will naturally call `ops::*` — that's M5's prerequisite that's now met.
- `cmd_create_space` optimistic-ack UX bug (J-077, J-078). Future UX pass.

---

## ✅ DONE — M4 AI Client Binary: SHIPPED (429 tests, --ai-mode resident, mention→reply smoke green)

**Status: SHIPPED — J-077.** The AI Client is a *mode of `xgen-client`* (locked §1): `xgen-client --ai-mode --service` runs a long-running headless resident that consumes inbound events through an `AiBehavior` plugin and emits replies under existing pacing + mute constraints. New `xgen-client/src/ai_behavior.rs` (trait + reference `EchoPlugin` with locked deterministic reply format) and `xgen-client/src/ai_service.rs` (runtime loop, `AiPacingTracker` sibling of PacingManager for drop-on-throttle, plugin loader). `__HEALTH__` extended with `mode=ai operator_known=N/M`. Single-Node smoke confirmed: alice mentions bob (AI) → bob replies after `ai_pacing_ms`; back-to-back mention drops the second with literal warn line `dropping reply — pacing cap not yet elapsed (honest behaviour over polite behaviour) ai_pacing_ms=2000`. Spec §6.15 added to Ch6 (10 subsections); D-065 captures M4 architecture AND names the recurring "honest behaviour over polite behaviour" principle with its other instances across the protocol (operator resolution, Node event rejection, mute semantics, the create-space ack bug carry-over).

**Next session entry point: M5 — see the ACTIVE section above.** D-066 (2026-05-17, post-M4) made the decision: rather than "Joe's pick between multiparty redesign and Phase 3," the next milestone is the `ops::*` refactor that unblocks `--aicontrol` v1 and feeds clean handlers into the multiparty baseline pass. The four-step roadmap locked: M5 (`ops::*`) → M6 (multiparty baseline with present `--batch`) → M7 (`--aicontrol` v1) → M8 (multiparty improved pass with A/B metrics filled in). Phase 3 MLS operationalisation runs as an independent parallel workstream.

**Carry-overs (none blocking):**
- `cmd_create_space` doesn't await ack — Client prints "Space created" even on Node-side rejection. Pre-existing UX bug surfaced again during M4 smoke (bob's create-space attempt was rejected by M3's 3041 path but the optimistic stdout said success). Future Client UX pass, ideally adopting D-065's "wait for ack then report" honest pattern.
- Consolidated Node-side event-accept pipeline. Today's fragmentation (`accept_message` for message.*, dedicated arm for `membership.join`, catch-all `_ =>` for everything else) is fragile. Structural work for a future milestone; not blocking M5 candidates.
- `EventStore` HashMap iteration determinism. Doesn't affect M4 (the AI resident applies events in arrival order, not via sync-request replay).
- `prev_events` integrity for joins from non-members (M3 carry-over, timestamp-sort workaround in `cmd_ai_status` still in place).
- `docs/xgen_appendix_f_en.md` comprehensive example rewrite — Joe's gate of "M2 + M3" reached at M3 close-out; available whenever it surfaces as priority.
- AttachConsole hybrid-app polish (cosmetic Windows console flash).
- Cross-platform pipe server. D-043 still Windows-only.

---

## ✅ DONE — M3 AI Operator Role: SHIPPED (J-075)

411 tests. Operator as distinct role within Spaces (per-(AI, Space)). `SpaceMember.invited_by` + `SpaceState.ai_operator_delegations` + `resolve_operator` three-step fall-upward algorithm (stored delegation → AI's inviter → Space owner, transparently skips members who left). Both `state.space_create` and `state.dm_space_create` from an AI sender rejected with **3041 `ai_role_violation`** (wire name widened from `ai_flag_immutable`; code unchanged). Client CLI: `init --ai [--cap k=v]`, `register` honours `[ai]` config, new `ai delegate`/`ai revoke`/`ai status` subcommand group. Two-Node federation smoke (Rust integration) verifies decision #6's three cross-Node scenarios with strict assertions. Spec §3.6.10.6 rewritten; D-064 captures locked architecture.

---

## ✅ DONE — M2 Node Pipe Server: SHIPPED (J-074)

Six Node-side flags (`--ping`, `--health`, `--stop`, `--reload-config`, plus pipe-side `--batch`) became real implementations. New `xgen-node/src/pipe.rs` ports the Client's pipe-server skeleton to the Node with the four control commands plus a read-only `__BATCH__` subset (status / connections / peers / spaces / identity list / version / whoami). `__HEALTH__` returns the rich `HEALTHY pid=… state=RUNNING conns=… peers=… spaces=… uptime=…s` line. `__RELOAD_CONFIG__` returns honest `NOT_IMPLEMENTED` (real reload is a separate milestone). Pipe server spawns inside `app::run_node` so both `--service` and Tauri get it; `_pipe_shutdown_hold` at the `run_node` async-block scope (J-071 lesson). 391 tests held through M2.

---

## ✅ DONE — M1 Binary Consolidation: SHIPPED (J-073)

Six-commit chain (`e864715` → `c23c06a` → `1da3f1e` → `df877cb` → `4a9243b` → J-073 commit) collapsed four binaries to two: Tauri compiled into both per D-062, library-first dispatch per D-063, all 19 fundamental flags wired, Client `--batch` parallel implementations collapsed, Client `--service` headless resident operational, `cmd_init` instance-aware. Full matrix: 45/49 headless + 4/4 visual cells (N1, N2, C1, C2) confirmed by Joe. Full breakdown: J-068 → J-073.

---

## ✅ DONE — MULTIPARTY_S1 (local fan-out) — first of the five-file Multiparty suite

**Status: COMPLETE — M1 PASS, M2 PASS-with-caveat (J-067, 391 tests pass, 4 bugs found+fixed in-session)**

`docs/tests/MULTIPARTY_S1_multiclient_one_node.md` executed against running CLI binaries. Pre-flight reading of the Node revealed a **structural gap**: the Node had no local fan-out at all (each connection handler ingested events but never wrote anything back to other clients). Three additional bugs surfaced during M1/M2 execution. All four fixes are local and committed:

- **F-001** — local fan-out: new `xgen-node-lib::fanout` module with `OutboundMsg`/`ClientSenders`/`apply_fanout`/`collect_sync_history`/`topological_sort_events`. `handle_connection` rewritten as `tokio::select!`-loop between `conn.recv()` and per-connection outbound `mpsc::Receiver`. `transport.sync_request` handler added. Detect new joiners on `membership.join` and push the Space's prior DAG history to them.
- **F-002** — first post-auth message dispatched outside the loop so `sync_request` arriving first was dropped: deferred-first-message pattern routes the first inbound through the same handler as the loop body.
- **F-003** — `xgen-client/src/batch.rs::get_dag_tips` returned cross-Space tip leaks: added a Space filter inside the event-receive loop.
- **F-004** — duplicate `get_dag_tips` in `xgen-client/src/main.rs` (used by CLI `--batch`) wasn't covered by F-003 fix: same Space filter applied. ~~Both copies must be kept in sync until de-duplicated.~~ **De-duplicated in J-068 (M1 Phase 3-narrow):** single canonical implementation at `xgen-client/src/batch.rs:239`.

Plus `xgen-client init --passphrase` added (matches `xgen-node init --passphrase`) to unblock scripted instance setup.

**M1 P1 Smoke — PASS**: cell-for-cell pairing-table across alice/bob/carol on one Node, all 9 events visible in every expected log, content-leak `grep` returned zero unauthorised occurrences. **M2 P2 Stress — PASS with caveat**: 300 messages concurrently dispatched within 96 ms (under the 1 s requirement), 294/300 (98%) accepted, zero errors/timeouts/duplicates/orphans, 6 messages silently dropped between client WS write and Node receive — cause unclear, recommended for follow-up.

**Follow-up tasks (originally deferred — current status):**
1. ~~Unify the two `get_dag_tips` copies into one shared implementation.~~ **Done in J-068 (M1 Phase 3-narrow).**
2. Characterise the 6/300 P2 message loss (WS-frame tracing / tcpdump). **Still deferred — post-M1 / post-multiparty-redesign.**
3. Long-lived-client `--batch` mode (eliminates per-`send` connect-auth-sync overhead, enables direct observation of real-time fan-out). **Still deferred — post-multiparty-redesign.**
4. Five other improvements to `--batch` documented in `tasks/BATCH_FLAG_review.md`. **Still deferred — post-multiparty-redesign.**

**Multiparty next-session pointer superseded.** The earlier plan (S1 Tauri rerun → S2–S5 present pass → `--batch` improvements → A/B re-run) is paused. Per `tasks/BINARY_CONSOLIDATION_M1.md`, the full multiparty test suite will be **redesigned from scratch after M1/M2 land**, not re-run against the present shape. Current active work is M1 — see the "🟡 PARTIAL — M1 Binary Consolidation" section above for the resumed-work entry point.

---

## ✅ DONE — AI Identity, Pacing, and Temperature (D-059, D-060, D-061)

**Status: COMPLETE — 387 tests pass (J-065, 352 xgen-core + 12 xgen-node + 23 xgen-client-lib)**

`tasks/AI_USERS_AND_PACING_ph2.md` implemented across all three Parts:

- **Part A — AI Identity Extension (D-059, §3.6.10):** `is_ai` + `ai_capabilities` on `IdentityRecord` and `IdentityMessage::Register`; registration step 8 shape validation; `is_ai` immutability on `identity.update`; protocol-level capability enforcement on `state.dm_space_create`; error codes 3040/3041/3042; operator delegation EventTypes (`state.ai_operator_delegate`, `state.ai_operator_revoke`).
- **Part B — Per-Space Pacing Rules (D-060, §3.7.12):** `human_pacing_ms` / `ai_pacing_ms` on `SpaceState` with defaults 500 / 2000; `state.space_pacing` EventType (owner-only); client-side `PacingManager` in `xgen-client/src/pacing.rs` per Ch6 §6.14.2 with all four edge cases (clock skew, missing `is_ai`, missing rules, cap-of-zero); Tauri command `get_pacing_state`.
- **Part C — Temperature Property (D-061, §3.7.13):** `xgen.room_temperature` / `xgen.member_temperature` reserved `meta_atts` keys with `clamp_temperature`; `TemperatureThresholds` with validity check; `member_temperature_visibility` Space field + `state.space_temperature_visibility` EventType; `should_include_member_temperature` filter; `membership.mute` EventType with `auto_temperature` reason recognition; `NoOpTemperaturePlugin` in `xgen-node/src/plugins/temperature.rs`; `TemperatureUpdate` + bucket derivation in `xgen-client/src/temperature.rs`; Tauri `emit_temperature_update` helper.

Out of scope (deferred per the disposition): the math model that produces temperature values (plugin-owned); Phase 3 Node-side enforcement of pacing / `spontaneous_post`; Svelte UI components rendering the DOM contracts; the 13-step manual two-Node verification.

---

## ✅ DONE — Full integration stress test

**Status: COMPLETE — 6/6 PASS, 43/43 checks, 14.6 s (J-059, 300 tests)**

`stress-complete` subcommand implemented and executed against a 3-node topology (Node A: 9080, Node B: 9081, Node C: 9082 + Bootstrap). All 6 scenarios pass. Comm record at `docs/tests/stress_complete_events.json`.

Key milestones:
- ~~`stress-complete` subcommand implementation~~ — **DONE** (J-059)
- ~~Live run: 6/6 scenarios PASS~~ — **DONE** (J-059)

Two bugs found and fixed during live run: stack overflow in large async fn (32 MB thread dispatch), B↔C federation recv hang (replaced with explicit goodbye).

**Next priority order:**
1. ~~New appendix: all object/data structures~~ — **DONE** (`xgen_appendix_i_en.md`, 2026-05-15)
2. ~~AI Identity + Pacing + Temperature in code (D-059/D-060/D-061)~~ — **DONE** (J-065, 2026-05-15)
3. UI work — Phase 2 protocol, stress testing, data structures appendix, and the AI/pacing/temperature Rust surface are all complete; UI design can now resume

---

## ✅ DONE — Phase 2 integration testing

**Status: COMPLETE — 60/60 PASS (D-054–D-056, J-056–J-058, 300 tests)**

All Phase 2 protocol layers (11–19) are complete. Integration smoke test `smoke-ph2` passes all 60 steps against two live `xgen-node` processes over real TCP. One transport-layer bug discovered and fixed during the live run (D-056 — `recv()` routing collision between DAG Events and control messages on shared type-prefix strings).

Key milestones:
- ~~xgen-core crate split~~ — **DONE** (D-044, J-045)
- ~~Phase 2 protocol implementation — layers 11–19~~ — **DONE** (D-045–D-053, J-046–J-054)
- ~~Part A: CLI extensions (`--batch`, `smoke-ph2`)~~ — **DONE** (D-054, J-056)
- ~~Server-side gap closure~~ — **DONE** (D-055, J-057)
- ~~Part B: live run — all 60 steps PASS~~ — **DONE** (D-056, J-058)
- ~~Full integration stress test — 6/6 PASS~~ — **DONE** (J-059)

---

## ✅ DONE — Phase 2 Track 1 infrastructure (Sessions 14–18)

All of the following are COMPLETED. Do not re-implement.

| Instruction file | What it covers | Journal |
|---|---|---|
| `CLIENT_CORE_UI_ph2.md` | Tauri scaffold, 11 lifecycle states, state indicator wired, `--instance` flag | J-034, J-038–J-040 |
| `NODE_CORE_UI_ph2.md` | Node Tauri scaffold, systray, 7 lifecycle states + degraded stacking, `--service` flag | J-040 |
| `FIXES_core_ui_ph2.md` | Four UI bugs fixed (startup sequence, state event, systray tooltip, window show/hide) | J-041 |
| `FIXES_sec_01_ph2.md` | `--instance` label path traversal fix — `validate_instance_label` in both binaries | J-042 |
| `BATCH_FLAG_ph2.md` | `--batch` flag, named pipe IPC (D-043), `.xgb` format, 8 batch commands, clap dispatch | J-043–J-044 |
| `XGEN_CORE_SPLIT_ph2.md` | xgen-core crate split — extract protocol logic into GPL library (D-022, D-044) | J-045 |

**Batch command set** (available in `.xgb` files and pipe dispatch):
`whoami`, `status`, `register`, `create-space`, `create-room`, `invite`, `join`, `send`
See `docs/xgen_appendix_f_en.md` §F.8 for full reference.

**Key decisions added this phase:** D-042 (Tauri event emission), D-043 (named pipe naming convention `\\.\pipe\xgen-{binary}-{label}`)

---

## ✅ PHASE 1 IS COMPLETE — DO NOT RE-IMPLEMENT

All Phase 1 deliverables are done:

1. **Binary wiring** — both `xgen-node` and `xgen-client` are real runnable processes.
2. **Smoke test** — `xgen-client smoke-test --node-a ws://127.0.0.1:8080/xgen --node-b ws://127.0.0.1:8081/xgen` runs all 17 steps against real Node processes over real TCP. Verified 2026-04-29. Tag `v0.10.3`.
3. **Documentation gates** — complete. All lifecycle, Console, deployment model, and UI architecture decisions are documented. Phase 2 implementation may begin.
4. **Stress test** — complete. `docs/tests/STRESSTEST_ph1_findings.md` status: COMPLETED. All findings resolved, verification run passed (commit `8c9402b`).

---

## ✅ DONE — Priority 0: Global Event tracing interface

`docs/tests/LOGGING_debug_ph2.md` implemented (J-027). Definitive implementation instructions (session header/footer, LOCAL actions, EventDirection rename) are in `LOGGING_implementation.md` — implement this before any Phase 2 protocol features. `event_trace` module lives in `xgen-common/src/` (Fix 17 applied, J-029). `Event` and `EventType` also moved to `xgen-common/src/wire.rs` and re-exported from `xgen-node`. Role gate active. Content field never logged. 173/173 tests passing. Smoke test with debug logging confirmed full Event pairing across client and both Nodes (J-029).

---

## 🔧 DONE — Phase 1 debug logging

`docs/tests/LOGGING_debug_ph1.md` is complete and verified (J-025). Datetime-stamped log files, config level switch, subscriber init, operational log calls — all implemented in both binaries. Do not re-implement.

The audit log (`docs/tests/LOGGING_audit_ph2.md`) is **deferred** — implement alongside Tier 2+ Auth Module work only.

---

## ✅ DONE — Documentation fixes (FIXES_ph1.md)

All 17 fixes from `FIXES_ph1.md` have been applied (Fix 14 deferred by project owner). Fix 16 (Node space state replay on restart) and Fix 17 (event_trace relocation) are complete in Rust source. Documentation fixes 1–15 applied to Ch3/Ch4. See `FIXES_ph1.md` for the full record.

---

## ⏸ POSTPONED — UI Phase 2 prep (run 1.5)

UI design work for Phase 2 Track 1 is paused at the element-modelling step (J-033, 2026-05-08).

**Deliverables in `ui/run_1.5/`:**

- `skeleton_audit.md` — initial audit of chat mockups (`ui/backup/fixed_samples/`). Detailed div/span vs semantic-tag conversion conventions. Top-of-file note: framed against the wrong reference; see `comparative_analysis.md` for the corrected take.
- `comparative_analysis.md` — corrected analysis. Miss Design's skeleton (`ui/backup/skeleton/`) already implements ~95% of recommended semantic structure. The gap between her skeleton and the chat mockups lives in CSS reset rigour, visual coding density, and Run 2 evolutions (D-038, D-039, Run 2 Change 1) — not in HTML structure. The current `ui/xgen-mockup-*.html` files are a partial merge attempt that did not fully capture the chat mockups' visual quality.

**Visual merge plan postponed.** A 10-milestone roadmap in `comparative_analysis.md` covers merging the chat mockups' visual treatment onto Miss Design's semantic structure under Ch2 fixed conditions (lifecycle state coverage for all 7 Node + 11 Client states, open-enum fallback rules, slot system intact, Layer 4 boundary, accessibility `:focus-visible`, replaceable skins). Architecture: `tokens.css` always-loaded variables only; `skin-{name}.css` self-contained with own reset; reset coupled to skin so a missing skin degrades to raw HTML. Theme loader behaviour locked in `D-041` (default `skin-dark.css`; fallback chain on skin failure: requested → default → raw HTML).

**Gating step before resume:** confirm and expand the absent-element list in `ui/docs/xgen-ui-design-brainstorm.md` Point 3 (event types in the message stream — member-originated, self mirrored, system/protocol, module-injected) and Point 2 (avatar as first-class object — DOM element with hover context menu). The list is currently marked "to be confirmed" and must be reconciled with Ch3's authoritative event taxonomy. A Run 3 design briefing is drafted from the consolidated list before any visual merge work begins.

Do not start the visual merge or write any skin CSS until the element list is confirmed and the Run 3 briefing exists.

Recorded in `JOURNAL.md` J-033 and `DECISIONS.md` D-041.

**This entire track is postponed until Phase 2 protocol implementation is complete.**

---



XGen Protocol is an open, federated, identity-verified communication protocol. Think of what Discord would have been if built as open infrastructure. The core thesis: no single entity should own the communication layer.

This is not a product — it is protocol infrastructure. Phase 1 is a minimal working implementation. Phase 2 is the full protocol. Phase 3+ is everything else.

**The spec is authoritative.** When this file and the spec conflict, the spec wins. When the spec is ambiguous, flag it — do not resolve it silently.

---

## Current State — Where We Are

**M5 SHIPPED (J-078). M6 PENDING. 435 tests passing. Phase 2 protocol complete; Phase 3 areas open. Roadmap: M5 ✅ → M6 → M7 → M8.**

Current project status as of 2026-05-17:

- **Phase 1**: complete (J-029, tag `v0.10.3`, 17-step smoke test passing over real TCP). See historical snapshot below.
- **Phase 2 protocol**: complete (J-058, `smoke-ph2` 60/60 PASS, layers 11–19 all shipped). See "Phase 2 — Status" section below.
- **Phase 2 Track 1 UI**: partially complete (Tauri scaffold, lifecycle states, named pipe IPC, `--batch` all done at J-040–J-045; the deeper visual-merge work is POSTPONED, see the ⏸ POSTPONED block above).
- **Post-Phase-2 protocol work shipped:** AI Identity + Pacing + Temperature (J-065, D-059/D-060/D-061), full integration stress test (J-059, 6/6 PASS).
- **M1–M4 milestone series shipped**: binary consolidation (M1, J-068–J-073), Node pipe server (M2, J-074), AI operator role (M3, J-075), AI Client resident mode (M4, J-077).
- **M5 shipped**: `ops::*` refactor — J-078, D-067, 12 atomic commits, 17/17 smoke PASS.
- **M6 pending**: multiparty baseline pass with present `--batch` — see ACTIVE block at the top of this file.
- **Phase 3 areas**: state migration depth, federation depth, MLS operationalisation. Specced but unimplemented. D3 (MLS) runs as a parallel workstream alongside M5→M8 per D-066.

### Historical snapshot — Phase 1 completion (April 2026, tag `v0.10.3`, 173 tests)

This table records how Phase 1 landed and is preserved as a historical reference. Test counts and tags are frozen as of April 2026; current counts and milestones are above.

| Layer | Content | Tests | Tag |
|---|---|---|---|
| 1 | Crypto (Ed25519, SHA-256, base64url, ChaCha20+Argon2id) | 25 | v0.1.1 |
| 2 | Wire format (Event, EventType, framing, validation steps 1–7) | 53 | v0.2.2 |
| 3 | DAG event store (append-only, tips, pending buffer) | 79 | v0.3.2 |
| 4 | WebSocket transport (challenge-response auth, keepalive) | 88 | v0.4.2 |
| 5 | Node identity and announcement | 100 | v0.5.2 |
| 6 | Federation handshake (state machine, registry) | 121 | v0.6.2 |
| 7 | Identity registration (8-step pipeline, registry) | 142 | v0.7.2 |
| 8 | Space and Room protocol (state machine, roles, permissions) | 160 | v0.8.2 |
| 9 | Message exchange (validation steps 8–13, accept_event) | 171 | v0.9.3 |
| 10 | Smoke test — spec 3.7.11, 17-step end-to-end | 173 | v0.10.1 |
| CLI | init, status, connections, spaces, peers, identity list, whoami (D-025–D-028) | 173 | v0.10.2 |
| Binaries | xgen-node WebSocket server + xgen-client network commands + 17-step smoke test over real TCP | 173 | v0.10.3 |

Phase 1 definition of done met: 17-step smoke test passes. Tag `v0.10.1`.  
Phase 1 CLI completeness: both binaries have full clap CLI, state file types, and all observability commands. Tag `v0.10.2`.  
Phase 1 binary wiring verified: smoke test passes against two real running Node processes over TCP. Tag `v0.10.3`.

---

## Architecture Rules — Non-Negotiable

**1. Library-first.** All protocol logic lives in `lib.rs`. `main.rs` is a thin CLI shell only — argument parsing, startup, shutdown. No business logic in `main.rs`. This is what makes Phase 2 Tauri integration possible without rewriting.

**2. Spec is authoritative.** `docs/xgen_ch3_specification.md` is the source of truth. `IMPLEMENTATION_GUIDE_ph1.md` is the implementation guide. When they conflict, the spec wins.

**3. Verify after every write.** Read back every file after writing it. Silent write failures have caused reconstruction work in past sessions.

**4. DECISIONS.md before advancing.** Every implementation decision beyond spec prescription must be recorded in `DECISIONS.md` before moving to the next layer. Format: title, date, layer, spec reference, decision narrative.

**5. Tests before advancing.** Run `cargo test` and confirm all tests pass before moving to the next layer. Do not skip.

---

## File Placement Rules (D-025 — Updated)

All runtime files are prefixed with the binary name. **`xgen-node_*` for all Node files, `xgen-client_*` for all client files.**

**Tier 1 — System files: mandatory co-location with binary, not configurable**

| File | Binary | Description |
|---|---|---|
| `xgen-node_config.toml` | xgen-node | Node configuration (TOML) |
| `xgen-node_state.json` | xgen-node | Live status snapshot, written every 5s (D-026) |
| `xgen-node_identities.db` | xgen-node | Identity registry (SQLite) |
| `xgen-node_federation.db` | xgen-node | Federation registry (SQLite) |
| `xgen-client_config.toml` | xgen-client | Client configuration (TOML) |
| `xgen-client_state.json` | xgen-client | Identity, known nodes, joined spaces |

**Tier 2 — User-configurable files: default to binary folder, redirectable via config**

| File | Config field | Description |
|---|---|---|
| `xgen-node_keypair.enc` | `keypair_path` | Ed25519 private key — may redirect to HSM or secure share |
| `xgen-client_keypair.enc` | `keypair_path` | Ed25519 private key — may redirect to OS keystore (Phase 2) |
| Log output | `log_path` | May route to system log aggregator |

No file moves silently. Every Tier 2 redirect is explicit in config.

---

## meta_atts Key Namespace Rules (Spec 3.1.3)

`meta_atts` keys use dot-separated namespaces:

- `xgen.*` — **reserved** for protocol use only. Examples: `xgen.client`, `xgen.thread_id`, `xgen.tags`
- Third-party keys MUST use reverse-domain prefix. Examples: `com.example.priority`, `org.myapp.color`
- All lowercase, snake_case segments, dots as separators, no hyphens
- Max key length: 128 characters
- Values are strings. Structured values are JSON-encoded strings, not nested objects.

---

## Error Code Convention

Error codes are plain integers on the wire and in exit codes (e.g. `4002`). For human-readable display in logs, UI, and documentation, codes are shown with an `E` prefix and zero-padded to 6 digits (e.g. `E004002`). The `E` prefix is display-only — never transmitted, never used programmatically. `E004002` and `4002` are the same error.

Domain ranges: 1000–1999 transport, 2000–2999 federation, 3000–3999 identity, 4000–4999 state resolution, 5000–5999 E2E encryption, 6000–6999 migration, 7000–7999 bootstrap, 8000–8999 reputation, 9000–9999 DM promotion. Future domains extend naturally: domain 10 = 10000–10999, etc.

---

## Transport Pluggability (Spec 3.3.1)

WebSocket over TLS is the mandatory production transport. The protocol also explicitly permits Tor hidden services, I2P, and pluggable transport proxies as alternative stream transports — no protocol changes required. Phase 1 uses `ws://` localhost only. Production uses `wss://`. DPI resistance is a Phase 3 area; no Phase 1 impact.

---

## Key Cryptographic Decisions

- **Keypair encryption at rest:** ChaCha20-Poly1305 + Argon2id KDF. Phase 1 local node uses empty passphrase (file still encrypted for integrity).
- **Event ID derivation:** SHA-256 hash of canonical JSON → `xgen://hash/sha256:<hex>`
- **Signature format:** `ed25519:<base64url-pubkey>:<base64url-sig>` — covers canonical form only, not wire bytes
- **Canonical form:** fixed field order, no whitespace, object keys sorted lexicographically, `event_id` and `signature` excluded
- **DAG root types:** `state.space_create`, `state.room_create`, `state.dm_space_create` require empty `prev_events`. All others require at least one.
- **Cycle detection:** reduces to self-reference check only at insertion time (append-only store invariant)
- **prev_events fanin limit:** 10 (Phase 1)
- **Node announcement TTL:** 90 days
- **Session ID derivation:** `hash_uri(sort([node_a_id, node_b_id]) + timestamp)` — sorted so both sides derive same value

---

## Versioning Scheme

`[state].[layer].[session]` — three components, stored in `Cargo.toml`.

- `state`: 0 while building Phases 1 and 2; 1 when Phase 1 and Phase 2 complete and stable
- `layer`: implementation layer number (1–10)
- `session`: work session in which that layer was completed

Tags are monotonically increasing: `v0.1.1` → `v0.2.2` → … → `v0.10.x`

---

## Phase 2 — Status

Phase 2 shipped in two tracks. Both reached their Phase-2 deliverables; deeper work in each track has been scheduled as separate milestones.

### Track 1 — UI infrastructure (Phase-2 deliverables shipped; visual merge POSTPONED)

**Status: Phase-2 infrastructure SHIPPED (Sessions 14–18, J-034–J-045); visual-merge POSTPONED (see ⏸ block above).**

The Tauri scaffolding, lifecycle state machines, named pipe IPC, `--instance` segregation, `--batch` flag, and `xgen-core` crate split all landed during Sessions 14–18. Both binaries open windows with custom chrome; lifecycle states from Appendix E are wired; Node systray works with state-coloured icons; first-run SETUP is functional; `--service` headless mode works on both binaries.

The **visual merge of design Claude's chat mockups onto Miss Design's semantic skeleton** is POSTPONED at the element-modelling step (J-033). The gating condition (confirmed absent-element list in `ui/docs/xgen-ui-design-brainstorm.md` and a Run 3 design briefing) has not been met; see the `⏸ POSTPONED — UI Phase 2 prep (run 1.5)` section earlier in this file for the full status.

Library-first rule still applies for any future UI work: lifecycle state machine logic stays in `lib.rs`, Tauri `main.rs` stays thin, Svelte calls Tauri commands only.

### Track 2 — Protocol (Phase-2 deliverables shipped; Phase 3 areas open)

**Status: Phase-2 protocol SHIPPED (D-045–D-053, J-046–J-058, `smoke-ph2` 60/60 PASS, `stress-complete` 6/6 PASS).**

All Phase-2 protocol layers (11–19) shipped through Sessions 18 onward. `smoke-ph2` runs 60/60 PASS against two live Node processes over real TCP (J-058). The full integration stress test runs 6/6 PASS across a 3-node topology with Bootstrap discovery (J-059). The xgen-core crate split (D-022, D-044) landed at J-045 and the dual-licence boundary (BSL 1.1 thin shells, GPL-2.0-or-later xgen-core library) is in place.

Post-Phase-2 protocol work also shipped: AI Identity + per-Space pacing + temperature property (D-059/D-060/D-061, J-065); M1–M4 series (binary consolidation, Node pipe server, AI operator role, AI Client resident mode).

**Phase 3 areas — specced but unimplemented:**

| Area | Status | Reference |
|---|---|---|
| State migration depth | Wire shape specced (3.12, Layer 14); deep testing pending | Future milestone (D1, folded into M8) |
| Federation depth | Phase-1 federation works; N-Node topologies, defederation flow, reputation merge pending | Future milestone (D2, folded into M8) |
| MLS operationalisation | Wire shape specced (3.10, Appendix I Part X.6); openmls integration pending | Future milestone (D3, parallel workstream alongside M5→M8) |
| `self` account | Local-only synthetic Identity, accessible from any client | D-021 — deferred |
| Registry file encryption | Identity and federation registries at rest | Deferred |
| Slovak translation pass | Single pass after full document completion | Deferred |
| DPI resistance | Investigation only | D-023 — Phase 3 |

**Roadmap locked (D-066, amended 2026-05-17):** M5 ✅ (J-078) → **CLI Precedence Audit (D-068)** → M6 (multiparty baseline pass with present `--batch`) → M7 (`--aicontrol` v1) → M8 (multiparty improved pass with A/B metrics, plus D1 migration scenario + D2 federation depth folded in). D3 (MLS) runs as an independent parallel workstream. The CLI audit is a hard prerequisite for M6, inserted between M5 and M6 because the testing model from M6 onwards depends on reliable flag overrides. See the ACTIVE block at the top of this file for the current step.

---

## Repository Layout

```
docs/
  xgen_ch0_content.md             # table of contents
  xgen_ch1_philosophy.md          # philosophy, motivation, Human and Agent Operation
  xgen_ch2_architecture.md        # architecture, primitives, deployment model (D-037)
  xgen_ch3_specification.md       # AUTHORITATIVE SPEC (§3.1–3.8 Phase 1 + §3.9–3.16 Phase 2 — complete)
  xgen_ch4_implementation.md      # Phase 1 complete; Phase 2 scope defined
  xgen_ch5_protocol.md            # stub
  xgen_ch6_client_design.md       # UI architecture, Console §6.9–6.11, IPC protocol §6.9
  xgen_appendix_a_en.md           # Why XGen must be its own protocol
  xgen_appendix_b_en.md           # Kyberia lineage and acknowledgment
  xgen_appendix_c_en.md           # Mermaid class diagrams
  xgen_appendix_d_en.md           # Node data, privacy, and storage (GDPR reference)
  xgen_appendix_e_en.md           # APPLICATION LIFECYCLE STATES — state machine spec for BOTH binaries
  xgen_appendix_f_en.md           # CLI reference and usage examples
  xgen_appendix_g_en.md           # Log line convention
ui/
  client.html                     # Client skeleton (design Claude)
  node.html                       # Node admin skeleton (design Claude)
  console.html                    # Console overlay skeleton (design Claude)
  docs/
    xgan-ui-overview.md           # Design Claude's overview and open questions
    xgan-ui-debug-console-questions.md  # Console Q&A
    xgen-ui-chat-briefing.md      # ALL design decisions answered — read before UI work
IMPLEMENTATION_GUIDE_ph1.md       # Phase 1 layer-by-layer guide — COMPLETED
IMPLEMENTATION_GUIDE_ph2.md       # Phase 2 layer-by-layer guide (layers 11–19) — ACTIVE
DECISIONS.md                      # Implementation decision log (D-000 through D-066)
JOURNAL.md                        # Contemporaneous development journal (IP record)
CLAUDE.md                         # This file
LICENSE                           # BSL 1.1
```

Source crates:
```
xgen-common/    # shared types (no runtime, no I/O) — BSL 1.1
xgen-core/      # all protocol logic — GPL-2.0-or-later (created in Phase 2 crate split)
xgen-node/      # thin Node shell — main.rs + lifecycle, depends on xgen-core — BSL 1.1
xgen-client/    # thin client shell — main.rs + commands, depends on xgen-core — BSL 1.1
```

Build target directory is kept outside the project folder to avoid file locking:
```
C:/cargo-targets/XGenProtocol
```

---

## Document Header Convention

### Core pattern

```
# Title
> **Status**: {}  
> Version: {}  
> Date: {MMM YYYY}  
> **Last updated**: YYYY-MM-DD  
> Language: {}  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  
```

### Specification

- Every `> ...` line requires **two trailing spaces before EOL** (mandatory for correct line rendering)
- `{MMM YYYY}` = month-name + year, e.g. `May 2026`
- **This header MUST be updated on every file edit**

Status values:
- `ACTIVE` — current, act on it
- `PENDING` — written, not yet the current task
- `COMPLETED` — done, do not re-execute
- `DEPRECATED` — no longer valid / replaced — replacement named if applicable
- `ARCHIVED` — frozen historical record, do not modify

**When looking for the next task**, scan `tasks/` and `docs/tests/` file headers. The next instruction file to run is the first one with `PENDING` status that is not explicitly deferred.

**Note on folder convention:** New instruction files for Code Claude are written to `tasks/` at the project root (not under `docs/`). The `docs/tests/` folder holds the legacy instruction files written before this convention; it stays in place until a future cleanup migrates everything to `tasks/`. Both folders are scanned for `PENDING` files.

---

## License Header

Every source file MUST carry this exact header:

```rust
// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.
```

Not PolyForm. Not MIT. Not any other license. BSL 1.1 exactly as above.

---

## Build Commands

```sh
cargo build                              # debug build
cargo build --release                    # release build
cargo test                               # run all tests
cargo test smoke                         # run smoke test only
cargo test --package xgen-common         # test one crate
```

Build output goes to `C:/cargo-targets/XGenProtocol` (set via `CARGO_TARGET_DIR` in `build.sh`). Binaries are copied to `bin/` in the project folder by `build.sh`.

---

*Read `DECISIONS.md` (current range D-000 through D-066) before making any decision that isn't explicitly covered by the spec. If you're unsure whether something needs a DECISIONS.md entry, it does.*
