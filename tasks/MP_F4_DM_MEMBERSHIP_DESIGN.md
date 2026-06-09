# MP-F4 — DM invitee's room-join dropped by node-side membership resolution — DESIGN

> **Status**: ACTIVE  
> Version: 1.1  
> Date: Jun 2026  
> **Last updated**: 2026-06-09  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. What this is

The design for **MP-F4**, on the Joe-locked **F4-A = A1** (room-scope the membership `state_key`).
Third node-side fix-arc of the loop-to-green (MP-R1-D10): MP-F2 ✅ (J-324) → MP-F3 ✅ (J-326) →
MP-F1a ✅ (J-328) → **MP-F4** (this) → MP-F1b + 4 thin verbs → R1 rerun.

Phase-0 (`tasks/MP_F4_DM_MEMBERSHIP_AUDIT.md` v1.1, commit `71e8b72`) confirmed the gap and locked the
fix direction. This design turns A1 into a precise, provable change. **The spine is §3 F4-D2 — the
cross-scope-conflict proof:** once room-level and space-level membership occupy different key-groups,
a space-level removal (ban/kick/leave/eject) must still dominate a room-level join across *every*
ordering. Phase-0's static read says the `apply_join` space-membership guard + the ban/kick/leave room
cascade carry it; per project principle (D-065) this design **proves it, not assumes it**.

This design is authored before the runbook and is locked with Joe first (D-071).

---

## 2. Grounding (against live `main`, from the Phase-0 audit + this design's deeper read)

- **The collision (audit §2.1):** `state_key_for_event` (`xgen-core/src/resolution/state_key.rs:44`)
  is the single producer of conflict keys. Its `MembershipJoin | MembershipLeave` arm keys
  `("membership", "{space}:{sender}")` — **room-agnostic** (state_key.rs:48). A space-join (`room_id=""`)
  and a room-join (`room_id=R`) by one identity therefore produce the **byte-identical** key string.
- **The drop (audit §2.2):** `derive_resolved` (`derive.rs:76`) groups by key (`find_conflicts`),
  restricts each group to its causal **frontier** (`frontier_of`, derive.rs:179); a frontier of size
  ≥ 2 is a genuine §3.9.1 conflict → `resolve()` elects one winner, the loser is skipped in
  `fold_skipping`. When the DM invitee's space-join (b2) and room-join (b3) are concurrent siblings
  (both on `[invite]`), the frontier is `{b2, b3}` → both `MembershipJoin` → Layer-1 abstains → empty
  home-nodes → Layer-5c lexicographic elects one, drops the other. It is **resolution dropping an
  admitted fact** — b3 is in the DAG; the resolved view loses it. ⇒ the fix is in **keying**.
- **The applier model (the appliers A1 must stay convergent with) — read in full this design:**
  - `apply_join` (state.rs:982): `room_id` non-empty ⇒ **room-level** — **guards** `if
    !self.members.contains_key(joiner) → Err(NotASpaceMember)` (state.rs:988), then
    `room.members.insert`. `room_id` empty ⇒ **space-level** — removes from `pending_invites`, inserts
    into `self.members`.
  - `apply_leave` (state.rs:1022): `room_id` non-empty ⇒ room-level (`room.members.remove`, idempotent,
    no guard). Empty ⇒ space-level (`self.members.remove` + **cascade** `for room … room.members.remove`).
  - `apply_kick` (state.rs:1039): permission-gated; `room_id` non-empty ⇒ room-level
    (`room.members.remove`). Empty ⇒ space-level (`self.members.remove` + **cascade**).
  - `apply_ban` (state.rs:1064) / `apply_node_eject` (state.rs:1092): **always space-level** (no
    `room_id` branch) — remove from members + pending, add to `banned`, **cascade** remove from all
    rooms. `apply_invite` (state.rs:946) / `apply_node_unban` (state.rs:1114): always space-level.
- **Sharpening of the audit's scope (honest, D-065):** the audit named A1 as "room-scope join/leave."
  Reading the appliers shows **kick also has a room-level branch** (state.rs:1052) and **room-level
  leave is live** (`ops::leave` honors `--room`, app.rs:533; `ops::join` honors `--room`, the finding).
  Today room-level join/leave/kick all share the room-agnostic key, so a room-kick and a room-join of
  the same id in the same room **conflict and converge** by sharing that key. If A1 room-scopes
  *only* join, it would put room-kick on the old room-agnostic key and room-join on a new room-scoped
  key → they would **no longer conflict** → a **new** room-kick-vs-room-join non-convergence. So A1
  must be **scope-aware for all three room-capable membership events (join/leave/kick)** — not a
  scope-creep, a *correctness requirement* surfaced by grounding the kick/leave branches.
