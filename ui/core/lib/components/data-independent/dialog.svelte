<script lang="ts">
  // dialog — data-independent, COMPOSITE (M-RP6.1e-C1): the 31st `core` component and a di
  // composite MODAL container (flagged as a gap since J-432). Root IS the native <dialog>.
  //
  // THE HEADLINE — native <dialog> + showModal() supplies TOP-LAYER stacking, ::backdrop, the
  // focus trap, background-inert, and Esc-dismiss FOR FREE. This is exactly why `dialog` does NOT
  // hand-roll a W-2 owned-popup behaviour machine (combobox / tag-select / color-picker /
  // entity-context-menu / menu each build one only because no native element gave it to them; this
  // one does). No machine here.
  //
  // MODAL-ONLY (v1). Open = el.showModal(); close = el.close(). There is deliberately NO `modal`
  // prop — a prop with exactly one legal value is a lie (D-065); non-modal (show(), no backdrop, no
  // trap) is deferred until a real consumer asks. And we NEVER set the `open` *attribute*: <dialog
  // open> is the non-modal path (no top layer, no backdrop, no trap) — it would silently give a
  // worse dialog that LOOKS like it works. The attribute is native-owned output; we drive the
  // element through its METHODS only. The verify proof is el.matches(':modal') — both paths report
  // open:true, only :modal separates showModal() from the attribute.
  //
  // COMPOSITE (N-054 multiply): it composes a real `button` child (the Close button), which
  // self-registers as `…__close` — so a `dialog` cell yields MULTIPLE registry entries. Precedent:
  // section (native root + header + children body slot) and status-indicator (composite-register).
  //
  // CHILDREN LIFETIME: native <dialog> closed = display:none, NOT unmounted. So `…__close` (and, in
  // C3, the About body) register on MOUNT and stay registered across open/close — the OPPOSITE of
  // menu-item (which mounts on popup-open). Consequence for verify: the registry count is STABLE
  // across open/close; only `open` flips.
  import { envelope } from '$common/components/base/envelope';
  import Button from './button.svelte';
  import type { Snippet } from 'svelte';

  let {
    title = '',
    open = $bindable(false),
    closeLabel = 'Close',
    onClose,
    id,
    children,
  }: {
    /** Header title. */
    title?: string;
    /** Open state ($bindable). Drives showModal()/close() — see the reconciliation below. */
    open?: boolean;
    /** Close-button label. Default "Close" (Joe: not "OK"). */
    closeLabel?: string;
    /** Fired after the dialog has closed, on ANY path (button, Esc). */
    onClose?: () => void;
    id?: string;
    /** Body content. */
    children?: Snippet;
  } = $props();

  // The native element. `open` is read off THIS for the getter (the DOM truth), not off the prop.
  let el: HTMLDialogElement;

  // Composite-derived stable child id (so the self-registering Close button reads cleanly).
  const cid = (s: string) => (id ? `${id}__${s}` : undefined);

  // OPEN-STATE RECONCILIATION (the one non-obvious axis). Native <dialog> owns its own open state
  // and Esc fires cancel -> close WITHOUT consulting the prop, so a naive $bindable would lie the
  // moment the user presses Esc. Two rules close it:
  //
  //   1. Prop -> element. This $effect calls showModal()/close(). GUARDED (`el.open === open` ->
  //      return) so the effect and the close handler never ping-pong.
  $effect(() => {
    if (!el) return;
    if (el.open === open) return; // element already agrees — nothing to do
    if (open) el.showModal();
    else el.close();
  });

  //   2. Element -> prop. The native `close` event fires on EVERY close path (button + Esc); this
  //      handler writes `open = false` BACK into the binding — that is what keeps `open` honest (it
  //      is not optional polish). `cancel` (Esc, preventable) is NOT intercepted; listening to
  //      `close` alone covers every path. onClose fires once the state has settled.
  function handleClose() {
    if (open) open = false;
    onClose?.();
  }

  // Close button just asks to close via the prop; the $effect + close handler do the rest — one
  // path, no direct el.close() from here.
  function requestClose() {
    open = false;
  }

  // Getter G. `open` MUST be read from the DOM (el.open), not the prop: the registry reports what
  // RENDERED, not what we intended — a desync must surface, not be papered over (Rule 1 in
  // component form). `title` is a string primitive, so no $state.snapshot needed.
  const debug = () => ({ title, open: el?.open ?? false });
</script>

<dialog
  bind:this={el}
  onclose={handleClose}
  use:envelope={{ name: 'dialog', id, debug }}
>
  {#if title !== ''}
    <h2 class="dialog-title">{title}</h2>
  {/if}
  <div class="dialog-body">
    {@render children?.()}
  </div>
  <div class="dialog-footer">
    <Button label={closeLabel} id={cid('close')} onclick={requestClose} />
  </div>
</dialog>
