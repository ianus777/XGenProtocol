# XGen UI — Region / Dock Model
> **Status**: ACTIVE  
> Version: 1.3  
> Date: Jul 2026  
> **Last updated**: 2026-07-11  
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

- **Config-grid renderer (A, M-RP6.1+): ✅ BUILT at M-RP6.1f (J-499)** — `region-shell` (`core`, the **32nd**). Renders a restricted subset: **`leaf` + `split` only**, fixed `sizes`, **no runtime mutation**. Rearranging = editing the descriptor (or the DEV `__XGEN_LAYOUT__` handle). A **`tabs` node is DROPPED with a DEV warn** (renderer B owns tabs — an unfed branch would be an unverified branch, D-065/N-091). A `leaf` whose `widgetId` the registry cannot resolve is **DROPPED** — the same prop-injected `widgets: Record<widgetId, Component>` shape `message.svelte` already shipped (W-13 reconcile; **one mechanism, not two** — N-093). A `split` whose children all drop collapses; a `sizes`/`children` length mismatch degrades to equal weights + a warn. **It never throws** — see §9's stale-tree rule, and its one live gap in the note there. `sizes[]` ride an **inline `flex: {n} 1 0`** (descriptor **data**, not skin — the one carve-out from N-090).
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

A shell primitive the regions share: **one active selection** across the layout `{ regionId, entity: EntityDescriptor }`. R8 (inspector) reads it to render the selected object's parameter rows; R1/R2 write it on row activation; `entity-context-menu` reads the same selection.

**✅ BUILT at M-RP6.1f (J-499)** — `ui/common/lib/stores/selection.svelte.ts` (a new `stores/` folder): `{ regionId, entity } | null` with `current` / `set(regionId, entity)` / `clear()`, plus a DEV `__XGEN_SEL__` handle. Verified: `null → set → replace → clear` (**one selection, never a list**).

**It is a `$common` store by NECESSITY, not convenience.** Both consumers — **R8** and **`entity-context-menu`** — are widgets living in `ui/common/…/widgets/`, and **W-3 forbids a `common` widget from importing a shell dep**. A shell-local bus would be **structurally unconsumable** by the very components it exists for. *(Recorded so the home is never “tidied” back into the shell.)*

**The shape is FINAL at one meaning.** There is exactly **one** selection concept — the *entity* selection. The shelf's minus-button was deliberately killed (`docs/xgen-widget-surfaces-phase0.md` S-6) precisely because it would have forced a **second** (widget/leaf) selection bus, with the permanent hazard that clicking a *room* arms a *delete-panel* button. **Do not reintroduce a second bus.**

**⚠️ No WRITER exists yet** (W-8 honesty — the phase-limit is surfaced, not hidden). The first writer lands with R3 at **M-RP6.1g**; the first reader with R8 at **M-RP6.1h**.

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

> **⚠️ GAP, measured at M-RP6.1f (J-499 / N-095) — half of that last rule is not yet true.** A **null / absent** layout **unmounts `region-shell` entirely → a blank centre** (measured: registry 30→21, shell out of the DOM, `.app-center` empty). So *“never crash on a stale tree”* ✅ **holds** — but *“fall back to default”* ❌ **does not**: today it falls back to **nothing**.
>
> **Deliberately not fixed at 6.1f:** `loadLayout()` returns a constant and **cannot** return null, so a `?? DEFAULT_LAYOUT` guard would be an **unreachable branch in a closed milestone** — the same D-065/N-091 argument that kept the `tabs` branch out. The fallback belongs to the **loader** (parse a real file → find it missing/corrupt/schema-stale → recover), which is **M-RP7.3's** code.
>
> **Pinned to M-RP7.3's DoD:** *a missing / corrupt / schema-stale layout file falls back to `DEFAULT_LAYOUT`, never to a blank centre — and the fallback is **exercised** (feed it a corrupt file), not asserted.*

**Sequencing.** Contract is free now (`version` in the descriptor; the verbs are layout-shaped siblings of the M-RP6.1 read/write verbs). Baseline auto-save/load lands at **M-RP7.3**; named layouts + manager widget at **M-RP7.6** (after renderer B can produce varied layouts worth naming).

**⚠️ The `get_layout` stub was RESOLVED AS “NO RUST” at M-RP6.1f (J-499, D2)** — correcting this section's earlier *“M-RP6.1 may stub `get_layout`”* provision. A Rust command returning a hardcoded default would either **duplicate the descriptor type in Rust** (the **D-067 drift surface** this project exists to eliminate) or return an opaque blob Rust does not own — theatre for one call site. **The seam lives in the frontend** as `async loadLayout()`, today returning `DEFAULT_LAYOUT`. At **M-RP7.3** only its *body* becomes `invoke('get_layout')`, and **Rust persists the tree as an opaque blob** (the `get_substitutions` shape — *the webview owns the live tree, Rust persists it*), so **Rust never learns the node shape**. One function, one swap, zero drift.

---

## 10. App frame — the non-dockable complement (D-107)

Added at M-RP6.1 Phase-0 (J-488; `docs/xgen-client-frame-phase0.md`). The model above describes the **dockable** layout; this section adds the **fixed frame** around it.

The client window is a **BorderPane**: a fixed **top pane** (menu-bar), a fixed **bottom pane** (status-bar), and a **center** — and the center is the *only* part the `Layout` descriptor (§3) subdivides. The menu-bar and status-bar are **fixed frame chrome, NOT dockable regions/widgets, and live OUTSIDE the descriptor.** The dock engine (renderer B, M-RP7) mutates the center; the frame is stable.

- **Why not widgets.** W-12/W-13 keep *system widgets* present-but-dockable. The frame needs something stronger: File→Exit must be un-dockable and unclosable, period. Making it frame chrome (outside the dock tree entirely) is that guarantee — a cleaner answer than a special widget flag.
- **Frame containers are `core`; window-effects are shell-wired.** The status-bar and menu family are reusable `core` components (the node app needs an un-minimalized status-bar too). `core` imports no Tauri/protocol, so real-window effects ride **seams**: the status-bar resize-grip via `onResizeGrip?` (shell → `startResizeDragging`), the menu Exit via a command callback (shell → exit command).
- **New components this introduces:** `icon` (core, svg glyph) · `separator` (core, vertical|horizontal, shared menu+status-bar) · `Accelerator` (`ui/common` value-object, display+dispatch) + a lean shell keymap registry · `menu-bar`/`menu`/`menu-item` (core, minimal File→Exit) · `status-bar` (core). Skin tokens `--fs-s1: 9px` / `--fs-s2: 8px` below `--fs-0: 10px`.
- **Relationship to §1.** “Every region is a widget” still holds for the **center**. The frame is not a region; it is the window shell the center layout sits inside. The two are complements: descriptor-driven dockable center + fixed frame chrome.

See `docs/xgen-client-frame-phase0.md` for the full frame Phase-0 and the frame-first M-RP6.1 build order. Crystallised as **D-107**.

---

*End of region/dock model.*
