# M10.3 — Parameterized T2–T4 Mock + `accepted_tiers` Enforcement + Dormant-Tier Activation — Design
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jun 2026  
> **Last updated**: 2026-06-14  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. Status & lineage

Third M10 sub-arc (heaviest). Decisions M10.3-D1..D5 **Joe-LOCKED (J-366)** off the Phase-0 audit
(`tasks/M10_3_MOCK_TIER_AUDIT.md` v1.0, `325afec`, grounded vs `main` @`52897e5`, 5 findings). The four framing
calls (C1–C4, J-365) hold; this design locks the five shape questions the audit surfaced. Production arc
(`xgen-core`/`xgen-node` + the `xgen-auth-module` mock path). **Next-active = Clair: author the runbook**
`tasks/M10_3_MOCK_TIER_IMPL.md`. No code until the runbook lands.

## 1. Scope

**In:** the mock `--tier <N>` issue path; the C2 per-issuer `accepted_tiers` check (live-read); the dormant
tier-gate + TTL activation/witnesses; tier-appropriate `module_policy.erasability` issuance.

**Out (recorded boundaries):** MP-F13 → M10.4; MP-F6 fold + MP-C-06/MP-C-16 re-run → M10.5; erasure
*enforcement* (D3-gated); a mock-refusing node behaviour (flagged); **the dormant richer per-tier claim
schemas** (`Tier2/3/4Claims`) — not wired this arc (D4; no production consumer).

## 2. Locked decisions

