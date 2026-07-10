# XGen Client — App Frame (Menu-bar + Status-bar) Phase-0
> **Status**: ACTIVE  
> Version: 1.3  
> Date: Jul 2026  
> **Last updated**: 2026-07-10  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

The M-RP6.1 D-071 Phase-0 gate for the client UI panel arc. Opened at the M-RP6.1 boundary; dependency (node↔client surface) confirmed **GO** at M-RP6.0 / J-473 (`m-rp6.0-gate-go`). This doc locks the **app frame** concept — the fixed menu-bar (top) and status-bar (bottom) that surround the dockable region layout — plus the component prerequisites the frame introduces, and re-sequences M-RP6.1 to build the frame **before** the risky center-shell work. Design/records-only, no code. Crystallises into **D-107**; extends the region/dock model (D-103) with the frame concept.

---

## 1. Gate result

- **Dependency:** node↔client channel — **GO** (M-RP6.0/J-473, all G1–G5 green, harness-primary).
- **Finding F-1** (no CDP-drivable connect/send pre-UI; client Tauri verb-commands absent) is **not a defect** — it is M-RP6.1's deliverable. The *read* half (a real `get_self_state` verb + reactive push) closes here; the *write* half at M-RP6.3.
- **Verdict: GO to design + build**, frame-first.

## 2. The frame concept (BorderPane terms)

The client window is a **BorderPane**: a fixed **top pane** (menu-bar), a fixed **bottom pane** (status-bar), and a **center** that holds the dockable region layout (renderer A now → dock engine B at M-RP7).

- **The frame is fixed chrome, NOT dockable regions.** Menu-bar and status-bar are **outside** the `Layout` descriptor (§3 of the region/dock model). Only the center is subdivided by the descriptor. The dock engine mutates the center; the frame is stable.
- **Consequence — exit is always reachable.** File→Exit lives in the frame, outside the dock tree, so it can never be docked away, closed, or torn off. This is a stronger guarantee than W-13 (which keeps *system widgets* present); the frame is not a widget at all.
- **Frame containers are `core` components; window-effects are shell-wired.** The status-bar (and later the menu-bar family) are reusable `core` library components — the **node** app will need an un-minimalized status-bar too. Because `core` stays app-agnostic (imports no Tauri/protocol — the `link`/`entity-avatar` rule), any real-window effect (the resize-grip drag, the Exit action) is exposed as a **seam** the consuming shell wires to its own Tauri call.

## 3. Safety order — frame first

Frame work lands **before** the center region shell. Rationale: a civilized exit and a live connection light must exist before we touch (and risk) the center. Hard fallback is the recovery tag `m-rp6.0-gate-go` (`git reset --hard` returns to the last known-good app). The menu-bar also doubles as a **live dev-state signal** — File→Exit today; View / Layout / etc. gain menu homes as regions land, so the menu visibly tracks what is wired.

## 4. Locked decisions

### 4.1 Menu-bar (minimal now, general family by accretion)

Taxonomy (JavaFX-standard, fully skinnable, **no native OS menu**):