- **Readers are opaque (audit §7):** the 5 production readers of `state_key_for_event`
  (`runtime.rs:673`; `conflict.rs` ×3; `derive.rs` ×2 — `conflicts_in_log` + the frontier path; and
  `xgen-client/src/ai_service.rs:544`, the R2-F01 client gate) all consume `StateKey` opaquely (`==` /
  HashMap key). No reader inspects the tuple shape. `StateKey` has **zero** serde/persist/wire usage —
  purely in-memory, recomputed each derive. ⇒ A1 is **behavioural; no migration.**

---

## 3. Locked decisions (F4-D1..D5)

### F4-D1 (F4-A = A1, Joe-LOCKED) — scope-aware membership `state_key`

> **v1.1 (J-330) — the join/leave/kick room-scope widening is Joe-LOCKED.** A1 is scope-aware across
> **all three room-capable membership events (join / leave / kick)**, not join/leave alone. This
> completes (does not contradict) the J-329 lock "room-scope join/leave": grounding the appliers showed
> kick and leave also carry room-level branches and room-level leave is live, so leaving them on the
> room-agnostic key while scoping join would regress room-kick/room-leave-vs-room-join convergence
> (§2 sharpening). Invite/ban/eject/unban stay always-space-level (no room-level applier branch).

The membership key reflects the **scope of the membership fact** the event asserts:

| Event | Scope discriminator | Key (A1) |
|---|---|---|
| `MembershipJoin`, `MembershipLeave` | `event.room_id` | empty ⇒ `("membership", "{space}:{sender}")` *(unchanged)*; non-empty ⇒ `("membership", "{space}:room:{room}:{sender}")` |
| `MembershipKick` | `event.room_id` | empty ⇒ `("membership", "{space}:{target}")` *(unchanged)*; non-empty ⇒ `("membership", "{space}:room:{room}:{target}")` |
| `MembershipInvite`, `MembershipBan`, `MembershipNodeEject`, `MembershipNodeUnban` | — (always space-level; no room-level applier branch) | `("membership", "{space}:{target}")` *(unchanged, room-agnostic)* |

- The `room:` infix in the room-scoped key field disambiguates it from a space-level key whose
  identity literal could otherwise collide with a room literal — a defensive, zero-ambiguity format.
  The exact string is a runbook detail; the **invariant** locked here is *space-level and room-level
  membership of one identity occupy different key-groups; room-level membership is further partitioned
  per room.*
- `Kick` splits out of today's bundled `Invite|Kick|Ban|NodeEject|NodeUnban` arm into a scope-aware
  arm (target-keyed, like the bundle, but with the room dimension when room-level). Invite/Ban/Eject/
  Unban stay in the always-space-level arm. This split is **required** (§2 sharpening), not optional.
- **Affected-identity rule unchanged:** join/leave key on `sender` (sender *is* the affected
  identity); kick keys on `target_identity`. A1 changes only the *scope dimension*, never the
  affected-identity dimension.

### F4-D2 (the spine, the load-bearing proof) — space-level removal dominates room-level join across all orderings

A1 moves the **space-removal ↔ room-join** convergence guarantee from *shared-key conflict
resolution* (today: both on the room-agnostic key → `resolve()` picks one) to *order-insensitive
apply* (A1: different keys → both fold → the apply logic must be order-insensitive). This design
**proves** the latter holds for every removal × room-join pair, via the **guard + cascade** being
complementary.

Let `id` = the affected identity, `R` = a room, `RM` = a space-level removal of `id`
(`ban` / space-`kick` / space-`leave` / `node_eject`), `J` = `id`'s room-join of `R`. Under A1, `RM`
(room-agnostic key) and `J` (room-scoped key) are in **different key-groups** ⇒ neither is dropped by
`resolve()` ⇒ both are folded by `fold_skipping` in deterministic topo order (lexicographic tie-break
for concurrent events — same order on every node). Two orderings exhaust the cases:

- **`RM` folds before `J`:** `RM` removes `id` from `self.members` (+ cascade removes `id` from all
  rooms incl. `R`; ban/eject also `banned.insert`). Then `J` (room-level `apply_join`) hits the
  **guard** `!self.members.contains_key(id) → Err(NotASpaceMember)` → swallowed → `id` **not** added to
  `R`. Final: `id` ∉ members, ∉ `R`. ✓
- **`J` folds before `RM`:** `J` adds `id` to `R.members` (its guard passes iff `id`'s space-join
  folded earlier — the only case where a room-join is meaningful). Then `RM` removes `id` from
  `self.members` **and the cascade** `for room … room.members.remove(id)` undoes the `R` insert.
  Final: `id` ∉ members, ∉ `R`. ✓

