# RUNBOOK — M-SPACE-ADMISSION Leg E-2: clause ④, the gap — presence intervals across three doors

> **Status**: COMPLETED  
> Version: 1.1  
> Date: Aug 2026  
> **Last updated**: 2026-08-23  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — LOCK STATE

| | |
|---|---|
| **Phase-0** | `tasks/M_SPACE_ADMISSION_LEGE2_PHASE0.md` **v1.1 ACTIVE** — read §3, §4b, §5 and §7 before this file |
| **Status** | ✅ **COMPLETED (J-770, 2026-08-23).** Implemented by Clair; every gate re-driven by Chat from `HEAD` under Rule 5. **Floor 1629 → 1641 / 0 / 62 × 56 SUITES.** 🛑 **THREE OF ITS OWN SPECIFICATIONS WERE DEFECTIVE — see §9** |
| **Blocking on** | ✅ **NOTHING.** §7 ruled **(b)** and recorded as a `D-154`④ clarification; §4's fork settled **(B)** by Chat |
| **Tree** | every citation measured at **`0d93117`** = `origin/main`, tree clean (`D-152`) |
| **Floors in / out** | cargo **1629 → 1641 / 0 / 62 × 56 SUITES** · vitest **172 / 172 × 9 FILES** · svelte-check **0 / 34 / 15** · catalogue **UNMEASURED** |
| **Rule 5** | Chat re-drives **every** gate from `HEAD`. Nothing is adopted on report |
| **Rule 6** | Clair reports deviations and never absorbs them. **Leg E-1's `F-3` was caught this way and it was the finding of the leg** |
| 📌 **Naming** | Always **"M-SPACE-ADMISSION Leg E-2"**, never bare — `M-RP-MEMBER-ACT` has its own Leg E-2 on disk (Phase-0 §1) |

---

## §1 — WHAT E-2 IS

🎯 **The commit in which a returning member stops receiving the conversation she missed.**

🔒 **`D-154`④, as clarified 2026-08-23:** *everything up to her departure, plus everything from the rejoin forward* — **the gap closed to CONTENT and OPEN TO MEMBERSHIP STRUCTURE.**

## §1b — WHAT E-2 IS NOT

1. ❌ **Not `get_rejoin_anchor`** — Leg G, a new wire verb, Joe's seat.
2. ❌ **Not the gap's MARKER.** `D-154` leaves *"something was said while you were away"* with Joe as appearance. **E-2 builds the boundary and stops.**
3. ❌ **Not unifying `topological_sort` / `topological_sort_events`** (Phase-0 §4(C)) — filed, named, out.
4. ❌ **Not the federation delta** (`fanout.rs:662`) — node-to-node, `D-089`, and ④ is about what a **person** reads.
5. ❌ **Not `ui/**`.**
6. 🛑 **Not `build_membership_event`** — the empty `prev_events` is the documented root-adjacent contract.
7. ❌ **Not room scoping.** Today every door serves a Space's events regardless of room. **Unchanged.** *Named because it looks adjacent and is not.*

---

## §2 — GROUNDING. **EVERY LINE MEASURED AT `0d93117`.**

### §2a — the three doors

| door | site | serves | gate today | today's payload |
|---|---|---|---|---|
| **①** | `fanout.rs:285-298`, delivered `:348-359` | the joining identity, **pushed** | `req.new_joiner.is_some()` | `store.range(0)` → `topological_sort_events` → drop the triggering event → one `HistoryBatch` |
| **②** | `fanout.rs:496-503` | **any identity that asks, pulled** | `:496` `space.is_member(requester)` | per member-Space `store.range(0)` → `topological_sort_events` → `since` cursor → `limit` page |
| **③** | `fanout.rs:608-618` | a **pending invitee** | unexpired `pending_invites` entry | `store.range(0)` → `is_structural_bootstrap_type` filter → `topological_sort_events` |

🔑 **DOOR ② IS THE ONE THE PHASE-0 FOUND AND THE ONE MOST LIKELY TO BE FORGOTTEN.** `is_member` is the **present-tense** accessor E-1 gated ⇒ **a rejoiner passes it**, and `:501` serves her the entire store.

📌 **DOOR ③ AFTER THE §7 RULING IS ALMOST CERTAINLY A NON-EDIT — and Clair CONFIRMS that by reading, not by assuming.** `is_structural_bootstrap_type` (`:549-563`) already serves **only** structural events, and the ruling says structure passes. ⇒ **the intersection of "what ③ serves" and "what ④ withholds" should be EMPTY.** 🛑 **If it is not empty, that is a finding and it is REPORTED, not patched** (Rule 6).

