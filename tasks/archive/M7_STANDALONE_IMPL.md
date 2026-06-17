# M7-standalone — Implementation Runbook (live config reload)
> **Status**: COMPLETED  
> Version: 1.2  
> Date: Jun 2026  
> **Last updated**: 2026-06-02  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 Status

Implementation runbook for M7-standalone, on `tasks/M7_STANDALONE_DESIGN.md` v1.3 (M7S-D1…D6 locked). **SHIPPED + CLOSED at J-226 (2026-06-02).** Two drafting findings (Rule 3) resolved 2026-06-01: F-R2 adopted (reload writes no config); CP-1 resolved (baseline retained).

**As-built (J-226):**
- **C1** `f8777ac` — `config_reload.rs` pure substrate (`reload_plan` + `format_*` + `listen_addr_valid`); +11 tests.
- **C2** `a63a73b` — `handle_reload` handler body + CP-1 baseline; **C3 folded** (the `cmd_reload_config` exit-code change was two lines); +8 tests.
- **CP-1 home (confirm-at-pickup, RESOLVED):** a **dedicated `Arc<Mutex<NodeConfig>>`** threaded `run_node → start_pipe_server`, **not** a field on `NodeRuntime` — `NodeRuntime` lives in xgen-core and must not depend on the xgen-node `NodeConfig` type (layering). The handler already reaches the snapshot via the new param.
- **Snapshot update rule (Design A):** the runbook §3 rule ("restart-required does not update the snapshot — snapshot = what's *running*") and the §4 test-bullet ("re-run does not re-report") were in tension. Implemented the lie-free reading (Design A): restart-required fields never update the snapshot, so a divergent field is honestly re-reported `PENDING_RESTART` on every reload (true until restart) and edit-then-revert never produces a false report. The §4 test became "re-run keeps reporting PENDING_RESTART; snapshot stable." Recorded in J-226 per D-065.
- Suite **984**/0/1 (+19 from 965). §2.6.3 correction landed at close.

---

## §1 What ships

The §2-design mechanism behind the `__RELOAD_CONFIG__` handler (`xgen-node/src/pipe.rs:819`, today a `NOT_IMPLEMENTED` stub): re-read → all-or-nothing gate → diff/classify → apply `[logging].level` live → report. Node-only (M7S-D5), legacy pipe surface only (M7S-D2). `--batch` / `--aicontrol` untouched (D-066).

---

## §2 Two drafting findings (Rule 3)

- **F-R1 (→ CP-1, RESOLVED) — the diff baseline.** The field-level no-lie report (M7S-D4) lists *changed* fields, so the reload must diff the re-read config against the **running** config. The `__RELOAD_CONFIG__` handler holds `config_path` + `runtime` but **not** the startup-loaded `NodeConfig`. A baseline must be retained — warranted by M7S-D4's field-level "never silently ignored" contract, not optional. Resolved shape in §3.
- **F-R2 (ADOPTED — refines design §2 step 5) — "persist restart-required" is a no-op.** The operator **edits `config.toml` first, then runs `--reload-config`**; the new values are therefore **already on disk** before reload runs. Reload re-reads that same file (M7S-D2/M7S-D3 = `try_load_config(config_path)`), so there is **nothing to write back** for restart-required fields. **Consequence:** v1 writes **no** config; it applies reloadable fields live and reports the rest as pending-restart. This **avoids the `toml::to_string_pretty` footgun** (a full re-serialise would destroy operator comments + formatting). Adopted: design §2 step 5 → "already on disk; report only."

---

## §3 CP-1 (RESOLVED) — the running-config baseline seam

Resolved shape (the *decision*; exact wiring site = confirm-at-pickup, D-078, at Commit 2):

