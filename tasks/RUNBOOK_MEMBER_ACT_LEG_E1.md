# M-RP-MEMBER-ACT — Leg E-1 Runbook: the `DM Spaces` widget (Clair)
> **Status**: ACTIVE  
> Version: 1.1  
> Date: Aug 2026  
> **Last updated**: 2026-08-12  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — SEAT, AND WHAT THIS LEG IS

🔒 **LOCKED BY JOE 2026-08-12 (*"e-1) locked"*). CLAIR MAY IMPLEMENT.**

📌 **Locked WITHOUT an adversarial read of this runbook** — Chat recommended one, Joe chose to lock directly. **Recorded as provenance, not as an objection:** if E-1 surfaces a defect, the record should show the read was offered and declined rather than never considered. ⚠️ **Rule 6 therefore carries more weight than usual here** — this document has been read by its author and by nobody else.

**You are CLAIR — Code Claude.** You implement from this runbook once Joe locks it. **You never push.** Deviations are **reported under Rule 6, never absorbed** — *an implementer who silently absorbs a bad instruction ships the architect's mistake* (M-RP7.1b is the evidence: your `migrateLayout(raw, fallback)` deviation stopped `core` importing a shell constant).

**E-1 builds ONE widget and mounts it. It does NOT build the filter.** The R1 filter is **E-3** and must not appear in this leg's diff — 🔒 **the home ships before the filter, and no commit Joe pushes ever contains the filter without the home** (Phase-0 §5). Until E-3 lands, a DM Space renders in **both** R1 and the new panel. That is expected, temporary, and not a defect.

**Read first:** `tasks/M_RP_MEMBER_ACT_LEG_E_PHASE0.md` **v1.3** (§2 audit · §3 F4/F5/F6/F11 · §4① constraints · §5 leg table) · `CLAUDE.md` PLAY head · `JOURNAL.md` J-718.

---

## §1 — GROUNDING (measured 2026-08-12 at `4cf2cfd`; re-verify, do not inherit)

📌 **Every `file:line` below came from a tool that printed it** — v1.0 of the Phase-0 estimated seven of them and all seven were wrong (F11/W1). **If a pointer does not match, that is a Rule 6 report, not a silent adjustment.**

| fact | where |
|---|---|
| `KnownSpace.counterpart: string \| null` **already ships** | `spaces-state.svelte.ts:34` (Rust `xgen-common/src/state.rs:198`) |
| the counterpart is the **session identity** for the self thread | `xgen-client/src/ops.rs:89` + `:94` (K3 backfill) |
| R1 renders every Space, unfiltered | `spaces-panel.svelte:50` |
| R1's DM-highlight suppression (C-bis-6, F-D in miniature) | `spaces-panel.svelte:58-63` |
| open-or-draft activation, to be **reused not reinvented** | `members-panel.svelte:244-300`; `findDm` at `:220`; the latch pair at `:267-268` |
| a region widget's props contract is **`{ regionId }` and nothing else** | `registry.ts` `RegionWidgetProps` |
| `buildWidgetRegistry` admits a `regionId` outside `REGION_IDS` | `layout-default.ts:64` |
| `entity-avatar` draws a **circle** on `flags.isDm` | `entity-avatar.svelte:59` |

**Live baseline, Joe's client, recorded as a SCREEN (N-184/N-190 — a count means nothing without one):** registry **168** · **7 Spaces, 3 of them DMs** · no latch · R1 `selectedId: null` · R2 empty (`no-room`) · `installed: []` · zero saved UI states.

---

## §2 — SCOPE: THE FILES YOU MAY OPEN

1. **NEW** `ui/common/lib/components/widgets/dm-spaces.svelte`
2. `ui/common/lib/plugins/registry.ts` — one `CLIENT_PLUGINS` row
3. `ui/client/src/layout-default.ts` — `REGION_IDS` + `REGION_NAMES` **only**

