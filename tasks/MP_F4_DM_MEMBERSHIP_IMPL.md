# MP-F4 — DM invitee's room-join dropped by node-side membership resolution — IMPLEMENTATION RUNBOOK

> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-09  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. What this is

The build plan realising **F4-A = A1** (Joe-LOCKED J-329/J-330): room-scope the membership
`state_key` so a space-level and a room-level membership fact of one identity occupy **different**
key-groups. Executes `tasks/MP_F4_DM_MEMBERSHIP_DESIGN.md` (F4-D1..D5; §3 F4-D2 spine). Third
node-side fix of the loop-to-green (MP-R1-D10), after MP-F2/F3/F1a.

The change is **small and local** — one production function (`state_key.rs`), one helper, no applier
change, no wire/persistence change. The weight is in the **proof**: the three-pair D-076 witness set
(§3 S4) is a hard DoD item — the widening enlarges the convergence surface, so its correctness is
**tested, not asserted** (D-065).

---

## 2. Commit shape

**One implementation commit** (the arc is one fn + tests + the witness flip). Steps S1–S6 land
together; the build is green only with the tests, so splitting buys nothing.

- Production: `xgen-core/src/resolution/state_key.rs` (the key transform + helper).
- Tests: `state_key.rs` units, `derive.rs` convergence/permutation units, the `mp_r1_c5`
  `MP-C-07-LOCAL` witness flip.
- **Not touched:** appliers (`space/state.rs`), the 5 `StateKey` readers, any wire/`--features`/
  reason-string surface.

*(This runbook-authoring commit is doc-only: the runbook + the design v1.0→v1.1 bump. The
implementation commit above is the next step.)*

---

## 3. Steps

### S1 — the key transform + helper (F4-D1)

In `state_key.rs`, introduce one helper and make the three room-capable membership arms scope-aware:

```text
fn membership_scope_key(space: &str, room: &str, affected: &str) -> StateKey:
    if room.is_empty():  StateKey::new("membership", "{space}:{affected}")          // space-level (UNCHANGED)
    else:                StateKey::new("membership", "{space}:room:{room}:{affected}") // room-level (NEW)
```

- `MembershipJoin | MembershipLeave` → `membership_scope_key(space_id, room_id, sender)` (sender IS
  the affected identity).
- `MembershipKick` → **split into its own arm** → `membership_scope_key(space_id, room_id, target)`
  (`target = content["target_identity"].as_str()?`).
- `MembershipInvite | MembershipBan | MembershipNodeEject | MembershipNodeUnban` → **unchanged**
  always-space-level arm: `StateKey::new("membership", "{space}:{target}")` (no room-level applier
  branch; room-agnostic on target).

The `room:` infix guarantees a room-scoped key can never alias a space-level key. The affected-identity
dimension (sender for join/leave, target for kick) is **unchanged** — A1 changes only the scope
dimension.

### S2 — confirm the no-drift / no-migration surface (F4-D3)

- `state_key_for_event` is the single producer; **do not** touch the 5 readers (`runtime.rs:673`,
  `conflict.rs` ×3, `derive.rs` ×2, `xgen-client/src/ai_service.rs:544`) — they consume `StateKey`
  opaquely (`==` / HashMap key). No call-site is compiler-forced.
- `StateKey` is never serialized/persisted/wire-sent → no migration, no wire/reason-string change.
- Appliers (`apply_join/leave/kick/ban/node_eject`) are **unchanged** — A1 corrects *which events
  conflict*, not *what appliers do*. The guard + cascade already present carry F4-D2 (S4 pair 1).

### S3 — `state_key.rs` unit tests (the keying contract)

1. space-join (`room_id=""`) and room-join (`room_id=R`) by one sender in one space → **distinct**
   keys (the A1 fix; mirrors the MLS-commit per-epoch key tests).
2. two room-joins of one id in **different** rooms → distinct; **same** room → same key.
3. room-kick and room-join of one id in the **same** room → **same** key (so they still conflict —
   the regression the kick widening prevents); room-kick and **space**-join of one id → distinct keys.
4. invite/ban/eject/unban keys unchanged (room-agnostic on target) — pins the always-space arm.

### S4 — `derive.rs` convergence tests — **THE THREE-PAIR D-076 WITNESS SET (hard DoD)**

Each is a full permutation harness (`assert_converges`). **Per-pair sensitivity is stated explicitly
(D-065 — no vacuous greens):**

- **Headline / the finding (primary key-revert witness):** a create-DM-shaped log where the invitee's
  space-join + room-join are concurrent siblings on `[invite]` → every permutation derives the invitee
  as **both** a Space member and a room member. **Genuinely RED on reverting A1's `state_key` change**
  (the room-agnostic key collapses them onto one key → size-2 frontier → one dropped). This is the
  test that proves A1 closes the finding.
