# M-SPACE-ADMISSION Leg E Phase-0 — the rejoin story: creating `left_at` and paying for every clause deferred onto it

> **Status**: ACTIVE  
> Version: 1.1  
> Date: Aug 2026  
> **Last updated**: 2026-08-23  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — WHAT THIS FILE IS

Leg E of **M-SPACE-ADMISSION — who may join a Space, and how a leaver comes back**. It is the leg that **creates `left_at`**, and therefore the leg onto which four previous legs deferred work they could not build.

🎯 **Its subject is `D-154`'s five clauses.** Everything else in it is a consequence of them.

🛑 **It is a Phase-0, not a runbook.** It grounds, measures, splits and routes. It writes no code and locks nothing. §8's one question is Joe's; **the rest of this document did not wait for it** (`D-123`).

📌 **v1.1 (2026-08-23, J-765) carries three annotations at their sites** (`D-131` — corrected, never erased): **`F-E`'s citation was false** (§4b), **the boundary shape is decided** (§5d), and **v1.0's own leg split contradicted `E-0` §5e two screens away** (§7). **v1.0's text stands beneath each.**

---

## §1 — STATE, RE-MEASURED AT OPEN

| | measured |
|---|---|
| `HEAD` at v1.0 | **`2965e08`** = `origin/main` by `git ls-remote origin refs/heads/main` |
| `HEAD` at v1.1 | **`72262f6`** = `origin/main` by `ls-remote`, tree clean |
| JOURNAL | max **J-764** |
| DECISIONS | 161 entries, max **D-155** |
| ROADMAP | **v7.49** |
| cargo | **1623 / 0 / 62 × 56 SUITES** — measured on the delivered tree at `2965e08` (J-763); **zero `.rs` since** ⇒ carried by construction |
| vitest | **172 / 172 × 9 FILES** — carried by scope |
| svelte-check | **0 / 34 / 15** — carried by scope |
| catalogue | 🛑 **UNMEASURED.** Not carried, not cited. |

🛑 **Every `file:line` in this document is measured at `2965e08`/`72262f6` — identical trees for `.rs` — and says so** (`D-152`).

---

## §2 — THE BACKLOG SWEEP. **STEP ONE, NOT A COURTESY.**

🔑 **Leg D's second deviation (J-763) was a DoD item assigned to Leg D that appeared in no Leg D document.** `C-8`'s species one layer up: *a register that exists, is authoritative, and is not consulted at the moment of allocation.* Leg E is named as the destination in more places than any other leg in this arc, so the sweep ran **before a line of §4 was written**.

**Method:** `git ls-files`, `.claude/worktrees/` excluded, case-sensitive `Leg E` / `LEG E` / `LEG_E` / `LEGE`, every hit in a `M-SPACE-ADMISSION` document opened.

### §2a — THE INVENTORY. **TEN ITEMS, ALL CHAT'S.**

| # | item | assigned at | grounded at HEAD |
|---|---|---|---|
| **1** | **`D-154`'s five clauses** — the design | `DECISIONS.md:5787`; `M_SPACE_ADMISSION_PHASE0.md:348` | — |
| **2** | **`D-3` — the `AlreadyMember` gate must consult `left_at`** | `M_SPACE_ADMISSION_LEGD_PHASE0.md:86`, `:136`; `RUNBOOK_SPACE_ADMISSION_LEG_D.md:51`, `:92`; `CLAIR_LEG_D_HANDBACK.md:54` | `state.rs:1173` (was `:1112`) |
| **3** | **`C-3` mechanical — `new_joiner` polarity** | `M_SPACE_ADMISSION_E0_PHASE0.md:204`; `ROADMAP:415` | `xgen-core/src/node/runtime.rs:1713` — 🛑 **§5: it is not mechanical** |
| **4** | **`C-4` — the `left_at` filter on `federation_nodes`** | `M_SPACE_ADMISSION_E0_PHASE0.md` §8; `§9c` item 2 (**taken by Chat, not ruled**) | `runtime.rs:2312` (was `:2260`) |
| **5** | **`C-5` — the two blunt privacy breaks** | `M_SPACE_ADMISSION_E0_PHASE0.md:229` (`§9b`, retaken from Joe) | `fanout.rs:272` · `fanout.rs:488` — unchanged |
| **6** | **`C-6` — `CutoverResult.member_ids` already divergent from spec** | `M_SPACE_ADMISSION_E0_PHASE0.md` §8 | `xgen-core/src/migration/state_machine.rs:233` — 📌 **path corrected** |
| **7** | **`C-7` — `/// Active members`** | `M_SPACE_ADMISSION_E0_PHASE0.md` §8 | `state.rs:232` — unchanged |
| **8** | **`F-E` — `build_membership_event` emits `prev_events: vec![]`** | `M_SPACE_ADMISSION_E0_PHASE0.md:211`, `:230` | `state.rs:2131` — 🛑 **§4b: the citation is FALSE** |
| **9** | **The A-bis inverted test's RENAME** | J-763; `ROADMAP:426`; carried here by Joe 2026-08-23 | `xgen-node/src/tests/space_admission_third_party_join.rs:115` |
| **10** | **§8's convergence argument + §15.7's surviving anchor note** | `M_SPACE_ADMISSION_PHASE0.md` §8, §15.7 | 📌 the ANCHORING half is `Leg G`'s; §5 splits DELIVERY out and keeps it here |

