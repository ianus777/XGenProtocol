// data-dependent (dd) shared types. A dd component materializes a DOMAIN view-model —
// NOT the raw protocol type. `core` never imports `IdentityRecord`/`SpaceState` (keeps
// the GPL reference lib protocol-free; N-057 source-agnostic). The SHELL owns the
// protocol → descriptor map; the dd owns kind → presentation.

/**
 * `EntityDescriptor` — a record projection of ONE address-book entry (an identity, a space,
 * or a room — a location peer to space), source-agnostic. This IS the W-11 dd-socket payload
 * (`temperature-indicator`
 * and future dd-consumers plug into the same slot shape).
 *
 * - `kind` stays on the descriptor so the dd still BRANCHES on it (a domain view-model,
 *   not a presentation one — `{shape,initials,colour}` would demote it to a di).
 * - `name` is `Option` on both source records (IdentityRecord.display_name /
 *   SpaceState.name) → a fallback is mandatory when absent.
 * - `image` is RESERVED-UNFED — no source record carries an image today; the slot exists,
 *   honestly empty (D-065). `entity-avatar` v1 never reads it.
 * - `flags` carries the two kind-bool/status inputs per source: identity → `isAi`/`revoked`;
 *   space → `isDm`/`e2e`. `e2e` is reserved; `entity-avatar` v1 does not draw an e2e lock.
 */
export interface EntityFlags {
  isAi?: boolean;
  revoked?: boolean;
  isDm?: boolean;
  e2e?: boolean;
}

export interface EntityDescriptor {
  kind: 'identity' | 'space' | 'room'; // room = a location peer to space (M-RP5.0c, hexagon)
  name?: string;
  id: string;
  flags?: EntityFlags;
  image?: string; // reserved-unfed (D-065)
}
