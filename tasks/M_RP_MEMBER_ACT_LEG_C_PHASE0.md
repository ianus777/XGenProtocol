# M-RP-MEMBER-ACT Leg C — R7 acts: the row opens the DM and writes the bus — Phase-0
> **Status**: COMPLETED  
> Version: 1.5  
> Date: Aug 2026  
> **Last updated**: 2026-08-08  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

✅ **CLOSED AT J-700 — Leg C SHIPPED IN THREE COMMITS:** C-1 `524d4f7` · C-2 `b5d0908` · C-3 `6a6c066`.
This Phase-0's findings were carried into `RUNBOOK_MEMBER_ACT_LEG_C.md` (**v1.2 COMPLETED**); what the run
actually found is in that runbook's **§8**, including the line numbers this document's citations staled
under (`D-131` — superseded, kept not erased).

📌 **THE CATALOGUE CLAIM AT `§` "Catalogue = 435" HELD.** Re-measured live at C-1 after a full
`location.reload()`: **ids 435 = unique 435 = domCount 435, zero orphans both directions.**

🛑 **WHAT THIS DOCUMENT COULD NOT HAVE KNOWN:** `§6` leg 5 is **not drivable on Joe's client state** — the
only DM's counterpart is erased, and an erased non-counterpart member is hidden in a group room, so no
rendered group-room row has an existing DM. **Leg 5 stays UNRULED and is filed as `OWED-4` in
`M_RP_MEMBER_ACT_LEG_C_BIS.md`.**

---

## §0 — WHAT THIS IS, AND THE ONE SENTENCE THAT SIZES IT

Leg C of `M-RP-MEMBER-ACT` — the members panel acts: LMC opens the DM, RMC opens the menu. `M_RP_MEMBER_ACT_PHASE0.md` §6 assigns it: **`interactive` on, `onActivate` → open-or-draft, plus `selection.set()` so R8 shows the member's card (`L-7` (a), Joe uttered).**

🔑 **THIS PHASE-0 EXISTS BECAUSE LEG C IS APPEARANCE-ADJACENT AND TOUCHES SHARED `ui/core` (`D-071`).** Phase-0 -> Joe locks -> runbook -> Clair. **Not straight to a runbook.**

🛑 **AND IT FOUND SOMETHING THE MILESTONE PHASE-0 DID NOT: `L-7` REQUIRES TWO SELECTIONS AND THE PROJECT HAS ONE BUS.** See §3.2. *That is not a detail of Leg C; it is the shape of Leg C.*

**Prerequisite satisfied:** the subsystem audit exists (`tasks/AUDIT_MEMBERS_PANEL.md`); the milestone Phase-0 is at v1.10 ACTIVE. This document does not re-derive either; it cites them and measures what they left unmeasured.

---

## §1 — WHAT LEG C INHERITS AND MAY NOT RE-OPEN

🔒 Uttered by Joe unless marked. Provenance per `D-141`.

| # | binds Leg C how | source |
|---|---|---|
| **L-7** | **LMC DOES BOTH** — opens the DM **and** writes the selection bus so R8 shows the member's card | 2026-08-06, uttered *"(a)"* |
| **L-8** | **NAVIGATION-ON-CLICK IS INTENDED** — the roster is replaced by the DM's | uttered *"yes, intended"* |
| **L-9** | **RMC opens the menu WITHOUT selection** — Leg D, not Leg C | uttered |
| **L-4** | **CREATION IS LAZY.** The click opens; **first send creates**. ⇒ Leg C signs nothing | uttered |
| **OQ1-G1** | 🔒 **THE `core` PROP SPLIT IS LEG C's FIRST COMMIT, MEASURED ALONE** — `interactive` keeps ARIA + click/keyboard; a new flag suppresses `selectAt`'s `selected` write | Chat under `D-143`, DELEGATED |
| **OQ6-E2** | **the self row takes the SAME path as any peer** — no `self_open` command; `self_open` stays unregistered (R-5) | Chat under `D-123` |
| **OQ7-W4** | **R7 STAYS THIN during a draft** — self only; R8's card carries the counterpart | DELEGATED |
| **OQ4** | **NO** — the R8 card does not ship before the DM opens | Chat's, taken |

⚠️ **Leg C-bis owns first-send, `canSend`'s second arm, and R5's draft branch. Leg C must not reach into any of them.**

---

## §2 — GROUNDED, MEASURED AT `7203474` (2026-08-07)

