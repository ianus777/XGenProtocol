# Client `--batch` Audit — M6 Pass 1 of Phase 0
> **Status**: COMPLETED  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-18  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## What this file is

A snapshot audit of the `xgen-client --batch` surface as of 2026-05-18, written as Pass 1 of M6 Phase 0. It establishes the factual baseline against which the M6 design discussion (Pass 2) and final design doc (Pass 3) are built.

The audit answers three questions:

1. Which CLI subcommands does `xgen-client` define today?
2. Which of those subcommands are dispatched through `--batch` (the pipe surface)?
3. Where are the gaps — commands documented in Appendix F but absent from code, or implemented in code but missing from `--batch`?

The audit is informational. It identifies the Client-side work that may need to land as M6 Phase 1, but does not prescribe it. Lock decisions happen in Pass 3.

---

## Method

Read three source files plus one spec chapter, cross-reference what each declares:

| File | Read | Authority |
|---|---|---|
| `xgen-client/src/app.rs` | `ClientCommand` enum, `cmd_*` shims, `run_batch_file` dispatch arms | Code authority — the CLI surface as compiled today |
| `xgen-client/src/batch.rs` | `dispatch_line` match arms — pipe-side verb routing | Code authority — the pipe surface as compiled today |
| `xgen-client/src/ops.rs` | Exported function names — shared command implementation layer (M5/D-067) | Code authority — the canonical handler set |
| `docs/xgen_appendix_f_en.md` §F.3, §F.8.3 | Documented Client subcommand list and `.xgb` command set | Spec authority — what the CLI surface *should* be |

The four sources should agree. Where they disagree, that's a gap.

---

## Findings

### Finding 1 — Client `--batch` is feature-complete for what exists in code

Every non-test variant of the `ClientCommand` enum is wired through `batch::dispatch_line`, and every wired arm routes through `ops::*` (post-M5/D-067). The four long-running test commands are explicitly rejected from `--batch` with a clear error rather than silently misbehaving.

**Coverage table:**

| `ClientCommand` variant | In CLI (`run_batch_file`) | In pipe (`batch::dispatch_line`) | Routes through `ops::*` |
|---|---|---|---|
| `Init` | ✅ | ✅ | N/A (init is local-only — calls `app::cmd_init` directly) |
| `Whoami` | ✅ | ✅ | ✅ `ops::whoami` |
| `Status` | ✅ | ✅ | ✅ `ops::status` |
| `Spaces` | ✅ | ✅ | ✅ `ops::spaces` |
| `Version` | ✅ | ✅ | N/A (local-only — calls `app::cmd_version`) |
| `Register` | ✅ | ✅ | ✅ `ops::register` |
| `CreateSpace` | ✅ | ✅ | ✅ `ops::create_space` |
| `CreateRoom` | ✅ | ✅ | ✅ `ops::create_room` |
| `Invite` | ✅ | ✅ | ✅ `ops::invite` |
| `Join` | ✅ | ✅ | ✅ `ops::join` |
| `Send` | ✅ | ✅ | ✅ `ops::send` |
| `History` | ✅ | ✅ | ✅ `ops::history` |
| `Ai Delegate` | ✅ | ✅ | ✅ `ops::ai_delegate` |
| `Ai Revoke` | ✅ | ✅ | ✅ `ops::ai_revoke` |
| `Ai Status` | ✅ | ✅ | ✅ `ops::ai_status` |
| `SmokeTest` | ✅ (direct in-process) | ❌ rejected (long-running) | N/A |
| `StressTest` | ✅ (direct in-process) | ❌ rejected (long-running) | N/A |
| `SmokePh2` | ❌ rejected (cannot be in-batched) | ❌ rejected (long-running) | N/A |
| `StressComplete` | ❌ rejected (cannot be in-batched) | ❌ rejected (long-running) | N/A |

**Total:** 15 of 15 user-facing verbs wired through `--batch`. The four test commands are appropriately gated.

**Architectural implication.** The drift surface that produced F-003/F-004 in J-067 is architecturally closed (M5/D-067) — there is now exactly one implementation of each verb, in `ops::*`. Both the CLI in-process `--batch` path (`run_batch_file`) and the pipe dispatcher (`batch::dispatch_line`) call into `ops::*`. No parallel implementations to drift between.

### Finding 2 — Three Appendix-F-documented commands are spec-only

Appendix F §F.3 lists three Client subcommands that **do not exist in the `ClientCommand` enum**:

| Documented command | Documented purpose | Current state |
|---|---|---|
| `rooms --space <id>` | List Rooms in a Space (read-only, no network) | **Missing from code entirely.** No `Rooms` variant in `ClientCommand`. |
| `members --space <id>` | List members of a Space (read-only, no network) | **Missing from code entirely.** No `Members` variant in `ClientCommand`. |
| `federate --space <id> --peer <endpoint>` | Initiate federation for a Space with a peer Node (network, write) | **Missing from code entirely.** No `Federate` variant in `ClientCommand`. |

These are gaps in the *Client CLI surface itself*, not in `--batch` coverage. The `--batch` pipe cannot dispatch commands that don't exist as CLI subcommands.

**Origin trace:** Appendix F §F.3 §F.10 Session 2 documents that the row for `federate` was added in the M2/M3/M4 documentation sweep on 2026-05-17. `rooms` and `members` predate that — they appear in the original §F.0.5 spaces-collision discussion as part of the "Client-only subcommand list" but were never implemented in code. The spec describes them as zero-network reads from the local state file.

