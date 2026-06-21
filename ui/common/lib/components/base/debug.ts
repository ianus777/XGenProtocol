// Dev-only component-state registry (N-024). Read by the CDP debug harness
// (`cdp-debug.ps1`) over `window.__XGEN_DEBUG__`. Compiled out of production:
// every caller guards behind `import.meta.env.DEV`, so this module and its
// imports tree-shake away in a release build (it has no top-level side effects).

type Getter = () => unknown;

interface Entry {
  type: string; // N-020 type-class
  get: Getter;
}

type Reading = { type: string; state?: unknown; error?: string } | null;

const registry = new Map<string, Entry>();

function readOne(id: string): Reading {
  const entry = registry.get(id);
  if (!entry) return null;
  try {
    return { type: entry.type, state: entry.get() };
  } catch (err) {
    // Isolation: one throwing getter must not blind the whole snapshot.
    return { type: entry.type, error: String(err) };
  }
}

interface DebugApi {
  ids(): string[];
  get(id: string): Reading;
  snapshot(): Record<string, Reading>;
}

function ensureInstalled(): void {
  const w = window as unknown as { __XGEN_DEBUG__?: DebugApi };
  if (w.__XGEN_DEBUG__) return;
  w.__XGEN_DEBUG__ = {
    ids: () => [...registry.keys()],
    get: (id: string) => readOne(id),
    snapshot: () =>
      Object.fromEntries([...registry.keys()].map((id) => [id, readOne(id)])),
  };
}

/** Register a component's live state-getter under its debug id (dev only). */
export function register(id: string, type: string, get: Getter): void {
  ensureInstalled();
  registry.set(id, { type, get });
}

/** Remove a component from the registry (called on the action's destroy). */
export function unregister(id: string): void {
  registry.delete(id);
}
