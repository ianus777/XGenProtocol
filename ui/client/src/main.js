import { mount } from 'svelte';
import '$assets/modern-normalize.css';
import '$assets/xgen-normalize.css';
import '$assets/skin.css';
import './app.css';
import App from './app_client.svelte';

const app = mount(App, {
  target: document.getElementById('app'),
});

export default app;
