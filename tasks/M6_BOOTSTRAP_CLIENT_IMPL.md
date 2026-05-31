# Bootstrap-Client — Implementation Runbook (D-071 arc)
> **Status**: COMPLETED  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-31  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Purpose

Sequenced implementation plan for the **bootstrap-client** arc. Design is LOCKED in
`tasks/M6_BOOTSTRAP_CLIENT_DESIGN.md` (BC-D1/D2/D3 + scope boundary); this runbook
turns it into Clair commits. Audit (reality map): `tasks/M6_BOOTSTRAP_CLIENT_AUDIT.md`.

**Scope (A3-D1, client-only):** this Node registers *itself* with Bootstrap Nodes
and manages its own advertisement. The 5 verbs: `bootstrap show` (READ) ·
`register` (WRITE) · `deregister` (DESTRUCTIVE) · `set-info` (WRITE) ·
`set-tiers` (WRITE). **Directory-fetch is OUT of scope** (BC-D3 / scope boundary).

## Prime invariant (mandatory regression at C3, D-065)

No `[bootstrap]` config + empty registrations store = registers with nobody =
**today byte-for-byte**. An explicit regression test lands with the first wiring
commit and must stay green every commit thereafter.

## Baseline

`cargo test --workspace` at arc open (post-J-189): **793** passed / 0 failed /
1 ignored. Each commit reports its delta.

## Checkpoints (Joe-lock before the named commit)

