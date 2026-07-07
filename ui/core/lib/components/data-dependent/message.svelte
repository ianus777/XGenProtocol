<script lang="ts">
  // message — a data-dependent COMPOSITE (M-RP5.5, the message dd sub-family opener,
  // `docs/xgen-dd-message-family-phase0.md` v1.0). It materializes ONE `MessageDescriptor` into a
  // chat row — the message decides HOW ONE MESSAGE LOOKS; the future `message-stream` (M-RP5.6)
  // decides RELATIONSHIPS between messages (ordering / grouping / dividers / scroll). No sibling
  // knowledge here.
  //
  // Source-agnostic (the entity dd discipline): consumes a `MessageDescriptor` view-model, `core`
  // imports NO protocol type. `author` REUSES `EntityDescriptor` — the message avatar IS an
  // entity-avatar of the author. The shell owns protocol → descriptor.
  //
  // Step A scope: `kind:'text'` FULL render + `isOwn` flip. Composes the REAL atomics (no
  // re-implement) — `entity-avatar` (author glyph, self-registers `__avatar`) + `label` (author
  // name, `__name`) + `paragraph` (body text-node, `__body`) — plus the `details` widget socket.
  // Deferred to later steps: grouped / edited / deleted states (B); `system` kind (C); `bodyExtras`
  // (reserved-unfed, D-065). The composite therefore yields MULTIPLE registry entries per cell (the
  // status-indicator / entity-item precedent — the matrix multiplies).
  //
  // WIDGET SOCKET (W-12 all-widgets model). `details` is a `WidgetMount[]`: the message is a HOST
  // SURFACE that renders declared widgets by DURABLE id (J-475). Resolution is source-agnostic — the
  // consumer supplies a `widgets` registry (widgetId → component); a mount the host CANNOT resolve is
  // DROPPED (W-13 reconcile), never crashes the row. `core` imports no concrete widget; the region
  // shell (M-RP6.x) / sampler owns the registry. `body` is ALWAYS a text node (`{text}` via
  // `paragraph`), never `{@html}` — the inline-mark formatter is the deferred kind-4 `use:render`.
  import { envelope } from '$common/components/base/envelope';
  import type { Component } from 'svelte';
  import EntityAvatar from './entity-avatar.svelte';
  import Label from '../data-independent/label.svelte';
  import Paragraph from '../data-independent/paragraph.svelte';
  import type { MessageDescriptor } from './types';

  let {
    descriptor,
    widgets = {},
    id,
  }: {
    descriptor: MessageDescriptor;
    widgets?: Record<string, Component>; // widgetId → component; unknown id dropped (W-13)
    id?: string;
  } = $props();

  const kind = $derived(descriptor.kind);
  const author = $derived(descriptor.author);
  const isOwn = $derived(!!descriptor.isOwn);

  // Author display name for the header line (the avatar owns its own initials fallback separately).
  // Absent name → the id, honestly (the entity-item / avatar getter convention).
  const authorName = $derived(author?.name ?? author?.id ?? '');
  const hasBody = $derived(!!descriptor.body);

  // details socket — resolve each declared mount against the consumer registry; DROP unknown ids
  // (W-13). The resolved list is the RENDER truth, so `detailsCount` reports what is actually shown
  // (== declared-known), and a dropped unknown id lowers the count. Stable per-index key so a
  // re-render does not churn the mounts.
  const resolvedDetails = $derived(
    (descriptor.details ?? [])
      .map((m, i) => ({
        key: `${m.widgetId}-${i}`,
        component: widgets[m.widgetId],
        props: m.props ?? {},
      }))
      .filter((d) => !!d.component),
  );

  // Composite-derived stable child ids (so the self-registering atomics read cleanly, not ordinal).
  const cid = (s: string) => (id ? `${id}__${s}` : undefined);

  // Aggregate getter. `author` = a readable identifier (name ?? id) or null for the authorless
  // system kind; `detailsCount` = RESOLVED (rendered) mounts, the drop-unknown proof.
  const debug = () => ({
    kind,
    isOwn,
    author: author ? (author.name ?? author.id) : null,
    hasBody,
    detailsCount: resolvedDetails.length,
  });
</script>

<!-- dd-composite root = honest HTML for a message: <article> (a self-contained composition — a
  comment/post; N-075 dd-root). It composes cleanly into the stream's role="log" later. `data-own`
  reflects the shell-set mirror; the skin owns the both-sides reserved avatar column + the flip. The
  avatar (author), name (label), and body (paragraph) are REAL atomics that self-register. -->
<article
  use:envelope={{ name: 'message', id, debug }}
  data-kind={kind}
  data-own={isOwn || undefined}
>
  {#if author}
    <EntityAvatar descriptor={author} variant="list" id={cid('avatar')} />
  {/if}
  <div class="msg-content">
    <div class="msg-header">
      <Label text={authorName} id={cid('name')} />
      {#if resolvedDetails.length}
        <span class="msg-details">
          {#each resolvedDetails as d (d.key)}
            {@const W = d.component}
            <W {...d.props} />
          {/each}
        </span>
      {/if}
    </div>
    <Paragraph text={descriptor.body ?? ''} class="message-paragraph" id={cid('body')} />
  </div>
</article>
