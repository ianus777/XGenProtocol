# XGen Protocol — Privilege-Model (Arc D) Phase-0 Audit
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-03  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — Method, scope, vocabulary

### 0.1 Purpose

The D-071 Phase-0 gate for **Arc D — enforcement-hardening** (the planned *privilege-model* arc), selected post-J-241 (M8 CLOSED). Grounds the three Wave-3 candidate gaps against live code **before** any design/lock:

- **PG-13** — tier-gate (`verify_tier_assertion`) not wired into the Room/Space join path.
- **PG-12** — per-Room × per-Role permission override (enforcement point exists, override layer missing).
- **PG-10** — AI capability flags hard-enforced at event time + AI-not-Space-owner.

Reading: the three cluster on one theme — *authorization specified but not enforced* — across **two chokepoints**: an **admission** seam (tier-gate on join, PG-13) and an **action** seam (`check_permission`, PG-12 + PG-10). One combined audit, not three (Joe-locked 2026-06-03).

### 0.2 Verdict vocabulary

Inherited from `PROTOCOL_GAP_AUDIT.md` §0.4: **NO-GAP** · **GAP-CONFIRMED** · **SPEC-DRIFT** · **NEEDS-DESIGN** · **N/A**. Severity S1–S4 per §0.5 there.

### 0.3 Method

Per normative claim: locate the responsible code via `Select-String` across `xgen-*/src/**/*.rs` → read the site → verdict + `file:line` evidence. Spec side read against `docs/xgen_ch2_architecture.md` + `xgen_ch3_specification.md`. Doc-only — suite at J-241's **1048**/0/2, not re-run.

### 0.4 Session-locked outcomes (2026-06-03, feed §6)

- **PG-10 → NO-GAP** (register reclassify, audit-trail note, no build work).
- **PG-13 → wire now as an honest Tier-1 no-op** (forward-looking; the join-path plumbing is the load-bearing deliverable).
- **PG-12 → min scope for Arc D** (per-Room overrides on the existing fixed-role enum); the full first-class Role object model spins to **Arc E** (primitive completion, sibling to PG-08 / PG-03).

---

## §1 — PG-13: tier-gate not wired into join

**Claim (ch2 L798/936).** Room/Space `auth_tier` (spec field; code: "required tier") is *enforced at protocol level* on join.

**Grounding.**

| Element | Finding | Evidence |
|---|---|---|
| The gate predicate | `verify_tier_assertion(assertion_tier, space_auth_tier)` exists; ordered `AuthTier 1..4`; returns `TierMismatch`/`UnknownTier`. | `tiers.rs:142` |
| Production callers | **Zero.** Only the definition + six `#[cfg(test)]` callers. | `tiers.rs:247–289` (all in `mod tests`) |
| Space side of the gate | **Present.** `SpaceState.auth_tier: u32`, parsed from create content, default `1`. | `state.rs:119`, `:175`, `:246/375` |
| Joiner side of the gate | **Absent.** The joiner's assertion tier comes from a Trust Assertion; `TrustAssertion` is not yet a Rust struct (deferred, PG-03). | `flavours.rs:36` |
| Join path shape | `MembershipJoin` is in `validate_event`'s `skip_membership` set and skips step-13 permission (the join *makes* the member). The tier-gate is therefore a **new** semantic pre-check, not a tweak to an existing one. | `exchange.rs` (`validate_event` skip table); `runtime.rs` dispatch step 4 |
| Tier-1 reality | Resolution already acknowledges `auth_tier` as inert under Tier-1: `let _ = space_state.auth_tier; // always tied in Phase 2 Tier 1`. | `resolution/algorithm.rs:57` |

**Verdict: GAP-CONFIRMED (S2).** The predicate exists and the Space side is present; the gate is simply never called on the join path.

**Coupling (load-bearing).** A *real* tier-gate needs a joiner-tier source = a Trust Assertion = **PG-03**. Until PG-03 lands, the joiner tier is implicitly `1`, every Space is `auth_tier=1`, so the gate evaluates `verify_tier_assertion(1, 1) = Ok` — a **genuine no-op** (D-065 honest framing: the wiring is forward-looking; it gates nothing today, becomes load-bearing only at Tier 2–4). The deliverable is the *join-path plumbing*, not new enforcement behaviour at Tier 1.

