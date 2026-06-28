# M-RP2.17 — `color` (di·A, atomic `<input type="color">` swatch, own atomic)

> **Status**: COMPLETED  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-28  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. What this is

Author + skin `color` — the **twelfth** `core` component, the next atomic di after `date`
(N-038 catalogue row *color*). Built, tuned, CDP-verified **in the sampler** (D-097, now the
standing DoD). Exposes the **native** Chromium colour picker; zero custom-palette code.

**Joe-locked at the design walk (2026-06-28):**

1. **OWN ATOMIC** — a singleton (no type-family); stands alone, not folded. The `range` case
   (own atomic on disjoint skin/surface), not the `textfield` case.
2. **Value:** always a string hex `#rrggbb`, **default `#000000`**, never empty; getter `{value}`
   (no `type` — singleton).
3. **Prop surface — leanest yet:** `value` / `disabled` / `id` / `name` only.
4. **Skin:** own `.color` key, swatch pseudos, no `--ctl-h`.
5. **Milestone M-RP2.17**, sampler cells `color#default` + `color#disabled`.
6. **Horizon item logged:** a themed **`color-picker` composite #2** (custom saturation/hue/
   eyedropper/themed swatches) — the deferred composite, the `password-field`-off-`textfield`
   shape. Recorded at close (registry Composites + ROADMAP note), **not built** here.

---

## 1. Why own atomic (for the N-entry)

