# M-RP-SETTINGS — Leg A — the Settings shell + Plugins section
> **Status**: COMPLETED  
> Version: 1.0  
> Date: July 2026  
> **Last updated**: 2026-07-16  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. Read first

Rule-0 stack: CLAUDE.md PLAY block → JOURNAL J-534 (this milestone's design lock) → this runbook → Phase-0 `docs/xgen-settings-phase0.md` (§0, D-C the single-modal shape, §6 legs). Item 3, not item 1.

**Lane:** Clair implements; Chat re-drives every verify leg on the real client `9222` (Rule 5). **Appearance is Joe's** — the Discord-shaped layout's exact widths, density, compactness, colours ship **PROVISIONAL** → `M-RP-SKIN`. This runbook fixes only the **mechanics** (the shell exists, the sidebar swaps, the entry points route, the manager is hosted honestly).

**Client-only.** No sampler (`ui/sampler` catalogue stays 328, by scope). **No Rust** — the File menu item + `settings.open` command are frontend; `cargo test` must stay **1517/0/62 IDENTICAL** (the honest proof no Rust landed).

---

## 1. What this leg is

Stand up the **one** Settings modal (Discord-shaped: a left category menu ~¼ width + a content pane that swaps per selection, compact), mount today's **read-only** `plugin-list` as its **Plugins** section, wire the two entry points, and **absorb `plugins-dialog`**. No new plugin verbs — the action row is **Leg B**, the settings mechanism is **Leg C**. This leg is visible-first: the whole Settings frame + the manager on screen for Joe's eyes.

---

## 2. Grounding (verified 2026-07-16 — re-confirm before editing; N-116)

- **Today's plugin path:** `gear` shelf face → command `widget.manager` → `pluginsOpen = true` → `<PluginsDialog bind:open={pluginsOpen} />` (in `app_client.svelte`); `plugins-dialog.svelte` wraps the core `dialog` (C1) hosting the read-only `plugin-list` widget (`$common`).
- **File menu** (`app_client.svelte`): `File` = `{ Restart (app.restart) · separator · Exit (Ctrl+Q, app.exit) }`. `Help` = `{ About (help.about) }`. `commandTable` maps command ids → handlers; the menu-bar and the keymap both resolve through it.
- **Core `dialog`** (`ui/core/.../dialog.svelte`) is the modal container; About / UI-state / Plugins dialogs all wrap it (shell-local, `ui/client/src/`).
- **`plugin-list.svelte`** takes a plain `id`, is mounted directly by its host (not `region-node`), reads `installed.active`, renders read-only rows (name + `[system]/[user]` badge + meta line). W-3: it imports only `$common`.

---

## 3. Scope — the shell, the sidebar, the sections, the entry points

### 3.1 The Settings shell — `settings-dialog.svelte` (new, shell-local `ui/client/src/`)
- Wrap the core `dialog`; inside it a **two-pane** layout: **category menu (~¼ width)** on the left + a **content pane** on the right (Discord-shaped, compact — appearance PROVISIONAL).
- Props: `open = $bindable(false)` + `section?: string` (which category to land on; default the first). The About/UI-state-dialog "always-mounted, closed = `display:none`" posture.

### 3.2 The sidebar — a selectable category list
- The category menu is a **roving-tabindex** selectable list; selecting a category **swaps the content pane**. Reuse an existing rover pattern if one fits cleanly, or note the reuse-vs-`M-RP-ROVING`-extraction call (do **not** extract here — that is its own filed milestone).
- **≥ 2 real sections** so the swap is exercised, not asserted (N-091). **Plugins** is one. The second must be **real content**, not an empty placeholder — proposal: an **About / General** section that renders the already-shipped about info (the `get_about_info` content component, reused — S-2 one-component, not a second home; Help ▸ About is left untouched). The exact second section is a Leg-A mechanic call under the "real content, no empty branch" constraint; if only Plugins is genuinely ready, say so and defer the swap-proof to the leg that adds the second real section — do not ship a one-item sidebar that pretends to be a nav.

### 3.3 The Plugins section — host the existing `plugin-list` (read-only)
- Mount the read-only `plugin-list` widget as the content of the **Plugins** category (the mount **moves** from `plugins-dialog` into `settings-dialog`). No row changes — the action row is Leg B.
- **Remove `plugins-dialog.svelte`** (absorbed). *(Device can't delete; if working on the user's disk, `Move-Item` it into a `_to_delete/` folder and tell Joe — do not leave a dead file imported nowhere.)*

### 3.4 Entry points (two, one modal)
- New command **`settings.open`** in `commandTable` → opens Settings on the **default** section.
- New **File ▸ Settings** menu item placed **above Restart** (File becomes `Settings · Restart · —— · Exit`), command `settings.open`.
- **Re-point the gear:** `widget.manager` now opens Settings **landed on the Plugins section** (`settings.open` with `section: 'plugins'`, or a sibling handler that sets the section). Retire `pluginsOpen` / `<PluginsDialog>`.
- The gear label may stay "Plugins" (it lands on Plugins) — a UI-surface call, Joe's.

### 3.5 Appearance
- Widths, density, compactness, the sidebar/content chrome ship **PROVISIONAL** → `M-RP-SKIN`. The mechanic (shell exists, sidebar swaps, entry points route) is the DoD; the look is not.

---

## 4. Definition of Done (each verified with real output; Rule 7)

1. `settings-dialog.svelte` exists: core-`dialog` + two-pane (sidebar ~¼ + content); `bind:open` + `section` prop; `:modal` true on open (read `el.matches(':modal')`, never the `open` attribute — J-496).
2. **File ▸ Settings** opens Settings on the default section; **gear** opens the **same** modal on the **Plugins** section; both driven live. File menu order is `Settings · Restart · —— · Exit`.
3. Sidebar **swaps the content pane** between ≥2 real sections — **driven, not asserted** (N-091); the Plugins section renders the read-only `plugin-list`.
4. `plugins-dialog.svelte` removed / imported nowhere; `pluginsOpen` + `<PluginsDialog>` gone.
5. **`cargo test` 1517/0/62 IDENTICAL** (proves no Rust), summed programmatically. `npm test` + `vite build` quoted. Sampler catalogue **328 unchanged, by scope** (`git show --stat` — no `ui/sampler`, no `.rs`).
6. Registry: read a baseline **quiescent after a full reload** (N-132), stating store + selection + disabled + saved-state counts (N-105/N-108/N-112/N-115). Enumerate the delta from the current baseline (the `plugin-list` rows re-host; the sidebar/section nav may add registrations — measure, do not derive).
7. Verified live `9222` only, Rule 5 re-drive by Chat.
8. Appearance PROVISIONAL → `M-RP-SKIN`; expected, not a defect.

---

## 5. Out of scope (do not build here)

- The action row `[info][settings][disable][uninstall]` + kind-glyph + `session.disabled` → **Leg B** (`M_RP_SETTINGS_B_ACTION_ROW.md`).
- A plugin's `settingsComponent` opening in the content pane + the backdrop setting → **Leg C**.
- Absorbing Help ▸ About into Settings — **not** this leg (the About *content component* may be reused for a Settings section, but the Help menu item stays).
- The `M-RP-ROVING` extraction — reuse an existing rover; do not extract.
- Any appearance decision — Joe's, via `M-RP-SKIN`.

---

## 6. Notes / tooling

- CDP harness `cdp-debug.ps1 -App client` (9222); coords CSS px (DPR does not apply); `get(id)` → `{type,state}` (read `.state.foo`); read the DOM in a **separate** eval after a mutation (Svelte effect-tick); single-expression `JSON.stringify(...)` evals; registry keys on `data-debug-id` not `id` (N-110); baseline **after a full reload** (N-132).
- Joe launches `tauri dev`; Chat drives the running app over CDP. Joe pushes — never Chat.
