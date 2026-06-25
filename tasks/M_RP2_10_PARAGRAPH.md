# M-RP2.10 — display-di `paragraph` (root <p>, single-paragraph prose) + found the `--fs-*` type scale
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

Author + skin (one pass) the second **display-kind di**: `paragraph` — root `<p>`, a single paragraph of prose (N-032). Read-only, value-carrying, the display half of the di model. AND **found the `--fs-*` type-size scale** (deferred here from M-RP2.9), retro-keying the four shipped components onto it.

## Locks (design walk, Joe-locked 2026-06-25 "go by your recomm")

1. **Formatter seam = text-node today.** Render `<p use:envelope>{text}</p>` — a plain text node (no `{@html}`). The reserved inline-mark formatter (N-032: `_x_`/`*x*`, whitelist `<strong>`/`<em>`/`<br>`, escape char) is deferred to a future `common` `use:render` action — the render-side counterpart to the edit-side `use:processor`. The action will own the delimiter map + whitelist + sanitization and rewrite node content only when applied; paragraph never opens `{@html}`. **Not built now** (D-065); seam documented.
2. **Found `--fs-1: 12px` + `--fs-2: 14px` only.** No `--fs-3`/`--fs-4` seed — no current consumer (D-065); grow the scale when a heading/lead component needs it. Numeric ascending=larger, matches `--sp-*`/`--rad`; avoids the `--t*` color-ramp collision.
3. **Found `--lh: 1.5`.** Retro-key the four shipped `line-height: 1.5` too (folded into the same retro-key pass; avoids a second later).
4. **Retro-key the four shipped skins** (`.button`/`.textfield`/`.select`/`.label`): `font-size: 12px` → `var(--fs-1)`, `line-height: 1.5` → `var(--lh)`. Components stay zero-`<style>`; CDP re-verify confirms all four still resolve to 12px.
5. **`.paragraph` skin:** `font-size: var(--fs-2)`, `color: var(--t)` (content, brighter than label's caption `--t2`), `line-height: var(--lh)`, `margin-block-end: var(--sp-3)`.

**Inherited (label/N-032, confirm not re-litigate):** value prop `text` · debug getter `{text}` · `use:envelope` unchanged · block is native to `<p>` (xgen-normalize floor: `*{margin:0}` zeroes UA margins, `p{font-size/weight/line-height:inherit}`).

## Phases

**Phase 1 — tokens + retro-key** in `ui/assets/skin.css`: add `--fs-1`/`--fs-2`/`--lh` to `:root` (after `--ctl-h`); replace the four `font-size: 12px` → `var(--fs-1)` and four `line-height: 1.5` → `var(--lh)` in `.button`/`.textfield`/`.select`/`.label`.

**Phase 2 — author** `ui/core/lib/components/data-independent/paragraph.svelte`: `text` prop + `id`; `use:envelope={{ name:'paragraph', id, debug }}`; `debug = () => $state.snapshot({ text })`; text-node body `{text}`; zero `<style>`; seam documented in the header comment.

**Phase 3 — skin** `.paragraph` appended to `skin.css` (lock 5).

**Phase 4 — wire demo, both shells** (`app_client.svelte` + `app_node.svelte`): import `Paragraph`; mount `<Paragraph text="..." id="demo" />` beside the others. No `$state` var (read-only).

**Phase 5 — CDP verify both apps:** `paragraph#demo` → `{text}`; computed-style `.paragraph` → `font-size: 14px` (`--fs-2`), `color: rgb(236,233,225)` (`--t`); **re-verify the four retro-keyed still = 12px / 1.5**; screenshots; clean teardown.

**Phase 6 — records (D-074 atomic):** notes N-036; components Built row + detail (v0.14→0.15); ROADMAP RP node + frontier (v3.94→3.95); CLAUDE PLAY → M-RP2.10 CLOSED / Next `image` / pointer J-414→J-415; this task → COMPLETED; JOURNAL J-415. All `.md` `Last updated` bumped.

## Definition of Done

- [x] `--fs-1`/`--fs-2`/`--lh` founded; four shipped skins retro-keyed (font-size + line-height)
- [x] `paragraph.svelte` authored (text prop, envelope, `{text}` getter, text-node body, zero `<style>`, seam documented)
- [x] `.paragraph` skinned (`--fs-2`, `--t`, `--lh`, `--sp-3` spacing)
- [x] demo wired both shells
- [x] CDP both apps: `paragraph#demo` `{text}` + computed-style 14px/`--t`; four retro-keyed re-verified 12px/1.5 (real output in JOURNAL)
- [x] screenshots both apps eye-checked
- [x] records updated (N-036, components v0.15, ROADMAP, CLAUDE PLAY, JOURNAL J-415, task COMPLETED)
