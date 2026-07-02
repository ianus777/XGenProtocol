# XGen Protocol — Claude Code Briefing
> For: Claude Code (claude.ai/code)  
> Date: May 2026  
> **Status:** ACTIVE  
> **Last updated:** 2026-07-02  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  

---

## 🟢 UI component-library / substrate (RP track) — M-RP2.21 ✅ CLOSED (J-432): `link` — the **sixteenth** `core` component, the **first navigation-kind di** (atomic `<a href>`); commits `<a>`-vs-`<button>`; synthesised `disabled` (drops `href`); bundled-safe `external` rel; returns to accent-derived colour; getter `{text,href,external,disabled}`; **M-RP3.2 ✅ (J-433): sampler tabbed by class×arity** — four-panel di/dd container, all panels mounted + inactive CSS-hidden (never `{#if}`, preserves the CDP registry), DI·atomic holds the 44-cell grid; pure sampler chrome; **M-RP2.22 ✅ (J-434): `status-indicator` built** — the FIRST di composite (`<div class="status-indicator">` = led + label + optional link; aggregate getter + children self-register under stable ids, matrix 44→55); **M-RP4.0 ✅ (J-435): text-processor kind-1 transformer built** — edit-side `common` forwarded attachment (`processor/transform.ts` pure core + `configs.ts` `arrowMorph`/`emojiMorph` + `processor.ts` `createAttachmentKey` attachment, caret-preserving sink + re-entrancy guard + DEV `__XGEN_PROC__`) + `textarea` processor-host (one-line `{...rest}` spread), matrix 55→56; the processor resolved into a **four-kind taxonomy on two engines** (edit attachment / render `use:render`): 1 transformer · 2 converter (bridge) · 3 filter/guard · 4 renderer — **all four codified (D-099/N-056), only kind 1 built** (D-065); **M-RP4.2 ✅ (J-441): user-owned substitution pairs — source-agnostic `$common` `substitutions` store + ` | `/first-space grammar, fed by the real client TOML (Rust loader → `get_substitutions` command → store) or a sampler literal; presets (`configs.ts`) retired + deleted (N-057/D-100)**; **M-RP4.4 ✅ (J-443 Rust / J-444 close): sampler real config-load path + clean-slate-on-start** — all 3 binaries (client/node/sampler host) wipe+regen config on start (D-101, suspends J-438 seed-once this phase); sampler host writes a subset `[substitutions]` config + `get_substitutions`, `app_sampler.svelte` hydrates via `invoke` (frontend literal retired, N-057 seam → two hand-synced Rust consts, shared-const crate out of scope); CDP-verified live (config generated on start, morph from loaded rules, delete-on-start wipes a sentinel, count 56); **next (reorder, J-445/N-059) — di-composite backlog (passive, N-054) → widget definition (N-059 promoted) → M-RP4.3 (in-app TOML editor + write-back; the first `widget`) → M-RP4.1 (kind-3 number-clamp)** → kind-2 converter field → kind-4 `use:render` (deferred) → dd-components

**Where we are.** The reference component library is being laid down on the D-095 tier split (`ui/{client,node,common,core,assets}` mirrors the crate workspace; `$common` = base substrate, `$core` = the GPL reference library). Substrate at `common/lib/components/base/` (`logic`/`envelope`/`debug`, N-023/N-024): `use:envelope` stamps the type-class + `data-debug-id` and registers an opt-in debug getter into `window.__XGEN_DEBUG__`, read out-of-band via the CDP harness. **Three `core` components built + registry-verified live in both apps** (client 9222 / node 9322, real `tauri dev` + CDP): `toggle` (M-RP2.3, J-403 — bind-in `bind:checked`), `button` (M-RP2.4, J-405 — event-out `onclick`; restored the window close affordance, retired the throwaway `Button.svelte`), and `textfield` (M-RP2.5, J-407 — string bind-in `bind:value`). The substrate **generalizes** across all three binding shapes (boolean-in / event-out / string-in) — the question each successive pass was run to answer.

**This close (J-407).** `textfield` authoring pass (the Joe-locked next move): author → wire `$core` into both shells → self-verify both apps → record. Atomic native `<input type="text">`, `type` fixed (not a prop — neighbouring semantics are their own components); native-state surface (`value`/`placeholder`/`disabled`/`readonly`/`id`/`pattern`/`name`); `pattern`→`:invalid` for template-mismatch (consumer's rule, skin's look); processor-**ready** (a future `common` `use:` action shared with `<textarea>`, not built). Chat self-drove the CDP loop: baseline `textfield#demo` `{value:""}` → dispatched a real `input` event → `{value:"hello"}` (client) / `{value:"world"}` (node) — **re-landing the live-reactive-read delta on the bind-in path** the terminal button couldn't self-redump. N-029 finding: driving `bind:value` over CDP needs a dispatched `input` event, not a bare `el.value=`; the N-028 race-fix + clean teardown reconfirmed (zero orphan ports). Sampler design-of-record + A/B/C phase taxonomy unchanged (N-028).

**Owed to Joe — both routed items CLOSED (J-406, 2026-06-22):** (1) the D-095 dev-tooling-exemption footnote landed (`ui/sampler/` and peers are mirror-exempt); (2) the GPL-overview question is resolved — no decision needed: one development license locks created code, GPL converts on handover per the fundamental records, so no `DECISIONS.md` touch.

