# M-RP-MEMBER-ACT — the members panel acts: LMC opens the DM, RMC opens the menu — Phase-0
> **Status**: ACTIVE  
> Version: 1.9  
> Date: Aug 2026  
> **Last updated**: 2026-08-06  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — WHAT THIS IS, AND THE ONE SENTENCE THAT SIZES IT

R7's rows are inert. This milestone makes them act, in the shape Joe locked at **J-591** and re-locked and extended on **2026-08-06**.

🔑 **THIS IS A COMMAND-SURFACE MILESTONE, NOT A FEATURE MILESTONE.** `AUDIT_MEMBERS_PANEL.md` §5, re-measured at `d69c830`: `ops::create_dm_space` (`ops.rs:806`) and `ops::self_open` (`ops.rs:1002`) are **built, tested, idempotent, and unreachable from the UI** — `desktop.rs` exposes **19 `#[tauri::command]`s and neither is among them**. The DM machinery is not being written; it is being *wired*.

⚠️ **A CLAIM IN THE RECORD SAID THE SELF THREAD WAS UNBUILT.** It was inferred from where the item sat on the ROADMAP and never checked against the code (J-591). **A roadmap says what is planned, not what exists.** *Recorded here so this Phase-0's own sizing is not read as optimistic.*

**Prerequisite already satisfied (`D-071`):** the subsystem audit exists — `tasks/AUDIT_MEMBERS_PANEL.md`. This Phase-0 does not re-derive it; it cites it.

---

## §1 — THE LOCKS THIS MILESTONE INHERITS

🔒 **All of these were uttered by Joe. None is Chat's inference.** Provenance marked per `D-141`.

| # | lock | source |
|---|---|---|
| **L-1** | **LMC → DM (primary) · RMC → the avatar's context menu** (settings + invite-to-room, secondary) | J-591, uttered |
| **L-2** | **NOT a group-selection surface.** Group communication is *create a room, then invite from the avatar's RMC*. **NO multi-select, NO checkboxes, ever** | J-591, uttered, recorded as a prohibition |
| **L-3** | **THE SELF ROW OPENS A DM** — the Skype shape. *"self row in members' region? if so, it opens dm. similar to that in skype"* | J-591 / `M_RP_MEMBERS.md:414`, uttered |
| **L-4** | **CREATION IS LAZY.** LMC opens the thread, **empty if it does not exist**, no confirmation. Opening signs nothing; **first send** fires `create_dm_space` (3 events) + the message (a 4th). Ten drafts, zero trace | J-591 / `M_RP_MEMBERS.md:322-332`, uttered, Discord-evidenced |
| **L-5** | **`selected` IS NOT OVERLOADED — opening a DM IS the selection.** *"when i go to dm someone, i pick him. otherwise the dm thread will not display itself."* | J-591 **L14**, uttered; it dissolved Chat's objection |
| **L-6** | **NO CLIENT-SIDE BACKUP.** *"nothing will change. we have backup of the node, that has to be enough"* | J-591, uttered |
| **L-7 🆕** | **(a) — LMC DOES BOTH.** One gesture: **open the DM + write the selection bus + R8 shows the member's card.** Grounded in L-5 — the card is the pick made visible, not a second meaning on one click | 2026-08-06, uttered (*"(a)"*) |
| **L-8 🆕** | **NAVIGATION-ON-CLICK IS INTENDED.** Clicking a member in a 9-person room re-scopes R7 to the 2-member DM and that roster disappears. Discord's shape, adopted deliberately | 2026-08-06, uttered (*"yes, intended"*) |
| **L-9 🆕** | **RMC ON THE MEMBER AVATAR OPENS THE MENU *WITHOUT SELECTION*** — no bus write, no DM open, no R8 change, no navigation. This is the act-without-leaving escape hatch that makes L-8 acceptable | 2026-08-06, uttered |

🔓 **NOT YET LOCKED, and it is the only genuinely unrecorded ask:** **DM Spaces disappearing from the Spaces panel.** Corpus searched 2026-08-06 — `JOURNAL.md`, `JOURNAL_ARCHIVE.md`, `DECISIONS.md`, `CLAUDE.md`, `docs/ROADMAP.md` + every `.md` under `tasks/`, `docs/`, `ui/docs/` = **512 files**. **No decision exists.** See §5-OQ3.

---
## §2 — THE LOCK THIS MILESTONE REVERSES, AND THE ONE IT DOES NOT

### 2.1 — REVERSED, DELIBERATELY: J-591's *"R7 MUST NOT call `selection.set()`"*

Verbatim from J-591:

> 🔒 **R7 passes `selected={counterpartId}` LOCALLY and MUST NOT call `selection.set()`.** Had it written the bus, **opening a DM would silently change what R8 displays** — a local highlight hijacking an app-wide primitive.

🔑 **`L-7` INVERTS THE REASON RATHER THAN DISCARDING IT.** *"Silently"* was the objection. Under `L-7` it is the **point**: the user clicked a person and R8 shows that person. **The lock was right for the panel that existed; it does not survive the panel Joe now wants.** ⚠️ *Recorded as a reversal with its reason, not as a lock quietly dropped — `D-141`.*

### 2.2 — **NOT** REVERSED: `M-RP-PANEL-INERT`

⚠️ **CHAT CLAIMED THIS MILESTONE REVERSES `M-RP-PANEL-INERT` AND THAT WAS WRONG (2026-08-06, corrected by Joe).** `M_RP_PANEL_INERT.md` §0 records inertness as **deferred, not rejected**, and J-675 already filed that **`members-panel.svelte:11-14` overstates its own source.** *Chat quoted that known-overstated comment as authority.* ⇒ **`entity-panel`'s `interactive` prop is being USED as designed, not overridden.**

📌 **OWED, CHEAP, AND IT RIDES THIS MILESTONE (`D-131`):** the annotation on `members-panel.svelte:11-14` that J-675 filed and nobody has written.

---

## §3 — GROUNDED, MEASURED AT `d69c830`

| # | fact | site |
|---|---|---|
| **G1** | `create_dm_space` takes `CreateDmSpaceArgs { invitee: String }` — **an identity, not a relationship.** Any valid XGID; no prior contact needed | `app.rs`, `ops.rs:806` |
| **G2** | `self_open` takes **no id** — auto-resolves the session identity (M11-D5) — and returns `SelfThreadResult { space_id, room_id, created }`. **`created` is the idempotence signal** | `ops.rs:1019` — ⚠️ *v1.0/v1.1 cited `:1002`, which is inside the result struct. Corrected at v1.2 from Clair's **M1**; `M_RP_MEMBERS.md:309` carries the same drift and is annotated in Leg A* |
| **G2a** | 🔑 **THE COUNTERPART IS RECORDED IN THE DISPLAY NAME.** `create_dm_space` writes `KnownSpace.name = format!("DM with {}", args.invitee)` for a peer, or the constant `SELF_THREAD_LABEL` (`"self"`) when `invitee == identity_id` | `ops.rs:965-969` |
| **G2b** | 🔑 **AND THAT NAME IS ALREADY USED AS A LOOKUP KEY, SHIPPED AND TESTED.** `self_open` finds an existing thread by `s.role == "owner" && s.name == SELF_THREAD_LABEL` | `ops.rs:1039-1042` |
| **G2c** | 🛑 **`self_open` IS EAGER WHERE `L-4` IS LAZY.** Absent ⇒ it calls `create_dm_space` **immediately** and returns `created: true` — three signed events on the first click | `ops.rs:1052-1056` |
| **G3a** | 🛑 **`composer-panel.submit()` HARD-BAILS ON A NULL id** — `if (!text \|\| spaceId == null \|\| roomId == null) return;` — and both ids come from `roomLatch`. `echo.send(spaceId: string, roomId: string, text: string)` has no draft arm | `composer-panel.svelte:66-77`, `echo-state.svelte.ts:162` |
| **G3** | `desktop.rs` exposes **19** commands: `apply_window_geometry · fetch_identity · fill_space_records · get_about_info · get_address_book · get_conn_stats · get_pacing_state · get_resident_status · get_self_state · get_spaces · get_state · get_substitutions · get_ui_state · get_window_geometry · quit · resume_resident · send_message · set_substitutions · set_ui_state`. **Neither DM op is among them** | measured |
| **G4** | `entity-panel` already has `interactive?: boolean` (default `true`), `onActivate?: (id) => void`, and `selected` **`$bindable`** | `entity-panel.svelte:51-74` |
| **G5** | 🛑 **`selectAt` writes `selected = it.descriptor.id` UNCONDITIONALLY, *before* calling `onActivate`** | `entity-panel.svelte:101-106` |
| **G6** | R7 passes `selected={counterpart}` as a **`$derived`, non-bindable** value | `members-panel.svelte`, EntityPanel mount |
| **G7** | `entity-context-menu` takes **`descriptor: EntityDescriptor` as a PROP**, is **gesture-agnostic** (`open(anchor)`/`close()`, consumer wires the trigger), and **does not import `selection` at all**. `MenuAnchor` = element \| DOMRect \| `{x,y}` | `entity-context-menu.svelte:14, 55-72` |
| **G8** | 🛑 **ZERO `oncontextmenu` handlers exist in project code.** All 30 matches are `node_modules` type declarations | measured across `ui/**` |
| **G9** | `roomLatch.resolveLatched()` resolves **only against the known-Space tree**; nothing else can be latched | `room-latch.svelte.ts:43-47` |
| **G10** | 🛑 **Lock #12: no latched room ⇒ `canSend` false.** One predicate, read by the composer at `:56` and gating `sendEnabled` at `:58` | `room-latch.svelte.ts:73`, `composer-panel.svelte:56-58` |
| **G11** | **`KnownSpace` = `space_id · name · node_endpoint · role · rooms`. NO `is_dm` field** — the Spaces panel cannot tell a DM from a Space except by the display string | `xgen-common/src/state.rs` |
| **G12** | `spaces-panel.svelte:31` already reserves **`flags.isDm`** (J-501) to draw a circle instead of a rounded square. **The field was expected and never arrived** | measured |
| **G13** | 🛑 **`is_dm` is set once at Space creation and never changed.** A DM promoted to a full Space keeps the flag ⇒ the counterpart highlight picks an arbitrary member once there are 3+ | `AUDIT_MEMBERS_PANEL.md` §8, filed 2026-07-31 |

