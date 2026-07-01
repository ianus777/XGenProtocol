// configs.ts — named Tier-1 (trusted code) transform configs for kind 1. Each pair is
// CONVERGENT (a `replace` never contains its `find`), so they also pass the untrusted
// convergence lint — i.e. they are safe to expose even as default/settings-backed rules.
// Authoring rule (the common-config review gate): keep every pair convergent.

import type { TransformConfig } from './transform';

/** ASCII arrows -> Unicode arrows. */
export const arrowMorph: TransformConfig = [
  { find: '-->', replace: '→' },
  { find: '<--', replace: '←' },
  { find: '=>', replace: '⇒' },
];

/** ASCII emoticons -> emoji. */
export const emojiMorph: TransformConfig = [
  { find: ':)', replace: '🙂' },
  { find: ':(', replace: '🙁' },
  { find: '<3', replace: '❤️' },
];
