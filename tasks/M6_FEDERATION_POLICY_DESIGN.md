# Federation Policy 2b — Per-Peer Policy & Enforcement — Design (D-071 arc)
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

Design phase of **federation-admin-control, sub-arc 2b (policy)** — the policy half of
the federation-admin-control arc, split out at 2a design open (J-171). Entry artifact:
`tasks/M6_FEDERATION_ADMIN_CONTROL_AUDIT.md` (shared with 2a). 2a (approval/queue) shipped
end-to-end and CLOSED at J-178; 2b opens on top of its `FederationState` + registry. This
doc records the **locked** FAC-D3/D4 decisions (Joe-lock, 2026-05-30); the implementation
runbook is the next artifact.

## Verdict carried in

**GAP IDENTIFIED — HIGH.** No `FederationPolicy` type, no per-peer allow/deny store, no
consult site in the push path or the inbound event path. (Per the audit; verified
2026-05-30.)

## Scope (2b)

`federation set-policy` (WRITE) · `federation show-policy` (READ) + a per-peer policy
store + the enforcement sites. Builds on 2a's `FederationState` + `FederationRegistry`.

## Locked decisions

### FAC-D4 — policy shape. **LOCKED.**

Minimal v1 shape:

```
FederationPolicy {
    mode: PolicyMode,            // Allow | Deny
    allowed_spaces: Option<Vec<…>>,  // None = all shared spaces; Some = narrow to subset
}
```

- **`rate_limit` DEFERRED** — it couples policy to the still-unbuilt federation-under-load
  measurement story (a known carry-over gap). Pin `mode` + `allowed_spaces` now; add
  rate-limiting as its own beat once load-measurement exists. Bundling it here would
  half-build two subsystems.

**Sub-decisions (locked):**

1. **Store location = sibling store, NOT a `FederationRelationship` field.** Mirrors 2a's
   `pending_queue.rs` (the audit's "sibling to the registry; D-035 path" hint). Keeps the
   relationship a clean handshake-derived record, and lets an operator **pre-deny** a peer
   before any handshake. Policy is operator-authored (operator lifecycle); state is
   protocol-derived (handshake lifecycle) — different lifecycles, different stores.
2. **`allowed_spaces` is restrictive-only.** Effective set = `relationship.shared_spaces ∩
   policy.allowed_spaces`. Policy can **narrow, never widen**. The protocol-derived
   `shared_spaces` stays authoritative; policy is a pure safety governor (a security
   control can deny, never grant). `allowed_spaces` element type mirrors `shared_spaces`
   (runbook confirms the exact type by code-trace).

### FAC-D3 — enforcement site(s). **LOCKED.**

**Both sites, one shared pure helper.** The two policy fields cut in opposite directions,
so a single-site design leaves one of them toothless:

- `mode: Deny` → teeth are **inbound** (drop the peer's events pre-apply; the relationship
  survives, unlike `defederate`). Also short-circuit **outbound** (stop pushing to a denied
  peer).
- `allowed_spaces` → **outbound** (don't leak out-of-set spaces to the peer) **and inbound**
  (don't apply the peer's out-of-set events).

**Helper:** a pure `policy_permits(peer, space_id) -> bool` (sibling to 2a's
`approval_gate_decision`; pure xgen-core fn, no-drift per D-067), **default-permit** when
no policy entry exists. Called at two sites:

- **Outbound:** `apply_federation_push` (exists today).
- **Inbound:** the inbound federated-event ingest path (the audit's "F-3 gate"). **Exact
  site is code-traced by the runbook, not assumed** — 2a proved the doc name ≠ the live
  site (D-078); the inbound location is the load-bearing find of step 2.

**Why not push-path-only** (the stub's narrower option): a `Deny` that blocks only outbound
is operator-surprising — the peer's events still arrive and apply. Deny must bite inbound to
mean anything. `defederate`/`reject` already cover "stop everything"; policy's job is the
granular in-between, which *requires* the inbound filter.

## Prime invariant (2b)

**Default policy (absent entry) = Allow + all spaces = today, byte-for-byte.** Sibling to
2a's `require_approval = false`. The consult helper returns *permit* for any peer without a
stored policy, so both enforcement sites are zero-regression by construction. A mandatory
explicit default-permit regression test lands with the enforcement commit (D-065).

## What the runbook must build

1. **Store** — `xgen-core/src/federation/federation_policy.rs` sibling store
   (`HashMap<NodeXgid, FederationPolicy>`, `set`/`get`/`remove`/`all`/`save`/`load`,
   default-absent = permit) + the `FederationPolicy` / `PolicyMode` types. Modeled on
   `pending_queue.rs`.
2. **Enforcement** — the `policy_permits` helper + both consult sites; the inbound-site
   code-trace is the D-078 beat. Default-permit regression test (prime invariant).
3. **Verbs** — `federation set-policy` (WRITE, audited) + `federation show-policy` (READ,
   not audited) in `admin_ops::*`; clap variants; pipe arms; the live policy-store `Arc`
   threaded `run_node → start_pipe_server → dispatch_line → dispatch_admin` (sibling to the
   2a queue threading).
4. **Close** — `docs/xgen_node_admin_ops_design.md` §6.A1 marks set-policy/show-policy
   SHIPPED; `tasks/M6_BACKING_AUDIT.md` A1 row; ROADMAP; this doc → COMPLETED.

## Proposed commit plan (runbook seed)

| # | Commit | Checkpoint |
|---|---|---|
| 1 | Store + `FederationPolicy`/`PolicyMode` types | #1 — D4 shape + store location |
| 2 | `policy_permits` helper + both consult sites + default-permit regression | #2 — D3 inbound site confirmed by code-trace |
| 3 | `set-policy` + `show-policy` verbs + threading | — |
| 4 | Close (doc-only) | — |

## Decisions log

| ID | Decision | Status | Rationale |
|---|---|---|---|
| FAC-D3 | Enforcement site(s) = both (outbound `apply_federation_push` + inbound ingest), one pure `policy_permits` helper, default-permit | **LOCKED** | the two fields cut opposite directions; Deny must bite inbound; defederate/reject cover "stop all", policy is the granular in-between |
| FAC-D4 | Policy shape = `{ mode: Allow\|Deny, allowed_spaces: Option<Vec> }`; rate_limit deferred; sibling store; allowed_spaces restrictive-only | **LOCKED** | minimal first; pre-deny needs a sibling store; policy narrows never widens; rate_limit couples to unbuilt load-measurement |

## Cross-refs

- Audit: `tasks/M6_FEDERATION_ADMIN_CONTROL_AUDIT.md`. Sibling 2a (COMPLETED):
  `tasks/M6_FEDERATION_ADMIN_CONTROL_DESIGN.md` + `..._IMPL.md`.
- `docs/xgen_node_admin_ops_design.md` §6.A1. `apply_federation_push` (outbound push path),
  the inbound F-3 ingest gate. 2a precedents: `federation/pending_queue.rs`,
  `approval_gate_decision`. D-071 / D-069 / D-065 / D-067 / D-078.

---

*Design phase: FAC-D3/D4 LOCKED (2026-05-30). Next artifact: the 2b implementation runbook.*
