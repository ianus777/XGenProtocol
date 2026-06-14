# M10.3 — Parameterized T2–T4 Mock + Dormant-Tier-Path Activation — D-071 Phase-0 Audit
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jun 2026  
> **Last updated**: 2026-06-14  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. Scope, method, headline

**In scope (grounded to file:line on live `main`).** Brief §3's six grounding items (A1–A6); the C2 check
insert-point + ordering sketch (§7); the per-tier claim/TTL defaults the mock should carry (§8, grounded
against `tiers.rs`); the blast radius of the C2 validation behaviour change (A7); a findings register (§10).

**Method.** Symbol definitions grepped in production code (D-078), not inferred. Where spec/code disagree, the
code is the live truth and the gap is a finding (D-065). Grounded against `main` @ **`52897e5`** (M10.3 open).
Re-grounded fresh (M10.2 shifted app.rs by ~70 lines; registration.rs/runtime.rs line numbers re-verified).

**The four calls are LOCKED (brief §2, J-365) and not reopened:** C1 (mock = parameterized `--tier <N>` on the
one binary), C2 (`accepted_tiers` enforcement = a new explicit per-issuer check), C3 (dormant-tier activation;
erasure issuance-side only), C4 (`module_kind: mock` = expression-only).

**Headline of the grounding (read first).** The locked frame holds, sharpened on three real catches — exactly
the kind the brief anticipated:

1. **C2 is clean in concept but needs the issuer record threaded into `validate_assertion`.** The gate's policy
   (`AssertionPolicy`, `registration.rs:163`) carries **only** `trusted_issuers: HashSet<String>` — a set of
   issuer *URIs*, not the per-issuer `accepted_tiers`. M10.2's live-read derives that URI set from the registry
   (`app.rs:2895`); it does **not** carry the records. So `assertion.tier ∈ issuer.accepted_tiers` cannot be
   checked inside `validate_assertion` as shaped — the gate must additionally derive a per-issuer
   `accepted_tiers` map (mirroring `trusted_issuers()`) and thread it into the policy. This is exactly the
   M10.2-A1 prediction come due (**M10.3-A1**).

2. **Empty `accepted_tiers` MUST mean "unrestricted", or C2 regresses M10.2.** M10.2 seeds config issuers with
   **empty** `accepted_tiers` (`app.rs` `seed_trusted_auth_modules`), and `trusted_issuers()` trusts them
   regardless of tier. If C2 read empty as "accepts no tier", every seeded/empty-tier issuer's assertions would
   start rejecting — a regression + a broken empty-baseline. C2 must be **restrictive-only** (empty ⇒ no
   per-issuer restriction), biting only where an operator explicitly set `accepted_tiers` (**M10.3-A2**).

3. **"Per-tier claims" on the wire is the tier integer + TTL + descriptor — the `TierNClaims` schemas are
   dormant-unwired.** `Tier2Claims`/`Tier3Claims`/`Tier4Claims` (`tiers.rs:72/83/100`) have **no production
   reader** (all references are `#[cfg(test)]`). `validate_assertion` checks only `TrustClaims.tier_verified`
   (xgen-common) + `policy.required_claims`. So what differs T1→T4 *that validation sees* = the `tier` integer
   (which `assertion_tier_of` reads → the gates) and the TTL (`valid_until`, grounded per tier). The richer
   claim schemas are expression-only and unvalidated (**M10.3-A4**).

Everything else (the Thread participation gate, the activation mechanism, the mock parameterization, the D-088
touch-point) is shipped substrate the arc exercises or extends.

---

## A1 — The `accepted_tiers` field + the live-read seam + the C2 insert point (brief §3.1)

*Goal: where C2's per-issuer tier check inserts, and how it gets the issuer's `accepted_tiers`.*

