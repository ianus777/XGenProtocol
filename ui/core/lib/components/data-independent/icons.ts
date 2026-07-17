// icons.ts — the bundled glyph registry for `icon` (M-RP6.1a). A plain map of
// `name -> d` (or `d[]` for multi-path glyphs), co-located in `core` with
// label/image/led and imported at build time (tree-shaken, no runtime fetch).
//
// Every glyph is authored on a 24x24 viewBox and is FILL-based (Material-style
// solid shapes) so `fill` tinting works (D5). A registry entry is a single `d`
// string OR a `d[]`; `icon.svelte` renders `{#each paths as d}<path d={d} />`
// (no `{@html}` — the shape geometry only, XSS-free by construction, N-032).
//
// Seed set is 3 demonstrative glyphs (D-065) — the mechanism is what M-RP6.1a
// proves; frame consumers (menu-item / status-bar resize-grip) add their own
// real Material/Lucide-fill paths as they land. Source `.svg` design files live
// alongside in `ui/assets/icons/` for provenance.
//
// Stroke-based (Lucide-style outline) glyphs are deferred until a glyph needs
// them (D-065) — that would add a `stroke` variant then.

export type IconPath = string | string[];

export const icons: Record<string, IconPath> = {
  // Downward chevron/caret — the disclosure/expand glyph (combobox/section reuse `--tri`
  // today; a real glyph home starts here).
  'caret-down': 'M6 9l6 6 6-6z',
  // A small filled dot — a generic bullet / presence pip.
  dot: 'M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8z',
  // A filled square — a neutral placeholder / stop glyph.
  square: 'M5 5h14v14H5z',

  // ── Frame-shelf glyphs (M-RP6.1i). Material Icons (filled), Apache-2.0. Path `d` copied
  //    byte-for-byte from the real 24×24 source SVG (saved alongside in ui/assets/icons/ for
  //    provenance, D-108); the original's fill="none" bounding rect is dropped so the glyph
  //    is COLOUR-FREE geometry only — colour arrives from the tint (D-110). The generator +
  //    icons.manifest.json land at M-RP-ICON-ADOPT; provenance is written now so that is a
  //    move, not an archaeology dig.

  // gear — settings command surface (widget.manager, 6.1l).
  // Material name: settings · Apache-2.0
  // https://github.com/google/material-design-icons/blob/master/src/action/settings/materialicons/24px.svg
  gear:
    'M19.14,12.94c0.04-0.3,0.06-0.61,0.06-0.94c0-0.32-0.02-0.64-0.07-0.94l2.03-1.58c0.18-0.14,0.23-0.41,0.12-0.61 l-1.92-3.32c-0.12-0.22-0.37-0.29-0.59-0.22l-2.39,0.96c-0.5-0.38-1.03-0.7-1.62-0.94L14.4,2.81c-0.04-0.24-0.24-0.41-0.48-0.41 h-3.84c-0.24,0-0.43,0.17-0.47,0.41L9.25,5.35C8.66,5.59,8.12,5.92,7.63,6.29L5.24,5.33c-0.22-0.08-0.47,0-0.59,0.22L2.74,8.87 C2.62,9.08,2.66,9.34,2.86,9.48l2.03,1.58C4.84,11.36,4.8,11.69,4.8,12s0.02,0.64,0.07,0.94l-2.03,1.58 c-0.18,0.14-0.23,0.41-0.12,0.61l1.92,3.32c0.12,0.22,0.37,0.29,0.59,0.22l2.39-0.96c0.5,0.38,1.03,0.7,1.62,0.94l0.36,2.54 c0.05,0.24,0.24,0.41,0.48,0.41h3.84c0.24,0,0.44-0.17,0.47-0.41l0.36-2.54c0.59-0.24,1.13-0.56,1.62-0.94l2.39,0.96 c0.22,0.08,0.47,0,0.59-0.22l1.92-3.32c0.12-0.22,0.07-0.47-0.12-0.61L19.14,12.94z M12,15.6c-1.98,0-3.6-1.62-3.6-3.6 s1.62-3.6,3.6-3.6s3.6,1.62,3.6,3.6S13.98,15.6,12,15.6z',

  // diskette — save command (layout.save / layout.saveAs, 6.1k).
  // Material name: save · Apache-2.0
  // https://github.com/google/material-design-icons/blob/master/src/content/save/materialicons/24px.svg
  diskette:
    'M17 3H5c-1.11 0-2 .9-2 2v14c0 1.1.89 2 2 2h14c1.1 0 2-.9 2-2V7l-4-4zm-5 16c-1.66 0-3-1.34-3-3s1.34-3 3-3 3 1.34 3 3-1.34 3-3 3zm3-10H5V5h10v4z',

  // load — open/load command (layout.load, 6.1k).
  // Material name: folder_open · Apache-2.0
  // https://github.com/google/material-design-icons/blob/master/src/file/folder_open/materialicons/24px.svg
  load:
    'M20 6h-8l-2-2H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2zm0 12H4V8h16v10z',

  // lock — the grid-lock command (layout.lock, M-RP7.6). PROVISIONAL shape (D5) — the real glyph
  // (and any lock/unlock swap) is a skin decision, deferred to M-RP-SKIN; do not tune it here.
  // Material name: lock · Apache-2.0 · the source fill="none" bounding rect is dropped → colour-free
  // geometry only (D-110/D-108), colour arrives from the tint.
  // https://github.com/google/material-design-icons/blob/master/src/action/lock/materialicons/24px.svg
  lock:
    'M18 8h-1V6c0-2.76-2.24-5-5-5S7 3.24 7 6v2H6c-1.1 0-2 .9-2 2v10c0 1.1.9 2 2 2h12c1.1 0 2-.9 2-2V10c0-1.1-.9-2-2-2zm-6 9c-1.1 0-2-.9-2-2s.9-2 2-2 2 .9 2 2-.9 2-2 2zm3.1-9H8.9V6c0-1.71 1.39-3.1 3.1-3.1 1.71 0 3.1 1.39 3.1 3.1v2z',

  // ── Plugin-manager glyphs (M-RP-SETTINGS Leg B). Material Icons (filled), Apache-2.0. Path `d` copied
  //    byte-for-byte from the real 24×24 source SVG (saved alongside in ui/assets/icons/ for provenance,
  //    D-108); the source's fill="none" bounding rect (`M0 0h24v24H0z`) is DROPPED so the glyph is
  //    COLOUR-FREE geometry only — colour arrives from the tint / currentColor (D-110).

  // info — the [info] action / "little i" (Joe). Material name: info · action/info
  info:
    'M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-6h2v6zm0-8h-2V7h2v2z',
  // delete — the [uninstall] action / recycle-bin motif (Joe). Material name: delete · action/delete
  delete:
    'M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z',
  // toggle-on / toggle-off — the [disable] action, a switch (Joe: "rather some switch icon than lock").
  // The row swaps them by the live disabled flag: on = enabled/mounted, off = disabled. toggle/toggle_on
  // + toggle/toggle_off.
  'toggle-on':
    'M17 7H7c-2.76 0-5 2.24-5 5s2.24 5 5 5h10c2.76 0 5-2.24 5-5s-2.24-5-5-5zm0 8c-1.66 0-3-1.34-3-3s1.34-3 3-3 3 1.34 3 3-1.34 3-3 3z',
  'toggle-off':
    'M17 7H7c-2.76 0-5 2.24-5 5s2.24 5 5 5h10c2.76 0 5-2.24 5-5s-2.24-5-5-5zM7 15c-1.66 0-3-1.34-3-3s1.34-3 3-3 3 1.34 3 3-1.34 3-3 3z',
  // chevron-left — the round Back button (Joe: "icon <"). Material name: chevron_left · navigation/chevron_left
  'chevron-left': 'M15.41 7.41L14 6l-6 6 6 6 1.41-1.41L10.83 12z',
  // close — the round modal-close button (Joe: "icon ×"). Material name: close · navigation/close
  close:
    'M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z',

  // ── Per-plugin identity glyphs (M-RP-SETTINGS Leg B). A plugin's leading icon (descriptor `icon`). Same
  //    Material/Apache-2.0/colour-free discipline; each chosen to match the plugin's purpose (Joe: "look for
  //    plugin name"). PROVISIONAL — Joe may retune the mapping at M-RP-SKIN.

  // person — Self Panel. social/person
  person:
    'M12 12c2.21 0 4-1.79 4-4s-1.79-4-4-4-4 1.79-4 4 1.79 4 4 4zm0 2c-2.67 0-8 1.34-8 4v2h16v-2c0-2.66-5.33-4-8-4z',
  // search — Inspector Panel (a magnifier = inspect). action/search
  search:
    'M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z',
  // extension — Plugin List (the puzzle-piece = plugin/extension). action/extension
  extension:
    'M20.5 11H19V7c0-1.1-.9-2-2-2h-4V3.5C13 2.12 11.88 1 10.5 1S8 2.12 8 3.5V5H4c-1.1 0-1.99.9-1.99 2v3.8H3.5c1.49 0 2.7 1.21 2.7 2.7s-1.21 2.7-2.7 2.7H2V20c0 1.1.9 2 2 2h3.8v-1.5c0-1.49 1.21-2.7 2.7-2.7 1.49 0 2.7 1.21 2.7 2.7V22H17c1.1 0 2-.9 2-2v-4h1.5c1.38 0 2.5-1.12 2.5-2.5S21.88 11 20.5 11z',
  // wallpaper — Grid Backdrop. device/wallpaper
  wallpaper:
    'M4 4h7V2H4c-1.1 0-2 .9-2 2v7h2V4zm6 9l-4 5h12l-3-4-2.03 2.71L10 13zm7-4.5c0-.83-.67-1.5-1.5-1.5S14 7.67 14 8.5s.67 1.5 1.5 1.5S17 9.33 17 8.5zM20 2h-7v2h7v7h2V4c0-1.1-.9-2-2-2zm0 18h-7v2h7c1.1 0 2-.9 2-2v-7h-2v7zM4 13H2v7c0 1.1.9 2 2 2h7v-2H4v-7z',
  // signal — Connection Stats (cellular-alt bars). device/signal_cellular_alt
  signal: 'M17 4h3v16h-3zM5 14h3v6H5zm6-5h3v11h-3z',
};
