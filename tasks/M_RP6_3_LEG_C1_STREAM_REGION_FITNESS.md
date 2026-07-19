# M-RP6.3 Leg C1 — message-stream region fitness (`core`)
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jul 2026  
> **Last updated**: 2026-07-18  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — What this leg is, and what it is deliberately not

Leg C1 makes the shipped `core` `message-stream` **fit a region tile** and gives
it the **shape** of the live status row. It wires nothing live. It creates no
widget. It touches no shell file, no Rust, and no plugin registry.

**Everything live lands at C2** (`stream-panel` widget + projection + gap-item
feed + grace timer), verified in the real client against a real node.

The split exists because F-2/F-3/F-4 of `M_RP6_3_COMPOSER.md` §9.1 are all
`core` changes, and a registry delta that mixes a `core` change with a shell
change cannot be attributed to either (the M-RP6.1k `dialog`-footer lesson).

> **⚠️ READ `tasks/M_RP6_3_COMPOSER.md` §9 BEFORE THIS FILE.** §9 holds the
> grounding and the eight locked decisions. This runbook implements C1 only.

**Appearance is NOT specified here.** Ms Design owns every colour, glyph, type,
spacing and animation choice in Leg C. This document specifies structure,
mechanics and observability only (§5 of the parent doc, unchanged).

---

## §1 — Scope

**Files, and only these:**

| file | change |
|---|---|
| `ui/core/lib/components/data-dependent/message-stream.svelte` | height model · live `now` · render the `status` row kind · getter G additions |
| `ui/core/lib/components/data-dependent/stream/grouping.ts` | `StreamRow` gains a third variant · `computeRows` signature + grouping break |
| `ui/core/lib/components/data-dependent/stream/grouping.test.ts` | new cases (see §4) |
| `ui/sampler/…` (fixtures + panel wiring) | a sized-host fixture + a status-row fixture |
| `ui/assets/skin.css` | **only if a new rule is structurally unavoidable** — see §3.4 |

**Explicitly out of scope — do not touch:**
`message.svelte` · `data-dependent/types.ts` (`MessageDescriptor` does **not**
change, §9.7) · `$common/**` · `ui/client/**` · `ui/node/**` · any `.rs` file ·
`plugins/registry.ts` · `layout-default.ts`.

---

## §2 — Grounded anchors (verified against HEAD 2026-07-18)

Do not re-derive these; do re-check them if the file has moved under you.

- `message-stream.svelte` scoped `<style>` currently carries
  `.message-stream { position: relative; overflow-y: auto; min-height: 64px;
  max-height: 340px; }` — **that `max-height` is the F-2 box.**
- The rooting is: non-scrolling `.message-stream-shell` (position: relative) →
  `.message-stream` (the scroll viewport, `role="log"`, the envelope root) →
  `.message-stream-bg` (absolute) + `.message-stream-rows`. The jump pill is a
  **sibling** of the viewport inside the shell. **Do not change this rooting** —
  it is the J-485 structure and the registry root rides `.message-stream`.
- `const now = new Date();` is a module-scope-per-instance const, captured once
  at component init — **that is F-3.**
- `computeRows(messages, now)` returns `StreamRow[]`, currently
  `{kind:'message'…} | {kind:'divider'…}`.
- `rows`, `count`, `groupedCount`, `dividerCount`, `resolvedBg`,
  `backgroundDeclared`, `showEmpty` are all `$derived`.
- Getter G today: `{ count, selected, hasEmpty, groupedCount, dividerCount,
  atBottom, backgroundMountCount, backgroundLive }`.
- Sampler catalogue baseline: **328**. Client registry is not read this leg.

---

## §3 — The build

### §3.1 — F-2 · the height model

**Goal:** the stream fills whatever box its host gives it and scrolls inside it;
it never imposes a height of its own.

- `.message-stream-shell` → `height: 100%; min-height: 0;`
- `.message-stream` → keep `overflow-y: auto`; **delete `max-height: 340px`**;
  replace `min-height: 64px` with `min-height: 0` and add `height: 100%`.
- `min-height: 0` must ride **every** level of the chain or a flex parent
  refuses to shrink and the scrollbar migrates to the document — the J-499 D5
  finding, and the reason this is not a one-line delete.

**⚠️ The sampler consequence, and why it is not optional.** In the sampler the
stream sits in ordinary document flow, so removing the cap makes an unbounded
stream grow without limit. That is **correct component behaviour** and a
**sampler fixture problem**: the fixture must supply a sized host (a plain
wrapper with an explicit height). Do not restore a cap inside the component to
make the sampler look tidy — that is the box coming back wearing a smaller
number.

The proof that it fills a **region leaf** is a C2 leg, in the real client. C1
proves it fills **a sized host**. Say which one you proved.

### §3.2 — F-3 · `now` becomes live

`now` is read by `formatDayDivider` only, and only to choose between
`Today` / `Yesterday` / weekday / date-only.

