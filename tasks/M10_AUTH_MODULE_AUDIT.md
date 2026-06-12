# M10 — Auth Module Reference Set — D-071 Phase-0 Audit
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-12  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. Scope of this audit

**In scope (grounded to file:line):** the seven foundation surfaces M10 builds on (§§A1–A7 below), the
RC-F-01 3010–3016 wire-band map, and a findings register for the claim-vs-reality slips surfaced (D-065).

**Out of scope (locked boundaries — recorded, not audit-depth here):**
- **MP-F13** (home-node discovery) — a *named M10 sub-arc* (M10.4) with its own mini-Phase-0; not grounded here.
- **GDPR identity-orphan erasure mechanics** — **hook only** (Fork 3); only the *attachment surface* is grounded,
  not the erasure engine (PG-02 content half stays D3-gated).
- **M10.2/M10.3 design** (the binary + the mock) — downstream of this audit.

**Audit method:** symbol definitions grepped in production code (D-078), not inferred from call-sites or the
spec. Where spec and code disagree, the code is the live truth and the gap is a finding. Grounded against
`main` at **HEAD `5d8fec1`** (J-358). No design, no code in this doc.

**Headline of the grounding (read first):** the M10 foundation is **two disconnected trust surfaces, not one.**
The 5 `auth-module` CRUD verbs operate an `AuthModuleRegistry` that **no validation path consumes** (AMR-D1
standalone, by design); the gate `validate_assertion` actually enforces reads its trusted-issuer set from a
**separate `[node].trusted_auth_modules` config list**. A real T1 module must satisfy the *config* surface to be
honoured — registering it via the CRUD verbs alone does **not** make it trusted (M10-A-02). Everything else
(7-check path, TrustClaims extension point, baseline floor, tier-read) is shipped and clean; the wire-band has a
real double-definition (RC-F-01) and the erasure / mock-label surfaces are **absent** (hooks to build, not
existing seams). Details below.

---

## A1 — AuthModuleRegistry + the 5 CRUD/probe verbs + AuthModuleXgid (D-083)

*Goal: what a real external module must implement to register and be trusted.*

