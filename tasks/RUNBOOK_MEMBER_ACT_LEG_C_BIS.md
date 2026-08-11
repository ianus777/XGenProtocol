# RUNBOOK — M-RP-MEMBER-ACT Leg C-bis: the member with no DM opens a draft
> **Status**: ACTIVE  
> Version: 1.11  
> Date: Aug 2026  
> **Last updated**: 2026-08-11  
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

> 🔒 **AMENDED FOR THIS LEG ONLY — JOE, 2026-08-09 (v1.2):** *"make a placeholder, but with content. i will
> check it independently afterwards and maybe i do some updates manually or leave it as is."* ⇒ **C-bis-2 MAY
> EDIT `dm-intro.svelte` to place CONTENT.** 🔑 **This is not a copy decision being delegated — the copy was
> ALREADY DECIDED:** §5.5's sentence is Joe's, locked at J-703, verbatim. **C-bis-2 PLACES a locked string; it
> does not AUTHOR one.** 🛑 **THE AMENDMENT IS NARROW: `ui/client/src/skin.css` IS UNTOUCHED AND STILL A STOP.**
> The page therefore ships **UNSKINNED** — §5.5's truncation rules (`overflow-wrap: anywhere`, one-line header
> clamp, the header `min-width`) are **skin, and they land when Joe skins it.** ⚠️ **No new wording, no second
> sentence, no label, no control** may enter the file under this amendment (§5.7's button census is unchanged).

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

### C-bis-2 — the `dmDraft` store, and R7's click opens a draft — ✅ **DONE, `96a935f`, RE-DRIVEN AND PASSED (J-706)**

**Files:** `ui/common/lib/stores/dm-draft.svelte.ts` (NEW) ·
`ui/common/lib/components/widgets/members-panel.svelte` ·
`ui/common/lib/components/widgets/stream-panel.svelte` ·
`ui/common/lib/components/widgets/dm-intro.svelte` (⚠️ **FOURTH FILE, added at v1.2 under §0's Joe amendment**
— content only; **`skin.css` still a STOP**) ·
`ui/client/src/app_client.svelte` (⚠️ **FIFTH FILE, added at v1.3** — **ONE line + one import**: `dmDraft.note(sel)`
joins the existing two-latch effect at `:196-206`)

> ## 🛑 v1.3 — THE C-bis-2 RULINGS ARE SUPERSEDED. DRIVEN AND FALSIFIED AT THE HAND-BACK (2026-08-09).
>
> **The kickoff's `R-1` (clear the room latch on opening a draft) and `R-2` (`active` derived off the latch)
> are WITHDRAWN. Both were Chat's. Clair implemented them faithfully, drove them live, and reported the
> contradiction under Rule 6 rather than absorbing it.**
>
> 🔑 **THE MECHANISM, RE-DRIVEN FROM SOURCE:** `members-panel.svelte:57` —
> `const scope = $derived(roomLatch.effectiveSpaceId)`, *"the widget's authoritative scope"*. `clear()` nulls
> it ⇒ **R7 empties to `no-scope`**, which is the exact gate line 1 forbids. ⚠️ **AND A SECOND CONSEQUENCE
> NOBODY HAD NAMED:** `app_client.svelte:218` drives the members fill off that same getter as *"the sole
> tracked dependency"* ⇒ clearing **tears down the fill and forces a fresh node round-trip on every draft open
> and close.**
>
> 🔑 **AND `R-1` WAS NEVER NEEDED.** It defended against a send to the stale room. **That defence already
> existed and was already locked** — §5.2 and C-bis-3: *the draft branch goes **ABOVE** the early return* ⇒ a
> draft send **never consults `roomLatch` at all**. **A duplicated guarantee that cost R7 its scope.**
>
> 📌 **Species: *a claim narrower than the thing it describes.* Three widgets read that latch; the ruling
> reasoned about one, in a leg whose own gate names a second. Caught from OUTSIDE the text, by driving it —
> never by re-reading it.**
>
> 🔓 **FILED, NOT THIS LEG'S:** giving R7 a scope that survives a room change (`spaceLatch` = *the Space you
> are **browsing***) — ⚠️ **`roomLatch.effectiveSpaceId` was EXPLICITLY REFUSED as its source**
> (`space-latch.svelte.ts` header), and adopting it reverses **`L1`'s deliberate B1 cost**. **Real value, its
> own milestone, Joe's to schedule.**
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
   🛑 **v1.3: IT DOES *NOT* CALL `roomLatch.clear()`. THE LATCH NEVER MOVES WHEN A DRAFT OPENS** ⇒ R7 keeps
   its `scope`, its roster and its fill; §5.1's *"scope never moves"* holds **literally**.
   ⚠️ **`N-171` IS FIXED HERE BECAUSE THIS LEG OPENS THAT FUNCTION** — move the lookup **above** `latch()`.
   🛑 **The locked write ORDER is untouched.**
   ✅ **TWO GUARDS ARE REQUIRED, NOT A WIDENING** (Clair, accepted at v1.3): `findDmRoom` *(`D-131` — renamed `findDm` at C-bis-6, J-711; returns `{ space_id, room_id }`, and the self-collapse below is UNCHANGED)* collapses **self**
   and **no-DM** into one `null`, and the draft path is the first caller that must tell them apart — without a
   **self guard** a self-click opens a self-draft. The **empty-rooms** case (a malformed existing DM) stays a
   **no-op and stays a FINDING**; drafting on it would mint a duplicate.
