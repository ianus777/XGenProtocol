# Auth-Module-Registry — Implementation Runbook (D-071 arc)
> **Status**: ACTIVE  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-31  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Arc position

Implementation runbook for the **auth-module-registry** D-071 arc. Design LOCKED in
`tasks/M6_AUTH_MODULE_REGISTRY_DESIGN.md` v1.0 (AMR-D1/D2/D3, 2026-05-31). Audit:
`tasks/M6_AUTH_MODULE_REGISTRY_AUDIT.md`. Sibling worked example: the federation-policy
2b runbook `tasks/M6_FEDERATION_POLICY_IMPL.md` (COMPLETED). Executor: Clair, after each
checkpoint fires. **5 commits** (one more than the FAC arcs — C1 is a protocol-identity-model
prerequisite).

## Locked inputs (from the design doc — do not re-open)

- **AMR-D1 standalone:** ship record + store + 5 verbs. Registry-consultation wiring
  (registration steps 5–7 / Trust-Assertion-accept, `AuthModuleUntrusted`/3006) is an
  **explicit deferral** to its future arc. No runtime consumer reads the registry this arc.
- **AMR-D2 flavour:** new **`AuthModuleXgid`** principal flavour (7th), key-derived principal
  URI `xgen://pubkey/ed25519:<key>` — reuses the existing principal prefix + `principal_uri`/
  `principal_decode` (no new wire shape). Expands the D-072 six-flavour set → Appendix J §J.2 +
  graduates to a global **D-###** in DECISIONS.md.
- **AMR-D3 derive-don't-store:** `module_id: AuthModuleXgid` is the single source of truth;
  the `VerifyingKey` is recovered via `.pubkey()`. **No separate `public_key` field.**
- **A2-D1 block-only revoke / A2-D2 ad-hoc test** (locked in §6.A2) — agenda inputs.
- **Prime invariant:** empty registry = today, byte-for-byte (no consumer; trivially held —
  assert by the existing suite staying green throughout).

## Checkpoints (Joe-lock)

- **Checkpoint #1 — before Commit 1.** Pin the data-layer concretes by name against the
  Node/Identity + federation-policy precedents (D-078: doc shape ≠ live shape):
  - **Flavour:** name `AuthModuleXgid`; principal-flavour via `declare_flavour!`; `from_pubkey(&VerifyingKey)`
    (infallible, `principal_uri`) + `pubkey() -> Result<VerifyingKey, XgidDecodeError>`
    (`principal_decode`) — sibling to `NodeXgid`/`IdentityXgid`, **no new prefix**.
  - **Appendix J §J.2 edit wording** (six → seven) + the global **D-### text** for DECISIONS.md
    (AMR-D2 graduation). *Confirm the appendix file path at pickup.*
  - **`AuthModuleRecord` field set:** `module_id: AuthModuleXgid`, `endpoint_url: String`,
    `accepted_tiers: Vec<AuthTier>`, `registered_at`, `revoked: bool`, `revoked_at: Option<…>`
    (no `public_key`, AMR-D3). *Confirm the timestamp type used by sibling records at pickup.*
  - **Store API + path:** `AuthModuleRegistry.modules: HashMap<AuthModuleXgid, AuthModuleRecord>`;
    `new`/`register`/`revoke`/`set_tiers`/`get`/`all`/`len`/`is_empty`/`save`/`load(&Path)`;
    reuses `RegistryError`; on-disk `data_dir.join("xgen-node_auth_modules.json")`.
  - **`register` input surface:** `--pubkey` (operator pastes the module's Ed25519 key → the
    verb derives `module_id` via `from_pubkey`, so a malformed id is impossible) **vs**
    `--module-id` (paste the URI). *Lean: `--pubkey`.*
- **Checkpoint #2 — at Commit 4 (the probe; A2-D2 ad-hoc).** The genuine design-latitude piece.
  Pin: challenge/response **message shape**; **timeout**; the **"reachable" definition**; and how
  the module's **reported tiers** relate to the stored `accepted_tiers` (lean: report-only, no
  auto-update in v1); and whether unreachable is a *result* (probe is READ) vs a verb error
  (lean: unreachable = result; only unknown-module is an error).

## Commits

### Commit 1 — `AuthModuleXgid` flavour + protocol-identity-model updates
- NEW seventh `declare_flavour!(AuthModuleXgid, "Principal-flavour XGID identifying an Auth
  Module by its Ed25519 verifying key (Appendix J §J.2).")` in
  `xgen-common/src/xgid/flavours.rs`; `impl AuthModuleXgid { from_pubkey, pubkey }` reusing
  `principal_uri`/`principal_decode` (exact sibling to `NodeXgid`/`IdentityXgid`).
- Module-level doc comment six → seven; **Appendix J §J.2** six → seven (path confirmed at pickup).
- **DECISIONS.md:** AMR-D2 → global **D-###** (text per checkpoint #1) — the flavour-set
  expansion record. *Only commit touching the protocol-identity model + DECISIONS.md.*
- Tests: `auth_module_xgid_from_pubkey_roundtrip` (sibling to the node/identity round-trips);
  deref-to-Xgid; `principal_decode` reuse (wrong-prefix / wrong-length rejection inherited).
- No consumer this commit (the record in C2 is the first user; a `pub` cross-crate type is not
  an unused-warning).