- **The field** — `AuthModuleRecord.accepted_tiers: Vec<AuthTier>` (`module_registry.rs:59`), "the Auth Tiers
  this Node accepts attestations for from this module." Set by the CRUD verbs: `auth-module register --tier …`
  (`admin_ops.rs:2487`, M10.2) and `auth-module set-tiers` (replaces the set). **These already exist** — C2 makes
  them enforcement-bearing; no new node verb (C4/D-092 holds, §A7).

- **The gate's policy today carries no per-issuer tiers.** `AssertionPolicy` (`registration.rs:163`):
  `trusted_issuers: HashSet<String>` + `required_claims: Vec<String>` + `required_tier: u32`. M10.2's gate
  (`handle_identity_msg`, `app.rs:2879–2896`) clones the runtime's registry Arc, locks it, and sets
  `policy.trusted_issuers = registry.lock().await.trusted_issuers()` (`app.rs:2895`). `trusted_issuers()`
  (`module_registry.rs:149`) returns **only the non-revoked module URIs** — not the records. So
  `validate_assertion` sees the issuer *URI set*, never the issuer's `accepted_tiers`.

- **Consequence (M10.3-A1):** C2's `assertion.tier ∈ issuer.accepted_tiers` needs the issuer's tiers. The clean
  shape (mirrors the M10.2 live-read): a new registry method `accepted_tiers_by_issuer() -> HashMap<String,
  Vec<AuthTier>>` (non-revoked) + a new `AssertionPolicy` field `accepted_tiers_by_issuer`, derived in the SAME
  gate block (`app.rs:2894`, beside `trusted_issuers`). `validate_assertion` then checks it. `AuthTier` is
  xgen-core (`tiers.rs:35`); `AssertionPolicy` is xgen-core — no new dep.

- **The C2 insert point** — `validate_assertion` (`registration.rs:211`), the 7-check body. Natural slot is a
  **new check right after Step 1** (issuer ∈ `trusted_issuers`, `:218`): once the issuer is known-trusted, check
  that this issuer is *authorized to attest this tier*. So Step 1 (issuer trusted) → **C2 (issuer authorized for
  `assertion.tier`)** → Steps 2–7. Restrictive-only: skip when the issuer's tier set is empty/absent (M10.3-A2).

**A1 verdict.** C2 is a contained check, but the brief's "the gate already has the record" is half-true: the gate
has the registry Arc, but the *policy* `validate_assertion` sees carries only URIs. C2 = a new registry method +
a new policy field (derived live beside `trusted_issuers`) + a new check after Step 1. The reject wire-code is a
design-lock (§A2/§11) — distinct semantics from 3030.

---

## A2 — The node-wide `required_tier` check — distinct from C2 (brief §3.2)

*Goal: confirm the two gates compose without overlap or double-reject ambiguity.*

