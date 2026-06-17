# XGen Protocol — Privilege-Model (Arc D) Design
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jun 2026  
> **Last updated**: 2026-06-03  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — Scope

Design for **Arc D — enforcement-hardening** (privilege-model). Phase-0 backing: `tasks/PRIVILEGE_MODEL_AUDIT.md` v1.0. Closes **PG-13** (tier-gate not wired into join) and **PG-12-min** (per-Room × per-Role permission override). **PG-10 is NO-GAP** (already enforced; reclassify at close, no work here).

Decisions below are arc-local **PM-D1…D6** (D-069); promotion to a real D-NNN is evaluated at close only. Joe-locked 2026-06-03. Doc-only — suite at J-241's **1048**/0/2, not re-run.

Two seams, two chokepoints: **admission** (PG-13, tier-gate on `MembershipJoin`) and **action** (PG-12, per-Room overrides layered over `check_permission`). Neither needs a new wire EventType, the auth-module socket, or a resolution-layer change — M8 carries override convergence for free (see PM-D3).

---

## §1 — PM-D1 — PG-13 gate placement & error mapping

**Lock.** A new tier-gate semantic pre-check in `NodeRuntime::dispatch_event` **step 4** (semantic pre-checks), a **`MembershipJoin`-only** branch. It reads the Space's required tier and the joiner's tier, calls `auth::tiers::verify_tier_assertion`, and on `Err` returns `DispatchOutcome::Rejected` carrying spec wire code **3030** (`tier_mismatch`).

- **Space side** — `SpaceState.auth_tier` (`state.rs:119`), already present.
- **Trigger** — only `MembershipJoin`. Join is in `validate_event`'s `skip_membership`/step-13-skip set (the join *makes* the member), so the tier-gate is a **new** check, not a tweak to `check_permission`. Other event types are unaffected.
- **Placement within step 4** — after the existing AI checks (order is not load-bearing; an AI joining a Space is legitimate — AI is barred only from *owning*, PG-10). Self-contained branch.
- **Error mapping** — `AuthError::TierMismatch` → wire **3030**. `AuthError::UnknownTier` → also reject (malformed tier), 3030. The mechanism (a new `ExchangeError` variant wrapping `AuthError`, vs a direct `AuthError → (code,name)` map) is **CP-1** (runbook §3), not load-bearing for the design.

**Honesty (D-065).** Today every Space is `auth_tier=1` and every joiner resolves to tier 1 (PM-D2), so the gate evaluates `verify_tier_assertion(1, 1) = Ok` — a **genuine no-op**. The deliverable is the join-path plumbing; the gate becomes load-bearing only when a Tier 2–4 assertion source exists (PG-03 + a real higher-tier auth module — out of arc-D scope). `resolution/algorithm.rs:57` already documents `auth_tier` as inert under Tier-1.

---

## §2 — PM-D2 — PG-13 joiner-tier source

**Lock.** The joiner's tier is read from `event.sender`'s `IdentityRecord.trust_assertion: Option<serde_json::Value>` (`registry.rs:45`) via a single helper:

```
fn assertion_tier_of(record: &IdentityRecord) -> u32
//   None            => 1            (baseline: cryptographic identity only)
//   Some(v)         => v.get("tier").and_then(Value::as_u64).unwrap_or(1) as u32
```

- **Single upgrade site for PG-03.** When PG-03 lands a real `TrustAssertion` struct/schema, only `assertion_tier_of` changes (it reads the typed tier instead of poking JSON). Everything downstream — the gate call, the wire mapping — is stable. This is the deliberate seam that lets PG-13 wire ahead of PG-03.
- **Baseline is built-in, not a module.** Tier 1 = "cryptographic identity only" (`tiers.rs` header) — the hardcoded baseline every keypair-holder has. It is *not* an instance of the auth-module slot contract (that contract is Tier 2–4). The future "rebuild Tier-1 as the first auth-module" arc is unrelated to this read and out of arc-D scope.

---

## §3 — PM-D3 — PG-12 override carrier (the near-free find)

**Lock.** Carry per-Room overrides on the **existing** `state.room_update` EventType. No new wire type.

Rationale (audit §2): `state.room_update` already exists, is already **state-keyed per room** (`state_key.rs:70`), and therefore **already converges through M8's `derive_resolved`** — its applier is merely an inert no-op (`state.rs:481`, the SR-F2 deferral). Arc D gives it a real applier for the permission-override content slice.

- **No resolution-layer change.** Override events are state-keyed already; concurrent overrides on the same room are a same-key conflict that M8 resolves deterministically (Layer 5c lexicographic on `event_id` as the last-resort tiebreak). Convergence is free.
- **Ties off part of SR-F2.** This is exactly the "add a real applier when a feature needs it" hatch SR-D4/SR-Q4a left open. Room **name/topic** updates stay deferred (only the `permission_overrides` content slice gets a schema + applier here); `state.space_update` stays a no-op.

**Content schema** (`state.room_update`, applied to `event.room_id`):

