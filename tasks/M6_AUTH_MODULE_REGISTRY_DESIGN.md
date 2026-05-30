# Auth-Module-Registry — Design (D-071 arc, design phase)
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

Design phase of the **auth-module-registry** D-071 arc. Entry artifact is
`tasks/M6_AUTH_MODULE_REGISTRY_AUDIT.md` (audit phase, ACTIVE) — read it first; this
stub does not restate the evidence. Per D-069 the arc runs audit → design → impl;
per D-071 the audit precedes this design.

**This is a stub.** It opens the design doc and frames the decision space. No
design call is made here — those are Joe's, recorded below as they lock.

## Verdict carried in

**GAP IDENTIFIED — HIGH (whole-subsystem).** All 5 verbs (`list`, `register`,
`revoke`, `set-tiers`, `test`) operate on a registry of trusted Auth Modules that
does not exist. The load-bearing distinction: tier *verification logic* exists
(`auth/tiers.rs`) and `AuthModuleUntrusted`/3006 is forward-designed, but a
*registry of trusted modules* (records, endpoints, per-module tiers, probe) does
not — and the first does not imply the second.

## Design agenda (what this phase must produce)

From the audit's "what the design phase must build" — the targets, not the design:

1. `AuthModuleRecord` — module_id, endpoint URL, public key, accepted_tiers,
   registered_at, revoked + revoked_at (sibling in shape to `IdentityRecord` /
   `FederationRelationship`).
2. An Auth Module registry store — register/revoke/set-tiers/get/list + save/load
   (D-035), analogous to `IdentityRegistry` / `FederationRegistry`.
3. Wire the trust check — connect `AuthModuleUntrusted` (3006) + registration
   steps 5–7 to consult the registry at Trust-Assertion-accept time.
4. A module-probe mechanism for `auth-module test` (A2-D2: ad-hoc in v1).
5. Block-only revoke semantics (A2-D1, already locked) — mark untrusted; existing
   assertions age out via TTL; no retroactive cascade.
6. The 5 verb implementations in `admin_ops::*` once the above exist.

## Open design decisions

- **AMR-D1 — standalone registry vs co-design with the registration pipeline?
  [OPEN]** (the audit's named scope decision.) The trust-check enforcement (step 3
  above) is where the registry meets the deferred registration steps 5–7 (Ch3
  §3.11). Choose: ship the admin-surface registry standalone, or co-design it with
  the registration-pipeline wiring.
- **AMR-D2 — module_id derivation? [OPEN, candidate]** Hash-URI of the module key
  (audit's suggested shape) vs operator-assigned id — affects record identity +
  the 3006 check key.

(AMR-D2 is surfaced from the agenda; A2-D1 block-only revoke and A2-D2 ad-hoc test
are already locked in §6.A2 and are agenda items, not open. AMR-D1 is the live call.)

## Decisions log

| ID | Decision | Status | Rationale |
|---|---|---|---|
| — | (none yet — design not started) | — | — |

Arc-local IDs (`AMR-D#`) live in this doc per D-069; a call graduates to a global
`D-###` in DECISIONS.md only when locked. (A2-D1/A2-D2 remain in §6.A2.)

## Cross-refs

- Audit (entry artifact): `tasks/M6_AUTH_MODULE_REGISTRY_AUDIT.md`.
- `docs/xgen_node_admin_ops_design.md` §6.A2 (A2-D1/A2-D2) + Appendix K.2.5.
- Spec Ch3 §3.11; `xgen-core/src/identity/registration.rs` (steps 5–7, error 3006);
  `xgen-core/src/auth/tiers.rs` (the verification logic that exists).
- `tasks/M6_BACKING_AUDIT.md` A2 row. D-071 / D-069 / D-065. Sibling stubs:
  `tasks/M6_FEDERATION_ADMIN_CONTROL_DESIGN.md`, `tasks/M6_BOOTSTRAP_CLIENT_DESIGN.md`,
  `tasks/M6_PROTOCOL_AUDIT_LOG_DESIGN.md`.

---

*Stub. Design decisions await Joe; this is the decision scaffold, not the design.*
