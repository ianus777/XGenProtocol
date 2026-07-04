# M-RP2.30 — `meter` (display-di, bounded read-only value bar)
> **Status**: COMPLETED  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-04  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

The **26th `core`** and the **5th simple display-di** (after label/paragraph/image/led). Root = native `<meter>`. The read-only sibling of `range` (range = editable numeric-in; meter = read-only value-against-range-out). **di, not dd** — binds a plain `{value,min,max,optimum?}`, interprets no domain structure. Resumes the di-atomic series.

## Locked design (Joe, by-recomms)

1. **Surface** — bare `<meter>`, no built-in label/readout (the consuming composite/widget adds caption + value text; atomics carry none — the range/number rule). Getter `{value, min, max, optimum}`.
2. **Prop surface** — `value` / `min` (def 0) / `max` (def 1, native default) / `optimum?` / `low?` / `high?` / `width?` / `disabled?` / `id` / `name`. `value` is a plain prop (read-only display-di, not `$bindable`). `low`/`high`/`optimum` drive the native semantic fill.
3. **Width** — **full-width default** (`display:block; width:100%`, fills the container; unlike `range`'s pinned 160px). Optional `width?` prop pins a fixed width (`"200px"`/`"12rem"`, inline override). `min-width` floor ~80px so a narrow parent can't crush it. Simpler than tag-select's system — a meter has nothing to collapse.
4. **Semantic fill + new `--warn` token** — native picks the pseudo by optimum position: in-band → green (`--ok`), adjacent → amber, far → red (`--err`). XGen has no amber → **found `--warn`** in L2 (`#ba7517`, tune via HMR; reused later for form caution states). No `optimum` set → single neutral fill (`--t3`), not green.
5. **Skin** — pseudo-heavy `.meter` (PROVISIONAL): `::-webkit-meter-bar` track (`--s5` groove, radius) + `::-webkit-meter-optimum-value` (`--ok`) / `::-webkit-meter-suboptimum-value` (`--warn`) / `::-webkit-meter-even-less-good-value` (`--err`) fills; disabled dims (opacity). Verified by stylesheet-rule inspection + screenshot (N-042 method — `getComputedStyle` won't read shadow-pseudos; the `range` precedent).
6. **Classification / milestone** — 5th simple display-di; **M-RP2.30**; di track (does NOT open dd). 26th `core`.

## Build steps (D-071 arc)

- **A — component** (`ui/core/lib/components/data-independent/meter.svelte`, new): root `<meter>`; props per lock; `width?` → inline `style="width:…"` else the skin's 100%; getter `{value,min,max,optimum}`; zero `<style>`.
- **B — skin** (`ui/assets/skin.css`): found `--warn: #ba7517` in `:root`; add the pseudo-heavy `.meter` block (track + 3 semantic value pseudos + disabled).
- **C — sampler** (`ui/sampler/src/app_sampler.svelte`): DI·atomic → Display section → `meter` row: `#optimum` (in-band, green) / `#caution` (sub, amber) / `#danger` (over-high, red) / `#neutral` (no optimum, grey) / `#disabled`. Full-width in the cell; one `#fixed` (width set) to prove the prop.
- **D — CDP verify** (sampler 9422, both accents, real output — Rule 2): `vite build` clean; registry delta; per-cell getter `{value,min,max,optimum}`; `MET­ER` tag + `display:block`/`width:100%` (default) vs fixed cell; all `.meter*` rules in cascade (rule-inspection, N-042) + screenshot (green/amber/red fills render); `--warn` resolves; 0 orphans.
- **E — records (D-074 atomic close)**: N-071 ui-notes · registry vNN (meter row + display-di count) · ROADMAP vNN (RP node + tree + M-RP2.30 ✅ DONE) · CLAUDE PLAY (Entry head → J-457, next-active → dd track open) · JOURNAL J-457 · this runbook ACTIVE→COMPLETED. (No DECISIONS touch unless `--warn` warrants a note; likely N-071 only.)

## Definition of Done

- [x] `meter.svelte` authored (root `<meter>`, props, width contract, getter), `vite build` clean.
- [x] `--warn` token founded + `.meter` pseudo-heavy skin added (3 semantic fills + disabled).
- [x] Sampler row added (optimum/caution/danger/neutral/disabled + one fixed-width), full-width default.
- [x] CDP-verified, both accents, real output: getters, tag/block/width, semantic fills in cascade + screenshot, 0 orphans.
- [x] Records closed atomically (N-071/registry/ROADMAP/CLAUDE/JOURNAL J-457), runbook → COMPLETED.

## After this arc
- **dd track opens (M-RP5.0):** `section-header` (ungrounded warm-up) → `entity-avatar` (first domain-bound, D-071 audit on IdentityRecord/Appendix I).
- `temperature-indicator` widget later consumes `meter` as its readout, binding a real metric structure through the dd-socket.
- Kind 4 `use:render` stays deferred (D-065).
