# M-RP-SELECT-ORIENT — the panels keep saying where you are — Phase-0
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Aug 2026  
> **Last updated**: 2026-08-08  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — THE ONE SENTENCE

**Select a room and the Space stays lit. Select a member and the room stays lit.** Today both go dark, because R1 and R6 read their highlight from the **selection bus** — a single global slot holding whatever was clicked last — instead of from the latch that actually knows where you are.

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
| **J-3** | Scope is **R1 and R6**, and **`OQ-C3`** (the `activeIndex` staleness) folds in | uttered |
| **J-4** | This milestone lands **before Leg C** | uttered, adopting Chat's recommendation |

📌 **`D4`'s STATED COST HAS EXPIRED, WHICH IS WHY THIS IS A RE-PRICE AND NOT A CHANGE OF TASTE.** opt-1 won on *"no second latch"*. At the time opt-2 meant R1 growing its own. **It no longer does** — R2's Space latch has existed since M-RP6.2 and `roomLatch` since M-RP6.3 Leg D2. *The option was priced once, in July, and never re-checked — this project's most repeated error class.*

---

## §2 — GROUNDED, MEASURED AT `7305df2` (overnight session 2026-08-07/08)

📌 **THE SESSION CROSSED MIDNIGHT.** Where a measurement is dated below, it is dated to the **session**, not to a clock reading Chat did not take. *Two probe runs sit either side of 00:00 and Chat did not timestamp them individually; claiming a date per probe would be precision it does not have.*

| # | fact | site |
|---|---|---|
| **S1** | R1's highlight: `selection.current?.entity.kind === 'space' ? …id : undefined` — **bus-derived** | `spaces-panel.svelte:44-46` |
| **S2** | R6's highlight: `selection.current?.entity.kind === 'room' ? …id : undefined` — **bus-derived** | `rooms-panel.svelte:42-44` |
| **S3** | 🔑 **R2 ALREADY SOLVED THIS ONCE, LOCALLY.** `let latchedSpaceId = $state<string \| null>(null)` + an `$effect` writing it only on a `space` selection — because *"clicking a room would blank R2's own list"* | `rooms-panel.svelte:23-27` |
| **S4** | 🛑 **THAT LATCH IS PRIVATE TO R2.** No export, no store; R1 cannot read it | `rooms-panel.svelte` |
| **S5** | 🛑 **AND FOLDING IT INTO `roomLatch` IS EXPLICITLY FORBIDDEN**: *"THERE ARE TWO LATCHES AND THE NAME HIDES IT … R2's Space latch is NOT touched by this store and must not be folded into it."* | `room-latch.svelte.ts` header |
| **S6** | `roomLatch` exposes `latchedRoomId` (raw, *"resolvable or not — a verify/debug surface"*), `effectiveRoomId` / `effectiveSpaceId` (*"the room BOTH R5 and R6 act on"*), `canSend` — **all from ONE `resolveLatched()`** | `room-latch.svelte.ts:41-77` |
| **S7** | 🔑 **`roomLatch` IS ITSELF THE PRECEDENT FOR THE LIFT** — R5's room latch was lifted into `$common` for R6, and *duplicating it was REJECTED as a `D-067` drift surface* | `room-latch.svelte.ts` header |
| **S8** | R5 reads `effectiveRoomId` (`:70`); R6 reads `canSend` (`:56`), `effectiveSpaceId` + `effectiveRoomId` (`:68-69`) ⇒ **the stream's content, the composer's target and its enablement are already ONE resolution** | `stream-panel.svelte` · `composer-panel.svelte` |
| **S9** | **The bus has FIVE readers**, not one: `inspector-panel:40` · `self-panel:55` · `rooms-panel:25,:44` · `spaces-panel:45` · `app_client:196` | measured, `grep "stores/selection"` over `ui/**` |
| **S10** | 🛑 **`selection.svelte.ts:12` STANDS FALSE**: *"R8 (inspector) is the only consumer."* **It is the Leg A annotation of 2026-08-06 — an annotation that corrected one false claim and introduced another** | `selection.svelte.ts:8-12` |
| **S11** | The bus→latch bridge is an **`$effect`** reading `selection.current` | `app_client.svelte:195-198` |

### 🔬 MEASURED LIVE ON THE CLIENT (9222), NOT READ

