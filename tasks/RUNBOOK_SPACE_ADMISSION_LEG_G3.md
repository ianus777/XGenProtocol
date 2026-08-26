# RUNBOOK — M-SPACE-ADMISSION Leg G-3: the door
> **Status**: ACTIVE  
> Version: 1.1  
> Date: Aug 2026  
> **Last updated**: 2026-08-26  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — WHAT THIS LEG IS

**`M-SPACE-ADMISSION` Leg G-3 — the door.** `collect_invite_bootstrap`'s **authorization** widens so that a retained departed member may fetch what she needs to anchor a rejoin. **Joe ruled the wire shape at session open: option ② — one more key on the door that exists, no new `transport.*` variant.**

📌 **Phase-0:** `tasks/M_SPACE_ADMISSION_LEGG_PHASE0.md` v1.4 §2.3 + §4. 🔒 **Anchor commit `8741721`.**

🛑 **`G-3` ON ITS OWN CHANGES NOTHING A USER CAN SEE, AND THAT IS BY DESIGN.** `get_invite_bootstrap` (`xgen-client/src/batch.rs:262`) scans the served batch for a `MembershipInvite` naming the requester, **and a rejoiner has none** — hers was consumed at her first join. **`G-4` is what makes her pick a different anchor out of this batch.** ⇒ ***a leg whose whole effect is invisible until the next leg lands is a leg that will be reported as done and measured as nothing***, so this runbook's DoD asserts what the NODE SERVES, never what a user experiences.

🛑 **THIS RUNBOOK IS CLAIR'S. 🔒 LOCKED BY JOE 2026-08-26 — IMPLEMENT FROM THIS VERSION, IN A SESSION OPENED BY HER OWN KICKOFF.** Deviations are **reported, never absorbed** (Rule 6). 🔒 **§3 IS RULED: OPTION ② — ONLY HER OWN MEMBERSHIP EVENTS.** ✅ **VERIFIED AT LOCK: `HEAD` is still `8741721` and NO `.rs` has moved since** — every site below is live against the tree you will edit.

---

## §1 — 🛑 THE FINDING THAT SHAPES THE LEG: THE INVITE REQUIREMENT IS DOING TWO JOBS

✅ **MEASURED at `xgen-node/src/fanout.rs:751`:**

```
let pending = space.pending_invites.get(requester_id).ok_or(REFUSED)?;
```

🛑 **THERE IS NO BAN CHECK ANYWHERE ON THIS PATH.** ✅ `banned` has **zero occurrences in `fanout.rs`**. ✅ **The node's transport arm has none either** — its own comment says so: *"Authorization + the `valid_until` read-gate live inside collect_invite_bootstrap"* (`xgen-node/src/app.rs:1788-1798`). ✅ **The dispatch-level pre-check at `runtime.rs:1523` guards EVENT SUBMISSION, not transport requests** — it is not on this path at all.

🔑 **A BANNED IDENTITY IS EXCLUDED TODAY ONLY AS A SIDE EFFECT: she holds no pending invite, so line `:751` refuses her `1011`.** ⇒ ***the single line is proving entitlement AND excluding the banned, and widening it splits those two jobs apart. Only one of them is being replaced.***

🛑 **`left_at.is_some()` IS TRUE FOR A BANNED AND FOR A NODE-EJECTED IDENTITY**, not only for a leaver and a kicked member. ⇒ **a naive widening hands the Space's whole membership chain to someone the Space has permanently excluded.**

🔒 **⇒ `G-3` MUST CARRY A BAN TERM. THIS IS THE EXACT INVERSE OF `G-1`, AND THE DIFFERENCE IS MEASURED, NOT STYLISTIC:** in `G-1` a ban clause would have been a second source of truth because the pre-check already ran upstream in the same function; **here nothing runs upstream at all.** ⚠️ ***The same clause is a defect in one leg and a requirement in the other, and the only way to tell which is to open the path.***

📌 **`is_structural_bootstrap_type`'s doc comment already anticipates the ban in the PAYLOAD** — *"a banned identity must not bootstrap as if clean"* — **but that is disclosure OF the ban, not a gate against the banned.** *A payload that tells you someone is banned is not a check that stops them reading it.*

---

## §2 — GROUNDING. **OPEN EACH SITE BEFORE YOU EDIT (`D-153`).**