### Commit 2 — `AuthModuleRecord` + `AuthModuleRegistry` store (no wiring; checkpoint #1)
- NEW `xgen-core/src/auth/module_registry.rs` (sibling to `federation/federation_policy.rs`;
  declared in `auth/mod.rs`):
  - `AuthModuleRecord` — field set per checkpoint #1 (no `public_key`, AMR-D3).
  - `AuthModuleRegistry { modules: HashMap<AuthModuleXgid, AuthModuleRecord> }` — `new`/`register`
    (insert-or-replace, named for the verb) / `revoke` (set `revoked=true` + `revoked_at`, **retain**
    the record — A2-D1 block-only) / `set_tiers` / `get` / `all` / `len` / `is_empty` / `save` /
    `load(&Path)`; reuses `RegistryError`.
- NO run_node wiring this commit (unused store Arc → clippy `-D warnings`); first consumer = C3.
- Tests: serde round-trip; register insert-or-replace; revoke marks-untrusted-and-retains;
  set_tiers; get/all; save/load.

### Commit 3 — CRUD verbs (`list` / `register` / `revoke` / `set-tiers`)
- `admin_ops::auth_module_list` (READ, not audited) → `registry.all()`.
- `admin_ops::auth_module_register` (WRITE, audited) → from `--pubkey` (→ `from_pubkey` →
  `module_id`) + `--endpoint` + `--tiers`; build record (`registered_at` = now, `revoked=false`);
  `register` + `save`.
- `admin_ops::auth_module_revoke` (DESTRUCTIVE, audited) → `revoke(module_id)` + `save` (A2-D1).
- `admin_ops::auth_module_set_tiers` (WRITE, audited) → `set_tiers(module_id, tiers)` + `save`.
- `AuthModuleCommand::{List, Register, Revoke, SetTiers}` clap variants + pipe dispatch arms.
- `AdminContext` gains `auth_module_registry` (live Arc) + `require_*` + `*_path`; thread
  `run_node → start_pipe_server → dispatch_line → dispatch_admin` (sibling to the
  federation queue/policy threading).
- New admin error codes for unknown-module (revoke/set-tiers) + invalid-pubkey (register) +
  invalid-tier — pick the series at pickup, document by name. (Distinct from the deferred
  `AuthModuleUntrusted`/3006, which is the *enforcement* code, not an admin-verb error.)
- Tests: register→list round-trip; revoke marks-untrusted-but-still-listed; set-tiers; error
  paths (unknown module, malformed pubkey).

### Commit 4 — `auth-module test` ad-hoc probe (checkpoint #2)
- `admin_ops::auth_module_test` (READ, not audited) → look up the record; probe `endpoint_url`
  per the checkpoint-#2 shape; report reachability + response time + reported tiers.
- `AuthModuleCommand::Test` clap variant + pipe arm.
- Error vs result split per checkpoint #2 (lean: unknown-module = error; unreachable = result).
- Tests: probe success against a mock endpoint; unreachable result; unknown-module error.

### Commit 5 — close (doc-only)
- `docs/xgen_node_admin_ops_design.md` §6.A2 → all **5** verbs SHIPPED (honest as-built deltas
  vs the spec sketch, D-065).
- `tasks/M6_BACKING_AUDIT.md` A2 row + summary → SHIPPED; A2 arc CLOSED.
- `tasks/M6_AUTH_MODULE_REGISTRY_AUDIT.md` + `..._DESIGN.md` + this runbook → COMPLETED.
- CLAUDE PLAY flip → next D-071 arc (**bootstrap-client**). ROADMAP A2 row ✅.
- Confirm DECISIONS.md AMR-D2 global D-### present (landed at C1).
- JOURNAL close entry. Full verification + the empty-registry-is-today assertion.

## Definition of done (per commit)
- `cargo build --workspace --all-targets` 0 errors / 0 warnings.
- `cargo test --workspace` green (real counts recorded in JOURNAL, Rule 2); new tests named.
- `cargo clippy --workspace --lib --tests --all-features -- -D warnings` clean.
- Empty-registry-is-today: existing suite green throughout (no consumer wired, AMR-D1).
- Canonical docs updated in the same commit as the state change (D-069); the protocol-identity-
  model touch (Appendix J + DECISIONS.md D-###) lands at C1 only.
- (No "commit pushed" checklist item — the COMPLETED header is the signal; Joe pushes.)

## Cross-refs
- Design: `tasks/M6_AUTH_MODULE_REGISTRY_DESIGN.md` (AMR-D1/D2/D3 LOCKED). Audit:
  `tasks/M6_AUTH_MODULE_REGISTRY_AUDIT.md`.
- `docs/xgen_node_admin_ops_design.md` §6.A2 (A2-D1/A2-D2) + Appendix K.2.5.
- `xgen-common/src/xgid/flavours.rs` (six-flavour set, D-072 / Appendix J §J.2 — the C1 target);
  `xgen-core/src/auth/tiers.rs` (`AuthTier`); `xgen-core/src/identity/registration.rs`
  (steps 5–7, error 3006 — the deferred consultation point).
- Federation-policy precedents: `xgen-core/src/federation/federation_policy.rs`, the policy-store
  threading in `pipe.rs`/`admin_ops.rs`. D-035 / D-065 / D-067 / D-069 / D-072 / D-074 / D-078.

---

*Runbook ACTIVE. Clair executes from Commit 1 after checkpoint #1.*
