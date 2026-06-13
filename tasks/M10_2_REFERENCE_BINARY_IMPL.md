# M10.2 — Tier-1 Reference Auth Module (`xgen-auth-module`) — Implementation Runbook
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

Executes the M10.2 design (`tasks/M10_2_REFERENCE_BINARY_DESIGN.md` v1.0, J-363). D1–D5 **Joe-LOCKED — not
reopened.** First M10 sub-arc with a real new binary + a node-side validation behaviour change → heavier than
M10.1. Production arc: `xgen-core` (the live-read derivation + the seed + a `NodeRuntime` field), `xgen-node`
(the gate flip + registry relocation + startup seed), and a **new workspace member `xgen-auth-module`** (the
offline signer). Runbook commit precedes the impl commit(s); Joe pushes both. No code until this runbook lands.

## 1. Grounding confirmed (file:line on `main` @ `58ec205`)

Re-grounded fresh this session (the audit was @`a86db48`; the design commit `58ec205` shifted nothing material,
but line numbers re-verified per D-078). All anchors below are live.

**The single seam (blast-radius confirmed).** `validate_assertion` has **exactly one** production caller —
`registration.rs:486`, inside `accept_registration` (`registration.rs:419`); `accept_registration` has **exactly
one** production caller — `app.rs:2842`, inside `handle_identity_msg` (`app.rs:2814`), which itself has **one**
caller (`app.rs:2483`). The gate reads the trust policy in a short critical section
(`app.rs:2831–2840`: locks `runtime`, reads `rt.assertion_policy.clone()` + the already/prior-version, releases).

**The trust source today (the snapshot to flip).** `app.rs:745–749` builds `AssertionPolicy { trusted_issuers:
config.node.trusted_auth_modules.iter().cloned().collect(), required_claims: Vec::new(), required_tier: 1 }` and
installs it via `runtime.set_assertion_policy` (`runtime.rs:466`); stored as `NodeRuntime.assertion_policy`
(`runtime.rs:343`, init `AssertionPolicy::default()` in `new()` at `runtime.rs:421`). Config field `[node]
.trusted_auth_modules: Vec<String>` (`app.rs:132`).

**The registry (two loads today → relocate to one).** (a) startup storage-floor read `app.rs:525–539` (loads
`xgen-node_auth_modules.json` read-only at `:528`, sums `accepted_tiers` into `module_tiers` for the SE-D4
gate); (b) pipe-block load `app.rs:1164–1185` into `Arc<tokio::sync::Mutex<AuthModuleRegistry>>`
(`pipe_auth_module_registry`), threaded to **both** the aicontrol `NodeAdminDeps` (`app.rs:1231`) and the pipe
server (`app.rs:1260`) — one shared Arc for the CRUD verbs.

**`AuthModuleRegistry`** (`xgen-core/src/auth/module_registry.rs:77`): `AuthModuleRecord` (`:51`) — `module_id:
AuthModuleXgid` (`:55`), `endpoint_url: String` (`:57`, the `auth-module test` probe target — **the node never
calls the module to validate**), `accepted_tiers: Vec<AuthTier>` (`:59`), `registered_at` (`:61`), `revoked:
bool` (`:64`), `revoked_at` (`:67`). API: `register` (`:89`, **insert-or-REPLACE** — would un-revoke if reused
for the seed → the seed must NOT use it), `revoke` (`:98`, block-only), `get`/`all`/`len`/`is_empty`,
`save`/`load`. No `public_key` (key via `module_id.pubkey()`).

