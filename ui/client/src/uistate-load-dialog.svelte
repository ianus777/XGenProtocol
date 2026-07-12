<script>
  // uistate-load-dialog — SHELL-LOCAL (M-RP6.1k). The Load UI-state box: wraps the core
  // `dialog` (C1) and drives the shell-local `uiStateStore`. Same posture as `uistate-save-dialog` —
  // composes core, dresses via `.uistate-*` in skin.css (N-090), carries no <style>.
  //
  // The picker is a `combobox` (Joe): open the dropdown and pick an existing saved state; Load / Delete
  // then act on the selection. `combobox` carries the picked NAME as its value; `selected` resolves that
  // back to the entry id (names are unique — save overwrites a same-named entry).
  import Dialog from '$core/components/data-independent/dialog.svelte';
  import Combobox from '$core/components/data-independent/combobox.svelte';
  import Button from '$core/components/data-independent/button.svelte';
  import { uiStateStore } from './uistate.svelte';

  // onLoad(id): the shell applies the loaded arrangement to its `layout` (the load seam). Delete is
  // fully in-store (no shell state to touch), so the dialog owns it.
  let { open = $bindable(false), onLoad } = $props();

  let pickName = $state('');
  let confirmingDelete = $state(false); // Delete needs a second click (destructive)
  let wasOpen = false;

  const entries = $derived(uiStateStore.list());
  const optionNames = $derived(entries.map((e) => e.name));
  // The picked name resolves to an entry id (or null if the field text matches nothing yet).
  const selected = $derived(entries.find((e) => e.name === pickName.trim())?.id ?? null);

  // Reset the picker + guard each time the dialog opens.
  $effect(() => {
    if (open && !wasOpen) {
      pickName = '';
      confirmingDelete = false;
    }
    wasOpen = open;
  });

  // Changing the selection cancels a pending delete-confirm.
  $effect(() => {
    selected;
    confirmingDelete = false;
  });

  function doLoad() {
    if (!selected) return;
    onLoad?.(selected);
    open = false;
  }

  function doDelete() {
    if (!selected) return;
    if (!confirmingDelete) {
      confirmingDelete = true;
      return;
    }
    uiStateStore.remove(selected);
    pickName = '';
    confirmingDelete = false;
  }
</script>

<Dialog bind:open title="Load UI state" closeLabel="Cancel" id="uistate-load">
  <!-- tabindex=-1 + autofocus so showModal() lands initial focus HERE, not on the combobox input
    (which would open its dropdown on dialog-open). The combobox stays collapsed until clicked; the
    first Tab still reaches it. -->
  <!-- svelte-ignore a11y_autofocus -->
  <div class="uistate" tabindex="-1" autofocus>
    {#if entries.length === 0}
      <p class="uistate-note">No saved UI states yet.</p>
    {:else}
      <Combobox
        bind:value={pickName}
        options={optionNames}
        placeholder="Pick a saved state"
        id="uistate-load-pick"
      />

      {#if confirmingDelete}
        <p class="uistate-warn">Delete “{pickName.trim()}”? Click Delete again to confirm.</p>
      {/if}
    {/if}

    <!-- Owned action bar (the core footer is suppressed for .uistate). Order: Load · Delete · Cancel.
      Cancel is ALWAYS present so an empty box can still be dismissed; Load + Delete appear only when
      states exist. -->
    <div class="uistate-actions">
      {#if entries.length}
        <Button label="Load" onclick={doLoad} disabled={!selected} id="uistate-load-go" />
        <Button
          label={confirmingDelete ? 'Confirm delete' : 'Delete'}
          onclick={doDelete}
          disabled={!selected}
          id="uistate-load-del"
        />
      {/if}
      <Button label="Cancel" onclick={() => (open = false)} id="uistate-load-cancel" />
    </div>
  </div>
</Dialog>
