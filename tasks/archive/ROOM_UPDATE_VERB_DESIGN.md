# Design + Runbook — Thin-verb Arc 3: `room_update` (MP-C-08 / PG-12 per-room override)
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-10  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. Status

Phase-0 complete ([ROOM_UPDATE_VERB_AUDIT.md](ROOM_UPDATE_VERB_AUDIT.md) v1.0).
F-RU-1..4 **Joe-LOCKED** (2026-06-10) by recommendation. **Pre-fold gate
CLEARED:** `invite --role moderator` parses + seats Moderator — `apply_invite`
(state.rs:965) `Role::from_str(content["role"]).unwrap_or(Member)` → Moderator;
`apply_join` (state.rs:1006) seats the member at the invited role. No promote
path needed → the arc stays thin. Design + runbook folded; greenlit to impl.

---

## 2. The four locks (Joe, 2026-06-10)

| Fork | Lock |
|------|------|
| **RU-D1 — override CLI arg shape** | `--deny <role>:<perm>` + `--allow <role>:<perm>`, repeatable `Vec<String>`, parsed to `(Role, RoomPermission, Effect)`. Applier **replaces wholesale**. **Binding add (D-065):** the wholesale-replace semantics MUST be explicit in `RoomUpdateArgs` help text **and** the Appendix F entry — *"sets the room's COMPLETE override set; unlisted overrides are cleared."* A silent wipe on a production verb is the footgun. |
| **RU-D2 — oracle** | assert **both** halves: enforcement (assert-the-reject on the room2 post, inherits MP-F5) + positive per-room (override in resolved room2 state; room1 post converges). |
| **RU-D3 — topology** | single-node (override-honored is node-local). |
| **RU-D4 — witness role** | `(Moderator, SendMessages, Deny)`; bob seated as moderator (the matrix flagship). |

**4-dispatch-arm rule promoted to DECISIONS** (next free D-NNN, Chat lands it at
the bridge): a client verb-add = clap struct + ops + CLI shim + **FOUR** dispatch
arms (main CLI · app run-path · batch · **aicontrol**). Cited here per the ban
arc's empirical catch.

No DECISIONS change in-arc (RU-D# arc-local, D-069); the 4-arm D-NNN is Chat's bridge.

---

## 3. Change surface (mirror `create_room` + ban send-confirm; FOUR arms)

1. **`RoomUpdateArgs`** (app.rs): `--space <id>` `--room <id>` `--deny <role>:<perm>` (repeatable) `--allow <role>:<perm>` (repeatable). **Help text states wholesale-replace** (RU-D1).
2. **`ops::room_update`** (ops.rs): parse each `--deny`/`--allow` spec → `(Role::from_str, RoomPermission::from_str, Effect)`; an unparseable spec → `anyhow` error (`BAD_ARGUMENT`). `get_dag_tips(space)` anchor; `build_room_update_event(key, space, room, prev, &overrides)`; sign; `send_event_confirmed` → `apply_single_event_confirm("room-update")` (MP-F5 site). `RoomUpdateResult { event_id, space_id, room_id }`.
3. **`cmd_room_update`** shim (mirror `cmd_create_room`); print the room + the applied override count.
4. **`ClientCommand::RoomUpdate(RoomUpdateArgs)`** + **4 dispatch arms**: main.rs · app.rs run-path · batch.rs · **aicontrol.rs `Box::pin`**.

**Wire-neutral** (overrides ride signed content; builder shipped Arc D). Authoring
gated Admin+ (`StateRoomUpdate` → `ChangeInfo`, exchange.rs:851) — alice (owner) passes.

## 4. Witness — MP-C-08 (C5, single-node) + RED-on-revert

Scenario: alice creates S + **room1** (open) + **room2** → `room_update room2
--deny moderator:send_messages` → invites bob `--role moderator` → bob joins both
rooms → bob posts in room1 + room2.

**Oracle (RU-D2, both halves):**
- **positive / per-room:** room2's resolved state carries `(Moderator,
  SendMessages)→Deny`; bob's **room1** post is accepted + converges (same role,
  no override there → per-room independence);
- **enforcement (assert-the-reject, MP-F5):** bob's **room2** post reply is an
  Error with `reject_code` **4000** (PermissionDenied unmapped — pin empirically,
  MP-A-20 precedent, MP-F2-followon) + `event_id`; the post is absent everywhere.

**RED-on-revert:** neuter the override (`room_update` authors no Deny / wrong
effect) → room2 has no Deny → bob's room2 post is accepted (Ok, present) → the
enforcement assert (`.error()` + reject_code + absence) flips RED.

## 5. Runbook (single commit)

1. `RoomUpdateArgs` (wholesale-replace help) + `ClientCommand::RoomUpdate` + `ops::room_update` + `RoomUpdateResult` + `cmd_room_update` + **4 dispatch arms**.
2. `MP-C-08/*` batch (alice + bob; room1 open, room2 Deny) + manifest (single-node; waits order: bob joins after invite; room_update before bob's room2 post; per-room post ordering).
3. `mp_r1_c5::mp_c_08_*` runner: positive (room1 post converges + override in room2 resolved state) + enforcement (room2 post assert-the-reject, reject_code 4000 + absent).
4. Appendix F `room_update` entry **with the wholesale-replace note** (RU-D1, J-323).

**Verification:** build 0 + clippy clean (default + `--all-features` + `--features harness-control`); fast suite green; MP-C-08 heavy GREEN; **empirically pin** the room2 reject_code; RED-on-revert demonstrated.

**DoD:**
- [x] `room_update` verb (clap + ops + shim + **4** dispatch arms incl. aicontrol); wholesale-replace in help + Appendix F.
- [x] MP-C-08 GREEN: positive (bob's room1 post converges) + enforcement (room2 post assert-the-reject, reject_code **4000** + absent). Override-in-room2-state proven by its effect (the Deny firing).
- [x] RED-on-revert demonstrated (neuter `ops::room_update` → no override → room2 post accepted [Ok] → RED; restored → GREEN).
- [x] Appendix F `room_update` entry incl. wholesale-replace note.
- [x] PermissionDenied → reject_code **4000** (unmapped variant — MP-A-20 precedent confirmed; MP-F2-followon ledger).
- [x] build 0 + clippy clean (default + `--all-features`) + suites green.
- [ ] Matrix MP-C-08 → ✅ + §6 recount (**Chat**).

**Empirical (MP-C-08 heavy GREEN):** room1 post converged (no override); room2 post
`reject_code=4000` (PermissionDenied) + absent. Pre-fold gate (moderator-seating)
cleared. No surprises — the arc stayed thin (4-arm surface applied up front; no
new dispatch site surfaced mid-impl, unlike ban).

(No "commit pushed" item. Clair's code commit precedes Chat's doc-bridge.)

---

Per D-065 + D-069 + D-071 + D-074. MP-R1-D9 (assert-the-reject, inherited from
MP-F5) + MP-R1-D10 (loop-to-green) govern. RU-D# arc-local; 4-arm rule → D-NNN (Chat).