- 🛑 **THE DEFECT, DRIVEN.** Click a room ⇒ `roomsSel` = the room id, `latched` = the room. **Write an identity to the bus — exactly what `L-7` does ⇒ `roomsSel` → `null`, `latched` UNCHANGED.** ⇒ ***the composer keeps targeting the room while the room stops looking selected.***
- ✅ **REACHABLE TODAY WITHOUT LEG C.** Clicking the **R3 self card** — a shipped gesture (`self-panel:77` writes an identity selection) — reproduces it exactly. **This milestone fixes a live defect, not a hypothetical one.**
- 🔒 **AND THE TWO SPACE SOURCES DIVERGE, MEASURED.** Engineering → room `random` → click Design ⇒ R2 lists **Design's** rooms (`latchedSpaceId` moved) while `effectiveSpaceId` stayed **Engineering** (the room latch did not). *Recorded in J-691 as correct two-latch behaviour, which it is — and it is also why §4's source choice is not free.*

---

## §3 — THE TWO HALVES, KEPT APART

### ✅ 3.1 — R6 (rooms panel): A DEFECT, AND NOBODY RULED ON IT

`D4`/`D5` were taken when the bus held **spaces and rooms only**. **Identity selections arrived later with R3 and become common with `L-7`.** ⇒ **no decision covers "what does R6 do when the bus holds an identity", and the shipped answer — go dark while still being the room you are typing into — is one nobody chose.**

### 🔒 3.2 — R1 (spaces panel): NOT A DEFECT. A SUPERSEDE (`J-2`)

R1 un-highlighting on a **room** selection is `D4` opt-1 working exactly as locked and verified (`M_RP6_2_SPACES_ROOMS.md:219`, V5). **Changing it is overturning a July ruling, and `J-2` overturns it.** ⚠️ **The `D4` annotation and the new `D-146` are therefore load-bearing records of this milestone, not paperwork.**

### ⚠️ 3.3 — `OQ-C3`: A THIRD, UNRELATED DEFECT RIDING BY JOE'S CHOICE (`J-3`)

`entity-panel`'s `activeIndex` is seeded once (`:94`), never clamped; `tabindex` is `0` only where `i === activeIndex` (`:183`). **Driven in-session: shrink the list below the stored index and NO row is tabbable, `focusables` = 0, while `role="listbox"` stands.** Escapable **only with a mouse**. 📌 *Different mechanism, same theme — the panels stop telling you where you are, or cannot be reached at all.*

---

## §4 — THE SHAPE, AND THE ONE REAL CHOICE INSIDE IT

**R6 is settled. R1 is not, and §2's divergence is why.**

### ✅ R6's SOURCE IS SETTLED — `effectiveRoomId`, not `latchedRoomId`

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

## §5 — OPEN. `D-121` lenses: ① user-visible impact, then ② resource cost.

### 🔓 OQ-1 — R1's highlight source (§4). **ARCHITECTURE — a new shared store.**

**A — lift R2's Space latch into a `$common` store; R1 and R2 both read it.**
① R1 lights the Space you are browsing; R1 and R2 always agree. **The behaviour `J-2` describes.**
② A new store (~30 lines), R2 edited to read it instead of its private copy, R1 edited. **Two shipped widgets touched.**

**B — R1 reads `roomLatch.effectiveSpaceId`.**
① 🛑 **R1 lights a DIFFERENT Space from the one R2 is listing, whenever you browse without entering a room. Measured, not hypothetical.**
② One line in R1, no new store. **Cheapest, and it is unsound** ⇒ `D-143` fires.

**C — R1 grows its own private latch, mirroring R2's.**
① Identical to A while they agree. ② Two copies of one rule — **the `D-067` drift `roomLatch`'s header rejects by name.** Refused, listed for completeness.

📌 **Chat's recommendation: A.** 🔓 **A new `$common` store is architecture and therefore Joe's** — but note that **B and C are both refused on stated grounds, so A stands alone**, and §8 item 1 records that an option set reaching its answer by elimination is where a fourth option hides.

### 📌 SETTLED WITHOUT ASKING (`D-123`, so the record says who took them)

