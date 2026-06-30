# M-RP4.0 — text-processor engine: the four-kind taxonomy (codified) + kind-1 transformer built, `textarea` proving consumer

> **Status**: ACTIVE  
> Version: 1.1  
> Date: Jun 2026  
> **Last updated**: 2026-06-30  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. What this is

Discharge the long-deferred **text-processor** seam (N-029/N-032/N-038 — reserved, never empty machinery,
D-065). The design walk resolved the processor into a **four-kind taxonomy on two engines**, and set the
honest scope: **codify all four kinds** (the union shape + the §0.1 table, recorded in D-099/N-056), **build
only kind 1** (the live transformer) now, wired to its first proving consumer **`textarea`**, with a live
`#processed` cell in the sampler (D-097). Kinds 2/3/4 are reserved seams with named future consumers.

**Joe-locked decisions (this arc's design walk):**

1. **Two engines, four rule-kinds — orthogonal.** The processor is NOT one engine; it is an **edit-side**
   engine (a forwarded Svelte 5 attachment; the caret/re-entrancy plumbing) and a **render-side** engine
   (the deferred `use:render`, with the sanitiser). The four *kinds* (§0.1) are orthogonal to the two
   *engines*; **kind 2 is the bridge** — its `fromString`/mask runs edit-side, its `toString`/format runs
   render-side. Read the taxonomy as *two engines, four kinds, dispatched* — never "four engines."
2. **P-1a (A) — the edit seam is a forwarded *attachment*, not a `use:` action.** A `use:` action only
   attaches to elements in the component that writes them; a consumer can't forward one onto an atomic's
   internal element. So the engine ships as an **attachment** (`processor(rules)` → symbol-keyed prop via
   `createAttachmentKey`); the atomic spreads `{...rest}` onto its root, so `<Textarea {...processor(x)} />`
   lands it on the inner `<textarea>`. The atomic carries **no** processing logic — it only spreads.
   Reactivity = the attachment lifecycle (new rules → re-evaluate spread → cleanup + re-attach).
3. **P-2 — sink-agnostic pure core, separate from the wrapper.** `transform.ts` (`string + rules → string`,
   DOM-free, framework-free — like `logic.ts`) is built now. The render-side engine reuses the *idea*; the
   markup/`{@html}`/sanitiser sink is **NOT** built this arc.
4. **P-3 + provenance tiers.** Rules are an **ordered array**, order-significant. **Tier-1** (`common` code
   configs): trusted, full power. **Tier-2** (user data, e.g. from settings): **serializable literal
   `{find, replace}` pairs only** — caps (count, length) + a **convergence lint** (a pair whose `replace`
   re-matches its `find` is rejected; the engine re-runs the whole value each keystroke → it would loop).
   Untrusted **regex** is rejected (the ReDoS/complexity guard is reserved for an explicit advanced opt-in).
5. **P-4 — caret-preserving value sink.** On `input`, recompute; if changed, write `node.value`, restore
   caret/selection (shift by net length-delta of replacements before the caret), dispatch a
   re-entrancy-guarded synthetic `input` so Svelte's `bind:value` syncs. The build's hard bit.
6. **Scope (D-065 honest).** **BUILD: kind 1** (transformer) + `textarea`. **CODIFY ONLY (records, not
   machinery): kinds 2/3/4** — the union shape + the §0.1 table go into D-099 + N-056 so each kind has a
   declared home and growth is bounded; no runtime, no stub methods. Kind 1's `reversible` flag is declared
   on the type but **not implemented** (reserved).
7. **Forward-clean naming.** Kind 1's types are **`TransformRule` / `TransformConfig`**; the future union
   **`ProcessorRule = TransformRule | ConvertRule | ClampRule | RenderRule`** is *documented* (D-099) but
   only `TransformRule` exists in code now — so the namespace is clean when kinds 2/3/4 land.
8. **Reserved consumers (named, not built):** kind 2 → number/date/phone field (needs a *decoupled* text
   field — native `type=number` can't render `toString`; `toString` may delegate to `Intl`); kind 3 →
   `number` min/max clamp (M-RP4.1, fires on `change`/blur, composes with a converter); kind 4 → `paragraph`
   inline marks (the `use:render` arc, allowlist + sanitiser).
9. **Settings source (reserved).** Tier-2 pairs persist as a **section of the app's existing global settings
   file**, hydrated into a runtime reactive store the attachment reads; the settings UI edits the store;
   changes persist back. **No bespoke rules file.** Engine stays source-agnostic.

**Milestone M-RP4.0** (opens the **M-RP4** text-processor arc).

---

## 0.1 The four-kind taxonomy (the codified architecture — D-099 / N-056)

| # | kind | signature | ways | model `T` | engine / side | built? | first consumer | key characteristic / risk |
|---|---|---|---|---|---|---|---|---|
| **1** | **transformer** | `string → string` (live, on `input`) | one (+ optional per-pair `reversible`, default false) | none | **edit** (the M-RP4.0 attachment) | **BUILD NOW** | `textarea` (combo-morph: `arrowMorph`/`emojiMorph`) | caret + re-entrancy; convergence lint; Tier-1 code / Tier-2 literal pairs |
| **2** | **converter** | `string ↔ T` | two (`toString`, `fromString`) | number · Date · phone (canonical) | **both** (`fromString`/mask edit; `toString`/format render) | reserved | number / date / phone field | **straddles the split**; native `type=number` can't show `toString` → needs a decoupled text field; `toString` may delegate to `Intl` |
| **3** | **filter / guard** | `T → T` | one (idempotent, lossy) | the field's own `T` | **side-agnostic** (commit edit, or pre-display render) | reserved (M-RP4.1) | `number` (min/max clamp) | fires on `change`/blur, not keystroke; composes *with* a converter |
| **4** | **renderer** | `string → safeHTML` | one | none | **render** (the deferred `use:render` arc) | reserved | `paragraph` (inline marks `_x_`/`*x*`) | allowlist + real sanitiser; never `{@html}`, never regex as the safety boundary |

Two facts that travel with the table: **(a) kind ⟂ engine** — four kinds, two engines, kind 2 bridges; **(b)
scope** — codify all four, build only kind 1 (D-065).

---

## 1. Why this shape (for the N-entry)

The engine founds three things. (a) The **edit seam is a forwarded attachment** — the first time the library
forwards behaviour from a consumer onto an atomic's internal element without the atomic carrying the logic
(the resolution of "consumer layers it on", which `use:` could not satisfy). (b) The **kind taxonomy is
orthogonal to the engine split** — the durable mental model is *two engines (edit attachment / render
`use:render`), four rule-kinds (transform / convert / clamp / render)*, with kind 2 the bridge whose two
methods sit on opposite sides. (c) **Provenance tiers gate safety** — trusted `common` code vs serializable
user pairs; the literal-only Tier-2 subset is what makes runtime-editable rules safe (literals can't ReDoS;
the convergence lint stops the one loop a literal can cause).

It discharges the longest-standing reserved seam in the UI track (earmarked since `textfield`, N-029),
exactly as D-065 intended: built when a consumer is in hand, codified so growth is bounded.

---

## 2. Phase-0 references (read before authoring code)

- `ui/common/lib/components/base/envelope.ts` — action shape; the DEV-only `register`/debug pattern; the
  `import.meta.env.DEV` dead-code-elimination idiom (the processor's DEV test hook mirrors it).
- `ui/common/lib/components/base/logic.ts` — the pure/DOM-free/framework-free `common` style `transform.ts`
  matches.
- `ui/core/lib/components/data-independent/textarea.svelte` — the host; gains a one-line `{...rest}` spread
  (decision 2). Confirm the spread does NOT override `bind:value` / `use:envelope` / explicit attrs.
- `ui/sampler/src/app_sampler.svelte` — the DI·atomic panel + existing `textarea` row; the `#processed` cell
  appends here.
- `ui/docs/xgen-ui-notes.md` N-029 / N-032 (EDIT-vs-RENDER axis) / N-038 (sharpened spec + security shape) /
  N-040 (textarea reserve).
- `DECISIONS.md` D-065 (no empty machinery — the build/codify line), D-095/D-097 (common substrate / sampler).
  New **D-099** added at close.

---

## 3. Engine spec — new folder `ui/common/lib/components/processor/`

Build **kind 1 only**. Forward-clean naming (decision 7): the kind-1 type is `TransformRule`; `ProcessorRule`
is reserved as the future union and is **documented in D-099, not declared in code now**.

### 3a. `transform.ts` — pure core (DOM-free, framework-free)

```ts
export type TransformRule = {
  find: string;
  replace: string;
  reversible?: boolean;        // DECLARED, NOT IMPLEMENTED (reserved): a curated, collision-free pair the
};                             // author certifies invertible. Default false. No un-morph path built.
export type TransformConfig = TransformRule[];

// Sequential: each rule sees the prior rule's output; literals replace ALL occurrences. Pure, total.
export function applyRules(input: string, rules: TransformConfig): string { /* … */ }

// Provenance gate. trusted (Tier-1 code): pass. untrusted (Tier-2 user/settings):
//   caps (count <= CAP_RULES, find/replace length <= CAP_LEN) + convergence lint
//   (reject a rule whose `replace` still contains its `find`). Throws ProcessorRuleError.
export function assertSafeRules(rules: TransformConfig, opts: { trusted: boolean }): void { /* … */ }
```

(No regex rule-kind in code this arc: Tier-2 is literal-only, and Tier-1's named configs below are literal.
Regex + the ReDoS guard are reserved with the advanced opt-in.)

### 3b. `configs.ts` — named Tier-1 configs (trusted code)

```ts
export const arrowMorph: TransformConfig = [
  { find: '-->', replace: '→' }, { find: '<--', replace: '←' }, { find: '=>', replace: '⇒' },
];
export const emojiMorph: TransformConfig = [
  { find: ':)', replace: '🙂' }, { find: ':(', replace: '🙁' }, { find: '<3', replace: '❤️' },
];
```

Convergence (the Tier-2 authoring constraint + the `common`-config review rule): a `replace` must not contain
its `find`. `:)`→`🙂` safe; `a`→`aa` loops.

### 3c. `processor.ts` — the attachment (thin framework-coupled wrapper)

```ts
import { createAttachmentKey } from 'svelte/attachments';   // the ONE framework touch (isolated here;
                                                            // transform.ts stays framework-free)
import { applyRules, assertSafeRules, type TransformConfig } from './transform';

export function processor(rules: TransformConfig, opts: { trusted?: boolean } = {}) {
  const trusted = opts.trusted ?? false;                    // default-safe (settings-sourced rules)
  assertSafeRules(rules, { trusted });
  const attach = (node: HTMLInputElement | HTMLTextAreaElement) => {
    let reentrant = false;
    const onInput = () => {
      if (reentrant) return;
      const before = node.value;
      const next = applyRules(before, rules);
      if (next === before) return;
      const caret = /* net length-delta before selectionStart (P-4) */ 0;
      reentrant = true;
      node.value = next;
      node.setSelectionRange(caret, caret);
      node.dispatchEvent(new Event('input', { bubbles: true }));   // sync Svelte bind:value
      reentrant = false;
    };
    node.addEventListener('input', onInput);
    return () => node.removeEventListener('input', onInput);
  };
  return { [createAttachmentKey()]: attach };               // spread-forwardable
}
```

DEV test hook (mirrors envelope's DEV idiom; for CDP pure-function verify): behind `import.meta.env.DEV`,
`window.__XGEN_PROC__ = { applyRules, assertSafeRules }`. Dead-code-eliminated in prod.

---

## 4. Atomic retrofit — `textarea.svelte`, one line

Add `...rest` to `$props()` and spread it on `<textarea>` so a forwarded attachment lands. **Ready, not
containing** (D-065): no processor import/logic, only a generic spread. Verify the spread does not shadow
`bind:value` / `use:envelope` / explicit attrs (Svelte: `bind:`/`use:`/directives aren't overridden by
`{...rest}`; explicit attrs win on collision — none collide). Only `textarea` is retrofitted now (`number`
in M-RP4.1; do not preemptively retrofit every atomic — D-065). Header note: the reserved-seam paragraph's
tail becomes "processor-host: forwards `{...rest}`; see M-RP4.0 / N-056."

---

## 5. Sampler integration (D-097) — DI·atomic panel, `textarea` row

Import `processor` + `arrowMorph`; append one cell to the existing `textarea` row:

| cell `id` | markup | shows |
|---|---|---|
| `textarea#processed` | `<Textarea {...processor(arrowMorph, { trusted: true })} id="processed" value={…} />` | typing `-->`/`<--`/`=>` morphs live to `→`/`←`/`⇒`; bound value reflects the morphed text |

Matrix **55 → 56** (the attachment adds no registry entry; the host textarea is the one id).

---

## 6. CDP verification (Chat self-drives — sampler only)

Launch detached minimized; poll 9422 (retry until non-null); fresh launch (no stale HMR); split `.click()`
from the DOM read by a tick (J-433); `cdp-debug.ps1 -App sampler -Mode {state,eval,screenshot}`; teardown
(5175/9422 free, 0 orphans). Quote **actual** output in the JOURNAL (Rule 2); never invent (Rule 5).

1. **Count:** `ids().length === 56`; `textarea#processed` present.
2. **Transform + binding sync (core proof):** set `el.value='a --> b => c'`, dispatch bubbling `input`,
   **tick**, read: `el.value === 'a → b ⇒ c'` AND registry `textarea#processed` → `{ value: 'a → b ⇒ c' }`
   (proves the synthetic input synced `bind:value`, not just the DOM).
3. **Pure core (DEV hook):** `__XGEN_PROC__.applyRules('-->', [{find:'-->',replace:'→'}]) === '→'`;
   sequential order holds (`applyRules('a-->b', arrowMorph)`).
4. **Provenance guard:** `assertSafeRules([{find:'x',replace:'xx'}], {trusted:false})` **throws**
   (convergence: `xx` contains `x`); same under `{trusted:true}` **passes**; `arrowMorph` under
   `{trusted:false}` **passes** (convergent literals).
5. **No-op safety:** a value with no morph token round-trips unchanged (no spurious input dispatch).
6. **Screenshot (eye-check):** the `#processed` cell in the DI·atomic textarea row; type the arrows, confirm
   live morph + caret stays at the typing point (P-4 risk — eyeball; CDP can't drive `:focus`+caret).

---

## 7. Records (D-074; written after verification, Rule 4)

- `ui/docs/xgen-ui-notes.md` — **N-056** (the forwarded-attachment edit seam; the **four-kind taxonomy +
  §0.1 table**; kind ⟂ engine, kind 2 the bridge; two-tier provenance + serializable literal subset +
  convergence lint; caret-preserving value sink; `reversible` declared-not-built; settings-backed rules
  reserved; render-side `use:render` still deferred). Version bump.
- `DECISIONS.md` — **D-099** (text-processor architecture: **two engines × four rule-kinds**, the §0.1 table
  as the canonical taxonomy; edit/render sink-split; attachment-forwarded edit seam; kind 2 straddles the
  split; two-tier provenance + serializable literal subset; settings-backed runtime rules; codify-four /
  build-one scope). New append; bump `Last updated`.
- `docs/ROADMAP.md` — open the **M-RP4** arc; **M-RP4.0 ✅**; M-RP4.1 (kind 3, number-clamp) 🟡; kind 2
  (converter field) 🟡; kind 4 / `use:render` ⏸️; version bump; same-commit with CLAUDE.
- `CLAUDE.md` — PLAY → M-RP4.0; prior-PLAY pointer → J-435; next-active → M-RP4.1 (kind-3 clamp) → kind-2
  converter field → kind-4 `use:render` (deferred) → dd-components.
- `JOURNAL.md` — **J-435** (newest-first; real CDP output incl. 56-count, morph + binding-sync, guard reads).
- `ui/docs/xgen-ui-components.md` — **no catalogue row** (processor is `common` infra); one-line note that
  `textarea` is now a processor-host. Version bump iff edited.
- `tasks/M_RP4_0_PROCESSOR_ENGINE.md` — Status → COMPLETED.

`.md` header rule: `> **Last updated**:` carries ONLY the date.

---

## 8. Commit plan (two commits, UI pattern `feat` → `docs`; Joe pushes)

**Commit 1 — implementation** (`transform.ts`, `configs.ts`, `processor.ts`, `textarea.svelte`,
`app_sampler.svelte`):

```powershell
$ProgressPreference='SilentlyContinue'
cd E:\Projects\XGenProtocol
git add ui/common/lib/components/processor/transform.ts
git add ui/common/lib/components/processor/configs.ts
git add ui/common/lib/components/processor/processor.ts
git add ui/core/lib/components/data-independent/textarea.svelte
git add ui/sampler/src/app_sampler.svelte
git status
git commit -m "feat(ui): text-processor kind-1 transformer - edit-side common attachment + textarea (M-RP4.0)" -m "Discharges the reserved processor seam (N-029/N-032/N-038, D-065). Resolved into a four-kind taxonomy on two engines (edit attachment / render use:render): 1 transformer string->string live, 2 converter string<->T (bridge), 3 filter/guard T->T, 4 renderer string->safeHTML. This commit BUILDS kind 1 only; 2/3/4 are codified (D-099/N-056) and reserved. Pure sink-agnostic transform.ts (applyRules + assertSafeRules); named Tier-1 configs (arrowMorph/emojiMorph); processor.ts forwarded Svelte 5 attachment (createAttachmentKey) - the atomic spreads {...rest}, carries no processing logic. Caret-preserving value sink + re-entrancy-guarded synthetic input to sync bind:value. Two provenance tiers (Tier-1 trusted code; Tier-2 serializable literal pairs only, caps + convergence lint, untrusted regex rejected). TransformRule.reversible declared, not implemented." -m "textarea gains a one-line {...rest} spread (ready, not containing; only textarea now, number in M-RP4.1). Sampler #processed cell in DI-atomic (matrix 55->56). Render-side use:render deferred; settings-backed rules reserved, no bespoke file."
git push
```

**Commit 2 — records** (`xgen-ui-notes.md`, `DECISIONS.md`, `ROADMAP.md`, `CLAUDE.md`, `JOURNAL.md`,
`xgen-ui-components.md` iff edited, `M_RP4_0_PROCESSOR_ENGINE.md`):

```powershell
$ProgressPreference='SilentlyContinue'
cd E:\Projects\XGenProtocol
git add ui/docs/xgen-ui-notes.md
git add DECISIONS.md
git add docs/ROADMAP.md
git add CLAUDE.md
git add JOURNAL.md
git add tasks/M_RP4_0_PROCESSOR_ENGINE.md
git status
git commit -m "docs(ui): close M-RP4.0 processor kind-1 - N-056, D-099, J-435, records" -m "N-056 (forwarded-attachment edit seam; four-kind taxonomy + table; kind ortho engine, kind 2 the bridge; two-tier provenance + serializable literal subset + convergence lint; caret-preserving sink; reversible declared-not-built; use:render deferred). D-099 (text-processor architecture: two engines x four rule-kinds, the canonical taxonomy table, edit/render sink-split, attachment-forwarded edit seam, kind 2 straddles, two-tier provenance, settings-backed rules, codify-four/build-one). ROADMAP opens M-RP4 (4.0 done; 4.1 kind-3 clamp + kind-2 converter pending; kind-4 use:render postponed). CLAUDE PLAY -> M-RP4.0; J-435 (real CDP: 56-count, morph + binding-sync, guard). Task -> COMPLETED." -m "Engine is common infra, no catalogue row; textarea noted as processor-host."
git push
```

---

## 9. Definition of Done

- [ ] `transform.ts` — `TransformRule` (with declared-not-implemented `reversible`); `applyRules`
      (sequential, literal replace-all) + `assertSafeRules` (Tier-2 caps + convergence lint); pure,
      framework-free, `applyRules` total.
- [ ] `configs.ts` — `arrowMorph` + `emojiMorph` (Tier-1, convergent).
- [ ] `processor.ts` — forwarded attachment (`createAttachmentKey`); caret-preserving value sink;
      re-entrancy-guarded synthetic `input`; validates at attach; DEV `window.__XGEN_PROC__` hook.
- [ ] `textarea.svelte` — one-line `{...rest}` spread on `<textarea>`; no processor import/logic; header note
      updated; `bind:value`/`use:envelope`/explicit attrs unshadowed.
- [ ] Sampler `textarea#processed` cell (DI·atomic) with `processor(arrowMorph,{trusted:true})`; matrix 55→56.
- [ ] CDP §6 run in the sampler — actual output captured: count 56, transform+binding-sync, pure-core via
      DEV hook, the guard outcomes, no-op safety, screenshot caret eye-check.
- [ ] N-056 (incl. the §0.1 four-kind table) + D-099 (the canonical taxonomy) written; ROADMAP M-RP4 arc
      opened (4.0 ✅) + CLAUDE same-commit; J-435 (real CDP output).
- [ ] Codify-four / build-one honoured: kinds 2/3/4 are records-only (no runtime, no stubs); only kind 1 built.
- [ ] Task Status → COMPLETED.

(`Status: COMPLETED` is the real signal — no "commit pushed" checklist item.)
