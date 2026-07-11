<script>
  // about-dialog — SHELL-LOCAL (M-RP6.1e-C3, D1). The client's About box: it wraps the core
  // `dialog` (C1) and renders the `get_about_info` payload (C2) + the client logo. It lives in
  // ui/client/ — NOT ui/core/ — because it renders client-specific data (ClientAboutInfo, the
  // "XGen Client" name, the CLIENT logo); `core` stays app-agnostic (the link/menu-bar rule).
  //
  // No <style> block (A1): the type-class comes from `envelope` on the composed components, and
  // the About-specific grid layout is shell chrome → it lives in ui/client/src/app.css keyed
  // `.about-*` (N-031: app.css is "shell chrome ONLY"). This file composes; app.css dresses.
  //
  // Values are core `label`s with stable `about-*` ids (A3): a <label> naming no control is a
  // deliberate semantic stretch, kept because putting each value in the registry is exactly what
  // makes the field-by-field CDP verify possible (plain text would be unreadable by the harness).
  import Dialog from '$core/components/data-independent/dialog.svelte';
  import Image from '$core/components/data-independent/image.svelte';
  import Link from '$core/components/data-independent/link.svelte';
  import Label from '$core/components/data-independent/label.svelte';
  import logoUrl from '$assets/logo_client_hda.png';

  let { open = $bindable(false), info = null, onOpenLink } = $props();
  // info = ClientAboutInfo | null (null in browser-dev / pre-fetch → em-dashes, never a crash).
  const c = $derived(info?.common ?? null);

  // Built + commit render TOGETHER (frame §6 / C2 §2.4): `built` is the last compile, `commit`
  // is what exactly identifies the build. One row, middot-joined, one registry entry.
  const builtText = $derived(c ? `${c.built} · ${c.commit}` : '—');
</script>

<Dialog bind:open title="About XGen Client" id="about">
  <div class="about">
    <div class="about-head">
      <Image src={logoUrl} alt="XGen" id="about-logo" />
      <div class="about-id">
        <span class="about-name"><Label text={c?.name ?? '—'} id="about-name" /></span>
        <!-- F2: company, never the personal name — a static shell literal, not an AboutInfo field. -->
        <span class="about-by">Developed by Alchemy Dump</span>
        <!-- A2: link types href AND text as REQUIRED; passing an undefined c.link would strip the
             href and trip link's own no-accessible-name DEV warn. Render conditionally. -->
        {#if c?.link}
          <Link href={c.link} text={c.link} external={true} onclick={onOpenLink} id="about-link" />
        {:else}
          <Label text="—" id="about-link" />
        {/if}
      </div>
    </div>

    <dl class="about-grid">
      <dt>Version</dt>
      <dd><Label text={c?.version ?? '—'} id="about-version" /></dd>
      <dt>Built</dt>
      <dd><Label text={builtText} id="about-built" /></dd>
      <dt>Rust</dt>
      <dd><Label text={c?.rustc ?? '—'} id="about-rustc" /></dd>
      <dt>Tauri</dt>
      <dd><Label text={c?.tauri ?? '—'} id="about-tauri" /></dd>
      <dt>Svelte</dt>
      <dd><Label text={c?.svelte ?? '—'} id="about-svelte" /></dd>
      <dt>Platform</dt>
      <dd><Label text={c?.platform ?? '—'} id="about-platform" /></dd>
      <dt>App dir</dt>
      <dd><Label text={c?.app_dir ?? '—'} id="about-app-dir" /></dd>
      <dt>Data dir</dt>
      <dd><Label text={c?.data_dir ?? '—'} id="about-data-dir" /></dd>
      <dt>Config</dt>
      <dd><Label text={c?.config_path ?? '—'} id="about-config" /></dd>
    </dl>
  </div>
</Dialog>
