# M7-completion cluster — Phase 0 backing audit
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jun 2026  
> **Last updated**: 2026-06-01  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. What this is

Phase-0 backing audit that **opens** the M7-completion cluster (ROADMAP row, scoped not yet
opened). The cluster closes the `--aicontrol`-shaped remainder of M7: **Block A** client-feature
(AC-D5 — `create-dm-space` · `members` · `leave`), **Block B** hardening (AC-D4 per-connection
token → AC-D6 idempotency keys), **Block C** `nodes`-filter `ordered_nodes` widening (the
C3-documented gap). Per the project's own discipline (D-071: subsystem audits precede dependent
milestones; D-078: production-grounded enumeration), the cluster opens with a live-tree audit of
the three "missing" `ops::*` paths + the post-events `.aicontrol` connection layer, **hunting a
stale-premise catch** of the kind that reshaped C2 (EV-D4 v1.0→v1.1) and C3 (Shape β).

Doc-only. No code, no build/test run. Suite stands at J-212's **939**/0/1, not re-run.

## 1. Premise reconciliation (drift note)

The cluster is named and scoped **only** in `docs/ROADMAP.md` (the 🟡 M7-completion tree row +
Near-future). CLAUDE.md PLAY and JOURNAL J-212 both say "Joe selects the next milestone" and list
the leftovers as **separate** candidate arcs (`--aicontrol` hardening arc · client-feature arc ·
the C5 integration test) — they never absorbed the cluster scoping. That is a Rule-0 / D-069
ROADMAP↔CLAUDE drift; ROADMAP prose is canonical for scope. The cluster-pointer drift-fix lands in
the same atomic commit as this audit (PLAY + JOURNAL flipped to point at the opened cluster).

## 2. Live-tree map

### 2.1 The three Block-A verb paths

`xgen-client/src/ops.rs` exposes **14** verbs: `whoami`, `status`, `spaces`, `rooms`, `register`,
`create_space`, `create_room`, `invite`, `join`, `send`, `history`, `ai_delegate`, `ai_revoke`,
`ai_status`. **`create_dm_space`, `leave`, `members` are absent** from `ops.rs` — the "3 missing
paths" fact holds *at the ops layer*. The audit's job is what lies beneath that surface absence.

**`members`.** `ops::ai_status` (`ops.rs:1031`) drains the Space history via `SyncRequest`
pagination, performs a causal replay into a `SpaceState`, and reads the resulting member set
directly: it returns `members_count: state.members.len()` and looks up per-member `role` /
`invited_by` from `state.members`. The membership set is **already computed** by an existing path;
a `members` verb is a *lift* of `state.members` into its own result struct, not new backing.
(Caveat: `ai_status` bails on DM Spaces — `"ai status against a DM Space is not supported in M3"`
— so a general `members` must also cover the DM case.)

**`leave`.** The protocol primitive is **fully present**, not missing:
- EventType `MembershipLeave` (`membership.leave`) — serde rename, `as_str`, `from_str` all in
  `xgen-common/src/wire.rs`.
- Builder — the generic `build_membership_event` (`state.rs:1221`) constructs invite/join/**leave**/kick/ban.
- Applier — `apply_leave` (`state.rs:604`), wired into `SpaceState::apply_event` at `state.rs:356`;
  removes the leaver from the Space and from every Room.
- Tests — `state.rs:1424` and `:2287` build a `MembershipLeave` and apply it green.

This is the **same shape** as `join`/`invite`, which already have `ops::*` verbs (build membership
event → sign → send). So `leave` is **adapter/lift work**, not event-design.

**`create-dm-space`.** Builder `build_dm_space_create_event` (`state.rs:1099`) + EventType
`StateDmSpaceCreate` (`dm.space_create`) both exist; no `ops::*` verb wires them. Adapter/lift,
parallel to `create_space` (verify the client-side DM flow at design — partial grounding here).

### 2.2 The `.aicontrol` connection layer, post-events arc

