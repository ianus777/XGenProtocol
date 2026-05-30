# Federation-Admin-Control 2a — Approval & Queue — Design (D-071 arc)
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

Design phase of **federation-admin-control, sub-arc 2a (approval & queue)** — the
second D-071 arc, **split into 2a/2b at design open (J-171)**. Entry artifact:
`tasks/M6_FEDERATION_ADMIN_CONTROL_AUDIT.md` (covers all 5 verbs; both sub-arcs cite
it). Per D-069: audit → design → impl. **Decisions locked J-171 (2026-05-30).**

**The split.** The audit's 5 verbs presuppose two separable subsystems:
- **2a (this doc) — approval/queue:** `federation accept` · `federation reject` ·
  `federation initiate` + the pending-request queue + relationship state model.
- **2b (`tasks/M6_FEDERATION_POLICY_DESIGN.md`, PENDING) — policy:**
  `federation set-policy` · `federation show-policy` + per-peer policy store +
  enforcement site (FAC-D3). Scheduled after 2a ships.

Decision IDs stay `FAC-D#` across both (shared audit lineage): D1/D1a/D2 here, D3 in 2b.

## Verdict carried in

**GAP IDENTIFIED — HIGH (whole-subsystem).** Federation **auto-establishes**: a peer
completing a valid handshake (signature + compatible caps/version) transitions
straight to ACTIVE and is persisted. There is no pending state, no approval queue, no
admin pause point. The handshake *already has* a `Reject` message with error codes
(`handshake.rs`, codes 2001/2002 for caps/version) — the protocol can already refuse
a peer; today it never refuses on operator policy.

## Decisions locked (J-171)

- **FAC-D1 — federation approval is opt-in, configurable, default-off. [LOCKED:
  Option B]** A Node config flag (working name `federation.require_approval`,
  default **`false`**). Off → today's auto-establish, **byte-for-byte unchanged**
  (zero regression; the J-156 `list`/`defederate` subset stays correct). On →
  inbound federation requests do **not** auto-establish; they enter the pending
  queue for admin `accept`/`reject`. Rationale: the only backward-compatible default;
  makes the gate a deliberate operator choice rather than an imposed posture; lets
  the queue + state model land without also requiring the policy store (2b). Chosen
  over A (always-on — a posture reversal that breaks unattended Nodes) and C
  (per-peer — needs the 2b policy store wired from day one). **Philosophy note:**
  XGen's identity-first stance could argue for A (approval as principle, not toggle);
  Joe chose B — the capability exists, the operator opts in.
- **FAC-D1a — pause-point behaviour: reject-with-retry, do NOT hold the socket.
  [LOCKED]** When `require_approval = true` and a peer's request is not yet approved,
  `run_receiving` does **not** block the connection awaiting a human. It records the
  request in the queue and sends the existing `Reject` message with a new
  **"approval pending, retry later"** error code (new code, e.g. 2003; pinned at
  runbook). The peer disconnects; after the operator `accept`s, the peer's normal
  reconnect/re-handshake (or an operator-initiated `initiate`) establishes the now-
  approved session. Rationale: never hold sockets open on unattended approval; the
  queue is the durable record, not a live connection.
