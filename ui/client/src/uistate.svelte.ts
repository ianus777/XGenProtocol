// uistate.svelte.ts — the client's UI-state store (M-RP6.1k). SHELL-LOCAL.
//
// Leg B: PERSISTENT. Hydrated from `xgen-client_uistate.json` via `get_ui_state` on start; every
// mutation writes back via `set_ui_state`. The Rust side round-trips the file as an OPAQUE string and
// never parses it (D-114) — THIS MODULE IS THE STORE'S ONLY PARSER.
//
// 🔑 UNKNOWN-KEY PRESERVATION (D-114, the milestone's core discipline). To guarantee a key a newer
// binary added survives a round-trip through this one, the store is held as the PARSED BLOB and mutated
// IN PLACE — never reconstructed from a lossy typed model. `session` and each named `state` are BAGS:
// we read/write the `layout` key today (`geometry` joins at Leg C) and leave every other key untouched,
// at the top level, the entry level, and the state level.
//
// RESERVE NOTHING: a UI state carries `layout` only until each further key (geometry / theme / shelf /
// room) grows a feeder in its own milestone (D-065). An unwritten key is an unverified key.
//
// A `.svelte.ts` module so its module-level `$state` participates in Svelte 5 reactivity (the
// self-state.svelte / selection.svelte precedent — those are `$common`; this one is shell-local).

import type { Layout } from '$core/components/layout/types';

/** A saved arrangement — `layout` today; `geometry` at Leg C. Extra keys are preserved verbatim. */
export interface UiStateBag {
  layout?: Layout;
  [k: string]: unknown;
}
interface NamedEntry {
  name: string;
  updated_at: string;
  state: UiStateBag;
  [k: string]: unknown;
}
interface Store {
  version: number;
  session?: UiStateBag | null;
  named: Record<string, NamedEntry>;
  active?: string | null;
  [k: string]: unknown;
}

const EMPTY = (): Store => ({ version: 1, session: null, named: {}, active: null });

let _store = $state<Store>(EMPTY());
let _seq = 0; // monotonic id source; seeded past existing ids on hydrate
let _hydrated = false;

async function tauriInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const { invoke } = await import('@tauri-apps/api/core');
  return invoke<T>(cmd, args);
}

// Fire-and-forget READ-MODIFY-WRITE: mutations persist without blocking the UI. We re-read the file
// and merge our authoritative parts (`named` / `active` / `version` / `session.layout`) over it, so a
// save never clobbers keys THIS module does not own — above all `session.geometry`, which RUST writes
// (D-114/N-107) between our reads — nor any unknown key a newer binary added. The `session` merge is
// PER-KEY (M-RP7.5, N-107): read the on-disk session bag FRESH, spread it, override `layout` only — a
// whole-`session` write would eat the geometry Rust just wrote. A write error (or the no-Tauri browser
// dev preview) leaves the session in memory — honest (W-8), not silently pretended-persisted.
async function persist(): Promise<void> {
  try {
    let onDisk: Record<string, unknown> = {};
    try {
      const raw = await tauriInvoke<string>('get_ui_state');
      if (raw && raw.trim()) {
        const p = JSON.parse(raw);
        if (p && typeof p === 'object' && !Array.isArray(p)) onDisk = p;
      }
    } catch {
      // corrupt/no-Tauri on read → start the merge from {}; the write below still runs.
    }
    // Per-key merge INSIDE `session` (N-107): preserve geometry (Rust's) + any unknown session key;
    // override `layout` only. `layout` undefined (pre-first-mutation) ⇒ NO `session` key written ⇒
    // geometry untouched and loadLayout still DEFAULTs (guarded below). `onDisk.session` is read fresh
    // above, so a geometry Rust wrote since our last read survives.
    const onDiskSession =
      onDisk.session && typeof onDisk.session === 'object' ? onDisk.session : {};
    const layout = _store.session?.layout;
    const merged = {
      ...onDisk,
      version: 1,
      named: _store.named,
      active: _store.active,
      ...(layout ? { session: { ...onDiskSession, layout } } : {}),
    };
    await tauriInvoke('set_ui_state', { json: JSON.stringify(merged) });
  } catch (e) {
    console.error('ui-state persist failed', e);
  }
}

// M-RP7.5 Leg A — the SESSION writer's debounce. Session mutations (fold/resize/move) are already
// discrete commits (resize writes once on pointerup, J-519), so this only coalesces rapid sequences
// into a single write. Routes through persist() → one N-107-correct write path, never a second.
let _sessionTimer: ReturnType<typeof setTimeout> | null = null;
function scheduleSessionPersist(): void {
  if (_sessionTimer) clearTimeout(_sessionTimer);
  _sessionTimer = setTimeout(() => {
    _sessionTimer = null;
    persist();
  }, 400);
}

