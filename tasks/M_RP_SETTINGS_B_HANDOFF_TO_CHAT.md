# M-RP-SETTINGS Leg B — HANDOFF TO CHAT (verify + doc-bridge)
> **Status**: COMPLETED  
> Version: 1.1  
> Date: July 2026  
> **Last updated**: 2026-07-17  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. What this is

Clair implemented **M-RP-SETTINGS Leg B — the plugin action row**, and Joe then directed an extended **visible-first appearance + Settings-window** pass live over HMR. Joe reviewed every change on the running client and said **"this round is good and can be closed."** Code is committed (**`15c1cd9`, not pushed — Joe pushes**). Two formal steps were deliberately **left for you** rather than faked:

1. the **CDP Rule-5 re-drive** on live client 9222 (the quiescent registry baseline shifts this round — measure it, do not assert it, N-105/N-108/N-132), and
2. the **doc-bridge** (JOURNAL J-537 + CLAUDE.md PLAY + ROADMAP), which by **Rule 4** is written *after* that verify.

Rule-0 stack before you act: CLAUDE.md PLAY head → JOURNAL J-536 (Leg-B GO) / J-535 (Leg A close, baseline 86) → runbook `tasks/M_RP_SETTINGS_B_ACTION_ROW.md` → this handoff.

**Lanes unchanged:** Chat re-drives every verification leg on the real client 9222 after a full reload (Rule 5); Joe pushes. Static gates below were run by Clair (real output) but re-run them if you want the belt.

---

## 1. What shipped (commit `15c1cd9`, 20 files, frontend-only)

### 1a. The five Leg-B mechanics (runbook §3, all descriptor-derived, honest per §4)
- **`PluginDescriptor.settingsComponent?`** → `hasSettings = !!settingsComponent`; **undefined on every row** → `[settings]` greyed for all, real reason *"No settings"* (never a missing verb).
- **`installed.svelte` disabled axis:** new `disabled` `$state<Set>` + `disable`/`enable`/`isDisabled`/`disabledIds`/`hydrateDisabled`, and a new **`mounted`** view (system + installed-not-disabled). The shell derives `widgetRegistry`/`bgWidgets`/`titles` from **`mounted`**; `plugin-list` reads **`active`** (disabled custom stays LISTED, its widget UNMOUNTS). `uninstall` also clears the disabled flag.
- **Action row** `[info][settings][disable][uninstall]` — native `<button>`s composing core `Icon` (the shelf-face pattern — core `button` renders text only), `aria-disabled` + guarded onclick (focusable/legible greyed), one seam `onAction(id, verb)` (W-3 held). `data-verb`/`data-plugin` are the CDP hooks.
- **Leading per-plugin icon** (see 1c) — colour host-derived in skin.
- **`info`** → a real **`plugin-detail`** view (descriptor `<dl>`) in the content pane; NOT a stub.
- **Persistence:** `session.disabled` via the N-107 per-key merge in `uistate.svelte.ts` (mirrors `setSessionInstalled`), **zero Rust**; boot `hydrateDisabled` runs after `hydrate`, before `loadLayout`.

### 1b. Row model — ONE line per plugin (Joe)
`[plugin-icon] Name  vX.Y.Z  … [info][settings][disable][uninstall]`. **Description + the host·delivery·surface axes were REMOVED from the list row** and now live only in the `info` detail view. The `[system]/[user]` **badge was replaced by the plugin VERSION** (Joe: the badge was redundant with the icon colour + the info view). New descriptor field **`version?`** = `'1.0.0'` on all 5 plugins (declared placeholder; real per-plugin versions come from D-118 manifests). Kind is still shown in the info view (carries the built-in/installed fact).

### 1c. Icons — real Material Icons, sourced this round (D-108/D-110)
11 new glyphs added to `ui/core/.../icons.ts`, each **byte-exact from the Material Icons repo (Apache-2.0)**, **colour-free** (the `fill="none"` bounding rect dropped so `currentColor` tints, D-110), with provenance **`.svg` saved in `ui/assets/icons/`** (D-108). Mapping:
- action row: `info` (little-i) · `gear` (settings, existing) · `toggle-on`/`toggle-off` (disable = a SWITCH, swaps by live state) · `delete` (uninstall = trash).
- chrome: `chevron-left` (round Back `<`) · `close` (round `×`).
- per-plugin (`descriptor.icon`): Self Panel → `person` · Inspector Panel → `search` · Plugin List → `extension` (puzzle) · Grid Backdrop → `wallpaper` · Connection Stats → `signal`.

