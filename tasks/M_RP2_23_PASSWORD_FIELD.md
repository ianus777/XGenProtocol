# M-RP2.23 — `password-field` (2nd di composite) — Runbook
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jul 2026  
> **Last updated**: 2026-07-02  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

Phase-0 audit closed against `textfield.svelte` + `button.svelte` + `status-indicator.svelte`. Design Joe-locked. No code until "go" on §3.

---

## 1. What this is

The **second di composite** (after `status-indicator`, N-054) and the home for the reveal toggle deferred at the `textfield` `type` fold (M-RP2.12/D-096). Root `<div class="password-field">` composes built atomics; owns `revealed` + `capsLock`; di (no domain interpretation). D-069 promotion-watch: 2nd composite — note if the N-054 registration model holds clean; no promotion yet.

## 2. Step A — `textfield` additive (own commit, first)

Two additive props on the closed atomic (default-absent → existing 44-cell registry unchanged, D-065):

- `redactValue?: boolean = false` — getter becomes `() => $state.snapshot({ type, value: redactValue ? null : value })`. password-field sets it true so the child never publishes the live secret into `window.__XGEN_DEBUG__`.
- `autocomplete?: string` — native `<input autocomplete>` pass-through (general: email/url/tel benefit too). Rendered `autocomplete={autocomplete || undefined}`.

DoD-A: build clean; sampler re-verify the existing textfield cells register `{type, value}` unchanged (redact off), zero matrix change.

## 3. Step B — build `password-field` (di composite, N-054 pattern)

Path: `ui/core/lib/components/data-independent/password-field.svelte`. Mirror `status-indicator` structure (root `<div use:envelope>`, imported children, `cid()` stable ids, aggregate getter, `{#if}` optional child).

**Composes:**
- `textfield` `__field` — `type={revealed ? 'text' : 'password'}`, `redactValue`, `bind:value`, `{placeholder}` `{disabled}` `{readonly}` `{name}` `{autocomplete}`.
- `button` `__reveal` — `mode="toggle"`, `bind:pressed={revealed}`, empty label + skin glyph, `ariaLabel={revealed ? 'Hide password' : 'Show password'}`.
- `label` `__capswarn` — `{#if capsLock}`, `text="Caps Lock is on"` (optional-child pattern, like status-indicator's link).

**Owns:** `revealed` (`= revealedByDefault`), `capsLock`.

**Caps-lock (composite-level, no textfield touch):** keyboard events bubble from the inner `<input>` to the wrapper `<div>`; attach `onkeyup`/`onkeydown`, set `capsLock = e.getModifierState?.('CapsLock') ?? false`.

**Props:** `value` ($bindable '') · `placeholder` · `disabled` · `readonly` · `id` · `name` · `autocomplete?` · `revealedByDefault?` (default false).

**Getter:** `() => ({ revealed, hasValue: value.length > 0, capsLock })` — boolean `hasValue`, never the value.

**Matrix:** +3 per cell (composite + `__field` + `__reveal`); `__capswarn` registers only while caps active.

## 4. Step C — skin (`skin.css`, L2)

`.password-field` = flex row (the `.status-indicator`/`.select` precedent). Reveal glyph via `.password-field button::before`, eye↔eye-off keyed off `aria-pressed` (reflected in toggle-mode). Caps-lock warn styling on the `label` child. Focus per N-055: field is editable → `--t3`; reveal button is an affordance → `--focus-ring` (both inherited from `.textfield`/`.button`). Aim: no new `:root` token. PROVISIONAL (Joe live-tunes via HMR).

## 5. Step D — sampler + CDP verify (D-097)

Add a `password-field` row to the sampler DI·composite panel. Cells: `#default`, `#disabled`, `#revealed` (revealedByDefault). CDP (both accents, fresh launch, real output, Rule 2):
- matrix +N; aggregate `{revealed,hasValue,capsLock}` + child getters under stable ids.
- reveal flips inner `type` password↔text (dispatched click on `__reveal`, read `__field` el.type).
- `redactValue` proof: type into `__field`, dispatched `input` → `__field` getter `value` is **null** while `bind:value` still carries the real string to the composite `hasValue:true`.
- caps-lock: dispatch a `keyup` with `getModifierState` shimmed → `capsLock:true`, `__capswarn` appears.
- skin rules in cascade (stylesheet-rule inspection, N-042); `aria-pressed` glyph swap; screenshots both accents.

## 6. Step E — close (one atomic, D-074)

N-060 (ui-notes) + components-registry bump (18th `core` component, 2nd di composite) + ROADMAP (RP node, M-RP2.23 ✅) + JOURNAL + CLAUDE PLAY (next-active → next di-composite from the N-054 backlog). All `.md` headers updated per the mandatory header spec. Two-commit shape acceptable (feat Step A+B+C, then docs close) or single atomic — Joe's call at close.

## 7. Logged / out of scope

- **Confirm-password match** → future `password-confirm` composite (di, wraps two fields; equality-check leans dd). Not built.
- Strength meter → future dd component (interprets the value).
- Real eye `icon` primitive (N-052) → glyph is skin `::before` for now; upgrade when `icon` lands.

## 8. DoD

- [x] Step A: `redactValue` + `autocomplete` additive; existing cells non-regressive (matrix unchanged, redact off).
- [x] Step B: `password-field` built; reveal binds `pressed→revealed`; caps-lock at composite level; getter leaks no value.
- [x] Step C: `.password-field` skin; glyph swap on `aria-pressed`; focus per N-055; no new token (or noted).
- [x] Step D: sampler cells + CDP proofs (reveal type-flip, redact-null, caps-lock, aggregate) both accents; screenshots.
- [x] Step E: N-060 + registry + ROADMAP + JOURNAL + CLAUDE PLAY atomic; headers updated.

## 9. As-built delta (revision round, Joe cosmetic requests)

The §3–§5 design was refined during build (all CDP-verified):
- Caps-lock warning is NOT a `label __capswarn` child — the child + import were dropped; the wrapper reflects `data-caps`, skin gives the field a red `--err-bright` border **and** an overlaid `::after` "Caps Lock is on!" hint (absolute, no reflow). Matrix is a flat **+9** (65), no conditional child.
- Reveal button is **transparent, icon-only** (no chrome): eye / eye-off via scoped `--eye`/`--eye-off` currentColor `mask-image`, swapped on `aria-pressed` (placeholder until the `icon` primitive, N-052). 18px, 3px gap.
- The N-039 password `***` inset is suppressed inside the composite; its reserved `padding-right:24px` normalized to `--sp-2` so the field width is identical password↔text (155/155, jump 0).
- Getter `{revealed, hasValue, capsLock}` unchanged.
