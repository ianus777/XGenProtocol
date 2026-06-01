# M7-completion cluster — implementation runbook
> **Status**: COMPLETED  
> Version: 1.8  
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
clippy `-D warnings`; baseline **964**/0/1 (A1 J-217 +6, A2 J-218 +1, A3 J-219 +2, B1 J-220 +9,
B2 J-221 +7, C1 J-222 +1, over the pre-cluster 939). **Block A CLOSED (verb set frozen: members ·
leave · create-dm-space); Block B done (B1 token + B2 idempotency); Block C done (C1).** Only the
D-074 atomic cluster close remains.

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
| **CP-1** ✅ | node-arm-only: `StateDmSpaceCreate` arm reusing the A1 constructor — **LOCKED J-219** (tip-chained invite, DMs single-homed, reject-if-absent ordering) | — | **Joe-lock** |
| A3 ✅ | `ops::create_dm_space` (client 3-event send, tip-chained invite) **+** node `StateDmSpaceCreate` arm + tests — **SHIPPED J-219** (948/0/1) | create-dm-space | — |
| — | ✅ *Block A CLOSED = verb set frozen (members · leave · create-dm-space)* | | |
| **CP-2** | token-binding seam, before B1 | — | **Joe-lock** |
| B1 ✅ | AC-D4 per-connection token (plane 1, `absent==proceed`, B-subsumable field) + tests — **SHIPPED J-220** (957/0/1; cadence=per-command, v1 inert) | token | — |
| B2 ✅ | AC-D6 idempotency key (per-`.aicontrol`-session, rides B1, B-subsumable) + tests — **SHIPPED J-221** (964/0/1; result-time binding, FIFO-bounded) | idempotency | — |
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

### CP-1 (Joe-lock) — node arm only, before A3 — **LOCKED J-219**
**The constructor `from_dm_space_create_node` is built and unit-tested at A1; CP-1 scoped the node
arm only.** The node ingest `match` (`xgen-core/src/node/runtime.rs:325`) built a `SpaceState` only
in the `StateSpaceCreate` arm; `StateDmSpaceCreate` fell to `_` and built nothing — A3 adds a
`StateDmSpaceCreate` arm that **reuses the A1 constructor** (no rebuild).

CP-1 was locked after a propagation trace (J-218/J-219). Resolution: (a) the new match arm calls
`from_dm_space_create_node` (confirmed against the live tree); (b) the auto-room applies through
`apply_room_create` (first DM Room); the auto-`membership.invite` is **rebuilt tip-chained to the
auto-room by `ops::create_dm_space`** (CP-1 (iii)) so it is **Accepted + persisted + a state no-op**
(`apply_invite` rejects under DM constraints, swallowed) — membership rides the **root**, not the
invite; (c) F-3/F-4 skip for `dm_space_create` holds (Phase 7.5). Also confirmed: **DMs are
single-homed** (no federation push, `DmFederationNotAllowed`); **room/invite are reject-if-space-absent,
not buffered** (the A3 ordering invariant, §A3); and the constructor's empty-`prev_events` invite is a
**latent bug** overridden at the call site, not fixed inside A3 (D-065). See design §M7C-D4 CP-1 trace
resolution. No new event design.

### A3 — `ops::create_dm_space` + node arm (SHIPPED J-219, CP-1 locked)
Client side: build `state.dm_space_create` (`build_dm_space_create_event(key, invitee, home_node)`),
take the constructor's creator-signed **auto-room** (correctly chained), and **rebuild the
auto-invite tip-chained to the auto-room** (CP-1 (iii) — the constructor's bundled invite has empty
`prev_events`, a latent bug overridden at the call site, D-065). **Send all three over ONE connection,
in order, root-first.** Node side: the CP-1 arm — a `StateDmSpaceCreate` match arm that **reuses the
A1-built `from_dm_space_create_node`** (no new constructor here).

**A3 ordering invariant (the correctness contract).** room/invite are **reject-if-space-absent**
(`runtime.rs:569`, step 1), NOT pending-buffered, so the chain
`dm_space_create (root) ← state.room_create ← membership.invite` MUST be sent **in order over one
connection** — `process_inbound` is sequential per event, so the root is fully ingested (builds the
SpaceState) before room/invite dispatch. Do NOT parallelize or reorder the sends.

Tests: the ordered 3-event path builds correct state — `members={creator}`,
`pending_invites={invitee}`, one Room, the tip-chained invite Accepted but a state no-op
(`dm_init_ordered_three_event_path_builds_state`, runtime.rs); the latent-constructor witness
(`from_dm_space_create_auto_invite_has_empty_prev_events_latent_bug`, state.rs). Client-side send is
component-tested per the M7 v1 boundary (no live Node).

