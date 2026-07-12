<script lang="ts">
  // plugin-list — the FIFTH widget (M-RP6.1l), and the gear's pane. `kind: system` (W-13,
  // non-removable), `surface: none` (W-12): it is CONTENT rendered inside a host (a shell dialog now,
  // Settings later — surfaces §3.2, S-2 "one component, two mounts"), so it spends no surface and is
  // NOT a region leaf. Level-2 marked via `data-tier="widget"` on the root (the self-panel /
  // inspector-panel shape).
  //
  // IT RENDERS THE REGISTRY, AND NOTHING MORE (D1/D5). The registry is `CLIENT_PLUGINS` — the client's
  // own compiled plugins, the only ones it can see (there is no node-plugin read verb; M-RP-PLUGINS-NODE
  // is filed, not smuggled in). Three honest rows: self-panel · inspector-panel · plugin-list (it lists
  // itself). A list that faked a universal registry would be worse than three honest rows.
  //
  // READ-ONLY (D6). No Remove / Disable / Launch / Settings control: no such verb exists, and a
  // permanently-disabled control with no countdown behind it is exactly what 6.1j forbade (the absent
  // slot ships ABSENT, not faked). What ships is the `[system]` badge — Ch6 §6.8.5's own drawing, which
  // IS W-13 made visible. NO W-8 phase-limit note anywhere (D7): a read-only list of what is loaded is
  // not a false statement about anything, so there is nothing to sweep at close (N-109 pre-empt).
  import { envelope } from '$common/components/base/envelope';
  import { CLIENT_PLUGINS, type PluginDescriptor } from '$common/plugins/registry';
  import Label from '$core/components/data-independent/label.svelte';

  // Mounted directly by the host (the plugins-dialog), not by `region-node` — so it takes a plain `id`
  // (default `plugin-list`), from which the composed row-value Labels derive their ids.
  let { id = 'plugin-list' }: { id?: string } = $props();
  const cid = (s: string) => (id ? `${id}__${s}` : undefined);

  // Alphabetical (D8) — the list is not a priority indicator (Ch6 §6.8.5). A copy so the source array
  // is never mutated in place.
  const plugins = $derived(
    [...CLIENT_PLUGINS].sort((a, b) => a.name.localeCompare(b.name)),
  );

  // Aggregate getter G (W-4). rowCount is RENDER-TRUTH: counted from `plugins` (what the {#each}
  // renders), the message.detailsCount precedent. system/custom split so the badge is observable off G
  // as well as off the painted DOM.
  const systemCount = $derived(plugins.filter((p) => p.kind === 'system').length);
  const customCount = $derived(plugins.filter((p) => p.kind === 'custom').length);
  const debug = () => ({ count: plugins.length, systemCount, customCount });

  // The three axes, rendered as one middot-joined meta line (host · delivery · surface) — the trust
  // posture made visible (D-112). Registry-visible via a `Label` (the inspector/about pattern), so the
  // field-by-field CDP verify reads painted text, never the getter (N-097).
  const metaText = (p: PluginDescriptor) => `${p.host} · ${p.delivery} · ${p.surface}`;
  const rid = (p: PluginDescriptor, s: string) => cid(`${p.id}__${s}`);
</script>

<div class="plugin-list" data-tier="widget" use:envelope={{ name: 'plugin-list', id, debug }}>
  <ul class="plugin-list-items">
    {#each plugins as p (p.id)}
      <li class="plugin-list-row">
        <div class="pl-row-head">
          <span class="pl-name"><Label text={p.name} id={rid(p, 'name')} /></span>
          <!-- [system] / [user] badge (Ch6 §6.8.5): W-13 made visible. `[user]` reserved for a future
            `kind: custom` plugin — no such plugin exists today, so only `[system]` renders. -->
          <span class="pl-badge" data-kind={p.kind}>[{p.kind === 'system' ? 'system' : 'user'}]</span>
        </div>
        {#if p.description}
          <span class="pl-desc"><Label text={p.description} id={rid(p, 'desc')} /></span>
        {/if}
        <span class="pl-meta"><Label text={metaText(p)} id={rid(p, 'meta')} /></span>
      </li>
    {/each}
  </ul>
</div>
