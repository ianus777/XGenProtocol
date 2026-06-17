# Design + Runbook — Thin-verb Arc 2: `ban` (MP-C-09 + MP-A-14)
> **Status**: COMPLETED  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-10  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. Status

Phase-0 complete ([BAN_VERB_AUDIT.md](BAN_VERB_AUDIT.md) v1.0). The four forks are
**Joe-LOCKED** (2026-06-10) by recommendation. Design + runbook folded (thin-arc
sizing — client verb only over a shipped builder/applier/gate). Greenlit straight
to impl.

---

## 2. The four locks (Joe, 2026-06-10)

| Fork | Lock |
|------|------|
| **BAN-D1 — MP-C-09 topology** | **single-node** — the post-ban-reject (the MP-F5 inheritance, the reason this arc follows F5) is node-local; cross-node ban-convergence rides proven MP-C-02/F1b machinery for little R1-floor gain. |
| **BAN-D2 — MP-A-14 shape** | **enforceable-green + fresh-identity behaviour + M10 breadcrumb** — assert the same-identity re-join refusal (the genuine RED-on-revert); record the fresh-identity open-join as behaviour; breadcrumb cross-identity linkage to M10 (auth-module/reputation). **Stated plainly as green-half + breadcrumb, NOT a clean evasion-blocked pass** (never to be mistaken for full coverage). |
| **BAN-D3 — tranche placement** | MP-C-09 → **C5** (membership-lifecycle cooperative); MP-A-14 → **C6** (logic-adversarial). Both inherit the MP-F5 assert-the-reject oracle. |
| **BAN-D4 — negative gate witness** | **skip** — moderator-bans→PermissionDenied is a separate `can_ban` assertion, not in the two BLOCKED rows; don't widen. |

