# Protocol-Audit-Log — Design (D-071 arc, design phase)
> **Status**: PENDING  
> Version: 0.1  
> Date: May 2026  
> **Last updated**: 2026-05-30  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Arc position

Design phase of the **protocol-audit-log** D-071 arc. Entry artifact is
`tasks/M6_PROTOCOL_AUDIT_LOG_AUDIT.md` (audit phase, ACTIVE) — read it first; this
stub does not restate the evidence. Per D-069 the arc runs audit → design → impl;
per D-071 the audit precedes this design.

**This is a stub.** It opens the design doc and frames the decision space. No
design call is made here — those are Joe's, recorded below as they lock.

**Recurring hazard, restated:** the §3.11.8 protocol audit log is **NOT** the A6
SQLite admin trail (`audit.rs`, shipped J-154). Conflating the two is the slip that
mis-marked the A4 row BACKED twice (corrected J-157). The verb here reads the
former; only the latter exists.

## Verdict carried in

**GAP IDENTIFIED — HIGH (whole-subsystem, compliance-grade).** The §3.11.8 log — a
normative, compliance-bearing requirement (Tier-3+ always-on) — has no
implementation: no entry type, no store, no writer hooks, no reader. The single
deferred verb `space audit-events` is only the *reader*; the load-bearing missing
piece is the **writer side**.

## Design agenda (what this phase must produce)

From the audit's "what the design phase must build" — the targets, not the design:

1. `ProtocolAuditEntry` — struct matching the §3.11.8 schema (`ts` / `event_type` /
   `event_id` / `node_id` + EventType-specific fields).
2. Append-only, monthly-rotated JSONL store at
   `audit/protocol_audit_YYYY-MM.jsonl`, operator-retained (no auto-delete).
3. **Writer hooks** at protocol-Event ingest for all 11 §3.11.8 EventTypes — the
   load-bearing part (the reader is useless without recorded entries).
4. The `space audit-events` reader (`admin_ops::space_audit_events`) — args
   space_id + optional event_type/since/until/limit/cursor; READ, not audited
   (A4-D3); errors SPACE_8001 / SPACE_8010.
5. Rotation + retention controls (monthly boundaries; manual deletion only;
   Tier-3+ always-on).

## Open design decisions

- **PAL-D1 — how do the writer hooks compose with the dispatch pipeline? [OPEN]**
  (the audit's named decision.) Does the protocol-audit write ride the same
  post-ingest path as `persist_event` / `trace_local`, and is the per-Space scope
  filter applied at **write-time** or **read-time**?
- **PAL-D2 — entry write timing vs persist? [OPEN, candidate]** Best-effort after
  persist (D-070 sibling, as Option B fan-out) vs same-transaction with persist —
  trades durability coupling against a fail-open audit gap.

(PAL-D2 is surfaced from the agenda; only PAL-D1 was named by the audit. Neither is
decided.)

## Decisions log

| ID | Decision | Status | Rationale |
|---|---|---|---|
| — | (none yet — design not started) | — | — |

Arc-local IDs (`PAL-D#`) live in this doc per D-069; a call graduates to a global
`D-###` in DECISIONS.md only when locked.

## Cross-refs

- Audit (entry artifact): `tasks/M6_PROTOCOL_AUDIT_LOG_AUDIT.md`.
- Spec `docs/xgen_ch3_specification.md` §3.11.8 (the normative requirement).
- `docs/xgen_node_admin_ops_design.md` §6.A4 (A4-D3) + Appendix K.2.6.
- `xgen-node/src/audit.rs` (the DISTINCT A6 admin trail — do not conflate);
  `xgen-common/src/event_trace.rs` (debug layer, not a store).
- `tasks/M6_BACKING_AUDIT.md` A4 row. D-071 / D-069 / D-065. Sibling stubs:
  `tasks/M6_FEDERATION_ADMIN_CONTROL_DESIGN.md`, `tasks/M6_BOOTSTRAP_CLIENT_DESIGN.md`,
  `tasks/M6_AUTH_MODULE_REGISTRY_DESIGN.md`.

---

*Stub. Design decisions await Joe; this is the decision scaffold, not the design.*
