# M-RP2.20 — `led` (di·A, atomic `<span class="led">`, simple display-di; caller-supplied state→colour map)

> **Status**: COMPLETED  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-29  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. What this is

Author + skin `led` — the **fifteenth** `core` component and the **fourth simple display-di** (after
label/paragraph/image, N-032). Built, tuned, CDP-verified **in the sampler** (D-097). An atomic inline
**`<span class="led">`** status light. The headline is the **caller-supplied colour map**: the atomic
carries a `states: Record<string,string>` map it does **not** interpret (the `select` options-prop
precedent, N-034) — fully data-independent. Also the **first component whose colour is caller-supplied,
not accent-derived** — the skin-swap re-themes shell chrome but the dots keep their mapped colours.

**Joe-locked at the catalogue concept-lock (N-049, 2026-06-29):**

1. **Simple display-di**, atomic inline `<span class="led">` (no native status-light element; `<span>`
   is the neutral inline root, chosen for composite use beside a label; `<output>` avoided — reserved
   for the deferred dd progress/meter/output primitives).
2. **`states: Record<string,string>`** — the caller-supplied map (e.g. `{ "ON":"#ff0000",
   "OFF":"var(--t4)" }`); values accept **hex OR `var(--token)`** (consumer hardcodes or rides the skin
   tokens; the atomic stays colour-agnostic).
3. **`state: string`** — the current key; selects which colour shows.
4. **`pulse?: boolean`** — optional animation, **orthogonal to colour**.
5. **Resolve `colour = states[state] ?? "#000000"`** — **full black `#000000` is the reserved
   unknown/undefined sentinel** (always-visible solid; a transparent dot would disappear). **Consumers
   must never map a real state to `#000000`** — the contract lands on the caller (written into the
   `.svelte` header so it is not a silent trap).
6. **`title = state ?? "?"`** — native hover tooltip shows the live key; a *set-but-unmapped* key still
   shows in the tooltip (diagnostic), only *truly-undefined* shows `"?"`.
7. **Getter `{ state, colour }`** (the resolved pair). **`role="img"` + `aria-label={title}`** (colour
   is not the only signal — a standalone `led` stays accessible).
8. **`.led` skin owns shape only** (size, `border-radius:50%`, pulse `@keyframes`); **colour rides an
   inline CSS var** set from the prop — the skin never hardcodes a state colour.

**Milestone M-RP2.20**, sampler cells `led#default` + `led#off` + `led#pulse` + `led#unknown`.

---

## 1. Why this shape (for the N-entry)

`led` is the first **simple display-di since the trio** — it inherits the read-only display-di pattern
(N-035: plain non-`$bindable` props, getter exposes the value, verify = render + computed-style, no
event to dispatch) but adds two firsts:

- **Caller-supplied map (the N-034 shape applied to a display-di).** Like `select`'s `options`, the
  atomic carries content (`states`) it does not interpret — keeping it fully data-independent while the
  consumer (or, later, the dd layer) supplies the state→colour semantics. The shells' bespoke
  `.state-dot` + `dotColor(state)` switch becomes *this*, generalised.
- **Colour is data, not skin.** Every prior component's colour came from the skin (`--accent*`, `--t*`,
  `--err`). `led`'s colour comes from the **prop**, injected as an **inline CSS custom property**
  (`--led-colour`) the `.led` skin reads. The skin owns only *shape*. This is the clean L2 split for a
  data-coloured atomic, and the precedent the `status-indicator` composite will build on.

The `#000000` sentinel + the `title` fallback are the **contract surface**: an unmapped/undefined state
never vanishes (black dot) and is always diagnosable (the key, or `"?"`, in the tooltip).

---

## 2. Phase-0 references (read before authoring)

- `ui/core/lib/components/data-independent/label.svelte` — the read-only display-di pattern (plain
  prop, `$state.snapshot` getter, `use:envelope`, no `$bindable`, no hardcoded class).
- `ui/core/lib/components/data-independent/select.svelte` — the caller-supplied map precedent (N-034):
  a prop carrying content the atomic does not interpret.
