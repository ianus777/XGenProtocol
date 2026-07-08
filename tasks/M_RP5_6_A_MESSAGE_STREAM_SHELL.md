# XGen Protocol — M-RP5.6 A Runbook: `message-stream` shell (Clair)
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-08  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. Read-order (Rule 0) + spec of record

Before touching code: CLAUDE.md PLAY → JOURNAL J-481 → this runbook. **Spec of record = `docs/xgen-dd-message-family-phase0.md` v1.1 §9** (the locked M-RP5.6 addendum). This runbook is the build sheet for **step A only**; if it and §9 ever disagree, §9 wins — stop and flag (Rule 6).

## 1. Scope — A (shell), not B (scroll)

**A builds:** the `message-stream` component + grouping computation + day-dividers + empty fallback + background layer, on **sampler fixtures**. **B (later)** = the scroll machine (stick-to-bottom / jump-pill / prepend-preserve). Do **not** build scroll behaviour in A — but the root **is** the scroll viewport (`overflow-y:auto`), so B has a home. No node↔client channel (fixtures only, J-476).

## 2. Component

- **New file:** `ui/core/lib/components/data-dependent/message-stream.svelte` (dd-composite, sibling of `message.svelte`).
- **Root:** `<div class="message-stream" role="log" use:envelope>` — also the scroll viewport (`overflow-y:auto`). Children = `message`s + interleaved day-divider rows, in the given order.
- **Pure helpers colocated** (unit-testable, the `transform.ts`/`clamp.ts` precedent): `ui/core/lib/components/data-dependent/stream/grouping.ts` (grouping + divider computation) + `formatDayDivider`. `core` stays protocol-free.

### Props
```ts
messages: MessageDescriptor[];   // ordered (chronological); the stream does NOT re-sort — consumer supplies order
background?: WidgetMount[];       // persistent fixed layer (see §5)
backgroundLive?: boolean;         // default true; settings switch (binding deferred to M-RP6.x)
selected?: string;                // $bindable id (see §6)
onSelect?: (id: string) => void;  // reserved
id: string;                       // envelope id
```
No change to `MessageDescriptor` (Phase-0 §4). `grouped` is passed DOWN to each `message` (stream-computed, §3), never read from the descriptor.

### Getter G
```ts
() => ({ count, selected, hasEmpty, groupedCount, dividerCount, atBottom, backgroundMountCount, backgroundLive })
```
- `count` = messages.length · `hasEmpty` = count===0 · `groupedCount` = # children rendered `grouped` · `dividerCount` = # divider rows · `backgroundMountCount` = # rendered background widgets (post W-13 drop) · `atBottom` = scroll observable (in A, initialize to `true`; B drives it live) · `backgroundLive` = the prop.

## 3. Grouping (Phase-0 §9.1)

Compute per render, walking `messages` in order. A `text` message is `grouped` **iff** the previous **rendered** row is a `text` message with the same `author.id`, timestamps within **5 min** (`GROUP_WINDOW_MS = 5*60*1000`, a build-time const, Joe-tunable), and **no day-divider was inserted between them**.

Breaks (→ `grouped=false`): different `author.id` · any `system` message (authorless) · a day boundary crossed (a divider sits between) · first row. `deleted` keeps its `author.id` → does **not** break a run. `system` messages never group.

Pass the computed boolean as the child `message`'s `grouped` prop.

## 4. Day-dividers (Phase-0 §9.2)

Between consecutive rows, insert a divider when the **local calendar day** changes (boundary = local midnight; compare `toDateString()` of the two timestamps). A divider is a stream row, not a message: `<div class="day-divider" role="separator">{label}</div>`. A divider **breaks grouping** (the row after it is never `grouped`).

`formatDayDivider(ts: Date, now: Date): string` — label always carries the date:
- today → `Today (Jul 8, 2026)`
- yesterday → `Yesterday (Jul 7, 2026)`
- 2–6 days ago (same week window) → `Saturday (Jul 6, 2026)` (weekday + date)
- ≥7 days → `Jul 1, 2026` (date only)

