<script>
    import AppLogo from "./assets/logo_proto_01_client_hda_64.png";
    import Button from "./lib/Button.svelte";
    async function handleExit() {
        try {
            console.log('Quit XG Client executed (simulated in browser)');
        } catch (error) {
            console.error('Failed to quit:', error);
        }
    }
</script>

<main id="core-ui-pane">
    <img id="app-logo" src={AppLogo} alt="XGen Client logo"/>
    <p id="core-ui-text">The XGen Client application core test UI</p>
    <Button text="Quit" app="client" onAction={handleExit}/>
</main>
