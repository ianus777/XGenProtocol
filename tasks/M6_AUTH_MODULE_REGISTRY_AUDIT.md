# Auth-Module-Registry — Backing Audit (D-071 arc, audit phase)
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

The **auth-module-registry** arc is one of the four post-M6 D-071 subsystem arcs
(`tasks/M6_BACKING_AUDIT.md`). Per D-071 each arc opens with a backing audit — a
read-only, evidence-cited pass mapping the deferred verbs to what exists, so the
design phase starts from reality. Audit phase only; gaps are routed, not designed.
Verified against the live tree on 2026-05-30; absences grep-confirmed.

The crux of this audit is a precise distinction: **tier *verification logic*
exists; a *registry of trusted Auth Modules* does not.** The first does not imply
the second.

## Scope — the deferred verbs

A2 (Phase 8) shipped **zero** verbs in M6. All **5** route here:

`auth-module list` · `auth-module register` · `auth-module revoke` ·
`auth-module set-tiers` · `auth-module test`

Design source: `docs/xgen_node_admin_ops_design.md` §6.A2 + Appendix K.2.5.
Note: §6.A2's A2-D1 lock had argued A2 "ships in M6" by removing the *cascade*
deferral — but cascade was never the blocker; the **registry's absence** is. The
J-156/J-157 backing audit corrected this: A2 ships nothing in M6.

## What EXISTS (verified)

- **Tier model + verification** (`xgen-core/src/auth/tiers.rs`) — real:
  - `AuthTier` (Tier1–4, `from_u32`/`as_u32`/`ttl_days`).
  - `Tier2Claims` / `Tier3Claims` / `Tier4Claims` (the per-tier claim shapes).
  - `verify_tier_assertion(assertion_tier, space_auth_tier)` and
    `verify_assertion_ttl(...)` — the logic that *checks* a Trust Assertion's tier
    and expiry.
  - `AuthError` (`TierMismatch` / `AssertionExpired` / `UnknownTier`).
- **The trust-check error** `AuthModuleUntrusted` (code **3006**) in
  `xgen-core/src/identity/registration.rs` — defined and wired into the error
  table. It is **forward-designed**: the registration pipeline's Auth-Module steps
  (5–7) are explicitly deferred ("Phase 2 (Auth Module implementation)"), so the
  error exists but the check that would raise it against a registry is unbuilt.
- **The flavour-wrapper note** (`xgen-common/src/xgid/flavours.rs`) — states the
  `TrustAssertion` constructor "lands at Pass 2 alongside the auth-module surfaces
  it consumes — Pass 2 owns those by scope." Confirms the auth-module surfaces are
  not yet built.

## What is ABSENT (the gap, verified)

- **No Auth Module registry.** Grep for `AuthModuleRegistry` / `AuthModuleRecord`
  across the repo returns **no Rust source matches** (verified). Grep for
  `trusted_module|auth_module|AuthModule` matches **only**
  `xgen-core/src/identity/registration.rs` — i.e. the `AuthModuleUntrusted` error,
  not a registry (verified). There is no struct/store holding *which Auth Modules
  this Node trusts*.
- **No per-module record** — no module_id, endpoint URL, public key, accepted
  tiers, or revoked flag stored anywhere.
- **No module endpoint / probe mechanism** — `auth-module test` has no endpoint to
  reach; `auth-module register` has no store to write a URL into.

**The distinction, stated precisely:** verifying *a claim's tier number against a
Space's required tier* (which `verify_tier_assertion` does) is not the same as
maintaining *a registry of which external Auth Modules are trusted to issue such
claims* (which does not exist). All 5 verbs operate on the latter.

## Per-verb backing

| Verb | Class | Backing | Evidence |
|---|---|---|---|
| `auth-module list` | READ | **ABSENT** | no registry to enumerate |
| `auth-module register` | WRITE | **ABSENT** | no store to write a module record to |
| `auth-module revoke` | DESTRUCTIVE | **ABSENT** | no registry entry to mark untrusted |
| `auth-module set-tiers` | WRITE | **ABSENT** | no per-module record to hold tiers |
| `auth-module test` | READ | **ABSENT** | no module endpoint stored to probe |

## Verdict

**GAP IDENTIFIED — HIGH (whole-subsystem).** All 5 verbs are absent-backed: the
registry, module records, endpoint store, and probe mechanism do not exist. The
existence of tier-verification logic and the `AuthModuleUntrusted` error is a
**forward-designed surface**, not backing — it confirms the *intent* without
providing the *store*. M6 backing-map assumption is **confirmed**; the
verification-logic-exists / registry-absent distinction is the load-bearing point
for the design phase.

## What the design phase must build (inputs to the design arc — NOT the design)

1. **`AuthModuleRecord`** — module_id (hash-URI of the module key), endpoint URL,
   public key, accepted_tiers, registered_at, revoked + revoked_at. (Sibling in
   shape to `IdentityRecord` / `FederationRelationship`.)
2. **An Auth Module registry store** — register / revoke / set-tiers / get / list
   + save/load (D-035 file convention), analogous to `IdentityRegistry` and
   `FederationRegistry`.
3. **Wire the trust check** — connect the existing `AuthModuleUntrusted` (3006)
   and the deferred registration steps 5–7 to consult the registry at
   Trust-Assertion-accept time (per-module accepted-tiers enforcement).
4. **A module-probe mechanism** for `auth-module test` — a challenge/response to
   the module endpoint reporting reachability + response time + reported tiers
   (A2-D2: ad-hoc in v1; a formal health-check message is a later rung).
5. **Block-only revoke semantics** (A2-D1, design already locked) — mark the
   module untrusted; existing Trust Assertions age out via their TTL; no
   retroactive invalidation/cascade.
6. **The 5 verb implementations** in `admin_ops::*` once the above exist.

A scope note for the design phase: this arc intersects the broader **Auth Module
reference implementation** (Ch3 §3.11, the registration pipeline's deferred steps
5–7). The design phase should decide whether the registry ships standalone (admin
surface only) or co-designs with the registration-pipeline wiring, since the
trust-check enforcement is the point where the two meet.

## Carry-overs & cross-refs

- `docs/xgen_node_admin_ops_design.md` §6.A2 (verb specs, A2-D1 block-only,
  A2-D2 ad-hoc test) + Appendix K.2.5.
- Spec Ch3 §3.11 (Auth Module tier model); `xgen-core/src/identity/registration.rs`
  (deferred steps 5–7, error 3006).
- `tasks/M6_BACKING_AUDIT.md` A2 row. Future design stub:
  `tasks/M6_AUTH_MODULE_REGISTRY_DESIGN.md` (Joe-reserved).
- D-071 / D-069 / D-065. Sibling arc audits:
  `tasks/M6_FEDERATION_ADMIN_CONTROL_AUDIT.md`, `tasks/M6_BOOTSTRAP_CLIENT_AUDIT.md`,
  `tasks/M6_PROTOCOL_AUDIT_LOG_AUDIT.md`.

---

*End of audit (audit phase). Design + implementation are the subsequent arc steps.*
