# M-SPACE-ADMISSION Leg E-0 Phase-0 — the membership-reader census: what breaks when a leaver stays in the map
> **Status**: ACTIVE  
> Version: 1.1  
> Date: Aug 2026  
> **Last updated**: 2026-08-18  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §1 — 🔒 WHY THIS EXISTS. **FROZEN.**

`Q-2` ruled **(g)**: `apply_leave` stops removing the leaver from `SpaceState.members` and instead sets **`left_at`** on the retained `SpaceMember` (Phase-0 §15.5). `D-071` requires the subsystem audit to precede the dependent milestone.

🛑 **§12 CALLS THIS *"the `is_member` caller census"* AND THAT NAME IS TOO NARROW.** `(g)` does not change `is_member`. ***It changes `members`.*** Every reader of that map inherits the new semantics through whichever door it uses, and `is_member` is only one of three.

---

## §2 — 🛑 THE LIVE CONSEQUENCE THAT MADE THIS URGENT

**`can_change_admission` shipped in `bf7f297` (Leg C) and resolves its role through `member_role`.** Under `(g)`:

> a **departed Owner** remains in `members` with `left_at: Some(…)` ⇒ `member_role` still returns `Role::Owner` ⇒ `can_change_admission` still returns `true` ⇒ ***somebody who left a Space can still change its admission policy.***

🔑 **The same shape applies to every `can_*` in `membership.rs:126-163`**, all of which resolve a role through the same map.

⇒ **`(g)` IS NOT ADDITIVE.** ⚠️ **This is `D-151` clause 2 arriving exactly on schedule** — a value pinned in one place forecloses assumptions in code that never mentions it. The rule was minted against DM invites; this is the same rule against the permission table.

📌 **It also re-sequences the milestone: `E-0` gates LEG D, not just Leg E**, because Leg D's gate reads membership to decide admission and *former member* is the predicate `(g)` introduces.

---

## §3 — THE THREE DOORS, MEASURED AT `bf7f297`

| door | definition | TOTAL | PRODUCTION | TEST |
|---|---|---|---|---|
| **D-1** `SpaceState::is_member` | `state.rs:1380` — `members.contains_key` | **68** | **12** | 56 |
| **D-2** `SpaceState::member_role` | `state.rs:1373` — `members.get(…).map(role)` | **30** | **17** | 13 |
| **D-3** direct `.members.{get,iter,keys,values,len,contains_key,is_empty}` | — | **65** | **14** | 51 |

🛑 **CONVENTION, STATED SO THE NEXT COUNT IS COMPARABLE (`D-152` clause 1):** these are **call sites matching `.<door>(`**, at `bf7f297`, excluding `.claude/worktrees/` and `target/`. **A sweep for bare `is_member(` returns 71**; the extra three are **test-function NAMES** containing the substring (`exchange.rs:2193`, `:2277`, `derive.rs:471`), not calls. **PRODUCTION/TEST is decided by the file's first `#[cfg(test)]` line and by `\tests\` / `xgen-mptest` path membership.**

🛑 **AND `is_member` IS NOT ONE FUNCTION — THE CENSUS IS NOT A PARTITION UNTIL THE RECEIVER IS READ.** Three definitions exist: `SpaceState::is_member` (`state.rs:1380`), **`MlsGroup::is_member` (`encryption/group.rs:63`)** and **`client_mls.rs:137`**. The four `group.rs` hits are **MLS group membership and are OUT OF SCOPE**. ⚠️ ***Counting `.is_member(` and calling it the census would have inflated D-1 by the MLS sites*** — and an inflated count is the flavour that does not invite a second look.

📌 **The 22 hits whose receiver a regex could not name were READ, not guessed:** all 22 are `node.spaces[...]` / `rt.spaces.get(...).unwrap()` ⇒ `SpaceState`, and all 22 are inside test assertions.

---

## §4 — THE CLASSIFICATION. **THIS IS THE WORK.**

Every **production** site — **43** across the three doors — is tagged exactly one of:

| tag | meaning | consequence under `(g)` |
|---|---|---|
| **CURRENTLY** | means *is in this Space now* | 🛑 **BREAKS** unless the door gates on `left_at.is_none()` |
| **EVER** | means *was ever admitted* — history, audit, key material, DAG causality | ✅ **correct as-is**; `(g)` is what makes it expressible |
| **INDIFFERENT** | the leaver case cannot arise on this path | ✅ no change, **and the reason is recorded** |

🛑 **`INDIFFERENT` REQUIRES ITS REASON WRITTEN AT THE SITE.** *An untagged site and an indifferent site read identically in a diff*, which is `N-197`'s shape applied to an audit rather than to a test.

📌 **Test sites are NOT classified individually.** They are a **regression surface**: whichever way §5 rules, the suite tells us which assertions encoded the old semantics. **Their count is recorded so the post-change delta is checkable, not so each is argued.**

---

## §5 — 🔓 THE RULING. **JOE'S.**

**Does `member_role` (and `is_member`) gate on `left_at`, or does `(g)` get separate accessors?**

