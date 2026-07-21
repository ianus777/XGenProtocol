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
  // the host: the sampler passes nothing → live-only (the pure layer, I/O stubbed). The live in-app
  // effect stays store-mediated: substitutions.setRules(draft) updates the $common store → every
  // processor-host re-morphs. Seed comes from substitutions.source (Step-A additive) — no second read.
  //
  // TWO CHANNELS, ONE PER HOST (M-RP-PROCESSOR-WIRE Leg A, §3.3). The client no longer passes
  // `onApply`: the Settings modal mounts this widget through D-120's GENERIC, PROP-LESS `<C />`, so
  // there is no prop to pass. Its channel is the STORE'S persist seam, filled once by the shell at
  // boot beside the existing `get_substitutions` read. `onApply` is KEPT, not replaced — it is the
  // sampler's live-only channel (D-097 / W-8), and the two are invoked independently so neither is
  // load-bearing on the other.
  //
  // PHASE-LIMIT (W-8): write-back durability is HOST-DEPENDENT, and the closing note DERIVES from the
  // seam rather than asserting a constant. In the client, Leg C carries the user's rules across
  // clean-slate-on-start, so they survive a restart. In the sampler, `main.rs` still clean-slates, so
  // they do not. One component, two hosts, opposite truths — a fixed string would be a lie in one of
  // them, and a note derived from the mechanism cannot drift out of true.
  //
  // GETTER (W-4 / 1b): one aggregate {dirty, valid, count} — observable task-state, never
  // the rules payload. The child textarea self-registers as `<id>__rules` (N-054).
  import { envelope } from '$common/components/base/envelope';
  import { substitutions } from '$common/components/processor/store.svelte';
  import {
    parseRules,
    assertSafeRules,
    findUnreachableRules,
  } from '$common/components/processor/transform';
  import TextArea from '$core/components/data-independent/textarea.svelte';
  import Button from '$core/components/data-independent/button.svelte';

  let {
    // Defaulted (M-RP-PROCESSOR-WIRE Leg A): the Settings modal mounts this through D-120's prop-less
    // `<C />`, so no id reaches it there — and `use:envelope` then falls back to a MODULE-LEVEL ORDINAL
    // that changes on every open/close cycle, making the registry unenumerable across drives. The
    // default gives the widget and its three children stable ids in the generic mount; the sampler
    // passes `id="demo"` explicitly and is unaffected. (grid-plate-settings solves the same problem by
    // hardcoding its control's id.)
    id = 'substitutions-editor',
    rows = 6,
    onApply,
  }: {
    id?: string;
    rows?: number;
    /** The SAMPLER's live-only persistence channel (imperative one-shot). The client uses the store's
     *  persist seam instead — it has no prop to pass through the generic settings mount. */
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

  // Reachability diagnostic (M-RP-PROCESSOR-SEED Leg D, D-100 ④) — SEPARATE from `validation`
  // above, and that separation is the decision, not a layout choice. `assertSafeRules` throws and
  // `setRules` fails safe to an EMPTY list, so folding this in would cost a user with one shadowed
  // pair their entire rule set in order to report it. This never gates Apply: a shadowed rule is
  // not invalid, it is unreachable, and it is the user's data to fix. Reuses `parsed` — no third
  // parse of the same draft.
  const unreachable = $derived(findUnreachableRules(parsed));

  async function apply() {
    if (!dirty || !validation.valid) return;
    substitutions.setRules(draft); // live in-app effect (also sets source → dirty clears)
    // Two channels, invoked INDEPENDENTLY (§3.3: neither is load-bearing on the other). A single
    // try around both would make a failing persist swallow the sampler's callback and vice versa.
    try {
      await substitutions.persist(draft); // the client's channel — no-op when no host is attached
    } catch {
      // Durable write failed: the in-app effect already applied; the write is the host's to report.
    }
    try {
      await onApply?.(draft); // the sampler's channel — absent in the client (no prop to pass)
    } catch {
      // As above.
    }
  }

  function revert() {
    draft = substitutions.source;
  }

  // The closing note DERIVES from the persist seam (§4 Leg A amendment) — never a constant, never a
  // host branch. Seam filled (client) ⇒ Leg C carries the rules across clean-slate-on-start; seam
  // unfilled (sampler) ⇒ the config is still wiped every launch. WORDING IS PROVISIONAL → M-RP-SKIN.
  const note = $derived(
    substitutions.durable
      ? 'Changes are saved and applied on the next start.'
      : 'Changes apply this session; config resets on restart.',
  );

  // `shadowed` is a COUNT — task-state, never the payload (W-4; §4.3 admits a field on exactly
  // that condition). It exists so "no diagnostics" is a number a probe can READ rather than an
  // element it fails to find: an absent node and a broken selector are the same clean-looking
  // nothing (N-110/N-139), and this is the one field that tells them apart.
  const debug = () =>
    $state.snapshot({ dirty, valid: validation.valid, count, shadowed: unreachable.length });
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

  <!-- Reachability notice (Leg D). Its own block, deliberately NOT inside `.subs-status`'s
    valid/invalid branch: this is not a validation state, and it must be able to appear while the
    set is perfectly valid and Apply is enabled. WORDING + APPEARANCE PROVISIONAL → M-RP-SKIN. -->
  {#if unreachable.length}
    <ul class="subs-shadow" aria-live="polite">
      {#each unreachable as u (u.find)}
        <li>“{u.find}” can’t be typed — “{u.shadowedBy}” changes first.</li>
      {/each}
    </ul>
  {/if}

  <div class="subs-actions">
    <Button label="Apply" onclick={apply} disabled={!dirty || !validation.valid} id={cid('apply')} />
    <Button label="Revert" onclick={revert} disabled={!dirty} id={cid('revert')} />
  </div>

  <p class="subs-note">{note}</p>
</div>
