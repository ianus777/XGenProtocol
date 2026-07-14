<script>
  import { onMount, onDestroy } from 'svelte';
  import MenuBar from '$core/components/data-independent/menu-bar.svelte';
  import StatusBar from '$core/components/data-independent/status-bar.svelte';
  import Shelf from '$core/components/data-independent/shelf.svelte';
  import RegionShell from '$core/components/layout/region-shell.svelte';
  import AboutDialog from './about-dialog.svelte';
  import UistateSaveDialog from './uistate-save-dialog.svelte';
  import UistateLoadDialog from './uistate-load-dialog.svelte';
  import PluginsDialog from './plugins-dialog.svelte';
  import { uiStateStore } from './uistate.svelte';
  import { loadLayout, widgetRegistry, REGION_TITLES, DEFAULT_LAYOUT } from './layout-default';
  import { migrateLayout } from '$core/components/layout/resolve';
  import { resizeSplit, foldLeaf, move } from '$core/components/layout/mutate';
  import { substitutions } from '$common/components/processor/store.svelte';
  // The self-state store (M-RP6.1g, D3) — ONE channel, TWO views: the shell WRITES it below (the existing
  // state listen + get_state + a once get_self_state invoke), and BOTH the status-bar (here) AND the
  // self-panel widget READ it. STATE_COLOURS/PULSING_STATES relocated here (they were shell-local; the
  // widget needs them and cannot receive shell props — W-3).
  import { selfState, STATE_COLOURS, PULSING_STATES } from '$common/stores/self-state.svelte';
  // The selection bus (M-RP6.1f, D3): its first real writer is now the self-panel widget (D5). This
  // side-effect import keeps the DEV __XGEN_SEL__ handle installed for CDP; the shell is the bus's host,
  // its readers (R8 inspector / entity-context-menu) wire in at 6.1h+.
  import '$common/stores/selection.svelte';
  import { KeymapRegistry } from '$common/keymap/registry';
  import { accelerator } from '$common/keymap/accelerator';

  // Connection state now lives in the self-state store (D3 — one source, two views); no shell-local
  // mirror. The store seeds itself to INITIALISING until the first Tauri event arrives.
  let unlisten;

  // About dialog (M-RP6.1e-C3): the get_about_info payload (build metadata + paths), fetched once
  // on mount (static per session), and the open state flipped by Help→About.
  let aboutInfo = $state(null);
  let aboutOpen = $state(false);

  // UI-state Save/Load dialogs (M-RP6.1k). The diskette/load shelf faces open these; the store
  // PERSISTS to xgen-client_uistate.json (Rust get/set_ui_state) and named states carry window geometry.
  let saveOpen = $state(false);
  let loadOpen = $state(false);

  // Plugin list dialog (M-RP6.1l). The gear shelf face opens this; it mounts the `plugin-list` widget
  // ($common) — a READ-ONLY list of the client's own compiled plugins (there is no node-plugin verb).
  let pluginsOpen = $state(false);

  // Centre region layout (M-RP6.1f). Seeded by `await loadLayout()` on mount (D2 — async so M-RP7.3 is a
  // one-line swap to invoke('get_layout')). Shell-local this milestone (D7 — the shell is the only
  // consumer; the widget-manager/shelf promotion to a $common store is reserved, not built).
  let layout = $state(null);

  // DEV-only CDP handle (N-024 idiom) so the verify pass can drive the drop / tabs / mismatch paths
  // (§5.3): push a test layout, read region-shell's getter G. Dead-code-eliminated in a release build.
  if (import.meta.env.DEV && typeof window !== 'undefined') {
    window.__XGEN_LAYOUT__ = {
      get current() { return layout; },
      set(l) { layout = l; },
      // M-RP7.3 — drive the pure algebra before any gesture code exists (§4). `move` relocates a region to
      // an edge of a target; `fold` sets/clears a leaf's fold axis. Both delegate to the shell handlers so
      // the reactive reassignment fires. Dead-code-eliminated in a release build, as `set` already is.
      move(sourceId, targetId, edge) { handleMove(sourceId, targetId, edge); },
      fold(regionId, collapsed) { handleFold(regionId, collapsed); },
    };
  }

  // ── Fold (M-RP7.1b, D6) ───────────────────────────────────────────────────────────────────────
  // The tree surgery moved into the pure algebra (`foldLeaf`, mutate.ts, M-RP7.3 L2) — the shell keeps NO
  // tree code, only the reactive reassignment that triggers the shell to re-resolve. Held in MEMORY this leg
  // — the session feeder that PERSISTS `session.layout` is M-RP7.5 (D6). `foldLeaf` preserves the unfold
  // contract exactly (undefined ⇒ delete the key), so an unfold never persists `collapsed: undefined`.
  function handleFold(regionId, collapsed) {
    if (!layout) return;
    layout = foldLeaf(layout, regionId, collapsed);
  }

  // ── Splitter resize (M-RP7.2/7.3, N-120) ──────────────────────────────────────────────────────
  // The seam gesture reports the split's `path` and the pair's two DESCRIPTOR indices (`aIdx`/`bIdx`,
  // N-120 — never resolved positions) with the boundary `fraction` on release; `resizeSplit` writes the new
  // INTEGER weights (L2/L3). IN MEMORY only — `session.layout` has no writer until M-RP7.5.
  function handleResize(path, aIdx, bIdx, fraction) {
    if (!layout) return;
    layout = resizeSplit(layout, path, aIdx, bIdx, fraction);
  }

  // ── Move (M-RP7.3, L3 — drag-to-dock, no gesture yet) ──────────────────────────────────────────
  // Relocate a region to an edge of a target (`move`, mutate.ts). No pointer wiring this milestone — the
  // DEV handle drives it so the tree surgery is proven before M-RP7.4 gives it a gesture. IN MEMORY only.
  function handleMove(sourceId, targetId, edge) {
    if (!layout) return;
    layout = move(layout, sourceId, targetId, edge);
  }

  // ── Keymap wiring (M-RP6.1d — the 6.1c-deferred shell half) ──────────────────────────────
  // Windows is the only shipped target; the keymap objects take `platform` as a parameter (no
  // `navigator` read), so the shell names it here. mac stays correct-by-construction for a future build.
  const PLATFORM = 'win';

  // The command TABLE — the single source of truth an accelerator OR a File→Exit menu-item both
  // resolve to. `exitCommand` REUSES the exact Tauri close the existing Quit/Shut-Down button wires
  // (invoke('quit')) — no new close call invented (runbook §2.4, Rule 5).
  const commandTable = {
    'app.exit': handleQuit,
    'help.about': () => (aboutOpen = true),
    // M-RP6.1k — the diskette/load faces resolve here.
    'uistate.save': () => (saveOpen = true),
    'uistate.load': () => (loadOpen = true),
    // M-RP6.1l — the gear resolves here. This entry existing is what lets the gear enable (below);
    // the 6.1j countdown is now discharged — no shelf face is disabled.
    'widget.manager': () => (pluginsOpen = true),
  };
  function runCommand(commandId) {
    commandTable[commandId]?.();
  }

  // The keymap registry singleton: Ctrl+Q → app.exit. One global keydown → resolve → run.
  const registry = new KeymapRegistry();
  registry.register(accelerator('Ctrl+Q'), 'app.exit');

  function onKeydown(e) {
    const commandId = registry.resolve(e, PLATFORM);
    if (commandId) {
      e.preventDefault();
      runCommand(commandId);
    }
  }

  // The top-pane menu-bar: File → Exit carries the SAME "app.exit" command AND the Ctrl+Q
  // accelerator (which renders the trailing hint). One Accelerator, one command — Ctrl+Q and
  // File→Exit are one truth.
  const menus = [
    {
      label: 'File',
      items: [{ label: 'Exit', accelerator: accelerator('Ctrl+Q'), command: 'app.exit' }],
    },
    // Help → About (M-RP6.1e-C3). NO accelerator — F1 conventionally means Help *contents*.
    {
      label: 'Help',
      items: [{ label: 'About', command: 'help.about' }],
    },
  ];

  // ── Shelves (M-RP6.1j — mount the shipped shelf, J-508; M-RP6.1k — UI-state faces; M-RP6.1l — gear) ──
  // The bottom (system) strip's faces — shell-local (D5, the layout-default D7 precedent; the shell is
  // the only consumer, no $common store). The 6.1j countdown is now DISCHARGED (M-RP6.1l):
  //   6.1k — diskette/load ENABLED; their `uistate.*` commands exist in the table.
  //   6.1l (this milestone) — gear ENABLED; `widget.manager` now exists in the table. NO face is disabled.
  // RENAMED layout.save/layout.load → uistate.save/uistate.load (D-114): the store is NOT a layout — it
  // holds geometry, and will hold shelf/theme/room; `layout.*` would be a lie by M-RP6.2. There is no
  // uistate.saveAs — one diskette, one dialog, two outcomes (overwrite the active state, or a new name).
  const SHELF_BOTTOM = [
    { icon: 'gear', label: 'Plugins', command: 'widget.manager', disabled: false },
    { icon: 'diskette', label: 'Save UI state', command: 'uistate.save', disabled: false },
    { icon: 'load', label: 'Load UI state', command: 'uistate.load', disabled: false },
  ];

  onMount(async () => {
    window.addEventListener('keydown', onKeydown);

    // Seed the centre layout (D2). Not Tauri, never throws, so it runs OUTSIDE the try that swallows the
    // no-Tauri (browser dev preview) case — the grid must render even without a backend.
    layout = await loadLayout();

    try {
      const { listen } = await import('@tauri-apps/api/event');
      const { invoke } = await import('@tauri-apps/api/core');

      // Subscribe to live state changes — write the SAME store the status-bar and the self-panel read
      // (D3). No new emit channel: this is the existing `xgen-client-state-changed` (D1).
      unlisten = await listen('xgen-client-state-changed', (event) => {
        selfState.setConnection(event.payload);
      });

      // Fetch the current state immediately — handles the case where the startup
      // sequence ran before this listener was registered (pre-listener race, already solved).
      selfState.setConnection(await invoke('get_state'));

      // M-RP6.1g — self-identity (D1/D2). Static per session (no in-app registration exists), so fetched
      // ONCE here (the get_about_info shape). Inside the same try so the browser-dev/no-Tauri path keeps
      // working. Unregistered → registered:false + a real keypair-derived XGID (rendered honestly, D6).
      selfState.setIdentity(await invoke('get_self_state'));

      // M-RP4.2 — hydrate the user-owned substitution pairs from the client
      // TOML ([substitutions] rules). The store parses + validates (Tier-2);
      // every processor-host then sources from it.
      substitutions.setRules(await invoke('get_substitutions'));

      // M-RP6.1e-C3 — About data (build metadata + Rust/Tauri/Svelte versions + paths). Static
      // per session, so fetched once here; the dialog reads it synchronously.
      aboutInfo = await invoke('get_about_info');

      // M-RP6.1k Leg B — hydrate the persistent UI-state store from disk (get_ui_state). The
      // Save/Load dialogs read it reactively; a corrupt/absent store leaves it empty (N-095).
      await uiStateStore.hydrate();
    } catch (_) {
      // Running outside Tauri (browser dev preview) — state stays at placeholder.
    }
  });

  onDestroy(() => {
    window.removeEventListener('keydown', onKeydown);
    if (unlisten) unlisten();
  });

  async function handleQuit() {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('quit');
    } catch (e) {
      console.error('Quit failed:', e);
    }
  }

  // M-RP6.1e-C3 (F1) — open the About website in the OS browser. The `link` atomic stays dumb;
  // the consumer wires this. preventDefault stops the <a target=_blank> in-app-webview path;
  // the lazy import keeps the browser-dev preview (no Tauri) working — the handleQuit pattern.
  async function handleAboutLink(e) {
    e?.preventDefault?.();
    try {
      const { openUrl } = await import('@tauri-apps/plugin-opener');
      await openUrl(aboutInfo?.common?.link ?? 'https://www.alchemydump.com');
    } catch (err) {
      console.error('Open link failed:', err);
    }
  }

  // ── UI-state Save/Load (M-RP6.1k) ───────────────────────────────────────────────────────────
  // A named UI state carries the ARRANGEMENT: layout + window geometry (§4.2). The dialogs never reach
  // into `layout` themselves (the about/loadLayout seam shape). Geometry is RUST's (physical px, typed):
  // SAVE fetches the live rect and carries it OPAQUELY in the state (the shell never interprets it);
  // LOAD hands it back to Rust, which re-applies it through the same D-115 clamp. No-Tauri (browser dev)
  // → layout only. $state.snapshot detaches the tree so the stored copy is not a live proxy.
  async function tauriInvoke(cmd, args) {
    const { invoke } = await import('@tauri-apps/api/core');
    return invoke(cmd, args);
  }
  async function handleUistateSave(name) {
    let geometry;
    try {
      geometry = await tauriInvoke('get_window_geometry');
    } catch (_) {
      // browser-dev / no-Tauri → save layout only.
    }
    uiStateStore.save(name, { layout: $state.snapshot(layout), ...(geometry ? { geometry } : {}) });
  }
  async function handleUistateLoad(id) {
    const s = uiStateStore.load(id);
    // Guard: only apply a real layout — never assign undefined/null (that unmounts region-shell →
    // blank centre, the J-499/N-095 failure). A state saved without a layout key is left as-is.
    // A named state saved before M-RP7.1b carries v1/v2 boolean `collapsed`; migrate it (never null;
    // malformed → DEFAULT, D-115) so an older workspace loads with the correct explicit fold directions.
    if (s?.layout) layout = migrateLayout(s.layout, DEFAULT_LAYOUT);
    // Restore the named state's window rect through the same clamp (Rust owns geometry's meaning).
    if (s?.geometry) {
      try {
        await tauriInvoke('apply_window_geometry', { geom: s.geometry });
      } catch (_) {
        // browser-dev / no-Tauri → skip the window move.
      }
    }
  }

  // STATE_COLOURS / PULSING_STATES relocated to `$common/stores/self-state.svelte` (D3) — the widget
  // needs them and cannot receive shell props, so the shell now IMPORTS the same map it reads for the
  // status-bar (one map, two views). The relocation is a pure move (V7 — colour byte-identical).

  // ── Resize-grip wiring ─────────────────────────────────────────────────────────────────────
  // The status-bar's SE grip exposes `onResizeGrip?` and drives Tauri's startResizeDragging
  // (SE = width+height). The window is now OS-decorated (native title bar + native edge resize),
  // so the grip is a supplementary corner affordance — a conventional explicit resize handle at
  // the bottom-right — not the sole resize path it was under the original frameless design.
  // Lazy-imported inside the handler — the exact handleQuit pattern, so the browser-dev preview
  // (no Tauri) keeps working. 'SouthEast' is a valid ResizeDirection string literal (confirmed
  // against @tauri-apps/api window.d.ts — no enum import needed).
  async function handleResizeGrip() {
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      await getCurrentWindow().startResizeDragging('SouthEast');
    } catch (e) {
      console.error('Resize drag failed:', e);
    }
  }
