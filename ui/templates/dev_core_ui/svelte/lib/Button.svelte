<script>
    export let text = 'Exit';
    export let onAction = () => {};
    export let app = 'client'
</script>

<div class="button-pane">
    <button class="primary-{app}-button" on:click={onAction}>{text}</button>
</div>
