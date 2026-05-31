# Auth-Module-Registry — Design (D-071 arc, design phase)
> **Status**: ACTIVE  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-31  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Arc position

Design phase of the **auth-module-registry** D-071 arc. Entry artifact is
`tasks/M6_AUTH_MODULE_REGISTRY_AUDIT.md` (audit phase, ACTIVE) — read it first;
this doc does not restate the evidence. Per D-069 the arc runs audit → design →
impl; per D-071 the audit precedes this design. Design decisions are locked below;
the impl runbook is the next artefact.

## Verdict carried in

**GAP IDENTIFIED — HIGH (whole-subsystem).** All 5 verbs (`list`, `register`,
`revoke`, `set-tiers`, `test`) operate on a registry of trusted Auth Modules that
does not exist. Load-bearing distinction: tier *verification logic* exists
(`auth/tiers.rs`) and `AuthModuleUntrusted`/3006 is forward-designed, but a
*registry of trusted modules* (records, endpoints, per-module tiers, probe) does
not — and the first does not imply the second.

## Prime invariant

Nothing currently consults such a registry. An **empty registry = today
byte-for-byte**; this arc is purely additive admin surface and changes zero
runtime behaviour until a consumer wires in. (Sibling to the FAC `require_approval
= false` / absent-policy invariants.) Mandatory regression intent: the existing
suite stays green with an empty registry.

## Locked decisions

| ID | Decision | Status | Rationale |
|---|---|---|---|
| AMR-D1 | **Standalone registry** — ship record + store + 5 verbs; registry-consultation wiring (registration steps 5–7 / Trust-Assertion-accept) is an **explicit deferral** to its future arc. | LOCKED 2026-05-31 | The would-be consumers are themselves unbuilt (steps 5–7 deferred; `TrustAssertion` ctor lands at Pass 2). Can't wire enforcement into a non-existent consumer. `3006` is the waiting landing pad. FAC-consistent (store-before-consumer). |
| AMR-D2 | **New `AuthModuleXgid` principal flavour (7th)** — key-derived principal URI `xgen://pubkey/ed25519:<key>`, self-certifying, sibling to `Node`/`Identity`. Expands the D-072 six-flavour set → Appendix J §J.2 update + graduates to a global **D-###** in DECISIONS.md. | LOCKED 2026-05-31 | The key *is* the identity (protocol philosophy + D-072). `module_id` cryptographically bound to the key; the eventual 3006 check key is key-bound. Principal (not hash) flavour — module identified by its key, recoverable. |
| AMR-D3 | **Derive-don't-store the public key** — `module_id: AuthModuleXgid` is the single source of truth; the `VerifyingKey` is recovered via `.pubkey()` (parse-fallible, like `NodeXgid`). **No separate `public_key` field.** | LOCKED 2026-05-31 | Avoids dual source-of-truth drift; mirrors `Node`/`Identity` which don't store the key alongside the id. Constructed via `from_pubkey` (infallible) at register time → stored URI always valid. |

A2-D1 (block-only revoke) and A2-D2 (ad-hoc test) remain locked in
`docs/xgen_node_admin_ops_design.md` §6.A2 — agenda inputs, not re-opened here.

## Target shapes (design intent — exact field/API names pinned at runbook checkpoint #1)

**`AuthModuleXgid`** (`xgen-common/src/xgid/flavours.rs`): seventh `declare_flavour!`
entry, principal-flavour, with `from_pubkey(&VerifyingKey)` (infallible) +
`pubkey() -> Result<VerifyingKey, XgidDecodeError>` (sibling to `NodeXgid` /
`IdentityXgid`, same `principal_uri` / `principal_decode` path). Appendix J §J.2
six → seven.

**`AuthModuleRecord`** (`xgen-core`, sibling in shape to `FederationRelationship`):
- `module_id: AuthModuleXgid` — identity + key source (AMR-D2/D3).
- `endpoint_url: String` — where `auth-module test` probes.
- `accepted_tiers: Vec<AuthTier>` — tiers this module may issue (set by `set-tiers`).
- `registered_at: <timestamp>`.
- `revoked: bool` + `revoked_at: Option<...>` — block-only revoke (A2-D1).