function seedSeq(): void {
  _seq = Object.keys(_store.named).reduce((m, k) => {
    const n = parseInt(k.replace(/^s/, ''), 10);
    return Number.isFinite(n) && n > m ? n : m;
  }, 0);
}

export const uiStateStore = {
  /**
   * Load the persisted store from disk ONCE (M-RP6.1k, Leg B). A corrupt / absent / no-Tauri payload
   * leaves the empty in-memory store — the layout still renders via loadLayout()'s DEFAULT fallback
   * (N-095). Preserves unknown top-level keys (`...parsed`) before hardening the `named` map.
   */
  async hydrate(): Promise<void> {
    if (_hydrated) return;
    _hydrated = true;
    try {
      const raw = await tauriInvoke<string>('get_ui_state');
      if (raw && raw.trim()) {
        const parsed = JSON.parse(raw);
        if (parsed && typeof parsed === 'object' && !Array.isArray(parsed)) {
          _store = {
            ...EMPTY(),
            ...parsed,
            named: parsed.named && typeof parsed.named === 'object' ? parsed.named : {},
          };
          seedSeq();
        }
      }
    } catch (e) {
      // Corrupt store → keep the empty in-memory store; loadLayout falls back to DEFAULT (N-095).
      console.warn('ui-state hydrate: unreadable store, starting empty', e);
    }
  },

  get active(): string | null {
    return _store.active ?? null;
  },

  /** Named entries as a list, most-recently-updated first — what both dialogs render. */
  list(): Array<{ id: string; name: string; updated_at: string; state: UiStateBag }> {
    return Object.entries(_store.named)
      .map(([id, e]) => ({ id, name: e.name, updated_at: e.updated_at, state: e.state }))
      .sort((a, b) => String(b.updated_at).localeCompare(String(a.updated_at)));
  },

  /** The active entry's name, or '' — pre-fills the Save dialog. */
  activeName(): string {
    return (_store.active && _store.named[_store.active]?.name) || '';
  },

  /** The session arrangement (Leg D writes it on exit); null until then. */
  session(): UiStateBag | null {
    return _store.session ?? null;
  },

  /**
   * Save `state` under `name` and PERSIST. Overwriting a same-named entry reuses its id and preserves
   * that entry's unknown keys — spread the previous entry, then the previous state, then override with
   * `state` (which carries `layout` today). New name → a fresh id.
   */
  save(name: string, state: UiStateBag): string {
    const existing = Object.entries(_store.named).find(([, e]) => e.name === name);
    const id = existing ? existing[0] : `s${++_seq}`;
    const prevEntry = existing ? _store.named[id] : undefined;
    const prevState = prevEntry?.state ?? {};
    _store.named[id] = {
      ...(prevEntry ?? {}), // preserve unknown ENTRY-level keys on overwrite
      name,
      updated_at: new Date().toISOString(),
      state: { ...prevState, ...state }, // preserve unknown STATE-level keys, override layout
    };
    _store.active = id;
    persist();
    return id;
  },

  /** Load a named entry's arrangement, mark it active + PERSIST the active change; null if id is gone. */
  load(id: string): UiStateBag | null {
    const e = _store.named[id];
    if (!e) return null;
    _store.active = id;
    persist();
    return e.state;
  },

  /** Delete a named entry and PERSIST. */
  remove(id: string): void {
    delete _store.named[id];
    if (_store.active === id) _store.active = null;
    persist();
  },

  /**
   * M-RP7.5 Leg A — feed the live SESSION arrangement (fold/resize/move). Debounced per-key write: the
   * `layout` key only, merged into whatever `session` already holds (geometry stays Rust's — N-107).
   * `loadLayout()` reads this back on the next launch. NEVER a whole-`session` write.
   */
  setSessionLayout(layout: Layout): void {
    _store.session = { ...(_store.session ?? {}), layout };
    scheduleSessionPersist();
  },
};

// DEV-only CDP handle (mirrors __XGEN_SELF__ / __XGEN_SEL__, N-024). Dead-code-eliminated in a build.
if (import.meta.env.DEV && typeof window !== 'undefined') {
  (window as unknown as { __XGEN_UISTATE__?: unknown }).__XGEN_UISTATE__ = uiStateStore;
}