| # | fact | anchor |
|---|---|---|
| **D-1** | `collect_invite_bootstrap` refuses on: unknown Space · **no pending invite (`:751`)** · expired invite. `REFUSED = (1011, "invite_bootstrap_refused")` | `fanout.rs:744-782` |
| **D-2** | The `valid_until` read-gate sits **inside the pending-invite branch** ⇒ it has nothing to say about a requester who holds no invite | `fanout.rs:759-771` |
| **D-3** | `is_structural_bootstrap_type` serves the creates plus the whole membership chain; **content, MLS, pacing and `MembershipMute` are deliberately excluded (the INV-D1 privacy line)** | `fanout.rs:615-638` |
| **D-4** | 🔑 **The `D-154`④ presence-interval filter is NOT applied on this path**, and it would be a **no-op** if it were: its job is to withhold **content** during an absence, and **this path serves no content by construction** | `fanout.rs:640-706` vs `:772-780` |
| **D-5** | `collect_sync_history`'s gate is `if !space.is_member(requester_id)` — **present-tense**, so a former member is refused there and stays refused. **This leg does not touch it.** | `fanout.rs:510-517` |
| **D-6** | 🛑 **ch3 documents this verb in FOUR places, all saying *pending invitee*:** the `1011` row `:1237` · the prose at `:1318-1320` · **the *Authorization and the served set* paragraph `:1335`** · the M8.5-B summary `:2442` | `docs/xgen_ch3_specification.md` |
| **D-7** | `apply_ban` and `apply_node_eject` insert into `space.banned`; **`apply_kick` does not** ⇒ one `banned` test covers both permanent exclusions and leaves a kicked member eligible | `xgen-core/src/space/state.rs` |

---

## §3 — 🔒 RULED (Joe, 2026-08-26): **② — ONLY HER OWN MEMBERSHIP EVENTS.** THE QUESTION, AND WHY IT WAS ASKED

**She left the Space months ago. She reinstalls, and before she is a member again her client asks the Space for what it needs to put her back. While she is standing outside, what does the Space hand her?**

**Today, an invitee gets the whole membership chain — every join, leave, kick, ban and ejection the Space has recorded.** The question is whether a **former member standing outside** gets the same.

| | what a person would see | |
|---|---|---|
| **① The same set an invitee gets** | Her client receives the full membership chain, **including who else was removed while she was away, and when.** She never sees that she holds it; the people named never consented to her return and cannot see that she has it. | One term changes. The verb serves one fixed payload to everyone it admits. |
| **② Only her own membership events** | She gets exactly what anchors her rejoin — her own invite, join, leave, kick — **and learns nothing new about anyone else until she is actually back in**, at which point `D-154`④ governs as it already does. | One term plus a filter on the collected set. |

🎯 **CHAT RECOMMENDS ②.** **Lens ①:** her rejoin experience is **identical** either way — what differs is what a person **outside the room** learns about **other people**. **`D-154`④-as-clarified ruled what a RETURNING member receives; a requester who has been admitted by the gate but has not yet rejoined is not yet returning, and ① would widen that disclosure from *after readmission* to *on request, while still outside*.** ⚠️ **That widening is real, it is not what ④ decided, and it should not arrive as a side effect of a mechanism change.** **Lens ②:** ① places a durable copy of third-party membership history on the device of a non-member — **the arc's carried caveat ②, made materially worse by being pre-admission.** ② creates no such copy. **Lens ③:** ② is a filter over a set already in hand — **cheaper, because it serves fewer events.**

⚠️ **THE HONEST CAVEAT FOR ②, NAMED AND NOT TRADED AWAY:** **the door's payload stops being one fixed set and starts depending on who knocks.** That is a real complexity cost, it must be documented at `wire/types.rs:168` and in ch3, and **it is the kind of divergence that later gets forgotten by someone reading only one branch.** 📌 **② also means `G-4`'s client must not assume the full chain is present** — the runbook for `G-4` will state that either way, but under ① it would be an assumption that happens to hold.

🔒 **BOTH ARMS WERE FULLY SPECIFIED IN §4 AND JOE RULED ②. 🛑 ARM ① IS DEAD — `G3-2` HAS ONE BRANCH NOW, AND `V-7` IS MANDATORY, NOT CONDITIONAL.**