| # | fact | site |
|---|---|---|
| **C1** | `entity-panel` props: `items · title · badge · collapsible · collapsed($bindable) · selected($bindable) · onActivate · interactive(=true) · emptyText · id` | `entity-panel.svelte:44-76` |
| **C2** | 🔑 **`selectAt(i)` WRITES THREE THINGS IN ONE BODY**: `activeIndex = i` (`:104`) · `selected = it.descriptor.id` (`:105`) · `onActivate?.(...)` (`:106`) | `entity-panel.svelte:101-107` |
| **C3** | **`interactive` gates FIVE sites**: `<ul role>` `listbox`/`list` (`:171`) · `aria-multiselectable` (`:173`) · the `<li>` branch itself (`:177`) — carrying `role="option"` (`:180`), `aria-selected` (`:182`), `tabindex` (`:183`), `onclick` (`:184`), `onkeydown` (`:185`) — versus the inert `role="listitem"` (`:192`) | measured |
| **C4** | 🛑 **`activeIndex` IS SEEDED ONCE AND NEVER RE-SEEDED.** `let activeIndex = $state(initialActive())` (`:94`) — a deliberate one-time capture. **Nothing clamps it against `items.length`** | `entity-panel.svelte:90-94` |
| **C5** | `tabindex={i === activeIndex ? 0 : -1}` (`:183`) ⇒ **if `activeIndex >= items.length`, NO row is tabbable, and the `<ul>` itself carries no tabindex** | `entity-panel.svelte:183`, `:169-174` |
| **C6** | `selection` is a **SINGLE SLOT**: `set(regionId, entity)` replaces; its own doc says *"a second `set` overwrites"* | `selection.svelte.ts:34-42` |
| **C7** | 🛑 **`roomLatch.note()` WRITES ONLY ON `sel.entity.kind === 'room'`.** An `identity` selection latches **nothing** | `room-latch.svelte.ts:79-82` |
| **C8** | `effectiveRoomId` / `effectiveSpaceId` / `canSend` all derive from **one** `resolveLatched()`, which scans `spacesState.spaces` for the room id | `room-latch.svelte.ts:41-77` |
| **C9** | ✅ **A DM SPACE CARRIES EXACTLY ONE ROOM** — `rooms: vec![KnownRoom { room_id, name: "dm", joined: true }]` ⇒ the latch target is `space.rooms[0].room_id` and it resolves | `ops.rs:970` region |
| **C10** | ✅ **`counterpart` IS ON THE READ PATH** — gate 4b proved `DM with ...sno_FWmw` -> `Some("xgen://...")` through `ops::spaces` | J-689, runbook §5.4b |
| **C11** | 🔑 **R7 ALREADY HIDES ERASED MEMBERS.** `memberRows` filters `notFound.has(id)` **except** `id === counterpart`, and `counterpart` is `undefined` outside a DM | `members-panel.svelte`, memberRows / counterpart derivations |
| **C12** | R7 mounts `<EntityPanel items={rows} selected={counterpart} interactive={false} ... />` — **no `onActivate`, `selected` non-bindable** | `members-panel.svelte`, EntityPanel mount |

### 🔬 MEASURED LIVE, NOT READ (sampler 9422, fresh launch, full `location.reload()` before and after)

- **Catalogue = 435.** `ids 435 / unique 435 / domCount 435`. 🔒 ***The floor stated BY SCOPE since before Leg B is now MEASURED, and it is correct.***
- **`entity-panel` cells = SEVEN**, not eight: `#spaces · #dms · #empty · #collapsed · #inert · #unresolved · #rooms`. ⚠️ **`M_RP_MEMBER_ACT_PHASE0.md` §5-OQ1 says *"8 sampler cells"*. It is 7 — annotate there (`D-145`; that document is ACTIVE).**
- **Cell cost = 8 registrations for a 3-row panel** (`3 x entity-item` + `3 x entity-avatar` + `entity-panel` + `section`), enumerated from `#inert` and `#unresolved`.
- 🔑 **THE FUSION, DRIVEN NOT ARGUED.** One `.click()` on row 2 of `entity-panel#rooms` — **a cell with `bind:selected` and NO `onActivate`** — moved all three: `selected` `null -> "xgen://room/dev-2b"` · `aria-selected` `[false,false] -> [false,true]` · `tabindex` `["0","-1"] -> ["-1","0"]`. ⇒ ***`selected` is written by the click alone, independently of any consumer callback. C2 is measured, not inferred.*** *(The probe mutated state; cleaned by `location.reload()` per N-123 — post-reload `selected` back to `null`, catalogue back to 435.)*

