<script lang="ts">
  // link — data-independent, NAVIGATION-kind (M-RP2.21): an atomic hyperlink. The FIRST
  // navigation-kind di — a new kind alongside interactive (toggle/button/textfield/…) and
  // display (label/paragraph/image/led). It neither binds an editable value (interactive) nor
  // is purely read-only (display): it ACTS (navigates) while carrying a `text` label. Atomic
  // (N-020): the root IS the native <a>.
  //
  // link IS an <a> — DISTINCT from the existing button link-styled shape (a <button> that only
  // *looks* like a link, acts via onclick, no navigation). Never conflate (N-049): the
  // discriminator is real navigation (a destination, right-click-open works) vs an action.
  //
  // Value prop is `text` (the display-di semantic value-name family — label/paragraph = text,
  // image = src; link = text). `text=""` is allowed for an icon-only link, in which case
  // `ariaLabel` supplies the accessible name (the image-`alt` guard shape: DEV-warn if a link
  // has neither visible text nor an ariaLabel — no accessible name).
  //
  // external -> target="_blank" + rel="noopener noreferrer" (the safe rel bundled; a consumer
  // shouldn't have to remember noopener). disabled is SYNTHESISED (an <a> has no native
  // disabled): drop href (non-navigating), aria-disabled="true", tabindex=-1 (non-focusable),
  // block onclick; the skin greys it — keeps library-wide disabled consistency.
  //
  // CONSUMER-WIRING (the atomic stays dumb — it never imports Tauri or a router): leaving to the
  // OS browser is the consumer's onclick -> Tauri shell.open(href) (a raw target=_blank inside a
  // Tauri WebView can spawn a blank in-app webview, not the system browser); an in-app SPA route
  // is the consumer's onclick -> router (preventDefault + navigate); the real href is retained
  // for a11y + right-click-open. Opening a MODAL is NOT a link (no destination) — that is a
  // `button` flipping an open state; the modal surface is a future `modal`/`dialog` component.
  //
  // The type-class is supplied by `envelope` (mergeClasses), so no `class` is hardcoded (N-023).
  // No local CSS: a bare <a> is function-complete; all appearance is skin, keyed `.link` — and
  // ACCENT-derived (color: var(--accent2), re-themes gold/blue per shell). link returns to the
  // accent pattern: `led` was the deliberate caller-supplied-colour exception, not a new norm.
  import { envelope } from '$common/components/base/envelope';

  let {
    href,
    text,
    onclick,
    external = false,
    disabled = false,
    ariaLabel,
    id,
  }: {
    href: string;
    text: string;
    onclick?: (e: MouseEvent) => void;
    external?: boolean;
    disabled?: boolean;
    ariaLabel?: string;
    id?: string;
  } = $props();

  if (import.meta.env.DEV && text === '' && !ariaLabel) {
    console.warn('[xgen link] an icon-only link (text="") needs `ariaLabel` — no accessible name otherwise.');
  }

  // disabled <a> has no native semantics: drop href (non-navigating) + mark/grey it.
  const effectiveHref = $derived(disabled ? undefined : href);
  const target = $derived(external ? '_blank' : undefined);
  const rel = $derived(external ? 'noopener noreferrer' : undefined);

  // N-024 opt-in. Navigation carries no editable value; the registry stays uniform (N-030 §4).
  // `href` is the PROP value even when disabled drops it from the rendered element (verify reads
  // the rendered attribute separately).
  const debug = () => ({ text, href, external, disabled });
</script>

<a
  use:envelope={{ name: 'link', id, debug }}
  href={effectiveHref}
  {target}
  {rel}
  aria-label={ariaLabel}
  aria-disabled={disabled || undefined}
  tabindex={disabled ? -1 : undefined}
  onclick={disabled ? (e) => e.preventDefault() : onclick}
>{text}</a>