- **#1 — CLOSED (J-190, all four pins LOCKED).**
  - **(a) Data-layer names** — `BootstrapRegistration` + `BootstrapRegistrationStore`
    + `BootstrapSelfInfo`. Store file `xgen-node_bootstrap.json` (D-035 sibling).
  - **(b) Store files** — **ONE combined file** (`xgen-node_bootstrap.json` =
    registrations map + self-info record), sibling to the single-file precedent.
  - **(c) `register` input shape** — `--url` (bootstrap node) + `--pubkey` (the
    bootstrap node's key, used to verify the `register_ack` signature). The Node's
    OWN endpoint / region / capabilities are pulled from existing config, **not
    re-typed** (mirrors A2 `register` derive-don't-retype discipline).
  - **(d) `set-tiers` mapping — Option A (LOCKED).** Code-trace verdict:
    `set-tiers` operates on `auth_tiers_served: Vec<u8>` (modular Tier 1–4 set), but
    **that value has NO field in the `Register`/`Keepalive` wire frames nor in
    `BootstrapInfo`** (frames carry only `capabilities: Vec<String>` of `xgen.*`
    tokens). Therefore `set-tiers` writes `auth_tiers_served` to the **local
    self-info store only**; `show` displays it; **re-advertise is a documented
    no-op for tiers** (the wire cannot carry it). A wire extension to propagate
    tiers (Option B) is a wire-format change → deferred to a separate protocol-design
    arc (hard Joe-lock), explicitly OUT of A3. This is an honest as-built delta vs
    the §A3 sketch's `federate`-stage tier re-advertise (D-065) — record it at C5.
  - **A3-D2 scope refinement (consequence of (d)):** best-effort re-advertise (C4)
    applies to **`set-info`** (endpoint/region — these DO map to `Register` fields);
    for **`set-tiers`** the `federate` stage is a documented no-op.
- **#2 — CLOSED (J-192, three pins LOCKED).**
  - **Pin 1 — send-path mechanics → reuse `Connection` primitives, NO transport auth.**
    New xgen-node module orchestrates `connect_url(url)` → `conn.send_bootstrap(signed
    Register)` → `conn.recv()` matching `Inbound::Bootstrap(RegisterAck)`. xgen-core
    stays pure (BC-D3). **No `client_authenticate`** — spec §3.14.3: the Bootstrap
    Node verifies the *message* signature; there is no transport challenge-response
    in the bootstrap flow (unlike federation). Adding one would invent wire behaviour
    the spec doesn't define (interop break, not hardening). Standalone client rejected
    (drift vs `send_bootstrap`, D-067).
  - **Pin 2 — ack verification key → operator `--pubkey` / stored `bootstrap_id`, with
    `node_id`-match assertion.** Verify the `register_ack` signature against the
    `bootstrap_id` the operator chose at register time (the checkpoint-#1(c) `--pubkey`),
    AND assert the ack's `node_id` equals it — closes MITM-at-URL (a peer answering the
    URL cannot self-declare an identity). Mirrors reconnect.rs's "peer at this URL must
    be who I expected". **Same key + same match-assertion applies to `keepalive_ack`**
    (verified against the *stored* `bootstrap_id` for that registration, not fresh input).
    Verifying against the ack's own self-declared `node_id` is rejected (circular).
  - **Pin 3 — keepalive scheduler → separate `bootstrap_keepalive.rs`, reconnect.rs
    shape, independent lifecycle.** New module mirroring reconnect.rs structure
    (`spawn_*` → extracted `*_tick` for tests → per-due detached spawn), but its own
    store / exchange / timing (refresh-before-TTL vs backoff ladder). NOT folded into
    the federation reconnect scheduler (entangles unrelated lifecycles). Decided now
    (C4 territory) so C2's send-path exposes the seam the C4 scheduler reuses.

---

## C1 — data layer (config + store), no wiring

**Files:** `xgen-node/src/app.rs` (config); NEW
`xgen-core/src/bootstrap/registration_store.rs` (sibling to `federation/pending_queue.rs`),
declared in `bootstrap/mod.rs`.

- `[bootstrap]` `BootstrapSection` on `NodeConfig` with `#[serde(default)]` (absent =
  empty = prime invariant). Seed fields only: bootstrap nodes to auto-register
  (url + pubkey/directory_url), optional initial region/capabilities/tiers.
- `BootstrapRegistration { bootstrap_id: NodeXgid, url, directory_url, registered_at,
  expires_at, … }` + `BootstrapRegistrationStore` (`new`/`add`/`remove`/`get`/`all`/`len`/`is_empty`/`save`/`load`,
  reuse `RegistryError`). `BootstrapSelfInfo { endpoint, region, capabilities: Vec<String>,
  auth_tiers_served: Vec<u8> }`. **ONE combined store file** `xgen-node_bootstrap.json`
  (registrations map + self-info record), per Checkpoint #1(b). **No run_node wiring**
  (an unused store Arc trips clippy `-D warnings`; first consumer = C3).

**DoD:** new types + store unit tests (round-trip, add/remove/get, backward-compat
config load without `[bootstrap]`); `cargo test --workspace` green (+N); clippy
`-D warnings` clean; build all-targets 0/0. Prime invariant trivially held (no consumer).

## C2 — framed send-path (the load-bearing commit) · checkpoint #2 CLOSED

**Files:** NEW send-path module in `xgen-node` (per BC-D3 — `xgen-core` stays pure);
reuse the `Connection` primitives (`connect_url` / `send_bootstrap` / `recv`), Pin 1.

- `register` / `keepalive` / `deregister` **send** + `*_ack` **receive and
  signature-verify**, driving the existing `BootstrapMessage` wire types over the
  framed transport (NOT HTTP). Sign outbound with the Node keypair. **No transport
  auth** (Pin 1) — orchestrate `connect_url → send_bootstrap(signed) → recv` only.
- Verify each ack against the **operator `--pubkey` / stored `bootstrap_id`** AND
  assert the ack's `node_id` matches it (Pin 2). Same rule for `register_ack` and
  `keepalive_ack`.
- Pure, testable seams (message construction + ack verification as functions; the
  socket exchange thin). Shape the send-path so the C4 `bootstrap_keepalive.rs`
  scheduler (Pin 3) reuses it.

**DoD:** unit tests for message build + ack verify — accept good sig, **reject bad
sig, AND reject `node_id` mismatch** (the MITM-at-URL case, Pin 2); an integration
test exercising a real register→ack round-trip against an in-process stub bootstrap
responder; `cargo test --workspace` green (+N); clippy clean; build 0/0. No verb
wiring yet (verbs = C3) — keep this commit send-path-only if an unused path would
trip clippy, else gate behind C3.

## C3 — the 5 verbs + AdminContext threading · PRIME-INVARIANT REGRESSION

**Files:** `xgen-node/src/admin_ops.rs`, `AdminContext`, clap `BootstrapCommand`,
pipe dispatch arms; thread the live store Arc (`run_node → start_pipe_server →
dispatch_line → dispatch_admin`, sibling to the A1/A2 threading, D-067).

- `bootstrap_show` (READ) — reads the registrations store.
- `bootstrap_register` (WRITE, audited) — drives the C2 send-path, on `register_ack`
  adds to the store (records `directory_url` from the ack).
- `bootstrap_deregister` (DESTRUCTIVE, audited) — sends `deregister`, removes from store.
- `bootstrap_set_info` / `bootstrap_set_tiers` (WRITE, audited) — write the self-info
  store locally first (A3-D2). `set-info` re-advertise deferred to C4; `set-tiers`
  re-advertise is a documented no-op (Checkpoint #1(d), Option A — wire carries no tiers).
- New admin error codes in the bootstrap sub-block (unknown-bootstrap-node /
  unreachable / bad-input) — distinct names per D-067; numbering pinned in-commit.

**DoD:** per-verb tests; **explicit prime-invariant regression** (no `[bootstrap]`
config + empty store → no bootstrap traffic, existing node behaviour byte-for-byte);
`cargo test --workspace` green (+N); clippy clean; build 0/0.

## C4 — keepalive scheduler + best-effort re-advertise (A3-D2)

**Files:** `xgen-node` scheduler task (mirror `reconnect.rs`); hook into
`set-info`/`set-tiers`.

- Periodic best-effort `keepalive` to each registration before `expires_at`; update
  `expires_at` on `keepalive_ack`. Spawned at `run_node` only when registrations exist.
- `set-info` re-advertise (re-register or keepalive carrying new endpoint/region) to
  all registered bootstrap nodes — **best-effort, local write already succeeded**
  (A3-D2); a fan-out failure does not fail the verb or roll back the local write.
  (`set-tiers` has no re-advertise — Checkpoint #1(d), wire carries no tiers.)

**DoD:** scheduler tick test (sends keepalive, updates TTL on ack); re-advertise
best-effort test (local write persists despite a failing fan-out); `cargo test
--workspace` green (+N); clippy clean; build 0/0. Prime invariant: no registrations
→ scheduler is a no-op / not spawned.

## C5 — close (doc-only)

- `docs/xgen_node_admin_ops_design.md` — §6.A3 SHIPPED banner (all 5 verbs + BC
  locks + honest as-built deltas vs the Block-4 sketch, D-065); §5.1 + category row
  A3 → ✅.
- `tasks/M6_BACKING_AUDIT.md` — A3 row ABSENT → SHIPPED ✅, arc CLOSED; remaining-arcs
  note (only node-policy remains).
- Audit + design + this runbook → COMPLETED.
- CLAUDE.md PLAY → **node-policy** (the 5th deferral) → then M7 `--aicontrol`.
- ROADMAP.md A3 row ✅. JOURNAL entry. DECISIONS.md unchanged (BC-D# arc-local, D-069).
- Verification: `cargo test --workspace` unchanged (doc-only); clippy clean.

## Conventions (carried)

- Explicit `git add <file>` per file; `git status` sanity-check before commit;
  multi-paragraph commit messages via repeated `-m`. Claude never pushes (Joe
  pushes via GitHub Desktop / PowerShell). No "commit pushed" DoD item — the
  `Status: COMPLETED` header is the real signal (D-074). Same-commit atomic close
  including JOURNAL.md (D-074).

## Cross-refs

- Design: `tasks/M6_BOOTSTRAP_CLIENT_DESIGN.md`. Audit: `tasks/M6_BOOTSTRAP_CLIENT_AUDIT.md`.
- `docs/xgen_node_admin_ops_design.md` §6.A3 + Appendix K.2.3. Spec §3.14.3 / §3.14.7.
- D-051 / D-065 / D-067 / D-069 / D-070 / D-074 / D-078. Worked-example runbooks:
  `tasks/M6_AUTH_MODULE_REGISTRY_IMPL.md`, `tasks/M6_FEDERATION_POLICY_IMPL.md`.

---

*Runbook ACTIVE. C1 SHIPPED (d4d3e69). Checkpoints #1 + #2 CLOSED — C2 is unblocked.*