3. **R5 mounts `dm-intro` in `above`** when `dmDraft.active`. **R7 keeps the group roster — `scope` never
   moves.**
   🛑 **v1.3 — AND R5 FEEDS `MessageStream` AN EMPTY LIST WHILE A DRAFT IS ACTIVE.** The latch still points
   at the room you came from, so without this the intro paints **on top of that room's conversation.**
   🛑 **A PROPS CHANGE, NEVER A CONDITIONAL MOUNT** — C-bis-1's unconditional-mount invariant stands and the
   **164 floor is its proof.** 📌 It renders `"No messages yet"` (`message-stream.svelte:267`) — honest, not a
   false statement; **its appearance is Joe's.**
3b. 🔒 **v1.3 — `active` IS STORED (`counterpart != null`), NOT DERIVED OFF THE LATCH, AND THE DRAFT CLOSES ON
   A `room` SELECTION VIA `dmDraft.note(sel)`.** 🔑 **This is the SHIPPED idiom, not a new mechanism:**
   `app_client.svelte:196-206` is *"ONE effect, TWO latches"* — `spaceLatch.note` joined `roomLatch.note`
   there at M-RP-SELECT-ORIENT C-1, and `dmDraft.note` makes it three. **Single writer, shell-fed, reads its
   argument and never `_active`** (`N-136`). ⚠️ **This is the FIFTH file and it is named as a scope change,
   not smuggled.** 📌 **Any `room` selection closes it — including re-selecting the room you were already in**,
   which a latch-value comparison would have missed. **The text map survives; §5.3 is unaffected.**
4. **THE PAGE IS FED, NOT LEFT BLANK (v1.2).** 🔑 **The name is resolved ONCE and passed as DATA; the SENTENCE
   is composed INSIDE `dm-intro`** — the copy lives in the file that owns it, so Joe edits **one** place and
   `stream-panel` carries **no wording at all.**
   - `stream-panel` resolves the counterpart's label **reactively** from `addressBook.book[id]?.display_name`,
     falling back to `…` + last-8 (`members-panel.svelte:51`'s `tail8` shape). ⚠️ **Reactive, not frozen at
     click time** — §5.6 case 1 (LIVE-JOINED, `unresolved`) is cleared by the next fill and the header must
     follow it. ⚠️ **`tail8` now exists in two files — REPORT the duplication, do NOT extract it** (a shared
     helper is a fifth file and a scope change; Chat files it).
   - `dm-intro` renders the label as the **heading** and, beneath it, §5.5's sentence **verbatim**:
     *"This is the start of private direct message stream with {counterpart\_display\_name}."* ⇒ **the name
     appears TWICE** (§5.5's lock, Discord's shape, marked PROVISIONAL by Joe).
   - 🛑 **ONE interpolation point, and it stays a TEXT NODE** — §5.8: no `{@html}`, no sanitiser surface, wire
     data escaped by construction.
   - 🛑 **NO xgid row** (§5.6-bis). 🛑 **NO control ships whose verb does not exist** (§5.7). The avatar stays
     the `aria-hidden` structural box — **there is no avatar source to wire and none is invented.**

**GATE C-bis-2**
- [ ] Click a never-DM'd member on the **live client**: the intro paints, R7 **still shows the group roster**.
- [ ] **The counterpart's name renders TWICE** — heading + sentence — and the sentence is **byte-verbatim §5.5**.
- [ ] **A NAMELESS counterpart FED, not asserted**: the page reads `…a1b2c3d4` in both places and the sentence
      still reads. ⚠️ **If no nameless member is reachable, SAY SO — do not assert it from the code.**
- [ ] Type → navigate to a room → return: **the typed text is still there.**
- [ ] 🛑 **v1.3 REPLACES v1.2's *"`roomLatch` reports NOTHING LATCHED throughout"* — THAT LINE IS NOW FALSE BY
      DESIGN.** The latch **stays**, so `canSend` **is true during a draft**, and the send-safety guarantee
      moves to its real home: **C-bis-3's draft branch ABOVE the early return** (§5.2). **What C-bis-2 asserts
      instead:** `roomLatch.latchedRoomId` is **BYTE-IDENTICAL before and after** the draft opens, and R7's
      `scope` / `panelState` / `rowCount` are **UNCHANGED**.
- [ ] **The stream renders EMPTY under the intro** — `streamCount: 0` — while the latch still points at the
      room you came from. 🛑 **Registry still 164:** `MessageStream` never unmounted.
- [ ] **Click a room → the draft closes** (`aboveMountCount: 0`) **and the room's stream returns.**
- [ ] Floors; **`cargo` IDENTICAL**.

✅ **GATE C-bis-2 CLOSED — CHAT RE-DROVE EVERY LINE ON A FRESH CLIENT (J-706).** R7 `known` / scope `…7b208` /
`rowCount 2` **identical before, during and after** · latch `…bda3924a` → `…bda3924a` · `streamCount 2→0`,
`aboveMountCount 0→1` · `message-stream#region-stream__stream` **present DURING the draft** · heading
`BobLegB`, body **64 chars byte-verbatim §5.5**, `nameCount 2` · `ed25519` **absent** from the intro's HTML,
`controls 0`, avatar `aria-hidden="true"` and empty · **re-selecting the SAME room closes the draft** · cargo
**1597/0/62 × 56** · svelte-check **0 errors / 34 warnings / 15 files** · registry quiescent **164** twice ·
Joe's client state **byte-identical**.

🔑 **TWO LEGS DRIVEN THAT NO GATE NAMED, AND BOTH NEEDED IT:** the **self-click no-op** (Clair's new guard is
the only thing between a self row and a self-draft — argued at the ruling, now **driven**), and the
**existing-DM branch at `96a935f`** (`…sno_FWmw` → re-latches `…a297bb57`, no draft, counterpart highlight
intact). ⚠️ **The second one was reported as *"branch untouched in v1.3"* — but the `N-171` restructure moved
the lookup above BOTH branches, so it was touched and needed driving.** *A branch you did not run is a branch
you did not verify, however small the diff looks.*

---

### C-bis-3 — the composer's draft branch (no create yet) — ✅ **DONE, `8601e677`, RE-DRIVEN AND PASSED (J-707)**

> 🔑 **THE LEG'S REAL CONTENT WAS A PRIVACY PROPERTY, NOT A SEND GATE.** `draft` was ONE local unkeyed
> string, so without routing the draft's text through `dmDraft` **the private message typed to a counterpart
> would sit in the GROUP ROOM's composer the instant the draft closed.** Driven both ways: room buffer 8
> stays 8 across a draft open/close; the draft's 15 restores on return (§5.3, **end-to-end for the first
> time**).
> ⚠️ **AND THE GATE AS WRITTEN COULD NOT FAIL.** *"composer LIVE on a draft"* passes **before any code is
> written**, because v1.3 keeps the latch set ⇒ `canSend` is already true. `(canSend || dmDraft.active)` is
> **UNREACHABLE in its right arm** — R7 only renders rows when a room is latched, so a draft cannot exist with
> `canSend` false. **Clair implemented the locked predicate and REFUSED to claim the arm as verified.**
> 📌 **Recorded as unreachable, NOT as tested** — `N-091`'s discipline applied to a guard.
> 🔑 **What the re-drive found that neither report stated: `sendEnabled` came back FALSE on a fresh draft
> while `canSend` was TRUE**, because `hasText` reads through the routed buffer. ⇒ **the routing is also what
> stops a live room latch from arming Send over an empty draft with someone else's words in it.**

