# Protocol-Audit-Log — Design (D-071 arc, design phase)
> **Status**: ACTIVE  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-30  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Arc position

Design phase of the **protocol-audit-log** D-071 arc. Entry artifact:
`tasks/M6_PROTOCOL_AUDIT_LOG_AUDIT.md` (audit phase). Per D-069 the arc runs
audit → design → impl; per D-071 the audit precedes this design. **Decisions
locked J-164 (2026-05-30).** Next step: implementation runbook (Chat Claude + Joe);
this doc flips COMPLETED at runbook Commit 1.

**Recurring hazard (restated):** the §3.11.8 protocol audit log is **NOT** the A6
SQLite admin trail (`audit.rs`, J-154). Different log, different store, different
reader. The A6 trail records *admin-verb actions*; this records *protocol Events*.

## Verdict carried in

**GAP IDENTIFIED — HIGH (whole-subsystem, compliance-grade).** No `ProtocolAuditEntry`,
no store, no writer hooks, no reader. The single deferred verb `space audit-events`
is only the *reader*; the load-bearing missing piece is the **writer side**.

## Spec grounding (§3.11.8 — what shaped the locks)

1. **It is a recoverable projection of the DAG.** Entries record *summary facts*;
   the full Event is always recoverable from the DAG via `event_id`. A missed
   audit write is therefore a *replayable gap*, not data loss.
2. **Always-on at all tiers; tier governs retention, not writes.** The Node MUST
   maintain it and it cannot be disabled by config. Tier 3/4 impose retention
   *minimums* (operator + no-auto-delete concern) — there is **no per-event tier
   write-gate**.
3. **One Node-global file per calendar month** (`audit/protocol_audit_YYYY-MM.jsonl`),
   recording the 11 EventTypes in any Space **hosted-by OR federated-to** this Node
   (broader than `list-hosted`'s hosted-only scope). Per-Space scoping is therefore
   a **read-time** filter, not a write-time split.
4. **Tamper-evidence (hash-chain) is NOT required for the Node-level log.** §3.11.8
   SHOULDs it only for the *Auth Module* audit log (T3/T4, which lives inside the
   module, not the Node). Out of scope here (see Scope).

## Decisions locked (J-164)

- **PAL-D1 — writer-hook composition + scope filter. [LOCKED]** A **single
  post-accept writer hook** appends to the audit log, sibling-placed to where
  Option B's fan-out/federation hooks already sit (the post-persist point in
  `xgen-node::app::process_inbound`); it matches the 11 §3.11.8 EventTypes. Store
  is the **Node-global monthly JSONL** file (spec). Per-Space scoping is **read-time**
  in the `space audit-events` reader. (Exact hook site/line confirmed by code-trace
  at runbook pickup per the standing pre-impl discipline.)
- **PAL-D2 — durability posture: best-effort after persist + loud failure. [LOCKED]**
  The event lands in the DAG (persist) first; the audit append is best-effort after
  it (sibling to D-070 / Option B), **never fail-closed**. A write failure is
  surfaced loudly — `error`-level log + a health counter — **never silently
  swallowed** (D-065). Rationale: the log is a recoverable DAG projection, so
  fail-closed buys no durability the DAG doesn't already provide, while coupling
  protocol liveness to audit-disk health would let a full disk halt every Space on
  the Node. Best-effort + PAL-D3 rebuild gives an eventually-complete record
  provably consistent with the DAG. (Tamper-evidence, the stronger compliance axis,
  is orthogonal and out of scope for v1.)
- **PAL-D3 — rebuild-from-DAG: in scope, operator-invoked. [LOCKED]** An explicit
  operator action replays the DAG to (re)generate audit entries — for one Space or
  all hosted/federated Spaces. It closes any PAL-D2 gap **and** backfills Spaces
  whose events predate the log (cold start). **Never silent/automatic** — bounded
  and visible. Surfaced as a new admin verb (name + any optional startup-reconcile
  flag pinned at runbook).

## Scope (v1)

