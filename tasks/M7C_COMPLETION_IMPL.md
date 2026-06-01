# M7-completion cluster — implementation runbook
> **Status**: ACTIVE  
> Version: 1.3  
> Date: Jun 2026  
> **Last updated**: 2026-06-01  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. Framing

Clair-facing build plan for the M7-completion cluster under the M7C-D1–D4 locks
(`tasks/M7C_COMPLETION_DESIGN.md`). Closes the `--aicontrol`-shaped remainder of M7.

**Reading order (Rule 0):** CLAUDE PLAY → JOURNAL J-215 → this runbook §1–§2 → design doc
`tasks/M7C_COMPLETION_DESIGN.md` §1–§5 → audit `tasks/M7C_COMPLETION_AUDIT.md` (COMPLETED, the
grounding) → back here per-commit.

**Discipline.** D-065 (honest over polite — one verb deliberately exceeds pure adapter; say so).
D-066 (`--batch` and `pipe.rs` untouched). D-067 (route every new client verb through one
`ops::*` function; the CLI / batch / `.aicontrol` dispatchers stay thin shims). Explicit
`git add <file>` per file; Joe pushes. Each commit: `cargo test --workspace` + build all-targets +
clippy `-D warnings`; baseline **946**/0/1 (A1 J-217 +6, A2 J-218 +1, over the pre-cluster 939).

**The adapter caveat (D-065).** `leave` is pure adapter (new `ops::*` over existing backing).
`members` is a lift **plus** the shared key-less DM-seed constructor
`SpaceState::from_dm_space_create_node` — A1 needs it to replay a DM Space (see §3 A1).
`create-dm-space` *reuses* that same constructor in a node-side `StateDmSpaceCreate` ingest arm
(M7C-D4). The non-adapter element is the **shared constructor** (first needed at A1), not a single
verb: it is primitive-completion (the event/builder/applier/DM-state shape all exist) but genuinely
node-side protocol work — do not pretend it's surface-only.

## 2. Sequence overview

| Commit | Scope | Verb / unit | Checkpoint |
|---|---|---|---|
| A1 ✅ | `ops::members` (read/lift) **+ builds shared `from_dm_space_create_node`** + dispatchers + tests — **SHIPPED J-217** (945/0/1) | members | — |
| A2 ✅ | `ops::leave` (write, mirrors `join`) + dispatchers + tests — **SHIPPED J-218** (946/0/1) | leave | — |
| **CP-1** | node-arm-only: `StateDmSpaceCreate` match arm reusing the A1 constructor, before A3 | — | **Joe-lock** |
| A3 | `ops::create_dm_space` (client 3-event send) **+** node `StateDmSpaceCreate` arm reusing the A1 constructor + tests | create-dm-space | — |
| — | *Block A close = verb set frozen* | | |
| **CP-2** | token-binding seam, before B1 | — | **Joe-lock** |
| B1 | AC-D4 per-connection token (plane 1, `absent==proceed`, B-subsumable field) + tests | token | — |
| B2 | AC-D6 idempotency key (per-`.aicontrol`-session, rides B1, B-subsumable) + tests | idempotency | — |
| **CP-3** | `nodes`/`ordered_nodes` gap shape, before C1 (light) | — | confirm-at-pickup |
| C1 | `nodes` filter `ordered_nodes` widening (EV-D4 dimension) + tests | nodes filter | — |
| Close | D-074 atomic: canonical doc + audit/design/runbook COMPLETED + ROADMAP + PLAY + JOURNAL | — | — |

**What this cluster CANNOT close** (named homes, do not pull in): per-driver-identity control plane
(M7C-D1/D2 end-state B) → privilege-model arc · plugin-write verbs → temperature-plugin arc ·
`CONCURRENT_COMMAND_NOT_ALLOWED`/pipelined handler → own arc · `migrate-start` → migration
subsystem · live config reload → M7-standalone. STOP and surface if any commit drifts toward these.

## 3. Block A — client-feature (AC-D5)

Each verb routes through `ops::*` and is reached by all dispatchers (D-067). Per-verb touch set
(confirm-at-pickup, D-078, against the live tree): `xgen-client/src/ops.rs` (new `pub (async) fn`
+ Result struct) · `app.rs` (clap subcommand + `cmd_*` CLI shim) · `batch.rs` (batch arm) ·
`aicontrol.rs` (`dispatch_resolved` arm building its own session, per the M7 v1 pattern). M7 v1's
reconstruct-argv path means the `.aicontrol` arm is mechanical once clap knows the verb.

