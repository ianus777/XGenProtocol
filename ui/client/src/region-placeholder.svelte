<script lang="ts">
  // region-placeholder — the throwaway leaf for M-RP6.1f. ONE component, mapped to all 8 region ids in the
  // registry (D6); each real region widget later replaces ONE registry entry, no rewrite. Shell-local (it
  // is disposable client chrome, not a reusable `core` component).
  //
  // Renders ONE `section` (D6) — deliberately NO inner `paragraph`, which would double the registry delta
  // for a throwaway; the body is a plain text node in the section. `id = "region-" + regionId` so it
  // self-registers as `section#region-spaces`, … (the leaf-level registry entry the getter G leans on).
  import Section from '$core/components/data-independent/section.svelte';

  let { regionId }: { regionId: string } = $props();

  // Display names (region-dock §2). A placeholder body says what it is, honestly (M-RP6.1g+ replaces it).
  const NAMES: Record<string, string> = {
    spaces: 'R1 · Spaces',
    rooms: 'R2 · Rooms',
    self: 'R3 · Self / connection',
    'room-header': 'R4 · Room header',
    stream: 'R5 · Message stream',
    composer: 'R6 · Composer',
    members: 'R7 · Members',
    inspector: 'R8 · Selection info',
  };

  const title = $derived(NAMES[regionId] ?? regionId);
  const body = $derived(`${title} — placeholder (M-RP6.1g+)`);
</script>

<Section {title} id={`region-${regionId}`}>{body}</Section>
