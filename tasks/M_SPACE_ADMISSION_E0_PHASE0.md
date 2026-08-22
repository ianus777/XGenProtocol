# M-SPACE-ADMISSION Leg E-0 Phase-0 — the membership-reader census: what breaks when a leaver stays in the map
> **Status**: COMPLETED  
> Version: 1.2  
> Date: Aug 2026  
> **Last updated**: 2026-08-22  
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

## §3b — 🛑 THE §3 TABLE IS SUPERSEDED. RE-MEASURED AT `5da9e53`. **`D-131`: THE OLD NUMBERS STAY ABOVE.**

**§3's production figures were wrong in three independent ways, all found by Clair's adversarial cold read (`tasks/CLAIR_E0_PHASE0_READ.md`) and all re-driven from source by Chat (`D-153`).**

| door | §3 said | ✅ TRUTH at `5da9e53` | why it moved |
|---|---|---|---|
| **D-1** `is_member` | 12 | **13** | `F-2` |
| **D-2** `member_role` | 17 | **17** | unchanged |
| **D-3** direct `.members.*` | 14 | **20** | `F-2` +1 · `F-3` +7 · `F-4` −2 |
| | **43** | **✅ 50** | |

🛑 **`F-2` — §3's PRODUCTION/TEST CONVENTION IS ITSELF DEFECTIVE.** *"the file's first `#[cfg(test)]` line"* assumes one test module at the tail. **`xgen-node/src/admin_ops.rs` has TWO** — markers at `:3281` and `:4516`, the first closing at a column-0 brace on **`:3394`** ⇒ **`:3395-:4515` is PRODUCTION and §3 filed all of it as test.** Two sites were lost: **`admin_ops.rs:3460`** (D-3) and **`admin_ops.rs:4191`** (D-1). ✅ **Bound re-run over all 37 door-bearing files: `admin_ops.rs` is the only offender.** 🔑 ***A convention stated for reproducibility is still a claim, and this one was narrower than the tree it described*** — the dominant recurring defect class, in the clause written to prevent it.

🛑 **`F-3` — A LINE-ORIENTED SWEEP CANNOT SEE A MULTI-LINE CHAIN.** `rustfmt` breaks `state.members.keys()` across lines, leaving `.members` alone on one and `.keys()` on the next. **Seven production sites were invisible:** `ops.rs:2573 :2591 :2595 :2736` · `state_machine.rs:233` · `dm_promotion.rs:80 :130`.

🛑 **`F-4` — TWO COUNTED SITES ARE NOT THIS MAP.** `xgen-client/src/app.rs:3264` is `.members` on a **`MembersResult` projection**; `encryption/group.rs:68` is **`MlsGroup`** — the subsystem §3 itself excluded, counted anyway one paragraph later.

📌 **`F-10` — AND THE `65` IN §3 WAS NEVER THE STATED SET.** The distribution reproduces exactly: `contains_key 31 · len 22 · insert 17 · remove 12 · contains 8 · get 6 · iter 3 · keys 2 · clamp 2 · get_mut 1`. **§3's enumeration `{get,iter,keys,values,len,contains_key,is_empty}` sums to 64.** The 65th is **`get_mut`** (`state.rs:4017`) — **a write, not in the stated set, and a test site** ⇒ costs zero production. ⚠️ ***The total was right by one accident cancelling another.***

🔑 **`D-4` — `resolve_operator` IS A FOURTH DOOR AND §3 DOES NOT NAME IT.** `state.rs:1342` reads `self.members` **directly, five times** (`:1346 :1351 :1356 :1358 :1365`) rather than through either accessor ⇒ **`(i)` does not reach it.** Its five reads are counted inside D-3; its **two production observers** (`ai_service.rs:526`, `ops.rs:2564`) are additional. 🛑 **`CLAUDE.md:1470` documents it as *"transparently skips members who left"* — under `(g)` it stops skipping, and the record would then be false at a line nobody edits.**

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

