# M-SPACE-ADMISSION Leg E Phase-0 — the rejoin story: creating `left_at` and paying for every clause deferred onto it

> **Status**: ACTIVE  
> Version: 1.0  
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

🛑 **It is a Phase-0, not a runbook.** It grounds, measures, splits and routes. It writes no code and locks nothing. §8's one question is Joe's; **the rest of this document did not wait for it** (`D-123` — the recurring failure is UNDER-stepping).

---

## §1 — STATE, RE-MEASURED AT OPEN

| | measured |
|---|---|
| `HEAD` | **`2965e08`** = `origin/main` by `git ls-remote origin refs/heads/main` — **the remote ref, not the tracking ref** |
| tree | clean |
| JOURNAL | max **J-763** |
| DECISIONS | 161 entries, max **D-155** |
| ROADMAP | **v7.48** |
| cargo | **1623 / 0 / 62 × 56 SUITES** — measured on the delivered tree at `2965e08` (J-763) |
| vitest | **172 / 172 × 9 FILES** — carried by scope |
| svelte-check | **0 / 34 / 15** — carried by scope |
| catalogue | 🛑 **UNMEASURED.** Not carried, not cited. |

🛑 **Every `file:line` in this document is measured at `2965e08` and says so** (`D-152`). Inherited citations that no longer hold are shown as a delta in §3, not silently corrected.

---

## §2 — THE BACKLOG SWEEP. **STEP ONE, NOT A COURTESY.**

🔑 **Leg D's second deviation (J-763) was a DoD item assigned to Leg D that appeared in no Leg D document.** `C-8`'s species one layer up: *a register that exists, is authoritative, and is not consulted at the moment of allocation.* Leg E is named as the destination in more places than any other leg in this arc, so the sweep ran **before a line of §4 was written**.

**Method:** `git ls-files`, `.claude/worktrees/` excluded, case-sensitive `Leg E` / `LEG E` / `LEG_E` / `LEGE`, every hit in a `M-SPACE-ADMISSION` document opened.

### §2a — THE INVENTORY. **TEN ITEMS, ALL CHAT'S.**

| # | item | assigned at | grounded at `2965e08` |
|---|---|---|---|
| **1** | **`D-154`'s five clauses** — the design | `DECISIONS.md:5787`; `M_SPACE_ADMISSION_PHASE0.md:348` | — |
| **2** | **`D-3` — the `AlreadyMember` gate must consult `left_at`** | `M_SPACE_ADMISSION_LEGD_PHASE0.md:86`, `:136`; `RUNBOOK_SPACE_ADMISSION_LEG_D.md:51`, `:92`; `CLAIR_LEG_D_HANDBACK.md:54` | `state.rs:1173` (was `:1112`) |
| **3** | **`C-3` mechanical — `new_joiner` polarity** | `M_SPACE_ADMISSION_E0_PHASE0.md:204`; `ROADMAP:415` | `xgen-core/src/node/runtime.rs:1713` (was `:1665`) — 🛑 **see §5; it is not mechanical** |
| **4** | **`C-4` — the `left_at` filter on `federation_nodes`** | `M_SPACE_ADMISSION_E0_PHASE0.md` §8; `§9c` item 2 (**taken by Chat, not ruled**) | `runtime.rs:2312` (was `:2260`) |
| **5** | **`C-5` — the two blunt privacy breaks** | `M_SPACE_ADMISSION_E0_PHASE0.md:229` (`§9b`, retaken from Joe) | `fanout.rs:272` · `fanout.rs:488` — both unchanged |
| **6** | **`C-6` — `CutoverResult.member_ids` already divergent from spec** | `M_SPACE_ADMISSION_E0_PHASE0.md` §8 | `xgen-core/src/migration/state_machine.rs:233` — 📌 **path corrected: §5d writes a bare `state_machine.rs:233`** |
| **7** | **`C-7` — `/// Active members`** | `M_SPACE_ADMISSION_E0_PHASE0.md` §8 (*rides `(g)`*) | `state.rs:232` — unchanged |
| **8** | **`F-E` — `build_membership_event` emits `prev_events: vec![]`** | `M_SPACE_ADMISSION_E0_PHASE0.md:211`, `:230` — **folded here as a NAMED PRECONDITION, which is what §9b retook it to do** | `state.rs:2131` |
| **9** | **The A-bis inverted test's RENAME** — its name asserts the opposite of what it tests | J-763; `ROADMAP:426`; carried here by Joe 2026-08-23 | `xgen-node/src/tests/space_admission_third_party_join.rs:115` |
| **10** | **§8's convergence argument + §15.7's surviving anchor note** | `M_SPACE_ADMISSION_PHASE0.md` §8, §15.7 | 📌 **the ANCHORING half is `Leg G`'s.** §5 splits the DELIVERY half out and keeps it here |

