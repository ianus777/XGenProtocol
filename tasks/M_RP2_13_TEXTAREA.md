# M-RP2.13 — `textarea` (di·A, atomic `<textarea>`, multi-line free-text)
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

Author the eighth `core` component `textarea` (di·A, atomic, root `<textarea>`): multi-line free-text, string `bind:value`. The next atomic di per the locked N-038 track order. Root tag is `<textarea>`, not `<input>` → by the N-020 root-tag discriminator this is a **new atomic component, NOT a `textfield` fold**. It is the **edit-side multi-line counterpart** to `paragraph`'s render-side single prose string: `paragraph` wraps one read-only string visually (text node), `textarea` holds literal `\n`-bearing editable free text. Same string bind-in path as `textfield`/`select`. Authored + skinned in one pass (the L2 vocabulary exists).

## Locks (design walk, Joe-locked 2026-06-26 "all by your recomms")

1. **Stand-alone atomic, not a textfield fold** — own component `ui/core/lib/components/data-independent/textarea.svelte`, root `<textarea use:envelope>` (N-020). Shares the string-input vocabulary with `textfield` but is a distinct component (distinct root tag).
2. **Prop surface — parity with `textfield` minus what `<textarea>` can't carry, plus `rows`:**
   - **Keep:** `value` (`$bindable('')`, string `bind:value`), `placeholder`, `disabled`, `readonly`, `id`, `name`.
   - **Drop:** `type` (no such attribute on `<textarea>`); **`pattern`** (`<textarea>` has no native `pattern` — `<input>`-only; so no `:invalid`-via-pattern path here).
   - **Add:** `rows` (numeric, **default `3`**) — the one genuinely textarea-specific prop; sets initial visible height.
   - **`maxlength` stays OUT** — mirrors `textfield`'s deliberate omission (orthogonal native-state addition, not part of the atomic). Out of scope.
