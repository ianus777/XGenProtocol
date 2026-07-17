# M-RP-SETTINGS — Leg B — the plugin action row
> **Status**: ACTIVE  
> Version: 1.0  
> Date: July 2026  
> **Last updated**: 2026-07-16  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. Read first

Rule-0 reading stack: CLAUDE.md PLAY block → JOURNAL J-534 (this milestone's design lock) → this runbook → the Phase-0 doc `docs/xgen-settings-phase0.md` (§2 the three locks, §3 the row model + feeder discipline). This runbook is item 3, not item 1.

**Lane:** Clair implements; Chat re-drives every verification leg on the real client `9222` (Rule 5). **Appearance is Joe's** — everything visual in this leg (glyph shapes, icon set, row density, colours) ships **PROVISIONAL** and is tuned in `M-RP-SKIN`; this runbook fixes only the **mechanics** (which control exists, what feeds it, when it is honest).

**Client-only.** No sampler (`ui/sampler` catalogue must stay 328, by scope). No Rust planned — `cargo test` must stay **1517/0/62 IDENTICAL** (the honest proof no Rust landed).

---

## 1. What this leg is

**Prerequisite: Leg A (`M_RP_SETTINGS_A_SHELL.md`) ships first** — the Discord-shaped Settings modal (category menu + content pane) with the read-only `plugin-list` as its **Plugins section**. This leg gives that section's rows a live one-line action row. This IS the long-filed **M-RP6.1m** action row, landing per-line inside the Plugins section of Settings.

Row model (Joe's compact vision):

```
[kind-glyph]  Official Name   · meta ·        [info] [settings] [disable] [uninstall]
```

Entry points are already wired by Leg A (gear → Settings @ Plugins; File ▸ Settings → default). `plugins-dialog.svelte` is already absorbed. This leg touches only the Plugins-section rows + the disable store + the descriptor.

---

## 2. Grounding (verified 2026-07-16 — re-confirm before you edit; N-116)

- `PluginDescriptor` (`ui/common/lib/plugins/registry.ts`) carries `id/name/description/kind/host/delivery/surface/regionId?/component?` — **no `settingsComponent`/`hasSettings` yet.**
- `installed.svelte.ts` (`$common`) owns the runtime installed-set: `install`/`uninstall`/`hydrate`/`isInstalled`/`ids`/`active` (`active = [...CLIENT_PLUGINS, ...installed customs]`). Svelte-5 caveat: a `$state` `Set` is not reactive on `.add`/`.delete` — reassign a fresh `Set`.
- `app_client.svelte`: `widgetRegistry`/`bgWidgets`/`titles` are `$derived` off `installed.active`; the `gear` shelf face → command `widget.manager` → `pluginsOpen = true` → `<PluginsDialog bind:open={pluginsOpen} />`; a `__XGEN_PLUGINS__` DEV bridge already wraps install/uninstall (set + layout leaf + persist).
- `plugin-list.svelte` (`$common`, W-3) renders `installed.active` rows read-only (name + `[system]/[user]` badge + meta line), composing `Label`s under `id__…` ids. It takes a plain `id`, mounted directly by the host dialog — not by `region-node`.
- `session.installed: string[]` persists via the uistate store's N-107 per-key merge (`setSessionInstalled`), zero Rust (opaque-blob path, D-114). `session` top-level keys today: `layout`/`locked`/`installed` (+ Rust's `geometry`).

---

## 3. Scope — five mechanics

### 3.1 Descriptor: `settingsComponent?`
Add `settingsComponent?: Component` to `PluginDescriptor`. **Undefined on every row this leg** (grid-plate gets one in Leg C). The `settings` button's enabled state derives from `!!p.settingsComponent` (`hasSettings`) — so it is greyed for all today, for the real per-plugin reason *"this plugin has no settings"*, not because a verb is missing. It enables itself the moment a plugin ships a component (Leg C). Do **not** hardcode it greyed.

### 3.2 Disable: a new `session.disabled` set
- Extend `installed.svelte.ts` with a `disabled` `$state<Set<string>>` (fresh-Set reassignment) + `disable(id)`/`enable(id)`/`isDisabled(id)`/`hydrateDisabled(ids)`, and two views:
  - **`active`** (the LISTED set — `[...CLIENT_PLUGINS, ...installed customs]`, unchanged) → `plugin-list` reads this, so a disabled custom stays **listed** (shown disabled).
  - **`mounted`** (NEW — `[...CLIENT_PLUGINS, ...installed customs NOT in disabled]`) → the shell derives `widgetRegistry`/`bgWidgets`/`titles` from **this**, so a disabled custom's widget **unmounts**.
- The shell (app_client) `disable` wrapper = **remove the layout leaf** (the D-119 `removeRegion` path) + mark disabled + persist `session.disabled`; `enable` = **re-inject the leaf** (`insertLeaf`) + unmark + persist. Reuses D-119's leaf primitives — no new algebra.
- Only **custom** plugins disable. System rows (W-13) never disable → the button is greyed for them, legibly. `surface:'none'` customs: none exist today; if one did, disable would just toggle `mounted`/its socket. `version` stays 3 (a session key, not a Layout field).

### 3.3 The action row (plugin-list stays presentation-only, W-3)
- Render `[info][settings][disable][uninstall]` per row as **icon-buttons with hover tooltips** (use the core `button`/`icon` shapes; order exactly as above).
- plugin-list is `$common` and must not import a shell dep → the buttons emit through **one callback seam** `onAction?(id: string, verb: 'info'|'settings'|'disable'|'uninstall'): void` (the `onCommand`/`onManage` precedent; one seam extends cleanly to future buttons). The **shell** (plugins-dialog → app_client) wires it to the existing `__XGEN_PLUGINS__` handlers + the new disable/enable handlers.
- Each button's state is **descriptor-derived** (§4 table). A verb with no feeder anywhere ships **absent**, never dead-grey.

### 3.4 Leading kind-glyph (module vs widget)
- A leading `icon` per row whose **which-glyph + colour derive from `host`** (`node` = module · `client` = widget). It replaces/absorbs the text `[system]/[user]` badge's job of showing *what kind of thing this is*.
- **Appearance is Joe's:** ship a PROVISIONAL glyph + red/blue colour (flagged → `M-RP-SKIN`). The mechanic (glyph bound to `host`) is the DoD; the shape/colour is not. *(All rows are `host:'client'` today; `host:'node'` module rows arrive with `M-RP-PLUGINS-NODE`.)*

### 3.5 Info detail view
- `info` opens a minimal **plugin-detail view** (an in-modal detail panel or a small sub-view) showing the full descriptor: name, id, kind, host, delivery, surface, description. Keep it thin; appearance PROVISIONAL. This is `info`'s feeder — it must do something real this leg (no dead button).

---

## 4. Feeder discipline (the honesty contract)

| button | enabled when | greyed (reason true of the plugin) | absent when |
|---|---|---|---|
| **info** | always (detail view built here) | — | — |
| **settings** | `!!settingsComponent` (none today → greyed for all) | *"no settings"* — a per-plugin fact, not a missing verb | — |
| **disable** | `kind==='custom'` && not already disabled (→ shows *enable* when disabled) | `kind==='system'` (W-13) | — |
| **uninstall** | `kind==='custom'` (D-119) | — | `kind==='system'`: absent (or greyed-legible — Joe's visual call) |

---

## 5. Definition of Done (each verified with real output; Rule 7)

1. `PluginDescriptor.settingsComponent?` added; `hasSettings` derive drives the `settings` button; greyed for all rows this leg (real reason, not hardcoded).
2. `installed.svelte.ts`: `disabled` set + `disable`/`enable`/`isDisabled`/`hydrateDisabled` + `mounted` view; shell derives `widgetRegistry`/`bgWidgets`/`titles` from `mounted`, `plugin-list` from `active`.
3. `session.disabled` persists via the N-107 per-key merge; **`cargo test` 1517/0/62 IDENTICAL** (proves no Rust), summed programmatically.
4. Action row renders `[info][settings][disable][uninstall]` icon-buttons + tooltips + leading kind-glyph; every state descriptor-derived per §4.
5. **Disable a custom (connection-stats):** its leaf is removed and the widget unmounts, the row stays **listed** as disabled, re-enable re-injects, and the disabled state **persists across a full reload** (drive it, don't assert it — N-095/N-091 shape).
6. Uninstall still works (D-119); system rows' disable/uninstall greyed/absent per §4.
7. `info` opens the detail view (real content, not a stub).
8. Verified live `9222` only, Rule 5 re-drive by Chat; registry baseline read **quiescent after a full reload** (N-132), stating store + selection + disabled + saved-state counts (N-105/N-108/N-112/N-115). `npm test` + `vite build` quoted. Sampler catalogue **328 unchanged, by scope** (`git show --stat` — no `ui/sampler`, no `.rs`).
9. Appearance (glyphs, density, colours) ships PROVISIONAL → `M-RP-SKIN`; that is expected, not a defect.

---

## 6. Out of scope (do not build here)

- The **Settings** modal shell + File ▸ Settings entry + gear re-point + `plugin-list`-as-Plugins-section → **Leg A** (prerequisite, already shipped).
- The `settings` button's actual open-a-component mechanism → **Leg C** (grid-plate backdrop is the first tenant).
- Auto-disable-on-version-incompat (needs D-118 manifest semver) — the disable button's *second* feeder, future.
- `host:'node'` module rows → `M-RP-PLUGINS-NODE`.
- Any appearance decision — Joe's, via `M-RP-SKIN`.

---

## 7. Notes / tooling

- CDP harness `cdp-debug.ps1 -App client` (9222); coords CSS px (DPR does not apply); `get(id)` returns `{type,state}` — read `.state.foo`; read the DOM in a **separate** eval after a mutation (Svelte effect-tick); single-expression `JSON.stringify(...)` evals only. Read a baseline **after a full reload** (N-132), and the registry keys on `data-debug-id`, not `id` (N-110).
- Joe launches `tauri dev`; Chat drives the already-running app over CDP. Joe pushes — never Chat.