### §2b — the two sorts

| fn | crate | job |
|---|---|---|
| `topological_sort` | `xgen-core/src/node/runtime.rs:2367` | **the state fold** (`:650`, `:2241`) — *the order that decides who is a member.* Imported by `resolution/derive.rs:49` |
| `topological_sort_events` | `xgen-node/src/fanout.rs:405` | **delivery order** at all three doors |

✅ **Core's is ALREADY reachable from `xgen-node`:** `app.rs:59` imports it, `app.rs:5028` calls it, and **`app.rs:5642` `topological_sort_publicly_reachable_from_xgen_node` is an existing test asserting exactly that.**

### §2c — the field E-1 left

`SpaceMember::left_at: Option<String>` (`state.rs:114`; its doc comment opens at `:95`), `SpaceMember::is_present()` (`state.rs:121`). 🔑 **`left_at` holds the CURRENT boundary only** — its doc comment says so, and says the history is derived here. 🔒 **`apply_leave` is FIRST-WINS** (E-1 `J-3`): a second leave errors and does **not** move the mark. ***That is a boundary this leg depends on.***

---

## §3 — THE EDITS

### 🔒 `E2-1` — the interval walk (new, in `fanout.rs`)

A free function taking the Space's events and one identity, returning the **set of `event_id`s that identity may receive**.

🔒 **IT ORDERS WITH `xgen_core::node::runtime::topological_sort`, NOT with `topological_sort_events`** (Phase-0 §4b). *Same function the fold uses ⇒ the slice agrees with `left_at` by construction rather than by coincidence.*

**The walk, in that order, maintaining one `present` flag:**

| | |
|---|---|
| **start** | `present = true` — 🔑 **the first interval opens at index 0, NOT at her first join.** ④ says *"everything up to her departure"*, and a first-time joiner receives the whole store today |
| **closes** | `MembershipLeave` where `sender == her` · `MembershipKick` / `MembershipBan` / `MembershipNodeEject` where `content["target_identity"] == her` |
| **reopens** | `MembershipJoin` where `sender == her` **and `room_id` is empty** |
| **while `present == false`** | ✅ **`is_structural_bootstrap_type(e)` ⇒ ADMIT** (`D-154`④ as clarified: structure is not content) · everything else ⇒ **withhold** |
| **the boundary events themselves** | her own departure event and her own rejoin event are **structural ⇒ admitted** either way. *Recorded so nobody writes an off-by-one guard for a case the structural rule already covers* |

🛑 **BOTH DEPARTURE SHAPES OR THE WALK IS WRONG.** `leave` names the departed as `sender`; `kick` / `ban` / `node_eject` name her in `content["target_identity"]` and the **sender is the actor**. ***A walk reading only `sender` produces a plausible, non-empty, WRONG slice, and every `leave`-based test passes*** (`N-197`). **Its control is `E2-6`.**

🔒 **N cycles ⇒ N+1 intervals.** A one-boundary implementation is wrong by construction — Phase-0 §5d(D) was refused for this reason.

### 🔒 `E2-2` — door ① , the joiner push

`fanout.rs:285-298`. The existing `store.range(0)` → `topological_sort_events` → drop-triggering-event chain is **kept**; `E2-1`'s set is applied as one further filter. 📌 **`req.new_joiner` is the identity.** ⚠️ **Delivery order unchanged** — only the set changes.

### 🔒 `E2-3` — door ②, `collect_sync_history`

`fanout.rs:496-503`, **per Space inside the loop**, using `requester_id`. ⚠️ **The `since` cursor is applied AFTER the filter, over the already-filtered candidate list** — otherwise a cursor pointing into a gap resolves against events that are about to be withheld, and `:513`'s `position()` miss returns `(Vec::new(), None)`, **a silent empty sync**. 🛑 **Order of operations is load-bearing here; `E2-8`.4 is its test.**

### 🔒 `E2-4` — door ③, `collect_invite_bootstrap`

**Expected: NO EDIT** (§2a). 🛑 **Clair opens `is_structural_bootstrap_type` and confirms the intersection is empty. If it is not, REPORT — do not patch.**

### 🔒 `E2-5` — `V-4`, inherited from E-1 and discharged here