---

## §4 — THE EDIT

### G3-1 — the authorization

**The refusal at `:751` becomes: the requester must hold a pending invite **OR** be a retained departed member who is **not banned**.** ✅ **Sketch, not a paste target:**

```
let is_former_member = space
    .members
    .get(requester_id)
    .is_some_and(|m| !m.is_present())
    && !space.banned.contains(requester_id);

match space.pending_invites.get(requester_id) {
    Some(pending) => { /* existing valid_until read-gate, unchanged */ }
    None if is_former_member => { /* no invite, no expiry to check */ }
    None => return Err(REFUSED),
}
```

🔒 **`!m.is_present()` IS `G-1`'s AND `G-2`'s TERM, RE-READ, NOT RE-SPELLED** — `SpaceMember::is_present()`, `D-067`'s one fact in one place. **Third site, same spelling.**
🔒 **THE BAN TERM IS REQUIRED HERE (§1) AND MUST NOT BE COPIED FROM `runtime.rs`** — read `space.banned` directly; there is no upstream check to defer to.
🛑 **The `valid_until` gate stays exactly where it is, inside the invite arm.** A former member has no invite and therefore no expiry — **do not invent a substitute deadline.**

### G3-2 — the served set. 🔒 **§3 RULED ② — THIS IS THE ONLY BRANCH.**

⚫ **~~Under ①: nothing changes; the existing filter serves everyone the same set.~~ REFUSED at §3. Struck rather than deleted so a later reader can see the alternative was considered and rejected, not overlooked.**

🔒 **UNDER ②, WHICH IS THE RULING:** after collecting the structural set, **filter to events that name the requester** — her own `invite`/`join`/`leave`/`kick`, plus the Space/Room creates that make the batch parseable. 🛑 **The "names the requester" test must be read off the same field each event type actually uses** (`sender` for a join or leave; the target field for an invite or kick) — **measure it per type; do not assume one field.** ⚠️ **If a type's target field turns out ambiguous, that is a finding, not a thing to guess.**

### G3-3 — the meaning, in the two places that carry it

🔒 **THE WIRE STRING DOES NOT CHANGE. THE DOCUMENTATION OF WHAT IT MEANS DOES.** This is §4's caveat from the Phase-0, and it is the whole price of option ②-at-session-open: **`transport.invite_bootstrap_request` now serves a requester who holds no invite.**

1. **`xgen-core/src/wire/types.rs:168`** — the variant's doc comment describes the invitee case only. **Restate it as *bootstrap for someone entitled to enter*, naming both routes.**
2. **`docs/xgen_ch3_specification.md`, ALL FOUR SITES from `D-6`** — `:1237`, `:1320`, **`:1335`'s *Authorization and the served set* paragraph**, `:2442`. 🛑 **A census, not a search-and-replace: `D-6` lists four, and a fifth would be a finding.** 📌 **Annotate rather than overwrite where a sentence is being narrowed (`D-131`); the header convention applies (it bit `G-2` — `CLAUDE.md:1945`).**

---

## §5 — VERIFICATION

📌 **Tests belong beside the existing bootstrap tests in `xgen-node/src/fanout.rs`'s test module** — that is where `collect_invite_bootstrap`'s current coverage lives.