**Closing cost.** Add a tier-gate check on `MembershipJoin` in the dispatcher's step-4 semantic layer: read `SpaceState.auth_tier`, read the joiner's tier (Tier-1 default until PG-03), call `verify_tier_assertion`, reject on `Err`. Narrow, single-site.

---

## §2 — PG-12: per-Room × per-Role permission override

**Claim (ch2 L948/969).** Roles are defined at the Space level and cascade to Rooms; a Room may *override* specific permissions for specific Roles (narrow **or** extend), but cannot create new Roles or grant permissions the Space has not defined.

**Grounding — as-built.**

| Element | Finding | Evidence |
|---|---|---|
| The role model | Fixed 4-variant enum `Role {Member, Moderator, Admin, Owner}` — **no `Guest`** (spec has 5 built-ins). Ordered privilege. | `membership.rs` |
| The permission table | Hardcoded threshold functions (`can_invite ≥ Moderator`, `can_ban ≥ Admin`, `can_manage_federation == Owner`, …). **No `permissions[]` list, no `position`, no `color`, no custom roles.** | `membership.rs` |
| The enforcement seam | `check_permission(event, space)` is **Space-role-only — no `room_id` parameter.** Role via `space.member_role(sender)` → `can_X`. | `exchange.rs:695` |
| Per-Room override layer | **Absent.** No `RoomPermission` / override structure anywhere; the spec mechanic ("Moderators can't post in #announcements") is unrepresentable. | (grep: no `RoomPermission`) |

**Grounding — spec.** ch2 "The Role Model" (L948–988) specifies a **first-class Role object** `{ id, name, color, permissions[], position, meta-atts }`, 5 built-ins including `Guest`, custom roles between built-ins, and per-Room overrides that **narrow or extend** a Role's permissions in a specific Room, bounded by the Space-defined permission set. (Correction to the initial register framing: overrides are *not* narrows-only.)

**Verdict: NEEDS-DESIGN / GAP-CONFIRMED (S2).** The delta is larger than "override layer missing": the *entire* first-class Role object model is unbuilt (as-built is a fixed enum + threshold table). The Room-override is one slice of that.

**Scope (Joe-locked 2026-06-03).** Arc D takes **PG-12-min**: per-Room × per-Role overrides keyed on the **existing fixed enum** (Member/Moderator/Admin/Owner). The "cannot grant permissions the Space hasn't defined" bound is naturally satisfied — the enum's threshold table *is* the Space-defined permission set. The full first-class Role object model (custom roles, `permissions[]`, `position`, `Guest`) → **Arc E** primitive completion (sibling to PG-08 Thread / PG-03 TrustAssertion).

**Design surface (resolve at the design phase, not here):**
- **Override storage** — a field on Room state vs a new `room.permission_override` Event (state-mutating, so a wire EventType + applier).
- **Override granularity** — per (Room × Role × permission) tri-state (allow / deny / inherit)?
- **Enforcement-lookup shape** — `check_permission` gains a `room_id`-aware override lookup that layers over the `can_X` threshold result (override wins where set; threshold is the default).
- **Interaction with the `can_X` table** — the table becomes the *default* layer; the override is the *per-Room* layer on top.

---

## §3 — PG-10: AI capability enforcement + AI-not-owner

**Claim (ch3 §3.6.10.4 / L2052).** AI capability flags hard-enforced at event time; AI MUST NOT be a Space owner.

**Why the register flagged it GAP-CONFIRMED (and why that was wrong).** `PROTOCOL_GAP_AUDIT` §1.2.3 grepped `validate_event` (the F-4 unified core) and `capability.rs`, found no AI enforcement, and concluded GAP. But `validate_event` **excludes AI checks by design** (`exchange.rs` doc: "AI role + AI operator target/permission checks are NOT in the validation core per design doc §7.7 — they live in step 4 of the `process_inbound` dispatcher"), and `capability.rs` is *NodeAnnouncement* capabilities, not AI-identity capabilities. The grep checked the wrong surfaces.

**Grounding — the production site is `dispatch_event` step 4.**

