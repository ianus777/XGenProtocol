<script lang="ts">
  // grid-plate-settings — the Grid Backdrop plugin's OWN settings component (M-RP-SETTINGS Leg C, D-B / B2).
  // The FIRST `settingsComponent` in the project: the Settings modal hosts it in its content pane
  // (component-per-plugin — NOT a declarative `settings_schema`, which is superseded, D-B/D-120). It renders
  // the control for the ONE backdrop value and WRITES the $common backdrop store; grid-plate reads that store
  // and PAINTS the flip (N-096 — one store, a writer here, a reader there; W-3 forbids either $common file
  // importing a shell store, so the store is the only shared channel).
  //
  // v1 = one boolean (B2): show the backdrop pattern, or a plain fill. The full backdrop-type menu (static /
  // generative / data-driven, base-vs-stack) is M-RP-BACKDROP. Composed from a `core` toggle (which registers
  // under a stable id, so the hosted control is CDP-enumerable while the settings drill-in is open); the LOOK
  // is Joe's — PROVISIONAL → M-RP-SKIN.
  //
  // W-3 holds: imports only `$common` (the backdrop store) and `$core` (toggle) — no shell / Tauri dep.
  import Toggle from '$core/components/data-independent/toggle.svelte';
  import { backdrop } from '$common/stores/backdrop.svelte';

  // The toggle binds a local mirror; the effect writes every change straight to the store. `checked` is
  // seeded once from the store (already hydrated by the shell at boot, before this pane can open), so it
  // opens showing the persisted choice. Assigning the SAME value to `$state` does not notify, so the initial
  // effect run is an inert no-op (no spurious persist / repaint); there is no read-back into `checked`, so no
  // feedback loop.
  let checked = $state(backdrop.pattern);
  $effect(() => {
    backdrop.setPattern(checked);
  });
</script>

<div class="grid-plate-settings">
  <label class="grid-plate-settings-row">
    <Toggle bind:checked shape="switch" id="grid-plate-settings__toggle" />
    <span>Show backdrop pattern</span>
  </label>
</div>
