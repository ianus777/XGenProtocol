# M-RP2.9 — display-di `label` (root <label>, caption)
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

Author + skin (one pass) the first **display-kind di** component: `label` — root `<label>`, a short caption naming another control. First of the display-di trio (`label` / `paragraph` / `image`, identities locked N-032): value-carrying but **read-only** — the display half of the di model, vs the four interactive di built so far (toggle / button / textfield / select). Founds the read-only display-di pattern that `paragraph` / `image` follow.

## Locks (design walk, Joe-locked 2026-06-25)

1. **Value-prop = `text`** (not `value`). `value` is the editable/`$bindable` marker across the codebase; display-di take a semantic value-name (label & paragraph = `text`, image = `src`). Read-only ⇒ plain prop, no `$bindable`.
2. **No `for` on the atomic.** Association (`for=`/nesting) is a composite concern (N-032) — wired by the group (`textfield-group`, implicit nesting). Keep `id` (debug + future nest target). Standalone label = valid-but-inert, tolerated.
3. **Register `{ text }` debug getter.** Registry stays uniform (N-030 §4: registry is one projection of the value). Founds the display-di verify pattern: no event to dispatch — verify = snapshot returns the passed text + computed-style probe.
4. **Skin from existing vocabulary, no new token.** `.label` = `color: var(--t2)` + `font-size: 12px` (the established control size). The `--fs-*` type scale is **deferred to `paragraph`** (M-RP2.10), where two text components in hand justify founding it and retro-keying the shipped skins in one deliberate pass. Block-level stays a skin concern (default inline, N-032).
5. **`use:envelope` unchanged.** Content-agnostic substrate reused verbatim. Only deltas from interactive di: plain prop (no `$bindable`), no handler, verify = render + computed-style (no dispatch).

## Phases

**Phase 1 — author** `ui/core/lib/components/data-independent/label.svelte`: `text` prop + `id`; `use:envelope={{ name:'label', id, debug }}`; `debug = () => $state.snapshot({ text })`; body `{text}`; zero `<style>`.

**Phase 2 — skin** `.label` appended to `ui/assets/skin.css`: `color: var(--t2)`, `font-size: 12px`, `line-height: 1.5`. Inline default.

**Phase 3 — wire demo, both shells** (`ui/client/src/app_client.svelte` + `ui/node/src/app_node.svelte`): import `Label`; mount `<Label text="Demo label" id="demo" />` in `<main>` beside the existing demos. No `$state` var (read-only).

**Phase 4 — CDP verify both apps** (Chat self-drives, N-028 working mode): launch detached `-Debug`, poll 9222/9322, retry `snapshot()` until non-null. Confirm `label#demo` → `{text:"Demo label"}`; computed-style probe `.label` → `color: rgb(200,196,188)` (`--t2`), `font-size: 12px`. `-Mode screenshot` both apps, eye-check. Clean teardown (9222/9322/5173/5174 free, 0 orphans).

**Phase 5 — records (D-074 atomic):** `ui/docs/xgen-ui-notes.md` N-035; `ui/docs/xgen-ui-components.md` Built row + detail (v0.13→0.14); `docs/ROADMAP.md` RP node M-RP2.9 ✅ + frontier; `CLAUDE.md` PLAY → M-RP2.9 CLOSED / Next `paragraph` / pointer J-413→J-414; this task → COMPLETED; `JOURNAL.md` J-414. All `.md` `Last updated` bumped.

## Definition of Done

- [x] `label.svelte` authored (text prop, envelope, `{text}` getter, zero `<style>`)
- [x] `.label` skinned in `skin.css`
- [x] demo wired in both shells
- [x] CDP both apps: snapshot `{text:"Demo label"}` + computed-style `--t2` / 12px (real output quoted in JOURNAL)
- [x] screenshots both apps eye-checked
- [x] records updated (N-035, components v0.14, ROADMAP, CLAUDE PLAY, JOURNAL J-414, task COMPLETED)
