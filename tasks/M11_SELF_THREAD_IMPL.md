# M11 — `self` Thread: Implementation Runbook (D-021)
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-14  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. Purpose & gate

The Clair-facing build plan for M11, executing the Joe-LOCKED design
(`tasks/M11_SELF_THREAD_DESIGN.md` v1.0, M11-D1..D5) on the Phase-0 grounding
(`tasks/M11_SELF_THREAD_PHASE0_AUDIT.md` v1.0). **No code until Joe locks this runbook.**

The shipped feature: a **self-DM** (shape B) — `state.dm_space_create` with
`invitee == creator` — that gives the user a Node-side, never-federated personal thread
reusing their existing keypair. The entire protocol/applier delta is **D1: a two-site
constructor guard** (a few lines). Everything else is a thin `xgen-client` convenience (D5),
a wording lock (D2), and a ch6 close note (D4).

**Discipline (standing):** Clair commits **code + tests only** (commits 1–3); the ch6 note
(D4) + the canonical-record flips (CLAUDE PLAY / JOURNAL / ROADMAP / DECISIONS eval) are the
**doc-bridge close** (one-writer, D-074 atomic). Code commits ship first; Joe pushes — hand him
the PowerShell block, never push. Per-commit gate: `cargo build --workspace --all-targets` 0 ·
`cargo clippy --workspace --lib --tests --all-features -- -D warnings` clean · `cargo test
--workspace` green (record the count). No wire/event/reject/ch3 edit anywhere — there are none.

---

## 1. Grounding confirmed at pickup (file:line, current `main`)

These were re-verified against the tree before this runbook; the runbook commits cite them.

**D1 guard — the two sites (the entire delta):**
- `from_dm_space_create` (`xgen-core/src/space/state.rs:342`): `creator = event.sender.clone()`
  (`:374`); `invitee` parsed (`:361-366`); the seeding to guard is
  `pending_invites.insert(invitee, PendingInvite{ Member, invited_by: Some(creator), … })`
  (**`:419-422`**). Both are `IdentityXgid`.
- `from_dm_space_create_node` (`xgen-core/src/space/state.rs:487`): `creator = event.sender.clone()`
  (`:516`); `invitee` parsed (`:503-508`); the seeding to guard is
  `pending_invites.insert(invitee, …)` (**`:530-533`**).
- Narrowness guard (must stay GREEN): `from_dm_space_create_node_seeds_owner_and_pending_invite_keyless`
  (`state.rs:2821`) asserts a **non-self** DM still seeds `pending_invites[invitee]`.
- Test harness available in `state.rs` `mod tests` (`:1944`): `alice_key()`, `bob_key()`,
  `sender_id(&key)`, `sign_event`, `build_dm_space_create_event(&key,&invitee_id, HOME)`,
  `event_id_str`, `xid`. A self-DM fixture = `build_dm_space_create_event(&alice,
  &sender_id(&alice), HOME)`.

**D1 residue (surfaced, not suppressed — see §6 C3):** `from_dm_space_create` *also* builds the
`invite_event` (`:394-403`) and **returns** it (`:462`). `ops::create_dm_space` discards that
returned invite (`ops.rs:711`, `_constructor_invite`) and **rebuilds + sends** its own
self-targeted `membership.invite` as chain event #3 (`ops.rs:721-732`). The node swallows it
(`apply_invite` → `DmInvitationNotAllowed` at `state.rs:946-948`, *before* it inspects the target;
accept-either at `ops.rs:786-787`) **without seeding `pending_invites`** (it errors before any
mutation). So D1 (constructor-only) removes the **state** artifact on both client and node; the
still-sent wire self-invite is inert and matches every DM (audit E-1). Suppressing it would touch
`ops`, beyond D1's locked two-site scope.

**D5 verb surface — `create-dm-space` is a real 4-arm verb; `self` mirrors it (D-092):**
- enum + clap doc: `ClientCommand::CreateDmSpace(CreateDmSpaceArgs)` (`app.rs:351`); `CreateDmSpaceArgs`
  (`app.rs:540`).