- `ui/core/lib/components/data-independent/toggle.svelte` + `ui/assets/skin.css` `.toggle[role="switch"]`
  — the **attribute-as-skin-hook** precedent (the skin keys an attribute, not a second class, since
  `envelope` owns `class`); `led`'s `pulse` reuses this via a reflected attribute.
- `ui/assets/skin.css` — `.label`/`.image` (display-di skin shape) + the `--t*`/`--err` tokens a
  consumer map might reference via `var(--token)`.
- `ui/docs/xgen-ui-notes.md` N-032 (display-di identities), N-034 (options-prop), N-049 (the `led`
  concept-lock this runbook executes).
- `DECISIONS.md` D-096 (own-atomics) — `led` is a new simple display-di, no fold question; applies
  D-096, no amendment.

---

## 3. Component spec — `ui/core/lib/components/data-independent/led.svelte`

Root IS `<span class="led">` (the type-class via `envelope`, not hardcoded). Zero local `<style>`.

**Props:**

| prop | type | default | note |
|---|---|---|---|
| `states` | `Record<string,string>` | `{}` | caller-supplied state→colour map (hex or `var(--token)`); the atomic does not interpret it (N-034) |
| `state` | `string` | `undefined` | current key; plain prop (display-di, **not** `$bindable`) |
| `pulse` | `boolean` | `false` | optional animation, orthogonal to colour |
| `id` | `string` | — | |

- **Derived:** `colour = $derived(states[state] ?? '#000000')` (the black sentinel); `title = $derived(state ?? '?')`.
- **Getter:** `const debug = () => ({ state: state ?? null, colour });` — a plain serialisable pair
  (the resolved `colour` may be a `var(--token)` string, not a computed rgb — that is correct; the
  computed colour is a skin/computed-style concern verified separately).
- **Markup:**
  ```svelte
  <span
    use:envelope={{ name: 'led', id, debug }}
    role="img"
    aria-label={title}
    {title}
    data-pulse={pulse || undefined}
    style="--led-colour: {colour}"
  ></span>
  ```
  (`data-pulse` present only when `true` — the toggle `role="switch"` attribute-hook precedent;
  `--led-colour` carries the resolved value, including a `var(--token)` reference, which resolves when
  the skin reads it.)
- **No processor seam, no `$bindable`, no `value`** (display-di, read-only).
- **Header MUST carry the contract:** consumers must never map a real state to `#000000` (the sentinel).

---

## 4. Skin spec — add `.led` to `ui/assets/skin.css`

Own `.led` key. Place after `.image` (the display-di neighbour), before the existing layout/section
blocks if any. **Shape only — colour rides the inline var. Baseline; Joe live-tunes via HMR.**

- `.led` — `display: inline-block`, fixed dot size (PROVISIONAL ~`10px` square), `border-radius: 50%`,
  `background: var(--led-colour, #000000)` (the inline prop var; `#000000` fallback doubles the
  sentinel), `vertical-align: middle`. No new `:root` token (the size is a skin literal; promote to a
  token only if a second consumer needs it, D-069).
- `.led[data-pulse]` — `animation: led-pulse <dur> ease-in-out infinite` (opacity pulse 1↔~0.4, or a
  soft `box-shadow` halo — Joe eye-checks). `@keyframes led-pulse { … }` defined in the same block.
- No `:hover`/`:focus`/`:disabled` (display-di, non-interactive).
- The `@keyframes` is verified by **stylesheet inspection** (a `CSSKeyframesRule`, not a selector) +
  the computed `animation-name` on `led#pulse` (N-042 method family).

---

## 5. Sampler integration (the standing sampler-DoD, D-097)

Add a `led` row to the **Display** section of `ui/sampler/src/app_sampler.svelte` (plain-JS shell, bare
`$state` — N-041), after the `image` row. All cells share one `ledStates` map demonstrating **both**
value kinds (hex + `var(--token)`):

```js
const ledStates = { ON: '#22c55e', OFF: 'var(--t4)', ERR: 'var(--err)' };
```

