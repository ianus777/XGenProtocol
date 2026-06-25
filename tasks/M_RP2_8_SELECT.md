# M-RP2.8 — select (di·A, single-select, atomic `<select>`)
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jun 2026  
> **Last updated**: 2026-06-25  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Goal

Author the fourth `core` component — `select` (data-independent · single-select · atomic `<select>` · pick-only) — and **skin it in the same pass**. First author-and-skin-in-one-pass milestone (the L2 vocabulary now exists, M-RP2.7/N-033), and the first **content-carrying** di component (prior three were pure native-state; `<select>` carries `<option>` content).

## Locked (J-412 walk)

- **Q1 \u2014 options surface:** `options` prop, accepts `string[]` **or** `{value,label,disabled?}[]` (normalized internally to `{value,label,disabled}[]`); optional `placeholder` \u2192 a leading disabled `<option value="">`. Keeps the root atomic `<select>` (N-020) and the component data-*independent* (consumer passes a small static set, like a radio group's items); matches how the future dd layer will feed it. Sets the **content-carrying di precedent**.
- **Q2 \u2014 value / native-state:** `bind:value` (string) + `disabled` / `id` / `name` / `required`. No `multiple` (separate semantic/shape). Getter `() => $state.snapshot({ value })`.
- **Q3 \u2014 skin scope:** classic `appearance:none` + custom arrow on `.select`, assembled from the founded L2 tokens; the open option-list popup left native (OS/engine-rendered). `appearance:base-select` + `::picker(select)` (Chromium 135+) noted as a **future enhancement** once the pinned WebView2 version is confirmed \u2014 not depended on now.
- **Q4 \u2014 arrow:** inline-SVG `background-image` on `.select` itself (a wrapper would change the root off `<select>`, breaking N-020; `::after` on `<select>` is unreliable). Root stays `<select>`, L1 stays empty (holds the zero-`<style>` streak across all built components).
- **Q5 \u2014 verify:** driving `bind:value` over CDP needs a dispatched **`change`** event (bare `el.value=` won't fire Svelte's bind) \u2014 the N-029 `input`-event finding, restated for `change`.
- Demo `select` added to **both** shells.

## Phase-0 audit (state at open, 2026-06-24)

- di component dir `ui/core/lib/components/data-independent/` holds `toggle.svelte` / `button.svelte` / `textfield.svelte` + `.gitkeep`. **No `select` stub** anywhere in `ui` (excl. node_modules/backup/templates).
- Substrate unchanged: `use:envelope` (class + `data-debug-id` stamp + debug getter registration), `window.__XGEN_DEBUG__` (N-023/N-024). No substrate change needed \u2014 `options` is just a prop; envelope/debug are content-agnostic.
- L2 vocabulary available (N-033): `--s`/`--s2`/`--s5` surfaces, `--rad`, `--ctl-h`, `--sp-*`, accent-tinted `--focus-ring`, `--err`, `--motion`. `select` **assembles** from these (N-019 reuse applied to styling) \u2014 no new tokens expected beyond the arrow asset.
- `$core` alias wired in both shells (Vite); demo components imported from `$core` in `app_client.svelte` / `app_node.svelte`.

## Phase 1 — author `select.svelte`

`ui/core/lib/components/data-independent/select.svelte`. Root `<select use:envelope>`; `$props` = `{ value = $bindable(), options = [], placeholder, disabled, id, name, required }`; normalize `options` (strings \u2192 `{value:s,label:s}`; objects pass through, default `disabled:false`); render optional placeholder `<option value="" disabled>` then `{#each}` `<option>`; register the debug getter `() => $state.snapshot({ value })`. **No `<style>` block.**

## Phase 2 — wire demo into both shells

Add `<Select id="demo" bind:value={…} options={[…]} placeholder="…" />` to `app_client.svelte` + `app_node.svelte` (alongside the demo toggle/textfield/button), imported from `$core`. A local `$state` holds the bound value for live read-back.

## Phase 3 — skin `.select` in `skin.css`

`.select` keyed appearance assembled from L2: `--s` background, `1px solid var(--s5)` border, `var(--rad)`, `min-height var(--ctl-h)`, `var(--sp-1) var(--sp-2)` padding (with right padding for the arrow), `color var(--t)`, `font-size 12px`; `appearance:none` + `-webkit-appearance:none`; inline-SVG `background-image` arrow (right-aligned, `--t3`-toned, no-repeat); `:focus-visible` \u2192 accent border + `var(--focus-ring)`; `:disabled` \u2192 `--s2`/`--t4` grey + `not-allowed`; `:invalid` \u2192 `--err` border. No L1.

## Phase 4 — register + verify (Chat self-drives, N-028 working mode)

- Add the `select` row to `xgen-ui-components.md` **Built components**: `| select | core | A | data-independent · single-select | <select> | ui/core/lib/components/data-independent/select.svelte | () => $state.snapshot({ value }) | N-022/N-024/N-034 |`.
- Launch both apps (`tauri dev`, detached, **not** minimized); poll 9222/9322, retry `snapshot()` past the mount race.
- CDP: baseline `select#demo` `{value:""}` (or first option) \u2192 set value + dispatch a real `change` event \u2192 assert `{value:"<picked>"}` in **both** apps (the bind-in live-reactive read).
- Computed-style probe: `.select` skinned + `appearance:none`; `-Mode screenshot` both apps for eye-check.
- Clean teardown: ports 9222/9322/5173/5174 free, 0 orphans.

## Phase 5 — close (J-413, records-only)

`ui/docs/xgen-ui-notes.md` N-034 (select authored+skinned; first content-carrying di; options-prop precedent; the `change`-event verify note); `ui/docs/xgen-ui-components.md` registry row + detail note (v bump); `docs/ROADMAP.md` (RP node M-RP2.8 \u2705, frontier advance, version bump); `CLAUDE.md` PLAY (M-RP2.8 \u2705 CLOSED, Next \u2192 display-di `label`, entry pointer J-412\u2192J-413); this task file Status ACTIVE\u2192COMPLETED, DoD checked. Joe pushes.

---

## Definition of Done

- [x] `select.svelte` authored: atomic `<select>`, `options` prop (string[] | object[] normalized), `placeholder`, `bind:value`, native-state (`disabled`/`id`/`name`/`required`), debug getter `{ value }`, zero-`<style>`.
- [x] Demo `select` wired into both shells, imported from `$core`.
- [x] `.select` skinned in `skin.css` from L2 vocabulary + inline-SVG arrow; L1 still empty.
- [x] Both apps CDP-verified: baseline \u2192 dispatched `change` \u2192 `{value}` delta read live (both 9222/9322).
- [x] Both apps eye-verified + screenshots captured; computed-style confirms `.select` skinned.
- [x] Registry row landed in `xgen-ui-components.md`.
- [x] J-413 records landed (N-034, components registry, ROADMAP, PLAY, this file COMPLETED).
