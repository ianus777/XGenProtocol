# M-RP-SELECT-ORIENT — the panels keep saying where you are — Phase-0
> **Status**: ACTIVE  
> Version: 1.2  
> Date: Aug 2026  
> **Last updated**: 2026-08-08  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — THE ONE SENTENCE

**Select a room and the Space stays lit. Select a member and the room stays lit.** Today both go dark, because R1 and R2 read their highlight from the **selection bus** — a single global slot holding whatever was clicked last — instead of from the latch that actually knows where you are.

🔒 **THIS MILESTONE LANDS BEFORE `M-RP-MEMBER-ACT` LEG C.** Leg C's `L-7` puts an **identity** on the bus on every member click; if the panels still read the bus, every click extinguishes the room highlight. **Fixing it after Leg C means Leg C's live verify cannot tell whether a highlight moved because of `L-7` or because of the fix.**

⚠️ **ONE HALF OF THIS IS A DEFECT AND THE OTHER HALF IS A SUPERSEDE. They are not the same kind of change and this document does not blur them.**

---

## §1 — PROVENANCE, AND A CORRECTION THAT CAME FIRST (`D-141`)

🛑 **CHAT CALLED R1's BEHAVIOUR A DEFECT. IT IS NOT — IT IS A LOCK JOE UTTERED.**
`M_RP6_2_SPACES_ROOMS.md:202`, **LOCKED 2026-07-17** on *"go by your recommendation"*: **`D4` = opt-1 (bus-pure)** — R1 highlights only while the bus holds that Space. `:128` names the rejected alternative and its reason — ***"Simplest, single truth, no second latch"*** — and `:131` files **opt-2** as *"a later polish pass if Joe wants the persistent 'which space am I in'."*

🔑 **THE ERROR WAS NOT SEARCHING BEFORE ASSERTING.** Chat told Joe *"R1 has the same defect, shipped today"*, Joe scoped the milestone on that sentence, and only the fix-side grounding found the lock. **Corpus searched after the fact: every `*.md` in the repo for `opt-1` / `bus-pure`.** *This is the milestone's first finding and it is about Chat, not the code.* 📌 **Joe's reading was reasonable because Chat's own bullet asserted it; the mis-scope is Chat's, not a miscommunication.**

### 🔒 WHAT JOE HAS UTTERED

| # | ruling | how |
|---|---|---|
| **J-1** | **The milestone is named `M-RP-SELECT-ORIENT`** | uttered |
| **J-2** | 🔒 **`D4` opt-1 IS SUPERSEDED. Selecting a room KEEPS the Space lit.** | ***Restated in the 2026-08-07/08 session AFTER being shown the July lock and told plainly that changing it supersedes `D4`.*** Recorded as **Joe's by utterance**, not inferred |
| **J-3** | Scope is **R1 and R2**, and **`OQ-C3`** (the `activeIndex` staleness) folds in | uttered |
| **J-4** | This milestone lands **before Leg C** | uttered, adopting Chat's recommendation |

📌 **`D4`'s STATED COST HAS EXPIRED, WHICH IS WHY THIS IS A RE-PRICE AND NOT A CHANGE OF TASTE.** opt-1 won on *"no second latch"*. At the time opt-2 meant R1 growing its own. **It no longer does** — R2's Space latch has existed since M-RP6.2 and `roomLatch` since M-RP6.3 Leg D2. *The option was priced once, in July, and never re-checked — this project's most repeated error class.*

---

## §2 — GROUNDED, MEASURED AT `7305df2` (overnight session 2026-08-07/08)

📌 **THE SESSION CROSSED MIDNIGHT.** Where a measurement is dated below, it is dated to the **session**, not to a clock reading Chat did not take. *Two probe runs sit either side of 00:00 and Chat did not timestamp them individually; claiming a date per probe would be precision it does not have.*