### A1 — `ops::members` (lift + the shared DM-seed constructor)
Read verb. Replay the Space into a `SpaceState` (reuse the `ai_status` history-drain pattern,
`ops.rs:1031`) and return `state.members` (id → role, `invited_by`, `joined_at`). **Covers DM Spaces
— and that is what forces a constructor here:** the drain seeds a regular Space from
`from_space_create`, but the only DM seed, `from_dm_space_create` (`state.rs:226`), takes the
**creator's key** and a non-creator read-side replay has none. So A1 **also builds the shared
key-less `SpaceState::from_dm_space_create_node`** (LOCKED J-215 — sibling of `from_space_create`;
owner = `event.sender`, invitee from `content["invitee"]`, DM constraints) and seeds the DM branch
with it. **One seed, two callers (D-067):** A1's replay here, A3's node ingest arm. Do **not**
inherit `ai_status`'s DM bail — that is operator-resolution-specific, not a membership-read limit.
Tests: regular-Space membership read; DM-Space membership read (exercises the constructor);
empty/unknown Space error.

### A2 — `ops::leave` (adapter, mirrors `join`)
Write verb. Build `membership.leave` via `build_membership_event` (`state.rs:1221`) → sign → send →
await accept. Mirrors `ops::join` end-to-end. The node accepts it on signature + step-11
sender-membership (no special role; `validate_steps_8_13` special-cases only invite/kick/ban).
Tests: member leaves (accepted, removed from space + rooms); non-member leave rejected; round-trip.
Carry-to-verify (not a blocker): confirm fan-out/federation propagates the leave (generic path).

### CP-1 (Joe-lock) — node arm only, before A3
**The constructor `from_dm_space_create_node` is built and unit-tested at A1; CP-1 no longer covers
it.** CP-1 now scopes the **node arm only**. The node ingest `match`
(`xgen-core/src/node/runtime.rs:325`) builds a `SpaceState` only in the `StateSpaceCreate` arm;
`StateDmSpaceCreate` falls to `_` and builds nothing. A3 adds a `StateDmSpaceCreate` arm that
**reuses the A1 constructor** (no rebuild). Surface for Joe: (a) the new match arm calls
`from_dm_space_create_node` (now a concrete, tested constructor — confirm the call shape against the
live tree, not constructor-vs-inline); (b) confirm the client-sent auto-room (`state.room_create`)
applies through `apply_room_create` (first DM Room) with the DM SpaceState present. **J-217
refinement (D-065):** the auto-`membership.invite` does **NOT** apply — `apply_invite` rejects
invites once `dm_constraints_active` (`state.rs:619`, 3.16.1); the invitee's pending invite is
seeded by `from_dm_space_create_node` at construction (as `from_dm_space_create` always did), so the
auto-invite is a no-op/reject on ingest **by design**. Confirm the node arm tolerates that reject
(does not fail the whole ingest on it); (c) the F-3/F-4 skip for `dm_space_create` already holds
(Phase 7.5) — confirm it still does. No new event design.

### A3 — `ops::create_dm_space` + node arm
Client side: build `state.dm_space_create` (`build_dm_space_create_event(key, invitee, home_node)`),
run `from_dm_space_create` locally for the creator-signed auto-room + auto-invite, **send all three**
(M7C-D4 client 3-event send). Node side: the CP-1 arm — a `StateDmSpaceCreate` match arm that
**reuses the A1-built `from_dm_space_create_node`** (no new constructor here). Tests: client produces
3 well-formed events; node ingest builds the DM SpaceState + applies room + invite; member set
correct; the **first-production-exerciser** path end-to-end (component-level acceptable per the M7 v1
test boundary; flag any live-Node-only gap). **Block A close: verb set frozen.**

## 4. Block B — hardening (M7C-D1/D2)

### CP-2 (Joe-lock) — token-binding seam, before B1
Confirm against the live `xgen-client/src/aicontrol.rs` handler: the token is a **first-message
field on the `.aicontrol` connection** (plane 1), `absent==proceed`, verified once per handler
before dispatch; the field shape is **B-subsumable** (the same field later carries a driver-bound
credential under the privilege-model arc — no wire change). Confirm it stays orthogonal to in-flight
count (pipelined-handler-arc-compatible). Surface the exact field name + placement for Joe.