</script>

<!-- Fixed BorderPane frame (M-RP6.1e-B): the full-width menu-bar and the bottom status-bar are
  frame chrome, OUTSIDE the future Layout descriptor (region-dock model §2 — File→Exit can never
  be docked away), below the OS-native title bar. The center is the ONLY scroller (D5) and holds
  a placeholder until 6.1f. Window move + min/max/close come from the native title bar (Joe's
  pivot away from the original frameless design), so there is no in-app drag region. -->
<div class="app-frame">
  <MenuBar {menus} platform={PLATFORM} onCommand={runCommand} id="app-menubar" />

  <!-- Top shelf (M-RP6.1j): user favourites. Mounts EMPTY (D1) → [data-empty] → the skin collapses it
    to height 0 (no stray hairline under the menu-bar). Registered (N-053) so pinning is a one-line
    population later (surfaces §6 ④, gates nothing). aria-label distinguishes the two toolbars. -->
  <Shelf position="top" items={[]} ariaLabel="Favourites" onCommand={runCommand} id="app-shelf-top" />

  <!-- The centre region shell (M-RP6.1f): renderer A reads the Layout descriptor and tiles placeholder
    leaves. It FILLS .app-center (no whole-grid scroll, D5) — each leaf owns its own scroll. -->
  <main class="app-center">
    {#if layout}
      <RegionShell {layout} widgets={widgetRegistry} titles={REGION_TITLES} onFold={handleFold} onResize={handleResize} id="region-root" />
    {/if}
  </main>

  <!-- Bottom shelf (M-RP6.1j / M-RP6.1k / M-RP6.1l): system commands, above the status-bar. All three
    faces are now ENABLED (their commands exist in the table) — the 6.1j countdown is discharged, no
    shelf face is disabled. -->
  <Shelf position="bottom" items={SHELF_BOTTOM} ariaLabel="System" onCommand={runCommand} id="app-shelf-bottom" />

  <!-- The connection light + caption migrate here from the retired hand-rolled .state-indicator
    (D1); the SE grip is a supplementary corner resize affordance (D3). -->
  <StatusBar
    states={STATE_COLOURS}
    state={selfState.connection.state}
    pulse={PULSING_STATES.includes(selfState.connection.state)}
    caption={selfState.connection.label}
    onResizeGrip={handleResizeGrip}
    id="app-statusbar"
  />

  <!-- The About modal (M-RP6.1e-C3). A top-layer <dialog> — DOM position doesn't affect stacking;
    opened by Help→About (help.about → aboutOpen). Always mounted (closed = display:none). -->
  <AboutDialog bind:open={aboutOpen} info={aboutInfo} onOpenLink={handleAboutLink} />

  <!-- UI-state Save/Load modals (M-RP6.1k). Same always-mounted top-layer posture as About; opened by
    the diskette/load shelf faces (uistate.save / uistate.load). -->
  <UistateSaveDialog bind:open={saveOpen} onSave={handleUistateSave} />
  <UistateLoadDialog bind:open={loadOpen} onLoad={handleUistateLoad} />

  <!-- Plugin list modal (M-RP6.1l). Same always-mounted top-layer posture; opened by the gear shelf
    face (widget.manager). A READ-ONLY list of the client's own compiled plugins. -->
  <PluginsDialog bind:open={pluginsOpen} />
</div>
