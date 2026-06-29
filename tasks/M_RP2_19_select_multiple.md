# M-RP2.19 — `select-multiple` (di·A, atomic `<select multiple>`, own atomic; first `string[]` array value-type)

> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-29  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. What this is

Author + skin `select-multiple` — the **fourteenth** `core` component, the **last input-family
atomic di** (N-038 catalogue). Built, tuned, CDP-verified **in the sampler** (D-097). The headline is
the **first array value-type** in the library: `bind:value` → **`string[]`** (the 5th binding shape
after boolean-in `checked` / event-out `onclick` / string-in `value` / number / FileList), with the
**`[]`-not-`null` empty model** — set-absent is empty-set, not scalar-null.

**Joe-locked at the design walk (2026-06-29):**

1. **OWN ATOMIC** (D-a), **not a `select` fold** — applies sharpened D-096, no amendment. Same root
   tag `<select>`, but **two of three fold criteria fail**: value-type diverges (`string[]` vs
   `string`) **and** skin-surface diverges (scrolling list-box vs dropdown). Shared tag alone never
   folds (the `range`-vs-`date` precedent).
2. **Binding (D-b):** `bind:value` on `<select multiple>` → native **`string[]`**; no `bind:group`
   (that is for checkbox/radio sets). Serializable — no getter de-serialisation (unlike FileList).
3. **Empty model (D-c):** **`[]`, not `null`.** An array prop is always an array — callers
   `.length`/`.map` with no null-guard. Diverges from single `select`'s `null` empty; the divergence
   is correct (this is the N-038 array landing).
4. **Getter (D-d):** `{ values, count }` — mirrors `file`'s structured shape for sampler-row
   consistency (the matrix already reads `{count, …}`).
5. **Options-prop (D-e):** N-034 carries over **unchanged** — same caller-supplied `options` map as
   `select`; the two siblings stay API-symmetric on options.
6. **`size` (D-f):** expose **`size?: number`**, default **4** (the one genuinely multi-specific
   knob — visible rows of the list-box).
7. **Milestone M-RP2.19**, sampler cells `select-multiple#default` + `select-multiple#seeded` +
   `select-multiple#disabled`.

---

## 1. Why own atomic + the new shape (for the N-entry)

Own atomic under **sharpened D-096**: `<select multiple>` shares the `<select>` tag but fails the
value-type criterion (`string[]` vs scalar `string`) **and** the skin-surface criterion (a static
scrolling list-box, not a dropdown that opens). Two of three fail → own atomic, same logic that split
`range` from `date`. Applies D-096, **no amendment** (no `DECISIONS.md` touch).

The **substrate question** this pass answers: every prior binding has been a **scalar or host-object**
(boolean / string / number / FileList). `select-multiple` is the **first plain-array value-type** —
`string[]` via `bind:value` — and the first prop whose **empty state is a non-null container** (`[]`).
`$state.snapshot` flattens an array cleanly (it is a plain proxy, not a host object), so the getter is
trivial — the new ground is the **`[]` empty model + array round-trip**, not serialisation.

---

## 2. Phase-0 references (read before authoring)

- `ui/core/lib/components/data-independent/select.svelte` — the sibling: options-prop (N-034), root
  `<select>`, the **single-select `null` empty** this component deliberately diverges from (`[]`).
- `ui/core/lib/components/data-independent/file.svelte` — the `{count, …}` getter shape precedent
  (D-d mirrors it) + the sampler `{count}`-reading row precedent.
- `ui/assets/skin.css` — the **`.select`** block (the list-box must read as a sibling surface:
  `bg:--s4`, `border:--s5`, `--rad`, `color:--t2`, `--fs-1`, `--lh`, `padding`) + how `:focus-visible`
  / `:disabled` are handled there.
- `ui/common/lib/components/base/envelope` + `debug` — the getter runs through
  `window.__XGEN_DEBUG__`; returns a **plain** object (`{values:[...], count}`) so CDP
  `returnByValue` round-trips.
- `ui/docs/xgen-ui-notes.md` N-024 (registry getter), N-034 (options-prop), N-038 (track order /
  the array-value-type call-out), N-041 (plain-JS shells use bare `$state`).
- `DECISIONS.md` D-096 (own-atomics; sharpened fold criterion).

---

## 3. Component spec — `ui/core/lib/components/data-independent/select-multiple.svelte`

Root IS `<select multiple>`. Zero local `<style>`.

**Props:**

| prop | type | default | note |
|---|---|---|---|
| `value` | `string[]` | `[]` (`$bindable`) | **`bind:value`** — native `string[]`; empty = `[]` (NOT `null`) |
| `options` | `Array<{value, label}>` | `[]` | caller-supplied (N-034, carries over from `select`) |
| `size` | `number` | `4` | visible rows of the list-box (multi-specific knob, D-f) |
| `disabled` | `boolean` | `false` | inert + skin-greyed |
| `id` | `string` | — | |
| `name` | `string` | — | |