- Arm 1 **CLI**: `main.rs:225` → `app::cmd_create_dm_space` (`app.rs:2309`).
- Arm 2 **run-path**: `app.rs:959` → `cmd_create_dm_space`.
- Arm 3 **batch**: `batch.rs:423` → `ops::create_dm_space` directly.
- Arm 4 **aicontrol**: `aicontrol.rs:428` → `ops::create_dm_space`; plus the write-command list
  (`aicontrol.rs:155`) and the result-id map (`aicontrol.rs:164`, `"create-dm-space" => "space_id"`).
- Core: `ops::create_dm_space` (`ops.rs:656`) — resolves the signing key + `identity_id` from the
  session (`ops.rs:670-677`), sends the 3-event chain, writes a `KnownSpace` labelled
  `"DM with <invitee>"` (`ops.rs:808-818`). **`KnownSpace` carries `name` only — no `is_dm`,
  `invitee`, or `self` marker** (`xgen-common::state::KnownSpace`). This is the create-if-absent
  detection gap (§6 C1).

**D2/reach (no code):** `collect_sync_history` member-gates per Space (`xgen-node/src/fanout.rs:457`),
dispatched at `app.rs:1667` — the self-DM (user = Owner) rides it. Reach = "any client
**authenticated as the user**," Node-resident not device-local.

**D4 (close):** `docs/xgen_ch6_client_design.md` — no `self` section today.

---

## 2. Joe-lock checkpoints (before code)

Surface these to Joe and get them locked before Commit 1 — they are the runbook's grounded calls
the design delegated ("the runbook grounds the exact surface and arms"):

- **C1 (load-bearing) — create-if-absent detection.** Recommend **label-based**: write
  `KnownSpace.name = "self"` at create; create-if-absent scans `state.spaces` for an owned
  KnownSpace with that name. (Alternative — a marker field on `KnownSpace` — is a wider
  `xgen-common` schema touch, outside "thin client.")
- **C2 — create-core reuse (D-067).** Recommend the self verb share `create_dm_space`'s chain core
  with the **label parameterized** (one core, no drift), rather than relabel-after.
- **C3 — leave the benign wire self-invite (§1 residue).** Confirm Joe is content that D1
  (constructors-only = the entire delta) leaves `ops` still sending a swallowed self-targeted
  invite; suppressing it is out of D1 scope.
- **C4 — label string.** `name="self"` as the stable detection/storage key; **"Saved Messages"** as
  the human display in help text + ch6 (D3 names both).
- **C5 — W4 home / commit placement** (its own witness commit vs folded into Commit 1).
- **C6 — variant name** `SelfThread` + clap command `"self"` (`Self` is a reserved word; the command
  string is fine).

None contradict the locked shape. **One grounding resolves an open design question:** D5's "may not
need all four arms" → **it needs all four** (a `self` verb is a real new `ClientCommand`,
sibling-shape to `CreateDmSpace`); each arm is a thin shim.

---

## 3. Commit 1 — D1 guard + W1/W2/W3 (xgen-core, the protocol/applier delta)

**Change (two sites, a few lines):** wrap each `pending_invites.insert(...)` in a self-skip guard.
- `from_dm_space_create` (`state.rs:419-422`): only seed when `invitee != creator`. `invitee` is
  moved into the insert — compare before the move (e.g. `if invitee != creator { pending_invites.insert(invitee, …) }`,
  `IdentityXgid: PartialEq`). Keep `pending_invites` the (now possibly empty) map.
- `from_dm_space_create_node` (`state.rs:530-533`): same guard, `invitee != creator`.
- **Constructor-only.** Do **not** add an `apply_join` belt-and-suspenders check (it already
  short-circuits `AlreadyMember`, `state.rs:1000-1001`/`:992-993` — redundant; widens the touch).
- Leave the `invite_event` build/return (`:394-403`/`:462`) untouched (§1 residue, C3).

**Witnesses (this commit) — all unit-level in `state.rs` `mod tests`:**
- **W1a** `from_dm_space_create_self_dm_seeds_no_pending_invite` — self fixture; assert
  `!state.pending_invites.contains_key(self_id)` (and `state.pending_invites.is_empty()`).
  **RED-on-revert:** drop the guard → the vestigial entry returns.
