# M-RP6.1c — `Accelerator` (ui/common) + lean keymap registry (shell) build runbook
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-10  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

For Clair. Third frame prerequisite of the M-RP6.1 client-UI-frame arc (Phase-0 J-488 / D-107 / `docs/xgen-client-frame-phase0.md` §4.4, §6). Per-component design **locked by Joe "all by recomms"** (Chat design walk, this session). This is **NOT a visual component** — no envelope, no sampler cell, no CDP, no registry delta. It is ONE pure `ui/common` value-object (single definition → `toDisplay()` + `matches()`, so display and dispatch never drift) plus a lean shell-level keymap registry that consumes it. Verify = **vitest** (the `stream/grouping.ts` precedent), not the CDP loop. Registry stays **299** (unchanged, no envelope).

---

## 1. Goal

- **`Accelerator`** — a canonical, DOM-free value-object in `ui/common`. Authored from a string (`"Mod+Q"`), stored as a normalized `{key, mods}`, projecting two ways from the single definition: `toDisplay(platform)` for the menu-item hint, `matches(event, platform)` for keydown dispatch. Pure, unit-testable, imports nothing from the DOM or Tauri (sibling to `Converter<T>`'s one-object-two-reps shape).
- **keymap registry** — a lean shell module: a binding table `{accelerator → command}` + one global `keydown` listener that walks bindings via `matches`. Object built **fully** now; the table starts with **one** binding: `Mod+Q → exit`.

## 2. Locked design (Joe "all by recomms")

### 2.1 `Accelerator` value-object (`ui/common`, pure)

- **Authoring (A3).** `Accelerator.parse(spec: string)` — author as a string, store canonical. No public struct constructor needed; parse is the factory.
- **Canonical internal form.** `key: string` (normalized **lowercase**, e.g. `'q'`) + `mods: {ctrl, shift, alt, meta}` (all boolean) + `usesMod: boolean` (true when authored with the logical `Mod` token — NOT eagerly resolved, so the object stays platform-free).
- **Modifier tokens (case-insensitive), `+`-separated, last token = the key:**
  - logical primary → `usesMod=true`: `Mod` · `CmdOrCtrl` · `CommandOrControl`
  - `Ctrl` · `Control` → `ctrl`
  - `Shift` → `shift`
  - `Alt` · `Option` → `alt`
  - `Cmd` · `Command` · `Meta` · `Super` · `Win` → `meta`
  - unknown modifier token, empty spec, or missing key → **throw** (parse tests cover this).
- **Platform is a parameter, never stored.** `toDisplay(platform)` and `matches(event, platform)` take `platform: 'win' | 'mac' | 'linux'`. Keeps the object pure + lets tests pass platform explicitly. The **shell** owns platform detection (§2.2), `ui/common` never reads `navigator`.
- **Effective-mods resolution (the `Mod` fold), computed per call:** start from `mods`; if `usesMod` then set `meta=true` on `mac`, else `ctrl=true` (win/linux). Literal `Ctrl`/`Cmd` are **not** folded — they mean exactly that modifier on every platform.
- **`matches(event, platform)` — exact-modifier match.** `event.key.toLowerCase() === this.key` AND all four of `event.ctrlKey/shiftKey/altKey/metaKey` **equal** the effective-mods booleans exactly. So `Mod+Q` must **not** fire under `Ctrl+Shift+Q`. Key compared via `event.key` (lowercased); `event.code` noted as a future layout-independent option, not built.
  - **`event` is duck-typed `KeyLike`** = `{ key: string; ctrlKey: boolean; shiftKey: boolean; altKey: boolean; metaKey: boolean }`. A real `KeyboardEvent` satisfies it; a plain object literal satisfies it in tests → `matches()` needs **no DOM** to verify.
- **`toDisplay(platform)` — platform-conventional.** `win`/`linux` → modifier **words** + key, joined by `+`, order `Ctrl+Alt+Shift+Meta` (omit absent), key upper-cased (`"Ctrl+Q"`). `mac` → symbol glyphs concatenated, order `⌃⌥⇧⌘` + key (`"⌘Q"`). Uses effective-mods (so `Mod+Q` renders `Ctrl+Q` on win, `⌘Q` on mac).
- **Getter?** None — no envelope, not a registry component. This is a plain exported class/object.

### 2.2 keymap registry (shell — `ui/client`)

- **Home:** a shell module (e.g. `ui/client/…/keymap.ts`), imports `Accelerator` (+ the `Platform` type) from `$common`. Shell-level because it holds a real global `keydown` listener (DOM) and calls the Tauri exit seam — both forbidden in `core`/`common`.
- **Command (F1 now, `id?` reserved).** A binding is `{ accel: Accelerator; command: () => void; id?: string }`. Direct handler now; the optional `id` is reserved so M-RP6.1d's File→Exit `menu-item` can later reference the **same** command (single-source-of-truth), without building a command-table this milestone.
- **API:**
  - `register(accel, command, id?)` — append a binding.
  - `lookup(event): Command | null` — first binding whose `accel.matches(event, platform)` wins.
  - `dispatch(event): boolean` — `lookup`; if hit → `event.preventDefault()` + run command + return `true`; else `false`.
  - `attach()` / `detach()` — add/remove the single `window` `keydown` listener (→ `dispatch`). Called on client mount/unmount.
- **Platform detection lives here** — one `PLATFORM` const detected once (Tauri OS / `navigator`), passed into the registry ctor and later into `menu-item` hint render. Not in `ui/common`.
- **The one binding:** `register(Accelerator.parse('Mod+Q'), exitCommand, 'exit')`.
- **`exitCommand` = a thin shell fn** calling the window close. **Reuse the exact Tauri close the existing client Quit/Shut-Down button already wires** (confirm against the real shell code — Rule 5; do not invent a new close call). M-RP6.1d's File→Exit will call this same `exitCommand`.

## 3. Files to touch

1. `ui/common/…/accelerator.ts` — new pure value-object (§2.1). Mirror an existing `ui/common` pure module (e.g. the processor `transform.ts` pure core) for placement + export convention. **No** DOM/Tauri/Svelte imports.
2. `ui/common/…/accelerator.test.ts` (or the repo's vitest naming/location) — the unit suite (§5).
3. `ui/client/…/keymap.ts` — new shell registry (§2.2), imports `Accelerator` from `$common`.
4. `ui/client/…/<shell entry>.svelte` (the real client root that owns lifecycle) — construct `Keymap(PLATFORM)`, register `Mod+Q → exitCommand`, `attach()` on mount / `detach()` on destroy; define/locate `exitCommand` reusing the existing Quit close seam.

## 4. No sampler cell

`Accelerator` has no envelope and no visual — it gets **no** `app_sampler.svelte` cell and **no** CDP registry entry. Do not add one. Registry stays 299. (The frame *assembly* — the menu-item hint reading the same `Accelerator`, Ctrl+Q actually quitting — is verified in the real client at 6.1d / assembly, not here.)

## 5. Verify plan (vitest — Rule 2, quote the real N/N pass line)

**Clair runs `vitest` in the feat commit and quotes the actual pass output.** No CDP, no port polling, no detached launch. Cover at minimum:

- **parse** — `"Ctrl+Q"` → `{key:'q', mods.ctrl=true, usesMod=false}`; `"Mod+Q"` → `usesMod=true`; `"Ctrl+Shift+K"` → both mods + `key:'k'`; **throws** on `"Ctrl+"` (no key), `"Foo+Q"` (unknown modifier), `""` (empty).
- **toDisplay** — `parse("Mod+Q").toDisplay('win')` === `"Ctrl+Q"`; `.toDisplay('mac')` === `"⌘Q"`; `parse("Ctrl+Shift+K").toDisplay('win')` === `"Ctrl+Shift+K"`, `.toDisplay('mac')` === `"⌃⇧K"`.
- **matches (exact-modifier)** — `parse("Mod+Q").matches({key:'q',ctrlKey:true,shiftKey:false,altKey:false,metaKey:false}, 'win')` === `true`; the **same** event with `shiftKey:true` === `false`; on `'mac'` the `Mod` binding matches `metaKey:true`/`ctrlKey:false`, not the reverse; key is case-insensitive (`key:'Q'` still matches).
- **Keymap.dispatch (spy, no DOM)** — register a **spy** `command` + `Accelerator.parse('Mod+Q')`; feed a matching synthetic `KeyLike` → `dispatch` returns `true`, spy called **once**; feed a non-matching event → `false`, spy **not** called. (Avoids actually closing the app.)

**Real-client leg (manual, not automated):** with the registry wired, Ctrl+Q quits the real client window — eye-confirmed once by Joe. The automated proof is the spy-dispatch above; auto-verifying a real Ctrl+Q would close the app under test.

## 6. Close (D-074 two-commit)

Clair feat first (code-only: §3 files) — **including the passing vitest run quoted in the commit/handoff**. Then Chat doc-bridge:
- `ui/docs/xgen-ui-notes.md` **N-085** (`Accelerator` = first `ui/common` value-object of the frame arc / one-definition-two-projections / platform-as-parameter / `KeyLike` duck-type for DOM-free `matches` / vitest-not-CDP verify / F1-direct-handler with `id?` reserved for 6.1d).
- `docs/xgen-ui-components.md` — note only (Accelerator is a `ui/common` value-object, **no registry entry**; count unchanged 299). No new component row.
- `docs/ROADMAP.md` (M-RP6.1c ✅ DONE, vX bump, next-active **M-RP6.1d `menu-bar` minimal**).
- `CLAUDE.md` PLAY (head → new J-491; registry unchanged 299; next-active M-RP6.1d).
- `JOURNAL.md` +J-491 (quote the real vitest N/N).
- this task → COMPLETED.

**No new D** — §4.4 concept already locked under D-107. `temperature-indicator` (M-RP6.5) stays ⏸️ POSTPONED. Not pushed — Joe pushes.

## 7. Definition of Done

- [ ] `accelerator.ts` authored in `ui/common` — `parse`/`toDisplay`/`matches`, canonical `{key,mods,usesMod}`, platform-as-parameter, `KeyLike` duck-type, no DOM/Tauri import.
- [ ] `accelerator.test.ts` — parse/toDisplay/matches + Keymap.dispatch-spy legs (§5).
- [ ] `keymap.ts` authored in `ui/client` — `register`/`lookup`/`dispatch`/`attach`/`detach`, `PLATFORM` detected shell-side, binding `Mod+Q → exitCommand`.
- [ ] shell entry wires `Keymap` (attach on mount / detach on destroy); `exitCommand` reuses the existing Quit close seam (confirmed against real code, Rule 5).
- [ ] `vitest` green — real N/N pass line quoted.
- [ ] `vite build` clean — module count quoted.
- [ ] No sampler cell, no CDP, registry unchanged 299.
- [ ] Records bridged (§6), task flipped COMPLETED.

---

*End of M-RP6.1c runbook.*