- **`menu-bar`** — horizontal strip of top-level triggers; roving focus Left/Right; `role="menubar"`. Frame chrome (top pane), but composed of `core` menu components.
- **`menu`** — a multi-level text container; its popup runs the **W-2 owned-popup behaviour machine** (open→focus-in→roving→dispatch→close; Esc/outside-click/select-then-close/focus-leave dismiss; focus-return; portal + flip/shift). **Refined at M-RP6.1d / J-492 build:** that machine currently lives **entirely inside the CLOSED `entity-context-menu` widget** (interwoven with its dd concerns), so `menu` built a **fresh, minimal, self-contained** machine rather than refactoring the closed widget; **extracting a shared W-2/owned-popup module is DEFERRED to the 2nd populated menu or the submenu-flyout**, at which point both `entity-context-menu` + `menu` adopt it. The minimal build ships **inline-absolute** (no portal — the fixed top-pane dropdown didn't clip in the real client). Arc-local, no new D — the J-490/J-491 doc-wording-fix precedent.
- **`menu-item`** — `<li role="menuitem">` with a leading **icon** slot + a trailing **accelerator hint**.
- **`menu-separator`** — see §4.2 (the shared `separator`).
- **`menu-check-item`** — `role="menuitemcheckbox"` + checkmark + bindable `checked`. (`menuitemradio` sibling noted, deferred.)

**Scope (D-065):** build **minimal** now — `menu-bar` + one `menu` ("File") + one `menu-item` ("Exit"). `separator` / `check-item` / **submenu-flyout** grow when a second menu needs them. The submenu-flyout (open-on-hover / Right-arrow, nested popup positioning) is the one genuinely new mechanic with no precedent; deferred until consumed.

**Second menu — Help (M-RP6.1e-C, J-493).** The consolidation adds a **Help** menu with one item **About** (opens the About `dialog`). This is the **2nd populated menu** — the first real exercise of `menu-bar` roving Left/Right (6.1d had one), and the trigger for the **deferred shared-W-2/owned-popup extraction** (N-086): at this point either `menu`'s fresh-minimal machine proves it generalises, or it's extracted into a shared module both `menu` + `entity-context-menu` adopt.

### 4.2 `separator` — shared `core` component

One `separator` core component, orientation **`vertical` | `horizontal`**, skinnable rule (thickness / colour / inset via L2 tokens). Used **twice**: vertical between status-bar cells, horizontal as `menu-separator`. Built once, no duplication.

### 4.3 `icon` — `core` component, SVG glyph (distinct from `image`)

`icon` is its **own** `core` component, NOT folded into `image` (M-RP2.11). Cleared by D-096 on **two** axes:

- **value-type differs** — `image` carries a `src` **reference** (where a raster lives); `icon` carries a **shape definition** (the SVG path / instruction string — the geometry itself, not a URL). Different value-type.
- **surface differs** — `image` = raster content (`<img src>`, intrinsic size, `border-radius`, alt **required**); `icon` = a token-scaled, square, tintable UI glyph.

This mirrors JavaFX's own split (verified): `ImageView` is raster-only (png/jpg/gif/bmp, no SVG) ≈ our `image`; SVG is a **separate** `SVGPath` **Shape** node (fillable/tintable) ≈ our `icon`. JavaFX icons lean vector precisely for tint + scale — so `icon`'s **primary** path is inline `<svg>` (tintable via `currentColor`), with raster (`<img>`) a secondary mode.

- Root: inline `<svg>` (primary, tintable) / `<img>` (secondary, raster).
- Value (new registry value-type): a raw path **`d` string** (most one-path glyphs) or inner-svg markup (multi-element escape hatch) — a shape definition, not a `src`.
- Props: size (token-scaled 16/20/24), square, `tint` (default = surrounding text colour), `alt`/decorative (optional; glyph usually ornamental beside a label).
- No `src`, no `border-radius`, no `image` coupling; asset home `ui/assets/icons/`.
- `.ico` files explicitly excluded (Windows-native baggage). png/jpg/svg only.

### 4.4 `Accelerator` — one value-object, two projections

One canonical `Accelerator` value-object (parses `"Ctrl+Q"` → normalized modifier-set + key, platform-aware `Ctrl`↔`Cmd`), projecting two ways from a **single definition** (no display/dispatch drift):

- **display** — `toDisplay()` → `"Ctrl+Q"` (or `"⌘Q"`) for the menu-item hint.
- **dispatch** — `matches(keyEvent)` → boolean, for the keydown handler.

Layering:

- **`Accelerator`** (value-object + parser) → **`ui/common`**, pure, DOM-free (unit-testable like `stream/grouping.ts`; sibling to `Converter<T>`'s one-object-two-reps shape).
- **keymap registry** — **split** (D3, refined at M-RP6.1c / J-491 build): the **pure table + `resolve(event) → commandId | null`** lives in **`$common`** (`KeymapRegistry`, DOM-free + unit-testable; both the client and node shells keymap). Only the **singleton instance + binding population + the one global `keydown` listener** are **shell-level** (→ 6.1d). Build the objects **fully** now; the shell table starts **lean** (one binding: Ctrl+Q → Exit) and grows. Bindings register command **ids** (`"app.exit"`), not handlers — so the `menu-item` (6.1d) references the *same* id (single source of truth). *(Original wording put the whole registry shell-side; the pure-table-in-`$common` split is a genuine testability win, flagged not silently deviated — the J-487/J-490 doc-wording-fix precedent, arc-local, no new D.)*
- **hint render** → inside `menu-item`, reading the *same* `Accelerator` → bind once, the menu shows the correct hint and the key fires the correct command automatically (single source of truth).

### 4.5 `status-bar` — `core` container

A thin fixed one-line strip (bottom pane): a horizontal container of small **display** cells (read-only, glanceable), built as a **`core`** component (the node app needs it too).

**Contents locked (Joe, J-493):** the strip ships with **one real cell per side** — **left `sb-cell` = a `status-indicator`** (the connection light+label, the migrated hand-rolled `.state-indicator`; single-source-of-truth with the future R3), **right `sb-cell` = the SE resize-grip**. Not an empty demo.

- **Side-stacking** — cells carry `side` `left` | `right`; flex, two groups.
- **Cells** — `<span class="sb-cell">` wrapper hosting any display component (`status-indicator` / `label` / `meter`). The cell owns positioning + separator; the inner component owns its look.
- **Separators** — real `separator` (§4.2, vertical) between cells, skinnable.
- **Resize grip** — our **own** skinnable SE-corner triangle glyph (not the native OS triangle), **always visible**. The component draws the glyph and exposes an **`onResizeGrip?` seam**; each shell wires it to Tauri (`startResizeDragging`, SE corner = width+height). Keeps `status-bar` `core` (no Tauri import). *(If a future need is to resize an internal pane rather than the OS window, that is dock-engine splitter territory, M-RP7 — out of scope here.)*
- **Font** — status-bar text defaults to **`--fs-s1: 9px`** (tune to `--fs-s2: 8px` manually if needed).
- **Single source of truth** — the connection `status-indicator` in the bar reads the **same** reactive `self-state` signal as R3 (one channel, two views: compact bar light + detailed R3).

### 4.6 Font tokens (verified against `skin.css`, Rule 5)

Current live scale (grepped 2026-07-09): `--fs-0: 10px; --fs-1: 12px; --fs-2: 14px`. Add **below** `--fs-0`, additive, **no rename** of the shipped scale:

- **`--fs-s1: 9px`**
- **`--fs-s2: 8px`**

(`s` = small. 10px keeps `--fs-0`. These are broadly useful — other dense-UI spots want sub-10 too — so they are general L2 tokens, not status-bar-only.)

## 5. New components / objects the frame introduces

| # | Thing | Tier | Notes |
|---|-------|------|-------|
| 1 | `icon` | `core` | inline-svg glyph, shape-definition value-type, tintable |
| 2 | `separator` | `core` | vertical \| horizontal; shared status-bar + menu |
| 3 | `Accelerator` | `ui/common` | value-object, display + dispatch projections |
| — | keymap registry | shell | lean table + global keydown; consumes `Accelerator` |
| 4 | `menu-bar` / `menu` / `menu-item` | `core` | minimal (File→Exit); composes `icon` + accel hint |
| 5 | `status-bar` | `core` | `sb-cell` + `separator`s + resize-grip seam |
| — | `--fs-s1` / `--fs-s2` | skin (L2) | 9px / 8px, below `--fs-0` |

## 6. Revised M-RP6.1 sub-milestone split

Frame first (safe order), then the original center body. Each step gets its own per-component design lock + runbook when it opens (this Phase-0 locks the concept + sequence, not each component's internals).

- **M-RP6.1a** — `icon` (`core`).
- **M-RP6.1b** — `separator` (`core`).
- **M-RP6.1c** — `Accelerator` (`ui/common`) + lean keymap registry (shell).
- **M-RP6.1d** — `menu-bar` minimal (File→Exit → shell Exit command).
- **M-RP6.1e** — **client frame consolidation, split A/B/C (J-493).** See §10.
  - **M-RP6.1e-A** — `status-bar` (`core`; `sb-cell` + `separator`s + resize-grip seam + `--fs-s1`/`--fs-s2`; **left cell `status-indicator`, right cell grip**). Sampler-cell + CDP (grip inert there).
  - **M-RP6.1e-B** — real-client frame consolidation (9222): status-bar mounted bottom; `.state-indicator` → `status-indicator`; grip → `startResizeDragging`; **center-only scroll**; remove center logo + redundant Quit; window-config flips (`resizable:true`, menu-bar drag-region, 900×600 / min 640×400).
  - **M-RP6.1e-C** — `dialog`/`modal` `core` + **Help→About** (2nd menu; About holds name/version/authors/logo).
- **M-RP6.1f** — center region-shell scaffold (renderer A reads the `Layout` descriptor, `get_layout` stub → default, placeholder leaves) + selection bus primitive. *Fixture-only.*
- **M-RP6.1g** — R3 Self/connection live: `get_self_state` read verb + scoped `app.emit('self-state', …)` push + webview `listen`; renders `entity-item`(self) + `status` + `led`; the status-bar connection cell reads the same signal. *Real client 9222 + node 9322 — closes F-1 read half.*
- **M-RP6.1h** — R8 inspector wired to the selection bus (generic `EntityDescriptor` rows; self = first inspectable). Read loop closes end-to-end.

## 7. Verify strategy (per D-097)

The sampler (9422) **cannot** test the frame+shell — it is `tauri`+`tauri-build` only (no protocol deps, cannot reach a node) and is a component-cell grid, not an app window. So M-RP6.1 **graduates from the sampler to the real client app** — its permanent home anyway (replaces the current lean chrome). Three-layer verify:

1. **Pure unit (vitest)** — `Accelerator` parse/match + the descriptor→layout walk + reconcile rules. No app.
2. **Real client 9222, offline** — `icon`/`separator`/menu-bar/status-bar + region shell + selection bus, R3 fed a **literal fixture** self-descriptor. Layout renders, bus carries `{regionId, entity}`, R8 renders rows, R3→bus→R8 loop closes — no channel needed.
3. **Real client 9222 + node 9322** — swap fixture → live `get_self_state` + `app.emit`. The real proof: **stop the node → connection `led`/status-indicator flips to disconnected via the push**, restart → flips back (F-1 read-half). Individual `core` components (`icon`/`separator`/`status-bar` in isolation) still get a sampler cell for the component-level CDP registry check; the frame *assembly* is verified in the real client.

## 8. Deferred (D-065)

- Submenu-flyout; `menu-separator` (until a 2nd menu), `menu-check-item` / `menuitemradio`.
- Full keymap beyond the first binding.
- Center regions R1/R2/R4–R7 (M-RP6.2+); the R5 message-stream **system-widget wrap** (M-RP6.x; the `message-stream` component itself already shipped, M-RP5.6).
- `temperature-indicator` (M-RP6.5) — ⏸️ POSTPONED until the main window is functional.
- Layout persistence auto-save/load (M-RP7.3); named layouts + manager widget (M-RP7.6).

## 9. Records produced (this Phase-0, design-only)

`docs/xgen-client-frame-phase0.md` (this doc) · `ui/docs/xgen-region-dock-model.md` v1.1→v1.2 (+§10 frame concept) · `DECISIONS.md` +D-107 · `docs/ROADMAP.md` v4.56→v4.57 (M-RP6.1 re-sequenced frame-first) · `CLAUDE.md` PLAY (head pointer → J-488) · `JOURNAL.md` +J-488. No code; registry unchanged (286). Not pushed — Joe pushes.

---

## 10. Consolidation grounding (M-RP6.1e, J-493)

Joe's "consolidation of the app's main UI" = migrate the legacy hand-rolled chrome to the shipped `core` library, add the status-bar, enable whole-window resize, and confine scroll to the center. Grounded against the **real** client files (Rule 5) before locking.

### 10.1 What the real client is today

- **`xgen-client/tauri.conf.json`** — the window is **frameless** (`decorations:false`), **`resizable:false`**, **420×260**, centered. Consequences: no native title bar → no native edge-resize; and no way to *move* the window either.
- **`ui/client/src/app_client.svelte`** — all legacy chrome sits inside `#core-ui-pane`: a hand-rolled `<img id=app-logo>`; a hand-rolled `.state-indicator` (local `dotColor()` state→colour map + `isPulsing()` + `currentState.label`); the redundant `<Button id=quit>`. The quit seam is `invoke('quit')` (the keymap + File→Exit already reuse it). The top-pane `menu-bar` (6.1d) is already mounted over `.app-body`.

### 10.2 Legacy chrome → core mapping

| Legacy | Becomes | Where |
|--------|---------|-------|
| `.state-indicator` (dot + label) | `status-indicator` (`led` + `label`) — `dotColor`→`led.states` map, `isPulsing`→`led.pulse`, `currentState.label`→`label` | status-bar **left cell** (6.1e-B) |
| `<img id=app-logo>` | (removed from frame) → hi-res logo in the About `dialog` | Help→About (6.1e-C) |
| `<Button id=quit>` | removed — File→Exit is the exit (D-065 cleanup) | 6.1e-B |
| `#core-ui-pane` / `.app-body` | the center pane (the **only** scroller); placeholder leaf until 6.1f | 6.1e-B |

### 10.3 Window-config decisions (locked “by recomms,” J-493)

- **`resizable: true`** — flip it on.
- **Drag-to-move** — make the **menu-bar strip a drag region** (`data-tauri-drag-region` on the bar background; interactive `<button>` triggers override, so clicks still open menus). Restores window-move on a frameless window.
- **Default + min size** — default **900×600** (a real main window), **min 640×400** (composition floor). Tunable in the 6.1e-B runbook.
- **Resize mechanic v1** — frameless → the OS draws no resize borders, so **SE-grip `startResizeDragging` is the resize affordance** (Joe's “grip on the right of the status-bar”). Full invisible-edge-drag on all four edges is deferred polish (D-065).

### 10.4 The split

- **6.1e-A `status-bar` core** — the component (§4.5): `sb-cell` + `separator` + `onResizeGrip?` seam + `--fs-s1`/`--fs-s2`. Left cell hosts a `status-indicator`, right the grip. **Sampler-cell + CDP** (grip inert there). *Next-active.*
- **6.1e-B client frame consolidation** — real-client assembly (9222, no sampler): the §10.2 migration + the §10.3 window flips + center-only scroll (M-RP4.9/J-466 flex-column). *Real client only.*
- **6.1e-C `dialog`/`modal` core + Help→About** — build `dialog` (flagged J-432); add the Help menu (§4.1); About = name/version/authors/hi-res logo (version from the real build, not hardcoded). The 2nd-menu shared-W-2 extraction decision lands here.

This is a **sequence lock within the already-Phase-0'd frame arc** (D-107) — no new Phase-0, no new D. Per-milestone runbooks written as each opens (6.1e-A first).

---

*End of client-frame Phase-0.*
