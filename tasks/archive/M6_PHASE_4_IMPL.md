# M6 Phase 4 — A6 Logging & audit administration
> **Status**: COMPLETED  
> Version: 1.1  
> Date: May 2026  
> **Last updated**: 2026-05-29  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Purpose

The first real verb phase, and the **landing-pad category** (M6 design §6.A6): Phase 4 makes
the Phase-2 `audit` skeleton load-bearing — `audit::insert_entry` becomes the audit-write
primitive every subsequent WRITE/DESTRUCTIVE verb (Phases 5–10) consumes — and ships the
5 A6 verbs. Authoritative spec: `docs/xgen_node_admin_ops_design.md` §6.A6 + Appendix K.2.1.

## Verbs (5 — locked Block 4)

| Verb | Class | Audited (A6-D4) | Notes |
|---|---|---|---|
| `audit query` | READ | no | filtered read of the SQLite admin trail (§2.6.4) |
| `audit export` | READ (data-extracting) | **yes** | filters → JSONL file for SIEM |
| `audit archive` | DESTRUCTIVE | **yes** | export rows `< before` to dated file, then prune; fail-safe toward retention |
| `log show-level` | READ | no | effective tracing levels |
| `log set-level` | WRITE | **yes** | runtime-only via tracing reload handle (A6-D1); does NOT persist to config |

Two distinct audit logs (A6-D3): `audit *` = SQLite admin trail; the §3.11.8 protocol log is
spec-managed, no admin verb. Don't conflate. Propagation interaction = none for all A6 verbs.

## Locked sub-decisions feeding implementation

- **A6-D1 reload handle.** `log set-level` needs a `tracing_subscriber::reload::Layer`-backed
  `EnvFilter` whose handle is stashed in a process-global (`OnceLock`) so the pipe verb can
  `reload`/`modify` it. The current Node subscriber init (`xgen-node/src/app.rs:283` —
  `fmt().with_env_filter(...).init()`) has **no handle** and must be reworked to the
  registry+layers form **preserving current behaviour** (log-file writer, format, the
  `resolve_log_level` precedence). This is the delicate piece — a regression breaks all Node
  logging — so it is sequenced last (Commit 3) with its own focused verification.
- **A6-D4 audit-the-auditor.** WRITE/DESTRUCTIVE verbs (`log set-level`, `audit archive`) and
  the data-extracting `audit export` write an audit entry; `audit query` / `log show-level`
  do not.
- **Async dispatch.** `admin_ops::*` verbs are `async fn` (design §2.3). `pipe::dispatch_line`
  is currently sync but is called from the async `start_pipe_server` (pipe.rs:259) — it
  becomes `async` and routes the two-token A6 verbs into `admin_ops::*`, rendering
  `OK` / `ERROR <CODE>: <message>` per §2.7 (`AdminError::batch_reply`).

## Commit sequence

| # | Scope | Status |
|---|---|---|
| 1 | `audit` module extensions (filtered `query`, `archive` export+prune, `export` to JSONL) + admin_ops `audit query`/`audit export`/`audit archive` verbs (Args/Result) + audit-the-auditor wiring helper + tests | ✅ (8 tests; node lib 102→110) |
| 2 | async `pipe::dispatch_line` routing for the audit verbs (read-only allowlist preserved) + clap verb grouping (`AdminCli`/`AdminCommand`/`AuditCommand`/`LogCommand`) + tests | ✅ (4 dispatch tests; node 110→114) |
| 3 | Log verbs + reload-handle unit: reworked Node subscriber to registry+layers with a global reload handle + a `LogFilterState` store; `log show-level` (read) + `log set-level` (apply + audited); logging-regression check PASSED (live resident → byte-identical log format) | ✅ (3 tests; node 114→117) |
| 4 | Phase close: this file → COMPLETED + CLAUDE PLAY + JOURNAL J-154 + ROADMAP | ✅ |

Each commit verified (`cargo test --workspace` + clippy `-D warnings` + build all-targets).
Commits are logical/atomic; Joe pushes.

## Error-code bands (A6, §2.7 / Appendix K.5)

`AUDIT_5001` archive write failed · `AUDIT_5002` prune-after-archive failed (fail-safe: keep
rows) · `AUDIT_5010` bad filter / malformed timestamp · `AUDIT_5020` export write failed ·
`LOG_5101` invalid level · `LOG_5102` unknown/unsettable module · `GENERIC_4000` bad args.

## Definition of Done

- [x] `audit query` filtered read (actor/verb/since/until/outcome/limit; default 100, cap 1000).
- [x] `audit export` writes JSONL + is audited.
- [x] `audit archive` exports `< before` then prunes; prune-failure keeps rows (AUDIT_5002).
- [x] `log show-level` returns effective levels.
- [x] `log set-level` applies at runtime via reload handle + is audited; survives until restart; does NOT persist to config.
- [x] audit-the-auditor honoured (writes only for WRITE/DESTRUCTIVE + export; READs do not write).
- [x] `pipe::dispatch_line` (now async) routes all 5 verbs via the clap grouping; read-only allowlist unchanged.
- [x] `cargo test --workspace` green (672 lib + 25 integration, 0 failed); clippy `-D warnings` clean; build all-targets 0 errors.
- [x] Node-wide logging behaviour unchanged after the reload-handle rework — verified live (resident wrote `… INFO xgen_common::event_trace: …` with the ChronoLocal format; `--log-level debug` precedence honoured).

## Verification (close)

- `cargo test --workspace`: 672 lib (63 client + 35 common + 457 core + 117 node) + 25 integration; 0 failed. +15 node lib vs the Phase-4-start 102: +8 audit (Commit 1) + 4 dispatch (Commit 2) + 3 log (Commit 3).
- clippy `--workspace --lib --tests -D warnings`: clean. build `--workspace --all-targets`: 0 errors.
- Live logging-regression: ran `xgen-node --instance regrcheck --service --local --port 8099 --log-level debug`; log file format byte-identical to pre-rework; `--stop` clean; instance cleaned up.

## Reply-format note (Joe-confirmed 2026-05-29)

The M2 pipe error wrapper (`ERROR: <body>`) is unchanged; admin-verb errors supply the
body `<CODE>: <message>`, so the pipe reply is `ERROR: <CODE>: <message>` — one colon after
`ERROR`, **consistent with every other error on the pipe** (special-casing admin verbs to
drop the colon would put two error spellings on one channel). **§2.7 aligned to this exact
spelling** (design v1.11 → v1.12) so doc and code agree; the plain-text form is non-canonical
(the authoritative structured form is M7's `--aicontrol` JSON). Actor recorded as
`os-user:<name>` (§2.6.1 OS-user-equals-administrator, v1 — the only truthful actor until M7
introduces distinct admin identities; the `os-user:` prefix avoids colliding with real
`xgen://pubkey/...` URIs. The §2.6.4 `actor` "identity_id URI" wording is what M7 fills in).

## Next

Phase 5 — A5 Identity registry (Appendix K.2.2).

---

*End of Phase 4 plan.*
