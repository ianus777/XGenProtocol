# M-RP6.3 Leg C2 — `stream-panel` widget + live projection (shell)
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-19  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — What this leg is, and what it is deliberately not

Leg C2 mounts R5 as a **real region widget in the real client** and feeds it
**live**. It creates one `$common` widget, one `$common` store, and one
`CLIENT_PLUGINS` descriptor row.

**It touches ZERO `core` and ZERO Rust. That constraint is the milestone's
spine.** C1 shipped every `core` change this milestone needs — the height model,
live `now`, and the `status` row kind with its four phases. If something here
appears to need a fifth phase, a fourth row kind, a `MessageDescriptor` field or
a `resident.rs` line, **stop and surface it** (Rule 3): it is a C1b or a
successor milestone, not a quiet edit.

> **⚠️ READ FIRST, IN THIS ORDER:** `tasks/M_RP6_3_COMPOSER.md` §9 **in full**
> (findings F-1…F-6, decisions C-1…C-11) → this file → the C1 runbook §8 close
> for what actually shipped. §9 is the grounding; this file is the build.

**Appearance is NOT specified here.** Ms Design owns every colour, glyph, type,
spacing and animation choice in Leg C and Leg D. C1 left her a live
`data-phase` hook on the status row to key on. This document specifies
structure, mechanics, copy-*meaning* and observability only (parent §5,
unchanged).

---

## §1 — Scope

**Files, and only these:**

| file | change |
|---|---|
| `ui/common/lib/stores/gaps.svelte.ts` | **NEW** — connection-gap episode store: identity, clock, grace timer, phase mapping |
| `ui/common/lib/components/widgets/stream-panel.svelte` | **NEW** — the R5 region widget: room latch · projection · synthetic rows · status feed |
| `ui/common/lib/plugins/registry.ts` | **ONE** `CLIENT_PLUGINS` row (`surface: 'region'`, `regionId: 'stream'`) |
| `ui/client/src/app_client.svelte` | **only if** the `xgen-client-state-changed` / poll wiring must call the gaps store's writer — see §3.1 |

**Explicitly out of scope — do not touch:**
`ui/core/**` (any file) · `data-dependent/types.ts` · `message-stream.svelte` ·
`stream/grouping.ts` · any `.rs` file · `layout-default.ts` (**it already
derives the registry from the descriptor — F-1; adding a register line is the
defect this leg must not reintroduce**) · `ui/node/**` · `ui/sampler/**` ·
`ui/assets/skin.css`.

---

## §2 — Grounded anchors (verified against HEAD, 2026-07-19)

Do not re-derive these; do re-check them if a file has moved under you.

**What C1 shipped (`message-stream.svelte`, `stream/grouping.ts`):**

- Props: `messages?: MessageDescriptor[]` · `status?: StreamStatus[]` ·
  `background?` · `backgroundLive?` · `widgets?` · `selected?` ($bindable) ·
  `onSelect?` · `id?`.
- `StreamStatus = { id, phase, timestamp, attempt?, maxAttempts?, remainingMs?,
  resolvedAfterMs? }`. Phases: `counting-down | dialling | resolved |
  exhausted`. **Four. No fifth.**
- `status` is an **ARRAY** — supplied `timestamp`-ordered; the stream does
  **not** re-sort.
- The status row renders as unstyled inline chrome
  `<div class="stream-status" data-phase>` — not registered, no skin rule, no
  `role`.
- Status rows are placed by timestamp in the single walk and **break a grouping
  run**.
- **`showEmpty = count === 0 && !backgroundDeclared`** — this is the C-5 trap,
  see §3.4.
