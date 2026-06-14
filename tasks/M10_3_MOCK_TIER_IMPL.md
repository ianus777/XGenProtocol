# M10.3 — Parameterized T2–T4 Mock + `accepted_tiers` Enforcement + Dormant-Tier Activation — Implementation Runbook
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-14  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. Status & lineage

Executes the M10.3 design (`tasks/M10_3_MOCK_TIER_DESIGN.md` v1.0, J-366). D1–D5 **Joe-LOCKED — not reopened.**
Third + heaviest M10 sub-arc. Production arc: `xgen-core` (the C2 substrate + 3012 + the dormant-gate witness),
`xgen-node` (the gate derivation = the behaviour flip), `xgen-auth-module` (the mock `--tier` path). Runbook
commit precedes the impl commit(s); Joe pushes both. No code until this runbook lands.

## 1. Grounding confirmed (file:line on `main` @ `41de74d`)

Re-grounded fresh (the J-366 design commit was doc-only; line numbers re-verified per D-078).

**The C2 seam (single, narrow — same as M10.2).** `validate_assertion` (`registration.rs:211`) has **one**
production caller (`registration.rs:486`, via `accept_registration`), reached from **one** node path
(`handle_identity_msg`, `app.rs:2862`; the gate block `:2879–2895`). The 7-check body: Step 1 issuer-trusted
(`:218`), Step 4 node-wide `tier ≥ required_tier` (`:230`, → 3030 `tier_mismatch`).

**The trust source (the M10.2 live-read site).** The gate clones `rt.assertion_policy`, clones the registry Arc,
and sets `policy.trusted_issuers = registry.lock().await.trusted_issuers()` (`app.rs:2895`). This is exactly
where C2 adds `policy.accepted_tiers_by_issuer = registry.…accepted_tiers_by_issuer()` (D1).

**The registry.** `AuthModuleRecord.accepted_tiers: Vec<AuthTier>` (`module_registry.rs:59`), set by the existing
CRUD verbs (`auth-module register --tier …`, `set-tiers`). The new method goes right after `trusted_issuers()`
(`module_registry.rs:149`): `accepted_tiers_by_issuer() -> HashMap<String, Vec<AuthTier>>` over **non-revoked**
records (`module_id.to_string() → accepted_tiers.clone()`).

**`AssertionPolicy`** (`registration.rs:163`): `trusted_issuers: HashSet<String>` + `required_claims` +
`required_tier`. Field-add blast radius is **3 literal sites** + 1 derivation: `Default` (`:176`),
`policy_trusting` test helper (`:1231`), the xgen-node snapshot (`app.rs:809`), and the gate derivation
(`app.rs:2895`). `AssertionPolicy::default()` callers ride the Default (unaffected). `AuthTier` is xgen-core
(`tiers.rs:35`); the new `HashMap<String, Vec<AuthTier>>` field needs no new dep.

