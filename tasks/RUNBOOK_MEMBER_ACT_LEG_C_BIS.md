# RUNBOOK — M-RP-MEMBER-ACT Leg C-bis: the member with no DM opens a draft
> **Status**: ACTIVE  
> Version: 1.1  
> Date: Aug 2026  
> **Last updated**: 2026-08-09  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — WHAT THIS IS, AND THE ONE THING THAT IS NOT IN IT

📌 **v1.1 (J-705) CORRECTS THREE DEFECTS THAT C-bis-1's RE-DRIVE FOUND IN v1.0. ALL THREE WERE CHAT'S,
NOT CLAIR'S** — annotated below at §1, §2 and §3. **The steps govern; the file headers were approximate.**

**LOCKED runbook for Clair.** Design authority is `tasks/M_RP_MEMBER_ACT_LEG_C_BIS.md` **v1.3** §5;
nothing here re-opens it. Five commits, each measured alone.

🛑 **`ui/client/src/skin.css` AND `dm-intro.svelte` ARE JOE'S FILES.** Clair **mounts** the intro widget and
**never authors or edits its markup or its skin**, on the `skin.css` precedent. **C-bis-1 lands a structural
placeholder Joe then replaces.** If a gate appears to require editing either, **STOP AND REPORT** (Rule 6).

⚠️ **ZERO RUST.** `create_dm_space` is already a Tauri command and this leg is its first caller from the
webview. **`cargo` returning `1597/0/62 × 56` IDENTICAL is this leg's PROOF, not its assumption.**

---

## §1 — FLOORS AT OPEN (re-measure, never inherit)

| floor | value | note |
|---|---|---|
| `cargo` | **1597/0/62 × 56** | must be **byte-identical** at every commit |
| `svelte-check` | **0/34/15** | run from `ui/` |
| catalogue | **435** | every tile UNFOLDED, after a full `location.reload()` |
| client registry quiescent | **164** | 🔒 **RE-MEASURED AND CONFIRMED 2026-08-09 at `3f3c3e7`** |
| client registry, Space + room | ⚠️ **NOT A FLOOR — SPACE-DEPENDENT** | 🛑 **v1.0 STATED `174` AS A FLOOR AND IT IS NOT ONE.** Re-driven at `e0d4d9a`: **173**, stable across BOTH rooms of the Space driven. The count varies with the Space's rendered content. **Use the QUIESCENT 164 as the invariant; a selected-room count is only comparable against itself, in the same Space, before and after.** |
| Joe's client state | **2856 B**, LastWriteTime **2026-08-05 07:12:25** | 🔒 **CONFIRMED UNCHANGED 2026-08-09.** NEVER WRITE |

⚠️ `cargo clippy … -D warnings` has **four pre-existing errors, never clean.** **Not a floor. Do not fix.**

---

## §2 — 🔑 THE MEASUREMENT THAT SHAPED THIS RUNBOOK (live client, 2026-08-09, `3f3c3e7`)

**The `background` socket is NOT the intro page's home**, and the Phase-0's §5.8 claim that it was is
annotated at the site by this runbook (`D-131`; `3f3c3e7` is pushed and cannot be edited).

- `message-stream.svelte:255` — `<div class="message-stream-bg" aria-hidden="true">`. **HARDCODED.**
  ⇒ the intro would be **the only content on screen and invisible to assistive tech**, and *"later
  elements"* (buttons) inside an `aria-hidden` container are unreachable. **Correct for wallpaper. Wrong
  for content.**
- `message-stream.svelte:125` — `showEmpty = count === 0 && !backgroundDeclared` ⇒ declaring a background
  **suppresses** the empty state.
- `message.svelte:133-136` — the entire `system` sub-tree is **one `<Paragraph>`**; `:92-98` FORCES the
  text-only fields off. ⇒ **a `system` row cannot host a widget mount.** It carries the sentence; it cannot
  carry the page.

**PAINTED HEIGHTS, read off the live client:**

```
.message-stream-shell   h=544   display:block   flex:0 1 auto
.message-stream         h=544   display:block   overflow-y:auto
.message-stream-rows     h=18   display:block
.stream-panel           h=544   display:block   (height:100%; min-height:0 — its own <style>)
.region-tile-body       h=560   display:block   flex:1 1 0%
.region-tile            h=582   display:flex    flex-direction:column
```

