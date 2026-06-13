# M10.2 — Tier-1 Reference Auth Module (`xgen-auth-module`) — D-071 Phase-0 Audit
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jun 2026  
> **Last updated**: 2026-06-13  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. Scope, method, headline

**In scope (grounded to file:line on live `main`).** Brief §3's six grounding items (A1–A6 below); the
registry→policy wiring sketch (§7) and issuance-shape options (§8) the design will lock; the blast radius of the
M10.2 validation behaviour change (A7); a findings register (§10).

**Method.** Symbol definitions grepped in production code (D-078), not inferred from call-sites or the spec.
Where spec and code disagree, the code is the live truth and the gap is a finding (D-065). Grounded against
`main` @ **`a86db48`** (HEAD, post-M10.1). **Re-grounded fresh** — the M10 audit (`M10_AUTH_MODULE_AUDIT.md`
v1.1) was at HEAD `5d8fec1`; M10.1 (`01ea770`) landed code in `registration.rs` / `trust_assertion.rs` since,
so line numbers shifted and are re-verified here. No design, no code in this doc.

**The two calls are LOCKED (brief §2, J-362) and not reopened here:** M10-A-02 = (b) wire registry → policy;
M10-A-06 = operator-CRUD kept (the binary is a pure signer + issuance endpoint).

**Headline of the grounding (read first).** The locked frame holds, sharpened on three points:

1. **The trust-source change is a single seam.** Production has exactly **one** caller of `validate_assertion`
   (`registration.rs:486`, inside `accept_registration`), reached from exactly **one** node path
   (`handle_identity_msg`, `app.rs:2842`). The gate's trust set is built **once at startup** from config
   (`app.rs:745–749`) and is a static snapshot — disconnected from the CRUD `AuthModuleRegistry`. The
   registry→policy wiring is therefore a narrow, well-bounded change at the policy-construction site, not a
   pipeline rewrite.

2. **The binary is an offline signer for M10.2.** `AuthModuleRecord.endpoint_url` is consumed **only** by the
   `auth-module test` connectivity probe (`module_registry.rs:57` doc; verb at `admin_ops.rs:2748`). The node
   **never calls the module** to validate — it verifies the presented assertion's Ed25519 signature offline
   against the `issuer` pubkey (= `module_id.pubkey()`). So the witness needs the binary only to **produce a
   signed assertion**; a running endpoint is not required (resolves brief §4's "endpoint vs offline" — offline
   sufficient, and spec-aligned: §3.8.7 makes Auth Module registration out-of-band/operator-mediated).

3. **Two real behaviour-shaping questions surfaced under the lock** (neither contradicts it): making
   `accepted_tiers` enforcement-bearing needs a *new per-issuer check* (richer than the current node-wide
   `required_tier`), and a live `revoke` needs the gate to read **live** registry state, not the startup
   snapshot. Both are design-lock calls (§10 M10.2-A1, M10.2-A2). Everything else (issuance shape, descriptor
   population, keypair surface) is shipped substrate the binary reuses.

---

## A1 — The issuance path + the synthetic test issuer + the `TrustAssertion` shape (brief §3.1)

*Goal: how an identity gets a module-signed Tier-1 assertion; what the synthetic issuer does today; the exact
shape the real binary produces.*

- **There is no production issuer today.** The only thing that drives the 7-check path is a **test fixture**
  (`registration.rs` tests): `issuer_key(seed)` fabricates a `SigningKey` (`:1195`); `make_assertion`
  (`:1203–1228`) builds a `TrustAssertion` and signs it via `.sign(issuer)`; `policy_trusting` (`:1230`) inserts
  the issuer's URI into `trusted_issuers`. The real `xgen-auth-module` binary replaces `issuer_key` +
  `make_assertion` with a real keypair + real issuance, and replaces `policy_trusting` with the registry→policy
  wiring (A2).

- **The exact shape the binary must produce** (from `make_assertion` `:1210–1227` + the `TrustAssertion` struct
  `xgen-common/src/trust_assertion.rs:248`):
  - `kind = "trust_assertion"` (`:253`, serde `type`); `tier` (`:256`); `issuer` = the module's
    **`AuthModuleXgid::from_pubkey(&module_key.verifying_key()).to_string()`** (`registration.rs:1199–1201`,
    `:1213`) — i.e. `xgen://pubkey/ed25519:<module key>`; `identity_id` = the registrant's id URI; `issued_at` /
    `valid_until` RFC-3339; `claims: TrustClaims` with `tier_verified = true`; then **`.sign(module_key)`**
    (`trust_assertion.rs:290`). `signature` is excluded from the canonical bytes by field order (`:279–282`,
    `TRUST_ASSERTION_FIELDS` `:56`), so the same key that names `issuer` must sign.

