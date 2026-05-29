# M6 Phase 9 (read subset) — A4 Space/Room admin: `space list-hosted`
> **Status**: COMPLETED  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-29  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Purpose

A4 Space & Room admin (design §6.A4, Appendix K.2.6) — the **backed read subset**,
shipped ahead of the design-gated `force-eject`. Per the J-156 backing audit
(`tasks/M6_BACKING_AUDIT.md`), A4 is SPLIT; this phase ships the one verb that is
genuinely backed.

## Backing reality (J-156 audit + this phase's recon)

| Verb | Class | Backing | Disposition |
|---|---|---|---|
| `space list-hosted` | READ | **BACKED** — reads live `runtime.spaces` (`home_node`, members, rooms, federation_nodes) | ✅ shipped |
| `space audit-events` | READ | **ABSENT** — reads the §3.11.8 protocol audit log, which is **unimplemented** (only the `event_trace` debug-tracing layer exists; no structured/queryable/rotating protocol-audit store, no `ProtocolAuditEntry`, no reader) | DEFERRED → **protocol-audit-log** arc (Joe-confirmed, J-157) |
| `space force-eject` | DESTRUCTIVE | needs `membership.node_eject` EventType | DEFERRED → A4-D1 wire sub-design session (design-gated) |
| `space set-node-policy` / `show-node-policy` | WRITE/READ | no `NodePolicy` store / enforcement | DEFERRED → node-policy arc |

**`audit-events` correction (J-157).** The backing audit's A4 row marked
`audit-events` BACKED ("reads §3.11.8 protocol log") — the same spec-exists-≠-
code-exists slip as its (since-corrected) A6 row. Recon confirmed the §3.11.8
protocol audit log is not built. Deferred to a protocol-audit-log subsystem arc
(build the structured rotating store + reader first). The audit doc's A4 row +
canonical §6.A4 are Joe's to correct (held with his other doc amendments).

## What shipped — `space list-hosted` (READ, not audited)

- Reads the **live** `runtime.spaces`, filtered to `home_node == this Node`
  (D-082 lock #4 — hosted/originated Spaces this Node homes, **never**
  federated-in replicas) + optional `--name-filter` (case-insensitive substring).
- `HostedSpaceSummary { space_id, name, member_count, room_count, federated_peers, created_at }`.
  `created_at` is `None` honestly (D-065): the Node persists no per-Space creation
  timestamp for originated Spaces in v1.
- Uses the existing A5 `AdminContext` runtime handle (`require_runtime`) — no new
  wiring. clap `AdminCommand::Space(SpaceCommand::ListHosted)`. `GENERIC_4000`
  only (a read filter; no domain error cases).

## Commit sequence (folded)

| # | Scope | Status |
|---|---|---|
| 1 | `space_list_hosted` verb + `HostedSpaceSummary`/Args/Result + clap `SpaceCommand` + verb test (via `build_space_create_event`/`sign_event`) | ✅ |
| 2 | pipe `dispatch_admin` `Space::ListHosted` arm (runtime already threaded) + dispatch-routing test | ✅ |
| 3 | Phase close: this file + JOURNAL J-157 + CLAUDE PLAY + ROADMAP | ✅ |

## Definition of Done

- [x] `space list-hosted` lists only hosted Spaces (`home_node == node_id`), excludes federated-in replicas; `--name-filter` works (case-insensitive); not audited.
- [x] `created_at` honest `None` (no stored creation timestamp in v1).
- [x] clap `space list-hosted` routes via `dispatch_line`; M2 read-only allowlist unchanged.
- [x] `cargo test --workspace` green (690 lib + 25 integration, 0 failed); clippy `-D warnings` clean; build all-targets 0 errors.

## Verification (close)

- `cargo test --workspace`: **690 lib** (63 client + 35 common + 465 core + 127 node) + 25 integration; 0 failed. +2 node lib vs Phase 7's 688 (1 verb test + 1 dispatch-routing test). xgen-core unchanged (465) — A4 read added no core code.
- clippy `--workspace --lib --tests --all-features -- -D warnings`: clean. build `--workspace --all-targets`: 0 errors.

## M6 write-path tally (after this phase)

**Implemented (12 verbs):** A6 (5) + A5 (4) + A1 subset (2) + A4 subset (1).
**Backed-but-not-yet-implemented:** A7 Plugin `list` + `status` (Phase 10, 2 reads — audit marks BACKED; verify at pickup). **Plus `force-eject`** pending its A4-D1 wire session.
**19 verbs route to subsystem arcs** (the 18 from the J-156 audit + `audit-events` → protocol-audit-log).

## Next

**A7 Plugin (Phase 10)** — `plugin list` + `plugin status`, the last backed verb
phase per the audit (verify the no-op-temperature-plugin backing at pickup). Then
`force-eject` (A4-D1 design-gated session) and Joe's pending doc amendments
(§5.1/§6, the audit doc's A4 row, the four arc stubs).

---

*End of Phase 9 (read subset) plan.*