⚠️ **SUPERSEDED AT v1.2 — `D-131`, ANNOTATED NOT DELETED.** v1.1 said *"D-1 (12) and D-3 (14) … 26 sites outstanding"* and predicted 🔑 ***"`D-3` is where EVER would live if it lives anywhere."*** ✅ **Both halves are now measured: the counts are 13 and 20 (§3b), and the prediction is FALSE — see §5c, §5d, §5e.**

---

## §5c — ✅ CLASSIFICATION RESULT: DOOR **D-1** (`is_member`), 13/13 PRODUCTION SITES, MEASURED AT `5da9e53`

| sites | what they are | tag |
|---|---|---|
| `exchange.rs:232, 370, 375, 397, 693` — **5** | validation-side membership gates — sender-must-be-member, AI-operator target checks | **CURRENTLY** |
| `dm_promotion.rs:72, 109, 148` — **3** | proposer · confirmer · rejecter of a DM promotion must be a party | **CURRENTLY** |
| `admin_ops.rs:1077, 4191` — **2** | `stale_membership_spaces` after an identity revoke · the removal precondition | **CURRENTLY** |
| `fanout.rs:488` — **1** | `collect_sync_history` — whose history may this requester pull | **CURRENTLY**, 🛑 privacy |
| `ai_service.rs:522` — **1** | `refresh_health_operator_counts` — spaces this AI is in | **CURRENTLY** |
| `runtime.rs:1665` — **1** | `new_joiner` detection | **CURRENTLY**, 🛑 **and INVERTED — see `C-3`** |

🔒 **EVER: 0 · INDIFFERENT: 0.**

---

## §5d — ✅ CLASSIFICATION RESULT: DOOR **D-3** (direct), 20/20 PRODUCTION SITES, MEASURED AT `5da9e53`

| sites | what they are | tag |
|---|---|---|
| `state.rs:1346, 1351, 1356, 1358, 1365` — **5** | `resolve_operator`'s five direct reads (**`D-4`**) | **CURRENTLY** |
| `state.rs:1100, 1112` — **2** | `apply_join`'s room guard · **the space-level `AlreadyMember` reject** | **CURRENTLY**, 🛑 **see `C-2`** |
| `ops.rs:2573, 2591, 2595, 2606, 2736` — **5** | AI-status projection (operator, role, invited-by, `members_count`) · the roster panel | **CURRENTLY** |
| `fanout.rs:272` — **1** | the fanout **recipient list** | **CURRENTLY**, 🛑 privacy |
| `runtime.rs:2106, 2260` — **2** | DM held-event drain · **`repopulate_dm_federation_nodes`** | **CURRENTLY**, 🛑 **see `C-4`** |
| `dm_promotion.rs:80, 130` — **2** | *the other party* · the delivery set | **CURRENTLY**, ⚠️ `:80` is `.keys().find()` on a `HashMap` |
| `admin_ops.rs:3460` · `node/app.rs:4045` — **2** | operator-facing member counts | **CURRENTLY** |
| `state_machine.rs:233` — **1** | `CutoverResult.member_ids` — Space migration | **CURRENTLY** 🔒 **RULED — Joe, 2026-08-22** |

🔒 **EVER: 0 · INDIFFERENT: 0.**

---

## §5e — 🔒 THE AGGREGATE, AND WHY **EVER IS ZERO**

| door | production | CURRENTLY | EVER | INDIFFERENT |
|---|---|---|---|---|
| **D-1** | 13 | 13 | 0 | 0 |
| **D-2** | 17 | 17 | 0 | 0 |
| **D-3** | 20 | 20 | 0 | 0 |
| | **50** | **✅ 50** | **✅ 0** | **✅ 0** |

✅ **§5's REOPEN TRIGGER IS DISCHARGED BY MEASUREMENT, NOT ARGUMENT.** It fires if EVER is the majority of the production set. **EVER is 0 of 50.** ⇒ **`(i)` STANDS.** 📌 *The trigger was live for three sessions and never had a chance of firing — but it could not be known to be idle until the census finished, which is exactly why it was recorded rather than waved off.*