- **The node-wide check** — Step 4 of `validate_assertion` (`registration.rs:230`):
  `verify_tier_assertion(assertion.tier, policy.required_tier)` → `AssertionTierInsufficient` → **3030
  `tier_mismatch`**. `verify_tier_assertion(assertion_tier, floor)` (`tiers.rs:158`) = "`assertion_tier ≥
  floor`". `required_tier` defaults to **1** (`registration.rs:181`), so this is a no-op today (any tier ≥ 1).

- **The two gates ask different questions (genuinely distinct):**
  - **Step 4 (node-wide floor):** *is this assertion's tier high enough for this Node to accept a registration?*
    — a single node-wide minimum (`required_tier`), tier-**ordered** (`≥`).
  - **C2 (per-issuer scope):** *is THIS ISSUER authorized to attest this tier?* — a per-issuer set-membership
    (`assertion.tier ∈ issuer.accepted_tiers`), **not** ordered. A T2-scoped issuer attesting T3 fails C2 even
    though T3 ≥ any floor; a T4-authorized issuer attesting T1 passes C2 (1 ∈ its set) but a node with
    `required_tier=2` would reject at Step 4.
  - **No overlap / no double-reject ambiguity:** an assertion must pass *both* (different conditions, different
    failure meanings). They never both reject the *same* legitimate assertion; their reject reasons are
    orthogonal (issuer-not-authorized vs tier-below-node-floor). C2 should carry a **distinct wire code** (not
    3030) so the rejection is diagnostically honest — see §11.

**A2 verdict.** Distinct. C2 = per-issuer set-membership (un-ordered); Step 4 = node-wide ordered floor. They
compose cleanly. The only design call is C2's reject code (a new auth-module-band code — 3012 is a free dormant
slot per RC-F-01 — vs reusing 3030; lean new, distinct semantics).

---

## A3 — The Arc-E Thread participation tier-gate + the activation mechanism (brief §3.3)

*Goal: the dormant gate C3 activates; what a higher-tier identity changes.*

- **The gate** — `NodeRuntime::dispatch_event`, the `EventType::ThreadCreate` arm (`runtime.rs:1485–1512`):
  - `thread_tier = event.content["auth_tier_min"].as_u64().unwrap_or(1)` (`:1487`).
  - **Check 1** (`:1488`): `thread_tier < space.auth_tier` → reject **3030 `thread_auth_tier_below_room`** (a
    thread can't require *less* than its Space).
  - `creator_tier = self.identity_registry.get(&event.sender).map(assertion_tier_of).unwrap_or(1)` (`:1498`).
  - **Check 2 — the participation gate** (`:1503`): `verify_tier_assertion(creator_tier, thread_tier)` → if
    `creator_tier < thread_tier`, reject (3030 `tier_mismatch`).

- **The activation mechanism — `assertion_tier_of`** (`runtime.rs:214`): `None → 1`, else
  `record.trust_assertion["tier"]`. Post-Arc-E this is the **validated** tier (doc `:207–213`). The gate is
  **dormant** only because every identity resolves to T1 today (no higher-tier issuer existed before M10.2). The
  moment a mock issues a T2 assertion and an identity registers with it, the stored validated tier is 2 →
  `assertion_tier_of → 2` → the gate fires for real.

- **What a higher-tier identity changes (the witness):** in a **T1 Space**, a `ThreadCreate` with
  `auth_tier_min: 2` passes Check 1 (`2 ≥ 1`). Then Check 2: a creator with a mock **T2** assertion →
  `verify_tier_assertion(2, 2) = Ok` → **accepted**; a **T1** creator → `verify_tier_assertion(1, 2) = Err` →
  **rejected** (3030). So the witness is: *T2 identity creates a T2-required thread (accepted) vs T1 identity
  (rejected)* — runnable in a T1 Space, no T2 Space needed. **RED-on-revert:** without the mock's T2 issuance the
  creator stays T1 and the T2 thread is rejected.

- **The sibling gate (note, not C3's named focus):** the **PG-13 Space-join tier-gate** (`runtime.rs:1393`,
  `verify_tier_assertion(joiner_tier, space.auth_tier)`) is the same family — also `assertion_tier_of`-driven,
  also dormant-until-higher-tier. C3's named witness is the Thread gate; the join gate activates by the same
  mechanism and is the natural second witness if the design wants breadth.

**A3 verdict.** The Thread participation gate is real, correct, and **dormant only for lack of a higher-tier
identity** — exactly the "needs a higher-tier identity to fire" path C3 activates. No code change to the gate;
M10.3 supplies the higher-tier identity (via the mock) and witnesses the fire. The gate needs *only* a
higher-tier identity (no other prerequisite) — the brief's premise holds.

---

## A4 — Per-tier claims/TTL handling (brief §3.4)

*Goal: how `TrustClaims` carries tier + TTL; what differs T1→T4; the dormant paths.*

- **What validation actually reads per tier (the live truth):**
  - **`TrustAssertion.tier: u32`** (`trust_assertion.rs:256`) — the only tier signal validation/`assertion_tier_of`
    consume. Drives Step 4 + C2 + the Thread/join gates.
  - **`TrustClaims.tier_verified: bool`** (`trust_assertion.rs:93`) — mandatory at every tier (Step 6).
  - **`valid_until`** (`trust_assertion.rs:265`) — the TTL. Per-tier defaults are **grounded** in `tiers.rs`:
    `AuthTier::ttl_days()` (`:57`) → T1 `None`, **T2 `365`** (`TIER2_TTL_DAYS`), **T3 `180`**, **T4 `90`** — the
    TTL *tightens as tier rises* (the same exposure-window-minimization principle as the INV-D6 invite ceiling,
    `runtime.rs:221`). So a faithful mock issues each tier with `valid_until = now + ttl_days(N)`.

- **The `TierNClaims` schemas are dormant-unwired (D-065 catch, M10.3-A4):** `Tier2Claims` (`tiers.rs:72`),
  `Tier3Claims` (`:83`), `Tier4Claims` (`:100`, carries `jurisdiction`) have **no production reader** — every
  reference is `#[cfg(test)]` (`:208/:229/:252`). `validate_assertion` does not deserialize or check them. So
  "per-tier claims" the mock can carry split into: **(a) validated** — `tier` + `tier_verified` + `valid_until`
  (these matter); **(b) expression-only** — the `TierNClaims` contact/jurisdiction fields (the mock *may*
  populate them under `claims` for realism, but nothing validates them; honest to label them dormant).

