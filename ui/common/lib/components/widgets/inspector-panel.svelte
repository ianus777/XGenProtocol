<script lang="ts">
  // inspector-panel — R8 Selection info: the SECOND real system widget (M-RP6.1h) and the selection
  // bus's FIRST CROSS-REGION reader. `kind: system` (W-13, non-removable), Level-2 marked via
  // `data-tier="widget"` on the root (the `self-panel` shape). It replaces the `inspector` placeholder
  // in the layout registry — ONE registry entry swapped, no rewrite (renderer A's prop-injected
  // registry, N-093).
  //
  // THE LOOP CLOSES HERE: R3 (`self-panel`) WRITES `{regionId, entity}` to the selection bus; R8 READS
  // it and renders that entity's rows. Same `$common` store, two regions — this is what makes R8
  // visibly a cross-region reader.
  //
  // RENDERS THE DESCRIPTOR, AND NOTHING MORE (D2). `EntityDescriptor` is `{kind, name?, id, flags?,
  // image?}` — a thin thing, so this is a thin panel. NO `get_entity_info`, no new Rust: a second Rust
  // projection today would be a second surface (D-067 drift) delivering ZERO new information (the only
  // selectable entity is *self*, whose fields already come from `get_self_state`). `image` is
  // reserved-UNFED (D-065) → NOT rendered.
  //
  // HEADER = `entity-avatar` (D6), NOT `entity-item`. The avatar renders the same five fields more
  // richly (kind → circle/square/hexagon, id → seed colour, initials) with NO new data. `entity-item`
  // is deliberately refused: it carries a `selected` prop with a live `[data-selected]` skin rule that
  // in R8 would be trivially always-true — the N-097 trap — so we route AROUND it. Grounded against
  // entity-avatar source: `variant="labeled"` draws shape + seed + initials only (the `<figcaption>`
  // stays reserved-unused), so the Name row below is NOT a duplicate.
  import { envelope } from '$common/components/base/envelope';
  import { selection } from '$common/stores/selection.svelte';
  import Section from '$core/components/data-independent/section.svelte';
  import Label from '$core/components/data-independent/label.svelte';
  import Paragraph from '$core/components/data-independent/paragraph.svelte';
  import EntityAvatar from '$core/components/data-dependent/entity-avatar.svelte';

  // The leaf mount (`region-node`) passes a widget ONLY `regionId` (grounded against region-node.svelte).
  // Derive the envelope id via the EXACT `region-placeholder` convention `id = "region-" + regionId`
  // (N-096 leaf-id convention), so the delta reads as a clean swap in place — `section#region-inspector`
  // OUT, `inspector-panel#region-inspector` IN.
  let { regionId, id = `region-${regionId}` }: { regionId: string; id?: string } = $props();
  const cid = (s: string) => (id ? `${id}__${s}` : undefined);

  // Everything derives from the ONE bus read (D1). R8 reads the bus and nothing else — with W-3
  // (a common widget can't import a shell dep) and a leaf mount that gives only `regionId`, store
  // mediation is the only channel (N-096).
  const sel = $derived(selection.current);

  // Flags row (D4): comma-joined names of the flags that are TRUE (e.g. "isAi, revoked"). The row is
  // conditional — its true-branch is unreachable from the shipped UI today (self carries no flags) but
  // it IS exercisable at verify by writing the bus directly (V4), so it is an exercisable branch, not an
  // unfed one (the N-095/N-097 line).
  const flagNames = $derived(
    Object.entries(sel?.entity.flags ?? {})
      .filter(([, v]) => v)
      .map(([k]) => k),
  );
  const hasFlags = $derived(flagNames.length > 0);

  // Aggregate getter G (W-4). `rowCount` is RENDER-TRUTH (the `message.detailsCount` precedent) — 4 fixed
  // rows (kind/name/id/source) + 1 when the flags row renders — which is what makes the conditional flags
  // row observable. The XGID and name are deliberately NOT republished: they are already CDP-readable off
  // the composed `label#…__id` / `…__name` getters, so republishing is duplication AND a payload leak
  // (the N-060 `hasValue` precedent).
  const debug = () => ({
    hasSelection: !!sel,
    regionId: sel?.regionId ?? null,
    kind: sel?.entity.kind ?? null,
    rowCount: sel ? 4 + (hasFlags ? 1 : 0) : 0,
  });
</script>

<div class="inspector-panel" data-tier="widget" use:envelope={{ name: 'inspector-panel', id, debug }}>
  <Section title="Selection" id={cid('section')}>
    {#if sel}
      <!-- Header: the avatar renders kind→shape + id→seed + initials (no name, grounded). -->
      <div class="inspector-head">
        <EntityAvatar descriptor={sel.entity} variant="labeled" id={cid('avatar')} />
      </div>
      <!-- Rows (D3, the About `<dl>` shape): keys plain `<dt>`, values registry-visible `Label`. -->
      <dl class="inspector-grid">
        <dt>Kind</dt>
        <dd><Label text={sel.entity.kind} id={cid('kind')} /></dd>

        <dt>Name</dt>
        <!-- honest absence, the About guard — never a fake name -->
        <dd><Label text={sel.entity.name ?? '—'} id={cid('name')} /></dd>

        <dt>ID</dt>
        <dd><Label text={sel.entity.id} id={cid('id')} /></dd>

        <dt>Source</dt>
        <!-- the field that makes R8 visibly a CROSS-REGION reader (the writer's regionId) -->
        <dd><Label text={sel.regionId} id={cid('source')} /></dd>

        {#if hasFlags}
          <dt>Flags</dt>
          <dd><Label text={flagNames.join(', ')} id={cid('flags')} /></dd>
        {/if}
      </dl>
    {:else}
      <!-- Empty state = the `entity-panel` pattern (D5): a composed Paragraph. Root + section stay
        mounted (clear → exact return to baseline is the honest orphan proxy, N-092a). -->
      <Paragraph text="Nothing selected" id={cid('empty')} />
    {/if}
  </Section>
</div>