**File:** `ui/common/lib/components/widgets/composer-panel.svelte`

- `sendEnabled` gates on `(canSend || dmDraft.active) && hasText`.
- 🔑 **`submit()` RETURNS EARLY on `roomId == null`** (`:65-76`), *"because a disabled button is a courtesy,
  never a guarantee"* ⇒ **the draft branch goes ABOVE that early return.** **Do not fabricate a `roomId`.**
- This commit **routes** the draft send to a stub that does nothing but report. **No `create_dm_space` yet.**

**GATE C-bis-3** — composer is LIVE on a draft, still dead with nothing latched · floors · **`cargo` IDENTICAL**.

---

### C-bis-4 — the send sequence and the failure surface — ✅ **DONE, `37c09d7`, BOTH GATES DRIVEN (J-708)**

> ## 🛑 §5.2 IS AMENDED AT v1.6 — THE SEQUENCE AS WRITTEN COULD NOT RESOLVE THE LATCH.
>
> **The runbook said `create → latch → send → clear` and asserted *"the latch becomes REAL, first time"*.
> IT DOES NOT.** Clair found it from source and reported under Rule 6; Chat re-drove every link independently:
> `resolveLatched()` scans `spacesState.spaces` (`room-latch.svelte.ts:48-55`) · **`setSpaces(` has EXACTLY ONE
> call site in the whole UI tree** (`app_client.svelte:256`, inside `loadSpaces`) · `create_dm_space` pushes the
> new `KnownSpace` and calls `write_client_state` — **DISK ONLY** (`ops.rs:1025-1041`).
> ⇒ a bare `latch(result.room_id)` **resolves to NULL**: the user sends their first message to someone and
> lands on **"Select a room"** with the message nowhere (`echo.forRoom(null) → []`).
>
> 🔒 **THE SEQUENCE IS NOW `create → REFRESH → latch → send → clear`.** The refresh is `await loadSpaces()`
> **inside the shell's injected transport** — the only place both halves are in scope, since `loadSpaces` lives
> in the shell and the widget cannot reach it (W-3).
> ⚠️ **THE CHEAPER ALTERNATIVE WAS CONSIDERED AND REFUSED:** pushing a synthesised `KnownSpace` from
> `result` would **invent** `name` / `role` / `node_endpoint` (`D-065`) and mint a second shape of the same
> record (`D-067`). **Re-reading the authority is correct.**
> 📌 **A `loadSpaces` failure AFTER a successful create is SWALLOWED, and that is right:** rejecting would
> report failure for a DM that WAS created — §5.4's lie inverted.