🔑 **WHY THE `§5b` PREDICTION FAILED — AND IT IS STRUCTURAL, NOT LUCK.** §5b expected EVER to live in `D-3`: key material, DAG causality, history. **None of those read this map.** MLS keeps its own member set in `encryption/group.rs` / `client_mls.rs` — ***the very subsystem §3 excluded from scope in order to make the census honest.*** No audit path, no history path and no causality path touches `SpaceState.members` either. ⇒ **`SpaceState.members` is, today, a purely present-tense structure.** ⚠️ **That is what makes `(g)` cheap AND what makes it dangerous: there is no existing reader whose meaning `(g)` preserves, so every one of the 50 changes meaning at once.**

📌 **`(i)`'s REAL COST, NOW COUNTABLE:** the two accessors carry **30 sites free** (D-1's 13 + D-2's 17). **D-3's 20 need individual edits regardless of the ruling**, five of them inside `resolve_operator` (`D-4`). ***The ruling was never going to reach 40% of the census, and §5 does not say so.***

---

## §6 — WHAT E-0 DOES **NOT** DO

1. **It changes no behaviour.** `apply_leave` still removes; `left_at` does not exist yet. **E-0 produces a classified census and a ruling, nothing else.**
2. **It does not touch MLS membership** (`group.rs`, `client_mls.rs`) — a different subsystem with a different meaning.
3. **It does not classify test sites individually** (§4).
4. **It does not fix the `can_change_admission` exposure** (§2) — that lands with `(g)` in Leg E, and §2 exists so Leg E does not discover it.

---

## §7 — DoD

- [x] All **~~43~~ ✅ 50** production sites tagged **CURRENTLY / EVER / INDIFFERENT**, each with its reason, each cited **with its tree** — §5b · §5c · §5d, measured at `5da9e53`
- [~] ~~The **EVER** count reported to Joe **before** §5 is ruled~~ — 🛑 **THIS ITEM WAS NEVER SATISFIABLE BY THE TIME IT WAS WRITTEN: §5 was ruled on 2026-08-18, before the count existed.** ✅ **Discharged the only way left — the count (`0`) was reported against a live reopen trigger, and §5e discharges it.** 📌 *Recorded as broken rather than ticked; `D-065`.*
- [x] §5 ruled **(i)**, and **confirmed by the measurement it was conditioned on** (§5e)
- [x] Test-site counts recorded per door as the regression surface — 56 · 13 · 51 at `bf7f297`; ⚠️ **not re-measured at `5da9e53`, and §3b's convention defect means the D-1/D-3 test figures are each overstated by the production sites they swallowed**
- [ ] `§12`'s row for `E-0` corrected — **the name says `is_member`; the scope is `members`** — 🎯 **rides this commit**
- [ ] Records: JOURNAL + `CLAUDE.md` + ROADMAP + `Status: COMPLETED`, one `D-074` commit

---

## §8 — 🔓 FINDINGS CARRIED OUT OF E-0. **NONE OF THESE IS E-0's TO FIX.**

🛑 **THE OWNER COLUMN BELOW IS SUPERSEDED BY §9 AND IS LEFT STANDING (`D-131`). FOUR OF THE NINE ROWS WERE MIS-ROUTED BY THIS DOCUMENT's OWN AUTHOR.**

**E-0 changes no behaviour (§6.1). These are what the census SAW, routed so Leg D and Leg E do not rediscover them.**

