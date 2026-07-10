<script lang="ts">
  // icon — data-independent, DISPLAY-kind (N-032): an inline-SVG, tintable, token-scaled
  // square UI glyph. The 28th `core` component and the first frame prerequisite of the
  // M-RP6.1 client-UI-frame arc (D-107). A display-di sibling of label/paragraph/image/led
  // — value-carrying but READ-ONLY (no bind-in, no event-out). Atomic (N-020): the root IS
  // an inline <svg>.
  //
  // WHY ITS OWN COMPONENT (not folded into `image`, D-096 cleared on TWO axes):
  //   - value-type — `image` carries a `src` REFERENCE (where a raster lives); `icon` carries
  //     a SHAPE DEFINITION (the SVG path `d` geometry itself, not a URL). A new value-type.
  //   - surface — `image` is raster content (intrinsic size, border-radius, alt required);
  //     `icon` is a token-scaled, square, tintable glyph.
  // Mirrors JavaFX's own split: ImageView is raster-only (≈ image); SVG is a separate
  // SVGPath Shape node, fillable/tintable (≈ icon).
  //
  // STRUCTURAL NOVELTY: icon is the FIRST SVG-rooted `core` component (every prior atomic is
  // HTML-rooted — label/paragraph/led <span|label|p>, image <img>). On an SVG element
  // `node.className` is a read-only SVGAnimatedString, so the envelope substrate was
  // generalised to attribute-based class-stamping (`setAttribute('class', …)`) — SVG-safe and
  // behaviorally identical for the 30+ HTML-rooted components (N-083).
  //
  // API (D1–D4): `name` keys the bundled glyph registry (icons.ts, `d`-strings); optional raw
  // `path` override (string | string[]) WINS over `name` when both set (the one-off escape
  // hatch). A registry entry is a `d` string or `d[]`; render {#each paths}<path> — NEVER
  // {@html} (shape geometry only → XSS-free, consistent with the anti-{@html} lean, N-032).
  // Unknown `name` with no `path` → DEV-warn + render empty (<svg> with no <path>), the W-13
  // unknown-id-drop precedent — the root still mounts + registers, so the empty state is
  // CDP-observable.
  //
  // tint (default = surrounding text colour) rides an inline `--icon-tint` custom property the
  // `.icon` skin reads as `fill: var(--icon-tint, currentColor)` (the led/chip/meter inline-var
  // precedent). label is DECORATIVE by default (`aria-hidden`, no role) — a glyph beside a text
  // label must not double-announce; when a `label` is passed the glyph becomes `role="img"` +
  // `aria-label` (a standalone/meaningful icon). No `src` / `border-radius` / `disabled`: icons
  // are inert display; interactivity belongs to the consumer's button/link.
  //
  // The type-class is supplied by `envelope` (N-023), so no `class` is hardcoded. No local CSS:
  // all appearance — the box, tint, alignment — is skin, keyed by `.icon` in the one skin file
  // (N-025). Glyphs are authored on a 24×24 viewBox (icons.ts) and rendered at 16/20/24 via the
  // svg width/height attrs; fill-based so `fill` tinting works (stroke variant deferred, D-065).
  import { envelope } from '$common/components/base/envelope';
  import { icons, type IconPath } from './icons';

  let {
    name,
    path,
    size = 16,
    tint,
    label,
    id,
  }: {
    name?: string;
    path?: IconPath;
    size?: 16 | 20 | 24;
    tint?: string;
    label?: string;
    id?: string;
  } = $props();

  // path override wins over name; a registry entry (or the override) may be a `d` string or
  // `d[]` — normalise to an array for the {#each}. Unknown name + no path → [] (empty glyph).
  const paths = $derived.by((): string[] => {
    const raw: IconPath | undefined = path ?? (name != null ? icons[name] : undefined);
    if (raw == null) return [];
    return Array.isArray(raw) ? raw : [raw];
  });

  // DEV-only: warn when nothing resolves (unknown name and no override) — renders empty, no throw.
  if (import.meta.env.DEV && path == null && (name == null || icons[name] == null)) {
    console.warn(
      `[xgen icon] no glyph for name=${JSON.stringify(name)} (not in registry, no path override) — rendering empty.`,
    );
  }

  // N-024 opt-in. Read-only (no user delta); registry stays uniform (N-030 §4). `name` reports
  // the shape identity — '(path)' when the raw override is used, else the registry key (or null).
  // `tint` reports the resolved value ('currentColor' when unset); `decorative` = no label.
  const debug = () => ({
    name: path != null ? '(path)' : (name ?? null),
    size,
    tint: tint ?? 'currentColor',
    decorative: label == null,
  });
</script>

<svg
  use:envelope={{ name: 'icon', id, debug }}
  viewBox="0 0 24 24"
  width={size}
  height={size}
  role={label != null ? 'img' : undefined}
  aria-label={label ?? undefined}
  aria-hidden={label == null ? 'true' : undefined}
  style={tint != null ? `--icon-tint: ${tint}` : undefined}
>
  {#each paths as d}<path {d} />{/each}
</svg>
