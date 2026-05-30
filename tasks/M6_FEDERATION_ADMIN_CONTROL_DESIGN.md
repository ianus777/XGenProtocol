# Federation-Admin-Control — Design (D-071 arc, design phase)
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

Design phase of the **federation-admin-control** D-071 arc. Entry artifact is
`tasks/M6_FEDERATION_ADMIN_CONTROL_AUDIT.md` (audit phase, ACTIVE) — read it first;
this stub does not restate the evidence. Per D-069 the arc runs audit → design →
impl as three canonical artifacts; per D-071 the audit precedes this design.

**This is a stub.** It opens the design doc and frames the decision space. No
design call is made here — those are Joe's, recorded below as they lock.

## Verdict carried in

**GAP IDENTIFIED — HIGH (whole-subsystem).** The 5 deferred verbs (`accept`,
`reject`, `set-policy`, `show-policy`, `initiate`) presuppose two subsystems that
do not exist — an approval/pending-request queue and a per-peer policy store —
plus an architectural change to the auto-establishing handshake for `initiate`.
The shipped subset (`list` + `defederate`, J-156) is unaffected.

## Design agenda (what this phase must produce)

From the audit's "what the design phase must build" — the targets, not the design:

1. Pending-request queue + a pause point in the handshake (`run_receiving` /
   `handle_federation_incoming`) so a request can wait pre-ACTIVE.
2. Relationship state model — a `FederationState` discriminator
   (pending/active/revoked/rejected) with backward-compatible load of existing
   (implicitly-active) JSON records.
3. Per-peer policy store + enforcement, with a consult site in
   `apply_federation_push` (and possibly the inbound F-3 gate).
4. Admin-gating treatment for `initiate` (see FAC-D1).
5. The 5 verb implementations in `admin_ops::*` once the above exist.

## Open design decisions

- **FAC-D1 — is federation approval opt-in? [OPEN]** (the audit's named decision.)
  Today's auto-establish-on-handshake is a legitimate posture, not a bug; admin
  approval is a *new mode*. Choose: always-on gate / configurable / per-peer. This
  choice drives the queue + state-model shape, so it is the first call to make.
- **FAC-D2 — where does relationship state live? [OPEN, candidate]** Extend
  `FederationRelationship` with a `state` field vs a sibling index. Must load
  existing records (no `state` field today) as implicitly-active.
- **FAC-D3 — policy enforcement site(s)? [OPEN, candidate]** Push-path only
  (`apply_federation_push`) vs also the inbound F-3 gate.

(FAC-D2/FAC-D3 are surfaced from the agenda for completeness; only FAC-D1 was named
by the audit. None are decided.)

## Decisions log

| ID | Decision | Status | Rationale |
|---|---|---|---|
| — | (none yet — design not started) | — | — |

Arc-local IDs (`FAC-D#`) live in this doc per D-069; a call graduates to a global
`D-###` in DECISIONS.md only when locked.

## Cross-refs

- Audit (entry artifact): `tasks/M6_FEDERATION_ADMIN_CONTROL_AUDIT.md`.
- `docs/xgen_node_admin_ops_design.md` §6.A1 (verb specs) + Appendix K.2.4.
- `tasks/M6_BACKING_AUDIT.md` A1 row.
- D-071 / D-069 / D-065. Sibling stubs: `tasks/M6_BOOTSTRAP_CLIENT_DESIGN.md`,
  `tasks/M6_AUTH_MODULE_REGISTRY_DESIGN.md`, `tasks/M6_PROTOCOL_AUDIT_LOG_DESIGN.md`.

---

*Stub. Design decisions await Joe; this is the decision scaffold, not the design.*
