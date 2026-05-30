# Protocol-Audit-Log — Backing Audit (D-071 arc, audit phase)
> **Status**: ACTIVE  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-30  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Why this exists

The **protocol-audit-log** arc is one of the four post-M6 D-071 subsystem arcs
(`tasks/M6_BACKING_AUDIT.md`). Per D-071 each arc opens with a backing audit — a
read-only, evidence-cited pass mapping the deferred verb to what exists, so the
design phase starts from reality. Audit phase only; gaps are routed, not designed.
Verified against the live tree on 2026-05-30; the absence was grep-confirmed.

The crux of this audit is a distinction that has already been mis-stated twice in
project history (the M6 audit-doc A4 row claimed BACKED; corrected at J-157): the
**§3.11.8 protocol audit log** is NOT the **A6 SQLite admin trail**. The verb that
needs this arc reads the former; only the latter exists.

## Scope — the deferred verb

A4's `space audit-events` (designed to read the §3.11.8 protocol audit log)
routes here. It was marked BACKED in an early backing-map snapshot and corrected to
DEFERRED at J-157 when recon found the §3.11.8 store is unimplemented.

Design source: `docs/xgen_node_admin_ops_design.md` §6.A4 (A4-D3) + Appendix K.2.6.
Spec source: `docs/xgen_ch3_specification.md` §3.11.8.

## What EXISTS (verified)

- **`event_trace`** (`xgen-common/src/event_trace.rs`) — a **debug-instrumentation
  layer**, not an audit store. `trace_event` / `trace_local` /
  `write_session_header` / `write_session_footer` emit via `tracing::debug!`/`info!`
  into the configured `tracing` subscriber (the debug-log files). Role-gated; the
  content field is never logged. It is a stream, not a **structured, queryable,
  rotating** store, and it has no reader.
- **The A6 SQLite admin trail** (`xgen-node/src/audit.rs`, the `audit_entries`
  table) — real and shipped (J-154). It records **admin-verb actions** (verb,
  actor, actor_via, target, args_hash, outcome, error_code, correlation_id) and is
  read by `audit query` / `audit export` / `audit archive`. Its own module doc
  says, verbatim: *"This is the SQLite admin trail — distinct from the §3.11.8
  protocol audit log that `space audit-events` (Phase 9, A4) reads. Do not conflate
  the two."* **This is NOT the §3.11.8 log.**
- **The spec, §3.11.8** — normative and detailed: an **append-only protocol audit
  log** recording all membership/state-change Events (11 EventTypes:
  `membership.{join,leave,invite,kick,ban}`, `state.{space_create,room_create,
  federation_add,federation_remove}`, `identity.register`, `system.key_rotation`),
  mandatory fields per entry (`ts`, `event_type`, `event_id`, `node_id` + EventType
  specifics), monthly-rotated JSONL at `audit/protocol_audit_YYYY-MM.jsonl`,
  operator-controlled retention (no auto-delete), always-on at Tier 3+.

## What is ABSENT (the gap, verified)

- **No protocol-audit store.** Grep for `ProtocolAuditEntry` / `protocol_audit_<n>`
  / `struct ProtocolAudit` returns **no Rust source matches** — only docs
  (JOURNAL, this design doc, Ch4, a legacy LOGGING task) (verified). There is no
  `ProtocolAuditEntry` type, no JSONL store, no writer hooks at the 11 EventType
  ingest points, no monthly rotation, and no reader.
- **No `space audit-events` verb.** The `SpaceCommand` clap enum
  (`xgen-node/src/admin_ops.rs`) has `ListHosted` / `ForceEject` / `Unban` only;
  there is no `AuditEvents` variant. The A4 section comment states it directly:
  the §3.11.8 log "is UNIMPLEMENTED (only the `event_trace` debug-tracing layer
  exists — no structured, queryable, rotating protocol-audit store)."

## A6 admin trail vs §3.11.8 protocol log (the distinction)

