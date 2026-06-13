# M10.1 — Wire-Band Reconcile + AI-D8 Module-Policy Descriptor — Implementation Runbook
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jun 2026  
> **Last updated**: 2026-06-13  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. Status & lineage

Executes `tasks/M10_1_WIREBAND_DESCRIPTOR_DESIGN.md` (v1.0, M10.1-D1..D4 **Joe-LOCKED J-360**), grounded on the M10
D-071 Phase-0 audit (`tasks/M10_AUTH_MODULE_AUDIT.md` v1.0, `f759a7d`). Authored by Clair after confirming the §4
impl-surface groundings to file:line on live `main` @`2698f43` (= audit base `5d8fec1` + the M10.1 design commit;
the audit's file:line refs re-confirmed below, all hold). Production arc (`xgen-common` + `xgen-core` + ch3 spec);
small surface; **no node/client behaviour change** (no enforcement consumer this arc — confirmed §2.6).

D1–D4 are locked; **do not reopen them.** The runbook adds the grounded *how* + the one design-close detail
(§4, the `erasability` member shape — grounded, spec-it-and-note, flagged for Joe at bridge, non-blocking).

**Commit order:** this runbook commit precedes the impl commit (Joe pushes both). The impl commit is one atomic
(D-074): code (xgen-common + xgen-core) + ch3/appendix spec edits + the four witnesses, together.

## 1. Scope

**In (locked):**
- **D1** — RC-F-01 renumber. Spec correction (ch3 §3.11.7) + **one reserved constant** (3031). Zero change to
  emitted codes.
- **D3** — AI-D8 module-policy descriptor at `claims.extra["module_policy"]` (signed, namespaced; first member
  `erasability`).
- **D4** — mock self-label at `claims.extra["module_kind"]` ∈ {`reference`, `mock`} (signed).

**Out (locked):**
- **D2** — the 7 event-validation codes (`exchange.rs:140` `_ => None` → 4000) are the **MP-F2-followon sibling
  decision**. NOT touched here. (Carried at close as M10-A-03.)
- The erasure **consumer** ("T4 refuses erasure" enforcement) stays D3-gated (Fork 3 = hook only). M10.1 lands
  only the *descriptor field*; **no default-by-tier resolution, no tier-gate read** (see §4).
- `assertion_tier_of` / `validate_assertion` / `to_registration_code` emitted arms — **untouched**.

## 2. Groundings confirmed (file:line, live `main` @`2698f43`)

### 2.1 Band sites (D1)
- **3010 / 3011 / 3030 emit** = `xgen-core/src/identity/registration.rs:120–122` — the `to_registration_code`
  match arms (confirmed verbatim):
  - `:120` `AssertionIdentityMismatch => (3010, "assertion_identity_mismatch")`
  - `:121` `AssertionClaimsInsufficient => (3011, "assertion_claims_insufficient")`
  - `:122` `AssertionTierInsufficient => (3030, "tier_mismatch")`
- **3030 is emitted at TWO sites** — `registration.rs:122` (registration check 4) **and**
  `xgen-core/src/auth/tiers.rs:144` (`AuthError::TierMismatch | UnknownTier => Some((3030, "tier_mismatch"))`,
  the join PG-13 gate). So D1's "auth_tier_insufficient **folds into** 3030" is a **pure spec fold** — the code
  already emits 3030 for tier-insufficiency at both gates; **no code change for the fold.**
- **ch3 §3.6.5** = `docs/xgen_ch3_specification.md` L1896–1912; the 3010/3011 rows L1911–1912 (Arc-E, **KEEP**).
- **ch3 §3.11.7** = L3825–3839; the colliding rows L3833 (`3010 auth_tier_insufficient`) + L3834
  (`3011 kyc_verification_pending`); 3012–3016 at L3835–3839; **L3829 reservation note** confirmed verbatim
  ("Codes 3010–3016 cover higher-tier Auth Module errors (this section)…").
- **3031** — grep of `xgen-common`/`xgen-core`/`xgen-node`/`docs` → **unused anywhere**. Free to reserve.

### 2.2 Descriptor home (D3/D4)
- **`TrustClaims`** = `xgen-common/src/trust_assertion.rs:91`; the open-namespace member
  `#[serde(flatten)] extra: BTreeMap<String, Value>` at `:105–106`. `has_claim` (`:115`) already consults `extra`.
- **`TrustAssertion`** = `:140`; `claims: TrustClaims` (`:158`); `signature: Option<String>` (`:162`, excluded
  from canonical bytes).
- **Canonical signer** = `canonical_bytes` (`:170`) → `canonical_object_json(value, TRUST_ASSERTION_FIELDS)`
  (`xgen-common/src/canonical.rs:48`). For the `claims` field it calls `canonical_value(v)` (`canonical.rs:57`),
  and `canonical_value` (`:81`) **recursively sorts every nested object key** (`:83–92`). **⇒ any member added to
  `claims.extra`, at any depth, is part of the signed bytes.** This is the load-bearing fact for witness 1.
- `serde_json::Value` + `BTreeMap` already imported in `trust_assertion.rs` (the `extra` field uses them).
- **`TRUST_ASSERTION_FIELDS`** (`:56`) is a fixed top-level set — confirms AE-D5: a *new top-level field* would be
  wrong; the descriptor lives **inside `claims`**.

### 2.3 Test anchors
- Band-reconcile witness anchor: `registration.rs:1249` `validate_assertion_rejects_identity_mismatch`, asserts
  `(3010, "assertion_identity_mismatch")` at `:1255` (xgen-core).
- Descriptor witness home: `trust_assertion.rs:232` `mod tests` (helpers `test_signing_key` `:239`,
  `signed_assertion` `:255`; existing `sign_verify_round_trip` `:326`) (xgen-common).

### 2.4 Erasability-model grounding (Arc-I / D-088)
- `tasks/ARC_I_ERASURE_DESIGN.md` **AI-D8** (L48) + **AI-D4** (L40) + **D-088** (`DECISIONS.md:46`): the
  module-policy descriptor is forward-extensible (erasability = **first** member, unknowns preserved verbatim);
  erasability is **monotonic in tier with protocol-fixed endpoints** — **T1 = max-erasable**, **T4 = no record
  destruction**, **T2/T3 = module-declared** in between. The descriptor is "carried on the Trust Assertion."
- The eventual consumer pairs the descriptor with `assertion_tier_of`
  (`xgen-core/src/node/runtime.rs:214`) — **NOT wired this arc** (no enforcement; default-by-tier is the
  consumer's call, §4).

### 2.5 Crates
- `xgen-common` — `TrustClaims` accessors + the descriptor types + descriptor witnesses (1, 2, 4).
- `xgen-core` — the reserved `3031` constant (the wire-code conceptual home, next to `RegistrationError`) +
  band-reconcile witness (3).
- ch3 (§3.6.5 unchanged / §3.11.7 + L3829 reconciled / §3.8.4 descriptor documented) + Appendix C/I as applicable
  (confirm home by grep at impl, §6).

### 2.6 No-behaviour-change confirmation
All changes are **additive**: new `pub` types + accessor methods (no existing call-site touched), one new `pub`
const (no emitter), doc edits. `assertion_tier_of`, `validate_assertion`, `to_registration_code`'s emitted arms,
and every existing TrustAssertion/registration test are **untouched**. Expectation: `cargo test --workspace`
shows zero existing-test outcome changes; only the new witnesses add. **If any existing test changes outcome →
STOP and surface (D-065)** — it would mean a behaviour change the design did not anticipate.

## 3. Commit plan (one atomic impl commit, D-074)

Order within the commit is build-clean-incremental; all land together.

**Step A — descriptor types + accessors (xgen-common, `trust_assertion.rs`).** Add, alongside `TrustClaims`:

```rust
/// Auth-Module self-label (D4 / AI-A7). Expression only — trust is the
/// `trusted_auth_modules` gate (M10-A-04), never this label.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModuleKind { Reference, Mock }

/// AI-D8 module-policy descriptor (D3). Forward-extensible: `erasability` is the
/// first member; unknown members are preserved verbatim (§8 open-doors).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ModulePolicy {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub erasability: Option<Erasability>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,   // forward module-policy members, verbatim
}

/// Erasability sub-descriptor. Itself forward-extensible (`retention` + unknowns).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Erasability {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retention: Option<Retention>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// AI-D4 protocol-fixed endpoints. T2/T3 modules declare one of these; the
/// interpretation of *absence* (default-by-tier) is the deferred consumer's call.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Retention { Erasable, Retained }
```

Typed accessors on `TrustClaims` (the "not raw map fiddling" requirement, design §3):

```rust
impl TrustClaims {
    pub const MODULE_KIND_KEY: &'static str = "module_kind";
    pub const MODULE_POLICY_KEY: &'static str = "module_policy";

    /// D4 read. Absent OR unparseable ⇒ `Reference` (the confirmed default —
    /// the project's reference build is the unlabelled baseline).
    pub fn module_kind(&self) -> ModuleKind { /* extra.get(KEY) → from_value → unwrap_or(Reference) */ }

    /// D4 write — sets the signed self-label.
    pub fn set_module_kind(&mut self, kind: ModuleKind) { /* extra.insert(KEY, to_value(kind)) */ }

    /// D3 read. `None` = absent (lenient: present-but-malformed also `None` —
    /// expression-only arc, the D3 consumer defines strictness). See §4.
    pub fn module_policy(&self) -> Option<ModulePolicy> { /* extra.get(KEY).and_then(from_value(clone).ok()) */ }

    /// D3 write.
    pub fn set_module_policy(&mut self, policy: &ModulePolicy) { /* extra.insert(KEY, to_value(policy)) */ }
}
```

- These are pure struct/serde additions; no `TrustAssertion` field change (AE-D5); `TRUST_ASSERTION_FIELDS`
  untouched (the descriptor rides `claims`, already in the field set).
- Re-export the four types from `xgen-common` as the existing `trust_assertion` items are (confirm the crate's
  re-export convention at impl; mirror `TrustAssertion`/`TrustClaims`).

**Step B — reserved 3031 constant (xgen-core, `registration.rs`).** Near `RegistrationError` /
`to_registration_code`:

```rust
/// RC-F-01 (M10.1-D1): `kyc_verification_pending` re-homed out of §3.11.7's old
/// 3011 collision to **3031** (adjacent to 3030 `tier_mismatch` — tier/KYC is one
/// domain). RESERVED / dormant — no emitter this arc (no Tier-3/4 KYC gate exists
/// yet). The `to_registration_code` map is UNCHANGED (zero change to emitted codes).
pub const ASSERTION_KYC_VERIFICATION_PENDING: (u32, &str) = (3031, "kyc_verification_pending");
```

- `(u32, &str)` mirrors `to_registration_code`'s tuple shape. `pub` ⇒ no `dead_code` warning. No new
  `RegistrationError` variant (a variant would imply an emit path that does not exist).

**Step C — ch3 + appendix spec reconcile (D1 + descriptor doc).** §6.

**Step D — the four witnesses.** §5.

## 4. The one design-close detail — `erasability` member shape (grounded; spec-it-and-note)

Design §3 flagged the `erasability` shape (enum vs object; default by tier) as the single Joe-call-if-non-obvious
detail. Grounded against Arc-I AI-D8/AI-D4 + the no-enforcement-this-arc scope, the call is **obvious**, so it is
specced here and **flagged for Joe at the J-361 bridge (non-blocking veto)**:

1. **Object, not flat enum.** `erasability: { "retention": "erasable" }` — an object — not `erasability:
   "erasable"`. Reason: AI-D8's defining property is forward-extensibility ("unknown future module requirements
   have a home without a protocol change"). A flat enum is a dead-end (adding e.g. a retention period later breaks
   the member's type); an object is itself extensible (verbatim `extra` one level down), mirroring the §8
   open-doors principle inside the descriptor. Matches the design §3 schema literally.
2. **`retention` ∈ {`erasable`, `retained`}** = exactly AI-D4's two **protocol-fixed endpoints** (T1 max-erasable
   / T4 no-destruction). A T2/T3 module declares one of these, bounded by the gradient. Minimal + honest; no
   speculative third value.
3. **No default-by-tier in M10.1.** The design §3 asked "default by tier?" — the answer is **not this arc.**
   Resolving *absence* of the descriptor into an erasability posture is **enforcement** (it pairs with
   `assertion_tier_of` and the tier-gradient), which is the D3-gated consumer's job (Fork 3 = hook only). M10.1
   ships **expression only**: the accessor returns `Option<ModulePolicy>` (`None` = absent); interpreting `None`
   by tier is deferred. This keeps the locked "no enforcement consumer this arc" honest.

This is within the locked scope — not a reopening of D1–D4. If Joe disagrees with the object shape or the
two-value `retention` at bridge, it is a one-struct change; surfaced so he can veto.

## 5. Proof obligations — the four witnesses (RED-on-revert; hard deliverables)

Each must fail if its mechanism is reverted (genuine RED-on-revert, recorded).

1. **Sign/verify roundtrip covers the descriptor** (xgen-common, `trust_assertion.rs` mod tests).
   Build a signed assertion with `set_module_kind(Mock)` + `set_module_policy({erasability:{retention:Erasable}})`;
   `verify()` passes. Then tamper a byte of the serialised `module_policy` (or flip `module_kind` post-sign) →
   `verify()` **fails**. RED-on-revert = the members are signature-covered. (Grounded by §2.2: canonical_value
   recurses.)
2. **§8 open-doors — unknown member survives + verifies** (xgen-common). Construct a `ModulePolicy` with an
   **unknown** `extra` member (e.g. `policy.extra.insert("future_member", json!("x"))`) AND an unknown member
   inside `Erasability.extra`; `set_module_policy` → sign → serialise → deserialise → read back via
   `module_policy()` → assert the unknown members are present (verbatim) AND `verify()` still passes. RED if the
   typed `ModulePolicy`/`Erasability` flatten-extra is removed (member dropped → verify still passes but the read
   loses it; assert presence to catch the drop).
3. **Band reconcile** (xgen-core, `registration.rs` mod tests + a doc assertion). Assert:
   `RegistrationError::AssertionIdentityMismatch.to_registration_code() == (3010, "assertion_identity_mismatch")`
   and `AssertionClaimsInsufficient` `== (3011, …)` and `AssertionTierInsufficient` `== (3030, "tier_mismatch")`
   (all **unchanged**); and `ASSERTION_KYC_VERIFICATION_PENDING == (3031, "kyc_verification_pending")` (reserved,
   distinct from the band). The existing `validate_assertion_rejects_identity_mismatch` (`:1249`) stays green
   unmodified = the wire-emit is unbroken. (Doc half: §3.11.7 no longer claims 3010/3011 — verified by §6 edit.)
4. **Mock-label accessor** (xgen-common). `module_kind()` on an assertion built with `set_module_kind(Mock)`
   returns `Mock` (signed value); on an assertion with **no** `module_kind` key returns `Reference` (the confirmed
   default). RED if the default flips or the read ignores `extra`.

## 6. Close deliverables (in the impl commit — Clair)

- **ch3 §3.11.7 reconcile (D1):** delete the colliding 3010 (`auth_tier_insufficient`) + 3011
  (`kyc_verification_pending`) rows; add a note that tier-insufficiency emits **3030 `tier_mismatch`** (the live
  code, see §3.6.5 impl-note + the membership/PG-13 gate); add **3031 `kyc_verification_pending`** as
  reserved/dormant (Tier 3/4, no emitter yet); 3012–3016 unchanged. **Update the L3829 reservation note** so its
  sub-range map is correct (3010–3011 = Arc-E identity-assertion §3.6.5; 3012–3016 = higher-tier; 3030–3031 =
  tier/KYC, 3030 live + 3031 reserved; 3020–3023 = replication).
- **§3.6.5** — unchanged (3010/3011 stay).
- **Descriptor schema documented:** ch3 **§3.8.4** (the TrustClaims `claims.extra` schema — the audit's A4 home) —
  document `module_kind` ∈ {reference, mock} + `module_policy.erasability.retention` ∈ {erasable, retained} as
  recognised open-namespace members; cite AI-D8 forward-extensibility. **Appendix C/I:** confirm by grep at impl
  whether either carries a TrustClaims/TrustAssertion schema entry; if so, add the descriptor; if not, skip
  (don't manufacture a home). **Appendix F: none** — no operator-visible CLI surface changes this arc (confirmed:
  module_kind is populated by M10.3's mock, not a M10.1 verb).
- **DoD / verification** captured per §7 with actual `cargo` output (Rule 2/5).

**NOT Clair (Chat bridge, J-361):** `CLAUDE.md` PLAY, `JOURNAL.md` J-361, `docs/ROADMAP.md`, `DECISIONS.md`
(candidate only — "module-policy lives in a signed `claims.extra` namespace, never a new top-level field"; +
possibly "retention endpoints are protocol-fixed, interior module-declared" — arc-local D-069, none pre-decided),
and the audit findings register flip (**M10-A-01 → RESOLVED**; **M10-A-03 → carried** as the MP-F2-followon
sibling decision).

## 7. Definition of Done — SHIPPED

- [x] Groundings §2 re-confirmed at impl start — all held on `2698f43` (3010/3011/3030 at registration.rs:120–122;
      3030 also at tiers.rs:144 ⇒ fold is spec-only; ch3 §3.6.5 L1911–12 / §3.11.7 L3833–34 / L3829;
      TrustClaims.extra trust_assertion.rs:105–106; canonical_value recursion canonical.rs:81).
- [x] Step A: `ModuleKind` / `ModulePolicy` / `Erasability` / `Retention` + four accessors
      (`module_kind`/`set_module_kind`/`module_policy`/`set_module_policy`) in `xgen-common/src/trust_assertion.rs`;
      re-exported from `xgen-common/src/lib.rs`. No `TrustAssertion` field change (AE-D5).
- [x] Step B: `ASSERTION_KYC_VERIFICATION_PENDING = (3031, "kyc_verification_pending")` in `xgen-core`
      `registration.rs`; `to_registration_code` emitted arms **unchanged** (zero emitted-code change).
- [x] Step C: ch3 §3.11.7 reconciled (3010/3011 collision rows removed; RC-F-01 note added; 3030 live + 3031
      reserved rows added; L3829 sub-range note corrected); §3.8.4 open-namespace descriptor documented; §3.6.5
      unchanged. **Appendix C/I: not applicable** — C references `TrustClaims` as a type but enumerates no members;
      I has no TrustClaims entry (no manufactured home). **Appendix F: none** (no operator-visible surface).
- [x] The four §5 witnesses present, each with a **genuine RED-on-revert recorded** (see "RED-on-revert results").
- [x] `cargo build --workspace --all-targets` — 0 errors (32.84s).
- [x] `cargo clippy --workspace --lib --tests -- -D warnings` clean **and** `--all-features` clean (one
      `unnecessary_get_then_check` nit fixed: `.get(…).is_none()` → `!….contains_key(…)`).
- [x] `cargo test --workspace` — **1374 passed, 0 failed** (all buckets). Pre-work 1370 ⇒ delta **+4** = the four
      witnesses only (xgen-common lib 141→**144**, xgen-core lib 693→**694**). No existing-test outcome change.
- [x] No node/client behaviour change confirmed — `assertion_tier_of` / `validate_assertion` /
      `to_registration_code` emitted arms untouched; all changes additive. §2.6 expectation held (not falsified).
- [x] Status: COMPLETED — the shipped signal (this DoD lists no "commit pushed"; Joe pushes).

### RED-on-revert results (each reverted in the working tree, observed RED, restored)

| Witness | Revert applied | Observed RED |
|---|---|---|
| 1 sign/verify covers descriptor | drop `"claims"` from `TRUST_ASSERTION_FIELDS` (descriptor leaves signed bytes) | `module_descriptor_is_signature_covered`: tamper → `left: Ok(()) right: Err(SignatureInvalid)` |
| 2 §8 open-doors | `ModulePolicy.extra` `#[serde(flatten)]` → `#[serde(skip)]` | `module_policy_unknown_members_round_trip`: `left: None right: Some({"k":1})` (unknown member dropped) |
| 3 band reconcile | const `3031` → `3017` | `band_reconcile_codes_unchanged_and_kyc_reserved`: `left: (3017,…) right: (3031,…)` |
| 4 mock-label default | `module_kind()` default `Reference` → `Mock` | `module_kind_accessor_reads_signed_value_and_defaults`: absent → `left: Mock right: Reference` |

### Erasability design-close detail (flagged for Joe at J-361 bridge — non-blocking)

Per §4: `erasability` is an **object** (`{retention}`) not a flat enum (forward-extensible, AI-D8); `retention` ∈
{`erasable`,`retained`} = AI-D4's protocol-fixed endpoints; **no default-by-tier in M10.1** (enforcement →
deferred D3 consumer; M10.1 is expression-only). Grounded against Arc-I/D-088; specced not guessed. If Joe vetoes
the object shape or the two-value `retention`, it is a one-struct change.

## 8. Next

Clair implements per §3 → Chat doc-bridge **J-361** (the §6 "NOT Clair" set + findings flip) → M10.1 closes →
**M10.2** (the T1 reference binary, where M10-A-02's config-seam-vs-registry→policy call lands with Joe).
