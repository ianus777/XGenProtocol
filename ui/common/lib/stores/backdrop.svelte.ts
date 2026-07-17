// backdrop.svelte.ts — the grid-backdrop setting store (M-RP-SETTINGS Leg C, D-B / B2). ONE value the
// grid-plate paints. THREE participants, ONE channel: the settings component (grid-plate-settings, $common)
// WRITES it, the grid-plate widget ($common) READS it and branches its render, and the SHELL mirrors it into
// region-shell's `backgroundLive` + persists it (a uistate session key). This store is the ONLY channel a
// $common writer and a $common reader can share — W-3 forbids either $common file importing a shell store,
// and the shell can read a $common store but cannot mediate a $common→$common link (the self-state /
// installed precedent, N-096).
//
// v1 holds ONE boolean (B2): whether the backdrop shows its pattern (the promoted dev raster) or a plain
// fill. RESERVE NOTHING beyond it — the full static / generative / data-driven, base-vs-stack menu is
// M-RP-BACKDROP, not this leg. Default true = the raster shows, so first launch is byte-identical to the
// pre-Leg-C inert plate (no visual regression).
//
// A `.svelte.ts` module so its module-level `$state` participates in Svelte 5 reactivity (the
// self-state.svelte / selection.svelte precedent).

let _pattern = $state(true);

export const backdrop = {
  /** Whether the grid backdrop paints its pattern. Reactive — grid-plate reads this and the shell mirrors it
   *  into `backgroundLive`; the value the whole B2 mechanic turns on. */
  get pattern(): boolean {
    return _pattern;
  },
  /** Written by grid-plate-settings (the control), and seeded by the shell on boot from the persisted
   *  session key (before loadLayout, so the choice paints on relaunch). */
  setPattern(on: boolean): void {
    _pattern = on;
  },
};

// DEV-only CDP handle (mirrors __XGEN_SELF__ / __XGEN_UISTATE__, N-024), so the verify pass can read + drive
// the value out-of-band. Dead-code-eliminated in a production build.
if (import.meta.env.DEV && typeof window !== 'undefined') {
  (window as unknown as { __XGEN_BACKDROP__?: unknown }).__XGEN_BACKDROP__ = backdrop;
}