Both orderings converge to the **identical** state. The **guard** handles removal-then-join; the
**cascade** handles join-then-removal — they are complementary, so space-removal dominates room-join
**without** a shared key. This is the invariant the runbook's permutation test pins (§5). *(Symmetric
sub-cases — space-`leave` self-removal, room-`leave` idempotent-remove, room-`kick` vs room-join on
the same `R` which now correctly **share** the room-scoped key and resolve to one winner — fold the
same way; enumerated in §5.)*

### F4-D3 — reader touch-points: none change; no migration

`state_key_for_event` is the single edit site. All 5 production readers (§2) consume `StateKey`
opaquely → **no call-site changes, none compiler-forced**. `StateKey` is never serialized/persisted/
sent on the wire → **no migration, no wire-format or reason-string change.** The change is invisible
to every layer except the conflict-grouping behaviour it is meant to correct. (D-067: one no-drift
location.)

### F4-D4 — witness flip + sensitivity (the falsifiable proof)

`MP-C-07-LOCAL` (`mp_r1_c5`) is GREEN on **delivery only** today. Post-A1 it asserts **2-party DM
message convergence**: the invitee (bob) resolves as a **room** member, his `b4 message.text` is
accepted (no step-11 `NotARoomMember`), and **both** message `event_id`s — alice `a3` and bob `b4` —
land in Node A's cooperative `.events` set. **Sensitivity witness (mandatory):** reverting A1's
`state_key` change makes b2/b3 a size-2 frontier again → room-join dropped → b4 step-11-rejected → b4
absent → the convergence assertion is **genuinely RED**. Added check (audit §F4-B): the flip asserts
**both** messages, so it also confirms alice's `a3` lands post-MP-F1a (the first DM-*message*
convergence assertion in the suite).

### F4-D5 — scope fence (F1b cross-link: flag, do NOT merge)

MP-F4 touches **only** `state_key.rs` (the membership conflict domain). MP-F1b / (iii) touches
`apply_federation_add` / `federation_nodes` population at membership-apply (`state.rs`) — **different
code**. Cross-link flagged for F1b Phase-0: when F1b populates a DM's `federation_nodes` from "its
members," that is **Space** members; the room-scoped membership facts A1 introduces are room-level and
orthogonal to `federation_nodes`. F1b grounds (iii)/gate-B against the **post-A1** (correct)
single-node DM membership model — the J-329 reason F4 runs first. **Federated `MP-C-07` (`mp_r1_c4`)
stays KNOWN-FAIL** (cross-node DM = MP-F1b); A1 must not flip its annotation.

---

## 4. Change surface

- **`xgen-core/src/resolution/state_key.rs`** — the `MembershipJoin | MembershipLeave` arm becomes
  scope-aware; `MembershipKick` splits into its own scope-aware arm; `Invite/Ban/NodeEject/NodeUnban`
  stay in the always-space-level arm. One helper (e.g. `membership_scope_key(event, affected_id)`)
  keeps the three room-capable arms drift-free (D-067). ~1 function, additive arms.
- **No other production file changes.** Appliers (`state.rs`) are unchanged — A1 corrects *which
  events conflict*, not *what an applier does*; the guard + cascade already present carry F4-D2.
- **Tests:** new `state_key.rs` unit tests (scope-distinct keys; room-keys per-room); new `derive.rs`
  convergence/permutation tests (the DM space-join+room-join concurrent case resolves to both; the
  F4-D2 cross-scope removal×room-join case; room-kick vs room-join on one room still resolves to one);
  the `MP-C-07-LOCAL` witness flip (`mp_r1_c5`).
- **No wire / persistence / reason-string / `--features` surface touched.**

---

## 5. Proof plan

**Unit — keying (`state_key.rs`):**
1. A space-join (`room_id=""`) and a room-join (`room_id=R`) by one `sender` in one `space` produce
   **distinct** keys (the A1 fix). Mirrors the MLS-commit per-epoch key tests.
2. Two room-joins of the same id in **different** rooms produce distinct keys; in the **same** room,
   the **same** key.
3. A room-kick and a room-join of the same id in the same room produce the **same** key (so they
   still conflict — the regression A1 must not introduce, §2 sharpening); a room-kick and a *space*-join
   produce **distinct** keys.
4. Invite/ban/eject/unban keys are unchanged (room-agnostic on target) — pins F4-D1's always-space arm.

**Unit — convergence (`derive.rs`, `assert_converges` permutation harness):**
5. **The finding:** create-DM-shaped log where the invitee's space-join + room-join are concurrent
   siblings on `[invite]` → every arrival permutation derives the invitee as **both** a Space member
   and a room member. Revert A1 ⇒ one is dropped (RED witness at the unit layer).