No DECISIONS change (BAN-D# arc-local, D-069).

---

## 3. Verb-add surface (mirror `ops::invite`, grounded)

1. **`BanArgs`** (app.rs, after `InviteArgs`): `--space <id>` + `--identity <target>`. Space-level ban (cascade removes from all rooms); no role/validity/note. Mirror `InviteArgs` minus the invite-only fields.
2. **`ClientCommand::Ban(BanArgs)`** enum variant (app.rs:357 region).
3. **`ops::ban`** (ops.rs, after `invite`): build `Event::new(MembershipBan, sender, RoomXgid::default(), space, prev_events, now, {"target_identity": args.identity})`, `prev_events` = `get_dag_tips(space)` (fallback `[space]`), sign, `send_event_confirmed` → `apply_single_event_confirm("ban")` — **the MP-F5 single-event site** (so a refused ban also surfaces structurally). `BanResult { event_id, target_identity, space_id }`.
4. **`cmd_ban`** CLI shim (app.rs, mirror `cmd_invite`).
5. **Dispatch arms** (**4** — corrected at impl: the audit's "3 arms" missed the `--aicontrol` routing, which is the path the mptest harness drives): main.rs (~235), app.rs run-path (~877), batch.rs (~385), **and `aicontrol.rs`'s per-verb `Box::pin` routing (~440)** — each mirrors the `Invite` arm. The aicontrol arm is load-bearing: without it the verb is `UNKNOWN_COMMAND` over `--aicontrol` (caught empirically — MP-C-09's ban came back `UNKNOWN_COMMAND` until the arm was added).

**Wire-neutral** (ban event already rides signed content). `can_ban` gate + `apply_ban` cascade unchanged (shipped).

---

## 4. The two witnesses (each a RED-on-revert hard deliverable)

### MP-C-09 (C5, single-node) — ban → converge → post-rejected
- alice (owner) creates S + room → invites bob → bob joins → alice **bans** bob → bob attempts a `send` into the room.
- **Oracle (MP-F5 assert-the-reject — prove the rewritten oracle holds for ban, not just a pass):** bob's `send` reply is an `Error` with `reject_code` present **as a field** + `event_id`; bob's post event is **absent** from the node's transcript; membership/room **excludes bob** (ban applied). Pin `reject_code` **empirically** (≈4000 step-11 non-member — observe, don't assume; same discipline as the A-02/04/17/20 re-grounding).
- **RED-on-revert:** revert the `ban` verb → bob stays a member → his post is **accepted** (reply Ok, present in the room) → the reject-assert + absence + exclusion flip RED.

### MP-A-14 (C6, single-node) — ban-evasion via new identity
- precondition: bob banned (the new verb). **Green half:** bob's **original** identity attempts re-join → refused (`apply_join` → `Banned`); assert-the-reject (reject_code present + join absent + bob not re-added). **Behaviour half:** bob registers a **fresh** identity → open-joins → asserted as "joins as a new member" (recorded behaviour; **M10 breadcrumb** — cross-identity linkage is auth-module/reputation, not a protocol gate).
- **RED-on-revert (the enforceable half):** revert the banned-rejoin refusal (i.e. revert the ban) → the original-identity re-join **succeeds** → the refusal-assert goes RED.
- **Honesty:** the matrix/row note states MP-A-14 is **green-half + breadcrumb**, not a clean evasion-blocked pass.

---

## 5. Runbook (single commit)

1. `BanArgs` + `ClientCommand::Ban` + `ops::ban` + `BanResult` + `cmd_ban` + 3 dispatch arms.
2. MP-C-09 batch (`docs/tests/multiparty_scenarios/MP-C-09/*` + manifest) + `mp_r1_c5::mp_c_09_*` runner (assert-the-reject + ban-applied + bob-excluded).
3. MP-A-14 batch (`MP-A-14/*` + manifest) + `mp_r1_c6::mp_a_14_*` runner (green-half refusal assert + fresh-identity-joins behaviour).
4. Appendix F — `ban` verb entry (close deliverable, J-323).
5. Matrix MP-C-09 + MP-A-14 → flip (Chat doc-bridge at close).

**Verification:**
- build 0 + clippy clean (default + `--all-features` + `--features harness-control`).
- fast suite green; the two new runners GREEN on HEAD (heavy `#[ignore]`).
- **empirically observe + pin** MP-C-09's reject_code (and MP-A-14's refusal code).
- **two RED-on-revert** demonstrations (MP-C-09 ban-revert → post accepted → RED; MP-A-14 ban-revert → re-join succeeds → RED).

**DoD:**
- [x] `ban` verb (clap + ops + shim + **4** dispatch arms incl. aicontrol), wire-neutral.
- [x] MP-C-09 (C5) GREEN: assert-the-reject (reject_code **4000** field + event_id) + bob-absent + bob-excluded-from-membership.
- [x] MP-A-14 (C6) GREEN: same-identity re-join **inert** (ban dominates; bob not re-admitted — membership-effect-absence, NOT assert-the-reject; re-join reply `is_ok=true`, accepted-but-inert) + fresh-identity joins (recorded behaviour + M10 breadcrumb).
- [x] Reject_code pinned empirically: MP-C-09 → **4000** (step-11 non-member, unmapped variant — MP-F2-followon).
- [x] Two RED-on-revert witnesses demonstrated (neuter `ops::ban` target → MP-C-09 post accepted [Ok], MP-A-14 bob re-admitted [3 members] — both RED; restored → GREEN).
- [x] Appendix F `ban` entry.
- [x] build 0 + clippy clean + suites green.
- [ ] Matrix MP-C-09 + MP-A-14 flipped + MP-A-14 noted green-half+breadcrumb (**Chat**).

**Impl-time mechanism correction (A-14, surfaced + grounded):** the audit's pivot
(c) framed A-14's green half as *assert-the-reject* (the banned re-join surfaces a
reject_code). Live grounding (runtime.rs:691, `let _ = state.apply_event(...)`)
+ the empirical run (`is_ok=true`) show the banned re-join is **accepted-but-inert**
— `dispatch_event` has no banned pre-check; `apply_join`'s `Banned` refusal is at
apply (swallowed), so the re-join is Ok-at-dispatch but dropped at resolution (M8
Layer-1 ban>join). So A-14's enforceable green is **membership-effect-absence**
(bob ∉ resolved members), not assert-the-reject. MP-C-09's post-ban *send* is the
genuine assert-the-reject (step-11 in `validate_event`, not swallowed) — the MP-F5
inheritance Joe sequenced for holds there. Audit pivot (c) carries a SUPERSEDED note.

(No "commit pushed" item. Clair's code commit precedes Chat's doc-bridge.)

---

Per D-065 + D-069 + D-071 + D-074. MP-R1-D9 (assert-the-reject, inherited from
MP-F5) + MP-R1-D10 (loop-to-green) govern. BAN-D# arc-local (D-069).
