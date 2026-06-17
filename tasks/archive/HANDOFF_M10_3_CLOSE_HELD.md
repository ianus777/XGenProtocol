# HANDOFF — M10.3 close HELD (3012 reject-code collision; renumber pending Joe-lock)
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jun 2026  
> **Last updated**: 2026-06-14  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. Why this handoff exists

**RETIRED — M10.3 CLOSED at J-368.** The renumber slot was Joe-locked (**3032**); the `3012→3032` code renumber landed + pushed; the J-368 doc-bridge ran (ch3 v0.54, design/M10.3-audit COMPLETED, M10.2-A1 RESOLVED, M10-A-04 fully RESOLVED, Appendix F v1.8 §F.10.1, ROADMAP v3.57). Next-active = M10.4 (MP-F13). This doc is kept as the contemporaneous record of the held interval; no further action.

M10.3 implementation is **shipped + pushed**, but the **close is HELD** on a D-065 catch that needs **one Joe-lock**
(the renumber slot) + **one tiny Clair code fix** before the Chat J-368 close bridge can run. This doc is the
single source of truth for resuming — do **not** close M10.3 until §3 is done, or the close re-introduces a wire-code
double-definition (the exact RC-F-01 bug M10.1 eliminated).

## 1. What is already done (on `main`, pushed)

M10.3 = parameterized T2–T4 mock + `accepted_tiers` enforcement + dormant-tier activation. Design Joe-LOCKED
J-366. Clair shipped 6 commits: runbook `a355ed2` → C1 `14972df` (xgen-core: `accepted_tiers_by_issuer()` +
`AssertionPolicy.accepted_tiers_by_issuer` + the C2 check at Step 1.5, restrictive-only) → C2 `a0e049b` (the mock:
`issue_tier1` → `issue(tier)`, CLI `issue --tier <N>`, grounded TTL 365/180/90, `module_kind: mock`, T4 retained;
dormant `Tier2/3/4Claims` not populated) → C3 `db3b882` (the gate derives `accepted_tiers_by_issuer` live →
`accepted_tiers` enforcement-bearing; empty-baseline node witness) → runbook-COMPLETED `80902ba`. **Verified:**
`cargo test --workspace` 1390/0; clippy clean (default + all-features); build 0; all 5 witnesses RED-on-revert;
empty-baseline invariant + M10.1/M10.2 witnesses intact. D1–D5 not reopened; D-092 not triggered.

## 2. The catch (D-065, Chat cross-file verification) — why the close is held

The M10.3 reject code **3012 collides with a documented spec reservation**:
- **Code** `xgen-core/src/identity/registration.rs:125`: `AssertionTierUnauthorized => (3012, "assertion_tier_unauthorized")`
  (also the `#[error]` string ~:98, the variant ~:99, the comment ~:239, the witness ~:1393/:1404).
- **Spec** `docs/xgen_ch3_specification.md` §3.11.7: **L3858 `3012 = watchlist_match`** (Tier 3/4 dormant
  reservation), and the L3854 reservation note states *"Codes 3012–3016 cover higher-tier Auth Module errors."*

So 3012 is double-defined (assertion_tier_unauthorized in code vs watchlist_match in spec). The audit/design "3012
free" was grounded on *no code emitter* but missed the *spec assignment*. Closing as-is would overwrite
`watchlist_match` or duplicate the row — not acceptable. **`watchlist_match` keeps 3012; the M10.3 code moves.**

## 3. PENDING — the one decision + the resume sequence

### 3a. Joe-lock needed: the renumber slot
Free slots in the 3000–3099 identity band (per the L3854 map): **3017–3019, 3024–3029, 3032+**.
**Chat recommendation: 3032** — adjacent to the tier-gating sub-band (3030 `tier_mismatch`, 3031
`kyc_verification_pending`); "issuer not authorized to attest this tier" belongs with the tier-authz codes.
Alternative: 3017 (the gap after the higher-tier band — but that band is higher-tier *requirements*, a different
concept). **Awaiting Joe's lock of the slot. Nothing below proceeds until then.**