3. **auto-grow is a deferred skin shape, NOT built** — the catalogue lists "textarea · auto-grow" as shape variants. Single-engine WebView2/Chromium gives a pure-CSS path (`field-sizing: content`, Chromium 123+) — **note it as the future auto-grow skin shape, build nothing** (D-065, no empty machinery; same posture as `select`'s `appearance:base-select` deferral). The atomic ships native fixed-`rows` + vertical resize.
4. **Processor seam — DEFERRED; `textarea` ships processor-READY, NOT the trigger.** Two locked grounds:
   - **N-038 sequence is locked:** *finish ALL atomic di → text-processor engine (own arc, all consumers in hand) → dd.* `textarea` is not the last atomic (`number`/`range`/`date`/`color`/`file`/`select multiple` follow). Building the engine here jumps the order and over-fits the seam to one consumer when N-038 names three (textarea edit-side, number formatting, paragraph render-side `use:render`).
   - **D-065:** the *atomic* `textarea` is function-complete without any processor — exactly as `textfield` shipped processor-ready, not processor-bearing. The processor is an opt-in layered `use:processor={config}` action, not part of the atom; its security surface (allowlist + sanitizer, ReDoS guard, named `common` configs) deserves its own arc/walk.
   - **Action:** reserve the **edit-side `use:processor`** insertion point in the header comment (the counterpart to `paragraph`'s render-side `use:render`). Build nothing. The N-038 "earliest natural trigger" line is a candidacy note, not a commitment; the locked trigger is "all atoms done."
5. **Getter → `{ value }`** — single field. `rows` is static config, not user-mutable state (`textfield` did not snapshot `placeholder`); value-only matches `select`/`textfield`-original. Read-only-of-config is not registry state.
6. **Skin — own `.textarea` key, assembling the same L2 vocabulary** as `.textfield` (`--s`/`--s5`/`--rad`/`--t`/`--fs-1`/`--lh`/padding/`:focus-visible`/`:disabled`/`:read-only`). Precedent: `.select` got its own block rather than grouping with `.textfield` (N-025 keys appearance per type-class; per-class clarity > DRY in the single removable layer). Differences from `.textfield`: **no `min-height: --ctl-h`** (a textarea isn't single-control-height — let `rows` drive height); **`resize: vertical`** (horizontal would break the flex-column width); **no per-type icon machinery**; **no `:invalid`/`pattern` rule**.
7. **`use:envelope` unchanged** — content-agnostic substrate reused verbatim; confirms generalization across the textfield→textarea tag change. Zero `<style>` (L1 empty) — all appearance is skin.

## Phases

**Phase 1 — author** `ui/core/lib/components/data-independent/textarea.svelte`:
- Root `<textarea use:envelope={{ name: 'textarea', id, debug }} bind:value {placeholder} {disabled} {readonly} {rows} {name}></textarea>`.
- `$props()`: `value = $bindable('')` (string), `placeholder = ''`, `disabled = false`, `readonly = false`, `rows = 3`, `id`, `name` — TS types matching `textfield` (minus `type`/`pattern`, plus `rows: number`).
- Getter `debug = () => $state.snapshot({ value })`.
- **Header comment block** modelled on `textfield.svelte`'s: atomic root `<textarea>`; the edit-side multi-line free-text semantic (N-022); string bind-in path; native-state surface; **the reserved edit-side `use:processor` seam** (counterpart to paragraph's render-side `use:render`, EDIT-vs-RENDER axis N-032) — processor-ready, NOT built (D-065); the `field-sizing: content` auto-grow shape noted as future skin, not built.
- Zero `<style>`.

**Phase 2 — skin** `skin.css`, new `.textarea` block (after `.textfield`, before `.select`):
- Assemble from L2: `padding: var(--sp-1) var(--sp-2)`, `margin: var(--sp-1) var(--sp-2)`, `background: var(--s)`, `border: 1px solid var(--s5)`, `border-radius: var(--rad)`, `color: var(--t)`, `font-size: var(--fs-1)`, `line-height: var(--lh)`, `transition: border-color var(--motion), box-shadow var(--motion)`.
- **`resize: vertical`** (horizontal disabled — flex-column width).
- `.textarea::placeholder { color: var(--t4); }`
- `.textarea:focus-visible { outline: none; border-color: var(--accent2, var(--t3)); box-shadow: var(--focus-ring); }`
- `.textarea:disabled { background: var(--s2); color: var(--t4); cursor: not-allowed; }`
- `.textarea:read-only { background: var(--s2); border-color: var(--s4); }`
- **No** `min-height: --ctl-h`, **no** `:invalid`, **no** icon rules, **no** new `:root` token.

**Phase 3 — wire demo, both shells** (`app_client.svelte` + `app_node.svelte`):
- `let demoTextarea = $state('')` (throwaway, with the M-RP2.13 comment).
- Mount `<Textarea bind:value={demoTextarea} id="demo" placeholder="Multi-line…" />` beside the other demo di. Import `Textarea` from `$core`.

**Phase 4 — CDP verify both apps** (Chat self-drives, real `tauri dev` + CDP; N-028 race-retry + clean teardown):
- Registry baseline: `textarea#demo` → `{value:""}` (both apps).
- Set value + dispatch **`input`** (textarea fires `input`, not `change` — N-029 dispatched-event subtlety) with a **newline-bearing** string (`"line one\nline two"` client / a distinct one node) → `{value:"line one\nline two"}` → proves the multi-line string (literal `\n`) round-trips on the bind-in path — the thing distinguishing it from `textfield`.
- Computed-style probe: tag `TEXTAREA`; `font-size` = `--fs-1` (12px), `color` = `--t` (rgb 236,233,225), `resize: vertical`; `border-radius` 6px.
- Screenshots both apps — eye-check the multi-line box renders + the vertical resize grabber is present + per-shell chrome.
- Clean teardown (ports 9222/9322/5173/5174 free, 0 orphans).
- N-039 caveat does not bite (no `type` mutation to fight the bind round-trip); synchronous reads + one dispatched `input` are safe.

**Phase 5 — records (D-074 atomic, same-commit):**
- `ui/docs/xgen-ui-notes.md` **N-040** — `textarea` built; stand-alone atomic (not a fold); prop surface (drops `type`/`pattern`, adds `rows`); the edit-side `use:processor` seam reserved + the explicit **processor-deferred / not-the-trigger** decision (N-038 sequence + D-065); auto-grow as future `field-sizing` skin shape; the `\n` round-trip verify.
- `ui/docs/xgen-ui-components.md` (v0.17→**0.18**) — Built `textarea` row (Tier `core`, Phase A, class·semantic *display-kind*? **NO — interactive di**, free-text multi-line; root `<textarea>`; getter `{value}`; ref `N-022/N-024/N-038/N-040`); add a `textarea` detail paragraph; the di-catalogue *free-text (multi line)* row now has its built component (note, like the D-096 build-note pattern).
- `docs/ROADMAP.md` — RP node M-RP2.13 ✅ + chain + frontier advance M-RP2.12→M-RP2.13; version bump.
- `CLAUDE.md` PLAY → M-RP2.13 ✅ CLOSED; Next → remaining atomic di (`number` next per N-038); pointer J-417→J-418.
- This task → **COMPLETED**.
- `JOURNAL.md` **J-418** (written last, Rule 4, real CDP output quoted).
- All touched `.md` `Last updated` bumped to close date.
- **No `DECISIONS.md` touch** — no global decision lands here (the processor-defer is the *application* of the existing N-038 sequence + D-065, not a new principle; auto-grow-deferral is arc-local). If the processor-defer-at-each-consumer pattern recurs to the four-recurrence bar later, it graduates then (D-069).

## Definition of Done

- [x] `textarea.svelte`: root `<textarea use:envelope>`; props `value`($bindable string)/`placeholder`/`disabled`/`readonly`/`rows`(=3)/`id`/`name`; no `type`, no `pattern`; getter `{value}`; header comment (edit-side `use:processor` seam reserved + processor-deferred rationale + auto-grow `field-sizing` noted); zero `<style>`
- [x] `skin.css`: new `.textarea` block assembled from L2 vocabulary; `resize: vertical`; placeholder/focus/disabled/read-only states; no `min-height: --ctl-h`, no `:invalid`, no icons, no new `:root` token
- [x] demo wired both shells (`<Textarea bind:value={demoTextarea} id="demo">` + `demoTextarea` state + `$core` import)
- [x] CDP both apps: `textarea#demo` `{value:""}` baseline (real output in J-418)
- [x] CDP both apps: dispatched `input` with a `\n`-bearing string → `{value:"line one\nline two"/<node string>}` — multi-line round-trip proven (real output)
- [x] CDP both apps: computed-style — tag `TEXTAREA`, `--fs-1`/`--t`, `resize: vertical`, radius 6px (real output)
- [x] screenshots both apps: multi-line box + vertical resize grabber + per-shell chrome
- [x] clean teardown (0 orphan ports)
- [x] records updated (N-040, components v0.18 + catalogue build-note, ROADMAP, CLAUDE PLAY, JOURNAL J-418, task COMPLETED); no DECISIONS.md touch
