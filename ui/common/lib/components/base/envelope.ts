// Component envelope (N-023): stamps the root type-class and merges pass-through
// classes, with a dev-only debug branch (N-024). Applied as a Svelte action:
//
//   <div use:envelope={'icon-button'}>                              // stateless
//   <div use:envelope={{ name: 'spaces-panel', id, debug: dbg }}>   // state-exposed
//
// The `./debug` import is referenced only inside the `import.meta.env.DEV` guard,
// so a production build dead-code-eliminates the branch and tree-shakes the import.

import { kebab, mergeClasses } from './logic';
import { register, unregister } from './debug';

let ordinal = 0;

export type EnvelopeParam =
  | string
  | { name: string; id?: string; debug?: () => unknown };

/**
 * Svelte action return shape, typed structurally so `common` stays framework-light
 * (no `svelte/action` import). Svelte's `use:` only requires this shape, not the type.
 */
interface ActionResult {
  destroy?(): void;
}

export function envelope(node: HTMLElement, param: EnvelopeParam): ActionResult {
  const { name, id, debug: getState } =
    typeof param === 'string'
      ? { name: param, id: undefined, debug: undefined }
      : param;

  const typeClass = kebab(name);
  node.className = mergeClasses(typeClass, node.className); // supplies, never erases (N-023)

  if (import.meta.env.DEV && getState) {
    const debugId = `${typeClass}#${id ?? ++ordinal}`; // N-011 stable id, else ordinal
    node.setAttribute('data-debug-id', debugId);
    register(debugId, typeClass, getState);
    return {
      destroy() {
        unregister(debugId);
      },
    };
  }

  return {};
}
