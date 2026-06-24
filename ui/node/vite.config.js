import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { fileURLToPath } from 'node:url';

// When running via `tauri dev`, TAURI_DEV_HOST is set.
const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [svelte()],
  resolve: {
    alias: {
      $common: fileURLToPath(new URL('../common/lib', import.meta.url)),
      $core: fileURLToPath(new URL('../core/lib', import.meta.url)),
      $assets: fileURLToPath(new URL('../assets', import.meta.url)),
    },
  },
  clearScreen: false,
  build: {
    // Output outside Google Drive — same pattern as CARGO_TARGET_DIR.
    outDir: 'C:/cargo-targets/XGenProtocol/node-dist',
    emptyOutDir: true,
  },
  server: {
    host: host || false,
    port: 5174,
    strictPort: true,
    hmr: host
      ? { protocol: 'ws', host, port: 5184 }
      : undefined,
    watch: {
      // Skip watching the Tauri src-tauri/ directory to avoid noise.
      ignored: ['**/src-tauri/**'],
    },
  },
});