---

## §4 — THE THREE COLLISIONS, GROUNDED

### 🛑 4.1 — `selected` HAS TWO WRITERS THE MOMENT R7 GOES INTERACTIVE (G5 + G6)

`selectAt` writes the child's local `selected` **before** `onActivate` fires. R7's `selected` is `$derived(counterpart)` and **non-bindable** ⇒ the child's write does not propagate back, and `M_RP_PANEL_INERT.md` §0 already measured the consequence: *"a click writes the child's copy at `:91`, nothing propagates back, and the wrong highlight sticks until the roster changes. In a group room, where the highlight must be `null`, one click manufactures one."*

⚠️ **UNDER `L-8` THIS PARTLY SELF-CORRECTS AND THAT IS EXACTLY WHY IT NEEDS DECIDING RATHER THAN ASSUMING.** The click navigates, the roster is replaced, and the new counterpart *is* the clicked member — so the wrong highlight is short-lived. **But "wrong for one frame during a navigation" is a claim about paint, and this project measured one day ago that paint does not follow from string and layout (`N-168`).** ⇒ **§5-OQ1.**

🛑 **RE-DERIVED 2026-08-06 — §4.1 AND CLAIR'S F5 ARE BOTH WRONG, IN THE SAME DIRECTION. ANNOTATED, NOT REPAIRED (`D-131`).**

**Measured:** `entity-panel` has **NO `$effect` re-syncing `selected` from props.** `selectAt:105` is a plain write to a `$bindable` passed **without `bind:`**, so the child's value persists until the **parent's** value changes. ✅ *That confirms `M_RP_PANEL_INERT.md` §0 exactly.*

🔑 **BUT §0's CLAIM WAS NEVER RE-READ AGAINST `L-8`, WHICH IS LOCKED IN THIS SAME DOCUMENT.** §0 describes a panel where clicking does **nothing else**: click X in a group room, stay there, X stays wrongly highlighted because the roster never changes. **Under `L-8` you always leave — and you leave TO A DM WITH X, where X IS the counterpart.** ⇒ the child writes `selected = X`; the eventual correct value **is also X**.

🔑 ***THE MANUFACTURED HIGHLIGHT IS NOT WRONG. IT IS EARLY.***

| case | what actually happens |
|---|---|
| group room → click X → DM with X opens | child writes `X`; the new roster's `counterpart` **is** `X`. **Same value, nothing to correct** |
| click X → **draft** (never contacted) | under **OQ7-R2** the roster empties and the rows unmount, taking the highlight with them; under **OQ7-R1** the pseudo-roster's counterpart is `X`. **Consistent either way** |
| 🛑 click X → **the open FAILS or is cancelled** | you stay in the group room with `X` spuriously highlighted **and nothing clears it** |

⇒ **THE ENTIRE RESIDUE IS AN ERROR PATH.** Not a frame, not a navigation transition. 📌 **And the cheapest guard for it is that the click does not write `selected` at all until the open succeeds** — an `onActivate`-only mount, **not a change to `entity-panel`** ⇒ ***the `ui/core` touch F5 warned about may not be needed at all.*** *To be confirmed against `entity-panel`'s prop surface when Leg C's runbook is written; the direction is established, the confirmation is not.*

🛑 **CONFIRMED 2026-08-06 AND IT RESOLVES AGAINST THE PARAGRAPH ABOVE — ANNOTATED, NOT REPAIRED (`D-131`). CLAIR'S LEG 0-bis **F-C**.** **THE `onActivate`-ONLY MOUNT DOES NOT EXIST.** Measured at `entity-panel.svelte:101-107`: `selectAt` writes `selected = it.descriptor.id` (`:105`) and calls `onActivate?.()` (`:106`) in one body, and **`interactive` gates BOTH together** (`:177`). There is no mount that takes the callback without the write. ⇒ ***the `ui/core` touch F5 warned about IS needed, and OQ1 was closed "no longer provisional" on a mechanism the code does not offer.*** 🔑 **The document itself said *"to be confirmed against `entity-panel`'s prop surface"* and the confirmation was never run before the disposition closed.**

🔑 **AND THE MEASUREMENT FOUND THE REAL SHAPE, WHICH IS LARGER THAN A MISSING GUARD.** `interactive` fuses **three** separable concerns — ① the ARIA contract (`role="listbox"` vs `list`, `:171-177`), ② the click/keyboard wiring, ③ the `selected` write. **Leg C needs ① and ② and must NOT have ③, and a boolean cannot say that.** ⚠️ ***`M-RP-PANEL-INERT`'s own doc comment at `:65-73` diagnosed exactly this and solved it by switching all three OFF at once*** — *"the highlight can never be WRITTEN from inside the component … R7, whose `selected` is a data-driven DM highlight with no feedback loop, so entity-panel's own `selectAt` would drift it."*

📌 **AND THE OTHER TWO CONSUMERS ARE ONLY ACCIDENTALLY CORRECT.** `rooms-panel:65` and `spaces-panel:62` both mount `{selected} {onActivate}` with `interactive` defaulting **true**, so the child writes optimistically **and** the parent sends the same value straight back down. ***The write is a duplicate that is always confirmed — until a consumer has a path where the parent does NOT confirm, which is precisely R7's failure path*** (`counterpart` stays `undefined`, `members-panel.svelte:111-112`).

⚠️ ***THREE PASSES, THREE CORRECTIONS, ONE QUESTION — §4.1 (Chat), F5 (Clair), this (Chat). EACH WAS RIGHT ABOUT WHAT IT LOOKED AT. NONE OF THE THREE CHECKED THE CLAIM AGAINST A LOCK IN THE SAME DOCUMENT.***

### 🛑 4.2 — THE DRAFT THREAD AND THE ROOM LATCH DO NOT COMPOSE, AND LOCK #12 FORBIDS THE ACT THAT CREATES THE DM

`L-4` says a draft is *"an in-memory object holding only the target identity — no `space_id`, no `room_id`"*. **G9**: `resolveLatched()` resolves only against the known-Space tree. ⇒ during a draft, `effectiveRoomId` and `effectiveSpaceId` are both `null`, so:

- the stream shows *"select a room"*,
- R7 falls to state ① (self only) — **the panel you just clicked in empties**,
- **G10**: `canSend` is **false**.

🔑 **THE FIRST SEND IS THE ACT THAT CREATES THE DM SPACE. THE SHIPPED GATE FORBIDS IT.** Lazy creation cannot fire its own first message through the current composer.

⚠️ **THIS IS THE *"WHICH LEG BUILDS THIS?"* GAP, NAMED IN THIS PROJECT'S OWN AUDIT** (`AUDIT_MEMBERS_PANEL.md` §6): *scope gets written in terms of files, requirements in terms of behaviours, and nobody reconciles the two.* **It is being reconciled here, before a runbook exists.** ⇒ **§5-OQ2.**

### 🛑 4.3 — `L-7` FORBIDS THE CHEAP FIRST LEG

The R8 card alone is two lines (`interactive`, `onActivate`) and needs nothing else — `inspector-panel` is fully kind-agnostic (`:77-89`, conditional flags row at `:47`), so an identity descriptor renders today.

**But `L-7` says one gesture does both.** Shipping the card first means LMC lights up and **does not open the DM** — an affordance promising an interaction that is not wired. ***That is precisely the shape `M-RP-PANEL-INERT` was created to refuse*** (§0: *"R7 would ship six affordances promising interactions Leg B does not wire"*). ⇒ the leg order in §6 puts the command surface first, and **§5-OQ4** offers Joe the alternative explicitly rather than assuming.

---
### 🔒 LEG 0 CLOSED 2026-08-06 — CLAIR'S ADVERSARIAL READ RAN. **FIVE PLAN-MOVING FINDINGS, THREE WORDING. ALL EIGHT RE-MEASURED BY CHAT.**

✅ **THE THREE CRITICAL FACTS SURVIVED** — the 19 commands (`invoke_handler` registers exactly 19, `:1085-1105`), `selectAt`'s unconditional write before `onActivate`, and `entity-context-menu`'s prop-not-bus contract. **G8, G11, G12 confirmed too.**

🔑 **F1 — §8 ITEM 2 WAS WRONG, AND WRONG IN THE DIRECTION THAT MAKES THE PLAN EASIER.** Chat doubted whether the find-existing-DM scan is buildable, because `KnownSpace` carries no `is_dm` and no counterpart. **It is buildable, against a shipped and tested mechanism:** the counterpart is embedded in `KnownSpace.name` (**G2a**) and `self_open` already scans on exactly that field (**G2b**). ⇒ **§6's order (B before E) is CORRECT and `is_dm` stays in Leg E.** *Chat suspected the right section and drew the wrong conclusion from it.*

🛑 **AND CHAT ADDS ONE FINDING TO F1 THAT THE READ DID NOT DRAW — IT IS A COLLISION, NOT A RELIEF.** `KnownSpace.name` now carries **two meanings**: *what the user is shown* and *which counterpart this Space is for*. 🔑 ***THAT IS THIS PROJECT'S NAMED DEFECT CLASS — one token, two meanings — and this arc has hit it five times*** (`T3` · `word` · `tail()` · mechanism-vs-surface labels · `selected`). ⚠️ **AND IT COLLIDES DIRECTLY WITH OQ3:** a DM home that renames DM Spaces to something humane — or `D-126`'s word form ever reaching that label — **breaks the scan silently.** *The scan is not free; it is borrowed against a field OQ3 wants to change.* ⇒ **OQ8.**

