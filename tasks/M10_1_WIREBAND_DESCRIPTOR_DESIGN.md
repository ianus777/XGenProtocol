# M10.1 — Wire-Band Reconcile + AI-D8 Module-Policy Descriptor — Design
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-13  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. Status & lineage

First M10 sub-arc. Decisions M10.1-D1..D4 **Joe-LOCKED (J-360)** off the M10 framing discussion; grounded on the
M10 D-071 Phase-0 audit (`tasks/M10_AUTH_MODULE_AUDIT.md` v1.0, `f759a7d`, vs `main` @`5d8fec1`) — no fresh full
audit (the audit grounded §A3 the wire band + §A4 the `claims.extra` extension point). Production arc
(`xgen-common`/`xgen-core` + ch3 spec); small surface. **Next-active = Clair: confirm the marked groundings +
author the runbook** (`tasks/M10_1_WIREBAND_DESCRIPTOR_IMPL.md`). No code until the runbook lands.

## 1. Scope

**In:** (a) RC-F-01 reconciliation of the 3010–3016 auth-module wire band; (b) the AI-D8 module-policy
descriptor + the mock self-label, both on `claims.extra`.

**Out (explicit):** M10-A-02 registry↔policy disconnect → **M10.2** (the binary's trust path, not here).
M10-A-03 the 7 event-validation codes (`exchange.rs:140` → 4000) → **split, sibling decision** (own small arc,
MP-F2-followon; a 30xx event-validation sub-band or 40xx slots) — flagged, not built in M10.1. The erasure
*consumer* ("T4 refuses erasure" enforcement) stays D3-gated (Fork 3 = hook only); M10.1 lands only the
*descriptor field* it reads.

## 2. Locked decisions

**M10.1-D1 — RC-F-01 renumber (Call 1).** Arc-E **keeps** 3010 `assertion_identity_mismatch` + 3011
`assertion_claims_insufficient` (live, emitted, test-asserted — moving shipped wire = break). The dormant
§3.11.7 higher-tier rows re-home: `auth_tier_insufficient` **folds into the live 3030 `tier_mismatch`** (the
real, emitted tier code); `kyc_verification_pending` takes **3031** (adjacent to 3030 — tier/KYC is one domain,
not the 3010 identity band). 3012–3016 stay reserved/dormant. Net: a spec correction + one reserved constant,
zero change to emitted codes.

**M10.1-D2 — event-validation codes split (Call 2).** The 7 unmapped signature/membership/permission codes are
**not** M10.1 scope; recorded as the MP-F2-followon sibling decision. M10.1 owns only the 3010–3016 + 3030/3031
auth-module/tier band.

**M10.1-D3 — AI-D8 module-policy descriptor (Call 3).** Lives under **`claims.extra["module_policy"]`** — a
single namespaced object, signed (canonical-bytes sort `claims.extra`; a new top-level field would be wrong,
AE-D5). First member = `erasability` (the Fork-3 hook the tier-gate reads). The §8 open-doors principle is
satisfied **structurally** — `extra` preserves unknown members verbatim, so future module-policy members need no
wire change. No enforcement consumer in M10.1.

**M10.1-D4 — mock self-label (Call 4).** Folded here (cheapest — the descriptor wire surface is already
opening): **`claims.extra["module_kind"]` ∈ {`reference`, `mock`}**, signed in `extra`. M10.3's parameterized
mock populates it rather than forcing a second wire change. (Enforcement remains the `trusted_auth_modules`
gate, M10-A-04 — the label is *expression*, not a trust mechanism.)

## 3. Descriptor schema (proposed; Clair confirms shape at runbook)

```jsonc
claims.extra: {
  "module_kind": "reference",          // D4 — "reference" | "mock"
  "module_policy": {                    // D3 — namespaced, signed, forward-extensible
    "erasability": { "retention": "erasable" }   // first member; exact enum vs D-088/Arc-I design — confirm
    // future members preserved verbatim (§8 open-doors, inherent to extra)
  }
}
```
- `erasability` shape (enum vs object; default by tier) is the one **design-close detail** — Clair grounds it
  against the D-088 / Arc-I module-policy design and the existing `assertion_tier_of` read; if it needs a
  Joe-call, surface it at runbook, don't guess.
- Typed read/write accessors (not raw map fiddling) in the crate that owns `TrustClaims`.

## 4. Grounding & impl surface (audit-grounded; Clair confirms file:line at runbook)

- Band sites (D1): `registration.rs:120–121` (3010/3011 emit), the live 3030 `tier_mismatch` site, ch3 §3.6.5
  (Arc-E definitions) + §3.11.7 (the dormant rows to re-home) + the L3829 reservation note. Reserve 3031.
- Descriptor home (D3/D4): `TrustClaims` + `claims.extra` (`trust_assertion.rs:105`), canonical-bytes signer.
- Spec: ch3 §3.11.7 corrected to 3030/3031; §3.6.5 unchanged; the reservation note updated.
- Crates: `xgen-common` (TrustClaims accessors + code constants) + any `xgen-core` reference; **no node/client
  behaviour change** (no enforcement consumer this arc).

## 5. Proof obligations (RED-on-revert; Clair builds in the runbook)

1. **Sign/verify roundtrip** — `module_kind` + `module_policy` set in `extra` are covered by the assertion
   signature (tamper → verify fails). RED on revert.
2. **§8 open-doors** — an assertion carrying an *unknown* `module_policy` member round-trips + verifies
   unchanged (extra preserves it). RED if a member is dropped.
3. **Band reconcile** — 3010/3011 unchanged + emitted as before; 3030 is the live tier code; 3031 reserved for
   KYC; ch3 §3.11.7 no longer claims 3010/3011. (Doc + constant assertion.)
4. **Mock label** — `module_kind` read accessor returns the signed value; absent ⇒ treated as `reference`/none
   per the confirmed default.

## 6. Close deliverables

- ch3 §3.11.7 reconcile (D1) + the descriptor schema documented (ch3 identity/assertion section + Appendix C/I
  as applicable — Clair confirms the home).
- Appendix F entries if any operator-visible surface changes (likely none — wire/struct only).
- DECISIONS: candidates only (e.g. "module-policy lives in a signed `claims.extra` namespace, never a new
  top-level field"); arc-local (D-069) until recurrence or Joe's explicit promotion — none pre-decided here.
- Matrix/`MP_findings`: M10-A-01 → RESOLVED at close; M10-A-03 → carried as the named sibling decision.

## 7. Next-active

Clair: confirm the §4 groundings to file:line + author `tasks/M10_1_WIREBAND_DESCRIPTOR_IMPL.md` (the renumber +
the `claims.extra` descriptor/accessors + the §5 witnesses) → Clair implements → Chat doc-bridge → close. The
single design-close detail to surface (Joe-call only if non-obvious): the `erasability` member shape (§3).
