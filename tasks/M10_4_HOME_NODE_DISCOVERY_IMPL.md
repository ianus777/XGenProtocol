# M10.4 — Production Identity→Home-Node Discovery (MP-F13) — Implementation Runbook
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

Executes the M10.4 design (`tasks/M10_4_HOME_NODE_DISCOVERY_DESIGN.md` v1.0 ACTIVE, J-370),
decisions **M10.4-D1..D5 Joe-LOCKED**. Shape B: the client learns the node's pubkey `node_id`
from an additive `AuthOk` echo and writes it into `content["home_node"]` so the Space-created
`home_node` carries the pubkey `NodeXgid` every consumer + both migration gates already expect.
**No node-side gate change** (D2). Production arc: `xgen-core` (wire + transport) + `xgen-node`
(populate caller) + `xgen-client` (capture + write). Spine-first, RED-on-revert. Clair owns the
code commits; Chat owns the doc-bridge (J-371). Joe pushes.

---

## 1. Grounding confirmed (file:line on `main` @ `ef3e5fd`)

D-078: every symbol below personally read on `main`. **Three crate-attribution corrections vs the
design/audit docs are folded here** (the design located the wire + auth fns in `xgen-common`/
`xgen-client`; both actually live in `xgen-core`). The corrections relocate touch-points and reveal
a slightly larger — but still fully additive — surface; **none forks D1–D5** (see §7).

### 1.1 The wire field (D1) — `xgen-core`, not `xgen-common`
- `TransportMessage::AuthOk { protocol_version, identity_id, timestamp }` —
  **`xgen-core/src/wire/types.rs:64-68`** (design said `xgen-common`; line numbers correct).
  Sibling variants: `Challenge` :48, `RegisterOk` :354 — both also carry no node id (audit §4 holds).
- Serde round-trip test `transport_auth_ok_round_trip` — `xgen-core/src/wire/types.rs:946-957`
  (must update for the new field).
- **Sole AuthOk constructor** = `connection.rs:375` (below); `:422` destructures; `types.rs:948` is
  the test. One populate site — no federation-handshake AuthOk to chase.

### 1.2 The node populate site (D1) — `server_authenticate`, not `app.rs`
- `Connection::server_authenticate(&mut self) -> Result<String, TransportError>` builds the AuthOk —
  **`xgen-core/src/transport/connection.rs:357`, AuthOk constructed at :375**. It has **no node-id in
  scope today**, so its signature must take one (design framed this as "populate at app.rs:632" — the
  value originates there but the construction is in xgen-core; the param is the bridge).
- The node's own pubkey id is **already in scope at the caller**: `handle_connection(..)` receives
  `home_node_id: NodeXgid` (`xgen-node/src/app.rs:1450`), threaded from
  `node_id_uri = pubkey_uri(&signing_key)` (`app.rs:632`) via the accept-loop call site
  (`app.rs:1360` → `home`). The `server_authenticate` call is `app.rs:1464`. **No new run_node
  plumbing** — `home_node_id` is right there.
- Non-production `server_authenticate` callers (must update to the new arg): test callers
  `xgen-node/src/transport/mod.rs:44, 93, 149`.

### 1.3 The client capture site (D1) — `client_authenticate`, not `xgen-client/connection.rs`
- `Connection::client_authenticate(&mut self, signing_key) -> Result<String, TransportError>` returns
  the echoed `identity_id` — **`xgen-core/src/transport/connection.rs:401`, AuthOk destructured at
  :422** (`AuthOk { identity_id, .. }` — the `..` currently drops everything else). Return type must
  widen to also surface the node_id.
