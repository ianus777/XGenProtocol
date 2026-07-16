<script>
  // about-content — SHELL-LOCAL (M-RP-SETTINGS Leg A). The About BODY, extracted verbatim from
  // about-dialog so it can be mounted in TWO places (S-2 "one component, two mounts"): the Help ▸ About
  // modal (about-dialog, unchanged) AND the Settings modal's About section. The Dialog wrapper stays in
  // about-dialog — this file is just the body, so it can be nested inside another modal (Settings) that
  // is itself already a <dialog> (a <dialog> cannot nest a <dialog>).
  //
  // `idPrefix` keys every composed value's registry id. about-dialog passes the default 'about' → the
  // exact ids the C3 verify measured (about-name … about-config), so that milestone's proof is preserved
  // byte-for-byte. Settings passes a DISTINCT prefix ('settings-about') so the two simultaneously-mounted
  // instances (both modals are always-mounted, closed = display:none) never collide on data-debug-id.
  //
  // No <style> (N-025): the `.about-*` presentation lives in skin.css (N-090), shared by both mounts.
  import Image from '$core/components/data-independent/image.svelte';
  import Link from '$core/components/data-independent/link.svelte';
  import Label from '$core/components/data-independent/label.svelte';
  import logoUrl from '$assets/logo_client_hda.png';

  let { info = null, onOpenLink, idPrefix = 'about' } = $props();
  // info = ClientAboutInfo | null (null in browser-dev / pre-fetch → em-dashes, never a crash).
  const c = $derived(info?.common ?? null);

  // Built + commit render TOGETHER (frame §6 / C2 §2.4): `built` is the last compile, `commit`
  // is what exactly identifies the build. One row, middot-joined, one registry entry.
  const builtText = $derived(c ? `${c.built} · ${c.commit}` : '—');

  const pid = (s) => `${idPrefix}-${s}`;
</script>

<div class="about">
  <div class="about-head">
    <Image src={logoUrl} alt="XGen" id={pid('logo')} />
    <div class="about-id">
      <span class="about-name"><Label text={c?.name ?? '—'} id={pid('name')} /></span>
      <!-- F2: company, never the personal name — a static shell literal, not an AboutInfo field. -->
      <span class="about-by">Developed by Alchemy Dump</span>
      <!-- A2: link types href AND text as REQUIRED; passing an undefined c.link would strip the
           href and trip link's own no-accessible-name DEV warn. Render conditionally. -->
      {#if c?.link}
        <Link href={c.link} text={c.link} external={true} onclick={onOpenLink} id={pid('link')} />
      {:else}
        <Label text="—" id={pid('link')} />
      {/if}
    </div>
  </div>

  <dl class="about-grid">
    <dt>Version</dt>
    <dd><Label text={c?.version ?? '—'} id={pid('version')} /></dd>
    <dt>Built</dt>
    <dd><Label text={builtText} id={pid('built')} /></dd>
    <dt>Rust</dt>
    <dd><Label text={c?.rustc ?? '—'} id={pid('rustc')} /></dd>
    <dt>Tauri</dt>
    <dd><Label text={c?.tauri ?? '—'} id={pid('tauri')} /></dd>
    <dt>Svelte</dt>
    <dd><Label text={c?.svelte ?? '—'} id={pid('svelte')} /></dd>
    <dt>Platform</dt>
    <dd><Label text={c?.platform ?? '—'} id={pid('platform')} /></dd>
    <dt>App dir</dt>
    <dd><Label text={c?.app_dir ?? '—'} id={pid('app-dir')} /></dd>
    <dt>Data dir</dt>
    <dd><Label text={c?.data_dir ?? '—'} id={pid('data-dir')} /></dd>
    <dt>Config</dt>
    <dd><Label text={c?.config_path ?? '—'} id={pid('config')} /></dd>
  </dl>
</div>