| id | finding | owner |
|---|---|---|
| **`C-2`** | 🛑 **`state.rs:1112` REFUSES THE REJOIN `Q-2`(a) PROMISED.** A bare `contains_key` returns `AlreadyMember`. Under `(g)` the leaver is still in the map ⇒ ***the two halves of `Q-2` contradict each other.*** **Reached by `(i)` only if `is_member` gates too.** | 🔓 **JOE — gates Leg D** |
| **`C-3`** | 🛑 **A SECOND, SILENT GATE ON THE REJOIN PATH, POLARITY INVERTED.** `runtime.rs:1665` computes `new_joiner` from `already_member`; `new_joiner` drives the **full-history push** (`fanout.rs:277` builds, `:340` sends). Under `(g)` a rejoiner is still a member ⇒ `None` ⇒ ***she rejoins to an empty room, with no reject, no log and no failing test.*** 🔑 **Compounds with `C-2`: fixing `:1112` alone turns a clean refusal into a silent one.** ⚠️ **Every other site in the census treats `false` as the refusal; this is the one where `true` is the wrong answer.** | 🔓 **JOE — gates Leg E** |
| **`C-4`** | 🛑 **A DEPARTED PARTY'S NODE STAYS IN THE FEDERATION SET.** `repopulate_dm_federation_nodes` (`runtime.rs:2260`) derives `federation_nodes` from `members ∪ pending_invites`; under `(g)` it never shrinks ⇒ **her home Node keeps receiving federated DM traffic.** ⚠️ **`:2250-2251` demands a byte-identical `Vec<NodeXgid>` for the `assert_converges` oracle** ⇒ **a convergence surface, not only a privacy one.** Sibling of `algorithm.rs:221`. | 🔓 **JOE** |
| **`C-5`** | 🛑 **THE TWO BLUNT PRIVACY BREAKS.** `fanout.rs:272` — the recipient list is `space.members.keys()` ⇒ **a departed member keeps receiving every event.** `fanout.rs:488` — `collect_sync_history` ⇒ **she keeps pulling full history on demand.** With `ops.rs:2736` listing her in the roster, these are the three a user would actually see. | 🔓 **JOE** |
| **`C-6`** | ⚠️ **`state_machine.rs:233` IS ALREADY DIVERGENT FROM SPEC, BEFORE `(g)`.** `ch3:4347` says the source Node notifies all **currently connected** members; the code takes **all members, connected or not**. Three candidate sets, spec names the narrowest, code implements the middle, `(g)` would promote it to the widest. **Sole production consumer is `migration_driver.rs:307`, which reads only `.len()` into a log field** — so today the whole effect is a wrong number in one log line, against a `pub` field whose doc tells its unwritten caller to send to everyone in it. 📌 Covering test `:492` is a bare `len() == 2` — **`F-D`'s species**. | 📌 **NOT BLOCKING — belongs to whoever writes `transport.redirect`** |
| **`C-1`** | 🛑 **THE CENSUS COUNTS READS AND `(g)` IS A WRITE.** `self.members.remove` fires at **four** production sites: `apply_leave:1142` (the `(g)` edit itself) · `apply_kick:1169` · `apply_ban:1189` · `:1214`. **`(g)` is specified for LEAVE ONLY.** ⇒ ***Does a kick or a ban set `left_at`, or actually remove?*** If they remove, the map holds leavers but not kicked members and **"in `members`" stops meaning one thing.** 🔑 **No read census can see this — it is the question the instrument was not shaped to ask.** | 🔓 **JOE — gates Leg E** |
| **`F-12`** | **`apply_leave` also removes from EVERY `RoomState.members`** (`:1145-1146`). `(g)` says nothing about rooms. | 🔓 **JOE** |
| **`C-7`** | **`state.rs:232`'s doc comment reads `/// Active members`** — the field `(g)` redefines. Sibling of §5's stale `space_admission` comment, one line away. | 📌 **rides `(g)`** |
| **`F-E`** | **`build_membership_event` emits `prev_events: vec![]`** — invisible to unit tests calling `apply_event`, a DAG violation on the node ingest path. **It already cost Clair a run in Leg C.** ⚠️ **Leg E's rejoin work IS node-ingest-path work** (`Leg G`'s `get_rejoin_anchor`) ⇒ 🎯 **Chat recommends folding it into Leg E's Phase-0 as a named precondition rather than leaving it on an open list.** | 🔓 **JOE** |

