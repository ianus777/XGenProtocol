import { mount } from 'svelte';
import './app.css';
import App from './app_client.svelte';

const app = mount(App, {
  target: document.getElementById('app'),
});

export default app;