---

## §3 — THE FOUR FINDINGS

### 🛑 3.1 — `interactive` FUSES FIVE SITES, NOT THREE, AND ONLY ONE MUST GO

The milestone Phase-0 named three concerns (ARIA · click/keyboard · the `selected` write). **Measured: `interactive` gates five sites (C3) and `selectAt` writes three things (C2).** Leg C needs **all of the ARIA and all of the wiring**; it must lose **exactly one line — `:105`**.

📌 **CONSEQUENCE FOR THE FLAG'S SHAPE:** it is not a second mode. It is a **single suppression** of `:105`, leaving `activeIndex` and `onActivate` untouched. ⇒ **`aria-selected` (`:182`) still renders from the prop**, which is correct: the row renders state, the parent produces it. *That is `M-RP-PANEL-INERT`'s own sentence, honoured without switching the panel off.*

🔓 **The flag's NAME and DEFAULT are an API surface, and `role="listbox"` is user-visible to a screen reader ⇒ §5-OQ-C4.**

### 🛑 3.2 — `L-7` NEEDS TWO SELECTIONS AND THE PROJECT HAS ONE BUS. **THE LARGEST FINDING IN THIS DOCUMENT.**

`L-7` says one gesture does both: **open the DM** and **write the bus so R8 shows the MEMBER**.

- **C6**: `selection` holds **one** slot; a second `set` overwrites.
- **C7**: `roomLatch.note()` writes `_latched` **only** for `kind === 'room'`.

⇒ **Writing an `identity` selection latches nothing, and latching by writing a `room` selection would overwrite the identity — so R8 would show the ROOM, not the member. `L-7` would be broken by the only mechanism currently available.**