- **Bind-site blast radius is THREE** (callers that bind the returned value):
  `xgen-client/src/session.rs:133` (`let auth_id = conn.client_authenticate(..)` — the **production
  capture site**, inside `ensure_connected`), `xgen-client/src/service.rs:107`
  (`let identity_id = match ..`), `xgen-node/src/transport/mod.rs:50` (`let returned_id = ..`, asserted
  at :57). **All other ~77 callers discard the Ok value** (`.await?;`, `if let Err(e) = ..`,
  `.await.ok();`, `.await.unwrap();` in statement position) → unaffected by widening the Ok type. The
  compiler is the exhaustive backstop; these three are the expected set.

### 1.4 The client write sites (D1) — exactly two signed-content writers
- `create_space` — `xgen-client/src/ops.rs:430`; resolves `home_node` (URL) at :448-451; builds the
  event at **`ops.rs:456`** `build_space_create_event(.., &home_node, ..)`; `ensure_connected` at :475.
- `create_dm_space` — `xgen-client/src/ops.rs:641`; resolves `home_node` (URL) at :664-667; builds at
  **`ops.rs:671`** `build_dm_space_create_event(&signing_key, &args.invitee, &home_node)`;
  `ensure_connected` at :718.
- **All other `home_node` references in ops.rs stay URL** (verified): `RegisterResult`/`WhoamiResult`/
  `StatusResult`/`CreateRoomResult` struct fields (display), `register`'s own node_endpoint record
  (:332/:395), `load_or_default_state` + `KnownSpace.node_endpoint` (:503/:583/:784). These are the
  client's *dial* record of which endpoint — correctly a URL (audit §1.4). Only the two
  `build_*_create_event` calls write the signed `content["home_node"]` the fix targets.
- **Ordering point (load-bearing):** both functions **build + sign the create event BEFORE
  `ensure_connected`** (ops.rs comment at :453-454 "before any network work"). Shape B needs
  `session.node_id`, which is captured during auth inside `ensure_connected` → **connect must precede
  build** for these two functions (§2).

### 1.5 SessionState (D1)
- `SessionState { home_node: String, .. }` — `xgen-client/src/session.rs:70-73` (`home_node` is the
  `ws://` URL; the T10 type-stability test at :174-202 asserts this and must stay green).
- `ensure_connected(&mut self, node_override)` — `session.rs:114-147`; opens the socket + runs
  `client_authenticate` at :133-136; this is where node_id is captured.

### 1.6 The two stall sites (D2) — confirmed; NO node-side change
- **Site 1** `migration_initiate` homed-here precondition — `xgen-node/src/admin_ops.rs:2096`:
  `Some(st) if st.home_node.as_str() == rt.node_id.as_str() => {}` else `MIG_6010`. Passes once
  `home_node` = source pubkey = `rt.node_id`. ✓
- **Site 2** cutover authority gate — `xgen-core/src/message/exchange.rs:717`:
  `Some(s) if event.sender.as_str() == s.home_node.as_str() => {}` else `SpaceMigrateAuthority` (6009).
  Source-signed cutover (`event.sender` = source pubkey) == `home_node` (source pubkey) → passes. ✓
  Defensive applier re-check — `xgen-core/src/space/state.rs:1158`
  (`event.sender.as_str() != self.home_node.as_str()` → `PermissionDenied`), then the flip
  `self.home_node = destination` at :1161 (`destination` = the migrate event's `destination_node_id`,
  an operator-supplied pubkey). ✓ **No edit to any of these three sites** — the value becoming correct
  is the whole fix.

