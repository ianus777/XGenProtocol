# M10 — Auth Module Reference Set — Phase-0 Framing Brief
> **Status**: COMPLETED  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-12  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. Purpose

This is the **framing brief** that opens M10 — it captures the milestone scope and the three Joe-locked
forks so the D-071 Phase-0 audit (Clair) and the per-sub-arc designs build on a fixed frame, not on the
spec alone. It is **not** the audit and **not** a design: it locks *what M10 is and is not*, decomposes the
work into sub-arcs, and lists what the audit must ground to file:line. No code. (Brief precedent:
`tasks/MP_R3_CAPSTONE_PHASE0_BRIEF.md`.)

M10 is the certified next-active after the Multiparty-tests milestone close (J-356) and the Round-2
checkpoint GO (J-357). Chain: **M10 → M11 → M12 → Round-2 final pre-UI gate → UI → Streams**.

---

## 1. What M10 is (grounded: ROADMAP M10 entry + J-357)

**Goal.** Ship the project's first proper Auth Module instance, proving the module architecture is genuinely
pluggable and tiered auth is swappable end-to-end — replacing today's dormant Local-Node-bypass baseline as
the *demonstrated* path, not as the floor (see Fork 1). It is a **UI gate**.

**Form (locked Joe, 2026-06-04).** The autonomous `system`-mode module is the proper form: a **separate
binary `xgen-auth-module`** with its own keypair, signing `TrustAssertion`s as itself, running alongside a
Node — not an in-node stub.

**Two build artifacts:**
1. A real **Tier-1 reference module** — exercises the full 7-check `validate_assertion` path with a *real*
   external issuer instead of today's synthetic test issuer.
2. **One parameterized T2–T4 mock** — tier carried as a config/claim parameter, *not* four separate modules.
   The genuine per-tier difference is verification rigor + legal accountability (which a mock by definition
   skips); what varies on the wire is only claim shape + TTL + tier integer. The core team is not an
   institution and cannot legally attest higher-tier identity, so the mock is the **reference template an
   institution forks** and replaces with its accountable version. Self-labels `mock`/test in manifest +
   assertions; honoured only via the explicit `trusted_auth_modules` gate — real *software*, never a
   deployable *trust* anchor.

**Foundation already shipped (the interface it plugs into):** `AuthModuleRegistry` + the 5 `auth-module`
CRUD/probe verbs + `AuthModuleXgid` (D-083); `TrustAssertion` SignedPrimitive + the 7-check
`validate_assertion` wired into registration, gated on `trusted_auth_modules` (PG-03 / Arc E).

---

## 2. The three locked forks (Joe, 2026-06-12)

**Fork 1 — Baseline vs module (the spine; gates the whole shape). LOCKED.**
The hardcoded crypto-identity baseline **stays the floor**; the Tier-1 module is a **demonstrator layered
over it**, not a replacement of the Local-Node bypass. M10 proves the interface works with a real autonomous
signer; it does not remove or rewrite the baseline path. This resolves the on-record tension between
`PRIVILEGE_MODEL_DESIGN` ("baseline is built-in, not a module") and ch2/ch4 ("Tier-1 reference module") —
the module demonstrates *over* the hardcoded baseline.

**Fork 2 — MP-F13 (home-node discovery). LOCKED = named M10 sub-arc, depth deferred.**
MP-F13 (production identity→home-node discovery, J-278 / F1B-D5 family) is the heaviest inbound and is not
strictly auth-module work. It is a **named M10 sub-arc** with its depth decided at its own mini-Phase-0 —
**not silently absorbed**. The deferred row **MP-C-16** (live migration) re-runs only after MP-F13's
disposition lands; it stays deferred until then.

**Fork 3 — GDPR identity-orphan depth. LOCKED = hook only.**
M10 scopes the **descriptor + tier-gate hook** (the AI-D8 module-policy descriptor on the TrustAssertion +
the "T4 refuses erasure" tier-gate). Heavy erasure mechanics are **flagged**, not built — the content-erasure
half of PG-02 stays D3-gated (gated on PG-05 real crypto). M10 lands the identity-orphan *surface* the module
owns, not the full erasure engine.

---

## 3. Sub-arc decomposition (proposed phase order)

Each sub-arc runs its own full D-071 cycle (Phase-0 audit → design → Joe-lock → runbook → Clair impl →
doc-bridge → close) with Appendix F / Appendix C / Appendix I deliverables as applicable, and a RED-on-revert
witness where it ships enforcement.