**Signing surface the binary reuses.** Keypair: `xgen-core/src/identity/keypair.rs` — `generate()` (`:56`),
`save(&key, &path, passphrase)` (`:62`, ChaCha20+Argon2id), `load(&path, passphrase)` (`:90`). XGID:
`AuthModuleXgid` (`flavours.rs:222`) — `from_pubkey(&VerifyingKey)` (macro-gen), `from_xgid(Xgid)` (`:139`),
`pubkey()`, `Display`→`xgen://pubkey/ed25519:<key>`. Assertion: `TrustAssertion` (`xgen-common/src/trust_assertion.rs:248`)
— `sign(self, &SigningKey)` (`:290`), `verify()` (`:309`), `canonical_bytes()` (`:279`). **`SignedPrimitive` is a
concept, not a trait** (A5) — nothing to implement. Descriptor accessors (M10.1, `01ea770`):
`TrustClaims::set_module_kind` (`:215`) / `module_kind` (`:206`, default `Reference`) / `set_module_policy`
(`:233`) / `module_policy` (`:225`); `ModuleKind::Reference` (`:148`), `ModulePolicy { erasability, .. }`
(`:161`), `Erasability { retention, .. }` (`:173`), `Retention::{Erasable,Retained}` (`:187`) — **must be set
before `.sign`** (they join the canonical bytes). `AuthTier::Tier1` (`tiers.rs:36`), `as_u32` (`:53`).

**The witness base** — `non_local_registration_with_valid_assertion_accepted` (`registration.rs:1377`) builds a
signed register with a `make_assertion` assertion + `policy_trusting(&issuer)` and asserts the validated tier is
persisted. M10.2's end-to-end witness replaces `make_assertion` with the **binary's** issuance and
`policy_trusting` with a **registry-derived** policy.

**Field-add blast radius (low).** `NodeRuntime` is constructed via a single `new(keypair)` (`runtime.rs:393`);
the other 15 `NodeRuntime {` matches are `..`-destructuring. One construction site to touch. `Arc` is imported
(`runtime.rs:22`); `tokio` is a full dep of xgen-core (`xgen-core/Cargo.toml:13`) → a `tokio::sync::Mutex`
field needs no new dep.

## 2. Architecture (the §5 design-close details, resolved — none forks; spec'd per the design's "obvious ⇒ spec it")

**A. Live-read shape = `NodeRuntime` field + lock-per-validation.** Add `NodeRuntime.auth_module_registry:
Option<Arc<tokio::sync::Mutex<AuthModuleRegistry>>>` + `set_auth_module_registry` (sibling to `assertion_policy`
/ `set_assertion_policy`). The gate already locks `rt` to read `assertion_policy`; in that same critical section
it **clones the registry Arc out** (cheap), **releases `rt`**, then locks the inner registry **briefly** to
derive the trusted set and override `policy.trusted_issuers`. Registration is a rare path and the derivation
copies a small `HashSet<String>` — **no contention footgun** (the CRUD verbs lock only the inner registry; the
gate never holds `rt` while locking it → no nested-lock cycle). This is the obvious shape — spec'd, not surfaced
to Joe. **`None` registry ⇒ empty trusted set ⇒ today** (test/baseline paths keep working).

**B. The live-read derivation lives in xgen-core (testable).** New `AuthModuleRegistry::trusted_issuers(&self)
-> HashSet<String>` = `self.all().iter().filter(|r| !r.revoked).map(|r| r.module_id.to_string()).collect()`.
The gate calls it; the witnesses unit-test it (register → present; revoke → absent; empty → empty). The gate
composition: `let mut policy = rt.assertion_policy.clone(); policy.trusted_issuers = reg.trusted_issuers();` —
`required_claims`/`required_tier` stay from the snapshot (config posture), `trusted_issuers` is live.

**C. The snapshot's `trusted_issuers` becomes config-fed-via-seed, not direct.** At `app.rs:745–749`, stop
collecting config into the snapshot's `trusted_issuers` — set it `HashSet::new()` (the snapshot now carries only
`required_claims`/`required_tier`; its `trusted_issuers` is documented unused, overridden live). Config feeds the
**registry seed** (D below), which the gate reads. End behaviour for an operator with config issuers is
preserved (seed → registry → gate trusts), migration-free.

**D. Registry relocation + config-seed.** Relocate the load to **run_node top-level, before the storage-floor
read** (~`app.rs:521`): `load` the registry → **seed** each `config.node.trusted_auth_modules` issuer add-only →
`save` if changed → wrap in `Arc::new(tokio::sync::Mutex::new(reg))` as `auth_module_registry`. Then:
- **rewire the storage-floor read** (`app.rs:525–539`) to read `accepted_tiers` from this in-memory instance
  (`auth_module_registry.lock().await`) instead of re-loading from disk — closes the "don't strand :528"
  guardrail.