**Files (v1.6 — the runbook's two-file list was WRONG):** `composer-panel.svelte` · `dm-draft.svelte.ts` ·
**`ui/client/src/app_client.svelte`** — ⚠️ **a THIRD file, because `ui/common` has ZERO `@tauri-apps` imports
(W-3/`N-096`) and NEITHER listed file can call `create_dm_space`.** The shell injects the transport, the
`echo`/`send_message` seam shape at `:796`.

```
create_dm_space(invitee)          → CreateDmSpaceResult   (xgen-client/src/ops.rs:827)
await loadSpaces()                → 🛑 v1.6 AMENDMENT — without this the latch resolves to NULL
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

✅ **BOTH C-bis-4 GATES ARE ALREADY DRIVEN (J-708) — C-bis-5 DOES NOT RE-OWE THEM.**

**GATE ② — THE FAILURE PATH, DRIVEN WITH THE NODE KILLED MID-DRAFT.** `draftError` carries **the node's own
words** (*"failed to connect to Node … os error 10061"*), the draft stays open (`aboveMountCount 1`), the text
is kept (`domValue` intact, store text identical), and **nothing implies the DM exists**: `echoCount 0`,
`streamCount 0`, latch unmoved, `spaceCount 5`, **client state SHA-256 IDENTICAL**.

**GATE ① — THE HAPPY PATH, DRIVEN ONCE, ON `LegF-N5`, WITH JOE'S PER-LEG CONSENT.** `create` → latch moves
`…88aa8324 → …b9c4c6f4` → **`effectiveRoomId` RESOLVES** → `echoCount 0 → 1`, the row reads the sent text and
**`Sent`** → draft cleared → `spaceCount 5 → 6`; client state `03F4890A… → 4AD3B7B3…`, 2856 → 3629 B.
🔑 **THAT RESOLUTION IS THE AMENDMENT'S PAYOFF AND THE WHOLE POINT OF THE LEG.**

⚠️ **AND THE STORE SAID GREEN WHILE THE SCREEN DID NOT — JOE FOUND BOTH DEFECTS IN A SCREENSHOT.** They are
**C-bis-6** and **C-bis-7** below. *A string is not a layout is not a picture, and the third layer has no probe
the human eye does not beat.*

### ✅ C-bis-6 — after the create, the client ORIENTS (Joe's rule, 2026-08-09)

🔒 **RULED BY JOE 2026-08-09 — OPTION A, AND IT COSTS NO LOCK.**

🔑 **`D4 opt-2` (*"a later room/identity selection KEEPS the Space lit"*) WAS NEVER RULING ON THIS CASE.** Every
selection it contemplated was **WITHIN** a Space — a room in R2, a person in R7. **Opening a DM from a member
row is a CROSS-SPACE navigation, and it did not exist when that decision was made**: member activation is what
this milestone built. ⇒ **A is a SCOPE CLARIFICATION, not a reversal.** Intra-Space selections keep the Space
lit; **a DM open is a Space change and moves the Space latch like any other Space change.** ✅ **Joe confirmed
both the option and this reading.**

**Files:** `members-panel.svelte` · `spaces-panel.svelte`. 🛑 **NO new store, NO shell change, ZERO `.rs`.**

> 🛑 **`D-131` — THIS FILE LIST WAS WRONG, AND IT SHIPPED LOCKED. ANNOTATED, NOT REPAIRED (J-711).** The leg
> took **FOUR** files: `space-latch.svelte.ts` · `members-panel.svelte` · `composer-panel.svelte` ·
> `spaces-panel.svelte`. **Clair reported it under Rule 6 rather than absorbing it, and Joe accepted the scope.**
> ⚠️ **WHY IT WAS WRONG, because the reason is worth more than the correction:** step 1 requires the SPACE
> context to move on **both** branches, and the two `roomLatch.latch()` sites are `members-panel` **and
> `composer-panel`** — the post-create latch does not live in members-panel at all. The store shortcut (having
> `roomLatch.latch()` move `spaceLatch`) is forbidden by BOTH latch headers, so the move must happen at the
> CALLERS. 🔑 **AND CHAT'S KICKOFF DIAGNOSED IT WRONG IN THE EASY DIRECTION:** it claimed the room latch
> *"bypasses the selection bus, so `spaceLatch` never learns."* **The bus IS written** — with an **identity**
> descriptor — and `note()` gates on `kind === 'space'` **BY DESIGN** (`D4 opt-2`, the reason the latch was
> lifted at all). ⇒ ***no bus routing could ever have moved the Space latch***, and an architecture claim made
> from memory instead of from `app_client:208-210` under-scoped a runbook Joe had locked (`N-180` again).
> ✅ **`spaceLatch.latch()` is NOT a new store** — it is the literal twin of `roomLatch.latch()`, shipped at
> **Leg C-2** for the identical reason, down to the same header correction (*"SINGLE WRITER" was already false;
> `clear()` has always written too*). **"NO new store" was honoured. "NO shell change" was honoured** —
> `app_client.svelte`, `skin.css` and `dm-intro.svelte` are untouched.

1. **The member-activation path moves the SPACE context, not just the room** — 🛑 **BOTH branches: the existing
   DM and the post-create path.** ⚠️ **THIS IS NOT CREATE-SPECIFIC** — clicking an existing DM has the same
   defect today, and Joe saw it; this heading's *"after the create"* was narrower than the defect it names.
2. **R1 suppresses its highlight when the latched Space is a DM** — the test is `counterpart != null`,
   **already on the record, NO Rust, NO new field**. 🛑 **In `spaces-panel`'s `$derived` ONLY, NEVER the store**
   (`F-D`: `resolveLatched` and `canSend` both read `spacesState.spaces`, and a store-side filter makes every
   DM **unsendable**). 📌 **This is `A3`'s render-only filter in miniature** — building it here proves the seam.
3. 📌 **The draft case falls out for free and MUST NOT be special-cased:** no DM Space exists yet, so the Space
   latch does not move and **R1 stays lit on the room you are drafting from** — which is what Joe ruled.

**GATE C-bis-6** — open a DM from a member row (**both** branches, driven separately): R1 **unlit** · R2 lists
the DM's `dm` room **and highlights it** · 🛑 **R2's `selectedId` matches a row it is actually drawing** (the
dangling-highlight defect) · R5/R6/R7 unchanged · open a draft: **R1 stays lit** · floors · **`cargo` IDENTICAL**.

✅ **GATE C-bis-6 DRIVEN GREEN — CHAT, 2026-08-10 (J-711), RULE 5: every line re-driven from a FRESH client with
the node up. No figure below was reported by Clair.** Fixture: `LegF Verification`, 9 members / 8 rows.

| branch | driven | result |
|---|---|---|
| **① existing DM** | clicked `LegF-N5` (`czSW35b…`, the live counterpart of an existing DM Space) | R1 `selectedId: null` **and zero rows carrying a highlight background** · R2 `count: 1`, the one row is `dm` · **R2 `selectedId` = `c85d56…` = THE DRAWN ROW'S ID** · R5 `effectiveRoomId` = the dm room · R6 `canSend: true`, `spaceId` = the DM Space |
| **② post-create** | typed into `LegF-N2`'s draft and clicked **Send** | Spaces **6 → 7** · **R2 `latchedSpaceId` = `19948c30…`, A SPACE THAT DID NOT EXIST TEN SECONDS EARLIER** · R2 one `dm` row, `selectedId` = the drawn row · R5 `draftActive: false`, `outboundCount: 1` — promoted IN PLACE · R6 text cleared, `echoCount: 1`, `draftError: null` · R1 unlit |
| **③ draft** | clicked `LegF-Bob` (no DM) | `draftActive: true`, `draftLabel: "LegF-Bob"` · **R1 STAYS LIT on `LegF Verification`, exactly one row** |

🔑 **② IS THE PROOF OF THE LEG.** Before C-bis-6, R2 stayed on the Space you drafted FROM. It now follows a Space
that was minted mid-gesture.

🛑 **THE "R1 UNLIT" PASS IS AN EMPTY RESULT, SO IT WAS POSITIVELY CONTROLLED (`N-099`).** The probe reads the
COMPUTED background of every R1 row, not `aria-selected`. Control: clicking a normal Space makes **exactly one**
row go `rgb(42, 47, 56)`. **The probe can see a highlight; in a DM it saw none.** *Without the control this line
would read identically if the selector were simply broken.*

⚠️ **HONEST LIMIT, STATED RATHER THAN PAPERED OVER:** Chat **cannot load PNGs into its own context**, so the
painted layer was judged by **Joe alone**, from three screenshots. **It immediately earned its keep** — his
photograph settled that the drafted-to row highlight is ABSENT (C-bis-7's unbuilt work, not a defect here) and
surfaced `N-183`, neither of which any store read would have shown. ***The third layer still has no probe the
human eye does not beat.***

📌 **`cargo` IDENTICAL is a SCOPE argument here, not a measurement, and it is labelled as one:** `git diff
--name-only` returns four files, **zero `.rs`**, **zero `ui/core`**. The Rust inputs are byte-identical, so a
56-terminator rebuild would prove nothing a one-line diff does not. **svelte-check WAS re-run by Chat: 0 errors /
34 warnings / 15 files, the floor.**

🛑 **THE REGISTRY FLOOR MOVED AND IT IS NOT A REGRESSION — `164` → `166` quiescent, cause measured, see `N-184`.**
R1 now draws six Spaces where the floor was set at five; one row = `entity-item` + `entity-avatar` = **`+2`**.
**Joe's client state: `3629 B` → `4204 B`, 21:08:23**, moved by gate ② with his explicit consent.

⚠️ **VISIBLE DIFFERENCE FROM THE REJECTED OPTION B, NAMED SO IT IS NOT LATER REPORTED AS A DEFECT:** while you
are in a DM, **R2 is a one-row column showing `dm`.** Under B it would be blank. **A tells you where you are.**

🛑 **THE DEFECT, MEASURED:** after a successful create the client sits in **TWO Spaces at once** —
`spaces-panel.selectedId` and `rooms-panel.latchedSpaceId` both `…b0ffd722` (**LegF**), while
`members-panel.scope` and `roomLatch.effective*` are `…f491c1c2` (**the new DM**). ⚠️ **AND R2 HOLDS A
`selectedId` FOR A ROOM IT IS NOT DRAWING** — a highlight aimed at nothing, which is why `LegF Room` reads
unselected on screen.

🔑 **ROOT CAUSE:** `roomLatch.latch()` is called **directly**, bypassing the selection bus. Every other
navigation goes click → selection → the shell's *"ONE effect, THREE latches"*, which moves **both**. **The
create path is the only place a room is latched that the user never clicked**, so `spaceLatch` never learns.

🔒 **JOE'S RULE:** *"we will not have there no dm spaces, so i dont state that the new dm space has to be
selected by logic. rather original space will be unselected in this case."* ⇒ **R1 highlights only non-DM
Spaces; entering a DM CLEARS the highlight**, and R2 follows rather than listing a Space you are not in.
🔑 **Forward-compatible by construction: when OQ3/A3 removes DMs from R1, this rule still holds.**

### 🟡 C-bis-7 — R7 reads the counterpart from the SPACE RECORD (the DM roster bug)

🛑 **THE DEFECT, MEASURED AND RE-DRIVEN (not timing — a full re-navigation and fresh fill still gives 1):**
in the new DM, R7 reports **`isDm: true` AND `counterpart: null` in the same breath**, `memberCount 1`,
showing **only self**. Clicking the counterpart from `LegF Room` takes 8 rows → **1**.

🔑 **ONE FACT DERIVED FROM TWO SOURCES THAT DISAGREE** (`D-067`'s shape): `isDm` comes from the **Space
record**; `counterpart` comes from **the roster's non-self member** (`members-panel:129`). The roster holds
only self, so there is no row and no avatar. ⚠️ **AND THE CLIENT ALREADY KNOWS WHO IT IS** — the Space record
carries `counterpart` (verified in the store AND on disk: all four real Spaces `NULL`, both DMs set), written
at creation by **OQ8-K3, whose stated purpose was that a post-K3 DM never needs a backfill.** **R7 is ignoring
the field that exists for exactly this.**

🔒 **JOE'S RULE (A), 2026-08-09:** *"honestly i prefer only counterpart, but as we have rule that user (joe)
stays always, so there have to be user and counterpart"* — and *"draft member panel state is correct.
decision to change members list is executed with existing dm stream."*

🔒 **THE RULE, STATED SO IT IS NOT LATER "FIXED" INTO A LIE: R7 SHOWS THE PARTICIPANTS OF THE STREAM YOU ARE
LOOKING AT, WHENEVER THEY EXIST.**

| stream | R7 | why |
|---|---|---|
| group room | the room's roster | a real fill |
| **existing DM** | **self + counterpart** | ✅ Joe's ruling; the counterpart is on the Space record |
| **draft** | the group roster it was opened from | 🛑 **the DM DOES NOT EXIST YET** — there is no membership, and synthesising two rows would invent a roster no fill produced (`D-065`) |

⚠️ **The draft case is NOT an exception to the rule — it is the rule meeting a Space that has not been
created.** Written this way deliberately: the difference looks arbitrary from either side alone, and a future
reader who "harmonises" it puts an invented roster on screen.
📌 The **draft-row highlight** (`selected={counterpart}` is `undefined` in a group room, `L16`, and
`selectOnActivate={false}` deliberately stops a click from moving it) folds in here — same question: **who is
this stream about.**

🔓 **FILED, NOT SCHEDULED:** whether the node returns **invited-but-not-joined** members in a roster at all.
**The client fix stands either way** — it is not bundled with a guess about the node.

🔒 **SCOPE ADDED BY JOE 2026-08-09 — THE DRAFT ROW IS HIGHLIGHTED TOO.** ⚠️ **Chat read Joe's earlier *"draft
member panel state is correct"* as closing this; it closed the ROW LIST only.** Joe: *"i would rather have the
counterpart be selected while draft to him."* ⇒ **`selected={counterpart ?? dmDraft.counterpart}`** *(⚠️ **`D-131`: THIS LITERAL DOES NOT TYPECHECK AND IT SHIPPED LOCKED.** `dmDraft.counterpart` is `string | null`, the prop is `selected?: string`, and `entity-panel:63` states outright *"Undefined = no selection."* **Clair reported it under Rule 6 rather than absorbing it.** Shipped form: **`counterpart ?? dmDraft.counterpart ?? undefined`** — null and undefined mean the same thing to this prop, so no behaviour hides in the coercion. **Annotated, not repaired — J-713.**)* — one
expression, and it **extends `L16`** (*the only highlight is the DM counterpart*) rather than fighting it:
during a draft the stream **is** about that person. 🛑 **`selectOnActivate={false}` STAYS** — the highlight
remains DERIVED, never a click write.

**Files:** `members-panel.svelte`.

> ✅ **FILE LIST AUDITED AGAINST THE SOURCE BEFORE IT BECAME AN INSTRUCTION — CHAT, J-712.** ⚠️ **C-bis-6''s
> locked list was wrong and Chat''s kickoff repeated the error, so this one was checked rather than trusted.**
> **It holds, and here is why, so the next reader does not re-derive it:** `spacesState` is **already imported**
> by `members-panel`, so re-sourcing `counterpart` from the Space record adds no dependency · `dmDraft.counterpart`
> exists (`dm-draft.svelte.ts:76`), so the draft highlight is ONE expression plus an import · and the one thing
> that could have forced a second file does not: 🔑 **`_book` is written WHOLESALE from `get_address_book`
> (`address-book.svelte.ts:148`), which returns the BARE GLOBAL `identity_id → SeenRecord` map — it is NOT
> Space-scoped.** ⇒ **a counterpart absent from the roster can still resolve a `display_name`.**

🛑 **THREE THINGS THE IMPLEMENTER MUST NOT GET WRONG, ADDED AT J-712.**

**① `memberCount` WILL STAY `1` WHILE `rowCount` BECOMES `2`, AND THAT IS CORRECT.** `memberCount` reports the
FILL — what the node actually returned — and the fill genuinely holds one member. The second row comes from the
**Space record**, not the fill. ⚠️ **Named here so it is not later reported as a defect and "fixed" by inflating
`memberCount`**, which would make a frontend count masquerade as a wire count. **The debug state must report both
honestly; a disagreement between them is the TRUTH, not a bug.** *(Same move as C-bis-6''s "R2 becomes a one-row
`dm` column" — name the visible difference before someone files it.)*

**② THE SYNTHESISED COUNTERPART ROW MUST NEVER BE STAMPED `unresolved`.** That marker means *"reached the roster
via a LIVE membership delta"* (`M-RP-LIVEFEED-REFRESH` Leg A) and is **never present on a row that came off the
wire** by any other route. 🛑 **Reusing it would make a frontend fabrication indistinguishable from a wire fact** —
the precise `D-065` line this leg is otherwise honouring. If the row needs a marker, it needs a NEW one, and that
is a finding to report under Rule 6, not a field to borrow.

**③ RE-SOURCING `counterpart` SILENTLY RE-AIMS THE FILTER AT `:150`.** `memberRows` keeps a member that would
otherwise be hidden **iff `m.identity_id === counterpart`** (§5a E2, J-648), and `:144` states outright that
`counterpart` being `undefined` outside a DM **IS** the DM exception. In a group room the Space record''s
`counterpart` is `null` ⇒ still `undefined` ⇒ behaviour unchanged. **That is the expected answer and it must be
DRIVEN, not asserted** — a group room with a not-found member is the case that proves it.

🛑 **SCOPE ADDED MID-LEG — JOE, 2026-08-10 (J-713), OPTION A-CLOSE. FOUND BY JOE IN A SCREENSHOT, REPRODUCED BY
CHAT IN ONE GESTURE.** ⚠️ **THE FILE LIST ABOVE BECOMES `dm-draft.svelte.ts` + `members-panel.svelte`.**

**THE DEFECT, MEASURED:** open a draft to `LegF-Bob`, then click `LegF-N5` (an existing DM). Result — `roomLatch`
= N5''s dm room ✅ · `spaceLatch` = N5''s DM Space ✅ · R7 = `Joe` + `LegF-N5`, `counterpart czSW35b…` ✅ · **R5 =
`draftActive: true`, `draftLabel: "LegF-Bob"`** ❌. **Three stores say N5. One says Bob.**

🛑 **AND IT IS NOT A MISLABEL — THE INTRO SUPPRESSES THE STREAM.** `stream-panel:231` forces
`streamMessages = []` whenever `dmDraft.active`, and `:183` mounts `dm-intro` on the same condition. ⇒ **N5''s
conversation was not merely mis-captioned, it was NOT RENDERED.**

🔑 **ROOT CAUSE, AND IT BELONGS TO NEITHER C-bis-6 NOR C-bis-7.** `dm-draft.svelte.ts:100` closes the draft on a
**`room`** selection only. Member activation **never writes one** — it calls `roomLatch.latch()` directly and puts
an **IDENTITY** on the bus (`L-7`, so R8 shows the card), which `note()` correctly ignores. **The draft''s only
close trigger is a path this navigation deliberately bypasses.** That shape shipped at **C-bis-2**. ⚠️ **What
C-bis-6 and C-bis-7 changed is that the disagreement became LEGIBLE** — R2 and R7 got confident about the DM, so
R5''s stale draft finally had something to contradict. 📌 ***Joe''s "N5 has two results" was never
nondeterminism; it was a hidden second input.***

🔒 **THE FIX — CLOSE, NOT PARK, AND IT IS THE SHIPPED RULE REACHING A PATH THAT DODGED IT.** `note()`''s own doc
already states the principle: *"A `room` selection CLOSES the draft (the user navigated to a conversation); the
text map is untouched, so re-opening the draft restores its text."* **Opening N5''s DM IS navigating to a
conversation.** Park was considered and rejected: it would need R5 to render a DM''s stream while a draft is
parked, which contradicts `:231`.

🛑 **THE TRAP, AND IT IS THE WHOLE REASON THIS PARAGRAPH IS LONG: `clear()` IS THE WRONG METHOD AND IT SITS
THIRTY LINES FROM THE RIGHT ONE.** `clear():137` is the POST-SEND close — it drops the counterpart **AND deletes
that counterpart''s text**, because the text was sent. `note():100` drops the counterpart and **leaves the text
map intact**. ⇒ **calling `clear()` here would silently eat what the user typed to Bob.** *Two methods, twenty
lines apart, names that do not advertise the difference.*

⇒ 🔒 **`dm-draft.svelte.ts` GAINS `close()`** — drops `_counterpart`, **never touches `_texts`** — and **`note()`
DELEGATES to it**, so the close rule exists in ONE place. 🛑 **DO NOT synthesise a fake `room` Selection to
trigger `note()`**: inventing a selection the user never made to reach a side effect is the `D-065` line, and it
would put a fabricated entity on a bus-shaped call. **`members-panel`''s EXISTING-DM branch calls
`dmDraft.close()` alongside the two latches it already moves** — the same shape as C-bis-6, the caller doing what
the bus effect would have done.

⚠️ **THE DRAFT BRANCH IS UNTOUCHED.** Clicking a member with NO DM still OPENS a draft; only the existing-DM
branch closes one. `open()` already replaces the open counterpart, so switching drafts is unaffected.

**GATE C-bis-7** — in a DM: **self + counterpart**, driven on the `LegF-N5` DM **whose counterpart never
joined** (the case the roster cannot supply) · in a group room: unchanged roster · **in a draft: the roster is
unchanged AND the drafted-to row is highlighted** · 🆕 **the STALE-DRAFT case (J-713): open a draft to `LegF-Bob`,
then click `LegF-N5` — `draftActive` MUST go `false`, `dm-intro` MUST unmount (`aboveMountCount 0`), and N5's own
stream MUST render** · 🆕 **and the TEXT MUST SURVIVE: type into Bob's draft, navigate to the N5 DM, click Bob
again — the typed text is STILL THERE.** ⚠️ **That second one is the `clear()`-vs-`close()` trap made into a
gate; without it, eating the user's text passes.** · floors · **`cargo` IDENTICAL**.

---

### 🟡 C-bis-8 — the §5.4 failure surface becomes VISIBLE (Joe's ruling, 2026-08-09)

🔒 **RULED: one line under the composer, rendering `dmDraft.error`.**

🔑 **IT NEEDS NO WORDING DECISION AND AUTHORS NO COPY — IT RENDERS THE NODE'S OWN STRING, VERBATIM.** Driven at
J-708 that text was *"failed to connect to Node: WebSocket error … (os error 10061)"*. **That is what makes the
surface honest rather than decorative**, and it is why this is not a copy STOP. ⚠️ **A LABEL in front of it
WOULD be copy and is JOE'S** — ship without one rather than invent one.

📌 **The call site already exists and is deliberately un-rendered** (C-bis-4): `dmDraft.error` is set, cleared
on `open` and on each `create`, and read by `composer-panel`''s debug getter as `draftError`.

> ✅ **FILE LIST AUDITED AGAINST THE SOURCE BEFORE IT BECAME AN INSTRUCTION — CHAT, J-714.** **`composer-panel.svelte`
> ALONE, and it holds:** `dmDraft` is already imported (`:62`) · `dmDraft.error` is already read (`:197`) · the
> root `.composer-panel` is a **flex column**, so a line under `.composer-actions` is structurally trivial.
> ✅ **THE LIFECYCLE IS CONFIRMED AT THE SOURCE, NOT ASSUMED:** `_error` is cleared on `open` (`:92`) **AND at
> the top of EVERY `create` (`:134`)**, set on no-transport (`:136`) and on a thrown create (`:142`).
> ⇒ 🔑 **the line SELF-CLEARS on retry with no extra wiring** — which is why the *"restart the node and send:
> the line clears"* gate line is true for a REASON rather than by hope.
> ✅ **AND THE GATE IS ALREADY PROVEN DRIVABLE:** J-708''s C-bis-4 gate ② produced the real string *"failed to
> connect to Node … (os error 10061)"* with the node killed mid-draft. **C-bis-8 adds only *"and it appears on
> screen."***

🛑 **THE THING THIS SECTION DID NOT SAY, AND IT LANDS IN JOE''S RESERVED AREA — ADDED J-714.**
**THE NEW ELEMENT WILL RENDER UNSTYLED.** `skin.css` knows exactly ONE composer selector — `.composer-panel
.textarea` (`skin.css:3137`). A new line has **no skin rule**, so it arrives at browser default: default colour,
default size, no spacing. 🛑 **`skin.css` IS JOE''S FILE AND NO LEG MAY TOUCH IT.**

📌 **That is not a blocker — it is the same shape as `composer-panel`''s own `<style>` block, which says of
itself *"structural only … PROVISIONAL, discharged at `M-RP-SKIN`."*** 🔒 **STRUCTURE IS THE LEG''S; COLOUR IS
JOE''S.** ⇒ **ship structural CSS ONLY** — it wraps, it carries `min-width: 0`, it does not overflow the tile
and it does not migrate a scrollbar. **NO colour, NO font-size, NO weight.**

🔒 **CLASS NAME RULED BY JOE 2026-08-11: `.composer-error`.** *Named because JOE writes the skin rule that will
target it, so the selector is his to know in advance.* Family precedent in `skin.css`: `.send-status`,
`.subs-status`, `.subs-note`, `.uistate-note`, `.ei-status`. **`.composer-error` names the THING rather than its
position, and `error` is the only word in that family that says *something went wrong* rather than *here is some
state*.**

⚠️ **C-bis-8 THEREFORE MINTS A ROW IN `docs/ROADMAP.md` §"On screen now, and NOT a bug"** — *"the DM failure line
looks unstyled" · owner **Joe / `M-RP-SKIN`** · **it IS a bug if** it overflows the tile or fails to wrap.*
🛑 **The row is written by the commit that CLOSES C-bis-8, and deleted by the commit that closes its owner.**

**GATE C-bis-8** — kill the node, send a draft: **the line appears carrying the node's words** · the draft stays
open with its text · **nothing implies the DM exists** · restart the node and send: **the line clears** ·
floors. ⚠️ **`N-177`: re-measure the button rect immediately before the click.**
🆕 **ADDED J-714 — TWO LINES, BOTH BECAUSE A PASS HERE IS OTHERWISE INDISTINGUISHABLE FROM A DIFFERENT PASS.**
🛑 **① THE TEXT MUST BE THE NODE'S, NOT AUTHORED — assert the rendered string CONTAINS the node's own words
(`os error 10061` at J-708), not merely that SOME line appeared.** *A hard-coded "Could not create DM" would
satisfy every other line of this gate, and it would be exactly the invented copy this leg refuses.*
🛑 **② "THE LINE CLEARS" IS AN EMPTY RESULT ⇒ POSITIVELY CONTROL IT (`N-099`).** Prove the probe can SEE the
line first — read it non-empty with the node down — then restart and read it empty. *Without the control, a
selector that never matched anything passes the clear check perfectly.*

---

### 🟡 C-bis-5 (continued) — THE VERIFICATION CHECKLIST

⚠️ **THESE ITEMS BELONG TO C-bis-5, NOT TO C-bis-8** — they were separated from their heading when C-bis-6/7/8
were written above them on 2026-08-09. **The heading is restated rather than the list moved, so no line changes
owner silently** (`D-131`'s spirit).

🔑 **AND C-bis-5 NOW RUNS LAST, AFTER 6/7/8 — NOT NEXT.** Three of its items (§5.5's header behaviour, §5.6's
nameless render, `OWED-1`'s discharge) are **changed by the legs above**: the skin is still Joe's, the DM
roster changes at C-bis-7, and the orientation changes at C-bis-6. **Verifying before them would verify a
state that is about to be replaced.**

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
