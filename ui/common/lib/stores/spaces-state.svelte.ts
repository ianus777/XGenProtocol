// spaces-state.svelte.ts — the R1/R2 known-Space store (M-RP6.2, D1). ONE source, TWO readers: the
// `spaces-panel` widget (R1) lists the Spaces, and the `rooms-panel` widget (R2) reads the SELECTED Space's
// embedded `.rooms` from the SAME payload (there is no second fetch — the whole tree comes in one
// `get_spaces` read, D1). The shell writes it once on mount (`get_spaces`); both widgets read it.
//
// A `$common` store because both widgets live in `ui/common/lib/components/widgets/`, and W-3 forbids a
// `common` widget importing a shell dep. The leaf mount (`region-node`) passes a widget ONLY its `regionId`,
// so anything a widget needs is store-mediated — a shell-local map would be structurally UN-consumable. This
// is the `self-state.svelte.ts` shape exactly.
//
// A `.svelte.ts` module so its module-level `$state` participates in Svelte 5 reactivity (the
// `selection.svelte.ts` / `self-state.svelte.ts` precedent).

/**
 * The `get_spaces` command's element shape (desktop.rs → `xgen_common::state::KnownSpace`). Carried
 * VERBATIM — snake_case as Rust serialises it, no rename/mapping layer (a mapping layer is a drift surface;
 * the `SelfStateInfo` precedent). This is a MIRROR of the Rust serialisation, NOT an import of a Rust type,
 * so `core` still imports no protocol type (W-3). `rooms` rides EMBEDDED — R2 reads them from here (D1).
 */
export interface KnownRoom {
  room_id: string;
  name: string;
  joined: boolean;
}
export interface KnownSpace {
  space_id: string;
  name: string;
  node_endpoint: string;
  /** "owner" | "admin" | "moderator" | "member" — no descriptor slot yet (v1 unfed, D6). */
  role: string;
  rooms: KnownRoom[];
}

let _spaces = $state<KnownSpace[]>([]);

export const spacesState = {
  /** The known-Space tree, or `[]` before the first `get_spaces` returns / for an unregistered client.
   *  Reactive — both `spaces-panel` (the list) and `rooms-panel` (the selected Space's rooms) read this. */
  get spaces(): KnownSpace[] {
    return _spaces;
  },
  /** Written by the shell from the once-on-mount `get_spaces` invoke. `[]` is the honest unregistered
   *  render (W-8) — R1 shows its "No spaces yet" empty state, no faked rows (N-091). */
  setSpaces(list: KnownSpace[]): void {
    _spaces = list ?? [];
  },
};

// DEV-only CDP handle (mirrors __XGEN_SELF__ / __XGEN_SEL__, N-024), so the verify pass can read the
// hydrated payload directly. Dead-code-eliminated in a production build.
if (import.meta.env.DEV && typeof window !== 'undefined') {
  (window as unknown as { __XGEN_SPACES__?: unknown }).__XGEN_SPACES__ = spacesState;
}
