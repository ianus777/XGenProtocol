<script>
  import { onMount, onDestroy, untrack } from 'svelte';
  import MenuBar from '$core/components/data-independent/menu-bar.svelte';
  import StatusBar from '$core/components/data-independent/status-bar.svelte';
  import Shelf from '$core/components/data-independent/shelf.svelte';
  import RegionShell from '$core/components/layout/region-shell.svelte';
  import AboutDialog from './about-dialog.svelte';
  import UistateSaveDialog from './uistate-save-dialog.svelte';
  import UistateLoadDialog from './uistate-load-dialog.svelte';
  import SettingsDialog from './settings-dialog.svelte';
  import { uiStateStore } from './uistate.svelte';
  import { loadLayout, buildWidgetRegistry, buildBgWidgets, buildTitles, DEFAULT_LAYOUT, DEFAULT_BACKGROUND } from './layout-default';
  import { migrateLayout } from '$core/components/layout/resolve';
  import { resizeSplit, foldLeaf, move, insertLeaf, removeRegion } from '$core/components/layout/mutate';
  // The runtime custom-plugin lifecycle (M-RP-CONNSTATS). `installed` (a $common store) owns the installed
  // SET; the shell wraps install/uninstall to ALSO inject/remove the layout leaf and persist (§4.7). The
  // region widgetRegistry / bgWidgets / titles DERIVE reactively from `installed.active` (below).
  import { installed } from '$common/plugins/installed.svelte';
  import { AVAILABLE_CUSTOM } from '$common/plugins/registry';
  import { substitutions } from '$common/components/processor/store.svelte';
  // The self-state store (M-RP6.1g, D3) — ONE channel, TWO views: the shell WRITES it below (the existing
  // state listen + get_state + a once get_self_state invoke), and BOTH the status-bar (here) AND the
  // self-panel widget READ it. STATE_COLOURS/PULSING_STATES relocated here (they were shell-local; the
  // widget needs them and cannot receive shell props — W-3).
  import { selfState, STATE_COLOURS, PULSING_STATES } from '$common/stores/self-state.svelte';
  // The known-Space store (M-RP6.2, D1) — ONE source, TWO readers: the spaces-panel (R1) lists Spaces, the
  // rooms-panel (R2) reads the selected Space's embedded rooms. The shell WRITES it once on mount below
  // (`get_spaces`); both region widgets READ it (they can't receive shell props — W-3). No widget-register
  // line: `widgetRegistry` DERIVES the swap from the CLIENT_PLUGINS descriptors (M-RP6.1l).
  import { spacesState } from '$common/stores/spaces-state.svelte';
  // The live inbound-event store (M-RP6.3 Leg B) — the M-RP6.6 ingest deferral closing. The shell listens
  // to `xgen-event` once and writes it; R5 reads it at Leg C. No second emit channel (D1/D-067).
  import { ingest } from '$common/stores/ingest.svelte';
  // The connection-gap episode store (M-RP6.3 Leg C2, C-10). A PURE $state store (the self-state precedent);
  // the shell feeds it from the SAME two live surfaces the status feed reads — see the feeder $effect below.
  import { gaps } from '$common/stores/gaps.svelte';
  // The grid-backdrop setting store (M-RP-SETTINGS Leg C, B2). ONE value the grid-plate paints. The shell
  // reads it here to mirror into region-shell's `backgroundLive` + persist it (a session key), seeds it on
  // boot from the persisted key; the $common grid-plate-settings component is what WRITES it (W-3/N-096 —
  // the $common plate and its $common settings component share this store, never a shell intermediary).
  import { backdrop } from '$common/stores/backdrop.svelte';
  // The selection bus (M-RP6.1f, D3): its first real writer is now the self-panel widget (D5). This
  // side-effect import keeps the DEV __XGEN_SEL__ handle installed for CDP; the shell is the bus's host,
  // its readers (R8 inspector / entity-context-menu) wire in at 6.1h+.
  // (Leg D2: the side-effect import became a NAMED one — the shell now READS the bus to feed the room
  // latch. The DEV __XGEN_SEL__ handle still installs, since importing the module is what installs it.)
  import { selection } from '$common/stores/selection.svelte';
  // M-RP6.3 Leg D2 — the shared room latch (§2.1) and the outbound echo store (§9.11.2). Both are PURE
  // $state stores fed by the shell (the gaps precedent): the shell drives `roomLatch.note` from the bus and
  // injects the guarded send transport into `echo`. They live in $common because R5 renders echoes and R6
  // creates them, both `$common` region widgets that receive only a `regionId` (W-3/N-096).
  import { roomLatch } from '$common/stores/room-latch.svelte';
  import { echo } from '$common/stores/echo-state.svelte';
  // M-RP-MEMBERS Leg B — the R7 members store. A PURE $state store fed by the shell (the roomLatch/gaps
  // feeder shape): the $effect below watches `roomLatch.effectiveSpaceId` and drives the fill
  // (fill_space_records + get_address_book), since W-3 forbids the $common store importing `invoke`.
  import { addressBook } from '$common/stores/address-book.svelte';
  import { KeymapRegistry } from '$common/keymap/registry';
  import { accelerator } from '$common/keymap/accelerator';

  // Connection state now lives in the self-state store (D3 — one source, two views); no shell-local
  // mirror. The store seeds itself to INITIALISING until the first Tauri event arrives.
  let unlisten;
  // M-RP6.3 Leg B — the live-ingest listener handle, torn down beside `unlisten`.
  let unlistenEvent;

  // M-RP6.6 Leg C / M-RP6.3 Leg A — the live-stats poll handle. Both published resident surfaces
  // (`get_conn_stats` bytes+RTT and `get_resident_status` attempt/countdown/terminal) change over time,
  // unlike the static identity block, so ONE interval reads both. Cleared on destroy.
  let livePoll;

  // About dialog (M-RP6.1e-C3): the get_about_info payload (build metadata + paths), fetched once
  // on mount (static per session), and the open state flipped by Help→About.
  let aboutInfo = $state(null);
  let aboutOpen = $state(false);

  // UI-state Save/Load dialogs (M-RP6.1k). The diskette/load shelf faces open these; the store
  // PERSISTS to xgen-client_uistate.json (Rust get/set_ui_state) and named states carry window geometry.
  let saveOpen = $state(false);
  let loadOpen = $state(false);

  // Settings modal (M-RP-SETTINGS Leg A). ONE Discord-shaped modal (sidebar + content pane); the plugin
  // manager is its Plugins section (the read-only `plugin-list`, absorbed from the retired plugins-dialog).
  // Two entry points: File ▸ Settings → default section; the gear (widget.manager) → the Plugins section.
  // `settingsSection` is the deep-link target the entry point sets (null = default).
  let settingsOpen = $state(false);
  let settingsSection = $state(null);

  // Centre region layout (M-RP6.1f). Seeded by `await loadLayout()` on mount (D2 — async so M-RP7.3 is a
  // one-line swap to invoke('get_layout')). Shell-local this milestone (D7 — the shell is the only
  // consumer; the widget-manager/shelf promotion to a $common store is reserved, not built).
  let layout = $state(null);

  // The RUNTIME-reactive registries (M-RP-CONNSTATS, D1; M-RP-SETTINGS Leg B). `installed.mounted` = the
  // always-present system rows + the installed customs that are NOT disabled; install/uninstall AND
  // disable/enable recompute these, so a custom's tile widget / backdrop / title appear/disappear live.
  // (plugin-list reads `installed.active` directly — the LISTED set, which KEEPS disabled customs, shown
  // disabled.) Computed HERE (not in the $common installed store) because the builders close over the
  // shell-local `RegionPlaceholder` (W-3 — a $common store may not import a shell component).
  const mountedPlugins = $derived(installed.mounted);
  const widgetRegistry = $derived(buildWidgetRegistry(mountedPlugins));
  const bgWidgets = $derived(buildBgWidgets(mountedPlugins));
  const titles = $derived(buildTitles(mountedPlugins));

  // Grid lock (M-RP7.6). One boolean, seeded from session.locked after hydrate() (default false). When
  // true: the fold/resize/move handlers early-return (the LOAD-BEARING guard — an access rule is only real
  // if its callers enforce it) AND the tile grips / fold buttons / seams go element-absent (the honesty
  // layer). The lock face's `pressed` tracks it; the toggle persists via setSessionLocked.
  let locked = $state(false);

  // Grid backdrop mounts (M-RP-PLATE, D3). Seeded to the ONE inert system plate; a `$state` (not the raw
  // DEFAULT_BACKGROUND literal) only so the DEV bridge can drive the W-13 drop leg (V5). The `backgroundLive`
  // literal is now BOUND to the backdrop setting (M-RP-SETTINGS Leg C, at the RegionShell mount below).
  let background = $state(DEFAULT_BACKGROUND);

  // M-RP-SETTINGS Leg C (B2) — persist the backdrop setting. The $common grid-plate-settings component
  // WRITES the $common backdrop store; the shell observes it here (W-3 — a $common component cannot import
  // the shell-local uistate store) and persists via a per-key session write (N-107). `backdrop.pattern` is
  // the ONLY dependency; the persist call is `untrack`ed because setSessionBackdrop READS `_store.session`
  // (to merge, N-107) and WRITES it — an un-untracked read inside this effect would make `_store.session` a
  // dependency and the write would self-invalidate → effect_update_depth_exceeded (the render scheduler
  // halts: state keeps mutating, the DOM freezes). The other setSessionX axes avoid this by persisting from
  // EVENT handlers, not an effect; the backdrop's writer is a $common component, so an effect is the only
  // channel — hence untrack. The first run is SKIPPED so the boot-seed does not re-persist an on-disk value.
  let backdropReady = false;
  $effect(() => {
    const on = backdrop.pattern; // the sole tracked dependency
    if (!backdropReady) {
      backdropReady = true;
      return;
    }
    untrack(() => uiStateStore.setSessionBackdrop(on));
  });

  // M-RP6.3 Leg C2 (C-10) — feed the connection-gap store. This is the ONE writer path: the reactive $effect
  // calls gaps.apply, and the DEV __XGEN_GAPS__.apply IS the same function (single-writer rule, the J-548
  // tickNow lesson). It observes BOTH surfaces the status feed rides — `selfState.connection` (event-timed,
  // so the episode START is accurate) and `selfState.resident` (the 2 s poll, refreshing the countdown
  // numbers). App-lifetime, because the shell is always mounted: an outage during a widget unmount is still
  // captured (§9.10.2 — the whole reason the episodes live in a store, not the widget). untrack because
  // gaps.apply reads/writes its own $state; an un-untracked read would make that state a dependency and
  // self-invalidate (N-136 — the backdrop effect above documents the same trap). `paused` is a verify gate.
  $effect(() => {
    const conn = selfState.connection; // track (event-timed lifecycle)
    const res = selfState.resident; // track (2 s poll)
    if (!gaps.paused) untrack(() => gaps.apply(conn, res));
  });

  // M-RP6.3 Leg D2 (§2.1) — feed the SHARED room latch. This is the ONE writer path, and the DEV
  // __XGEN_ROOM__.note IS the same function (single-writer rule, the J-548 tickNow lesson). It lives in the
  // SHELL, not in R5, because R5 and R6 are separate region widgets in separate tiles: a latch owned by R5
  // would forget the room the moment R5 was folded, taking the composer's target with it (the C-10 argument
  // exactly). `note` writes `_latched` and never reads it, so there is no self-invalidating read-modify-write
  // (N-136) — untrack is belt-and-braces, matching the gaps feeder above.
  $effect(() => {
    const sel = selection.current; // the sole tracked dependency
    untrack(() => roomLatch.note(sel));
  });

  // M-RP-MEMBERS Leg B (§3) — feed the R7 address-book store from the room latch. The ONE writer path (the
  // roomLatch feeder above, and the gaps feeder, are the shape): on a scope change, reset to inflight, then
  // fill_space_records + get_address_book, then setResult — or setFailed on reject. The store's own
  // late-guard (§3.5) discards a resolve whose spaceId no longer matches the current scope, so switching
  // rooms twice quickly never renders Space A's roster under Space B's heading. No timer, no polling (L6/§3):
  // the Rust `tokio::time::timeout` bounds the fill, so the invoke always resolves or rejects. untrack because
  // loadMembers reads/writes the store's own $state (the roomLatch/gaps feeder precedent, N-136).
  $effect(() => {
    const sid = roomLatch.effectiveSpaceId; // the sole tracked dependency
    untrack(() => loadMembers(sid));
  });
  async function loadMembers(sid) {
    if (sid == null) {
      addressBook.reset(); // state ① — no scope; no fill fired
      return;
    }
    addressBook.setInflight(sid);
    try {
      // fill_space_records returns FillMembersOutcome { fill, roster } (Leg A-quater); the roster renders, the
      // fill persists the address-book cache on disk (which get_address_book then reads back for the name
      // resolution) AND carries the not_found ids (③) the render rules read (M-RP-IDENTITY-RESOLUTION §4/§5).
      // tauriInvoke lazy-imports invoke (the browser-dev/no-Tauri path rejects → setFailed).
      const outcome = await tauriInvoke('fill_space_records', { spaceId: sid });
      const book = await tauriInvoke('get_address_book');
      addressBook.setResult(sid, outcome, book);
    } catch (_) {
      // ③ inflight → ④ failed (or ⑤ when the connection is down — the widget picks ⑤ by the connection led,
      // not the phase, L5). A stale rejection is discarded by the store's late-guard.
      addressBook.setFailed(sid);
    }
  }

  // M-RP-LIVEFEED-REFRESH Leg A — the live membership router. Called from the `xgen-event` listener beside
  // `ingest.push` (:552, untouched): every inbound Event flows through here, and only `membership.*` deltas
  // that change WHO IS IN THE ROOM NOW reach the address-book roster. The three non-obvious store rules
  // (R1/R3/R4) live in the setters (runbook §3); this function owns R2 — resolving WHICH identity each
  // event is about, which is NOT uniformly `sender`. The webview branches on `payload.type` holding the
  // WIRE STRING (`Event` is `#[serde(rename = "type")]`, wire.rs:476), never a Rust variant name. The
  // scope guard and the null-roster/idempotency rules are the store's; the router just resolves + dispatches.
  function routeMembershipEvent(payload) {
    const type = payload?.type;
    if (typeof type !== 'string' || !type.startsWith('membership.')) return;
    switch (type) {
      // join / leave: the subject signs their own event, so it IS `sender` (parent §6).
      case 'membership.join': {
        const subject = payload.sender;
        if (typeof subject !== 'string' || subject === '') return;
        // buildEntry (runbook §4): REAL wire fields only. The `unresolved` marker is stamped by addMember,
        // not here. `role: ''` is honest-empty — no wire field carries authority, and 'member' would be an
        // invented claim (D-065). `joined_at` is the real wire timestamp; `invited_by` is not carried.
        addressBook.addMember(payload.space_id, {
          identity_id: subject,
          role: '',
          joined_at: payload.timestamp,
          invited_by: null,
        });
        return;
      }
      case 'membership.leave': {
        const subject = payload.sender;
        if (typeof subject !== 'string' || subject === '') return;
        addressBook.removeMember(payload.space_id, subject);
        return;
      }
      // kick / ban / node_eject: `sender` is the MODERATOR (kick/ban) or the NODE (node_eject); the removed
      // person is `content.target_identity` (parent §6-i). That field is a CONVENTION, not a type (only
      // MembershipMuteContent declares it, wire.rs:712-713), so read it defensively and DROP on absence —
      // R2 forbids a `sender` fallback, which would remove the moderator instead of the target.
      case 'membership.kick':
      case 'membership.ban':
      case 'membership.node_eject': {
        const subject = payload?.content?.target_identity;
        if (typeof subject !== 'string' || subject === '') return;
        addressBook.removeMember(payload.space_id, subject);
        return;
      }
      // v1: ignored, deliberately — none of the three changes who is in the room NOW (parent §6). Listed
      // as explicit cases so the omission reads as a choice, not an oversight.
      case 'membership.invite': // an invite is not a join
      case 'membership.mute': // a mute does not remove anyone
      case 'membership.node_unban': // an unban permits a future rejoin; it is not itself a join
        return;
      // an unrecognised `membership.*` string (a membership event this leg predates) — return.
      default:
        return;
    }
  }

  // M-RP6.3 Leg D2 — inject the send transport into the echo store (W-3: `$common` imports no shell dep, and
  // there are ZERO @tauri-apps imports under ui/common — so the composer reaches `send_message` ONLY through
  // this seam).
  //
  // 🔒 THIS IS THE LIFECYCLE GUARD (§9.11.8). Without it a send during an outage takes the D1 bounded path
  // and resolves ~19 s later (measured 19 155 ms = a 3.1 s failed re-anchor + the 16.0 s bound) — correct,
  // but a very long time to watch a row say "Sending…". The guard short-circuits to `failed` immediately.
  //
  // ⚠️ IT RETURNS `failed`, NOT `timed_out`, AND THAT IS EXACT: nothing was written, so "never reached the
  // wire" is literally true — and `failed` is the ONE status the user may retry freely (lock #7), so the
  // sentence is preserved, visible and recoverable rather than lost in a textarea.
  //
  // ⚠️ AND THE GUARD IS THE NICETY, NOT THE GUARANTEE (§9.11.4): a half-open socket can report READY while
  // the peer is gone, so a send CAN still reach the bounded path. D1's `SEND_QUEUE_TIMEOUT` is what makes
  // that terminate. Do not read this guard as a promise.
  // ONE definition, named, so the DEV bridge's restore reinstates THE SAME transport rather than a second
  // copy of the guard (a duplicated guard is the D-067 drift this milestone's own central decision exists to
  // avoid — and a drifted copy would be reinstated by every verify run).
  async function guardedSend(spaceId, roomId, text) {
    if (selfState.connection?.state !== 'READY') {
      return { event_id: null, status: 'failed', code: null, reason: 'not connected' };
    }
    return sendMessage(spaceId, roomId, text);
  }
  echo.setTransport(guardedSend);

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
      // M-RP-PLATE (V5) — feed the background socket a bad/empty mount list to exercise the W-13 drop
      // (an unknown widgetId lowers `backgroundMountCount` to 0, no crash); restore to bring the plate back.
      get background() { return background; },
      setBackground(bg) { background = bg; },
    };

    // M-RP-CONNSTATS — the custom-plugin lifecycle DEV bridge (§4.7). The SHELL wrappers, so install/uninstall
    // move BOTH the set and the layout leaf (and persist). No install UI exists yet (the plugins dialog stays
    // read-only — M-RP6.1m); this drives the mechanism for verify. Dead-code-eliminated in a release build.
    window.__XGEN_PLUGINS__ = {
      install(id) { handleInstall(id); },
      uninstall(id) { handleUninstall(id); },
      // M-RP-SETTINGS Leg B — drive disable/enable without clicking (the real row button also runs this exact
      // handler; the bridge is a CDP convenience). One toggle, mirroring the row's Disable/Enable button.
      toggleDisabled(id) { handlePluginToggleDisabled(id); },
      get installed() { return installed.ids; },
      get disabled() { return installed.disabledIds; },
      get available() { return AVAILABLE_CUSTOM.map((p) => p.id); },
    };

    // M-RP6.3 Leg A — the resident DEV bridge. `status` reads the published surface the poll writes;
    // `reconnect()` drives the SAME guarded verb the [Reconnect] element will, and returns Rust's honest
    // boolean (false when the resident is not terminal — which is itself the guard's proof).
    // Dead-code-eliminated in a release build.
    window.__XGEN_RESIDENT__ = {
      get status() { return selfState.resident; },
      reconnect() { return resumeResident('dev-bridge'); },
    };

    // M-RP6.3 Leg B — the message-plane DEV bridge. `send()` drives the real `send_message` command
    // (the same one the Leg-D composer will call) and returns the honest four-way outcome; `ingest`
    // exposes the live inbound store. Leg B ships the SEAM — there is no composer and no rendered
    // stream yet, so this is how the plane is verified without inventing UI that belongs to Legs C/D.
    // 🔒 KEPT AT LEG D2, RETIRES AT LEG D3 CLOSE (§2.2). No user-facing impact in either direction — it is
    // DEV-guarded and verified absent from a production bundle. Kept on the internal axis and stated as
    // such: it is the ONLY way to drive the send path WITHOUT the composer, i.e. to tell whether a fault is
    // in the new UI or underneath it. Removing it in the milestone that ADDS that UI would remove the
    // control exactly when it becomes useful. ⚠️ Its repo reference count is 1 — its own definition —
    // because its callers are CDP evals living outside the tree: retiring on a reference count would delete
    // something whose users the count cannot see.
    window.__XGEN_SEND__ = {
      send(spaceId, roomId, text) { return sendMessage(spaceId, roomId, text); },
      get ingest() { return { received: ingest.received, dropped: ingest.dropped, latest: ingest.latest }; },
      clear() { ingest.clear(); },
    };

    // M-RP6.3 Leg D2 — the composer/echo DEV bridge. `send()` goes through the ECHO STORE (guarded
    // transport included), which is what the composer does, so the bridge and the UI exercise ONE path.
    // `inject()` resolves a row from a synthetic outcome WITHOUT touching the network — the only way to
    // drive all four statuses on demand (V4), since `rejected`/`timed_out` cannot be summoned from a healthy
    // local node. It calls the SAME `send` first, so the row it resolves is a real row.
    window.__XGEN_ECHO_BRIDGE__ = {
      send(spaceId, roomId, text) { return echo.send(spaceId, roomId, text); },
      get all() { return echo.all.map((e) => ({ ...e })); },
      get wired() { return echo.wired; },
      retry(localId) { return echo.retry(localId); },
      clear() { echo.clear(); },
      // Room latch — read the shared latch and drive its single writer.
      get room() {
        return {
          latchedRoomId: roomLatch.latchedRoomId,
          effectiveRoomId: roomLatch.effectiveRoomId,
          effectiveSpaceId: roomLatch.effectiveSpaceId,
          canSend: roomLatch.canSend,
        };
      },
      note(sel) { roomLatch.note(sel); },
      clearRoom() { roomLatch.clear(); },
      // Swap the transport for a stub returning a chosen outcome, then restore. Used ONLY at verify.
      stubTransport(outcome) {
        echo.setTransport(async () => outcome);
      },
      restoreTransport() { echo.setTransport(guardedSend); },
    };
  }

  // ── Fold (M-RP7.1b, D6) ───────────────────────────────────────────────────────────────────────
  // The tree surgery moved into the pure algebra (`foldLeaf`, mutate.ts, M-RP7.3 L2) — the shell keeps NO
  // tree code, only the reactive reassignment that re-resolves the shell, then FEEDS the new arrangement to
  // the session store (M-RP7.5 Leg B — `session.layout` now persists across relaunch). `foldLeaf` preserves
  // the unfold contract exactly (undefined ⇒ delete the key), so an unfold never persists `collapsed: undefined`.
  function handleFold(regionId, collapsed) {
    if (locked) return; // M-RP7.6 — the load-bearing refusal (also reached via __XGEN_LAYOUT__.fold, V1)
    if (!layout) return;
    layout = foldLeaf(layout, regionId, collapsed);
    uiStateStore.setSessionLayout($state.snapshot(layout));
  }

  // ── Splitter resize (M-RP7.2/7.3, N-120) ──────────────────────────────────────────────────────
  // The seam gesture reports the split's `path` and the pair's two DESCRIPTOR indices (`aIdx`/`bIdx`,
  // N-120 — never resolved positions) with the boundary `fraction` on release; `resizeSplit` writes the new
  // INTEGER weights (L2/L3), then FEEDS the session store (M-RP7.5 Leg B — persisted, debounced).
  function handleResize(path, aIdx, bIdx, fraction) {
    if (locked) return; // M-RP7.6 — belt to the dead-seam braces; a locked seam has no listener anyway
    if (!layout) return;
    layout = resizeSplit(layout, path, aIdx, bIdx, fraction);
    uiStateStore.setSessionLayout($state.snapshot(layout));
  }

  // ── Move (M-RP7.3, L3 — drag-to-dock, no gesture yet) ──────────────────────────────────────────
  // Relocate a region to an edge of a target (`move`, mutate.ts). Driven by the drag-to-dock gesture
  // (M-RP7.4) and the DEV handle; the reactive reassignment re-resolves the shell, then FEEDS the session
  // store (M-RP7.5 Leg B — persisted, debounced).
  function handleMove(sourceId, targetId, edge) {
    if (locked) return; // M-RP7.6 — the backstop for the transitively-inert bands (also reached via V1)
    if (!layout) return;
    layout = move(layout, sourceId, targetId, edge);
    uiStateStore.setSessionLayout($state.snapshot(layout));
  }

  // ── Custom-plugin install / uninstall (M-RP-CONNSTATS, D2/§4.7) ─────────────────────────────────
  // The store owns the SET; the shell wraps it to ALSO move the layout leaf and persist. Install: register
  // (reactive registry gains the widget) + inject a leaf at a defined target (PROVISIONAL — the user drags it
  // after, M-RP7.4). Uninstall: remove the leaf BEFORE deregistering, so no frame renders an unresolved leaf;
  // `removeRegion` collapse-degenerates so the freed space is absorbed (no blank centre). The installed-set
  // persists per-device (session key); a reload re-registers it BEFORE loadLayout (onMount, §4.7).
  const DEFAULT_INSTALL_TARGET = 'inspector'; // PROVISIONAL — a defined dock spot; the user relocates it.
  const DEFAULT_INSTALL_EDGE = 'bottom';

  function handleInstall(id) {
    if (installed.isInstalled(id)) return;
    const desc = AVAILABLE_CUSTOM.find((p) => p.id === id);
    if (!desc) return;
    installed.install(id); // reactive: widgetRegistry/titles gain it BEFORE the leaf resolves
    if (desc.surface === 'region' && desc.regionId && layout) {
      layout = insertLeaf(layout, desc.regionId, DEFAULT_INSTALL_TARGET, DEFAULT_INSTALL_EDGE);
      uiStateStore.setSessionLayout($state.snapshot(layout));
    }
    uiStateStore.setSessionInstalled(installed.ids);
  }

  function handleUninstall(id) {
    if (!installed.isInstalled(id)) return;
    const desc = AVAILABLE_CUSTOM.find((p) => p.id === id);
    if (desc?.surface === 'region' && desc.regionId && layout) {
      layout = removeRegion(layout, desc.regionId); // remove the leaf FIRST — never render an unresolved leaf
      uiStateStore.setSessionLayout($state.snapshot(layout));
    }
    installed.uninstall(id); // then deregister — the widget leaves the reactive registry
    uiStateStore.setSessionInstalled(installed.ids);
    // uninstall clears the disabled flag inside the store; persist the (now-shorter) disabled set too.
    uiStateStore.setSessionDisabled(installed.disabledIds);
  }

  // ── Disable / enable a custom plugin (M-RP-SETTINGS Leg B, §3.2) ────────────────────────────────
  // A SECOND runtime axis from uninstall: a disabled plugin stays INSTALLED (still listed, still persisted)
  // but its widget UNMOUNTS. Reuses the D-119 leaf primitives — no new algebra. Disable: remove the leaf
  // FIRST (never render an unresolved leaf), then mark disabled (mounted excludes it). Enable: mark enabled,
  // then re-inject the leaf at the default dock (its prior position is not restored — the user re-drags it,
  // M-RP7.4; matches install). One toggle (the row's Disable/Enable is one button, one bit). Persists
  // session.disabled (+ session.layout when a leaf moved). Only customs disable — a system plugin never
  // reaches here (the row button is greyed, W-13).
  function handlePluginToggleDisabled(id) {
    const desc = AVAILABLE_CUSTOM.find((p) => p.id === id);
    if (!desc || desc.kind !== 'custom' || !installed.isInstalled(id)) return;
    if (installed.isDisabled(id)) {
      installed.enable(id); // reactive: mounted regains it
      if (desc.surface === 'region' && desc.regionId && layout) {
        layout = insertLeaf(layout, desc.regionId, DEFAULT_INSTALL_TARGET, DEFAULT_INSTALL_EDGE);
        uiStateStore.setSessionLayout($state.snapshot(layout));
      }
    } else {
      if (desc.surface === 'region' && desc.regionId && layout) {
        layout = removeRegion(layout, desc.regionId); // leaf out BEFORE the mark — never an unresolved leaf
        uiStateStore.setSessionLayout($state.snapshot(layout));
      }
      installed.disable(id); // reactive: mounted drops it (belt to the leaf-removal braces)
    }
    uiStateStore.setSessionDisabled(installed.disabledIds);
  }

  // ── The plugin-list action seam (M-RP-SETTINGS Leg B, §3.3) ─────────────────────────────────────
  // settings-dialog handles `info` itself (its content-pane drill-in); it forwards the shell-level verbs
  // here. `settings` is greyed for every plugin this leg (no settingsComponent shipped), so a guarded row
  // button never fires it — it lands here only once a plugin ships one (Leg C), a no-op until then.
  function handlePluginAction(id, verb) {
    if (verb === 'uninstall') handleUninstall(id);
    else if (verb === 'disable') handlePluginToggleDisabled(id);
  }

  // ── Revert UI (M-RP7.5, Leg D) — renew the grid from the last autosave ──────────────────────────
  // Re-reads `session.layout` from disk via the SHIPPED loadLayout() (migrates a past-build tree, NEVER
  // returns null — N-095) and reassigns the grid. A REFRESH, not an undo: the feeder writes continuously,
  // so "last autosave" ≈ the live grid — this is "renew UI from disk". Zero Rust. No setSessionLayout
  // after: reloading and re-writing the same disk state is a no-op (runbook §6 D1).
  async function handleRevertUi() {
    layout = await loadLayout();
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
    // M-RP-SETTINGS Leg A — the gear opens Settings LANDED on the Plugins section (the deep-link).
    // (M-RP6.1l's `widget.manager` → pluginsOpen is retired; the gear now opens the one Settings modal.)
    'widget.manager': () => { settingsSection = 'plugins'; settingsOpen = true; },
    // M-RP-SETTINGS Leg A — File ▸ Settings opens the SAME modal on its default (first) section.
    'settings.open': () => { settingsSection = null; settingsOpen = true; },
    // M-RP7.5 Leg C — File ▸ Restart: bounce the process; the fresh boot restores the autosaved grid.
    'app.restart': handleRestart,
    // M-RP7.5 Leg D — standalone "renew from autosave" (loadLayout re-read), no bounce. The command is
    // LIVE (real handler) awaiting its own interactive element (Joe: placed elsewhere, not the File menu).
    'layout.revert': handleRevertUi,
    // M-RP7.6 — the grid lock. ONE boolean, ONE command (two commands would be two sources of truth for
    // one bit, D-067). Toggles + persists; the lock face's `pressed` reflects it, the handlers refuse.
    'layout.lock': () => {
      locked = !locked;
      uiStateStore.setSessionLocked(locked);
    },
    // M-RP6.3 Leg A (D4) — [Reconnect]: re-arm a resident that parked at the retry cap. The command is
    // LIVE (real handler, real Rust verb) awaiting its own interactive element, which lands in Leg C
    // with Ms Design (§5 — appearance is not this leg's). The `layout.revert` precedent: a built verb
    // with no element yet is ABSENT, which is the honest state; what this project refuses is a PAINTED
    // element that does nothing (J-500 / 6.1j).
    'resident.reconnect': () => resumeResident('command'),
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
      items: [
        // M-RP-SETTINGS Leg A — File ▸ Settings opens the Settings modal on its default section (the gear
        // is the other entry point, landing on Plugins). Placed ABOVE Restart → File = Settings · Restart ·
        // —— · Exit. No accelerator (Ctrl+, is a possible future skin/UX call, Joe's).
        { label: 'Settings', command: 'settings.open' },
        // M-RP7.5 Leg C — File ▸ Restart: bounce the process; the fresh boot's loadLayout() re-reads the
        // session autosave, so the arranged grid comes back (the "revert" is implicit in the reload). Joe
        // owns the glyph; this is the wiring. Standalone Revert (layout.revert) lives elsewhere, not here.
        { label: 'Restart', command: 'app.restart' },
        { separator: true }, // horizontal divider between the restart action and the destructive Exit
        { label: 'Exit', accelerator: accelerator('Ctrl+Q'), command: 'app.exit' },
      ],
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
  // $derived (M-RP7.6): the 4th face is the grid lock, whose `pressed` tracks `locked` reactively — a plain
  // const array would snapshot `locked` once and never repaint the latch. It is ENABLED (its command exists),
  // so no shelf face is disabled — the 6.1j countdown stays discharged.
  const SHELF_BOTTOM = $derived([
    { icon: 'gear', label: 'Plugins', command: 'widget.manager', disabled: false },
    { icon: 'diskette', label: 'Save UI state', command: 'uistate.save', disabled: false },
    { icon: 'load', label: 'Load UI state', command: 'uistate.load', disabled: false },
    { icon: 'lock', label: 'Lock layout', command: 'layout.lock', pressed: locked, disabled: false },
  ]);

  onMount(async () => {
    window.addEventListener('keydown', onKeydown);

    // Boot order (load-bearing — M-RP6.1k/7.6/M-RP-CONNSTATS). Hydrate the persistent UI-state store FIRST: it
    // is idempotent and no-Tauri-safe (its own try/catch), so it runs OUTSIDE the Tauri try below. From its
    // `session` we seed the installed custom-plugin set and the grid lock BEFORE the layout resolves — a
    // persisted custom leaf must find its registered widget (installed.hydrate → the reactive widgetRegistry
    // gains it) instead of W-13-dropping at resolveLayout (§4.7, D3).
    await uiStateStore.hydrate();
    installed.hydrate(uiStateStore.session()?.installed ?? []);
    // M-RP-SETTINGS Leg B — seed the disabled set AFTER installed, BEFORE loadLayout: a persisted disabled
    // custom must be excluded from `mounted` before the layout resolves (hydrateDisabled filters to
    // installed-and-available, so a stale id cannot mark a phantom).
    installed.hydrateDisabled(uiStateStore.session()?.disabled ?? []);
    locked = uiStateStore.session()?.locked ?? false;
    // M-RP-SETTINGS Leg C (B2) — seed the $common backdrop store from the persisted key BEFORE loadLayout, so
    // the plate paints the persisted choice on first paint (the hydrateDisabled precedent). Default true =
    // the raster shows (no visual regression from the pre-Leg-C plate). This runs before the persist effect's
    // first (skipped) pass, so no spurious write.
    backdrop.setPattern(uiStateStore.session()?.backdrop ?? true);

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

      // M-RP6.3 Leg B — live ingest. The resident drain reads the socket and emits every inbound
      // protocol Event here; the store carries them verbatim for R5 to project at Leg C.
      unlistenEvent = await listen('xgen-event', (event) => {
        ingest.push(event.payload);
        routeMembershipEvent(event.payload); // Leg A — live members router; ingest.push above stays R5's untouched store
      });

      // Fetch the current state immediately — handles the case where the startup
      // sequence ran before this listener was registered (pre-listener race, already solved).
      selfState.setConnection(await invoke('get_state'));

      // M-RP6.1g — self-identity (D1/D2). Static per session (no in-app registration exists), so fetched
      // ONCE here (the get_about_info shape). Inside the same try so the browser-dev/no-Tauri path keeps
      // working. Unregistered → registered:false + a real keypair-derived XGID (rendered honestly, D6).
      selfState.setIdentity(await invoke('get_self_state'));

      // M-RP6.2 — the known-Space tree (D1). Static per session (no live push until the resident, M-RP6.6),
      // so fetched ONCE here (the get_self_state shape). Rooms ride embedded, so R2 needs no second fetch.
      // Unregistered → get_spaces returns [] (honest empty; R1 shows "No spaces yet").
      spacesState.setSpaces(await invoke('get_spaces'));

      // M-RP4.2 — hydrate the user-owned substitution pairs from the client
      // TOML ([substitutions] rules). The store parses + validates (Tier-2);
      // every processor-host then sources from it.
      substitutions.setRules(await invoke('get_substitutions'));

      // M-RP-PROCESSOR-WIRE Leg A — the WRITE half of that same seam, filled here beside the read it
      // mirrors. `substitutions-editor` reaches the client through D-120's generic, prop-less settings
      // mount, so there is no `onApply` prop to pass; and the widget lives in `$common` and cannot
      // import `invoke` itself (W-3). Without this line its Apply would type-check, render and
      // silently never persist. The sampler leaves the seam null and keeps the prop (D-097 / W-8).
      substitutions.setPersist((rules) => invoke('set_substitutions', { rules }));

      // M-RP6.1e-C3 — About data (build metadata + Rust/Tauri/Svelte versions + paths). Static
      // per session, so fetched once here; the dialog reads it synchronously.
      aboutInfo = await invoke('get_about_info');

      // M-RP6.6 Leg C — live connection traffic (observed bytes in/out + last ping/pong RTT). Unlike the
      // static identity block, this CHANGES, so it is POLLED (the store slot renders absent-not-zero when a
      // metric has no value yet, D4). One immediate read, then a light interval; cleared on destroy.
      //
      // M-RP6.3 Leg A — the SECOND published surface rides the SAME interval: attempt count, the cap,
      // the countdown to the next dial, and the terminal flag (Phase-0 §0 G-3 — the lifecycle enum has
      // no payload to carry them). No second poll and no new push channel: a second channel would be a
      // second surface (D-067). Read in ONE try so a transient failure keeps BOTH last snapshots rather
      // than leaving the two halves of one moment disagreeing.
      const pollLiveStats = async () => {
        try {
          selfState.setTraffic(await invoke('get_conn_stats'));
          selfState.setResident(await invoke('get_resident_status'));
        } catch (_) {
          /* transient invoke failure — keep the last snapshots */
        }
      };
      await pollLiveStats();
      livePoll = setInterval(pollLiveStats, 2000);

      // M-RP6.3 Leg A (D4) — AUTO-RESUME. Window focus and network-return call the SAME guarded verb
      // the [Reconnect] action does; Rust ignores it unless the resident actually parked at the cap, so
      // a focus storm can never reset a live backoff. Detection lives here, not in Rust, deliberately:
      // the WebView already gives `focus`/`online` for free, where Rust-side link detection would mean
      // a new dependency and a per-platform surface.
      window.addEventListener('focus', onWindowFocus);
      window.addEventListener('online', onNetworkOnline);

      // (The UI-state store hydrate + the installed-set/grid-lock seed moved ABOVE loadLayout — they must run
      // before the layout resolves, M-RP-CONNSTATS §4.7. hydrate() is idempotent, so no double-read here.)
    } catch (_) {
      // Running outside Tauri (browser dev preview) — state stays at placeholder.
    }
  });

  onDestroy(() => {
    window.removeEventListener('keydown', onKeydown);
    if (unlisten) unlisten();
    if (unlistenEvent) unlistenEvent();
    if (livePoll) clearInterval(livePoll);
    window.removeEventListener('focus', onWindowFocus);
    window.removeEventListener('online', onNetworkOnline);
  });

  // ── Resident resume (M-RP6.3 Leg A, D4) ────────────────────────────────────────────────────────
  // The one verb behind THREE callers: the [Reconnect] action, window focus, and network-return. Rust
  // guards it (a resume is a no-op unless the resident is terminal) and returns whether it actually
  // signalled — so this never reports a reconnect that did not happen (D6's binding rule applied to an
  // action rather than a message). Lazy import keeps the browser-dev preview working (the handleQuit
  // pattern). Returns the honest boolean for the DEV bridge and for Leg C's button.
  //
  // `source` is passed through to the Rust log because leaving the terminal state has three possible
  // causes and the Leg-A verify hit one it could not attribute. Named handlers (not inline arrows) so
  // removeEventListener gets the SAME reference — an arrow here would leak both listeners on destroy.
  async function resumeResident(source) {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      return await invoke('resume_resident', { source });
    } catch (_) {
      return false; // no-Tauri (browser dev preview) — nothing to resume
    }
  }
  const onWindowFocus = () => resumeResident('focus');
  const onNetworkOnline = () => resumeResident('online');

  // ── Send (M-RP6.3 Leg B) ──────────────────────────────────────────────────────────
  // Multiplexes onto the resident socket (D1). Returns the Rust `SendOutcome` VERBATIM — the honest
  // four-way status (`accepted` / `rejected` / `timed_out` / `failed`), never a boolean. A caller that
  // reduced this to true/false would be the exact D6 violation: `timed_out` means the node MAY hold the
  // event, so it is neither sent nor failed. The Leg-D composer renders per-message state from this.
  async function sendMessage(spaceId, roomId, text) {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      return await invoke('send_message', { spaceId, roomId, text });
    } catch (e) {
      return { event_id: null, status: 'failed', code: null, reason: String(e) };
    }
  }

  async function handleQuit() {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('quit');
    } catch (e) {
      console.error('Quit failed:', e);
    }
  }

  // ── Restart (M-RP7.5, Leg C) — File ▸ Restart: bounce the process ────────────────────────────────
  // The fresh boot's loadLayout() re-reads the session autosave (Legs A/B), so the arranged grid comes
  // back — "revert + restart" with the revert implicit in the reload. `tauri-plugin-process` and the
  // `process:default` grant already ship (since M1) → ZERO Rust. Lazy import keeps the browser-dev
  // preview (no Tauri) working — the handleQuit pattern.
  async function handleRestart() {
    try {
      const { relaunch } = await import('@tauri-apps/plugin-process');
      await relaunch();
    } catch (e) {
      console.error('Restart failed:', e);
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
      <RegionShell {layout} widgets={widgetRegistry} {titles} {background} {bgWidgets} backgroundLive={backdrop.pattern} onFold={handleFold} onResize={handleResize} onMove={handleMove} {locked} id="region-root" />
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

  <!-- Settings modal (M-RP-SETTINGS Leg A). Same always-mounted top-layer posture; opened by File ▸
    Settings (default section) and the gear (Plugins section). Discord-shaped: a category sidebar + a
    content pane. Hosts the read-only plugin-list (Plugins section) and the reused About content (About
    section). `aboutInfo` is the same payload the Help ▸ About modal reads (S-2 — one fetch, two views). -->
  <SettingsDialog bind:open={settingsOpen} section={settingsSection} info={aboutInfo} onOpenLink={handleAboutLink} onPluginAction={handlePluginAction} />
</div>
