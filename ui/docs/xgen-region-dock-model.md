# XGen UI — Region / Dock Model
> **Status**: ACTIVE  
> Version: 1.1  
> Date: Jul 2026  
> **Last updated**: 2026-07-07  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

The main client UI panel is a **layout of dockable regions**. This doc locks the model so the two renderers — a lean config-grid now (M-RP6.1+) and an owned Maya-style dock engine later (M-RP7) — read the **same** layout descriptor, making the dock engine a *renderer upgrade*, not a region rewrite. Crystallises into **D-103**. Extends the `widget` tier (`xgen-widget-tier.md`, D-102) with the region-provider seam (W-12) and the system-lock (W-13).

---

## 1. The unification — every region is a widget

There is no separate "region" concept. A **region is a widget that owns a dockable surface.** Widgets split into two kinds by a single flag:

- **`kind: system`** — the built-in surfaces (R1–R8). Pre-installed, **non-removable**, but individually configurable and redockable exactly like a custom widget.
- **`kind: custom`** — installable / removable widgets. A custom widget MAY also provide a region, so a newly-installed widget can contribute a brand-new dockable surface to the layout.

This closes the loop on D-102: a widget already plugs into a `$common` store (its *data* seam); it now may also contribute a *surface* (its *layout* seam). The di/dd × atomic/composite grid remains the **content** tier; **widgets are the dockable surfaces that host content.**

## 2. Region registry

A single registry holds all region-owning widgets: the static system set (R1–R8) plus any dynamic custom-widget regions. The dock engine references entries **by widget id only** and does not care about `kind`.

| id | Region | Hosts (content components) | kind |
|----|--------|----------------------------|------|
| `spaces` | R1 · Spaces rail | `entity-panel` (space) | system |
| `rooms` | R2 · Rooms | `entity-panel` (room→hexagon) | system |
| `self` | R3 · Self / connection | `entity-item` + `status` + `led` | system |
| `room-header` | R4 · Room header | `label` / `section` (+ temperature) | system |
| `stream` | R5 · Message stream | `message` (unbuilt) | system |
| `composer` | R6 · Composer | `textarea` + `button` | system |
| `members` | R7 · Members | `entity-panel` (identity) | system |
| `inspector` | R8 · Selection info | `section` + `label` rows | system |

## 3. Layout descriptor — the shared contract

A serializable tree. Leaves reference widgets by id; splits and tabs compose them. This single structure is the contract both renderers honour, and it round-trips to disk (save/restore layouts).

```
LayoutNode =
  | { type: "leaf",  widgetId: string }
  | { type: "split", dir: "row" | "col", sizes: number[], children: LayoutNode[] }
  | { type: "tabs",  active: number, children: LayoutNode[] }
Layout = { version: number, root: LayoutNode }
```

- **Config-grid renderer (A, M-RP6.1+):** renders a restricted subset — `split` only, fixed `sizes`, no runtime mutation. Rearranging = editing the descriptor (or a dev toggle).
- **Dock engine renderer (B, M-RP7):** renders the full tree, supports `tabs`, and **mutates the tree** on drag-drop (hover-to-plug-in) + splitter resize.

Because both read one descriptor, a widget appearing/disappearing = a node inserted/removed; a region moving = a subtree relocated. No component inside a region is aware of which renderer is active.

## 4. Region-provider seam (W-12)

A widget declares its region via its manifest:

```
WidgetManifest = {
  id: string,
  title: string,
  kind: "system" | "custom",
  providesRegion: true,          // W-12: a widget owns exactly one region
  mount: (host) => void,         // renders content into the docked surface
  defaultDock?: LayoutHint,      // where it lands in the default layout
  settings?: SettingsSchema      // per-widget config, same seam for system + custom
}
```

A custom widget that ships `providesRegion` inserts itself into the registry on install and into the layout at `defaultDock`; on uninstall its node is removed and the tree re-flows. System widgets are seeded into the default layout at build time and cannot be removed (W-13).

## 5. Selection bus