- **FAC-D2 — relationship state lives as a field on `FederationRelationship`.
  [LOCKED]** Add a `FederationState` enum field (`Pending` / `Active` / `Rejected` /
  `Revoked`) to `FederationRelationship` (`registry.rs:40-54`). Existing persisted
  JSON records (no `state` field) **load as `Active`** via serde default — the
  implicitly-active records the audit found stay active (backward-compatible). Chosen
  over a sibling pending-index: the discriminator belongs with the relationship it
  describes; one store, one load path. `accept` flips `Pending → Active` (+ upsert /
  schedule reconnect); `reject` flips `Pending → Rejected` (kept as a tombstone so a
  rejected peer isn't silently re-queued on retry — retention/expiry pinned at runbook).

## Scope (2a)

**In:** the `require_approval` config flag (FAC-D1); a pending-request queue
(persisted; peer_node_id, peer_url, received_at, expiry, request details); the pause
point in `run_receiving` / `handle_federation_incoming` (FAC-D1a, reject-with-retry);
the `FederationState` field + backward-compatible load (FAC-D2); the verbs
`federation accept` / `federation reject` / `federation initiate`.
**Out (→ 2b):** `set-policy` / `show-policy`, the per-peer policy store, enforcement
site (FAC-D3). **Out (general):** changing the default posture (B keeps default-off);
any change to the shipped `list`/`defederate` subset.

## `federation initiate` (2a)

Operator-triggered outbound handshake to a named peer (`run_initiating` already
exists; the reconnect scheduler is its current caller). `initiate` gives the operator
a manual "start federation with this peer URL now" path. With FAC-D1 off it
establishes as today; with it on, the *initiating* side is the operator's own choice,
so it auto-accepts locally (the operator initiated it) — the gate is for **inbound**
requests, not operator-initiated outbound. (Confirm this asymmetry at runbook: initiate
= operator intent = no self-approval needed.)

## What the runbook must build / pin (inputs to impl)

1. `FederationState` enum + serde-default-`Active` on `FederationRelationship`;
   registry methods to transition (`mark_pending` / `mark_rejected`, alongside the
   existing `mark_active`).
2. The pending-request queue store (persisted JSON, D-035 path convention; sibling to
   the registry) + add/remove/get/list.
3. `require_approval` config flag on `NodeConfig` (default false).
4. The `run_receiving` pause point (FAC-D1a): on `require_approval && !approved`,
   enqueue + send `Reject` with the new pending code (pin the code).
5. `admin_ops::federation_accept` / `_reject` / `_initiate` + clap variants + pipe
   arms; `accept` completes the relationship (state → Active, upsert, schedule
   reconnect); `reject` → Rejected tombstone; errors (peer-not-in-queue, etc.).
6. Reject-tombstone retention/expiry; queue-entry expiry.

## Decisions log

| ID | Decision | Status | Rationale (short) |
|---|---|---|---|
| FAC-D1 | Approval opt-in: configurable `require_approval`, default-off | **LOCKED** (J-171) | Only backward-compatible default; gate is an operator choice, not imposed |
| FAC-D1a | Pause-point = reject-with-retry, don't hold socket | **LOCKED** (J-171) | Never hold sockets on unattended approval; queue is the durable record |
| FAC-D2 | `FederationState` field on the relationship; existing records load Active | **LOCKED** (J-171) | Discriminator belongs with the relationship; one store/load; backward-compatible |
| FAC-D3 | Policy enforcement site | **→ 2b** | Belongs to the policy sub-arc |

Arc-local `FAC-D#` per D-069; graduate to a global `D-###` only if a project-wide
principle emerges.

## Cross-refs

- Audit (entry artifact): `tasks/M6_FEDERATION_ADMIN_CONTROL_AUDIT.md`.
- Sub-arc 2b: `tasks/M6_FEDERATION_POLICY_DESIGN.md` (PENDING).
- `docs/xgen_node_admin_ops_design.md` §6.A1 + Appendix K.2.4.
- Code: `xgen-core/src/federation/handshake.rs` (`run_receiving`/`run_initiating`,
  `Reject` codes 2001/2002), `xgen-core/src/federation/registry.rs`
  (`FederationRelationship` :40-54, `mark_active`/`save`/`load`),
  `xgen-node/src/reconnect.rs` (scheduler), `xgen-node/src/admin_ops.rs` (A1 subset).
- `tasks/M6_BACKING_AUDIT.md` A1 row. D-071 / D-069 / D-065.

---

*Design decisions locked (J-171). Next: 2a implementation runbook. 2b (policy) opens after 2a ships.*