Two tests E-1 could not close:
1. **`C-3`** — a rejoiner produces `new_joiner: Some(_)` at `runtime.rs:1713`.
2. **`C-5b`** — `collect_sync_history` for a **RETURNED** member. 🔑 ***The filed finding said "self-closes under `(i)`" — true for a DEPARTED member and FALSE for a returned one, which is why this test exists.***

### 🔒 `E2-6` — the tests

1. **NO-OP for a first-time joiner** — byte-identical to today's payload at door ①. *The property that keeps this leg from being a regression.*
2. **One gap, door ①** — join → talk → leave → talk → rejoin. She receives the pre-departure talk, **not** the gap talk.
3. **Structure passes** — the same fixture: she **does** receive the `membership.*` events from the gap (`D-154`④ as clarified).
4. **Two cycles, two gaps** — leave → rejoin → leave → rejoin; **both** gaps closed.
5. 🛑 **THE KICK CONTROL** — the same shape but the departure is a **`membership.kick`**, where she is `content["target_identity"]` and not `sender`. *A `leave`-only suite cannot see the `N-197` failure.*
6. **Door ②** — a returned member issuing `sync_request` with an **empty cursor** receives the filtered set, not the store.
7. **Door ② cursor** — a cursor pointing at an event **inside a gap** does not produce a silent empty sync (`E2-3`).
8. **Two-sort equality regression** — `topological_sort` and `topological_sort_events` agree on a fixture DAG **with concurrency**. 📌 **This test exists whichever way §4 went;** it is the only thing that will make the two drifting apart visible.
9. **A never-departed member is unaffected** at both doors — the positive control that stops every probe here answering "withheld" for everyone.

🛑 **FIXTURE RULE (inherited):** a non-root membership event built with `state::build_membership_event` carries `prev_events: vec![]` **by contract** and will fail DAG validation on the node ingest path. **Chain it explicitly, or build via `Event::new`** — idiom at `xgen-node/src/tests/space_admission_gate.rs:56`. E-1's `fanout_excludes_a_departed_member` chains off `rt.dag_tips`; **follow it.**

---

## §4 — THE STRUCTURAL BINDING

🔒 **`E2-1` (the walk) and `E2-2` / `E2-3` (the doors) are BARRED from sharing a test.**

🔑 ***`M-1`'s species, and Leg E-1 paid for it twice: a filter proven only at the door cannot show the walk is right, and a walk proven only in isolation cannot show the door calls it.*** **`E2-6`.5's kick control belongs to the WALK; `E2-6`.6 belongs to DOOR ②. Each turns red on its own.**

---

## §5 — NEGATIVE CONTROLS

| control | disarm | required |
|---|---|---|
| **W-3a** | drop `E2-1`'s filter at door ① | `E2-6`.2 **RED** |
| **W-3b** | drop `E2-1`'s filter at door ② | `E2-6`.6 **RED** — 🔑 *the door the Phase-0 found; its own control or it is not covered* |
| **W-3c** | make the walk read only `event.sender` | 🛑 **`E2-6`.5 RED and `E2-6`.2 GREEN.** ***That exact split IS the `N-197` proof; if .2 also goes red the control is not isolating what it claims*** |
| **W-3d** | close the first interval at her first join instead of at index 0 | `E2-6`.1 **RED** |
| **W-3e** | admit nothing while `present == false` (drop the structural clause) | `E2-6`.3 **RED**, and `E2-6`.2 **stays green** |
| **W-3f** | order `E2-1` with `topological_sort_events` instead of core's | ⚠️ **may be GREEN** — 📌 **a green here is a MEASUREMENT, not a failure: it says the fixture has no concurrency at the boundary. RECORD IT, do not force it red.** `E2-6`.8 is the standing guard |

🛑 **`N-199` / `N-124b` ON EVERY CONTROL:** assert the mutation changed something **on CONTENT**, not on a remembered offset · restore → **stamp mtime to now** → **require `Compiling` in the log** · verify the restore **sha256-identical**. **An absent `Compiling` line is not efficiency; it is an unproven run.**

---

## §6 — GATES. **CHAT RE-DRIVES ALL OF THEM FROM `HEAD` (Rule 5).**

