# M-RP2.12 — `textfield` constrained `type` prop (string-input family fold) + per-type inset icons
> **Status**: COMPLETED  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-25  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Goal

Give `textfield` a constrained `type` prop, folding the structurally-identical string-`<input>` family into the one component: whitelist `text` (default) `| search | email | url | tel | password`. All share the `<input>` root, string `bind:value`, and `.textfield` skin — they differ only in UA-supplied validation / keyboard / masking. **Reverses N-029** ("type is fixed, not a prop") — lands `DECISIONS.md` D-096 (the re-lock, with the code). Excludes `number`/`range`/`date`/`color`/`file` (own atomics — value-type/structure differ). Adds a per-type **very-weak-grey inset icon** as a look-distinguisher (skin treatment).

## Locks (design walk, Joe-locked 2026-06-25 "all by your recomms" + Q3 icon addition + password-reveal deferral)

1. **Prop `type`, TS union, default `'text'`** — `'text'|'search'|'email'|'url'|'tel'|'password'`. **No runtime guard, no DEV-warn:** the TS union is the boundary (consumer is TS), and an out-of-whitelist value degrades safely (browser normalizes unknown `type`→`text`) — unlike image's `alt`, the type system has a safe native fallback, so a guard would be dead weight (D-065).
2. **Getter → `{ type, value }`** — `type` is meaningful, identity-changing state; carrying it makes the milestone self-verifying through the N-024 registry (the image-`alt` precedent: carry it so verify confirms it landed). No separate DOM probe needed for the prop-path proof.
3. **Per-type inset icons (skin, L2)** — same mechanism as `select`'s arrow: per-type `background-image` inline-SVG, right-inset, keyed by attribute selector `.textfield[type="…"]`. **`textfield.svelte` stays zero-`<style>`** (icons are pure appearance → skin, N-031 litmus). Glyph map: `text` **none** · `search` magnifier · `password` ⁂ (three asterisks) · `email` envelope · `url` link knot · `tel` handset. All glyphs **very-weak grey `#e6e6e6`** (the `img-placeholder` light-grey — lighter than `--t4`/`--t3`, reads as a weak hint; literal inside each SVG as `%23e6e6e6`, not a `:root` token — same pattern as `select`'s `%238a8880` arrow).
4. **`search` native clear-"x" suppressed** — `::-webkit-search-cancel-button { appearance: none; -webkit-appearance: none; }` so the magnifier sits clean (the UA clear-x is also right-edge, would collide).
5. **Right-padding clearance per-type** — iconed types carry the right-padding bump (`calc(var(--sp-4) + var(--sp-1))`, mirroring `select`); `text` keeps the default `--sp-2` (no icon, no bump).
6. **`use:envelope` unchanged** — content-agnostic; `type` is a native attribute passthrough. Substrate confirmed unaffected (Phase-0).
7. **`maxlength` stays OUT** — orthogonal to `type` (a native-state-surface addition, not part of the fold). Out of scope.
8. **Password reveal is NOT in the atomic** — atomic `textfield type="password"` stays pure (masks + static ⁂ icon, no reveal). A readable/reveal toggle is an interactive child → breaks atomicity → ships as the **`password-field` composite** (already named in N-038 / D-096), deferred to the first-composites track.

## Phases

**Phase 1 — author** `ui/core/lib/components/data-independent/textfield.svelte`:
- Add `type` to `$props()` with the TS union and default `'text'`.
- Root `<input type={type} …>` (was hardcoded `type="text"`).
- Getter `debug = () => $state.snapshot({ type, value })`.
- **Rewrite the header comment block** — the current block asserts the N-029 "`type` is fixed, NOT a prop" rule; replace with the folded-family framing + a `→ D-096` pointer. Keep the processor-ready / native-state-surface notes.
- Zero `<style>` preserved.

**Phase 2 — skin** `skin.css`, appended/extended in the `.textfield` block:
- Shared iconed rule (grouped selector `.textfield[type="search"], …[type="email"], …[type="url"], …[type="tel"], …[type="password"]`): `padding-right: calc(var(--sp-4) + var(--sp-1))`, `background-repeat: no-repeat`, `background-position: right var(--sp-2) center`.
- Per-type `background-image: url("data:image/svg+xml,…")` (5 inline-SVG glyphs @ `%23e6e6e6`, ~12px viewBox).
- `.textfield[type="search"]::-webkit-search-cancel-button { appearance: none; -webkit-appearance: none; }`.
- No `:root` token added. `text` untouched (no icon, default padding).