`xgen-client/src/aicontrol.rs` (the command pipe) is **structurally intact** after the M7-events
arc — the events arc added a *sibling* `.events` pipe (`events_pipe.rs`) and reshaped the
**node-side** fan-out, not this handler. As-built:
- **Per-connection handler tasks**, one spawned per accept; connections served concurrently.
- **Serial per connection** — each handler is a sequential read→dispatch→reply loop; a single
  connection never has two commands in flight. `ControlCode::ConcurrentCommandNotAllowed` is
  wired but **structurally non-firing** in v1 (serial-by-construction; reserved for a future
  pipelined handler).
- **Per-connection `Bindings`** namespace (`$name` / `$name.field` substitution).
- **State-file serialization** — the three state-mutating verbs run under a shared
  `StateFileLock = Arc<Mutex<()>>`; reads stay lock-free.
- **No `ConnId` here.** `ConnId` (xgen-common, EV-D1) + the multi-connection-per-identity
  `Vec<(ConnId, Sender)>` are **node-side** (`ClientSenders`). The client `.aicontrol` handler has
  no `ConnId` of its own.
- `build_state_data` reads `events_pipe::active_session_count()` (EV-D6) — a **process-wide**
  `AtomicUsize` counting client `.events` sessions.

## 3. Hypothesis verdicts

| # | Hypothesis (as opened) | Verdict | Grounding |
|---|---|---|---|
| H1 | `members` needs new `ops::*` backing | **STALE** | `ai_status` (`ops.rs:1031`) already replays into `SpaceState` and reads `state.members` (`members_count` + per-member role/`invited_by`). `members` = lift/surface-exposure. |
| H2 | no `membership.leave` event ⇒ self-leave is event-design needing a Joe-lock | **STALE (inverted)** | `membership.leave` is fully backed at xgen-core: EventType (`wire.rs`) + builder (`build_membership_event`, `state.rs:1221`) + applier (`apply_leave`, `state.rs:604`, wired at `:356`) + green tests (`:1424`, `:2287`). `leave` is adapter/lift, same shape as `join`/`invite`. |
| H3 | AC-D4/D6 scoped pre-events; connection model may have shifted | **NEEDS-JOE-LOCK** | The `.aicontrol` handler model is intact, but the events arc overloaded "connection/session" across **3 layers** (client `.aicontrol` handler `Bindings` · client `.events` process-wide `active_session_count()` · node-side `ConnId`/`Vec<(ConnId,Sender)>`). AC-D4 "per-connection token" + AC-D6 "session-scoped" were written against **one** referent. |

## 4. The catch migrated (H2 → H3) — honest surface

The cluster brief named **H2 (`leave`)** as the expected catch. The tree shows H2 resolving
**favorably**: the `membership.leave` primitive is complete end-to-end, so `leave` stays adapter
work and Block A does **not** reshape into an event-design beat. The actual load-bearing catch is
**H3**: not that the connection model broke, but that the events arc gave "connection" and
"session" three distinct referents, and AC-D4 (per-connection token) + AC-D6 (session-scoped
idempotency) cannot lock until the design says **which layer** the token and the idempotency key
bind to. This is the C2/C3-shape stale-premise: a design term whose meaning the live tree changed
underneath it.

## 5. Block verdicts (what reshapes)

- **Block A — lighter than scoped.** Framed as "new `ops::*` backing then surface exposure." Reality:
  the backing largely exists. `members` = lift of `state.members`; `leave` = adapter over the
  complete `membership.leave` primitive; `create-dm-space` = adapter over `build_dm_space_create_event`.
  Block A is **surface-exposure + ops-wiring**, not new protocol backing. Pure adapter (D-065)
  holds — *if* the node-acceptance + DM verifies below clear at design.