| | shape | cost |
|---|---|---|
| **(i)** 🎯 | **`is_member`/`member_role` gate on `left_at.is_none()`; add explicit `was_member` / `former_member_role`** | every **EVER** site must be found and moved — but ***the DEFAULT answer becomes the SAFE one, and the historical question has to be asked deliberately*** |
| **(ii)** | leave the accessors broad; every **CURRENTLY** site adds its own `left_at` check | no accessor churn, but **the unsafe answer stays the default** and every future caller re-derives the trap |
| **(iii)** | two maps — `members` and `former_members` | 🛑 **refused already by `§6.5`** on `D-067`: one fact in two places |

🎯 **Chat recommends (i)** — the count of **EVER** sites is what decides its real cost, **and §4's classification is what produces that count.** ⚠️ **The recommendation is therefore CONDITIONAL and stated as such: if EVER turns out to dominate the 43, (i) inverts from a safety win into churn, and (ii) becomes the honest answer.** *This document does not pretend to know that yet.*

🔒 **RULED (i) — Joe, 2026-08-18.** 🛑 **RECORDED WITH ITS REOPEN TRIGGER, BECAUSE THE RULING PRECEDED THE MEASUREMENT IT WAS CONDITIONED ON:** §7's DoD asked for the EVER count *before* §5 was ruled, and it was ruled first. ⇒ ***(i) stands UNLESS the completed classification returns EVER as the majority of the 43 production sites; if it does, it returns to Joe rather than being implemented through.*** 📌 **No work is lost either way — the tags are identical under (i) and (ii); only the mechanism differs.**

---

## §5b — ✅ CLASSIFICATION RESULT: DOOR **D-2** (`member_role`), 17/17 PRODUCTION SITES, MEASURED AT `bf7f297`

| sites | what they are | tag |
|---|---|---|
| `exchange.rs:844, 876, 888, 898, 908, 923, 958` — **7** | `check_permission`'s arms, one per privileged verb — **including Leg C's own `can_change_admission`** | **CURRENTLY** |
| `state.rs:867, 891, 1063, 1153, 1178, 1286, 1314` — **7** | `apply_mute` · `apply_room_create` · `apply_invite` · `apply_kick` · `apply_ban` · `apply_ai_operator_delegate` · `apply_ai_operator_revoke`, each `…ok_or(SpaceError::NotASpaceMember)?` | **CURRENTLY** |
| `state.rs:845` — **1** | the applier-side role read | **CURRENTLY** |
| `state.rs:1915` — **1** | `should_include_member_temperature` — is *this recipient* a moderator **now** | **CURRENTLY** |
| `algorithm.rs:221` — **1** | `layer4_role_priority` — *the sender with the higher Space role wins* | **CURRENTLY**, 🛑 and unlike the others |

🔒 **EVER: 0 · INDIFFERENT: 0** ⇒ ***every production `member_role` site breaks under `(g)` unless the accessor gates.***

🛑 **`algorithm.rs:221` IS NOT A PERMISSION CHECK — IT IS CONFLICT RESOLUTION, AND ITS FAILURE MODE IS SILENT AND CONVERGENT.** Under `(g)` a **departed Owner keeps winning Layer 4 tie-breaks against a sitting Admin, forever.** ⚠️ **It runs inside the FOLD**, so there is no reject, no log, and no reply that lied — ***every node simply agrees on a different resolved state than the design intends.*** 📌 **This is the site a permission-shaped audit would have skipped.**

📌 **`…ok_or(NotASpaceMember)?` DOES DOUBLE DUTY AT SEVEN SITES** — it answers *what role* and *are you here at all* in one expression. **Under `(g)` the second answer becomes wrong while the first stays right**, which is precisely why **(i)** fixes all seven at once and **(ii)** would need seven separate edits.

⚠️ **D-1 (12 production) and D-3 (14 production) ARE NOT YET CLASSIFIED — 26 sites outstanding, and the reopen trigger stays live until they are.** 🔑 **`D-3` is where EVER would live if it lives anywhere** — `.members.len()`, `.iter()`, key material, DAG causality.

---

## §6 — WHAT E-0 DOES **NOT** DO

1. **It changes no behaviour.** `apply_leave` still removes; `left_at` does not exist yet. **E-0 produces a classified census and a ruling, nothing else.**
2. **It does not touch MLS membership** (`group.rs`, `client_mls.rs`) — a different subsystem with a different meaning.
3. **It does not classify test sites individually** (§4).
4. **It does not fix the `can_change_admission` exposure** (§2) — that lands with `(g)` in Leg E, and §2 exists so Leg E does not discover it.

---

## §7 — DoD

- [ ] All **43 production sites** tagged **CURRENTLY / EVER / INDIFFERENT**, each with its reason, each cited **with its tree** (`D-152`)
- [ ] The **EVER** count reported to Joe **before** §5 is ruled — *the recommendation is conditional on it*
- [ ] §5 ruled; the ruling recorded at `§15.5`'s site in `M_SPACE_ADMISSION_PHASE0.md`
- [ ] Test-site counts recorded per door as the regression surface
- [ ] `§12`'s row for `E-0` corrected — **the name says `is_member`; the scope is `members`**
- [ ] Records: JOURNAL + `CLAUDE.md` + ROADMAP + `Status: COMPLETED`, one `D-074` commit

📌 **"Commit pushed" is not a DoD item.**
