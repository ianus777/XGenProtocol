# Clair — adversarial cold read of `tasks/M_SPACE_ADMISSION_E0_PHASE0.md` v1.1
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Aug 2026  
> **Last updated**: 2026-08-21  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — VERDICT

✅ **GO WITH FINDINGS.** Six blocking, six riding as notes.

🔑 **THE DOCUMENT'S ARITHMETIC IS SOUND AND I COULD NOT BREAK IT.** Every figure in §3 reproduces **exactly** on an independent run — `68 / 12 / 56`, `30 / 17 / 13`, `65 / 14 / 51`, the bare-`is_member(` **71**, and all three function-NAME hits by line. §5b's **17 site citations are individually exact**, and the definition at `state.rs:1373` is correctly excluded from them. 📌 **I checked the two accessor definition line numbers against my own first read, disagreed, re-measured, and the document was right and I was wrong** — `member_role` **1373**, `is_member` **1380**.

🛑 **THE DEFECTS ARE NOT IN THE COUNTING. THEY ARE IN THE DOOR *SET* (`F-1`), THE PRODUCTION/TEST *CONVENTION* (`F-2`), THE `D-3` *INSTRUMENT* (`F-3`, `F-4`), AND THE *REOPEN TRIGGER* (`F-6`).** Plus **`F-5`**, a single site whose breakage defeats `(g)`'s stated purpose and which the ruled option **(i)** does not reach.

📌 **METHOD, STATED (`D-152` clause 1): every citation below is measured at `5da9e53` (= `origin/main` by `ls-remote`), tree clean.** ⚠️ **`5da9e53` is ONE COMMIT AHEAD of the document's stated `bf7f297`** — it is J-761, the commit that added the Phase-0 itself. **I verified that no `.rs` file differs between the two trees**, so every `bf7f297` citation in the document holds at the tree I measured; they differ in documents only. 📌 **The kickoff's warning that the file may be untracked is stale — it is tracked.**

🔒 **§1 below was derived from source and written down BEFORE §3 was opened**, per the brief.

---

## §1 — MY INDEPENDENT DOOR SET, DERIVED BEFORE READING §3

**Doors — routes by which production code observes `SpaceState.members` (`state.rs:233`, a `pub` field):**

| | door | evidence |
|---|---|---|
| **D-1** | `SpaceState::is_member` | `state.rs:1380` — `members.contains_key` |
| **D-2** | `SpaceState::member_role` | `state.rs:1373` — `members.get(…).map(role)` |
| **D-3** | direct `.members` field access | the field is `pub` |
| 🛑 **D-4** | **`SpaceState::resolve_operator`** | `state.rs:1342` — **`pub fn`, reads `self.members` FIVE times** (`:1346 :1351 :1356 :1358 :1365`) |

**Non-doors a naive sweep conflates — each excluded for a stated reason:**