| # | fact | site |
|---|---|---|
| **S1** | R1's highlight: `selection.current?.entity.kind === 'space' ? …id : undefined` — **bus-derived** | `spaces-panel.svelte:44-46` |
| **S2** | R2's highlight: `selection.current?.entity.kind === 'room' ? …id : undefined` — **bus-derived** | `rooms-panel.svelte:42-44` |
| **S3** | 🔑 **R2 ALREADY SOLVED THIS ONCE, LOCALLY.** `let latchedSpaceId = $state<string \| null>(null)` + an `$effect` writing it only on a `space` selection — because *"clicking a room would blank R2's own list"* | `rooms-panel.svelte:23-27` |
| **S4** | 🛑 **THAT LATCH IS PRIVATE TO R2.** No export, no store; R1 cannot read it | `rooms-panel.svelte` |
| **S5** | 🛑 **AND FOLDING IT INTO `roomLatch` IS EXPLICITLY FORBIDDEN**: *"THERE ARE TWO LATCHES AND THE NAME HIDES IT … R2's Space latch is NOT touched by this store and must not be folded into it."* | `room-latch.svelte.ts` header |
| **S6** | `roomLatch` exposes `latchedRoomId` (raw, *"resolvable or not — a verify/debug surface"*), `effectiveRoomId` / `effectiveSpaceId` (*"the room BOTH R5 and R6 act on"*), `canSend` — **all from ONE `resolveLatched()`** | `room-latch.svelte.ts:41-77` |
| **S7** | 🔑 **`roomLatch` IS ITSELF THE PRECEDENT FOR THE LIFT** — R5's room latch was lifted into `$common` for R6, and *duplicating it was REJECTED as a `D-067` drift surface* | `room-latch.svelte.ts` header |
| **S8** | R5 reads `effectiveRoomId` (`:70`); R6 reads `canSend` (`:56`), `effectiveSpaceId` + `effectiveRoomId` (`:68-69`) ⇒ **the stream's content, the composer's target and its enablement are already ONE resolution** | `stream-panel.svelte` · `composer-panel.svelte` |
| **S9** | **The bus has SIX IMPORTERS** — `inspector-panel:25` · `self-panel:21` · `rooms-panel:12` · `spaces-panel:21` · `app_client:47` · **`room-latch.svelte.ts:36`**. 🛑 **CORRECTED 2026-08-08 (Clair's F7): this row said FIVE, but the grep it cites returns SIX.** `room-latch` reads `selection.current` only as `note()`'s **default argument** and `app_client:197` always passes it explicitly ⇒ **dormant in production, but the stated METHOD returns six** | measured, `grep "stores/selection"` over `ui/**` |
| **S10** | 🛑 **`selection.svelte.ts:12` STANDS FALSE**: *"R8 (inspector) is the only consumer."* **It is the Leg A annotation of 2026-08-06 — an annotation that corrected one false claim and introduced another** | `selection.svelte.ts:8-12` |
| **S11** | The bus→latch bridge is an **`$effect`** reading `selection.current` | `app_client.svelte:195-198` |

### 🔬 MEASURED LIVE ON THE CLIENT (9222), NOT READ

- 🛑 **THE DEFECT, DRIVEN.** Click a room ⇒ `roomsSel` = the room id, `latched` = the room. **Write an identity to the bus — exactly what `L-7` does ⇒ `roomsSel` → `null`, `latched` UNCHANGED.** ⇒ ***the composer keeps targeting the room while the room stops looking selected.***
- ✅ **REACHABLE TODAY WITHOUT LEG C.** Clicking the **R3 self card** — a shipped gesture (`self-panel:77` writes an identity selection) — reproduces it exactly. **This milestone fixes a live defect, not a hypothetical one.**
- 🔒 **AND THE TWO SPACE SOURCES DIVERGE, MEASURED.** Engineering → room `random` → click Design ⇒ R2 lists **Design's** rooms (`latchedSpaceId` moved) while `effectiveSpaceId` stayed **Engineering** (the room latch did not). *Recorded in J-691 as correct two-latch behaviour, which it is — and it is also why §4's source choice is not free.*

---

## §3 — THE TWO HALVES, KEPT APART

### ✅ 3.1 — R2 (rooms panel): A DEFECT, AND NOBODY RULED ON IT

`D4`/`D5` were taken when the bus held **spaces and rooms only**. **Identity selections arrived later with R3 and become common with `L-7`.** ⇒ **no decision covers "what does R6 do when the bus holds an identity", and the shipped answer — go dark while still being the room you are typing into — is one nobody chose.**

### 🔒 3.2 — R1 (spaces panel): NOT A DEFECT. A SUPERSEDE (`J-2`)

R1 un-highlighting on a **room** selection is `D4` opt-1 working exactly as locked and verified (`M_RP6_2_SPACES_ROOMS.md:219`, V5). **Changing it is overturning a July ruling, and `J-2` overturns it.** ⚠️ **The `D4` annotation and the new `D-146` are therefore load-bearing records of this milestone, not paperwork.**

### ⚠️ 3.3 — `OQ-C3`: A THIRD, UNRELATED DEFECT RIDING BY JOE'S CHOICE (`J-3`)

`entity-panel`'s `activeIndex` is seeded once (`:94`), never clamped; `tabindex` is `0` only where `i === activeIndex` (`:183`). **Driven in-session: shrink the list below the stored index and NO row is tabbable, `focusables` = 0, while `role="listbox"` stands.** Escapable **only with a mouse**. 📌 *Different mechanism, same theme — the panels stop telling you where you are, or cannot be reached at all.*

---

## §4 — THE SHAPE, AND THE ONE REAL CHOICE INSIDE IT

**R2 is settled. R1 is not, and §2's divergence is why.**

### ✅ R2's SOURCE IS SETTLED — `effectiveRoomId`, not `latchedRoomId`

`latchedRoomId` is *"resolvable or not — a verify/debug surface"*; `effectiveRoomId` is *"the room BOTH R5 and R6 act on"*. Using the raw latch would let the row stay lit while R5 says *"select a room."* ⇒ **`effectiveRoomId` puts R5's content, R6's target, `canSend` and the highlight on ONE resolution, so they can never disagree.** 🔒 **Chat's, taken under `D-123` — technical execution, no user-visible ambiguity.**

### 🔓 R1's SOURCE — §5-OQ-1, AND IT IS THE ONLY THING OPEN

Two candidate meanings, and **§2 measured them diverging**:

| | source | R1 lights | on the divergence case |
|---|---|---|---|
| **A** | a **lifted Space latch** (S3's value, shared) | the Space you are **browsing** | click Design ⇒ **Design lights, Design's rooms list. One story.** |
| **B** | `roomLatch.effectiveSpaceId` | the Space you are **talking in** | click Design ⇒ **ENGINEERING lights while Design's rooms list. The panel contradicts itself.** |

📌 **Chat's recommendation: A.** *It is the value R2 already computes and already trusts; B makes R1 disagree with the list directly beneath it.* ⚠️ **But A is the more expensive one and its cheap route is walled off** — S4 (private) and **S5 (folding it into `roomLatch` is forbidden by that file's own instruction)** ⇒ **A means a NEW `$common` store, and R2 must then read the lifted value rather than keep its own copy, or `D-067` drift is guaranteed.**

🔒 **A IS EXACTLY THE `roomLatch` LIFT REPEATED (S7)** — the same problem, the same remedy, and duplication was rejected there for the same reason. *That is precedent, not novelty.*

---

## §5 — 🔒 OQ-1 IS CLOSED AS OPTION A. `D-121` lenses: ① user-visible impact, then ② resource cost.

🔒 **CLOSED 2026-08-08 — AND IT WAS NEVER PROPERLY OPEN.** Chat settled it earlier the same session (*"This isn't a design question. It's plumbing… That was mine to settle, and I escalated it to you instead"*), then **re-opened it on Clair's F2 — a non-sequitur.** 🔑 ***B was never refused on soundness. B was refused because it does not do what Joe asked for.*** F2 corrected the REASON; it did not revive the option. ⇒ **`D-147`.**

🔒 **THE REQUIREMENT, JOE'S WORDS:** *"when user select room, space will deselect … i would like to fix it also in the spaces panel"* and *"we will be able to orient ourselves."* ⇒ **R1 lights the Space you are BROWSING.**

### 🔓 PRIOR ART — THIS WAS PUT TO JOE ONCE BEFORE AND NEVER REACHED DISK

⚠️ **An earlier session framed it correctly and Chat did not search for it.** That session carried the item as *the identity case — what R1 and the rooms panel do when the bus holds an identity, **never contemplated by `D4`*** — i.e. ***it already knew `D4` was a LOCK, not a defect*** — and offered **S-1** (identity case only, `D4` stands) vs **S-2** (S-1 plus overturn `D4` to opt-2). **Chat recommended S-2; Joe's decision was pending at close; nothing was written to disk.**

🔑 **`J-2` IS S-2, REACHED AGAIN BY A WORSE ROUTE.** This session called `D4` *"a defect, shipped today"* — **strictly worse than the earlier session's grasp of the same ground** — and Joe scoped the milestone on that sentence before the lock was found. 📌 **The outcome converged; the path did not.** ⇒ **`D-147` covers this too: a question that never reached disk is still a decision surface.**

### 🔒 A — LIFT R2's SPACE LATCH INTO A `$common` STORE. **TAKEN.**

① R1 lights the Space you are browsing; **R1 and R2 always agree, because they read one value.** The behaviour `J-2` describes.
② A new store (~30 lines), R2 edited to read it instead of its private copy, R1 edited. **Two shipped widgets touched.**
🔑 **IT IS THE `roomLatch` LIFT REPEATED (S7)** — same problem, same remedy; duplication was rejected there for the same reason.

### 🛑 B — R1 READS `roomLatch.effectiveSpaceId`. REFUSED, AND THE REASON MATTERS.

🔒 **REFUSED BECAUSE IT DOES NOT MEET THE REQUIREMENT** — `effectiveSpaceId` is the Space you are **talking in**; Joe asked for the Space you are **browsing**. **Measured diverging: click Design while latched to an Engineering room ⇒ R1 lights ENGINEERING while R2 lists Design's rooms.**
⚠️ **v1.0 refused it as *"unsound ⇒ `D-143`"*. THAT WAS WRONG (Clair's F2)** — B is deterministic and verifiable; `D-143` does not fire, and it was the **third** `D-143` misapplication this milestone. 📌 ***A corrected refusal-reason is not a restored option.***

### 🛑 C — R1 GROWS ITS OWN PRIVATE LATCH. REFUSED.

Two copies of one rule — **the `D-067` drift `roomLatch`'s header rejects by name.**

### 🔓 D — A TWO-STATE HIGHLIGHT. **FILED, NOT REFUSED. AN EXTRA, NOT AN ALTERNATIVE.**

**R1 shows the BROWSED Space one way and the TALKING-IN Space another.** 🔑 **Named in the very lock being superseded** — `M_RP6_2_SPACES_ROOMS.md:129` rejected opt-2 partly because it *"invents a second 'active vs selected' concept in R1"*. **That invention is D.**

🔒 **D IS A PLUS A SECOND VISUAL STATE.** ① R1 tells you both things at once. ② A's plumbing **plus** B's, plus a second `selected`-like concept in `entity-panel` that **does not exist** (it gates one `selected` per list), plus `skin.css` — **Joe's**.

📌 **Chat's recommendation: NOT NOW** — it delivers more than Joe asked for at the cost of a concept the panel does not have. 🔒 **FILED exactly as `D4` opt-2 was filed — and that filing is precisely why it existed to be found.**

### 🔒 THE LIFTED STORE'S WRITE RULE — SETTLED, CHAT'S (`D-123`)

**SPACE-SELECTION ONLY**, matching R2's rule at `rooms-panel:24-27`. 🔑 **Room-resolves-to-Space would import B's behaviour, which the requirement rejects** (Clair's F4 traced them as one axis). ✅ **THIS DISCHARGES CLAIR'S F6:** commit 1 changes no rendered behaviour, so commits 1 and 2 keep their clean before/after.

⚠️ **NOT SETTLED, NAMED AND OWED:** under Leg C, clicking a member enters a DM room whose Space was **never clicked** ⇒ R2 would list the previous Space's rooms. **Real; it is the DM home's problem — `M-RP-MEMBER-ACT` Leg E owns it.**

### 📌 SETTLED WITHOUT ASKING (`D-123`, so the record says who took them)

- **R2 ⇒ `effectiveRoomId`** — §4.
- **✅ DONE, NOT PENDING (Clair's F8):** `selection.svelte.ts`'s reader annotation **shipped at `d5c87c4`** and was **corrected again 2026-08-08** — it said *"R1/R6"* and *"five readers"*; it now says **R1/R2** and **six importers**. 📌 **The record work still OWED is the milestone-CLOSE annotation dropping the count once R1/R2 leave the bus — named here because §5 and §6 did not name it.**
- **`M_RP6_2_SPACES_ROOMS.md` is `COMPLETED`, not `ARCHIVED`** ⇒ **`D-145`: the `D4` supersede is ANNOTATED AT THE SITE**, not worked around.
- **A new project-wide `D-146`** records the supersede with the expired-cost rationale.

---

## §6 — PROPOSED COMMIT ORDER (`D-074`)

| # | commit | floor | gated on |
|---|---|---|---|
| **1** | 🆕 the lifted Space-latch store + **R2 reads it** (its private copy removed) | svelte-check · catalogue | **OQ-1** |
| **2** | **R1 highlights from it** — the `J-2` behaviour | svelte-check · catalogue | 1 |
| **3** | **R2 highlights from `roomLatch.effectiveRoomId`** | svelte-check · catalogue | — (independent of 1–2) |
| **4** | **`OQ-C3`** — clamp `activeIndex` against `items.length` in `entity-panel` | **catalogue** · svelte-check · **cargo untouched** | — |
| **5** | live verify (9222) + records incl. `D-146` and both annotations | — | all |

🔑 **WHY 1 AND 2 SPLIT:** commit 1 changes **no** rendered behaviour (R2 reads the same value from a new address) ⇒ **a clean before/after for commit 2, which is the one that changes the screen.** ⚠️ **CONTINGENT (Clair's F6): this holds ONLY if the lifted store replicates R2's exact current rule — space-selection-only. If §8-2 resolves toward room-resolves-to-Space, commit 1 DOES change which Space R2 scopes to, and the clean before/after is lost.** ⇒ ***§6 and §8-2 are in tension and the claim cannot be asserted until §8-2 is decided.***
🔑 **WHY 3 IS INDEPENDENT:** it touches a **different store and a different concern** — the room latch, not the Space latch. **If OQ-1 stalls, 3 and 4 can still land**, and 3 is the one Leg C actually needs. 🛑 **CORRECTED 2026-08-08 (Clair's F5): the original reason — *"it touches a different panel"* — WAS FALSE, and read as true only under the R6/R2 mislabel.** **Commits 1 and 3 BOTH edit `rooms-panel.svelte`** — commit 1 removes the private latch at `:23-27`, commit 3 changes the `selected` derive at `:43-45`. **Non-overlapping lines, so the independence CONCLUSION holds; the stated reason did not.**
⚠️ **COMMIT 4 TOUCHES `ui/core` AND ALL THREE CONSUMERS PLUS SEVEN SAMPLER CELLS.** *Measured alone, like `OQ1-G1`.*

---

## §7 — NOT TOUCHED

`selection`'s **shape** (S-6: one bus, do not widen — this milestone adds no second bus, it reads latches that already exist) · `roomLatch`'s room semantics · `note()`'s single-writer rule · R5, R8, R3 · `canSend` · the wire · `xgen-core` / `xgen-node` / `xgen-common` · `skin.css` (**Joe's**) · **everything in `M-RP-MEMBER-ACT` Leg C** — `interactive`, `onActivate`, `selectOnActivate`, the find-DM scan.

---

## §8 — WHERE THIS DOCUMENT IS MOST LIKELY WRONG

🛑 **CLAIR'S ADVERSARIAL READ LANDED 2026-08-08 AND RETURNED EIGHT FINDINGS. CHAT RE-DROVE ALL EIGHT; ALL EIGHT CONFIRMED. THE HEADLINE ONE WAS NOT ON THIS LIST.**

0. 🛑 **THE MILESTONE'S CENTRAL IDENTIFIER WAS WRONG AND §8 v1.0 DID NOT SUSPECT IT (F1).** **`R6` IS THE COMPOSER** (`layout-default.ts:31`); the rooms panel is **`R2`** (`:27`). v1.0 called the rooms panel R6 in nine places while using R6 correctly for the composer in `S8` — **one file labelled both**. 🔑 **The composer has NO highlight and is IMMUNE to this defect: it reads `roomLatch.canSend`, which derives from the latch, not the bus.** ⚠️ ***And it reached committed code — `selection.svelte.ts`'s J-692 annotation said "R1/R6", pushed at `d5c87c4`, corrected at J-693.*** 📌 **Every pre-existing REPO record was correct** (`M_RP_MEMBER_ACT_PHASE0.md:246,:320`) ⇒ **no shipped document carried the mislabel before this session.** ⚠️ **QUALIFIED 2026-08-08: J-693 said the mislabel *"dates entirely from one session"* — true of the RECORD, and NOT VERIFIABLE of the CONVERSATION.** *An earlier session's carry-over item is remembered as "R6 + R1 bus behaviour", which would put the mislabel earlier in chat. It never reached disk, so the repo cannot settle it; the narrower claim is the one that stands.* 🔒 **Every mechanism and file cite in v1.0 pointed unambiguously at `rooms-panel.svelte`, so the BUILD was always recoverable — but an implementer taking §6 commit 3's bare "R6" literally would open the composer and find nothing to change.**
1. ✅ **DISCHARGED — §5's OPTION SET REACHED A BY ELIMINATION AND A FOURTH OPTION WAS HIDING, EXACTLY AS FEARED (F2 + F3).** **B was mislabelled unsound** — it is deterministic and verifiable ⇒ `D-143` does not fire, the **third** misapplication this milestone. **And option D was named in the superseded lock itself** (`M_RP6_2:129`). 🛑 **BUT CHAT THEN COMPOUNDED IT: having learned B's refusal REASON was wrong, Chat re-opened `OQ-1` — although B still fails Joe's stated requirement.** ⇒ **`D-147`, and §5 now closes `OQ-1` as A with B refused on the requirement and D FILED.**
2. ✅ **DISCHARGED AND NOW DECIDED (F4).** The lifted store's write rule was undesigned; **resolving it toward room-resolves-to-Space ADOPTS option B's behaviour for the in-a-room case.** 🔒 **§5 settles it: SPACE-SELECTION ONLY**, because B fails the requirement. 🔒 **Grounded FUTURE-ONLY: the sole room-selection writer is `rooms-panel.svelte:49`, reachable only after a Space is latched.** ⚠️ **The future case is named and owed to `M-RP-MEMBER-ACT` Leg E** — a DM entered whose Space was never clicked.
3. ⚠️ **`rooms-panel:25`'s `$effect` READS THE BUS FOR SCOPE, NOT HIGHLIGHT.** Chat traced them as separate concerns and expects commit 1 to replace the effect wholesale while commit 3 touches only `:43-45`. ✅ **Clair confirmed the trace — but see §6: BOTH commits edit the same file, which v1.0 denied.** **Still READ, NOT DRIVEN.**
4. ⚠️ **NO CATALOGUE PREDICTION IS MADE.** A clamp is not a registration and commits 1–3 add none. **Stated as an expectation; the runbook MEASURES it.**
5. ⚠️ **THE `L-7` INTERACTION IS ARGUED, NOT DRIVEN.** That fixing R2 makes `OQ-C5` dissolve follows from the identity write no longer being read for the highlight — **but Leg C does not exist, so it cannot be exercised end to end until it does.**
6. ✅ **DISCHARGED — `J-2` DOES NOT OVERREAD.** Clair verified `M_RP6_2:128/:131` against §1's quotations, the `:204-212` annotation, and `D-146`'s provenance: **Joe restated the supersede AFTER disclosure of the July lock**, so it is an utterance, not an inference from silence. ⚠️ **One loose phrase stands: `D-146`'s *"opt-2 now means R1 READING a latch that already exists"* UNDERSTATES the lift** — the latch exists **privately** in R2, and making it readable is the new `$common` store, which is this milestone's real cost. **The re-price DIRECTION is sound; the phrasing flatters it.**
7. 🔑 **v1.0's ITEM 7 SAID ITS AUTHOR HAD ALREADY GOT THE MILESTONE'S CENTRAL FACT WRONG ONCE. IT WAS WRONG ABOUT WHICH FACT.** It meant the `D4` mis-call; **the central IDENTIFIER was also wrong and item 7 did not see it.** ⇒ ***Chat's self-suspicion list correctly predicted items 1 and 2 and completely missed item 0 — which is the ninth consecutive arc in which Chat's own re-reads caught nothing that mattered.***