| Dimension | A6 admin trail (EXISTS) | §3.11.8 protocol log (ABSENT) |
|---|---|---|
| Records | admin-verb actions (who ran what verb) | protocol Events in the DAG (membership/state/identity/system) |
| Written by | `admin_ops::record_action` | (would be) protocol-Event ingest hooks |
| Read by | `audit query`/`export`/`archive` (shipped) | `space audit-events` (deferred → this arc) |
| Store | SQLite `xgen-node_audit.db` | (would be) JSONL `audit/protocol_audit_YYYY-MM.jsonl` |
| Status | **SHIPPED (J-154)** | **UNIMPLEMENTED** |

## Per-verb backing

| Verb | Class | Backing | Evidence |
|---|---|---|---|
| `space audit-events` | READ | **ABSENT** | §3.11.8 store unbuilt; only `event_trace` debug logs + the distinct A6 SQLite admin trail exist |

## Verdict

**GAP IDENTIFIED — HIGH (whole-subsystem, compliance-grade).** The §3.11.8 protocol
audit log — a normative, compliance-bearing requirement (Tier 3+ always-on) — has
no implementation: no entry type, no store, no writer hooks, no reader. The verb
`space audit-events` cannot ship until the store exists. The recurring hazard here
is **conflation with the A6 admin trail**; this audit states the distinction
explicitly so the design phase does not re-discover it. M6 backing assessment
(deferred) is **confirmed**.

Scope note: the gap is larger than the one verb — the missing piece is the
**writer side** (hooks that record the 11 EventTypes as they ingest), which is
protocol infrastructure, not an admin verb. The verb is only the *reader*.

## What the design phase must build (inputs to the design arc — NOT the design)

1. **`ProtocolAuditEntry`** — a struct matching the §3.11.8 schema (mandatory
   `ts` / `event_type` / `event_id` / `node_id` + EventType-specific fields).
2. **An append-only, monthly-rotated JSONL store** at
   `audit/protocol_audit_YYYY-MM.jsonl`, operator-retained (no auto-delete).
3. **Writer hooks** at protocol-Event ingest for all 11 §3.11.8 EventTypes, firing
   for Events in hosted/federated-to Spaces — the **load-bearing** part (the verb
   is useless without recorded entries).
4. **The `space audit-events` reader** (`admin_ops::space_audit_events`) — args
   `space_id` + optional `event_type` / `since` / `until` / `limit` / `cursor`;
   result `{ events, returned, next_cursor }`; READ, not audited (A4-D3); errors
   `SPACE_8001` not-hosted / `SPACE_8010` bad-filter.
5. **Rotation + retention controls** — monthly file boundaries; manual operator
   deletion only; Tier-3+ always-on semantics.

A design decision the design phase must take: **how the writer hooks compose with
the existing dispatch pipeline** — whether the protocol-audit write rides the same
post-ingest path as `persist_event` / `trace_local`, and where the per-Space scope
filter is applied (write-time vs read-time).

## Carry-overs & cross-refs

- `docs/xgen_ch3_specification.md` §3.11.8 (the normative requirement).
- `docs/xgen_node_admin_ops_design.md` §6.A4 (A4-D3 — the verb spec) + Appendix K.2.6.
- `xgen-node/src/audit.rs` (the DISTINCT A6 admin trail — do not conflate).
- `xgen-common/src/event_trace.rs` (the debug layer — not an audit store).
- `tasks/M6_BACKING_AUDIT.md` A4 row (note: its A4 `audit-events` cell still reads
  BACKED — a known stale row, reserved for Joe's correction alongside the §5.1/§6.A4
  amendments; this audit is the authoritative statement of the ABSENT reality).
- Future design stub: `tasks/M6_PROTOCOL_AUDIT_LOG_DESIGN.md` (Joe-reserved).
- D-071 / D-069 / D-065. Sibling arc audits:
  `tasks/M6_FEDERATION_ADMIN_CONTROL_AUDIT.md`, `tasks/M6_BOOTSTRAP_CLIENT_AUDIT.md`,
  `tasks/M6_AUTH_MODULE_REGISTRY_AUDIT.md`.

---

*End of audit (audit phase). Design + implementation are the subsequent arc steps.*