| gate | what |
|---|---|
| **W-1** | `cargo test --workspace --no-fail-fast` — **detached**, logged, `XGEN_EXIT_SENTINEL=` appended, `^test result:` summed **case-sensitively**. Floor in **1629 / 0 / 62 × 56 SUITES**. **Delta MEASURED with `--skip` on the delivered tree**, never arithmetic. **`--no-fail-fast` or the run reports a fraction of the suites** (E-1 `F-2`) |
| **W-2** | **All three doors opened individually** and their treatment confirmed — ② edited, ③ confirmed a non-edit **by reading `is_structural_bootstrap_type`** |
| **W-3a…f** | §5, each run separately, each restored per `N-199` |
| **W-4** | **NO-OP for a first-time joiner**, proven at door ① |
| **W-5** | **`V-4` DISCHARGED** — `C-3` and `C-5b` both have tests |
| **W-6** | **The kick control isolates** — `E2-6`.5 red, `E2-6`.2 green under `W-3c` |
| **W-7** | **The two-sort regression test exists and passes** |
| **W-8** | Scope: **zero `ui/**`** · `fanout.rs:662` untouched · `build_membership_event` untouched · the marker not invented · the two sorts not unified |
| **W-9** | 📌 **`V-6`'s corrected closure rule (E-1 §9b): where the change PRESERVES an assertion, no automated instrument can enumerate the affected tests — name READING as the mechanism and say so.** ⚠️ **Directly live here: every existing history test asserts what a PRESENT member receives, and this leg does not change that.** Expect **few or zero** existing tests to go red, **and do not read that as the change not landing** — that inference is exactly what E-1's §2c got wrong |

---

## §7 — OPEN AT LOCK

✅ **Nothing.** §7 ruled **(b)** → `D-154`④ clarification. §4 settled **(B)** by Chat. ⚠️ **`D-154`④'s caveat is CARRIED, NOT DISCHARGED and is not E-2's to close:** a returning member learns **who else was removed, and when**, about people who cannot see that she now holds it. **Filed at `D-154`④ beside the `self.banned` look and the `D-093` bytes-without-membership gap.**

---

## §8 — DoD

- [x] **`E2-1` … `E2-6` implemented; SIX findings and one observation REPORTED, none absorbed (Rule 6)**
- [x] **`E2-4` resolved BY READING: `is_structural_bootstrap_type` serves 3 creates + the 7-event membership chain — precisely the set ④-as-clarified admits while absent. Intersection EMPTY, non-edit confirmed**
- [x] **`W-1` … `W-9` re-driven by Chat from `HEAD`; `W-7` RETIRED as unwritable and substituted (§9a)**
- [x] **`W-3a` … `W-3f` each run by Chat, each sha256-restored, `Compiling xgen-node` on every disarm. `W-3f` GREEN and RECORDED (§10)**
- [x] **`V-4` DISCHARGED** — `a_rejoiner_dispatches_as_a_new_joiner` (`C-3`) and `sync_history_withholds_the_gap_for_a_returned_member` (`C-5b`)
- [x] **Phase-0 v1.1 → v1.2; this runbook → COMPLETED**
- [x] **`roadmap-format-gate.ps1` exit 0**
- [x] **`D-074` atomic commit: `fanout.rs` + JOURNAL + CLAUDE.md + ROADMAP + task docs**

📌 **"Commit pushed" is not a DoD item.**

---

## §9 — 🛑 CLOSE ANNOTATIONS (J-770, 2026-08-23). **THREE OF THESE ARE DEFECTS IN THIS DOCUMENT.**

Corrected at close, never erased (`D-131`). §3, §5 and §6 above stand as written; each is annotated here.

### §9a — 🛑 **`E2-6`.8 / `W-7` WERE NOT WRITABLE AS SPECIFIED. THE TWO SORTS ARE PROVABLY NOT ORDER-EQUAL.** *(Clair, `F-1`; verified by Chat BY INSPECTION)*

The gate asked for *"`topological_sort` and `topological_sort_events` agree on a fixture DAG with concurrency."* ✅ **They do not, and it is a proof, not a probe:**

- **`topological_sort_events`** (`fanout.rs:413-459`) re-sorts **all remaining** events each round and emits everything ready **within that sweep** — `a` emitting makes `b` ready immediately, in the same pass. `{a, z}` with `a→b` ⇒ **`[a, b, z]`**. *Depth-favouring.*
- **`topological_sort`** (`runtime.rs:2367-2425`) is **Kahn with a `VecDeque`**: pop `a`, push `b` to the **back**. ⇒ **`[a, z, b]`**. *Breadth-favouring.*

