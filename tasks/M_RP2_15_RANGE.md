# M-RP2.15 — `range` (di·A, atomic `<input type="range">`, bounded numeric slider)
> **Status**: COMPLETED  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-27  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Goal

Author the tenth `core` component `range` (di·A, atomic, root `<input type="range">`): bounded numeric, slider/drag, **numeric** `bind:value`. The next atomic di after `number` per the locked N-038 track order (catalogue row *numeric (bounded)*). Mechanically the same `<input>` root as `number`, and the same value-type (number) — so by the *literal* D-096 criterion (same root + same value-type) it would fold into `number`. The walk rejects that: D-096's criterion is **necessary but not sufficient**. The textfield fold was good because the family was *genuinely interchangeable* (one skin, one prop surface, a thin `type` switch). `range` shares root + value-type with `number` but diverges on **skin** (track/thumb pseudo-elements — zero shared appearance), **prop surface** (no `placeholder`, no live `:invalid`, no `readonly`, bounds are the defining attribute), and **interaction/empty model** (clamped drag, always-valued). Folding would put two disjoint skins behind one class and a prop that swaps the whole rendering — the polymorphic-contract problem D-096 exists to prevent, on the *appearance* axis. So `range` stays its own atomic, and the fold criterion is **sharpened**.

## Locks (design walk, Joe-locked 2026-06-27 "locked, go ahead" — Q1–Q5 by recomms)

1. **Own atomic, not a `number`/textfield fold.** Own component `ui/core/lib/components/data-independent/range.svelte`, root `<input type="range" use:envelope>`. Distinct from `number` on **skin + prop surface + interaction model**, not value-type (which it shares). **Criterion sharpened:** the fold test is root + value-type **+ shared skin/surface** (genuine interchangeability), not value-type alone. Recorded in **N-042**; **D-096 gains a one-line sharpening clause** (`range` is the first case that tests sufficiency — the criterion as written would mislead a future reader into folding it; this is a refinement of the decision, not just an application). DECISIONS.md *is* touched this milestone (light amendment), in contrast to M-RP2.13/M-RP2.14.
2. **Value — always present, never `null`; default `0`.** `value = $bindable(0)` typed `number` (not `number | null`). A range is always valued (the clean divergence from `number`'s empty=`null`). `0` is deterministic; mid `(min+max)/2` needs both bounds read at init (machinery) for marginal benefit (D-065). **No clamping in the atomic:** if a consumer sets `min > 0`, they pass an in-range initial — the documented mount-desync-if-`min>0` is a consumer responsibility, exactly as `number` does not clamp. The actual baseline runtime value is **verified at CDP and reported honestly** (expectation: the demo's initial, a JSON number).
3. **Prop surface** — control vocabulary, slider-shaped:
   - **Keep:** `value` (numeric $bindable, default `0`), `min`, `max`, `step` (optional numbers; native defaults 0/100/1 documented — bounds are the *defining* attribute), `disabled`, `id`, `name`.
   - **Drop:** `placeholder` (slider has no empty text state), `pattern` (already dropped for number), **`readonly`** (native no-op on `type=range` — a divergence from `number` worth recording), **`:invalid`** from the skin (dead — the thumb is clamped, can never be out of range).
   - **`maxlength` stays OUT** (mirrors the family).
4. **Getter → `{ value }`** — single field, value-only (`min`/`max`/`step` are static config). Always a `number` (never `null`).
5. **Skin — own `.range` key; first pseudo-element-heavy skin (new territory, PROVISIONAL — Joe eye-checks).**
   - `appearance: none` + `-webkit-appearance: none` on the input, then style `::-webkit-slider-runnable-track` + `::-webkit-slider-thumb`. Vendor-prefixed is fine — single-engine WebView2/Chromium (the same justification as the toggle switch `::before` and the select arrow).
   - **Track** = a thin rounded groove from `--s5`; **thumb** = `var(--accent, var(--pr))` → per-shell gold (client) / blue (node), so the handle reads as the live control (matches the switch thumb language).
   - **Focus ring on the thumb** via `:focus-visible` → reuse `var(--focus-ring)`.
   - **No `--ctl-h`** (track ~4px / thumb ~16px — `.textarea` likewise dropped it); literal dims in `.range` like the switch pill (40/22/16). **No new `:root` token.**
   - **Accent fill DEFERRED** (the tinted portion left of the thumb): WebKit gives no free fill — it needs a value-driven `linear-gradient`/JS. v1 = plain track + accent thumb; the gradient-fill is a future skin shape (D-065).
   - `:disabled` → greyed thumb/track + `cursor: not-allowed`.
