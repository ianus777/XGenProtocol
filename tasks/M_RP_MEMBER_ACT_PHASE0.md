# M-RP-MEMBER-ACT — the members panel acts: LMC opens the DM, RMC opens the menu — Phase-0
> **Status**: ACTIVE  
> Version: 1.2  
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

## §5 — OPEN, AND JOE'S. Each carries `D-121` lenses: ① user-visible impact per option, then ② resource cost.

---

### 🔒 DISPOSITION 2026-08-06 — **FOUR ADOPTED, ONE STILL OPEN. PROVENANCE: DELEGATED.**

⚠️ **JOE: *"let's go by your recomms"*. 🔑 THAT IS ADOPTION, NOT EXAMINATION, AND IT IS RECORDED AS SUCH SO A LATER REVISIT READS IT CORRECTLY** (`D-141`; the `D-127` shape). *The identical phrase carried `M-RP-TAIL8`'s mechanism **M3**, and that runbook marks it the same way. `AUDIT_MEMBERS_PANEL.md` §8 names the cost of the pattern: three decisions of record that nobody judged.*

| # | adopted | seat it really belongs to |
|---|---|---|
| **OQ1** | **P1** — `selected` stays the DM counterpart | appearance-adjacent ⇒ **Joe's**, taken on recommendation |
| **OQ2** | **S2** — the draft sits beside the latch; `canSend` becomes two-armed | 🔑 **ARCHITECTURE — Joe's reserved area** (`D-123`), taken on recommendation |
| **OQ3** | **A3, sequenced last** — `is_dm` plumbed + filter, but **a DM home ships first** | appearance + architecture ⇒ **Joe's**, taken on recommendation |
| **OQ4** | **NO** — the R8 card does not ship before the DM opens | sequencing ⇒ Chat's, and it was Chat's to take |

🔒 **AND THE META-RECOMMENDATION IS ADOPTED WITH THEM: CLAIR READS THIS DOCUMENT ADVERSARIALLY BEFORE ANY RUNBOOK EXISTS.** ⇒ **these four dispositions are ADOPTED but PROVISIONAL** — Clair's read is the next gate, and §8's own record (five defects in `RUNBOOK_TAIL8.md`, every one caught from outside the text) is why.

🔓 **OQ5 IS NOT ANSWERED BY THIS AND MUST NOT BE READ AS ANSWERED.** **Chat made no recommendation on any of its three items**, so *"go by your recomms"* cannot reach them. Two need Joe:
- **the partial first send** — `M_RP_MEMBERS.md:336` already routes it here and calls it Joe's;
- **erased members are clickable** — new, unrecorded anywhere before 2026-08-06.

The third (**cross-node invite discovery**) is a **measurement nobody has taken**, not a decision — Chat's to run, and it is not a prerequisite for Legs A–D.

---

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

### 🔓 OQ4 — may the R8 card ship before the DM opens?

① **Yes:** the card is useful on its own and lands weeks earlier; the cost is a click that selects but does not navigate, for one milestone's duration. **No:** nothing half-wired ever reaches the user.
② Yes saves nothing in total; it only re-orders. **It buys feedback on the card's content early**, which matters because *"just data that we have now"* has never been seen on a member.

📌 **Chat's recommendation: NO** — §4.3's precedent is this project's own and it was expensive to learn. *Offered because it is a real choice, not to be talked out of.*

### 🔓 OQ6 — 🆕 does the SELF row create eagerly, or lazily like a peer? (Clair **F3**)

**E1 — eager, as `self_open` already behaves.** ① You always have a self thread; the first click signs 3 events and reaches the network. Skype's shape, and `L-3` cites Skype. ② **Zero** — it is the shipped behaviour.
**E2 — lazy, matching `L-4`.** ① One rule for every row in the panel: nothing is signed until you send. ② A draft arm for self, and `self_open`'s create-if-absent becomes create-on-send — a Rust change to a tested op.

📌 **Chat's recommendation: E1.** *`L-3` names Skype and Skype's self-thread always exists; and self-create writes only to your own node, so the asymmetry costs no one else anything.* ⚠️ **But the asymmetry must be WRITTEN DOWN either way** — two rows in one panel with different irreversibility is exactly the un-walked assumption this arc keeps paying for.

### 🔓 OQ7 — 🆕 what does R7 show during a DRAFT? (Clair **F4** — `L-8` and §4.2 currently contradict)

**R1 — the draft feeds R7 a pseudo-scope** so the panel shows the two of you. ① `L-8`'s promise holds uniformly; the panel never blinks empty. ② R7 gains a second scope source — **more unscoped work than `L-8` implies**, and a second scope authority (the `D-067` shape, again).
**R2 — `L-8` is narrowed in writing: existing DMs re-scope, drafts show self-only.** ① Clicking a never-contacted person empties the members panel until you send. **Visibly odd, and it is the first thing a new user does.** ② Zero.

📌 **Chat's recommendation: R1, but its cost is real and §4.2 understated it.** *An empty members panel at the exact moment you started a conversation is the worst instance of `L-8`'s cost, not a corner case.* 🔒 **This is appearance and it is Joe's.**

### 🔓 OQ8 — 🆕 `KnownSpace.name` is both the label and the counterpart key (Chat, from Clair's **F1**)

The find-existing-DM scan works **because** the counterpart XGID sits inside the display string (**G2a/G2b**). 🛑 **That is one token carrying two meanings, and OQ3 wants to change that exact field.**

