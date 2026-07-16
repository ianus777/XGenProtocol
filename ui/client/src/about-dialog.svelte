<script>
  // about-dialog — SHELL-LOCAL (M-RP6.1e-C3, D1). The client's Help ▸ About box: it wraps the core
  // `dialog` (C1) and renders the About body (`get_about_info` payload C2 + the client logo). It lives in
  // ui/client/ — NOT ui/core/ — because it renders client-specific data (ClientAboutInfo, the "XGen
  // Client" name, the CLIENT logo); `core` stays app-agnostic (the link/menu-bar rule).
  //
  // The body moved out to `about-content.svelte` (M-RP-SETTINGS Leg A) so the same body renders in TWO
  // places — here, AND the Settings modal's About section (S-2 "one component, two mounts"). This file
  // keeps the Dialog wrapper + the default `idPrefix`, so the C3 verify's registry ids (about-name …
  // about-config) are byte-for-byte unchanged. Help ▸ About behaviour is untouched.
  import Dialog from '$core/components/data-independent/dialog.svelte';
  import AboutContent from './about-content.svelte';

  let { open = $bindable(false), info = null, onOpenLink } = $props();
</script>

<Dialog bind:open title="About XGen Client" id="about">
  <AboutContent {info} {onOpenLink} />
</Dialog>
