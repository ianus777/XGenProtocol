<script>
  // app_sampler.svelte — SAMPLER shell (M-RP3.0 scaffold). Plain-JS app shell
  // (bare $state, no TS annotations — the N-041 gotcha). Its ONLY jobs in v0:
  //   1) prove the pipeline — import a real `$core` component, let `envelope`
  //      register it in window.__XGEN_DEBUG__, so CDP on 9422 can read it;
  //   2) the runtime client<->node skin-swap mechanism (D-098).
  // M-RP3.1 replaces the single smoke cell with the class x phase matrix of all 10.
  import { onMount } from 'svelte';
  import Button from '$core/components/data-independent/button.svelte';

  // Skin-swap: flipping [data-shell] on <html> re-aliases --accent* live (app.css),
  // so every skinned component re-themes at once. Replaces "run in both real shells".
  let shell = $state('client');

  function applyShell(s) {
    shell = s;
    document.documentElement.dataset.shell = s;
  }
  function swapShell() {
    applyShell(shell === 'client' ? 'node' : 'client');
  }

  onMount(() => applyShell('client'));
</script>

<div class="sampler-bar">
  <span class="sampler-title">XGen Sampler</span>
  <span class="sampler-shell-tag">accent: {shell}</span>
  <button class="sampler-swap" onclick={swapShell}>swap skin →</button>
</div>

<div class="sampler-body">
  <!-- v0 smoke cell — proves $core import + envelope + the debug registry work
       end-to-end in this new app. Registry id is `button#smoke` (envelope keys by
       component type, not app name). M-RP3.1 grows this into the full matrix. -->
  <div class="sampler-smoke">
    <span class="sampler-smoke-label">button#smoke</span>
    <Button label="Sample button" id="smoke" />
  </div>
</div>

<style>
  /* The skin-swap control is a sampler tool affordance, NOT a sampled component,
     so it is deliberately a plain styled button (kept out of the .button skin). */
  .sampler-swap {
    background: var(--s4);
    color: var(--t);
    border: 1px solid var(--s5);
    border-radius: var(--rad);
    padding: 4px 10px;
    font-size: 12px;
    cursor: pointer;
  }
  .sampler-swap:hover { background: var(--s5); }
</style>