| Sub-claim | Spec | As-built | Verdict |
|---|---|---|---|
| AI-not-Space-owner (3041) | L2052: Nodes MUST reject `state.space_create` / `state.dm_space_create` from an `is_ai` sender. | Enforced twice: inline `is_ai` reject for space-creation **and** `check_ai_capability`. Tested (`ai_dm_space_create_rejected_regardless_of_dm_initiate`, `ai_space_create_rejected_regardless_of_capabilities`). | **NO-GAP** |
| `dm_initiate` (3042) | L2017: when `false`, AI MUST NOT create a DM Space. | Enforced via `check_ai_capability`. (L2052 AI-not-owner supersedes it for the only gated event type — spec says so explicitly; net behaviour conformant.) | **NO-GAP** |
| `spontaneous_post` | L2018/L2034: **"not Node-validated in Phase 2 — client-side and admin-policy concern."** | Code matches verbatim (`exchange.rs`: "`spontaneous_post` is NOT Node-validated in Phase 2"). | **NO-GAP** |

Evidence: `runtime.rs:905` (`check_ai_capability`), `:915` (`check_ai_operator_targets_pub`), `:919` (`check_permission_pub`), + the inline `is_ai` space-creation reject in `dispatch_event` step 4; `exchange.rs:218` (`check_ai_capability` body); spec `xgen_ch3_specification.md` L2017/L2018/L2034/L2052.

**Verdict: NO-GAP.** All three sub-claims conform to spec as-built. **Recommendation:** reclassify PG-10 in `PROTOCOL_GAP_AUDIT.md` §5 from GAP-CONFIRMED → NO-GAP with an audit-trail note (the grep-surface error). No build work. **Removed from Arc D scope.**

---

## §4 — Cross-cutting

**The authorization pipeline = two live seams.** (a) *Admission* — the tier-gate on join (PG-13, currently absent). (b) *Action* — `check_permission` (PG-12, Space-role-only, no Room layer). PG-10 was a presumed third edge but is already closed and spec-conformant.

**Couplings.** PG-13 ↔ **PG-03** (the joiner-tier source; PG-13 is a Tier-1 no-op until PG-03). PG-12-full ↔ **Arc E** (the first-class Role object model).

**DECISIONS.** No change at audit. Arc-D design decisions promote as arc-local **PM-D#** at the design phase (D-069); cross-cutting promotion evaluated at close only.

---

## §5 — Confirm-at-design (open questions for the design phase)

1. **PG-13 error mapping** — surface `AuthError::TierMismatch` (the existing 3030 "assertion tier below required tier") through the dispatch `Rejected` path; confirm the wire-code mapping.
2. **PG-13 placement** — exact position of the tier-gate in `dispatch_event` step 4 (before or after the existing AI checks; only `MembershipJoin` triggers it).
3. **PG-12 override storage** — Room-state field vs new `room.permission_override` Event (the latter is state-mutating: a wire EventType + state-key + applier, and convergence-relevant per M8's resolution layer).
4. **PG-12 override granularity** — the (Room × Role × permission) tri-state shape.
5. **PG-12 enforcement-lookup** — how `check_permission` layers the per-Room override over the `can_X` threshold default (override wins where set).

---

## §6 — Recommendation + arc shape

**Arc D = PG-13 + PG-12-min.** PG-10 → NO-GAP (register note only).

Provisional sequence (audit → design → Joe-lock → runbook → Clair):
- **Design** (`tasks/PRIVILEGE_MODEL_DESIGN.md`, PM-D# arc-local) — resolves §5 confirm-at-design items; PG-12-min data model + enforcement seam; PG-13 gate placement + error mapping.
- **C1 (Clair)** — PG-13: wire the tier-gate onto `MembershipJoin` in dispatch step 4 (Tier-1 no-op). Smaller; de-risks the join chokepoint.
- **C2 (Clair)** — PG-12-min: per-Room × per-Role overrides + `check_permission` room-aware lookup.
- **Close** (D-074 atomic) — `PROTOCOL_GAP_AUDIT.md` §5: PG-13 ✅, PG-12 ✅, **PG-10 reclassified NO-GAP**; ROADMAP; JOURNAL; PM-D# promotion evaluation.

**Audit complete (v1.0).** Three gaps grounded: PG-13 GAP-CONFIRMED (S2, coupled to PG-03), PG-12 NEEDS-DESIGN (S2, min-scope locked), PG-10 NO-GAP (register reclassify). Per Rule 0 / D-065 / D-069 / D-071. Earns a JOURNAL line when Arc D opens (design lock) per D-074.
