# M-RP-SETTINGS — the Settings modal + plugin manager (Phase-0)
> **Status**: COMPLETED  
> Version: 1.2  
> Date: July 2026  
> **Last updated**: 2026-07-17  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. One sentence

`M-RP-SETTINGS` stands up **one** Settings surface — an in-DOM modal shaped like Discord's: a **left category menu (~¼ width) + a content pane** that swaps per selection, more compact (never a new OS window). The **plugin manager is a section inside it** (the `Plugins` category), and a plugin's own settings render in the content pane.

**Two entry points, one modal:** the `gear` shelf face opens Settings landed on the **Plugins** section; a new **File ▸ Settings** menu item (above Restart) opens it on its default section. It proves the per-plugin settings mechanism end-to-end via the grid backdrop, which is `grid-plate`'s own setting.

---

## 1. What is already locked / reused (no re-decision)

- **The install/uninstall engine — D-119.** Reactive registry · `AVAILABLE_CUSTOM` vs `CLIENT_PLUGINS` vs the installed-set · register-before-`loadLayout` · `session.installed` per-device. The `[uninstall]` action reuses `installed.uninstall` + the shell's leaf-remove wrapper verbatim.
- **The pane — `plugin-list` reused.** `plugin-list.svelte` (the 5th widget, `kind:'system'`, `surface:'none'`) already exists as the read-only list. It becomes the **Plugins section** of Settings (gaining the action row in Leg B); `plugins-dialog.svelte` is **absorbed** into the Settings modal. The list component is the reusable part.
- **The backdrop socket — J-532.** `region-shell`'s `background?: WidgetMount[]` + `backgroundLive` ship FED and inert. The backdrop setting binds a chosen backdrop into this socket; it does not build a new one.
- **The core `dialog`** (C1, M-RP6.1e) — the modal container; the About/Plugins/UI-state dialogs all wrap it.

---

## 2. Decisions this walk settles (Joe-locked this session)

### D-A — "Settings window" = an in-DOM modal, not a new OS window

D-112 penciled Settings as `surface:'window'`. **Joe's reading (2026-07-16): `surface:'window'` means a *standalone modal area* — the same mechanism we already have for About and Plugins — not a modern second OS window.** So there is no frame arc: no second Vite entry, no second CDP target, no new typed Rust geometry struct (D-114/D-115 untouched). The taxonomy stands as written; `window` = a self-contained modal surface. *(Grounded: `tauri.conf.json` has exactly one window today; the four "dialogs" are in-DOM modals inside it.)*

### D-B — J-513 → **B** (component-per-plugin); `settings_schema` (A) is not built

The J-513 gate — *how a plugin's settings get drawn* — was binding-deferred until the grid works. The grid works. **Resolved: B.** A plugin that has settings ships **its own settings component**; the Settings modal hosts it. This is the shipped widget-tier pattern (a widget hosting content, §3.2); the declarative `settings_schema` auto-render (Ch6 §6.8.2/§6.8.5, **zero lines exist**) is **not** built and is superseded as a path — *"it does not need to be yet another widget system"* (Joe). This is a **technical/mechanism** choice; the *look* remains Joe's.

**Formalization (Joe-locked 2026-07-17): D-B takes the reserved number `D-120`.** Per the project's decision hygiene (D-117/D-118/D-119 precedent — a decision enters `DECISIONS.md` when its design is *built and looked at*, not predicted), the **D-120 entry is minted at Leg C close**, not now. The reserved number is fixed; the record lands with the code. Mechanic-locked in **§9**.

### D-C — ONE Discord-shaped Settings modal; the gear deep-links to its Plugins section (Joe, 2026-07-16)

Settings is a **single** in-DOM modal shaped like Discord's: a left **category menu (~¼ width)** + a **content pane** that swaps per selection (compact). Settled across the walk — the first draft had the gear open a plain Settings modal and absorb `plugins-dialog`; the second split it into two separate modals; **the final shape is ONE modal with the plugin manager as a category**:

