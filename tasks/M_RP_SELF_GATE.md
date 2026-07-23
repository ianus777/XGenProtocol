# M-RP-SELF-GATE — the self region as the client's utility gate
> **Status**: PENDING  
> Version: 0.1  
> Date: Jul 2026  
> **Last updated**: 2026-07-23  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — What this is, and what it is NOT

**This is a RECORDS document for a DEFERRED milestone.** It fixes what the self gate *is* so the
understanding is not lost; it does **not** authorise implementation.

- ⚠️ **ZERO code. ZERO `skin.css`.** Nothing in this document is buildable as written.
- ⚠️ **`self-panel.svelte` is NOT touched by this milestone** (Joe, 2026-07-23: *"self widget stays as
  is in the same format, just region we be created again for new"* · *"we can put it in the record, but
  we dont need to make edits on its regard"*). The gate is a **NEW region**, not a rewrite.
- **Appearance is NOT recorded here.** See §8.

## §1 — Why it exists

Joe, 2026-07-23: *"it is special widget that can be the gate to the whole user identity's setting"* and
*"this will be epicentre / heart of whole client"*.

The self surface was scoped as *"land the Self name + styling"*. That scope was **too small**: if the self
region is the entry point to identity settings and to the client's utilities, then **what it is** decides
what it shows and what it opens, and the name and styling are consequences.

## §2 — The reference, and what it is NOT

Joe supplied a Discord self-panel reference (2026-07-23), explicitly **not to copy**: *"i dont mean to copy
it, but to have such a gate where is doors to various utilities"*.

🔑 **The reference contains FOUR different kinds of thing at four different stages in XGen**, and reading it
as one thing is the error it invites:

| in the reference | XGen state, measured 2026-07-23 at `7408056` |
|---|---|
| avatar + name | **shipped** (`entity-item variant="card"`). ⚠️ `entity-avatar` reserves an `image?` that is **never fed** |
| self-set status line | ⚠️ **Track A.** `docs/xgen-status-gap-phase0.md` v1.0 ACTIVE cites *"per the Discord reference shot"* — the SAME reference, 2026-07-05. Protocol closed through PROTO-STATUS.2 (J-461); **client read path does not exist** (`self-panel.svelte` D6 ②: `status` ships ABSENT) |
| settings gear | modal area exists (D-122). **Cheapest door** |
| address book | ⚠️ **parts exist, destination does not.** `entity-avatar` (M-RP5.0) and `entity-item` (M-RP5.1) were built to materialize an address-book entry; **no store, no surface, no panel** |
| mic / headphones | ⚠️ **not identity** — device/call controls. XGen has no voice plane |

## §3 — LOCKED (Joe, 2026-07-23)

1. **The gate is a NEW region.** `self-panel.svelte` unchanged, zero edits. ⇒ the connection light stays,
   and the selection-bus writer (`self-panel.svelte:78`, D5's FIRST WRITER) is preserved by construction.
2. **Naming scope on the self surface is G ONLY** (D-124). L belongs on the other person's card.
3. **No name input in the self surface.** *"Customisable"* means **the toggle**, never renaming (D-124).
4. **All present functionality stays as it is** — Joe: *"all functionality can stay as is now"*.
   ⇒ **no new capability in this milestone.** Track A stays out. This is a reformat plus the toggle.
5. **The row is MIXED, and the kinds must stay distinguishable** (D-125). Utility buttons, toggles and
   indicators are different things and must not all be authored as buttons.

## §4 — FILED INTENT — recorded so it is not lost, NOT locked

Joe, 2026-07-23: *"locked will be on the round-0 before implementation. till that i will certainly meditate
about its appearance."* ⇒ these lock at **Phase-0** (D-071's canonical term), not here.

- **Plugin-extensible utility buttons.** ⚠️ If extensible, the strip becomes a **plugin surface under
  D-112/D-113** — materially larger than a widget. FILED INTENT, not a decision.
- **Help** as a utility button.
- **Further hot buttons**, including the device-control class.
- **A label indicating the G-name for orientation** (Joe, 2026-07-23: *"today it is not important"*).
- **The self panel card merging to "Self" when the toggle is ON.** ⚠️ Requires editing
  `self-panel.svelte`, which §3.1 forbids ⇒ **deferred to whenever the region is built**.

## §5 — Vocabulary

Fixed in **D-125**: the row is **utilities**; its button forms are **utility buttons**; the other kinds are
**toggles** and **indicators**. Joe, 2026-07-23: *"utilities -> utility buttons for button forms"*.

⚠️ **`merge`, never `collapse`** (D-124). `collapsed` is a **persisted layout schema field** on leaves
including `widgetId: 'self'`, twice migrated (`layout-default.ts:91`, v1/v2 boolean → v3 FoldAxis,
`migrateLayout`; `foldLeaf` / `handleFold`). 🔑 *Caught before it entered a decision record — the D-122
shape, third occurrence.*

## §6 — Blocked on

| # | blocker | note |
|---|---|---|
| 1 | **address book** — store + surface | the `entity-*` parts are already built for it |
| 2 | **Track A client read path** | only if the status line ever lands here |
| 3 | ⚠️ **the region ruling — tile or chrome** | `self` is one dock tile among eight (`layout-default.ts`). *"Heart of the client"* may mean chrome. **JOE'S, UNRESOLVED** |
| 4 | **display form per door** | D-122: decided **in situ**, in front of each one |

## §7 — Open, and JOE'S

- ⚠️ **§6.3 — tile or chrome.** Structural, not styling.
- ⚠️ **Fixed row or plugin-extensible** (§4.1).
- ⚠️ **Does the gate carry non-identity controls** (the mic/headphone class)?
- ⚠️ Carried from D-124: **name truthfulness by tier**, and **does S survive no-anonymity**.

## §8 — ⚠️ RE-OPEN-ON-BUILD CLAUSE (mandatory, D-122)

**Nothing in this document is an appearance verdict, and none may be treated as one at build time.**

D-122: *a display-form decision is a `[👁️ PERCEPTION]` call, and those cannot be made from records at all.*
**You cannot look at a document.** J-570 is the receipt: three typeface variants judged against fallback
fonts, because the thing looked at was not the subject.

⇒ At Phase-0 the following are **RE-OPENED, not read off this document**: layout, geometry, display form
per door, visual arrangement, and which utilities appear. 🔑 **The REASONING here is inherited; the
CONCLUSIONS are re-judged in front of the real screen.**

**Carried in unchanged** (already a captured verdict, J-575): `font-weight: 600` · `font-style: italic` ·
`letter-spacing: 0.05em` · `color: #E5E5E5` · ⚠️ `font-synthesis: none` REQUIRED.

## §9 — DoD

**[CHAT]**

- [x] D-124 and D-125 written
- [x] The four-kinds table grounded against code, not against the reference image
- [x] The `collapse` / `merge` collision recorded with its evidence
- [x] Re-open-on-build clause present and mandatory
- [ ] Phase-0 audit of the region model (tile vs chrome) — **at kickoff, not now**

**IMPLEMENTER** — none. This milestone writes no code.

## §10 — Handoff

**PENDING.** Unblocks on §6. Sibling milestones `M-RP-OWN-ROW-NAME` and `M-RP-INBOUND-NAME` live in the
**message/stream plane** and are NOT part of this document (J-576).