🛑 **F2 — THE STRONGEST FINDING: `L-4`'s FIRST SEND IS ASSIGNED TO NO LEG.** OQ2-S2 makes `canSend` two-armed — **that fixes the BUTTON, not the SEND.** `submit()` hard-bails on a null id and `echo.send` has no draft arm (**G3a**). The real path is **detect draft → `create_dm_space` → receive `{space_id, room_id}` → `echo.send` → promote the draft**, and **neither Leg B nor Leg C names `composer-panel.svelte` or `echo-state.svelte.ts`.** ⚠️ ***THIS IS THE "WHICH LEG BUILDS THIS?" GAP, FOUND INSIDE THE DOCUMENT THAT CONGRATULATES ITSELF ON RECONCILING IT IN §4.2.*** ⇒ **§6 gains Leg C-bis.**

🔑 **F3 — `L-3` AND `L-4` ARE DIFFERENT CREATION MODELS AND §6 WIRED THEM AS ONE.** `self_open` is **eager** (**G2c**): the first click on the self row signs three events and hits the network. A peer click is **lazy**. ⇒ **the two rows in one panel have different irreversibility, and Leg C would have shipped that silently.** ⇒ **OQ6.**

🔑 **F4 — `L-8` AND §4.2 DESCRIBE OPPOSITE R7 OUTCOMES AND THE DOCUMENT NEVER NOTICED.** `L-8` promises the panel re-scopes to the 2-member DM; §4.2 says during a draft R7 **empties** to state ①. **Both are right — for different cases:** an existing DM has a room in the tree and latches; a never-contacted draft does not. ⇒ **OQ7.**

⚠️ **F5 — OQ1-P1's COST IS OVERSTATED AS CERTAIN AND UNDERSTATED AS SIZE.** §4.1 lists *"one prop or one guard"* as a flat requirement of P1. It is **(a) contingent on the unmeasured paint** (§8 item 1 — under `L-8` every click navigates, so the spurious highlight self-corrects), and **(b) if incurred, it lands on `ui/core/.../entity-panel.svelte`** — the shared core composite whose last change was given **its own milestone** (`M-RP-PANEL-INERT`) precisely so a core touch stays attributable. ⇒ **not a Leg-C rider.** *§4.1 is annotated, not repaired.*

📌 **WORDING, ALL THREE ACCEPTED:** **M1** `self_open` is `:1019` not `:1002` — fixed at **G2**, and `M_RP_MEMBERS.md:309` carries the same drift plus `create_dm_space :793` (it is **`:806`**) · **M2** `M_RP_MEMBERS.md:309` and `:406` both say **18** commands where it is **19** — *the Phase-0 is right and the document it leans on is stale* · **M3** §4's heading says three collisions and 4.3 is a sequencing argument — noted, no change.

🔑 **WHAT THIS LEG PROVES ABOUT THE PROCESS, NOT THE PLAN.** §8 named six self-doubts. **Clair confirmed one, DISSOLVED another (item 2), and found three things §8 did not suspect at all — including the milestone's central behaviour having no owner.** ⚠️ ***Chat's own re-reads of this document had passed. Again.***

---

### 🔒 LEG 0-bis CLOSED 2026-08-06 — CLAIR'S **SECOND** ADVERSARIAL READ, OF v1.8. **ONE QUESTION: DO THE EIGHT DISPOSITIONS COMPOSE? THEY DO NOT.**

🔑 **WHY IT RAN AT ALL:** she certified **v1.1**; the document had reached **v1.8**. Measured `git diff --stat 344fe45..fd827d2 -- tasks/M_RP_MEMBER_ACT_PHASE0.md` = **208 insertions / 14 deletions** across three commits. ***The document she certified was not the document a runbook would be written from.***

**FOUR FINDINGS, ALL RE-MEASURED BY CHAT AND ALL CONFIRMED, PLUS A FIFTH CHAT DREW FROM HER MINOR.**

🛑 **F-A — `OQ6-E2` × `OQ8-K3` DO NOT COMPOSE FOR THE SELF ROW.** Leg B says *"the scan keys on the FIELD, not the name"*; **`OQ6-E2` point 1 says self is *"found by Leg B's name scan"***. Both are in scope, and they contradict. Grounded: `ops.rs:1042` is `s.role == "owner" && s.name == SELF_THREAD_LABEL` — **a name scan, carrying the exact unsoundness `D-143`/K3 was minted to remove**, and `OQ6-E2` puts it on the click path. `create_dm_space` pushes **unconditionally** (`ops.rs:970`, no dedup), so a failed lookup **mints a second self thread**. ⚠️ **CHAT'S CORRECTION TO THE SECOND HALF:** her *"orphaning Joe's live one"* is **not reachable today** — his live state holds five Spaces (`Engineering`, `Design`, `LegBSpace`, `LegF Verification`, `DM with …sno_FWmw`) and **no self thread**; a self thread created after K3 gets `counterpart` written at creation. **The backfill half shrinks; the lookup contradiction stands untouched and is the load-bearing half.** *Clair scoped it to that herself on the hand-back.*

🛑 **F-B — `OQ7-W4`'s *"R5 gets the empty-thread copy"* IS A DISPLAY PATH NO LEG OWNS.** `stream-panel.svelte:155-157` branches on `effectiveRoomId == null` alone, and its ten imports contain **no draft store**. Leg C-bis names only `composer-panel` + `echo-state` ⇒ **the const is owned, the branch that renders it is not.** 📌 **Second-order: W4's *"adds no third consumer of the draft store"* becomes R5 + R6**, not R6 alone.

🛑 **F-C — OQ1 CLOSED ON A MECHANISM `entity-panel` DOES NOT OFFER.** See §4.1's annotation. **Chat's error; the fourth pricing of OQ1's guard.**

🛑 **F-D — `OQ3-A3`'s DM FILTER MUST BE RENDER-ONLY, OR EVERY DM BECOMES UNREACHABLE.** `spacesState.spaces` has exactly three readers: `spaces-panel:37`, `rooms-panel:32`, **`room-latch.svelte.ts:46` (`resolveLatched`)**. ⚠️ **CHAT'S SHARPENING, AND IT IS WORSE THAN SHE STATED: `canSend` DERIVES FROM `resolveLatched`** (`room-latch.svelte.ts:73-75`) ⇒ filtering DMs out of the **store** does not merely blank R5/R6/R7, ***it makes every DM UNSENDABLE via Lock #12.*** **The panel that exists to start DMs would produce DMs you cannot write in.** ⇒ **the filter lives in `spaces-panel`'s `$derived`, never in the store.**

🛑 **F-E — `OQ2-S2`'s COST STATEMENT IS FALSE (Chat, drawn from Clair's minor note).** S2 was taken on *"the latch is **untouched**; one store gains one field and one predicate gains one arm"*, against S1 whose entire objection was blast radius. **`canSend` lives AT `room-latch.svelte.ts:73`. Making it two-armed IS editing the latch.** ⇒ S2 remains the smaller change, but **the sentence that won it is wrong**, and OQ2 is an **architecture** disposition taken on *"go by your recomms"*. 📌 **`canSend` also belonged to no leg** — now Leg C-bis's.

✅ **THE FOUR "VERIFY, DON'T RE-DERIVE" INTERACTIONS CHAT HANDED HER:** three verified sound; **one was defective** — `OQ6-E2 × OQ8` was listed as settled and is settled **for peers only**. 🔑 ***Had she trusted the list, F-A would have shipped.*** ⚠️ **FORWARD: a "verify, don't re-derive" list is an ASSERTION LIKE ANY OTHER and gets no exemption from the read it is handed to.**

🔑 **AND §8 ITEM 7 FIRED AGAIN, INSIDE THE DISPOSITION THAT NAMES IT.** F-C is Chat closing OQ1 on an unverified mechanism **in the same edit that declared the re-check rule closed.** ***Chat's own re-reads of v1.8 had passed. Seven consecutive arcs: every real defect came from outside the text — Clair executing it, or Joe looking at a screen.***

---

## §5 — OPEN, AND JOE'S. Each carries `D-121` lenses: ① user-visible impact per option, then ② resource cost.

---

### 🔒 DISPOSITION 2026-08-06 — **FOUR ADOPTED, ONE STILL OPEN. PROVENANCE: DELEGATED.**

⚠️ **JOE: *"let's go by your recomms"*. 🔑 THAT IS ADOPTION, NOT EXAMINATION, AND IT IS RECORDED AS SUCH SO A LATER REVISIT READS IT CORRECTLY** (`D-141`; the `D-127` shape). *The identical phrase carried `M-RP-TAIL8`'s mechanism **M3**, and that runbook marks it the same way. `AUDIT_MEMBERS_PANEL.md` §8 names the cost of the pattern: three decisions of record that nobody judged.*

