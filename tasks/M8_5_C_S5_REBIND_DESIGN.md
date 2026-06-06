# M8.5-C — S5 Identity Re-bind Design (re_registration + identity.home_changed)
> **Status**: ACTIVE  
> Version: 1.1  
> Date: Jun 2026  
> **Last updated**: 2026-06-06  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. Purpose

Design lock for **M8.5-C** (S5 identity re-bind), the last sub-arc of the M8.5
finalization box. Builds the two missing identity-mobility surfaces so an
orphaned Identity can re-home on a new Node with key continuity and notify the
network. Entry per Rule 0: CLAUDE.md PLAY → JOURNAL J-275 →
`tasks/M8_5_FINALIZATION_AUDIT.md` §5 → this design.

Closing M8.5-C closes the finalization box → next is M9.

---

## 2. Grounding (D-078 — live at `d2aa24c`, not the audit's stale `cecb5ee`)

Re-confirmed against the live tree:

- **M85-A8** — `RegisterArgs` (`xgen-client/src/app.rs:459`) carries only `--name`;
  `IdentityMessage::Register` (`xgen-core/src/wire/types.rs:326`) has no
  `re_registration` field.
- **M85-A9** — `IdentityReplicateMessage` (`types.rs:617`) is only
  `Replicate`/`ReplicateAck`. `identity.home_changed` exists **nowhere** in code
  (one comment in `m8_s4_durability.rs`).
- **M85-A10 (refined)** — `home_changed`'s natural home is
  `IdentityReplicateMessage` (sibling to `replicate`), an identity-protocol
  message — **not** the Space-DAG `EventType` enum the M8 findings pointed at.
  Confirms identity-registry-level, **M8-free**, no `derive_resolved` surface.

**New grounding the audit did not carry:**

1. **The spec already fully specifies the mechanism** (spec-vs-code drift, not
   green-field): ch3 §3.6.3 field table (L1841) defines `re_registration: boolean`;
   §3.13.8 specs the 5-step orphan-recovery procedure **and** the literal
   `identity.home_changed` JSON; §3.13.9 registers the EventType; §3.13.10
   defines codes 3020–3023.
2. **Node-side hook already exists** — `accept_registration`
   (`xgen-core/src/identity/registration.rs:312`) **Step 3** rejects
   `already_registered`; the register handler (`xgen-node/src/app.rs:2717`)
   already threads the `already` bool. `re_registration:true` bypasses Step 3.
3. **Applier pattern already exists** — `handle_incoming_replicate`
   (`replication.rs:120`) does the exact version-guard the `home_changed` applier
   needs (`incoming.update_version <= stored` ⇒ `3020 VersionStale`, else upsert).
4. **Steps 1/2/4 of §3.13.8 reuse live infra** — replica fetch (`identity.replicate`),
   new-home select (the existing `--node` target), re-replicate
   (`push_identity_to_peers` + `select_replicas`). Only step 3's flag and step 5's
   broadcast are missing.
5. **`identity.update`** (`types.rs:384`) is a live generic record-mutation channel
   — considered and **rejected** for home change (see S5-D1).

---

## 3. Locked decisions (Joe-locked 2026-06-06; arc-local, D-069)

**S5-D1 — mechanism = the spec's re-registration flow (§3.13.8); not `identity.update`.**
§3.13.8 already locked re-registration (analogous to F-5 / M8.5-A: the audit's
"fork" was a phantom — already decided, just never absorbed into code).
`identity.update` is the wrong tool: it mutates a record on its home Node, but a
home change is precisely the case where the old home is gone; and a private
update does not notify federated peers, which is what `home_changed` is for.
M8.5-C catches code up to the spec. Drift flagged, not silently chosen (D-065).

