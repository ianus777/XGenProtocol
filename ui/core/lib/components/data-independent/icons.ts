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
};