- **Pair 1 — cross-scope removal × room-join (F4-D2 spine / ban-evasion guard-rail):** a concurrent
  space-level **ban** + room-level **join** of one id → every permutation converges to *banned, in no
  room*. Repeat for space-**kick** and space-**leave** × room-join. **Sensitivity:** the `apply_join`
  space-membership guard + the removal cascade (a broken guard or cascade flips it RED). *Honest note:*
  this pair converges under a full A1 revert too (room-agnostic key → Layer-1 ban>join → same end
  state) — so it is **not** a key-revert flip; its job is to prove A1 opened **no ban-evasion** (the
  guard+cascade dominate without a shared key). It is the most important *safety* test even though its
  sensitivity is guard/cascade integrity, not the key change.
- **Pair 2 — room-kick × room-join (same room-scoped key, conflict preserved):** concurrent
  room-kick + room-join of one id in the **same** room → every permutation converges to one winner.
  **Genuinely RED on the *partial-A1* variant** (scope join only, leave kick room-agnostic → distinct
  keys → non-convergent). This proves the kick widening is load-bearing.
- **Pair 3 — room-leave × room-join (same room-scoped key, conflict preserved):** same shape with
  room-leave. **Genuinely RED on the partial-A1 variant** (leave-not-scoped). Proves the leave widening
  is load-bearing.

> The witness set's "RED on revert" is satisfied **per pair against its real failure mode** — the
> headline finding against a full key-revert; pairs 2/3 against the partial-A1 (scope-join-only)
> variant the §2-sharpening warns about; pair 1 against guard/cascade breakage. This is deliberately
> precise so no test is a tautology (the MP-F3/J-301 vacuity lesson).

### S5 — `MP-C-07-LOCAL` witness flip (F4-D4)

In `mp_r1_c5` (`mp_c_07_local_dm_facet2_delivery_lands`): extend delivery-only → assert **2-party
message convergence** — bob resolves as a room member, his `b4 message.text` is accepted (no step-11
`NotARoomMember`), and **both** message `event_id`s (alice `a3`, bob `b4`) are in Node A's cooperative
`.events` set. Confirm alice's `a3` lands post-MP-F1a (the first DM-*message* convergence assertion).
Rewrite the runner doc-comment + manifest prose (delivery-only → convergence). **Genuinely RED on
reverting A1.** MP-C-01 (`mp_r1_c5`'s sibling) stays GREEN. Federated `MP-C-07` (`mp_r1_c4`) stays
KNOWN-FAIL — A1 must not flip its annotation (cross-node DM = MP-F1b).

### S6 — verification (do, don't assume)

Build 0 + clippy clean on **default and `--features harness-control`**. Suites green: xgen-core lib
(+ the new units), xgen-node lib (convergence/M8 net), xgen-mptest (the flipped witness + MP-C-01).
Run the heavy `MP-C-07-LOCAL` + MP-C-01 with `--ignored --test-threads=1` against a harness-control
node build; quote actual counts at close (Rule 2/5).

---

## 4. Definition of Done

- [ ] **S1:** `membership_scope_key` helper + scope-aware join/leave/kick arms; invite/ban/eject/unban
      unchanged. Single edit site (`state_key.rs`).
- [ ] **S2:** 5 readers untouched; appliers untouched; no wire/persistence/reason-string change
      (confirmed, not assumed).
- [ ] **S3:** the 4 keying-contract units green.
- [ ] **S4 (HARD):** the headline finding test + all three D-076 pairs green, **each demonstrated RED
      against its stated failure mode** (finding → key-revert; pairs 2/3 → partial-A1; pair 1 →
      guard/cascade) — recorded in the close note, not just asserted.
- [ ] **S5:** `MP-C-07-LOCAL` flipped to 2-party convergence (a3+b4), RED on revert; doc-comment +
      manifest prose updated; `a3` post-MP-F1a confirmed.
- [ ] MP-C-01 stays GREEN (and is now concurrency-robust, not luck-dependent — D-077).
- [ ] Federated `MP-C-07` (`mp_r1_c4`) stays KNOWN-FAIL; annotation intact.
- [ ] Build 0 + clippy clean (default + `--features harness-control`); xgen-core / xgen-node /
      xgen-mptest suites green — **actual counts quoted** at close.
- [ ] No DECISIONS change (F4-D# arc-local, D-069). No `MP_findings.md` / matrix edits in the code
      commit (Chat doc-bridge owns MP-F4 → RESOLVED + the matrix flip at close).
- [ ] Arc docs `tasks/MP_F4_DM_MEMBERSHIP_{AUDIT,DESIGN,IMPL}.md` → COMPLETED at close.

No "commit pushed" line — Joe pushes. Commit order (standing): Clair's code FIRST, then Chat's
doc-bridge.

---

## 5. Entry point (Rule 0)

CLAUDE PLAY → JOURNAL J-330 → `tasks/MP_F4_DM_MEMBERSHIP_DESIGN.md` (F4-D1..D5 + §3 F4-D2 spine +
§5.6) → `tasks/MP_F4_DM_MEMBERSHIP_AUDIT.md` v1.1 → this runbook. The three-pair D-076 witness set
(§3 S4) is the central deliverable; the headline finding test is the primary key-revert witness.

---

*Per D-065 + D-067 + D-069 + D-071 + D-076 + D-077.*