```json
{ "permission_overrides": [
    { "role": "moderator", "permission": "send_messages", "effect": "deny" },
    { "role": "moderator", "permission": "send_messages", "effect": "allow" }
] }
```

**Replace semantics (locked):** the event carries the **complete** override set for that Room; the applier replaces the Room's prior set wholesale. Simplest convergent rule (idempotent, resolution-winner-takes-all). Merge semantics are **not** in min scope — flag only if a test genuinely needs it (**CP-3**).

---

## §4 — PM-D4 — PG-12 override model

**Lock.** `RoomState` (`state.rs`, currently `{room_id, space_id, name, topic, members}`) gains:

```
pub permission_overrides: HashMap<(Role, RoomPermission), Effect>   // absent key = inherit
```

- `RoomPermission` — a small fixed enum on the existing governance axes **plus `send_messages`**: `{ SendMessages, Invite, Kick, Ban, ChangeInfo }`. Derives `Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize`.
- `Effect` — `{ Allow, Deny }`.
- **Role set is unchanged** — the fixed 4-enum `Role {Member, Moderator, Admin, Owner}`. No custom roles, no `permissions[]`, no `position`, no `Guest`. The full first-class Role object is **Arc E**, explicitly not here.
- **"Cannot grant what the Space hasn't defined" holds for free** — the override axes are a subset of the existing `can_X` table, which *is* the Space-defined permission set under the fixed-enum model.
- **M8 oracle** — `permission_overrides` participates in `RoomState`'s `PartialEq`/`Eq` (added at M8 C1). A `HashMap` compares order-independently, so it is a correct convergence oracle. Purely additive. Map representation + serde shape (flat tuple-key vs nested) is **CP-2**.

---

## §5 — PM-D5 — PG-12 enforcement seam

**Lock.** `check_permission(event, space)` (`exchange.rs:695`) gains a room-aware override layer. **No signature change** — it already receives `event` (carrying `event.room_id`) and `space`.

Algorithm:
1. Resolve `role = space.member_role(sender)`.
2. Map `event.event_type → Option<RoomPermission>` (the fold table, **CP-4**):
   - `MessageText | MessageFile | MessageReaction | MessageRedact → SendMessages`
   - `MembershipInvite → Invite` · `MembershipKick → Kick` · `MembershipBan → Ban`
   - `StateRoomUpdate | StateSpaceUpdate → ChangeInfo`
   - others → `None` (no override layer; today's behaviour unchanged)
3. If a `RoomPermission` maps **and** `event.room_id` is non-empty, look up `space.rooms[room_id].permission_overrides[(role, perm)]`:
   - `Some(Deny)` → `PermissionDenied`
   - `Some(Allow)` → `Ok`
   - `None` → fall through to the existing `can_X` default.

**New behaviour (the spec's flagship case):** message events are today `=> Ok(())` (membership-only). The override layer makes `send_messages` per-Room-per-Role gateable — enabling *"Moderators can't post in #announcements"* — while remaining membership-only by **default** (an override must be explicitly set to bite).

**Authoring authority is already gated (no new work).** Producing a `state.room_update` is already gated behind `can_change_space_info` (Admin+) via `check_permission`'s existing `StateRoomUpdate` arm — so "only Admin+ may change a Room's overrides" comes built-in.

---

## §6 — PM-D6 — scope boundary (protocol-only)

**Lock.** Arc D ships the **protocol mechanism only**:
- ✅ `RoomPermission` / `Effect` types · `RoomState.permission_overrides` · `apply_room_update` applier · `check_permission` override layer · `verify_tier_assertion` wiring + `assertion_tier_of`.
- ✅ A `build_room_update_event` (override-carrying) builder fn so tests can produce the event (the `build_room_create_event` pattern, `state.rs:1050`).

**Explicitly out of arc D:**
- ❌ A user-facing way to author overrides (CLI / `--aicontrol` / client command) → rides the **UI / ops pass**.
- ❌ The first-class Role object model (custom roles, `permissions[]`, `position`, `Guest`) → **Arc E**.
- ❌ `state.space_update` applier + `state.room_update` name/topic content → stays deferred (SR-F2 remainder).
- ❌ M3-deferred operator privileges (capability override, AI silencing) → remain parked; **noted here so they are not mistaken for arc-D scope.**

---

## §7 — Confirm-at-pickup (D-078) → runbook §3

- **CP-1** — `AuthError → wire 3030` mechanism (new `ExchangeError` variant vs direct map). C1.
- **CP-2** — `permission_overrides` map representation + serde shape. C2.
- **CP-3** — replace-vs-merge for the override set (design locks **replace**; revisit only if a test needs merge). C2.
- **CP-4** — the exact `EventType → RoomPermission` fold table (esp. the message family). C2.

No DECISIONS.md change at open (PM-D# arc-local, D-069). No ROADMAP/JOURNAL-recorded state beyond the arc-open entry. Per Rule 0 / D-065 / D-069 / D-071 / D-074.
