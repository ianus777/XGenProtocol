<script>
    import AppLogo from "./assets/logo_proto_01_client_hda_64.png";
    import Button from "./lib/Button.svelte";
    async function handleExit() {
        try {
            console.log('Exit executed (simulated in browser)');
        } catch (error) {
            console.error('Failed to exit:', error);
        }
    }
</script>

<main id="core-ui-pane">
    <img id="app-logo" src={AppLogo} alt="XGen Client logo"/>
    <p id="core-ui-text">The XGen Client application core test UI</p>
    <Button text="Exit" app="client" onAction={handleExit}/>
</main>