✅ **All ten are present and none was missing.** The sweep's value was elsewhere — it produced three findings the inventory did not contain (§6).

### §2b — WHAT THE SWEEP ALSO CONFIRMS

📌 **`M-SPACE-ADMISSION` is ONE ROADMAP node** (`ROADMAP:368`) carrying per-leg `↳ Owes:` lines — legs are **not** separate tree nodes in this milestone. ⇒ **Leg E needs no new node**, and `M-RP-MEMBER-ACT` Leg E's missing-node defect (J-718) has no sibling here. *Stated because its absence is the kind of thing that gets rediscovered as a finding.*

---

## §3 — THE CENSUS RE-ANCHORED AT `2965e08`. **50/50 HOLDS; ONLY THE LINES MOVED.**

🔑 **`E-0` measured at `5da9e53`. Three commits later every applier citation has drifted.** The classification is `E-0`'s and is not re-litigated; the **anchors** are re-measured here because Leg E edits at these lines and `D-152` binds.

**Method:** case-sensitive `git grep` for `is_member(` and `member_role(` callers, plus a **line-joined** scan for `.members` on the four files holding rustfmt-broken chains (`F-3`'s species — a line-oriented sweep cannot see `ops.rs:2573`).

### §3a — DOOR `D-1` (`is_member`), 13/13 production

`ai_service.rs:522` · `exchange.rs:232, 370, 375, 397, 693` · **`runtime.rs:1717`** (was `:1665`) · `dm_promotion.rs:72, 109, 148` · `admin_ops.rs:1077, 4191` · `fanout.rs:488`

📌 `encryption/group.rs`'s `is_member` is **MLS's own member set**, not this map — excluded by `E-0` §3 and excluded here.

### §3b — DOOR `D-2` (`member_role`), 17/17 production

`exchange.rs:844, 876, 888, 898, 908, 923, 958` · `algorithm.rs:221` · `state.rs:906, 928, 952, 1124, 1214, 1239, 1347, 1375, 1976`

### §3c — DOOR `D-3` (direct), 20/20 production — **THE DELTA TABLE**

| what | `E-0` @ `5da9e53` | **HEAD `2965e08`** |
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

🔑 **`C-4` IS DM-ONLY.** The sibling `repopulate_regular_federation_nodes` (`runtime.rs:2333`) sources `federation_nodes` from the **relationships map**, not from `members` (`:2338` returns early for DM Spaces; `:2345` reads `relationships.get(...)`). ⇒ **a regular Space's federation set never contained a departed member's node**, and `C-4`'s fix is one filter in one helper, not two.

---

## §4 — THE BUILDABILITY PROOF. **LESSON ② FROM LEG D, DISCHARGED RATHER THAN ASSERTED.**

🛑 **Leg D died on this exact shape:** its §5 specified an edit in terms of `left_at` while its own §7 forbade creating it, and `V-3c` — *"the control that matters"* — described a state the code could not reach. **Leg E creates `left_at`, so every deferred clause becomes buildable at one moment.** This section proves it site by site; it does not assume it.

| deferred clause | what it needs | reachable once `left_at` exists? |
|---|---|---|
| **`D-154`① rejoin** | a write path that clears `left_at`, re-stamps `joined_at`, re-derives role | ✅ `apply_join:1183` is today a blind `HashMap::insert` — **`insert` REPLACES**, which is `D-154`'s founding finding. The rejoin branch replaces it |
| **`D-3` / `D-154`②③** | `state.rs:1173` gates on `left_at.is_none()`; `:1176`'s ban check becomes reachable for retained banned members | ✅ **and `V-3c` becomes runnable** — `apply_ban:1250` retaining a marked member is the state the control needs, and it is exactly what ③ creates |
| **`D-154`④ the gap** | a per-member boundary in the history slice | ✅ `left_at` + the re-stamped `joined_at` **are** the boundary. See §5 |
| **`D-154`⑤ rooms** | nothing new — `apply_leave:1207-1209` already strips room membership and `apply_join`'s Space branch never restores it | ✅ **already true; the clause is a NON-EDIT.** 📌 Recorded so nobody writes code to satisfy a clause the code already satisfies |
| **`C-3`** | `is_member` at `runtime.rs:1717` answering `false` for a departed member | ✅ **discharged by `(i)` alone** — see §5 |
| **`C-4`** | `left_at.is_none()` in `runtime.rs:2312`'s loop | ✅ one filter |
| **`C-5`** | `:272` filtered directly; `:488` via `is_member` | ✅ |
| **`C-7`** | a doc comment | ✅ |

⚠️ **`F-E` is the one item that is NOT unblocked by `left_at` and is a PRECONDITION on the work instead.** `build_membership_event` (`state.rs:2131`) emitting `prev_events: vec![]` is invisible to unit tests calling `apply_event` and is a DAG violation on the **node ingest path** — which is precisely the path Leg E's rejoin fixtures exercise. **It has already cost two runs.** ⇒ **named in E-1's preconditions, not filed as a rider.**

---

## §5 — 🛑 THE FINDING: `C-3` IS NOT MECHANICAL. UNDER `(i)` IT SELF-INVERTS INTO CLAUSE ④'s ENFORCEMENT SITE.

`C-3` has been carried for two legs labelled *"mechanical"*. `D-153` says a finding from another seat is not re-driven until its citation is opened. **The citation was opened.**

### §5a — THE FIRST HALF DISCHARGES ITSELF

`runtime.rs:1713-1726` computes `already_member` from **`is_member`** — a **`D-1`** door, and `(i)` gates `is_member` on `left_at.is_none()`.

⇒ a rejoiner reads `already_member = false` ⇒ `new_joiner = Some(sender)` ⇒ **the push fires.** 🔑 ***`C-3`'s silent-empty-room failure is closed by the accessor ruling and needs no edit of its own.*** 📌 The compounding with `D-3` that `E-0` warned of **does not materialise**, because both sides move together in this leg rather than one at a time.

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

🔒 **`fanout.rs:262-289` IS WHERE CLAUSE ④ LIVES, AND IT IS DELIVERY, NOT ANCHORING.** Leg G's `get_rejoin_anchor` decides what a rejoin **hangs off** (`prev_events`, convergence). This decides what she is **sent**. **Two mechanisms, one clause; only the second is in Leg E** — and it is in Leg E because Leg E is the leg that creates the `left_at` / re-stamped `joined_at` pair the slice needs.

⚠️ **THE HONEST BOUND:** the slice is expressible from `SpaceMember` alone **only if the departure timestamp is retained**. Clause ① clears `left_at` on rejoin. ⇒ ***a single `Option<String>` cannot hold both "she left at T1" and "she is back since T2" after the rejoin has been applied.*** The boundary must survive the write that clears it. **This is a data-model consequence of clause ① that `D-154` does not name, it is Chat's to solve, and E-1 owns it** — three candidate shapes (a retained departure list on the member record · a per-member watermark written at rejoin · deriving the boundary from the log's own `membership.*` events at slice time) are priced in E-1's design, not here.