| # | check | requirement |
|---|---|---|
| **V-1** | **THE SUBJECT.** A departed member, holding **no invite**, calls `collect_invite_bootstrap`. | **`Ok`**, and the batch contains **her own last membership event** — the thing `G-4` will anchor on. |
| **V-2** | 🔒 **THE BAN CONTROL, AND IT IS THE ONE THIS LEG EXISTS TO GET RIGHT.** A **banned** former member, same call. | **`Err((1011, "invite_bootstrap_refused"))`.** 🛑 **Without it, the widening hands the membership chain to someone the Space permanently excluded.** |
| **V-3** | **THE EJECTION CONTROL.** A **node-ejected** former member. | **`Err`** — via the same `banned` test (`D-7`). |
| **V-4** | **THE KICK CONTROL.** A **kicked** member (not banned). | **`Ok`** — she is eligible to return (`D-154`②③), so she is eligible to fetch her anchor. |
| **V-5** | **THE STRANGER CONTROL.** Never a member, no invite. | **`Err`** — unchanged behaviour. |
| **V-6** | **THE INVITEE IS UNTOUCHED.** Every existing `collect_invite_bootstrap` test stays **GREEN and UNEDITED**, including the expiry cases. | 🛑 **If any must be weakened, that is a FINDING, reported and never absorbed.** |
| **V-7** | 🔒 **THE DISCLOSURE CONTROL — MANDATORY, §3 RULED ②.** A third party's `leave`/`kick`/`ban`, occurring during her absence, is in the store. | **It is NOT in her batch.** 🔑 **This is §3's ruling made falsifiable; without it, ② is an intention rather than a behaviour.** |
| **V-8** | 🛑 **RED-ON-REVERT.** Remove the `is_former_member` arm. | **`V-1` and `V-4` go RED with `Err((1011, …))`** — **the observed code recorded, not just the failure.** Then remove only the **ban term**: **`V-2` and `V-3` must go RED by returning `Ok`.** ⚠️ **Both reverts run separately: one revert proving two independent terms is not a control, it is a coincidence.** |

🔒 **FLOOR: cargo `1648 / 0 / 62 × 56 SUITES` at `8741721`. IT MUST MOVE.** **Delta with `--skip` plus libtest's own `filtered out`, never arithmetic.**
🛑 **Detached + sentinel · `--no-fail-fast` · `^test result:` summed CASE-SENSITIVELY · require `Compiling xgen-node` in the log.** 🔑 **`N-207`: a run that returns the RIGHT number over a binary you did not build is not a measurement — the result line cannot distinguish *my source passed* from *your binary passed*.** 📌 **`N-204`: chunk your writes; the ~70-line figure is a working rule, not a measured threshold.**

---

## §6 — WHAT THIS LEG MUST NOT DO

1. 🛑 **No new `transport.*` variant, no wire-string change.** Session-open ruling ②; **the meaning is restated, the name is not.**
2. 🛑 **No touch to `collect_sync_history`'s `is_member` gate** (`D-5`). A former member is not a member; `D-154`④ governs what she reads once she is back, and `E-2` already built it.
3. 🛑 **No client change.** `ops::join`'s anchor selection is `G-4`. **Do not make this leg look effective by reaching into `batch.rs`.**
4. 🛑 **No second definition of *departed* and no second definition of *banned*.** Read `is_present()` and `space.banned` directly.
5. 🛑 **Do not invent an expiry for the former-member arm.** She has no invite to expire.
6. 🛑 **No change to `is_structural_bootstrap_type`'s type set.** Under ② the narrowing is a filter on instances, **not** a change to which types are structural.
7. 🛑 **No ch3 edit beyond `D-6`'s four sites, the header, and any fifth site the census turns up** — *and a fifth site is a FINDING to report, not a silent extra edit.* 📌 **Written this way deliberately: `G-2`'s §5.7 fenced the header out along with scope, and the standing convention won.**

---

## §7 — DoD

- [x] §3 ruled by Joe (② — only her own membership events, 2026-08-26) and the ruling recorded at its site before implementation begins
- [ ] `G3-1` shipped in `xgen-node/src/fanout.rs`, **ban term included**, comment carrying §1's finding — *the invite requirement was doing two jobs and only one is being replaced*
- [ ] `G3-2` shipped as §3 ruled
- [ ] `G3-3`: `wire/types.rs:168` restated, and **all four ch3 sites** updated with a census confirming four
- [ ] `V-1` … `V-8` run and green; **`V-8`'s two reverts run SEPARATELY**, observed codes recorded
- [ ] `cargo` moved from **1648 / 0 / 62 × 56 SUITES**, delta measured with `--skip` and `filtered out` cross-checked, **`Compiling xgen-node` present in the log**
- [ ] `vitest` / `svelte-check` carried **by scope**, proven by `git diff --name-only`
- [ ] **The `G-3`-alone-is-invisible limit restated in the hand-back**, alongside `G-2`'s standing limit: **`3048` has still never been observed on a wire**
- [ ] Every deviation reported, none absorbed (Rule 6); Chat re-drives every number from `HEAD` (Rule 5)

📌 **"Commit pushed" is deliberately not a DoD item** — `Status: COMPLETED` in this header is the shipped signal. **Clair never pushes.**
