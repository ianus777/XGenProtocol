# M8.5-C — S5 Identity Re-bind Implementation Runbook
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-06  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. Scope

Implementation runbook for **M8.5-C** (S5 identity re-bind). Builds the two
missing identity-mobility surfaces per the locked design
`tasks/M8_5_C_S5_REBIND_DESIGN.md` v1.1 (S5-D1..D4). Two commits: **C1** (node +
core), **C2** (client). Clair implements after each Joe-lock checkpoint.

Entry per Rule 0: CLAUDE.md PLAY → JOURNAL J-276 → design v1.1 → this runbook.

**Locked decisions (D-069 arc-local):** S5-D1 re-registration per §3.13.8;
S5-D2 reuse existing transport challenge (no new challenge; 3001 is the
ownership backstop; **no new error code**); S5-D3 delta-shaped signed
version-guarded `HomeChanged`; S5-D4 two surfaces only, orchestrated
`recover-identity` deferred.

---

## 2. Grounding anchors (live `d2aa24c`; Clair re-confirms via CP before editing)

- `IdentityMessage::Register` — `xgen-core/src/wire/types.rs:326`
  (`is_ai` skip-when-false precedent at `:333`).
- `IdentityReplicateMessage` — `types.rs:617` (`Replicate` / `ReplicateAck`).
- `accept_registration` — `xgen-core/src/identity/registration.rs:312`;
  Step 1 (3001 `IdentityMismatch`) `:342`; Step 3 (3007 `AlreadyRegistered`) `:350`.
- `sign_register` `:266`, `verify_register` `:282`, canonical field order `:38`.
- `handle_incoming_replicate` — `xgen-core/src/identity/replication.rs:120`;
  `ReplicationError::VersionStale` (3020) `:29`.
- Register handler — `xgen-node/src/app.rs:2717` (`already` + `accept_registration`).
- Inbound replicate dispatch — `app.rs:2419` (`Inbound::IdentityReplicate`);
  variant arms `:2823` / `:2830`; applier call `:2847`.
- Client register flow — `xgen-client/src/app.rs:2102`; `RegisterArgs` `:459`.
- Stored home pointer — `IdentityRecord.home_node` (`NodeXgid`) + `update_version`;
  persisted via `identity_registry.save()`.

---

## 3. C1 — node + core

### 3.1 `re_registration` flag (S5-D1/D2)

1. Add to `IdentityMessage::Register` (`types.rs:326`), after `is_ai`'s pattern:
   ```rust
   #[serde(default, skip_serializing_if = "is_false")]
   re_registration: bool,
   ```
2. **CP-1** — confirm `re_registration:false` serializes identically to today
   (field omitted) so existing registrations' canonical signing bytes are
   unchanged. Add a round-trip assertion.
3. Update **every** `IdentityMessage::Register { .. }` construction site
   (the `..` rest-pattern matches keep compiling; explicit constructions in
   tests + the client need the field — grep `Register {` in
   `xgen-core`/`xgen-node`/`xgen-client`).
4. `accept_registration` (`registration.rs:312`) — add `re_registration: bool`
   param (read from the variant). **CP-2** — confirm Step 3 (`:350`) is the only
   duplicate gate. Replace:
   ```
   if already_registered { return Err(AlreadyRegistered); }
   ```
   with: reject only when `already_registered && !re_registration`; when
   `already_registered && re_registration`, permit re-home — build the record
   with `home_node` = this Node and `update_version` = prior + 1 (read the prior
   via the registry; the handler already has `already`/registry access).
   **No new error code** (3001 Step-1 + 3007 Step-3 unchanged; 3022 dormant).
5. Thread the flag at the handler (`app.rs:2717`) into the `accept_registration`
   call.

### 3.2 `identity.home_changed` (S5-D3)

1. Add the variant to `IdentityReplicateMessage` (`types.rs:617`):
   ```
   #[serde(rename = "identity.home_changed")]
   HomeChanged {
       protocol_version: String,
       identity_id: String,
       old_home_node_id: String,
       new_home_node_id: String,
       new_home_node_url: String,
       update_version: u64,
       timestamp: String,
       #[serde(skip_serializing_if = "Option::is_none")]
       signature: Option<String>,
   }
   ```
2. Builder + sign + verify — mirror `sign_register`/`verify_register`
   (`registration.rs:266/282`) with a canonical field order for `home_changed`
   (signature excluded, lexicographic per the §3.6.3 convention at `:38`).
   **CP-3** — confirm the identity-keypair signing helper is reachable here (one
   signing path, no new crypto).
3. Applier `handle_incoming_home_changed(msg, &mut registry)` in `replication.rs`
   beside `handle_incoming_replicate`, same `Result<(), ReplicationError>`:
   - verify signature vs `identity_id` pubkey;
   - `registry.get(identity_id)`:
     - `Some(existing)` with `update_version <= existing.update_version` ⇒
       `Err(VersionStale {..})` (3020);
     - `Some(existing)` newer ⇒ re-point: `home_node = new_home_node_id`,
       `update_version = msg.update_version`, `upsert`;
     - `None` ⇒ **no-op + `tracing::info!` log** (peer holds no record; it pulls
       via the existing replicate/refresh path).
   **CP-4** — confirm `home_node` is the single persisted home pointer and
   `save()` persists it after `upsert`.
