# M-RP2.24 — `star-rating` (3rd di composite)
> **Status**: COMPLETED  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-03  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

Runbook for `star-rating` — the **19th `core` component**, the **3rd di composite** (after `status-indicator` N-054, `password-field` N-060). Third di-composite backlog pick (N-054 list). Design Joe-locked 2026-07-03 (this session, Locks 1–3 + all-next).

---

## Locked design

**Shape (Decision 1 = B).** Self-contained `<div class="star-rating">` (composite by root-marker only, N-020/N-022). Renders internal star elements in an `{#each}` — **does NOT compose child atomic components**. Registers ONE aggregate getter; matrix multiplies **flat +1 per cell → +3** for 3 cells (65→68), unlike the child-self-register composites. Refines the composite definition: *a di-composite is a `<div class="type">` assembly; composing child atomics (status/password) is the common case, not a requirement.* → **D-069 promotion-watch** (definition-refinement; note only unless it recurs).

**di + passive.** Caller supplies `max`/`value`; interprets no domain structure (di). Hover-preview is transient presentational `$state` (like button's `:active`), not load/save/validate/host-I/O — clears the widget bar (N-059). Stays passive.

**Props / value (Lock 1).**
- `value: number` — `$bindable`, default `0` (= unrated), numeric bind-out
- `max: number` — default `5`
- `readonly?: boolean` — default `false` (show-a-rating, non-interactive)
- `disabled?: boolean` — default `false`
- `clearable?: boolean` — default `true` (click active star → 0)
- `id?`, `name?`, `ariaLabel?`
- Getter (N-024): `{ value, max }`

**a11y (Lock 2).** Root `role="radiogroup"` (+ `aria-label` from `ariaLabel`); each star `role="radio"` + `aria-checked={i === value}`; roving `tabindex` (active star / star 1 if unrated = `0`, rest `-1`). Keyboard: ←/↓ decrement, →/↑ increment (clamp 1..max), Home = 1, End = max. `disabled`/`readonly` drop interaction (no tabindex, no handlers).

**Hover-preview + clearable (Lock 3).** `hovered: number` transient `$state` (default `0`); star `mouseenter` sets preview, root `mouseleave` restores. Fill target = `hovered || value`. Click star `i`: if `clearable && i === value` → `0`, else → `i`. Suppressed when `readonly`/`disabled`.

**Glyph (all-next).** ★/☆ via currentColor `mask-image` placeholder (password-field eye pattern, N-052; scoped `--star` var, SVG placeholder until the `icon` primitive). Filled = `--accent2` (re-themes gold/blue), empty = `--t4`. Whole-star only v1; half-star average = future readonly shape.

---

## Steps

**A — component.** `ui/core/lib/components/data-independent/star-rating.svelte` (`lang="ts"`, single file). Root `<div use:envelope={{ name:'star-rating', id, debug }} role="radiogroup">`; `{#each Array(max)}` star `<span role="radio">` (or `<button>` if focus needs it — author's call, keep one root-marker div). No `<style>` (all appearance → skin). `$state.snapshot` the getter.

**B — skin.** `.star-rating` block in `skin.css` (L2 only): flex row, `gap: var(--sp-1)`; star sizing (~20px), `mask` fill via `--star`, filled `--accent2` / empty `--t4`; `:focus-visible` = affordance accent ring (N-055 — stars *act*); `disabled`/`readonly` greyed + `cursor` off. No new `:root` token if avoidable; PROVISIONAL.

**C — sampler.** DI·composite panel, **3 cells**: `star-rating#default` (`value:0`, `max:5`), `star-rating#rated` (`value:3`), `star-rating#readonly` (`value:4`, `readonly`). Matrix **65→68** (Shape B = 1 entry/cell → +3).

**D — CDP self-verify** (sampler 9422, both accents via skin-swap; real output, Rule 2):
- `ids().length===68`; `#default {value:0,max:5}`, `#rated {value:3,max:5}`, `#readonly {value:4,max:5}`
- click star 4 on `#default` → getter `{value:4}` (set delta); click star 4 again → `{value:0}` (clearable)
- keyboard: focus `#rated`, `ArrowRight`→4, `ArrowLeft`→2, `Home`→1, `End`→5
- hover star 5 on `#default` → preview fill = 5 stars while `value` getter unchanged; `mouseleave` → restores
- a11y: root `role=radiogroup`, star `role=radio` + `aria-checked` on the value star; `#readonly` non-interactive (no tabindex / click no-ops)
- computed colour: filled star `--accent2` gold `rgb(194,136,64)` ↔ blue `rgb(58,122,176)`; empty `--t4`; `.star-rating` rules in cascade
- screenshot eye-check (both accents); teardown 0 orphans

**E — close (D-074 atomic, two commits).**
- **feat**: `star-rating.svelte` + `skin.css` + sampler `app_sampler.svelte`
- **docs**: N-061 (ui-notes v0.44) + components registry v0.33 (19th core, 3rd di composite) + ROADMAP v4.16 (RP node M-RP2.24 ✅, next-active → `file-field`) + JOURNAL J-447 + CLAUDE PLAY (next-active flip) + this runbook → COMPLETED

---

## Definition of Done
- [x] `star-rating.svelte` built (self-contained, no child atomics, no `<style>`)
- [x] `.star-rating` skin in `skin.css` (L2)
- [x] sampler 3 cells, matrix 65→68 CDP-confirmed
- [x] all D-block CDP proofs captured with real output (Rule 2)
- [x] both-accent screenshots eye-checked, 0 orphan ports
- [x] N-061 / registry v0.33 / ROADMAP v4.16 / J-447 / CLAUDE PLAY written
- [x] Status header → COMPLETED
