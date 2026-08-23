# RUNBOOK — M-SPACE-ADMISSION Leg E-2: clause ④, the gap — presence intervals across three doors

> **Status**: PENDING  
> Version: 1.0  
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
| **Status** | 🛑 **PENDING — NOT LOCKED.** Clair does not open this file until Joe locks it |
| **Blocking on** | ✅ **NOTHING.** §7 ruled **(b)** and recorded as a `D-154`④ clarification; §4's fork settled **(B)** by Chat |
| **Tree** | every citation measured at **`0d93117`** = `origin/main`, tree clean (`D-152`) |
| **Floors in** | cargo **1629 / 0 / 62 × 56 SUITES** · vitest **172 / 172 × 9 FILES** · svelte-check **0 / 34 / 15** · catalogue **UNMEASURED** |
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

- [ ] `E2-1` … `E2-6` implemented from this file, **no improvisation** — a blocked edit is REPORTED (Rule 6)
- [ ] `E2-4` resolved **by reading**, and any non-empty intersection REPORTED rather than patched
- [ ] `W-1` … `W-9` re-driven by Chat from `HEAD`, none adopted on report
- [ ] `W-3a` … `W-3f` each run, each restored with `Compiling` observed; **`W-3f`'s result RECORDED whichever way it goes**
- [ ] **`V-4` discharged** — the gate E-1 could not close
- [ ] Phase-0 v1.1 → **v1.2**; this runbook → **COMPLETED**
- [ ] `roadmap-format-gate.ps1` exit 0 before any ROADMAP commit
- [ ] `D-074` atomic commit

📌 **"Commit pushed" is not a DoD item.**