6. **`use:envelope` unchanged** — content-agnostic substrate reused verbatim; confirms generalization onto a range-bound `<input>`. Zero `<style>` (L1 empty) — all appearance is skin.
7. **Processor seam — N/A.** `range` is a bounded numeric drag, not free-text/free-number entry — the numeric-formatting processor consumer is `number`, not `range` (no typed digits to reformat). No `use:processor` seam reserved here; this is NOT a third defer-per-consumer instance.

## Phases

**Phase 1 — author** `ui/core/lib/components/data-independent/range.svelte`:
- Root `<input type="range" use:envelope={{ name: 'range', id, debug }} bind:value {disabled} {min} {max} {step} {name}>`.
- `$props()`: `value = $bindable(0)`, `disabled = false`, `min`, `max`, `step`, `id`, `name` — TS types: `value?: number`, `min`/`max`/`step?: number`, `disabled?: boolean`, `id`/`name?: string`. (No `placeholder`/`readonly`/`pattern`/`type`.)
- Getter `debug = () => $state.snapshot({ value })`.
- **Header comment** modelled on `number.svelte`'s: atomic root `<input type="range">`; why it is NOT a `number`/textfield fold (the *sharpened* criterion — shares root + value-type with `number` but diverges on skin/surface/interaction; → D-096 clause + N-042); always-valued (never `null`, default `0`, no clamping — consumer responsibility); native thumb/track is the affordance (the custom +/− stepper is the `number` composite track, not this); **no processor seam** (bounded drag, no typed entry to reformat).
- Zero `<style>`.

**Phase 2 — skin** `skin.css`, new `.range` block (after `.number`, before `.select`):
- `.range { appearance: none; -webkit-appearance: none; width: <literal>; background: transparent; cursor: pointer; }` (transparent so the track pseudo-element owns the groove).
- `.range::-webkit-slider-runnable-track { height: <~4px>; border-radius: <pill>; background: var(--s5); }`
- `.range::-webkit-slider-thumb { appearance: none; -webkit-appearance: none; width: <~16px>; height: <~16px>; border-radius: 50%; background: var(--accent, var(--pr)); border: 1px solid var(--accent2, var(--pr2)); margin-top: <centering offset>; }`
- `.range:focus-visible { outline: none; box-shadow: var(--focus-ring); }` (ring on the control; if the thumb-specific ring reads better, move to `::-webkit-slider-thumb:focus` at eye-check — provisional).
- `.range:disabled { cursor: not-allowed; }` + greyed thumb (`.range:disabled::-webkit-slider-thumb { background: var(--s4); border-color: var(--s5); }`) + dimmed track.
- **No** `:invalid`, **no** `--ctl-h`, **no** icon rules, **no** accent fill, **no** new `:root` token. Exact literal dims decided at implement + eye-checked (PROVISIONAL).

**Phase 3 — wire demo, both shells** (`app_client.svelte` + `app_node.svelte`):
- `let demoRange = $state(50)` (bare — the shells are plain JS, no TS annotation; N-041 gotcha).
- Mount `<Range bind:value={demoRange} id="demo" min={0} max={100} step={1} />` beside the other demo di. Import `Range` from `$core`. (If the bundler/linter objects to the name, alias at import and flag in records — mirrors the `number`→`NumberField` shadowing fix.)

**Phase 4 — CDP verify both apps** (Chat self-drives, real `tauri dev` + CDP; N-028 race-retry + clean teardown):
- Registry baseline both apps: `range#demo` → record the **actual** value (expect `{value:50}`); assert `typeof === "number"` (a JSON number, not a string) — the always-valued baseline.
- Set `el.value` + dispatch a real **`input`** event (N-029; range fires `input` on drag) → registry delta to a new **number** (e.g. `75` client / `25` node, `typeof number`, not `"75"`) — the number round-trip + the live-reactive-read on the slider path.
- Computed-style **`.range`**: tag `INPUT`, `type === "range"`, `appearance: none`.
- Computed-style **pseudo-elements** (separate CDP task — N-041 finding): `getComputedStyle(el, '::-webkit-slider-thumb')` → `background-color` = per-shell accent (client gold `--pr` `rgb(154,106,48)` / node blue `--inf` `rgb(42,96,144)`); `getComputedStyle(el, '::-webkit-slider-runnable-track')` → present, `background-color` = `--s5` `rgb(52,59,71)`.
- Screenshots both apps — eye-check the slider renders: track groove + accent thumb, per-shell colour, per-shell chrome.
- Clean teardown (ports 9222/9322/5173/5174 free, 0 orphans).

