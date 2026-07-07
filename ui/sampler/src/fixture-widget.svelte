<script>
  // Sampler-local FIXTURE stub widget (M-RP5.5 A). NOT a real widget — a minimal render target for
  // the `message` `details` widget socket, proving mount-by-widgetId resolution + drop-unknown
  // (W-13). It deliberately does NOT register with `envelope`: the socket is verified via
  // message#…'s `detailsCount` getter + DOM presence, so keeping the stub registration-free keeps
  // the registry delta per message cell predictable (message + composed atomics only).
  let { label = 'widget', tone = 'info' } = $props();
</script>

<span class="fixture-widget" data-tone={tone}>{label}</span>

<style>
  .fixture-widget {
    display: inline-block;
    padding: 0 4px;
    border-radius: 4px;
    background: var(--s4);
    color: var(--t3);
    font-size: 10px;
    line-height: 1.4;
    white-space: nowrap;
  }
  .fixture-widget[data-tone='muted'] {
    opacity: 0.7;
  }
</style>