- **Drop** `placeholder`/`pattern`/`readonly`/`min`/`max`/`step`/`type`/`multiple` (the last is
  fixed-true, not a prop — it is the component's identity).
- **Getter (the design point):** `const debug = () => ({ values: [...value], count: value.length });`
  — a **plain** object; `[...value]` (or `$state.snapshot(value)`) is safe for a plain array.
- `bind:value` on the element; `multiple` hardcoded; `{size}`; `use:envelope={{ name:
  'select-multiple', id, debug }}`.
- Options render via the N-034 `{#each options}` pattern, same as `select`.
- **No processor seam** (a pick, not typed entry).

---

## 4. Skin spec — add `.select-multiple` to `ui/assets/skin.css`

Own `.select-multiple` key. Place after `.select`, before `.file`. **Baseline; Joe live-tunes via HMR.**

- Read as a sibling **list-box surface**: `bg:--s4`, `border:1px solid --s5`, `--rad`, `color:--t2`,
  `--fs-1`, `--lh`, `padding:--sp-1`. No dropdown-arrow chrome (it is not a dropdown).
- Selected `<option>` rows: accent-tinted background (`--accent2`) so a selection reads against the
  list — `.select-multiple option:checked` (where the engine honours it; document if UA-overridden,
  N-042-style).
- `:focus-visible` → accent border (`--accent2`); `:disabled` → greyed + `cursor:not-allowed`.
- No new `:root` token. Verify selected-row styling by **stylesheet-rule inspection + screenshot**
  (N-042 — `option:checked` author styling is partially UA-controlled).

---

## 5. Sampler integration (the standing sampler-DoD, D-097)

Add a `select-multiple` row to `ui/sampler/src/app_sampler.svelte` (plain-JS shell, bare `$state` —
N-041). Each cell binds its own `value` state. **All cells share a small `options` array** (e.g.
`[{value:'a',label:'Alpha'},{value:'b',label:'Beta'},{value:'c',label:'Gamma'}]`):

| cell `id` | props / seed | shows |
|---|---|---|
| `select-multiple#default` | `value = []` | empty list-box, nothing selected |
| `select-multiple#seeded` | `value = ['a','c']` | two rows pre-selected (the `[]`-vs-seeded contrast) |
| `select-multiple#disabled` | `disabled`, `value = ['b']` | greyed, inert, one selected |

Matrix **34 → 37**. (Unlike `file`, an array CAN be pre-seeded from markup — the `#seeded` cell is the
honest array round-trip seed.)

---

## 6. CDP verification (Chat self-drives — sampler only, both accents via skin-swap)

Launch detached (`Start-Process run-sampler.ps1 -Debug -WindowStyle Minimized`); poll CDP 9422
(retry until non-null, N-044); `cdp-debug.ps1 -App sampler -Mode {state,eval,screenshot}`; clean up
(5175/9422 free, 0 orphans).

1. `ids().length === 37`; all three `select-multiple#` present.
2. **Seed proof (the `[]` empty model + array shape):** `#default` getter `{values:[], count:0}`
   (empty = `[]`, **not** `null`); `#seeded` getter `{values:["a","c"], count:2}`.
3. **Bind path (the array round-trip — the headline proof):** select rows via real DOM + dispatch
   **`change`** (`<select>` fires `change`):
   ```js
   const el = document.querySelector('[data-debug-id="select-multiple#default"]');
   Array.from(el.options).forEach(o => o.selected = (o.value === 'a' || o.value === 'b'));
   el.dispatchEvent(new Event('change', { bubbles: true }));
   ```
   → bound getter `{ values:["a","b"], count:2 }` — a `string[]` round-trips through `bind:value`
   (the substrate's first plain-array value-type proven).
4. **`size`:** `#default` element `.size === 4` (default); `multiple` attr present (`.multiple === true`).
5. **Computed-style:** element `SELECT`/`multiple`; list-box surface (`--s4`/`--s5`) verified;
   `option:checked` accent styling via **stylesheet-rule inspection** (N-042).
6. **Disabled:** `select-multiple#disabled` `.disabled === true` + greyed, getter still
   `{values:["b"], count:1}`.
7. **Skin-swap:** `--accent2` flips gold (`--pr2`, client) ↔ blue (`--inf2`, node).
8. **Screenshot (both shells eye-checked):** list-box renders; `#seeded` shows two highlighted rows;
   `#disabled` greyed.

Quote **actual** CDP output in the JOURNAL (Rule 2); never invent (Rule 5).

---

## 7. Records (D-074; written after verification, Rule 4)

- `ui/docs/xgen-ui-notes.md` — **N-050** (build + the first `string[]` array value-type via
  `bind:value` + the `[]`-not-`null` empty model + the `{values, count}` getter + the
  `change`-dispatch array round-trip verify method + the sharpened-D-096 own-atomic call vs `select`).
- `docs/ROADMAP.md` — M-RP2.19 ✅ (tree node + RP narrative); version bump; same-commit with CLAUDE.
  Note the **input-family atomic axis now closed** (last atomic di).
- `CLAUDE.md` — PLAY → M-RP2.19; prior-PLAY pointer bumped → J-430. Di queue now reads
  `led` + `link` → `status-indicator`.
- `ui/docs/xgen-ui-components.md` — promote the *select-multiple* row to built (M-RP2.19, J-430).
- `JOURNAL.md` — **J-430** (newest-first; real CDP output).
- `tasks/M_RP2_19_select_multiple.md` — Status → COMPLETED.
- **No `DECISIONS.md` touch** — applies sharpened D-096 (own atomic; the array binding shape is a
  substrate proof, not a decision).

`.md` header rule: `> **Last updated**:` carries ONLY the date.

---

## 8. Commit plan (two commits, UI pattern `feat` → `docs`; Joe pushes)

Multi-file discipline: write each file (Filesystem) + `get_file_info`-verify before the next;
`git add` per file; `git status` sanity; multi-`-m` commit.

**Commit 1 — implementation** (`select-multiple.svelte`, `skin.css`, `app_sampler.svelte`):

```powershell
cd E:\Projects\XGenProtocol
git add ui/core/lib/components/data-independent/select-multiple.svelte
git add ui/assets/skin.css
git add ui/sampler/src/app_sampler.svelte
git status
git commit -m "feat(ui): select-multiple - fourteenth core component, last input-family atomic (M-RP2.19)" -m "Atomic <select multiple>, the first array value-type: bind:value -> string[], the 5th binding shape after boolean-in/event-out/string-in/number/FileList. Own atomic (applies sharpened D-096, no amendment): shares the <select> tag but fails the value-type (string[] vs string) AND skin-surface (list-box vs dropdown) criteria. Empty model is [] not null (set-absent vs scalar-null). Getter {values, count}; bind:value carries the live array. Options-prop carries over from select (N-034). size? prop, default 4." -m "Own .select-multiple skin: list-box surface (--s4/--s5), accent-tinted option:checked rows, focus/disabled states. Built/tuned/CDP-verified in the sampler (D-097): select-multiple row + 3 cells (matrix 34->37), string[] round-trip via change dispatch, [] empty model + seeded ['a','c'] proven, skin-swap accent gold<->blue."
git push
```

**Commit 2 — records** (`xgen-ui-notes.md`, `ROADMAP.md`, `CLAUDE.md`, `xgen-ui-components.md`,
`JOURNAL.md`, `M_RP2_19_select_multiple.md`):

```powershell
cd E:\Projects\XGenProtocol
git add ui/docs/xgen-ui-notes.md
git add docs/ROADMAP.md
git add CLAUDE.md
git add ui/docs/xgen-ui-components.md
git add JOURNAL.md
git add tasks/M_RP2_19_select_multiple.md
git status
git commit -m "docs(ui): close M-RP2.19 select-multiple - N-050, J-430, records" -m "N-050 (first string[] array value-type via bind:value + []-not-null empty model + {values,count} getter + change-dispatch array round-trip verify + sharpened-D-096 own-atomic-vs-select) + ROADMAP M-RP2.19 done (input-family atomic axis closed) + CLAUDE PLAY -> M-RP2.19 + components registry select-multiple row promoted." -m "Di queue now reads led + link -> status-indicator. Task -> COMPLETED. No DECISIONS touch (applies sharpened D-096)."
git push
```

*(Final `git add` lists confirmed against `git status` at close.)*

---

## 9. Definition of Done

- [ ] `select-multiple.svelte` authored to §3 (`bind:value` → `string[]`, `[]` empty, `{values,count}`
      getter, `multiple` hardcoded, `size?` default 4, options per N-034).
- [ ] `.select-multiple` list-box skin added to §4 (surface + `option:checked` accent + focus/disabled).
- [ ] Sampler row + `default`/`seeded`/`disabled` cells added (§5) and live.
- [ ] CDP verification §6 run in the sampler, both accents — actual output captured (incl. the `[]`
      empty-model proof + the `change`-dispatch array round-trip).
- [ ] N-050 written; ROADMAP + CLAUDE updated same-commit; components registry row promoted; JOURNAL
      J-430 written (real CDP output).
- [ ] Task Status → COMPLETED.

(`Status: COMPLETED` is the real signal — no "commit pushed" checklist item.)