- **R6 ⇒ `effectiveRoomId`** — §4.
- **`selection.svelte.ts:12` gets a second annotation** naming all five readers (`D-131` — a broken claim is annotated at the site, and this one is Chat's own).
- **`M_RP6_2_SPACES_ROOMS.md` is `COMPLETED`, not `ARCHIVED`** ⇒ **`D-145`: the `D4` supersede is ANNOTATED AT THE SITE**, not worked around.
- **A new project-wide `D-146`** records the supersede with the expired-cost rationale.

---

## §6 — PROPOSED COMMIT ORDER (`D-074`)

| # | commit | floor | gated on |
|---|---|---|---|
| **1** | 🆕 the lifted Space-latch store + **R2 reads it** (its private copy removed) | svelte-check · catalogue | **OQ-1** |
| **2** | **R1 highlights from it** — the `J-2` behaviour | svelte-check · catalogue | 1 |
| **3** | **R6 highlights from `roomLatch.effectiveRoomId`** | svelte-check · catalogue | — (independent of 1–2) |
| **4** | **`OQ-C3`** — clamp `activeIndex` against `items.length` in `entity-panel` | **catalogue** · svelte-check · **cargo untouched** | — |
| **5** | live verify (9222) + records incl. `D-146` and both annotations | — | all |

🔑 **WHY 1 AND 2 SPLIT:** commit 1 changes **no** rendered behaviour (R2 reads the same value from a new address) ⇒ **a clean before/after for commit 2, which is the one that changes the screen.**
🔑 **WHY 3 IS INDEPENDENT:** it touches a different panel and a different store. **If OQ-1 stalls, 3 and 4 can still land** — and 3 is the one Leg C actually needs.
⚠️ **COMMIT 4 TOUCHES `ui/core` AND ALL THREE CONSUMERS PLUS SEVEN SAMPLER CELLS.** *Measured alone, like `OQ1-G1`.*

---

## §7 — NOT TOUCHED

`selection`'s **shape** (S-6: one bus, do not widen — this milestone adds no second bus, it reads latches that already exist) · `roomLatch`'s room semantics · `note()`'s single-writer rule · R5, R8, R3 · `canSend` · the wire · `xgen-core` / `xgen-node` / `xgen-common` · `skin.css` (**Joe's**) · **everything in `M-RP-MEMBER-ACT` Leg C** — `interactive`, `onActivate`, `selectOnActivate`, the find-DM scan.

---

## §8 — WHERE THIS DOCUMENT IS MOST LIKELY WRONG

1. 🛑 **§5's OPTION SET REACHES A BY ELIMINATION.** B is refused as unsound, C as drift ⇒ A stands alone. **That is the exact shape in which a fourth option hides**, and Leg C's Phase-0 was caught doing the same thing. **A read should attack the framing, not the arithmetic.**
2. 🛑 **THE LIFTED STORE'S SEMANTICS ARE NOT DESIGNED HERE.** Does it latch on a `space` selection only (R2's current rule), or also when a **room** selection resolves to a Space? **Those differ the moment you enter a room in a Space you did not click.** *Not measured, not decided — it belongs in the runbook's design pass and it may reopen OQ-1.*
3. ⚠️ **`rooms-panel:25`'s `$effect` READS THE BUS FOR SCOPE, NOT HIGHLIGHT.** Chat traced them as separate concerns and expects commit 1 to replace the effect wholesale while commit 3 touches only `:42-44`. **READ, NOT DRIVEN.**
4. ⚠️ **NO CATALOGUE PREDICTION IS MADE.** Commit 4 adds no ids by the `OQ1-G1` arithmetic (a clamp is not a registration), and commits 1–3 add none. **Stated as an expectation; the runbook MEASURES it.**
5. ⚠️ **THE `L-7` INTERACTION IS ARGUED, NOT DRIVEN.** That fixing R6 makes `OQ-C5` dissolve follows from the identity write no longer being read for the highlight — **but Leg C does not exist, so it cannot be exercised end to end until it does.**
6. 📌 **`D4`'s SUPERSEDE RESTS ON ONE READING OF ONE EXCHANGE.** `J-2` is recorded as Joe's by utterance because he restated the behaviour **after** being shown the lock. ⚠️ ***If that overreads, the whole R1 half falls and only R6 + `OQ-C3` remain.*** **Joe can strike it with one word.**
7. 🔑 **THIS DOCUMENT HAS NOT BEEN READ BY ANYONE OUTSIDE ITS AUTHOR, AND ITS AUTHOR ALREADY GOT THIS MILESTONE'S CENTRAL FACT WRONG ONCE** (§1). ⇒ **Clair's adversarial read runs before Joe locks OQ-1.**