**S5-D2 — keypair-ownership proof reuses the existing transport challenge.**
Registration already runs transport challenge-response *before*
`accept_registration`, and Step 1 asserts `identity_id == authenticated_id`
(the challenge-authenticated id) — so ownership is proven by the same path as a
fresh registration (§3.13.8 step 4: "the standard challenge-response"). No new
challenge. `re_registration:true` only bypasses Step 3 (`already_registered`,
which maps to existing code **3007**). The keypair-ownership backstop is the
**existing** Step 1 assertion `identity_id == authenticated_id` → **3001
`identity_mismatch`** — it fires *before* the re_registration branch, so an
attacker setting `re_registration:true` on an id they do not own is already
rejected. **3022 is therefore dormant** (spec'd, not emitted) — see §4.3
amendment (v1.1).

**S5-D3 — `home_changed` is a delta-shaped, version-guarded, signed message.**
New `IdentityReplicateMessage::HomeChanged` variant matching the §3.13.8 JSON.
Applier mirrors `handle_incoming_replicate`: verify signature against the
`identity_id` pubkey → version-guard (`update_version > stored` else `3020
identity_version_stale`) → re-point the stored record's `home_node` + bump
`update_version` → persist. Delta-shaped (not full-record): peers already hold
the record from replication; `home_changed` only re-points it. A peer holding no
prior record ⇒ **no-op + log** (it will obtain the record via the existing
replicate/refresh path). `new_home_node_url` is carried for client reachability.

**S5-D4 — build only the two missing surfaces; defer the orchestrated command.**
M8.5-C builds (a) the `re_registration` flag and (b) `home_changed` emit/apply,
wired into the existing register/replicate flow. The client emits `home_changed`
to its known peers per §3.13.8 step 5. The one-shot `recover-identity`
orchestration (fetch→select→re-register→broadcast as a single command) is UX
sugar over these surfaces → **deferred** (M8.5 is correctness-only, audit scope
fence §7). This is exactly what the blocked M8 S5 test needs: the flag settable
from `xgen-client` + `home_changed` observable in peer logs.

---

## 4. Wire surfaces

### 4.1 `re_registration` on `identity.register` (S5-D1/D2)

Add an optional flag to `IdentityMessage::Register`, omitted-when-false to keep
the canonical signing form of existing human registrations byte-identical
(mirror the `is_ai` precedent at `types.rs:333`):

```rust
#[serde(default, skip_serializing_if = "is_false")]
re_registration: bool,
```

`accept_registration` gains the flag (threaded from the variant). Step 3 becomes:
`already_registered && !re_registration` ⇒ `AlreadyRegistered`;
`already_registered && re_registration` ⇒ permit (re-home: store the record,
home_node = this Node, bump `update_version` from the prior record). Fresh
registration (`!already_registered`) is unchanged regardless of the flag.

### 4.2 `IdentityReplicateMessage::HomeChanged` (S5-D3)

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
    signature: Option<String>,   // signed by the Identity keypair
}
```

Builder + signature (reuse the identity-keypair signing path used by
`register`/`update`). Applier = new fn in `replication.rs` beside
`handle_incoming_replicate`, returning the same `Result<(), ReplicationError>`
shape (3020 on stale).

### 4.3 Error codes (§3.13.10)

> **Amendment (v1.1, 2026-06-06) — grounding correction (D-065).** The v1.0 plan
> to add a 3022 emitter collides with the live `accept_registration` Step 1,
> which already returns **3001 `identity_mismatch`** when
> `identity_id != authenticated_id`. Under the locked S5-D2 challenge model the
> keypair always owns the id by the time the re_registration branch runs, so a
> distinct 3022 emitter would be unreachable. 3022 is left **dormant** (spec'd,
> not emitted), same treatment as 3023. No new `RegistrationError` variant.

- **3001 `identity_mismatch`** — *existing*; the keypair-ownership backstop
  (Step 1), unchanged.
- **3007 `already_registered`** — *existing*; the duplicate gate (Step 3) that
  `re_registration:true` bypasses. Unchanged for `!re_registration`.
- **3020 `identity_version_stale`** — *existing* (`ReplicationError`); reused for
  the `home_changed` version-guard.
- **3022 `identity_home_node_mismatch`** — spec'd; **dormant** (3001 covers the
  mismatch under the challenge model). Documented, not emitted.
- **3023 `identity_not_found`** — spec'd; **dormant** (no replica-fetch path in
  S5-D4 scope).

---

## 5. Build split (C1 node · C2 client) — mirrors M8.5-B

**C1 (node + core):**
- `re_registration` field on `IdentityMessage::Register` (`types.rs`); `is_false`
  skip; signing form unchanged for false.
- `accept_registration` Step-3 bypass when `re_registration` (`registration.rs`);
  no new error code (3001/3007 unchanged, 3022 dormant per §4.3); thread flag
  from the handler (`app.rs:2717`).
- `IdentityReplicateMessage::HomeChanged` variant + builder + signature
  (`types.rs` + signing helper).
- `handle_incoming_home_changed` applier (`replication.rs`): verify sig →
  version-guard (3020) → re-point `home_node` + bump version → persist.
- Node receive-arm dispatch for `identity.home_changed` — a new
  `IdentityReplicateMessage::HomeChanged { .. }` arm beside `Replicate`
  (`app.rs:2823`) / `ReplicateAck` (`app.rs:2830`) under `Inbound::IdentityReplicate`
  (`app.rs:2419`), calling the applier (mirror the `handle_incoming_replicate`
  call site at `app.rs:2847`). Notification only — no ack.

**C2 (client):**
- `RegisterArgs` gains `--re-registration` (bool flag), threaded into the
  `identity.register` send path (`app.rs:2102` register flow).
- Client emit of `identity.home_changed` after a successful re-registration:
  build + sign + send to known peers.

**Doc-close (rides the same commit, D-074):** ch3 — fold the as-built shape into
§3.6.3 / §3.13.8 (mark the previously-spec'd-only surfaces as realized);
DECISIONS unchanged (S5-D# arc-local).

---

## 6. CP checkpoints (D-078 — Clair grounds these against live code before each phase)

- **CP-1** — confirm the `is_false` + `skip_serializing_if` pattern leaves the
  canonical signing bytes of a `re_registration:false` register identical to
  today's (no signature break for existing registrations).
- **CP-2** — confirm `accept_registration` Step-3 is the *only* duplicate gate;
  no second already-registered check downstream that the flag must also clear.
- **CP-3** — confirm the identity-keypair signing/verify helper used by
  `register`/`update` is reachable for `home_changed` (one signing path, no new
  crypto).
- **CP-4** — confirm the `home_node` field on the stored `IdentityRecord` is the
  single persisted home pointer the applier mutates (and that `save()` persists
  it), so the version-guard + re-point is complete.
- **CP-5** — confirm the client's "known peers" set for the step-5 broadcast
  (Space-membership home nodes / connected peers) is sourceable without a new
  discovery surface; if not, scope the broadcast to currently-connected peers and
  note it.

---

## 7. Tests (production-grounded, D-078)

- **xgen-core** — `accept_registration`: re-registration permitted with flag +
  already_registered (record re-homed, version bumped); rejected without flag
  (unchanged); 3022 on identity/auth mismatch. `handle_incoming_home_changed`:
  accepts newer version (record re-pointed); rejects stale (3020); no-prior-record
  ⇒ no-op.
- **xgen-core wire** — `HomeChanged` round-trip (rename tag, all fields); sign +
  verify round-trip.
- **client seam (mirror M8.5-B e2e)** — `--re-registration` drives the wire flag;
  a re-home against an ephemeral stub Node yields `RegisterOk`; `home_changed`
  emitted to the peer is observable.

---

## 8. Scope fence (OUT of M8.5-C)

- Orchestrated one-shot `recover-identity` command (S5-D4 — deferred UX sugar).
- 3023 replica-fetch path (dormant; not touched).
- Trust-Assertion renewal at re-home (§3.13.8 "Trust Assertion continuity" —
  Tier-1-auth-rebuild concern; T1 has empty assertions; forward-note).
- Multi-device re-home interaction (R2-F09, D3-gated).

---

## 9. Promotion eval (deferred to close)

All S5-D1..D4 are arc-local (D-069) — applications of existing conventions
(re-registration is spec'd; the version-guard mirrors `handle_incoming_replicate`;
the challenge reuse is the standard path). No promotion candidate anticipated;
re-confirm at close.

---

## 10. Next-active

**Runbook** (`tasks/M8_5_C_S5_REBIND_IMPL.md`, Joe-approved) → Clair implements
C1 then C2 → combined-commit close (resolve audit §5 M85-A8..A10; ch3 doc-close;
ROADMAP/PLAY/JOURNAL; S5-D# eval). Clair stands down until the runbook exists.

Per Rule 0 + D-065 + D-069 + D-071 + D-074 + D-077 + D-078.