- **M10.1 — wire-band reconciliation + module-policy descriptor.**
  Resolve **RC-F-01 / MP-F2-followon**: ch3 defines 3010/3011 twice (§3.6.5 + code = assertion_identity_mismatch
  / assertion_claims_insufficient, Arc E PG-03; §3.11.7 + the L3829 reservation = auth_tier_insufficient /
  kyc_verification_pending). M10 owns the **3010–3016 auth-module band** — pick which family keeps 3010/3011,
  renumber the other, and map the 7 unmapped MP-F2-followon codes. Land the **AI-D8 module-policy descriptor**
  (erasability/retention as the first forward-extensible member + the §8 open-doors principle) on the
  TrustAssertion, since the module populates it. Lands first because the module's assertions *use* these codes.

- **M10.2 — Tier-1 reference module binary (`xgen-auth-module`).** The load-bearing artifact: own keypair,
  signs assertions, registers via the existing CRUD verbs + `trusted_auth_modules`, drives the real 7-check
  path end-to-end.

- **M10.3 — parameterized T2–T4 mock + dormant-tier-path activation.** Per-tier claims/TTLs, the Arc-E Thread
  participation gate at T2–T4, and the D-088 erasure tier-gate ("T4 refuses erasure" needs a T4 identity to
  refuse). Mock safety per §1.

- **M10.4 — MP-F13 sub-arc (Fork 2).** Own mini-Phase-0 first; depth decided there.

- **M10.5 — fold MP-F6 (low-sev swallowed apply-error, runtime.rs:691 breadcrumb) + re-run the deferred
  multiparty rows MP-C-06 (re-home) and MP-C-16 (migration; gated on M10.4).**

- **M10 close.** Appendix C (Space/Identity) + Appendix I + Appendix F reconcile; gap-audit register flips;
  DECISIONS promotions (candidates only, none pre-decided); ROADMAP/CLAUDE/JOURNAL atomic close.

---

## 4. Re-homed findings / rows M10 owns (from J-356 / J-357)

| Item | What | M10 home |
|---|---|---|
| RC-F-01 / MP-F2-followon | 3010/3011 double-definition + 7 unmapped wire codes | M10.1 |
| AI-D8 descriptor | module-policy (erasability/retention) on TrustAssertion | M10.1 |
| D-088 identity-orphan | PG-05-independent erasure half — **hook only** (Fork 3) | M10.3 (gate) / flagged |
| MP-F6 | swallowed apply-error (low-sev) | M10.5 (fold) |
| MP-F13 | home-node discovery (J-278) — **named sub-arc** (Fork 2) | M10.4 |
| MP-C-06 | re-home (keypair-relocation + re-home-notify) — deferred row | M10.5 (re-run) |
| MP-C-16 | live migration — deferred row, gated on MP-F13 | M10.5 (re-run, gated) |

---

## 5. What the D-071 Phase-0 audit (Clair) must ground to file:line

1. `AuthModuleRegistry` + the 5 CRUD/probe verbs + `AuthModuleXgid` (D-083) — current surface, what a real
   module must implement to register and be trusted.
2. The 7-check `validate_assertion` + `accept_registration` wiring + `trusted_auth_modules` gate + the
   synthetic test issuer it exercises today (the thing the real T1 module replaces).
3. The **3010–3016 wire band**: every current definition site (ch3 §3.6.5, §3.11.7, the L3829 reservation,
   `registration.rs:120-122`, the `3030 tier_mismatch` usage) — the authoritative map for the RC-F-01 renumber.
4. The TrustAssertion struct + `TrustClaims` (Arc E) — the extension point for the AI-D8 descriptor.
5. The Local-Node bypass / hardcoded baseline path (Fork 1) — confirm the demonstrator layers cleanly without
   touching the floor.
6. The D-088 erasure tier-gate touch-points — the minimal hook surface for Fork 3 (where "T4 refuses erasure"
   would attach), with heavy mechanics out of scope.
7. The manifest/assertion `mock` self-labelling surface — how mock-safety is expressed and enforced via the
   `trusted_auth_modules` gate.

---

## 6. Close-criterion sketch (refined at design-lock, not binding here)

A green M10 ships: a real T1 reference module driving the 7-check path end-to-end with a non-synthetic issuer;
one parameterized mock exercising every dormant T2–T4 path (claims/TTLs, Thread gate, erasure tier-gate); the
3010–3016 band reconciled and the AI-D8 descriptor landed; MP-F6 folded; MP-C-06 + MP-C-16 re-run (MP-C-16
gated on the MP-F13 sub-arc disposition). The identity-orphan erasure mechanics remain a flagged, D3-gated
boundary — not a defect, a named horizon.

---

## 7. State

- **Status**: ACTIVE — this brief is the live frame the M10.1 D-071 Phase-0 audit picks up next.
- **Next-active**: Clair opens the M10 D-071 Phase-0 audit (§5 grounding) → design → Joe-lock → runbook.
- No DECISIONS change at open (M10 sub-arc decisions are arc-local, D-069; promotions at close).
- All philosophy-level sub-questions (renumber direction, MP-F13 depth, descriptor field set) belong to the
  relevant sub-arc's design-lock, not to this brief.