- **The dormant paths that only fire above T1:** the Thread gate (A3), the join gate (A3 sibling), and the
  tier-graded **invite-validity ceiling** (`runtime.rs:221–229`, "only Tier 1 is defined now: 14 days") — the
  last is M8.5-B and **outside C3's named scope** (a tier-graded *invite* ceiling, distinct from the assertion's
  own TTL; recorded as a sibling dormant path, not built here).

**A4 verdict.** Per-tier, the mock must set `{ tier: N, tier_verified: true, valid_until: now + ttl_days(N),
module_kind: mock, module_policy.erasability: per-tier }`. The TTL defaults are grounded (365/180/90), not
invented. The richer `TierNClaims` schemas are dormant-unwired — the design should decide whether the mock
populates them for expression (and label them unvalidated) or omits them (§8/§11).

---

## A5 — The mock issue surface on `xgen-auth-module` (brief §3.5)

*Goal: the M10.2 issue path → the minimal `--tier <N>` mock parameterization.*

- **The current base** — `xgen_auth_module::issue_tier1(module_key, identity_id, valid_until)` (`lib.rs:51`):
  builds a `TrustAssertion` with `tier: 1` (`:73`), `module_kind: Reference` (`:62`), `module_policy.erasability
  .retention: Erasable` (`:65`), `tier_verified: true`, then `.sign(module_key)`. The CLI `issue` verb
  (`main.rs`) computes `valid_until = now + valid_days` and calls it.

- **The minimal C1 parameterization** — generalize to `issue_tier(module_key, identity_id, tier: AuthTier,
  valid_until, module_kind: ModuleKind)` (or a sibling `issue_mock`), where:
  - `tier = N` (N ∈ {2,3,4} for the mock; 1 for the reference default).
  - `module_kind = Mock` for the `--tier <N>` mock path; `Reference` stays the T1 default (`issue_tier1`
    preserved as the honest default, C1).
  - `valid_until` default = `now + ttl_days(N)` (A4: T2=365, T3=180, T4=90; T1 keeps its current default).
  - `module_policy.erasability.retention` = **`Erasable` for N ∈ {1,2,3}, `Retained` for N = 4** (C3 — the
    issuance-side D-088 tier-gate; T4 = legal-hold).
  - CLI: a `--tier <N>` flag on `issue` (default 1) is the leanest shape (or a `mock` subcommand) — §11, minor.

- **No second crate (C1)** — the mock is the same binary parameterized; "different module instances" = the one
  binary run with different keypairs/configs. The binary's CLI is its own arg parser (not node verbs) — **D-092
  not triggered** (§A7).