- **W1b** `from_dm_space_create_node_self_dm_seeds_no_pending_invite` — same assertion for the
  node constructor. RED-on-revert genuine.
- **W2** `self_dm_creator_is_owner_and_dm_room_member` — self fixture via `from_dm_space_create`;
  assert `members[self].role == Owner` (`:382`) and the auto-room (`state.rooms`) contains `self`
  (`:415`). Functional "it's a usable thread" at the state layer. (The post/read admission half —
  `validate_event` step-11 — is exercised by W4's node test in §5, where the registry is live.)
- **W3** `self_dm_never_federates` — self fixture; `apply_federation_add` on the self-DM →
  `Err(DmFederationNotAllowed)` (`state.rs:660-661`; `dm_constraints_active: true` at `:454`).
  Pairs with the degenerate `{this_node}` party set (audit §5.1).
- **Narrowness regression (cite, keep GREEN):** the existing `..._seeds_owner_and_pending_invite_keyless`
  (`:2821`) — proves the guard is narrow (normal DMs still seed `pending_invites[invitee]`).

**Gate:** build 0 · clippy clean · `cargo test --workspace` green; record the count. Code-only commit.

---

## 4. Commit 2 — D5 `self` verb + label + D2 wording (xgen-client)

**New verb (4 D-092 arms, sibling-shape to `CreateDmSpace`):**
- `ClientCommand::SelfThread(SelfThreadArgs)` in `app.rs` (variant `SelfThread`, clap command
  `"self"`; C6). `SelfThreadArgs` = no positional id (D5 — auto-resolve session identity); inherit
  `--node` / `--quiet` as the siblings do. The clap doc-comment carries the **D2 reach wording**
  ("reachable from any client authenticated as you — your own devices; Node-resident, not
  device-local"). Help/display label **"Saved Messages"** (C4).
- `ops::self_open(ctx) -> SelfThreadResult` (`ops.rs`, beside `create_dm_space`):
  1. resolve `identity_id` from `ctx.session.identity` (the `ops.rs:670-677` pattern) → invitee = self.
  2. **create-if-absent** (C1): scan `load_or_default_state(...).spaces` for an owned KnownSpace
     named `"self"`; if found, return its ids (the "open" payload) without a network round-trip.
  3. if absent: run the create-dm-space chain with `invitee = self` and the **"self"** label (C2 —
     parameterize the label on the shared core; do **not** fork the chain logic). The chain is the
     proven path (audit §4 / MP-C-07); the self-targeted auto-invite swallows as always.
  4. return `SelfThreadResult { space_id, room_id, created: bool }`.
- **Four arms**, each a thin shim to `ops::self_open` (no logic): CLI (`main.rs`, beside `:225`) →
  a `cmd_self_thread` shim; run-path (`app.rs`, beside `:959`); batch (`batch.rs`, beside `:423`);
  aicontrol (`aicontrol.rs`, beside `:428`) **plus** add `"self"` to the write-command list
  (`:155`) and `"self" => "space_id"` to the result-id map (`:164`).
- **Label (D3):** the create path writes `KnownSpace.name = "self"` (the detection key, C1/C4);
  posting/reading reuse existing `Send`/`History` against the returned room (no new verbs).

**Witnesses (this commit):**
- **V-idempotent** `self_open_is_create_if_absent` — call `ops::self_open` twice against the same
  client state; second call returns the same `space_id`/`room_id`, `created: false`, and
  `state.spaces` gains **no** duplicate self KnownSpace. (Model the first call's effect with a stub
  WS like `send_confirm_integration.rs:179`, or seed the KnownSpace and assert the second call
  short-circuits without a chain.)
- **V-autotarget** the verb writes `invitee = session identity` (no typed id) and labels `"self"`.

**Gate:** build 0 · clippy clean · `cargo test --workspace` green; record the count. Code-only commit.

---

## 5. Commit 3 — W4 reach witness (xgen-node, test-only)

Proves **D2's reach** independent of the D5 verb: the self-DM is reachable by any client
authenticated as the user, via member-gated sync.

- A node integration test (home: `xgen-node/src/tests/`, sibling to the existing DM/sync tests;
  confirm the cleanest module at pickup, C5): stand up a `NodeRuntime`, register the user, ingest
  the self-DM create chain signed by the user (root `state.dm_space_create` invitee=self → auto-room
  → swallowed self-invite), then issue a `sync_request` **as the same identity** (`collect_sync_history`,
  `fanout.rs:447`) and assert the response contains the self-DM Space + its events. A "second client"
  is modeled as a second same-identity sync.
- This test also lands **W2's admission half** if convenient: post a `message.text` from the user to
  the dm Room and assert it is admitted (step-11 registered + Space member + Room member + sig,
  `exchange.rs:489/629/672/676/685`).
- **Honest framing (D-065):** W4 is a *positive reach* witness — its RED-on-revert is weak (not tied
  to the D1 guard, unlike W1). It documents that the shipped self-DM is reachable, the load-bearing
  D2 property. No production change — test-only.

**Gate:** build 0 · clippy clean · `cargo test --workspace` green; record the count. Code-only commit.

*(C5: if the team prefers core+node together, fold W4 into Commit 1 — it has no production code. Kept
separate here so Commit 1 stays a pure xgen-core delta.)*

---

## 6. Close — D4 ch6 note + canonical bridge (doc-bridge seat, atomic D-074)

After commits 1–3 land, the **doc-bridge close** (not a Clair code commit):
- **D4 — ch6 note** in `docs/xgen_ch6_client_design.md` (a new short subsection; **not** a ch3
  normative edit). Content (design §M11-D4): what it is (personal single-user thread; messages +
  chronological history) · **reuses the user's existing identity** (the anchor line that prevents
  drift to "a separate account") · never-federated / never-broadcast **by reference** to
  `DmFederationNotAllowed` · attachments as an **inherited** M12 capability (present-tense concept,
  forward-referenced mechanism) · the boundary (not an account, not a Node-side service, no new
  protocol surface) · the **D2 reach wording** verbatim ("any client authenticated as the user —
  their own devices; Node-resident, not device-local"). Authored at close so it reflects the shipped
  shape.
- **Canonical flips:** this design + audit → COMPLETED; `tasks/M11_SELF_THREAD_IMPL.md` →
  COMPLETED; CLAUDE PLAY head; JOURNAL (next J-); `docs/ROADMAP.md` (M11 design-lock → DONE).
  **DECISIONS:** M11-D1..D5 are arc-local (D-069) — eval at close, promote none unless Joe calls it;
  D-021 reconciled (registered-via-existing-identity / never-federated spirit preserved).

---

## 7. Witness ↔ decision ↔ commit matrix

| Witness | Proves | Commit | RED-on-revert |
|---|---|---|---|
| W1a / W1b | D1 — no `pending_invites[self]` (both constructors) | 1 | **Yes** (guard removed → entry returns) |
| W2 | self-DM creator is Owner + dm-Room member (+ admission via W4) | 1 (+5) | functional |
| W3 | D1/privacy — never federates (`DmFederationNotAllowed`) | 1 | structural |
| W4 | D2 — reachable by any client authenticated as the user | 3 | positive reach (weak) |
| V-idempotent / V-autotarget | D5 — create-if-absent → open; auto-target, "self" label | 2 | functional |

---

## 8. Out of scope (named — do not build)

- Attachments → M12 (the ch6 note forward-references; text-first here).
- Operator-confidentiality / E2E → moot for B (audit §7).
- Renaming the internal `DM` primitive for the one-party case → named, not fixed (D-069).
- Suppressing the swallowed wire self-invite → out of D1's two-site scope (C3).
- Any new wire type, event kind, reject code, or ch3 normative edit → there are none.

---

## 9. Entry (Rule 0)

CLAUDE.md PLAY → JOURNAL J-377 → this runbook → `tasks/M11_SELF_THREAD_DESIGN.md` →
`tasks/M11_SELF_THREAD_PHASE0_AUDIT.md`.

*No code until Joe locks this runbook (§2 checkpoints first). Clair commits code-only (1–3); the
doc-bridge assembles the close. Joe pushes.*