### B1 — AC-D4 per-connection token
Add the token to the connection-open path; `absent==proceed`; on present-and-invalid →
`PERMISSION_DENIED` (AC-D4's reserved code, now activated for this surface only). Per-connection
state on the handler. Tests: absent→proceed; valid→proceed; invalid→`PERMISSION_DENIED`;
B-subsumability witness (field round-trips an opaque value unchanged).

### B2 — AC-D6 idempotency key
Optional first-class field; per-`.aicontrol`-session dedupe riding B1's per-connection state;
"session" defined to widen to driver-identity later with no wire change (M7C-D2). Replayed key →
the prior result, no re-execution. Tests: dedupe within a session; distinct keys independent;
absent→do-it-over (AC-D6 default).

## 5. Block C — `nodes` filter `ordered_nodes` widening

### CP-3 (confirm-at-pickup) — gap shape before C1
Re-read the live `xgen-common/src/aicontrol/filter.rs` + `matches` (EV-D4 v1.1, 3-param,
`event_nodes` caller-supplied) and the C3-documented `ordered_nodes` gap; confirm the widening shape
before coding (its own light pre-check — no stale premise surfaced for it yet, but it was not
deeply traced).

### C1 — widening
Extend the `nodes` dimension to cover `content["ordered_nodes"]` (the `node_priority` reference set
that `derive_event_nodes` excluded). Tests: `ordered_nodes` membership matches; non-match; existing
`nodes` behaviour unchanged (regression).

## 6. Close (D-074 atomic)

Canonical doc `docs/xgen_aicontrol_implementation.md`: add the 3 client verbs (§7 surface), the
AC-D4 token + AC-D6 idempotency as-built, the `nodes`/`ordered_nodes` widening; note the shared
`from_dm_space_create_node` constructor (first needed at A1, reused by A3's node arm) as the one
non-adapter delta (D-065). `tasks/M7C_COMPLETION_AUDIT.md`
already COMPLETED; flip `_DESIGN.md` + this runbook → COMPLETED. ROADMAP M7-completion ✅ + version;
CLAUDE PLAY → next milestone; JOURNAL close entry (same commit). Joe's call whether any M7C-D#
promotes to a global D-### (default: arc-local, D-069). Verification: full suite + build + clippy.

## 7. Joe-lock checkpoints (consolidated)

- **CP-1** before A3 — **node arm only** (the `StateDmSpaceCreate` match arm reusing the A1-built `from_dm_space_create_node`; confirm call shape + applier + F-3/F-4 skip). The constructor itself is built + tested at A1.
- **CP-2** before B1 — token-binding seam (plane-1 field, `absent==proceed`, B-subsumable, name+placement).
- **CP-3** before C1 — `ordered_nodes` widening shape (light confirm-at-pickup).

## 8. Discipline notes

- Adapter discipline (D-065): `leave` pure adapter; `members` = lift + the shared
  `from_dm_space_create_node` constructor (A1); `create-dm-space` = adapter + the CP-1 node arm that
  reuses that constructor. The shared constructor (first needed at A1) is the one non-adapter
  element, recorded honestly. No verb introduces a new EventType (all exist).
- `--batch`/`pipe.rs` untouched (D-066); the `.aicontrol` arms are sisters, not forks.
- B-subsumability (M7C-D1/D2): B1/B2 field shapes must accept the future driver-bound credential
  without a wire/format change — the test witnesses are the guardrail.
- Block A close is the gate: Block B designs token/idempotency state against the **frozen** verb set.

## 9. Cross-references

- Design: `tasks/M7C_COMPLETION_DESIGN.md` (M7C-D1–D4). Audit: `tasks/M7C_COMPLETION_AUDIT.md`.
- Code: `xgen-client/src/{ops.rs,app.rs,batch.rs,aicontrol.rs}`,
  `xgen-core/src/space/state.rs` (`from_dm_space_create:226`, `build_dm_space_create_event:1098`,
  `apply_leave:604`, `build_membership_event:1221`), `xgen-core/src/node/runtime.rs` (`:325`),
  `xgen-core/src/message/exchange.rs` (`validate_steps_8_13`),
  `xgen-common/src/aicontrol/filter.rs` (`matches`, EV-D4 v1.1).
- Inherited locks: AC-D4/AC-D5/AC-D6 + EV-D4 in `docs/xgen_aicontrol_implementation.md` v1.4.
- Discipline: D-065 · D-066 · D-067 · D-069 · D-074 · D-078.
