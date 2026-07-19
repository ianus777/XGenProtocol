<script>
  // Sampler-local FIXTURE stub widget (M-RP5.5 A). NOT a real widget — a minimal render target for
  // the `message` `details` widget socket, proving mount-by-widgetId resolution + drop-unknown
  // (W-13). It deliberately does NOT register with `envelope`: the socket is verified via
  // message#…'s `detailsCount` getter + DOM presence, so keeping the stub registration-free keeps
  // the registry delta per message cell predictable (message + composed atomics only).
  let { label = 'widget', tone = 'info' } = $props();
</script>

<!-- Appearance lives in `skin.css` (§ SAMPLER FIXTURE STUBS, M-RP6.9): it was duplicated here and in
  the registering twin, Svelte-scoped so neither could reuse the other, and the raw numbers read as
  arbitrary. One named source now — see that block for the tokens and for why the stub is a proxy
  rather than the thing. -->
<span class="fixture-widget" data-tone={tone}>{label}</span>
