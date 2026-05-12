<script>
    import AppLogo from "./assets/logo_proto_01_node_hda_64.png";
    import Button from "./lib/Button.svelte";
    async function handleExit() {
        try {
            console.log('Shutdown XG Node executed (simulated in browser)');
        } catch (error) {
            console.error('Failed to shut down:', error);
        }
    }
</script>

<main id="core-ui-pane">
    <img id="app-logo" src={AppLogo} alt="XGen Client logo"/>
    <p id="core-ui-text">The XGen Node application core test UI</p>
    <Button text="Shut Down" app="node" onAction={handleExit}/>
</main>
