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
};
