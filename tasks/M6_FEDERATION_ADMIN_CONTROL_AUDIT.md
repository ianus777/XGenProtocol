# Federation-Admin-Control — Backing Audit (D-071 arc, audit phase)
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

The **federation-admin-control** arc is one of the four post-M6 D-071 subsystem
arcs (`tasks/M6_BACKING_AUDIT.md`). Per D-071 — "subsystem audits precede
dependent milestones" — each arc opens with a **backing audit**: a read-only,
evidence-cited pass mapping the arc's deferred verbs to what actually exists in
code, so its design phase starts from reality rather than from the spec's view.

This is the audit phase only. It records what *is* and routes gaps to the design
phase; it does **not** design the fix (that is the next arc step). Verified
against the live tree on 2026-05-30; load-bearing absences were grep-confirmed.

## Scope — the deferred verbs

M6 Block 4 designed **7** A1 federation verbs; the J-156 honest-subset shipped
`federation list` + `federation defederate` (both backed by `FederationRegistry`).
The remaining **5** route here:

`federation accept` · `federation reject` · `federation set-policy` ·
`federation show-policy` · `federation initiate`

Design source: `docs/xgen_node_admin_ops_design.md` §6.A1 + Appendix K.2.4.

## What EXISTS (verified)

- **`FederationRegistry`** (`xgen-core/src/federation/registry.rs`) — real and
  load-bearing (wired in the federation milestone, consumed by the A1 shipped
  subset). Public API: `upsert` / `remove` / `get` / `all` / `mark_active` /
  `mark_lost` / `update_next_reconnect` / `peer_record` / `due_for_reconnect` /
  `peer_records` / `save` / `load`.
- **`FederationRelationship`** (`registry.rs:40-54`, verified) — fields:
  `peer_node_id`, `shared_spaces`, `negotiated_version`, `negotiated_serialisation`,
  `session_id`, `last_connected`, `peer_url: Option<String>`. **No `state` /
  `status` field** — a recorded relationship is, implicitly, established/active.
- **`PeerOperationalRecord`** — operational health only (`lost_connection: bool`,
  `last_seen`, `last_successful_session`, `next_reconnect_attempt`, `operator_notes`,
  `priority`). The only state discriminator is `lost_connection` (connected vs
  reconnect-scheduled) — not an approval lifecycle.
- **Handshake state machine** (`xgen-core/src/federation/handshake.rs`) —
  `IDLE → … → ACTIVE`. Both sides transition straight to ACTIVE on a successful
  handshake; there is **no pending/approval state** in the machine.
- **Reconnect scheduler** (`xgen-node/src/reconnect.rs`) — the production caller of
  `run_initiating`; it iterates **peers already in the registry** (`due_for_reconnect`),
  not a request queue.
- **In-tree evidence of the gap** — the A1 honest-subset comment in
  `xgen-node/src/admin_ops.rs` (the A1 section header, verified): "there is NO
  admin-approval pending-request queue (federation auto-establishes on handshake)
  and NO per-peer policy store / enforcement."

## What is ABSENT (the gap, verified)

- **No admin-approval / pending-federation-request queue.** Grep across the
  federation module + `app.rs` for pending/approval/queue in a federation context
  returns nothing. Federation auto-establishes to ACTIVE on handshake completion,
  then is persisted — there is no pause point at which an administrator could
  `accept` or `reject` a request.
- **No relationship state field.** `FederationRelationship` carries no
  `pending`/`active`/`revoked` discriminator (verified at `registry.rs:40-54`).
- **No per-peer policy store or enforcement layer.** No `FederationPolicy` type,
  no per-peer allow/deny/rate-limit store, no consult site in the push path.
- **No admin-gate in the handshake flow.** Auto-establish is baked into the
  control flow (`run_initiating`/`run_receiving` → post-handshake → `mark_active`);
  `initiate` would not merely *call* the handshake but would require pausing its
  completion behind an approval step.

## Per-verb backing

| Verb | Class | Backing | Evidence |
|---|---|---|---|
| `federation accept` | WRITE | **ABSENT** | no pending-request queue to act on |
| `federation reject` | DESTRUCTIVE | **ABSENT** | nothing to reject (no request store) |
| `federation set-policy` | WRITE | **ABSENT** | no per-peer policy store / enforcement |
| `federation show-policy` | READ | **ABSENT** | reads a policy store that doesn't exist |
| `federation initiate` | WRITE | **ABSENT (architectural)** | handshake auto-establishes; no admin pause point |

## Verdict

**GAP IDENTIFIED — HIGH (whole-subsystem).** All 5 deferred verbs presuppose two
subsystems that do not exist (an approval/pending-request queue, and a per-peer
policy store), plus an architectural change to the handshake flow for `initiate`.
The M6 backing-map assumption (all 5 absent) is **confirmed** with no surprises.

The shipped subset (`list` + `defederate`) is unaffected — it reads/removes the
existing registry and stays correct.

## What the design phase must build (inputs to the design arc — NOT the design)

1. **Pending-request queue** — a store of inbound handshake requests awaiting
   admin approval (peer_node_id, received_at, expiry, request details), plus a
   **pause point** in `handle_federation_incoming` / `run_receiving` so a request
   can wait pre-ACTIVE. Consumed by `accept` (complete + upsert) / `reject`.
2. **Relationship state model** — a `FederationState` discriminator
   (pending/active/revoked/rejected) on the relationship or in a sibling index,
   with backward-compatible load of existing (implicitly-active) JSON records.
3. **Per-peer policy store + enforcement** — a `FederationPolicy` per peer
   (mode allow/deny, allowed_spaces, rate_limit) with a consult site in
   `apply_federation_push` (and possibly the inbound F-3 gate). Consumed by
   `set-policy` / `show-policy`.
4. **Admin-gating design decision for `initiate`** — auto-accept (today) vs
   admin-approval-required, and whether that is per-Node or per-peer. This is a
   protocol-design call for the design phase, not a verb detail.
5. **The 5 verb implementations** in `admin_ops::*` once the above exist.

A design decision the design phase must take: **whether approval is opt-in.**
Today's auto-establish is a legitimate posture; admin-gating is a new mode, not a
bug fix. The design phase chooses whether the gate is always-on, configurable, or
per-peer — that choice drives the queue + state-model shape.

## Carry-overs & cross-refs

- `docs/xgen_node_admin_ops_design.md` §6.A1 (verb specs) + Appendix K.2.4 (index).
- `tasks/M6_BACKING_AUDIT.md` A1 row (the high-level map this deepens).
- Future design stub: `tasks/M6_FEDERATION_ADMIN_CONTROL_DESIGN.md` (Joe-reserved).
- D-071 (audit precedes dependent design); D-069 (canonical-document rule);
  D-065 (honest scope). Sibling arc audits: `tasks/M6_BOOTSTRAP_CLIENT_AUDIT.md`,
  `tasks/M6_AUTH_MODULE_REGISTRY_AUDIT.md`, `tasks/M6_PROTOCOL_AUDIT_LOG_AUDIT.md`.

---

*End of audit (audit phase). Design + implementation are the subsequent arc steps.*
