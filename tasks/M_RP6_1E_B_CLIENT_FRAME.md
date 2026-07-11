# M-RP6.1e-B — Real-client frame consolidation (status-bar mount · state-indicator migration · resize · center-only scroll)
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-11  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. What this is

The second step of the M-RP6.1e client-frame consolidation split (J-493). **No new component is built** — this milestone *assembles* the already-shipped `core` frame components into the **real client shell** and retires the legacy hand-rolled chrome.

- **Scope authority:** `docs/xgen-client-frame-phase0.md` §10 (v1.4) + J-493. The scope was locked there; this runbook grounds it against the real files and adds two items the J-493 scope list missed (§1 catches).
- **Verify surface: REAL CLIENT ONLY (9222, `run-client.ps1 -Debug`).** The sampler structurally cannot host shell/window effects (D-097). **Sampler catalogue registry stays 309** — this milestone adds no sampler cell. The *client's own* registry changes; measure it, never predict (Rule 5).
- **No new D-series** — this is a D-107 extension. Decisions below are **arc-local D1–D10** (D-069).

---

## 1. Grounding (Rule 5 — read before writing code)

Confirmed against the real files on 2026-07-11:

| File | State today |
|---|---|
| `xgen-client/tauri.conf.json` | frameless (`decorations:false`), **`resizable:false`**, 420×260, `center:true`, no min size |
| `xgen-client/cdp.dev.conf.json` | **duplicates the whole `windows[0]` object** (420×260, `resizable:false`) + `additionalBrowserArgs: --remote-debugging-port=9222` |
| `xgen-client/capabilities/default.json` | `core:default` + `process:default` only |
| `ui/client/src/app_client.svelte` | `<MenuBar>` in `.app-frame`; `.app-body` centres `#core-ui-pane` = `<img id=app-logo>` + hand-rolled `.state-indicator` (`dotColor()` / `isPulsing()` / `currentState.label`) + `<Button id=quit>`; quit seam `invoke('quit')` |
| `ui/client/src/app.css` | `.app-frame` / `.app-body` / `#core-ui-pane` / `img#app-logo` / `.state-indicator` / `.state-dot` / `.state-label` / `@keyframes dot-pulse` / `.button-pane` |
| `status-bar` props (shipped, J-494) | `states` · `state` · `pulse` · `caption` · `linkHref/linkText/linkExternal/onLinkClick` · `secondaryText?` · `grip=true` · `onResizeGrip?` · `id` |

**Catch 1 — the dev-config overlay.** Tauri's `--config` overlay replaces the `windows` array wholesale. Flipping only `tauri.conf.json` would leave the **debug** window (the one we CDP-verify on) at 420×260 / `resizable:false`. **Both files flip.** *(Not in the J-493 scope list — added here, Joe-locked 2026-07-11.)*

**Catch 2 — capabilities.** Tauri v2's `core:window:default` is getters-only; `start_dragging` and `start_resize_dragging` are mutating commands. `data-tauri-drag-region` calls `startDragging` internally. Both almost certainly need explicit permissions. **Confirm at build** — a denial surfaces as a console permission error, not a silent no-op. *(Not in the J-493 scope list — added here, Joe-locked 2026-07-11.)*

---

## 2. Locked decisions (arc-local, Joe-locked 2026-07-11 — "all locked")

- **D1 — `.state-indicator` → `status-indicator`.** The `dotColor()` switch becomes a literal `STATE_COLOURS: Record<string,string>` handed to `status-bar`'s `states` prop; `isPulsing()` becomes a `PULSING_STATES` array. **All 11 client lifecycle states are enumerated explicitly** — `led`'s unknown sentinel is **black** (`#000000`) while the legacy `default:` returned `var(--t4)`, so an unenumerated state would change colour silently. No fallback branch; the sentinel is the honest signal (Rule 5 / led contract).
- **D2 — drag region without touching `core`.** A **shell wrapper** `<div class="frame-top" data-tauri-drag-region>` around `<MenuBar>`. `.frame-top{display:flex}` + the menu-bar pinned `flex:0 0 auto` so the empty strip to its right belongs to the **wrapper** (Tauri drags only when the event target itself carries the attribute; the menu `<button>` children override, so clicks still open menus). **No `dragRegion` prop on `menu-bar`** — a shell concern does not enter `core`.
- **D3 — grip wiring.** `onResizeGrip` → `getCurrentWindow().startResizeDragging(<SouthEast>)`, **lazy-imported** from `@tauri-apps/api/window` inside the handler — the exact `handleQuit` pattern, so the browser-dev preview keeps working outside Tauri.
- **D4 — `secondaryText` deferred.** The left cell holds the `status-indicator` only. The real version string is 6.1e-C's job (About reads it from the build, not hardcoded) — do not hand-roll a version tag here.
- **D5 — center pane.** `.app-body` → **`.app-center`**, the **only** scroller, holding **one muted `paragraph` placeholder** until 6.1f. **No shipped filler content** — center-only scroll is proven at verify by CDP height-injection (non-destructive, honest).
- **D6 — geometry.** `resizable: true`, default **900×600**, `minWidth: 640` / `minHeight: 400`, `center: true` — **in both conf files** (Catch 1).
- **D7 — dead CSS removed.** `img#app-logo`, `.state-indicator`, `.state-dot`, `.state-label`, `@keyframes dot-pulse`, `.button-pane`. The logo PNG **stays on disk** (`ui/client/src/assets/logo_client_64.png` — About needs it at 6.1e-C); only the import goes.
- **D8 — Quit button removed**, import removed. **`handleQuit` STAYS** — the command table, Ctrl+Q and File→Exit all resolve to it. The client's registry loses `button#quit`.
- **D9 — capabilities.** Add `core:window:allow-start-dragging` + `core:window:allow-start-resize-dragging` to `capabilities/default.json`. If the built-in permission names differ from these, **flag (Rule 6), don't guess** — the API shape (`ResizeDirection` enum vs a plain string) is likewise a build-time confirm.
- **D10 — no `core` change.** `status-bar` / `status-indicator` / `menu-bar` ship as-is. If any of them *needs* a change to mount cleanly, **stop and flag** — that is a scope breach, not a fix.

