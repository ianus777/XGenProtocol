# M-RP2.14 — `number` (di·A, atomic `<input type="number">`, numeric free-entry)
> **Status**: COMPLETED  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-26  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Goal

Author the ninth `core` component `number` (di·A, atomic, root `<input type="number">`): numeric free-entry, **numeric** `bind:value`. The next atomic di after `textarea` per the locked N-038 track order. Mechanically the same `<input>` root as `textfield`, but a **distinct atomic, NOT a member of the `textfield` `type` fold** — the boundary D-096 drew is *same root + same value-type*, and `number` breaks the second half: Svelte's `bind:value` on `type="number"` coerces to a **number** (`null` when empty), not a string. Folding it in would force `textfield`'s `value` prop polymorphic (`string | number | null`) and defeat the single-typed contract the fold exists to give. So `number` stays its own component (D-096 **held, not amended**) — the first registry value that is neither boolean (toggle) nor string (everything since): a JSON **number**.

## Locks (design walk, Joe-locked 2026-06-26 "by your recomm" — boundary held + Q1–Q6 by recomms)

1. **Own atomic, not a textfield fold (D-096 held).** Own component `ui/core/lib/components/data-independent/number.svelte`, root `<input type="number" use:envelope>`. Distinct from `textfield` on value-type, not root tag. No `DECISIONS.md` touch — holding the boundary is *applying* D-096, not amending it.
2. **Value type — `number | null`, default `null`.** `value = $bindable<number | null>(null)`. Default `null` = "nothing entered yet"; rejecting a plain `0` default because `0` is a real entry — conflating empty with zero is dishonest for a reference atomic (D-065). The actual empty/cleared runtime value is **verified at CDP and reported honestly**, not asserted from Svelte internals (expectation: `null`; if the runtime yields `undefined`/`""`, the records state what was observed, and the type/default is reconciled to it before close).
3. **Prop surface** — control vocabulary, numeric bits swapped in:
   - **Keep:** `value` (numeric $bindable), `placeholder` (shows when empty), `disabled`, `readonly`, `id`, `name`.
   - **Drop:** `type` (fixed `number`), `pattern` (ignored on `type=number`).
   - **Add:** `min`, `max`, `step` (optional numbers) — the numeric analogue of textarea's `rows`: native attributes that shape the control (`step` drives the native-spinner increment). Config, not state → **not** in the getter.
   - **`maxlength` stays OUT** (mirrors textfield/textarea; orthogonal).
4. **Native spinner KEPT, no suppression.** The UA up/down spinner *is* the atomic's affordance (per the catalogue scoping — the custom-button **stepper** is a separate composite, later track). No `::-webkit-inner-spin-button`/`::-webkit-outer-spin-button` suppression. (Contrast M-RP2.12, where the search clear-x was suppressed because it collided with our inset icon — here there is nothing to collide with and the spinner is wanted.)
5. **Getter → `{ value }`** — single field, value-only (`min`/`max`/`step` are static config, not user-mutable state). Now carries a `number | null`; this is the first non-string/non-boolean registry value.
6. **Skin — own `.number` key**, assembled from the M-RP2.7 L2 vocabulary like `.textfield` (per-class clarity > DRY, the `.select`/`.textarea` precedent). Single-line control → **keeps `min-height: --ctl-h`** (unlike `.textarea`). **Keeps `:invalid` → `--err`** — meaningful here via native numeric constraint validation (out-of-`min`/`max`, bad `step`), the same treatment `.textfield` uses for email/pattern. **No icon machinery, no `resize`.** No new `:root` token.
7. **Processor seam — DEFERRED; ships processor-READY (second consumer to defer).** N-038 names `number` as the processor's **numeric-formatting** consumer. Same two locked grounds as `textarea`: (1) the N-038 sequence builds the engine in its own arc after *all* atomic di (all consumers in hand) — `number` is not the last atomic (`range`/`date`/`color`/`file`/`select multiple` follow); (2) D-065 — the atomic is function-complete without it. Reserve the **edit-side `use:processor`** insertion point in the header; build nothing. This is the **second** defer-per-consumer instance (after `textarea`); noted as a D-069 promotion-watch, **not** at the four-recurrence bar.
8. **`use:envelope` unchanged** — content-agnostic substrate reused verbatim; confirms generalization onto a numeric-bound `<input>`. Zero `<style>` (L1 empty) — all appearance is skin.

