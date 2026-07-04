# M-RP4.1 — kind-3 filter/guard: `number` min/max clamp
> **Status**: COMPLETED  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-04  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

Runbook for **kind 3** of the four-kind processor taxonomy (D-099/N-056): the **filter/guard** (`T → T`, idempotent). First consumer = `number` min/max clamp on **commit** (`change`), not per-keystroke. Lands the third pure-core rule kind; the `ProcessorRule` union stays codify-4 / build-progressively (D-065).

## Locked design (Joe, Phase-0)

1. **Pure core** — `transform.ts`: add `ClampRule { min?, max? }` + `applyClamp(n, rule)` — idempotent, total, DOM-free (`logic.ts` posture, sibling to `applyRules`). Union `ProcessorRule` stays a comment (no runtime union).
2. **Engine** — new sibling attachment `processor/clamp.ts` (NOT a branch of `processor.ts`: kind 1 is `input`-shaped w/ caret restore; kind 3 is `change`-shaped, numeric coerce, no caret). Listens on **`change`**, reads numeric value, coerces to `[min,max]` via `applyClamp`, writes back + dispatches `input` to sync `bind:value` (re-entrancy-guarded). DEV hook `__XGEN_CLAMP__`.
3. **Host** — `number.svelte` gains `...rest` + `{...rest}` on `<input>` (the 1-line additive; the comment's "reserved insertion point"). First clamp-host; ships guard-ready, no logic (D-065).
4. **Trigger** — `change` (commit). Clamp mid-type is hostile.
5. **Delivery** — forwarded attachment, mirrors kind-1 (`<Number {...clamp({min,max})} />`), consistent with N-056.

## Steps (Chat, pure — sampler-verifiable end to end)

- **A. Pure core.** `transform.ts`: `ClampRule` type + `applyClamp(n: number|null, rule): number|null` (null passes through; `min`/`max` optional; clamp guarded per bound; idempotent). +DEV hook exposure.
- **B. Engine.** `processor/clamp.ts`: `clamp({min?,max?})` → forwardable attachment (`createAttachmentKey`); `change` listener; coerce; write-back + synthetic `input`; re-entrancy guard; `__XGEN_CLAMP__` DEV hook.
- **C. Host.** `number.svelte`: add `...rest` prop + `{...rest}` spread on `<input>`. Additive, zero behaviour change otherwise.
- **D. Sampler.** `app_sampler.svelte`: `number#clamped` cell `<Number {...clamp({min:0,max:10})} … />` in the di-atomic panel; matrix +1.
- **E. CDP verify** (sampler 9422, both accents, real output): baseline; drive value `99` + `change` → coerced `10`; `-5` → `0`; in-range `7` → `7` (no-op); idempotent re-run stable; `bind:value` synced; 0 orphans.

## Definition of Done

- [x] A `ClampRule` + `applyClamp` (idempotent, total, null pass-through) + DEV hook
- [x] B `clamp.ts` attachment, `change`-triggered, re-entrancy-guarded, `__XGEN_CLAMP__`
- [x] C `number.svelte` `{...rest}` additive (guard-ready, no logic)
- [x] D sampler `number#clamped` cell, matrix +1
- [x] E CDP verified in sampler, both accents, real output, 0 orphans (99→10 / -5→0 / in-range no-op; registry 97→98)
- [x] Records atomic per D-074 (D-099 amendment + N-069 + registry v0.41 + ROADMAP v4.24 + PLAY + JOURNAL J-455); D-099 kind-3 marked built

## Notes

- No Rust / effect layer — pure `$common` + `$core`, fully sampler-verifiable (no D-097 blind spot).
- Kind 3 is the first `null`-aware pure kind (kind 1 is string→string); `applyClamp(null)=null`.
- Next after: kind-2 converter field (the bridge, `Intl`) → kind-4 `use:render` (deferred) → dd-components.
