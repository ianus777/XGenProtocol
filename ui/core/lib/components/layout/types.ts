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

/** A region id references a region-owning widget by its durable id (region-dock §2). */
export type RegionId = string;

export type LayoutNode =
  | { type: 'leaf'; widgetId: RegionId }
  | { type: 'split'; dir: 'row' | 'col'; sizes: number[]; children: LayoutNode[] }
  | { type: 'tabs'; active: number; children: LayoutNode[] };

export interface Layout {
  version: number;
  root: LayoutNode;
}