**M10.3-D1 — C2 shape (live-read the issuer's accepted_tiers).** Today `validate_assertion` sees only
`AssertionPolicy.trusted_issuers: HashSet<String>` (URIs) — not the issuer record (A1). Add
`AuthModuleRegistry::accepted_tiers_by_issuer()` + a new `AssertionPolicy` field (issuer → accepted_tiers),
**derived live at the gate** beside `trusted_issuers` (the M10.2-D2 live-read site, ~`app.rs:2895`) so the
registry stays the single source of truth and `accepted_tiers` becomes enforcement-bearing (the M10.2-A1
deferral, come due). The **C2 check inserts after Step 1** (issuer-trusted, `registration.rs:218`) and is
**distinct** from the node-wide `tier ≥ required_tier` Step 4 (`registration.rs:230`): per-issuer set-membership
vs node-wide ordered floor — no double-reject (different questions).

**M10.3-D2 — empty `accepted_tiers` = unrestricted (restrictive-only).** Forced by the M10.2 baseline: config
seeding inserts issuers with *empty* `accepted_tiers` and trusts them (A2). So **empty ⇒ the issuer may attest
any tier**; C2 is **invisible at the empty/T1 baseline** and bites only when an operator explicitly sets a tier
scope. Preserves the empty-baseline invariant byte-for-byte.

**M10.3-D3 — reject code 3012 `assertion_tier_unauthorized`.** C2's rejection takes a **new** code in the
auth-module band (3012, first free reserved slot per the M10.1 reconcile), **distinct from 3030 `tier_mismatch`**
(3030 = "tier too low for this Space"; 3012 = "this issuer isn't authorized to attest this tier"). Two gates,
two wire codes.

**M10.3-D4 — mock issuance = tier + grounded TTL + descriptor; dormant claim schemas NOT populated.** Audit A4:
`Tier2/3/4Claims` are `#[cfg(test)]` — no production validator reads them. What validation actually consumes per
tier = the **tier integer** (→ `assertion_tier_of`, `registration.rs:~214` → the gates) + **TTL** (grounded
T2=365 / T3=180 / T4=90 days via `ttl_days`) + the **descriptor**. So the mock issues exactly that:
`tier ∈ {2,3,4}` + the grounded TTL + `module_kind: mock` + tier-appropriate `module_policy.erasability`
(T2–T3 `erasable`, **T4 `retained`**). It does **not** populate the dormant richer claim schemas (populating an
unread struct would be theater). **This narrows the C3 lock honestly:** M10.3 activates + witnesses the
tier-integer-driven gates + the TTLs; the richer per-tier claim schemas stay a **flagged future** (wire them
when a production consumer exists).

**M10.3-D5 — CLI = a flag on the existing `issue`.** `xgen-auth-module issue --tier <N>`: N=1 → reference
(today's default, unchanged), N ∈ {2,3,4} → auto-sets `module_kind: mock` + the tier-appropriate TTL/erasability.
One parameterized issue path, no new subcommand. The mock's trust is still operator-set via the existing CRUD
verbs (D-092 not triggered — the binary CLI is its own arg parser, not a node verb).

## 3. Impl surface (audit-grounded; Clair confirms file:line at runbook)

- C2 (D1): `AuthModuleRegistry::accepted_tiers_by_issuer()` (new, over `module_registry.rs:59` `accepted_tiers`);
  the `AssertionPolicy` field + its live-derivation at the gate (~`app.rs:2895`, beside `trusted_issuers`);
  the C2 check in `validate_assertion` after Step 1 (`registration.rs:218`), emitting 3012.
- Reject code (D3): the auth-module wire band (3012 `assertion_tier_unauthorized`); the reason-string + the
  `RejectInfo` plumbing (MP-F2 path).
- Dormant activation (D4): the Thread participation tier-gate (`runtime.rs:1503`,
  `verify_tier_assertion(creator_tier, thread_tier)`); `assertion_tier_of` (`runtime.rs:~214/217`) as the
  activation read; the sibling join gate (`runtime.rs:1393`); `ttl_days` (T2/T3/T4 = 365/180/90).
- Mock issuance (D5): the `xgen-auth-module` `issue_tier1()` base → the `--tier <N>` parameterization +
  `module_kind: mock` + tier-appropriate `erasability`.
- D-088 (issuance-side only): the attach point (`assertion_tier_of` + `module_policy()`) — **not built**; no
  erasure op exists in-tree (boundary explicit).

## 4. Proof obligations (RED-on-revert; Clair builds in the runbook)

1. **Mock issuance** — `issue --tier 2|3|4` produces a valid signed assertion (tier + grounded TTL +
   `module_kind: mock` + tier-appropriate `erasability`, T4 `retained`); accepted by a node that trusts the
   issuer for that tier. RED on revert.
2. **C2 accepted_tiers scope** — an issuer scoped to T2 has its **T3** assertion **rejected with 3012**; its T2
   assertion accepted. RED on revert. (Distinct from a Space-required_tier reject = 3030.)
3. **Empty-baseline invariant** — an issuer with empty `accepted_tiers` attests any tier (C2 invisible); the
   M10.2 seeded/empty path unchanged byte-for-byte.
4. **Thread participation tier-gate** — in a tier-gated Thread, a sufficient-tier identity participates, an
   insufficient-tier one is refused (the dormant gate, now fired by a real higher-tier identity). RED on revert.
5. **`module_kind: mock` populated** — issued mock assertions carry the label (expression-only; the
   `trusted_auth_modules`/registry gate remains the safety mechanism, C4).

## 5. Design-close details (Clair confirms at runbook; Joe-call only if non-obvious)

- The exact `AssertionPolicy` field type for the issuer→accepted_tiers map + its live-derivation point (mirror
  the M10.2 `trusted_issuers` derivation).
- The C2 check's precise position relative to Steps 2–3 (after Step 1; before or after the tier-floor Step 4 —
  order doesn't change outcome but pick the clearest).
- Whether `--tier 1` is a strict no-op vs the existing default (D5: it must remain today's reference behaviour).

## 6. Close deliverables

- Appendix F: the `issue --tier <N>` mock flag + the 3012 reject semantics (operator-visible).
- ch3 reconcile: activate 3012 in the auth-module band (the M10.1 reconcile reserved 3012–3016 dormant; this
  arc fills 3012).
- Findings flips at close: **M10.2-A1 RESOLVED** (accepted_tiers enforcement-bearing) + **M10-A-04 second half
  RESOLVED** (`module_kind: mock` populated); M10.3-A4 recorded as the dormant-schema boundary.
- DECISIONS: candidates only, arc-local (D-069). Matrix as applicable.

## 7. Next-active

Clair: author `tasks/M10_3_MOCK_TIER_IMPL.md` (the `accepted_tiers_by_issuer` + `AssertionPolicy` field + the
C2 check/3012 + the mock `--tier` path + the dormant-gate witnesses), confirming the §3 groundings + §5 details
to file:line → implement → Chat doc-bridge → close. No code until the runbook lands.

## 8. Close (J-368)

**SHIPPED + CLOSED.** Clair shipped 6 commits (`a355ed2` runbook → C1 `14972df` / C2 `a0e049b` / C3 `db3b882`
→ runbook-COMPLETED `80902ba`); D1–D5 honored, D-092 not triggered. Verified `cargo test --workspace` 1390/0;
clippy clean (default + all-features); 5 witnesses RED-on-revert; empty-baseline invariant + M10.1/M10.2
witnesses intact.

**D3 slot correction — 3012 → 3032 (D-065, J-367 catch).** The locked D3 reject code **3012** was grounded on
*no code emitter* but missed a *spec reservation*: ch3 §3.11.7 already assigned **3012 = `watchlist_match`**
(L3858, Tier-3/4 dormant). Closing as-is would have re-introduced the exact RC-F-01 double-definition M10.1
eliminated. Resolution (Joe-locked slot): `watchlist_match` keeps **3012**; the M10.3 auth-tier-authz code moves
to **3032**, adjacent to the 3030/3031 tier-authz sub-band. Code renumber + witness sweep landed pre-bridge
(registration.rs + xgen-auth-module/end_to_end.rs; 1390/0 held); ch3 activates the 3032 row this close. The D3
decision direction (a distinct auth-module-band code for issuer-tier-authz, separate from 3030) is unchanged —
only the integer moved.

**Findings flipped (this bridge):** M10.3-A1/A2 RESOLVED (D1 live-read / D2 empty=unrestricted); **M10.3-A3
RESOLVED-with-correction** (the “3012 free” call was wrong — shipped 3032); M10.3-A4 dormant-schema boundary
recorded; M10.3-A5 no-action. Cross-arc: **M10.2-A1 CARRIED → RESOLVED** (`accepted_tiers` enforcement-bearing);
**M10-A-04 → fully RESOLVED** (`module_kind: mock` populated — the J-361 “second half”). DECISIONS: no change
(arc-local, D-069). **Next-active: M10.4 (MP-F13).**
