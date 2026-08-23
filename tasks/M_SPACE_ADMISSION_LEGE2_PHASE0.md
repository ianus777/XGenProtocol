# M-SPACE-ADMISSION Leg E-2 Phase-0 — clause ④, the gap: what a rejoiner may read, and the three doors that serve it

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

Leg E-2 of **M-SPACE-ADMISSION — who may join a Space, and how a leaver comes back**. Its subject is exactly one clause:

🔒 **`D-154`④ — *everything up to her departure, plus everything from the rejoin forward. THE GAP STAYS CLOSED.***

**It is a Phase-0.** It grounds, measures and routes. §7 carries one question for Joe; **the rest did not wait for it** (`D-123`).

🛑 **THE HEADLINE, AND IT CORRECTS A DOCUMENT I WROTE:** the Leg E Phase-0 §5c states *"`fanout.rs:262-289` IS WHERE CLAUSE ④ LIVES."* **Measured at `7b7b42f`: there are THREE client-facing doors to a Space's history, not one** (§3). Fixing the named one closes the door and leaves the window open.

---

## §1 — STATE, RE-MEASURED AT OPEN

| | measured |
|---|---|
| `HEAD` | **`7b7b42f`** = `origin/main` by `git ls-remote origin refs/heads/main`, tree clean |
| JOURNAL | max **J-767** · DECISIONS 161, max **D-155** (`D-154` has **six** clauses) · ROADMAP **v7.52** |
| cargo | **1629 / 0 / 62 × 56 SUITES** — measured on the delivered E-1 tree (J-767) |
| vitest · svelte-check | **172 / 172 × 9 FILES** · **0 / 34 / 15** — carried by scope |
| catalogue | 🛑 **UNMEASURED** |

📌 **NAMING, AND IT IS NOT PEDANTRY.** `Leg E-2` is **already taken** — `M-RP-MEMBER-ACT` has its own Leg E-2, with `tasks/RUNBOOK_MEMBER_ACT_LEG_E2.md` and `CLAIR_LEG_E2_RUNBOOK_READ.md` on disk. A repo-wide sweep for `E-2` returns **48 files**, almost all of them that milestone's. 🔒 **RULE FOR THIS LEG: always *"M-SPACE-ADMISSION Leg E-2"*, never bare.** *The milestone-naming rule exists for exactly this, and this is the first time in the arc where a bare ID would have resolved to the wrong milestone.*

---

## §2 — THE BACKLOG SWEEP. **RAN BEFORE ANYTHING ELSE.**

**Method:** `git ls-files`, worktrees excluded, `E-2` case-sensitive — then **scoped to the M-SPACE-ADMISSION arc**, because the unscoped sweep is 90% `M-RP-MEMBER-ACT` noise (§1).

| # | inherited item | assigned at |
|---|---|---|
| **1** | **Clause ④ itself** — the presence-interval filter | `M_SPACE_ADMISSION_LEGE_PHASE0.md:262` · `DECISIONS.md` `D-154`④ |
| **2** | 🛑 **`V-4` UNDISCHARGED** — no test proves `C-3`'s `new_joiner` polarity or `C-5b` for a departed member | `RUNBOOK_..._LEG_E1.md` §11 · `LEGE_PHASE0:262` |
| **3** | ⚠️ **The two-departure-shapes caveat** — `sender` for `leave`, `content["target_identity"]` for `kick`/`ban`/`node_eject` | `LEGE_PHASE0:195` |
| **4** | 🔒 **`first-wins` on `left_at` is a boundary E-2 depends on** — `apply_leave` refuses a second leave rather than moving the mark | `RUNBOOK_..._LEG_E1.md:114` (`J-3`) |
| **5** | 🛑 **The gap's MARKER is Joe's and E-2 does not invent it** | `LEGE_PHASE0:303` |
| **6** | 📌 **`V-6`'s corrected closure rule** — where a semantic change PRESERVES an assertion, no automated instrument can enumerate the affected tests; name reading as the mechanism | `RUNBOOK_..._LEG_E1.md` §9b |

✅ **Six items, all present.** As in Leg E-1, the sweep's value was what it did **not** contain — §3 and §4.

---

## §3 — 🛑 THE DELIVERY-SITE CENSUS. **THREE DOORS, NOT ONE.**

`E-0` censused the sites that **read** membership. Nobody had censused the sites that **serve history**. Method: `git grep` for `store.range(0)` across the workspace, every hit classified by *who receives the bytes*.

| # | site | who it serves | gate today | clause ④ |
|---|---|---|---|---|
| **①** | `fanout.rs:285-298` → delivered at `:348-359` | the **joining identity**, pushed | `req.new_joiner.is_some()` | 🛑 **`store.range(0)` — the ENTIRE store.** The named site |
| **②** | `fanout.rs:496-503` `collect_sync_history` | **any identity that asks**, pulled | `:496` **`space.is_member(requester)`** | 🛑 **A REJOINER PASSES THIS GATE.** She rejoined, so she is present ⇒ `:501` serves **the entire store**, paginated |
| **③** | `fanout.rs:608-618` `collect_invite_bootstrap` | a **pending invitee** | an unexpired `pending_invites` entry | ⚠️ structural-only (`is_structural_bootstrap_type`), and **§7 asks about it** |
| ❌ | `fanout.rs:662` federation delta | **a peer NODE**, not an identity | node-to-node | 📌 **Deliberately OUT.** `D-089` pairwise federation: a peer Node holds the log, and clause ④ is about what a **person** may read |
| ❌ | `runtime.rs:650 / 830 / 855 / 2241` · `app.rs:3938 / 4023` · `migration_driver.rs:132 / 194` | **nobody** — the local state fold, conflict detection, migration | — | 📌 **OUT: these never leave the Node.** *Named so the next reader does not re-derive the classification* |

