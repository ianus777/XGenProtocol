# Federation Policy 2b — Per-Peer Policy & Enforcement — Design (D-071 arc)
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

Design phase of **federation-admin-control, sub-arc 2b (policy)** — the policy half of
the federation-admin-control arc, **split out at 2a design open (J-171)**. Entry
artifact: `tasks/M6_FEDERATION_ADMIN_CONTROL_AUDIT.md` (shared with 2a). **PENDING** —
opens after 2a (approval/queue) ships. This is a stub: it frames the decision space,
makes no design call.

## Verdict carried in

**GAP IDENTIFIED — HIGH.** No `FederationPolicy` type, no per-peer allow/deny/rate-limit
store, no consult site in the push path. (Per the audit; verified 2026-05-30.)

## Scope (2b)

`federation set-policy` · `federation show-policy` + a per-peer policy store +
the enforcement site. Builds on 2a's `FederationState` + registry.

## Open design decisions

- **FAC-D3 — policy enforcement site(s). [OPEN]** (the audit's named decision.)
  Push-path only (`apply_federation_push`) vs also the inbound F-3 gate. Drives where
  the policy consult lives and what a policy can actually block.
- **FAC-D4 — policy shape. [OPEN, candidate]** mode (allow/deny) + allowed_spaces +
  rate_limit, vs a richer rule set. Pin minimally first.

## What the design phase must build (from the audit)

A `FederationPolicy` per peer (mode, allowed_spaces, rate_limit); a store (sibling to
the registry; D-035 path); a consult site (FAC-D3); the two verbs in `admin_ops::*`.

## Decisions log

| ID | Decision | Status | Rationale |
|---|---|---|---|
| FAC-D3 | Enforcement site | **OPEN** | — |
| FAC-D4 | Policy shape | **OPEN (candidate)** | — |

## Cross-refs

- Audit: `tasks/M6_FEDERATION_ADMIN_CONTROL_AUDIT.md`. Sibling 2a:
  `tasks/M6_FEDERATION_ADMIN_CONTROL_DESIGN.md` (ACTIVE).
- `docs/xgen_node_admin_ops_design.md` §6.A1. `apply_federation_push` (push path),
  the F-3 inbound gate. D-071 / D-069 / D-065.

---

*Stub. Opens after 2a ships; decisions await Joe.*
