# M10.3 — Parameterized T2–T4 Mock + Dormant-Tier-Path Activation — Phase-0 Framing Brief
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

Third M10 sub-arc, and the **heaviest** (three converging threads — the mock, `accepted_tiers` enforcement, and
dormant-tier-path activation). Warrants a **genuine D-071 Phase-0 audit**. This brief locks scope + the four
calls so Clair's audit builds on a fixed frame. Frame inputs: the M10 audit
(`tasks/M10_AUTH_MODULE_AUDIT.md` v1.2), the M10.1 descriptor (`claims.extra` `module_kind`/`module_policy`),
and the M10.2 binary + registry→policy live-read (shipped `e824844..8a6024c`, J-364). No code until the M10.3
design is Joe-locked.

## 1. Scope

**Goal.** Now that a real module can issue higher-tier assertions, exercise + witness every tier-conditional
path that has been dormant since Arc E. Three artifacts land together:
(a) the **parameterized T2–T4 mock** (the reference template an institution forks);
(b) **`accepted_tiers` enforcement** (the M10.2-A1 deferral comes due);
(c) **dormant-tier-path activation** (per-tier claims/TTLs, the Arc-E Thread participation tier-gate, the D-088
erasure tier-gate — issuance-side).

**In:** the mock issue path on `xgen-auth-module`; the per-issuer `accepted_tiers` check at the gate; the
dormant-tier witnesses; tier-appropriate `module_policy.erasability` issuance.

**Out (explicit, recorded boundaries):** MP-F13 → M10.4; MP-F6 fold + MP-C-06/MP-C-16 re-run → M10.5; **erasure
*enforcement*** (the "T4 refuses erasure" consumer) stays D3-gated — M10.3 issues + witnesses the *field*, not a
node that refuses erasure; a "refuse mock in production-mode" node behaviour (flagged future, not M10.3).

## 2. Locked calls (Joe, J-365)

**M10.3-C1 — mock form = extend the one binary.** The mock is a parameterized `--tier <N>` (N ∈ {2,3,4}) issue
path on the shipped `xgen-auth-module`, self-labelling `module_kind: mock`. This *is* the "reference template an
institution forks" (J-358) — they fork `xgen-auth-module` and swap mock issuance for real KYC. Reference-T1
stays the honest default; mock higher-tiers are the same code parameterized; different module instances = the
one binary run in different configs (own keypairs). No second crate.

**M10.3-C2 — `accepted_tiers` enforcement = a new explicit check.** M10.2's live-read already hands the gate
the issuer's `AuthModuleRegistry` record, so `assertion.tier ∈ issuer.accepted_tiers` is a clean addition. It is
**distinct** from the existing node-wide `tier ≥ required_tier` (`registration.rs:230`): two different gates —
"is this tier high enough for this Space" vs "is this issuer authorized to attest this tier". Witness: a
T2-scoped issuer's T3 assertion is rejected.

**M10.3-C3 — dormant-tier scope; erasure stays issuance-side.** Activate + witness the per-tier claims/TTLs and
the Arc-E Thread participation tier-gate (existing gates that only needed a higher-tier identity to fire). The
**D-088 erasure tier-gate is issuance-side only**: the mock issues tier-appropriate `module_policy.erasability`
(T1–T3 `erasable`, **T4 `retained`**), and M10.3 witnesses *the field*, **not** erasure-refusal enforcement —
that consumer stays D3-gated (Fork 3 = hook only, J-358). "T4 refuses erasure" is *expressed* in the assertion,
not yet *enforced* by a node.

**M10.3-C4 — `module_kind: mock` = expression-only.** No new node-side special-casing of `mock`. The operator's
explicit `trusted_auth_modules`/registry trust is the safety mechanism (a mock is real *software*, never a
deployable *trust* anchor); the label is informational. A "refuse mock in production-mode" node behaviour is a
flagged future, not M10.3.

## 3. What the D-071 Phase-0 audit must ground (Clair, to file:line)

1. **The `accepted_tiers` field + the live-read seam** — `module_registry.rs:59` (the per-module
   `accepted_tiers`); the M10.2 `trusted_issuers()` live-read + the gate's access to the issuer record at
   validate time; exactly where the new C2 check inserts in `validate_assertion` relative to the node-wide
   `required_tier` check (`registration.rs:230`).
2. **The node-wide `required_tier` check** — confirm it is genuinely distinct from C2 (different question) so the
   two gates compose without overlap or double-reject ambiguity.
3. **The Arc-E Thread participation tier-gate** — where tier conditions a Thread op today (the dormant gate C3
   activates); what a higher-tier identity changes.
4. **Per-tier claims/TTL handling** — how `TrustClaims` carries tier + TTL; what differs T1→T4 on the wire
   (claim shape + TTL + tier integer, per the M10 scope); the dormant paths that only fire above T1.
5. **The mock issue surface on `xgen-auth-module`** — the M10.2 `issue_tier1()` path; the minimal parameterization
   to a `--tier <N>` mock that self-labels `module_kind: mock` + sets tier-appropriate `erasability`.
6. **The D-088 erasure tier-gate touch-point** — confirm the issuance-side `erasability` is all M10.3 builds;
   ground where an enforcement consumer *would* attach (so the boundary is explicit) without building it.

Honesty guardrails: respect the locked forks; if grounding contradicts a locked call — e.g. `accepted_tiers`
can't be checked cleanly without a wire/persist change, or the Thread tier-gate needs more than a higher-tier
identity, or a dormant path is actually already dead/removed — surface it and re-lock (D-065). Enumerate by
grepping symbol definitions (D-078).

## 4. Design-questions deferred to design-lock (surface at audit; Joe-call only if non-obvious)

- The exact insert point + ordering of the C2 `accepted_tiers` check vs the node-wide `required_tier` check.
- The mock's per-tier claim/TTL defaults (what a T2/T3/T4 mock assertion carries) — grounded against Arc-E
  claim handling, not invented.
- Whether the `--tier` mock path is a new subcommand or a flag on the existing `issue` (CLI shape, minor).

## 5. Proposed plan & close-criterion sketch

Phase-0 audit → design (lock C1–C4 shapes + the witness set) → runbook → impl (the mock `--tier` issue path +
the C2 check + the dormant-path witnesses) → close (Appendix F for the mock CLI; findings flips — M10.2-A1
RESOLVED, M10-A-04 second-half RESOLVED; ch-doc reconcile if a tier path's spec drifted). A green M10.3 ships:
the parameterized mock issuing T2/T3/T4 assertions; the `accepted_tiers` per-issuer check (T2-issuer's T3
rejected); the Thread participation tier-gate firing; tier-appropriate `erasability` on issued assertions
(T4 `retained`); `module_kind: mock` populated; the empty-baseline invariant intact. Production change across
`xgen-core`/`xgen-node` + the `xgen-auth-module` mock path.

## 6. State

- **Status**: ACTIVE — the live frame the M10.3 D-071 Phase-0 audit picks up next.
- **Next-active**: Clair opens the M10.3 D-071 Phase-0 audit (§3 grounding) → design → Joe-lock → runbook.
- No DECISIONS change at open (M10.3 decisions arc-local, D-069). The M10.1 arc-local candidate
  ("module-policy lives in a signed `claims.extra` namespace") remains a candidate.
