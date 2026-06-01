# M7-completion cluster — design (M7C-D# locks)
> **Status**: ACTIVE  
> Version: 1.1  
> Date: Jun 2026  
> **Last updated**: 2026-06-01  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. What this is

Design phase for the M7-completion cluster, opened by `tasks/M7C_COMPLETION_AUDIT.md` (J-213).
Locks the M7C-D# decisions (arc-local per D-069) that the runbook + Block A/B/C build against.
The audit's three hypotheses carry in: **H1 (`members`) STALE**, **H2 (`leave`) STALE/inverted**,
**H3 (AC-D4/D6 connection model) NEEDS-JOE-LOCK — the real catch**. Block-A verb verifies run
during this phase surfaced a **second catch** in `create-dm-space` (node DM-ingest gap). Doc-only.

## 1. Locks

### M7C-D1 — control-plane token binds per-connection (plane 1)

The events arc gave "connection/session" three referents: the client `.aicontrol` handler
(per-handler `Bindings`), the client `.events` process-wide `active_session_count()`, and the
node-side `ConnId` / `Vec<(ConnId,Sender)>`. **The AC-D4 token binds to plane 1** — the
`.aicontrol` pipe handler. It authenticates the AI driver to the resident's command pipe as a
first-message field, `absent==proceed` (AC-D4's reserved seam). Planes 2/3 are wrong: plane 2 is a
counter, plane 3 is node↔client WS fan-out, neither is the local command-pipe IPC the token guards.
**Forward-compatible by construction:** the token authenticates the connection, orthogonal to
in-flight command count, so the deferred pipelined-handler arc needs no re-litigation.

### M7C-D2 — idempotency key per-`.aicontrol`-session

AC-D6's idempotency key is scoped to the `.aicontrol` handler session (the serial read→dispatch→
reply loop with its own `Bindings`). **"Session" is defined to *widen* to driver-identity later**
without changing the key's wire shape (optional first-class field, session-scoped semantics).

### M7C-D1/D2 are written B-subsumable (the deferred end-state)

The better end-state is **B — per-driver-identity**: token + idempotency + audit attribution keyed
to a *named driver principal* spanning multiple connections and surviving reconnects (the events
arc already made one-identity-many-connections real on the node side). B is the right answer for
multi-connection drivers, reconnect-resilient idempotency, and per-driver privilege. Its cost is a
control-plane **privilege model** that does not exist (AC-D4's reserved trio — `authorize` stage +
`PERMISSION_DENIED` + token — is inert pending exactly this). So B is **deferred to a future
privilege-model / control-plane-identity arc** (named home in §6), and A is written as a strict
subset: the D1 token field and D2 key field are shaped so the same fields carry a driver-bound
credential when B lands — **zero wire/format rework**. Same "smallest correct floor now, named rung
above" posture AC-D4 already took toward the pipelined-handler arc.

### M7C-D3 — Block-A verb set