- **How the assertion reaches the node (witness path — real end-to-end today).** The registrant attaches it to
  its register: `IdentityMessage::Register { … trust_assertion: Option<serde_json::Value> … }` carries the
  serialised assertion. `accept_registration` extracts it (`registration.rs:474`), parses it tolerantly
  (`:478`), and calls `validate_assertion(&assertion, identity_id, policy, now)` (`:486`). The in-process witness
  shape **already exists**: `non_local_registration_with_valid_assertion_accepted` (`registration.rs:1377+`)
  registers with a `make_assertion`-issued tier-2 assertion against `policy_trusting(&issuer)` and asserts the
  validated tier is persisted. M10.2's end-to-end witness is this test with (a) a **real binary** as the issuer,
  (b) a **registry-trusted** issuer (not `policy_trusting`), and (c) a **revoke → reject** RED-on-revert leg.

- **There is no specced issuance wire protocol for T1.** ch3 §3.8.7 ("Auth Module Registration with a Node",
  L3046) describes the **operator/out-of-band** handshake (the operator provides the module pubkey to the Node
  operator, who adds it to the trusted list — §3834 confirms T2–T4 follow "the same out-of-band process as Tier
  1"); the `auth-module test` probe is connectivity-only with **no challenge/response** (`module_registry.rs`
  doc; M10 audit A1). So *issuance* (identity → assertion) is unspecced today — the design chooses its shape
  (§8).

**A1 verdict.** The shape is fully grounded and the witness path is real. The binary's job at the protocol level
is exactly `make_assertion` made production: build a `TrustAssertion` naming itself as `issuer`, set the M10.1
descriptor (A6), `.sign(module_key)`. The only open call is *how the identity obtains it* (§8 issuance shape).

---

## A2 — The Call-1b trust-source seam (brief §3.2)

*Goal: where `validate_assertion` reads its trusted-issuer set today, and the minimal change to read the
registry + where the config-seed lands.*

- **The 7-check gate** — `validate_assertion` (`xgen-core/src/identity/registration.rs:211`). Step 1 (`:218`):
  `if !policy.trusted_issuers.contains(&assertion.issuer) → AuthModuleUntrusted` (3006). Steps 2–7 are pure
  signature/identity/tier/expiry/claims checks (`:222–249`); only steps 1 + 7 consult `policy`. The gate takes
  `policy: &AssertionPolicy` (`:214`).

- **`AssertionPolicy`** — `registration.rs:163`: `trusted_issuers: HashSet<String>` (`:166`),
  `required_claims: Vec<String>` (`:169`), `required_tier: u32` (`:173`). `Default` = empty set / empty claims /
  tier 1 (`:176–183`).

- **The seam (single site)** — `xgen-node/src/app.rs:745–749`:
  ```
  runtime.set_assertion_policy(AssertionPolicy {
      trusted_issuers: config.node.trusted_auth_modules.iter().cloned().collect(),  // ← the trust source today
      required_claims: Vec::new(),
      required_tier: 1,
  });
  ```
  Built **once at startup** from the `[node].trusted_auth_modules: Vec<String>` config field
  (`app.rs:132`), installed on the runtime (`set_assertion_policy`, `runtime.rs:466`), stored as the
  `NodeRuntime.assertion_policy` field, read at the gate via `rt.assertion_policy.clone()`
  (`app.rs:2838`) and passed into `accept_registration` (`:2842`). **It never reads the registry.**

- **The CRUD `AuthModuleRegistry` is a separate, live surface.** Loaded into `Arc<Mutex<AuthModuleRegistry>>`
  **inside the pipe block** (`app.rs:1164–1185`, with an explicit "AMR-D1 standalone — no runtime path reads it"
  note), handed to the pipe + aicontrol admin verbs. The 5 verbs mutate this in-memory Arc **and** save to disk
  (`admin_ops.rs:2487` register / `:2560` revoke / `:2627` set_tiers each `Arc::clone` + `…path()` + save). A
  **second, transient** read exists at `app.rs:520–535` (startup reads the registry read-only to extract
  `accepted_tiers` for the storage-engine floor, then drops it). So today: a config-derived **snapshot** the gate
  reads, and an `Arc<Mutex>` **registry** the verbs mutate — disconnected (M10-A-02 made concrete).

