# M-RP6.1c — `Accelerator` + `KeymapRegistry` (ui/common) build runbook
> **Status**: ACTIVE  
> Version: 1.1  
> Date: Jul 2026  
> **Last updated**: 2026-07-10  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

For Clair. Third frame prerequisite of the M-RP6.1 client-UI-frame arc (Phase-0 J-488 / D-107 / `docs/xgen-client-frame-phase0.md` §4.4, §6). Per-component design **locked by Joe** — Clair's grounded design walk + two Chat tightenings, "as you recommend" (this session). This is **NOT a visual component** — no envelope, no sampler cell, no CDP, no registry delta. It is TWO pure DOM-free objects in `$common` (`ui/common/lib`): the `Accelerator` value-object (single definition → `toDisplay()` + `matches()`, so display and dispatch never drift) and a pure `KeymapRegistry` table (`register` + `resolve(event)→commandId|null`). Verify = **vitest** (a standing harness stood up here). Component registry stays **299** (unchanged, no envelope).

> **v1.1 supersedes v1.0.** v1.0 was written before Clair's grounding pass. Four locked changes: (1) **scope narrowed** — 6.1c ships library + tests only; the live `keydown` listener, the `Ctrl+Q → app.exit` binding, and the `exitCommand` (Tauri close) **defer to 6.1d** (no Exit command exists until the menu-bar; Phase-0 §7 schedules only a pure-unit leg here). (2) **`Ctrl`-as-shortcut** replaces the `Mod`/`usesMod` token. (3) The **registry pure-table moves into `$common`** (was shell) — commandId-based; only the singleton instance + population + listener stay shell (6.1d). (4) verify = a **standing vitest** harness, not an ad-hoc node run.

---

## 1. Goal

- **`Accelerator`** — a canonical, DOM-free value-object in `$common`. Authored from a string (`accelerator("Ctrl+Q")`), stored as a normalized `{key, mods}`, projecting two ways from the single definition: `toDisplay(platform)` for the (future) menu-item hint, `matches(event, platform)` for dispatch. Pure, unit-testable; imports nothing from the DOM / Tauri / Svelte (sibling to the processor `transform.ts` pure core).
- **`KeymapRegistry`** — a **pure** binding table in `$common`: `register(accel, commandId)` + `resolve(event) → commandId | null` (walks bindings via `accel.matches`, first-registered wins, dedups on the canonical string). **No DOM, no listener, no command execution** — those are shell concerns deferred to 6.1d.

## 2. Locked design

### 2.1 `Accelerator` value-object (`$common`, pure)

- **Authoring (D1).** `accelerator(spec: string)` factory — author as a human string, store canonical. Bindings are **Tier-1 trusted code** (not user input) → parse **throws** on malformed (fail-fast at author time), not the lenient Tier-2 processor posture. A struct form is also exposed for programmatic construction.
- **Canonical internal form.** `key: string` (normalized **lowercase** for letters, e.g. `'q'`; verbatim for named keys, e.g. `'F1'`, `'ArrowLeft'`, `'Delete'`) + `mods: { shortcut, shift, alt, meta }` (all boolean). Platform-free storage.
- **Modifier model (D2 — SHORTCUT abstraction, JavaFX-style).** The primary accelerator key is abstract:
  - `Ctrl` **=== `Control`** → `shortcut` (the platform accelerator key: `ctrlKey` on win/linux, `metaKey`/⌘ on mac). **Not** a literal-ctrl token — one shortcut token, no overload. (If a real literal-ctrl-on-mac is ever needed, add a distinct token then; out of scope, Windows-only target.)
  - `Shift` → `shift` (literal) · `Alt` · `Option` → `alt` (literal)
  - `Cmd` · `Command` · `Meta` · `Super` → `meta` (literal, the rare explicit case)
  - `+`-separated, case-insensitive tokens, **last token = the key**. Unknown modifier token / empty spec / missing key → **throw**.
- **Platform is a parameter, never stored (D2).** `toDisplay(platform)` and `matches(event, platform)` take `platform: 'win' | 'mac' | 'linux'`, **default `'win'`** (the only real target today; keeps the module DOM-free — no `navigator` read at import). Tests pass platform explicitly.
- **Effective-mods (per call):** `ctrl = shortcut && platform !== 'mac'` · `meta = metaLiteral || (shortcut && platform === 'mac')` · `shift`/`alt` literal. (So `Ctrl+Q` → `ctrlKey` on win, `metaKey`/⌘ on mac.)
- **`matches(event, platform)` — exact-modifier match.** `event.key` matches `this.key` (case-insensitive for single letters, verbatim for named keys) AND all four of `event.ctrlKey/shiftKey/altKey/metaKey` **equal** the effective-mods booleans exactly. So `Ctrl+Q` must **not** fire under `Ctrl+Shift+Q`. (`event.code` noted as a future layout-independent option, not built.)
  - **`event` is duck-typed `KeyLike`** = `{ key: string; ctrlKey: boolean; shiftKey: boolean; altKey: boolean; metaKey: boolean }`. A real `KeyboardEvent` satisfies it; a plain object literal satisfies it in tests → `matches()` needs **no DOM**.