A shell primitive the regions share: **one active selection** across the layout `{ regionId, entity: EntityDescriptor }`. R8 (inspector) reads it to render the selected object's parameter rows; R1/R2 write it on row activation; `entity-context-menu` reads the same selection. Introduced with the region shell (M-RP6.1), consumed from M-RP6.2.

## 6. Constraint additions to the widget tier

- **W-12 — a widget owns exactly one region.** Every widget (system or custom) maps to exactly one dockable surface in the layout descriptor. (Promotes the earlier "MAY own a region" to the universal rule.)
- **W-13 — system widgets are non-removable.** `kind: system` widgets are pre-installed, always present in the default layout, and cannot be uninstalled or fully closed (they may collapse, redock, retab, and be configured). This prevents a user closing the Composer with no way back.

## 7. Renderer roadmap

- **M-RP6.1–6.5** — regions on live data via config-grid renderer **A**; each region built + verified as a system widget (Phase-0 + sampler + CDP). Reads the locked descriptor.
- **M-RP7 — owned dock engine (renderer B):** 7.1 layout tree + splitters · 7.2 drag + drop-zone overlays (Maya-style hover-to-plug-in) · 7.3 save/restore layouts · 7.4 custom-widget-contributed regions live · 7.5 (stretch) tear-off into separate OS windows (Tauri `WebviewWindow` + cross-window state sync).

## 8. Relationship to other decisions

D-102 (the `widget` tier this extends with a layout seam) · W-11 dd-socket (the data seam; W-12 is its layout sibling) · D-095 (the `ui/{...}` tier split) · D-056 (one shared command layer — the dock engine is shell-level, above components) · D-065 (build-when-consumed; renderer A before B so tiles prove content before they become draggable). All-widgets framing locked by Joe (2026-07-07).

## 9. Layout persistence

The live layout descriptor (§3) is saved to disk and restored on start. Layout is a **local UI preference** — stored in the client config dir (Tauri `app_config_dir()`), never federated, per-device.

**Baseline — one active layout (auto).**
- **Auto-load on start:** `get_layout()` → saved `Layout` or the default layout when absent.
- **Auto-save on exit:** persist the live tree in the window close hook (`on_window_event` / `CloseRequested`) before quit; also debounced-save on each mutation so a crash loses at most the last change.
- File: `xgen-client_layout.json`.

**Manual + named layouts (a layout manager).**
- Storage widens to a set: `xgen-client_layouts.json` = `{ active, layouts: { <id>: { name, layout, updated_at } } }`.
- Verbs: `list_layouts()` · `save_layout(name?)` (overwrite active or save-as) · `load_layout(id)` · `delete_layout(id)` · `rename_layout(id, name)`. Same read/write shape as `get_substitutions`/`set_substitutions` — the webview owns the live tree, Rust persists the blob.
- The **layout manager is itself a widget** (fits the model): pick / save-as / rename / delete / set-active. `delete_layout` confirms in-UI (destructive).

**Identity + reconcile rules.**
- **`widgetId` is the durable identity; the display name is a mutable label.** A saved layout references ids, so renaming a widget is a non-issue. A widget update MUST keep its id — same id or it is a different widget.
- On load, reconcile the saved tree against the current registry: **drop nodes with unknown `widgetId`** (removed/uninstalled widget), **re-inject missing `system` widgets** (W-13 — a saved layout can never lose the Composer), then re-flow.
- **`version` bump + migrate** only for descriptor **schema** changes (node shape). Prop/name drift on the same id is the widget's own concern, not a layout concern. On unrecoverable mismatch → fall back to default, never crash on a stale tree.

**Sequencing.** Contract is free now (`version` in the descriptor; the verbs are layout-shaped siblings of the M-RP6.1 read/write verbs). M-RP6.1 may stub `get_layout` (returns default) at no cost. Baseline auto-save/load lands at **M-RP7.3**; named layouts + manager widget at **M-RP7.6** (after renderer B can produce varied layouts worth naming).

---

*End of region/dock model.*