- **The minimal change (M10-A-02 = (b)).** Change the trust source at the seam from config to the registry:
  derive `trusted_issuers` from the registry's **non-revoked** records' `module_id.to_string()`. The config
  field is **not** dropped — it **bootstrap-seeds the registry** at startup (config-seed sub-lock). The exact
  wiring (load-site, snapshot-vs-live-read, where the seed runs) is the §7 sketch + the design-lock calls
  (M10.2-A2, M10.2-A3). Spec support: §3016 makes step 1 "issuer is a **registered** Auth Module on this Node
  (3.8.7)" — the registry **is** the §3.8.7 registration store, so reading it is *more* spec-aligned than the
  config seam.

**A2 verdict.** The seam is a single, narrow site (`app.rs:745–749` + the gate read at `:2838`). The gate logic
(`validate_assertion` step 1) is unchanged if the policy is rebuilt from the registry; the change is *where the
trusted set comes from*. The one design subtlety the seam forces: the policy is a startup snapshot, so a live
`revoke` is invisible to it unless the wiring reads the registry live (M10.2-A2).

---

## A3 — `AuthModuleRecord` / `endpoint_url` semantics (brief §3.3)

*Goal: does the node ever call the module, or only verify the presented signature? (offline signer vs live
endpoint).*

- **`AuthModuleRecord`** — `xgen-core/src/auth/module_registry.rs:51`: `module_id: AuthModuleXgid` (`:55`),
  `endpoint_url: String` (`:57`), `accepted_tiers: Vec<AuthTier>` (`:59`), `registered_at: String` (`:61`),
  `revoked: bool` (`:64`), `revoked_at: Option<String>` (`:67`). **No `public_key`** — the key is recovered via
  `module_id.pubkey()` (AMR-D3 derive-don't-store).

- **`endpoint_url` is consumed by exactly one thing: the connectivity probe.** Its own doc says "Where the module
  is reached (the `auth-module test` probe target, C4)" (`:57`). The probe (`auth_module_test`,
  `admin_ops.rs:2748`) is a **TCP connectivity check** (5 s fail-fast, `AUTH_MODULE_PROBE_TIMEOUT_SECS`), **no
  challenge/response** (M10 audit A1). **Validation never touches it** — `validate_assertion` verifies the
  presented assertion's signature **offline** against `assertion.verify()` → `AuthModuleXgid::from_xgid(issuer)
  .pubkey()` (`trust_assertion.rs:309–321`). The node has **no code path that calls the module** during
  registration.

- **Consequence for M10.2.** The binary is a **pure offline signer**. The witness (module issues → identity
  registers → node validates → accept; revoke → reject) runs with **no live endpoint** — the node only needs the
  signed assertion bytes. The `endpoint_url` an operator supplies at `auth-module register` is **decorative for
  validation** (only the `test` probe uses it); for an offline-issuance module the probe is moot unless the
  operator also runs a liveness endpoint. This is the clean confirmation of the locked frame, sharpened: not
  only is the binary a "pure signer + issuance endpoint" (M10-A-06), the *endpoint half is not even load-bearing
  for the validation gate* — it serves only operator connectivity diagnostics.

**A3 verdict.** Offline signer. The node verifies the signature, never calls the module. Brief §4's "endpoint vs
offline" resolves to **offline is sufficient for M10.2** (a minor honesty note for §8/§10: `endpoint_url` stays a
required register field but is inert at the gate).

---

## A4 — The keypair / signing surface the binary reuses (brief §3.4)

*Goal: the canonical-bytes signer; how a module identity/keypair is created + expressed (`AuthModuleXgid`,
D-083).*

- **Keypair (own keypair, encrypt-at-rest).** `xgen-core/src/identity/keypair.rs`: `generate() -> SigningKey`
  (`:56`), `save(&SigningKey, &Path, passphrase) -> Result<(), KeypairError>` (`:62`, ChaCha20-Poly1305 +
  Argon2id at rest), `load(&Path, passphrase) -> Result<SigningKey, KeypairError>` (`:90`). The binary reuses
  these for its own `xgen-auth-module_keypair.enc` (the Node/Client keypair pattern, D-035 file convention).

- **Module identity expression — `AuthModuleXgid` (D-083).** `xgen-common/src/xgid/flavours.rs:222`
  `declare_flavour!(AuthModuleXgid, …)` (the seventh XGID flavour, third *principal* flavour). `from_pubkey(pk:
  &VerifyingKey) -> Self` (the principal-flavour constructor, macro-generated `:331`/`:350`/`:369` family);
  `.to_string()` yields `xgen://pubkey/ed25519:<key>` (= the assertion's `issuer` URI); `pubkey() ->
  Result<VerifyingKey, XgidDecodeError>` recovers the key (the verifier's path). No new URI prefix, no new wire
  shape (inherits §J.5 invariances).

- **The canonical-bytes signer.** `TrustAssertion::canonical_bytes()` (`trust_assertion.rs:279`) →
  `canonical::canonical_object_json(&value, TRUST_ASSERTION_FIELDS)`; `sign(self, key: &SigningKey) -> Self`
  (`:290`) signs those bytes and formats `ed25519:<b64url-pubkey>:<b64url-sig>` (mirrors
  `crypto::signing::format_signature`, `xgen-core/src/crypto/signing.rs:26`). `verify()` (`:309`) re-derives the
  key from `issuer` and checks the canonical bytes.

- **`SignedPrimitive` is a concept, not a trait (D-065 honesty).** The brief §3.4 phrasing "the
  `SignedPrimitive`/canonical-bytes signer" could read as a trait to implement. There is **no `trait
  SignedPrimitive`** in the tree (grep: only doc-prose — "the third `SignedPrimitive`", `trust_assertion.rs:11`).
  Event, `node_announcement`, and `TrustAssertion` each self-certify via their own canonical-bytes + Ed25519
  helpers; there is nothing generic to implement. The binary's concrete signing surface is
  `TrustAssertion::sign` + `keypair::{generate,save,load}`. Recorded so the design doesn't hunt for a trait
  (§10 M10.2-A5).

**A4 verdict.** Every signing/keypair primitive the binary needs is shipped and public. The binary = `generate`
a keypair → express it as `AuthModuleXgid::from_pubkey` → build + `sign` a `TrustAssertion`. No new crypto.

---

## A5 — What "Tier-1 verification" means in code (brief §3.5)

*Goal: ground the issuance logic's honesty — likely proof-of-key-possession only, no external KYC.*

- **The only thing a T1 assertion attests in code is `claims.tier_verified == true` at `tier = 1`.**
  `validate_assertion` check 6 (`registration.rs:241`) requires `assertion.claims.tier_verified`; check 4
  (`:230`, `verify_tier_assertion(tier, required_tier)`) is satisfied at `tier ≥ 1`; check 7 (`:245`) requires
  any `policy.required_claims`, but those are **empty by default** (`AssertionPolicy::default` `:180`;
  `app.rs:747` sets `required_claims: Vec::new()`). So a T1 assertion carries **no email/phone/KYC claim** —
  `TrustClaims.tier_verified: bool` is mandatory, the contact claims (`email_verified` / `phone_verified` /
  `email_hash` / `phone_hash`) are all `Option` and absent for a bare T1 (`trust_assertion.rs:91–107`).

- **There is no in-code "T1 verification" logic** — no production issuer exists (A1), so nothing in the tree
  performs a T1 check before issuing. The reference binary defines what "verified" means for T1; the honest
  floor it can attest is **proof-of-key-possession**: the requester demonstrated control of the identity key
  (the issuance handshake of §8), nothing more. Tier 1 in ch1's compliance table is the "pseudonymous /
  self-asserted" floor; the reference module is the demonstrator that this floor *works mechanically*, not a KYC
  service (those are T2–T4, M10.3 mock + the institutional arc).

**A5 verdict.** "Tier-1 verification" in code = `tier_verified:true` at tier 1 with no contact claims required.
The issuance logic must be honest: the reference binary attests **proof-of-key-possession only**. This is an
honesty constraint on the §8 issuance design, not a defect (§10 M10.2-A5).

---

## A6 — M10.1 descriptor population (brief §3.6)

*Goal: confirm the binary sets `module_kind: reference` + a `module_policy` (erasability) on issued assertions.*

- **The descriptor shipped in M10.1** (`01ea770`, verified on disk at `trust_assertion.rs:132–239`):
  `ModuleKind { Reference, Mock }` (`:148`); `ModulePolicy { erasability: Option<Erasability>, extra }` (`:161`);
  `Erasability { retention: Option<Retention>, extra }` (`:173`); `Retention { Erasable, Retained }` (`:187`).
  Both ride `TrustClaims.extra` under the keys `module_kind` / `module_policy` (`MODULE_KIND_KEY` `:198`,
  `MODULE_POLICY_KEY` `:200`).

- **The setters the binary calls** (must run **before** `.sign`, since the descriptor joins the canonical bytes —
  `canonical_object_json` recurses into `claims`): `TrustClaims::set_module_kind(ModuleKind::Reference)` (`:215`)
  and `set_module_kind` for the descriptor; `set_module_policy(&ModulePolicy { erasability: Some(Erasability {
  retention: Some(Retention::Erasable), .. }), .. })` (`:233`). Read-side accessors `module_kind()` (`:206`,
  defaults `Reference` when absent) and `module_policy()` (`:225`, `None` when absent) are signature-covered (the
  M10.1 witnesses `module_descriptor_is_signature_covered` `:600` + `module_policy_unknown_members_round_trip`
  `:630` prove it). ch3 §3.8.4 already documents `module_kind` (L2967, "Absent ⇒ reference").

- **The binary CAN and SHOULD set both** on the assertions it issues: `module_kind = Reference` (it *is* the
  reference build) and a `module_policy.erasability.retention` consistent with a T1 module (`Erasable` — T1 is
  the max-erasable endpoint per AI-D4). This is **expression only** — M10.1 landed the field with no enforcement
  consumer (the D3-gated erasure consumer is out of scope, brief Out). The binary populating it makes the
  reference assertion a *complete* example, but absence still resolves to `Reference` so it is not load-bearing
  for the witness.

**A6 verdict.** The descriptor surface is shipped, signature-covered, and trivially populated by the binary via
the two setters before signing. M10.2 SHOULD populate both (reference build + erasable T1 policy) so the
reference assertion is a faithful example; nothing in the validation gate consumes them, so it is expression, not
enforcement.

---

## A7 — Blast radius of the validation behaviour change (validate_assertion callers + AMR-D1 boundary)

*Goal: ground exactly what the registry→policy wiring touches, so the design knows the surface (the brief's
explicit guardrail — this is a node-side validation behaviour change, unlike M10.1).*

- **`validate_assertion` production callers: exactly ONE.** `registration.rs:486` (inside
  `accept_registration`). Every other match is a unit test (`registration.rs:1239–1355`). → the gate function
  itself need not change unless the policy *shape* changes (M10.2-A1).

- **`accept_registration` production callers: exactly ONE.** `app.rs:2842` (`handle_identity_msg`). → the
  behaviour-change chain is a single line of node code: the policy fed to `accept_registration:2842` must come
  from the registry instead of the startup snapshot.

- **AMR-D1 boundary — the registry gains its FIRST runtime consumer.** Today the registry has **zero** runtime
  consumers (AMR-D1 standalone; admin-verbs-only, loaded in the pipe block `app.rs:1164`). M10.2 makes the gate
  (`handle_identity_msg`) its first consumer. The load-site must move from the pipe block to `run_node`
  top-level and be shared (sibling to `bootstrap_store` / `node_policy_store`, shared top-level at
  `app.rs:1186–1189`) — M10.2-A3.

- **The M10.2 PRIME INVARIANT (must be preserved + asserted).** An **empty registry + empty config seed** →
  empty `trusted_issuers` → every production assertion fails step 1 (3006) → **byte-for-byte identical to
  today's empty-config behaviour**. Behaviour changes only when records exist. The config-seed makes existing
  config-based trust keep working migration-free (a `[node].trusted_auth_modules` operator gets those issuers
  seeded → still trusted). This is the M10.2 analogue of AMR-D1's "empty registry = today" invariant, now with a
  live consumer.

- **Out of the blast radius (confirmed unchanged):** Local-Node bypass (`registration.rs:472 if !local_node`,
  §3.8.8) — Local mode never enters the gate, so the floor is untouched (Fork 1, M10 audit A5). The other 6
  validate-assertion checks (signature/identity/expiry/claims) are unchanged. The 5 CRUD verbs are unchanged in
  behaviour (they already mutate + save the registry); they simply become *consulted*.

**A7 verdict.** The behaviour change is one node-code chain (`app.rs` policy construction → `accept_registration`
:2842 → `validate_assertion` :486), plus moving the registry load to top-level and seeding it. Narrow,
bounded, and gated by a clean empty-baseline prime invariant.

---

## 7. Registry → policy wiring sketch (deliverable; design locks the exact shape)

The locked call M10-A-02 = (b). The minimal wiring that makes `register`/`revoke` enforcement-bearing while
keeping the empty-baseline prime invariant (A7):

1. **Move the registry load to `run_node` top-level** and hold it as a shared `Arc<Mutex<AuthModuleRegistry>>`
   (sibling to `bootstrap_store` / `node_policy_store`, `app.rs:1186–1189`). The pipe/aicontrol verbs receive the
   same Arc (they already expect one — `pipe.rs:787`, `aicontrol.rs:82`); the transient floor read at
   `app.rs:520–535` can read from this handle instead of re-loading.

2. **Bootstrap-seed (config-seed sub-lock).** After load, for each `config.node.trusted_auth_modules` issuer URI
   **absent** from the registry, insert a seed `AuthModuleRecord` (parse the URI to `AuthModuleXgid`,
   `accepted_tiers = [Tier1]`, `registered_at = now`, `revoked = false`); save if changed. **Add-only,
   idempotent, never un-revoke** (M10.2-A4) — a re-seed on restart must not resurrect an operator-revoked
   module. Migration-free: pre-M10.2 configs keep working.

3. **The gate reads the registry (not the startup snapshot).** Build the trusted set from the registry's
   **non-revoked** records' `module_id.to_string()`. Two shapes for the design to choose (M10.2-A2):
   - **(a) live-read (recommended):** in `handle_identity_msg`, lock the registry and build the `AssertionPolicy`
     (or just its `trusted_issuers`) **fresh per registration** from the live records. `revoke` is instantly
     live; single source of truth; `xgen-core` stays free of the registry type (the set is built in `app.rs` and
     passed in, as today). The `required_claims`/`required_tier` stay node-config (keep that part of the policy).
   - **(b) snapshot-refresh:** keep the startup snapshot, and have each CRUD mutation call back into the runtime
     to rebuild `set_assertion_policy` from the registry. More moving parts (mutation↔runtime coupling) and a
     window where a crash between mutation and refresh diverges disk from snapshot. Not recommended.

4. **`validate_assertion` step 1 (`registration.rs:218`) is unchanged in logic** under shape (a) — it still does
   `policy.trusted_issuers.contains(issuer)`; the set is just sourced live. **If `accepted_tiers` enforcement
   lands** (M10.2-A1), the policy shape grows to carry per-issuer accepted tiers and check 4 (or a new check 4b)
   consults it — that is the one place the gate function itself changes.

**Seam summary:** trust source flips at `app.rs:745–749` (build from registry, not config); registry load moves
to top-level + seeds; gate read at `app.rs:2838` sources live. Single chain; xgen-core untouched under shape (a)
unless accepted_tiers enforcement is in scope.

---

## 8. Issuance-shape options + recommendation (deliverable; Joe locks at design)

How an identity obtains a module-signed assertion to present at registration. Grounded by A1 (no specced
issuance protocol; §3.8.7 is out-of-band) + A3 (node never calls the module → no live endpoint needed for the
gate).

| Option | Shape | Needs live endpoint? | Spec fit | Cost |
|---|---|---|---|---|
| **(b) offline-signed token** *(recommended)* | `xgen-auth-module issue --identity <id> [--tier 1] [--valid-days N]` → prints/writes a signed `TrustAssertion` (JSON) the identity pastes into its register. | **No** | §3.8.7 out-of-band; matches "pure signer" (M10-A-06) | Lowest — a CLI signer; no WS server, no challenge protocol to spec |
| (a) challenge/response | identity connects to a running module endpoint, proves key possession (signs a nonce), module returns the assertion. | Yes | unspecced today (the `auth_module_test` probe is connectivity-only); would need a new Auth Module wire protocol | Highest — new transport + protocol; the deferred "Auth Module protocol" arc the `test` probe already points at |
| (c) operator-mediated | operator runs (b) and hands the assertion to the registrant out-of-band. | No | §3.8.7 | Same as (b), framed as an operator step |

**Recommendation: (b) offline-signed token for M10.2.** It is the leanest path to the locked witness (module
issues a real signed assertion → identity registers → node validates against a registry-trusted issuer → accept;
revoke → reject), needs no live endpoint (A3), is spec-aligned (§3.8.7 out-of-band), and matches "pure signer +
issuance endpoint" (M10-A-06) without building a wire protocol. The challenge/response live endpoint is a
**flagged future enhancement** (rides the deferred Auth Module protocol arc / MP-F13-era surfaces), not M10.2.
*(c) collapses to (b) operationally.* Joe locks at design.

---

## 9. Out-of-scope boundaries (recorded, not audit-depth — brief Out)

- **M10.3** — the parameterised T2–T4 mock + dormant-tier-path activation (per-tier claims/TTLs, the typed
  `mock` field). M10.2 ships only the T1 reference; `ModuleKind::Mock` exists but M10.2 populates only
  `Reference` (A6).
- **M10.4 / MP-F13** — home-node discovery; its own mini-Phase-0.
- **Module-presented manifest/handshake (M10-A-06)** — flagged future. M10.2's module presents nothing to the
  node beyond the operator-CRUD record + the offline assertion; no manifest, no module-initiated handshake.
- **Erasure *enforcement*** — the `module_policy.erasability` consumer stays D3-gated (PG-02 / Arc-I AI-D4). The
  binary *populates* the descriptor (expression); nothing reads it to refuse erasure (no erasure op exists —
  M10 audit A6/M10-A-05).
- **accepted_tiers as a *per-Space* gate** — PG-13 join-time tier gating is unchanged; M10.2 touches only
  registration-time assertion validation.

---

## 10. Findings register (D-065)

| ID | Severity | Surface | Finding | Routing |
|---|---|---|---|---|
| **M10.2-A1** | S2 → CARRIED → M10.3 (accepted_tiers enforcement, J-364) | A2/A7 policy shape | **`accepted_tiers` enforcement is bigger than the minimal trust-source wiring.** Wiring `trusted_issuers` from non-revoked records makes `register`/`revoke` enforcement-bearing. But the registry's per-module `accepted_tiers` (`module_registry.rs:59`) is *not* what check 4 enforces — check 4 is node-wide `tier ≥ required_tier` (`registration.rs:230`). Making `accepted_tiers` enforcement-bearing needs a **new per-issuer check** (`assertion.tier ∈ issuer.accepted_tiers`), which means the gate must see the issuer's record — i.e. `policy.trusted_issuers: HashSet<String>` grows to a per-issuer map (or `validate_assertion` takes the registry). | **M10.2 design** — decide whether per-issuer `accepted_tiers` enforcement lands in M10.2 (small extension, but a `validate_assertion` shape change) or defers (M10.2 ships trusted_issuers + revoked only; accepted_tiers stays registry bookkeeping). The locked call ("`accepted_tiers` become enforcement-bearing") leans include; the cost is the gate-shape change. |
| **M10.2-A2** | S2 → ✅ RESOLVED (D2 live-read, M10.2/J-364) | A2/A7 architecture | **A live `revoke` requires the gate to read live registry state, not the startup snapshot.** The policy is built once at startup (`app.rs:745`); `revoke` is a runtime admin verb mutating the `Arc<Mutex>`. "Revoke that actually rejects" needs the gate to see the mutation → either live-read the registry at validate time (recommended) or refresh the snapshot on every CRUD mutation. | **M10.2 design** — lock the architecture: live-read (single source of truth, instant revoke, xgen-core untouched) vs snapshot-refresh (mutation↔runtime callback). §7 recommends live-read. |
| **M10.2-A3** | S3 → ✅ RESOLVED (registry to run_node top-level; AMR-D1 closed, M10.2/J-364) | A7 load-site | **The registry must move to `run_node` top-level (two consumers now).** Today it loads inside the `#[cfg(windows)]` pipe block (AMR-D1 standalone). The gate is its first runtime consumer, so it must load top-level + be shared (sibling to bootstrap/node-policy stores) and held for the gate to consult; the transient floor read (`app.rs:520`) folds into the shared handle. | **M10.2 design/impl** — mechanical, but it relocates the load + threads the Arc into `handle_identity_msg`. |
| **M10.2-A4** | S3 → ✅ RESOLVED (seed add-only/idempotent/revoke-wins, M10.2/J-364) | A2 config-seed | **Config-seed precedence: revoke must win over config presence; re-seed must be idempotent add-only.** If the seed re-inserts a config-named issuer that an operator later revoked, restart silently un-revokes it — the exact "revoke that doesn't revoke" footgun M10-A-02(a) was rejected to avoid. The seed must ADD only config issuers absent from the registry and never touch an existing (incl. revoked) record. | **M10.2 design** — lock the seed semantics (add-only, idempotent, revoke-wins). Sharpens brief §4's config-seed-precedence deferred question with a recommendation. |
| **M10.2-A5** | S4 → ACKNOWLEDGED (honesty; reflected in impl — no SignedPrimitive trait, T1 = key-possession, J-364) | A4/A5 framing | **(i) "SignedPrimitive" is a concept, not a trait** — the brief §3.4 wording could imply a trait to implement; there is none. The binary's signing surface is concretely `TrustAssertion::sign` + `keypair::{generate,save,load}`. **(ii) "Tier-1 verification" in code = proof-of-key-possession only** — no contact claims required by default; the issuance logic must attest only what it checks (key control), not KYC. | **Noted** — no build action; an honesty constraint on the §8 issuance design + a guard so the design doesn't hunt for a `SignedPrimitive` trait. |

