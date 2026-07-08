# XGen Protocol — Message Family Phase-0 (M-RP5.5 / M-RP5.6)
> **Status**: ACTIVE  
> Version: 1.1  
> Date: Jul 2026  
> **Last updated**: 2026-07-08  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. Scope + two-component split

The messages plane is built on **sampler fixtures** — no node↔client channel needed (J-476). Two `core` components, unit-then-container:

- **`message`** (M-RP5.5) — a **dd-composite** materializing ONE `MessageDescriptor` → honest HTML (N-075). Composes `entity-avatar` / `label` / `paragraph`, status via the avatar corner-slot, plus widget sockets.
- **`message-stream`** (M-RP5.6) — a **dd-composite**, the listbox-analogue of `entity-panel`: wraps N `message`s + ordering / grouping / day-dividers / scroll.

Records honesty: J-465 closed the **entity** dd-composite sub-family. `message` + `message-stream` open a **new message dd sub-family** — not a tier reopen.

The **R5 region system-widget** wrap (register in the layout descriptor, W-12/W-13) needs the region shell (M-RP6.1) → **deferred to the M-RP6.x arc**. Building it now would be against a non-existent shell (same posture that POSTPONED `temperature-indicator`, J-470).

## 2. The family — types vs states

Same discipline as `EntityDescriptor` (where `revoked` is a flag, not a kind): most of the list are **states/fields on `text`**, not separate types.

**v1 TYPES (2):**
- `text` — avatar (both sides, reserved column) + author name + body + details. `isOwn` flips the whole row.
- `system` — authorless notice (join / leave / rename), no avatar either side, centered special-adjust line.

**v1 STATES/FIELDS on `text`:**
- `grouped` — continuation render mode, **computed by the stream** (same author within a window). Suppresses the **header line only** (name + details); **the avatar stays** (reader orientation — Joe).
- `edited` — appended "(edited)" marker.
- `deleted` — tombstone render branch (body / details dropped).

**Deferred (D-065, reserved-unfed):** reply / quote, attachment / media, reactions.

## 3. Composed-atomics map

| type / state | composed atomics |
|---|---|
| `text` full | `entity-avatar`(author, flips side) + `label`(name) + `paragraph`(body) + `details` socket |
| `text` grouped | `entity-avatar`(author) + `paragraph`(body) — header line (name + details) suppressed |
| `text` edited | + `label`("(edited)") |
| `text` deleted | `paragraph`(tombstone, italic-muted) — body / details dropped |
| `system` | `paragraph`(notice, centered) |

Notes: the author's self-**status** rides `entity-avatar`'s existing `status?` corner-slot (M-RP5.1b) — no message-level `status` wiring. **Send-status `led`** (if shown) lives **inside the `details` socket**, not a fixed atomic — keeps v1 lean and honest to the mockup.

## 4. `MessageDescriptor` — the dd-socket

Mirrors `EntityDescriptor`: source-agnostic, `core` imports no protocol type, the **shell** owns protocol → descriptor. `author` **reuses `EntityDescriptor`** (the message avatar IS an entity-avatar of the author).

```ts
export type MessageKind = 'text' | 'system';

export interface WidgetMount {
  widgetId: string;             // durable identity (J-475); unknown-id dropped on render
  props?: Record<string, unknown>;
}

export interface MessageDescriptor {
  kind: MessageKind;
  id: string;                   // event / ULID
  author?: EntityDescriptor;    // absent for system; REUSES the entity dd-socket
  body?: string;                // text-node only (never @html); deleted → ignored
  timestamp: string;            // message formats its own (like status "5m ago")
  isOwn?: boolean;              // shell-set; flips avatar side + alignment
  edited?: boolean;             // → "(edited)" marker
  deleted?: boolean;            // → tombstone render, body/details ignored
  details?: WidgetMount[];      // header/details region: time · temperature · badges · icon-buttons · send-status led
  bodyExtras?: WidgetMount[];   // below body: attachments / reactions — reserved-unfed (D-065)
  // reserved-unfed (D-065): replyTo?
}
```

Design calls (locked):
- **One socket, both kinds** — `system` = `kind:'system'` + `body`=notice, no author.
- **`author: EntityDescriptor` reuse** — avatar always rendered (even grouped); initials/xgid-tail fallback from the descriptor.
- **`isOwn` shell-set** — `core` never computes `author.id===self`; the message owns the mirror.
- **`body` text-node** — the inline-mark/link formatter stays the deferred kind-4 `use:render` (D-065).
- **Widget sockets** — `details` + `bodyExtras`, each a `WidgetMount[]`. The `message` is a **host surface** for system/custom widgets (all-widgets model, W-12): renders declared widgets, drops unknown-`widgetId` (W-13 reconcile). Fixture-testable now with empty slots.

