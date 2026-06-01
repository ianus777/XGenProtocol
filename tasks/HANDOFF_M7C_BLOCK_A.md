# M7-completion Block A — Clair handoff (start at A1)
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-01  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. You are here

M7-completion cluster — docs all locked (audit J-213, design J-214, runbook J-215; all pushed).
**Block A Commit A1 is the first code beat.** This is a code session: A1 touches Rust. The runbook
is the spec; this note is the pointer + the boundaries.

## 2. Reading order (Rule 0)

CLAUDE PLAY (the J-215 block) → JOURNAL J-215 → this handoff → `tasks/M7C_COMPLETION_IMPL.md`
§1–§3 (the build plan; §3 is Block A) → `tasks/M7C_COMPLETION_DESIGN.md` §1 (M7C-D3/D4) for the why.
Do not re-derive — the locks are set; build to them.

## 3. A1 — `ops::members` (read / lift), in one paragraph

New client read verb. Reuse the `ai_status` history-drain (`xgen-client/src/ops.rs:1031`) to replay
the Space into a `SpaceState`, then return `state.members` (id → role, `invited_by`, `joined_at`) in
a new Result struct. **Covers DM Spaces** — both `from_space_create` and `from_dm_space_create` seed
`state.members`; do **not** inherit `ai_status`'s DM bail (that bail is operator-resolution-specific,
not a membership-read limit). Route through one `ops::*` fn (D-067) reached by all dispatchers:
`ops.rs` (fn + Result struct) · `app.rs` (clap subcommand + `cmd_*` CLI shim) · `batch.rs` (batch
arm) · `aicontrol.rs` (`dispatch_resolved` arm). **Confirm the exact touch-set against the live tree
at pickup (D-078)** — the M7 v1 verbs are the working template. Tests: regular-Space membership read,
DM-Space membership read, unknown/empty Space error.

## 4. Boundaries — STOP lines

- **A1 + A2 are checkpoint-free. STOP at CP-1 (Joe-lock) before A3** — do not start the node
  DM-init arm (`from_dm_space_create_node`) without the checkpoint. Surface, wait for Joe.
- **`--batch` / `pipe.rs` untouched** (D-066); the `.aicontrol` arm is a sister, not a fork.
- **Do NOT pull in any CANNOT-close item:** per-driver-identity / privilege-model arc · plugin-write
  → temperature-plugin arc · pipelined handler → own arc · `migrate-start` → migration subsystem ·
  live config reload → M7-standalone. Drift toward these → STOP and surface (Rule 3).
- **One verb per commit.** No `git add .` — explicit `git add <file>` per file. Joe pushes.

## 5. Verify (each commit)

`cargo test --workspace` (baseline **939**/0/1; A1 adds tests) + build all-targets + clippy
`-D warnings`. Adapter discipline (D-065): `members` is a pure lift — no new EventType, no new
backing.

## 6. After A1

A2 `ops::leave` (write, mirrors `join`) — also checkpoint-free. Then **CP-1 (Joe-lock)** before A3.
Full sequence + per-commit detail in the runbook §3–§6.

## 7. Cross-references

- Spec: `tasks/M7C_COMPLETION_IMPL.md` (runbook) · `tasks/M7C_COMPLETION_DESIGN.md` (M7C-D1–D4).
- Code template: `xgen-client/src/ops.rs:1031` (`ai_status` drain) + the M7 v1 verb wiring across
  `ops.rs` / `app.rs` / `batch.rs` / `aicontrol.rs`. Membership seeding: `state.rs`
  (`from_space_create:189`, `from_dm_space_create:263`).