🔑 ***DOOR ② IS THE FINDING.*** `:496`'s `is_member` is the **present-tense** accessor E-1 gated — and gating it **correctly** is exactly what lets a rejoiner through. 🛑 ***E-1 made door ② reachable by the person clause ④ exists to bound.*** Before E-1 the record was removed and a rejoin re-created it; the behaviour is unchanged, but the **reasoning that made ② look safe** — *"member-only, so it is fine"* — stopped being true the moment "member" acquired a history.

📌 **AND `V-4`'s SECOND HALF LANDS HERE.** `C-5b` was filed as *"`collect_sync_history` self-closes under `(i)`"* — true for a **departed** member, and **false for a returned one.** ⇒ ***`C-5b` was half a finding, and the half that was filed is the half that does not matter.***

---

## §4 — 🛑 THE SECOND FINDING: **TWO TOPOLOGICAL SORTS, AND THE BOUNDARY DEPENDS ON WHICH ONE YOU ASK**

| fn | crate | used by |
|---|---|---|
| `topological_sort` | `xgen-core/src/node/runtime.rs:2367` | **the state fold** — `:650`, `:2241`. *This is the order that decides who is a member.* |
| `topological_sort_events` | `xgen-node/src/fanout.rs:405` | **all three delivery doors** — `:290`, `:502`, `:618` |

**Two implementations, two crates, one DAG.** A topological sort chooses **one** linear extension; a DAG with concurrency has many. ⇒ ***an event concurrent with a `membership.leave` can fall on either side of the departure, and if the two sorts disagree, the slice a person is SERVED disagrees with the membership the Space COMPUTED.***

🛑 **THAT IS NOT A STYLE OBSERVATION — IT IS clause ④'s CORRECTNESS CONDITION.** Clause ④ says *"up to her departure."* **"Her departure" is a position in an order, and this leg currently has two.**

🎯 **CHAT'S PROPOSED RESOLUTION, and it is a design fork E-2's runbook must settle before a line is written:**

| | |
|---|---|
| **(A)** prove the two sorts agree and keep deriving from the delivery sort | cheapest; ⚠️ **an equality proven today is a coincidence maintained by nobody** — two functions in two crates with no test binding them |
| **(B)** ✅ **derive the boundary from the RESOLVED fold, not from the delivery sort** | the fold is already the authority on membership; asking it *"when was she present"* makes the slice agree with `left_at` **by construction**, not by coincidence |
| **(C)** unify the two sorts | correct in the long run, **out of scope**, and a whole-codebase change under a leg about privacy |

🎯 **CHAT RECOMMENDS (B), with (A)'s equality added as a REGRESSION TEST regardless** — because the two functions will keep drifting whether or not this leg reads them, and a test is the only thing that makes the drift visible. 📌 **(C) is filed, named, and not taken here.**

---

## §5 — WHAT THE SLICE ACTUALLY IS

**Presence intervals for one identity, derived from that Space's own `membership.*` events:**

- opens at **index 0**, not at her first join — 🔑 *clause ④ says "everything up to her departure", and a first-time joiner already receives the whole store today; **the change must be a NO-OP for a first-time joiner**, and that is a testable property, not a hope*
- **closes** at each departure: `MembershipLeave` where `sender == her`, or `MembershipKick` / `MembershipBan` / `MembershipNodeEject` where `content["target_identity"] == her` ⚠️ **both shapes, item 3 of §2**
- **reopens** at each `MembershipJoin` where `sender == her` and `room_id` is empty
- N leave/rejoin cycles ⇒ N+1 intervals. 🛑 **A one-boundary implementation is wrong by construction** (`§5d`(D) was refused for this reason)

✅ **AND THE FORWARD HALF NEEDS NO CODE.** *"Everything from the rejoin forward"* is already served: E-1's `fanout.rs:275-280` recipient list includes her the moment she is present again. ⇒ ***E-2 only has to close gaps in the PAST***, which is a materially smaller leg than §7 of the Leg E Phase-0 implies.

⚠️ **`N-197` WATCH, WRITTEN IN NOW:** a walk that reads only `event.sender` **silently under-counts gaps for kicked, banned and ejected members** — it produces a plausible, non-empty, wrong slice, and every test built on a `leave` fixture passes. **Its negative control must use a KICK, not a leave.**

---

## §6 — GATES

