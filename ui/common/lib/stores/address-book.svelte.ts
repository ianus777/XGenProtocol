// address-book.svelte.ts — the R7 Members store (M-RP-MEMBERS Leg B). ONE source, read by the
// `members-panel` widget (R7): the resolved membership of the latched Space's conversation, plus the
// address-book cache that resolves each member's `identity_id` → display name.
//
// A `$common` store because `members-panel` lives in `ui/common/lib/components/widgets/`, and W-3 forbids
// a `common` widget importing a shell dep. The leaf mount (`region-node`) passes a widget ONLY its
// `regionId`, so anything the widget needs is store-mediated — a shell-local map would be structurally
// UN-consumable. This is the `self-state.svelte.ts` / `spaces-state.svelte.ts` shape exactly.
//
// A PURE `$state` store FED BY THE SHELL — no `.svelte.ts` in this codebase runs its own effect, and W-3
// forbids importing `invoke` here (there are ZERO `@tauri-apps` imports under `ui/common`). The shell's
// `$effect` watches `roomLatch.effectiveSpaceId` and drives the fill (the roomLatch/gaps feeder shape),
// calling the setters below. The invoke lives in the shell; this store only holds state.
//
// ⚠️ THE LATE-RESPONSE GUARD LIVES HERE (§3.5): `setResult`/`setFailed` take the `spaceId` they were fired
// for and DISCARD themselves if the scope has moved on since (a stale resolve landing under a new heading
// is the §4a divergence). `setInflight` writes `_spaceId` FIRST, so it is the guard's reference.

/** One member row — the `MembersResult.members` element (ops.rs `MemberEntry`). Carried VERBATIM,
 *  snake_case as Rust serialises it, no rename/mapping layer (the drift-surface rule; spaces-state
 *  precedent). A MIRROR of the Rust serialisation, not an import of a Rust type (W-3). NO name field —
 *  the name is resolved from `book` (below). `role`/`joined_at` arrive free and are deliberately
 *  discarded by the widget (L10). The ONE non-mirror field is `unresolved` — a frontend-only marker, see
 *  its own doc; it is stamped locally and never present on a row that came off the wire. */
export interface MemberEntry {
  identity_id: string;
  role: string;
  joined_at: string;
  invited_by: string | null;
  /** FRONTEND-ONLY, NOT part of the Rust serialisation (M-RP-LIVEFEED-REFRESH Leg A / runbook §5). Stamped
   *  `true` by `addMember` on a member that reached the roster via a LIVE membership delta — i.e. the
   *  address book was never consulted for them through a fill. Absent (`undefined`) on every fill-sourced
   *  member. `toDescriptor` branches on it to OMIT the `isAi` claim: an absent book record must render
   *  UNKNOWN, never DEFINITELY-NOT-AN-AI (the N-097 trap, inverted). Cleared naturally by the next fill,
   *  which replaces the roster wholesale with un-marked Rust rows. */
  unresolved?: boolean;
}

/** The `fill_space_records` command's `roster` half (ops.rs `MembersResult`). `is_dm` drives the L16
 *  DM-only counterpart highlight — no new plumbing (it rides here already). */
export interface MembersResult {
  space_id: string;
  is_dm: boolean;
  owner_id: string;
  members: MemberEntry[];
  events_replayed: number;
}

/** The `fill_space_records` command's `fill` half (ops.rs `FillReport`) — observability counts. Not read
 *  by R7 v1, but part of the `FillMembersOutcome` the shell receives (Leg A-quater). */
export interface FillReport {
  candidates: number;
  fetched: number;
  not_found: number;
  touched: number;
}

/** The `fill_space_records` command return (ops.rs `FillMembersOutcome`, Leg A-quater). Named struct, not
 *  a positional tuple — the shell reads `.roster`, not `.1`. */
export interface FillMembersOutcome {
  fill: FillReport;
  roster: MembersResult;
}

/** One address-book cache entry (address_book.rs `SeenRecord`). The book is `#[serde(transparent)]` over a
 *  `BTreeMap`, so `get_address_book` returns the bare map. Only `display_name` + `is_ai` are read by R7 v1:
 *  the name resolution and the `isAi` flag (L9 — `revoked` SHIPS UNFED). */
export interface SeenRecord {
  identity_id: string;
  display_name?: string;
  is_ai: boolean;
  home_node: string;
  last_seen: string;
  update_version: number;
  revoked: boolean;
}

/** The `get_address_book` command return — the bare `identity_id → SeenRecord` map. */
export type AddressBook = Record<string, SeenRecord>;

/** `idle` before any scope · `inflight` while a fill is in flight · `ready` on resolve · `failed` on reject.
 *  ⚠️ `phase` is NOT knowledge — knowledge is `roster !== null` (§3, L5). `phase` exists only to separate
 *  state ③ (inflight) from ④ (failed) when the roster is unknown. Do NOT derive "roster known" from it. */
export type MembersPhase = 'idle' | 'inflight' | 'ready' | 'failed';

let _spaceId = $state<string | null>(null);
let _isDm = $state(false);
let _roster = $state<MemberEntry[] | null>(null);
let _book = $state<AddressBook>({});
let _phase = $state<MembersPhase>('idle');