**K1 — use the name as the key; accept the coupling, record it.** ① None today. ② Zero now; **a silent breakage the day the label changes** — a DM home, a humane label, `D-126`'s word form.
**K2 — `KnownSpace` gains a real `counterpart` (and/or `is_dm`) field, and the label becomes free.** ① None directly — it unblocks OQ3-A3 renaming DMs safely. ② **Rust: `xgen-common/src/state.rs` + the writer + the TS mirror. Pulls part of Leg E forward into Leg B** — the very move F1 just showed was not needed for the scan, but IS needed for OQ3.

📌 **Chat's recommendation: K1 for Leg B, K2 folded into Leg E** — with the coupling written at the call site so the scan cannot be broken silently. *Doing K2 now would trade a measured no-op for real Rust before anything works end to end.* ⚠️ **If Joe wants DM Spaces renamed as part of OQ3, K2 stops being optional.**

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
| **B** | **The command surface** — Tauri commands wrapping `create_dm_space` + `self_open`; **find-existing-DM by `KnownSpace.name` (G2a/G2b, OQ8-K1)**; the draft object per OQ2 | **cargo** + svelte-check | OQ2, OQ6, OQ8 |
| **C** | **R7 acts** — `interactive`, `onActivate` → open-or-draft **and** `selection.set()`; R8 renders the member card; `L-3` self row → `self_open` | svelte-check | OQ1, OQ7, B |
| **C-bis** | 🆕 🛑 **FIRST SEND PROMOTES THE DRAFT** — `composer-panel.submit()` + `echo-state` gain the draft arm: detect draft → `create_dm_space` → `{space_id, room_id}` → `echo.send` → promote in place. **Added at v1.2 from Clair's F2 — `L-4`'s central behaviour previously had NO OWNER** | svelte-check | C |
| **D** | **RMC → the menu, no selection** — the first `oncontextmenu` in the codebase (G8); `entity-context-menu` mounted with the row's descriptor as a prop | svelte-check | C |
| **E** | **DM home + `is_dm`** per OQ3, **including G13's stale-flag question and OQ8-K2** | **cargo** + svelte-check | OQ3, D |
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

1. 🛑 **THE ONE-FRAME HIGHLIGHT IN §4.1 IS REASONED, NOT MEASURED.** No probe has been run. If the stale highlight persists rather than flashing, **OQ1-P1 gets more expensive** and P2 gets more attractive. **This is the first thing Leg C's runbook should measure, and it must be measured at the PAINT layer** (`N-168`, `D-140`) — a store read will not decide it.
2. ✅ **DISSOLVED AT v1.2 BY CLAIR'S F1, ANNOTATED NOT DELETED (`D-131`) — AND THE DOUBT WAS WRONG IN THE EASY DIRECTION.** The scan **is** buildable: the counterpart rides inside `KnownSpace.name` (**G2a**) and `self_open` already keys on that field (**G2b**). §6's order stands; `is_dm` stays in Leg E. 📌 **But the relief is borrowed — see OQ8**, and Clair's own bounded caveat rides with it: per `AUDIT_MEMBERS_PANEL.md` §4.8 the Spaces tree records **only your own actions**, so the name-scan finds only DMs **you** created — a peer-created DM is not in local `KnownSpace` at all, and clicking that peer makes a second Space. *That is the pre-existing duplicate-DM case (§4c-ii, host-by-race), not a defect in the scan.* *Superseded text follows.* — **The find-existing-DM scan is described and not designed.** `M_RP_MEMBERS.md:329` says *"scan known Spaces for an existing DM with that identity"* — but **G11: `KnownSpace` has no `is_dm` and no counterpart field.** Today that scan has nothing to scan on. ⇒ **OQ3's `is_dm` plumbing may be a prerequisite of Leg B, not Leg E.** *If so, this §6 order is wrong and E's Rust half moves forward.*
3. **`self_open`'s idempotence is tested at the op layer, not through a Tauri command that does not yet exist.** `created: bool` (G2) is the right signal; nothing has exercised it from the UI.
4. **Leg D assumes `entity-context-menu` works when mounted.** It is **COMPLETE and never instantiated** (J-675) — verified by its own sampler gates at M-RP5.3, never in the client against a real roster.
5. ✅ **DISCHARGED 2026-08-06 — Leg 0 ran and this document was not executable as written.** *Superseded text follows.* — **This Phase-0 has not been read by anyone outside its author.** *Five defects in `RUNBOOK_TAIL8.md` were caught by Clair trying to execute it or by Joe looking at a screen; Chat's own re-reads passed every time.* ⚠️ **An adversarial read by Clair before Joe locks anything is worth its cost, and the `M-RP-TAIL8` arc is the evidence.** 🔑 **OUTCOME: five plan-moving findings. §8 confirmed ONE of its own doubts, DISSOLVED another, and MISSED THREE — including the milestone's central behaviour having no leg.** ***The cost was worth paying and the document should not be trusted to have found its own remaining errors either.***
6. **The seat line on `L-9` is Chat's reading, and it is cheap to be wrong about.** *"RMC without member's selection"* is taken to mean no bus write, no navigation, no R8 change. **If Joe meant only "no highlight", Leg D is different.** Stated because a one-word ruling is easy to over-extend — the `D-141` failure mode.