**Next-active (UI/RP track):** M-RP2.6 ✅ CLOSED (J-410) — the `button` retrofit (+`ariaLabel`, +`mode` momentary/toggle, +`pressed` $bindable; getter → `{clicks,disabled,pressed}`; `aria-pressed` toggle-mode-only) and `toggle` `shape` (switch → `role="switch"` + reflected `aria-checked`) landed **additive & skin-free**, CDP-verified in both apps — the pressed-latch self-redump finally landed (the event-driven proof the terminal Quit/Shut-Down couldn't, N-028 finding 1); Quit/Shut-Down stay momentary, untouched. **M-RP2.7 ✅ CLOSED (J-412): first skin pass** — N-031 CSS source stack stood up (`modern-normalize.css` relocated to `ui/assets/` as pristine L0 → new `xgen-normalize.css` floor → one `skin.css` L2 → `app.css` gutted to chrome; `$assets` alias both shells), L2 vocabulary founded (radius/spacing, accent-tinted focus ring, disabled/invalid/pressed, switch via `appearance:none`+`::before` — Q5 **skin-only**, `toggle.svelte` stays `<style>`-free), per-shell `--accent*` (gold/`--pr` client, blue/`--inf` node). `button{}` wrinkle closed (re-keyed onto `.button`); CDP computed-style probe + both-app screenshots verified; a `*/`-in-comment bug that dropped `:root{--accent}` was caught + fixed in verify. **M-RP2.8 ✅ CLOSED (J-413): `select`** (di·A, single-select, atomic `<select>`) authored + skinned in one pass — first content-carrying di (normalized `options` prop, N-034); `.select` assembles from L2 + inline-SVG arrow, L1 empty; CDP-verified both apps (dispatched `change` → `{value}`). **M-RP2.9 ✅ CLOSED (J-414): `label`** (di·A, atomic `<label>`, read-only caption) — the first display-kind di (value-carrying but read-only, N-032); value prop `text` (not `value`); `.label` assembles from L2 (`--t2` + 12px), no new token; CDP-verified both apps (`{text}` + computed-style), founding the display-di render-then-computed-style verify pattern. **M-RP2.10 ✅ CLOSED (J-415): `paragraph`** (di·A, atomic `<p>`, single-paragraph prose) — the second display-di; text-node body `{text}` (never `{@html}`), the inline-mark formatter reserved as a future `common` `use:render` action (D-065, not built); also **founded the `--fs-*` type scale** (`--fs-1: 12px` + `--fs-2: 14px` + `--lh: 1.5`, four shipped skins retro-keyed, no fs-3/fs-4 seed); `.paragraph` = `--fs-2`/`--t`. **M-RP2.11 ✅ CLOSED (J-416): `image`** (di·A, atomic `<img>`, `src` + required `alt`) — the **third/final display-di**, the trio complete; first void-element di (value in the `src` attribute, not a text body); `alt` required (DEV-warn on omission, `alt=""` valid for decorative); getter `{src, alt}`; `.image` = `border-radius: var(--rad)`, no new token; demo backed by a new bundled `ui/assets/img-placeholder.svg`; CDP-verified both apps (`{src, alt}` + computed-style radius/block). **M-RP2.12 ✅ CLOSED (J-417): `textfield` `type` fold** — a constrained `type` prop (TS union, default `text`) folds the string-input family `text|search|email|url|tel|password` into the one component (all share `<input>` root, string `bind:value`, `.textfield` skin); **reverses N-029 → D-096**; getter now `{type,value}`; per-type very-weak-grey `#e6e6e6` inset icons (skin, N-039: search magnifier / email envelope / url link / tel phone / password `***`; `text` none; search clear-x suppressed); `textfield.svelte` zero-`<style>` held; password reveal deferred to the `password-field` composite. CDP both apps: registry `{type:"text"}`+`{type:"search"}`, `el.type` sweep all six, per-type `background-image` (text none / five present), `bind:value` delta on `type=search`, `:invalid`→`--err` (email + pattern, detached), screenshots. **M-RP2.13 ✅ CLOSED (J-418): `textarea`** (di·A, atomic `<textarea>`, multi-line free-text, string `bind:value`) — the **eighth** `core` component, a stand-alone atomic (root `<textarea>`, **not** a `textfield` fold), the edit-side multi-line counterpart to `paragraph`'s render-side prose (N-032 axis); prop surface = textfield's minus `type`/`pattern` plus `rows` (default 3); getter `{value}`; own `.textarea` skin (`resize: vertical`, no `--ctl-h`/icons/`:invalid`); **edit-side `use:processor` seam reserved, processor kept DEFERRED** (N-038 sequence + D-065 — ships processor-ready, NOT the trigger); auto-grow (`field-sizing`) reserved as a future skin shape. CDP both apps: registry `{value:""}` → dispatched `input` with a `\n`-bearing string round-trips (`lineCount=2`, the textfield-distinguishing proof), computed-style `TEXTAREA`/`--fs-1`/`--t`/`resize:vertical`/`--rad`, screenshots. **M-RP2.14 ✅ CLOSED (J-419): `number`** (di·A, atomic `<input type="number">`, numeric free-entry, numeric `bind:value`) — the **ninth** `core` component; same `<input>` root as `textfield` but a **distinct** atomic on the value-type discriminator (Svelte coerces `bind:value` to a number, `null` when empty — D-096 **held**, not amended); first registry value neither boolean nor string; prop surface = textfield's minus `type`/`pattern` plus `min`/`max`/`step`; getter `{value}`; native spinner kept (custom stepper = later composite); own `.number` skin (keeps `--ctl-h` + `:invalid`→`--err`, no icons/resize); **edit-side `use:processor` seam reserved, processor kept DEFERRED** (2nd consumer — D-069 watch; N-038 sequence + D-065). CDP both apps: baseline `{value:null}` (empty=null) → dispatched `input` carries a JSON **number** (42/7, not a string), `:invalid` out-of-range→`--err`, computed-style `INPUT`/`type=number`/`--ctl-h`/`--fs-1`/`--t`/`--rad`. Build snags caught+fixed: runes can't carry TS generics/annotations in the plain-JS shells; pseudo-class computed-style needs a separate-task read. **M-RP2.15 ✅ CLOSED (J-420): `range`** — the tenth `core` component, atomic `<input type="range">` (bounded numeric slider, numeric `bind:value`). Shares root AND value-type with `number`, so the *literal* D-096 criterion would fold it in — but it **stays its own atomic** on disjoint skin/prop-surface/interaction, the case that **sharpens D-096** to root + value-type + **shared skin/surface** (→ D-096 amendment, N-042). Always-valued (default 0, never `null`, no clamping); prop surface drops `placeholder`/`pattern`/`readonly`/`:invalid`, keeps `min`/`max`/`step`; getter `{value}`; **no processor seam** (bounded drag, not typed entry). First **pseudo-element-heavy skin** (`.range` = `appearance:none` + `::-webkit-slider-runnable-track` `--s5` groove + `::-webkit-slider-thumb` per-shell accent; fill deferred; PROVISIONAL). CDP both apps: baseline `typeof number` (always-valued) → dispatched `input` carries a JSON number 75/25; element computed-style `INPUT`/`type=range`/`appearance:none`/`160px`; the `::-webkit-slider-*` skin verified via **stylesheet-rule inspection + screenshot** (`getComputedStyle` returns UA defaults on the shadow-pseudos — a method finding, N-042). **M-RP3.0 ✅ CLOSED (J-422): sampler arc (M-RP3) — component track PAUSED.** Stood up `xgen-sampler`, a third Tauri/WebView2 app (own `ui/sampler` frontend + a minimal host crate — `tauri`+`tauri-build` only, no protocol deps; ports Vite 5175 / CDP 9422) as the component test-bed: build/tune/CDP-verify the `core` library in isolation with a live client↔node skin-swap (`[data-shell]` flips `--accent` gold↔blue, `#9a6a30`↔`#2a6090`). **D-097** (test-bed split — components in the sampler; the two shells *running together* in the real apps, the sampler's structural blind spot) + **D-098** (runtime = full WebView2 sibling, minimal host, D-095-mirror-exempt). v0 mounts one `button#smoke` — registry-verified end-to-end in the new app (`ids()=["button#smoke"]`); live `skin.css` editing is Vite HMR (Joe edits the canonical file, watches live; no fs-plugin in dev). Real shells reverted to original chrome — logo + state-indicator + the real Quit/Shut-Down button; the throwaway component demos (M-RP2.3–2.15, the pre-sampler live-in-both-apps probes) removed from both shells, redundant under the sampler test-bed (D-097) (J-428). **M-RP3.1 ✅ CLOSED (J-423):** populated the sampler — all 10 `core` components live, **22 `envelope`-registered instances** in a semantic-group×state grid (class/phase deferred — degenerate while all di·A), ragged state-map (disabled interactive-only; invalid `textfield`+`number` only; no faked focus column), polished `client|node` segmented skin-swap as tool chrome. Surfaced an **atomic gap**: `toggle` has no `disabled` prop → `toggle#switch` shown, logged for the di resume. CDP both apps: `ids().length===22`, `number#invalid`+`textfield#invalid` border `--err` (default stays `--s5`), disabled `cursor:not-allowed`, skin-swap toggle `accent-color` gold↔blue, both-shell screenshots differ; `envelope` keys by `data-debug-id=type#id` so reused `id` is collision-free. **M-RP2.16 ✅ CLOSED (J-425): `date`** (di·A, atomic `<input>` date-input family) — the **eleventh** `core` component, resuming the di-atomic series. The five date-input siblings (`date`/`time`/`datetime-local`/`month`/`week`) **fold into one component** via a constrained `type` prop (default `date`): same `<input>` root + same value-type (**string** `bind:value`, empty=`''`) + shared skin/surface → passes the *sharpened* D-096 (the `textfield` fold again, not `range`; **applies D-096, no amendment**, N-046); getter `{type,value}` carries the per-type format; own `.date` skin (`.number` box + `:invalid` + recoloured calendar-picker indicator; picker chrome dark via `color-scheme:dark`, N-043 now exercised); `readonly` a native pass-through (not sampler-exercised). **Sampler-DoD now STANDING (→ D-097 note):** a component milestone isn't done until its sampler row + applicable-state cells are CDP-verified **in the sampler** — replaces dual-shell demo wiring (real apps = integration + two-apps-together only). CDP (sampler, both accents via skin-swap): matrix **22→29** (7 `date#`), every `date#…` under one `"type":"date"` carrying its own input type (the fold proof), dispatched `input` "2026-12-31" string round-trips through `bind:value`, `:invalid`→`--err` specific (default stays `--s5`), all 6 `.date` rules in cascade (stylesheet-rule inspection, N-042 method), `--accent2` `#c28840`↔`#3a7ab0`, screenshot eye-checked (five native pickers render). **M-RP2.17 ✅ CLOSED (J-426): `color`** (di·A, atomic `<input type="color">`, native swatch + picker) — the **twelfth** `core` component, a **singleton that stands alone** (own atomic on disjoint swatch skin — the `range` case: shares root + value-type string with `date` but a swatch, not a text box; **applies D-096, no amendment**, N-047). Always-valued string `#rrggbb` (default `#000000`, never empty), getter `{value}` (no `type`); the **leanest prop surface yet** (`value`/`disabled`/`id`/`name`) — no `:invalid`/`readonly`/min-max-step; no processor seam. Own `.color` swatch skin (`::-webkit-color-swatch*` pseudos, compact 36×24, no `--ctl-h`, `opacity:0.5` disabled). The native picker dialog (saturation/hue/eyedropper/RGB) is OS/Chromium-painted, **not skinnable** → a themed **`color-picker` composite** is logged (the `password-field` shape, deferred). CDP (sampler, both accents): matrix **29→31**, both `color#` under `"type":"color"`, dispatched `input` `#123456` string round-trips through `bind:value`, computed-style `INPUT`/`type=color`/`appearance:none`/36×24, disabled `not-allowed`+`opacity:0.5`, all 5 `.color` rules in cascade, `--accent2` `#c28840`↔`#3a7ab0`, screenshot (swatches + the native picker render). Verify finding: interactive native-popup controls (`color`, the `date` pickers) can self-mutate under stray events in a minimized window — read the **non-interactive** cell for the seed proof + the dispatched round-trip for the bind proof (N-047). **M-RP2.18 ✅ CLOSED (J-427): `file`** (di·A, atomic `<input type="file">`, native picker button) — the **thirteenth** `core` component. Own atomic, but the headline is the **binding shape**: the **first non-`value` binding** — `bind:files`, a **FileList** (the 4th binding shape after boolean-in/event-out/string-in), and the first value-type `$state.snapshot` can't serialise. Getter **de-FileLists** to `{count, files:[{name,size,type}]}`; the bindable prop carries the live FileList. Props `accept`/`multiple`/`disabled`/`id`/`name`; `value` unsettable (security); no processor seam; fires `change` not `input`. Own `.file` skin styles the file-button pseudo (`::file-selector-button` + `::-webkit-file-upload-button`, separate rules) to match `.button`; UA "No file chosen" accepted → a custom drag-drop row is the deferred **`file-field` composite**. CDP (sampler, both accents): matrix **31→34**, FileList round-trip via DataTransfer + `change` → `{count:1,files:[{name:"test.txt",size:1,type:"text/plain"}]}` (the first non-`value` binding proven), `multiple` true/false, disabled `not-allowed`+`opacity:0.5`, all 8 `.file` rules in cascade, `--accent2` `#c28840`↔`#3a7ab0`, screenshot ("Choose File"+"test.txt" / "Choose Files" / greyed). **M-RP2.19 ✅ CLOSED (J-430): `select-multiple`** (di·A, atomic `<select multiple>`) — the **fourteenth** `core` component, the **last input-family atomic di**. Own atomic under the **sharpened** D-096 (shares the `<select>` tag but fails value-type — `string[]` vs scalar string — AND skin-surface — list-box vs dropdown; applies D-096, no amendment). The **first plain-array value-type**: `bind:value` → `string[]` (the 5th binding shape); empty model **`[]` not `null`**; getter `{values,count}`; options-prop carries over from `select` (N-034); `size?` default 4; `multiple` hardcoded. Own `.select-multiple` skin (list-box surface, accent-tinted `option:checked`). CDP (sampler, both accents): matrix 34→37, `#default {values:[],count:0}` (the `[]` empty-model proof), `#seeded {values:["a","c"],count:2}`, round-trip select a+b → `{values:["a","b"],count:2}`, `tag=SELECT`/`multiple=true`/`size=4`, 5 `.select-multiple` rules in cascade incl `option:checked`, `--accent2` `#c28840`↔`#3a7ab0`, screenshots (gold/blue selected rows). **M-RP2.20 ✅ CLOSED (J-431): `led`** (di·A, atomic inline `<span class="led">` status light) — the **fifteenth** `core` component, the **fourth simple display-di** (after label/paragraph/image). Two firsts: a **caller-supplied colour map** (`states: Record<string,string>` + `state` key — the N-034 options-prop shape applied to a display-di) and the **first data-coloured atomic** (colour rides an inline `--led-colour` CSS var the `.led` skin reads, skin owns shape only → the **first component whose colour is NOT accent-derived**). Contract: `colour = states[state] ?? #000000` (black = reserved unknown/undefined sentinel, consumers never map a real state to black); `title = state ?? '?'`; getter `{state,colour}`; `role=img`; `pulse?` via reflected `data-pulse`; values hex or `var(--token)`. CDP (sampler): matrix 37→41, `#default {ON,#22c55e}`=rgb(34,197,94), `#off {OFF,var(--t4)}`=rgb(88,92,100), `#unknown {???,#000000}`=rgb(0,0,0) (sentinel renders), `#pulse` animation-name=led-pulse, `.led`+`.led[data-pulse]`+`@keyframes` in cascade, no-accent-dependency proven (bg identical client↔node). **M-RP2.21 ✅ CLOSED (J-432): `link`** (di·A, atomic `<a href>`) — the **sixteenth** `core` component and the **first navigation-kind di** (a new kind alongside interactive and display). It acts (navigates) while carrying a `text` label; commits the `<a>`-vs-`<button>` split (`link` IS an `<a>`; the look-alike stays `button`). Props `href`/`text`/`onclick?`/`external?`/`disabled?`/`ariaLabel?`/`id`; getter `{text,href,external,disabled}`. Three mechanics: **synthesised `disabled`** (no native `<a>` disabled → drops `href` + `aria-disabled` + `tabindex=-1` + blocks `onclick`, skin greys; the first faked native-absent state); **bundled-safe `external`** (→ `target=_blank` + `rel="noopener noreferrer"`); **returns to accent-derived colour** (`.link` `var(--accent2)`, re-themes gold/blue — `led`'s caller-supplied colour was the one-off). The atomic never imports Tauri/router: `shell.open` (OS browser), `onclick`→router (SPA), modal-open=`button` are consumer wiring. CDP (sampler): matrix 41→44, `#disabled` href **dropped to null** (the synthesised-disabled proof) + `aria-disabled=true`/`tabindex=-1`/greyed `--t4`, `#external` `_blank`+safe `rel`+`aria-label`, 4 `.link` rules in cascade, **accent-swap proven** (`#default` gold `rgb(194,136,64)` ↔ blue `rgb(58,122,176)`). Logged: future `modal`/`dialog` (native `<dialog>`) + `icon` primitive. **M-RP2.22 ✅ CLOSED (J-434): `status-indicator`** — the **seventeenth** `core` component and the **FIRST di composite** (root `<div class="status-indicator">` = `led` + `label` + optional trailing `link`; binding none, di). Founds the composite build pattern: the composite registers an aggregate getter `{state,caption,hasLink}` and the real child atomics self-register under composite-derived stable ids (`<id>__led`/`__label`/`__link`, zero changes to the closed atomics), so a composite row yields multiple registry entries and the matrix **multiplies** (44→55). `{#if linkHref}` renders the optional link (distinct from N-053). CDP-verified (sampler): `ids().length===55`, aggregate + child getters, link-iff-href, root `DIV.status-indicator`, `.status-indicator` flex skin in cascade, combined accent proof (link rides `--accent2` gold↔blue, led's caller colour does not). **M-RP4.0 ✅ CLOSED (J-435): text-processor kind-1 transformer** — edit-side `common` forwarded attachment (`processor/transform.ts`+`configs.ts`+`processor.ts`, `createAttachmentKey`, caret-preserving sink + re-entrancy guard + DEV `__XGEN_PROC__`) + `textarea` processor-host (`{...rest}`), matrix 55→56; four-kind taxonomy on two engines (edit attachment / render `use:render`) codified (D-099/N-056), only kind 1 built (D-065). CDP (sampler): `ids().length===56`, `-->`/`<--`/`=>` morph live + `bind:value` sync, DEV pure-core hook, convergence-lint guard, no-op safety. **Next: M-RP4.1** (kind-3 number-clamp, on `change`) → kind-2 converter field → kind-4 `use:render` (deferred) → dd-components; further composites (`password-field`/`color-picker`/`file-field`/`combobox`/`tag-select`/`star-rating`) reuse the N-054 composite pattern. *(Post-M-RP2.18, both non-build: J-428 reverted both real shells to original chrome — logo + state-indicator + Quit/Shut-Down, throwaway demos removed; J-429 added `led`/`link`/`status-indicator` to the di catalogue, components registry v0.24.)* **Entry (Rule 0): this PLAY → JOURNAL J-446 (M-RP2.23 close: password-field, the 2nd di composite CDP-verified) / J-445 (widget tier concept-lock) → `ui/docs/xgen-ui-notes.md` (… N-057 source-agnostic substitution store, N-058 sampler real config-load path, N-059 the `widget` tier concept-lock, N-060 password-field the 2nd di composite) → `ui/docs/xgen-ui-components.md` (Built-components registry; `textarea` processor-host, source = user list) → `docs/ROADMAP.md` (UI subtree + RP node; M-RP2.23 ✅) → `DECISIONS.md` (D-099 processor taxonomy; D-100 substitution grammar + single-string TOML home + source-agnostic store; D-101 clean-slate-on-start + sampler real config-load path). M-RP4.4 task = COMPLETED; M-RP2.23 task = COMPLETED (runbook). Next-active (reordered J-445/N-059): **M-RP2.23 `password-field` ✅ CLOSED (J-446)** — the first di-composite from the backlog, design-locked, runbook `tasks/M_RP2_23_PASSWORD_FIELD.md` (Step A `textfield` `redactValue`+`autocomplete` additive → Step B composite → skin → sampler/CDP → close). **Next-active = the remaining di-composite backlog** (passive; `color-picker`/`file-field`/`combobox`/`tag-select`/`star-rating`, N-054) → the **`widget` definition** (N-059 promoted to a spec — Level-2 app-assembly tier, home `ui/common`) → **M-RP4.3** (in-app TOML editor + write-back; the first `widget`, PENDING) → M-RP4.1 (kind-3 number-clamp, on `change`) → the kind-2 converter field → kind-4 `use:render` (deferred) → dd-components.** UI sessions additionally read both `ui/docs/` files at open. Not pushed — Joe pushes.