---

## 3. Files touched (expected — 5)

```
xgen-client/tauri.conf.json          (window flips)
xgen-client/cdp.dev.conf.json        (window flips — MUST mirror, Catch 1)
xgen-client/capabilities/default.json (window drag/resize permissions)
ui/client/src/app_client.svelte      (frame assembly + migration + removals)
ui/client/src/app.css                (shell chrome: flex column, center scroller, dead CSS out)
```

No `ui/core/**`, no `ui/common/**`, no `ui/sampler/**`, no Rust. Scope-clean check is part of the DoD.

---

## 4. Build steps

### Step A — window config (both files)

`tauri.conf.json` → `app.windows[0]`:
```json
{ "title": "XGen Client", "width": 900, "height": 600, "minWidth": 640, "minHeight": 400,
  "decorations": false, "resizable": true, "center": true }
```
`cdp.dev.conf.json` → the **same** object **plus** `"additionalBrowserArgs": "--remote-debugging-port=9222"` (keep it — it is the only reason this overlay exists, D-105).

### Step B — capabilities (D9)

`capabilities/default.json` `permissions` += `core:window:allow-start-dragging`, `core:window:allow-start-resize-dragging`.

### Step C — `app_client.svelte`

1. **Imports:** drop `Button` and `AppLogo`. Add `StatusBar` (`$core/components/data-independent/status-bar.svelte`) and `Paragraph` (`$core/components/data-independent/paragraph.svelte`).
2. **State map (D1)** — replace `dotColor()` / `isPulsing()`:
   ```js
   const STATE_COLOURS = {
     SETUP: 'var(--t4)',            CLOSING: 'var(--t4)',
     INITIALISING: 'var(--t3)',
     CONNECTING: 'var(--inf)',      AUTHENTICATING: 'var(--inf)',  RECONNECTING: 'var(--inf)',
     READY: 'var(--ok)',
     DEGRADED_AUTH: 'var(--pr)',    DEGRADED_FEDERATION: 'var(--pr)', DEGRADED_NODE: 'var(--pr)',
     DISCONNECTED: 'var(--err)',
   };
   const PULSING_STATES = ['INITIALISING', 'CONNECTING', 'AUTHENTICATING', 'RECONNECTING'];
   ```
3. **Grip handler (D3):**
   ```js
   async function handleResizeGrip() {
     try {
       const { getCurrentWindow } = await import('@tauri-apps/api/window');
       await getCurrentWindow().startResizeDragging('SouthEast'); // confirm enum vs string — Rule 6
     } catch (e) { console.error('Resize drag failed:', e); }
   }
   ```
4. **Markup — the BorderPane frame** (chrome outside the future `Layout` descriptor, D-107):
   ```svelte
   <div class="app-frame">
     <div class="frame-top" data-tauri-drag-region>
       <MenuBar {menus} platform={PLATFORM} onCommand={runCommand} id="app-menubar" />
     </div>

     <main class="app-center">
       <Paragraph text="No layout mounted — the center region shell lands at M-RP6.1f." id="center-placeholder" />
     </main>

     <StatusBar
       states={STATE_COLOURS}
       state={currentState.state}
       pulse={PULSING_STATES.includes(currentState.state)}
       caption={currentState.label}
       onResizeGrip={handleResizeGrip}
       id="app-statusbar"
     />
   </div>
   ```
5. **Removed:** `<img id=app-logo>`, the hand-rolled `.state-indicator` block, `<Button id="quit">`. **Kept:** `handleQuit`, the command table, the keymap registry, the `keydown` listener, the `listen`/`get_state`/`get_substitutions` `onMount` block (D8).

### Step D — `app.css` (shell chrome only, N-031)

```css
.app-frame { display: flex; flex-direction: column; height: 100%; overflow: hidden; }
.frame-top { flex: 0 0 auto; display: flex; }        /* the drag strip (D2) */
.frame-top > .menu-bar { flex: 0 0 auto; }           /* menu-bar keeps intrinsic width */
.app-center { flex: 1; min-height: 0; overflow-y: auto; padding: 12px 16px; }  /* the ONLY scroller */
.app-frame > .status-bar { flex: 0 0 auto; }
```
Delete: `.app-body`, `#core-ui-pane`, `img#app-logo`, `.state-indicator`, `.state-dot`, `.state-label`, `@keyframes dot-pulse`, `.button-pane`. Keep the `:root` accent alias block untouched.