**Impact assessment for M6:**

- **`rooms` and `members`** are trivial — they read `xgen-client_state.json` and project subsets, same shape as `whoami` / `status` / `spaces`. Each would be a small atomic commit adding the variant + `ops::rooms` / `ops::members` + dispatcher arms. Estimated: 2 commits total.
- **`federate`** is more substantive. It needs a real network exchange against the home Node, plus likely a complementary Node-side admin verb for accepting incoming federation requests. It is genuinely *deferred*, not just unimplemented — Node-side federation management is in M6 Phase 6, and the Client side is naturally co-designed with it.

### Finding 3 — Node `--batch` is read-only with 7 verbs

The Node-side `pipe::dispatch_line` is intentionally restricted to the safe read-only set per the original M2 disposition:

```
status, connections, peers, spaces, version, whoami, identity list
```

Everything else returns the explicit error:

```
command not supported in pipe-batch mode (allowed: status, connections, peers, spaces, identity list, version, whoami): <line>
```

**Node `--batch` is exactly the surface M6 expands.** The verb set listed under `docs/xgen_aicontrol_implementation.md` §7.2–§7.8 (federation management, Auth Module management, Bootstrap configuration, Space/Room operator actions, identity registry administration, logging/audit administration, plugin management) is what M6 ships through this pipe.

### Finding 4 — Naming convention asymmetry between binaries

A minor observation worth flagging for the Pass 2 verb-naming discussion:

- **Client side:** uses two-word subcommand groups with a space (`ai delegate`, `ai revoke`, `ai status`). Clap's `Subcommand` derive supports this via the `Ai(AiArgs)` → `AiCommand::Delegate(...)` nesting pattern.
- **Node side:** has no multi-word subcommands yet. The `identity list` allowance in `pipe::dispatch_line` is matched as two tokens (`[a, b] if a == "identity" && b == "list"`), not as a single hyphenated token.

The current Client convention is two-token (`ai delegate`). The Node side has the same shape latent (`identity list`). The verb-naming discussion in Pass 2 should pick one of:

- **Continue two-token.** `federation accept`, `auth-module register`, `bootstrap configure`. Matches Client convention; reads naturally.
- **Hyphenated single-token.** `federation-accept`, `auth-module-register`, `bootstrap-configure`. Matches the `docs/xgen_aicontrol_implementation.md` §7 sketches.
- **Dotted.** `federation.accept`, `auth_module.register`. Matches Event-type naming (`state.federation_add`), but doesn't match any existing CLI subcommand convention.

This is a discussion item, not an audit finding. Just flagging that the choice affects ~30+ verb names so picking early is leverage.

---

## Recommendations for M6 Phase 1

These are recommendations, not decisions. Pass 3 locks them.

### R1 — Implement `rooms` and `members` as M6 Phase 1 (small, fast, isolates the easy work)

Two atomic commits, each following the M5 per-verb pattern:

```
commit 1: add ClientCommand::Rooms + RoomsArgs + ops::rooms + cmd_rooms shim + dispatch arm
commit 2: add ClientCommand::Members + MembersArgs + ops::members + cmd_members shim + dispatch arm
```

Both are zero-network reads from `xgen-client_state.json`. The result data is already in `ClientState.spaces[i].rooms[]` and `ClientState.spaces[i].members[]` (or the equivalent — schema check during implementation).

Why include these in M6 rather than defer:
- Closing them removes a spec-vs-code drift surface.
- Both are trivially safe — no new network surface, no protocol changes.
- Documenting them as "implemented" in Appendix F is cleaner than carrying "documented but absent" indefinitely.

### R2 — Defer `federate` to M6 Phase 6 (federation management)

The Client-side `federate` command is the user-facing front of Node-side federation management. It belongs with the Phase 6 federation verbs, not with the Phase 1 trivia. Implementing it earlier would mean designing the Client/Node interaction twice — once now for a stub, once again when Phase 6 lands.

Mark Appendix F's `federate` row as "Phase 6" until then, or remove it from the documented surface and re-add when Phase 6 lands. Pass 3 decides which.

### R3 — Treat M6 Phase 1 as optional, not mandatory

If Pass 3 decides the `rooms` / `members` gaps are small enough to ignore for now, Phase 1 collapses to zero commits and M6 starts directly at Phase 2 (`admin_ops::*` scaffolding). The audit's job is to surface the gap; the decision is yours.

---

## Out of scope for this audit

- Node `--batch` write verbs. Enumerated in Pass 2.
- Per-verb argument schemas. Locked in Pass 3.
- Privilege model, audit trail shape, verb naming convention, failure semantics, live-reload bucket. Discussed in Pass 2, locked in Pass 3.
- Any code changes. M6 Phase 0 is documentation-only.
- `--aicontrol` JSONL surface. Out of scope for M6 entirely (lives in M7+).
- Live config reload. Out of scope for M6 entirely (lives in M7 as a standalone milestone per Pass 0 decision 2026-05-18).

---

## Status disposition

This file is `COMPLETED` because:

1. The audit is a one-shot snapshot, not an ongoing task.
2. The factual claims are sourced from code reads on 2026-05-18 — re-running the audit later would produce a different document (post-Phase-1, post-Phase-2, etc.).
3. The recommendations under §"Recommendations for M6 Phase 1" become inputs to the Pass 3 design doc, not actions to track here.

If M6 Phase 1 ships any of the R1/R2/R3 recommendations, the JOURNAL.md entry for that work cross-references this audit. No re-opening of this file.

---

*End of audit.*
