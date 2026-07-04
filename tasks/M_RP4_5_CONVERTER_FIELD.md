# M-RP4.5 — kind-2 converter/bridge field (`converter-field`)
> **Status**: COMPLETED  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-04  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

Kind **2** of the four-kind processor taxonomy (D-099 / N-056): the **converter / bridge** — `string ↔ T`. The one kind that is a **component, not an attachment**: display = formatted `string`, bound value = typed `T`, the two representations coexist. 3 of 4 kinds after this (1 transformer, 2 converter, 3 filter/guard; kind 4 renderer deferred).

## Phase-0 finding (headline)

Kinds 1 & 3 forward an attachment onto an atomic's inner `<input>` and sync back through a **single** `bind:value` — legal because display-rep and bound-rep share a type. Kind 2 has **two reps of different type**, which one binding cannot carry → a new host component owning both slots. (`textfield` isn't a processor-host today anyway — no `{...rest}` — so an attachment path was never viable here.)

## Locked design (Joe, by-recomms)

1. **Host** — new **di atomic** `converter-field`, root `<input type="text">`, `core/…/data-independent/`. Owns `value: T` (`$bindable`, typed out) + internal `text` (display/edit string) + `invalid`. Single native root → atomic (not composite; not a widget — control-state, not task-lifecycle, per D-102/J-453).
2. **Config + parse** — `Converter<T> { toString(v:T):string; fromString(s:string): T | PARSE_FAILED; toEditable?(v:T):string }`. `PARSE_FAILED` = unique symbol (so `null`/`NaN` T stay representable). First concrete = `intlNumber(opts?, locale?)` in `transform.ts`: `toString`=`Intl.NumberFormat.format`; `fromString` derives the locale's group/decimal glyphs via `formatToParts`, strips group, normalises decimal, `Number()`+finite-check; `toEditable`=raw `String(v)` (no grouping). **Parse-failure = reject-and-mark**: keep the user's text, set `[data-invalid]`, **value unchanged**. Empty text = no-op (revert display to current value, never an "invalid empty").
3. **Timing** — parse on `change`/`blur`; success reformats display via `toString`; `focus` shows the raw `toEditable` form; **nothing on `input`** (decoupled → no caret-restore machinery, unlike kind 1).
4. **Provenance** — **Tier-1 only**. A converter is code-supplied logic, never a user-authored string → **no caps, no convergence lint, no `assertSafeRules`**. The Tier-2 gate stays kind-1-specific.
5. **Getter** — `{ value, text, valid }` via `$state.snapshot` (T must be JSON-snapshotable; true for the Intl-number concrete).

## Build steps (D-071 arc)

- **A — pure core** (`ui/common/lib/components/processor/transform.ts`, additive): `PARSE_FAILED` symbol + `Converter<T>` type + `intlNumber()` factory. DOM-free (ECMA-402, not DOM) — stays the `logic.ts` posture. No `window` here.
- **B — host** (`ui/core/lib/components/data-independent/converter-field.svelte`, new): `<script lang="ts" generics="T">`; two-rep state; focus/blur/change handlers; `data-invalid` reflect; DEV `__XGEN_CONVERT__ = { intlNumber, PARSE_FAILED }` (component-scoped, the framework touch — transform.ts stays pure).
- **C — skin** (`ui/assets/skin.css`, additive): `.converter-field` assembled from the `.number` L2 vocabulary (single-line, `--ctl-h`), text-input (no spinner), `[data-invalid="true"]` → `--err` / focused `--err-bright` (attribute hook, since parse-fail is NOT native `:invalid`).
- **D — sampler** (`ui/sampler/src/app_sampler.svelte`): const `numConv = intlNumber({ maximumFractionDigits: 2 })` (stable identity); DI·atomic → Interactive → `converter-field` row: `#default` (seeded grouped number), `#disabled`.
- **E — CDP verify** (sampler 9422, both accents, real output — Rule 2): `vite build` clean; pure core via `__XGEN_CONVERT__` (`intlNumber` round-trip + `fromString('abc')===PARSE_FAILED`); live `converter-field#default` — drive `1234.5` → blur → getter `value:1234.5`, `text:"1,234.5"`; drive `abc` → blur → `[data-invalid]`, `valid:false`, value unchanged; empty blur → revert, valid:true; registry delta; skin in cascade; accent-swap; 0 orphans.
- **F — records (D-074 atomic close)**: D-099 amendment (kind 2 built) · N-070 ui-notes · registry vNN · ROADMAP v4.25 (RP node + tree + M-RP4.5 ✅ DONE) · CLAUDE PLAY (Entry head → J-456, next-active → kind 4 deferred / dd-components) · JOURNAL J-456 · this runbook → COMPLETED.

## Definition of Done

- [x] `transform.ts` kind-2 pure core added (Converter/PARSE_FAILED/intlNumber), `vite build` clean.
- [x] `converter-field.svelte` authored (generics, two-rep, focus/blur/change, data-invalid, DEV hook).
- [x] `.converter-field` skin added (assembled from `.number`, `[data-invalid]` err look).
- [x] Sampler row added (default + disabled), stable converter identity.
- [x] CDP-verified in the sampler, both accents, quoting real output: pure-core round-trip + PARSE_FAILED, live blur-format, parse-fail→data-invalid+value-unchanged, empty-blur revert, registry delta (98→100), 0 orphans.
- [x] Records closed atomically (D-099/N-070/registry/ROADMAP/CLAUDE/JOURNAL J-456), runbook → COMPLETED.
