<script lang="ts">
  // status-indicator — data-independent, COMPOSITE (M-RP2.22): the FIRST di composite and the
  // SEVENTEENTH `core` component. Root IS `<div class="status-indicator">` — the N-020/N-022 composite
  // marker (`<div class="type">` wrapper via `envelope`, N-023), vs an atomic whose root is a native
  // tag. It composes three already-built atomics into the general status-row: `led` (required) +
  // `label` (required) + an OPTIONAL trailing `link`. Binding none; **di** — the caller supplies the
  // state->colour map, the caption, and the link target; the composite interprets no domain structure
  // (that interpretation is what a dd component would do). N-049 reserved this as a build-time walk.
  //
  // COMPOSITE-REGISTRATION MODEL (the first-composite precedent): the composite root registers ONE
  // aggregate getter; the children are the REAL atomic components, each self-registering (they pass
  // their own `debug` getter unconditionally — `envelope` registers on the `use:` action, keyed
  // `id ?? ordinal`). We hand them composite-derived STABLE ids `<id>__led` / `<id>__label` /
  // `<id>__link` so the registry reads cleanly (not ordinal noise). Zero changes to the three closed
  // atomics (D-065 — don't retrofit built components for a new one's convenience). A composite row
  // therefore yields MULTIPLE registry entries (composite + each child), the first time one sampler
  // cell is more than one `ids()` entry.
  //
  // The aggregate getter is `{ state, caption, hasLink }` — `colour` is DELIBERATELY omitted: the
  // composite would otherwise duplicate `led`'s `?? "#000000"` sentinel; colour is verified on the
  // `led#…__led` child entry, so the composite reports only what it itself owns (no logic duplication,
  // no drift).
  //
  // The trailing link is rendered IFF `linkHref` is set — a genuine `{#if}` on an optional sub-element
  // (a status row may have no detail link). This is NOT the N-053 panel-mounting case (that rule —
  // hide tabs via CSS, never `{#if}` — is about keeping every COMPONENT mounted for registry
  // completeness; an absent optional link is correctly absent). The link's consumer-wiring contract is
  // intact (N-052): `linkExternal` bundles the safe target+rel, `onLinkClick` is forwarded for
  // shell.open / SPA-route; the composite never imports Tauri or a router.
  //
  // The shells' bespoke `.state-dot` + dotColor/isPulsing generalises onto the minimal form (led +
  // label, no link) — a LATER shell-migration step, not this milestone (sampler-only, D-097).
  import { envelope } from '$common/components/base/envelope';
  import Led from './led.svelte';
  import Label from './label.svelte';
  import Link from './link.svelte';

  let {
    states = {},
    state,
    pulse = false,
    caption = '',
    linkHref,
    linkText = 'Details →',
    linkExternal = false,
    onLinkClick,
    id,
  }: {
    states?: Record<string, string>;
    state?: string;
    pulse?: boolean;
    caption?: string;
    linkHref?: string;
    linkText?: string;
    linkExternal?: boolean;
    onLinkClick?: (e: MouseEvent) => void;
    id?: string;
  } = $props();

  // Composite-derived stable child ids (so the self-registering children read cleanly, not ordinal).
  const cid = (s: string) => (id ? `${id}__${s}` : undefined);

  // N-024 opt-in. Aggregate of what the COMPOSITE owns; `colour` lives on the led child entry.
  const debug = () => ({ state: state ?? null, caption, hasLink: !!linkHref });
</script>

<div use:envelope={{ name: 'status-indicator', id, debug }}>
  <Led {states} {state} {pulse} id={cid('led')} />
  <Label text={caption} id={cid('label')} />
  {#if linkHref}
    <Link href={linkHref} text={linkText} external={linkExternal} onclick={onLinkClick} id={cid('link')} />
  {/if}
</div>