- **Registry type + storage** — `xgen-core/src/auth/module_registry.rs:77` `pub struct AuthModuleRegistry`
  (`modules: HashMap<AuthModuleXgid, AuthModuleRecord>`, `:79`). Record at `:51` `pub struct AuthModuleRecord`
  — fields `module_id: AuthModuleXgid` (`:55`), `endpoint_url: String` (`:57`), `accepted_tiers: Vec<AuthTier>`
  (`:59`), `registered_at: String` (`:61`), `revoked: bool` (`:65`), `revoked_at: Option<String>` (`:68`). **No
  `public_key` field** — the key is recovered via `module_id.pubkey()` (AMR-D3 derive-don't-store). API:
  `register` (`:89`, insert-or-replace), `revoke` (`:98`, block-only — sets `revoked=true`, **retains** the
  record), `set_tiers` (`:111`), `get`/`all`/`len`/`is_empty`, `save`/`load` (`:139`/`:148`, JSON, reuses
  `federation::registry::RegistryError`). Persist path `xgen-node_auth_modules.json` (admin_ops.rs:439
  `auth_module_registry_path`).
- **The 5 `auth-module` verbs** (all in `xgen-node/src/admin_ops.rs`):
  - `auth_module_list` (`:2448`) — READ, not audited; enumerates all records incl. revoked.
  - `auth_module_register` (`:2487`) — WRITE/audited; takes `--pubkey --endpoint --tier…`, **derives**
    `module_id` from `--pubkey` (`module_id_from_pubkey`, `:2378`); persists + A6 trail.
  - `auth_module_revoke` (`:2560`) — DESTRUCTIVE/audited; block-only; unknown id → `AUTHMOD_6101`.
  - `auth_module_set_tiers` (`:2627`) — WRITE/audited; replaces the accepted-tier set.
  - `auth_module_test` (`:2748`) — READ, not audited; **connectivity-only TCP probe** (5 s fail-fast,
    `AUTH_MODULE_PROBE_TIMEOUT_SECS=2681`), **no challenge/response** (honest-thin; signed-nonce handshake is
    unspecced, deferred to the Auth Module protocol arc). Unknown id → `AUTHMOD_6101`; unreachable → a
    `reachable:false` *result*, not an error.
  - clap surface: `enum AuthModuleCommand` (admin_ops.rs:4477) under `AdminCommand::AuthModule` (`:4350`).
  - dispatch arms: pipe `xgen-node/src/pipe.rs:563–615` (5 arms); aicontrol `xgen-node/src/aicontrol.rs:384–388`
    (5 arms). Three-of-the-four-arm surface (no CLI-subcommand / `--batch` arm — these are admin-pipe verbs).
  - Admin error band `AUTHMOD_61xx` (admin_ops.rs:2360–2366): `6101` unknown-module, `6102` invalid-pubkey,
    `6103` invalid-tier — **distinct** from the deferred wire-level `auth_module_untrusted`/3006.
- **`AuthModuleXgid` (D-083)** — `xgen-common/src/xgid/flavours.rs:222` `declare_flavour!(AuthModuleXgid, …)`
  (the third *principal* flavour, alongside `NodeXgid`/`IdentityXgid`; key-derived URI `xgen://pubkey/ed25519:`,
  no new wire shape). Constructors `from_pubkey` (`:361`) + `pubkey()` decode. Re-exported `xgid/mod.rs:52`.
- **The registration surface a module hits** — there is **none module-presented**. Registration is
  **operator-mediated**: the operator runs `auth-module register --pubkey <module key> --endpoint <url>` and the
  Node stores a record. The module presents nothing — no manifest, no capability advertisement, no module-
  initiated handshake. The `test` probe is operator-initiated connectivity-only. See **M10-A-06**.

**A1 verdict.** The registry + 5 verbs + XGID flavour are **shipped and clean**. What a real T1 binary must add
is **not** here: (a) a module that *signs `TrustAssertion`s* (the registry stores only `endpoint_url` + tiers,
never drives issuance); (b) a module-presented manifest/identity surface (absent — A1's "registration surface"
is operator CRUD); (c) the **wiring from registry → validation policy**, which does not exist today (M10-A-02).

---

## A2 — The 7-check validate_assertion + accept_registration + trusted_auth_modules + the synthetic test issuer

*Goal: the live path the real T1 module replaces; where the synthetic issuer sits today.*

- **`validate_assertion`** — `xgen-core/src/identity/registration.rs:193`. The 7 checks, in §3.8.5 order, each
  to its site:
  1. issuer ∈ `policy.trusted_issuers` (`:200`) — else `AuthModuleUntrusted` → **3006**.
  2. signature verifies (`:204`, `assertion.verify()`) — else `AssertionSignatureInvalid` → **3004**.
  3. `identity_id` == registrant (`:208`) — else `AssertionIdentityMismatch` → **3010**.
  4. `tier ≥ policy.required_tier` (`:212`, reuses `verify_tier_assertion`) — else `AssertionTierInsufficient`
     → **3030** (NOT 3010 — see A3).
  5. `valid_until` future vs `now` (`:216–221`; unparseable = expired) — else `AssertionExpired` → **3005**.
  6. `claims.tier_verified == true` (`:223`) — else `AssertionClaimsInsufficient` → **3011**.
  7. all `policy.required_claims` present (`:227`, `has_claim`) — else `AssertionClaimsInsufficient` → **3011**.
- **Wiring into `accept_registration`** — `registration.rs:401`; the `!local_node` branch at `:454–469` requires
  a `trust_assertion` (`:456`, else `TrustAssertionRequired`/3003), parses it tolerantly (`:460`), and calls
  `validate_assertion(&assertion, identity_id, policy, now)` (`:468`). Anchor `registration.rs:120–122` confirmed
  = the `to_registration_code` map (3010 `assertion_identity_mismatch` / 3011 `assertion_claims_insufficient` /
  3030 `tier_mismatch`). Steps 5–7 were dead code before Arc E.
- **`trusted_auth_modules` gate** — `AssertionPolicy` (`registration.rs:144`); **empty by default**
  (`:158–165` — `trusted_issuers: {}`, `required_claims: []`, `required_tier: 1`). Built at startup at
  `xgen-node/src/app.rs:745–746` from `config.node.trusted_auth_modules` (the `[node]` config field,
  `app.rs:132`) and installed via `runtime.set_assertion_policy`. Consulted under one runtime lock in
  `handle_identity_msg` (`app.rs:2838` `rt.assertion_policy.clone()`), passed into `accept_registration`
  (`app.rs:2848`). Default empty ⇒ in production mode every assertion fails step 1 (3006) until an operator adds
  an issuer; **Local Node mode skips the whole branch** (`registration.rs:454 if !local_node`, §3.8.8 — A5).
- **The synthetic test issuer** — there is **no production issuer**. The only issuer that drives this path is a
  test fixture: `registration.rs` tests fabricate a `SigningKey` (`issuer_key(0xA1)`), build + sign an assertion
  with it (`make_assertion`, `:1188`+ → `.sign(issuer)`), and hand-build a trusting policy
  (`policy_trusting`, `:1212` — inserts the issuer's pubkey URI into `trusted_issuers`). No `xgen-auth-module`
  binary, no module-initiated issuance, no registry→policy bridge. The path is **dormant-but-correct**: real
  Ed25519, real 7-check, but unreachable in production until an operator trusts a real issuer in config.

**A2 verdict.** The 7-check path is shipped, tested, and correct. The exact seam the real T1 issuer plugs into
is the **`[node].trusted_auth_modules` config list** (issuer pubkey URI), *not* the `AuthModuleRegistry`
(M10-A-02). What is dormant-but-correct today: every check runs, but production trust is gated behind a config
list that is empty by default and disconnected from the CRUD verbs.

---

## A3 — The 3010–3016 wire band (RC-F-01 renumber map)

*Goal: every definition site for the integers M10 owns, so the renumber is grounded not guessed.*

| Code | Meaning A — §3.6.5 / Arc E (live, code-backed) | Meaning B — §3.11.7 (higher-tier, dormant spec) | **Live truth (code)** | Site(s) |
|---|---|---|---|---|
| 3010 | `assertion_identity_mismatch` | `auth_tier_insufficient` | **A** (`assertion_identity_mismatch`) | ch3 §3.6.5 L1911 ✓ · ch3 §3.11.7 L3833 ✗ · `registration.rs:120` |
| 3011 | `assertion_claims_insufficient` | `kyc_verification_pending` | **A** (`assertion_claims_insufficient`) | ch3 §3.6.5 L1912 ✓ · ch3 §3.11.7 L3834 ✗ · `registration.rs:121` |
| 3012 | — | `watchlist_match` | **dormant** (no emitter) | ch3 §3.11.7 L3835 only |
| 3013 | — | `eidas_loa_insufficient` | **dormant** | ch3 §3.11.7 L3836 only |
| 3014 | — | `government_credential_required` | **dormant** | ch3 §3.11.7 L3837 only |
| 3015 | — | `clearance_level_insufficient` | **dormant** | ch3 §3.11.7 L3838 only |
| 3016 | — | `data_localisation_violation` | **dormant** | ch3 §3.11.7 L3839 only |
| 3030 | `tier_mismatch` (registration check 4 + join PG-13 tier-gate) | (§3.11.7 says this should be 3010 `auth_tier_insufficient`) | **`tier_mismatch`** | `registration.rs:122` · join gate via `tiers.rs`/`verify_tier_assertion` (D-067 shared) — **no §3.6.5 table row; only impl-note L3003** |

- **ch3 §3.6.5** region: **L1896–1912** (header L1896; the 3010/3011 rows L1911–1912). Implementation-status
  note **L3003** (under §3.8.5) already records the live truth: check 3 → 3010, checks 6–7 → 3011, check 4 →
  **3030** `tier_mismatch`.
- **ch3 §3.11.7** region: **L3825–3839** (header L3825 "Auth Module Error Codes for Higher Tiers"; the 3010–3016
  rows L3833–3839). **L3829** reservation note: "Codes 3010–3016 cover higher-tier Auth Module errors (this
  section)… 3020–3023 cover identity replication" — confirmed verbatim at L3829.
- **Confirmed collision:** 3010 + 3011 are defined **twice** with different meanings — §3.6.5/Arc E (live,
  code-emitted, tested) vs §3.11.7 (higher-tier, **zero code emitters**: grep of `xgen-core`/`xgen-node`/
  `xgen-common` for 3012–3016 and the §3.11.7 names → **none in code**). The §3.11.7 band 3010–3016 is entirely
  dormant spec. 3030 `tier_mismatch` is the live tier code and has **no §3.6.5/§3.11.7 table row** — it lives in
  the membership/PG-13 sub-band (referenced only by the impl-note L3003).
- **The 7 unmapped MP-F2-followon variants** — `xgen-core/src/message/exchange.rs` `ExchangeError`, the
  `_ => None` arm at **`:140`** (so they deliver generic **4000**): `EventIdMismatch` (`:60`, step 8),
  `DagError` (`:66`, step 10), `UnknownSender` (`:69`, step 11), `NotASpaceMember` (`:72`, step 11),
  `NotARoomMember` (`:75`, step 11), `SignatureFailure` (`:78`, step 12), `PermissionDenied` (`:81`, step 13).
  (`MissingEventId` `:114` and the buffer-state `HeldPending` `:63` also map to `None` but are not reject-class
  on the wire.) **These are event-validation codes (signature / membership / permission), NOT auth-module
  errors** — they do not naturally belong in the 3010–3016 auth-module band. See **M10-A-03**.

**A3 recommendation (NOT a decision — that is the M10.1 design-lock).**
1. **§3.6.5 / Arc E keeps 3010 + 3011.** They are shipped, wire-emitted, test-asserted
   (`validate_assertion_rejects_identity_mismatch` asserts `(3010, "assertion_identity_mismatch")`); moving them
   is a wire break. The §3.11.7 higher-tier set has **zero emitters** — moving dormant spec text is free.
2. **The §3.11.7 higher-tier codes renumber up out of 3010/3011.** Within the reserved auth-module band, the
   natural map is to shift the genuinely-new higher-tier codes into free slots (e.g. `watchlist_match`,
   `eidas_loa_insufficient`, `government_credential_required`, `clearance_level_insufficient`,
   `data_localisation_violation` → 3012–3016, which are already their live spec slots and need no move; the only
   *colliding* §3.11.7 rows are the 3010/3011 pair). So the minimal reconcile is: **delete §3.11.7's
   3010/3011 rows** (`auth_tier_insufficient` / `kyc_verification_pending`) and re-home them — `auth_tier_insufficient`
   **folds into the live 3030 `tier_mismatch`** (it is the same gate, already emitted), and `kyc_verification_pending`
   takes a free band slot (e.g. 3017, or 3011's vacated meaning is replaced — design-lock decides). 3012–3016
   stay as-is.
3. **The 7 unmapped event-validation variants are a *separate* code-assignment decision** (M10-A-03) — flag at
   design-lock whether they ride M10.1 or a sibling arc; they are not 3010–3016 auth-module codes.

---

## A4 — TrustAssertion + TrustClaims (the AI-D8 extension point)

*Goal: the struct M10.1 extends with the module-policy descriptor.*

- **`TrustAssertion`** — `xgen-common/src/trust_assertion.rs:140`. Fields = exact §3.8.4 wire schema: `kind`
  (`:144`, `"trust_assertion"`), `tier` (`:147`), `issuer` (`:149`), `identity_id` (`:151`), `issued_at`
  (`:153`), `valid_until` (`:156` — AE-D1, **not** `expires_at`; **no `jurisdiction`** per AE-D5 reversal,
  `:135–138`), `claims: TrustClaims` (`:158`), `signature: Option<String>` (`:162`, excluded from canonical
  bytes). Canonical sign/verify: `canonical_bytes` (`:170`, `TRUST_ASSERTION_FIELDS` `:56` — field order
  `type → tier → … → claims`, **claims keys sorted**, signature excluded), `sign` (`:181`), `verify` (`:200`,
  re-derives the issuer key from the `issuer` field).
- **`TrustClaims`** — `trust_assertion.rs:91`. `tier_verified: bool` (`:93`, MANDATORY) + optional
  `email_verified` / `phone_verified` / `email_hash` / `phone_hash` (`:94–103`) + **`#[serde(flatten)] extra:
  BTreeMap<String, Value>` (`:105–106`)** — the open-namespace forward-compat member, preserved round-trip
  (per ch3 §3.8.4; mirrors `wire::AiCapabilities::extra`). `has_claim` (`:115`) reads known fields then consults
  `extra`.
- **The forward-compat / open-doors attach point** — `claims.extra` (`:106`). Because `canonical_bytes` sorts
  claims keys and includes the whole `claims` object, **any member added to `extra` is part of the signed
  bytes** — i.e. the AI-D8 module-policy descriptor (erasability/retention) attached as a `claims.extra` key is
  **signed by the issuing module**, which is exactly the property the descriptor needs (the module *attests* its
  retention policy). This realizes the §8 open-doors principle (unknown members preserved verbatim) with no wire
  break.

**A4 verdict.** Clean extension point: **`claims.extra` is the home for the AI-D8 descriptor**, and it is signed
by construction. A *new top-level `TrustAssertion` field* would be the wrong shape — `TRUST_ASSERTION_FIELDS`
(`:56`) is a fixed canonical set, and AE-D5 already records that adding a top-level field either breaks the
signature contract or becomes an unsigned side-field. M10.1 lands the descriptor inside `claims` (either in
`extra` or as a typed claims sub-field with a matching `TRUST_CLAIMS`-ordering update) — design-lock chooses;
no struct re-architecture required.

---

## A5 — Local-Node bypass / hardcoded crypto-identity baseline (Fork 1)

*Goal: confirm the demonstrator layers over the floor without touching it.*

- **The baseline identity path (Local-Node bypass, §3.8.8)** — one bool, `local_mode`, threaded end-to-end:
  config field `NodeConfig.node.local_mode` (`xgen-node/src/app.rs:117`), OR'd with the `--local` override at
  `app.rs:505` (`config.node.local_mode || opts.local_override`), carried into `handle_identity_msg` and passed
  to `accept_registration(…, local_mode, …)` (`app.rs:2848`). The skip is `registration.rs:454`
  `if !local_node { …steps 4–7… }` — Local Node mode **does not require or validate a trust assertion**; the
  identity registers on its own keypair alone (the hardcoded crypto-identity floor). ch3 §3.8.8 header at
  **L3040**.
- **Where the module path and the baseline path diverge / coexist** — they are the **same function**
  (`accept_registration`), distinguished only by the `local_mode` bool at `registration.rs:454`. Production mode
  (`!local_mode`) requires + validates an assertion against the policy; Local mode skips. The baseline build of
  every `IdentityMessage::Register` carries `trust_assertion: None` (`registration.rs:273`,
  `build_register_with_ai`).

**A5 verdict (Fork 1 confirmed).** The T1 module demonstrates **over** the baseline with **no collision and no
floor change.** The demonstrator requires only that the operator run a *production-mode* Node (`local_mode =
false`) and trust the module's issuer; the Local-Node bypass branch is untouched. M10 adds a *demonstrated path*
(a real issuer satisfying the `!local_node` branch), it does not remove or rewrite the floor. Fork 1 holds as
locked.

---

## A6 — D-088 erasure tier-gate touch-points (Fork 3, hook only)

*Goal: the minimal hook surface; heavy mechanics out of scope.*

- **The tier-read that a "T4 refuses erasure" gate would key on** — `assertion_tier_of(record: &IdentityRecord)
  -> u32` (`xgen-core/src/node/runtime.rs:214`): `None → 1`, else `record.trust_assertion["tier"]`. Post-Arc-E
  this is the **validated** tier (`:207–213` doc-note), already the basis of the PG-13 join tier-gate. This is
  the natural attach point for a tier-graded erasure permission (refuse when `assertion_tier_of(record) == 4`).
- **The descriptor carrier** — `TrustClaims.extra` (A4) is where the module's declared erasability/retention
  policy (the D-088 "Auth Module declares the interior" half) would ride, signed.
- **What is present vs absent today** — **the erasure operation does not exist.** Grep of
  `xgen-core`/`xgen-node`/`xgen-common` for erasure/RTBF/tombstone/retention found **no production erasure code**.
  The only erasure-named code is (a) `client_mls.rs:339` `envelope_with_destroyed_key` — the **Arc-H crypto-shred
  *substrate demo*** (per-message `CK` wrap, D-088 AH-D1), **explicitly NOT** the destroy-to-erase storage
  operation (`:339–342` doc-note; fenced behind the erasure-impl arc per the D-088 cascade), and (b) unrelated
  type-erasure (`dag/store.rs`). D-088 itself is **design-only** in `DECISIONS.md:46` (PG-02 design-locked /
  impl-deferred; content half gated on PG-05/D3).

**A6 verdict (Fork 3, hook only).** The minimal hook M10 lands is **two surfaces, not an engine**: (1) the
**AI-D8 module-policy descriptor** on `claims.extra` (the carrier the module owns, M10.1); (2) the existing
**`assertion_tier_of` tier-read** (runtime.rs:214) as the gate basis. The actual erasure *consumer* (the
operation that would read the tier and refuse at T4) **does not exist in the tree** and is **out of scope** —
content-erasure mechanics stay D3-gated (PG-05), identity-orphan mechanics are the PG-02 impl arc. Honest
boundary: M10 ships the descriptor + the tier-read it would gate on; it does **not** ship a working "T4 refuses
erasure" because there is nothing to refuse yet (no erasure verb). This is consistent with brief Fork 3, made
concrete — see **M10-A-05**.

---

## A7 — Mock self-labelling surface (mock safety)

*Goal: how `mock`/test is expressed in manifest + assertions and enforced via trusted_auth_modules.*

- **Manifest/assertion `mock` label field** — **absent.** There is no `mock`/`test` field on `TrustAssertion`
  (`trust_assertion.rs:140`), on `TrustClaims` (`:91`), or on `AuthModuleRecord` (`module_registry.rs:51`). There
  is no manifest type for an Auth Module at all (A1: registration is operator-CRUD, no module-presented manifest).
  Grep for a non-test `mock` self-label surface in `xgen-core/src/auth` + `admin_ops.rs` → none (`mock` hits are
  all `MockClock` test infrastructure, unrelated).
- **The `trusted_auth_modules` enforcement** — **real and load-bearing.** A module's assertions are honoured
  only if its `issuer` URI is in `AssertionPolicy.trusted_issuers` (`registration.rs:200`), which is built solely
  from the `[node].trusted_auth_modules` config list (`app.rs:746`), **empty by default**. So a mock module is
  inert software unless an operator *explicitly* trusts it; plus the `AuthModuleRegistry.revoked` block-flag
  (`module_registry.rs:98`) for the CRUD surface.

**A7 verdict.** The **enforcement** half (the explicit `trusted_auth_modules` gate) **exists and is sufficient**
to make a mock real *software* but never a deployable *trust* anchor — mock safety today rests **entirely** on
that gate. The **expression** half (the §1 "self-labels `mock`/test in manifest + assertions") **does not exist**
— there is no field that says "I am a mock." M10 must **add** the self-label surface (most naturally a
`claims.extra` key riding the AI-D8 descriptor in M10.1, or a typed field in M10.3 when the parameterized mock
lands). See **M10-A-04**.

---

## 8. Findings register (D-065)

| ID | Severity | Surface | Finding | Routing |
|---|---|---|---|---|
| **M10-A-01** | S2 (known inbound) | A3 wire band | **RC-F-01 confirmed.** 3010 + 3011 are double-defined: §3.6.5/Arc E (live, code-emitted, tested) vs §3.11.7 higher-tier (zero emitters, dormant spec). Live truth = the Arc-E meanings (`registration.rs:120–121`); tier code is **3030 `tier_mismatch`**, not §3.11.7's "3010 auth_tier_insufficient". The §3.11.7 3010–3016 band is entirely dormant. | **M10.1** — renumber per §A3 recommendation (Arc-E keeps 3010/3011; §3.11.7 rows re-home; 3030 stays the live tier code). Decision at design-lock. |
| **M10-A-02** | **S2 (new)** | A1/A2 registry↔policy | **Registry and validation are disconnected.** `validate_assertion`'s trusted-issuer set is built from `[node].trusted_auth_modules` config (`app.rs:746`), **not** from `AuthModuleRegistry` (the 5 CRUD verbs). `auth-module register` does **not** make a module trusted for validation; the operator must *also* add the issuer URI to config. The registry's `accepted_tiers`/`revoked` are never consulted by the gate. This is the AMR-D1 "standalone, no runtime consumer" boundary made concrete. | **M10.2 design** — decide: (a) the real T1 module's issuer lands in `[node].trusted_auth_modules` config (status-quo seam), or (b) M10 wires the registry → policy (closes AMR-D1's deferral, makes `revoke`/`accepted_tiers` enforcement-bearing). Load-bearing for "registers via the existing CRUD verbs + trusted_auth_modules". |
| **M10-A-03** | **S3 (new)** | A3 event-validation codes | The "7 unmapped MP-F2-followon codes" (`exchange.rs:140` `_ => None` → 4000) are **event-validation** codes (signature/membership/permission), **not** auth-module errors — they don't belong in the 3010–3016 auth-module band. The brief's "map the 7" under M10.1 conflates two domains. | **M10.1 scope-note** — reconcile the 3010–3016 auth-module band as M10.1's core; treat the event-validation code-assignment as a **sibling decision** (own domain, e.g. a 30xx event-validation sub-band or a 40xx slot), Joe-lock whether it rides M10.1 or splits out. |
| **M10-A-04** | **S2 (new)** | A7 mock label | **Mock self-label surface absent.** No `mock`/test field on `TrustAssertion`/`TrustClaims`/`AuthModuleRecord`. §1's "self-labels mock/test in manifest + assertions" requires a NEW field. Mock safety today is **enforcement-only** (the `trusted_auth_modules` gate) with no expression surface. | **M10.1** (if the label rides the AI-D8 `claims.extra` descriptor) **or M10.3** (typed field with the parameterized mock). Enforcement exists; expression must be built. |
| **M10-A-05** | **S3 (new)** | A6 erasure hook | **No erasure operation in the tree** (Fork 3 made concrete). No erasure verb / identity-orphan op / retention enforcement; the only erasure-named code is the Arc-H crypto-shred *substrate demo* (`client_mls.rs:339`, explicitly not the storage op) + unrelated type-erasure. "T4 refuses erasure" has nothing to gate. | **M10.3 (gate basis) / flagged.** Hook = AI-D8 descriptor (M10.1) + `assertion_tier_of` tier-read (runtime.rs:214). Enforcement consumer stays D3-gated (PG-02). Consistent with Fork 3 = hook only. |
| **M10-A-06** | **S3 (new)** | A1 module surface | **No module-presented registration surface.** Registration is operator-CRUD (`auth-module register --pubkey --endpoint --tier`); `AuthModuleRecord` carries only `endpoint_url` + tiers; `test` is connectivity-only (no challenge/response, no manifest exchange, no module-initiated handshake). The brief §5.1 "endpoint/manifest it presents" does not exist as a module-presented surface. | **M10.2 design** — the autonomous `xgen-auth-module` binary defines what (if anything) it presents; today the Node-side surface is operator-mediated bookkeeping only. Shapes the T1 binary's interface. |

*RC-F-01 / MP-F2-followon was the known inbound (M10-A-01 → M10.1). Five new findings surfaced (M10-A-02..06);
none reopen a locked fork — they sharpen what each sub-arc must build.*

---

## 9. Definition of Done (this audit)

- [x] §§A1–A7 each grounded to file:line; every `[GROUND]` marker replaced.
- [x] A3 RC-F-01 table complete with live-truth column + a renumber **recommendation** (not a decision).
- [x] Findings register populated — RC-F-01 confirmed (M10-A-01) + five new (M10-A-02 registry↔policy
      disconnect, M10-A-03 event-validation codes out-of-band, M10-A-04 mock-label absent, M10-A-05 no erasure
      op, M10-A-06 no module-presented surface).
- [x] Locked-fork boundaries respected (MP-F13 depth, GDPR-orphan mechanics, M10.2/M10.3 design all out;
      Fork 1 confirmed at A5, Fork 3 hook-only honesty at A6).
- [x] Header bumped v0.1 → v1.0; scaffold note removed; Status stays ACTIVE.
- [ ] Audit committed (Clair's commit precedes Chat's doc-bridge; Joe pushes).

**Next after this audit:** M10.1 design (wire-band reconcile per §A3 + AI-D8 descriptor on `claims.extra` per
§A4) → Joe-lock → runbook. The two design-shaping calls for M10.1/M10.2: **(M10-A-02)** does the real T1 module's
trust ride config or a registry→policy wiring, and **(M10-A-03)** do the event-validation codes ride the
wire-band reconcile or split to a sibling decision.