6. **The spine (F4-D2):** a concurrent space-level **ban** + room-level **join** of one id converges
   to "banned, in no room" under **every** permutation (proves the guard + cascade dominate without a
   shared key — i.e. A1 opened no ban-evasion). Repeat for space-`kick` and space-`leave` × room-join.
7. **No-regression:** room-kick vs room-join on the same room resolves to exactly one winner
   (preserved by the shared room-scoped key).

**Integration / witness:**
8. **`MP-C-07-LOCAL` flip** (F4-D4): delivery-only → 2-party message convergence (`a3` + `b4` in Node
   A's set); genuinely RED on revert.
9. **MP-C-01 regression:** stays GREEN — and is now *robust* to concurrency rather than luck-dependent
   (audit §2.4): c2 (`room=""`) and c3 (`room=R`) are now distinct keys, both fold under any order.
10. **Convergence net:** the 285 binary-convergence integration tests + the M8 net + the existing
    `state_key.rs` / `derive.rs` suites stay green (they exercise space-level membership only — A1's
    room-non-empty branch does not touch them).

Build 0 + clippy clean (default + `--features harness-control`) at each step.

---

## 6. Safety — D-076 discharge

A1 **directly touches the resolution surface** (the conflict domain) — non-trivial, unlike the
delivery-only MP-F2/F3.

**Invariant (the discharge statement):** *a node's resolved membership is byte-identical for every
Space and every existing membership conflict, EXCEPT that (i) a room-level join/leave/kick no longer
collides with a space-level join/leave of the same identity, and (ii) a room-level join/leave/kick is
partitioned per room.* The only behavioural delta is the DM invitee (and any future concurrent
space+room membership) now resolving correctly as a room member.

**Why it holds:**
- Existing membership conflicts are all space-level (`room_id=""`) — their keys are **unchanged**
  (F4-D1 room-empty branch = today's key), so `resolve()` is byte-identical for them (the `derive.rs`
  convergence suite is the regression net).
- The new room-level partition only *separates* facts that were wrongly conflated; it never *merges*
  facts that were separate. Nothing that converged via the old shared key loses convergence **except**
  the space-removal×room-join pair, whose convergence is re-established by F4-D2's guard+cascade proof
  (pinned by test §5.6), and the room-kick×room-join pair, whose convergence is preserved by the
  shared room-scoped key (test §5.7).

So every prior convergence either is untouched (space-level) or is re-proven under A1 (the two
cross-/intra-room pairs). The crux is the F4-D2 proof — the design's spine, not a footnote.

---

## 7. Scope fence + honest boundary (D-065)

- **In scope:** the membership conflict-domain fix (A1) + the witness flip. Node-side, `state_key.rs`.
- **Kick/leave room-scoping is in scope** — required for convergence-correctness (§2 sharpening), not
  scope-creep. Surfaced as a refinement of the audit's "join/leave" wording.
- **MP-F1b (cross-node DM convergence)** is NOT touched; federated `MP-C-07` stays KNOWN-FAIL. F1b
  cross-link flagged (F4-D5), not merged.
- **The §2.3 proximate-cause ambiguity (audit):** *why* b2/b3 end up concurrent (a live-state
  staleness window vs the `get_dag_tips` fallback anchoring b3 to the Space root) is **not** resolved
  here and **does not need to be** — A1 fixes the finding whichever holds, because it corrects the
  conflict *domain* rather than the *anchor*. A ~30-min runtime trace to retire the routed
  "bootstrap re-issue" framing cleanly is optional design/runbook colour, off the critical path.
- **The constructor empty-`prev_events` auto-invite latent bug** (`from_dm_space_create`, J-219) — out
  of scope; `create_dm_space` already rebuilds the invite tip-chained.
- **Honest boundary:** A1 closes *single-node* 2-party DM message convergence. It also closes the
  **latent room-agnostic conflation** for *all* future concurrent space+room membership (the D-077
  forward-coherence win over the narrower A2), not the DM only.
- **No DECISIONS change** (F4-D# arc-local, D-069).

---

## 8. Entry point (Rule 0)

CLAUDE PLAY → JOURNAL J-329 → `tasks/MP_F4_DM_MEMBERSHIP_AUDIT.md` v1.1 (verdict + A1 lock) → this
design → (after Joe-lock) `tasks/MP_F4_DM_MEMBERSHIP_IMPL.md` (runbook). The cross-scope-conflict
proof (§3 F4-D2 + §5.6) is the runbook's central deliverable.

---

*Per D-065 + D-067 + D-069 + D-071 + D-076 + D-077.*
