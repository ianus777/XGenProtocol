# XGen Protocol — Privilege-Model (Arc D) Implementation Runbook
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-03  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §1 — Overview

Implements `tasks/PRIVILEGE_MODEL_DESIGN.md` v1.0 (PM-D1…D6). Two code commits + a doc-only close, each **green-on-landing** (build + clippy `-D warnings` clean, suite passes). Baseline suite: J-241 **1048**/0/2. Clair implements; Joe pushes; Claude never pushes.

- **C1** — PG-13: wire `verify_tier_assertion` onto the join path (honest Tier-1 no-op). Smaller; de-risks the join chokepoint first.
- **C2** — PG-12-min: per-Room × per-Role override model + `state.room_update` applier + `check_permission` override layer.
- **Close** — D-074 atomic doc-only: gap-audit §5 (PG-13 ✅, PG-12 ✅, **PG-10 → NO-GAP**), ROADMAP, JOURNAL, PM-D# promotion eval.

---

## §2 — C1: PG-13 tier-gate on join

**Files:** `xgen-core/src/node/runtime.rs` (dispatch step 4), `xgen-core/src/auth/tiers.rs` or `message/exchange.rs` (helper + wire mapping, per CP-1).

**Steps:**
1. Add `assertion_tier_of(record: &IdentityRecord) -> u32` (PM-D2): `None → 1`; `Some(v) → v["tier"].as_u64().unwrap_or(1) as u32`. Home it next to the gate consumer; doc-comment names it the **single PG-03 upgrade site**.
2. In `dispatch_event` **step 4**, after the existing AI checks, add a `MembershipJoin`-only branch:
   - `let joiner_tier = assertion_tier_of(sender_record);`
   - `let space_tier = space.auth_tier;`
   - `verify_tier_assertion(joiner_tier, space_tier)` → on `Err`, `return DispatchOutcome::Rejected(...)` carrying wire **3030** (CP-1: variant vs map).
   - Sender record lookup: `self.identity_registry.get(&event.sender)` (already in scope at step 4).
3. Wire-code surface (**CP-1**) — recommended: a thin `AuthError::to_wire_code()` returning `Some((3030, "tier_mismatch"))` for `TierMismatch`/`UnknownTier`, mirroring `ExchangeError::to_wire_code`. Keep the rejection string structured.

**Tests (+):**
- Tier-1 join passes the gate (the no-op: `verify(1,1)=Ok`) — pin it so a future PG-03 change can't silently regress the baseline.
- A synthetic Space with `auth_tier=2` + a joiner whose `trust_assertion` carries `{"tier":1}` → `Rejected` with 3030. (Constructs the gate's *teeth* ahead of PG-03 — the assertion JSON is hand-built; no TrustAssertion struct needed.)
- A synthetic `auth_tier=2` Space + joiner `{"tier":2}` → Accepted.

**Gate:** `cargo test -p xgen-core` + `--workspace`; build all-targets; clippy `-D warnings` (default + `--all-features`). No DECISIONS/ROADMAP change (PM-D# arc-local).

---

## §3 — C2: PG-12-min per-Room overrides

**Files:** `xgen-core/src/space/membership.rs` (`RoomPermission`, `Effect`), `xgen-core/src/space/state.rs` (`RoomState` field, `apply_room_update`, dispatch arm, builder), `xgen-core/src/message/exchange.rs` (`check_permission` override layer).

**Steps:**
1. **Types** (`membership.rs`): `RoomPermission { SendMessages, Invite, Kick, Ban, ChangeInfo }` + `Effect { Allow, Deny }`. Derive `Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize` (snake_case wire). `from_str`/`as_str` on `RoomPermission` mirroring `Role`.
2. **State** (`state.rs`): add `RoomState.permission_overrides: HashMap<(Role, RoomPermission), Effect>` (PM-D4; representation = **CP-2**). Default empty in `apply_room_create`. Confirm it rides `RoomState`'s `PartialEq`/`Eq` (M8 oracle).
3. **Applier** (`state.rs`): split the `state.rs:481` arm — `StateRoomUpdate => self.apply_room_update(event)`; **leave `StateSpaceUpdate => Ok(())`**. `apply_room_update`: resolve room by `event.room_id`; parse `content["permission_overrides"]` (array of `{role, permission, effect}`); **replace** the room's set (PM-D3; merge = **CP-3**). Unknown role/permission strings → skip that entry (forward-compat), do not error.
4. **Builder** (`state.rs`, PM-D6): `build_room_update_event(key, space_id, room_id, prev_events, overrides) -> Event` — the `build_room_create_event` pattern (`:1050`). Test-facing.
5. **Enforcement** (`exchange.rs` `check_permission`, PM-D5): add the room-aware layer per design §5 — `event_type → Option<RoomPermission>` fold (**CP-4**); for a mapped permission + non-empty `room_id`, consult `space.rooms[room_id].permission_overrides[(role, perm)]`: `Deny → PermissionDenied`, `Allow → Ok`, `None →` existing `can_X` default. **Message arms** (`MessageText | …`) now consult `SendMessages` before their `Ok(())`.

**Tests (+):**
- Override `(Moderator, SendMessages) → Deny` in a room → a Moderator's `message.text` there is `PermissionDenied`; absent → still `Ok` (default membership-only).
- Override `(Member, Invite) → Allow` → a Member (default `can_invite=false`) may invite **in that room**; unaffected elsewhere.
- `apply_room_update` replace semantics: second override event for the room wholesale-replaces the first.
- Authoring gate intact: a Member's `state.room_update` is `PermissionDenied` (existing `can_change_space_info`), an Admin's passes.
- M8 convergence: two concurrent override `state.room_update`s on one room → both ingest orders converge to the same `RoomState` (reuse the C2/derive convergence harness shape).

**Gate:** as C1.

---

## §4 — Close (doc-only, D-074 atomic)

One commit, no code:
- `tasks/PRIVILEGE_MODEL_{AUDIT,DESIGN,IMPL}.md` → **COMPLETED** (v1.1).
- `tasks/PROTOCOL_GAP_AUDIT.md` §5: **PG-13 → ✅ DONE**, **PG-12 → ✅ DONE**, **PG-10 → reclassify GAP-CONFIRMED → NO-GAP** with the audit-trail note (grep-surface error; enforced in `dispatch_event` step 4). Update the open/done rollup.
- `docs/ROADMAP.md` — arc D row → ✅; Present nav flip.
- `JOURNAL.md` — close entry (J-NNN).
- `CLAUDE.md` PLAY — arc D CLOSED; next-active = Joe selects (arc E primitives / migration / jurisdictional).
- **PM-D# promotion eval** — likely all stay arc-local (D-069): arc D *implements* gaps, it doesn't establish a cross-cutting discipline. No DECISIONS.md change expected.

**Suite at close:** C1 + C2 deltas over 1048.

---

## §5 — DoD

- C1, C2 each: build + clippy clean, suite green, the listed tests present.
- Close: all docs COMPLETED, gap-audit §5 reflects PG-13/PG-12 done + PG-10 reclassified, ROADMAP/JOURNAL/PLAY updated in the **same** commit (D-074).
- (Per the task-file DoD rule, "commit pushed" is **not** a checklist item — the `Status: COMPLETED` header is the signal.)

Per Rule 0 / D-065 / D-069 / D-071 / D-074 / D-078.
