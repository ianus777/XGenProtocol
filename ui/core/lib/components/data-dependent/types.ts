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

/**
 * Message family dd-socket (M-RP5.5, `docs/xgen-dd-message-family-phase0.md` v1.0). Mirrors
 * `EntityDescriptor`: source-agnostic, `core` imports no protocol type — the SHELL owns the
 * protocol → descriptor map, the `message` dd owns kind → presentation.
 *
 * Two v1 TYPES (`kind`): `text` (avatar both sides + author name + body + details) and `system`
 * (authorless centered notice). grouped / edited / deleted are STATES (fields on `text`), not
 * kinds — same discipline as `EntityDescriptor.revoked` being a flag, not a kind.
 */
export type MessageKind = 'text' | 'system';

/**
 * A declared widget placement inside a `message` socket (`details` / `bodyExtras`). The all-widgets
 * model (W-12): the message is a HOST SURFACE for system/custom widgets. `widgetId` is the DURABLE
 * identity (J-475) — a mount whose id the host cannot resolve is DROPPED on render (W-13 reconcile),
 * so a stale/uninstalled widget never crashes the row.
 */
export interface WidgetMount {
  widgetId: string;
  props?: Record<string, unknown>;
}

/**
 * `MessageDescriptor` — a record projection of ONE message. This IS the W-11 dd-socket payload for
 * the message family (`message` + the future `message-stream`, M-RP5.6).
 *
 * - `author` REUSES `EntityDescriptor` — the message avatar IS an entity-avatar of the author.
 *   Absent for `system` (authorless notice).
 * - `body` is a TEXT NODE only (never `{@html}`); the inline-mark/link formatter stays the deferred
 *   kind-4 `use:render` (D-065). `deleted` ignores `body`.
 * - `isOwn` is SHELL-SET — `core` never computes `author.id === self`; the message owns the mirror.
 * - `details` / `bodyExtras` are widget sockets (each a `WidgetMount[]`). `details` = the header
 *   region (time · temperature · badges · icon-buttons · send-status led); `bodyExtras` = below the
 *   body (attachments / reactions). `bodyExtras` is RESERVED-UNFED in v1 (D-065).
 */
export interface MessageDescriptor {
  kind: MessageKind;
  id: string; // event / ULID
  author?: EntityDescriptor; // absent for system; REUSES the entity dd-socket
  body?: string; // text-node only (never @html); deleted → ignored
  timestamp: string; // the message formats its own (like status "5m ago")
  isOwn?: boolean; // shell-set; flips avatar side + alignment
  edited?: boolean; // → "(edited)" marker (M-RP5.5 B)
  deleted?: boolean; // → tombstone render, body/details ignored (M-RP5.5 B)
  details?: WidgetMount[]; // header/details region widgets
  bodyExtras?: WidgetMount[]; // below body: attachments / reactions — reserved-unfed (D-065)
  // reserved-unfed (D-065): replyTo?
}