> (Prior PLAY blocks: the AFI/F17 audit-against-code head J-398/J-400 ← doc-opt J-391…J-396 ← J-390 ← … ← M6 / XGID-retrofit — archived to `CLAUDE_HISTORY.md` per D-094, latest at J-435.)  

---

## 🟪 MANDATORY — Behaviour rules (read before doing anything else)

These rules exist because fabricated results have occurred. A summary that says "done" when the work was not actually done causes real damage — wasted sessions, false confidence, incorrect state in CLAUDE.md and JOURNAL.md. Honesty about failure is always better than a fabricated success.

**Rule 0 — Mandatory session-open reading sequence.** On any session open, the FIRST reads are always: (1) CLAUDE.md PLAY block; (2) latest JOURNAL entry; (3) any ACTIVE HANDOFF notes in `tasks/`; (4) THEN whatever document Joe pointed the session at. This holds regardless of how the session is opened: a narrow pointer ("read X" or just a filename pasted) is treated as "expand to context per (1)–(3), THEN read X." Runbook-as-ground-truth is a failure mode; runbooks are item 4 on the reading stack, not item 1. The bridges (PLAY block + JOURNAL + HANDOFF) are the project's structural defences against operational-state drift between sessions; bypassing them produces offers to do work that is stale relative to the actual current state. Rule 0 originated from the post-J-098 session-open failure (recorded in J-099): Chat Claude read a runbook in isolation when the runbook was two commits stale, missing a Joe-lock entirely. Sibling-shape to how D-076 v1.1's amendment made the second load-bearing property of the wire-format principle explicit — same pattern at the meta-level for session-open discipline. Skipping Rule 0 is a Rule 3 stop-and-surface moment in retrospect; the safe pattern is to follow it on every session open without exception.

**Rule 1 — Never fabricate results.** If a command fails, report the failure. Do not describe what the output *should* have been. Do not write a journal entry claiming success until success is actually confirmed.

**Rule 2 — Show actual output, not a description of output.** Every verification step requires quoting real terminal output in the journal entry. Do not paraphrase. Do not summarise. Paste the actual lines. If you cannot produce the actual output, the verification step is not complete.

**Rule 3 — Stop and report when a tool fails.** If a shell command, file operation, or any tool call fails or returns an unexpected result: (1) stop immediately, (2) report exactly what failed and the error, (3) do not attempt to work around it silently, (4) do not write a success summary. Joe will decide how to proceed.

**Rule 4 — Write the journal entry last.** The JOURNAL.md entry is written *after* all work is complete and all verification steps are confirmed with real output. Order: do the work → run verification → confirm outputs → write journal entry quoting actual output → update CLAUDE.md → commit and push.

**Rule 5 — Never invent numbers.** Test counts, file counts, line counts — these must come from actual command output. If you did not run `cargo test`, you do not know the current test count — say so.

**Rule 6 — When in doubt, do less and ask.** If a task instruction is ambiguous, or completing it would require a decision not covered by the instruction file, stop and flag the ambiguity. Do not make the decision silently. Write a clear question to Joe and wait.

**Rule 7 — Definition of Done is a checklist, not a formality.** Every task file ends with a Definition of Done checklist. Each item must be independently verified before being marked complete. Mark items complete only when confirmed with actual output or observation.

| Situation | Correct behaviour |
|---|---|
| Command succeeds | Quote actual output in journal |
| Command fails | Stop, report the exact error, do not continue |
| Tool unavailable | Report it, do not fabricate the result |
| Ambiguous instruction | Ask Joe, do not assume |
| Verification step fails | Stop, report, do not write success summary |
| Unknown test count | Run `cargo test` and quote output — never invent a number |