📌 **"Commit pushed" is not a DoD item.**

---

## §9 — 🔒 THE RULINGS, AND THE ROUTING CORRECTED

### §9a — Joe, 2026-08-22 — **`D-154`**

**Five clauses, ruled together.** ① the rejoin is *back as of now* (`left_at` clears · `joined_at` re-stamped · **role re-derived**) · ② **kick is remembered, not erased** · ③ **ban follows kick**, `self.banned` unchanged · ④ **the gap stays closed** — history up to departure plus from the rejoin forward · ⑤ **a rejoin restores presence in the Space only; rooms are not restored.** 🔑 ***A rejoin restores PRESENCE, never POSITION*** — the principle all five share. **Full reasoning, costs and named non-settlements at `D-154`.**

⇒ **`C-2` CLOSED** (① — and `:1112` must gate, or ②/③ make the ban check dead code) · **`C-1` CLOSED** (②③) · **`C-3`'s POLICY HALF CLOSED** (④; `Leg G` is now load-bearing) · **`F-12` CLOSED** (⑤).

### §9b — 🛑 FOUR ROWS WERE MIS-ROUTED. RETAKEN BY CHAT, NAMED (`D-123`).

| | why it was never Joe's |
|---|---|
| **`C-5`** | *"does a departed member keep receiving every event and pulling full history?"* — **one honest answer and three dishonest ones.** `RUNBOOK_LIVEFEED_LEG_A.md:19`: Joe owns choices **between honest options**; he does not own whether the system does something indefensible. **Filed for Leg E.** |
| **`F-E`** | where a known fixture precondition is written down is **sequencing and records**. Chat attached a recommendation and then asked permission to act on it — ***`:4625`'s "proposing is not deciding", inverted into deciding and then asking.*** **Goes into Leg E's Phase-0.** |
| **`C-7`** | a stale doc comment. Records. |
| **`C-6`** | already filed non-blocking; belongs to whoever writes `transport.redirect`. |

### §9c — 📌 TWO ITEMS TAKEN BY CHAT WITH A RECOMMENDATION, REVERSIBLE ON ONE WORD

1. **`F-3`'s byte cap — 64 bytes, char-boundary truncation.** A malformed value is never displayed ⇒ **`D-121` lens ① is zero either way** (`CLAUDE.md:828`'s *visually identical ⇒ technical execution*). Joe ruled the part that was his — **(a′), malformed stored as raw JSON text.** 🔑 **The load-bearing constraint is not the number: every node must truncate IDENTICALLY or the stored value diverges.**
2. **`C-4`'s convergence pin.** The `left_at` filter on `federation_nodes` is Chat's; the pin at `runtime.rs:2250-2251` governs **determinism of ORDER, not membership of the SET** ⇒ filtering does not disturb it. ⚠️ **Chat told Joe *"your queue is empty"* while this had been taken rather than ruled — recorded so the taking is visible rather than silent.**

### §9d — 🔑 WHAT THE RULING SESSION ITSELF DEMONSTRATED

🛑 **Chat's first form of question ① offered Joe a decision he had already made on 2026-08-18** — §5's option **(i)** gates `is_member` explicitly — **and argued for it on CHEAPNESS** (*"one accessor versus 13 hand edits"*) **for a site that no accessor change reaches**, `:1112` being a direct `contains_key`. ⇒ ***the cheap answer was doing the arguing because the site had not been opened.*** **Joe asked for the claim to be grounded; opening the site is what found `D-154`'s whole subject — that `insert` REPLACES.**

📌 **And the questions became answerable only after two rejected framings.** Option tables naming code sites are unanswerable by the seat that owns **meaning**; ***"what does the Space remember about her?"*** is the same decision, stated in the vocabulary of the person who has to make it. **Recorded as method, not as apology.**