**Phase 3 — wire demo, both shells** (`app_client.svelte` + `app_node.svelte`):
- `let demoSearch = $state('')` (throwaway, with the M-RP2.12 comment).
- Mount `<Textfield type="search" bind:value={demoSearch} id="demo-search" />` after the existing `id="demo"` textfield. (One non-default authored instance proves the prop→input path + bind on a non-text type; the existing `id="demo"` instance proves the `text` default holds.)

**Phase 4 — CDP verify both apps** (Chat self-drives, real `tauri dev` + CDP; N-028 race-retry + clean teardown):
- Registry: `textfield#demo` → `{type:"text",value:""}` (default holds) and `textfield#demo-search` → `{type:"search",value:""}` → dispatched `input` event → `{type:"search",value:"…"}` (string bind-in re-proven on a non-text type, N-029 dispatched-event subtlety).
- `el.type` sweep: set `type` across all six whitelist values, read back `el.type` — browser-acceptance + per-type quirk observation.
- Computed-style: `background-image !== "none"` for search/email/url/tel/password, `=== "none"` for text; confirm email/url native type-validation shares the existing `.textfield:invalid` red-border look (no conflict).
- Screenshots both apps — eye-check the six glyphs render very-weak-grey + right-inset, and the `search` clear-x is gone.
- Clean teardown (ports 9222/9322/5173/5174 free, 0 orphans).

**Phase 5 — DECISIONS D-096** (the fold decision only — the icons are skin, recorded in notes, not here):
> **D-096 — `textfield` `type` folds the string-input family into one component (reverses N-029)**. Atomic discriminator = root structure + value-type, not the `type` literal. String-`<input>` types sharing root + `bind:value` + `.textfield` skin fold into one `type` prop (whitelist `text|search|email|url|tel|password`); value-type-changing/chrome-adding types (`number`/`range`/`date`/`color`/`file`) stay own atomics; custom-chromed variants (`password-field` eye-toggle, custom stepper) are composites. Why: one file/one skin for the family; native passthrough degrades safely (unknown→text); the N-038 catalogue boundary gives the principled atomic/shape/composite line.

**Phase 6 — records (D-074 atomic, same-commit):**
- `ui/docs/xgen-ui-notes.md` **N-039** — the fold (+ `→ D-096`), the getter `{type,value}` change, the per-type icon family (the look-distinguisher + `#e6e6e6` literal + `select`-arrow mechanism), the `::-webkit-search-cancel-button` suppress, the verify. Notes-only home for the icon treatment.
- `ui/docs/xgen-ui-components.md` (v0.16→**0.17**) — Built `textfield` row: ref `N-022/N-024/N-029/N-038/N-039`, getter `{type,value}`, root note `<input type=…>`; rewrite the `textfield` detail para (folded family, no longer "type fixed"); fold the di-catalogue table rows (*constrained text* / *secret* / *search-field shape*) into the `textfield` row with a note.
- `docs/ROADMAP.md` — RP node M-RP2.12 ✅ + frontier advance; version bump.
- `CLAUDE.md` PLAY → M-RP2.12 ✅ CLOSED / Next: first composites (incl. `password-field` reveal) / pointer J-416→J-417.
- This task → **COMPLETED**.
- `JOURNAL.md` **J-417** (written last, Rule 4, real CDP output quoted).
- All touched `.md` `Last updated` bumped to close date.

## Definition of Done

- [x] `textfield.svelte`: `type` prop (TS union, default `text`), root `type={type}`, getter `{type,value}`, header comment rewritten (N-029 reversal), zero `<style>` preserved
- [x] `skin.css`: 5 per-type icon rules (`#e6e6e6` inline-SVG, right-inset) + grouped padding/position + `::-webkit-search-cancel-button` suppress; no new `:root` token; `text` untouched
- [x] demo wired both shells (`type="search"` `id="demo-search"` + `demoSearch` state)
- [x] CDP both apps: `textfield#demo` `{type:"text",value:""}` + `textfield#demo-search` `{type:"search",value:""}` + `input` delta `{value:"find me"/"node find"}` (real output in J-417)
- [x] CDP `el.type` sweep all six whitelist values round-trip (real output, both apps)
- [x] CDP computed `background-image` present (5 iconed) / none (text); `:invalid`→`--err` shared by email-validation + `pattern` (detached, both apps)
- [x] screenshots both apps: magnifier (search) + plain (no icon); client also email red-border + envelope. url/tel/password glyphs proven present via computed `background-image` + `el.type` sweep, not individually eyeballed
- [x] clean teardown (0 orphan ports)
- [x] D-096 recorded (fold only)
- [x] records updated (N-039, components v0.17 + catalogue fold, ROADMAP, CLAUDE PLAY, JOURNAL J-417, task COMPLETED)