*The two locked calls (M10-A-02, M10-A-06) hold — no finding reopens a fork. A1 + A2 are the load-bearing
design calls; A3 + A4 are mechanical/seed details; A5 is honesty framing. The `endpoint_url` = offline-signer
grounding (A3) is a clean confirmation, not a finding (it resolves brief §4's endpoint question in the locked
direction).*

---

## 11. Design-shaping questions teed up for Joe (don't decide here — Joe locks at design)

1. **Issuance shape (§8 / brief §4).** Offline-signed token (recommended), challenge/response (live endpoint,
   future), or operator-mediated. Recommendation: **(b) offline** for M10.2.
2. **Config-seed precedence (M10.2-A4 / brief §4).** Reconcile config + CRUD record naming the same issuer.
   Recommendation: **add-only, idempotent, revoke-wins**.
3. **`accepted_tiers` enforcement scope (M10.2-A1).** Does per-issuer `accepted_tiers` enforcement land in M10.2
   (gate-shape change) or stay registry bookkeeping (trusted_issuers + revoked only)?
4. **Trust-read architecture (M10.2-A2).** Live-read the registry at the gate (recommended) vs snapshot-refresh
   on CRUD mutation.

(1) + (2) are the brief §4 deferred questions, now with recommendations. (3) + (4) are the design calls that
surfaced under the locked frame.

