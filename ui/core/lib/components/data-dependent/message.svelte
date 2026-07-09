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
  // The composite therefore yields MULTIPLE registry entries per cell (the status-indicator /
  // entity-item precedent — the matrix multiplies).
  //
  // Step B scope: the three render-states on `text` (`docs/xgen-dd-message-family-phase0.md` §2).
  //   - `grouped` — a continuation row: the header line (name + details) AND the avatar are both
  //     suppressed (M-RP5.7 / D-106 — a same-author run otherwise repeats the identical avatar and
  //     grouping reads as invisible); the avatar grid COLUMN stays reserved in the skin so the body
  //     stays aligned under the head (no left-shift). It is STREAM-COMPUTED (same author within a
  //     window), so it is a PROP, not a descriptor field — Phase-0 §4's locked descriptor excludes it
  //     and §5 says the message only *accepts* the flag; the future `message-stream` (M-RP5.6) sets it.
  //   - `edited` — a trailing muted "(edited)" marker after the body (descriptor field).
  //   - `deleted` — a tombstone: the body paragraph is replaced by a skin-owned placeholder
  //     (`--msg-deleted`), and `details` + `edited` are dropped; the avatar + name STAY (who/where is
  //     still context). Precedence: `deleted` wins over `edited`/`details`/`grouped`-body.
  // Deferred to later steps: `system` kind (C); `bodyExtras` (reserved-unfed, D-065).
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
    grouped = false,
    id,
  }: {
    descriptor: MessageDescriptor;
    widgets?: Record<string, Component>; // widgetId → component; unknown id dropped (W-13)
    grouped?: boolean; // STREAM-COMPUTED continuation flag (M-RP5.6 sets it); not a descriptor field
    id?: string;
  } = $props();

  const kind = $derived(descriptor.kind);
  const author = $derived(descriptor.author);
  const isOwn = $derived(!!descriptor.isOwn);
  const edited = $derived(!!descriptor.edited);
  const deleted = $derived(!!descriptor.deleted);

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
  // system kind; `detailsCount` = RESOLVED (rendered) mounts, the drop-unknown proof. `grouped` /
  // `edited` / `deleted` report the Step-B render-state (deleted forces `detailsCount` to 0 via the
  // render guard below, so the getter mirrors what is actually shown).
  //
  // Option A normalization on `system` (M-RP5.5 C): the system kind is authorless / centered, so the
  // text-only fields are structurally meaningless — the getter FORCES them off. Deliberate: the getter
  // tracks RENDER truth, not descriptor truth (the `deleted → detailsCount:0` precedent, J-479). A
  // stray field on a system descriptor never renders, so it never reports.
  const debug = () =>
    kind === 'system'
      ? {
          kind,
          author: null,
          hasBody,
          detailsCount: 0,
          isOwn: false,
          grouped: false,
          edited: false,
          deleted: false,
        }
      : {
          kind,
          isOwn,
          author: author ? (author.name ?? author.id) : null,
          hasBody,
          detailsCount: deleted ? 0 : resolvedDetails.length,
          grouped,
          edited,
          deleted,
        };
</script>

<!-- dd-composite root = honest HTML for a message: <article> (a self-contained composition — a
  comment/post; N-075 dd-root). It composes cleanly into the stream's role="log" later.

  TOP-LEVEL KIND SPLIT (M-RP5.5 C). Two v1 kinds, two visibly separate sub-trees so the `system`
  path reads NONE of the text-only fields:
   - `system` — an authorless centered notice. No `entity-avatar`, no `.msg-header` / `label`
     (`__name`), no `details`, no `edited` marker, no tombstone path: ONE centered `paragraph`
     (`__body`). Root `<article data-kind="system">`, NO `data-own` (a system notice has no side).
   - `text` — the A/B sub-tree (below): reserved avatar column + header guard + body/tombstone;
     `data-own` reflects the shell-set mirror, the skin owns the both-sides column + the flip. The
     avatar (author), name (label), and body (paragraph) are REAL atomics that self-register. -->
{#if kind === 'system'}
  <article use:envelope={{ name: 'message', id, debug }} data-kind="system">
    <Paragraph text={descriptor.body ?? ''} class="message-paragraph" id={cid('body')} />
  </article>
{:else}
<article
  use:envelope={{ name: 'message', id, debug }}
  data-kind={kind}
  data-own={isOwn || undefined}
  data-grouped={grouped || undefined}
  data-deleted={deleted || undefined}
>
  <!-- Grouped continuation rows suppress the avatar too (M-RP5.7 / D-106): a same-author run repeats
    the identical avatar otherwise, so grouping reads as invisible. The `entity-avatar` child is NOT
    rendered on a grouped cell (element absent, not `visibility:hidden`) → that cell registers no
    `__avatar` (and no `__name`, dropped since B). The skin keeps the avatar grid track reserved so
    the continuation body stays column-aligned under the head (no left-shift). -->
  {#if author && !grouped}
    <EntityAvatar descriptor={author} variant="list" id={cid('avatar')} />
  {/if}
  <div class="msg-content">
    <!-- Grouped continuation rows suppress the whole header line (name + details); the avatar column
      still reserves space so bodies stay aligned. The `label` (`__name`) is therefore NOT rendered on
      a grouped cell → that cell registers one fewer entry. -->
    {#if !grouped}
      <div class="msg-header">
        <Label text={authorName} id={cid('name')} />
        <!-- `details` is dropped on a tombstone (deleted precedence). -->
        {#if !deleted && resolvedDetails.length}
          <span class="msg-details">
            {#each resolvedDetails as d (d.key)}
              {@const W = d.component}
              <W {...d.props} />
            {/each}
          </span>
        {/if}
      </div>
    {/if}
    {#if deleted}
      <!-- Tombstone: the body paragraph is dropped (so `__body` is NOT registered). The placeholder
        copy is skin-owned (`content: var(--msg-deleted)`) — the component holds no string. -->
      <span class="msg-deleted"></span>
    {:else}
      <Paragraph text={descriptor.body ?? ''} class="message-paragraph" id={cid('body')} />
      {#if edited}
        <span class="message-edited">(edited)</span>
      {/if}
    {/if}
  </div>
</article>
{/if}