`create-dm-space` · `members` · `leave`. Grounded against the live tree this phase:
- **`members` — lift + the shared DM-seed constructor.** `ops::ai_status` (`ops.rs:1031`) already
  replays history into a `SpaceState` and reads `state.members`. For a **regular** Space the drain
  seeds from `from_space_create:189`. For a **DM** Space the only seed is `from_dm_space_create:263`,
  which takes the **creator's key** — and a non-creator client replaying for a read has none. So
  `members` needs a **key-less** DM seed: `SpaceState::from_dm_space_create_node` (M7C-D4, LOCKED
  J-215). This surfaced at A1 (Clair's catch): the original grounding confirmed a DM `SpaceState`
  *has* members, not that a non-creator could *seed* one — so the constructor is first needed at
  **A1**, not A3. The `ai_status` DM bail is operator-resolution-specific, not a membership-read
  limit. New `ops::members` reads `state.members`.
- **`leave` — pure adapter.** `membership.leave` is fully backed: builder `build_membership_event`
  (`state.rs:1221`), applier `apply_leave` (`state.rs:604`, wired at `:356`). The node accepts a
  member-initiated leave with no semantic reject — `validate_steps_8_13` special-cases only
  invite/kick/ban at step 13; leave falls to default and is gated on step-11 sender-membership +
  signature. New `ops::leave` = build → sign → send, mirroring `join`.
- **`create-dm-space` — adapter over the shared seed.** It reuses the A1-built constructor in a node
  ingest arm; the non-adapter element is that **shared constructor** (first needed at A1), not this
  verb. See M7C-D4.

### M7C-D4 — shared key-less DM-seed constructor + DM-create flow (client 3-event send + node arm)

`from_dm_space_create` (`state.rs:226`) takes the **creator's key** and returns
`(SpaceState, auto_room_event, invite_event)` pre-signed with it. The node never holds that key, so:
- **Client side (the verb):** `ops::create_dm_space` builds `state.dm_space_create`
  (`build_dm_space_create_event(key, invitee, home_node)`), runs `from_dm_space_create` locally to
  get the creator-signed auto-room + auto-invite, and **sends all three events** to the node. Thicker
  than `create_space` (3 events vs 1) but still adapter — no new event design.
- **The shared seed (the non-adapter element).** Both the A1 `members` replay and the A3 node
  ingest must build a DM `SpaceState` **without the creator's key**. The fix is **one** key-less
  constructor `SpaceState::from_dm_space_create_node` (sibling of `from_space_create`; owner =
  `event.sender`, invitee from `content["invitee"]`, DM constraints; key-less because both callers
  verify signatures but do not author). **One seed, two callers (D-067):** A1's client-side replay
  (first need) and A3's node-side ingest arm. So the non-adapter element is the **shared
  constructor**, not the `create-dm-space` verb — and it lands at A1, not A3.
- **Node side (the arm that reuses it).** The node's ingest `match` (`runtime.rs:325`) creates a
  `SpaceState` only in the `StateSpaceCreate` arm (`from_space_create`); `StateDmSpaceCreate` falls
  to the `_` arm and silently builds nothing (it needs an already-existing space). **A3 adds a
  `StateDmSpaceCreate` arm that calls the A1-built `from_dm_space_create_node`** (no rebuild), after
  which the separately-arriving auto-room (`state.room_create`) and auto-invite
  (`membership.invite`) apply through the normal appliers. This is **primitive-completion, not new
  event-design** — the event, builder, applier, and DM `SpaceState` shape all already exist; the
  shared constructor + the create-on-ingest arm are the only missing pieces. Recorded honestly as
  the one Block-A element that exceeds pure adapter (D-065).

### M7C-D4 CP-1 trace resolution (J-219, as-built — the doc follows the behavior)

CP-1's propagation trace corrected three points in the sketch above ("apply through the normal
appliers" was imprecise for the invite). The behavior, traced + tested:

- **(a) Root carries membership; the auto-invite is a no-op-by-reject but persists + fans out
  locally.** `from_dm_space_create_node` seeds `members={creator}` + `pending_invites={invitee}` from
  the root's `content["invitee"]`, so membership rides the **root**, not the invite. The auto-invite
  is sent (A3 (iii)) tip-chained to the auto-room, so it is **Accepted + persisted + fanned out** as a
  well-formed DAG record; `apply_invite` rejects it under DM constraints (3.16.1) and the error is
  swallowed in `ingest_event` — a **state no-op**.
- **(b) DMs are single-homed — no federation push.** `apply_federation_add` rejects with
  `DmFederationNotAllowed` (`state.rs:495`), so a DM's `federation_nodes` is always empty and
  `apply_federation_push` sends to nobody. The invitee participates by connecting to the DM's home
  Node; there is no "invitee's home node" forming separate DM state via federation.
- **(c) room/invite are reject-if-space-absent, NOT pending-buffered.** A non-create event targeting
  an unbuilt Space is hard-`Rejected` at `dispatch_event` step 1 (`runtime.rs:569`), before every
  buffering path. So correct ingest is **ordering-dependent**: the root must arrive first. See the A3
  ordering invariant in the runbook §A3.

**Latent constructor issue (D-065, flagged not fixed):** `from_dm_space_create` builds its bundled
auto-invite via `build_membership_event` → **empty `prev_events`** (root-shaped). A node gate-rejects
that at `validate_event` step 10 (non-root needs ≥1 predecessor). No production caller sent it before
A3, so the malformed shape was never exercised through dispatch. **A3 overrides at the call site**
(rebuilds the invite tip-chained to the auto-room) and leaves the constructor untouched, with a
pinning witness test (`from_dm_space_create_auto_invite_has_empty_prev_events_latent_bug`). The
constructor fix (and removal of the A3 override) is its own future touch.

**Known out-of-scope DM-feature gap:** how an invitee on a *different* Node discovers and joins a
single-homed DM (federation disabled; the invitee starts as a `pending_invite`, not a member, so gets
no fan-out until they join) is a pre-existing DM-feature question. **A3 forms creator-home-Node state
only** — it neither builds nor breaks the invitee-join-across-nodes flow.

## 2. Block A — client-feature (AC-D5)

Ships `ops::members`, `ops::leave`, `ops::create_dm_space`, each then exposed on the `.aicontrol`
surface (the adapter wraps the new `ops::*` exactly as M7 v1 wrapped the existing 14). **Freezes the
verb set** — Block B's token/idempotency state designs against final inputs. Order within A:
`members` (A1) ships the lift **and** the shared key-less `from_dm_space_create_node` constructor its
DM-replay branch needs; `leave` (A2) is pure adapter; `create-dm-space` (A3) last, reusing the A1
constructor in its node ingest arm (CP-1, node-arm-only). Block A close = the verb set is frozen.

## 3. Block B — hardening (gated on M7C-D1/D2)

AC-D4 per-connection token first (introduces the plane-1 per-connection state), then AC-D6
idempotency key riding that state. Both per M7C-D1/D2; both B-subsumable. Keep AC-D4 compatible with
the deferred pipelined-handler arc (`CONCURRENT_COMMAND_NOT_ALLOWED` stays wired-but-non-firing
under the serial handler). Cannot start before Block A freezes the verb set.

## 4. Block C — `nodes` filter `ordered_nodes` widening

The C3-documented gap (EV-D4 `nodes` dimension). Independent of A/B; no stale premise surfaced for
it this pass (not deeply traced — runs its own pre-Block-C check). Close the cluster D-074-atomic
after C.

## 5. Sequence

Audit (J-213 ✅) → this design → runbook → **Block A** (members · leave · create-dm-space + node
DM-init arm; freezes verb set) → **Block B** (AC-D4 token → AC-D6 key) → **Block C** (`nodes`
widening) → close. Each block stabilises the inputs the next needs.

## 6. Explicitly OUT — named homes

- Per-driver-identity control plane (**M7C-D1/D2 end-state B**) → **privilege-model / control-plane-identity arc** (activates AC-D4's reserved trio as a unit; A is B-subsumable so it lands with zero rework).
- Plugin-write verbs (A7-D1) → temperature-plugin arc (gated on a 2nd plugin).
- `CONCURRENT_COMMAND_NOT_ALLOWED` / pipelined handler → own arc (AC-D4 stays compatible).
- `migrate-start` (A4-D2) → migration subsystem.
- Live config reload → M7-standalone.

## 7. Carries to implementation (verify, not blockers)

- `leave` federation propagation — fan-out/federation is the generic event-type-agnostic path; no
  leave-specific exclusion found. Confirm at impl.
- `create-dm-space` node DM-init arm — confirm the three client-sent events build the DM SpaceState
  correctly on ingest (the arm is the new work; the auto-room/invite appliers are existing).

## 8. Next-active

Implementation runbook `tasks/M7C_COMPLETION_IMPL.md` — Block A/B/C commit plan + Joe-lock
checkpoints (token-binding seam before Block B; node DM-init arm before `create-dm-space`). Clair
stood down until the runbook closes.

## 9. Cross-references

- Audit: `tasks/M7C_COMPLETION_AUDIT.md` (H1/H2/H3 + the §6 questions this doc locks).
- Code: `xgen-client/src/{ops.rs,aicontrol.rs}`, `xgen-core/src/space/state.rs`
  (`from_dm_space_create:226`, `apply_leave:604`, `build_dm_space_create_event:1098`),
  `xgen-core/src/node/runtime.rs` (`:325` ingest match), `xgen-core/src/message/exchange.rs`
  (`validate_steps_8_13`), `xgen-common/src/wire.rs` (`MembershipLeave`/`StateDmSpaceCreate`).
- Inherited: AC-D4/AC-D5/AC-D6 + EV-D1/EV-D4/EV-D6 in `docs/xgen_aicontrol_implementation.md` v1.4.
- Discipline: D-065 · D-066 · D-069 · D-071 · D-078.
