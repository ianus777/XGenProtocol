# Bootstrap-Client — Design (D-071 arc, design phase)
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

Design phase of the **bootstrap-client** D-071 arc. Entry artifact is
`tasks/M6_BOOTSTRAP_CLIENT_AUDIT.md` (audit phase, ACTIVE) — read it first; this
stub does not restate the evidence. Per D-069 the arc runs audit → design → impl;
per D-071 the audit precedes this design.

**This is a stub.** It opens the design doc and frames the decision space. No
design call is made here — those are Joe's, recorded below as they lock.

## Verdict carried in

**GAP IDENTIFIED — HIGH (whole-subsystem).** All 5 verbs (`show`, `register`,
`deregister`, `set-info`, `set-tiers`) are absent-backed. The gap is wider than
"just the send-path": the client send-path is a placeholder **and** there is no
`[bootstrap]` config section, no registrations store, no self-info store. The
server-side directory/reputation machinery is real but is the orthogonal
Bootstrap-Node-*server* role. Scope is **client-only** in this arc (A3-D1).

## Design agenda (what this phase must produce)

From the audit's "what the design phase must build" — the targets, not the design:

1. Client HTTP send-path (reqwest) for `bootstrap.register` / `keepalive` /
   `deregister` + `*_ack` receive, in `xgen-node`, driving the existing
   `BootstrapMessage` wire types.
2. `[bootstrap]` config section on `NodeConfig` (D-035 convention).
3. Local registrations store — per-Bootstrap-Node records, add/remove/get/list/update.
4. Local self-info store — the `BootstrapInfo` + advertised tiers this Node publishes.
5. Re-advertise + keepalive scheduling (A3-D2: local update succeeds even if a
   re-advertise fails — honest per D-065); TTL keepalive task.
6. The 5 verb implementations in `admin_ops::*` once the above exist.

## Open design decisions

- **BC-D1 — where does local bootstrap state live? [OPEN]** (the audit's named
  decision.) TOML `[bootstrap]` config section vs a sibling JSON/SQLite store — and
  how registration TTL / keepalive scheduling composes with the existing
  reconnect-scheduler pattern (`xgen-node/src/reconnect.rs`).
- **BC-D2 — config vs store split for self-info/tiers? [OPEN, candidate]** If
  state is partly config and partly a store, which fields are operator-edited
  config vs runtime-mutable store (set-info / set-tiers write the latter).

(BC-D2 is surfaced from the agenda; only BC-D1 was named by the audit. Neither is
decided.)

## Decisions log

| ID | Decision | Status | Rationale |
|---|---|---|---|
| — | (none yet — design not started) | — | — |

Arc-local IDs (`BC-D#`) live in this doc per D-069; a call graduates to a global
`D-###` in DECISIONS.md only when locked.

## Cross-refs

- Audit (entry artifact): `tasks/M6_BOOTSTRAP_CLIENT_AUDIT.md`.
- `docs/xgen_node_admin_ops_design.md` §6.A3 (A3-D1/A3-D2) + Appendix K.2.3.
- Spec §3.14.3 / §3.14.4 / §3.14.7. `tasks/M6_BACKING_AUDIT.md` A3 row.
- D-071 / D-069 / D-065. Sibling stubs:
  `tasks/M6_FEDERATION_ADMIN_CONTROL_DESIGN.md`,
  `tasks/M6_AUTH_MODULE_REGISTRY_DESIGN.md`, `tasks/M6_PROTOCOL_AUDIT_LOG_DESIGN.md`.

---

*Stub. Design decisions await Joe; this is the decision scaffold, not the design.*
