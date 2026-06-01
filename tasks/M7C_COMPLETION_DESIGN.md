# M7-completion cluster — design (M7C-D# locks)
> **Status**: ACTIVE  
> Version: 1.0  
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
- **`members` — pure adapter (lift).** `ops::ai_status` (`ops.rs:1031`) already replays history into
  a `SpaceState` and reads `state.members`. Both `from_space_create:189` and `from_dm_space_create:263`
  seed `state.members`, so a `members` verb covers DM Spaces too — the `ai_status` DM bail is
  operator-resolution-specific, not a membership-read limit. New `ops::members` reads `state.members`.
- **`leave` — pure adapter.** `membership.leave` is fully backed: builder `build_membership_event`
  (`state.rs:1221`), applier `apply_leave` (`state.rs:604`, wired at `:356`). The node accepts a
  member-initiated leave with no semantic reject — `validate_steps_8_13` special-cases only
  invite/kick/ban at step 13; leave falls to default and is gated on step-11 sender-membership +
  signature. New `ops::leave` = build → sign → send, mirroring `join`.
- **`create-dm-space` — adapter + node-init fix (the one non-pure-adapter verb).** See M7C-D4.

### M7C-D4 — DM-create flow: client 3-event send + node key-less DM-init arm

`from_dm_space_create` (`state.rs:226`) takes the **creator's key** and returns
`(SpaceState, auto_room_event, invite_event)` pre-signed with it. The node never holds that key, so:
- **Client side (the verb):** `ops::create_dm_space` builds `state.dm_space_create`
  (`build_dm_space_create_event(key, invitee, home_node)`), runs `from_dm_space_create` locally to
  get the creator-signed auto-room + auto-invite, and **sends all three events** to the node. Thicker
  than `create_space` (3 events vs 1) but still adapter — no new event design.
- **Node side (the fix):** the node's ingest `match` (`runtime.rs:325`) creates a `SpaceState` only
  in the `StateSpaceCreate` arm (`from_space_create`); `StateDmSpaceCreate` falls to the `_` arm and
  silently builds nothing (it needs an already-existing space). **Block A adds a `StateDmSpaceCreate`
  arm**: a key-less DM-init mirroring `from_space_create` (owner = `event.sender`, invitee from
  `content["invitee"]`, DM constraints), after which the separately-arriving auto-room
  (`state.room_create`) and auto-invite (`membership.invite`) apply through the normal appliers.
  Key-less because the node verifies signatures but does not author. This is **primitive-completion,
  not new event-design** — the event, builder, applier, and DM `SpaceState` shape all already exist;
  only the node's create-on-ingest arm is missing. Recorded honestly as the single Block-A verb that
  exceeds pure adapter (D-065).

## 2. Block A — client-feature (AC-D5)

Ships `ops::members`, `ops::leave`, `ops::create_dm_space`, each then exposed on the `.aicontrol`
surface (the adapter wraps the new `ops::*` exactly as M7 v1 wrapped the existing 14). **Freezes the
verb set** — Block B's token/idempotency state designs against final inputs. Order within A: the two
pure-adapter verbs (`members`, `leave`) first; `create-dm-space` last (carries the node-init arm).
Block A close = the verb set is frozen.

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