🛑 **MEASURE THESE BY WALKING THE PARENT CHAIN UP FROM `.message-stream-shell` — NEVER BY A FLAT
`querySelector` ON THE CLASS.** There are **EIGHT** `.region-tile-body` elements in the client;
`document.querySelector('.region-tile-body')` returns the **FIRST IN DOCUMENT ORDER, A DIFFERENT REGION**,
and at J-705 that produced **738/760** — numbers that are real, wrong, and measurement-shaped.
⚠️ **v1.0 PRESENTED THIS LIST AS A FLAT TABLE, WHICH INVITES EXACTLY THAT READ.** Clair walked the chain
and was right; the seat that wrote the gate is the one that fell into it.

🔑 **THE STREAM FILLS ITS TILE AND WILL NOT COLLAPSE** — 18px of content inside 544px. ⇒ **a naive sibling
above it OVERFLOWS rather than shares.** That is why C-bis-1 exists as its own commit.
📌 **And the rows are already top-anchored in mostly-empty space**, so the intro will not look like an anomaly.

---

## §3 — THE COMMITS

### C-bis-1 — `stream-panel` becomes a flex column and grows an `above` socket — ✅ **DONE, `e0d4d9a`, RE-DRIVEN AND PASSED (J-705)**

**Files:** `ui/common/lib/components/widgets/stream-panel.svelte` ·
`ui/common/lib/components/widgets/dm-intro.svelte` (NEW, placeholder)
🛑 **v1.0 ALSO LISTED `ui/client/src/layout-default.ts` HERE AND THAT WAS WRONG.** Clair reported it under
Rule 6 and **the report is correct**: `N-096` means a region widget receives only `regionId`/`id`, so a
store-mediated socket with a LOCAL registry has **no layout surface at all** — neither the mounts nor the
registry can arrive as a prop. **Nothing in `layout-default.ts` was needed and nothing was touched.**

1. **`stream-panel`'s root becomes a flex column.** Its `<style>` block only:
   ```css
   .stream-panel { height: 100%; min-height: 0; display: flex; flex-direction: column; }
   ```
   and `MessageStream`'s wrapper takes `flex: 1 1 0; min-height: 0`.
   🛑 **`MessageStream` STAYS UNCONDITIONALLY MOUNTED** — `:198`'s invariant (*no conditional mount → no
   registry churn*) is **preserved deliberately**, and the registry floor is the proof.
2. **An `above` socket**, `WidgetMount[]`, resolved with `resolveMounts(above, widgets, cid('a-'))`
   (`mounts.ts:51`), rendered **before** the stream, `flex: 0 0 auto`.
   ⚠️ **Reactivity: `resolveMounts` re-derives on the `widgets` REFERENCE.** A registry mutated in place
   yields nothing — **reassign a fresh object** (`mounts.ts` header; `D-119` paid for this once).
   ⚠️ Unknown `widgetId` ⇒ **DROPPED** (W-13), so the count getter is a drop-unknown proof.
3. **`dm-intro.svelte`** — a **structural placeholder only**: avatar, heading, paragraph, in that order,
   fed by props. 🛑 **NO copy decisions, NO skin rules, NO layout opinions — Joe replaces this.**
4. **The getter grows** `aboveMountCount: resolvedAbove.length`.

**GATE C-bis-1**
- [ ] ⚠️ **THE FLEX CHANGE IS THE RISK.** Read the painted heights of all six elements in §2 **BEFORE and
      AFTER**. `.stream-panel` and `.message-stream-shell` **must still be 544** with no draft.
- [ ] Client registry **164 quiescent** — **UNCHANGED.** The always-mounted invariant is what this proves.
      ⚠️ **The selected-room count is NOT a floor** (§1): compare it only against itself, same Space, before
      and after.
- [ ] `svelte-check` **0/34/15** · catalogue **435** · **`cargo` IDENTICAL**.

---

### C-bis-2 — the `dmDraft` store, and R7's click opens a draft