- **Retain** the startup-effective `NodeConfig` in a shared handle reachable by the pipe handler. **Recommended home:** a field on `NodeRuntime` behind the existing `Arc<Mutex<…>>` (the handler already reaches `runtime`), or a dedicated `Arc<Mutex<NodeConfig>>` threaded to `start_pipe_server` alongside `config_path`. The exact site is verified against the live runtime struct at Commit 2 pickup.
- **Update rule on a successful reload:** **reloadable** fields (`[logging].level`) update the snapshot (next diff baseline stays correct); **restart-required** fields do **not** update the snapshot (snapshot stays = what's *running*, not what's on disk) — so a second reload does not falsely re-report an already-seen restart-required edit as freshly changed.
- **F-R2 confirmed:** no config write-back (§2).

---

## §4 Commit plan

**Commit 1 — pure diff/classify + report substrate** (checkpoint-free after CP-1; no wiring).
- A pure `reload_plan(old: &NodeConfig, new: &NodeConfig) -> ReloadPlan` producing per-field dispositions via the M7S-D6 classification table: `[logging].level` → reloadable; `[node].listen` / `[node].local_mode` / `[paths].keypair_path` / `[sync].*` / `[federation].require_approval` → restart-required; `[bootstrap]` → N/A/seed-only.
- The `[node].listen` `SocketAddr` semantic check (M7S-D3) as a gate predicate.
- The report-line formatter (M7S-D4): `RELOADED: …; PENDING_RESTART: …; NA: …; REJECTED: field=reason` — only changed deltas; `REJECTED` always carries a reason (`parse`/`semantic`/`unknown`).
- Unit tests: each disposition; the `listen` gate (valid/invalid); report formatting incl. the empty-segments case; the no-lie cases (restart-required change → `PENDING_RESTART`; `[bootstrap]` change → `NA`, never `PENDING_RESTART`).

**Commit 2 — baseline retention + the `__RELOAD_CONFIG__` handler body** (the CP-1 shape).
- Retain the startup `NodeConfig` snapshot (CP-1 home) + the update rule.
- Handler: re-read via `try_load_config` → **all-or-nothing gate** (parse fail → `REJECTED: …=parse`; `listen` `SocketAddr` fail → `REJECTED: listen=semantic`; on either, apply nothing) → `reload_plan(snapshot, new)` → apply `[logging].level` live via `LOG_RELOAD` (A6-D1 path) + update snapshot level → build + return the report line. **No config write-back (F-R2).** Replace the `NOT_IMPLEMENTED` string.
- Tests: handler over the duplex/in-process pattern — reloadable change applied + reported; restart-required change reported pending-restart + not applied + snapshot unchanged; `[bootstrap]` change → NA; bad parse + bad `listen` → REJECTED with reason, nothing applied; a re-run after a restart-required edit does not re-report it as changed (snapshot-baseline correctness).

**Commit 3 — client-side surface tidy** (fold into Commit 2 if trivial).
- `cmd_reload_config` (`pipe.rs:1142`) surfaces the structured line as-is; confirm the non-Windows stub message (`pipe.rs:1190`) still reads correctly. No logic.

**Close — D-074 atomic, doc-only.**
- **§2.6.3 correction** in `docs/xgen_node_admin_ops_design.md` (the M7S-D1 text: `[node].local_mode` → Restart-required, gates trust admission, admitted identities persist unreconciled, reload would desync gate from registry; no live-seam footnote).
- Canonical doc updates as needed; ROADMAP M7-standalone ✅ + version; CLAUDE PLAY → next milestone; JOURNAL close entry. Flip audit + design + this runbook → COMPLETED.

---

## §5 Per-commit DoD

- `cargo test --workspace` green (baseline **965**/0/1; record the delta).
- `cargo build --workspace --all-targets` 0/0.
- `cargo clippy --workspace --lib --tests --all-features -- -D warnings` clean.
- Explicit `git add <file>` per file; `git status` before commit; multi-paragraph `-m`. Joe pushes manually.
- No DECISIONS.md change (M7S-D# arc-local, D-069; D-074 per-commit). No "commit pushed" checklist item.

---

## §6 Next-active

**Clair picks up Commit 1.** The CP-1 *decision* is resolved (§3); the only open item is the exact baseline-handle wiring site, a confirm-at-pickup (D-078) at Commit 2. No blocking Joe-lock remains.