- **Sections** = app-level categories + a **Plugins** category (the manager: the `plugin-list` rows + the action row) + per-plugin settings that render in the content pane.
- **Two entry points, one modal:** the `gear` shelf face (`widget.manager`) opens Settings **landed on the Plugins section**; **File ▸ Settings** (new `settings.open` command + a new File menu item **above Restart** — File is `Restart · —— · Exit` today) opens it on the **default** section. `plugins-dialog.svelte` is **absorbed** (its job becomes Settings' Plugins section).
- The deep-link is trivial: the Settings modal takes a `section` argument; the gear passes `plugins`, File ▸ Settings passes the default.

*The sidebar is a selectable category list (content-pane swap) — a roving-tabindex list; whether it reuses an existing rover or triggers the filed `M-RP-ROVING` extraction is a Leg-A mechanic call. And a plugin's `[settings]` and the backdrop app-setting both resolve to one `settingsComponent` in the content pane — no second home (D-067).*

---

## 3. The row model (Joe's compact one-liner vision)

Every plugin is **one line**. Appearance (glyphs, spacing, colours, icons) is **Joe's**; the *mechanics below* — which control shows, and its enabled/greyed/absent state — are descriptor-derived and mine.

```
[kind-glyph]  Official Name   · meta ·        [info] [settings] [disable] [uninstall]
```

- **Leading kind-glyph** — red/blue, distinguishing **module vs widget**. Mechanic: the which-one derives from the `host` axis (`node` = module · `client` = widget/ui). Colour/shape is Joe's. Also makes the `[system]`/`[user]` distinction visual rather than a text badge. *(All rows are `host:'client'` today; `host:'node'` module rows enter with `M-RP-PLUGINS-NODE`.)*
- **meta** — "some useful data" (Joe, exact fields TBD). Candidates already on the descriptor: `[system|user]`, host, delivery, surface, status (installed/disabled). Trimmed for a compact line at build; Joe picks the fields.
- **action bar** — icon-buttons with hover tooltips, order **`[info] [settings] [disable] [uninstall]`**.

### The feeder discipline (no dead controls)

A control is greyed **only for a reason true of that plugin** (legible to the user, W-13 rendered); a verb that was **never built for anyone** ships **absent**, never dead-grey (J-500 / 6.1j).

| button | verb / feeder | enabled when | greyed when | absent when |
|---|---|---|---|---|
| **info** | a plugin-detail view (new, small) | always | — | — |
| **settings** | the plugin's own `settingsComponent` (D-B) | plugin ships settings | plugin has none (a real reason) | — |
| **disable** | a new `session.disabled` set (mirror of `session.installed`, N-107 per-key) | plugin is disableable | system plugin can't be disabled (W-13) | — |
| **uninstall** | `installed.uninstall` + leaf-remove (D-119) | `kind:'custom'` | — | system rows: absent or greyed-legible |

**disable — two feeders, one now:** v1 = **user toggle** (deregister-but-keep-installed: a distinct state from uninstall — the plugin stays installed, its widget unmounts). The second feeder — **auto-disable when a plugin is incompatible with the app version** (Joe) — needs manifest semver vs app version (D-118), which is future; the button's *state* already accommodates it.

**Further buttons "reveal through dev" (Joe)** — each lands with its feeder, never before.

---

## 4. Descriptor / store additions

- `PluginDescriptor.settingsComponent?: Component` — presence = `hasSettings` (D-B). `grid-plate` gets one first (its backdrop setting); others as they earn one.
- `session.disabled: string[]` in the UI-state store — the disable set, per-device, N-107 per-key merge, zero Rust (the `installed`/`layout`/`locked` precedent, D-114). `installed.active` (or the shell's derive) filters out disabled ids from the mounted set while keeping them installed/listed.
- No `Layout` schema change (`version` stays 3 — install/disable are session keys, not layout fields).

---

## 5. The settings mechanism's first real tenant is `grid-plate`

The J-513=B mechanism and the backdrop setting are **one proof**: opening a plugin's own settings component. **`grid-plate` is the exemplar** — the backdrop *is* its setting. **v1 delivers ONE value the plate visibly renders (B2, Joe-locked 2026-07-17)** — so the writer is proven on the **painted DOM** (N-097: a setting that moves nothing on screen is an untested writer), not by getter alone. `grid-plate` stops being fully inert: it reads one value from a `$common` backdrop store and branches its render. The *look* of the two states is Joe's (→ M-RP-SKIN); the mechanic is the round-trip settings-component → `$common` store → plate re-render + shell → `backgroundLive`/persist → hydrate-on-boot. This proves D-B end-to-end.

The full **backdrop-type menu** — static / generative / data-driven, base-vs-stack (the socket is `WidgetMount[]`, a stack) — is large enough to be its **own follow-on milestone** (`M-RP-BACKDROP`, filed), not this arc.

---

## 6. Leg roadmap (design-only; no code until Joe says go — Clair implements)

- **Leg A — the Settings shell + Plugins section (read-only).** The Discord-shaped modal: category menu (~¼) + content pane. First real content = the existing read-only `plugin-list` as the **Plugins** section. Wire both entry points: gear → Settings @ Plugins, new **File ▸ Settings** → Settings @ default. `plugins-dialog` absorbed. **No new verbs** → safe, visible-first: the whole Settings frame + manager on screen for Joe's eyes.
- **Leg B — the action row.** The Plugins-section rows gain `[info][settings][disable][uninstall]` + leading kind-glyph, states descriptor-derived. Live feeders: uninstall (D-119, custom-only), disable (new `session.disabled`), info (detail view in the content pane). `settings` present-but-greyed until a plugin ships one. This IS M-RP6.1m.
- **Leg C — the settings mechanism (D-B → D-120) + the backdrop setting.** A plugin's `settingsComponent` opens **in the content pane** by **reusing the Leg-B drill-in** (generalize `detailId` → `{ id, mode: 'info' | 'settings' }`; the `settings` verb is intercepted **locally**, not forwarded to the shell); `grid-plate` is the first tenant → proves D-B and delivers a **minimal B2 backdrop setting** (one value the plate paints). Mechanic-locked in §9.

Each leg: real client `9222` only (no sampler — these are shell-local), Rule 5 re-drive, registry baseline read **quiescent after a full reload** (N-132), stating store/selection/saved/disabled state (N-105/N-108). `cargo test` must stay **1517/0/62 IDENTICAL** unless a leg genuinely lands Rust (none is planned — all frontend + the opaque-blob store path).

---

## 7. Filed follow-ons (not this arc)

- **`M-RP-BACKDROP`** — the backdrop-type menu (static/generative/data-driven, base-vs-stack).
- **`M-RP-PLUGINS-NODE`** — the Rust/protocol read verb exposing `host:'node'` module rows (the red glyph's first real rows).
- **Auto-disable-on-incompat** — the disable button's second feeder (needs D-118 manifest semver).
- **M-RP6.1m alignment** — this arc IS the M-RP6.1m action row, landing per-line inside Settings rather than as a separate row milestone.

---

## 8. Appearance is Joe's

Per the standing autonomy split (broadened this session: mechanics are mine, **visual UI is Joe's**), everything in §3's look — the glyphs, the icon set, the row density, the modal chrome — is Joe's to shape. This doc fixes only the mechanics: which control exists, what feeds it, and when it is honest.

---

## 9. Leg C — the settings mechanism (D-B → D-120), mechanic-locked (2026-07-17)

**✅ BUILT + CLOSED (J-540, commits `5f4a6fe` + `8b7ca1a`) — D-120 minted.** Re-driven live 9222 (full reload, Rule 5): baseline **99 === unique 99** (store `backdrop:true`, N-108); `[settings]` **enabled for grid-plate only** → drill to “Grid Backdrop” (99→76) → the **real toggle flips the painted `data-pattern` both ways** (N-097) + persists + survives reload; the `untrack` fix (**N-136**) proven; `cargo` **1517/0/62 IDENTICAL**. The B2 minimal (one boolean the plate paints) is a mechanism proof; the full 4-type menu (type 1 = **solid/gradient**, Joe 2026-07-17) is **M-RP-BACKDROP**.

**Locked this session:** the D-number (**D-120**, minted at close), the swap machinery (**reuse**), and the backdrop-setting shape (**B2**). Grounded against live code (N-116 — grepped, not assumed).

### 9.1 Grounding (the three questions)

- **The content pane swaps two ways** (`settings-dialog.svelte`). (a) the sidebar **section swap** — all sections mounted, inactive `[data-active]` → `display:none` (CDP-stable). (b) inside the Plugins panel, a **list↔detail `{#if}` swap** — `{#if detailId && detailPlugin}<PluginDetail>{:else}<PluginList>`, driven by the `info` verb → `detailId`, Back → `null`, the shared header owning Back + ×. **The `settings` verb is currently FORWARDED to the shell** (`onPluginAction` → `app_client.handlePluginAction`, which handles only `uninstall`/`disable`) → today a **no-op**.
- **The backdrop is threaded but unpersisted.** `region-shell` takes `background` / `backgroundLive` / `bgWidgets`; `app_client` passes **`backgroundLive={true}` as a hardcoded literal** — the seam Leg C binds. `grid-plate` **accepts `backgroundLive` and IGNORES it** (inert by contract; the getter reads it back). `DEFAULT_BACKGROUND` is fixed in `layout-default.ts`; **no backdrop choice is persisted anywhere** (reserved for this milestone — "no store key, no descriptor key, no schema change").
- **`settingsComponent` is unfed.** `PluginDescriptor.settingsComponent?` is `undefined` on every row; `hasSettings = !!settingsComponent` → the `[settings]` button is greyed for all. Confirmed still unfed.

### 9.2 The mechanism (D-B → D-120)

A plugin with settings ships its **own settings component**; the Settings modal **hosts it in the content pane** (NOT a declarative `settings_schema`). Leg B built the greyed-for-all `[settings]` button *on purpose* — Leg C feeds **one** descriptor (`grid-plate`'s) and that row's button lights up; **no `plugin-list` change is needed**.

- **Swap = REUSE (Joe-locked).** Generalize the Leg-B drill-in from `detailId` to a single drill target carrying a **mode** (`{ id, mode: 'info' | 'settings' }`, or a parallel `settingsId` — Clair's call, same shape). Back returns to the list; the shared header fits. A second machine would be N-086 wrong-abstraction risk.
- **`settings` intercepted LOCALLY** in `settings-dialog.handlePluginAction` (like `info`), **not forwarded**. `app_client.handlePluginAction` is untouched (still `uninstall`/`disable`).
- **Generic mount:** `{@const C = plugin.settingsComponent}<C />` inside the Plugins panel — a third content-pane target.
- **D-120 is minted at Leg C close** (built-and-looked-at, the D-119 precedent), not here.

### 9.3 The backdrop setting (B2 — one value the plate paints)

- `grid-plate` ships a **`settingsComponent`** (a new `$common` widget). Being `$common` it **cannot import a shell store (W-3)** → the live backdrop value lives in a **new `$common` store** (settings-component writes, `grid-plate` reads — the `self-state` / `installed` precedent, N-096).
- `grid-plate` **stops being fully inert (B2):** it reads **one** value from that store and branches its render. The *look* of the states is Joe's (→ M-RP-SKIN); the mechanic reads one value.
- The **shell** mirrors the value into `backgroundLive` (replacing the `backgroundLive={true}` literal) and **persists** it via `uistate` — a new **session key** mirroring `setSessionDisabled` (N-107 per-key merge; geometry stays Rust's; **zero Rust**, `cargo` stays identical), **hydrated before `loadLayout`** if the value must be live at first paint.
- **RESERVE NOTHING beyond this one key.** The full backdrop-type menu (static / generative / data-driven, base-vs-stack) stays **`M-RP-BACKDROP`**.

### 9.4 Verify (the honesty legs)

Real client **9222** only, Rule 5, baseline **quiescent after a full reload** (N-132, cite from 99). `cargo test` **1517/0/62 IDENTICAL** (all frontend + the opaque-blob store path). The B2 proof is on the **painted DOM**: toggling the setting **flips what the plate renders**, the choice **survives a reload**, and the value stays getter-observable (`region-shell` G `backgroundLive` + `grid-plate` G). Registry moves by the settings-component's mount — **measured after a full reload, not derived**.