**Files:** `ui/common/lib/stores/dm-draft.svelte.ts` (NEW) ·
`ui/common/lib/components/widgets/members-panel.svelte` ·
`ui/common/lib/components/widgets/stream-panel.svelte`
⚠️ **v1.0 LISTED `layout-default.ts` HERE AND OMITTED `stream-panel.svelte`, WHICH IS THE MIRROR IMAGE OF
C-bis-1's ERROR** — step 3 below plainly edits `stream-panel`. Clair caught both in one read. **Corrected;
and as §0 now says, THE STEPS GOVERN.**

1. **`dmDraft`** — a sibling store. **NOT a third state inside `roomLatch`**: that store's header declares
   *one predicate, both widgets*, and a state meaning *"no room, but pretend"* would make **`canSend` lie**,
   which is the exact failure it exists to prevent (`room-latch.svelte.ts:5-18`).
   🛑 **DO NOT NAME IT `draft`** — `composer-panel.svelte` already uses `draft` for its local text variable.
   - `active: boolean` · `counterpart: identityId | null` · text **keyed by counterpart**
   - **survives navigation with its typed text** (Phase-0 §5.3, Joe). **No persistence** — the client holds
     no user data (J-598, Joe's lock); it dies with the session like every other client state.
2. **`onMemberActivate`** opens a draft when the member has **no existing DM**; unchanged when one exists.
   ⚠️ **`N-171` IS FIXED HERE BECAUSE THIS LEG OPENS THAT FUNCTION** — move the lookup **above** `latch()`.
   🛑 **The locked write ORDER is untouched.**
3. **R5 mounts `dm-intro` in `above`** when `dmDraft.active`. **R7 keeps the group roster — `scope` never
   moves.**

**GATE C-bis-2**
- [ ] Click a never-DM'd member on the **live client**: the intro paints, R7 **still shows the group roster**.
- [ ] Type → navigate to a room → return: **the typed text is still there.**
- [ ] 🛑 **`roomLatch` reports NOTHING LATCHED throughout.** If `canSend` is true here, the design is violated.
- [ ] Floors; **`cargo` IDENTICAL**.

---

### C-bis-3 — the composer's draft branch (no create yet)

**File:** `ui/common/lib/components/widgets/composer-panel.svelte`

- `sendEnabled` gates on `(canSend || dmDraft.active) && hasText`.
- 🔑 **`submit()` RETURNS EARLY on `roomId == null`** (`:65-76`), *"because a disabled button is a courtesy,
  never a guarantee"* ⇒ **the draft branch goes ABOVE that early return.** **Do not fabricate a `roomId`.**
- This commit **routes** the draft send to a stub that does nothing but report. **No `create_dm_space` yet.**

**GATE C-bis-3** — composer is LIVE on a draft, still dead with nothing latched · floors · **`cargo` IDENTICAL**.

---

### C-bis-4 — the send sequence and the failure surface

**Files:** `composer-panel.svelte` · `dm-draft.svelte.ts`

```
create_dm_space(invitee)          → CreateDmSpaceResult   (xgen-client/src/ops.rs:827)
roomLatch.latch(result.room_id)   → the latch becomes REAL, first time
echo.send(space_id, room_id, text)
dmDraft.clear()
```

✅ `room_id` is **in the result** ⇒ **no resolution step, no round trip.**
✅ The verb signs and sends a three-event causal chain and **aborts writing nothing** on timeout
(`ops.rs:838-843`, test `create_dm_space_aborts_and_writes_no_record_when_chain_times_out`) ⇒ **the client is
clean either way.**
⚠️ **IT NEEDS A LIVE NODE AND IT CAN FAIL.** 🛑 **`D-065`: keep the draft OPEN, keep the typed text, surface
the failure, and let NOTHING on screen imply the DM exists.**

**GATE C-bis-4**
- [ ] Happy path on the live client: send → the DM exists → **every shipped mechanism takes over untouched.**
- [ ] 🛑 **FAILURE PATH DRIVEN, NOT ASSERTED: kill the node, send, and watch it fail.** Draft open, text
      intact, failure visible.
- [ ] Floors; **`cargo` IDENTICAL**.

---

### C-bis-5 — verification, the `OWED-4` measurement, and records

- [ ] **`OWED-1` DISCHARGED — verified on the live client**: no member row presents as actionable while doing
      nothing.
- [ ] **`OWED-4` MEASURED AND SHOWN TO JOE.** A second, **non-erased** DM is now reachable, which ends the
      fixture blockade on `§6` leg 5. 🛑 **MEASURE AND SHOW. DO NOT RULE** (`D-146`).
- [ ] **§5.6 exercised: the NAMELESS counterpart FED, not asserted** — the page renders `…a1b2c3d4` and the
      sentence still reads.
- [ ] **§5.5 exercised: a 128-BYTE NO-SPACE name FED** — the header neither overflows nor blanks.
- [ ] **§5.6-bis held: NO xgid row; no full XGID of another identity anywhere on the page.**
- [ ] **§5.7 held: NO control ships whose verb does not exist.** Count the buttons and justify **each one**.
- [ ] **J-618 read and ANNOTATED AT THE SITE** (`D-131`) — §7 of the Phase-0.
- [ ] `D-074` atomic records: task doc · `CLAUDE.md` PLAY · `JOURNAL.md` · `docs/ROADMAP.md`.

---

## §4 — 🛑 TOOLING THAT COST SOMETHING TO LEARN

- 🛑 **`__XGEN_DEBUG__` EXPOSES `ids`, `get`, `snapshot` — THERE IS NO `list`.** Verified 2026-08-09 by
  `Object.keys` against the live client at `3f3c3e7`. Chat called `list()` while measuring for this runbook
  and it **threw**. ***It failed loudly, which was luck, not
  method*** — a probe whose pass condition is an empty result would have returned a clean-looking `[]` and a
  **false absence would have entered the record.** **N-099: positively control every such probe.**
- 🛑 **`get(id)` returns `{type, state}`** — read `get(id).state.<field>` (`N-169`). ⚠️ **AND `get` NEEDS THE
  FULL `type#id`**: `get('region-stream')` returns **`null`**, while `get('stream-panel#region-stream')`
  resolves. **Another empty result that means "wrong key", not "absent"** — measured at J-705.
- 🛑 **`data-debug-id` carries the FULL `type#id`** (`entity-panel#region-spaces__panel`), never the bare
  `id` prop (`N-170`). The `#` means an unquoted CSS attribute value cannot express it — filter
  `[data-debug-id]` and compare with `getAttribute`.
- **Post-mutation DOM reads need a SEPARATE eval** — click and read in one call returns the PRE-change DOM.
- **`-At` IS FORBIDDEN ON KEYBOARD LEGS**: it focuses by CLICKING, and on `entity-panel` a click IS an
  activation. `el.focus()`, then ArrowDown. ⚠️ `el.focus()` does **not** move `activeIndex`.
- **CDP port opens BEFORE Svelte mounts the bridge** — poll `!!window.__XGEN_DEBUG__` until non-null.
- 🛑 **THE NODE MUST BE RUNNING** or the roster fill fails and R7 shows self only.
- 🛑 **Commit messages via `-F <file>`, and WRITE THE FILE BEFORE HANDING OVER THE COMMAND** (J-700 left a
  tree staged-not-committed, looking finished). `[System.IO.File]::WriteAllText(path, text, (New-Object
  System.Text.UTF8Encoding($false)))`. **NEVER `Set-Content -Encoding UTF8`** — PS 5.1 writes a BOM.
- 🛑 **CRLF: `CLAUDE.md` (1137/1137) and `docs/ROADMAP.md` (519/519) ONLY.** Verify by **COUNTING BYTES**;
  `core.autocrlf=true` makes `git diff` blind to it.
- 🛑 **NEVER RUN A WHOLE-FILE TRANSFORM WHERE A LINE-INDEXED EDIT WILL DO.** At J-703 Chat generalised a
  two-line blank-line fix into a global strip and **deleted 294 blank lines from `CLAUDE.md`**; caught only
  by the byte count, reverted from HEAD. Same species as J-697's global regex.
- 🔒 **`roadmap-format-gate.ps1` MUST RETURN 0** before any commit touching `docs/ROADMAP.md`.

---

## §5 — RULES

- **Rule 5:** Chat re-drives every measured leg independently. **No number enters the record on report alone.**
- **Rule 6:** Deviations are **reported, never absorbed.** A gate that cannot be met as written is a STOP.
- 🛑 **VERIFY THE ARTIFACT, NEVER THE EXIT CODE — and never the word "done", including Joe's and your own.**
- **Joe pushes.** Clair never pushes.
