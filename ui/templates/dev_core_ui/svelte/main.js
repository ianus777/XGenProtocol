import { mount } from 'svelte'
import 'modern-normalize' // modern-normalize.css (mit license)
import './app.css'
import App from './app_client.svelte' // or './app_node.svelte'

const app = mount(App, {
  target: document.getElementById('app'),
})

export default app