### 1.7 The MP-C-16 witness (D3, M10.5 — not this arc)
- `mp_r2_fixed::mp_c_16_live_migration_space_rehomes` — `xgen-mptest/tests/mp_r2_fixed.rs:300-323`,
  `#[ignore]` box-gated; creates the Space via the **client** path (`create-space` at scenario
  `alice.jsonl` a2, so today `content["home_node"]` = a's URL → the RED). The box-gated re-run +
  home_node-flip-on-both (the per-Space home query the doc-comment flags at :319-322) is **M10.5**.

---

## 2. Architecture (the §5 design-close details — resolved; none forks D1–D5)

- **AuthOk field shape:** `node_id: Option<String>` with
  `#[serde(default, skip_serializing_if = "Option::is_none")]`. Additive optional → old clients ignore
  it (they already `..`-rest the variant), old nodes omit it, new clients tolerate its absence.
  **No `protocol_version` bump** (Ch3 §3.0.3 additive-field rule). RESOLVED.
- **`client_authenticate` return shape:** widen to a small struct
  `pub struct AuthOutcome { pub identity_id: String, pub node_id: Option<String> }` (in
  `xgen-core/src/transport/connection.rs`), returned `Result<AuthOutcome, TransportError>`. Faithful
  to the design's "captured in `client_authenticate`"; ripples only to the three §1.3 bind sites.
  (Sibling-style to the existing `EventConfirm` result enum in the same module.)
- **Absent-echo fallback (D1 safety; proof obligation #4):** `SessionState.node_id: Option<String>`,
  populated in `ensure_connected` from `AuthOutcome.node_id`. `create_space` / `create_dm_space` write
  `session.node_id` (the pubkey) into `content["home_node"]`; **if it is `None`, return an explicit
  error** — `"home node did not advertise its node_id (older protocol?); refusing to create a Space
  with a transport URL as home_node"` — **never silently fall back to the URL** (that reintroduces the
  bug). A current node always sends it, so this is a should-never-fire guard, not a UX path. RESOLVED
  (Joe-call only if he wants different copy).
- **`--node-override` ↔ node_id:** the override is a *transport URL*; `ensure_connected(node_override)`
  dials it and captures **that** node's `node_id` from its AuthOk into `session.node_id`. Reading
  `session.node_id` after `ensure_connected` therefore always matches the connection actually used.
  RESOLVED by the connect-before-build reorder (below).
- **Connect-before-build reorder (§1.4):** in `create_space` and `create_dm_space`, call
  `ensure_connected` (capturing `session.node_id`) **before** `build_*_create_event`; pass
  `session.node_id` (not the URL) as the create event's `home_node`. Mind the borrow scoping —
  `ensure_connected` takes `&mut self` on the session while the build needs `signing_key` from
  `session.identity`; resolve the pubkey into a local `String` before the connection borrow (mirror the
  existing scoped-borrow pattern already in these functions). The "assigned IDs before network"
  rationale is preserved for everything except the `home_node` value, which now legitimately depends on
  the connection.

---

## 3. Commit plan (3 work commits; each builds clean + `cargo test --workspace` green + clippy clean)

> CARGO_TARGET_DIR = `C:/cargo-targets/XGenProtocol`. Run the **workspace test** before any
> `--features harness-control` build (the workspace default build clobbers the harness-control node
> binary — the standing binary-clobber note). No "commit pushed" line — Joe pushes.

### C1 — wire echo + node populate (`xgen-core` + `xgen-node`) — the spine
1. `wire/types.rs:64-68` — add `node_id: Option<String>` to `AuthOk` (serde attrs per §2).
2. `transport/connection.rs:357` — `server_authenticate(&mut self, node_id: &str)`; populate
   `AuthOk { .., node_id: Some(node_id.to_string()) }` at :375.
3. `xgen-node/src/app.rs:1464` — `conn.server_authenticate(home_node_id.as_str())` (in scope).
4. `xgen-node/src/transport/mod.rs:44, 93, 149` — pass a node_id string to the test callers.
5. Update `transport_auth_ok_round_trip` (types.rs:946) — assert `node_id` round-trips when `Some`,
   and a back-compat case (JSON without `node_id` deserializes to `None`).
6. **Witness C1 (RED-on-revert):** a wire/transport test asserting AuthOk serializes `node_id` when
   the node populates it (revert step 2 → field absent/`None` → RED).

### C2 — client capture + create-path write + safe fallback (`xgen-core` + `xgen-client`) — load-bearing
1. `transport/connection.rs` — add `AuthOutcome` struct; widen `client_authenticate` →
   `Result<AuthOutcome>`; build it at :422 from `AuthOk { identity_id, node_id, .. }`.
2. Update the **three** bind sites (§1.3): `session.rs:133` (capture `out.node_id` into
   `SessionState.node_id`; keep `tracing` on `out.identity_id`), `service.rs:107`
   (`Ok(out) => out.identity_id`), `transport/mod.rs:50`+`:57` (`.identity_id`). Let the compiler
   surface any others.
3. `session.rs:70` — add `SessionState.node_id: Option<String>` (default `None` in `new`).
4. `ops.rs` `create_space` + `create_dm_space` — reorder connect-before-build (§2); write
   `session.node_id` into `content["home_node"]`; error on `None` (§2 fallback). Keep all other
   `home_node`/`node_endpoint` URL records unchanged.
5. **Witness 1 (RED-on-revert):** after a `create_space` via the client builder path with a captured
   node_id, the resulting `SpaceState.home_node` equals the pubkey node_id, not the URL (revert step 4
   → URL → RED). The fast witness that fixes the in-process blind spot (audit §1.2).
6. **Witness 4 (RED-on-revert):** `node_id == None` → `create_space` returns the explicit error and
   writes nothing (revert the guard → it would write the URL → RED).

### C3 — namespace-reconciliation witnesses (D2 proof; test-only, no production change)
Proves both stall sites clear with a pubkey `home_node` — confirming D2's "no node-gate change."
1. **Witness 2 (xgen-node, RED-on-revert):** a `SpaceState` whose `home_node` = the node's pubkey
   passes the `migration_initiate` homed-here precondition (no `MIG_6010`); with a URL `home_node` it
   fires `MIG_6010`. (Drive the precondition equality directly or via the admin op against an
   in-process runtime — Clair's call on the lightest faithful form.)
2. **Witness 3 (xgen-core, RED-on-revert):** a source-signed `state.space_migrate` against a
   pubkey-`home_node` `SpaceState` is **not** rejected `6009` by `validate_event` and
   `apply_space_migrate` flips `home_node` to the destination; against a URL `home_node` it is rejected
   `6009`. (Pure in-process; no binary spawn — this is the "fast witness," distinct from the box-gated
   MP-C-16 at M10.5.)

---

## 4. Witnesses (RED-on-revert; the design's §4 four + the C1 spine)

| # | Witness | Crate | Commit | RED on revert of |
|---|---|---|---|---|
| C1 | AuthOk carries `node_id` when the node populates it | xgen-core | C1 | populate at connection.rs:375 |
| 1 | client `create_space` writes the pubkey node_id into `content["home_node"]` (not the URL) | xgen-client | C2 | the ops.rs write |
| 4 | absent echo (`node_id=None`) → explicit error, no URL write | xgen-client | C2 | the fallback guard |
| 2 | pubkey `home_node` clears Site 1 (`MIG_6010`); URL fires it | xgen-node | C3 | (test-only; URL value) |
| 3 | pubkey `home_node` clears Site 2 (`6009`) + applier flips; URL fires `6009` | xgen-core | C3 | (test-only; URL value) |
| (M10.5) | box-gated MP-C-16 end-to-end re-home green | xgen-mptest | — | deferred to M10.5 (D3) |

---

## 5. Definition of Done — SHIPPED

- [ ] AuthOk carries additive optional `node_id`; `server_authenticate` populates it from the node's
      pubkey id; `transport_auth_ok_round_trip` updated (Some + back-compat None).
- [ ] `client_authenticate` returns `AuthOutcome`; the three bind sites updated; no other caller broke.
- [ ] `SessionState.node_id` captured in `ensure_connected`; `create_space` + `create_dm_space` write
      the pubkey node_id into `content["home_node"]` (connect-before-build), with the explicit
      absent-echo error (no silent URL write).
- [ ] No edit to the three D2 stall sites (admin_ops.rs:2096 / exchange.rs:717 / state.rs:1158).
- [ ] All five fast witnesses (C1, 1, 2, 3, 4) present and genuinely RED-on-revert (recorded).
- [ ] `cargo test --workspace` green (run **before** any harness-control build).
- [ ] `cargo clippy --workspace --all-targets` clean on **default and `--all-features`**.
- [ ] No DECISIONS change (M10.4-D# arc-local, D-069). No "commit pushed" line.

---

## 6. Close deliverables (for the Chat doc-bridge, J-371)

- **ch3:** record the additive `AuthOk.node_id` wire field in the transport-message spec section.
- **Appendix F:** only if operator-visible — the echo is an internal handshake field (likely **no**
  Appendix F entry); a `create_space` behaviour note only if warranted. Confirm at close.
- **Findings flips:** M10.4-A1 RESOLVED (namespace reconciled), M10.4-A2 RESOLVED (both sites),
  M10.4-A4 RESOLVED-as-Shape-B; M10.4-A3 confirmed (escape didn't fire); M10.4-A5 recorded
  (leave-as-legacy). **MP-F13 → RESOLVED at the M10.5 MP-C-16 re-run**, not at this code commit (no
  unobserved-result claim — J-352 precedent).
- **DECISIONS:** candidate only (the "Space `home_node` = pubkey node_id, written by the client from
  the AuthOk echo" invariant) — not promoted (D-069).

---

## 7. Surfaced at runbook (D-078; confirm-at-impl; none forks the locked design)

1. **Crate relocations (vs design/audit §3):** the wire types live in `xgen-core/src/wire/types.rs`
   (not `xgen-common`) and the auth fns in `xgen-core/src/transport/connection.rs` (not
   `xgen-node`/`xgen-client`). The design's line numbers (AuthOk@64, RegisterOk@354) are correct;
   only the crate names drifted. Does not change D1 (Shape B, additive AuthOk echo, client writes
   node_id) — only relocates the touch-points.
2. **Surface slightly larger than the design implied, still additive:** D1 lands as two `xgen-core`
   API touches — `server_authenticate` gains a `node_id` param (the populate value `home_node_id` is
   already in scope at the caller, so no run_node plumbing) and `client_authenticate` widens its
   return to `AuthOutcome`. Both are narrow + additive; consistent with the Joe-locked "narrow — just
   the echo + the write, NOT the full re-home-notify broadcast."
3. **Connect-before-build reorder** in `create_space`/`create_dm_space` (§1.4 / §2) — a real but
   contained restructure; the only behavioural shift is that the `home_node` value now depends on the
   connection (correct under Shape B). Flagged; not a fork.
4. **`client_authenticate` bind-site count = 3** (§1.3); the ~77 discarding callers are unaffected by
   the Ok-type widen — the compiler is the backstop if any were miscounted.

None of (1)–(4) contradicts a locked decision (D-065/D-078), so no re-lock — these are the file:line
confirmations the design asked Clair to make at runbook. Proceed to C1.

---

## Close (J-371)

**COMPLETED — shipped + verified.** C1 `77c906d` (AuthOk echo + node populate, 1392/0) → C2 `59e9193`
(client capture + pubkey write + absent-echo refusal + connect-before-build, 1394/0) → C3 `de53cf0`
(D2 both-stall-site witnesses, 1397/0). `cargo test --workspace` 1397/0 (+7 over the 1390 baseline);
clippy clean default + all-features; all witnesses RED-on-revert verified. Chat doc-bridge J-371
(ch3 v0.55 auth_ok.node_id field; design + audit COMPLETED; findings A1/A2/A4 RESOLVED, A3 confirmed,
A5 recorded; MP-F13 → M10.5). No DECISIONS change (arc-local, D-069). Next: M10.5.