---

## §6 — NEW FINDINGS FROM THIS SWEEP

### §6a — 🔓 `N-1` — A FIFTH REMOVAL SITE `D-154` DOES NOT NAME. **JOE'S.**

`E-0`'s `C-1` counted **four** `self.members.remove` sites. Re-measured at `2965e08`, all four are present and the fourth has a name:

| site | path |
|---|---|
| `apply_leave` | `state.rs:1203` |
| `apply_kick` | `state.rs:1230` |
| `apply_ban` | `state.rs:1250` |
| **`apply_node_eject`** | **`state.rs:1275`** |

`D-154` rules **leave ①**, **kick ②**, **ban ③**. 🛑 **`membership.node_eject` — the Node-administrator force-eject (M6 A4-D1), authority `sender == home_node`, reversible via `membership.node_unban` — is unruled.** If it removes while the other three retain, ***"in `members`" means two things depending on how you left*** — the precise ambiguity clause ② was ruled to prevent, at a site clause ② does not reach. **→ §8.**

### §6b — 📌 `N-2` — `§12`'s LEG TABLE ISSUES `G` TWICE, AND ONE OF THE ROWS MISCITES `D-154`. **CHAT'S. RECORDS.**

`M_SPACE_ADMISSION_PHASE0.md:348` and `:349` both carry the designation **`G`** — the older *"THE REJOIN ANCHOR"* row (gated on Leg E) and the newer *"THE REJOIN ANCHOR VERB — ITS OWN LEG"* row (Joe, 2026-08-18, J-756). **`D-134`: designations are issued unique; the lettered split is a repair applied at revision.**

🛑 **And `:348` cites `D-154`③ for *the gap stays closed*, which is clause ④.** ③ is *ban follows kick*. `DECISIONS.md:5787`'s table is the authority and `CLAUDE.md:157` states ④ correctly. ⇒ **a reader who follows the citation lands on the wrong clause.** Repaired by annotation at the site (`D-131`), not by silent swap; the superseded row is struck, not deleted.