- **`toDisplay(platform)` — platform-conventional.** win/linux → modifier **words** + key, joined `+`, order `Ctrl+Alt+Shift+Meta` (omit absent), key upper-cased for letters (`"Ctrl+Q"`). mac → symbol glyphs concatenated, order `⌃⌥⇧⌘` + key (`shortcut`→⌘, `shift`→⇧, `alt`→⌥) → `"⌘Q"`, `"⇧⌘K"`. Uses effective-mods.
- **`canonical(): string`** — a stable platform-free normalized string (e.g. `"shortcut+shift+k"`), used as the `KeymapRegistry` dedup key. No getter/envelope — plain exported object.

### 2.2 `KeymapRegistry` (`$common`, pure — the D3 split)

- **Home:** `$common` (`ui/common/lib/keymap/registry.ts`), imports `Accelerator`. **Pure** — the reusable table + `resolve` (the node shell will want keymaps too, and it's unit-testable). This is the D3 refinement of Phase-0 §4.4 (which put the whole registry shell-side); only the singleton **instance**, binding **population**, and the one **`keydown` listener** stay shell-side → **6.1d**.
- **Construction:** `new KeymapRegistry(platform)` — platform passed in (shell detects it in 6.1d; tests pass `'win'`/`'mac'`). Platform-free otherwise.
- **Command indirection (commandId, not a handler).** A binding is `{ accel: Accelerator; commandId: string }`. The registry stores + resolves **ids** (strings) — it does not hold or run `() => void`. Execution (id → fn) is the shell's job (6.1d). This keeps the table pure/testable **and** gives 6.1d's File→Exit `menu-item` the same `commandId` to reference (single source of truth, no display/dispatch drift).
- **API:**
  - `register(accel, commandId)` — append a binding; dedup on `accel.canonical()` (first-registered wins, later dup ignored or throws — Clair's call, document which).
  - `resolve(event): string | null` — first binding whose `accel.matches(event, this.platform)` wins → its `commandId`; else `null`.
- **No** `attach`/`detach`/`dispatch`/DOM here — deferred to 6.1d.

## 3. Files to touch

1. `ui/common/lib/keymap/accelerator.ts` — the value-object (§2.1). No DOM/Tauri/Svelte imports. (Mirror the processor `transform.ts` pure-core export convention.)
2. `ui/common/lib/keymap/registry.ts` — the pure `KeymapRegistry` (§2.2). Imports only `accelerator.ts`.
3. `ui/common/lib/keymap/accelerator.test.ts` + `registry.test.ts` (or one `keymap.test.ts`) — the vitest suite (§5).
4. **vitest harness** — add `vitest` as a devDep on the **sampler** package (it already aliases `$common` and is the component test-bed) + a `vitest.config.js` (resolve the `$common` alias for the runner) + a `test` script (`npm test` → `vitest run`). First standing UI unit harness; 6.1f's descriptor→layout walk (also pure-unit per §7) and the existing `grouping.ts`/`transform.ts`/`clamp.ts` can retro-adopt it later.

**Explicitly NOT this milestone (→ 6.1d):** any `ui/client` file, the `keydown` listener, `register(accelerator("Ctrl+Q"), "app.exit")`, the id→fn command dispatch, `exitCommand` (Tauri close), `PLATFORM` detection wiring, the `menu-item` hint render.

## 4. No sampler cell

`Accelerator`/`KeymapRegistry` have no envelope and no visual → **no** `app_sampler.svelte` cell, **no** CDP registry entry, **no** registry delta. Component registry stays **299**. (The frame *assembly* — the menu-item hint reading the same `Accelerator`, Ctrl+Q actually quitting — is verified in the real client at 6.1d, not here.)

## 5. Verify plan (standing vitest — Rule 2, quote the real N/N pass line)

**Clair authors + runs the suite green as part of the feat** (quote the real `vitest` N/N in the handoff). **Chat independently re-runs `npm test`** at verify and records the real output in the doc-bridge — that is the verify leg in place of the CDP loop. Cover at minimum:

- **parse** — `accelerator("Ctrl+Q")` → `{key:'q', mods.shortcut=true}`; `"Ctrl+Shift+K"` → `shortcut+shift`, `key:'k'`; `"Cmd+S"` → literal `meta`; named key `"F1"` / `"Ctrl+Delete"` preserved verbatim; **throws** on `"Ctrl+"` (no key), `"Foo+Q"` (unknown modifier), `""` (empty).
- **toDisplay** — `accelerator("Ctrl+Q").toDisplay('win')` === `"Ctrl+Q"`, `.toDisplay('mac')` === `"⌘Q"`; `accelerator("Ctrl+Shift+K").toDisplay('win')` === `"Ctrl+Shift+K"`, `.toDisplay('mac')` === `"⇧⌘K"` (shortcut→⌘, in ⌃⌥⇧⌘ order).
- **matches (exact-modifier)** — `accelerator("Ctrl+Q").matches({key:'q',ctrlKey:true,shiftKey:false,altKey:false,metaKey:false}, 'win')` === `true`; the **same** event with `shiftKey:true` === `false` (exact); on `'mac'` the same binding matches `{metaKey:true, ctrlKey:false}` and **not** `{ctrlKey:true}`; key case-insensitive (`key:'Q'` still matches).
- **KeymapRegistry.resolve (pure, no DOM, no spy needed)** — `new KeymapRegistry('win')`, `register(accelerator("Ctrl+Q"), "app.exit")`; `resolve({key:'q',ctrlKey:true,…})` === `"app.exit"`; a non-matching event → `null`; dedup on canonical (registering the same accel twice behaves per the documented rule).

**Real-client leg — DEFERRED to 6.1d** (Ctrl+Q actually quitting is eye-confirmed once the shell listener + Exit command exist). Nothing real-client this milestone.

## 6. Close (D-074 two-commit)

Clair feat first (code-only: §3 files) — **including the passing vitest run quoted**. Then Chat doc-bridge:
- **Chat re-runs `npm test`**, quotes real N/N.
- `ui/docs/xgen-ui-notes.md` **N-085** (`Accelerator`+`KeymapRegistry` = first `$common` value-objects of the frame arc / one-definition-two-projections / `Ctrl`-as-shortcut platform model / platform-as-parameter `'win'` default / `KeyLike` duck-type for DOM-free `matches` / commandId indirection / first standing vitest harness).
- `docs/xgen-client-frame-phase0.md` — **in-place §4.4 refinement** (the D3 split: the `KeymapRegistry` **pure table + `resolve`** lives in `$common`; only the instance + population + `keydown` listener are shell). The J-487/J-490 doc-wording-fix precedent — arc-local, **no new D** (D-069). Version bump + `> **Last updated**:` date only.
- `docs/xgen-ui-components.md` — note only (`Accelerator`/`KeymapRegistry` are `$common` value-objects, **no registry entry**; count unchanged 299). No new component row.
- `docs/ROADMAP.md` (M-RP6.1c ✅ DONE, vX bump, next-active **M-RP6.1d `menu-bar` minimal** — File→Exit registers `accelerator("Ctrl+Q") → "app.exit"` + the shell listener + `exitCommand`).
- `CLAUDE.md` PLAY (head → new J-491; registry unchanged 299; next-active M-RP6.1d).
- `JOURNAL.md` +J-491 (quote the real vitest N/N).
- this task → COMPLETED.

**No new D.** `temperature-indicator` (M-RP6.5) stays ⏸️ POSTPONED. Not pushed — Joe pushes.

## 7. Definition of Done

- [ ] `accelerator.ts` in `$common` — `accelerator()` parse (throw on malformed), `{key, mods:{shortcut,shift,alt,meta}}`, `Ctrl`-as-shortcut, `toDisplay`/`matches`/`canonical`, platform-as-parameter (`'win'` default), `KeyLike` duck-type, no DOM/Tauri/Svelte import.
- [ ] `registry.ts` in `$common` — pure `KeymapRegistry(platform)`, `register`/`resolve`, commandId-based, canonical dedup, no DOM/listener.
- [ ] vitest harness stood up (sampler devDep + `vitest.config.js` + `test` script), `$common` alias resolves for the runner.
- [ ] test suite — parse / toDisplay / matches / resolve legs (§5).
- [ ] `vitest` green — real N/N pass line quoted (Clair); Chat re-run quoted at verify.
- [ ] `vite build` clean — module count quoted.
- [ ] No `ui/client` file touched; no listener/Exit/PLATFORM wiring (those are 6.1d).
- [ ] No sampler cell, no CDP, component registry unchanged 299.
- [ ] Records bridged (§6, incl. the in-place frame-phase0 §4.4 refinement), task flipped COMPLETED.

---

*End of M-RP6.1c runbook.*