### 1d. Settings becomes a WINDOW, not a plain dialog (Joe, visible-first)
- **Own header row** (spans both grid columns): round Back `<` (drill-in only) + context **title** (`"Settings"` in the list, the plugin **name at 14px** in the detail) + round **`×`** close, all one line. The stock dialog **title + footer are suppressed** for this modal.
- **Solid window-linked area:** `--settings-inset: 120px` (Joe's value; Clair shipped 40px, Joe changed it) → `width/height = calc(100vw/100vh − 2×inset)`, base `margin:auto` centres it → equal gap on all four edges; **resizes with the main window**. One tunable token.
- **Independent vertical scroll** on the two columns (nav + content), `min-height:0` + `overflow-y:auto`; header stays fixed. **Thin scrollbars** (`::-webkit-scrollbar` 8px, ~half the default) scoped to the two scrollers.
- **⚠️ Regression found by Joe + fixed:** the open-state `display:flex` (needed for the scroll layout) first sat on `.dialog:has(.settings)` unconditionally — specificity `0,2,0` **beat** the UA `dialog:not([open]){display:none}` (`0,1,1`), so the **closed modal stayed visible** in normal flow (no backdrop). Fixed by scoping to **`.dialog[open]:has(.settings)`**. Worth an N-note (a shell rule that overrides `display` on a native `<dialog>` must be `[open]`-scoped or it leaks the closed state).

---

## 2. Static gates (Clair-run, real output — re-run if you want the belt)
- `vite build` (ui/client): **175 modules** (Leg A 174; +1 = `plugin-detail.svelte`).
- `npm test` (ui/sampler): **77 passed** (4 files), unchanged.
- **Scope proves the Rust/sampler claims:** `git show --stat 15c1cd9` = 20 files, **no `.rs`, no `ui/sampler/`** → `cargo test` **1517/0/62 IDENTICAL by construction**, sampler catalogue **328 unchanged**. (`icons.ts` is `core` but data-only — a glyph map, no component/cell, so no catalogue impact.)

---

## 3. Verify legs to re-drive (live client 9222, full reload first — N-132)

**Baseline (N-105/N-108/N-112/N-115 — state the conditions):** read the client registry **quiescent, empty store, no selection, nothing folded, zero saved states**, AFTER a full reload. It SHIFTS this round — **measure it, do not assert it.** Expected delta from Leg A's 86, for enumeration (verify, don't trust): per plugin-list row **+3** (−`__desc` Label −`__meta` Label +`__icon` Icon +4 action Icons) × 4 system rows = **+12**, plus **+1** `icon#settings__close` (header, always mounted) → **≈ 99**. The new stable ids are `icon#plugin-list__<pid>__{icon,info,settings,disable,uninstall}` and `icon#settings__close`. Confirm `count===unique`.

1. **Open paths:** gear (`widget.manager`) → Settings @ **Plugins**, `:modal` true (read `.matches(':modal')`, never the attribute); File ▸ Settings → @ **About** default.
2. **⚠️ Closed-modal (the regression):** at fresh start and after close, `dialog#settings` is **display:none / not visible** and NOT in normal flow. Drive open → close → confirm it disappears and the registry returns to baseline (no leak). This is the leg Joe's bug maps to.
3. **Window chrome:** the modal fills the window minus 120px each edge (measure the rect vs `innerWidth/Height`); resize the window → the modal tracks it; header fixed; the two columns scroll independently (inject tall content or drive scrollTop on `.settings-nav` / `.settings-content`).
4. **Action-row states (paint + `data-verb`/`aria-disabled`):** system rows → `settings`/`disable`/`uninstall` all `aria-disabled`; `info` always live. Drive the row buttons (`.click()` — plain onclick, trusted enough; synthetic KEY events are not, J-496).
5. **Disable/enable + persist (DoD §5.5 — drive it, N-095 shape):** `__XGEN_PLUGINS__.install('connection-stats')` → its row disable button (or `__XGEN_PLUGINS__.toggleDisabled('connection-stats')`) → widget UNMOUNTS from the grid, row stays LISTED as `[user]`-equivalent with the switch flipped to **Enable**; re-enable re-injects; **disabled state persists across a full reload** (`session.disabled` on disk).
6. **`info` detail:** click a row's info → `plugin-detail#plugin-detail` renders the real `<dl>` (Name/Version/Id/Kind/Host/Delivery/Surface/Description) as painted text (N-097); the shared header shows Back `<` + the plugin name (14px); Back returns to the list.
7. **Version line + no badge:** each row paints `v1.0.0`; no `[system]/[user]` badge remains in the list (`.pl-badge` is gone).
8. **Icons:** each leading + action `<svg class="icon">` has a real single/multi `<path>` with non-degenerate `getBBox()`; the plugin icon inherits the host colour (`.pl-icon[data-host="client"]` blue today).
9. **Gates:** re-confirm `cargo test` **1517/0/62 IDENTICAL** (run detached + poll, N-117; apps down or it locks the exe), `npm test` **77**, `vite build` **175**, and `git show --stat 15c1cd9` = 20 files no `.rs`/sampler.