| gate | what |
|---|---|
| **W-1** | cargo floor **1629 / 0 / 62 × 56 SUITES**; delta **measured** with `--skip` on the delivered tree, never arithmetic |
| **W-2** | **All THREE doors** (§3 ①②③ per §7's ruling) carry the same predicate — **verified by opening each, not by a grep count** |
| **W-3** | 🛑 **`V-4`, INHERITED AND DISCHARGED HERE:** a test proving `C-3` (`new_joiner: Some(_)` for a rejoiner at `runtime.rs:1713`) and `C-5b` (`collect_sync_history` for a **returned** member, per §3) |
| **W-4** | **NO-OP for a first-time joiner** — byte-identical slice to today (§5) |
| **W-5** | **Two cycles, two gaps** — leave → rejoin → leave → rejoin, and both gaps closed |
| **W-6** | ⚠️ **The kick control**, not a leave control (§5's `N-197` watch) |
| **W-7** | §4's chosen resolution implemented, **and the two-sort equality regression test present regardless** |
| **W-8** | Negative controls: each disarmed separately, each RED, `Compiling` observed, sha256-restored (`N-199`, `N-124b`) |
| **W-9** | 📌 **`V-6`'s corrected rule applies:** where the change PRESERVES an assertion, **name reading as the closure mechanism** and say so in the gate |
| **W-10** | Scope: **zero `ui/**`**; the marker not invented; `topological_sort` not unified (§4(C)) |

---

## §7 — 🔓 THE ONE OPEN QUESTION. **JOE'S** (`D-155`).

A person left a Space months ago. Things happened while she was away: people joined, someone was removed, the roster changed. **Now she is coming back.** Clause ④ already rules that she does not get to read the *conversation* she missed.

### **Does she get to see the membership changes that happened while she was away — who joined, who left, who was removed?**

| | outcome, as a person would see it |
|---|---|
| **(a)** | **No — the gap is the gap.** She returns and sees the roster as it is now, with no account of how it got that way. Someone who joined during her absence simply *is* there. |
| **(b)** | **Yes — structure is not content.** She sees that three people joined and one was removed while she was away, but not a word anyone said. The Space's shape is public to its members; its conversation is not. |

🎯 **CHAT RECOMMENDS (b).**

**① USER-VISIBLE IMPACT.** Under **(a)** she returns to a roster containing strangers with **no way to learn they were ever admitted** — and the Space's own retained log says they were. It is the same disagreement between history and record that `D-154`⑥ was ruled to prevent, pointed at a returning member instead of an auditor. Under **(b)** the roster she sees is *explained* by the events she holds. ⚠️ **And (a) has a mechanical cost that is not obvious: her rejoin must CHAIN onto the DAG, and Leg G's `get_rejoin_anchor` exists because a rejoin needs a valid anchor** — an identity denied the intervening membership chain may be unable to compute one.

**② TIER.** Membership structure is already the least-protected class in this codebase: `is_structural_bootstrap_type` (`fanout.rs:549-563`) **already serves the full membership chain to a not-yet-member pending invitee**, deliberately, under INV-D1. ⇒ ***(a) would give a RETURNING MEMBER strictly less than a stranger holding an invite gets today***, which is hard to defend as a privacy position.

**③ RESOURCE COST.** **(b)** is one clause in the interval walk — membership events pass, content does not. **(a)** is simpler to state and harder to build, because of the anchoring consequence above.

⚠️ **THE HONEST CAVEAT, NAMED AND NOT TRADED AWAY.** **(b)** means *"the gap stays closed"* is **not literally true**: a returning member learns something about the period she was excluded from — specifically, **who else was removed, and when.** That is third-party information about people who never consented to her return. `D-154`④'s wording says *the gap stays closed*, and **(b) is a narrowing of it, not an application of it.** 🛑 **If (b) is ruled, it goes into `D-154` as a clarification at its own site (`D-131`), not as an unrecorded implementation choice.**

---

## §8 — WHAT LEG E-2 MUST NOT DO

1. ❌ **Not `get_rejoin_anchor`** — Leg G, a new wire verb, Joe's seat.
2. ❌ **Not the gap's MARKER** — `D-154` leaves it with Joe; E-2 builds the boundary and stops.
3. ❌ **Not unify `topological_sort` / `topological_sort_events`** (§4(C)) — filed, named, out.
4. ❌ **Not the federation delta** (`fanout.rs:662`) — node-to-node, `D-089`.
5. ❌ **Not `ui/**`.**
6. 🛑 **Not `build_membership_event`** — the empty `prev_events` is the documented root-adjacent contract (Leg E Phase-0 §4b).

---

## §9 — DoD

- [ ] §7 ruled by Joe; if **(b)**, recorded as a `D-154`④ clarification at its own site
- [ ] §4's fork settled in the runbook **before** implementation, with the two-sort regression test either way
- [ ] Runbook written, locked by Joe, implemented by Clair from the locked revision
- [ ] `W-1` … `W-10` re-driven by Chat from `HEAD` (Rule 5), none adopted on report
- [ ] **`V-4` discharged** — the gate E-1 could not close
- [ ] `roadmap-format-gate.ps1` exit 0 before any ROADMAP commit
- [ ] `D-074` atomic close

📌 **"Commit pushed" is not a DoD item.**