**The 3012 reject is a *registration* error, NOT a dispatch `RejectInfo` (D-065 precision on D3's wording).**
A registration reject flows `validate_assertion → RegistrationError → accept_registration → handle_identity_msg
Err-arm → `e.to_registration_code()` → wire `Error { error_code }` (`app.rs:~2944`). So 3012 = a new
`RegistrationError::AssertionTierUnauthorized` variant (`registration.rs:77`) + a `to_registration_code` arm
(`:111`) `=> (3012, "assertion_tier_unauthorized")`. The wire plumbing is automatic (generic over the enum).
`RejectInfo`/MP-F2 is the **dispatch_event** path — that is the Thread gate's 3030 (below), a different
mechanism for a different gate. The design's "RejectInfo path (MP-F2)" reads generically as "the reject
machinery"; the precise C2 mechanism is `to_registration_code`. **Not a re-lock** — the code lands; the
mechanism is the correct registration-reject path.

**The Thread participation tier-gate (dormant activation, D4).** `dispatch_event` `EventType::ThreadCreate` arm
(`runtime.rs:1485–1512`): `thread_tier = content["auth_tier_min"]` (`:1487`); Check 1 `thread_tier <
space.auth_tier` → 3030 `thread_auth_tier_below_room` (`:1488`); `creator_tier = assertion_tier_of(...)`
(`:1498`); **Check 2** `verify_tier_assertion(creator_tier, thread_tier)` (`:1503`) → 3030 `tier_mismatch`.
`assertion_tier_of` (`runtime.rs:214`) reads the **validated** stored tier (`record.trust_assertion["tier"]`,
`:217`). The **reject side is already tested** (`pg08_thread_create_above_creator_tier_rejected_3030`,
`runtime.rs:4544` — a T1 creator's T2 thread rejected 3030). What's missing = the **accept side** (a T2-record
creator's T2 thread accepted) — witness 4. Test pattern grounded: `setup_space_with_room()` (`:3440`),
`alice_thread_create(node, space, room, key, auth_tier_min)` (`:4486`), and the tier-2 record pattern from
`pg13_tier2_join_into_tier2_space_accepted` (`:4239`, `rec.trust_assertion = Some(json!({"tier": 2}))`).

**Per-tier TTL (grounded, D4).** `AuthTier::ttl_days()` (`tiers.rs:57`): T1 `None`, T2 **365** (`:22`), T3 **180**
(`:23`), T4 **90** (`:24`). `from_u32` (`:43`), `as_u32` (`:53`).

**The mock issue base (D5).** `xgen_auth_module::issue_tier1(module_key, identity_id, valid_until)` (`lib.rs:51`)
→ tier 1, `ModuleKind::Reference`, `Retention::Erasable`. Descriptor accessors `set_module_kind`/`set_module_policy`
shipped M10.1 (`trust_assertion.rs`).

## 2. Architecture (the §5 design-close details, resolved — none forks)

**A. The C2 policy field + live-derivation.** Add `AssertionPolicy.accepted_tiers_by_issuer: HashMap<String,
Vec<AuthTier>>`, `Default` empty. Derived **live at the gate** beside `trusted_issuers` (`app.rs:2895`) from
`AuthModuleRegistry::accepted_tiers_by_issuer()` (non-revoked) — mirrors the M10.2 `trusted_issuers()` pattern
exactly, registry stays single source of truth.

**B. The C2 check (after Step 1, restrictive-only).** In `validate_assertion`, right after Step 1
(`registration.rs:218`):
```
// Step 1.5 (C2) — the issuer is authorized to attest THIS tier (per-issuer scope).
if let Some(tiers) = policy.accepted_tiers_by_issuer.get(&assertion.issuer) {
    if !tiers.is_empty() && !tiers.iter().any(|t| t.as_u32() == assertion.tier) {
        return Err(RegistrationError::AssertionTierUnauthorized);  // 3012
    }
}
```
**Restrictive-only (D2):** empty/absent tiers ⇒ skip (the issuer may attest any tier). This is what keeps the
M10.2 empty-baseline byte-for-byte. Position: after Step 1 (issuer must be trusted before its tier-scope
matters); independent of Step 4 (per-issuer set-membership vs node-wide ordered floor — distinct, no
double-reject, A2 of the audit). Place it right after Step 1 for clearest reading.

**C. 3012 = a new `RegistrationError` variant + `to_registration_code` arm** (§1). Distinct from
`AssertionTierInsufficient → 3030` (`registration.rs:97/:122`). No `RejectInfo` change (that path is the Thread
gate's 3030, unchanged).

**D. Mock issuance (D4/D5).** Generalize the lib: `issue(module_key, identity_id, tier: AuthTier, valid_until)`
sets `module_kind = (tier == Tier1 ? Reference : Mock)` and `erasability.retention = (tier == Tier4 ? Retained :
Erasable)`; `issue_tier1` becomes a thin `issue(.., Tier1, ..)` wrapper (M10.2 witnesses + `--tier 1` unchanged).
The CLI: `--tier <N>` (default 1) on the existing `issue`; `valid_until = now + ttl_days(tier).unwrap_or(<T1
default 365>)` days (a `--valid-days` override stays if present). N ∈ {2,3,4} ⇒ auto mock + grounded TTL +
tier-appropriate erasability. **No new subcommand; D-092 not triggered** (the binary CLI is its own arg parser;
trust is still operator-set via the existing CRUD verbs).

**E. Witness 4 (Thread accept side).** Add a runtime test mirroring `pg13_tier2_join_into_tier2_space_accepted`:
a creator whose record carries `trust_assertion: {"tier": 2}` creates a T2 thread (`auth_tier_min: 2`) in a T1
Space (Check 1 passes `2 ≥ 1`) → `verify_tier_assertion(2, 2) = Ok` → **accepted** + thread inserted. The
existing `pg08_thread_create_above_creator_tier_rejected_3030` is the RED side.

## 3. Commit plan (3 work commits; each builds clean + workspace-green)

**C1 — xgen-core: the C2 substrate + the dormant-gate accept witness (behaviour-neutral).**
- `AuthModuleRegistry::accepted_tiers_by_issuer()` (`module_registry.rs`, after `trusted_issuers()`).
- `AssertionPolicy.accepted_tiers_by_issuer` field + `Default` empty (`registration.rs`); update the
  `policy_trusting` helper (`:1231`) + the xgen-node snapshot literal (`app.rs:809`, `HashMap::new()`).
- `RegistrationError::AssertionTierUnauthorized` + `to_registration_code` arm `(3012, "assertion_tier_unauthorized")`.
- The C2 check in `validate_assertion` after Step 1 (restrictive-only, B).
- **Behaviour-neutral in production:** the gate does NOT yet derive the map (C3), so production policies carry an
  empty `accepted_tiers_by_issuer` ⇒ C2 no-op. The check is exercised by unit tests with hand-built policies.
- Witnesses (unit): `accepted_tiers_by_issuer` (per-issuer, revoke-aware); `validate_assertion` C2 (T2-scoped
  issuer's T3 → 3012; T2 → ok; **empty → unrestricted**, the empty-baseline at the validate level); **witness 4**
  the Thread accept side (E) + the existing reject pin. DoD: build 0; clippy clean (default + all-features);
  `cargo test --workspace` green.

**C2 — xgen-auth-module: the parameterized mock `--tier` path.**
- Generalize `issue_tier1` → `issue(.., tier: AuthTier, ..)` (D); keep `issue_tier1` as the wrapper. `--tier <N>`
  on the CLI `issue` (default 1; grounded TTL per tier). Dep `xgen-core` for `AuthTier`/`ttl_days` (already a
  dep).
- Witnesses: **lib unit** — `issue --tier 2|3|4` produces tier N + grounded TTL + `module_kind: mock` (witness 5)
  + tier-appropriate erasability (T2/T3 erasable, **T4 retained**); `--tier 1` byte-identical to `issue_tier1`
  (witness: reference unchanged). **integration** (`tests/end_to_end.rs`, extend M10.2's): witness 1 (mock issues
  T2 → registry trusts the issuer for [Tier2] → `accept_registration` against a policy with
  `accepted_tiers_by_issuer` = {issuer: [Tier2]} → **accepted**); **witness 2** (issuer scoped [Tier2], mock
  issues **T3** → `accept_registration` → **3012**; its T2 accepted; RED on revert). DoD: `cargo build -p
  xgen-auth-module`; tests green.

**C3 — xgen-node: the gate derivation (the behaviour flip) + empty-baseline regression.**
- At the gate (`app.rs:2895`, in the `if let Some(registry)` block): add `policy.accepted_tiers_by_issuer =
  registry.lock().await.accepted_tiers_by_issuer();` (or fold both derivations into one lock). Now C2 bites for
  operator-scoped issuers.
- Witnesses (node): **empty-baseline** — a node whose registry issuers all have empty `accepted_tiers` (the
  M10.2 install base) accepts any tier (C2 invisible); the M10.2 seed/empty witnesses stay green (regression
  lock). DoD: build 0 (default; no `harness-control` rebuild needed — run `cargo test --workspace` normally);
  clippy clean (default + all-features); `cargo test --workspace` green (record the count delta).

## 4. Witnesses (RED-on-revert; the design's five)

1. **Mock issuance + accept** (C2 lib + integration) — `issue --tier 2|3|4` → valid signed assertion (tier +
   grounded TTL + `module_kind: mock` + erasability, T4 `retained`); accepted by a node trusting the issuer for
   that tier. RED: un-trust / wrong scope.
2. **C2 `accepted_tiers` scope** (C1 unit + C2 integration) — a T2-scoped issuer's **T3** assertion rejected
   **3012**; its T2 accepted. Distinct from a Space-floor 3030. RED: revert C2 → T3 accepted.
3. **Empty-baseline invariant** (C1 unit + C3 node) — empty `accepted_tiers` ⇒ any tier accepted; the M10.2
   seeded/empty path unchanged byte-for-byte. RED: empty-as-deny-all → T1 rejected.
4. **Thread participation tier-gate** (C1 runtime) — a T2-record creator's T2 thread **accepted** (the accept
   side, now reachable via a higher-tier identity); the T1-creator reject pin holds. RED: creator T1 → rejected.
5. **`module_kind: mock` populated** (C2 lib) — issued mock assertions carry the label. RED: not set.

## 5. Definition of Done

- [ ] C1: `accepted_tiers_by_issuer()` + `AssertionPolicy` field (Default empty) + C2 check (after Step 1,
      restrictive-only) + `AssertionTierUnauthorized`/3012; behaviour-neutral (gate not yet deriving the map);
      unit witnesses 2/3 + witness 4 (Thread accept) green.
- [ ] C2: `xgen-auth-module issue --tier <N>` parameterized (mock for N∈{2,3,4}, grounded TTL/erasability,
      `module_kind: mock`); `--tier 1` byte-identical to reference; lib + integration witnesses 1/2/5 green.
- [ ] C3: gate derives `accepted_tiers_by_issuer` live (the behaviour flip); empty-baseline node regression
      green (M10.2 install base unchanged).
- [ ] All five witnesses carry a genuine RED-on-revert.
- [ ] `cargo build --workspace --all-targets` 0; clippy `-D warnings` clean (default + all-features);
      `cargo test --workspace` green (record the count delta over the 1382 M10.2 baseline).
- [ ] Empty-baseline prime invariant asserted: C2 invisible when no issuer has a non-empty `accepted_tiers` (the
      entire M10.2 install base) — the regression guard.
- [ ] D-092 confirmed not triggered (no node verb surface change; the mock CLI is the binary's own arg parser;
      `accepted_tiers` set via the existing CRUD verbs). No DECISIONS change (M10.3-D# arc-local, D-069).

*(Runbook DoD never lists "commit pushed" — Status: COMPLETED is the shipped signal. Clair's runbook commit
precedes the impl commits; Joe pushes. Next: Chat doc-bridge J-367 — close deliverables §6.)*

## 6. Close deliverables (for the Chat doc-bridge, J-367)

- **Appendix F** — the `issue --tier <N>` mock flag (operator/forker-visible); the **3012 `assertion_tier_unauthorized`**
  reject semantics (registry `accepted_tiers` now enforcement-bearing: an issuer scoped to T2 cannot attest T3).
- **ch3** — activate **3012** in the auth-module band (the M10.1 reconcile reserved 3012–3016 dormant; this arc
  fills 3012). Record the C2 vs 3030 distinction (issuer-tier-unauthorized vs Space-tier-floor).
- **Findings flips:** **M10.2-A1 RESOLVED** (`accepted_tiers` enforcement-bearing); **M10-A-04 second half
  RESOLVED** (`module_kind: mock` populated by a real mock); M10.3-A4 recorded as the dormant-schema boundary
  (`Tier2/3/4Claims` not wired — flagged future).
- **DECISIONS:** candidates only, arc-local (D-069). Matrix/ROADMAP as applicable.

## 7. Surfaced at runbook (confirm-at-impl; none forks the locked design)

- **D3 wording precision (D-065):** the 3012 reject is a `RegistrationError`/`to_registration_code` (registration
  wire-error), **not** a dispatch `RejectInfo` (that is the Thread gate's 3030). Implement via the registration
  path; the design's "RejectInfo path (MP-F2)" is the generic reject-machinery phrasing.
- **AssertionPolicy field-add** touches the snapshot (`app.rs:809`) in C1 (empty) and the gate (`:2895`) in C3
  (derived). Confirm both compile with the new field; `Default` covers `AssertionPolicy::default()` callers.
- **`--tier 1` strict no-op:** verify the parameterized `issue` with N=1 yields a byte-identical reference
  assertion (module_kind reference, erasable, T1 TTL) — the M10.2 witnesses are the regression lock.
- **C2 key match:** `policy.accepted_tiers_by_issuer` is keyed by `module_id.to_string()` = `assertion.issuer`
  (same key as `trusted_issuers`); confirm a trusted issuer is always present in the map (both derived from the
  same non-revoked set), so `get` Some for any issuer that cleared Step 1.