---

## 5. Verification (REAL CLIENT 9222 ONLY — D-097)

Launch is Joe's (long-running `cargo tauri dev` cannot go through Windows-MCP): `run-client.ps1 -Debug`. Harness: `.\cdp-debug.ps1 -App client -Mode eval -Expression '…'` from the repo root. Single-line evals; **wrap eval returns as JSON objects** (PS 5.1 mangles bare strings, N-086).

### 5.1 Chat re-drives (non-destructive, Rule 2 — quote real output)

- **V1 Registry** — `ids()` sorted; `count === unique === domCount`; **0 orphans both directions**. Expect the status-bar subtree in, `button#quit` out. **Measure — do not predict (Rule 5).**
- **V2 Getter G** — `status-bar#app-statusbar` → `{leftCount:1, rightCount:1, hasGrip:true}` (D4: no `secondaryText`).
- **V3 Live state migration** — `status-indicator#app-statusbar__status-indicator` caption **equals** `currentState.label`; `led#…__led` computed background equals `STATE_COLOURS[currentState.state]`; **not black** (the D1 sentinel proof — a black led means an unenumerated state).
- **V4 Window** — `getCurrentWindow().isResizable() === true`; inner size ≈ 900×600 (**proves Catch 1: the dev overlay flipped too** — a 420×260 window here means `cdp.dev.conf.json` was missed).
- **V5 Drag region** — `.frame-top[data-tauri-drag-region]` present; the menu-bar root does **not** carry the attribute (so menu clicks are not drags).
- **V6 Center-only scroll** — inject height into `.app-center`'s child, then: `.app-center.scrollTop` moves; `document.documentElement.scrollTop === 0`; `.menu-bar` `getBoundingClientRect().top` and `.status-bar` `.bottom` are **constant** before/after the scroll. Restore the injected height.
- **V7 Menus still open** — click the File trigger → `menu#app-menubar__file` `{open:true}`; Esc → `{open:false}`, `menu-item` unregisters, 0 orphans. (Do **not** run File→Exit — see 5.2.)
- **V8 Console clean** — no Tauri permission denial (the D9 proof).

### 5.2 Clair owns (in-session / destructive — Rule 2, attributed)

- **V9 Grip resize (the real proof)** — a real pointer-down on `.sb-grip` starts an OS resize drag and the window size changes. This begins an OS drag loop and cannot be looped or stubbed (`__TAURI_INTERNALS__.invoke` is non-configurable, N-086) — **prove once**.
- **V10 Drag-to-move** — a pointer-down on the empty `.frame-top` strip moves the window.
- **V11 `vite build`** clean; module count quoted.
- **V12 Exit still works** — File→Exit **or** Ctrl+Q exits 0 (one path, once — the Quit button is gone, so this is the only exit).

---

## 6. Definition of Done

- [ ] Step A — both `tauri.conf.json` **and** `cdp.dev.conf.json` carry `resizable:true` / 900×600 / min 640×400 (Catch 1)
- [ ] Step B — capabilities carry the window drag + resize permissions (or the correct names, Rule-6 flagged if different)
- [ ] Step C — `app_client.svelte`: status-bar mounted bottom; `.state-indicator` migrated (11 states enumerated); grip wired; logo + Quit removed; `handleQuit` retained
- [ ] Step D — `app.css`: flex column, `.app-center` the only scroller, drag strip, dead CSS removed
- [ ] Scope-clean: no `ui/core/**`, no `ui/common/**`, no `ui/sampler/**`, no Rust touched (D10)
- [ ] V1–V8 green in the **real client 9222**, real CDP output quoted (Chat)
- [ ] V9–V12 green, attributed (Clair)
- [ ] Sampler catalogue registry **unchanged at 309** (this milestone adds no sampler cell)
- [ ] Client registry delta **measured**, not predicted (Rule 5)
- [ ] Records: `JOURNAL.md` J-495 · `CLAUDE.md` PLAY · `docs/ROADMAP.md` · `ui/docs/xgen-ui-notes.md` N-088 · `docs/xgen-client-frame-phase0.md` §10.4 build-note · this task → `Status: COMPLETED`

**D-074:** Clair's feat = commit 1 (code only, 5 files). Chat's doc-bridge = commit 2. Joe pushes both; Chat never pushes.

---

## 7. Deferred (D-065)

- Full-edge (four-side) invisible resize borders — SE-grip is v1.
- `secondaryText` / version tag → 6.1e-C (About reads the real build version).
- The center region shell + `Layout` descriptor + selection bus → 6.1f.
- Help→About + `dialog` `core` + the hi-res logo's new home → 6.1e-C.
- The **node** app's own frame (menu-bar / status-bar, `grip=false` if non-resizable) — its own milestone.

---

*End of M-RP6.1e-B runbook.*