**A5 verdict.** A small, contained extension of the shipped `issue_tier1`: add the tier + module_kind +
per-tier-TTL + per-tier-erasability parameters; keep `issue_tier1` (reference T1) as the honest default. The
descriptor accessors (`set_module_kind`, `set_module_policy`) shipped in M10.1 — no new descriptor work.

---

## A6 — The D-088 erasure tier-gate touch-point (brief §3.6)

*Goal: confirm issuance-side `erasability` is all M10.3 builds; ground where an enforcement consumer would
attach, without building it.*

- **What M10.3 builds (issuance-side only):** the mock sets `module_policy.erasability.retention` per tier (A5):
  T1–T3 `Erasable`, **T4 `Retained`**. This rides `claims.extra["module_policy"]`, signed (M10.1). M10.3
  witnesses *the field on the issued assertion* (T4 carries `retained`), nothing more.

- **Where an enforcement consumer WOULD attach (boundary, not built):** a "T4 refuses erasure" gate would read
  (1) the identity's tier via **`assertion_tier_of(record)`** (`runtime.rs:214`) and (2) the stored assertion's
  **`module_policy().erasability`** via the M10.1 `TrustClaims::module_policy()` accessor
  (`trust_assertion.rs:225`) — and refuse an erasure op when `retention == Retained`. **There is no erasure
  operation in the tree** (M10 audit A6 / M10-A-05: grep for erasure/RTBF/tombstone → no production op), so there
  is nothing to gate. `module_policy()` / `erasability` have **no production reader** today (only the M10.1
  accessor + tests). The consumer stays D3-gated (PG-02 / Fork 3 = hook only).

**A6 verdict.** Issuance-side is the whole of M10.3's D-088 surface — the mock expresses tier-appropriate
retention; the node does not enforce it. The enforcement attach point is grounded (`assertion_tier_of` +
`module_policy()`) and explicitly **not built**. Honest boundary: "T4 refuses erasure" is *expressed*, not
*enforced*.

---

## A7 — Blast radius of the C2 validation behaviour change

*Goal: ground every `validate_assertion` path + the empty/T1-only baseline (the change must be invisible when no
issuer has a non-empty `accepted_tiers`).*

- **`validate_assertion` production callers: exactly ONE** — `registration.rs:486` (inside `accept_registration`),
  reached from **one** node path (`handle_identity_msg`, `app.rs:2862`; the gate block `:2879–2898`). Every other match is a test
  (`registration.rs:1239–1355`). The C2 behaviour-change surface is the same single seam M10.2 used.

- **The C2 change surface:** (a) `AuthModuleRegistry::accepted_tiers_by_issuer()` (new, xgen-core); (b)
  `AssertionPolicy.accepted_tiers_by_issuer` field (new, xgen-core); (c) the gate derives it live beside
  `trusted_issuers` (`app.rs:2894–2895`); (d) the new check in `validate_assertion` after Step 1; (e) a reject
  variant + wire code. The single production caller is untouched in signature (the policy carries the new field).

- **Empty / T1-only baseline (the invariant):** **restrictive-only** semantics (M10.3-A2) — C2 is a no-op when
  the issuer's `accepted_tiers` is empty/absent. Today **every** issuer has empty `accepted_tiers` unless an
  operator set it (M10.2 seeds empty; CRUD `register --tier` sets it). So with no operator-set tier scope:
  empty registry → step 1 rejects (C2 unreached); seeded/empty issuers → C2 skipped → **byte-for-byte M10.2**.
  C2 bites *only* where an operator explicitly registered an issuer with a non-empty `accepted_tiers` — the
  intended enforcement, invisible otherwise. This is the M10.3 prime invariant (assert it).