| # | adopted | seat it really belongs to |
|---|---|---|
| **OQ1** | **P1** — `selected` stays the DM counterpart | appearance-adjacent ⇒ **Joe's**, taken on recommendation · 🔒 **CLOSED 2026-08-06 on the §4.1 re-derivation — no longer provisional** |
| **OQ2** | **S2** — the draft sits beside the latch; `canSend` becomes two-armed | 🔑 **ARCHITECTURE — Joe's reserved area** (`D-123`), taken on recommendation |
| **OQ3** | **A3, sequenced last** — `is_dm` plumbed + filter, but **a DM home ships first** | appearance + architecture ⇒ **Joe's**, taken on recommendation |
| **OQ4** | **NO** — the R8 card does not ship before the DM opens | sequencing ⇒ Chat's, and it was Chat's to take |
| **OQ7** | **W4** — the draft renders where it must; **R7 stays thin because R8's card carries the counterpart** | appearance ⇒ **Joe's**, taken on recommendation · 🔒 **CLOSED 2026-08-06** |
| **OQ6** | **E2 (lazy)** — one rule for every row; self is a draft target like any peer | 🔑 **CHAT's under `D-123`** — the only user-visible difference is an offline error; the rest is cost. ⚠️ *Routed to Joe twice in error* |
| **OQ8** | **K3, in Leg B** — a real `counterpart` field + a one-time backfill; the parse lives in a migration, never in a lookup | 🔒 **JOE, UTTERED** — *"we must not be afraid of proper solutions, even if they are slightly heavier"* · **the only OQ this arc he did not take on recommendation** |
| **OQ9** | **C** — the const ships with an honest note; **no control built**; destination NAMED as Settings › an app-level copy section | 🔒 **JOE** — client-owned (*"quasi private message, just for user"*) · *"not so important, but we can access it in the future by a need"* |

🔒 **AND THE META-RECOMMENDATION IS ADOPTED WITH THEM: CLAIR READS THIS DOCUMENT ADVERSARIALLY BEFORE ANY RUNBOOK EXISTS.** ⇒ **these four dispositions are ADOPTED but PROVISIONAL** — Clair's read is the next gate, and §8's own record (five defects in `RUNBOOK_TAIL8.md`, every one caught from outside the text) is why.

🔓 **OQ5 IS NOT ANSWERED BY THIS AND MUST NOT BE READ AS ANSWERED.** **Chat made no recommendation on any of its three items**, so *"go by your recomms"* cannot reach them. Two need Joe:
- **the partial first send** — `M_RP_MEMBERS.md:336` already routes it here and calls it Joe's;
- **erased members are clickable** — new, unrecorded anywhere before 2026-08-06.

The third (**cross-node invite discovery**) is a **measurement nobody has taken**, not a decision — Chat's to run, and it is not a prerequisite for Legs A–D.

---

### 🔒 OQ1 — **CLOSED 2026-08-06: P1. PROVENANCE DELEGATED** (*"we go by your recomm"*, `D-141`).

🔒 **`selected` STAYS THE DM COUNTERPART.** 🔑 **AND IT IS NO LONGER PROVISIONAL ON A PAINT MEASUREMENT** — §4.1's re-derivation shows the deciding question was **code semantics plus `L-8`**, both readable today. §8 item 1 said the paint decides it. **It does not.**

📌 **P1's COST, THIRD AND CURRENT PRICING:** not *"one prop or one guard, cheapest"* (§4.1 v1.0) and not *"contingent on an unmeasured paint"* (F5). It is **a guard on the FAILURE PATH ONLY**, and its cheapest form is *the click does not write `selected` until the open succeeds* — likely **no `ui/core` touch at all.**

🛑 **SUPERSEDED 2026-08-06 BY CLAIR'S F-C — THE FIFTH PRICING, AND THE FOURTH TIME IT MOVED. ANNOTATED, NOT REPAIRED (`D-131`).** The mechanism named above **does not exist** (§4.1). 🔒 **RULED: G1 — THE `core` PROP SPLIT.** `interactive` keeps the ARIA contract and the click/keyboard wiring; **a new flag suppresses the `selected` write.**

🔒 **AND `D-143` DECIDES IT RATHER THAN PREFERENCE.** The alternative (G2 — R7-side `bind:selected` + local `$state` + reset in `onActivate`) creates **two writers of one piece of state with no defined precedence** ⇒ a claim that can go false with nothing to decide it ⇒ **unsound** ⇒ D-143 fires. ⚠️ ***G2 is not a guard; it is the original §4.1 defect plus a correction race — building the collision this milestone opened by diagnosing.***

🔑 **G1 RIDES LEG C AS ITS FIRST COMMIT. NOT ITS OWN MILESTONE.** ⚠️ ***Chat recommended own-milestone twice and reversed on two arguments it had not drawn:*** ① an own-milestone G1 **ships a prop with ZERO consumers** until Leg C opts in — `D-065`'s no-empty-machinery case, and `D-143` explicitly hands the floor to `D-065` where the cheap option is sound; ② the attribution argument **dissolves at commit granularity** — `D-074` atomic commits already separate a `core` change from its consumer, which is why Leg C-bis exists. 📌 **And `M-RP-PANEL-INERT` is not the precedent claimed: it got a milestone because it CHANGED THE BEHAVIOUR of shipped consumers. G1 changes nobody's behaviour until Leg C opts in.** 🔓 **Seat: milestone split is Joe's by `D-123`:4610 (*"what gets built and in what order"*); the dock-engine arc grant is a scoped exception that proves the default. Joe DELEGATED this one (`D-141`) — adopted, not examined.**

📌 **COST, AND ONE PIECE IS DELIBERATELY UNMEASURED:** one prop on `ui/core/.../entity-panel.svelte` · one guard in `selectAt` · **3 real consumers** (`members-panel:166`, `rooms-panel:65`, `spaces-panel:62`) **+ 8 sampler cells**. ⚠️ **Whether the catalogue floor of 435 moves at all is NOT predicted here — Leg C's runbook MEASURES it.** *The own-milestone case leaned on that floor moving, and it was never checked.*

⚠️ **WHAT REMAINS GENUINELY UNMEASURED IS NARROWER AND IS NOT A BLOCKER:** how LONG the early highlight shows before the roster catches up — the DM-open round trip. **Bounded, on the success path harmless, and unmeasurable until Leg B exists.**

🔓 **AND OQ7 IS NOT ANSWERED BY THIS.** *Chat asked whether OQ1 should be held until OQ7 settled and recommended it need not be; Joe took the recommendation.* **OQ1 is stable under BOTH R1 and R2** — but **R2 additionally clears the failure case for free**, so OQ7 still has a bearing on how much guard Leg C writes. *Stated so the phrase is not later read as having settled two questions.*

### 🔓 OQ1 — what does R7's `selected` mean once rows are clickable?

**P1 — `selected` stays the DM counterpart** (data-derived, today's meaning; `L-5`'s wording).
① The highlight always answers *"who am I talking to"*. In a group room nothing is highlighted even right after a click, so the click's only feedback is the navigation itself. Risk: one frame of a stale highlight during the transition (4.1).
② Cheapest. No change to the derivation. Needs `entity-panel`'s unconditional `selectAt` write neutralised for this consumer — one prop or one guard. ⚠️ **AMENDED v1.2 (Clair **F5**, annotated not repaired):** that cost is **CONTINGENT, NOT CERTAIN** — under `L-8` every click navigates and the spurious highlight self-corrects, so the guard is needed **only if the unmeasured paint is unacceptable** (§8 item 1). **And if it IS incurred it lands on `ui/core/.../entity-panel.svelte`** — the shared core composite whose last change got **its own milestone** so a core touch stays attributable. ⇒ ***not a Leg-C rider, and "cheapest" was asserted before either half was checked.***

**P2 — `selected` becomes `selection.current.entity.id`**, exactly like R1/R2.
① The highlight answers *"who did I last click"*, which in a DM is the same person, so the visible difference is narrow — **except when a DM is reached any other way** (the Spaces tree today), where **no member would be highlighted at all**. That is a visible regression against `L-5`'s purpose.
② Same order of cost; makes R7 structurally identical to its siblings, which is worth something and is **tertiary** under `D-121`.

📌 **Chat's recommendation: P1**, because P2's regression is real and `L-5` bought the counterpart highlight on purpose. **Grounded caveat: the one-frame claim in 4.1 is UNMEASURED** — no probe has been run. *Stated as a gap, not smoothed over.*

### 🔓 OQ2 — where does the draft thread live, so that first-send can fire?

**S1 — the latch learns a draft kind.** `roomLatch` gains a draft state carrying only the target identity; `canSend` becomes true for it.
① The user sees the Discord behaviour exactly: empty thread, composer live, type and send. R7 shows the two of you rather than emptying.
② Touches the shared latch, which **R5 stream, R6 composer and R7 members all read**. The blast radius is every conversation surface. A regression here breaks sending in normal rooms.

**S2 — the draft lives beside the latch, and `canSend` becomes a two-armed predicate** (`a room is latched` OR `a draft is open`).
① Identical to S1 for the user.
② Smaller: the latch is untouched; one store gains one field and one predicate gains one arm. **But it creates a second scope authority** — the `D-067` drift the address-book store centralised its rules to prevent.

🛑 **CORRECTED 2026-08-06 — CLAIR'S LEG 0-bis **F-E**. ANNOTATED, NOT REPAIRED (`D-131`): *"THE LATCH IS UNTOUCHED" IS FALSE.*** `canSend` lives **at `room-latch.svelte.ts:73`** and is read by `composer-panel:56`, gating `sendEnabled` at `:58`, and passed into `app_client:417`. **Making it two-armed IS editing the latch.** ⇒ S2 is **still the smaller change** — one added arm versus S1's new state kind — but ***the sentence that won it over S1 was wrong, and S1's objection (blast radius on the shared latch) applies to S2 in reduced form rather than not at all.*** ⚠️ **OQ2 is an ARCHITECTURE disposition taken on *"go by your recomms"* — Joe adopted a cost statement that did not hold. The disposition is NOT reversed; the record is corrected so a revisit reads it accurately.**

**S3 — first send is not lazy for a never-contacted person: the DM Space is created on OPEN.**
① **Reverses `L-4`.** The recipient gets an invitation the moment you click their name. Ten curious clicks = ten Spaces on their node.
② Cheapest by far — no draft object at all.

📌 **Chat's recommendation: S2**, and it is a close call against S1. The `D-067` objection to S2 is real but bounded (one predicate, one file), whereas S1 puts the composer's send gate at risk on every ordinary room. ⚠️ **S3 is listed because it is honest, not because it is live — it contradicts a lock Joe took with evidence in hand, and it should not be adopted without re-opening `L-4` explicitly.**