✅ **All ten are present and none was missing.** The sweep's value was elsewhere — it produced findings the inventory did not contain (§6), and **opening item 8's citation retired it** (§4b).

### §2b — WHAT THE SWEEP ALSO CONFIRMS

📌 **`M-SPACE-ADMISSION` is ONE ROADMAP node** (`ROADMAP:368`) carrying per-leg `↳ Owes:` lines — legs are **not** separate tree nodes in this milestone. ⇒ **Leg E needs no new node**, and `M-RP-MEMBER-ACT` Leg E's missing-node defect (J-718) has no sibling here.

---

## §3 — THE CENSUS RE-ANCHORED. **50/50 HOLDS; ONLY THE LINES MOVED.**

🔑 **`E-0` measured at `5da9e53`. Three commits later every applier citation has drifted.** The classification is `E-0`'s and is not re-litigated; the **anchors** are re-measured because Leg E edits at these lines and `D-152` binds.

**Method:** case-sensitive `git grep` for `is_member(` and `member_role(` callers, plus a **line-joined** scan for `.members` on the files holding rustfmt-broken chains (`F-3`'s species — a line-oriented sweep cannot see `ops.rs:2573`).

### §3a — DOOR `D-1` (`is_member`), 13/13 production

`ai_service.rs:522` · `exchange.rs:232, 370, 375, 397, 693` · **`runtime.rs:1717`** · `dm_promotion.rs:72, 109, 148` · `admin_ops.rs:1077, 4191` · `fanout.rs:488`

📌 `encryption/group.rs`'s `is_member` is **MLS's own member set**, not this map — excluded by `E-0` §3 and excluded here.

### §3b — DOOR `D-2` (`member_role`), 17/17 production

`exchange.rs:844, 876, 888, 898, 908, 923, 958` · `algorithm.rs:221` · `state.rs:906, 928, 952, 1124, 1214, 1239, 1347, 1375, 1976`

### §3c — DOOR `D-3` (direct), 20/20 production — **THE DELTA TABLE**

| what | `E-0` @ `5da9e53` | **HEAD** |
|---|---|---|
| `resolve_operator`'s five (**`D-4`**) | `state.rs:1346, 1351, 1356, 1358, 1365` | **`state.rs:1407, 1412, 1417, 1419, 1426`** |
| `apply_join` room guard · **the space-level `AlreadyMember` reject** | `state.rs:1100, 1112` | **`state.rs:1161, 1173`** |
| AI-status projection · roster panel | `ops.rs:2573, 2591, 2595, 2606, 2736` | **unchanged** |
| the fanout **recipient list** | `fanout.rs:272` | **unchanged** |
| DM held-event drain · **`repopulate_dm_federation_nodes`** | `runtime.rs:2106, 2260` | **`runtime.rs:2158, 2312`** |
| *the other party* · the delivery set | `dm_promotion.rs:80, 130` | **unchanged** |
| operator-facing member counts | `admin_ops.rs:3460` · `node/app.rs:4045` | **unchanged** |
| `CutoverResult.member_ids` | `state_machine.rs:233` | **`migration/state_machine.rs:233`** — path corrected |

🔒 **AGGREGATE: 13 + 17 + 20 = 50. `EVER` 0. `INDIFFERENT` 0.** `(i)` stands, measured at HEAD and not carried.

### §3d — 📌 A MEASURED NARROWING `E-0` COULD NOT SEE

🔑 **`C-4` IS DM-ONLY.** The sibling `repopulate_regular_federation_nodes` (`runtime.rs:2333`) sources `federation_nodes` from the **relationships map**, not from `members` (`:2338` returns early for DM Spaces). ⇒ **a regular Space's federation set never contained a departed member's node**, and `C-4`'s fix is one filter in one helper, not two.

---

## §4 — THE BUILDABILITY PROOF. **LESSON ② FROM LEG D, DISCHARGED RATHER THAN ASSERTED.**

🛑 **Leg D died on this exact shape:** its §5 specified an edit in terms of `left_at` while its own §7 forbade creating it, and `V-3c` described a state the code could not reach. **Leg E creates `left_at`, so every deferred clause becomes buildable at one moment.** This section proves it site by site.

| deferred clause | what it needs | reachable once `left_at` exists? |
|---|---|---|
| **`D-154`① rejoin** | a write path that clears `left_at`, re-stamps `joined_at`, re-derives role | ✅ — and see §5d, where opening the site inverts the reason |
| **`D-3` / `D-154`②③** | `state.rs:1173` gates on `left_at.is_none()`; `:1176`'s ban check becomes reachable | ✅ **and `V-3c` becomes runnable** — `apply_ban:1250` retaining a marked member is the state the control needs, and clause ③ creates it |
| **`D-154`④ the gap** | a per-member boundary in the history slice | ✅ **derived, not stored** — §5d |
| **`D-154`⑤ rooms** | nothing new — `apply_leave:1207-1209` already strips room membership and the Space-level `apply_join` never restores it | ✅ **already true; the clause is a NON-EDIT.** 📌 Recorded so nobody writes code to satisfy a clause the code already satisfies |
| **`C-3`** | `is_member` at `runtime.rs:1717` answering `false` for a departed member | ✅ **discharged by `(i)` alone** — §5a |
| **`C-4`** | `left_at.is_none()` in `runtime.rs:2312`'s loop | ✅ one filter |
| **`C-5`** | `:272` filtered directly; `:488` via `is_member` | ✅ |
| **`C-7`** | a doc comment | ✅ |

### §4b — 🛑 ANNOTATION AT THE SITE (`D-131`, J-765) — **`F-E`'s CITATION IS FALSE, AND THE PARAGRAPH BELOW IT IS RETRACTED.**

**`D-153` binds: a finding accepted from another seat is not re-driven until its citation has been opened. The citation was opened, and it does not say what six records say it says.**

✅ **MEASURED AT `72262f6`. Every PRODUCTION caller of `build_membership_event` sets `prev_events` on the very next statement:**

| caller | chain |
|---|---|
| `xgen-client/src/ops.rs:933` | `:940` — `invite_unsigned.prev_events = vec![room_id]` |
| `xgen-node/src/admin_ops.rs:4020` | `:4021` — `ev.prev_events = tips` (from `graphs.current_tips()`) |
| `xgen-core/src/space/state.rs:498` | ❌ **does not chain — and is DISCARDED in production** |

🔑 **`ops.rs:917-919` STATES IT IN THE CODE:** *"The constructor's bundled auto-invite (empty `prev_events` — latent bug, `D-065`) is discarded; the invite is rebuilt tip-chained below."* ⇒ **the one production event that would carry an empty `prev_events` never reaches a Node**, and the codebase had already recorded it, under `D-065`, as latent.

🔑 **AND `ops.rs:1766` STATES THE HELPER'S CONTRACT:** *"the empty-`prev_events` `build_membership_event` helper is for root-adjacent callers, not this one."* ⇒ ***the empty vec is the documented contract, not a defect.*** `join` (`ops.rs:1707`) and `leave` (`ops.rs:1803`) do not use the helper at all — they build `Event::new` with tips fetched from `get_dag_tips`, and `:1798` says why: *"a non-root event with empty `prev_events` would fail DAG validation."*

🛑 **WHAT `F-E` ACTUALLY IS: A FIXTURE-AUTHORING HAZARD, NOT A PRODUCTION DAG VIOLATION.** A **test** that uses the helper for a non-root event and forgets to chain produces exactly the failure that cost two runs. **The counter-idiom already exists in two test modules** — `xgen-node/src/tests/space_admission_gate.rs:56` and `space_admission_mutation.rs:50` both open with *"Deliberately NOT `state::build_membership_event`: that helper emits …"*.

🔑 ***THIS IS `F-A`'s EXACT SHAPE, ONE ARC LATER: a conclusion that is TRUE (fixtures must chain) resting on a citation that is FALSE (a production DAG violation).*** **And it is `D-153`'s named hardest case** — `F-E` arrived as a **non-blocking note** and was folded into `E-0` §8, `E-0` §9b, `ROADMAP:415`, J-761, J-763, `CLAUDE.md`, **and this document's own v1.0 §4 and §7** without the cited line ever being opened. **Seven records, one unopened citation.**

⇒ 🔒 **`F-E` STOPS BEING A PRECONDITION AND BECOMES A FIXTURE RULE**, written into the runbook's fixture section beside the existing idiom. 🛑 **AND NOTHING IN LEG E MAY "FIX" `build_membership_event`** — emitting a chain would break the root-adjacent contract its three callers rely on. 📌 **The `state.rs:498` latent bug is left exactly as the code already records it: named, unfixed, no production consumer.**

**v1.0's text, superseded:** *⚠️ `F-E` is the one item that is NOT unblocked by `left_at` and is a PRECONDITION on the work instead. `build_membership_event` (`state.rs:2131`) emitting `prev_events: vec![]` is invisible to unit tests calling `apply_event` and is a DAG violation on the node ingest path — which is precisely the path Leg E's rejoin fixtures exercise. It has already cost two runs. ⇒ named in E-1's preconditions, not filed as a rider.*

---

## §5 — 🛑 `C-3` IS NOT MECHANICAL. UNDER `(i)` IT SELF-INVERTS INTO CLAUSE ④'s ENFORCEMENT SITE.

`C-3` was carried for two legs labelled *"mechanical"*. **The citation was opened.**

### §5a — THE FIRST HALF DISCHARGES ITSELF

`runtime.rs:1713-1726` computes `already_member` from **`is_member`** — a **`D-1`** door, and `(i)` gates `is_member` on `left_at.is_none()`.

⇒ a rejoiner reads `already_member = false` ⇒ `new_joiner = Some(sender)` ⇒ **the push fires.** 🔑 ***`C-3`'s silent-empty-room failure is closed by the accessor ruling and needs no edit of its own.*** 📌 The compounding with `D-3` that `E-0` warned of **does not materialise**, because both sides move together.

### §5b — AND THE SECOND HALF OPENS

`fanout.rs:276-289`, measured:

```
let history = if req.new_joiner.is_some() {
    rt.stores.get(&space_id).map(|store| {
        let all: Vec<Event> = store.range(0).unwrap_or_default();
        ...
```

🛑 **`store.range(0)` is the ENTIRE store**, minus only the triggering event; delivered at `:340-349` as one `HistoryBatch`.

⇒ **the rejoiner receives EVERYTHING, including the gap.** 🔑 ***`D-154`④ is violated in the opposite direction, by the same line, at the same moment `C-3` is fixed.*** *A finding whose fix creates the defect the ruling was written to prevent is not mechanical.*

### §5c — WHAT THIS RE-ROUTES

🔒 **`fanout.rs:262-289` IS WHERE CLAUSE ④ LIVES, AND IT IS DELIVERY, NOT ANCHORING.** Leg G's `get_rejoin_anchor` decides what a rejoin **hangs off** (`prev_events`, convergence). This decides what she is **sent**. **Two mechanisms, one clause; only the second is in Leg E.**

### §5d — 🔒 THE BOUNDARY SHAPE. **DECIDED BY CHAT (`D-123`), REVERSIBLE ON ONE WORD.**

**The problem clause ① creates:** clause ① **clears `left_at`** on rejoin and re-stamps `joined_at`. ⇒ ***after the rejoin has been applied, one `Option<String>` cannot say both "she left at T1" and "she is back since T2"*** — and a member who leaves and returns twice has two gaps, not one.

| shape | ① user-visible | ② tier | ③ resource |
|---|---|---|---|
| **(A)** keep `left_at` and add `rejoined_at` | — | — | 🛑 **contradicts clause ①, which rules that `left_at` clears.** Refused on the ruling, not on cost |
| **(B)** an absence list on `SpaceMember` (`Vec<{left_at, rejoined_at}>`) | a permanent, federated, per-member record of every departure | 🛑 **the `§6.5` `former_members` GDPR shape re-minted under a new name** — third-party personal data on a replicated object with no erasure story | unbounded growth per member |
| **(C)** ✅ **DERIVE THE PRESENCE INTERVALS AT SLICE TIME FROM THE LOG ALREADY IN HAND** | identical to (B) for the reader; **nothing new is stored about anyone** | ✅ **none** — no new durable record, so no new erasure surface | ✅ **near-zero: `fanout.rs:277` ALREADY materialises and topologically sorts the whole store at this exact site.** The intervals are a filter over a `Vec` already built |
| **(D)** a single `history_from` watermark | — | — | 🛑 **holds ONE boundary; two leave/rejoin cycles lose the middle gap.** Insufficient by construction |

🔒 **CHAT TAKES (C).** 🔑 ***It is the same operation the whole state machine already is:*** `SpaceState` is **never persisted — it is folded from the store** (`§15.5`, measured at `runtime.rs:677/832/857`). Deriving a per-member presence interval from `membership.*` events is a fold, not a new mechanism.

⚠️ **THE OBJECTION THAT LOOKS LIKE IT APPLIES, AND WHY IT DOES NOT.** `§6.5` refused options **(d)** and **(h)** on convergence — *a predicate derived from a partially-synced log answers differently on different nodes.* **That objection was about a GATE**, where every node must reach the identical admission decision. **This is a DELIVERY filter applied by one Node to its own store, bounding what that Node sends from the same log it reads.** The invariant it must hold is local and holds on any partial log: ***no Node serves an event that falls inside a gap.*** 📌 **Stated explicitly because it reads like the refused option and is not one.**

⚠️ **THE HONEST COST, NAMED:** the interval walk must handle **both departure shapes** — `membership.leave`, where the departed party is `event.sender`, and `membership.kick` / `membership.ban` / `membership.node_eject`, where the departed party is `content["target_identity"]` and the sender is the actor. **A walk that reads only `sender` silently under-counts gaps for kicked and banned members** — `N-197`'s shape, and it is written into E-2's negative controls rather than trusted.

✅ **CONSEQUENCE FOR THE DATA MODEL: `left_at: Option<String>` IS THE WHOLE FIELD ADDITION.** The decision *removes* work from the field leg rather than adding it.

---

## §6 — NEW FINDINGS FROM THIS SWEEP

### §6a — 🔓 `N-1` — A FIFTH REMOVAL SITE `D-154` DOES NOT NAME. **JOE'S.**

`E-0`'s `C-1` counted **four** `self.members.remove` sites. Re-measured, all four are present and the fourth has a name:

| site | path |
|---|---|
| `apply_leave` | `state.rs:1203` |
| `apply_kick` | `state.rs:1230` |
| `apply_ban` | `state.rs:1250` |
| **`apply_node_eject`** | **`state.rs:1275`** — 🔑 **and `:1277` `self.banned.insert`: it EJECTS AND BANS** |

`D-154` rules **leave ①**, **kick ②**, **ban ③**. 🛑 **`membership.node_eject` — the Node-administrator force-eject (M6 A4-D1), authority `sender == home_node`, reversible via `membership.node_unban` — is unruled.** If it removes while the other three retain, ***"in `members`" means two things depending on how you left*** — the precise ambiguity clause ② was ruled to prevent, at a site clause ② does not reach. **→ §8.**

### §6b — 📌 `N-2` — `§12`'s LEG TABLE ISSUES `G` TWICE, AND ONE ROW MISCITES `D-154`. **CHAT'S. RECORDS.**

`M_SPACE_ADMISSION_PHASE0.md:348` and `:349` both carry **`G`** — the older *"THE REJOIN ANCHOR"* row and the newer *"THE REJOIN ANCHOR VERB — ITS OWN LEG"* (Joe, J-756). **`D-134`: designations are issued unique; the lettered split is a repair applied at revision.**

🛑 **And `:348` cites `D-154`③ for *the gap stays closed*, which is clause ④.** ③ is *ban follows kick*. `DECISIONS.md:5787` is the authority; `CLAUDE.md` states ④ correctly. ⇒ **a reader who follows the citation lands on the wrong clause.** Repaired by annotation (`D-131`), superseded row struck, not deleted.

### §6c — 📌 `N-3` — EVERY INHERITED APPLIER CITATION IS STALE AT HEAD. **CHAT'S.**

§3c is the table. **`state.rs` gained ~61 lines between `5da9e53` and `2965e08`**, so every applier citation in `E-0`, in `D-154`, in the Leg D documents and in `§15.5` points ~61 lines short. **`D-154`'s own *"`state.rs:1122`"* for the replacing `insert` is `:1183`.** ⇒ 🔒 **a `file:line` written into a document without its tree is a citation with a half-life measured in commits** — `D-152` clause 1 stated as a cost.

### §6d — 🛑 `N-200` — A FAILED `[System.IO.File]` READ LEAVES THE PREVIOUS FILE'S CONTENT IN THE VARIABLE.

A four-file sweep loop assigned `$lines = [System.IO.File]::ReadAllLines($abs)`; the fourth path (`xgen-core\src\space\state_machine.rs`) **does not exist** — the file is `xgen-core/src/migration/state_machine.rs`. The assignment threw, **`$lines` retained the third file's content**, and the loop printed `runtime.rs`'s eight hits under a `state_machine.rs` heading — **with an identical line count (6229) as the only tell.**

🔑 ***`N-197`'s species again, and worse than `E-0`'s "absent path vs absent match": here the absent path produced PLAUSIBLE NON-EMPTY OUTPUT ATTRIBUTED TO THE WRONG FILE.*** 🔒 **RULE: assert the path exists BEFORE the read inside any loop, and report each file's own line count with every batch.**

### §6e — 🛑 `N-201` — **THE `insert` `D-154` CALLS THE DEFECT IS, UNDER CLAUSE ①, THE CORRECT BEHAVIOUR.** *(J-765)*

`D-154`'s *Why it earned a `D`* reads: **`apply_join` ends at `self.members.insert(...)` — `HashMap::insert` REPLACES ⇒ whatever `(g)` preserved is overwritten wholesale on the way back in.** Opening `state.rs:1173-1194` at `72262f6`:

- `:1179-1182` — `match self.pending_invites.remove(joiner)` yields `(role, invited_by)`; **no invite ⇒ `(Role::Member, None)`**
- `:1183-1192` — the `insert` writes a fresh `SpaceMember` with that role, that `invited_by`, and `joined_at = event.timestamp`

🔑 **Compare clause ①: `left_at` clears · `joined_at` re-stamped · role RE-DERIVED.** ⇒ ***with `left_at: None` added to the literal, the existing `insert` performs clause ① exactly.*** **The replace is not the defect; it IS the ruling.**

✅ **AND IT SETTLES `invited_by`, WHICH `D-154` DOES NOT NAME.** Re-deriving it from `pending_invites` alongside role is the answer consistent with *presence, never position*: **a rejoiner admitted without an invite was admitted by nobody, and carrying the old inviter forward would assert an admission that did not happen** — a fact `resolve_operator` step 2 (`state.rs:1417-1421`) reads.

📌 **`D-154` IS NOT AMENDED AND NEEDS NO AMENDMENT.** All five clauses stand unchanged; **what is corrected is a supporting argument in its rationale**, and the correction makes the leg *smaller*. 🛑 **Recorded rather than absorbed** (`D-065`) — *a rationale that survives while its own site says otherwise is how a false citation gets a second life.*

---

## §7 — 🔓 THE LEG SPLIT. **CHAT'S SEAT (`D-123`); JOE LOCKS.**

### §7a — 🛑 ANNOTATION AT THE SITE (`D-131`, J-765) — **v1.0's SPLIT CONTRADICTED `E-0` §5e TWO SCREENS AWAY IN THIS FILE.**

v1.0 put the field and the writes in **E-1** and the fifty readers in **E-2**, as separate legs. **§3's own aggregate row quotes `E-0` §5e:** *"`SpaceState.members` is purely present-tense ⇒ **all 50 change meaning at once**."*

🛑 **GROUNDED, NOT REASONED — E-1-alone leaves the tree RED AND WRONG.** With `apply_leave` retaining and the accessors ungated, a departed member reads as present at every one of the 50 sites. Existing production tests assert the opposite by name: `state.rs:3044` · `state.rs:4192` · `runtime.rs:5885` · `derive.rs:1071` · and **`resolve_operator_skips_delegate_who_left_falls_back_to_inviter` (`state.rs:4179`)**, which no accessor ruling reaches because `resolve_operator` reads `self.members` directly.

🔑 ***A leg boundary that leaves the codebase asserting a falsehood is not a boundary — it is a half-applied migration with a commit in the middle of it.*** **Same species as Leg D's §5/§7 contradiction: two screens apart, in a document written by the seat that had just recorded the measurement it contradicted.** ✅ **Corrected, not erased; v1.0's four-row table follows the new one.**

### §7b — 🔒 THE SPLIT

| leg | content | gated on |
|---|---|---|
| **E-1** | **THE MEANING CHANGE — ONE COMMIT, ALL FIFTY READERS.** `left_at: Option<String>` on `SpaceMember` (`state.rs:85-95`) · the four writes (`apply_leave:1203`, `apply_kick:1230`, `apply_ban:1250`, and `apply_node_eject:1275` **pending §8**) retain-and-mark · **`apply_join:1173` gates on `left_at.is_none()`, `:1176`'s ban check becomes reachable ⇒ `D-3` lands and `V-3c` becomes runnable** · `(i)`'s two accessors gate, carrying 30 sites · **`D-3`'s 20 direct sites hand-edited**, incl. `resolve_operator`'s five (`D-4`), `fanout.rs:272` (`C-5`), `runtime.rs:2312` (`C-4`) · `C-7`'s doc comment | §8 |
| **E-2** | **CLAUSE ④ — THE GAP.** The presence-interval filter at `fanout.rs:276-289` per §5d(C). ⚠️ **The gap's MARKER — that something was said while she was away — is appearance and stays Joe's** (`D-154`'s named non-settlement); E-2 builds the boundary and leaves the marker open rather than inventing it | E-1 |
| **E-3** | **RECORDS AND THE CARRIED ITEMS.** The A-bis test **rename** · **`N-2`**'s two repairs · ch3's membership-lifecycle text · JOURNAL + CLAUDE.md + ROADMAP + task docs in **one atomic commit** (`D-074`) | all |

📌 **E-1 is large and that is the honest shape, not a scoping failure.** `E-0` measured that no reader's meaning survives `(g)`; **the size is the measurement, and splitting it would only hide the moment the meaning changes.**

📌 **`C-6` is NOT in this split** — filed non-blocking, belongs to whoever writes `transport.redirect`. **`F-E` is NOT in this split** — §4b retired it to a fixture rule. **`Leg G` is unchanged and still its own leg.**

**v1.0's split, superseded:** *E-1 the field and the writes · E-2 the readers · E-3 clause ④ · E-4 records.*

---

## §8 — 🔓 THE ONE OPEN QUESTION. **JOE'S** (`D-155`).

**Someone is thrown out of a Space by the Node operator** — not by a member with kick rights, and not by their own choice. The Space is being cleaned up, or the operator is acting on an abuse report.

### **What should the Space remember about that person afterwards?**

| | outcome, as a person would see it |
|---|---|
| **(a)** | **The same as a kick.** She stays in the record, marked as gone. If the ejection is ever reversed she comes back as an ordinary member. Anyone auditing the Space later can see she was once here, and when she left. |
| **(b)** | **Nothing. She is gone.** The record is wiped, as it is today. An auditor sees a Space she was never in. |

🎯 **CHAT RECOMMENDS (a).**

**① USER-VISIBLE IMPACT.** Under **(b)** the Space that ejected her **forgets that it did** — while the messages she wrote stay, signed by her, in the retained log. Someone reading that history later sees authored events from a person the Space has no record of ever admitting. **(a)** is the only answer in which the history and the membership record agree.

**② TIER CONSEQUENCE.** The same `T4` argument `D-154` already accepted: `D-093` retains the **bytes** and does not retain **who was in the room**. **(b)** reopens that hole for the single case most likely to be audited — an operator-forced removal.

**③ RESOURCE COST.** **(a)** is one line, structurally identical to clause ②'s. **(b)** is free in build time and expensive in meaning: it leaves `members` meaning two things depending on how you left.

🔑 **AND A FACT MEASURED AFTER v1.0, PUT IN FRONT OF THE QUESTION RATHER THAN USED TO CLOSE IT: `apply_node_eject` ALSO BANS** (`state.rs:1277`). ⇒ ***it reaches `apply_ban`'s exact end state***, and `D-154`③ already rules that a ban retains. **Retaining for `apply_ban` and removing for `apply_node_eject` would draw a line between two paths that end identically.** 🛑 **Whether `membership.node_eject` falls under ③ is a question about MEANING and stays Joe's — this is evidence for the answer, not the answer.**

⚠️ **THE HONEST CAVEAT, NAMED AND NOT TRADED AWAY.** **(a)** makes an ejection a durable record on a federated, replicated object, and `membership.node_eject` is **reversible** — a reversed ejection still leaves the record saying it happened. 📌 `D-154`'s own filed note applies and is still unlooked-at: **`self.banned` is already exactly this shape.** **That look is not this question and must not be smuggled into it.**

🛑 **IT GATES E-1's LOCK, NOT E-1's RUNBOOK.** The runbook carries **both arms** at `state.rs:1275` so the choice is visible in the document Clair reads, rather than settled by whichever arm got written first.

---

## §9 — WHAT LEG E MUST NOT DO

1. **It does not build `get_rejoin_anchor`.** That is `Leg G`, a new wire verb, Joe's seat.
2. **It does not invent the gap's marker.** `D-154` leaves it with Joe. E-2 builds the boundary and stops.
3. **It does not fix `C-6`.** Filed, non-blocking, another owner.
4. 🛑 **It does not "fix" `build_membership_event`** (§4b). Emitting a chain breaks the root-adjacent contract three callers rely on.
5. **It does not touch `ui/**`.** The roster's *rendering* of a departed member is appearance; `ops.rs`'s projection sites are data.
6. **It does not re-litigate `(g)`, `(i)`, `Q-2`(a) or any `D-154` clause.** They are locked. §8 asks about a site **no clause reaches**.
7. **It does not re-run the census.** `E-0` is `COMPLETED`; §3 re-anchors, it does not re-classify.

---

## §10 — DoD

- [ ] §8 ruled by Joe and written into this file
- [x] **§5d's boundary shape decided (Chat) — (C), derive at slice time**
- [x] **§4b — `F-E`'s citation opened and the finding corrected at all its sites**
- [ ] E-1 · E-2 · E-3 runbooked, each locked by Joe before Clair opens it
- [ ] Every gate re-driven by Chat from `HEAD` (Rule 5), none adopted on report
- [ ] `V-3c` **RUN** — with `D-3` absent, a retained banned member is refused `AlreadyMember` instead of `Banned`
- [ ] **`N-199` observed on every restore:** restore → stamp mtime → **require `Compiling <crate>` in the log**
- [ ] `N-2`'s duplicate `G` and miscited clause repaired by annotation (`D-131`)
- [ ] The A-bis test renamed to assert what it tests
- [ ] `roadmap-format-gate.ps1` exit 0 before any ROADMAP commit
- [ ] `D-074` atomic close: code + JOURNAL + CLAUDE.md + ROADMAP + task docs, one commit

📌 **"Commit pushed" is not a DoD item** — `Status: COMPLETED` is the signal.

---

## §11 — FLOORS

| instrument | floor | unit |
|---|---|---|
| cargo | **1623 / 0 / 62** | **× 56 SUITES** |
| vitest | **172 / 172** | **× 9 FILES** |
| svelte-check | **0 / 34 / 15** | — |
| catalogue | 🛑 **UNMEASURED** | — |

🛑 **Never cite a floor without its unit.** 🛑 **Do not write `435` for the catalogue.**

⚠️ **E-1 moves the cargo floor in both directions** — it adds tests and it **edits existing ones whose assertions `(g)` inverts**. The delta is a **measurement** taken with `--skip` on the delivered tree, never arithmetic against a carried number (`A-bis`'s method, J-755).