- **Out of the blast radius:** Local-Node bypass (`registration.rs:472 if !local_node`) — never enters the gate
  (Fork 1). Steps 2/3/4/5/6/7 unchanged. The CRUD verbs unchanged in behaviour (C2 just *consults*
  `accepted_tiers` they already set). The Thread/join gates (A3) are unchanged code — they *fire* because a
  higher-tier identity now exists, not because M10.3 edits them.

**A7 verdict.** One node-code seam (the same M10.2 chain), restrictive-only so it is invisible at the empty/T1
baseline and bites only on operator-configured per-issuer tier scope. Asserted by an empty-baseline witness.

---

## 7. C2 check — insert-point + ordering sketch (deliverable; design locks)

```
validate_assertion(assertion, registering_id, policy, now):
  Step 1   issuer ∈ policy.trusted_issuers            else 3006 AuthModuleUntrusted     (registration.rs:218)
  Step 1.5 [C2 NEW] if let Some(tiers) = policy.accepted_tiers_by_issuer.get(issuer):
             if !tiers.is_empty() && !tiers.contains(assertion.tier)
                                                       else <new code> issuer-tier-unauthorized
             (empty/absent ⇒ unrestricted — M10.3-A2; restrictive-only)
  Step 2   signature verifies                          else 3004                         (:222)
  Step 3   identity_id matches                          else 3010                         (:226)
  Step 4   tier ≥ policy.required_tier                  else 3030 tier_mismatch           (:230)
  Steps 5–7 expiry / tier_verified / required_claims    …                                 (:234–249)
```

- **Insert after Step 1** (issuer must be trusted before its tier-scope is meaningful). C2 is per-issuer
  set-membership; Step 4 is the node-wide ordered floor (A2) — distinct, no double-reject.
- **The policy gains `accepted_tiers_by_issuer: HashMap<String, Vec<AuthTier>>`,** derived live at the gate
  (`app.rs:2894`) from a new `AuthModuleRegistry::accepted_tiers_by_issuer()` (non-revoked), beside the existing
  `policy.trusted_issuers = registry.trusted_issuers()`.
- **Reject code (design-lock, §11):** a new auth-module-band code (3012 is a free dormant slot, RC-F-01) with a
  distinct name (e.g. `issuer_tier_unauthorized`) — *not* 3030 (which means "tier below the node's floor").

## 8. Per-tier claim/TTL defaults the mock should carry (deliverable; grounded against `tiers.rs`)

| Tier | `tier` | `tier_verified` | `valid_until` (TTL) | `module_kind` | `erasability.retention` |
|---|---|---|---|---|---|
| T1 (reference default) | 1 | true | now + (current default) | `reference` | `erasable` |
| T2 mock | 2 | true | now + **365 d** (`TIER2_TTL_DAYS`) | `mock` | `erasable` |
| T3 mock | 3 | true | now + **180 d** (`TIER3_TTL_DAYS`) | `mock` | `erasable` |
| T4 mock | 4 | true | now + **90 d** (`TIER4_TTL_DAYS`) | `mock` | **`retained`** |

- TTLs are `AuthTier::ttl_days()` (`tiers.rs:57`) — grounded, not invented (tighten as tier rises).
- `erasability` per C3: T1–T3 `erasable`, T4 `retained` (issuance-side D-088 tier-gate).
- **`TierNClaims` contact/jurisdiction fields = expression-only, unvalidated (A4)** — design decides whether the
  mock populates them (realism) or omits them; either way nothing validates them, so they are not load-bearing
  for any witness.

## 9. Out-of-scope boundaries (recorded, not audit-depth — brief Out)

- **MP-F13** (home-node discovery) → M10.4. **MP-F6 fold + MP-C-06 / MP-C-16 re-run** → M10.5.
- **Erasure *enforcement*** (the "T4 refuses erasure" consumer) → D3-gated (PG-02 / Fork 3). M10.3 issues +
  witnesses the *field* only (A6).