**Known out-of-scope (D-065):** invitee-join-across-nodes discovery for a single-homed DM (federation
disabled; invitee is a `pending_invite` until they join) — A3 forms **creator-home-Node state only**;
neither builds nor breaks it. **Block A close: verb set frozen.**

## 4. Block B — hardening (M7C-D1/D2)

### CP-2 (Joe-lock) — token-binding seam, before B1
Confirm against the live `xgen-client/src/aicontrol.rs` handler: the token is a **first-message
field on the `.aicontrol` connection** (plane 1), `absent==proceed`, verified once per handler
before dispatch; the field shape is **B-subsumable** (the same field later carries a driver-bound
credential under the privilege-model arc — no wire change). Confirm it stays orthogonal to in-flight
count (pipelined-handler-arc-compatible). Surface the exact field name + placement for Joe.

### B1 — AC-D4 per-connection token — **SHIPPED J-220**
Opaque `Command.token: Option<String>` (top-level, never in `args`) + pure `check_token`
(`xgen-common/aicontrol/token.rs`) + the gate in `dispatch_one` before dispatch; `expected_token`
threaded through `start_aicontrol_server` → handler. `absent==proceed`; present-and-invalid →
`PERMISSION_DENIED`/`Category::Permission`. **Two decisions the CP-2 lock left open (J-220):**
(a) **cadence = per-command re-check** (not verify-once-cache; stateless, no `authed` flag — a
connection can't auth once then change/omit, and B's per-command credential model needs no reshape);
(b) **validity source = v1 inert** — all 3 resident spawns pass `expected_token=None` (reserved trio
inert pending the privilege model); production enforcement (config/credential) is B / the
privilege-model arc, **not** built here (no `[aicontrol]` config invented). Coupling recorded:
`Command` must never gain `#[serde(deny_unknown_fields)]`. Tests: 4 `check_token` + 3 envelope
(incl. the B-subsumability witness — opaque value round-trips unchanged) + 2 handler-gate.

### B2 — AC-D6 idempotency key — **SHIPPED J-221**
Opaque `Command.idempotency_key` (same shape rule as `token`) + NEW
`xgen-common/aicontrol/idempotency.rs::IdempotencyStore` (bounded key→`Reply` FIFO cache) held
**per connection**; `dispatch_one` checks before dispatch (replay → cached reply, no re-execution;
`absent==do-it-over`) and records after. **Two decisions surfaced (J-221):** (2) **key-binding =
result-time** — record only completed, successful (`Reply::is_ok`) ops; errored/crashed → not
recorded → replay re-does; (1) **in-flight policy = none built** — the serial handler precludes
same-connection mid-flight replay (the replay isn't read until the original returns → then deduped);
cross-connection isn't deduped (per-connection store = per-session-scope consequence; B's per-driver
scope fixes it); forward constraint: a future pipelined handler must wait-or-reject in-flight keys
(never do-it-over). **Store lifecycle (Joe's check):** per-connection local (dies on disconnect),
**FIFO-bounded** (`DEFAULT_IDEMPOTENCY_CAP=1024`) — no unbounded growth; scope lives in *placement*
(per-session now → per-driver later), wire field unchanged. Tests: 4 store + 2 envelope (incl.
B-subsumability witness) + 1 handler (5-step result-time-binding proof).

## 5. Block C — `nodes` filter `ordered_nodes` widening

### CP-3 (confirm-at-pickup) — gap shape before C1 — **CLEAN (J-222)**
Three checks against the live tree, all clean (no EV-D4 re-lock): (1) `ordered_nodes` is where/what
C3 said — `state.node_priority` `content["ordered_nodes"]`, excluded at the single assembly site
`derive_event_nodes` (`fanout.rs:164`); (2) C1 is a `derive_event_nodes` widening, NOT a `matches`
change (the EV-D4 v1.1 3-param caller-supplied form is unchanged); (3) node/client asymmetry holds
(client rejects non-empty `nodes`→`BAD_ARGUMENT`, passes `&[]`; widening is node-side only).

### C1 — widening — **SHIPPED J-222**
`derive_event_nodes` gains a fifth source: fold `content["ordered_nodes"]` URIs into the returned set
(presence-based, mirroring `content["node_id"]`). `matches` + the client path untouched. Tests:
`derive_event_nodes_includes_ordered_nodes_source_5` (derive includes both refs; `matches` membership
match + non-match); regression via the existing four-sources test + the xgen-common `nodes` tests.

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
