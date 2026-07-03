# M-RP2.25 — `file-field` (4th di composite)
> **Status**: COMPLETED  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-03  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

Runbook for `file-field` — the **20th `core` component**, the **4th di composite** (after status-indicator N-054, password-field N-060, star-rating N-061). Fourth di-composite backlog pick (N-054). Design Joe-locked 2026-07-03 (Locks 1–3).

---

## Locked design

**Scope (Rule 6 flag).** The deferred spec (file.svelte comment) is "zone + selected-file list + remove + progress". Passive slice ONLY: **drop-zone + file-list display**. **No remove** (Lock 1 = A — FileList is immutable; remove pulls in a `File[]` model + `DataTransfer` write-back, tag-select territory — logged as follow-up). **Progress/upload = host I/O = widget-tier** (N-059), DEFERRED. file-field stays passive + FileList-native.

**Shape A — child-composite (Lock 2).** Composes the real `file` atomic as a **hidden** child input (`__input`, self-registers under the N-054 model) triggered by a styled drop-zone `<div>`, + a file-list display. Contrast to star-rating's Shape B. Composite exposes `bind:files` (FileList, forwarded from child); aggregate getter `{count, files:[{name,size,type}]}` (delegates to the child's de-FileList shape). Matrix: composite + hidden child = **2 entries/cell** → 3 cells → **+6** (68→74) — **confirm against real `ids()` at build**, don't assume.

**Drop + a11y (Lock 3).** Zone `dragover`/`dragenter` → `preventDefault` + `data-dragging="true"` (skin highlight); `dragleave`/`drop` clear it. `drop` writes `dataTransfer.files` into the hidden input's `.files` (respects `accept`/`multiple`; when `!multiple`, keep first, drop extras). Zone `role="button"` + `tabindex=0` + `aria-label`; Enter/Space → trigger the hidden input's picker. `disabled` drops all interaction (no dragging, no keyboard, no click), greyed.

**Props.** `files` ($bindable FileList|null, empty=null), `accept?`, `multiple?` (default false), `disabled?`, `id`, `name?`, `label?` (zone prompt, default "Drop files here or click to browse"). Getter `{count, files:[{name,size,type}]}`.

---

## Steps

**A — component.** `ui/core/lib/components/data-independent/file-field.svelte` (`lang="ts"`). Root `<div use:envelope={{name:'file-field',id,debug}}>`; hidden `<File bind:files id={cid('input')} .../>` (child atomic, self-registers); styled drop-zone div (drag handlers + role/tabindex/keydown); file-list render (`{#each files}` → name + size). Drop handler writes `input.files` via a `DataTransfer`. No `<style>`.

**B — skin.** `.file-field` block in `skin.css` (L2): drop-zone (dashed border, `--rad`, padding, `cursor:pointer`), `[data-dragging]` accent highlight (`--accent2` border/tint), file-list rows (`--fs-1`, `--t2`), `:focus-visible` affordance ring (N-055), `[aria-disabled]`/disabled greyed. Hidden input `display:none` (or visually-hidden but focusable — keep native picker reachable via the zone trigger). PROVISIONAL.

**C — sampler.** DI·composite panel, **3 cells**: `file-field#default`, `file-field#multiple` (`multiple`), `file-field#disabled`. Matrix 68→**74** (confirmed; child-composite +2/cell). Sampler import aliased `FileFieldComposite` (`file` atomic already holds `FileField`).

**+ drop-icon (approved touch-up, in-milestone).** Skin-only: outline folder + short centered down-arrow, `--drop` mask var on `.file-field`, `::before` on `.drop-zone` left of the label, fixed `--t3` (info-only, no accent on drag). No component change.

**D — CDP self-verify** (sampler 9422, both accents; real output, Rule 2):
- `ids().length===74` (confirm); aggregate `{count:0,files:[]}` baseline; child `file#…__input` present
- drop: build a `DataTransfer` with a File, dispatch `drop` on the zone → input.files set → aggregate `{count:1,files:[{name,size,type}]}` (+ `bind:files` reflects)
- `data-dragging`: dispatch `dragover` → `"true"`; `dragleave` → cleared
- `multiple`: `#multiple` child `multiple=true`; `!multiple` drop of 2 keeps 1
- a11y: zone `role=button`/`tabindex=0`/`aria-label`; Enter triggers (spy the input click)
- disabled: `#disabled` zone non-interactive, greyed
- computed colour: `[data-dragging]` border `--accent2` gold↔blue; `.file-field` rules in cascade
- screenshots both accents; teardown 0 orphans

**E — close (D-074 atomic, two commits).**
- **feat**: `file-field.svelte` + `skin.css` + sampler
- **docs**: N-062 (ui-notes v0.46) + registry v0.34 (20th core, 4th di composite) + ROADMAP v4.17 (M-RP2.25 ✅, next-active → `combobox`) + JOURNAL J-448 + CLAUDE PLAY + this runbook → COMPLETED

---

## Definition of Done
- [x] `file-field.svelte` built (Shape A, hidden child `file`, no `<style>`)
- [x] `.file-field` skin in `skin.css` (L2)
- [x] sampler 3 cells, matrix delta CDP-confirmed (68→74)
- [x] all D-block CDP proofs captured with real output (Rule 2)
- [x] both-accent screenshots eye-checked, 0 orphan ports
- [x] N-062 / registry v0.34 / ROADMAP v4.17 / J-448 / CLAUDE PLAY written
- [x] Status header → COMPLETED