## Phases

**Phase 1 — author** `ui/core/lib/components/data-independent/number.svelte`:
- Root `<input type="number" use:envelope={{ name: 'number', id, debug }} bind:value {placeholder} {disabled} {readonly} {min} {max} {step} {name}>`.
- `$props()`: `value = $bindable<number | null>(null)`, `placeholder = ''`, `disabled = false`, `readonly = false`, `min`, `max`, `step`, `id`, `name` — TS types: `value?: number | null`, `min`/`max`/`step?: number`, rest as textfield (minus `type`/`pattern`).
- Getter `debug = () => $state.snapshot({ value })`.
- **Header comment** modelled on `textarea.svelte`'s: atomic root `<input type="number">`; the value-type discriminator (why it is NOT a textfield fold — D-096 *same root + same value-type*; numeric `bind:value`, `null` when empty); native-spinner-is-the-atomic note (stepper = composite); the reserved **edit-side `use:processor`** seam (numeric formatting; processor-ready, NOT built — D-065 + N-038 sequence).
- Zero `<style>`.

**Phase 2 — skin** `skin.css`, new `.number` block (after `.textarea`, before `.select`):
- Assemble from L2 (same box as `.textfield`): `min-height: var(--ctl-h)`, `padding: var(--sp-1) var(--sp-2)`, `margin: var(--sp-1) var(--sp-2)`, `background: var(--s)`, `border: 1px solid var(--s5)`, `border-radius: var(--rad)`, `color: var(--t)`, `font-size: var(--fs-1)`, `line-height: var(--lh)`, `transition: border-color var(--motion), box-shadow var(--motion)`.
- `.number::placeholder { color: var(--t4); }`
- `.number:focus-visible { outline: none; border-color: var(--accent2, var(--t3)); box-shadow: var(--focus-ring); }`
- `.number:disabled { background: var(--s2); color: var(--t4); cursor: not-allowed; }`
- `.number:read-only { background: var(--s2); border-color: var(--s4); }`
- `.number:invalid { border-color: var(--err); }`
- **No** icon rules, **no** `resize`, **no** spinner suppression, **no** new `:root` token.

**Phase 3 — wire demo, both shells** (`app_client.svelte` + `app_node.svelte`):
- `let demoNumber = $state<number | null>(null)` (throwaway, with the M-RP2.14 comment).
- Mount `<Number bind:value={demoNumber} id="demo" placeholder="0" min={0} max={100} step={1} />` beside the other demo di. Import `Number` from `$core`. (Note: local import name `Number` shadows the global — acceptable in the demo shell; if the bundler/linter objects, alias to `NumberField` at import. Decide at implement; flag in records if aliased.)

