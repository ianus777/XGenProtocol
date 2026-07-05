// Content-derived colour seed (N-064 lineage). Pure, DOM-free, framework-agnostic —
// the substrate `chip` (M-RP2.27) founded and `entity-avatar` (M-RP5.0, the first
// dd-atomic) shares. Hashes an arbitrary key to a hue, then paints a fixed MUTED S/L
// band at that hue: fill light, text dark, border mid — all the same hue, never white,
// never bright. Deterministic + shell-independent: the same key reads the same colour
// under gold or blue (no `--accent` dependency).
//
// Factored OUT of `chip` at M-RP5.0 so `chip` and `entity-avatar` share one source.
// The returned strings are BYTE-IDENTICAL to `chip`'s pre-refactor inline values —
// changing the band here changes both consumers (a deliberate single point of truth).

export interface Seed {
  hue: number; // 0..359, deterministic per key
  bg: string; // fill — light band
  fg: string; // text/glyph — dark band
  bd: string; // border — mid band
}

/**
 * Deterministic string hash → hue 0..359 (djb2-ish, `| 0` 32-bit wrap).
 * Content-derived, stable per key; identical to `chip`'s original inline hash.
 */
function hashHue(key: string): number {
  let h = 5381;
  for (let i = 0; i < key.length; i++) h = ((h << 5) + h + key.charCodeAt(i)) | 0;
  return ((h % 360) + 360) % 360;
}

/**
 * Map a key to its muted-band colour triple. The S/L values match `chip`'s original
 * fixed band exactly (fill 45%/82%, text 55%/30%, border 40%/80%).
 */
export function seedColour(key: string): Seed {
  const hue = hashHue(key);
  return {
    hue,
    bg: `hsl(${hue} 45% 82%)`,
    fg: `hsl(${hue} 55% 30%)`,
    bd: `hsl(${hue} 40% 80%)`,
  };
}