### §6c — 📌 `N-3` — EVERY INHERITED APPLIER CITATION IS STALE AT HEAD. **CHAT'S.**

§3c is the table. The shape worth keeping: **`state.rs` gained ~61 lines between `5da9e53` and `2965e08`**, so every applier citation written in `E-0`, in `D-154`, in the Leg D documents and in `§15.5` now points ~61 lines short. **`D-154`'s own *"`state.rs:1122`"* for the replacing `insert` is `:1183`.** ⇒ 🔒 **a `file:line` written into a document without its tree is a citation with a half-life measured in commits**, which is `D-152` clause 1 stated as a cost rather than a rule.

### §6d — 🛑 `N-200` — A FAILED `[System.IO.File]` READ LEAVES THE PREVIOUS FILE'S CONTENT IN THE VARIABLE, AND THE LOOP PRINTS IT UNDER THE NEW FILE'S HEADING.

**Hit live this session.** A four-file sweep loop assigned `$lines = [System.IO.File]::ReadAllLines($abs)`; the fourth path (`xgen-core\src\space\state_machine.rs`) **does not exist** — the file is at `xgen-core/src/migration/state_machine.rs`. The assignment threw, **`$lines` retained the third file's content**, and the loop printed `runtime.rs`'s eight hits under a `state_machine.rs` heading — **with an identical line count (6229) as the only tell.**

🔑 ***`N-197`'s species again: the failure mode reads exactly like success.*** It is the sibling of `E-0`'s instrument failure ② — *an absent path and an absent match are indistinguishable* — except worse, because here the absent path produced **plausible non-empty output attributed to the wrong file.**

🔒 **RULE: a file read inside a loop asserts the path exists BEFORE the read (`if(-not (Test-Path $abs)){ throw }`), and the loop reports the file's own line count with every batch.** 📌 Caught because the two headings' line counts matched exactly; **without that coincidence the wrong data would have entered §3c.**

---

## §7 — 🔓 THE PROPOSED LEG SPLIT. **CHAT'S SEAT (`D-123`); THE SPLIT IS JOE'S TO LOCK.**