**Phase 4 — CDP verify both apps** (Chat self-drives, real `tauri dev` + CDP; N-028 race-retry + clean teardown):
- Registry baseline both apps: `number#demo` → record the **actual** empty value (expect `{value:null}`); reconcile Lock 2 type/default to what is observed before close.
- Set value + dispatch **`input`** (N-029) → assert the registry carries a **JSON number** (`typeof === "number"`, e.g. `42` client / `7` node), not a string `"42"` — the number-distinguishing proof (the analogue of textarea's `\n`-survives-the-rune proof).
- `:invalid` probe (detached `<input type="number">` with `min`/`max`): out-of-range → computed `border-color` = `--err` `rgb(138, 42, 42)`; in-range → `--s5` `rgb(52, 59, 71)`.
- Computed-style both: tag `INPUT`, `type === "number"`, `min-height`/`--ctl-h`, `font-size 12px` (=`--fs-1`), `color rgb(236,233,225)` (=`--t`), `border-radius 6px` (=`--rad`).
- Screenshots both apps — eye-check the number box + the native spinner arrows render + per-shell chrome.
- Clean teardown (ports 9222/9322/5173/5174 free, 0 orphans).

**Phase 5 — records (D-074 atomic, same-commit):**
- `ui/docs/xgen-ui-notes.md` **N-041** — `number` built; own atomic (D-096 held, the value-type discriminator); numeric `bind:value` + the empty=null finding (as observed); prop surface (drops `type`/`pattern`, adds `min`/`max`/`step`); native spinner kept; the reserved processor seam + the **second** defer-per-consumer instance (D-069 watch); the JSON-number verify.
- `ui/docs/xgen-ui-components.md` (v0.18→**0.19**) — Built `number` row (Tier `core`, Phase A, `data-independent · numeric`, root `<input type="number">`, getter `{value}`, ref `N-022/N-024/N-038/N-041`); add a `number` detail paragraph; the di-catalogue *numeric (exact)* row gets a build-note (the *stepper* shape variant deferred → composite track).
- `docs/ROADMAP.md` — RP node M-RP2.14 ✅ + both chains + Present clause + frontier M-RP2.13→M-RP2.14; version bump.
- `CLAUDE.md` PLAY → M-RP2.14 ✅ CLOSED; Next → `range` (next session per Joe); pointer J-418→J-419.
- This task → **COMPLETED**.
- `JOURNAL.md` **J-419** (written last, Rule 4, real CDP output quoted).
- All touched `.md` `Last updated` bumped to close date.
- **No `DECISIONS.md` touch** — D-096 held (applied, not amended); the processor-defer is application of the existing N-038 sequence + D-065. If the defer-per-consumer pattern reaches the four-recurrence bar later, it graduates then (D-069).

## Definition of Done

- [x] `number.svelte`: root `<input type="number" use:envelope>`; props `value`($bindable `number|null`, default `null`)/`placeholder`/`disabled`/`readonly`/`min`/`max`/`step`/`id`/`name`; no `type`/`pattern`; getter `{value}`; header comment (value-type discriminator + native-spinner + reserved/deferred `use:processor`); zero `<style>`
- [x] `skin.css`: new `.number` block assembled from L2; keeps `--ctl-h` + `:invalid`→`--err`; placeholder/focus/disabled/read-only states; no icons, no `resize`, no spinner suppression, no new `:root` token
- [x] demo wired both shells (`<Number bind:value={demoNumber} id="demo" min/max/step>` + `demoNumber` state + `$core` import; aliasing noted if used) — imported as `NumberField` (shadowing fix); `let demoNumber = $state(null)` bare (shells are plain JS)
- [x] CDP both apps: `number#demo` empty baseline value recorded as observed (real output in J-419); Lock 2 type/default reconciled to it — **empty = `null`** confirmed, matches the locked default
- [x] CDP both apps: dispatched `input` → registry carries a JSON **number** (`typeof number`, `42`/`7`), not a string — number round-trip proven (real output)
- [x] CDP both apps: `:invalid` (out-of-min/max) → `--err`; in-range → `--s5` (real output) — probed on the live `.number` (min 0/max 100) rather than detached; separate-task read needed
- [x] CDP both apps: computed-style — tag `INPUT`, `type=number`, `--ctl-h`/`--fs-1`/`--t`/`--rad` (real output)
- [x] screenshots both apps: number box renders (native spinner is UA hover/focus-revealed in Chromium) + per-shell chrome
- [x] clean teardown (0 orphan ports)
- [x] records updated (N-041, components v0.19 + catalogue build-note, ROADMAP, CLAUDE PLAY, JOURNAL J-419, task COMPLETED); no DECISIONS.md touch