- after the bare `runtime` is built (~`app.rs:745`): `runtime.set_auth_module_registry(Arc::clone(&auth_module_registry))`.
- in the pipe block (`app.rs:1164`): replace the fresh load with `let pipe_auth_module_registry =
  Arc::clone(&auth_module_registry);` → the gate (via the runtime field) + both verb servers share **one** live
  instance.

**The seed (D3) = new `AuthModuleRegistry::seed(&mut self, record) -> bool`** (add-only; returns `true` if
inserted). Per config issuer: parse `AuthModuleXgid::from_xgid(Xgid::new(s))`; **skip + warn** if
`.pubkey().is_err()` (a malformed config URI must not crash startup); build a record `{ module_id, endpoint_url:
"", accepted_tiers: vec![], registered_at: now, revoked: false, revoked_at: None }`; `seed(record)`.
- **add-only** ⇒ never touches an existing record ⇒ **revoke-wins** (a CRUD-revoked issuer has a record → seed
  skips it → stays revoked) **and idempotent** (re-boot: issuer already present → seed skips → no dup) fall out
  for free. (`register` at `:89` is insert-or-replace and would un-revoke — **do not use it for the seed**.)
- **seeded records carry empty `accepted_tiers`** — strictly storage-floor-neutral (the floor read flat-maps
  over empty → contributes nothing) and honest: M10.2 does not decide per-issuer tiers (D4 — M10.3 owns
  `accepted_tiers` enforcement; the operator-run `auth-module register --tier` still sets them for CRUD records).

**E. Empty-baseline prime invariant (the floor of the change).** Empty config + empty registry ⇒ seed adds
nothing ⇒ registry empty ⇒ `trusted_issuers()` empty ⇒ every production assertion fails step 1 (3006) ⇒
**byte-for-byte today**; `local_mode` never enters the gate (`registration.rs:472 if !local_node`) ⇒ baseline
floor untouched (Fork 1). Asserted by a witness (§4.4).