---

## 12. Definition of Done (this audit)

- [x] Brief §3's six items each grounded to file:line on live `main` @ `a86db48` (A1–A6).
- [x] Blast radius grounded — single `validate_assertion` caller + single `accept_registration` caller + AMR-D1
      first-consumer boundary + the M10.2 empty-baseline prime invariant (A7).
- [x] Registry→policy wiring sketch delivered with the seam + startup-seed point (§7).
- [x] Issuance-shape options delivered with a recommendation (§8, offline).
- [x] Locked-fork boundaries respected (M10-A-02 / M10-A-06 not reopened; M10.3 / MP-F13 / module-manifest /
      erasure-enforcement recorded out, §9, not audit-depth).
- [x] Findings register populated — two load-bearing design calls (M10.2-A1 accepted_tiers, M10.2-A2 live-read),
      two mechanical (M10.2-A3 load-site, M10.2-A4 seed), one honesty note (M10.2-A5).
- [x] Header v1.0, Status ACTIVE.
- [ ] Audit committed (Clair's commit precedes Chat's doc-bridge; Joe pushes).

**Next after this audit:** M10.2 design — lock the Call-1b wiring shape (§7: live-read vs snapshot; accepted_tiers
scope) + the issuance shape (§8: offline-signed token) + the config-seed semantics (§11 Q2/Q4) → Joe-lock →
runbook → impl (the `xgen-auth-module` bin + the registry→policy wiring + config-seed + the end-to-end
RED-on-revert witness extending `non_local_registration_with_valid_assertion_accepted` with a registry-trusted
issuer + a revoke→reject leg) → close.