- Replace the captured const with reactive state that advances on a timer.
- **The interval is a named constant** (D5), colocated with `GROUP_WINDOW_MS` in
  `grouping.ts`. Recommended `DIVIDER_REFRESH_MS = 60_000` — one minute is far
  finer than the one-day granularity the labels actually have, and cheap.
- The timer must be **created and torn down with the component** (`$effect`
  cleanup). A leaked interval in a component that mounts per region is a slow
  leak in a process that runs for days.
- `rows` is `$derived` off `computeRows(messages, now)`, so making `now`
  reactive is sufficient — **do not add a recompute call.**

**Honest note for the record:** this fixes *label staleness*, not *ordering*.
Nothing about message order depends on `now`.

### §3.3 — F-4 · the third `StreamRow` kind

Add to `grouping.ts`:

```
| { kind: 'status'; key: string; ... }
```

**Locked shape constraints — the payload is DATA, never rendered strings:**

- The row carries the **facts** (which phase it is in, the attempt pair, the
  remaining milliseconds, whether it has resolved and how long the gap lasted).
  It does **not** carry pre-formatted copy. Formatting and appearance are the
  component's and Ms Design's; a runbook that ships strings ships appearance.
- The **phases** are exactly the four §2/§9.7 states: counting-down · dialling ·
  resolved · exhausted. No fifth.
- **`key` must be STABLE across phase transitions** — the row *collapses in
  place* and *matures into* its own historical marker (§2). If the key changes
  on transition, Svelte destroys and recreates the node and the collapse is a
  swap, not a maturation. **This is the single easiest thing to get wrong in
  this leg.**

**Grouping:** a `status` row **breaks a grouping run** (§9.7) — it is a visible
interruption, and a continuation rendered across it reads wrong. Implement this
in `computeRows` the way a `divider` already does it, and prove it (§4).

**How the rows arrive.** `computeRows` gains a second, **optional** input for
status rows; absent input ⇒ byte-identical behaviour to today. Placement is by
timestamp, in the same single walk — **do not add a second pass**, and do not
let a status row participate in day-divider computation (it is not a message and
has no calendar day of its own that matters).

> **⚠️ C1 SHIPS THE SHAPE, FED ONLY BY A SAMPLER FIXTURE.** No `$common` store,
> no resident status read, no timer-driven countdown. An unfed branch is an
> unverified branch (N-091) — so the fixture **must** exercise all four phases,
> or the branch does not ship.

### §3.4 — skin

Prefer **zero** `skin.css` change. If the status row cannot be structurally
positioned without one, add the minimum and flag it — but note that every
appearance decision belongs to Ms Design, so a rule that sets colour, weight or
spacing is **out of scope even if it looks unfinished without it**. Unfinished
and honest beats finished and pre-empting her lane.

### §3.5 — getter G

Extend G additively. Required new fields:

- `statusRowCount` — render truth, so a dropped/unfed status row is observable.
- `statusPhase` — the current phase, or `null` when there is no status row.

Do **not** republish anything already on a child getter (N-060).

---

## §4 — Verification (sampler 9422, both accents)

CDP rules in force: single-expression `JSON.stringify` evals (PS 5.1) · DOM reads
in a **separate** eval after any mutation (N-099) · baseline only after a full
reload (N-132) · assert quiescence before counting (N-105) · a thrown eval is
**inconclusive**, not a failure (J-496) · `0/0/0` is not a pass (N-117).

| # | leg | what it proves |
|---|---|---|
| **V1** | quiescent registry + catalogue baseline after full reload | the floor these deltas are read against |
| **V2** | **fill** — mount in a sized host (e.g. 240 px and again at 640 px); measure the viewport's `clientHeight` against the host | the component takes the host's height at **two different heights** — one height proves nothing, it could be a coincidence of a hardcoded number |
| **V3** | **self-scroll** — inject content past the host height; assert the viewport scrolls and `document.scrollingElement.scrollTop` stays 0 | the scrollbar did not migrate to the document (the J-499 D5 failure mode) |
| **V4** | `max-height` is **gone** from the computed style, not merely overridden | the box was deleted, not shadowed |
| **V5** | **all four status phases** rendered from the fixture; `statusPhase` reads each | the branch is fed (N-091) |
| **V6** | **the stable key** — drive a phase transition on the *same* row and assert the DOM node is the SAME element (hold a reference across the transition) | it **matures**, it does not swap. A rendered-text check cannot tell these apart |
| **V7** | grouping break — a same-author pair inside the group window, split by a status row → the second message is **not** `grouped` | §9.7 |
| **V8** | `now` reactivity — advance the clock source and observe a divider label change **without a `messages` mutation** | F-3 actually fixed, not merely refactored |
| **V9** | regression — with no status rows supplied, `groupedCount` / `dividerCount` / `count` / scroll behaviour match the V1 baseline exactly | additive, byte-for-byte |
| **V10** | `npm test` (grouping unit suite) · `vite build` · sampler catalogue | gates |

**Unit tests (`grouping.test.ts`)** — pure, no DOM: status-row placement by
timestamp · grouping break across a status row · absent-status-input ⇒ identical
output to today · `formatDayDivider` unchanged.