| leg | content | gated on |
|---|---|---|
| **E-1** | **THE FIELD AND THE WRITES.** `left_at: Option<String>` on `SpaceMember` (`state.rs:85-95`) · `apply_leave:1203` stops removing · **`apply_join:1183`'s blind `insert` becomes a rejoin branch** (clears `left_at`, re-stamps `joined_at`, **re-derives role — a departed Owner returns as `Role::Member`**) · `:1173` gates on `left_at.is_none()` and `:1176`'s ban check becomes reachable ⇒ **`D-3` lands and `V-3c` becomes runnable** · **§5c's boundary-retention shape decided and built** · 🔒 **precondition: `F-E`** | `F-E` |
| **E-2** | **THE READERS.** `(i)`'s two accessors gate on `left_at.is_none()` and carry **30 sites free**; **`D-3`'s 20 hand-edited**, including `resolve_operator`'s five (**`D-4`** — no accessor ruling reaches it, and `CLAUDE.md:1470` documents it as *"transparently skips members who left"*, which goes false at a line nobody edits) · **`C-5`** `fanout.rs:272` · **`C-4`** `runtime.rs:2312` | E-1 |
| **E-3** | **CLAUSE ④ — THE GAP.** The history slice at `fanout.rs:276-289` (§5). ⚠️ **The gap's MARKER — that something was said while she was away — is appearance and stays Joe's** (`D-154`'s named non-settlement); E-3 builds the boundary and leaves the marker's wording and form open rather than inventing it | E-1, E-2 |
| **E-4** | **RECORDS AND THE CARRIED ITEMS.** The A-bis test **rename** · **`C-7`** · **`N-2`**'s two repairs · ch3's membership-lifecycle text · JOURNAL + CLAUDE.md + ROADMAP + task docs in **one atomic commit** (`D-074`) | all |

📌 **`C-6` is NOT in this split** — already filed non-blocking, belongs to whoever writes `transport.redirect`, and `(g)` promoting a wrong log-field number from *middle set* to *widest set* does not make it Leg E's. **Carried forward unchanged, not absorbed.**

📌 **`Leg G` is unchanged and still its own leg** — the anchoring half of clause ④, a new wire verb, Joe's seat.

---

## §8 — 🔓 THE ONE OPEN QUESTION. **JOE'S** (`D-155`).

**Someone is thrown out of a Space by the Node operator** — not by a member with kick rights, and not by their own choice. The Space is being cleaned up, or the operator is acting on an abuse report.

### **What should the Space remember about that person afterwards?**

| | outcome, as a person would see it |
|---|---|
| **(a)** | **The same as a kick.** She stays in the record, marked as gone. If the ejection is ever reversed she comes back as an ordinary member. Anyone auditing the Space later can see she was once here, and when she left. |
| **(b)** | **Nothing. She is gone.** The record is wiped, as it is today. An auditor sees a Space she was never in. |

🎯 **CHAT RECOMMENDS (a).**

**① USER-VISIBLE IMPACT.** Under **(b)** the Space that ejected her **forgets that it did** — while the messages she wrote stay, signed by her, in the retained log. Someone reading that history later sees authored events from a person the Space has no record of ever admitting. **(a)** is the only answer in which the history and the membership record agree. Under **(a)** an operator or auditor opening the roster sees a departed entry where **(b)** shows a silent absence.

**② TIER CONSEQUENCE.** This is the same `T4` argument `D-154` already accepted: `D-093` retains the **bytes** and does not retain **who was in the room**. **(b)** reopens that hole for the single case most likely to be audited — an operator-forced removal. **(a)** closes it identically to clause ②.

**③ RESOURCE COST.** **(a)** is one line, structurally identical to clause ②'s. **(b)** is free in build time and expensive in meaning: it leaves `members` meaning two things depending on how you left, which is the cost clause ② was ruled to avoid.

⚠️ **THE HONEST CAVEAT, NAMED AND NOT TRADED AWAY.** **(a)** makes an ejection a durable record on a federated, replicated object. `membership.node_eject` is **reversible** (`membership.node_unban`), and under **(a)** a reversed ejection still leaves the record saying it happened. 📌 `D-154`'s own filed note applies here and is still unlooked-at: **`self.banned` is already exactly this shape** — a permanent federated list of identities — and has been for a long time. **That look is not this question and must not be smuggled into it.**

---

## §9 — WHAT LEG E MUST NOT DO

1. **It does not build `get_rejoin_anchor`.** That is `Leg G`, a new wire verb, Joe's seat (`M_SPACE_ADMISSION_PHASE0.md:349`).
2. **It does not invent the gap's marker.** `D-154` names the marker as appearance and leaves it with Joe. E-3 builds the boundary and stops.
3. **It does not fix `C-6`.** Filed, non-blocking, another owner.
4. **It does not touch `ui/**` or `xgen-client`'s appearance surfaces.** The roster's rendering of a departed member is appearance; `ops.rs`'s projection sites are data.
5. **It does not re-litigate `(g)`, `(i)`, `Q-2`(a) or any `D-154` clause.** They are locked. §8 asks about a site **no clause reaches**.
6. **It does not re-run the census.** `E-0` is `COMPLETED`; §3 re-anchors, it does not re-classify.

---

## §10 — DoD

- [ ] §8 ruled by Joe and written into this file
- [ ] §5c's boundary-retention shape decided (Chat) and priced in E-1's design
- [ ] E-1 … E-4 runbooked, each locked by Joe before Clair opens it
- [ ] Every gate re-driven by Chat from `HEAD` (Rule 5), none adopted on report
- [ ] `V-3c` **RUN** — the control Leg D could not reach: with `D-3` absent, a retained banned member is refused `AlreadyMember` instead of `Banned`
- [ ] **`N-199` observed on every restore:** restore → stamp mtime → **require `Compiling <crate>` in the log**
- [ ] `F-E` discharged before E-1's fixtures run
- [ ] `N-2`'s duplicate `G` and miscited clause repaired by annotation (`D-131`)
- [ ] The A-bis test renamed to assert what it tests
- [ ] `roadmap-format-gate.ps1` exit 0 before any ROADMAP commit
- [ ] `D-074` atomic close: code + JOURNAL + CLAUDE.md + ROADMAP + task docs, one commit

📌 **"Commit pushed" is not a DoD item** — `Status: COMPLETED` is the signal.

---

## §11 — FLOORS

**Carried into Leg E from `2965e08`, and the cargo figure is MEASURED not inherited:**

| instrument | floor | unit |
|---|---|---|
| cargo | **1623 / 0 / 62** | **× 56 SUITES** |
| vitest | **172 / 172** | **× 9 FILES** |
| svelte-check | **0 / 34 / 15** | — |
| catalogue | 🛑 **UNMEASURED** | — |

🛑 **Never cite a floor without its unit.** 🛑 **Do not write `435` for the catalogue** — it is a carried number nobody has re-measured.

⚠️ **E-1 and E-2 both move the cargo floor.** The delta is a **measurement**, taken with `--skip` on this tree, never arithmetic against a carried number (the `A-bis` method, J-755).
