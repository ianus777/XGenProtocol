# M-RP-MEMBER-ACT — the members panel acts: LMC opens the DM, RMC opens the menu — Phase-0
> **Status**: ACTIVE  
> Version: 1.1  
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
| **G2** | `self_open` takes **no id** — auto-resolves the session identity (M11-D5) — and returns `SelfThreadResult { space_id, room_id, created }`. **`created` is the idempotence signal** | `ops.rs:1002` |
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
② Cheapest. No change to the derivation. Needs `entity-panel`'s unconditional `selectAt` write neutralised for this consumer — one prop or one guard.

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

### 🔓 OQ5 — inherited and still open (not opened by this milestone)

- **A partial first send.** Create succeeds, message fails ⇒ the recipient holds an invitation with no message. *Defect, or a legitimate "someone started a conversation with you"?* (`M_RP_MEMBERS.md:336` — explicitly deferred to *"the interaction milestone's Phase-0"*, which is this one.)
- **Erased members are clickable.** LegF-DAVE renders in a roster and can be LMC'd. `create_dm_space` takes any XGID (G1), so a DM to an erased identity **would be created and would go nowhere**. ⚠️ *Not previously recorded anywhere; surfaced by `M-RP-TAIL8`'s live run.*
- **How does somebody on another node learn they have been invited?** (`AUDIT_MEMBERS_PANEL.md` §7 — a measurement nobody has taken.)

---

## §6 — PROPOSED LEGS. Order is argued, not assumed; each leg leaves the app usable.

| leg | what | floor | gated on |
|---|---|---|---|
| **A** | 🛑 **`D-131` annotations first, before any code.** `members-panel.svelte:11-14` (J-675's filed overstatement) · `selection.svelte.ts:2` and J-591's *"`entity-context-menu` READS the bus"* — **it takes a prop and does not import `selection`** (G7) | none | nothing |
| **B** | **The command surface** — Tauri commands wrapping `create_dm_space` + `self_open`; find-existing-DM-for-counterpart; the draft object per OQ2 | **cargo** + svelte-check | OQ2 |
| **C** | **R7 acts** — `interactive`, `onActivate` → open-or-draft **and** `selection.set()`; R8 renders the member card; `L-3` self row → `self_open` | svelte-check | OQ1, B |
| **D** | **RMC → the menu, no selection** — the first `oncontextmenu` in the codebase (G8); `entity-context-menu` mounted with the row's descriptor as a prop | svelte-check | C |
| **E** | **DM home + `is_dm`** per OQ3, **including G13's stale-flag question** | **cargo** + svelte-check | OQ3, D |
| **F** | Records + close (`D-074`) | — | E |

🔑 **WHY B BEFORE C:** §4.3. Under `L-7` the click must do both or neither, so the op has to be reachable before the row is clickable.
🔑 **WHY E LAST:** hiding DMs from the tree before they have a home makes existing DMs unreachable (OQ3-A3).
📌 **A is free and unblocks nothing** — it is first because two records currently say the opposite of the code this milestone builds on, and a runbook written against them would be wrong in the same way this arc has already been wrong five times.

🔒 **AND ONE LEG SITS BEFORE ALL OF THEM, ADOPTED 2026-08-06: LEG 0 — CLAIR'S ADVERSARIAL READ OF THIS DOCUMENT.** No authority to code. Her brief is §8 plus the two places it does not yet suspect itself: **§6's leg ORDER** (see §8 item 2 — the find-existing-DM scan may pull `is_dm` forward into Leg B) and **§5's four DELEGATED dispositions**, which were adopted rather than examined and are the class `AUDIT_MEMBERS_PANEL.md` §8 warns about. ⚠️ ***Clair caught four defects in `RUNBOOK_TAIL8.md` by trying to RUN it; Chat's own re-reads passed every time. This document has had none.***

---

## §7 — NOT TOUCHED

The wire · any protocol event · `xgen-core` · `xgen-node` · the DM model itself (`L-4`, `L-6`, bilateral replication **H4** — locked at J-591 and **not** built here) · `entity-item.svelte` · the skin's unresolved rules (`N-168` is filed, not fixed) · `M-RP-OWN-ROW-NAME` · surface **A4**.

⚠️ **H4 (bilateral DM replication) IS LOCKED AND UNBUILT, AND THIS MILESTONE DOES NOT ADVANCE IT.** Its blocking measurement is still owed and is Chat's: **can a `space_id` derive from the identity pair rather than the signed root event?** *That answer decides whether H4 is a milestone or a rewrite (`M_RP_MEMBERS.md` §4c-ii), and it is not a prerequisite here — a DM created today is single-homed and stays so.*

---

## §8 — WHERE THIS DOCUMENT IS MOST LIKELY WRONG

1. 🛑 **THE ONE-FRAME HIGHLIGHT IN §4.1 IS REASONED, NOT MEASURED.** No probe has been run. If the stale highlight persists rather than flashing, **OQ1-P1 gets more expensive** and P2 gets more attractive. **This is the first thing Leg C's runbook should measure, and it must be measured at the PAINT layer** (`N-168`, `D-140`) — a store read will not decide it.
2. **The find-existing-DM scan is described and not designed.** `M_RP_MEMBERS.md:329` says *"scan known Spaces for an existing DM with that identity"* — but **G11: `KnownSpace` has no `is_dm` and no counterpart field.** Today that scan has nothing to scan on. ⇒ **OQ3's `is_dm` plumbing may be a prerequisite of Leg B, not Leg E.** *If so, this §6 order is wrong and E's Rust half moves forward.*
3. **`self_open`'s idempotence is tested at the op layer, not through a Tauri command that does not yet exist.** `created: bool` (G2) is the right signal; nothing has exercised it from the UI.
4. **Leg D assumes `entity-context-menu` works when mounted.** It is **COMPLETE and never instantiated** (J-675) — verified by its own sampler gates at M-RP5.3, never in the client against a real roster.
5. **This Phase-0 has not been read by anyone outside its author.** *Five defects in `RUNBOOK_TAIL8.md` were caught by Clair trying to execute it or by Joe looking at a screen; Chat's own re-reads passed every time.* ⚠️ **An adversarial read by Clair before Joe locks anything is worth its cost, and the `M-RP-TAIL8` arc is the evidence.**
6. **The seat line on `L-9` is Chat's reading, and it is cheap to be wrong about.** *"RMC without member's selection"* is taken to mean no bus write, no navigation, no R8 change. **If Joe meant only "no highlight", Leg D is different.** Stated because a one-word ruling is easy to over-extend — the `D-141` failure mode.