- **A "refuse mock in production-mode" node behaviour** → flagged future, not M10.3 (C4: `module_kind: mock` is
  expression-only; trust is the operator's `trusted_auth_modules`/registry choice).
- **The tier-graded invite-validity ceiling** (M8.5-B, `runtime.rs:221`) — a sibling dormant tier-path, not in
  C3's named scope.

## 10. Findings register (D-065)

| ID | Severity | Surface | Finding | Routing |
|---|---|---|---|---|
| **M10.3-A1** | **S2 (load-bearing)** | A1 policy shape | **C2 needs the issuer record threaded into `validate_assertion`.** The policy carries only `trusted_issuers: HashSet<String>` (URIs); M10.2's live-read does not surface `accepted_tiers`. C2 requires a new `AuthModuleRegistry::accepted_tiers_by_issuer()` + an `AssertionPolicy.accepted_tiers_by_issuer` field, derived live at the gate beside `trusted_issuers` (the M10.2-A1 prediction realized). | **M10.3 design** — lock the policy-shape addition + the registry method (recommend; mirrors the M10.2 live-read pattern). |
| **M10.3-A2** | **S2 (load-bearing)** | A1/A7 semantics | **Empty `accepted_tiers` must mean "unrestricted", or C2 regresses M10.2.** M10.2 seeds config issuers with empty `accepted_tiers`, and `trusted_issuers()` trusts them regardless. C2 reading empty as deny-all would reject every seeded/empty-tier issuer (breaks the empty-baseline + config-seed path). C2 must be restrictive-only (empty ⇒ no per-issuer restriction). | **M10.3 design** — lock restrictive-only semantics. Sub-question: do config-seeded issuers stay unrestricted, or should the seed default them to `[Tier1]` (a tightening)? Recommend stay-unrestricted (non-regressive). |
| **M10.3-A3** | S3 | A1/§7 wire code | **C2's reject needs a distinct wire code.** 3030 (`tier_mismatch`) means "tier below the node's required floor" (Step 4) — a *different* failure from "this issuer isn't authorized to attest this tier" (C2). Reusing 3030 would conflate two diagnostics. 3012 is a free dormant slot (RC-F-01 §3.11.7 band). | **M10.3 design** — Joe-call: new code (e.g. 3012 `issuer_tier_unauthorized`) vs reuse 3030. Lean new. |
| **M10.3-A4** | S3 (D-065 catch) | A4 claims | **`Tier2/3/4Claims` are dormant-unwired** (test-only; no production validator reads them). "Per-tier claims" the validation sees = the `tier` integer + `tier_verified` + `valid_until` (TTL). The richer contact/jurisdiction schemas are expression-only. The mock may populate them for realism but must not pretend they are validated. | **M10.3 design** — decide whether the mock populates `TierNClaims` (label them unvalidated) or omits them. No witness depends on them. |
| **M10.3-A5** | S4 (no action) | A3 witness shape | The Thread gate's Check 1 (`thread_tier ≥ space.auth_tier`) means a T2-required thread is creatable in a T1 Space (`2 ≥ 1`), so the participation-gate witness (T2 identity accepted / T1 rejected) runs in a T1 Space — no T2 Space needed. A clean grounding, recorded so the witness design picks the simplest setup. | **Noted** — witness-construction detail; the join gate (`runtime.rs:1393`) is the sibling second witness if breadth is wanted. |

*The four locked calls (C1–C4) hold — no finding reopens a fork. A1 + A2 are the load-bearing design calls; A3
is the wire-code; A4 is the dormant-schema honesty; A5 is a witness detail. **D-092 confirmed not triggered**
(the mock CLI is its own arg parser; `accepted_tiers` is set via the existing CRUD verbs — no new node verb).*

## 11. Design-shaping questions teed up for Joe (don't decide here — Joe locks at design)

1. **C2 insert-point + ordering** (§7 / M10.3-A1) — after Step 1, restrictive-only; the policy-shape addition.
   Recommendation: as sketched in §7.
2. **C2 reject wire-code** (M10.3-A3) — new code (3012 `issuer_tier_unauthorized`) vs reuse 3030. Lean new.
3. **Empty `accepted_tiers` for config-seeded issuers** (M10.3-A2) — stay unrestricted (recommend, non-regressive)
   vs seed-default `[Tier1]` (a tightening).
4. **Mock per-tier defaults** (§8 / M10.3-A4) — the table is grounded; the open call is whether to populate the
   dormant `TierNClaims` schemas (expression) or omit them.
5. **Mock CLI shape** (minor) — a `--tier <N>` flag on `issue` vs a `mock` subcommand.

## 12. Definition of Done (this audit)

- [x] Brief §3's six items each grounded to file:line on live `main` @ `52897e5` (A1–A6).
- [x] C2 insert-point + ordering sketch delivered (§7); the two gates confirmed distinct (A2).
- [x] Per-tier claim/TTL defaults grounded against `tiers.rs` `ttl_days` (§8) — not invented.
- [x] Blast radius grounded (single `validate_assertion` path + restrictive-only empty-baseline, A7).
- [x] Dormant-path activation grounded (Thread gate `runtime.rs:1503` + `assertion_tier_of` mechanism; sibling
      join gate; the `TierNClaims` dormant-unwired catch).
- [x] D-088 issuance-side boundary made explicit; enforcement attach point grounded, not built (A6).
- [x] Findings register populated — two load-bearing (M10.3-A1 policy shape, M10.3-A2 empty-semantics), one wire
      code (M10.3-A3), one honesty (M10.3-A4), one witness detail (M10.3-A5).
- [x] Locked-fork boundaries respected (C1–C4 not reopened; MP-F13 / M10.5 items / erasure-enforcement /
      mock-refusal recorded out, §9). D-092 confirmed not triggered.
- [x] Header v1.0, Status ACTIVE.
- [ ] Audit committed (Clair's commit precedes Chat's doc-bridge; Joe pushes).

**Next after this audit:** M10.3 design — lock the C2 shape (§7: policy-field + registry method + restrictive-only
+ reject code) + the mock parameterization (§8) + the witness set (T2/T3/T4 accepted; T2-issuer's T3 rejected via
C2; Thread participation gate fires; T4 `retained`; `module_kind: mock`; empty-baseline) + the §11 deferred
questions → Joe-lock → runbook → impl → close → M10.4 (MP-F13).

## 13. Close disposition (J-368)

M10.3 shipped + closed (J-368). Findings final state:

- **M10.3-A1 — RESOLVED.** D1 landed the live-read: `AuthModuleRegistry::accepted_tiers_by_issuer()` +
  `AssertionPolicy.accepted_tiers_by_issuer`, derived at the gate beside `trusted_issuers`.
- **M10.3-A2 — RESOLVED.** D2 shipped restrictive-only (empty/absent ⇒ unrestricted); empty-baseline invariant
  held byte-for-byte; config-seeded issuers stay unrestricted (non-regressive, as recommended).
- **M10.3-A3 — RESOLVED-with-correction (D-065).** The audit's “3012 is a free dormant slot” call was **wrong**:
  ch3 §3.11.7 had already assigned **3012 = `watchlist_match`** (L3858). The wire-code direction holds (a
  distinct auth-module-band code, not 3030), but the integer moved to **3032** (Joe-locked, J-367), adjacent to
  the 3030/3031 tier-authz sub-band. A genuine cross-file grounding miss the close caught before it shipped a
  double-definition — the value of the D-065 close-check.
- **M10.3-A4 — boundary recorded.** The dormant `Tier2/3/4Claims` schemas were **not** populated (no production
  reader = theater); the mock issues the tier integer + grounded TTL + descriptor only. Richer schemas remain a
  flagged future.
- **M10.3-A5 — no action** (witness-construction detail; the T1-Space participation-gate setup was used as noted).
