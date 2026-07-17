# M-RP-SETTINGS — Leg C — the settings mechanism (D-B → D-120) + the grid-plate backdrop setting
> **Status**: ACTIVE  
> Version: 1.0  
> Date: July 2026  
> **Last updated**: 2026-07-17  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. Read first

Rule-0 reading stack: `CLAUDE.md` PLAY block → `JOURNAL.md` J-539 (this leg's design lock) + J-537 (Leg B close, baseline 99) → **this runbook** → `docs/xgen-settings-phase0.md` **§9** (the mechanic lock — the canonical source) + §2 D-B. This runbook is item 3, not item 1.

**Lane:** Clair implements; Chat re-drives every verification leg on the real client `9222` (Rule 5) after a **full reload** (N-132). **Appearance is Joe's** — the *look* of the plate's two backdrop states, and any chrome, ships **PROVISIONAL → M-RP-SKIN**; this runbook fixes only the **mechanic** (which control exists, what value it drives, how it round-trips, how it is verified).

**Client-only.** No sampler (`ui/sampler` catalogue must stay **328**, by scope). **No Rust planned** — `cargo test` must stay **1517/0/62 IDENTICAL** (the honest proof no Rust landed; all frontend + the opaque-blob store path).

**This is the LAST leg of the SETTINGS arc.** Leg A (shell) + Leg B (action row + Settings window) are CLOSED. Leg C proves **D-B**: a plugin ships its own settings component, the content pane hosts it — with `grid-plate`'s backdrop as the first real tenant.

---

## 1. What this leg is

Two joined mechanics, one proof:

1. **The settings mechanism (D-B → D-120).** A plugin's own `settingsComponent` opens **in the Settings content pane** (component-per-plugin; NOT a declarative `settings_schema`). Leg B already shipped the greyed-for-all `[settings]` button whose enabled state is `!!p.settingsComponent`; this leg feeds **one** descriptor and hosts the component.
2. **The grid-plate backdrop setting (B2).** `grid-plate` ships the first `settingsComponent`. It drives **ONE value the plate visibly renders** — proven on the **painted DOM** (N-097), persisted per-device, restored on relaunch. The full static/generative/data-driven menu stays **`M-RP-BACKDROP`** (NOT this leg).

**Locked (J-539):** D-number **D-120** (minted at close) · swap machinery **= REUSE** the Leg-B drill-in · backdrop **= B2** (one painted value).

---

## 2. Grounding (verified 2026-07-17 — re-confirm before you edit; N-116)

- **`settings-dialog.svelte`** (shell) content-pane swap: (a) section swap — all sections mounted, inactive `[data-active]` → `display:none`; (b) inside the Plugins panel, a **list↔detail `{#if}` swap** — `{#if detailId && detailPlugin}<PluginDetail>{:else}<PluginList>`, driven by `info` → `detailId` (local), Back → `null`. The shared `.settings-header` owns Back (only in a drill-in) + the × close. `handlePluginAction(id, verb)`: `if (verb === 'info') detailId = id; else onPluginAction?.(id, verb)`.
- **`app_client.handlePluginAction(id, verb)`** handles only `uninstall`/`disable`. So today `settings` is **forwarded here and is a no-op**.
- **`plugin-list.svelte`** (`$common`, W-3): `actionsFor()` derives the `settings` button `disabled: !hasSettings`, `hasSettings = !!p.settingsComponent`; the guarded `onclick={() => !a.disabled && onAction?.(p.id, a.verb)}` never fires when greyed. `settingsComponent?` is **undefined on every row today.**
- **`PluginDescriptor.settingsComponent?: Component`** (`ui/common/lib/plugins/registry.ts`) exists (Leg B) but is set on no row. `grid-plate`'s row is `{ id:'grid-plate', kind:'system', host:'client', delivery:'compiled', surface:'none', component: GridPlate }`.
- **`grid-plate.svelte`** (`$common`): props `{ backgroundLive?, id = 'grid-plate' }`; **accepts `backgroundLive` and ignores it** (inert); getter `{ backgroundLive }`. One `<div class="grid-plate" use:envelope>`; the raster look is `.grid-plate` in `skin.css`.
- **`region-shell.svelte`** (`core`): props include `background`, `backgroundLive = true`, `bgWidgets`; resolves `background` against `bgWidgets`, mounts each `<W {...props} {backgroundLive} />`; G reports `backgroundMountCount` + `backgroundLive`.
- **`app_client.svelte`**: `<RegionShell … {background} {bgWidgets} backgroundLive={true} … />` — the `backgroundLive={true}` **literal is the seam this leg binds**. `background = $state(DEFAULT_BACKGROUND)` (+ a `__XGEN_*` DEV bridge). `bgWidgets = $derived(buildBgWidgets(mountedPlugins))`.
- **`uistate.svelte.ts`** (shell-local): the session bag holds independent per-key writers `setSessionLayout`/`setSessionLocked`/`setSessionInstalled`/`setSessionDisabled`, each an **N-107 per-key merge** (geometry stays Rust's), each debounced through `scheduleSessionPersist` → `persist()`. `disabled` is hydrated **before `loadLayout`** in `app_client`. A new backdrop key mirrors this shape exactly.
- `layout-default.ts`: `buildBgWidgets(plugins)` = `surface:'none' && component` rows; `DEFAULT_BACKGROUND = [{ widgetId: 'grid-plate' }]`.

---

## 3. Scope — the mechanics

### 3.1 The content-pane drill-in generalizes (swap = REUSE)

In `settings-dialog.svelte`, generalize the Leg-B drill-in so it carries a **mode**. Two equivalent shapes — Clair's call:

- **(preferred)** replace `detailId` with a single `drill = $state<{ id, mode: 'info' | 'settings' } | null>(null)`; or
- keep `detailId` and add a parallel `settingsId = $state(null)` (mutually exclusive).

Then:
- `handlePluginAction(id, verb)`: `if (verb === 'info') drill = { id, mode: 'info' }; else if (verb === 'settings') drill = { id, mode: 'settings' }; else onPluginAction?.(id, verb)`. **`settings` is intercepted LOCALLY** — never forwarded. `app_client.handlePluginAction` is **untouched**.
- The Plugins panel renders three targets: `info` → `<PluginDetail>`; `settings` → the plugin's own component; else → `<PluginList>`. Back (`drill = null`) returns to the list; the shared header shows Back + the plugin name in both drill modes.
- The settings drill-in resolves the plugin from `installed.active` (the LISTED set, the `detailPlugin` precedent); if it vanishes while open, the derived is null → fall back to the list.

**Generic mount (no per-plugin branch):**
```svelte
{#if drill?.mode === 'settings' && drillPlugin?.settingsComponent}
  {@const C = drillPlugin.settingsComponent}
  <C />
{/if}
```
The `[settings]` button already enables itself off `hasSettings` — **no `plugin-list` change**.

### 3.2 The `$common` backdrop store

New `ui/common/lib/stores/backdrop.svelte.ts` (the `self-state.svelte` / `installed.svelte` precedent; W-3 — a `$common` store the `$common` `grid-plate` and its `$common` settings component both read/write, and the shell mirrors). It holds **one value** (v1): a boolean (or a tiny enum if Joe names >2 visible states — mechanic sizes to the value). Getter + setter; a DEV `__XGEN_BACKDROP__` handle (mirrors `__XGEN_UISTATE__`).

### 3.3 `grid-plate` reads the value and PAINTS it (B2)

`grid-plate.svelte` **stops being fully inert**: read the `$common` backdrop value and branch the render on it (e.g. a `data-*` attr the skin keys, or a class). The two states' **look is Joe's → M-RP-SKIN**; the mechanic reads one value and reflects it so the skin can paint two states. `backgroundLive` stays accepted (its live/frozen contract is future reactive plates — M-RP-BACKDROP); this leg's visible value is the new one. Update the getter to report the rendered value so the flip is CDP-observable.

### 3.4 The grid-plate settings component

New `ui/common/lib/components/widgets/grid-plate-settings.svelte` (`$common`): renders the control for the one value (composed from `core` — the *look* is Joe's), **writes the `$common` backdrop store**. Set it as `grid-plate`'s `settingsComponent` in `registry.ts`. That single assignment lights the `[settings]` button on the Grid Backdrop row and nowhere else.

### 3.5 Shell binding + persistence

In `app_client.svelte`:
- Replace `backgroundLive={true}` with the bound value (or thread the backdrop value into the mount props if the plate reads it via props rather than the store directly — prefer the store, W-3/N-096).
- **Persist** the backdrop value via a new `uistate` session key — a `setSessionBackdrop(v)` method mirroring `setSessionDisabled` (N-107 per-key merge, **zero Rust**), added to `UiStateBag`.
- **Hydrate before `loadLayout`** (the `hydrateDisabled` precedent) if the value must be correct at first paint, and seed the `$common` store from it, so the persisted choice paints on relaunch.

---

## 4. Descriptor / store additions

- `grid-plate`'s `registry.ts` row gains `settingsComponent: GridPlateSettings`. No other row changes.
- `UiStateBag` gains one **session** key (e.g. `backdrop?`) — per-device, N-107 per-key merge, zero Rust, no `Layout` schema change (`version` stays 3).
- New `$common` files: `stores/backdrop.svelte.ts`, `components/widgets/grid-plate-settings.svelte`.

**RESERVE NOTHING** beyond this one key/value. Static/generative/data-driven, base-vs-stack → `M-RP-BACKDROP`.

---

## 5. Legs (visible-first)

- **C1 — the mechanism.** Generalize the drill-in (§3.1), intercept `settings` locally, generic-mount a `settingsComponent`. Ships with `grid-plate` as its one fed tenant (a settings component the pane hosts) → the `[settings]` button lights for Grid Backdrop. Hand back for Joe's eyes.
- **C2 — the backdrop value.** The `$common` store + `grid-plate` painting one value (§3.2/3.3) + the grid-plate settings component (§3.4) + the shell binding & persistence (§3.5). The painted flip + reload-survival is the proof.

*(C1 is unprovable without a fed tenant, so C1+C2 may land together; visible-first order = mechanism first, then the value it carries. Clair's call whether one commit or two — each with its own verify pass.)*

---

## 6. Definition of Done

- [ ] `settings` opens `grid-plate`'s component in the content pane (drill-in reuse); Back returns to the list; `info` still works; `app_client.handlePluginAction` untouched.
- [ ] The `[settings]` button is enabled ONLY on the Grid Backdrop row (descriptor-derived), greyed elsewhere.
- [ ] Toggling the backdrop setting **flips what the plate renders on the painted DOM** (N-097 — not getter-only).
- [ ] The choice **survives a full reload** (persisted session key, hydrated before `loadLayout`).
- [ ] `region-shell` G `backgroundLive` + `grid-plate` G reflect the value; registry delta measured **after a full reload** (N-132), quiescent, cite the store state (N-108).
- [ ] `cargo test` **1517/0/62 IDENTICAL**; sampler catalogue **328**; `vite build` + `npm test` clean.
- [ ] **D-120 minted in `DECISIONS.md`** at close (the settings mechanism = component-per-plugin, hosted in the Settings content pane; `settings_schema` superseded, not built) + the canonical records (JOURNAL close entry, CLAUDE.md PLAY, ROADMAP, `docs/xgen-settings-phase0.md` §9 → COMPLETED).
- [ ] Every `.md` write refreshes the header (each `>` line ends in two spaces; date = the close session's).

*(DoD checklists never include "commit pushed" — Joe pushes; `Status: COMPLETED` is the real signal.)*

---

## 7. Discipline

Real client **9222** only, Rule 5 (Chat re-drives every leg after a full reload, N-132; baseline cited from **99**). No backticks in any PowerShell handed to Joe — one physical line per command, or a here-string. Joe locks architecture, Joe pushes (never Chat). Appearance is Joe's → M-RP-SKIN. Filed follow-on: **`M-RP-BACKDROP`** (the backdrop-type menu).