| cell `id` | props | shows |
|---|---|---|
| `led#default` | `states={ledStates}` `state="ON"` | green hex dot |
| `led#off` | `states={ledStates}` `state="OFF"` | grey `var(--t4)` dot (the token path) |
| `led#pulse` | `states={ledStates}` `state="ERR"` `pulse` | red `var(--err)` dot, pulsing |
| `led#unknown` | `states={ledStates}` `state="???"` | **black `#000000` sentinel** (the unmapped-key proof) |

Matrix **37 → 41**. (`led` is a display-di — no disabled/invalid states; honest-ragged like
label/paragraph/image.)

---

## 6. CDP verification (Chat self-drives — sampler only)

Launch detached (`Start-Process run-sampler.ps1 -Debug -WindowStyle Minimized`); poll CDP 9422
(retry until non-null, N-044); fresh launch (avoid stale HMR, the J-430 finding); `cdp-debug.ps1 -App
sampler -Mode {state,eval,screenshot}`; clean up (5175/9422 free, 0 orphans).

1. `ids().length === 41`; all four `led#` present.
2. **Map-resolution proof (the headline):** registry getters —
   `led#default {state:"ON", colour:"#22c55e"}`, `led#off {state:"OFF", colour:"var(--t4)"}` (the raw
   map value travels — token reference preserved), `led#unknown {state:"???", colour:"#000000"}`
   (**the black sentinel for an unmapped key — the contract proof**), `led#pulse {state:"ERR",
   colour:"var(--err)"}`.
