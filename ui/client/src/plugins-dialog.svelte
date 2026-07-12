<script>
  // plugins-dialog — SHELL-LOCAL (M-RP6.1l, D4). The plugin list's FIRST entry point: it wraps the core
  // `dialog` (C1) exactly as `about-dialog` / `uistate-*-dialog` do, and mounts the `plugin-list` widget
  // ($common) as its body. Lives in ui/client/ — NOT ui/core/ — because it is the shell's chrome for the
  // gear command; `core` stays app-agnostic.
  //
  // THE SCOPE FORK (D4): Settings-the-window does not exist. This modal is the pane's first mount;
  // M-RP-SETTINGS becomes its SECOND mount (S-2 "one component, two mounts"), so nothing here is thrown
  // away — the `plugin-list` widget is the reusable part, this dialog is just the host for now.
  //
  // STOCK `dialog` footer (D9): a single Close button, no `:has()` suppression. The list is READ-ONLY,
  // so the stock one-button footer is exactly right — there is no action bar to own. (If a future need
  // ever reaches for the 6.1k `:has()` hack, that is the SECOND recurrence and the footer-snippet-slot
  // extraction becomes its own milestone — never a rider here.)
  import Dialog from '$core/components/data-independent/dialog.svelte';
  import PluginList from '$common/components/widgets/plugin-list.svelte';

  let { open = $bindable(false) } = $props();
</script>

<!-- Always mounted (closed = display:none), the About/UI-state modal posture; opened by the gear
  shelf face (widget.manager → pluginsOpen). -->
<Dialog bind:open title="Plugins" id="plugins">
  <PluginList id="plugin-list" />
</Dialog>
