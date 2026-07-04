<script lang="ts">
  // section — data-independent, a collapsible DISCLOSURE container. The 27th `core`. Root IS
  // native <section> (N-020 atomic). **di, binding = none** — it interprets no domain structure
  // (a bare structural container; the caller fills the body). **Atomic-ish** (the chip/star-rating
  // self-contained shape): it composes NO registering child *components* — the header bits + chevron
  // are raw internal elements, the body is a `children` snippet — so it registers ONE getter and the
  // matrix adds +1 per instance (a nested section is its own instance → its own id). Does NOT open
  // the dd track. **Supersedes** the seed `section-header` divider (a divider that had to reach out
  // and hide sibling rows; a self-contained container that owns its own body is cleaner).
  //
  // Skeleton (Joe-locked):
  //   <section class="section">
  //     <h{level} class="section-header"> title <span class="section-badge">2/5</span> </h{level}>
  //     <div class="section-body"> …children (incl. a nested <section>)… </div>
  //   </section>
  //
  // `title?` → an internal <h{level}> (level? default 2); omitted → a bare styled container (the
  // "simple div" use). `badge?` is a CALLER-supplied string (e.g. "2/5" unread/total — programmatic,
  // not user text) rendered as-is in the header. `collapsible?` (default false) turns the header into
  // a <button aria-expanded> + a chevron; `collapsed?` ($bindable) hides the body via [data-collapsed]
  // → display:none (NEVER {#if} — the slot stays mounted so its content/registry survives a collapse).
  //
  // The chevron reuses combobox's masked glyphs (--tri chevron / --tri-open closed-triangle), swapped
  // on [data-collapsed] (Joe-lock). Header is a SOLID BAND (not a rule line) so the future bg-colour /
  // picture-background widget-mods have a filled surface to target (deferred, D-065). Accent-neutral.
  // No local CSS — all appearance is skin, keyed `.section*` (N-025 / N-031 L2).
  import { envelope } from '$common/components/base/envelope';
  import type { Snippet } from 'svelte';

  let {
    title,
    badge,
    collapsible = false,
    collapsed = $bindable(false),
    level = 2,
    width,
    id,
    children,
  }: {
    /** Header title. Omitted = a bare container (no header). */
    title?: string;
    /** Caller-supplied header string (e.g. "2/5"). Programmatic, not user-facing text. */
    badge?: string;
    /** Show the collapse affordance (chevron + toggle). Default false. */
    collapsible?: boolean;
    /** Collapsed state ($bindable). Body hidden via [data-collapsed] while true. */
    collapsed?: boolean;
    /** Heading level for the <h{level}> title (default 2). */
    level?: number;
    /** Fixed width (e.g. "320px" / "24rem"). Omitted = full-width (100%). */
    width?: string;
    id?: string;
    /** Body content (any; a nested <section> is valid). */
    children?: Snippet;
  } = $props();

  const heading = $derived(`h${level}`);

  function toggle() {
    if (collapsible) collapsed = !collapsed;
  }

  const debug = () => $state.snapshot({ title, badge, collapsible, collapsed, width });
</script>

<section
  data-collapsed={collapsed ? 'true' : undefined}
  style={width ? `width: ${width}` : undefined}
  use:envelope={{ name: 'section', id, debug }}
>
  {#if title !== undefined}
    <svelte:element this={heading} class="section-header">
      {#if collapsible}
        <button
          type="button"
          class="section-toggle"
          aria-expanded={!collapsed}
          onclick={toggle}
        >
          <span class="chev" aria-hidden="true"></span>
          <span class="section-title">{title}</span>
        </button>
      {:else}
        <span class="section-title">{title}</span>
      {/if}
      {#if badge !== undefined}
        <span class="section-badge">{badge}</span>
      {/if}
    </svelte:element>
  {/if}
  <div class="section-body">
    {@render children?.()}
  </div>
</section>