---

## ✅ DONE — CLI Flag Precedence Audit (D-068): SHIPPED — J-079, 5 atomic commits, 463 tests, five violations closed

**Status: SHIPPED — J-079.** The CLI Precedence Audit (`tasks/CLI_PRECEDENCE_AUDIT.md`, D-068) closed on 2026-05-17 in five atomic commits. The audit surfaced and fixed **five distinct violations**, not just the originally-named `--port` defect: one flag-threading bug (`xgen-node --port` was structurally orphaned from `run_node`) plus four parallel hardcoded subscriber-init blocks (`xgen-client --service`, `--service --ai-mode`, Tauri shell; `xgen-node` Tauri shell) silently bypassing `[logging].level` and falling back to a hardcoded `"debug"` literal. Helpers `xgen_common::precedence::resolve_setting<T>` (generic flag>env>config>default) and `resolve_log_level` (XGEN_LOG-aware specialisation) shipped in commit 1. The two previously-compliant subscriber-init paths (Node `run_node`, Client short-lived CLI) were also refactored onto the canonical helper in commit 3 for consistency and regression-locking. After J-079, **every log-level resolution in the codebase routes through one function** — the drift surface that produced these violations is architecturally eliminated, same shape as M5/D-067 eliminated drift in `xgen-client-lib::ops`. Test count 435 → **463** (+10 unit precedence + 5 URL-rewrite + 6 Node integration + 7 Client integration). Doc sync: Appendix F §F.0.6 updated; DECISIONS.md D-068 gained a closing note; both `main.rs` files' doc comments aligned with §F.0.6.

**Commits:** `3e2f311` helper + tests → `f77fe25` `--port` plumbing → `32028ad` four-site convergence → `1b62fed` integration tests → `19714ad` doc sync.

**Carry-overs:**
- ~~`xgen-client --quiet` doesn't gate the per-subcommand `Connecting to <node>...` line~~ **CLOSED in J-080 (2026-05-18, commit `1d991a4`).** All 10 network-doing shims gain `quiet: bool`; gated per Appendix F §F.0.1.
- ~~Short-lived Client CLI log file lands in `<exe_dir>/logs/` instead of `<data_dir>/logs/`~~ **CLOSED in J-080 (commit `c217844`).** `init_logging` takes `data_dir`; symmetric with `--service` / `--ai-mode --service` / Tauri shell. Per D-035.
- ~~`xgen-node/src/desktop.rs::maybe_write_default_config` writes a non-schema `port = N` field~~ **CLOSED in J-080 (commit `73fbbad`).** `default_config_toml()` now serialises a full `NodeConfig` rooted at `data_dir`; roundtrip-tested.
- Plus the M4 carry-over (`cmd_create_space` optimistic-ack UX): **DEFERRED to M6/M7 design phase** per J-080 §4. Investigation revealed this is not a Client-side UX bug but a missing protocol primitive (no positive accept signal exists today). Context recorded in `tasks/NODE_ADMIN_PASS2_PROPOSALS.md` §"Pass-3 input: missing protocol accept signal" for M6 Pass 3 discussion.

---

## 🟪 DEPRECATED — M6 (original) Multiparty baseline pass with present `--batch`: descoped 2026-05-17

**Status: DEPRECATED.** The original M6 milestone (run the full Multiparty suite S1–S5 twice through present `--batch` to fill the "A" baseline column) is descoped on 2026-05-17. Replaced by **M9 Multiparty Redesign** (see roadmap below).

**Why descoped.** Three reasons surfaced when M6 was about to start after J-079:

1. **Shovel-readiness gap.** The two task files (`tasks/MULTIPARTY_S1_tauri_rerun.md`, `tasks/MULTIPARTY_S2_to_S5_present_pass.md`) were written before J-079 and assumed the binary as it stood at M5. The CLI Audit changed five sites in the logging and flag-resolution paths; the task files do not reflect that. Running them as-is would measure a binary whose behaviour has shifted from what the runbook anticipated.
2. **Metric-protocol applicability needs reconfirmation.** The metric set in `tasks/BATCH_FLAG_review.md` §"Baseline metrics protocol" was Joe-locked in conversation on 2026-05-16 (pre-J-079). Whether the same metrics still apply against the post-audit binary is a question that needs Joe's input, not a question Clair should silently answer at runtime.
3. **The bigger problem M6 was meant to solve has shifted.** M6 existed to create A/B evidence for the `--batch` → `--aicontrol` improvement. With the realisation that `--aicontrol` (and `--batch`) must be **read-write on both Node and Client** — not just Client — the surface that needs validating is bigger than originally scoped. A measurement pass on the Client-only present `--batch` would not produce comparable numbers against an improved surface that spans both binaries.

**What the descope means in practice.** The M6 slot is **reused** for the Node admin write path (see M6 (new) PENDING block below). The multiparty work is rescheduled as **M9 Multiparty Redesign** at the end of the M-series trunk — redesigned to measure both binaries' read-write surfaces (`--batch` and `--aicontrol`) against each other, not the original Client-only `--batch` A/B framing.

**Cross-references.** D-066 (original roadmap) gains a closing note pointing at this descope. D-069 (new this session) records the discipline lesson: delegated technical designs must be Joe-locked AND must flag their own open items before the implementing milestone is declared ACTIVE.

**Affected task files** (flipped to DEPRECATED in this same session):
- `tasks/MULTIPARTY_S1_tauri_rerun.md` → Status: DEPRECATED, pointing at M9
- `tasks/MULTIPARTY_S2_to_S5_present_pass.md` → Status: DEPRECATED, pointing at M9

The metric set in `tasks/BATCH_FLAG_review.md` §"Baseline metrics protocol" is retained as a starting point for M9's design phase; M9 may revise it once the both-binaries scope is locked.

---

## ✅ DONE — Propagation Reliability Audit: CLOSED (J-081, canonical doc shipped, federation gap surfaced)

**Status: SHIPPED — J-081.** Audit closed 2026-05-18. Canonical document at `docs/xgen_propagation_reliability.md`. All five stage sections written under per-section Joe-approval gate. Verdicts: §1 PARTIALLY VERIFIED (Stage 5 local fan-out — mechanism correct, two LOW documentation/observability gaps); **§2 GAP IDENTIFIED HIGH (Stage 6 Node-to-Node federation propagation — architecturally absent)**; §3 GAP IDENTIFIED HIGH (Stage 7 — consequence of §2); §4 PARTIALLY VERIFIED (Stage 8 sync catch-up — works for current workloads, spec-vs-impl + scale gaps); **§5 GAP IDENTIFIED HIGH (`TransportMessage::Error` — wire shape lacks `event_id`, no event-acceptance reject paths emit it).**

**Primary finding.** Node-to-Node federation event propagation does not exist as a production mechanism. Three independent traces converged: (1) `run_initiating` has zero production callers in `xgen-node/src/` — only tests; (2) no pull mechanism — `space.join_request` is only received in production, never sent; (3) stress-test "Federation Completeness" measures local-clients delivery only, not cross-Node propagation — J-059's 6/6 PASS is consistent with and expected from a system with no ongoing federation push. The design doc `docs/xgen_node_admin_ops_design.md` §4.2 sentence describing federation push describes a mechanism that does not exist in the codebase.

**Secondary finding.** `TransportMessage::Error` wire shape ([`xgen-core/src/wire/types.rs:75-82`](xgen-core/src/wire/types.rs:75)) has NO `event_id` field. Single production emit site is identity-replicate failure ([`xgen-node/src/app.rs:1085`](xgen-node/src/app.rs:1085)), not event acceptance. None of the event-acceptance reject paths in `process_inbound` emit `Error` — they all just log via `tracing::error!` + `trace_local(RejectEvent)`. The J-080 framing that "Error is the rejection signal for event acceptance" was confidently wrong across multiple sessions, refuted by direct trace.

**Pattern observation.** The audit found drift surfaces in 4 of 5 sections (§2 design doc federation push, §3 `process_inbound` validation asymmetry, §4 Ch4 §implementation sync flow + unimplemented `sync_response`/`sync_complete`, §5 design doc Error shape + emit paths). Recorded as fact in §6.2 of audit doc. Implication ("subsystem audits precede dependent milestones" as a new project principle) is a project-management conversation Chat Claude + Joe will have post-audit.

**Joe-locked direct during close-out — M6 (new) Phase 2 scope adjustment.** Rather than open a Pass 4 design session for the rejection signal, Joe locked the design call: `event_id: Option<String>` at the `TransportMessage` envelope level (base of the transport-message hierarchy); `EventAccepted` is the only new variant; `Error` covers rejection by populating envelope `event_id`. No new `EventRejected` variant. Practical effect: original 6 Phase 2 deliverables stand + envelope field + wire `Error` into 5 reject paths + client-side correlation. **✅ Documentation pass on the design doc closed 2026-05-18** (§3.1 corrected to locked envelope-level shape, §3.2-§3.4 envelope reference, §9 marked SUPERSEDED with pointer to canonical DECISIONS.md D-070; original Pass-3 §9 body preserved as historical record).

**Carry-overs into downstream milestones:**
- All HIGH-severity findings (§2, §3 peer-side ingestion, §5 Error wire shape, §5 reject-path emission) close in two coordinated downstream items: (a) Federation Event Propagation milestone (see PENDING block below), (b) M6 (new) Phase 2 with the Joe-locked envelope scope.
- `process_inbound` validation asymmetry (Paths B/C skip signature verification) is LOW today but HIGH on federation landing; **precondition** of the Federation Completion milestone, not parallel work.
- No follow-on task files filed (per D-069 discipline — downstream milestones go through their own Joe-locked design phase first).
- 468 tests unchanged — no code changes in this audit.

---

## ✅ DONE — Federation Event Propagation design phase: SHIPPED (canonical doc v1.0 ACTIVE, runbook handed off to Clair)

