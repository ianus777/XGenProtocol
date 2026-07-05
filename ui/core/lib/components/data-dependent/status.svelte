<script lang="ts">
  // status — a data-dependent ATOMIC (M-RP5.1a) that materializes a SELF-SET status (Track A
  // `state.status`, J-461) into a visual. Self-status is personal EXPRESSION (an emoji + a short
  // line), NOT connection/presence state — Track A deferred presence, so this NEVER renders
  // here/away. `variant` is the display density (the led.state / entity-avatar.variant discipline).
  //
  // Source-agnostic: consumes a view-model `{ emoji?, text?, updatedAt?, expiresAt? }` (the shell
  // maps Track A's `StatusRecord` → this); `core` imports NO protocol type.
  //
  // The `badge` variant is the avatar corner-slot payload (M-RP5.1b composes it); `line`/`full`
  // are standalone reads (card/row secondary, context menu). Emoji is a 1-grapheme cap (Track A),
  // taken grapheme-safe here.
  //
  // EXPIRY / EMPTY = MOUNTED-BUT-EMPTY (Joe-locked, correcting the runbook's "not rendered"
  // wording): the root `<span use:envelope>` ALWAYS mounts (so getter G always registers and
  // `expired:true` stays CDP-readable), but renders NO emoji/text content when expired or
  // content-less. `data-empty` then collapses it via the skin (`display:none`) — a stable
  // registry count (no cell blinks in/out on expiry) and the honest "status exists but expired"
  // state. "Absent" = empty content, not absent DOM.
  import { envelope } from '$common/components/base/envelope';

  let {
    status,
    variant = 'badge',
    id,
  }: {
    status: { emoji?: string; text?: string; updatedAt?: string; expiresAt?: string };
    variant?: 'badge' | 'line' | 'full';
    id?: string;
  } = $props();

  // F — lazy expiry: evaluated at render (no reactive timer). An expired status is empty, not stale.
  const expired = $derived(
    !!status.expiresAt && new Date(status.expiresAt).getTime() < Date.now(),
  );

  // Emoji is a single grapheme by contract (Track A cap); take the first grapheme defensively so a
  // ZWJ sequence / combining mark stays one unit (the entity-avatar idiom).
  const emoji = $derived(firstGrapheme(status.emoji));
  const hasText = $derived(!!(status.text && status.text.trim()));

  // `full` shows a relative "updated Nm ago" from `updatedAt` (lazy, like expiry).
  const rel = $derived(status.updatedAt ? relTime(status.updatedAt) : undefined);

  // visible = there is something to show AND it has not expired. When false, the root still mounts
  // (registers) but renders no content → `data-empty` collapses it.
  const visible = $derived(!expired && (!!emoji || hasText));

  // E — tooltip fallback: on the emoji-only badge, surface the text on hover; on line/full where
  // text is already shown, surface the relative time.
  const title = $derived(
    variant === 'badge' ? (status.text ?? undefined) : (rel ?? undefined),
  );

  function firstGrapheme(s: string | undefined): string | undefined {
    const v = (s ?? '').trim();
    if (!v) return undefined;
    if (typeof Intl !== 'undefined' && 'Segmenter' in Intl) {
      const seg = new Intl.Segmenter(undefined, { granularity: 'grapheme' }).segment(v);
      return [...seg][0]?.segment;
    }
    return [...v][0];
  }

  function relTime(iso: string): string {
    const then = new Date(iso).getTime();
    if (Number.isNaN(then)) return '';
    const s = Math.max(0, Math.round((Date.now() - then) / 1000));
    if (s < 60) return `updated ${s}s ago`;
    const m = Math.round(s / 60);
    if (m < 60) return `updated ${m}m ago`;
    const h = Math.round(m / 60);
    if (h < 24) return `updated ${h}h ago`;
    return `updated ${Math.round(h / 24)}d ago`;
  }

  // G — the dd-atomic self-registers ONE aggregate getter. `expired` stays observable even on an
  // empty (collapsed) instance.
  const debug = () => ({ variant, emoji: emoji ?? null, hasText, expired });
</script>

<!-- C — root: badge = `<span class="status" role="img">` (a glyph token; aria-label = text ?? emoji);
  line/full = plain `<span class="status">` (text carries meaning). `data-empty` collapses the
  expired/content-less case (mounted-but-empty). -->
<span
  use:envelope={{ name: 'status', id, debug }}
  role={variant === 'badge' ? 'img' : undefined}
  aria-label={variant === 'badge' ? (status.text ?? emoji ?? undefined) : undefined}
  {title}
  data-variant={variant}
  data-empty={!visible || undefined}
>
  {#if visible}
    {#if emoji}<span class="st-emoji">{emoji}</span>{/if}
    {#if variant !== 'badge' && hasText}<span class="st-text">{status.text}</span>{/if}
    {#if variant === 'full' && rel}<span class="st-time">{rel}</span>{/if}
  {/if}
</span>