Sharpened D-096 (root + value-type + **shared skin/surface**, N-042). `color` has **no siblings**
(unlike date's five), so the test is sideways — `color` vs `date`/`range`:

- **root** — `<input>`, shared with `date`.
- **value-type** — string (`#rrggbb`), the **same** value-type as `date`. Root + value-type alone
  would pull toward a fold — exactly the trap the sharpened criterion exists for.
- **skin/surface** — **disjoint**. `color` renders a **swatch** (`::-webkit-color-swatch` +
  `::-webkit-color-swatch-wrapper`), nothing shared with date's text-box + calendar indicator;
  the prop surface differs too (no min/max/step, no `:invalid`).

→ **own atomic**, the `range` case (shares root + value-type with a sibling but stands alone on
disjoint skin). **Applies D-096, no amendment** (no `DECISIONS.md` touch).

**The native picker is not skinnable.** The open dialog (saturation square / hue slider /
eyedropper / hex field / preset swatches) is OS/Chromium-painted. The `.color` skin styles **only
the closed-state swatch**. A themed palette = the deferred `color-picker` **composite #2**.

---

## 2. Phase-0 references (read before authoring)

- `ui/core/lib/components/data-independent/range.svelte` — the own-atomic-on-disjoint-skin
  precedent + always-valued (default, never null) + no-processor-seam + the pseudo-element verify
  method (N-042: `getComputedStyle` returns UA defaults on shadow-pseudos → stylesheet-rule
  inspection + screenshot).
- `ui/core/lib/components/data-independent/date.svelte` — the string-valued native-`<input>`
  sibling `color` diverges from on skin/surface.
- `ui/assets/skin.css` — `.range` (pseudo-element-heavy skin precedent) + `.date` (just added).
- `ui/docs/xgen-ui-notes.md` N-038 (track order), N-042 (D-096 amendment + pseudo verify),
  N-043 (color-scheme — note it's moot for the OS-painted picker), N-046 (date).
- `DECISIONS.md` D-096 (criterion + own-atomics list naming `color`).

---

## 3. Component spec — `ui/core/lib/components/data-independent/color.svelte`

Root IS `<input type="color">`. Zero local `<style>`.

**Props:**

| prop | type | default | note |
|---|---|---|---|
| `value` | `string` | `'#000000'` | string `bind:value`; always a valid `#rrggbb`, never empty |
| `disabled` | `boolean` | `false` | inert + skin-greyed |
| `id` | `string` | — | |
| `name` | `string` | — | |

- **Drop** `placeholder`/`pattern` (n/a), `readonly` (native no-op on color — the range precedent),
  `min`/`max`/`step` (n/a), `:invalid` (always a valid hex — never invalid), `type` (fixed).
- **Getter:** `const debug = () => $state.snapshot({ value });` — no `type` (singleton).
- **No processor seam** — a swatch pick, not typed entry (the range reasoning).
- **`alpha`/`colorspace`** (`#rrggbbaa`) reserved as a future shape, **not built**.
- `use:envelope={{ name: 'color', id, debug }}`.

---

## 4. Skin spec — add `.color` to `ui/assets/skin.css`

Own `.color` key, pseudo-element-heavy like `.range`. Place after `.date`, before `.select`.
**Baseline; Joe live-tunes via HMR (D-098).**

- Box: `-webkit-appearance:none` + `appearance:none`; a **compact** clickable square/rect (e.g.
  ~36×24, **no `--ctl-h`** — the swatch is small, as `.range`/`.textarea` dropped it); `padding:0`;
  `--sp-1`/`--sp-2` margin; `background: var(--s)`; `1px solid var(--s5)`; `border-radius: var(--rad)`;
  `cursor: pointer`; border/box-shadow transition.
- `:focus-visible` → `--accent2` border + `--focus-ring`.
- `:disabled` → `cursor: not-allowed` + dimmed (e.g. `opacity` or `--s2`), swatch still shows colour.
- `::-webkit-color-swatch-wrapper { padding: 0; }` (kills native inset).
- `::-webkit-color-swatch { border: none; border-radius: calc(var(--rad) - 1px); }` (the colour fill).
- No `:invalid`, no `--ctl-h`, no new `:root` token. Pseudo-elements verified by **stylesheet-rule
  inspection + screenshot** (N-042), not `getComputedStyle`.

---

## 5. Sampler integration (the standing sampler-DoD, D-097)

Add a `color` row to `ui/sampler/src/app_sampler.svelte` (plain-JS shell, bare `$state` — N-041).
Cells (ragged-honest — color has no invalid/type variants):

| cell `id` | value | shows |
|---|---|---|
| `color#default` | `#9a6a30` (gold) | baseline swatch (always-valued) |
| `color#disabled` | `#2a6090` (blue) | disabled swatch (colour shown, inert) |

Matrix **29 → 31**.

---

## 6. CDP verification (Chat self-drives — sampler only, both accents via skin-swap)

Launch detached (`Start-Process run-sampler.ps1 -Debug -WindowStyle Minimized`); poll CDP 9422
(retry until non-null, N-044); `cdp-debug.ps1 -App sampler -Mode {state,eval,screenshot}`; clean up
(5175/9422 free, 0 orphans).

1. `ids().length === 31`; both `color#…` present.
2. **Baseline:** `color#default` → `{value:"#9a6a30"}` (always-string hex, never empty).
3. **Bind path:** dispatch a real `input` (a new hex, e.g. `"#123456"`, N-029) on `color#default`
   → bound getter updates (string round-trips through `bind:value`).
4. **Computed-style:** element `INPUT`, `type=color`, `appearance:none`/`-webkit-appearance:none`,
   the swatch box dims (border `--s5`, `--rad`).
5. **`.color` cascade:** all rules present incl. `::-webkit-color-swatch` + `…-swatch-wrapper`
   (stylesheet-rule inspection, N-042 method).
6. **Skin-swap:** `--accent2` flips gold (`--pr2`, client) ↔ blue (`--inf2`, node).
7. **Screenshot (both shells eye-checked):** the swatch renders the seeded colour; `#disabled`
   greyed/inert.

Quote **actual** CDP output in the JOURNAL (Rule 2); never invent (Rule 5).

---

## 7. Records (D-074; written after verification, Rule 4)

- `ui/docs/xgen-ui-notes.md` — **N-047** (build + the own-atomic-vs-date/range reasoning + the
  native-picker-not-skinnable point + the `color-picker` composite-#2 deferral).
- `docs/ROADMAP.md` — M-RP2.17 ✅ (tree node + RP narrative); version bump; same-commit with CLAUDE.
  Add the **`color-picker` composite #2** horizon to the deferred/composite line.
- `CLAUDE.md` — PLAY → M-RP2.17; prior-PLAY pointer J-425 → J-426.
- `ui/docs/xgen-ui-components.md` — promote the *color* row to built (M-RP2.17, J-426); **add a
  `color-picker` entry to the Composites section** (themed custom palette, deferred — the
  `password-field` shape).
- `JOURNAL.md` — **J-426** (newest-first; real CDP output).
- `tasks/M_RP2_17_COLOR.md` — Status → COMPLETED.
- **No `DECISIONS.md` touch** — applies D-096 (own atomic on the sharpened criterion, no amendment);
  the composite #2 is a logged horizon, not a decision.

`.md` header rule: `> **Last updated**:` carries ONLY the date.

---

## 8. Commit plan (two commits, UI pattern `feat` → `docs`; Joe pushes)

Multi-file discipline: write each file (Filesystem) + `get_file_info`-verify before the next;
`git add` per file; `git status` sanity; multi-`-m` commit.

**Commit 1 — implementation** (`color.svelte`, `skin.css`, `app_sampler.svelte`):

```powershell
cd E:\Projects\XGenProtocol
git add ui/core/lib/components/data-independent/color.svelte
git add ui/assets/skin.css
git add ui/sampler/src/app_sampler.svelte
git status
git commit -m "feat(ui): color - twelfth core component, native swatch atomic (M-RP2.17)" -m "Atomic <input type=color>, a singleton that stands alone (own atomic on disjoint skin/surface - the range case, not the textfield fold; applies D-096, no amendment). Always-valued string hex (#rrggbb, default #000000), getter {value}; leanest prop surface yet (value/disabled/id/name)." -m "Own .color swatch skin (::-webkit-color-swatch* pseudos, no --ctl-h); the open picker dialog is OS/Chromium-native, not skinnable - a themed palette is the deferred color-picker composite. Built/tuned/CDP-verified in the sampler (D-097): color row + 2 cells (matrix 29->31), string round-trip on the bind path, skin-swap accent gold<->blue."
git push
```

**Commit 2 — records** (`xgen-ui-notes.md`, `ROADMAP.md`, `CLAUDE.md`, `xgen-ui-components.md`,
`JOURNAL.md`, `M_RP2_17_COLOR.md`):

```powershell
cd E:\Projects\XGenProtocol
git add ui/docs/xgen-ui-notes.md
git add docs/ROADMAP.md
git add CLAUDE.md
git add ui/docs/xgen-ui-components.md
git add JOURNAL.md
git add tasks/M_RP2_17_COLOR.md
git status
git commit -m "docs(ui): close M-RP2.17 color - N-047, J-426, records" -m "N-047 (native swatch atomic + own-atomic-vs-date/range reasoning) + ROADMAP M-RP2.17 done + CLAUDE PLAY -> M-RP2.17 + components registry color row promoted." -m "Logged the color-picker composite #2 horizon (themed custom palette, deferred - the password-field shape). Task -> COMPLETED. No DECISIONS touch (applies D-096)."
git push
```

*(Final `git add` lists confirmed against `git status` at close.)*

---

## 9. Definition of Done

- [ ] `color.svelte` authored to §3 (always-string `#rrggbb`, default `#000000`, `{value}` getter,
      zero `<style>`).
- [ ] `.color` swatch skin added to §4 (swatch pseudos, no `--ctl-h`).
- [ ] Sampler row + `default`/`disabled` cells added (§5) and live.
- [ ] CDP verification §6 run in the sampler, both accents — actual output captured.
- [ ] N-047 written; ROADMAP + CLAUDE updated same-commit; components registry row promoted +
      `color-picker` composite #2 logged; JOURNAL J-426 written (real CDP output).
- [ ] Task Status → COMPLETED.

(`Status: COMPLETED` is the real signal — no "commit pushed" checklist item.)
