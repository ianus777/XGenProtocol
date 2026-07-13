// Layout descriptor — the serializable region-tree contract (D-103, `ui/docs/xgen-region-dock-model.md`
// §3). This is the FULL type (`leaf | split | tabs`); it is the single structure BOTH renderers read:
// the config-grid renderer A (M-RP6.1f, this milestone) and the owned dock engine B (M-RP7). Typing the
// complete contract costs nothing and `version` is what a later schema change bumps (D1).
//
// `core` tier (D4): the NODE app inherits this renderer at M-RP7.x, so the contract + the walk live in
// the GPL reference lib, protocol-free — exactly the frame precedent (`menu-bar`/`status-bar` are `core`
// because both apps need them). No Tauri, no protocol import.
//
// Renderer A implements `leaf` + `split` ONLY. A `tabs` node is DROPPED by the resolver with a DEV warn
// (D1 / §3 rule 3) — no tab-strip code ships until it is consumed (D-065; N-091 — an unfed branch is an
// unproved branch). The type still carries `tabs` so the descriptor round-trips through A unchanged and B
// renders it later without a schema bump.
//
// M-RP7.1 (D5) — the FIRST schema change since D-103: `collapsed?` on a `leaf` (shipped v2 as a boolean).
//
// M-RP7.1b — the fold axis becomes the USER'S choice (Joe, 2026-07-13). `collapsed` stops being a boolean
// (axis DERIVED from the parent split) and becomes a `FoldAxis` — the DIRECTION the user picked. It names
// WHAT collapses ('width' → vertical strip / 'height' → horizontal stripe), NOT where the strip goes (the
// strip always parks at the tile's leading edge). Two axes, NOT four directions — a `'left' | 'top'`
// vocabulary would invite a `'right' | 'bottom'` later; naming the thing you mean stops the wrong extension
// being sayable. Still optional (absent ⇒ expanded), still a property of the TILE not the widget. This is
// the FIRST real schema change: `version: 3`, and the FIRST exercised migrate (`migrateLayout`, resolve.ts).

/** A region id references a region-owning widget by its durable id (region-dock §2). */
export type RegionId = string;

/** The dimension a folded tile collapses (M-RP7.1b). `'width'` ⇒ vertical strip at the leading edge;
 *  `'height'` ⇒ horizontal stripe at the top. The user picks it; it is not derived from the tree. */
export type FoldAxis = 'width' | 'height';

export type LayoutNode =
  | { type: 'leaf'; widgetId: RegionId; collapsed?: FoldAxis } // collapsed: boolean → FoldAxis (M-RP7.1b)
  | { type: 'split'; dir: 'row' | 'col'; sizes: number[]; children: LayoutNode[] }
  | { type: 'tabs'; active: number; children: LayoutNode[] };

export interface Layout {
  /** Descriptor schema version. Bumped to 3 at M-RP7.1b (leaf `collapsed`: boolean → `FoldAxis`); the
   *  v2 → v3 migrate is the first real one (`migrateLayout`, resolve.ts). */
  version: number;
  root: LayoutNode;
}
