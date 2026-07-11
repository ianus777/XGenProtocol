// self-state.svelte.ts — the R3 Self/connection store (M-RP6.1g, D3). ONE channel, TWO views: the shell
// WRITES it (the EXISTING `xgen-client-state-changed` listen + `get_state` + a once-on-mount
// `get_self_state` invoke), and BOTH the frame's `status-bar` AND the `self-panel` widget READ it — so the
// connection signal has exactly ONE source, literally two views (D3, the whole point).
//
// A `$common` store because the `self-panel` widget lives in `ui/common/lib/components/widgets/`, and W-3
// forbids a `common` widget importing a shell dep. The leaf mount (`region-node`) passes a widget ONLY its
// `regionId` — a widget CANNOT receive shell props — so anything the widget needs is store-mediated. A
// shell-local map would be structurally UN-consumable. Hence the relocation below is FORCED, not stylistic.
//
// A `.svelte.ts` module so its module-level `$state` participates in Svelte 5 reactivity (the
// `selection.svelte.ts` / `processor/store.svelte.ts` precedent).

// ── Connection lifecycle → colour (RELOCATED verbatim from app_client.svelte, D3) ─────────────────────────
// The widget needs this map and cannot receive it as a prop; the shell reads the SAME map for the
// status-bar. ONE map, two views. ALL 11 client lifecycle states enumerated — `led`'s unknown sentinel is
// BLACK (#000000), so an unenumerated state changes colour VISIBLY rather than falling back silently. No
// fallback branch (Rule 5 / the `led` contract). This is a PURE relocation — the values are byte-identical
// to the pre-move constant (proven by re-measuring the status-bar led colour before/after, V7).
export const STATE_COLOURS: Record<string, string> = {
  SETUP: 'var(--t4)',            CLOSING: 'var(--t4)',
  INITIALISING: 'var(--t3)',
  CONNECTING: 'var(--inf)',      AUTHENTICATING: 'var(--inf)',     RECONNECTING: 'var(--inf)',
  READY: 'var(--ok)',
  DEGRADED_AUTH: 'var(--pr)',    DEGRADED_FEDERATION: 'var(--pr)', DEGRADED_NODE: 'var(--pr)',
  DISCONNECTED: 'var(--err)',
};
export const PULSING_STATES = ['INITIALISING', 'CONNECTING', 'AUTHENTICATING', 'RECONNECTING'];

/**
 * The `get_self_state` command's return shape (desktop.rs::SelfStateInfo). Carried VERBATIM —
 * snake_case as Rust serialises it, no rename/mapping layer (a mapping layer is a drift surface; D3).
 * `registered:false` + a real keypair-derived `identity_id` is the honest unregistered render (D6/W-8).
 */
export interface SelfStateInfo {
  registered: boolean;
  identity_id: string | null;
  display_name: string | null;
  home_node: string | null;
  spaces_joined: number;
}

/** The connection view — the `xgen-client-state-changed` / `get_state` payload subset the two views read. */
export interface Connection {
  state: string;
  label: string;
}

let _connection = $state<Connection>({ state: 'INITIALISING', label: 'Initialising' });
let _identity = $state<SelfStateInfo | null>(null);

export const selfState = {
  /** The live connection lifecycle. Reactive — status-bar + self-panel both read this. */
  get connection(): Connection {
    return _connection;
  },
  /** The self-identity block, or null before the first `get_self_state` returns. */
  get identity(): SelfStateInfo | null {
    return _identity;
  },
  /** Written by the shell from the `xgen-client-state-changed` listen and `get_state` (D1 — no new emit). */
  setConnection(payload: Connection): void {
    _connection = payload;
  },
  /** Written by the shell from the once-on-mount `get_self_state` invoke. */
  setIdentity(payload: SelfStateInfo): void {
    _identity = payload;
  },
};

// DEV-only CDP handle (mirrors __XGEN_SEL__ / __XGEN_SUBS__, N-024), so the verify pass can read both
// facets. Dead-code-eliminated in a production build.
if (import.meta.env.DEV && typeof window !== 'undefined') {
  (window as unknown as { __XGEN_SELF__?: unknown }).__XGEN_SELF__ = selfState;
}
