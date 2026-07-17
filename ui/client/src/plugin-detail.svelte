<script>
  // plugin-detail — SHELL-LOCAL (M-RP-SETTINGS Leg B). The `info` verb's feeder: a thin, honest view of a
  // single plugin's descriptor, shown IN the Settings content pane (settings-dialog swaps the Plugins
  // section to this when a row's [info] fires — the seam is `onAction(id,'info')`). It RENDERS THE
  // DESCRIPTOR AND NOTHING MORE (the inspector-panel discipline, J-501): no invented fields, no second
  // Rust projection — the descriptor is the whole truth about a plugin, so an "info" over it is a thin
  // thing and this refuses to inflate it.
  //
  // Rows reuse the About `<dl>` shape (keys <dt>, values a composed core `Label` so the CDP field-by-field
  // read lands on PAINTED TEXT, not a getter — N-097). The Back button + the plugin name TITLE live in the
  // shared Settings header now (Joe: Back and the round × close on one line), so this view is just the
  // field grid. Appearance is PROVISIONAL → M-RP-SKIN.
  import Label from '$core/components/data-independent/label.svelte';
  import { envelope } from '$common/components/base/envelope';

  let { plugin, id = 'plugin-detail' } = $props();
  const cid = (s) => (id ? `${id}__${s}` : undefined);

  // The descriptor fields, in a stable order. `icon`/`regionId`/`component`/`settingsComponent` are internal
  // wiring (not user-facing facts), so they are deliberately NOT shown — the visible set is what identifies
  // and classifies the plugin (Kind carries the built-in/installed fact the [system]/[user] badge used to).
  // `rowCount` is render-truth (the About/inspector precedent).
  const rows = $derived(
    plugin
      ? [
          { key: 'Name', value: plugin.name },
          ...(plugin.version ? [{ key: 'Version', value: plugin.version }] : []),
          { key: 'Id', value: plugin.id },
          { key: 'Kind', value: plugin.kind },
          { key: 'Host', value: plugin.host },
          { key: 'Delivery', value: plugin.delivery },
          { key: 'Surface', value: plugin.surface },
          ...(plugin.description ? [{ key: 'Description', value: plugin.description }] : []),
        ]
      : [],
  );

  const debug = () => ({ pluginId: plugin?.id ?? null, rowCount: rows.length });
</script>

<div class="plugin-detail" use:envelope={{ name: 'plugin-detail', id, debug }}>
  <dl class="pd-grid">
    {#each rows as r (r.key)}
      <dt>{r.key}</dt>
      <dd><Label text={r.value} id={cid(r.key.toLowerCase())} /></dd>
    {/each}
  </dl>
</div>