🛑 **MECHANISM CORRECTED 2026-08-08, CONCLUSION UNCHANGED (`D-145`; Clair's F3 forced the re-drive and reached the OPPOSITE conclusion from the same fact).** The paragraph above gives the WRONG REASON. **The bus→latch bridge is an `$effect`** (`app_client.svelte:195-198`, reading `selection.current` as its sole tracked dependency) ⇒ **two SYNCHRONOUS `set` calls coalesce into ONE effect run that observes only the FINAL value.** ***The intermediate room selection is never seen by `note()` at all — it is not overwritten downstream, it is never observed upstream.***

🔒 **MEASURED ON THE LIVE CLIENT (9222), WITH A CONTROL:**

| drive | `latchedRoomId` after flush |
|---|---|
| `set(room)` then `set(identity)` — **N2's prescription** | **`null`** 🛑 |
| `set(room)` alone — **control** | **the room id** ✅ |

⇒ **`L-7` still cannot be delivered by the existing mechanism, and N2 is REFUTED BY MEASUREMENT rather than refused on judgement** — which matters, because `D-143` was the wrong instrument: **N2 is not unsound-but-workable, it simply does not work.**

🔑 ***NOBODY HAS RECORDED THIS.*** The milestone Phase-0's §6 reads Leg C as *"`onActivate` -> open-or-draft **and** `selection.set()`"*, as if they were two independent calls. **They are not: the first has no writer that does not destroy the second.**

⚠️ **AND IT IS THE THIRD ITEM LANDING ON `room-latch.svelte.ts`** — `F-E` already put `canSend`'s second arm there for Leg C-bis, and `resolveLatched` is read by R5, R6 and `canSend`. **The shared latch is edited twice across two legs, and the file's own header calls `note()` *"THE SINGLE WRITER"*.** ⇒ **§5-OQ-C1, and it is architecture — Joe's.** 📌 *R5 = stream, R6 = composer — correct here; see §5-OQ-C5's note on the R2/R6 mislabel elsewhere in this document's v1.2.*

### ⚠️ 3.3 — A PRE-EXISTING `ui/core` DEFECT THAT LEG C MAKES REACHABLE IN R7

**C4 + C5:** `activeIndex` is seeded once, never re-seeded, never clamped. `tabindex` is `0` only where `i === activeIndex`. ⇒ **when the list SHRINKS below the stored index, no row is tabbable and the panel drops out of the tab order entirely.**

🛑 **`L-8` PRODUCES EXACTLY THAT SHRINK ON EVERY CLICK:** click member #6 of a 9-person room -> the DM's roster is 2 rows -> `activeIndex = 5`, `items.length = 2`.

📌 **IT IS NOT LEG C's DEFECT.** `rooms-panel:65` and `spaces-panel:62` are `interactive` today and their `items` change without a remount ⇒ **latent in the shipped build**, reachable by selecting a Space with fewer rooms than the index last clicked.

🔒 **DRIVEN 2026-08-07 ON THE REAL CLIENT (9222). THE DEFECT IS REAL, AND THE MEASUREMENT IS SHARPER THAN THIS SECTION PREDICTED, IN BOTH DIRECTIONS.**

| step | rooms panel | `[tabindex]` | tabbable |
|---|---|---|---|
| select **Engineering** (2 rooms) | `general · random` | `["0","-1"]` | 1 |
| click room index **1** (`random`) | same | `["-1","0"]` | 1 |
| 🛑 select **Design** (1 room) | `ideas` | **`["-1"]`** | **0** |
| select **LegBSpace** (2 rooms) | 2 rows | `["-1","0"]` | 1 |

At the failing step **`focusables` inside the panel = 0**, the `<ul>` carries no `tabindex`, and its `role` is still **`listbox`** ⇒ ***a listbox announcing itself as focusable that nothing can focus.***

🔑 **NOT PERMANENT — AND THAT IS NOT THE RELIEF IT SOUNDS LIKE.** The panel returns the moment the list grows past the stale index. **But it is escapable ONLY WITH A MOUSE:** a keyboard user cannot reach the panel to press the arrow key that would move `activeIndex` back into range. ***A self-healing trap you cannot heal from inside.***

⚠️ **AND R7 WILL HIT IT HARDER THAN `rooms-panel` DOES — REASONED, NOT MEASURED.** Under `L-8` every click shrinks the roster (9 → 2) and under `OQ7-W4` a draft shrinks it to **one row**, so the stale index is out of range on essentially every click, and recovery needs a room with more members than the index last clicked. 📌 **R7 is `interactive={false}` today and carries no `tabindex` at all, so this cannot be tested until Leg C exists.**

🔒 **READ FROM THE SOURCE — NOW DRIVEN (see the table above; §8 item 1 DISCHARGED).** The falsifying probe was a `[tabindex]` census at the **DOM** layer after a real roster change (`D-140`), and it did not falsify. ⇒ **§5-OQ-C3: rider, own milestone, or filed.** ⚠️ ***`D-071` and this project's own record say a `core` fix is not a rider by default; it is named here rather than discovered in the runbook.***

### ✅ 3.4 — `OQ5`'s ERASED-MEMBER ITEM IS NOT REACHABLE IN THE FORM IT IS WRITTEN

`OQ5` reads: *"erased members are clickable and a DM to one would go nowhere."*

**C11, measured:** `memberRows` **drops every `notFound` member**, with one exception — `id === counterpart` — and `counterpart` is `undefined` unless `addressBook.isDm`.

⇒ 🔒 **IN A GROUP ROOM AN ERASED MEMBER DOES NOT RENDER AT ALL. THE ONLY ERASED ROW THAT CAN EXIST IS THE COUNTERPART OF THE DM YOU ARE ALREADY IN** — which is exactly the LegF-DAVE row `M-RP-TAIL8` used as its live lever.

🔑 **AND CLICKING IT IS A NO-OP BY CONSTRUCTION.** The DM exists, `counterpart` is on the read path (**C10**), the scan finds it, the latch resolves (**C9**) — **you re-enter the conversation you are already in.** Nothing is created. Under `L-4`, **Leg C signs nothing on any path.**

⚠️ **WHAT *IS* REACHABLE IS A DIFFERENT THING, AND NO DESIGN PREVENTS IT: a live-joined member carries `unresolved: true`, renders, and is clickable BEFORE the client knows whether they are erased.** *You cannot guard against an erasure you have not learned of.* 📌 **That is not `OQ5`'s question and it is not a defect — it is the honest limit of a resolution that happens after the fact.**

⇒ **`OQ5`-item-2 SPLITS** (§5-OQ-C2), and **the half with a wire consequence — may a DM be *created* to an erased identity — is LEG C-bis's, not Leg C's**, because creation is what first-send does.

---

## §4 — THE CATALOGUE: WHAT MOVES IT, MEASURED

| change | catalogue effect |
|---|---|
| the new prop on `entity-panel` | 🔒 **ZERO.** A prop registers no id. The getter at `:141-147` gains a field, readable via `get(id).state` — **not a new registration** |
| R7 flipping `interactive` on | **ZERO** — the rows already register in both modes (`#inert` and `#unresolved` prove it) |
| 🆕 **a sampler cell demonstrating the flag** | **+8 for a 3-row cell** (3 items + 3 avatars + panel + section), measured |

🔒 **⇒ THE FLOOR MOVES IF AND ONLY IF A SAMPLER CELL IS ADDED, AND BY A PREDICTABLE AMOUNT. The runbook MEASURES it; this is the arithmetic, not the measurement.**

📌 **A cell SHOULD be added** — the library's discipline is that a `core` prop is demonstrated in the sampler, and `#inert` exists for precisely the flag this one mirrors. ⚠️ **Recorded as Chat's lean, not a ruling: it is `ui/sampler` content and it is cheap either way.**

---

## §5 — OPEN, AND JOE'S. `D-121` lenses: ① user-visible impact per option, then ② resource cost.

### 🔓 OQ-C1 — how does the DM's room get latched, given `L-7` owns the bus? (§3.2) **ARCHITECTURE.**

**N1 — `roomLatch` gains a direct `latch(roomId: string)` writer.** Leg C calls it; the bus carries the identity.
① **None.** The user clicks a person, the conversation opens, R8 shows the person. `L-7` delivered exactly.
② One method on `room-latch.svelte.ts`. ⚠️ **The file's header declares `note()` *"THE SINGLE WRITER"*, so a claim in a shipped comment goes false** — though `clear()` already writes `_latched`, so the claim is **already** inexact and the annotation is owed either way (`D-131`).

**N2 — R7 writes a `room` selection, then a second `identity` selection.** Ordering-dependent.
🛑 **REFUTED BY MEASUREMENT 2026-08-08 (§3.2). N2 DOES NOT WORK** — the two synchronous writes coalesce into one `$effect` run that never observes the room, so nothing latches (`null` vs the control's room id). ⚠️ **The costs originally stated here were BOTH wrong: there is no "R8 flickers" (nothing renders the intermediate) and `D-143` was the wrong instrument (this is not unsound-but-workable, it is inoperative).** 📌 *Clair reached the opposite conclusion from the same batching fact, which is what forced the drive.*
① *(moot)* ② *(moot)*

**N3 — R8 reads a separate identity channel.** ① None. ② A second bus — **exactly what `xgen-widget-surfaces-phase0.md` S-6 was locked to prevent.** Refused.

📌 **Chat's recommendation: N1**, with the `room-latch` header annotated in the same commit. *It is the only option that does not make a shipped invariant depend on call order.* 🔓 **Architecture is Joe's reserved area (`D-123`).**

### 🔓 OQ-C2 — the erased DM counterpart's row: clickable or not? (§3.4) **APPEARANCE.**

**E-a — clickable, like every other row.** ① Clicking re-enters the DM you are in; R8 shows the erased member's card, which is where the tail-8 and the erased marker already live. **No new state, nothing created.** ② Zero.
**E-b — the erased row is not interactive.** ① One row behaves differently from its neighbours with nothing on screen saying why. ② A per-row `interactive` concept that **does not exist** — `entity-panel` gates the whole list. **Real `ui/core` work.**

📌 **Chat's recommendation: E-a.** *The harm `OQ5` feared is not reachable here (§3.4), and E-b buys nothing while inventing per-row interactivity.* 🔓 **Whether an erased row LOOKS clickable is appearance and yours.**

📌 **`OQ5`'s remaining halves are RE-SITED, not answered:** *may a DM be **created** to an erased identity* ⇒ **Leg C-bis** (creation lives there) · *the partial first send* ⇒ **Leg C-bis** · *cross-node invite discovery* ⇒ a measurement of Chat's, prerequisite to nothing in A–D.

### ✅ OQ-C3 — the `activeIndex` staleness (§3.3). **SEQUENCED, SHIPPED, AND FIXED.**

✅ **SHIPPED 2026-08-08 AS C-4 (`62c72f6`) IN `M-RP-SELECT-ORIENT`, CLOSED AT J-697 — THE DEFECT NO LONGER EXISTS.** `entity-panel.svelte` now clamps on read: `activeClamped = $derived(Math.min(activeIndex, count - 1))`, with **all four** consumers routed (tabindex render, ArrowDown, ArrowUp, Enter/Space → `selectAt`) and the raw `$state` still freely written — **clamp-on-read, never re-seeded**, so N-136's self-invalidating read-modify-write stays foreclosed. 🔑 **AND §3.3's SEVERITY MEASUREMENT WAS UNDERSTATED: the defect is reachable in TWO ORDINARY CLICKS**, not by any contrived list-shrink — select Engineering (2 rooms) → room `random` (index 1) → Design (1 room) leaves `rows: 1`, `li[tabindex="0"]: 0`, `role="listbox"` standing, **escapable only with a mouse**. Driven live at `cd53c6d`; after C-4, exactly one tabbable row and **Enter on it FIRES `onActivate`**. 📌 **Leg C inherits a fixed `entity-panel` and owes this nothing.**

🛑 **THE HEADING ABOVE READ `🔓 … SEQUENCING` UNTIL J-697, WITH THE BODY ALREADY ANNOTATED CLOSED BENEATH IT** — *a heading that outlives the paragraph beneath it*, the species this project has now hit four times. **Repaired, not annotated: this document has never been locked (`D-145`).** ⚠️ **The three options below (T-a / T-b / T-c) and Chat's T-b recommendation are HISTORY — Joe took T-c. They are left standing as the record of the call, not as live options.** Effectively **T-c** (its own milestone) rather than Chat's recommended T-b, and it carries the attribution cost T-b was trying to avoid without polluting Leg C's `core` commit. ⇒ **Leg C inherits a clamped `activeIndex`. The options below stand as history.**

**T-a — file it, name the owner, do not touch it in Leg C.** ① The members panel can drop out of the tab order after a navigation — **a keyboard user loses the panel**, silently. Already true of the sibling panels today. ② Zero now.
**T-b — fix it in the same `core` commit as `OQ1-G1`.** ① The defect never reaches R7. ② ~2 lines (clamp `activeIndex` against `items.length`), **but it changes the behaviour of `rooms-panel` and `spaces-panel`, which is precisely why `M-RP-PANEL-INERT` got its own milestone.**
**T-c — its own milestone.** ① Same as T-a until it lands. ② A milestone for two lines.

🔒 **SEVERITY MEASURED 2026-08-07 (§3.3), AND IT CUTS BOTH WAYS — STATED SO THE CALL IS MADE ON THE NUMBER, NOT THE FEAR.** ✅ **Milder than feared:** the state is **not permanent** — the panel returns as soon as the list grows past the stale index. 🛑 **Worse than feared:** it is escapable **only with a mouse**, because a keyboard user cannot reach the panel to press the key that would fix it — ***a self-healing trap you cannot heal from inside*** — and **R7 under `L-8`/`OQ7-W4` shrinks the list on essentially every click**, where `rooms-panel` only shrinks it when you pick a smaller Space.

📌 **Chat's recommendation: T-b, and it is a genuinely close call.** *`D-143` does not decide it — a stale index is a real unsoundness, but `D-065`'s no-empty-machinery does not apply and `D-071` cuts the other way.* ⚠️ **The honest statement: the fix is trivial and the ATTRIBUTION is the whole cost.** 🔒 **Milestone split is Joe's (`D-123`:4610).** ✅ **The shrink case is now DRIVEN, so whichever is chosen the runbook inherits a measurement rather than a prediction.**

### 🔓 OQ-C5 — RE-SITED 2026-08-08, NOT ANSWERED HERE

**When `L-7` puts an identity on the bus, R2's room highlight goes out** (`rooms-panel:44` matches on `kind === 'room'`) **while the latch — and therefore R5's content and the composer's target — keeps the room.** Measured on the live client, and **reachable TODAY via the R3 self card**, so it is not Leg C's to fix.

🔒 **JOE OPENED `M-RP-SELECT-ORIENT` — the panels keep saying where you are — TO LAND BEFORE LEG C.** ⇒ **`OQ-C5` DISSOLVES: R2 will highlight from `roomLatch.effectiveRoomId`, so an identity selection no longer extinguishes it.** ⇒ **`tasks/M_RP_SELECT_ORIENT_PHASE0.md`.**

🛑 **LABEL CORRECTED 2026-08-08 (v1.3, Clair's F1): this § said "R6" for the rooms panel in v1.2. `R6` IS THE COMPOSER** (`layout-default.ts:31`); the rooms panel is **`R2`** (`:27`). **The composer derives no highlight and is immune to this defect — it reads `roomLatch.canSend`, which comes from the latch, not the bus.** 📌 *The mislabel was confined to the two lines of this § and to `M_RP_SELECT_ORIENT_PHASE0.md` v1.0; the rest of this document and all pre-existing records use R6 correctly.*

### 🔓 OQ-C4 — the flag's name and default. **`ui/core` API, screen-reader-visible.**

Candidates: **`selectOnActivate` (default `true`)** — reads as what it does, and `false` is the new behaviour · **`ownsSelection`** — truer to the concept, vaguer at the call site · **`selectable`** — collides with `interactive` in a reader's head.

📌 **Chat's recommendation: `selectOnActivate = true`.** *Default `true` keeps all three shipped consumers and all seven sampler cells byte-identical in behaviour; R7 opts out with one word that says why.* 🔓 **Naming in `ui/core` is close enough to appearance that it is offered rather than taken.**

---

## §6 — PROPOSED COMMIT ORDER INSIDE LEG C (`D-074`)

| # | commit | floor | note |
|---|---|---|---|
| **C-1** | 🔒 **`OQ1-G1` ALONE** — `entity-panel.svelte` gains the flag; `selectAt:105` guarded; the getter reports it. **No consumer changes.** Optional sampler cell per §4 | **catalogue** (+8 iff the cell lands, else 0) · svelte-check | ***measured alone, as the milestone Phase-0 requires*** |
| **C-2** | `roomLatch.latch()` per **OQ-C1** + the header annotation | svelte-check | gated on OQ-C1 |
| **C-3** | **R7 acts** — `interactive` on, the new flag off, `onActivate` -> find-DM-by-`counterpart` -> `latch()` **and** `selection.set('members', descriptor)`; **existing-DM ONLY** | svelte-check | gated on C-1, C-2, OQ-C2 |
| **C-4** | live verify (client 9222) + records | — | Rule 5, Chat re-drives |

🛑 **C-3 CORRECTED 2026-08-08 — CLAIR'S F2, RE-DRIVEN AND CONFIRMED. "draft otherwise" WAS NOT BUILDABLE IN LEG C AND CONTRADICTED §1 AND §7.** **The DM-draft object does not exist anywhere** — no store in `ui/common/lib/stores`, no state, nothing shipped by Legs A/B — and §1 forbids Leg C from reaching into Leg C-bis, which owns it. ⇒ **for a never-contacted member, `find` returns `undefined` and Leg C has nothing it is ALLOWED to write.** 🔓 ***Leg C therefore ships an EXISTING-DM-ONLY click, and what a never-DM'd click does in Leg C is an open question sited to Leg C-bis's Phase-0 — not silently "a draft".*** ⚠️ **AND A NAMING HAZARD FOR WHOEVER WRITES LEG C-bis:** `composer-panel.svelte:52` already has `let draft = $state('')` — **the textarea's TEXT BUFFER. Two different things called `draft` in one subsystem.**

🔑 **WHY C-1 IS ALONE:** it is the only commit that can move the catalogue, and the milestone Phase-0 locked that this be **measured rather than predicted**.
🔑 **WHY C-2 PRECEDES C-3:** C-3 cannot be written without knowing which writer it calls; and a latch method with no caller for one commit is the accepted `D-065` shape at **commit** granularity, not at **milestone** granularity — the `OQ1-G1` argument, applied.

---

## §7 — NOT TOUCHED

`composer-panel.svelte` · `echo-state.svelte.ts` · `canSend`'s second arm · R5's draft branch (**all Leg C-bis**) · `oncontextmenu` and `entity-context-menu` (**Leg D**) · `is_dm`, the DM home, the Spaces filter (**Leg E**) · `self_open` (**R-5: untouched, unregistered**) · the wire, `xgen-core`, `xgen-node`, `xgen-common` · `skin.css` (**Joe's**) · the four pre-existing clippy errors (**not a tracked floor; their own milestone if ever**).

---

## §8 — WHERE THIS DOCUMENT IS MOST LIKELY WRONG

1. ✅ **DISCHARGED 2026-08-07 — §3.3's SHRINK CASE WAS DRIVEN ON THE REAL CLIENT AND DID NOT FALSIFY.** `[tabindex]` census at the DOM layer: Engineering (2 rooms) → click index 1 → Design (1 room) ⇒ **`["-1"]`, tabbable 0, focusables 0**, `role="listbox"` intact. Svelte does **not** re-create the component on an `items` change. **OQ-C3 does not dissolve; it is live and it is Joe's.** *Superseded text follows.* — 🛑 **§3.3's SHRINK CASE IS READ, NOT DRIVEN.** `activeIndex` is not clamped **in the source**; that no row ends up tabbable **after a real roster change** has not been observed. **The falsifying probe is a `[tabindex]` census at the DOM layer, before and after** (`D-140`). *If Svelte re-creates the component on an `items` identity change, the defect does not exist and OQ-C3 dissolves.*
2. ✅ **CORPUS REPAIRED 2026-08-08 AFTER CLAIR'S ADVERSARIAL READ — THE CONCLUSION SURVIVES, THE MECHANISM DID NOT.** Two repairs: **(a)** the corpus was too narrow — `app_client.svelte:195-198` is where a room actually becomes latched, and it was NOT in the five named files; the bus has **FIVE readers** (`inspector-panel:40` · `self-panel:55` · `rooms-panel:25,:44` · `spaces-panel:45` · `app_client:196`), so `selection.svelte.ts:12`'s *"R8 is the only consumer"* — **Chat's own Leg A annotation** — stands FALSE. **(b)** 🛑 **`_latched` has exactly THREE write sites, all in `room-latch.svelte.ts`; there is no sixth writer.** ⇒ **the collision is real, but NOT for the reason §3.2 gave** — see §3.2's own annotation. *Superseded text follows.* — 🛑 **§3.2 ASSUMES LEG C LATCHES AT ALL.** If the DM is instead reached by writing a `room` selection from somewhere else entirely, the collision changes shape. **No such path was found; the corpus searched was `room-latch.svelte.ts`, `selection.svelte.ts`, `members-panel.svelte`, `rooms-panel.svelte`, `spaces-panel.svelte` (`D-139`).**
3. ⚠️ **§3.4's "no-op" IS ARGUED FROM C9/C10/C11 AND NOT RUN.** The DM re-entry path has never been clicked, because nothing is clickable yet. **First real exercise is C-4.**
4. ✅ **DISCHARGED 2026-08-07 — `counterpart` REACHES THE WEBVIEW, AND THIS IS THE FIRST TIME THE FIELD HAS BEEN READ BY ANYTHING.** Live client 9222, `window.__XGEN_SPACES__.spaces`: the key is present on **all five** Spaces, `null` on the four ordinary ones, and **`"xgen://pubkey/ed25519:L87…sno_FWmw"` on the DM.** 🔑 **The on-disk state holds NO `counterpart` key ⇒ the read-path migration is proven END TO END, disk → Rust → IPC → store** — gate 4b asserted on the Rust struct; this is the field crossing the boundary. ⇒ **`spacesState.spaces.find(s => s.counterpart === id)` will resolve.** *Superseded text follows.* — ⚠️ **THE FIND-DM SCAN ITSELF IS NOT DESIGNED HERE.** `spacesState.spaces.find(s => s.counterpart === id)` is the obvious form and `counterpart` is proven on the read path (C10) — **but the TS mirror's field has never been read by any consumer.** *First consumer, first proof.*
5. ⚠️ **`OQ7-W4`'s R7-during-a-draft is INHERITED, not re-verified here.** W4 says R7 shows self only. **§3.4's clickability question does not arise in a draft, because a draft roster has no members.**
6. 📌 **THIS DOCUMENT HAS NOT BEEN READ BY ANYONE OUTSIDE ITS AUTHOR.** 🔑 ***Chat's own re-reads have caught ZERO defects across seven arcs; every real one came from Clair executing or Joe looking.*** ⇒ **Clair's adversarial read is Leg C's Leg 0, and it runs before Joe locks anything.**
7. ⚠️ **FOUR OF THIS DOCUMENT'S FIVE ORIGINAL LIVE MEASUREMENTS WERE TAKEN ON THE SAMPLER, NOT THE CLIENT.** The catalogue, the cell census, the cell cost and the fusion are all sampler-side. ✅ **PARTLY DISCHARGED 2026-08-07:** two probes since ran **on the real client at 9222** — §3.3's shrink case and `counterpart`'s arrival in the store. ⚠️ **`members-panel` ITSELF STILL HAS NOT BEEN DRIVEN** — deliberately (R7 is inert, there is nothing to drive), but it means **C11's erased-row filter is read from source and has never been watched hiding a row.** 📌 *And the members fill read `state: "failed"` throughout the probe run — the node was not launched, which is R7's honest state ④ and not a defect.*