### 🔓 OQ3 — do DM Spaces leave the Spaces panel? (the ask with no record)

**A1 — leave them.** ① The tree keeps showing `DM with xgen://pubkey/…`, a raw XGID where a name belongs. ② Zero.

**A2 — plumb `is_dm` and filter.** ① DMs vanish from the tree; the Spaces panel becomes servers-only, Discord's shape. 🛑 **And G13 bites: `is_dm` never changes, so a DM PROMOTED to a real Space stays hidden from the tree forever.** ② `KnownSpace` gains a field (**Rust — the cargo floor returns**), plus the TS mirror and one filter.

**A3 — plumb `is_dm`, filter, AND give DMs a home first** (a DM list surface).
① The complete Discord shape. **Without a home, a DM with someone you no longer share a room with is unreachable** once it leaves the tree.
② A2's cost plus a new region/surface — the largest piece in this milestone by a wide margin, and it is appearance, so it is yours.

📌 **Chat's recommendation: A3, sequenced last** (§6 Leg E). **And G13 is a prerequisite of A2/A3, not a footnote** — filtering on a flag that lies makes the lie load-bearing in a second place. *Whether `is_dm` means "was born a DM" or "is a DM" is undecided in the code and the field carries no comment either way (`AUDIT_MEMBERS_PANEL.md` §8).*

🔒 **CONSTRAINT ADDED 2026-08-06 — CLAIR'S LEG 0-bis **F-D**, SHARPENED BY CHAT. THE FILTER IS RENDER-ONLY.** `spacesState.spaces` has **exactly three readers**: `spaces-panel:37` (tree render) · `rooms-panel:32` · **`room-latch.svelte.ts:46`, inside `resolveLatched`** — the scope every conversation region resolves through. 🛑 **AND `canSend` DERIVES FROM `resolveLatched` (`:73-75`)** ⇒ removing DMs from the **store** does not merely blank R5/R6/R7: ***it makes every DM UNSENDABLE via Lock #12.*** **The milestone whose purpose is to start DMs would ship DMs you cannot write in.**

⇒ 🔒 **A3's filter lives in `spaces-panel`'s `$derived` ONLY. The store retains every DM so the latch, `canSend`, and the DM home can all still resolve them.** ⚠️ *Written here because `is_dm` naturally lands where the store is built, which is exactly the wrong place, and Leg E is appearance work where a mechanical trap is easy to walk into.*

### 🔓 OQ4 — may the R8 card ship before the DM opens?

① **Yes:** the card is useful on its own and lands weeks earlier; the cost is a click that selects but does not navigate, for one milestone's duration. **No:** nothing half-wired ever reaches the user.
② Yes saves nothing in total; it only re-orders. **It buys feedback on the card's content early**, which matters because *"just data that we have now"* has never been seen on a member.

📌 **Chat's recommendation: NO** — §4.3's precedent is this project's own and it was expensive to learn. *Offered because it is a real choice, not to be talked out of.*

### 🔒 OQ6 — **CLOSED 2026-08-06: E2 (LAZY). 🔑 TAKEN BY CHAT UNDER `D-123`, NOT BY JOE — AND THE ROUTING WAS THE ERROR.**

⚠️ **CHAT PUT THIS TO JOE AS A DESIGN CHOICE FOR TWO TURNS. IT IS NOT ONE.** Joe: *"honestly i dont know what you need to work it as as in the skype time ago."* 🔑 **HE COULD NOT PICTURE THE DIFFERENCE BECAUSE THERE IS NONE TO PICTURE.** The Skype shape `L-3` reaches for is: your own entry sits in the list · clicking it opens a normal conversation with yourself · it persists · it is the same thread from any device. ***All four hold under E1 AND E2.*** Whether the thread is minted on the click or on the first message **is invisible in that experience.**

⇒ **the only USER-VISIBLE difference is the offline error — everything else is implementation cost, which is Chat's seat.** ***Routing it to Joe twice was the under-stepping the record keeps naming.***

🔒 **E2, ON FOUR MEASURED GROUNDS:**
1. 🔑 **IT COSTS NOTHING NEW AND REMOVES WORK.** Self becomes a draft target like any peer — found by **Leg B's name scan** (`name == "self"` is the same lookup, `ops.rs:1039-1042`), created by **Leg C-bis's create-on-first-send**, which C-bis builds for peers anyway. ⇒ ***E1 needs a Tauri command wrapping `self_open` that exists for no other purpose; E2 needs none.***

🛑 **CORRECTED 2026-08-06 — CLAIR'S LEG 0-bis **F-A**. ANNOTATED, NOT REPAIRED (`D-131`). *"FOUND BY LEG B'S NAME SCAN" CONTRADICTS `OQ8-K3` IN THE SAME DOCUMENT.*** Leg B says the scan keys on **the FIELD, not the name**; this line says self is found by **the NAME**. 🔑 **The name scan is `s.role == "owner" && s.name == SELF_THREAD_LABEL` (`ops.rs:1042`) — keyed on the SAME free-form user-writable field `create_space` writes at `ops.rs:662`. That is EXACTLY the unsoundness K1 was refused for**, and E2 puts it on the click path. ⚠️ **A runbook cannot be written from two contradictory lookup instructions**, and `create_dm_space` **pushes unconditionally with no dedup** (`ops.rs:970`) ⇒ a failed lookup **mints a second self thread**.

🔒 **RESOLVED — CHAT'S, UNDER `D-123` + `D-143`, NOT ROUTED TO JOE.** *The cheap option (keep a name scan for self only) is unsound; `D-143` decides it and the user-visible surface is zero, so routing it would repeat the OQ6 error this arc already paid for.* ⇒ **the SELF lookup keys on `counterpart` like every peer**, and **`OQ8-K3`'s backfill gains a self case**: `name == SELF_THREAD_LABEL ⇒ counterpart = identity_id`. ✅ **Feasible — `load_or_default_state` already receives `identity_id` (`ops.rs:59-63`).** 📌 **Reachability measured, and it is narrower than Clair stated:** Joe's live state holds **no self thread**, and any self thread created after K3 gets `counterpart` written at creation (`ops.rs:970` is where K3 adds it) ⇒ **the backfill self-case covers only a CLI-minted pre-K3 thread. It is written anyway, because "nobody has one today" is not a property the code can rely on.**
2. **OFFLINE, E1 FAILS AND E2 DOES NOT.** `self_open`'s create arm calls `create_dm_space`, which signs three events and **sends them over a connection with send-confirm** (`ops.rs:907-952`). ⇒ **the first click on your own row errors offline**, while a peer click opens a draft. 📌 *And the self row is always FIRST — the row most likely to be clicked while exploring.*
3. **ONE RULE FOR EVERY ROW IN THE PANEL** ⇒ **Clair's F3 is CLOSED rather than documented.**
4. `self_open` **stays untouched and still tested via the CLI.** It simply is not the UI's path.

✅ **`D-021` WAS CHECKED BEFORE RECOMMENDING AND DOES NOT DECIDE THIS.** Its text reads *"a local-only synthetic Identity … never registered on any Node … Events are never broadcast"* — **but its own closing sentence anticipates what shipped:** *"In Phase 2, a 'Saved Messages' Space MAY be implemented as a proper DM Space where both sides of the DM are the user's own keypair."* ⇒ M11 built the Phase-2 form; **not a divergence.** 📌 *`D-021`'s real requirement — reachable from any client, not device-local — is satisfied by E2 the moment the thread exists.* **Corpus: `DECISIONS.md`, `JOURNAL.md`, `JOURNAL_ARCHIVE.md`, `docs/ROADMAP.md` (`D-139`).**

🔓 **THE ONE THING THAT WOULD FLIP THIS BACK TO E1, AND NOBODY HAS ASKED FOR IT:** wanting the self thread to exist on the node **before** you first write in it, so a second device sees it waiting. **That is a real requirement; it is simply not one anyone has stated.** *If Joe states it, E1 returns with a reason attached.*

⚠️ **THIS IS CHAT'S SECOND POSITION ON OQ6.** The first recommended **E1** on *"it is the shipped behaviour, zero cost"* — **true of the OP, false of the MILESTONE.** ***A cost priced in isolation and never checked against what the other legs already build — the same shape as OQ1's three passes.***

### 🔓 OQ6 — 🆕 does the SELF row create eagerly, or lazily like a peer? (Clair **F3**)

**E1 — eager, as `self_open` already behaves.** ① You always have a self thread; the first click signs 3 events and reaches the network. Skype's shape, and `L-3` cites Skype. ② **Zero** — it is the shipped behaviour.
**E2 — lazy, matching `L-4`.** ① One rule for every row in the panel: nothing is signed until you send. ② A draft arm for self, and `self_open`'s create-if-absent becomes create-on-send — a Rust change to a tested op.

📌 **Chat's recommendation: E1.** *`L-3` names Skype and Skype's self-thread always exists; and self-create writes only to your own node, so the asymmetry costs no one else anything.* ⚠️ **But the asymmetry must be WRITTEN DOWN either way** — two rows in one panel with different irreversibility is exactly the un-walked assumption this arc keeps paying for.

### 🔒 OQ7 — **CLOSED 2026-08-06: W4. PROVENANCE DELEGATED** (*"w4 (by your recomm)"*, `D-141`).

🛑 **AND THE R1/R2 FRAMING BELOW IS SUPERSEDED, NOT ANSWERED — IT ASKED ABOUT R7 WHEN THE DRAFT EMPTIES FOUR REGIONS.** Measured region by region with `effectiveSpaceId` and `effectiveRoomId` both `null`:

| region | during a draft, TODAY |
|---|---|
| **R4** room header | 🛑 **placeholder text — no widget exists at all** |
| **R5** stream | *"Select a room to see its messages."* (`stream-panel.svelte:56`) |
| **R6** composer | *"Select a room to send a message."*, send disabled (`composer-panel.svelte:64`) |
| **R7** members | **self only, no message** — state ①, and `NOTE['no-scope']` is `null` **deliberately** |
| **R8** inspector | ✅ **the clicked member's card** — `selection.clear` is called **NOWHERE in the codebase**, so the bus survives navigation |

🔑 ***FOUR REGIONS SAY "NOTHING IS SELECTED" AT THE MOMENT THE USER CLICKED A PERSON TO TALK TO. R7 EMPTYING WAS THE LEAST OF IT.***

🔒 **W4 — THE DRAFT RENDERS WHERE IT MUST, AND R7 STAYS THIN.** R5 gets the empty-thread copy · R6 live per **OQ2-S2** · **R7 shows self only, and that is CORRECT because `L-7`'s card in R8 carries the counterpart.** *"Who am I talking to"* is answered — **it moved to the region `L-7` just gave it to.**

📌 **W4 costs no new R7 work and adds no third consumer of the draft store** (W1 would have made the `D-067` second authority feed R5, R6 **and** R7). ⚠️ **W2 was listed and refused: it is OQ2-S3 wearing a different name and reverses `L-4`.**

⚠️ **W4 LEAVES OQ1's FAILURE-PATH GUARD ON LEG C's LIST.** R2 would have cleared it for free by unmounting the rows; **W4 keeps R7 mounted with a self row, so the guard survives.** *Stated because the two questions were settled in opposite directions and the interaction is easy to lose.*

🛑 **AND W4 CARRIES A REQUIREMENT NO LEG OWNED — THE SECOND INSTANCE OF CLAIR'S F2, FROM THE SAME LOCK.** `L-4` was locked on Discord's *"This is the beginning of your direct message history with …"*. **That is R5's empty-thread copy.** `stream-panel` has **one** no-room string and it is the wrong one. ⇒ **Leg C-bis gains R5's draft copy.** ***A lock's own evidence named a screen the plan did not build — again.***

### 🔓 OQ7 — 🆕 what does R7 show during a DRAFT? (Clair **F4** — `L-8` and §4.2 currently contradict)

**R1 — the draft feeds R7 a pseudo-scope** so the panel shows the two of you. ① `L-8`'s promise holds uniformly; the panel never blinks empty. ② R7 gains a second scope source — **more unscoped work than `L-8` implies**, and a second scope authority (the `D-067` shape, again).
**R2 — `L-8` is narrowed in writing: existing DMs re-scope, drafts show self-only.** ① Clicking a never-contacted person empties the members panel until you send. **Visibly odd, and it is the first thing a new user does.** ② Zero.

📌 **Chat's recommendation: R1, but its cost is real and §4.2 understated it.** *An empty members panel at the exact moment you started a conversation is the worst instance of `L-8`'s cost, not a corner case.* 🔒 **This is appearance and it is Joe's.**

### 🔒 OQ8 — **CLOSED 2026-08-06: K3, IN LEG B. UTTERED BY JOE:** *"ok k3 then. we must not be afraid of proper solutions, even if they are slightly heavier."*

✅ **THE PRINCIPLE IN THAT SENTENCE IS NOW `D-143`, MINTED 2026-08-06 (J-683)** — *when the cheap option is unsound, the proper one is taken even if it is heavier; **the trigger is unsoundness, not effort***. 🔑 **OQ8 IS ITS FIRST NAMED APPLICATION, AND OQ9 IS ITS FIRST NAMED NON-APPLICATION** — the two rulings drew both edges of the rule one hour apart, which is why the entry could be written with a boundary instead of a slogan.

🛑 **K1 AND K2 ARE BOTH REFUSED, AND THE MEASUREMENTS THAT KILLED THEM ARRIVED AFTER v1.2's RECOMMENDATION.**

**① K1 IS UNSOUND AGAINST USER INPUT.** `create_space` writes **`name: args.name.clone()`** — a free-form user string, max 128 chars (`ops.rs:660`) — into the **same field** the scan keys on: `.find(|s| s.role == "owner" && s.name == SELF_THREAD_LABEL)` where the label is the literal `"self"` (`:765`, `:1042`). ⇒ ***A Space you create and name `self` is indistinguishable from your self thread.*** 🔑 **Latent today because `self_open` has no UI path — and `OQ6-E2` PUTS THAT SCAN ON THE CLICK PATH.** *The two dispositions were taken an hour apart and the second made the first unsafe.*

**② K1 WORKS IN ONE DIRECTION ONLY.** Leg B asks *"is there a DM with X?"* — **build and compare, no parsing.** Leg E's DM home asks *"this DM Space, who is it with?"* — **`name.strip_prefix("DM with ")`, parsing a display string a user can write.** ✅ *Measured: **nothing in the codebase parses `KnownSpace.name`**; the DM home would be the first.*

**③ K2 ALONE MANUFACTURES THE DUPLICATE-DM CASE.** A new field needs `#[serde(default)]` or existing state files fail to load — and **Joe's `xgen-client_state.json` holds a real DM today**. With `counterpart: None` the scan finds nothing ⇒ ***clicking DAVE creates a SECOND DM Space***, the exact case §4c-ii calls pre-existing, here caused by our own change.

🔒 **K3 — `KnownSpace` GAINS `counterpart: Option<String>` + `#[serde(default)]`, PLUS A ONE-TIME BACKFILL AT `load_or_default_state` THAT PARSES THE LEGACY NAME AND WRITES THE FIELD.** ⇒ **the parse exists EXACTLY ONCE, in a migration — never in a lookup and never in a render path.** After one run **the name is free**, so OQ3's DM home may rename DM Spaces to anything, including `D-126`'s word form, and nothing breaks.

📌 **COST:** one field · two write sites (`ops.rs:660` normal, `:970` DM) · the TS mirror · ~5 lines of backfill. **Cargo floor. IN LEG B, pulled forward from Leg E.**

🔒 **AMENDED 2026-08-06 — CLAIR'S LEG 0-bis **F-A**: THE BACKFILL GAINS A SELF CASE, AND THE SELF LOOKUP MOVES ONTO THE FIELD WITH EVERY PEER.** The peer arm parses `name.strip_prefix("DM with ")`; **the self thread's stored name is the bare literal `"self"` (`ops.rs:965-966`), which has no prefix and would yield `counterpart = None`** ⇒ the field scan finds nothing ⇒ `create_dm_space` pushes unconditionally (`ops.rs:970`) ⇒ **a second self thread.** ⇒ **backfill rule: `name == SELF_THREAD_LABEL ⇒ counterpart = identity_id`**, feasible because `load_or_default_state` already receives `identity_id` (`ops.rs:59-63`). ⇒ ***after the migration there is ONE lookup rule for every row in the panel, and the name scan at `ops.rs:1042` is off the UI path entirely.*** 📌 **`self_open` itself stays untouched and CLI-tested (`OQ6-E2` point 4).**

⚠️ **CHAT'S v1.2 RECOMMENDATION WAS *"K1 for Leg B, K2 folded into Leg E"* AND IT IS SUPERSEDED, NOT AMENDED.** Clair's **F1** was right that the scan is buildable today; Chat was right that the relief was borrowed; **the conclusion drawn from both was wrong.** 🔑 ***THIRD RECOMMENDATION-INVERSION OF THIS ARC — SEE §8 ITEM 7 FOR THE MECHANISM THEY SHARE.***

### 🔓 OQ8 — 🆕 `KnownSpace.name` is both the label and the counterpart key (Chat, from Clair's **F1**)

The find-existing-DM scan works **because** the counterpart XGID sits inside the display string (**G2a/G2b**). 🛑 **That is one token carrying two meanings, and OQ3 wants to change that exact field.**

**K1 — use the name as the key; accept the coupling, record it.** ① None today. ② Zero now; **a silent breakage the day the label changes** — a DM home, a humane label, `D-126`'s word form.
**K2 — `KnownSpace` gains a real `counterpart` (and/or `is_dm`) field, and the label becomes free.** ① None directly — it unblocks OQ3-A3 renaming DMs safely. ② **Rust: `xgen-common/src/state.rs` + the writer + the TS mirror. Pulls part of Leg E forward into Leg B** — the very move F1 just showed was not needed for the scan, but IS needed for OQ3.

📌 **Chat's recommendation: K1 for Leg B, K2 folded into Leg E** — with the coupling written at the call site so the scan cannot be broken silently. *Doing K2 now would trade a measured no-op for real Rust before anything works end to end.* ⚠️ **If Joe wants DM Spaces renamed as part of OQ3, K2 stops being optional.**

### 🔒 OQ9 — **CLOSED 2026-08-06: OPTION C — THE CONST, AND NO CONTROL YET. DESTINATION NAMED, NOT BUILT.**

✅ **THE CLASS DISTINCTION BELOW IS NOW `D-144`, MINTED 2026-08-06 (J-683)** — *owner content and client state copy are different classes; the second is authored by the client and by nobody else.* 🔑 **IT NO LONGER LIVES ONLY IN THIS PHASE-0**, which was the whole objection to leaving it here: a future milestone wiring `topic` would never have looked in a members-panel interaction document. 📌 **AND THE GROUNDING SHARPENED ON THE WAY OUT:** the receptacle already ships — `entity-item.svelte:44` and `entity-panel.svelte:37` both comment `secondary?: string; // topic / last-message / handle`, and `spaces-panel.svelte:38` ships `secondary` **UNFED citing `D-065`** ⇒ ***the day `topic` is wired, nothing has to be built to widen it.***

🔑 **THE QUESTION DISSOLVED WHEN THE ASK WAS RE-READ.** Joe's request was **hygiene**: *"no hardcode in the code … into a constant with a note."* ⚠️ ***"NOT HARDCODED" IS NOT "USER-EDITABLE", AND CHAT TURNED THE FIRST INTO THE SECOND*** — then spent three turns siting a feature **nobody had asked for**, neither Joe nor any user.