**⚠️ V6 and V8 are the two legs that can silently not-run.** V6 passes trivially
if you compare rendered text instead of element identity. V8 passes trivially if
you mutate `messages` at the same time — then you have proved the recompute, not
the clock. *A field is not verified by reading it in the state where every
implementation agrees.*

---

## §5 — Gates

- `cargo test` — **unchanged**, and that is the point: it **proves** the no-Rust
  claim rather than asserting it. Floor **1541/0/62**. Run it with **both apps
  stopped** (N-117 — the dev client holds the exe; `0/0/0` is not a pass).
- `npm test` — floor **77**, must grow by the new grouping cases.
- `vite build` — floor **184** modules.
- Sampler catalogue — **328** plus exactly the new fixture cells; state the
  number you measured, never one you derived by arithmetic (N-108).
- Client registry — **not read this leg** (no shell file touched). Ground that
  **by scope** (`git show --stat`), not by re-measurement theatre.

---

## §6 — Definition of Done

- [x] `max-height` deleted; the stream fills a sized host at **two** heights and
      self-scrolls without the document scrolling
- [x] `now` advances on a named-constant timer, torn down with the component;
      a divider label changes with no `messages` mutation
- [x] `StreamRow` carries a `status` variant holding **data, not copy**; all
      four phases render from a sampler fixture
- [x] the status row's key is stable across a phase transition — proven by
      **element identity**, not by text
- [x] a status row breaks a grouping run
- [x] absent status input ⇒ output identical to today (unit + CDP)
- [x] G gains `statusRowCount` + `statusPhase`; no child field republished
- [x] zero shell files, zero `$common`, zero Rust, zero `MessageDescriptor`
      change — verified against `git show --stat`
- [x] gates measured per §5, apps stopped for `cargo test`
- [x] every appearance decision left to Ms Design; **no `skin.css` rule added**

---

## §8 — Close (J-548)

**CLOSED.** Feat `58ca561` [Clair, code-only, 4 files: `message-stream.svelte`
· `stream/grouping.ts` · `stream/grouping.test.ts` (new) ·
`app_sampler.svelte`; +386/−19].

**Chat re-drove every non-destructive leg itself (Rule 5), and every number
reproduced exactly** — registry **386/386** · fill **240→240 / 640→640**
(offsetHeight; `clientHeight` 238/638 excludes the skin's own 0.8px borders under
`box-sizing: border-box`) · `max-height: none` · viewport scrollTop 500 with
document scrollTop 0 and the document not scrollable · **V6 MATURED-IN-PLACE on
Chat's own independently-stamped probe** · **V8 labels re-derived with `count` 5
and `dividerCount` 4 held** · unmount **386→377** hook gone → remount **exactly
386** · `cargo test` **1541/0/62 across 56 terminator lines** · `npm test` **93**
· vite **184 client / 169 sampler**.

**Deviations accepted (Rule 6):** `tick` → `tickNow` (Chat's suggested name
collided with Svelte's imported `tick` — her catch, same single-writer shape) ·
a mount toggle on the status fixture (the sampler's tabs are `display:none`, so
an `{#if}` is the only way to drive the cleanup proxy) · the existing
`stream-scroll` fixture wrapped in a sized host (fixture hygiene forced by the
cap deletion; **no cap returned to the component**) · a stale top-of-file comment
still naming `max-height: 340px` swept unprompted (**N-109 applied without being
told**).

**`statusPhase` = the last-placed episode's phase, accepted with its reason
written down:** episodes are placed by timestamp and only one connection exists,
so the live episode is always the most recent — last-placed *is* current. It is a
**consequence of ordering, not an independent rule**, and `statusRowCount` plus
the per-row `data-phase` carry the full picture when history accumulates.

**Unproven by design, recorded as such:** that the `DIVIDER_REFRESH_MS` interval
actually fires at 60 s is not drivable in a CDP session. What was driven is the
**cleanup proxy** (unmount → the id leaves `__XGEN_STREAM__`), which proves the
same `$effect` teardown ran.

**The one number Chat did not isolate:** the **+49** attributed to the
`stream-fit` subtree. The **+9** for `stream-status` was measured directly as an
unmount delta; 328 + 49 + 9 = 386 is arithmetically consistent with the measured
total, and is recorded as *consistent*, not as *separately verified*.

**Next: Leg C2 — `stream-panel` widget + live projection**, authored against what
C1 actually shipped.

*(Per house rule, "commit pushed" is NOT a DoD item — the `Status: COMPLETED`
header is the real signal. Joe pushes.)*

---

## §7 — Handback

Report to Joe: the measured numbers (each one **seen**, never derived), which
legs you drove and which you could not, any deviation from this runbook flagged
rather than absorbed (Rule 6), and — explicitly — **whether V6 was proven by
element identity and V8 without a `messages` mutation.** If either was not, say
so; an inconclusive leg recorded as inconclusive is worth more than a green one
that never ran.

Chat writes the canonical records (JOURNAL / CLAUDE.md PLAY / ROADMAP / this
task doc) from that report, and authors the C2 runbook against what C1 actually
shipped — not against this document.
