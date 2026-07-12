<script>
  // uistate-save-dialog — SHELL-LOCAL (M-RP6.1k). The Save UI-state box: it wraps the core
  // `dialog` (C1) exactly as `about-dialog` does, and drives the shell-local `uiStateStore`. Lives in
  // ui/client/ — NOT ui/core/ — because it composes shell state; `core` stays app-agnostic.
  //
  // The name field is a `combobox` (Joe): type a NEW name, or open the dropdown and pick an EXISTING
  // state to overwrite. The combobox's free-text input carries the name; its options are the existing
  // state names. This replaces the earlier textfield + inline list (more compact, one control).
  //
  // No <style> block: the type-class comes from `envelope` on the composed controls; the box layout is
  // shell chrome dressed in ui/assets/skin.css keyed `.uistate-*` (N-090). The store PERSISTS to
  // xgen-client_uistate.json (Leg B): saving here writes to disk + carries the window geometry (Leg D).
  import Dialog from '$core/components/data-independent/dialog.svelte';
  import Combobox from '$core/components/data-independent/combobox.svelte';
  import Button from '$core/components/data-independent/button.svelte';
  import { uiStateStore } from './uistate.svelte';

  // onSave(name): the shell captures the CURRENT arrangement and calls uiStateStore.save — the dialog
  // never reaches into the shell's `layout` itself (the loadLayout/about seam shape).
  let { open = $bindable(false), onSave } = $props();

  let name = $state('');
  let confirming = $state(false); // an existing name needs a second click to overwrite (destructive)
  let wasOpen = false;            // pre-fill only on the open TRANSITION, not on every store change

  // Pre-fill with the active state's name each time the dialog opens; reset the overwrite guard.
  $effect(() => {
    if (open && !wasOpen) {
      name = uiStateStore.activeName();
      confirming = false;
    }
    wasOpen = open;
  });

  const entries = $derived(uiStateStore.list());
  const optionNames = $derived(entries.map((e) => e.name)); // the combobox dropdown
  const trimmed = $derived(name.trim());
  const isOverwrite = $derived(entries.some((e) => e.name === trimmed));

  // Changing the name cancels a pending overwrite-confirm.
  $effect(() => {
    trimmed;
    confirming = false;
  });

  function attempt() {
    if (!trimmed) return;
    if (isOverwrite && !confirming) {
      confirming = true; // first click on an existing name → ask; second click → overwrite
      return;
    }
    onSave?.(trimmed);
    open = false;
  }
</script>

<Dialog bind:open title="Save UI state" closeLabel="Cancel" id="uistate-save">
  <!-- tabindex=-1 + autofocus so showModal() lands initial focus HERE, not on the combobox input
    (which would open its dropdown on dialog-open). The combobox stays collapsed until clicked; the
    first Tab still reaches it. -->
  <!-- svelte-ignore a11y_autofocus -->
  <div class="uistate" tabindex="-1" autofocus>
    <Combobox
      bind:value={name}
      options={optionNames}
      placeholder="Name this workspace, or pick one to overwrite"
      id="uistate-save-name"
    />

    {#if confirming}
      <p class="uistate-warn">“{trimmed}” exists — click Overwrite to replace it.</p>
    {/if}

    <!-- Owned action bar (the core footer is suppressed for .uistate). Save + Cancel on one line. -->
    <div class="uistate-actions">
      <Button
        label={confirming ? 'Overwrite' : 'Save'}
        onclick={attempt}
        disabled={!trimmed}
        id="uistate-save-go"
      />
      <Button label="Cancel" onclick={() => (open = false)} id="uistate-save-cancel" />
    </div>
  </div>
</Dialog>