## 5. Stream-vs-message split

Exact `entity-panel` / `entity-item` boundary:

- **`message` owns** ONE message — avatar / name / body / details / edited / deleted, its own timestamp formatting, and *accepting* a `grouped` flag. No sibling knowledge.
- **`message-stream` owns** relationships — chronological **ordering**, **grouping computation** (sets each message's `grouped`), **day-dividers**, **scroll** (auto-to-bottom / scrollback / jump-to-latest), empty state.

Principle: the stream decides *relationships between messages*; the message decides *how one message looks*.

## 6. a11y + open behaviour

- **`role="log"`** (live region) for the stream — NOT `role="listbox"` (a chat is a scrolling log, not a select-one-of list).
- **click-select** (not roving focus) — a light select feeds the R8 inspector + `entity-context-menu` selection bus (M-RP6.x). Deferred wiring; the click hook is reserved.
- **Grouping window + day-divider rule** — grouping threshold (author + time gap) and divider boundary (local-day) are M-RP5.6 build-time constants, Joe-tunable.

## 7. Sub-milestone breakdown

Following the `entity-context-menu` precedent (one milestone, Joe-gated internal steps, D-074 close at end):

**M-RP5.5 `message` dd-composite**
- Phase-0 (this doc)
- **A** — `MessageDescriptor` type + `text` full (avatar + name + body + `details` socket). Sampler DD·composite + CDP.
- **B** — states on `text`: grouped / edited / deleted. Sampler cells each + CDP.
- **C** — `system` notice + `isOwn` flip verified both sides. Closes family v1. → D-074 close.

**M-RP5.6 `message-stream` dd-composite** (entity-panel analogue)
- Phase-0 addendum (grouping algo, day-divider rule, scroll machine, `role="log"`, select hook).
- **A** — shell: `section` chrome + ordered N `message`s + empty + grouping computation + day-dividers. Fixtures.
- **B** — scroll behaviour (auto-bottom / scrollback / jump-to-latest). Sampler + CDP. → D-074 close.
- *(R5 system-widget registration → deferred to M-RP6.x region shell.)*

## 8. Open items (locked this session)

1. 2 kinds (`text` / `system`); grouped / edited / deleted = fields — **LOCKED**.
2. `author: EntityDescriptor` reuse; avatar always rendered — **LOCKED**.
3. `isOwn` shell-set; message owns the mirror — **LOCKED**.
4. `details` + `bodyExtras` widget sockets (`WidgetMount[]`); send-status led inside `details`; reactions deferred — **LOCKED**.
5. Stream = `core` dd-composite now; R5 system-widget wrap → M-RP6.x — **LOCKED**.
6. `role="log"`, click-select (not roving) — **LOCKED**.
7. Grouped = suppress header line only, avatar stays — **LOCKED**.
8. One M-RP5.5 (A/B/C), one M-RP5.6 (A/B) — **LOCKED**.

---

## 9. M-RP5.6 addendum — `message-stream` (LOCKED, v1.1)

The D-071 Phase-0 gate for the stream (opened at J-480, locked at J-481). `message-stream` = `core` dd-composite, the `entity-panel` analogue. Root `<div class="message-stream" role="log" use:envelope>` — **also the scroll viewport** (`overflow-y:auto`). Children = `message`s + interleaved day-divider rows, chronological. The stream **sets each child's `grouped` prop** (§5). `role="log"` (a scrolling live region, NOT `listbox`), **click-select** (not roving, §6).

### 9.1 Grouping (Q1 — LOCKED)
A `text` message renders `grouped` iff the previous **rendered** row is a `text` message, same `author.id`, within the window, no day-divider between.
- Window = **5 min** (build-time const, Joe-tunable).
- Breaks on: different author · any `system` message · day boundary · first row.
- `deleted` tombstone keeps its `author.id` → does **not** break a run.
- `system` messages never group (authorless, centered).

### 9.2 Day-dividers (Q2 — LOCKED)
A separator row is inserted when the local calendar day changes between consecutive rows (boundary = local midnight). It is a stream row, not a message: `<div class="day-divider" role="separator">`; it **breaks grouping**. The label **always carries the date**; the relative prefix drops once old (build-time formatter, Joe-tunable):
- today → `Today (Jul 8, 2026)`
- yesterday → `Yesterday (Jul 7, 2026)`
- 2–6 days → `Saturday (Jul 6, 2026)` (weekday + date)
- ≥7 days → `Jul 1, 2026` (date only)

### 9.3 Scroll machine (Q3 — LOCKED, full A)
Two independent behaviours on the `role="log"` viewport:
- **Stick-to-bottom** — if the user is at/near bottom (≤ **80px**, build-time const), an appended message auto-scrolls to keep the bottom in view. If scrolled up, an append does **not** yank; a **jump-to-latest pill** appears instead (click → scroll to bottom, pill hides).
- **Preserve-position-on-prepend** — loading older messages adjusts `scrollTop` by the inserted block height so the viewport stays anchored (no jump).
- Exercised on fixtures via **sampler append/prepend controls** (no live channel yet, J-476). CDP reads `atBottom` transitions + prepend `scrollTop` invariance.

### 9.4 Background / empty layer (Q6 — LOCKED, persistent + switchable)
`background?: WidgetMount[]` — a **persistent fixed layer** behind the log (`position:absolute; inset:0`; z-index below messages; does **not** scroll — messages scroll over it, chat-wallpaper style). Reuses the `WidgetMount[]` / W-13 socket → accepts a **static display object** (image/paragraph) OR a **lifecycled/reactive widget**.
- `backgroundLive?: boolean` (default `true`) — the settings switch; `false` = static mode (passed into the mount; a reactive widget renders frozen, a static object ignores it).
- Fallback: `background` unset **and** `count===0` → default composed `paragraph` ("No messages yet"). Never bare.
- Deferral (Rule 6): the `backgroundLive` settings **binding** ($common store / layout field) is M-RP6.x; M-RP5.6 exposes the prop + drives it from a sampler control (same posture as the R5-wrap deferral).

### 9.5 Select hook (Q4 — LOCKED)
Click a message → `selected` ($bindable id) + `[data-selected]` row mirror + reserved `onSelect?`. Feeds the future R8 inspector / `entity-context-menu` selection bus (M-RP6.x). **No roving tabindex** (it's a log). Wiring deferred; hook reserved now (§6).

### 9.6 Getter G (LOCKED)
`{count, selected, hasEmpty, groupedCount, dividerCount, atBottom, backgroundMountCount, backgroundLive}` — mirrors `entity-panel` plus the stream observables (`groupedCount` / `dividerCount` / `atBottom`) and the background pair, so grouping + scroll + background are CDP-readable.

### 9.7 Stream-level surface (LOCKED)
No change to `MessageDescriptor` (§4) — grouping stays a **stream-computed prop** passed down. `message-stream` props: `messages: MessageDescriptor[]` (ordered) · `background?: WidgetMount[]` · `backgroundLive?: boolean` (default `true`) · `selected?: string` ($bindable) · `onSelect?`.

### 9.8 Sub-milestone steps (LOCKED)
- **A** — shell: root `<div role="log">` + ordered N `message`s + grouping computation (sets `grouped`) + day-dividers + empty fallback + background layer render. Static fixtures. Sampler DD·composite cells + CDP. Runbook `tasks/M_RP5_6_A_MESSAGE_STREAM_SHELL.md`.
- **B** — scroll machine (stick-to-bottom + jump-pill + prepend-preserve) driven by sampler append/prepend controls. CDP (`atBottom` transitions, prepend `scrollTop` invariance). → D-074 close.
- *(R5 system-widget wrap → deferred to M-RP6.x region shell, §1.)*

### 9.9 Open items (locked this session, J-481)
1. Root `<div role="log">` = scroll viewport; children = messages + divider rows; stream sets `grouped` — **LOCKED**.
2. Grouping = 5-min same-author window, breaks on author/system/day/first, `deleted` keeps author — **LOCKED**.
3. Day-dividers = date-carrying (Today/Yesterday/weekday+date/date-only), separator row, breaks grouping — **LOCKED**.
4. Scroll = full A (stick-to-bottom + jump-pill + preserve-on-prepend), sampler-driven — **LOCKED**.
5. `background?: WidgetMount[]` persistent fixed layer + `backgroundLive?` switch (default true); unset+count0 → default paragraph — **LOCKED**.
6. Select = click → `selected` $bindable + `[data-selected]` + reserved `onSelect?`, no roving; wiring M-RP6.x — **LOCKED**.
7. Getter G eight-field; no `MessageDescriptor` change; stream-level props — **LOCKED**.
8. Two steps: A (shell) / B (scroll) — **LOCKED**.