**Both are valid linear extensions.** Clair measured the same divergence on a diamond-plus-independent-chain (`[a,b,c,d,m,n]` vs `[a,m,b,c,n,d]`).

🔑 ***Order-equality could only be made TRUE by unifying the two sorts, which §1b.3 of this same document FORBIDS.*** **A gate that can only pass by doing what the runbook prohibits was never a gate.**

✅ **SUBSTITUTED — `two_sorts_preserve_the_event_set_and_causal_order`** — set-preservation plus causal validity, **which is what `E2-1` actually depends on and which order-equality never pinned.** 🔑 ***The divergence does not touch correctness, and that is precisely what option (B) bought:*** the walk uses core's sort, so its boundary agrees with the fold's `left_at` **by construction**, and the delivery order stays the delivery sort's. ⚠️ **The pre-existing caveat still applies and is not this leg's:** a *live incrementally-applied* `left_at` can differ from the *rebuilt* fold under concurrency — J-743 `F-2`, restated at E-1's `J-3`. **The walk agrees with the RESOLVED fold.**

### §9b — 🛑 **`E2-3`'s STATED MECHANISM PRODUCES THE EXACT DEFECT `E2-6`.7 FORBIDS. THE SENTENCE IS INVERTED.** *(Clair, `F-2`; verified by Chat)*

§3's `E2-3` reads: *"The `since` cursor is applied AFTER the filter … **otherwise** a cursor pointing into a gap resolves against events that are about to be withheld, and `:513`'s `position()` miss returns `(Vec::new(), None)`, a silent empty sync."*

🛑 **Backwards.** Applying the cursor after the filter is what **causes** the miss: **before clause ④ every member-Space event was in the candidate list**, so a `position()` miss meant a genuinely unknown cursor and `(vec![], None)` was truthful. **The filter can now remove a cursor that resolves perfectly well** — and an empty page with no `continue_from` is **byte-identical to *caught up*** (`collect_sync_history_empty_when_caught_up` pins that exact shape). ✅ **Measured, not argued: with the prescribed order the suite ran 16 green / 1 red, and the red one was `E2-6`.7.**

✅ **RESOLVED with the unfiltered-order fallback, flagged in code at the site.** ✅ **Chat verified the index arithmetic:** `candidate` is the permitted subsequence of `unfiltered_order` **in the same order** (both are pushed in one loop), so the count of permitted ids at-or-before the cursor **is** the resume index. **A cursor unknown in BOTH orders keeps today's refusal — only a WITHHELD one is rescued.**

📌 **And `E2-3` points at a non-existent `E2-8.4`.** The tests are `E2-6`; §8 is the DoD. Chat's.

### §9c — 🛑 **`E2-1`'s CLOSE CONDITIONS OMIT THE `room_id`-EMPTY REQUIREMENT FOR `leave` AND `kick`.** *(Clair, `F-3`; verified by Chat against the appliers)*

✅ **MEASURED AT `b82f942`:** `apply_leave` (`state.rs:1277`) and `apply_kick` (`state.rs:1314`) **each return early on a room-level event without touching `left_at`**; `apply_ban` and `apply_node_eject` have **no room-level branch** and close regardless.

🔑 ***A walk closing on a room-level leave would open a gap the fold never opened — the exact walk-disagrees-with-`left_at` failure option (B) exists to eliminate.*** **Implemented to match the appliers**, plus one test beyond the specified nine — `walk_ignores_a_room_level_leave_because_the_applier_does` — **flagged as an addition rather than slipped in.**

### §9d — ✅ **CLAIR'S OWN THREE, ACCEPTED AND RECORDED**

📌 **`F-4` — `E2-6`.1 was a test that could not fail, and only the control found it.** Written first against `setup_three_member_space`, it passed under `W-3d` — **that fixture holds only structural events, so the disarm was invisible to it.** 🔑 ***`F-3`'s species from Leg E-1, one leg later, caught by its own author.*** Rebuilt on bob in `setup_gap_space`, who has real content before his join, **with a precondition assertion so the fixture cannot silently degrade back.**

📌 **`F-5` — the admit set is built by SUBTRACTION, not collected from the sort output.** ✅ Verified: core's `topological_sort` **`filter_map`s away id-less events** (`runtime.rs:2375`) and Kahn never emits a cycle member, while `topological_sort_events` explicitly *"guarantee[s] the function preserves all input"* (`fanout.rs:457`). Both losses are store-unreachable today, **but subtracting keeps `W-4` true BY CONSTRUCTION rather than by an infeasibility argument, and fails toward *delivered as today* rather than toward *silently withheld from everyone*.**