- **Block B — gated on H3.** Cannot lock AC-D4/AC-D6 until the connection/session referent is
  named. The serial-per-connection handler means AC-D4's "concurrency-aware" is latent until the
  deferred pipelined-handler arc lands; the lock must keep AC-D4 forward-compatible with it
  (ROADMAP's "AC-D4 stays compatible").
- **Block C — unchanged.** `nodes`-filter `ordered_nodes` widening (the C3-documented gap) is
  independent of A/B; no stale premise surfaced here this pass (not deeply traced — flagged for its
  own pre-Block-C check).

## 6. Open questions for the design phase (M7C-D# candidates, arc-local per D-069)

1. **H3 layer-binding (load-bearing).** Does the AC-D4 per-connection token bind to (a) the client
   `.aicontrol` handler session (per-handler `Bindings`, the natural client-local seam), (b) a
   node-side `ConnId`, or (c) the identity? And does AC-D6's idempotency key live per-`.aicontrol`-
   handler (serial, natural) or process-wide (mirroring `active_session_count()`)? Name the layer
   before Block B.
2. **`leave` node-acceptance + federation (Block A verify).** Confirm the node accepts a
   member-initiated `membership.leave` with no semantic reject and propagates it (the xgen-core
   applier is proven; the node accept/validate + fan-out/federation path for member-initiated leave
   is not traced this pass).
3. **`members` DM coverage (Block A verify).** `ai_status` bails on DM Spaces; a general `members`
   must handle DM membership or explicitly scope it out.
4. **`create-dm-space` client flow (Block A verify).** Trace the client-side DM-create path against
   `build_dm_space_create_event` to confirm adapter-only.
5. **AC-D4 ↔ pipelined-handler compatibility.** Keep the token compatible with the deferred
   pipelined-handler arc (`CONCURRENT_COMMAND_NOT_ALLOWED` currently non-firing).

## 7. Explicitly OUT (named homes, unchanged from ROADMAP)

Plugin-write verbs → temperature-plugin arc (gated on a 2nd plugin) · `CONCURRENT_COMMAND_NOT_ALLOWED`
/ pipelined handler → own arc · `migrate-start` → migration subsystem · live config reload →
M7-standalone.

## 8. Next-active

**COMPLETED at J-214** — the design phase closed against this audit; `tasks/M7C_COMPLETION_DESIGN.md`
(ACTIVE) is now the live artefact. Design-phase verifies confirmed all three Block-A verbs (`members`
+ `leave` pure adapter; `create-dm-space` adapter + a key-less node DM-init arm — a second catch this
audit's H-set did not cover, surfaced during the D4 trace). H3 resolved as M7C-D1/D2 (token +
idempotency bind to the `.aicontrol` per-connection plane, B-subsumable). Original next-active below.

**Design phase** — author `tasks/M7C_COMPLETION_DESIGN.md`, lock M7C-D# on the §6 questions
(H3 layer-binding first, it gates Block B), then runbook → Block A → Block B → Block C → close.
Block A freezes the verb set (dependency for Block B's token/idempotency state). Clair stood down
until the design closes.

## 9. Cross-references

- Verb backing: `xgen-client/src/ops.rs` (14 verbs; `ai_status:1031`), `xgen-common/src/wire.rs`
  (`EventType::MembershipLeave`/`StateDmSpaceCreate`), `xgen-core/src/space/state.rs`
  (`apply_leave:604`, `apply_event:356`, `build_membership_event:1221`, `build_dm_space_create_event:1099`).
- Connection layer: `xgen-client/src/aicontrol.rs`; `events_pipe.rs` (`active_session_count`);
  `xgen-common/src/conn.rs` (`ConnId`, EV-D1); node-side `ClientSenders` (`fanout.rs`).
- Locks consumed: AC-D4 / AC-D5 / AC-D6 + EV-D1/EV-D4/EV-D6 in `docs/xgen_aicontrol_implementation.md`
  v1.4 + the M7 / M7-events audit+design task files.
- Discipline: D-065 (honest over polite) · D-066 (`--batch` untouched) · D-069 (arc-local M7C-D#) ·
  D-071 (audit precedes dependency) · D-078 (production-grounded enumeration).
