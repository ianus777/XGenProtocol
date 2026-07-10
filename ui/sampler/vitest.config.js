// vitest.config.js — the first standing UI unit harness (M-RP6.1c, D4-V1). Lives on the sampler
// package because it already aliases `$common` and is the component test-bed; the suites it runs
// are the pure `$common` value-objects (keymap/*.test.ts today; the M-RP6.1f descriptor→layout
// walk next). Separate from vite.config.js (the Svelte build) so `npm test` and `vite build` don't
// interfere. Pure Node environment — no DOM, no Svelte plugin.
import { defineConfig } from 'vitest/config';
import { fileURLToPath } from 'node:url';

export default defineConfig({
  resolve: {
    alias: {
      $common: fileURLToPath(new URL('../common/lib', import.meta.url)),
    },
  },
  // The suites live in ../common (outside this package's root), so widen Vite's fs allow-list to
  // the repo root or Vite refuses to load them ("Failed to load url … Does the file exist?").
  server: {
    fs: {
      allow: [fileURLToPath(new URL('../..', import.meta.url))],
    },
  },
  test: {
    include: ['../common/lib/**/*.test.ts'],
    environment: 'node',
  },
});