### 3b. Clair (once the slot — call it `<N>` — is locked): the renumber
Tiny code-only follow-on. In `xgen-core/src/identity/registration.rs`: change `3012 → <N>` in the `#[error]`
string (~:98), the `to_registration_code` map (:125), the comment (~:239), and the §7 witness assertion
(~:1404, `assert_eq!(err.to_registration_code(), (<N>, "assertion_tier_unauthorized"))`). Sweep any
harness/wire-code/mptest test that pins the value. Rebuild + re-verify (1390/0 must hold; the witness flips to
`<N>`). Commit; Joe pushes.

### 3c. Chat (after the renumber lands): the J-368 close bridge
Atomic doc-only close (D-074). Files:
- **ch3** `docs/xgen_ch3_specification.md`: add a row **`<N>` `assertion_tier_unauthorized`** in the tier-gating
  sub-band (after the 3031 row, ~L3864); update the L3854 reservation note to "Codes 3030–`<N>` cover Auth-Tier /
  KYC gating" (or note `<N>` explicitly); bump ch3 header version (currently v0.53).
- **design** `tasks/M10_3_MOCK_TIER_DESIGN.md`: ACTIVE → COMPLETED, v1.0 → v1.1, close stamp **noting the D3 slot
  correction 3012 → `<N>`** (D-065).
- **M10.3 audit** `tasks/M10_3_MOCK_TIER_AUDIT.md`: ACTIVE → COMPLETED, v1.0 → v1.1, findings disposition —
  M10.3-A1/A2 RESOLVED (D1 live-read / D2 empty=unrestricted), **M10.3-A3 RESOLVED-with-correction** (the "3012
  free" call was wrong; shipped `<N>`), M10.3-A4 dormant-schema boundary recorded, M10.3-A5 no-action.
- **M10.2 audit** `tasks/M10_2_REFERENCE_BINARY_AUDIT.md`: **M10.2-A1 CARRIED → RESOLVED** (`accepted_tiers`
  enforcement-bearing); v1.1 → v1.2.
- **M10 audit** `tasks/M10_AUTH_MODULE_AUDIT.md`: **M10-A-04 → fully RESOLVED** (mock population landed — the
  J-361 "second half"); v1.2 → v1.3.
- **Appendix F** `docs/xgen_appendix_f_en.md`: new content section **§F.12** (= `issue --tier <N>` mock flag + the
  `<N>` reject semantics) + a Session entry; **the session log renumbers §F.11 → §F.13** (current state: §F.10 =
  the binary section from J-364, §F.11 = session log); v1.7 → v1.8.
- **ROADMAP** `docs/ROADMAP.md`: v3.56 → v3.57; M10 detail annotation (M10.3 SHIPPED + CLOSED, note the 3012→`<N>`
  close-catch); M-series + Post-gate-chain markers **M10.3 DESIGN J-366 → M10.3 DONE J-368**.
- **CLAUDE.md** PLAY head (M10.3 closed; next-active = M10.4) + **JOURNAL J-368** (the close entry).
- Delete/supersede this HANDOFF at close (or mark COMPLETED).
- DECISIONS: no change (arc-local, D-069). The M10.1 arc-local candidate remains a candidate.

## 4. Next-active after M10.3 closes

**M10.4 — MP-F13** (production identity → home-node discovery; J-278 / F1B-D5 family). A *named* M10 sub-arc per
the J-358 fork-2 lock, **depth decided at its own mini-Phase-0** — not silently absorbed. MP-C-16 (live
migration) re-runs only after MP-F13's disposition lands. Then **M10.5** (fold MP-F6 + re-run MP-C-06 / MP-C-16).

## 5. Quick state map (for session-open)

M10 🟢 · M10.1 ✅ (J-361) · M10.2 ✅ (J-364) · **M10.3 impl shipped, CLOSE HELD (J-367)** · M10.4 (MP-F13) next.
Chain: M10 → M11 → M12 → Round-2 final gate → UI → Streams. ROADMAP v3.56 · ch3 v0.53 · Appendix F v1.7.
HEAD after M10.3 impl: `80902ba` (runbook-COMPLETED) on top of the C-commits; all pushed.