**F. The binary = library-first (arch rule #1).** `xgen-auth-module` = a lib (issuance logic, unit-testable) +
a thin `main` (CLI). The lib exposes the issuance the witnesses call; the CLI is minimal (just enough for an
operator/witness to keygen + issue), **not a product CLI**. Deps: `xgen-common` (TrustAssertion + AuthModuleXgid)
+ `xgen-core` (keypair + AuthModuleRegistry + accept_registration for the integration test). **D-092 does NOT
fire** — no node verb surface changes (the CRUD verbs are unchanged, only consulted; the binary's CLI is its own
arg parser, not the node's verb dispatch).

**Operator-visible semantic to record at close (Appendix F, D-065 honest):** with the registry as the trust
source, **removing an issuer from `[node].trusted_auth_modules` no longer un-trusts it** once seeded ("config
seeds, registry rules", D3) — the operator un-trusts via `auth-module revoke`. Locked by D3; flag it, don't
re-lock.

## 3. Commit plan (3 work commits; each builds clean + workspace-green)

**C1 — xgen-core: the live-read derivation + the seed + the runtime field (behaviour-neutral).**
- `AuthModuleRegistry::trusted_issuers()` (B) + `seed()` (D) in `module_registry.rs`.
- `NodeRuntime.auth_module_registry: Option<Arc<tokio::sync::Mutex<AuthModuleRegistry>>>` + `set_auth_module_registry`
  (A); init `None` in `new()` (`runtime.rs:421` area). Add `use tokio::sync::Mutex` / `crate::auth::module_registry::AuthModuleRegistry`
  if absent.
- **No node wiring** → the field is `None` everywhere, no consumer → behaviour-neutral.
- Unit tests: `trusted_issuers` (register→present / revoke→absent / empty→empty); `seed` (add-only insert /
  idempotent re-seed / revoke-then-seed stays revoked = revoke-wins). DoD: build 0; clippy clean (default +
  all-features); `cargo test --workspace` green (+N).

**C2 — the `xgen-auth-module` binary crate (the offline signer).**
- New workspace member (add to `Cargo.toml:[workspace].members`). `xgen-auth-module/Cargo.toml` (BSL header in
  source; deps xgen-common, xgen-core; clap for the thin CLI). Library-first: `src/lib.rs` + `src/main.rs`.
- `lib.rs`: `issue_tier1(module_key, identity_id, valid_until) -> TrustAssertion` — build `TrustAssertion {
  kind, tier: 1, issuer: AuthModuleXgid::from_pubkey(&module_key.verifying_key()).to_string(), identity_id,
  issued_at, valid_until, claims: { tier_verified: true, .. } }`, then `claims.set_module_kind(Reference)` +
  `claims.set_module_policy(&ModulePolicy { erasability: Some(Erasability { retention: Some(Erasable), .. }), .. })`,
  then `.sign(module_key)` (descriptor before sign — §1). Plus tiny helpers (`module_xgid(key)`; reuse
  `keypair::{generate,save,load}`).
- `main.rs`: `xgen-auth-module keygen --out <path> [--passphrase …]` (generate+save, print the AuthModuleXgid
  URI) + `xgen-auth-module issue --keypair <path> --identity <id> [--valid-days N] [--out <path>]` (load, issue,
  write/print JSON). Minimal.
- Witnesses: **lib unit** — `issue_tier1` output `.verify()` ok, `issuer == module_xgid`, `tier == 1`,
  `claims.tier_verified`, `module_kind() == Reference`, `module_policy().erasability.retention == Erasable`.
  **integration** `tests/end_to_end.rs` (witness 1 + 2): identity key → `build_register` with
  `trust_assertion = issue_tier1(module_key, id, FUTURE)` → `sign_register` → a registry with the module
  `register`ed → policy `{ trusted_issuers: reg.trusted_issuers(), .. }` → `accept_registration` = **Ok**;
  **RED**: empty registry / wrong issuer → Err(3006). Then `reg.revoke(module)` → `reg.trusted_issuers()` drops
  it → `accept_registration` = **Err(3006)** (live revoke; **RED**: if `trusted_issuers` ignored `revoked`).
  DoD: `cargo build -p xgen-auth-module`; the bin builds; tests green.

**C3 — xgen-node: the gate flip + relocation + startup seed (the behaviour change).**
- Relocate load+seed to run_node top (D); rewire the storage-floor read (`app.rs:525–539`) to the shared
  instance; `set_auth_module_registry` on the bare runtime; `pipe_auth_module_registry = Arc::clone(...)` at
  `:1164`.
- `app.rs:745–749`: snapshot `trusted_issuers: HashSet::new()` (C); update the comment (config → seed).
- Gate (`handle_identity_msg`, `app.rs:2826`+): in/after the `rt`-lock block, clone the registry Arc out;
  override `policy.trusted_issuers = reg.trusted_issuers()` via a brief inner lock (A).
- Node witnesses (`xgen-node/src/tests/`, accept_registration-via-registry-policy pattern per
  `identity_integration.rs` precedent): **config-seed** honoured after boot + idempotent + revoke-wins (witness
  3, partly riding C1's seed unit + a node-level boot assertion); **empty-baseline** empty config+registry =
  today + `local_mode` register still Ok (witness 4). DoD: build 0 (default; the binary-clobber hazard does not
  apply — no `harness-control` rebuild needed; run `cargo test --workspace` normally); clippy clean (default +
  all-features); `cargo test --workspace` green; **the empty-baseline witness is the regression lock.**

## 4. Witnesses (RED-on-revert; hard deliverables — map to the design's four)

1. **End-to-end accept** (C2 `tests/end_to_end.rs`) — binary `issue_tier1` → register → `accept_registration`
   against a **registry-trusted** issuer = Ok. RED: un-trust (empty/wrong registry) → 3006.
2. **Live revoke** (C2 integration) — trusted issuer `revoke`d → `trusted_issuers()` drops it → next
   `accept_registration` = 3006, **no restart** (proves the derivation is live, not a snapshot). RED: revoke
   ignored → stays accepted.
3. **Config-seed** (C1 `seed` unit + C3 node boot) — config issuer honoured after a fresh boot; re-boot
   idempotent (no dup); a CRUD-revoked issuer stays revoked across a re-seed (revoke-wins). RED: seed
   un-revokes / duplicates.
4. **Empty-baseline invariant** (C3 node) — empty config + empty registry ⇒ behaviour byte-for-byte today;
   `local_mode` register unaffected. RED: relocation/seed accidentally trusts something or breaks the baseline.

## 5. Definition of Done

- [ ] C1: `trusted_issuers()` + `seed()` + `NodeRuntime.auth_module_registry` field/setter; behaviour-neutral
      (field `None`); unit tests green.
- [ ] C2: `xgen-auth-module` workspace member (lib + thin CLI); `issue_tier1` populates the M10.1 descriptor +
      signs as itself; lib unit + `tests/end_to_end.rs` (witnesses 1+2) green; bin builds.
- [ ] C3: gate live-reads the registry; registry relocated to one shared instance (floor read rewired, not
      stranded); config bootstrap-seeds (add-only/idempotent/revoke-wins); snapshot `trusted_issuers` emptied;
      witnesses 3+4 green.
- [ ] All four witnesses have a recorded genuine RED-on-revert.
- [ ] `cargo build --workspace --all-targets` 0; clippy `-D warnings` clean (default + all-features);
      `cargo test --workspace` green (record the count delta).
- [ ] Empty-baseline prime invariant asserted (no regression to `local_mode`/baseline; the M10-A-02(a) "revoke
      that doesn't revoke" footgun is closed — revoke bites live).
- [ ] No DECISIONS change (M10.2-D# arc-local, D-069); candidates only, flagged for the bridge.
- [ ] D-092 confirmed not triggered (no node verb surface change); the binary CLI is minimal, not a product CLI.

*(Audit/runbook DoD never lists "commit pushed" — Status: COMPLETED is the shipped signal. Clair's runbook
commit precedes the impl commit(s); Joe pushes.)*

## 6. Close deliverables (for the Chat doc-bridge, J-364)

- **Appendix F** — the `xgen-auth-module` CLI (keygen/issue) operator surface; the **registry-as-trust-source**
  semantic change to `auth-module register`/`revoke` (now enforcement-bearing: register makes an issuer trusted,
  revoke un-trusts live) + the **config-seeds-then-registry-rules** note (config removal no longer un-trusts;
  use `revoke`).
- **ch2/ch4** reconcile only if the "demonstrator over baseline" framing needs a doc note (§3.8.7 already
  aligns; likely none).
- **Findings flips:** **M10-A-02 RESOLVED** (registry→policy wired, AMR-D1 structurally closed — first runtime
  consumer); **M10-A-06 RESOLVED** (operator-CRUD offline-signer binary shipped); **M10.2-A1 carried → M10.3**
  (`accepted_tiers` enforcement); M10.2-A2/A3/A4 resolved-in-impl; M10.2-A5 honoured (honesty).
- **DECISIONS:** candidates only, arc-local (D-069) — none pre-decided. Matrix/ROADMAP as applicable.

## 7. Surfaced at runbook (confirm-at-impl; none forks the locked design)

- **Seeded-record `accepted_tiers`** = empty (floor-neutral + D4-honest). If impl finds the storage gate or a
  list verb chokes on an empty-tier record, fall back to `vec![Tier1]` (also floor-neutral: T1 ≤ the floor
  minimum) — note the choice.
- **Snapshot `trusted_issuers` now unused** (overridden live) — documented at `app.rs:745`; not removed (the
  `AssertionPolicy` shape is locked by `validate_assertion`).
- **Malformed config issuer** — `seed` skips + warns (no startup crash); confirm the warn line shape.
- **`run_node` ordering** — `auth_module_registry` must be in scope from the top-level load (~`:521`) through the
  runtime build (`:745`) to the pipe block (`:1164`); confirm no early `return`/`?` strands it.