🔒 **RULED, AND JOE CONFIRMED (*"ok, i understand (c). it is not so important, but we can access it in the future by a need"*):**

1. **The string is CLIENT-OWNED.** 🔒 **Joe's call, and it is correct** — *"quasi private message, just for user."* It is the client describing **its own state** to the one person reading it. ⇒ **not the Space owner's, not the node operator's.**
2. **Leg C-bis ships the const** with the note already ruled: the existing `PROVISIONAL → M-RP-SKIN` line, **plus** *this string is a candidate for runtime configuration; the owner is undecided and is not this milestone's.*
3. **No control is built.** 📌 **The destination is NAMED so the next reader inherits an answer instead of an open question** (`D-140`'s whole point): **Settings › a new app-level copy/language section — NOT a widget settings pane.** **Trigger: somebody actually wanting to edit copy.**

🛑 **TWO SITINGS WERE PROPOSED AND BOTH REFUSED, WITH REASONS, SO THEY ARE NOT RE-PROPOSED LATER:**

- **The SPACE / node setting** (Joe's first instinct — *"when owner sets new space, he can customize some default text labels"*). 🔑 **Refused on a mechanical fact:** the empty-thread line renders **on a DRAFT** — `L-4`: no `space_id`, nothing signed — and once you send, **the thread is no longer empty**. ⇒ ***the line is only ever visible at a moment when NO SPACE EXISTS.*** A Space setting cannot supply it in any design. ⚠️ **And the wider idea needs a line drawn** — see the class distinction below.
- **The SELF widget** (Joe's second — *"the closest house is self widget"*, later withdrawn by him: *"the self widget has no sense"*). **`self-panel` is a status readout** (identity + connection light), not a preferences home — and 🔒 **J-591's S2 already moved even self-record editing OUT of it**, to *"Settings › Account, in the single place"*.
- **The MESSAGES widget pane** — mechanically correct under the shipped rule (`registry.ts:99`; the widget that renders the string owns its pane, two live instances). ⚠️ **Refused anyway:** it mints the precedent *"copy lives in whichever widget rendered it"*, which at **14 existing copy sites across ~6 widgets** means six panes to hunt through for one sentence. ***A cheap siting that makes the proper one harder.***

🔑 **AND THE CLASS DISTINCTION THIS PRODUCED IS THE VALUABLE OUTPUT — IT EXISTS NOWHERE ELSE IN THE RECORD.**

| class | example | who authors it |
|---|---|---|
| **owner content** | Space name · room name · `topic` · a welcome message | ✅ **the owner** — describing their own place. 📌 **The channel is BUILT AND UNWIRED:** `SpaceState`/`RoomState` carry `name` + `topic: Option<String>` (`xgen-core/src/space/state.rs:116-117`, `:188-189`), event-sourced, and **`xgen-client` contains ZERO occurrences of `topic`** |
| **client state copy** | *"Select a room"* · *"No messages in this room yet"* · *"I cannot reach the others"* | 🛑 **the client** — describing **itself** |

🛑 **IF A SPACE OWNER COULD REWRITE THE SECOND CLASS THEY COULD MAKE A MEMBER'S CLIENT LIE ABOUT ITS OWN STATE** — `members-panel`'s *"I cannot reach the others"* means the fill failed; an owner rewriting it to something reassuring has made the client misreport what it knows. ***That is precisely what `D-065` forbids, committed by a third party through a supported feature.*** ⚠️ **Filed here because a future milestone wiring `topic` could widen into copy-override without anyone deciding to.**

📌 **WHY C AND NOT "BUILD IT PROPERLY NOW", GIVEN `OQ8`'s SENTENCE.** Joe's K3 principle is *do not fear the proper solution when the cheap one is **unsound***. **K1 was unsound** — a user-writable lookup key. **The const is not unsound**; it is complete for what it does. ⇒ ***"proper" here means NOT building a settings section for a want nobody has stated, and not minting the precedent that makes the real version harder.***

### 🔓 OQ9 — 🆕 WHO OWNS USER-FACING COPY AT RUNTIME? (Joe, 2026-08-06: *"can we make this line text potentially customisable … no hardcode in the code"*)

✅ **THE EXTRACTION ITSELF IS SETTLED AND FREE — IT IS ALREADY THE HOUSE PATTERN, MEASURED AT 14 SITES.** Module-level `UPPER_SNAKE` const + a note naming who owns the final wording: `stream-panel.svelte:52-56` (`SESSION_START` · `SESSION_START_DROPPED` · `NO_MESSAGES` · `SELECT_ROOM`) · `composer-panel.svelte:63-64` · `members-panel.svelte:65` (`NOTE`, a Record over the five states) · `rooms-panel.svelte:54`. Every one reads **`// Functional copy, PROVISIONAL (appearance and final phrasing → M-RP-SKIN)`**. ⇒ **R5's draft copy is written the same way. Doing it otherwise would be the odd one out.**

⚠️ **ANNOTATED 2026-08-06, NOT REPAIRED (`D-131`) — *"EVERY ONE READS"* IS FALSE, AND THE PATTERN IT DESCRIBES IS REAL.** Re-measured across `ui/**` excluding the sampler: **exactly ONE site carries that comment verbatim** — `composer-panel.svelte:60`. The others carry **variants** of it: `stream-panel.svelte:45` (*"Functional copy (Ms Design's WORDING is PROVISIONAL …)"*), `send-status.svelte:47` (*"Copy is FUNCTIONAL and PROVISIONAL …"*), `substitutions-editor.svelte:127`/`:159` (*"WORDING IS PROVISIONAL → M-RP-SKIN"*). 🔑 **The claim was a census stated as a quotation** — the house pattern holds, its *wording* does not, and the ruling below is unaffected because it prescribes what R5's note must SAY, not which existing string it must match.

🔑 **WHAT IS *NOT* SETTLED IS THE OWNER, AND JOE'S PHRASE NAMED ONE.** *"Accessible by the node's setting"* is a different claim from what those 14 notes make:

| owner | what it means |
|---|---|
| **M-RP-SKIN** | phrasing finalised **once**, by Joe. Today's precedent, not runtime |
| **M-RP-SETTINGS** | **the user** customises their own client's copy. No trust question |
| **the node** | 🛑 **the operator supplies the words every client on that node renders** |

⚠️ **THE THIRD TOUCHES THE TRUST MODEL AND MUST NOT BE WRITTEN INTO A COMMENT AS IF SETTLED.** XGen's premise is a **content-blind node** (`D-093` clause 1); a node that sets UI copy can put words in the client's mouth. 📌 *For this string it is narrow — a DM is single-homed on the creator's node, so it is effectively your own — but a note establishes a direction, and direction is what later readers act on.*

🔑 **AND IT IS NOT A PER-STRING QUESTION.** If copy becomes configurable, **all 14 sites want the same treatment** — that is a **copy/localisation architecture**, not a comment. ***Writing "node-configurable" on one string mints an expectation the codebase cannot honour***, which is this project's named cost.

🔒 **RULED (Chat, `D-123` — what a comment may CLAIM is a records question): THE NOTE STATES WHAT IS TRUE, NOT WHAT IS PROMISED.** R5's draft copy ships as a const carrying **(a)** the existing `PROVISIONAL → M-RP-SKIN` line verbatim, and **(b)** one sentence: *this string is a candidate for runtime configuration; the owner — client settings vs node-supplied — is undecided and is not this milestone's.* ⇒ **Joe gets the extraction he asked for, at zero cost, with no promise nobody made.**

🔓 **THE OWNER ITSELF IS OPEN AND IS JOE'S — IT IS ARCHITECTURE.** 📌 *It is also bigger than this milestone: whoever opens it inherits 14 existing sites, `M-RP-SETTINGS` (design-locked at J-534) as the natural host, and the `D-093` question above.*

### 🔓 OQ5 — inherited and still open (not opened by this milestone)

- **A partial first send.** Create succeeds, message fails ⇒ the recipient holds an invitation with no message. *Defect, or a legitimate "someone started a conversation with you"?* (`M_RP_MEMBERS.md:336` — explicitly deferred to *"the interaction milestone's Phase-0"*, which is this one.)
- **Erased members are clickable.** LegF-DAVE renders in a roster and can be LMC'd. `create_dm_space` takes any XGID (G1), so a DM to an erased identity **would be created and would go nowhere**. ⚠️ *Not previously recorded anywhere; surfaced by `M-RP-TAIL8`'s live run.*
- **How does somebody on another node learn they have been invited?** (`AUDIT_MEMBERS_PANEL.md` §7 — a measurement nobody has taken.)

---

## §6 — PROPOSED LEGS. Order is argued, not assumed; each leg leaves the app usable.

| leg | what | floor | gated on |
|---|---|---|---|
| **0** | ✅ **CLOSED 2026-08-06** — Clair's adversarial read. 5 plan-moving findings, 3 wording, all re-measured by Chat | none | — |
| **A** | 🛑 **`D-131` annotations first, before any code.** `members-panel.svelte:11-14` (J-675's filed overstatement) · `selection.svelte.ts:2` and J-591's *"`entity-context-menu` READS the bus"* — **it takes a prop and does not import `selection`** (G7) · 🆕 **`M_RP_MEMBERS.md:309` and `:406`** — *18 commands* (it is **19**), `self_open :1002` (it is **`:1019`**), `create_dm_space :793` (it is **`:806`**) | none | nothing |
| **B** | **The command surface** — a Tauri command wrapping `create_dm_space`; the draft object per OQ2. 🔒 **OQ8-K3: `KnownSpace` gains `counterpart: Option<String>` + `#[serde(default)]` + a one-time backfill at `load_or_default_state`** — the scan keys on the FIELD, not the name. 🆕 🔒 **F-A: the backfill's SELF case (`name == SELF_THREAD_LABEL ⇒ counterpart = identity_id`), and the self lookup keys on `counterpart` like every peer** — the name scan at `ops.rs:1042` leaves the UI path. 📌 **OQ6-E2: NO `self_open` command** | **cargo** + svelte-check | OQ2 ✅, OQ6 ✅, OQ8 ✅ |
| **C** | **R7 acts** — `interactive`, `onActivate` → open-or-draft **and** `selection.set()`; R8 renders the member card. 🆕 🔒 **F-C / OQ1-G1 IS THIS LEG'S FIRST COMMIT, SEPARATE AND MEASURED ALONE:** `ui/core/.../entity-panel.svelte` gains a flag suppressing `selectAt`'s `selected` write, `interactive` keeps ARIA + click wiring. **3 consumers + 8 sampler cells; the runbook MEASURES whether catalogue 435 moves rather than predicting it.** 📌 **OQ6-E2: the self row takes the SAME path as any peer** · **W4: R7 stays THIN during a draft** | svelte-check **+ catalogue** | OQ1 ✅, OQ7 ✅, B |
| **C-bis** | 🆕 🛑 **FIRST SEND PROMOTES THE DRAFT** — `composer-panel.submit()` + `echo-state` gain the draft arm: detect draft → `create_dm_space` → `{space_id, room_id}` → `echo.send` → promote in place. **Added at v1.2 from Clair's F2.** 🆕 **at v1.4: R5's DRAFT COPY** as a const per OQ9-C. 🆕 🔒 **at v1.9 from F-B: `stream-panel.svelte` gains the DRAFT BRANCH that renders it** — today R5 branches on `effectiveRoomId == null` alone and imports no draft store, so the const had no display path. 🆕 🔒 **from F-E: `room-latch.svelte.ts`'s two-armed `canSend`**, which belonged to no leg | svelte-check | C, OQ9 ✅ |
| **D** | **RMC → the menu, no selection** — the first `oncontextmenu` in the codebase (G8); `entity-context-menu` mounted with the row's descriptor as a prop | svelte-check | C |
| **E** | **DM home + `is_dm`** per OQ3, **including G13's stale-flag question**. 📌 **OQ8-K2 no longer lives here — K3 took the field into Leg B**, so the DM home may rename DM Spaces freely. 🆕 🔒 **F-D: THE FILTER IS RENDER-ONLY** — `spaces-panel`'s `$derived`, never the store; `resolveLatched` and `canSend` both read `spacesState.spaces` and a store-side filter makes every DM **unsendable** | **cargo** + svelte-check | OQ3, D |
| **F** | Records + close (`D-074`) | — | E |

🔑 **WHY B BEFORE C:** §4.3. Under `L-7` the click must do both or neither, so the op has to be reachable before the row is clickable.
🔑 **WHY E STAYS LAST, AND IT IS NOW MEASURED RATHER THAN ASSUMED:** Clair's **F1** dissolved the doubt that `is_dm` had to move forward — the scan runs against `KnownSpace.name`, which ships today. **E is last because DMs need a home before they lose the tree**, not because of the scan.
📌 **A is free and unblocks nothing** — it is first because three records currently say the opposite of the code this milestone builds on.
🛑 **AND C-bis IS THE LEG THIS DOCUMENT DID NOT HAVE.** §4.2 diagnosed the collision correctly and then assigned only its **enable gate** (OQ2-S2) to a leg. *The document that quotes the audit's "which leg builds this?" rule committed the error the rule names, and Clair found it by trying to run the send path.*

🔒 **AND ONE LEG SITS BEFORE ALL OF THEM, ADOPTED 2026-08-06: LEG 0 — CLAIR'S ADVERSARIAL READ OF THIS DOCUMENT.** No authority to code. Her brief is §8 plus the two places it does not yet suspect itself: **§6's leg ORDER** (see §8 item 2 — the find-existing-DM scan may pull `is_dm` forward into Leg B) and **§5's four DELEGATED dispositions**, which were adopted rather than examined and are the class `AUDIT_MEMBERS_PANEL.md` §8 warns about. ⚠️ ***Clair caught four defects in `RUNBOOK_TAIL8.md` by trying to RUN it; Chat's own re-reads passed every time. This document has had none.***

---

## §7 — NOT TOUCHED

The wire · any protocol event · `xgen-core` · `xgen-node` · the DM model itself (`L-4`, `L-6`, bilateral replication **H4** — locked at J-591 and **not** built here) · `entity-item.svelte` · the skin's unresolved rules (`N-168` is filed, not fixed) · `M-RP-OWN-ROW-NAME` · surface **A4**.

⚠️ **H4 (bilateral DM replication) IS LOCKED AND UNBUILT, AND THIS MILESTONE DOES NOT ADVANCE IT.** Its blocking measurement is still owed and is Chat's: **can a `space_id` derive from the identity pair rather than the signed root event?** *That answer decides whether H4 is a milestone or a rewrite (`M_RP_MEMBERS.md` §4c-ii), and it is not a prerequisite here — a DM created today is single-homed and stays so.*

---

## §8 — WHERE THIS DOCUMENT IS MOST LIKELY WRONG

1. ✅ **PARTLY DISSOLVED 2026-08-06, ANNOTATED NOT DELETED (`D-131`) — AND THE PART THAT DISSOLVED IS THE PART THAT NAMED THE LAYER.** §4.1's re-derivation settles OQ1 from **code semantics + `L-8`**, not from paint: the early highlight carries the value that is about to be correct, so the residue is an **error path**, not a transition. 📌 **What survives is narrower: the round-trip DURATION is still unmeasured**, and it cannot be measured until Leg B exists. *Superseded text follows.* — 🛑 **THE ONE-FRAME HIGHLIGHT IN §4.1 IS REASONED, NOT MEASURED.** No probe has been run. If the stale highlight persists rather than flashing, **OQ1-P1 gets more expensive** and P2 gets more attractive. **This is the first thing Leg C's runbook should measure, and it must be measured at the PAINT layer** (`N-168`, `D-140`) — a store read will not decide it.
2. ✅ **DISSOLVED AT v1.2 BY CLAIR'S F1, ANNOTATED NOT DELETED (`D-131`) — AND THE DOUBT WAS WRONG IN THE EASY DIRECTION.** The scan **is** buildable: the counterpart rides inside `KnownSpace.name` (**G2a**) and `self_open` already keys on that field (**G2b**). §6's order stands; `is_dm` stays in Leg E. 📌 **But the relief is borrowed — see OQ8**, and Clair's own bounded caveat rides with it: per `AUDIT_MEMBERS_PANEL.md` §4.8 the Spaces tree records **only your own actions**, so the name-scan finds only DMs **you** created — a peer-created DM is not in local `KnownSpace` at all, and clicking that peer makes a second Space. *That is the pre-existing duplicate-DM case (§4c-ii, host-by-race), not a defect in the scan.* *Superseded text follows.* — **The find-existing-DM scan is described and not designed.** `M_RP_MEMBERS.md:329` says *"scan known Spaces for an existing DM with that identity"* — but **G11: `KnownSpace` has no `is_dm` and no counterpart field.** Today that scan has nothing to scan on. ⇒ **OQ3's `is_dm` plumbing may be a prerequisite of Leg B, not Leg E.** *If so, this §6 order is wrong and E's Rust half moves forward.*
3. **`self_open`'s idempotence is tested at the op layer, not through a Tauri command that does not yet exist.** `created: bool` (G2) is the right signal; nothing has exercised it from the UI.
4. **Leg D assumes `entity-context-menu` works when mounted.** It is **COMPLETE and never instantiated** (J-675) — verified by its own sampler gates at M-RP5.3, never in the client against a real roster.
5. ✅ **DISCHARGED 2026-08-06 — Leg 0 ran and this document was not executable as written.** *Superseded text follows.* — **This Phase-0 has not been read by anyone outside its author.** *Five defects in `RUNBOOK_TAIL8.md` were caught by Clair trying to execute it or by Joe looking at a screen; Chat's own re-reads passed every time.* ⚠️ **An adversarial read by Clair before Joe locks anything is worth its cost, and the `M-RP-TAIL8` arc is the evidence.** 🔑 **OUTCOME: five plan-moving findings. §8 confirmed ONE of its own doubts, DISSOLVED another, and MISSED THREE — including the milestone's central behaviour having no leg.** ***The cost was worth paying and the document should not be trusted to have found its own remaining errors either.***
6. **The seat line on `L-9` is Chat's reading, and it is cheap to be wrong about.** *"RMC without member's selection"* is taken to mean no bus write, no navigation, no R8 change. **If Joe meant only "no highlight", Leg D is different.** Stated because a one-word ruling is easy to over-extend — the `D-141` failure mode.
7. 🛑 **🆕 CHAT INVERTED ITS OWN RECOMMENDATION THREE TIMES IN THIS ARC, AND THE MECHANISM WAS IDENTICAL EVERY TIME.** **OQ1** — P1's cost stated three ways across three passes (§4.1, Clair's F5, the §4.1 re-derivation) · **OQ6** — E1 recommended on *"it is the shipped behaviour, zero cost"*, **true of the OP and false of the MILESTONE** · **OQ8** — *"K1 now, K2 later"* superseded by K3 once `OQ6-E2` put the name-scan on the click path. 🔑 ***EACH TIME A COST WAS PRICED AGAINST ONE LEG AND NEVER RE-CHECKED AGAINST THE OTHERS — AND EACH TIME THE FIRST PRICING WAS CHEAPER THAN THE TRUTH.*** ⚠️ **FORWARD: when a disposition lands, re-read every OTHER open disposition against it before the next one is taken.** *`OQ6` and `OQ8` were settled an hour apart and the second was made unsafe by the first; nothing in this document's process would have caught that.*