4. Node receive arm — add `IdentityReplicateMessage::HomeChanged { .. } =>` beside
   `:2823`/`:2830` under `Inbound::IdentityReplicate` (`:2419`); call the applier
   (mirror `:2847`); persist via `identity_registry.save()`. Notification only —
   **no ack** sent.

### 3.3 C1 tests (D-078)

- `registration.rs` — re-registration permitted with flag + `already_registered`
  (record re-homed, `update_version` bumped); rejected without flag (3007,
  unchanged); fresh registration unaffected by the flag either way.
- `replication.rs` — `handle_incoming_home_changed`: newer version re-points;
  stale ⇒ 3020; no-prior-record ⇒ no-op (registry unchanged).
- `types.rs` — `HomeChanged` round-trip (rename tag + all fields); sign↔verify
  round-trip.

### 3.4 C1 doc-close (rides the commit, D-074)

ch3 §3.13.9 — mark `identity.home_changed` realized; §3.13.10 — note 3022/3023
dormant (3001 covers mismatch under the challenge model). No DECISIONS change.

### 3.5 C1 DoD

- [ ] `re_registration` field + `is_false` skip; CP-1 round-trip green.
- [ ] `accept_registration` Step-3 bypass; CP-2 confirmed; re-home bumps version.
- [ ] `HomeChanged` variant + sign/verify; CP-3 confirmed.
- [ ] `handle_incoming_home_changed` applier; CP-4 confirmed; node receive arm.
- [ ] C1 tests green; clippy `-D warnings` clean both feature sets.
- [ ] ch3 §3.13.9/§3.13.10 doc-close edits in the same commit.
- [ ] `Status: COMPLETED` header set on close (this is the shipped signal).

---

## 4. C2 — client

### 4.1 `--re-registration` (S5-D2/D4)

1. `RegisterArgs` (`xgen-client/src/app.rs:459`) gains:
   ```rust
   /// Re-register an existing Identity on this Node (orphan recovery, §3.13.8).
   #[arg(long)]
   pub re_registration: bool,
   ```
2. Thread into the `identity.register` build/send (`app.rs:2102` register flow) —
   set the new wire field before `sign_register`.

### 4.2 `home_changed` emit (S5-D4 step 5)

1. After a successful re-registration (`RegisterOk`), build + sign a
   `HomeChanged` (old home from the prior record / client config; new home = the
   `--node` target id + url; `update_version` = the re-homed record's version).
2. **CP-5** — confirm the client's known-peer set (Space-membership home nodes /
   connected peers) is sourceable without a new discovery surface; if not, scope
   the emit to currently-connected peers and note it. Send to that set.

### 4.3 C2 tests (mirror M8.5-B e2e against an ephemeral stub Node)

- `--re-registration` drives the wire flag; a re-home against the stub yields
  `RegisterOk`.
- `home_changed` is emitted and observable at the peer (mirror the
  `get_invite_bootstrap` e2e harness shape from M8.5-B C2).

### 4.4 C2 doc-close (rides the commit, D-074)

ch3 §3.6.3 — mark `re_registration` realized on `identity.register`; §3.13.8 —
note the client-emit-on-re-home flow as built.

### 4.5 C2 DoD

- [ ] `--re-registration` flag wired through to the wire field.
- [ ] `home_changed` emit post-re-home; CP-5 confirmed/noted.
- [ ] C2 e2e tests green; clippy clean both feature sets.
- [ ] ch3 §3.6.3/§3.13.8 doc-close edits in the same commit.
- [ ] `Status: COMPLETED` header set on close.

---

## 5. Close (doc-only, after C1+C2)

- Resolve audit `tasks/M8_5_FINALIZATION_AUDIT.md` §5 — M85-A8/A9/A10 RESOLVED
  banner (mirror the §3 INV banner). Audit Status → COMPLETED (S5 was the last
  open item; the finalization box closes).
- S5-D# promotion eval (design §9): expect all arc-local (D-069) — re-confirm.
- ROADMAP bump; JOURNAL close entry; PLAY flip M8.5-C CLOSED → **next-active M9**;
  design + runbook Status → COMPLETED.
- Suite count recorded; no DECISIONS change anticipated.

---

## 6. Order of play

Joe-lock checkpoint **#1** (CP-1/CP-2/CP-3) → Clair C1 → checkpoint **#2** (CP-4 +
C1 review) → C2 after checkpoint **#3** (CP-5) → checkpoint **#4** (end-to-end
re-home green) → doc-only close. Joe pushes each commit; Clair never pushes.

Per Rule 0 + D-065 + D-069 + D-071 + D-074 + D-077 + D-078.