Use `Intl.DateTimeFormat` for the `Mon D, YYYY` + weekday parts (DOM-free, matches `converter-field`'s `Intl` precedent). Build-time formatter, Joe-tunable.

## 5. Background + empty (Phase-0 §9.4)

- `background?: WidgetMount[]` → a **persistent fixed layer**: `<div class="message-stream-bg">` absolutely positioned (`position:absolute; inset:0`, z-index below the message rows), **does NOT scroll** (siblings of the scrolled row-list, not inside it — messages scroll over it, wallpaper style). Render each `WidgetMount` through the consumer widgets registry, **drop unknown `widgetId`** (W-13, same as `message.details`).
- `backgroundLive` (default `true`) is passed into each mount (a reactive widget renders frozen when `false`; a static object ignores it). The settings **binding** is M-RP6.x — in A just expose the prop + a sampler control.
- **Fallback:** `background` unset **and** `count===0` → render a default composed `paragraph` ("No messages yet"), centered. Never bare. (If `background` is set, it shows through when empty — no separate empty paragraph.)

## 6. Select hook (Phase-0 §9.5)

Click a message row → set `selected` (=that message id, `$bindable`) + mirror `[data-selected]` on the row + call `onSelect?.(id)`. **No roving tabindex** (it's a log, not a listbox). No live selection-bus wiring in A (that's M-RP6.x) — just the hook.

## 7. Skin

Structural rules (grid/scroll/layer stacking) live in the component `<style>`; **all appearance** goes in `ui/assets/skin.css` (`.message-stream`, `.day-divider`, `.message-stream-bg`, `[data-selected]`). Accent-neutral where possible; if any accent is used it must swap gold↔blue. Divider = muted `--t3`/`--fs-1`, centered, a hairline rule is fine. Keep new tokens to zero if achievable.

## 8. Sampler fixtures + cells (DD·composite tab)

Add fixtures to the sampler DD·composite panel. Reuse existing `message` fixtures; add ordered arrays that exercise:
- **`stream-basic`** — a few `text` from 2+ authors + one `system`, same day → proves grouping runs (consecutive same-author collapse) + system breaks a run.
- **`stream-days`** — messages spanning ≥3 local days incl. today + yesterday + an ≥7-day-old one → proves all four divider label bands + divider-breaks-grouping.
- **`stream-empty`** — `messages=[]`, no `background` → default paragraph.
- **`stream-bg`** — `messages=[]` + `background=[{widgetId:'<a fixture display widget>'}]` + a `stream-bg-unknown` variant with an unknown `widgetId` (drop proof, W-13); a sampler toggle drives `backgroundLive`.

## 9. CDP verification (D-097, sampler 9422 — real output, Rule 2)

Harness: kill stale `node`/`cargo`/`xgen-sampler` + free 5175/9422 → `.\run-sampler.ps1 -Debug` (poll 5175 then 9422) → `.\cdp-debug.ps1 -App sampler -Mode eval -Expression "..."`. Single-line eval expressions only. Iterate `article.message` / `div.day-divider`-style rather than quoting `[data-debug-id="…"]` selectors.

Must show:
1. `ids().length` new total, `count===unique`, **0 orphans both directions** (`message-stream#…` roots + their child `message#…` + each message's `__avatar`/`__name`/`__body`).
2. Grouping: `stream-basic` getter `groupedCount` matches the fixture; the DOM confirms grouped rows drop `.msg-header` (via child `message`).
3. Dividers: `stream-days` `dividerCount` matches; each `.day-divider` text matches the four label bands; the row after each divider is `grouped=false`.
4. Empty: `stream-empty` `hasEmpty:true` + a rendered default paragraph.
5. Background: `stream-bg` `backgroundMountCount` = 1 (known) vs 0 (unknown-widget drop, W-13); `.message-stream-bg` present, `position:absolute`, does not scroll; `backgroundLive` toggles.
6. Select: click a row → `selected` updates + `[data-selected]` mirrors.
7. Both accents (`--accent2` `#c28840` ↔ `#3a7ab0` via skin-swap); divider `--t3` identical client↔node.
8. `vite build` clean; screenshot to `temp/`.

## 10. Definition of Done (Rule 7 — verify each with real output)

- [ ] `message-stream.svelte` + `stream/grouping.ts` created (`Filesystem:*`, verified via `get_file_info`).
- [ ] Root `<div role="log">` = scroll viewport; children = messages + divider rows in order.
- [ ] Grouping computed + passed down; `groupedCount` correct on `stream-basic` (CDP-quoted).
- [ ] Day-dividers inserted on local-day change; all four label bands verified on `stream-days` (CDP-quoted); divider breaks grouping.
- [ ] `background` persistent fixed layer + W-13 unknown-drop; `backgroundLive` toggles; empty fallback paragraph.
- [ ] Select hook: `selected` $bindable + `[data-selected]` + `onSelect?`.
- [ ] Getter G eight-field, CDP-readable.
- [ ] Registry `count===unique`, **0 orphans both directions**, both accents (CDP-quoted).
- [ ] `vite build` clean + screenshot.

*(DoD never includes "commit pushed" — the `Status: COMPLETED` header is the close signal.)*

## 11. Close (D-074 two-commit)

Clair feat commit first, then Chat Claude doc-bridge (JOURNAL J-482 + registry `xgen-ui-components.md` + ROADMAP M-RP5.6 A ✅ + CLAUDE.md PLAY + this runbook → COMPLETED). Joe pushes both. Then **B** (scroll machine) opens.
