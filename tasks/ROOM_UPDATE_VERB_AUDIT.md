# Phase-0 Audit — Thin-verb Arc 3: `room_update` (MP-C-08 / PG-12 per-room override)
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-10  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. What this is

The D-071 Phase-0 audit for the **third** thin-verb arc (order Joe-LOCKED J-334:
auth-tier → MP-F5 → ban → **room_update** → thread×3). Grounds the verb-add
surface + the four grounding asks, frames the forks for Joe-lock. No code,
nothing pre-decided.

Arc goal: ship a client `room_update` verb so an admin can author a
`state.room_update` carrying per-Room × per-Role permission overrides (PG-12),
unblocking **MP-C-08** (multi-room space + per-room `Deny` override). Same thin
shape as ban — the builder (`build_room_update_event`), the applier
(`apply_room_update`), and the enforcement layer (`check_permission` override
gate) all shipped in **Arc D (PG-12-min)**; **only the client verb is missing**.

---

## 2. Verb-add surface (thin — client verb only; mirror `create_room` + ban's send-confirm)

`build_room_update_event(key, space_id, room_id, prev_events, overrides: &[(Role,
RoomPermission, Effect)])` ([state.rs:1579](../xgen-core/src/space/state.rs#L1579))
already exists; it serialises overrides to `content["permission_overrides"]` = an
array of `{role, permission, effect}` ([state.rs:1606](../xgen-core/src/space/state.rs#L1606)).
`apply_room_update` ([state.rs:870](../xgen-core/src/space/state.rs#L870)) **replaces
the room's complete override set wholesale** (Arc D CP-3 — present array = the full
set; empty clears; absent key untouched). There is **no** client `room_update` verb.

**Verb-add = FOUR dispatch arms (the ban lesson, applied up front).** Mirror
`create_room` (room-scoped: `--space --room`) + ban/invite's send-confirm:

| # | Site | Change |
|---|------|--------|
| 1 | `RoomUpdateArgs` (new clap struct) | `--space <id>` `--room <id>` + an override-spec arg (fork F-RU-1) |
| 2 | `ops::room_update` (new) | parse override specs → `Vec<(Role, RoomPermission, Effect)>`; `build_room_update_event` chained off the room/Space tip; sign; `send_event_confirmed` → `apply_single_event_confirm("room-update")` (the MP-F5 site — a refused update surfaces structurally too). `RoomUpdateResult`. |
| 3 | `ClientCommand::RoomUpdate` + `cmd_room_update` shim | mirror `cmd_create_room`. |
| 4 | **4 dispatch arms** | main.rs CLI · app.rs run-path · batch.rs · **aicontrol.rs `Box::pin` routing** — all four (the ban arc shipped a verb that was `UNKNOWN_COMMAND` over `--aicontrol`, the harness path, until the 4th arm was added; do not repeat). |

**Wire-neutral** (the overrides already ride signed `content`; the builder shipped
in Arc D). **Authoring authority already gated:** `check_permission`'s
`StateRoomUpdate` arm requires Admin+ (`ChangeInfo`, [exchange.rs:851](../xgen-core/src/message/exchange.rs#L851)) — alice (owner) passes; a non-admin update is refused at validation.

---

## 3. The four grounding asks

### Ask (a) — verb-add surface = 4 arms → **applied (see §2)**. Mirror `create_room`.

### Ask (b) — does PG-12 have teeth today? → **HAS TEETH (enforced at validate; not dormant for the gate itself)**

`check_permission` ([exchange.rs:820–833](../xgen-core/src/message/exchange.rs#L820))
maps a governable event to its `RoomPermission` (`event_room_permission`; e.g.
`message.*` → `SendMessages`), looks up the room's `permission_overrides` for the
**sender's role**, and on `Effect::Deny` returns `ExchangeError::PermissionDenied`
— during `validate_event`, so the violating event is `DispatchOutcome::Rejected`.
Unit-proven: the "Moderators can't post in #announcements" flagship test
([exchange.rs:2464](../xgen-core/src/message/exchange.rs#L2464)).

The gate is **live, not decorative.** It is a "no-op today" only in the same sense
PG-13 was: no client verb authors an override, so no room carries one, so the
lookup always misses. **This arc removes exactly that blocker** → once a `Deny`
override is authorable, the gate bites. **MP-C-08 is green-eligible** (does not
route a finding).

### Ask (c) — MP-C-08 oracle shape → **INHERITS MP-F5 (enforcement half) + positive (per-room) half — not positive-only**

The row is cooperative ("posts honor per-room overrides; each room converges
independently; override enforced + converged"). "Honoring a Deny override" means a
violating post **is rejected** — and post-MP-F5 that reject is **batch-observable**
(it routes through `ops::send` → `apply_single_event_confirm`, the MP-F5 site →
reject_signal → `reject_code` + `event_id`). So the oracle is **both**:
- **enforcement (assert-the-reject, inherits MP-F5):** a Deny-violating post is an
  Error with `reject_code` (PermissionDenied → **4000**, the unmapped variant —
  same as MP-A-20's `can_invite`; pin empirically, MP-F2-followon) + `event_id`;
  the post is **absent** everywhere;
- **positive (per-room independence):** the override is present in the room's
  resolved state; a **permitted** post (same role, a room with *no* override)
  is accepted + converges. Two rooms — one Deny, one open — is the per-room witness.

Not positive-only. The enforcement half is the MP-F5 inheritance Joe sequenced for.

### Ask (d) — single-node vs cross-node → **single-node (lean)**

The override-honored property (a Deny in a room rejects a post there) is
**node-local** (`check_permission` is node-local). Cross-node convergence of the
override + posts rides already-proven MP-C-02 machinery for little R1-floor gain.
Lean single-node (the F-BAN-1 call again).

---

## 4. MP-C-08 witness + RED-on-revert (J-323)

Scenario (single-node): alice creates S + **room1** (open) + **room2** (will carry
the override) → alice `room_update room2` with `(Moderator, SendMessages, Deny)` →
invites bob as **moderator** → bob joins both rooms → bob posts in room1 (allowed)
+ room2 (denied).

- **Oracle:** room2's resolved state carries the `(Moderator, SendMessages)→Deny`
  override; bob's room1 post is accepted + converges (positive + per-room
  independence); bob's room2 post is an Error with `reject_code` 4000 + `event_id`,
  absent everywhere (enforcement, assert-the-reject).
- **RED-on-revert:** neuter `ops::room_update` (no override authored / wrong
  effect) → room2 has no Deny → bob's room2 post is **accepted** (Ok, present) →
  the enforcement assert (`.error()` + reject_code + absence) flips RED.

---

## 5. Forks for Joe-lock (recommendations; none pre-decided)

- **F-RU-1 — override CLI arg shape.** `--deny <role>:<permission>` + `--allow
  <role>:<permission>`, each a repeatable `Vec<String>` parsed to `(Role,
  RoomPermission, Effect)`. *Lean: this.* Single-value-per-flag works with the
  harness `reconstruct_argv` (a JSON array value does **not** reconstruct — it
  becomes one mangled arg; MP-C-08 needs one Deny, so the batch passes
  `{"deny": "moderator:send_messages"}` → `--deny moderator:send_messages`). The
  applier replaces wholesale, so the flags express the room's complete set.
- **F-RU-2 — oracle shape.** Assert **both** the enforcement (assert-the-reject on
  the room2 post) **and** the positive per-room half (override in resolved state +
  room1 post converges). *Lean: both* (per Ask (c)).
- **F-RU-3 — topology.** Single-node *(lean)* vs cross-node. Per Ask (d).
- **F-RU-4 — witness role/permission.** `(Moderator, SendMessages, Deny)` (the
  matrix flagship "Moderators can't post in #announcements"); bob invited as
  moderator. *Lean: this* — faithful to the flagship; `Member` would work too.

---

## 6. Phase-0 DoD

- [x] Verb-add surface enumerated: client verb only; **4 dispatch arms** (incl. aicontrol); mirror `create_room`; wire-neutral; authoring gated Admin+.
- [x] Ask (a) 4-arm surface applied up front (ban lesson).
- [x] Ask (b) PG-12 teeth: **has teeth** (check_permission Deny→PermissionDenied at validate; unit-proven). No-op only for lack of an authoring verb → MP-C-08 green-eligible.
- [x] Ask (c) oracle: **inherits MP-F5** (enforcement assert-the-reject, reject_code≈4000) **+ positive per-room** half; not positive-only.
- [x] Ask (d) topology: single-node (override-honored is node-local).
- [x] RED-on-revert witness stated (neuter the override → room2 post accepted → RED).
- [x] Forks framed (F-RU-1 arg shape · F-RU-2 oracle · F-RU-3 topology · F-RU-4 role); nothing pre-decided.

**Next:** design phase — lock F-RU-1..4, author the folded runbook, impl → close.
Appendix F gets the `room_update` entry (close deliverable, J-323). No DECISIONS
change (RU-D# arc-local, D-069).

---

Per D-065 + D-069 + D-071 + D-074. MP-R1-D9 (assert-the-reject, inherited from
MP-F5) + MP-R1-D10 (loop-to-green) govern.
