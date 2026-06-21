<script>
  export let text = 'Shut Down';
  export let onAction = () => {};
  export let app = 'node';
</script>

<div class="button-pane">
  <button class="primary-{app}-button" on:click={onAction}>{text}</button>
</div>