- `onSelect` is a **reserved hook**: the stream sets its own `$bindable
  selected` and calls `onSelect?.(mid)`. **It does NOT write the selection
  bus.** The consumer decides. *(This corrects J-547's assumption.)*
- Getter G: `{ count, selected, hasEmpty, groupedCount, dividerCount, atBottom,
  backgroundMountCount, backgroundLive, statusRowCount, statusPhase }`.
- DEV hook `window.__XGEN_STREAM__[id] = { tick }`.

**The stores this leg reads:**

- `$common/stores/ingest.svelte.ts` — `ingest.events: IngestEvent[]` (oldest
  first), `received`, `dropped`, `latest`, `push()`, `clear()`.
  `IngestEvent = { event_id?, event_type?, sender?, space_id?, room_id?,
  timestamp?, content?, prev_events? }` — protocol Events **verbatim**,
  snake_case. DEV hook `__XGEN_INGEST__`.
- `$common/stores/self-state.svelte.ts` — `selfState.connection: {state,
  label}` · `selfState.identity: SelfStateInfo | null` (`identity_id`) ·
  `selfState.resident: ResidentStatus | null` =
  `{ attempt, max_attempts, next_attempt_in_ms, terminal, connect_timeout_ms,
  ping_interval_ms }`. DEV hook `__XGEN_SELF__`.
  **⚠️ There is NO episode id and NO gap start time on this surface. §3.1
  exists because of that.**
- `$common/stores/selection.svelte.ts` — `selection.current?.entity`
  (`EntityDescriptor`), `selection.set(regionId, descriptor)`.
  **`EntityDescriptor.kind` is `'identity' | 'space' | 'room'` — there is no
  message kind, so a message selection is not expressible on the bus today.**

**The latch precedent (`rooms-panel.svelte`, D3):**

```
let latchedSpaceId = $state<string | null>(null);
$effect(() => {
  const c = selection.current;
  if (c?.entity.kind === 'space') latchedSpaceId = c.entity.id;
});
```

The effect **reads `selection.current` and WRITES the latch; it never READS the
latch** — that is what avoids the N-136 self-invalidating read-modify-write.
**Copy this shape exactly**, with `'room'`.

**The registry mechanism (F-1):** `layout-default.ts::buildWidgetRegistry`
maps every id in `REGION_IDS` to `RegionPlaceholder`, then **overrides** from
the `surface === 'region' && regionId && component` plugin rows.
`buildTitles` takes the tile title from the plugin's `name`. So **one descriptor
row replaces the `stream` placeholder and retitles the tile — there is no
`app_client` register line to add.**

---

## §3 — The build

### §3.1 — The gaps store (C-10) — episode identity, minted client-side

**The problem, grounded:** C1's `StreamStatus` needs a **stable `id` across
phase transitions** and a **retrospective `resolvedAfterMs`**. The resident
publishes neither, and never publishes a gap start time (§2). **So C2 mints
episode identity and measures episode duration itself.**

**Where it lives: a `$common` store, NOT the widget.** This **amends C-7**,
which put the grace timer in the widget. Reason, stated so it is not read as
drift: a widget-local tracker **loses all outage history when the tile is
folded, the layout changes, or the plugin is toggled**, and restarts the grace
timer on remount — so a still-live outage would blink out and return two seconds
later. A second mount (M-RP-SETTINGS' `surface: 'window'` arc) would give two
views of one connection two disagreeing stories. **C-7's actual constraint —
nothing added to `resident.rs`, nothing in `core` — is untouched.**

**Shape:**

- Module-level `$state` (the `ingest` / `self-state` precedent, a `.svelte.ts`
  module so it participates in reactivity).
- Exposes an ordered `StreamStatus[]` (oldest first) — **the exact C1 type,
  imported from `core`, not redeclared.** A second copy of that interface is a
  D-067 drift surface.
- **`GRACE_MS` and every other tunable is a NAMED CONSTANT in this file** (D5).
  Recommended `GRACE_MS = 2_000` per §9.7. Nothing may hardcode a second copy.

**🔑 THE SINGLE-WRITER RULE — the `tickNow` lesson applied forward.** The store
has **ONE** function that ingests a lifecycle+status observation, e.g.
`applyStatus(state, resident, at?)`. The reactive `$effect` calls it, and the
DEV hook exposes **that same function**. **Do not add a setter that injects a
finished episode** — a verify seam that skips the mechanism verifies the wrong
thing (J-548). Every V leg below drives the production path.

**Phase mapping — derived, not stored:**

| observation | phase |
|---|---|
| leaves `READY` | episode opens: mint `id`, record `timestamp` = now (the gap start) |
| `resident.next_attempt_in_ms != null` | `counting-down` (carry `attempt` / `maxAttempts` / `remainingMs`) |
| `next_attempt_in_ms == null`, not terminal | `dialling` |
| `resident.terminal === true` | `exhausted` |
| returns to `READY` | `resolved`, `resolvedAfterMs = now − timestamp` |

`id` is minted **once per episode** and never changes — it is what
`StreamRow.key` derives from, and C1 proved by element identity that a changed
key turns the collapse into a swap. A monotonic counter plus the start epoch is
sufficient; **do not derive it from phase or from any field that mutates.**

**The grace window:** on leaving `READY` the episode is **pending** — it exists
internally but is **NOT** in the published array. A `GRACE_MS` timer runs; if it
fires, the episode is published. **If `READY` returns first, the pending episode
is discarded entirely** and nothing was ever shown. *Silence is the correct UI
for a blip.*

**Resolved episodes stay in the array** — that is C1's "the live widget IS the
historical marker, matured", and §2's driven `statusRowCount: 2`.

**Honest limits, to be written into the close, not discovered at verify:**

1. **Episode START is event-timed** (the lifecycle listen), so `timestamp` and
   `resolvedAfterMs` are accurate. **The countdown NUMBERS are poll-sampled at
   the existing 2 s interval**, so `remainingMs` / `attempt` can be up to ~2 s
   stale. Say which is which; do not present a poll-sampled number as live.
2. **A frozen peer produces NO lifecycle transition at all** (§0 G-4 as
   amended — measured 27 s at `READY`). **No transition ⇒ no episode ⇒ no
   row.** Leg C may not paper over this. The row describes a **broken socket**,
   never "the node is down".
3. **`terminal` is durable only while the app is ignored** — any window `focus`
   resumes a parked resident. The `exhausted` phase may not be drawn or worded
   as permanent.
4. **Episode history is session-scoped and in-memory.** A client restart clears
   it. That is correct and is not a persistence gap to fill here.

**DEV hook:** `window.__XGEN_GAPS__` exposing the store (the `__XGEN_INGEST__` /
`__XGEN_SELF__` pattern, N-024), DCE'd in production.

**Wiring:** the store observes `selfState` reactively if it can do so without a
shell edit; if a shell call is required, it is **one call** added beside the
existing `setResident` poll in `app_client.svelte` — **no new emit channel, no
new invoke, no new poll** (D1/D-067). State which you did.

### §3.2 — The descriptor row (F-1)

**ONE** row appended to `CLIENT_PLUGINS`:

- `id: 'stream-panel'` · `name` (this becomes the **tile title**, replacing
  `REGION_NAMES['stream']` = "R5 · Message stream" — pick a user-facing name,
  the `Rooms` / `Spaces` precedent) · `description` · `version: '1.0.0'` ·
  `kind: 'system'` · `host: 'client'` · `delivery: 'compiled'` ·
  `surface: 'region'` · `regionId: 'stream'` · `component: StreamPanel`.
- **`icon`: LEAVE UNSET** unless a verified in-repo glyph exists. The
  `spaces-panel` / `rooms-panel` rows document exactly this: a Material `d` path
  is **not** fabricated from memory (D-108). `plugin-list` falls back to its
  documented placeholder.
- **`settingsComponent`: UNSET** — this plugin has no settings this leg, and an
  unset field is the honest greyed-button reason.

**That is the whole registration.** `buildWidgetRegistry` picks it up;
`buildTitles` retitles the tile; `plugin-list` gains a row for free. **Do not
add anything to `layout-default.ts`.**

### §3.3 — The room latch (C-5, F-6)

Copy the `rooms-panel` D3 shape with `'room'` (§2). Then:

- **`onSelect` is NOT wired to `selection.set`.** Nothing this widget does moves
  the bus off a room, so the N-136 trap is not merely avoided — it is
  **unreachable**. A message selection is not expressible on the bus anyway
  (§2). Filed, not built.
- `selected` (the `$bindable`) may be used for local click-highlight only.
- **Filter by `room_id` alone.** Room ids are hash-derived `xgen://` globals;
  `space_id` is redundant. Do not add a space latch this leg.
- **Stale-latch guard** (N-095 spirit): a latched room id that no longer
  resolves must fall back to the "select a room" state, never throw.

### §3.4 — The projection (C-2, C-4) and the two synthetic rows (C-9, C-5)

**Project on read. NEVER a mirror store** (C-4): derive the array off
`ingest.events`, filtered by the latched room, mapped through the §9.3
allowlist. **No append API, no second store, no reconciliation** — grouping,
dividers and the scroll machine all recompute free.

**The allowlist is EXPLICIT with a `default: ignore` arm** — reproduce §9.3
exactly:

| event type | → |
|---|---|
| `message.text` | `kind:'text'` · `id = event_id` · `body = content.text` · `timestamp` · `isOwn = sender === selfState.identity.identity_id` |
| `membership.join` · `leave` · `kick` · `ban` · `node_eject` | `kind:'system'` centred notice |
| `message.redact` | **not a row** — mutates the referenced descriptor's `deleted` |
| `message.file` · `message.reaction` | ignore (`bodyExtras` / `details`, reserved-unfed) |
| everything else, incl. `Unknown` | ignore, silently and by design |

**Author (C-8):** `author = { kind: 'identity', id: sender }` with **no
`name`** — nothing in the client resolves an XGID to a display name.
`entity-avatar` falls back to xgid-tail initials. **Do not fabricate a name
map** (J-501).

**🔑 C-9 — THE HEAD MARKER IS A SYNTHETIC `kind:'system'` DESCRIPTOR, PREPENDED
BY THIS WIDGET. ZERO `core`.** `MessageKind` is `'text' | 'system'` and `system`
is an authorless centred notice — the same render §9.3 already routes
`membership.*` through. It is **not** a fifth `status` phase and **not** a
fourth `StreamRow` kind.

- Always present when a room is latched; always first.
- **Two states** (C-6): normal — *the view begins at session start*; and
  `ingest.dropped > 0` — *and part of this session was discarded* (F-5).
- `id` uses a **reserved prefix** (e.g. `__head__`) so it can never collide with
  an `event_id`, and so `onSelect` can filter it.
- `timestamp` = session start. **A session crossing midnight will put a day
  divider between the marker and the first message. That is the truthful
  render, not a bug.**
- It is **deleted, not softened**, by M-RP6.4 — its removal is written into that
  milestone's DoD (N-109), not left to be noticed.

**⚠️ C-5 AMENDED — THE MARKER MAKES `core`'s EMPTY STATE UNREACHABLE.**
`showEmpty` requires `count === 0`; a permanently-present marker makes
`count ≥ 1` forever, so *"No messages yet"* **can never fire again from this
widget**. Therefore:

| truth | what renders |
|---|---|
| no room latched | the widget's own **"select a room"** state — the stream is fed **nothing** (no marker; there is no room to mark) |
| room latched, zero projected | marker **+ a SECOND synthetic `system` row** carrying the "no messages" meaning |
| room latched, messages | marker, then the projected rows |

Two honest empty states, distinct copy for distinct truths (N-091) — now
composed **by the widget**, one level up from where C1 put them. Its *wording
and appearance* remain Ms Design's; its *existence and meaning* are specified
here.

### §3.5 — The status feed (C-7, C-11)

Pass the gaps store's array straight into C1's `status` prop. **Do not
re-order, do not re-map, do not format.** C1 places by timestamp and breaks
grouping runs; both are already proven.

**🔑 C-11 — THE RESOLVED ROW MARKS A DISCONTINUITY, NOT AN ALL-CLEAR.** There
is no backfill, so messages other people sent during the gap are **gone and
nothing will fill the hole**. A resolved row that means only *connection
restored · 8s* is true and **incomplete** — the same D6 rule as the head
marker (*the stream may never look complete when it isn't*), applied to the
middle of the stream instead of the top. `resolvedAfterMs` already ships and
the row already renders in the right position, so this costs nothing but the
decision.

**Its wording and appearance are Ms Design's.** What is locked here is the
**meaning**: the resolved row asserts *the record is discontinuous here*.

**Filed, NOT built here — per-message send state.** "not delivered yet" /
pending / failed indicators belong on **outbound** rows, and there are none in
C2: every projected row is inbound (delivered by definition), and the node
excludes the author from fan-out so your own sends never ingest.
`MessageDescriptor.details` is the reserved socket (its own comment names
"send-status led"), and Leg B already ships the four-way `SendOutcome` behind
it. **→ Leg D.**

---

## §4 — Verification (real client 9222 + live node)

CDP rules in force: single-expression `JSON.stringify` evals, PS 5.1 (use a
**named function expression**, not an arrow IIFE) · single-quoted PowerShell
string so JS double quotes survive · DOM reads in a **separate** eval after any
mutation (N-099) · baseline only after a **full reload** (N-132) · assert
quiescence before counting (N-105) · a thrown eval is **inconclusive**, not a
failure (J-496) · `0/0/0` is not a pass · **N-117: the dev client HOLDS THE EXE
— stop both apps before any `cargo run`.**

> **⚠️ BINDING MEASUREMENT PRECONDITION — `count` IS NO LONGER THE MESSAGE
> COUNT.** G's `count` is `messages.length`, and this widget prepends synthetic
> rows. **A latched empty room reads `count: 2`, not `0`.** Subtract the
> synthetic rows, or a working empty room reads as two phantom messages. This
> is the J-548 hidden-element family: *a number that is right about the wrong
> quantity*.

> **⚠️ BINDING — DRIVE A **ROOM**-LEVEL `membership.join`, NEVER A SPACE-LEVEL
> ONE (C-3).** `state_key.rs:252–253` builds a Space join with `room_id = ""`
> and a Room join with a real `room_id`; `derive.rs:1026–1030` emits the pair.
> A room-scoped filter **correctly** excludes the Space join, and Clair's first
> live ingest at Leg B was exactly that class. Drive a Space join, see an empty
> stream, and you will record a working seam as dead. **Confirm the exact join
> verb with `xgen-client.exe --help` before driving it — do not fabricate a
> command line.**

| # | leg | what it proves |
|---|---|---|
| **V1** | quiescent client registry baseline after a **full reload** | the floor the widget delta is read against |
| **V2** | the `stream` tile renders the panel, not `RegionPlaceholder`; the tile **title** is the plugin's `name`; `plugin-list` shows the new row | F-1 — one descriptor, three readers, **zero** register lines |
| **V3** | **region fill** — the panel's stream `offsetHeight` tracks its **region leaf** at two different tile sizes (resize a splitter between reads) | C1 proved fill against a **sized host**; fill against a **region leaf has never been driven**. Two sizes because one could be a coincidence |
| **V4** | no room latched → "select a room"; latch a room with nothing projected → **marker + "no messages"**, `count` = 2 synthetic | C-5 as amended; both empty states fed (N-091) |
| **V5** | **live text projection** — `bob` sends to the latched room → the row appears; `isOwn` false; author renders **initials only** | C-2 / C-8, the primary seam |
| **V6** | **room-scoped filter** — an event for a *different* room does not render, **stays in `ingest`**, and appears on switching to that room | C-5; proves filtered ≠ dropped |
| **V7** | **ROOM-level `membership.join`** → a `system` centred notice row | C-3, the phantom-defect trap |
| **V8** | **head marker `dropped` state** — force `ingest.dropped > 0` and assert the marker moves to state 2 **without changing its `id`** | C-6/C-9; the marker mutates in place |
| **V9** | **grace, negative leg** — one **single-expression** eval calling the store's own writer down-then-up synchronously; assert **no episode is published** and `statusRowCount` is unchanged | the blip is never shown. *Synchronous by construction, so it cannot straddle `GRACE_MS`* |
| **V10** | **grace, positive leg** — drive down via the store's own writer, sleep > `GRACE_MS` in PowerShell, read in a **separate** eval → exactly **one** episode, phase per §3.1 | the timer fires and publishes |
| **V11** | **a REAL outage** — stop the node, watch `counting-down` → `dialling`, restart, watch it **collapse in place to `resolved`**; assert the `data-phase` element is the **SAME node** across the transition | the production path end-to-end, and C1's maturation under a real feed. **Element identity, never rendered text** |
| **V12** | **episode survives an unmount** — with a resolved episode present, fold/unfold the tile (or toggle the plugin) and assert the episode is **still there** with the same `id` | C-10, the whole reason the store is not in the widget |
| **V13** | grouping break — a same-author pair inside the group window split by a published episode → the second message is **not** `grouped` | C1's break under a live feed |
| **V14** | gates per §5 | — |

**⚠️ THE LEGS THAT CAN SILENTLY NOT-RUN.** **V11** passes trivially if you
compare rendered text instead of element identity — hold a reference or stamp
your own marker on the node, as C1's V6 was driven. **V9** passes trivially if
the two writer calls are in **separate** evals: the PS round-trip may exceed
`GRACE_MS` and you will have proved a timeout, not a discard — **it must be one
synchronous expression**. **V3** passes trivially against a fixed-size tile;
resize between reads. **V5** is not proven by your own send — the node excludes
the author from fan-out (`fanout.rs:301`), so **drive it from `bob`**; zero
would be correct there too (J-546).

---

## §5 — Gates

- `cargo test` — **unchanged**, floor **1541/0/62 across 56 terminator lines**.
  Identical is the **proof** of the zero-Rust claim, not an assertion of it.
  Run with **both apps stopped** (N-117).
- `npm test` — floor **93**. New pure logic (the projection map, the phase
  mapping) **should** grow it; if it does not, say why rather than leaving the
  number unexplained.
- `vite build` — floor **184 client / 169 sampler**. **Sampler must not move** —
  nothing in `ui/sampler/**` is touched.
- **Sampler catalogue — 386, unchanged.** No `core` change, no sampler fixture.
- **Client registry** — the only delta this leg is allowed to produce. Measure
  it after a full reload; state the measured number and, per N-108, say plainly
  which components of it were **seen** and which were **derived by arithmetic**.
- `git show --stat` — confirm zero `core`, zero `.rs`, zero `skin.css`, zero
  `ui/sampler/**`, zero `layout-default.ts`.

---

## §6 — Definition of Done

- [ ] `gaps.svelte.ts` mints stable episode ids and measures `resolvedAfterMs`;
      **one writer**, and the DEV hook exposes **that same function**
- [ ] the grace window discards a blip (V9) and publishes a real gap (V10)
- [ ] episode history **survives an unmount** with ids intact (V12)
- [ ] one `CLIENT_PLUGINS` row mounts R5 at `regionId: 'stream'`, retitles the
      tile, and lists in `plugin-list` — **no register line, no
      `layout-default.ts` edit**
- [ ] the room latch copies the D3 shape; `onSelect` is **not** wired to the bus
- [ ] the §9.3 allowlist is explicit with a `default: ignore` arm; a
      **ROOM-level** `membership.join` renders a system notice
- [ ] the head marker renders as a synthetic `system` descriptor with a stable
      reserved id and **both** states
- [ ] both empty states render distinct copy, widget-composed (C-5 amended)
- [ ] the stream fills a **region leaf** at two tile sizes and self-scrolls
- [ ] a real outage produces a row that **matures in place** to `resolved`,
      proven by **element identity**
- [ ] the resolved row's meaning is recorded as a **discontinuity** (C-11)
- [ ] **zero `core`, zero Rust, zero sampler, zero skin** — verified against
      `git show --stat`
- [ ] gates measured per §5, apps stopped for `cargo test`
- [ ] every appearance decision left to Ms Design; **no `skin.css` rule added**

*(Per house rule, "commit pushed" is NOT a DoD item — the `Status: COMPLETED`
header is the real signal. Joe pushes.)*

---

## §7 — Handback

Report to Joe: every measured number (each one **seen**, never derived — and
where a number *was* derived, say so, N-108), which legs you drove and which you
could not, and any deviation from this runbook **flagged rather than absorbed**
(Rule 6).

Explicitly, in the report, answer these four:

1. Was **V11** proven by **element identity**, or by rendered text?
2. Was **V9** driven in **one synchronous expression**?
3. Was **V7** a **ROOM**-level join, and how did you confirm the verb?
4. Did you subtract the **synthetic rows** from `count` everywhere you quoted it?

If any was not, say so. **An inconclusive leg recorded as inconclusive is worth
more than a green one that never ran.** And if anything in this runbook is
internally contradictory — read it whole before starting; the last two legs of
this arc were both saved that way (J-499, J-548).

Chat writes the canonical records (JOURNAL / CLAUDE.md PLAY / ROADMAP / this
task doc) from that report.