---

## 4. Flags / deviations for the record (Rule 6 — surfaced, not absorbed)

- **⚠️ D9 debt — the 2nd `:has()` footer suppression.** Settings now suppresses the stock `dialog` **title + footer** to own its header/`×`. Per J-512 D9 that makes the **dialog header/footer-SNIPPET extraction OWED as its own milestone** (so About/UI-state/Settings share one custom-chrome mechanism instead of each hacking `:has()`). Flagged, **not built**. Joe's framing settles the tension: *Settings is a WINDOW, not a common dialog* — the divergence is the point, not an accident to reconcile; the snippet extraction is housekeeping, not a blocker. **File it (M-RP-DIALOG-CHROME or similar) in the records.**
- **Version = declared placeholder** (`1.0.0` for all five) → real per-plugin versions arrive with **D-118** manifests / **M-RP-PLUGINS-NODE**. Not a defect; note it.
- **Host-tint kept on the plugin icon** (module red / widget blue) — all rows are `client` today, so every icon renders blue. Joe approved live; if he later wants neutral icons it's a one-line skin change.
- **Uninstall-on-system ships GREYED-LEGIBLE** (not absent) — the W-13-rendered reading (J-513); Joe's absent-vs-greyed call, PROVISIONAL → M-RP-SKIN.
- **Action buttons are native `<button>` composing core `Icon`**, not core `Button` (which is text-only) — the shelf-face/menu-item precedent.
- **The closed-modal `display` regression** (§1d) — worth an **N-note**: a shell rule overriding `display` on a native `<dialog>` must be `[open]`-scoped, else it outranks the UA `:not([open])` hide and leaks the closed state.
- **Appearance is PROVISIONAL → M-RP-SKIN:** glyph shapes/mapping, the host colours, row density, the 120px inset, the 8px scrollbar, the round-button sizing — all Joe's to retune.

---

## 5. Doc-bridge to write (Rule 4 — after the §3 verify)

- **JOURNAL J-537** — M-RP-SETTINGS Leg B CLOSED: the plugin action row + the Settings window. Feat **`15c1cd9`** [Clair, code-only, 20 files, NOT pushed]. Record the measured baseline (§3), the static gates, the §4 flags, and the D9-debt milestone filing. Note this was a heavy **visible-first** round — Joe directed the row model, per-plugin icons, version line, and the whole window chrome live over HMR (list the corrections as design evolution, not defects).
- **CLAUDE.md PLAY head** — add the Leg B CLOSED block; **cite the new measured baseline** (with conditions), keep it compact.
- **ROADMAP** — Leg B ✅; **next-active = Leg C** (the settings mechanism, D-B, + the `grid-plate` backdrop setting in the content pane — the settingsComponent's first real tenant). File the **dialog header/footer-snippet extraction** milestone (D9).
- **No new `D`, no new `core`** (the action row is a `$common` widget + shell chrome; `dialog`/`icon` core untouched except the additive glyph-map data). The plugin icons + version are additive descriptor fields.

---

## 6. Files (commit `15c1cd9`)

`registry.ts` (+version, +icon, +settingsComponent) · `installed.svelte.ts` (disabled axis + mounted) · `plugin-list.svelte` (one-line row, action bar, version, seam) · `plugin-detail.svelte` (NEW, info view) · `settings-dialog.svelte` (header + Back/×, onPluginAction seam, detail swap) · `app_client.svelte` (mounted derive, disable/enable + action handlers, hydrateDisabled, DEV bridge) · `uistate.svelte.ts` (session.disabled + merge) · `icons.ts` (+11 glyphs) · `skin.css` (row/actions/version/header/round-buttons/window-size/scroll/scrollbars, the `[open]`-scope fix) · `ui/assets/icons/*.svg` (11 provenance sources).