**Status: SHIPPED — Pass 3 close 2026-05-18.** Design phase closed in same-day session that followed Pass 2. All ten framework decisions locked across F-1 (hybrid push direction) + F-1a (tip exchange) + F-1b (drop-on-peer-down) + F-1c (per-peer record) + F-2 (long-lived continuous session) + F-2 lifecycle + F-2a (one WS per pair bidirectional) + F-3 (event signature + federation relationship verification) + F-4 (unified validation core) + F-4a (30s HeldPending uniform) + F-4b (structural before / semantic after) + F-5 (transitive locked-out v1) + F-6 (sync_complete fold-in) + F-6a/b (wire shape + 5s configurable safety-net) + F-7 (pagination fold-in) + F-7a (1000 default `[sync].batch_size`) + F-8 + F-9 (Ch4 + admin-ops doc corrections at Pass 3) + F-10 (HeldPending extended for unknown signer Identity) + F-10a.

Canonical doc: `docs/xgen_federation_propagation_design.md` (v1.0, Status ACTIVE). Three Pass-2 addenda consolidated into the main doc as §10 (F-7), §11 (F-8), §12 (F-9), §13 (F-10) at Pass 3 and deleted from disk. All `[JOE-LOCK]` markers walked to final form: `[JOE-LOCK: locked 2026-05-18 (Pass 2 conversation, Pass 3 promotion)]`. F-8 corrections applied to `docs/xgen_ch4_implementation.md` §4.11.3 + §4.12.3 (forward-references to canonical design doc; located by content match against unique phrases "per-peer outbound queue" and "Node sends `transport.sync_request` to its peers for the missing predecessors" rather than the audit's stale line numbers). F-9 correction applied to `docs/xgen_node_admin_ops_design.md` §4.2 (Federation propagation Stage-6 sub-bullet now a forward-reference). All in the Pass 3 commit.

**Next: Federation Event Propagation implementation (🟡 PENDING).** Runbook at `tasks/FEDERATION_PROPAGATION_COMPLETION.md` (Status: ACTIVE, v1.0) is the next-active task for Clair. Nine phases: (1) `sync_complete` + pagination wire shape + four-call-site migration, (2) `process_inbound` validation pipeline unification — the precondition, (3) federation handshake reshape to tip exchange, (4) federation event push — the load-bearing phase, (5) per-peer record + reconnect scheduling, (6) HeldPending generalisation for unknown signer Identity, (7) F-3 federation-relationship verification gate, (8) documentation pass, (9) integration tests. **Hard ordering: Phase 2 MUST land before Phase 4 — federation push without validation asymmetry closure lands the audit's HIGH-severity vulnerability vector.** Runbook makes the ordering hard.

**Coordination with M6 (new) Phase 2:** the envelope-level `event_id` on `TransportMessage::Error` work locked at audit close (per M6 design doc §6.5) wires into the rejection paths that this milestone's Phase 2 + Phase 7 produce. M6 (new) is blocked behind this milestone going DONE; M6 Phase 2 ships its wire-layer rejection signal in M6's own milestone.

**Test baseline at runbook handoff: 468.** No code changes in Pass 3.

**Carry-overs at design close:**
- ✅ D-070 promoted to DECISIONS.md (2026-05-18, same-day post-Pass-3): "Two events of equal importance, opposite direction" named protocol principle, with corrected post-audit framing requiring BOTH existence (acceptance + rejection signals) AND envelope-level `event_id` correlation on both directions. M6 design doc §9 draft preserved as historical record; DECISIONS.md D-070 is the canonical authoritative form.
- ✅ D-071 promoted to DECISIONS.md (2026-05-18, same-day post-D-070): "Subsystem audits precede dependent milestones" project-management principle. Sibling to D-065 and D-070 (protocol-design); D-071 names the discipline J-081 retroactively instantiated. Pairs with D-069: audit phase → design phase → implementation phase, each producing a canonical artefact.
- Pass 2 task file (`tasks/FEDERATION_PROPAGATION_DESIGN.md`) and Pass 3 task file (`tasks/FEDERATION_PROPAGATION_PASS_3.md`) both flipped to COMPLETED in the Pass 3 commit.

**Cross-references:** `docs/xgen_federation_propagation_design.md` (canonical, v1.0 ACTIVE). `tasks/FEDERATION_PROPAGATION_COMPLETION.md` (runbook, ACTIVE). `tasks/FEDERATION_PROPAGATION_DESIGN.md` (Pass 2 task, COMPLETED). `tasks/FEDERATION_PROPAGATION_PASS_3.md` (Pass 3 task, COMPLETED). `docs/xgen_propagation_reliability.md` (J-081 audit, ARCHIVED). D-065 (honest behaviour over polite behaviour). D-069 (Joe-locked design phase + canonical-document rule).

---

## 🟡 PENDING — M6 (new) Node admin write path

**Status: PENDING.** Phase 0 (design) closed 2026-05-18: 12 framework decisions locked, canonical design doc shipped at `docs/xgen_node_admin_ops_design.md`. **M6 (new) does not go ACTIVE until BOTH the Propagation Reliability Audit milestone (now ✅ DONE) AND the Federation Event Propagation completion milestone (see PENDING block above) close.** Block 4 (verb-by-verb walks across the seven categories) is also deferred to its own session; the design doc's §6 verb-list sections are stubbed pending Block 4.

**Phase 2 scope adjustment (Joe-locked direct at audit close, no new design pass needed).** The original 6 Phase 2 deliverables in `docs/xgen_node_admin_ops_design.md` §5.2 stand. Added at audit close (J-081 §6.5 of audit doc):

- `event_id: Option<String>` on the `TransportMessage` envelope (base of the transport-message hierarchy), populated when the message pertains to a specific event.
- `EventAccepted` remains the only new variant.
- Event-rejection paths in `process_inbound` ([`xgen-node/src/app.rs:846-851`](xgen-node/src/app.rs:846), [`855-858`](xgen-node/src/app.rs:855), [`885-897`](xgen-node/src/app.rs:885), [`913-921`](xgen-node/src/app.rs:913), [`926-934`](xgen-node/src/app.rs:926)) emit `Error` with `event_id: Some(...)`. No new `EventRejected` variant — `Error` covers rejection by populating envelope `event_id`; `error_code` namespace already encodes semantic meaning.
- Client-side handling correlates envelope `event_id` against in-flight submissions.
- Confirm during implementation: serde derive handles `Option<String>` as omittable for backward-compat with pre-M6 clients (likely yes via `#[serde(skip_serializing_if = "Option::is_none")]`).

Structural realisation latitude for Clair: Rust type design, serde derives, module organisation, internal refactors that preserve wire shape — *cleaner is better*. Wire-format-visible changes beyond the locked envelope `event_id` addition require Joe-lock (threshold: would a future contributor reading the change ask "why was this decided?" — if yes, pause for Joe; if no, ship as normal engineering judgment).

**What this is.** The Node binary today has a partial pipe-server surface: `--batch` shipped in M2 with a **read-only** verb subset (`status`, `connections`, `peers`, `spaces`, `identity list`, `version`, `whoami`). There is no Node-side **write path** for administration. An operator who needs to add a federated peer, register an Auth Module, update Bootstrap configuration, change moderation policy on a hosted Space, or reload config live on a running Node has no automation surface for any of this. `--reload-config` returns honest `NOT_IMPLEMENTED` today.

M6 (new) closes this gap: it ships the Node admin write path as an extension to `--batch` first (humans / scripts / CI), with the same verbs becoming available to `--aicontrol` in M7. The principle is symmetry with the Client: both binaries get full read-write `--batch` AND full read-write `--aicontrol`.

**Why this comes before M7.** `--aicontrol` is the AI-shape protocol over an administrative surface. The surface itself has to exist first. Designing `--aicontrol` Node verbs before the underlying admin subsystems exist would mean designing a JSONL protocol with nothing to call. M6 (new) ships the underlying subsystems; M7 wraps them in the AI-shape protocol.

**Categories of Node admin verbs to design** (sketch — not locked, design-phase deliverable):

- **Federation management** — accept/reject incoming federation requests, initiate federation with a peer, defederate, per-peer allow/deny policy, submit defederation signals to Bootstrap Nodes (§3.15).
- **Auth Module management** — register a new Auth Module, revoke trust, change accepted Tiers.
- **Bootstrap configuration** — register/deregister with Bootstrap Nodes, change `bootstrap_info` metadata, update advertised `auth_tiers_served`.
- **Space and Room operator actions** — force-eject (Node-operator authority, distinct from member-initiated kick), set Node-level moderation policy, trigger Space migration as source Node.
- **Identity registry administration** — revoke a registration with audit trail, update stored Trust Assertion expiry, manage replica relationships.
- **Logging and audit administration** — rotate audit logs, query audit log (read), set log levels per module at runtime (the real `--reload-config` story).
- **Plugin management** — load, configure, unload, query status of moderation plugins (the home of the temperature plugin's runtime surface).

**Design-phase deliverables (must be Joe-locked before M6 is declared ACTIVE, per D-069):**

1. **Verb-set enumeration.** Exact list of verbs per category, with their `args` and `data` schemas. The set probably grows to 30+ verbs.
2. **Privilege model.** Which verbs require what proof of Node-operator authority (the Node keypair? a separate admin keypair? OS-user identity over the pipe?). Today the pipe is unauthenticated on the assumption that pipe-access = Node-operator-on-same-host; whether this holds for write-path verbs is part of the design.
3. **Live-reload semantics.** `--reload-config` becomes a real verb — which config fields are reloadable without restart, which require restart, what the rollback path is on bad config. This is the heart of D-069's "admin makes config updates during the Node's going" use case.
4. **Audit trail integration.** Every write-path verb produces a protocol audit log entry per §3.11.8 (the audit-log facility already specced). Schema additions if needed.
5. **Symmetry with `xgen-client-lib::ops::*`.** The Node equivalent (likely `xgen-node-lib::admin_ops::*` or similar) follows the M5 pattern: one canonical function per verb, three dispatchers (CLI arm, batch arm, future aicontrol arm) all thin shims. No drift surface.

**Not in M6 — explicitly:**
- `--aicontrol` itself. M6 ships the surface that `--aicontrol` will wrap; the wrapping is M7's job.
- Client-side admin verbs. The Client doesn't have an admin role in the same sense; its `ops::*` already covers the Identity-side actions.
- The full canonical `--aicontrol` document. **Already created 2026-05-17** at `docs/xgen_aicontrol_implementation.md`, covering both binaries from day one. M7's design phase resolves its §12 open items and Joe-locks the result; M6 does not edit this document.

**Entry point for the next session:** Federation Event Propagation implementation runbook (`tasks/FEDERATION_PROPAGATION_COMPLETION.md`, Status ACTIVE). M6 (new) sits behind that.

---

## ✅ DONE — M5 `ops::*` refactor: SHIPPED (435 tests, 12 atomic commits, 17/17 smoke PASS, F-003/F-004 architecturally closed)

**Status: SHIPPED — J-078.** Every user-facing `xgen-client` verb (13 total) now routes through a single `xgen-client-lib::ops::<verb>` function. All three dispatchers (`main.rs` CLI arm, `app::run_batch_file` CLI batch driver, `batch::dispatch_line` pipe arm) became thin shims calling the same `ops::*` function; each dispatcher owns its own output format. New `xgen-client/src/session.rs` (`SessionState`, `ClientIdentity`, idempotent `ensure_identity` / `ensure_connected` helpers — extension fields `bindings` / `spaces` present-but-empty for M7-shape stability). New `xgen-client/src/ops.rs` (one `pub async fn <verb>(ctx, args) -> Result<<Verb>Result>` per verb; pure data extraction; the canonical `load_or_default_state` helper). The drift surface that produced F-003/F-004 in J-067 is architecturally eliminated — there is now nowhere a second `get_dag_tips` (or any other implementation duplicate) could be introduced without being noticed. 17/17 smoke PASS against two live Nodes on `:8080`/`:8081` confirms the refactor preserves wire-correct behaviour end-to-end. Test count 429→435 (+6, all from new ops/session unit tests in commits 1-4). D-067 captures the structural outcome.

**Carry-overs:**
- ~~`xgen-node --port <port>` did not override `xgen-node_config.toml::listen` on first invocation during M5 smoke setup; second invocation of the same command succeeded. Flag-vs-config precedence bug in `xgen-node`.~~ **Scheduled as the CLI Precedence Audit (D-068, `tasks/CLI_PRECEDENCE_AUDIT.md`) — see ACTIVE block at the top of this file.**
- Tauri commands for the 13 protocol verbs still don't exist; current Tauri shell is lifecycle-indicator + pipe-server only. When verb-level Tauri commands eventually land they will naturally call `ops::*` — that's M5's prerequisite that's now met.
- `cmd_create_space` optimistic-ack UX bug (J-077, J-078). Future UX pass.

---

## ✅ DONE — M4 AI Client Binary: SHIPPED (429 tests, --ai-mode resident, mention→reply smoke green)

**Status: SHIPPED — J-077.** The AI Client is a *mode of `xgen-client`* (locked §1): `xgen-client --ai-mode --service` runs a long-running headless resident that consumes inbound events through an `AiBehavior` plugin and emits replies under existing pacing + mute constraints. New `xgen-client/src/ai_behavior.rs` (trait + reference `EchoPlugin` with locked deterministic reply format) and `xgen-client/src/ai_service.rs` (runtime loop, `AiPacingTracker` sibling of PacingManager for drop-on-throttle, plugin loader). `__HEALTH__` extended with `mode=ai operator_known=N/M`. Single-Node smoke confirmed: alice mentions bob (AI) → bob replies after `ai_pacing_ms`; back-to-back mention drops the second with literal warn line `dropping reply — pacing cap not yet elapsed (honest behaviour over polite behaviour) ai_pacing_ms=2000`. Spec §6.15 added to Ch6 (10 subsections); D-065 captures M4 architecture AND names the recurring "honest behaviour over polite behaviour" principle with its other instances across the protocol (operator resolution, Node event rejection, mute semantics, the create-space ack bug carry-over).

**Carry-overs (none blocking):**
- ~~`cmd_create_space` doesn't await ack — Client prints "Space created" even on Node-side rejection.~~ **DEFERRED to M6/M7 design phase** in J-080 (2026-05-18). Investigation revealed the underlying problem is not Client-side UX but a missing protocol primitive: no positive accept signal exists today (`xgen-node-lib::fanout` deliberately excludes the originator from fan-out; rationale documented only as a test code comment). Path A: do not speculatively patch fan-out; record the context as a Pass-3 input for M6 design. See `tasks/NODE_ADMIN_PASS2_PROPOSALS.md` §"Pass-3 input: missing protocol accept signal".
- Consolidated Node-side event-accept pipeline. Today's fragmentation (`accept_message` for message.*, dedicated arm for `membership.join`, catch-all `_ =>` for everything else) is fragile. Structural work for a future milestone; not blocking M5 candidates.
- `EventStore` HashMap iteration determinism. Doesn't affect M4 (the AI resident applies events in arrival order, not via sync-request replay).
- `prev_events` integrity for joins from non-members (M3 carry-over, timestamp-sort workaround in `cmd_ai_status` still in place).
- `docs/xgen_appendix_f_en.md` comprehensive example rewrite — Joe's gate of "M2 + M3" reached at M3 close-out; available whenever it surfaces as priority.
- AttachConsole hybrid-app polish (cosmetic Windows console flash).
- Cross-platform pipe server. D-043 still Windows-only.

---

## ✅ DONE — M3 AI Operator Role: SHIPPED (J-075)

411 tests. Operator as distinct role within Spaces (per-(AI, Space)). `SpaceMember.invited_by` + `SpaceState.ai_operator_delegations` + `resolve_operator` three-step fall-upward algorithm (stored delegation → AI's inviter → Space owner, transparently skips members who left). Both `state.space_create` and `state.dm_space_create` from an AI sender rejected with **3041 `ai_role_violation`** (wire name widened from `ai_flag_immutable`; code unchanged). Client CLI: `init --ai [--cap k=v]`, `register` honours `[ai]` config, new `ai delegate`/`ai revoke`/`ai status` subcommand group. Two-Node federation smoke (Rust integration) verifies decision #6's three cross-Node scenarios with strict assertions. Spec §3.6.10.6 rewritten; D-064 captures locked architecture.

---

## ✅ DONE — M2 Node Pipe Server: SHIPPED (J-074)

Six Node-side flags (`--ping`, `--health`, `--stop`, `--reload-config`, plus pipe-side `--batch`) became real implementations. New `xgen-node/src/pipe.rs` ports the Client's pipe-server skeleton to the Node with the four control commands plus a read-only `__BATCH__` subset (status / connections / peers / spaces / identity list / version / whoami). `__HEALTH__` returns the rich `HEALTHY pid=… state=RUNNING conns=… peers=… spaces=… uptime=…s` line. `__RELOAD_CONFIG__` returns honest `NOT_IMPLEMENTED` (real reload is a separate milestone). Pipe server spawns inside `app::run_node` so both `--service` and Tauri get it; `_pipe_shutdown_hold` at the `run_node` async-block scope (J-071 lesson). 391 tests held through M2.

---

## ✅ DONE — M1 Binary Consolidation: SHIPPED (J-073)

Six-commit chain (`e864715` → `c23c06a` → `1da3f1e` → `df877cb` → `4a9243b` → J-073 commit) collapsed four binaries to two: Tauri compiled into both per D-062, library-first dispatch per D-063, all 19 fundamental flags wired, Client `--batch` parallel implementations collapsed, Client `--service` headless resident operational, `cmd_init` instance-aware. Full matrix: 45/49 headless + 4/4 visual cells (N1, N2, C1, C2) confirmed by Joe. Full breakdown: J-068 → J-073.

---

## ✅ DONE — MULTIPARTY_S1 (local fan-out) — first of the five-file Multiparty suite

**Status: COMPLETE — M1 PASS, M2 PASS-with-caveat (J-067, 391 tests pass, 4 bugs found+fixed in-session)**

Detail folded — see prior versions for full F-001 through F-004 history. M1 P1 Smoke PASS; M2 P2 Stress PASS-with-caveat (300 messages dispatched within 96 ms, 294/300 accepted, 6 silently dropped between client WS write and Node receive — cause unclear, follow-up deferred to post-multiparty-redesign).

---

## ✅ DONE — AI Identity, Pacing, and Temperature (D-059, D-060, D-061)

**Status: COMPLETE — 387 tests pass (J-065, 352 xgen-core + 12 xgen-node + 23 xgen-client-lib)**

All three Parts shipped: Part A — AI Identity Extension (D-059, §3.6.10); Part B — Per-Space Pacing Rules (D-060, §3.7.12); Part C — Temperature Property (D-061, §3.7.13). Out of scope deferred: math model that produces temperature values (plugin-owned); Phase 3 Node-side enforcement of pacing / `spontaneous_post`; Svelte UI components; the 13-step manual two-Node verification.

---

## ✅ DONE — Full integration stress test (J-059, 6/6 PASS, 14.6 s, 300 tests)

3-node topology (Node A: 9080, Node B: 9081, Node C: 9082 + Bootstrap). All 6 scenarios pass, 43/43 checks. Two bugs found and fixed during live run: stack overflow in large async fn (32 MB thread dispatch), B↔C federation recv hang (replaced with explicit goodbye). Comm record at `docs/tests/stress_complete_events.json`.

---

## ✅ DONE — Phase 2 integration testing (60/60 PASS, D-054–D-056a, J-056–J-058, 300 tests)

All Phase 2 protocol layers (11–19) complete. Integration smoke test `smoke-ph2` passes all 60 steps against two live `xgen-node` processes over real TCP. One transport-layer bug discovered and fixed during the live run (D-056a — `recv()` routing collision between DAG Events and control messages on shared type-prefix strings).

---

## ✅ DONE — Phase 2 Track 1 infrastructure (Sessions 14–18, 173 → 300 tests through this phase)

Tauri scaffold, 11 Client lifecycle states + 7 Node + degraded stacking, named pipe IPC (D-043), `--instance` flag, `--batch` flag, xgen-core crate split (D-022, D-044). Detailed table folded — see prior versions or the per-instruction-file headers under `docs/tests/` for full breakdown.

---

## ✅ PHASE 1 IS COMPLETE — DO NOT RE-IMPLEMENT

All Phase 1 deliverables done: binary wiring, 17-step smoke test against real TCP, documentation gates, stress test. Tag `v0.10.3`. See historical snapshot below for the layer-by-layer record.

---

## ✅ DONE — Phase 1 logging + event tracing

Phase 1 debug logging (`docs/tests/LOGGING_debug_ph1.md` — J-025): datetime-stamped log files, config level switch, subscriber init, operational log calls in both binaries. Audit log (`docs/tests/LOGGING_audit_ph2.md`) deferred — alongside Tier 2+ Auth Module work only.

Global Event tracing interface (`docs/tests/LOGGING_debug_ph2.md` — J-027, J-029): `event_trace` module in `xgen-common/src/` (Fix 17 applied). `Event` and `EventType` moved to `xgen-common/src/wire.rs`. Role gate active. Content field never logged. 173/173 tests; smoke test with debug logging confirmed full Event pairing across client and both Nodes.

---

## ✅ DONE — Documentation fixes (FIXES_ph1.md)

All 17 fixes applied (Fix 14 deferred by project owner). Fix 16 (Node space state replay on restart) and Fix 17 (event_trace relocation) complete in Rust source. Documentation fixes 1–15 applied to Ch3/Ch4.

---

## ⏸ POSTPONED — UI Phase 2 prep (run 1.5)

UI design work for Phase 2 Track 1 is paused at the element-modelling step (J-033, 2026-05-08). Resume condition: confirmed absent-element list in `ui/docs/xgen-ui-design-brainstorm.md` (Points 2 and 3) reconciled with Ch3's authoritative event taxonomy + Run 3 design briefing drafted. Until those gate, no visual merge work begins. Recorded in `JOURNAL.md` J-033 and `DECISIONS.md` D-041.

---

XGen Protocol is an open, federated, identity-verified communication protocol. Think of what Discord would have been if built as open infrastructure. The core thesis: no single entity should own the communication layer.

This is not a product — it is protocol infrastructure. Phase 1 is a minimal working implementation. Phase 2 is the full protocol. Phase 3+ is everything else.

**The spec is authoritative.** When this file and the spec conflict, the spec wins. When the spec is ambiguous, flag it — do not resolve it silently.

---

## Current State — Where We Are

**Federation Event Propagation implementation Phases 1-8 shipped (J-082 + J-083 + J-084 + J-085 + J-086 + J-087 + J-088 + J-089 across 2026-05-18 and 2026-05-19).** Phase 8 (J-089) closes the documentation pass — six accumulated doc-vs-code drift surfaces from Phases 5-7 fixed, plus the standard "forward-reference → implementation-complete" updates. Tests: 468 (handoff) → 476 (Phase 1) → 480 (Phase 2) → 488 (Phase 3) → 491 (Phase 4) → 505 (Phase 5) → 516 (Phase 6) → 519 (Phase 7) → **519** (Phase 8 close — documentation only per DoD). Next active phase: Phase 9 (deployment-level integration tests, six DoD scenarios). After Phase 9 ships, milestone flips PLAY → DONE and M6 (new) unblocks. Roadmap: M5 ✅ → CLI Audit ✅ → J-080 ✅ → M6 Phase 0 Pass 3 ✅ → Propagation Reliability Audit ✅ → Federation design (Pass 2 + Pass 3) ✅ → **Federation implementation Phases 1-8 ✅, Phase 9 (deployment integration tests) next** → M6 (new) → M7 → M8 → M9.

Current project status as of 2026-05-19:

- **Phase 1**: complete (J-029, tag `v0.10.3`, 17-step smoke test passing over real TCP). See historical snapshot below.
- **Phase 2 protocol**: complete (J-058, `smoke-ph2` 60/60 PASS, layers 11–19 all shipped).
- **Phase 2 Track 1 UI**: partially complete; deeper visual-merge work POSTPONED.
- **Post-Phase-2 protocol work shipped:** AI Identity + Pacing + Temperature (J-065), full integration stress test (J-059).
- **M1–M5 shipped**: binary consolidation, Node pipe server, AI operator role, AI Client resident mode, ops refactor.
- **CLI Audit shipped (D-068)**: J-079, 5 atomic commits, 463 tests, five violations closed.
- **J-080 carry-over pass**: 468 tests; 3 of 4 carry-overs closed; item 4 deferred to M6 design.
- **M6 Phase 0 closed 2026-05-18**: 12 framework decisions locked, canonical design doc shipped at `docs/xgen_node_admin_ops_design.md`.
- **Propagation Reliability Audit CLOSED (J-081, 2026-05-18)**: 4 of 5 sections found drift; Stage 6 federation propagation architecturally absent.
- **Federation Event Propagation design phase**: SHIPPED (Pass 2 + Pass 3 closed 2026-05-18). Canonical design doc at `docs/xgen_federation_propagation_design.md` (v1.0, Status ACTIVE) — all 10 F-items locked, three Pass-2 addenda consolidated as §10–§13, F-8 + F-9 corrections shipped.
- **Federation Event Propagation implementation**: 🟢 PLAY. Nine-phase runbook; **Phases 1-8 ✅ SHIPPED** (J-082..J-089). **Phase 8 ✅ (J-089, 2026-05-19)** — Documentation pass closing the six accumulated doc-vs-code drift surfaces from Phases 5-7 plus the standard forward-reference → implementation-complete updates. Ch3 §3.3.6 wire shape rewritten to shipped `{ protocol_version, since, new_tip, continue_from }`; Ch3 §3.9.6 + §3.9.8 add `4006 identity_record_timeout` with predecessor-code-wins sub-rule; Ch4 §4.11.2 rewritten to JSON-backed `FederationRegistry`; Ch4 §4.11.3 + §4.12.3 + admin-ops §4.2 forward-references → implementation-complete; design doc §6.4 leading authority paragraph names `SpaceState.federation_nodes` + B1 implementation note; new design doc §15 Implementation Complete records all eight shipped phases. 519 tests at Phase 8 close (unchanged — documentation only). Phase 9 (deployment-level integration tests covering six DoD scenarios) is next-active.
- **M6 (new)**: Node admin write path PENDING. Phase 0 design closed; ACTIVE flip waits behind Federation Event Propagation milestone closure.
- **Phase 3 areas**: state migration depth, federation depth, MLS operationalisation. D3 (MLS) parallel.

### Historical snapshot — Phase 1 completion (April 2026, tag `v0.10.3`, 173 tests)

This table records how Phase 1 landed and is preserved as a historical reference. Test counts and tags are frozen as of April 2026; current counts and milestones are above.

| Layer | Content | Tests | Tag |
|---|---|---|---|
| 1 | Crypto (Ed25519, SHA-256, base64url, ChaCha20+Argon2id) | 25 | v0.1.1 |
| 2 | Wire format (Event, EventType, framing, validation steps 1–7) | 53 | v0.2.2 |
| 3 | DAG event store (append-only, tips, pending buffer) | 79 | v0.3.2 |
| 4 | WebSocket transport (challenge-response auth, keepalive) | 88 | v0.4.2 |
| 5 | Node identity and announcement | 100 | v0.5.2 |
| 6 | Federation handshake (state machine, registry) | 121 | v0.6.2 |
| 7 | Identity registration (8-step pipeline, registry) | 142 | v0.7.2 |
| 8 | Space and Room protocol (state machine, roles, permissions) | 160 | v0.8.2 |
| 9 | Message exchange (validation steps 8–13, accept_event) | 171 | v0.9.3 |
| 10 | Smoke test — spec 3.7.11, 17-step end-to-end | 173 | v0.10.1 |
| CLI | init, status, connections, spaces, peers, identity list, whoami (D-025–D-028) | 173 | v0.10.2 |
| Binaries | xgen-node WebSocket server + xgen-client network commands + 17-step smoke test over real TCP | 173 | v0.10.3 |

---

## Architecture Rules — Non-Negotiable

**1. Library-first.** All protocol logic lives in `lib.rs`. `main.rs` is a thin CLI shell only — argument parsing, startup, shutdown. No business logic in `main.rs`. This is what makes Phase 2 Tauri integration possible without rewriting.

**2. Spec is authoritative.** `docs/xgen_ch3_specification.md` is the source of truth. `IMPLEMENTATION_GUIDE_ph1.md` is the implementation guide. When they conflict, the spec wins.

**3. Verify after every write.** Read back every file after writing it. Silent write failures have caused reconstruction work in past sessions.

**4. DECISIONS.md before advancing.** Every implementation decision beyond spec prescription must be recorded in `DECISIONS.md` before moving to the next layer. Format: title, date, layer, spec reference, decision narrative.

**5. Tests before advancing.** Run `cargo test` and confirm all tests pass before moving to the next layer. Do not skip.

---

## File Placement Rules (D-025 — Updated)

All runtime files are prefixed with the binary name. **`xgen-node_*` for all Node files, `xgen-client_*` for all client files.**

**Tier 1 — System files: mandatory co-location with binary, not configurable**

| File | Binary | Description |
|---|---|---|
| `xgen-node_config.toml` | xgen-node | Node configuration (TOML) |
| `xgen-node_state.json` | xgen-node | Live status snapshot, written every 5s (D-026) |
| `xgen-node_identities.db` | xgen-node | Identity registry (SQLite) |
| `xgen-node_federation.json` | xgen-node | Federation registry (JSON-backed `FederationRegistry`) |
| `xgen-client_config.toml` | xgen-client | Client configuration (TOML) |
| `xgen-client_state.json` | xgen-client | Identity, known nodes, joined spaces |

**Tier 2 — User-configurable files: default to binary folder, redirectable via config**

| File | Config field | Description |
|---|---|---|
| `xgen-node_keypair.enc` | `keypair_path` | Ed25519 private key — may redirect to HSM or secure share |
| `xgen-client_keypair.enc` | `keypair_path` | Ed25519 private key — may redirect to OS keystore (Phase 2) |
| Log output | `log_path` | May route to system log aggregator |

No file moves silently. Every Tier 2 redirect is explicit in config.

---

## meta_atts Key Namespace Rules (Spec 3.1.3)

`meta_atts` keys use dot-separated namespaces:

- `xgen.*` — **reserved** for protocol use only. Examples: `xgen.client`, `xgen.thread_id`, `xgen.tags`
- Third-party keys MUST use reverse-domain prefix. Examples: `com.example.priority`, `org.myapp.color`
- All lowercase, snake_case segments, dots as separators, no hyphens
- Max key length: 128 characters
- Values are strings. Structured values are JSON-encoded strings, not nested objects.

---

## Error Code Convention

Error codes are plain integers on the wire and in exit codes (e.g. `4002`). For human-readable display in logs, UI, and documentation, codes are shown with an `E` prefix and zero-padded to 6 digits (e.g. `E004002`). The `E` prefix is display-only — never transmitted, never used programmatically. `E004002` and `4002` are the same error.

Domain ranges: 1000–1999 transport, 2000–2999 federation, 3000–3999 identity, 4000–4999 state resolution, 5000–5999 E2E encryption, 6000–6999 migration, 7000–7999 bootstrap, 8000–8999 reputation, 9000–9999 DM promotion. Future domains extend naturally: domain 10 = 10000–10999, etc.

---

## Transport Pluggability (Spec 3.3.1)

WebSocket over TLS is the mandatory production transport. The protocol also explicitly permits Tor hidden services, I2P, and pluggable transport proxies as alternative stream transports — no protocol changes required. Phase 1 uses `ws://` localhost only. Production uses `wss://`. DPI resistance is a Phase 3 area; no Phase 1 impact.

---

## Key Cryptographic Decisions

- **Keypair encryption at rest:** ChaCha20-Poly1305 + Argon2id KDF. Phase 1 local node uses empty passphrase (file still encrypted for integrity).
- **Event ID derivation:** SHA-256 hash of canonical JSON → `xgen://hash/sha256:<hex>`
- **Signature format:** `ed25519:<base64url-pubkey>:<base64url-sig>` — covers canonical form only, not wire bytes
- **Canonical form:** fixed field order, no whitespace, object keys sorted lexicographically, `event_id` and `signature` excluded
- **DAG root types:** `state.space_create`, `state.room_create`, `state.dm_space_create` require empty `prev_events`. All others require at least one.
- **Cycle detection:** reduces to self-reference check only at insertion time (append-only store invariant)
- **prev_events fanin limit:** 10 (Phase 1)
- **Node announcement TTL:** 90 days
- **Session ID derivation:** `hash_uri(sort([node_a_id, node_b_id]) + timestamp)` — sorted so both sides derive same value

---

## Versioning Scheme

`[state].[layer].[session]` — three components, stored in `Cargo.toml`.

- `state`: 0 while building Phases 1 and 2; 1 when Phase 1 and Phase 2 complete and stable
- `layer`: implementation layer number (1–10)
- `session`: work session in which that layer was completed

Tags are monotonically increasing: `v0.1.1` → `v0.2.2` → … → `v0.10.x`

---

## Phase 2 — Status

Phase 2 shipped in two tracks. Both reached their Phase-2 deliverables; deeper work in each track has been scheduled as separate milestones.

### Track 1 — UI infrastructure (Phase-2 deliverables shipped; visual merge POSTPONED)

The Tauri scaffolding, lifecycle state machines, named pipe IPC, `--instance` segregation, `--batch` flag, and `xgen-core` crate split all landed during Sessions 14–18. Both binaries open windows with custom chrome; lifecycle states from Appendix E are wired; Node systray works with state-coloured icons; first-run SETUP is functional; `--service` headless mode works on both binaries.

The **visual merge of design Claude's chat mockups onto Miss Design's semantic skeleton** is POSTPONED at the element-modelling step (J-033). The gating condition has not been met; see the `⏸ POSTPONED — UI Phase 2 prep (run 1.5)` section earlier in this file for the full status.

### Track 2 — Protocol (Phase-2 deliverables shipped; Phase 3 areas open)

All Phase-2 protocol layers (11–19) shipped. `smoke-ph2` runs 60/60 PASS. `stress-complete` runs 6/6 PASS. xgen-core crate split landed at J-045; dual-licence boundary in place.

Post-Phase-2 protocol work shipped: AI Identity + per-Space pacing + temperature property (D-059/D-060/D-061, J-065); M1–M5 series.

**Phase 3 areas — specced but unimplemented:**

| Area | Status | Reference |
|---|---|---|
| State migration depth | Wire shape specced (3.12, Layer 14); deep testing pending | Future milestone (folded into M8) |
| Federation depth | Foundational gap closes in Federation Event Propagation milestone; deeper work (N-Node topologies, defederation flow, reputation merge) folded into M8 | Federation Event Propagation milestone (PENDING) + M8 |
| MLS operationalisation | Wire shape specced (3.10, Appendix I Part X.6); openmls integration pending | Future milestone (D3, parallel workstream alongside M-series) |
| `self` account | Local-only synthetic Identity, accessible from any client | D-021 — deferred |
| Registry file encryption | Identity and federation registries at rest | Deferred |
| Slovak translation pass | Single pass after full document completion | Deferred |
| DPI resistance | Investigation only | D-023 — Phase 3 |

**Roadmap:** M5 ✅ → CLI Audit ✅ → J-080 ✅ → M6 Phase 0 Pass 3 ✅ → Propagation Reliability Audit ✅ → Federation design (Pass 2 + Pass 3) ✅ → **Federation implementation Phases 1-5 ✅, Phase 6 (F-10 HeldPending generalisation) next** → ~~M6 multiparty~~ DEPRECATED → M6 (new) → M7 → M8 → M9. D3 (MLS) parallel.

---

## Repository Layout

```
docs/
  xgen_ch0_content.md             # table of contents
  xgen_ch1_philosophy.md          # philosophy, motivation
  xgen_ch2_architecture.md        # architecture, primitives, deployment model
  xgen_ch3_specification.md       # AUTHORITATIVE SPEC (§3.1–3.16 complete)
  xgen_ch4_implementation.md      # Phase 1 complete; Phase 2 scope defined
  xgen_ch5_protocol.md            # stub
  xgen_ch6_client_design.md       # UI architecture
  xgen_appendix_*.md              # supporting appendices
  xgen_federation_propagation_design.md      # Canonical Federation Event Propagation design (v1.0, ACTIVE — Pass 3 consolidated)
  xgen_propagation_reliability.md            # J-081 audit canonical doc
  xgen_node_admin_ops_design.md              # M6 Phase 0 canonical design doc
  ROADMAP.md                                  # Coarse-grained project navigation map
tasks/
  FEDERATION_PROPAGATION_DESIGN.md           # Pass 2 task file (COMPLETED at Pass 3 close)
  FEDERATION_PROPAGATION_PASS_3.md           # Pass 3 task file (COMPLETED at session close)
  FEDERATION_PROPAGATION_COMPLETION.md       # Implementation runbook for Clair (Status ACTIVE)
  ... (other task files for past milestones)
ui/
  ... (UI skeletons, postponed work)
IMPLEMENTATION_GUIDE_ph1.md       # Phase 1 layer-by-layer guide — COMPLETED
IMPLEMENTATION_GUIDE_ph2.md       # Phase 2 layer-by-layer guide
DECISIONS.md                      # Implementation decision log (D-000 through D-069)
JOURNAL.md                        # Contemporaneous development journal (IP record)
CLAUDE.md                         # This file
LICENSE                           # BSL 1.1
```

Source crates:
```
xgen-common/    # shared types (no runtime, no I/O) — BSL 1.1
xgen-core/      # all protocol logic — GPL-2.0-or-later (created in Phase 2 crate split)
xgen-node/      # thin Node shell — main.rs + lifecycle, depends on xgen-core — BSL 1.1
xgen-client/    # thin client shell — main.rs + commands, depends on xgen-core — BSL 1.1
```

Build target directory is kept outside the project folder to avoid file locking:
```
C:/cargo-targets/XGenProtocol
```

---

## Document Header Convention

### Core pattern

```
# Title
> **Status**: {}  
> Version: {}  
> Date: {MMM YYYY}  
> **Last updated**: YYYY-MM-DD  
> Language: {}  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  
```

### Specification

- Every `> ...` line requires **two trailing spaces before EOL** (mandatory for correct line rendering)
- `{MMM YYYY}` = month-name + year, e.g. `May 2026`
- **This header MUST be updated on every file edit**

Status values:
- `ACTIVE` — current, act on it
- `PENDING` — written, not yet the current task
- `COMPLETED` — done, do not re-execute
- `DEPRECATED` — no longer valid / replaced — replacement named if applicable
- `ARCHIVED` — frozen historical record, do not modify

**When looking for the next task**, scan `tasks/` and `docs/tests/` file headers. The next instruction file to run is the first one with `PENDING` or `ACTIVE` status that is not explicitly deferred.

**Note on folder convention:** New instruction files for Code Claude are written to `tasks/` at the project root (not under `docs/`). The `docs/tests/` folder holds the legacy instruction files written before this convention; it stays in place until a future cleanup migrates everything to `tasks/`. Both folders are scanned for `PENDING`/`ACTIVE` files.

---

## License Header

Every source file MUST carry this exact header:

```rust
// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.
```

Not PolyForm. Not MIT. Not any other license. BSL 1.1 exactly as above.

---

## Build Commands

```sh
cargo build                              # debug build
cargo build --release                    # release build
cargo test                               # run all tests
cargo test smoke                         # run smoke test only
cargo test --package xgen-common         # test one crate
```

Build output goes to `C:/cargo-targets/XGenProtocol` (set via `CARGO_TARGET_DIR` in `build.sh`). Binaries are copied to `bin/` in the project folder by `build.sh`.

---

*Read `DECISIONS.md` (current range D-000 through D-075) before making any decision that isn't explicitly covered by the spec. If you're unsure whether something needs a DECISIONS.md entry, it does.*