export const addressBook = {
  /** The scope this state describes (the Space the roster is FOR). The widget compares this against
   *  `roomLatch.effectiveSpaceId` so a lagging roster from a previous scope never renders under a new
   *  heading (belt to the setter guards' braces, §3.5). */
  get spaceId(): string | null {
    return _spaceId;
  },
  /** L16 — whether the latched conversation is a DM (drives the counterpart highlight). */
  get isDm(): boolean {
    return _isDm;
  },
  /** L3 — the roster, `null` = NOT KNOWN. An empty array is unreachable (L5: if you are scoped to a room
   *  you are in it), so `null` and `[]` are NOT the same render. */
  get roster(): MemberEntry[] | null {
    return _roster;
  },
  /** The address-book cache — the widget resolves each member's `identity_id` → `display_name` here. */
  get book(): AddressBook {
    return _book;
  },
  /** Context + elapsed, NOT knowledge (see `MembersPhase`). */
  get phase(): MembersPhase {
    return _phase;
  },

  /** Behaviour 1 — a new scope. Writes `_spaceId` FIRST (the late-guard reference), clears the roster to
   *  `null` (unknown), phase → `inflight`. */
  setInflight(spaceId: string): void {
    _spaceId = spaceId;
    _roster = null;
    _phase = 'inflight';
  },
  /** Behaviour 2 — resolve. ⚠️ Late-guard (§3.5): a resolve whose `spaceId` no longer matches the current
   *  scope is DISCARDED — without it, switching rooms twice quickly renders Space A's roster under Space
   *  B's heading. */
  setResult(spaceId: string, roster: MembersResult, book: AddressBook): void {
    if (spaceId !== _spaceId) return; // late-response guard
    _isDm = roster.is_dm;
    _roster = roster.members;
    _book = book ?? {};
    _phase = 'ready';
  },
  /** Behaviour 3 — reject. Roster stays `null` (unknown); ③ → ④ is a transition, not a separate render.
   *  Same late-guard as `setResult`. */
  setFailed(spaceId: string): void {
    if (spaceId !== _spaceId) return; // late-response guard
    _phase = 'failed';
  },
  /** Behaviour 4 — no scope. Reset to idle; no invoke fired by the shell in this case. */
  reset(): void {
    _spaceId = null;
    _isDm = false;
    _roster = null;
    _phase = 'idle';
  },

  // ── Live membership deltas (M-RP-LIVEFEED-REFRESH Leg A) ──────────────────────────────────────────
  // Fed by the shell's `xgen-event` router (app_client.svelte), NOT by the fill. Three rules the router
  // cannot enforce from outside the store live HERE (runbook §3); the fourth (R2 — resolving WHICH
  // identity the event is about, which is NOT uniformly `sender`) is the router's:
  //   R1 — a delta onto an UNKNOWN roster (`_roster === null`) is DROPPED, never promoted. Adding to
  //        `null` would yield a one-member roster the panel cannot tell from a real one — *"I do not know
  //        who is here"* must not become a confident lie.
  //   R3 — idempotent: add-existing and remove-absent are both no-ops. The drain makes no exactly-once
  //        promise and the client has no replay suppression of its own.
  //   R4 — the scope guard is `_spaceId` (written FIRST by `setInflight`, the late-response reference) vs
  //        the event's `space_id`, exactly as `setResult`/`setFailed` discard on mismatch. A delta for a
  //        Space the user is not scoped to must not touch the on-screen roster. Guarding here (not in the
  //        router) keeps ONE scope authority for the store (the D-067 drift a second one would open).
  /** Add a live-joined member. Stamps the `unresolved` marker (§5): this member reached the roster via a
   *  delta, so the book was never consulted for them. No-op if the roster is unknown (R1), if the scope has
   *  moved on (R4), or if the member is already present (R3). Reassigns a fresh array (Svelte 5 $state). */
  addMember(spaceId: string, entry: MemberEntry): void {
    if (spaceId !== _spaceId) return; // R4 — scope guard
    if (_roster === null) return; // R1 — no delta onto an unknown roster
    if (_roster.some((m) => m.identity_id === entry.identity_id)) return; // R3 — idempotent add
    _roster = [..._roster, { ...entry, unresolved: true }];
  },
  /** Remove a member the live channel says has left or been removed. No-op if the roster is unknown (R1),
   *  if the scope has moved on (R4), or if the member is absent (R3 — not an error). */
  removeMember(spaceId: string, identityId: string): void {
    if (spaceId !== _spaceId) return; // R4 — scope guard
    if (_roster === null) return; // R1 — no delta onto an unknown roster
    if (!_roster.some((m) => m.identity_id === identityId)) return; // R3 — idempotent remove (no-op if absent)
    _roster = _roster.filter((m) => m.identity_id !== identityId);
  },
};

// DEV-only CDP handle (mirrors __XGEN_SPACES__ / __XGEN_SELF__, N-024), so the verify pass can drive the
// store to each of the five panel states and read them back. Dead-code-eliminated in a production build.
if (import.meta.env.DEV && typeof window !== 'undefined') {
  (window as unknown as { __XGEN_MEMBERS__?: unknown }).__XGEN_MEMBERS__ = addressBook;
}