- **`RoomState.members`** — a **different field on a different struct** (`state.rs:130`, `HashSet<IdentityXgid>`), reached by `is_room_member` (`state.rs:1386`) and by `room.members` direct. ⚠️ **`apply_leave` removes from BOTH** — see `F-12`.
- **MLS `.members`** — `encryption/group.rs` and `encryption/client_mls.rs` **each define their OWN `is_member`** over a `HashSet<String>` (`group.rs:63`, `client_mls.rs:137`). **The whole `.members` set in both files is MLS**, not merely the two definition lines.
- **`args.members` / `let members: usize` / `0..members`** — an **integer CLI argument** in `xgen-client/src/app.rs` (≈19 of that file's 22 `.members` hits; e.g. `:3742 .clamp(2, 50)`).
- **`MembersResult.members` / `projected.members`** — a `Vec<MemberEntry>` **projection**, not a `SpaceState` (`client/app.rs:3264`, `client/ops.rs:2819`).
- **Comments** — `fanout.rs:261`, `client/app.rs:12`.

**Routes PROVEN CLOSED rather than assumed absent:**

- 🔒 **serde — STRUCTURALLY IMPOSSIBLE.** `SpaceState` derives **`Debug, Clone, PartialEq, Eq` and nothing else** (`state.rs:197`). There is **no `Serialize` / `Deserialize`** ⇒ the field cannot be reached by serialisation. *This closes the kickoff's serde question by construction, not by an absence of hits.*
- ✅ **destructuring — none.** Every `SpaceState {` hit repo-wide is **struct-literal CONSTRUCTION** (`state.rs:353 :485 :602`, plus test fixtures); **no `SpaceState { members, .. }` pattern binds anywhere.**
- ✅ **`can_*` permission helpers are NOT doors.** All nine (`membership.rs:128-182`) take a **resolved `&Role`** — strictly downstream of `D-2`.
- ✅ **no `Deref`, no wrapper re-export.**

---

## §2 — BLOCKING FINDINGS

### 🛑 `F-1` — §3's PARTITION IS MISSING A FOURTH DOOR, AND IT IS THE ONE OPTION **(i)** CANNOT FIX

**`SpaceState::resolve_operator` (`state.rs:1342`) is a `pub` method that reads `self.members` five times.** Its **production call sites are invisible to all three doors**:

- `xgen-client/src/ai_service.rs:526` — `if state.resolve_operator(ai_identity_id).is_some()`
- `xgen-client/src/ops.rs:2564` — `let resolved = state.resolve_operator(&args.ai);`

🔑 **AND THE SEMANTIC IT BREAKS IS ALREADY NAMED IN THE PROJECT RECORD.** `CLAUDE.md` describes the M3 fall-upward algorithm as ***"transparently skips members who left"***. Under `(g)` **it stops skipping** — `contains_key` returns true for a leaver ⇒ **a departed delegate, or a departed inviter, is returned as the live operator.**

🛑 **THREE EXISTING TESTS ASSERT THAT BEHAVIOUR BY NAME:** `state.rs:3990 resolve_operator_skips_delegate_who_left_falls_back_to_inviter` · `:4011 resolve_operator_falls_to_owner_when_inviter_gone` · `:4024 resolve_operator_returns_none_for_non_member`.

🔑 **THE LOAD-BEARING PART: `resolve_operator` USES NEITHER ACCESSOR.** All five reads are direct `self.members.contains_key` / `.get` ⇒ **`D-3`** ⇒ ***gating `is_member` / `member_role` under option (i) leaves this door fully broken.*** §5's claim that (i) makes *"the DEFAULT answer the SAFE one"* is **true for `D-1`/`D-2` and false for the densest `CURRENTLY` cluster in the codebase.**

📌 **The document reasons at the CALLER level in §2 (`can_change_admission`) and at the READ level in §3. It cannot be both.** For a census titled *"what breaks"*, the two `resolve_operator` callers are observers that break, and no row of §3 contains them.

**⇒ CHANGES:** §3 gains **D-4** with its two production call sites; §5's cost table must state that **(i) does not reach `D-3`**.

---

### 🛑 `F-2` — §3's PRODUCTION/TEST CONVENTION IS WRONG FOR `admin_ops.rs`, AND IT MISFILES TWO PRODUCTION SITES AS TEST

§3: *"PRODUCTION/TEST is decided by the file's **first** `#[cfg(test)]` line."*

🛑 **`xgen-node/src/admin_ops.rs` HAS PRODUCTION CODE AFTER ITS FIRST `#[cfg(test)]`.** Measured:

- `:3281` — `#[cfg(test)] mod bootstrap_verb_tests {` … **closes at `:3394`**
- **`:3395–:4515` — PRODUCTION** (`pub struct SpaceListHostedArgs` `:3411`, `pub async fn space_list_hosted` `:3437`, `NodeSetPolicyArgs` `:3511`, …)
- `:4516` — `#[cfg(test)] mod tests {` → EOF

**Two door sites fall in the misfiled window, and both are `CURRENTLY`:**

| site | door | what it is | break under `(g)` |
|---|---|---|---|
| `admin_ops.rs:3460` | **D-3** | `member_count: s.members.len()` in `space_list_hosted` | **an operator's hosted-Space listing reports inflated membership** |
| `admin_ops.rs:4191` | **D-1** | `if !space.is_member(args.identity_id)` — a removal precondition | **you can "remove" somebody who already left** |

✅ **BOUNDED, NOT ASSUMED — I RAN THE SAME TEST ON EVERY OTHER FILE CARRYING DOOR HITS, AND `admin_ops.rs` IS THE ONLY ONE.** (`runtime.rs`, `state.rs`, `ops.rs`, `exchange.rs`, `fanout.rs`, `ai_service.rs`, `derive.rs`, `algorithm.rs`, `dm_promotion.rs`, `state_machine.rs`, `client/app.rs` — all zero column-0 production items after their first marker.)

**⇒ CHANGES:** the convention becomes *"inside a `#[cfg(test)]` **module span**"*, not *"after the first marker"*; **D-1 production 12 → 13, D-3 production 14 → 15.**

---

### 🛑 `F-3` — `D-3`'s INSTRUMENT CANNOT SEE A MULTI-LINE METHOD CHAIN; **SEVEN** PRODUCTION SITES ARE MISSING

`.members` at end-of-line with the method on the **next** line is invisible to a single-line `.members.<method>(` sweep. **Seven production sites, all genuine `SpaceState`:**

| site | chain | why it matters under `(g)` |
|---|---|---|
| **`space/dm_promotion.rs:80`** | `.members` → `.keys().find(…)` for *"the other member"* | 🛑 **the comment says *"DM Space always has exactly 2 members"* — under `(g)` it can hold 3, and `find` over a `HashMap` is ITERATION-ORDER DEPENDENT ⇒ `deliver_to` becomes NON-DETERMINISTIC.** A convergence hazard, not merely a wrong recipient |
| `space/dm_promotion.rs:130` | `.members` → `.keys()` → `ConfirmResult.deliver_to` | promotion delivered to a departed party |
| **`migration/state_machine.rs:233`** | `.members` → `.keys()` → `CutoverResult.member_ids` | **Space migration carries leavers to the destination Node** — and this is a genuine `CURRENTLY`-vs-`EVER` question the 43 never posed |
| **`client/ops.rs:2736`** | `.members` → `.values()` → `Vec<MemberEntry>` | 🛑 **this feeds the client members roster** ⇒ **the R7 members panel lists departed people** — the most directly user-visible break in the census |
| `client/ops.rs:2573` | `.members` → `.get(args.ai)` | AI-status projection |
| `client/ops.rs:2591` | `.members` → `.get(args.ai)` → `ai_member_role` | reports a departed AI's role |
| `client/ops.rs:2595` | `.members` → `.get(args.ai)` → `ai_invited_by` | ditto |

📌 `state.rs:1375` and `:1382` are also multi-line chains, but they are **the accessor DEFINITIONS** — correctly not call sites.

**⇒ CHANGES:** **D-3 production +7**, and §3's convention must state that the sweep is line-oriented and therefore blind to chains.

---

### 🛑 `F-4` — `D-3`'s PRODUCTION **14** CONTAINS TWO SITES THAT ARE NOT `SpaceState.members`

- **`xgen-client/src/app.rs:3264`** — `r.members.len()` where `let r = crate::ops::members(…)` ⇒ **a `MembersResult`, not a `SpaceState`.** A false positive.
- **`xgen-core/src/encryption/group.rs:68`** — `self.members.len()` on **`MlsGroup`**. 🛑 **§3 declares MLS out of scope for `D-1` and then silently counts an MLS site in `D-3`.**

🔑 ***The paragraph warning that "counting `.is_member(` and calling it the census would have inflated D-1 by the MLS sites" is the paragraph under which an MLS site entered D-3.***

**⇒ CHANGES:** **D-3 production −2.**

**NET, ALL FOUR APPLIED:** **D-1 13 · D-2 17 · D-3 20 = 50 production reads** (not 43), **plus D-4's 2 observers**. **Outstanding after §5b: 33, not 26.**

---

### 🛑 `F-5` — `state.rs:1112` DEFEATS `(g)`'s ENTIRE PURPOSE, AND OPTION **(i)** DOES NOT REACH IT

`apply_join`, Space-level branch:

> `if self.members.contains_key(joiner) { return Err(SpaceError::AlreadyMember); }`

🛑 **Under `(g)` a leaver is still in `members` ⇒ A REJOIN IS REJECTED AS `AlreadyMember`.** `Q-2` ruled **(a)**: *a former member is re-admitted **without** an invite*. **This line makes that impossible.**

🔑 **AND IT IS `D-3`** — a bare `contains_key` ⇒ **gating the accessors under (i) changes nothing here.** The same applies to `state.rs:1100`, the room-level branch's *"must be a Space member to join a room"* guard, where under `(g)` a leaver could join a room.

📌 **Corroboration from the suite rather than from argument:** `derive.rs:471` is named `convergence_mp_f7_rejoin_anchored_after_leave_is_member` — rejoin-after-leave is already an asserted convergence property.

📌 **This also settles §2's re-sequencing claim in §2's favour** — `apply_join` is what Leg D's gate runs through, so **`E-0` does gate Leg D.** See `F-9` for the fact that no DoD item lands it.

**⇒ CHANGES:** §5's table must show that **(i) is not sufficient**; `D-3` needs a disposition of its own.

---

### 🛑 `F-6` — §5's REOPEN TRIGGER IS ANCHORED TO A DEAD DENOMINATOR AND IS NEARLY UNREACHABLE

§5 recommends (i) **conditionally**: *"if EVER turns out to **dominate** the 43…"*. The recorded trigger reads: *"UNLESS the completed classification returns EVER as the **majority** of the 43."*

🛑 **Three problems, compounding:**

1. **`dominate` → `majority` is a strict tightening**, applied when the ruling was *recorded* rather than when it was *argued*.
2. **It is anchored to `43`, a number `F-2`/`F-3`/`F-4` move to `50`.** A trigger anchored to a superseded denominator cannot be evaluated at all.
3. 🔑 **IT IS NEARLY UNREACHABLE BY CONSTRUCTION.** §5b landed **`D-2` at 17/17 `CURRENTLY`, `EVER` 0**. For a majority of 50, `EVER` must reach **26 of the remaining 33 — 79% of everything not yet classified.** ⇒ ***the trigger could not fire even if nearly every remaining `D-3` site were historical.***

📌 **This is not an argument against (i).** It is that **the ruling now has no live falsifier**, and §5 recorded the trigger precisely *because* the ruling preceded its own evidence.

**⇒ CHANGES:** re-state the trigger against the corrected denominator and in a form that can actually fire. The natural one is *"`EVER` is non-trivial within `D-3`"*, since §5b already names `D-3` as where `EVER` would live if it lives anywhere.

---

## §3 — FINDINGS THAT RIDE AS NOTES

### 📌 `F-7` — §4's TAG CONSEQUENCE COLUMN PRESUMES AN ACCESSOR EXISTS

`CURRENTLY` ⇒ *"**BREAKS** unless the **door** gates on `left_at.is_none()`"*. **True for `D-1`/`D-2`. False for `D-3`, where there is no door to gate** — each site needs its own edit. The scheme therefore **cannot express *"CURRENTLY, and not auto-fixable by (i)"***, which is exactly where `F-1` and `F-5` live. **The three tags remain a partition of MEANING; they are not a partition of REMEDY, and the column conflates the two.**

### 📌 `F-8` — §7 CARRIES A DoD ITEM THAT CANNOT BE SATISFIED

*"The **EVER** count reported to Joe **before** §5 is ruled"* — §5 itself records that this did not happen. **An unsatisfiable box stays unticked forever and trains the reader to skip the list** — §4's own `N-197` argument, applied to the DoD. Mark it **discharged-by-exception**, with the reopen trigger named as its replacement.

### 📌 `F-9` — §2 ASSERTS A RE-SEQUENCING THAT NO DoD ITEM LANDS

§2: *"it **re-sequences** the milestone: `E-0` gates **LEG D**, not just Leg E."* ✅ **The claim is correct** (`F-5`). 🛑 **But `M_SPACE_ADMISSION_PHASE0.md:347` has Leg E depending on *"Legs A + D + E-0"*, and nothing anywhere has Leg D depending on `E-0`** — while §7's §12 item corrects **only the NAME**. ***A re-sequencing announced in prose and landed in no record is not a re-sequencing.***

### 📌 `F-10` — `D-3`'s STATED ENUMERATION IS NOT THE ONE THAT PRODUCED THE COUNT

Prose lists `{get, iter, keys, values, len, contains_key, is_empty}`. Measured in scope: `contains_key 31 · len 22 · get 6 · iter 3 · keys 2` = **64**; **`values` and `is_empty` have ZERO occurrences**, and **65 reconciles only if `.get_mut` is also counted** (`state.rs:4017`, a test site). ⇒ **the convention paragraph does not describe the instrument.** Costs **0** production sites — a `D-152` clause-2 shape rather than a miscount.

### 📌 `F-11` — `D-1`'s **TOTAL 68** INCLUDES THE FOUR MLS SITES THE SAME PARAGRAPH DECLARES OUT OF SCOPE

`group.rs:117 :118 :130 :131`. The PRODUCTION figure is unaffected (all four are test), but **TOTAL and the out-of-scope declaration contradict each other inside one paragraph.**

### 📌 `F-12` — `apply_leave` TOUCHES **TWO** MEMBER SETS, AND `(g)` IS SPECIFIED FOR ONLY ONE

`apply_leave` (`state.rs:1134`): `:1139` room-only early return · **`:1142` removes from `SpaceState.members`** · **`:1145-1146` removes from EVERY `RoomState.members`**. 🛑 **`(g)` as ruled sets `left_at` on the `SpaceMember` and says nothing about room membership** ⇒ a leaver becomes *"in the Space with `left_at`"* **and** *"in no room"*, and `is_room_member` (`state.rs:1386`) keeps answering `false`. **That may well be the right answer — but nothing has said so**, and Leg E meets it on line one. *Filed, not decided.*

---

## §4 — WHAT I VERIFIED AND FOUND SOUND — DO NOT RE-LITIGATE

- ✅ **§5b's 17 `D-2` sites — EVERY LINE NUMBER EXACT** (`exchange.rs:844 876 888 898 908 923 958`; `state.rs:845 867 891 1063 1153 1178 1286 1314 1915`; `algorithm.rs:221`), with the definition at `:1373` correctly excluded. **`EVER 0 / INDIFFERENT 0` upheld — I opened all 17 and could not move one.**
- ✅ **§2's `can_change_admission` claim — VERIFIED** at `exchange.rs:958` and `state.rs:845`. A departed Owner keeps `Role::Owner` ⇒ `can_change_admission` returns `true`.
- ✅ **The `71` reconciliation is exact**: raw `is_member(` **74**, minus **3** definitions = **71**, minus **68** call sites = **3 function NAMES**, and all three named lines are function names as claimed.
- ✅ **The MLS exclusion is sound and STRONGER than stated** — both files define their own `is_member` over a `HashSet<String>`, so the exclusion covers their entire `.members` set, not two lines.
- ✅ **`algorithm.rs:221` is conflict resolution, not permission, and its failure mode is silent and convergent.** Confirmed at source: `layer4_role_priority`, `role_of` = `member_role`, running inside the fold.
- 🔑 **AND THE DOCUMENT UNDERSTATES IT.** `max_role` is `.max()` over roles and the winner requires **`winners.len() == 1`**. Under `(g)` a departed member **tying** a live member at the same role makes `winners.len() == 2` ⇒ **`None`** ⇒ ***resolution falls through to Layer 5.*** So `(g)` can change **which layer decides**, not only who wins — a second silent effect, and it fires **without any departed Owner**.
- 🔑 **`state.rs:1915` is sharper than *"is this recipient a moderator now"*:** under `(g)` **a departed moderator keeps receiving `xgen.member_temperature`** — a **privacy leak to somebody who left**, not merely a stale role read.
- ✅ **No `.rs` file differs between `bf7f297` and `5da9e53`** ⇒ every `bf7f297` citation holds at the tree I measured.

---

## §5 — ONE THING OUTSIDE `E-0`'s SCOPE, FLAGGED NOT FIXED

🛑 **`SpaceState.admission`'s doc comment is STALE IN THREE CLAUSES, AND LEG C MADE IT SO.** `state.rs` still reads *"there is no mutation event, no applier arm and no `state_key_for_event` arm"* — **all three now exist**: `state_key.rs:111` (the `StateSpaceAdmission` arm), `state.rs:663` (`apply_space_admission`), and the event itself. Only *"Nothing reads this field until Leg D"* survives. **`N-109`'s shape, sitting in the field Leg E is about to edit.** Not `E-0`'s DoD; named so Leg E does not discover it.

---

## §6 — MY OWN INSTRUMENT FAILURES, BOTH CAUGHT

🛑 **① A METHOD CENSUS RAN AGAINST `target/` AND THE OUTPUT DID NOT SAY SO.** I used `grep -rhon`; **`-h` suppresses filenames, so the piped `grep -v "/target/"` had nothing to match and filtered NOTHING.** It returned a complete, plausible table summing to **303** occurrences. **Caught only because 303 exceeded the known 155-line total** — not because anything failed. Re-run without `-h`, the real distribution is `contains_key 31 · len 22 · insert 17 · remove 12 · contains 8 · get 6 · iter 3 · keys 2 · clamp 2 · get_mut 1`. ***A filter that cannot see its subject removes nothing and reports success*** — `N-197`, mine.

🛑 **② A BRACE-DEPTH SCANNER RETURNED SPANS FOR 8 OF 13 FILES AND NOTHING AT ALL FOR THE OTHER 5.** A silent partial failure whose empty result **reads exactly like "these files contain no test modules"** — which for `state.rs` and `ops.rs` would have been badly wrong. Discarded, replaced with a column-0-item heuristic, and cross-checked against `admin_ops.rs`'s known structure **before** `F-2` was written.

📌 **Both are the species `F-4` and `F-10` describe. They are reported rather than absorbed, because a read that hides its own instrument failures is not a control.**

---

## §7 — SUMMARY

| | finding | blocks? |
|---|---|---|
| `F-1` | fourth door `resolve_operator`; **(i) cannot fix it** | 🛑 |
| `F-2` | prod/test convention wrong for `admin_ops.rs`; 2 sites misfiled | 🛑 |
| `F-3` | multi-line chains invisible; **7** production sites missing | 🛑 |
| `F-4` | `D-3` production holds 1 non-`SpaceState` site + 1 MLS site | 🛑 |
| `F-5` | `state.rs:1112` blocks `(g)`'s purpose; **(i) does not reach it** | 🛑 |
| `F-6` | reopen trigger anchored to a dead denominator, nearly unreachable | 🛑 |
| `F-7` | tag consequence column presumes an accessor exists | 📌 |
| `F-8` | unsatisfiable DoD item | 📌 |
| `F-9` | §2's re-sequencing lands in no record | 📌 |
| `F-10` | stated `D-3` enumeration ≠ the one counted | 📌 |
| `F-11` | `D-1` TOTAL includes the MLS sites it excludes | 📌 |
| `F-12` | `apply_leave` touches two member sets; `(g)` specifies one | 📌 |

**Corrected production census: `D-1` 13 · `D-2` 17 · `D-3` 20 = 50, plus `D-4`'s 2 observers. Outstanding after §5b: 33.**

🛑 **I did not edit the Phase-0.** 🛑 **I wrote no product code.** 🛑 **I did not push.**