**In:** `ProtocolAuditEntry` type; Node-global monthly-rotated append-only JSONL
store; the single post-accept writer hook (11 EventTypes); the `space audit-events`
reader (§6.A4 A4-D3); the operator rebuild action (PAL-D3).
**Out:** hash-chain / tamper-evidence (Node-level log doesn't require it — deferred);
the **Auth Module** audit log (§3.11.8 T3/T4 — separate subsystem, lives in the
module, not the Node); automatic/scheduled rebuild (PAL-D3 is operator-invoked only).

## Entry schema (from §3.11.8)

Mandatory in every entry: `ts` (RFC 3339 UTC, ms), `event_type`, `event_id`
(DAG-linking hash URI), `node_id`. Plus EventType-specific summary fields per the
§3.11.8 table — the 11 types: `membership.{join,leave,invite,kick,ban}`,
`state.{space_create,room_create,federation_add,federation_remove}`,
`identity.register`, `system.key_rotation`. Summary facts only — the full Event is
recovered from the DAG via `event_id`.

## What the runbook must build / pin (inputs to impl)

1. `ProtocolAuditEntry` — struct + serde (flat JSON-Lines per §3.11.8); the 11
   per-EventType field sets.
2. Append-only monthly-rotated store at `audit/protocol_audit_YYYY-MM.jsonl`
   (operator-retained, no auto-delete); rotation on the month boundary.
3. The single writer hook — **pin the exact post-persist site** in
   `process_inbound` (code-trace at pickup, sibling to the Option B hooks); the
   11-EventType match; loud-failure handling (PAL-D2).
4. `space audit-events` reader (`admin_ops::space_audit_events`) — args + result
   per §6.A4 A4-D3; **read-time `space_id` filter**; pagination across month-file
   boundaries; errors `SPACE_8001` / `SPACE_8010`.
5. The PAL-D3 rebuild verb — name, scope (one Space / all), and whether a
   startup-reconcile flag accompanies it.

## Decisions log

| ID | Decision | Status | Rationale (short) |
|---|---|---|---|
| PAL-D1 | Single post-accept writer hook + Node-global monthly JSONL + read-time space filter | **LOCKED** (J-164) | Spec settles store + scope; single site mirrors Option B / persist placement |
| PAL-D2 | Best-effort after persist + loud failure; never fail-closed | **LOCKED** (J-164) | Log is a recoverable DAG projection; fail-closed would couple liveness to audit-disk health for no real gain |
| PAL-D3 | Rebuild-from-DAG, operator-invoked, in scope; never silent | **LOCKED** (J-164) | Closes PAL-D2 gaps + cold-start backfill; bounded + visible |

Arc-local IDs (`PAL-D#`) live in this doc per D-069; a call graduates to a global
`D-###` only if it becomes a project-wide principle.

## Cross-refs

- Audit (entry artifact): `tasks/M6_PROTOCOL_AUDIT_LOG_AUDIT.md`.
- Spec `docs/xgen_ch3_specification.md` §3.11.8 (the normative requirement).
- `docs/xgen_node_admin_ops_design.md` §6.A4 (A4-D3 — the reader verb) + Appendix K.2.6.
- `xgen-node/src/audit.rs` (the DISTINCT A6 admin trail — do not conflate);
  `xgen-common/src/event_trace.rs` (debug layer, not a store);
  `xgen-node/src/app.rs::process_inbound` (the post-persist hook site, PAL-D1).
- `tasks/M6_BACKING_AUDIT.md` A4 row (v1.1). D-071 / D-069 / D-065 / D-070.
  Sibling stubs: `tasks/M6_FEDERATION_ADMIN_CONTROL_DESIGN.md`,
  `tasks/M6_BOOTSTRAP_CLIENT_DESIGN.md`, `tasks/M6_AUTH_MODULE_REGISTRY_DESIGN.md`.

---

*Design decisions locked (J-164). Next: implementation runbook; this doc flips COMPLETED at runbook Commit 1.*