**`AuthModuleRegistry`** store (`xgen-core`, sibling to `federation_policy.rs`):
`modules: HashMap<AuthModuleXgid, AuthModuleRecord>` (`Borrow<str>` from the flavour
macro enables string-keyed lookup). API: `new` / `register` (insert-or-replace) /
`revoke` (mark untrusted, retain) / `set_tiers` / `get` / `all` (list) / `len` /
`is_empty` / `save` / `load`. Reuses `RegistryError`. On-disk:
`xgen-node_auth_modules.json` (D-035 convention; sibling naming to
`xgen-node_federation_policy.json`).

**Probe** (`auth-module test`, A2-D2 ad-hoc): challenge/response to `endpoint_url`
reporting reachability + response time + reported tiers. Message shape + timeout +
"reachable" definition locked at runbook checkpoint #2. READ, not audited.

## Verb backing (post-design)

| Verb | Class | Backing after this arc |
|---|---|---|
| `auth-module list` | READ | `registry.all()` |
| `auth-module register` | WRITE | `register(record)` from `--endpoint` + `--pubkey` (→ `module_id`) + `--tiers` |
| `auth-module revoke` | DESTRUCTIVE | `revoke(module_id)` — mark untrusted, retain (A2-D1) |
| `auth-module set-tiers` | WRITE | `set_tiers(module_id, tiers)` |
| `auth-module test` | READ | probe stored `endpoint_url` (A2-D2) |

## Explicit deferral (AMR-D1 boundary)

Out of this arc: connecting the registry to the registration pipeline (steps 5–7)
and to Trust-Assertion acceptance so `AuthModuleUntrusted`/3006 fires against the
registry (per-module accepted-tiers enforcement). Lands when the
registration-pipeline / `TrustAssertion` arc is built; the registry's `get` +
`accepted_tiers` + `revoked` are the ready consultation points.

## Implementation roadmap (→ runbook)

5 Clair commits, 2 checkpoints:
1. **C1** `AuthModuleXgid` flavour (`xgen-common`) + Appendix J §J.2 six→seven +
   AMR-D2 → global D-### in DECISIONS.md. *Only commit touching the protocol-identity
   model; cross-crate prerequisite.*
2. **C2** `AuthModuleRecord` + `AuthModuleRegistry` store (`xgen-core`), no wiring.
   *Checkpoint #1: record field set + API names.*
3. **C3** CRUD verbs `list`/`register`/`revoke`/`set-tiers` in `admin_ops` + clap +
   pipe + live-store `AdminContext` threading.
4. **C4** `auth-module test` ad-hoc probe (A2-D2). *Checkpoint #2: probe message
   shape + timeout + "reachable".*
5. **C5** close (doc-only): §6.A2 SHIPPED, audit A2 → SHIPPED, design + runbook →
   COMPLETED.

## Decisions log

| ID | Decision | Status | Rationale |
|---|---|---|---|
| AMR-D1 | Standalone registry; consultation wiring deferred | LOCKED 2026-05-31 | consumers unbuilt; store-before-consumer |
| AMR-D2 | New `AuthModuleXgid` principal flavour (7th) | LOCKED 2026-05-31 | key *is* identity; D-072; → Appendix J + global D-### |
| AMR-D3 | Derive-don't-store public key | LOCKED 2026-05-31 | single source of truth; mirrors Node/Identity |

Arc-local IDs (`AMR-D#`) live in this doc per D-069. AMR-D2 graduates to a global
`D-###` in DECISIONS.md at C1 (it changes the protocol-identity flavour set).

## Cross-refs

- Audit (entry artifact): `tasks/M6_AUTH_MODULE_REGISTRY_AUDIT.md`.
- `docs/xgen_node_admin_ops_design.md` §6.A2 (A2-D1/A2-D2) + Appendix K.2.5.
- Spec Ch3 §3.11; `xgen-core/src/identity/registration.rs` (steps 5–7, error 3006);
  `xgen-core/src/auth/tiers.rs` (verification logic that exists);
  `xgen-common/src/xgid/flavours.rs` (the six-flavour set, D-072 / Appendix J §J.2).
- `tasks/M6_BACKING_AUDIT.md` A2 row. D-071 / D-069 / D-072 / D-065. Sibling docs:
  `tasks/M6_FEDERATION_ADMIN_CONTROL_DESIGN.md`, `tasks/M6_BOOTSTRAP_CLIENT_DESIGN.md`,
  `tasks/M6_PROTOCOL_AUDIT_LOG_DESIGN.md`.

---

*Design phase complete (decisions locked). Impl runbook is the next artefact.*