🛑 **NOT in this leg, and each has an owner:** `spaces-panel.svelte` (**E-3**) · `loadLayout` / the re-inject (**E-2**) · `DEFAULT_LAYOUT` (🔒 **stays at eight leaves — Joe ruled ①-B: the home is placed by re-inject ONLY, so drift between a fresh and a re-injected tree is structurally impossible**) · `skin.css` (**Joe's file, never in a Clair commit**) · any `.rs` (🔒 **`K2` shipped in Leg B; the cargo floor does NOT return — see §6**) · `ui/core/**` (would move the catalogue).

⚠️ **If E-1 cannot be built inside these three files, STOP AND REPORT.** The C-bis-6 precedent is exactly this: a locked two-file list needed four, you reported it, Joe accepted the scope. **A scope that has to grow is a finding; a scope that grows silently is a defect.**

---

## §3 — WHAT TO BUILD

### 3.1 The widget

**`dm-spaces.svelte` — the fourth `entity-panel` over the container umbrella** (N-013: Spaces and Rooms are *containers* in UI scope). R1 lists non-DM containers; this lists DM containers. **Same component, same row shape, complementary predicate.**

🔒 **ONE EXPORTED PREDICATE, TWO READERS.** Add to `spaces-state.svelte.ts`:

```ts
export const isDmSpace = (s: KnownSpace): boolean => s.counterpart != null;
```

⚠️ **Do NOT inline `counterpart != null` in the widget.** E-3 adds the complementary test to `spaces-panel`, and **two inline copies of one predicate are a `D-067` drift surface** — flip one, forget the other, and a Space renders in both panels or in neither. *(`spaces-state.svelte.ts` is a fourth file: it is a one-line export, and it exists precisely so E-3 cannot drift from E-1. **Report it as a scope note in your hand-back** so the record is honest about the count.)*

**Rows, in order:**
1. 🔒 **the self thread pinned FIRST** — `s.counterpart === selfId`. *(Joe: it is the only row that is not another person; sorting it under your own display name is arbitrary.)*
2. every other DM, **sorted by the resolved display name** used for its label (§3.2), case-insensitive.

**Descriptor per row:** `{ kind: 'space', id: s.space_id, name: <label>, flags: { isDm: true } }`.
📌 `flags.isDm` is **already drawn** by `entity-avatar:59` (circle vs rounded square) and has never been fed. **Feed it — this is its first consumer.**

🛑 **NO draft row.** A draft is not a Space, and a phantom row invents one (`N-091`). The draft already has two homes: R7's highlighted member row and R5's draft view.
🛑 **NO controls of any kind** — no remove, archive, export, pin, retention. `§5.7`'s census governs: *no control ships whose verb does not exist*, and none of these exist on any layer.
🛑 **NO store of its own.** 🔒 **The panel is a VIEW, never a STORE** — it derives from `spacesState` and persists nothing (Joe's *"the client is just reader-sender, doesn't hold any users data"*; `D-121` lens ② question 4).

### 3.2 The label (Leg E ③ = L2, resolve at render)

Resolve `s.counterpart` through the address book to a display name; fall back to **`tail8`**. 🔒 **Never rewrite `KnownSpace.name`** — `L3` was refused on `D-143`: the label is a display string a user can write, so it must never be a lookup key, which is the whole reason `counterpart` exists.

📌 **`descriptorFromId` (`members-panel.svelte:136`) is COPIED, not lifted.** This is the **second** independent impl and `J-508`'s extraction bar is **four**. Copying is the ruled choice; **do not refactor `members-panel`.**

🔓 **The fallback WORDING and whether a row shows the name alone or name-plus-discriminator are JOE'S.** Ship the name alone (matching R7, so two surfaces cannot disagree about one identity) and **surface it as a Joe item in the hand-back** rather than choosing.

### 3.3 Activation

**Reuse the existing shape — do not invent a second finder.** On row activation, for the matching `KnownSpace`:

```
roomLatch.latch(space.rooms[0].room_id)
spaceLatch.latch(space.space_id)
dmDraft.close()                 // close(), NOT clear() — clear() DELETES the draft's text
selection.set(regionId, descriptor)
```

⚠️ **`close()` not `clear()`** — a half-typed message to someone else must survive the visit (C-bis-7, Phase-0 §5.3).
⚠️ **Both latches, always** — `roomLatch.latch()` alone leaves `spaceLatch` on the previous Space (the J-708 ① split: R2 draws the old Space's rooms while R5/R6 target the DM's room). C-bis-6 fixed exactly this.
⚠️ **`rooms[0]` missing ⇒ NO-OP and REPORT.** A DM Space carries exactly one room; an empty `rooms` is a **finding**, not a case to paper over. No invented fallback. (`findDm` already guards it this way.)
🛑 **The self row is NOT excluded here.** `members-panel:249` refuses a self-click because a self *DM draft* makes no sense — but **the self thread is a real Space with a real room**, and this panel is the only GUI door to it (F5). **Do not copy the `!== selfId` guard.**

### 3.4 The descriptor row

In `CLIENT_PLUGINS` (**not** `AVAILABLE_CUSTOM`):

```
id: 'dm-spaces' · name: 'DM Spaces' · kind: 'system' · host: 'client'
delivery: 'compiled' · surface: 'region' · regionId: 'dm-spaces' · version: '1.0.0'
```

🔒 **`name` is `DM Spaces`, Joe's, and the reason is recorded because it is load-bearing:** *"direct messages we will have in the messages panel"* — **the widget lists Spaces, not streams.** Bare name, matching `Spaces`/`Rooms`/`Members`.
📌 **`icon` UNSET.** There is no verified DM glyph in-repo and **`D-108` forbids fabricating a Material `d` path from memory.** `plugin-list` falls back to its documented placeholder; the glyph is `M-RP-ICON-ADOPT` / `M-RP-SKIN`.
📌 **No `settingsComponent`** — nothing to configure yet, and `hasSettings` must stay descriptor-true (`D-113`'s correction: a control is greyed only for a reason true of that plugin).

`REGION_IDS` gains `'dm-spaces'`; `REGION_NAMES` gains a fallback title. 🛑 **`DEFAULT_LAYOUT` is NOT touched** (§2).

### 3.5 Appearance

🔓 **Joe's, entirely.** Ship **no `skin.css` edit and no component `<style>` block** (N-090/N-025 — zero component-local style, always). If the panel needs a rule that does not exist, **name it in the hand-back**; Joe writes it.

---

## §4 — HOW TO GET IT ON SCREEN, GIVEN §2 FORBIDS `DEFAULT_LAYOUT`

E-2 (the re-inject) does not exist yet, so nothing places the leaf. **Use the DEV bridge, exactly as M-RP-CONNSTATS did** (`app_client.svelte:406-408` — *"No install UI exists yet … this drives the mechanism for verify"*):

```js
__XGEN_LAYOUT__.set(insertLeaf(__XGEN_LAYOUT__.current, 'dm-spaces', 'spaces', 'bottom'))
```

🔒 **Target `spaces`, edge `bottom` — Joe's ruling, and it is verified in BOTH trees:** in his live tree `spaces`'s parent already runs `col`, so it is a **sibling** insertion → `[spaces, dm-spaces, self]`; under `DEFAULT_LAYOUT` the parent runs `row`, so it **wraps** → `[spaces, dm-spaces]`. **One pair, right answer in both.**

⚠️ **This is a VERIFY affordance, not the shipped path.** E-2 makes it permanent. **Do not persist it** — `uiStateStore.setSessionLayout` writes Joe's disk.

---

## §5 — VERIFY (drive it; do not predict it)

🛑 **Baseline and result must be measured in ONE sitting, on ONE client, with the SAME Space tree and draft state.** `N-184` (Space-dependent) and `N-190` (draft-dependent) make a carried number meaningless. **Record the screen, or record no number.**

| # | check | how |
|---|---|---|
| **V1** | the panel mounts and registers | `__XGEN_DEBUG__.get('dm-spaces#region-dm-spaces').state` |
| **V2** | **3 rows** at Joe's tree, self thread **first** if present | the aggregate getter **and** the painted DOM |
| **V3** | every row's avatar is a **circle** | computed style / the `flags.isDm` branch — **the first time this branch has ever been fed** |
| **V4** | activation opens the DM | R5 `effectiveRoomId` = the DM's room · R2 `count: 1` · **R1 unlit** |
| **V5** | 🛑 **R1 unlit is an EMPTY RESULT and must be POSITIVELY CONTROLLED (`N-099`)** | in the same session, click a **non-DM** Space and show exactly one R1 row goes `rgb(42, 47, 56)` |
| **V6** | a half-typed draft **survives** activation | type into a draft, activate a DM row, read `__XGEN_DRAFT__` — text intact (`close()` not `clear()`) |
| **V7** | registry transition | before → after, **enumerated not derived**; **state the screen** |
| **V8** | floors | `svelte-check` at **0/34/15** · catalogue **435 BY SCOPE** (zero `ui/core`) |

🛑 **`cargo` IS NOT A FLOOR FOR THIS LEG AND MUST NOT BE CITED AS ONE.** Leg E touches zero `.rs` — `K2` shipped in Leg B — so an identical `cargo` result is a **scope argument, not a measurement**. Phase-0 §6's *"cargo + svelte-check"* is stale (`F8`).

🛑 **DO NOT SEND A MESSAGE.** A send mints a **permanent DM** in Joe's live client. Nothing in E-1 requires one. **If you think a leg needs one, stop and ask Joe.**
📌 **Joe's client state is READ-ONLY to you.** No `setSessionLayout`, no install, no saved UI state.
📌 **Any probe that persists a mutation OWES A CLEANUP CALL, and the cleanup is part of the probe** (`N-123` — a leftover inline override once survived every edit Joe made and he reported it as a bug in his own CSS). **End the session at baseline and show it.**

---

## §6 — DEFINITION OF DONE

- [ ] `dm-spaces.svelte` built; **no component `<style>`**, no `skin.css` edit
- [ ] `isDmSpace` exported from `spaces-state.svelte.ts`; **the widget uses it, never an inline test**
- [ ] descriptor row in `CLIENT_PLUGINS`; `REGION_IDS` + `REGION_NAMES` updated; **`DEFAULT_LAYOUT` untouched**
- [ ] self thread pinned first; rest sorted by resolved label; **no draft row, no controls, no store**
- [ ] activation writes **both** latches, `dmDraft.close()`, and the bus; `rooms[0]`-missing is a no-op **and a report**
- [ ] **V1–V8 driven and recorded, each with its screen stated**
- [ ] V5's positive control shown — **an empty result with no control is not a pass**
- [ ] `svelte-check` 0/34/15 · catalogue 435 by scope · **no `cargo` claim made**
- [ ] Joe's client returned to baseline, shown
- [ ] deviations reported (Rule 6), **including the `spaces-state.svelte.ts` fourth file**
- [ ] 🔓 hand-back names the **label wording** as Joe's, and any skin rule the panel wants

---

## §7 — WHERE THIS RUNBOOK IS MOST LIKELY WRONG

1. **The `spaces-state.svelte.ts` predicate makes §2's three files four.** Deliberate and argued (§3.1) — but it is exactly the *scope written in files, requirements in behaviours* mismatch that blocked Leg C. **If it wants to be a fifth file, that is a finding.**
2. **Nothing here has been driven live.** The Phase-0's F5 (the self thread's only GUI door) rests on three call sites and **has never been exercised** — **V4 on the self row is its first real test.** If the self thread does not exist in Joe's tree (J-689: *"Joe has no self thread"*), say so; **do not create one.**
3. **The registry delta is unpredicted on purpose.** `N-184` says one Space row registers **two** entities. Three DM rows *suggests* +6 plus the panel — **measure it; a number derived by arithmetic does not enter the record until it has been seen.**
4. **This runbook has not been read by anyone but its author.** Every real defect in this arc came from outside the text — Joe's recall or you executing it. **Attack it.**