**Phase 5 — records (D-074 atomic, same-commit):**
- `DECISIONS.md` **D-096** — one-line sharpening clause: the fold criterion is root + value-type **+ shared skin/surface** (genuine interchangeability); `range` named as the case that tests sufficiency and stays its own atomic. `Last updated` bumped.
- `ui/docs/xgen-ui-notes.md` **N-042** — `range` built; own atomic + the sharpened criterion (→ D-096 clause); always-valued/default-0/no-clamp; the pseudo-element skin (first of its kind, provisional, accent thumb, fill deferred); `readonly`/`:invalid` dropped; the CDP number round-trip + pseudo computed-style verify (+ the separate-task pseudo read).
- `ui/docs/xgen-ui-components.md` (v0.19→**0.20**) — Built `range` row (Tier `core`, Phase A, `data-independent · numeric (bounded)`, root `<input type="range">`, getter `{value}`, ref `N-022/N-024/N-038/N-042`); add a `range` detail paragraph; the di-catalogue *numeric (bounded)* row gets a build-note (served by the built `range`; accent-fill shape deferred).
- `docs/ROADMAP.md` (v3.99→**4.00**) — RP node M-RP2.15 ✅ + both chains + Present clause + frontier M-RP2.14→M-RP2.15.
- `CLAUDE.md` PLAY → M-RP2.15 ✅ CLOSED; Next → `date` (next atomic di per N-038); pointer J-419→J-420.
- This task → **COMPLETED**.
- `JOURNAL.md` **J-420** (written last, Rule 4, real CDP output quoted).
- All touched `.md` `Last updated` bumped to close date.

## Definition of Done

- [x] `range.svelte`: root `<input type="range" use:envelope>`; props `value`($bindable `number`, default `0`)/`min`/`max`/`step`/`disabled`/`id`/`name`; no `placeholder`/`readonly`/`pattern`/`type`; getter `{value}`; header comment (sharpened criterion + always-valued/no-clamp + native-thumb affordance + no processor seam); zero `<style>`
- [x] `skin.css`: new `.range` block — `appearance:none` + `::-webkit-slider-runnable-track` (groove `--s5`) + `::-webkit-slider-thumb` (accent fill, per-shell); `:focus-visible` ring; `:disabled` greyed; no `:invalid`, no `--ctl-h`, no icons, no accent-fill, no new `:root` token (provisional dims 160/4/16px, eye-checked)
- [x] demo wired both shells (`<Range bind:value={demoRange} id="demo" min/max/step>` + `let demoRange = $state(50)` bare + `$core` import) — imported as `Range` (no shadowing fix needed)
- [x] CDP both apps: `range#demo` in `ids()`; baseline `typeof number`, always-valued (node `{value:50}`; client read `{value:56}` — a stray, re-driven cleanly), real output in J-420
- [x] CDP both apps: dispatched `input` → registry carries a JSON **number** (`typeof number`, `75`/`25`), not a string — number round-trip on the slider path proven (real output)
- [x] CDP both apps: computed-style `.range` — tag `INPUT`, `type=range`, `appearance:none`, `160px` (real output)
- [x] CDP both apps: pseudo-element skin — `getComputedStyle` returns UA defaults on `::-webkit-slider-*` (shadow-pseudo limitation, FINDING); verified instead via **stylesheet-rule inspection** (all 7 `.range` rules parsed + in cascade) + screenshot (real output)
- [x] screenshots both apps: slider renders — track + per-shell accent thumb (gold client ~75% / blue node ~25%) + per-shell chrome
- [x] clean teardown (ports 9222/9322/5173/5174 free, 0 orphans)
- [x] records updated (DECISIONS.md D-096 amendment, N-042, components v0.20 + catalogue build-note, ROADMAP v4.00, CLAUDE PLAY, JOURNAL J-420, task COMPLETED)
