# M-RP-MEMBER-ACT Leg C-bis — the member with no DM: creation, the first send, and the erased identity
> **Status**: PENDING  
> Version: 1.0  
> Date: Aug 2026  
> **Last updated**: 2026-08-08  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — WHAT THIS FILE IS, AND WHY IT EXISTS BEFORE ITS DESIGN DOES

🛑 **THIS IS A DEBT REGISTER, NOT A DESIGN.** It was created at Leg C's close (J-700) for one reason: `N-109`
requires that a limitation shipped by one milestone be written into the **DoD of the milestone that lifts
it**, not left as folklore. Leg C shipped a **dead control** and two **blocked measurements**; without this
file they would have had nowhere to land except a JOURNAL entry nobody re-reads.

⚠️ **NOTHING BELOW IS LOCKED.** No option is chosen, no approach is endorsed. Everything here is an
**obligation inherited from Leg C**, plus the ground truth needed to design against it. **The design pass is
a separate exercise and needs its own Phase-0** (`D-071`: subsystem audits precede dependent milestones).

📌 **Leg C closed at `6a6c066`** — C-1 `524d4f7` (`selectOnActivate`), C-2 `b5d0908` (`roomLatch.latch()`),
C-3 `6a6c066` (R7 acts). All three gates green; see `RUNBOOK_MEMBER_ACT_LEG_C.md` **v1.2 COMPLETED** §8.

---

## §1 — THE FOUR OWED ITEMS

### **OWED-1 — 🛑 THE DEAD CONTROL. This is the one with a user-visible cost TODAY.**

**Leg C ships a member row with no existing DM as clickable and doing NOTHING** (`RUNBOOK_MEMBER_ACT_LEG_C.md`
§5-b). Measured live at C-4: bus, latch, `rowCount` and registry all unmoved on such a click — a **silent,
complete no-op**.

⚠️ **A row that looks clickable and does nothing is a dead control** — `6.1j` and `D-113`'s correction both
bite. It was accepted **only** because it is named, owned, and time-bound **to this leg**.

🔒 **THE OBLIGATION: this leg does not close while that click is still silent.** Whatever `J-692` is ruled
to be — open a creation affordance, do nothing but say so, or refuse the click outright — **the outcome must
be that the row no longer lies about being actionable.** ⚠️ **It is legitimate for the answer to be *"the
row stops being clickable"*;** that also discharges the debt. What is NOT legitimate is closing this leg
with the no-op still in place.

📌 **`J-692` IS THE BLOCKER AND IT IS JOE'S.** *What should a click on a member with no existing DM do?* —
open, unruled since 2026-08-06. **Chat does not answer it.**

### **OWED-2 — `OQ5`: DM creation to an ERASED identity (re-sited here at J-690)**

Leg C ships the erased DM **counterpart** row clickable (`L-4` = `OQ-C2` = **E-a**, Joe): clicking re-enters
the DM you are already in. Verified live at C-4 — bus takes `kind: 'identity'`, latch unmoved, **no state
created**, no crash, erased marker and tail-8 both still rendering.

🛑 **THAT IS THE *EXISTING*-DM CASE ONLY.** Creating a *new* DM **to an identity that is erased from the
registry** is untouched and unruled.

⚠️ **JOE'S STANDING CONDITION ON `L-4`, CARRIED FORWARD VERBATIM:** what an erased counterpart's row
*shows* depends on **retention** — history-expiry and auth-tier rules. **Those do not exist in code.** Leg C
was explicitly forbidden to fake an archive, an expiry, or any retention state it cannot honour, and shipped
under that constraint. 🔒 **The milestone that builds retention owes this row its behaviour** — and if that
milestone lands before this one, **this obligation moves there.**

### **OWED-3 — the partial first send**

Out of scope for Leg C by `L-7`. Untouched, undesigned. Named here so it is not rediscovered.

### **OWED-4 — 🔓 `§6` leg 5: BLOCKED BY FIXTURE, AND STILL UNRULED**

**The behaviour:** clicking a member in a group room moves the latch to the DM ⇒ `scope` changes
(`members-panel:46`, `roomLatch.effectiveSpaceId`) ⇒ **R7 re-renders to the DM's roster: two people.**
Correct by every rule the project has; it is also **the panel you just clicked in replacing itself.**

🛑 **IT COULD NOT BE MEASURED AT C-4, AND THE REASON IS THE FIXTURE, NOT THE CODE.** Measured on Joe's
client state at `6a6c066`:

