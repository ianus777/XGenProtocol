<script lang="ts">
  // connection-stats — the FIRST `kind: 'custom'` widget (M-RP-CONNSTATS), and with it the runtime
  // install → dock → uninstall lifecycle. The plate (M-RP-PLATE) proved the containment/mount half; this
  // proves the harder install/uninstall half. The widget's DATA is deliberately thin — the lifecycle IS the
  // milestone. Level-2 marked via `data-tier="widget"` on the root (the self-panel / plugin-list shape).
  //
  // READS the ONE self-state store (D-067 — the same source the status-bar + self-panel read); it adds NO
  // channel and NO Rust. Live traffic (bytes/latency/uptime/reconnect) does NOT exist yet — that is the
  // M-RP6.6 resident, out of scope. So the seeded rows are ONLY what has a real source today.
  //
  // A METRIC-ROW LIST, not a fixed template (Joe: compact + extensible). Each row is a `{label, value}`
  // entry, so a future metric is one array entry, never a rewrite. Rows whose source is absent render
  // ABSENT (the `hasValue` precedent, N-060) — no fabricated placeholder rows (an unfed row is an unverified
  // branch, N-091). The State row additionally carries a `led` (the caller-supplied colour map, STATE_COLOURS).
  //
  // W-3: imports only `$common` (envelope, self-state) + composed `$core` display components (the self-panel /
  // inspector-panel precedent — a common widget composing core is fine; only a SHELL dep is forbidden). Zero
  // component-local CSS (N-090): `.connection-stats` lives in skin.css, PROVISIONAL → M-RP-SKIN (Joe owns the
  // look; this milestone owns the lifecycle).
  import { envelope } from '$common/components/base/envelope';
  import { selfState, STATE_COLOURS, PULSING_STATES } from '$common/stores/self-state.svelte';
  import Led from '$core/components/data-independent/led.svelte';
  import Label from '$core/components/data-independent/label.svelte';

  // The leaf mount (`region-node`) passes a widget ONLY `regionId` (grounded against region-node.svelte:227).
  // Derive the envelope id via the EXACT `region-placeholder` / self-panel convention `id = "region-" +
  // regionId` (N-096), so the install/uninstall delta reads as a clean enumerable subtree.
  let { regionId, id = `region-${regionId}` }: { regionId: string; id?: string } = $props();
  const cid = (s: string) => (id ? `${id}__${s}` : undefined);

  const conn = $derived(selfState.connection);
  const identity = $derived(selfState.identity);

  // The metric rows — real data only (§4.1). A row is one array entry; adding a future metric appends one.
  interface Row { key: string; label: string; value: string; led?: boolean }
  const rows = $derived.by<Row[]>(() => {
    const r: Row[] = [];
    // State — always present (a connection always has a lifecycle state); the only row with a led.
    r.push({ key: 'state', label: 'State', value: conn.label, led: true });
    const id = identity; // the `get_self_state` block; null before it returns / no-Tauri (browser dev)
    if (id) {
      r.push({ key: 'registered', label: 'Registered', value: id.registered ? 'Yes' : 'No' });
      // Endpoint — the home node (the self-panel's own `secondary` source, D-067); absent when unregistered.
      if (id.home_node) r.push({ key: 'endpoint', label: 'Endpoint', value: id.home_node });
      r.push({ key: 'spaces', label: 'Spaces joined', value: String(id.spaces_joined) });
    }
    return r;
  });

  // Aggregate getter G (W-4). `rowCount` is RENDER-TRUTH (the inspector `rowCount` / message `detailsCount`
  // precedent) — the count of rows ACTUALLY rendered, NOT an assertion that N future rows exist. This is what
  // keeps the compact/extensible growth CDP-observable and future rows honest.
  const debug = () => ({ state: conn.state, rowCount: rows.length });
</script>

<div class="connection-stats" data-tier="widget" use:envelope={{ name: 'connection-stats', id, debug }}>
  <dl class="connection-stats-grid">
    {#each rows as row (row.key)}
      <dt>{row.label}</dt>
      <dd>
        {#if row.led}
          <span class="cs-state">
            <Led states={STATE_COLOURS} state={conn.state} pulse={PULSING_STATES.includes(conn.state)} id={cid('state-led')} />
            <Label text={row.value} id={cid(row.key)} />
          </span>
        {:else}
          <Label text={row.value} id={cid(row.key)} />
        {/if}
      </dd>
    {/each}
  </dl>
</div>
