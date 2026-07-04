<script lang="ts">
  // substitutions-editor — the FIRST `widget` (M-RP4.3), the Level-2 UI-plugin tier
  // (ui/docs/xgen-widget-tier.md v1.0, D-102). Home ui/common/lib/components/widgets/.
  //
  // WIDGET, not a composite (N-059 discriminator): it owns task-state with a lifecycle
  // that persists across renders — a `draft` buffer that DIVERGES from its source, a
  // `dirty` flag, live validation — and performs host I/O (persist). Removing it loses a
  // behaviour, not a layout. Level-2 marked via `data-tier="widget"` on the root so `ids()`
  // and the sampler WIDGET tab partition it from composites.
  //
  // SHAPE (Joe-lock, Phase-0): ONE textarea holding the raw " | " rules string (D-100
  // 1:1-with-TOML) — no per-pair rows, no stringifyRules this instance. Explicit Apply /
  // Revert gated on `dirty && valid`.
  //
  // I/O SEAM (W-3 / 1c — callback injection, the FIRST-INSTANCE FINDING). A widget lives in
  // `common`, which must NOT hard-depend on a shell dep: a bare `invoke` import inside the widget
  // fails to resolve outside a Tauri host (caught at M-RP4.3 build). So persistence is an INJECTED
  // callback `onApply` (the imperative-one-shot fallback in the spec's I/O-seam menu), passed by
  // the host: the real client shell passes an `invoke('set_substitutions', …)`-backed callback; the
  // sampler passes nothing → live-only (the pure layer, I/O stubbed). The live in-app effect stays
  // store-mediated: substitutions.setRules(draft) updates the $common store → every processor-host
  // re-morphs. Seed comes from substitutions.source (Step-A additive) — no second read.
  //
  // PHASE-LIMIT (W-8, D-101): write-back is SESSION-ONLY this phase — clean-slate-on-start
  // wipes the config to seed every launch. Surfaced in-UI; NOT a silent gap.
  //
  // GETTER (W-4 / 1b): one aggregate {dirty, valid, count} — observable task-state, never
  // the rules payload. The child textarea self-registers as `<id>__rules` (N-054).
  import { envelope } from '$common/components/base/envelope';
  import { substitutions } from '$common/components/processor/store.svelte';
  import { parseRules, assertSafeRules } from '$common/components/processor/transform';
  import TextArea from '$core/components/data-independent/textarea.svelte';
  import Button from '$core/components/data-independent/button.svelte';

  let {
    id,
    rows = 6,
    onApply,
  }: {
    id?: string;
    rows?: number;
    /** Host-injected persistence (imperative one-shot). Absent → live-only (pure layer). */
    onApply?: (rules: string) => void | Promise<void>;
  } = $props();

  const cid = (s: string) => (id ? `${id}__${s}` : undefined);

  // Local draft — the divergent-from-source buffer (the widget's task-state, W-2).
  let draft = $state<string>('');

  // Seed once from the live source (Step-A additive). Guarded so async store hydration
  // (the sampler/client invoke lands AFTER child mount) seeds the draft when it arrives,
  // without ever clobbering an edit the user already started.
  let seeded = false;
  $effect(() => {
    if (!seeded && substitutions.source !== '') {
      draft = substitutions.source;
      seeded = true;
    }
  });

  const parsed = $derived(parseRules(draft));
  const count = $derived(parsed.length);
  const dirty = $derived(draft !== substitutions.source);

  // Live Tier-2 validation (caps + convergence lint) — the inline-warning source.
  const validation = $derived.by(() => {
    try {
      assertSafeRules(parseRules(draft), { trusted: false });
      return { valid: true, msg: '' };
    } catch (e) {
      return { valid: false, msg: e instanceof Error ? e.message : String(e) };
    }
  });

  async function apply() {
    if (!dirty || !validation.valid) return;
    substitutions.setRules(draft); // live in-app effect (also sets source → dirty clears)
    try {
      await onApply?.(draft); // host-injected persist; absent in the sampler (live-only, D-097/W-8)
    } catch {
      // Persist failed / no host: the in-app effect already applied; durable write is the host's.
    }
  }

  function revert() {
    draft = substitutions.source;
  }

  const debug = () => $state.snapshot({ dirty, valid: validation.valid, count });
</script>

<div
  class="substitutions-editor"
  data-tier="widget"
  use:envelope={{ name: 'substitutions-editor', id, debug }}
>
  <TextArea bind:value={draft} {rows} id={cid('rules')} placeholder="find replace | find replace" />

  <div class="subs-status" aria-live="polite">
    {#if validation.valid}
      <span class="subs-count">{count} rule{count === 1 ? '' : 's'}</span>
    {:else}
      <span class="subs-warn">{validation.msg}</span>
    {/if}
  </div>

  <div class="subs-actions">
    <Button label="Apply" onclick={apply} disabled={!dirty || !validation.valid} id={cid('apply')} />
    <Button label="Revert" onclick={revert} disabled={!dirty} id={cid('revert')} />
  </div>

  <p class="subs-note">Changes apply this session; config resets on restart.</p>
</div>