| fact | measured |
|---|---|
| Spaces carrying a `counterpart` (i.e. DMs) | **exactly one** |
| that counterpart | **erased** (present in `notFoundIds`) |
| an erased NON-counterpart member in a group room | **hidden** — `§5a` E2 protects it only *inside* the DM, where `counterpart` is non-null |
| ⇒ rendered group-room rows with an existing DM | **ZERO** |

⇒ **no reachable input produces leg 5.** `§6` leg 1's full sentence (*group room → click → R5 shows that
DM*) is blocked for the same reason and was driven only in its in-DM form (= `§6` leg 4).

🔒 **IT STAYS UNRULED — `D-146`: a lock waits until its stated cost expires, and nothing has expired.**
Ruling now would be a judgement about something **nobody has yet seen on screen.**

📌 **THE OBLIGATION IS SHARED, DELIBERATELY.** The first of **this leg** (which creates DMs) or
**`M-RP-PEOPLE`** (which surfaces people independent of Space) to land makes a second, non-erased DM
reachable. **Whichever lands first owes the measurement, and owes SHOWING IT TO JOE before anyone rules.**

---

## §2 — GROUND TRUTH INHERITED (re-measure; do not trust these lines)

🛑 **CITE THE SITE, RE-MEASURE THE LINE.** Leg C's own runbook staled its `members-panel` line numbers
during its own execution — C-3 shifted them by roughly +11 and +40. **Every number here is a starting
point.**

| # | fact | site |
|---|---|---|
| **B1** | `findDmRoom` — the named local lookup, field match on `counterpart`, self excluded explicitly | `members-panel.svelte` |
| **B2** | `onMemberActivate` — `latch()` then `selection.set()`, two independent stores, NOT ordering-dependent | `members-panel.svelte` |
| **B3** | `KnownSpace.counterpart` — *"DM counterpart XGID, or the session identity for the self thread. `null` for a Space."* | `spaces-state.svelte.ts:32-34` |
| **B4** | `roomLatch.latch(roomId)` — the direct writer; **three** writers total (`note`, `clear`, `latch`) | `room-latch.svelte.ts` |
| **B5** | `entity-panel` `selectOnActivate` — default `true`; R7 passes `false` to keep its highlight DERIVED | `entity-panel.svelte` |
| **B6** | The erased-row composition problem — two marks stack on one string | `N-168` (FILED, NOT FIXED) |
| **B7** | `latch()` unconditional / `selection.set()` guarded — the half-apply shape | `N-171` (FILED, NOT FIXED) |

---

## §3 — WHAT THIS LEG MUST NOT DO

🛑 **IT MUST NOT FAKE RETENTION.** History-expiry and auth-tier rules do not exist in code. No archive
state, no expiry state, no "this conversation has been cleared" that nothing behind it can honour.

🛑 **IT MUST NOT RE-OPEN `L-3`, `L-5` OR `L-9`.** `selectOnActivate` default `true`, R7's highlight stays
**derived**, and no new store or bus. `D-146` — a locked option re-opens when its **stated cost expires**,
not when taste changes, and the supersede must name the expired cost.

🛑 **`M-RP-PEOPLE` IS NOT THIS LEG.** Filed separately; never named `contacts` (`address_book.rs:38`
reserves that word for Ch2's private contact record). ⚠️ `last_seen` must **never** render as the person's
activity.

---

## §4 — DoD (to be extended by this leg's own design pass)

- [ ] **`OWED-1` discharged** — a member row with no existing DM **no longer presents as actionable while
      doing nothing**, by whichever route `J-692` is ruled. **Verified on the live client, not inferred.**
- [ ] **`OWED-2` addressed or explicitly re-sited** to the retention milestone, named in that milestone's DoD.
- [ ] **`OWED-3`** designed or explicitly deferred with an owner named.
- [ ] **`OWED-4` measured and SHOWN to Joe** if this leg makes a second non-erased DM reachable — **shown
      before any ruling**, per `D-146`.
- [ ] `N-171`'s half-apply corrected **if** `onMemberActivate` is opened by this leg (move the lookup above
      `latch()`; the locked write ORDER is untouched).
- [ ] Floors held: cargo · svelte-check · catalogue · client registry quiescent baseline — **re-measured,
      never inherited.**
- [ ] Joe's client state file **byte-identical**, read before and after.

⚠️ **NO "commit pushed" ITEM.** It is unflippable inside the commit that performs the push;
`Status: COMPLETED` is the real signal.