3. **Computed colour (the inline-var → skin path):** `getComputedStyle(led#default).backgroundColor`
   = `rgb(34, 197, 94)` (#22c55e); `led#off` = the resolved `--t4` rgb (`rgb(88, 92, 100)`); `led#unknown`
   = `rgb(0, 0, 0)`. Confirms `--led-colour` (incl. the `var(--token)` case) drives `.led` background.
4. **`pulse`:** `led#pulse` has `data-pulse` present; `getComputedStyle(led#pulse).animationName !==
   "none"` (the `led-pulse` keyframes applied); `led#default` `animationName === "none"`.
5. **A11y / shape:** every `led#` element `tagName === "SPAN"`, `role="img"`, non-empty `aria-label`
   (= the state key, `"???"` for unknown), `title` present; computed `border-radius` round (`50%` →
   resolved px), `display: inline-block`.
6. **Skin rules:** `.led` + `.led[data-pulse]` parsed + in cascade; `@keyframes led-pulse` present
   (CSSKeyframesRule, by-type inspection — N-042 family).
7. **No accent dependency (the call-out):** flipping `[data-shell]` client↔node leaves all four
   `led#` `backgroundColor` **unchanged** (colour is caller-supplied, not accent-derived) — the proof
   `led` breaks the accent-swap pattern by design.
8. **Screenshot (eye-check):** four dots — green / grey / red (pulsing, caught mid-cycle) / black;
   the `#unknown` black dot is visible (sentinel renders, does not vanish).

Quote **actual** CDP output in the JOURNAL (Rule 2); never invent (Rule 5).

---

## 7. Records (D-074; written after verification, Rule 4)

- `ui/docs/xgen-ui-notes.md` — **N-051** (build + the caller-supplied-map display-di + the colour-as-
  data / inline-CSS-var mechanism + the `#000000` sentinel contract + the `data-pulse` attribute-hook +
  the no-accent-dependency call-out + the render+computed-style verify).
- `docs/ROADMAP.md` — M-RP2.20 ✅ (RP node + Present narrative); version bump; same-commit with CLAUDE.
- `CLAUDE.md` — PLAY → M-RP2.20; prior-PLAY pointer → J-431. Di queue now reads `link` →
  `status-indicator`.
- `ui/docs/xgen-ui-components.md` — promote `led` from Planned to built (a Built-note + the registry/
  catalogue row); `link` + `status-indicator` stay Planned.
- `JOURNAL.md` — **J-431** (newest-first; real CDP output).
- `tasks/M_RP2_20_led.md` — Status → COMPLETED.
- **No `DECISIONS.md` touch** — a new simple display-di; applies D-096, no amendment (the map shape is
  the N-034 precedent reused).

`.md` header rule: `> **Last updated**:` carries ONLY the date.

---

## 8. Commit plan (two commits, UI pattern `feat` → `docs`; Joe pushes)

Multi-file discipline: write each file (Filesystem) + `get_file_info`-verify before the next;
`git add` per file; `git status` sanity; multi-`-m` commit.

**Commit 1 — implementation** (`led.svelte`, `skin.css`, `app_sampler.svelte`):

```powershell
cd E:\Projects\XGenProtocol
git add ui/core/lib/components/data-independent/led.svelte
git add ui/assets/skin.css
git add ui/sampler/src/app_sampler.svelte
git status
git commit -m "feat(ui): led - fifteenth core component, fourth simple display-di, caller-supplied colour map (M-RP2.20)" -m "Atomic inline <span class=led> status light. Caller-supplied states:Record<string,string> map (hex or var(--token)) + state key picks the colour (the N-034 options-prop shape applied to a display-di); pulse? orthogonal boolean. Resolve colour = states[state] ?? #000000 (black is the reserved unknown/undefined sentinel; consumers must never map a real state to black). title = state ?? '?'; getter {state, colour}; role=img + aria-label. First component whose colour is caller-supplied, not accent-derived." -m "Own .led skin owns SHAPE only (size, border-radius:50%, pulse @keyframes); colour rides an inline --led-colour CSS var from the prop (clean L2 split). pulse via reflected data-pulse attribute (the toggle role=switch attribute-hook precedent). Built/tuned/CDP-verified in the sampler (D-097): led row + 4 cells (matrix 37->41), map resolution incl the #000000 sentinel for an unmapped key, computed background incl the var(--token) path, pulse animation, and the no-accent-dependency proof (dots unchanged across skin-swap)."
git push
```

**Commit 2 — records** (`xgen-ui-notes.md`, `ROADMAP.md`, `CLAUDE.md`, `xgen-ui-components.md`,
`JOURNAL.md`, `M_RP2_20_led.md`):

```powershell
cd E:\Projects\XGenProtocol
git add ui/docs/xgen-ui-notes.md
git add docs/ROADMAP.md
git add CLAUDE.md
git add ui/docs/xgen-ui-components.md
git add JOURNAL.md
git add tasks/M_RP2_20_led.md
git status
git commit -m "docs(ui): close M-RP2.20 led - N-051, J-431, records" -m "N-051 (caller-supplied-map display-di + colour-as-data via inline --led-colour CSS var + the #000000 sentinel contract + data-pulse attribute-hook + no-accent-dependency call-out + render+computed-style verify) + ROADMAP M-RP2.20 done + CLAUDE PLAY -> M-RP2.20 + components registry led promoted (link + status-indicator stay Planned)." -m "Di queue now reads link -> status-indicator. Task -> COMPLETED. No DECISIONS touch (new simple display-di; applies D-096, the map is the N-034 precedent)."
git push
```

*(Final `git add` lists confirmed against `git status` at close.)*

---

## 9. Definition of Done

- [x] `led.svelte` authored to §3 (`states`/`state`/`pulse`/`id`; `colour`/`title` derived; `#000000`
      sentinel; getter `{state,colour}`; `role="img"` + `aria-label`; inline `--led-colour`; contract
      in header).
- [x] `.led` shape-only skin added to §4 (dot + `border-radius:50%` + `var(--led-colour)` + `.led[data-pulse]`
      + `@keyframes led-pulse`).
- [x] Sampler row + `default`/`off`/`pulse`/`unknown` cells added (§5) and live (matrix 37→41).
- [x] CDP verification §6 run in the sampler — actual output captured (incl. the `#000000` sentinel
      proof, the `var(--token)` computed-colour path, and the no-accent-dependency proof).
- [x] N-051 written; ROADMAP + CLAUDE updated same-commit; components registry `led` promoted; JOURNAL
      J-431 written (real CDP output).
- [x] Task Status → COMPLETED.

(`Status: COMPLETED` is the real signal — no "commit pushed" checklist item.)
