# M10.2 — Tier-1 Reference Auth Module (`xgen-auth-module`) — Phase-0 Framing Brief
> **Status**: COMPLETED  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-13  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. Purpose & lineage

Second M10 sub-arc, and the **first with a real new binary** — so unlike M10.1 (which rode the M10-audit grounding),
M10.2 warrants a **genuine D-071 Phase-0 audit**. This brief locks scope + the two calls (M10-A-02, M10-A-06)
so Clair's audit builds on a fixed frame. Frame inputs: the M10 audit (`tasks/M10_AUTH_MODULE_AUDIT.md` v1.1)
and the M10.1 descriptor (`claims.extra` `module_kind`/`module_policy`, shipped `01ea770`). No code until the
M10.2 design is Joe-locked.

## 1. Scope

**Goal.** Ship `xgen-auth-module` — a separate binary with its own keypair that **issues** Tier-1
`TrustAssertion`s (signs them as itself), so an identity can present a real (non-synthetic) module-signed
assertion at registration and drive the live 7-check `validate_assertion` end-to-end against a
registry-trusted issuer. This turns the shipped foundation from *dormant-but-correct* into *demonstrated*, and
proves the module architecture is genuinely pluggable (Fork 1: a demonstrator over the hardcoded baseline,
floor untouched).

**In:** the binary (keypair + Tier-1 assertion issuance + signing); the **registry → policy wiring** (Call 1b);
the config `trusted_auth_modules` → registry bootstrap-seed; an end-to-end witness (module issues → identity
registers → node validates against the registry-trusted issuer → accept; revoke → reject).

**Out (explicit):** M10.3 the parameterized T2–T4 mock + dormant-tier-path activation; M10.4 MP-F13; a
**module-presented** registration manifest/handshake (M10-A-06 → flagged future, not built); erasure
*enforcement* (the `module_policy.erasability` consumer stays D3-gated — M10.1 landed only the field).

## 2. Locked calls (Joe, J-362)

**M10-A-02 (the spine) = (b) wire registry → policy.** `validate_assertion` consults `AuthModuleRegistry`
as the trust source; `register`/`revoke`/`accepted_tiers` become enforcement-bearing (closes the AMR-D1
"standalone, no runtime consumer" deferral). Rejected: (a) status-quo config seam — it leaves the two
disconnected surfaces and ships a `revoke` that doesn't revoke (a known footgun). **Config-seed sub-lock:**
`[node].trusted_auth_modules` is **not** deprecated and **not** a second live source — at startup it
**bootstrap-seeds the registry** (migration-free; existing config-based trust keeps working), and the gate
reads the registry. The seed-vs-CRUD precedence detail is a design-lock item (§4).

**M10-A-06 = operator-CRUD kept.** The trust decision stays operator-controlled (a module self-asserting trust
would break the `trusted_auth_modules` safety model). The binary is a pure **signer + issuance endpoint**; the
operator registers it via the existing 5 CRUD verbs. A module-presented manifest/handshake is a **flagged
future enhancement**, not M10.2.

## 3. What the D-071 Phase-0 audit must ground (Clair, to file:line)

1. **The issuance path** — how an identity requests + receives a module-signed Tier-1 assertion: challenge/
   response, out-of-band, or operator-mediated. What today's *synthetic test issuer* does (the thing the real
   binary replaces) and the exact `TrustAssertion` shape it must produce.
2. **The Call-1b trust-source seam** — exactly where `validate_assertion` reads its trusted-issuer set today
   (`app.rs:746` config → `AssertionPolicy.trusted_issuers`, `registration.rs:200`) and the minimal change to
   make it read `AuthModuleRegistry` (+ where the startup config-seed lands).
3. **`AuthModuleRecord`/`endpoint_url` semantics** — does the node ever *call* the module, or only verify the
   presented assertion's signature against the trusted issuer pubkey? (Determines whether the binary needs a
   live endpoint at all for M10.2, or is purely an offline signer.)
4. **The keypair/signing surface** the binary reuses (the `SignedPrimitive`/canonical-bytes signer; how a
   module identity/keypair is created + expressed — `AuthModuleXgid`, D-083).
5. **What "Tier-1 verification" means in code** — likely proof-of-key-possession only (no external KYC); ground
   it so the issuance logic is honest about what it attests.
6. **M10.1 descriptor population** — confirm the binary sets `module_kind: reference` + a `module_policy`
   (erasability) on the assertions it issues (the fields shipped in M10.1).

Honesty guardrails: respect the locked forks; if grounding contradicts a locked call (e.g. the registry can't
be a clean trust source without a wire/persist change, or `endpoint_url` implies the node must call the
module), surface it and re-lock — don't paper over (D-065). Enumerate by grepping symbol definitions (D-078).

## 4. Design-questions deferred to design-lock (surface at audit; Joe-call only if non-obvious)

- Issuance shape: challenge/response vs offline-signed-token vs operator-mediated (grounded by §3.1).
- Config-seed precedence: if both config and a CRUD record name an issuer, which wins / how reconciled at
  startup (idempotent re-seed).
- Whether `revoke` is a registry-state flag the gate reads (preferred) vs a config edit.
- Does the binary need a running endpoint in M10.2, or is offline issuance sufficient for the witness (§3.3).

## 5. Proposed plan & close-criterion sketch

Phase-0 audit → design (lock Call-1b wiring shape + issuance shape) → runbook → impl (the `xgen-auth-module`
bin + the registry→policy wiring + config-seed + the end-to-end witness) → close (Appendix F for any
operator-visible surface; ch2/ch4 reconcile if needed; matrix/findings flips — M10-A-02 + M10-A-06 resolved at
close). A green M10.2 ships: a real Tier-1 module issuing signed assertions; a node that validates them against
a **registry-trusted** issuer (config-seeded); `revoke` that actually rejects; an end-to-end RED-on-revert
witness. Production change across `xgen-core`/`xgen-node` (+ the new binary crate); behaviour change at the
validation gate (expected, unlike M10.1).

## 6. State

- **Status**: ACTIVE — the live frame the M10.2 D-071 Phase-0 audit picks up next.
- **Next-active**: Clair opens the M10.2 D-071 Phase-0 audit (§3 grounding) → design → Joe-lock → runbook.
- No DECISIONS change at open (M10.2 decisions arc-local, D-069). The M10.1 arc-local candidate
  ("module-policy lives in a signed `claims.extra` namespace") remains a candidate.