📌 **`F-6` — a Space CREATOR emits no `membership.join`.** Surfaced by `W-3d`'s blast radius: two **pre-existing** pagination tests went red because alice never reopens under `present = false`. 🔑 ***⇒ "open at index 0" is not a clause-④ nicety — it is required for an owner to receive anything at all.*** ✅ **Chat reproduced both reds independently.**

⚠️ **ONE OBSERVATION, CARRIED NOT FIXED:** a close event naming her **before her first join** closes an interval the fold never closed — `mark_departed` no-ops on an absent member. **Errs restrictive; fixing it means re-implementing the fold inside the walk.** ✅ **Accepted as filed.**

### §9e — 📌 **TWO INSTRUMENT ERRORS, BOTH CAUGHT BY A GUARD RATHER THAN BY OUTPUT** *(Clair)*

🛑 **`head -60` on the control script sent SIGPIPE and killed it mid-run, leaving `W-3e` ARMED — and the next read took a stale `w3f.log` from the previous run as current.** **Caught by the sha256 check, not by the output.** Restored and re-run with file redirection.

🛑 **The detached cargo log read BEFORE the sentinel: `54 suites / 1641 / 61 ignored` — plausible, complete-looking, WRONG.** 🔑 ***The task notification's "exit code 0" is the LAUNCHER'S, not cargo's; the sentinel is what separates them.*** **`N-197`'s species at the harness layer, twice in one session.**

📌 **And Clair caught herself transcribing `E2-3`'s inverted reasoning into a code comment and corrected it — the J-670 shape, where a bad runbook line outlives the document that carried it.** 🔑 ***That is the failure mode `D-131` annotation exists to prevent, caught at the moment of copying.***

---

## §10 — 🔒 CLOSE MEASUREMENTS (Chat, Rule 5, from `HEAD` `b82f942`)

**Delivered bytes unchanged after all six controls: `fanout.rs` sha256 `507B85CD…`, matching the hand-back.**

| gate | measured | |
|---|---|---|
| **W-1** | `1641 / 0 / 62 × 56 SUITES` · `XGEN_EXIT_SENTINEL=0` · `Compiling xgen-node` present · `FAILED` **0** case-sensitive | ✅ |
| **W-2** | `--skip` on the 12 returns **exactly `1629 / 0 / 62 × 56`**, `filtered out = 12` | ✅ **MEASURED** |
| **W-3a** | **2 RED** — 📌 *one more than the hand-back reported: the structure test also goes red, because it asserts both halves* | ✅ |
| **W-3b** | 2 RED, incl. `sync_history_resumes_from_a_pre_departure_cursor_skipping_the_gap` | ✅ |
| **W-3c** | 🔒 **EXACTLY 1 RED — the kick walk test. Leave-based door tests GREEN.** ***The `N-197` proof, isolating exactly as `W-3c` requires*** | ✅ |
| **W-3d** | 4 RED, **incl. both pre-existing pagination tests** — `F-6` reproduced independently | ✅ |
| **W-3e** | 4 RED incl. the structure test; **`rejoiner_push_withholds_the_gap_conversation` GREEN** | ✅ both halves |
| **W-3f** | **GREEN, 44 passed** — 📌 **RECORDED AS A MEASUREMENT, NOT FORCED RED.** Every fixture chains off the single running tip ⇒ no concurrency at any boundary | ✅ |
| **all six** | `Compiling xgen-node` on **every** disarm · **all six sha256-restored** · zero stray `.bak` · `numstat` identical after | ✅ `N-199` |
| **W-2 (doors)** | all three opened individually; ③ confirmed a **NON-EDIT by READING** | ✅ |
| **W-5** | `V-4` **DISCHARGED** — `C-3` and `C-5b` both have named tests | ✅ |
| **W-7** | **RETIRED as unwritable, substituted** — §9a | ⚠️ |
| **W-8** | **1 file, 9 hunks** · `fanout.rs` federation delta untouched · `build_membership_event` untouched · **neither sort touched** · zero `ui/**` | ✅ |
| **W-9** | ✅ **named as required: essentially no existing tests moved, and that is the EXPECTED outcome, not evidence the change did not land.** Every existing history test asserts what a **PRESENT** member receives. **The closure mechanism is READING; no automated instrument could enumerate the affected set** | ✅ |
