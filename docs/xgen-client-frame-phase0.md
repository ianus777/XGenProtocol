# XGen Client — App Frame (Menu-bar + Status-bar) Phase-0
> **Status**: ACTIVE  
> Version: 1.6  
> Date: Jul 2026  
> **Last updated**: 2026-07-11  
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

**Second menu — Help (M-RP6.1e-C, J-493).** The consolidation adds a **Help** menu with one item **About** (opens the About `dialog`). This is the **2nd populated menu** — the first real exercise of `menu-bar` roving Left/Right (6.1d had one), and the nominal trigger for the **deferred shared-W-2/owned-popup extraction** (N-086).

**Extraction verdict — NOT TRIGGERED (Joe-locked, J-496). Recorded, not left silent.** J-492 deferred the extraction to "the 2nd populated menu **or** the submenu-flyout", and Help→About *is* the 2nd populated menu — so the trigger nominally fires. It is declined, for a stated reason: **Help is a second *instance* of the `menu` machine, not a second *shape*.** It demands nothing the fresh-minimal machine does not already do, while `entity-context-menu` still diverges on **every** axis that made the fresh-minimal build right at J-492 (portal, dd header, async `onSelect`, variant/purpose). Refactoring a **closed, verified** widget with **no forcing function** is exactly the wrong-abstraction risk N-086 was written to avoid. **The trigger is re-scoped to its real forcing function: the submenu-flyout, or a menu that needs the portal.** Whichever arrives first re-opens this decision; until then both machines stand as built. *(Arc-local, no new D — the J-490/J-491 doc-wording-fix precedent.)*

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

**Built (M-RP6.1e-A, J-494) — resolved build calls (Clair, within the runbook's latitude).** (1) **`sb-cell` = an internal, non-registering layout part** (own file, no `use:envelope`) — a value-less flex group; a registry getter would be ordinal noise (the N-064 opt-out pattern; the grip shares it). (2) **Grip glyph = a pure-CSS SE-corner `clip-path` triangle** in `--t4`, not an `icon` — a positioned corner affordance, not an inline-with-text glyph; kept wholly in `skin.css`, no `icons.ts` churn (logged for the icon-adoption backlog). (3) **2nd left cell via a `secondaryText?` prop** → a vertical `separator` + a `label` (the `#secondary` sampler cell exercises the vertical divider). (4) A **`grip` boolean prop** (default `true`) so `hasGrip` is honestly prop-driven (a non-resizable host can drop it). Getter G `{ leftCount, rightCount, hasGrip }`. Grip a11y: pointer-only, `aria-hidden`, `onpointerdown → onResizeGrip?.(e)` — no keyboard equivalent faked. CDP-green (sampler 9422, both accents): registry 299→309 (+10), 0 orphans, accent-neutral chrome, caption 9px. See N-087.

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
  - **M-RP6.1e-A** ✅ **DONE (J-494)** — `status-bar` (`core`; `sb-cell` + `separator`s + resize-grip seam + `--fs-s1`/`--fs-s2`; **left cell `status-indicator`, right cell grip**). Sampler-cell + CDP green (registry 299→309, 0 orphans, both accents); grip seam inert-but-fires, accent-neutral. N-087.
  - **M-RP6.1e-B** ✅ **DONE (J-495)** — real-client frame consolidation (9222): status-bar mounted bottom; `.state-indicator` → `status-indicator`; grip → `startResizeDragging`; **center-only scroll**; center logo + redundant Quit removed; window resizable 900×600 / min 640×400. **Window went `decorations: true` mid-milestone** (Joe) — the drag-region provision is **withdrawn**, frameless is **deferred to M-RP8**; see the revised §10.3. Client registry 7, 0 orphans; sampler unchanged 309. **+N-088** (the `#app` no-height latent bug).
  - **M-RP6.1e-C** — `dialog`/`modal` `core` + **Help→About**. **Split three ways (Joe-locked, J-496)** — the milestone is `dialog` **+ real Rust**, so it does not close in one commit pair:
    - **M-RP6.1e-C1** ✅ **DONE (J-496)** — `dialog` (`core`, the **31st**): native `<dialog>` + `showModal()` — top layer, `::backdrop`, focus trap, background-inert, Esc **all native**, so **no W-2 machine**. Composes a `button` Close child; `open` $bindable reconciled **both ways** (guarded `$effect` prop→element + a `close`-event listener element→prop — without the write-back, Esc makes the prop lie and the dialog can never reopen). Getter G `{title, open}`, `open` read from the **DOM**. **Sampler-verified 9422** (registry 309→**313**, 0 orphans, both accents); the load-bearing leg is **`el.matches(':modal')`** — `showModal()` reflects the `open` attribute itself, so the attribute cannot discriminate a real modal from the non-modal downgrade. **+N-089**.
    - **M-RP6.1e-C2** 🟡 — **`get_about_info`** (real Rust). **Not** a D-092 four-armed verb — a **shell read command** (the `get_substitutions` precedent). Home **`xgen-common::about`**: a shared **`AboutInfo`** env block (Built · Rust · Tauri · Svelte · Platform · paths **passed in**, never derived) + **typed per-app extensions** `ClientAboutInfo` / `NodeAboutInfo` — **the node's About differs by addition, not contradiction**, so the common block is the anti-drift win. Needs: `build.rs` env vars (Built = **date + short git SHA**; `rustc -V`; Tauri version; Svelte version read from the client `package.json`), **`data_dir` promoted to managed state** (only `ConfigPath` is managed today), the Tauri command + a capability check. Verify **real client 9222** via CDP `invoke`.
    - **M-RP6.1e-C3** 🟡 — **Help→About assembly**: the Help menu (`help.about`, **no accelerator** — F1 conventionally means Help *contents*) + the dialog mount + the logo assets. About = name · version · link · hi-res logo · Built · Rust/Tauri/Svelte · Platform · app dir · data/config paths · **Close** (not "OK"). Verify **real client 9222**.
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

### 10.3 Window-config decisions — **REVISED at build (J-495)**

**As shipped (M-RP6.1e-B, J-495):**

- **`resizable: true`** — flipped on. *(Unchanged from the J-493 lock.)*
- **`decorations: true`** — **the native title bar is ON.** *(This REVERSES the J-493 frameless lock — see the note below.)*
- **Default + min size** — **900×600**, **min 640×400**. *(Unchanged.)* *Known wrinkle (logged, not fixed): at DPR 1.25 these land as **physical** px → 720×480 CSS on screen; the 900×600 **logical** intent is not literally met at scaled DPI.*
- **Both config files** — `tauri.conf.json` **and** `xgen-client/cdp.dev.conf.json` carry the window block. The `--config` overlay **replaces the `windows` array wholesale**, so a flip in one without the other leaves the **debug** window (9222 — the surface every CDP verify runs on) at the old geometry. **Any future window-config change touches both files.**
- **Capabilities** — Tauri v2's `core:window:default` is **getters-only**. `start_resize_dragging` needs an explicit `core:window:allow-start-resize-dragging` grant (present). `allow-start-dragging` was added for the drag region and then **removed** with it.
- **Move** — the **native title bar** (OS-provided). The J-493 `data-tauri-drag-region` provision is **WITHDRAWN**.
- **Resize** — **native edges** (OS-provided) **plus** the status-bar SE grip, which **stays wired** to `startResizeDragging('SouthEast')` as a supplementary corner affordance.

**Why frameless was deferred (Joe, 2026-07-11).** A frameless window during development has **no window controls and no way to move it** — a real practical cost paid on every build-run cycle, for a chrome decision whose payoff only arrives at polish time. So native decorations are adopted as a **temporary development affordance**. **This is a deferral, not an abandonment:** the frameless / custom-chrome endpoint stands, filed as **M-RP8 — `title-bar` `core` + frameless restore**, scheduled **after the widget grid is live on BOTH apps** (by then the client and node frames are structurally identical, so it is one component and two mounts rather than building it twice).

**The honest technical finding that shaped this.** The intuition “just customise the native bar — extra buttons, immune to OS theming” does **not** survive contact with Win32: a native caption can have its **colours** changed (Win11 DWM attributes; **silently ignored on Win10**) and nothing else — **extra buttons are impossible**, and geometry/glyphs stay OS-owned. The Discord-style bar people picture **is** a custom (frameless) title bar. There is no cheap middle option; an interim DWM colour-tint was considered and **rejected** as work we would delete. So the choice is binary and correctly sequenced: **native now (free, disposable), custom later (one milestone, no rework)**.

**Joe's rule, locked:** **no native elements within the window's main pane.** The native chrome is the OS title bar and nothing else — which is why the SE grip **stays** even though native edges now resize.

**Why keeping the grip is what makes M-RP8 cheap.** The `onResizeGrip → startResizeDragging` seam is the only genuinely risky part of going frameless, and it is **already built, granted, and proven end-to-end** (J-495: real drag, window 720×480 → 743×470, zero permission denials). Keeping it live and exercised means M-RP8 never has to rebuild it — a seam that stays wired cannot rot. What M-RP8 adds is only: a `title-bar` `core` component (drag-region root + title + minimise/maximise/close seams, the same shape as `onResizeGrip`), 4 `icons.ts` glyphs, `decorations:false` in both apps, and the `allow-start-dragging` grant back.

**Note for the M-RP8 builder (learned the hard way, J-495).** The custom title-bar must own its **own** drag-region root. It **cannot** ride the menu-bar's strip: the `core` skin sets `.menu-bar { width: 100% }` (it is designed as a full-width bar with its own background + border), and Tauri drags **only when the event target itself carries `data-tauri-drag-region`** — never an ancestor. The 6.1e-B attempt to shrink the bar and drag on a wrapper worked, but was reverted with the pivot; do not rediscover it.

### 10.4 The split

- **6.1e-A `status-bar` core** — the component (§4.5): `sb-cell` + `separator` + `onResizeGrip?` seam + `--fs-s1`/`--fs-s2`. Left cell hosts a `status-indicator`, right the grip. **Sampler-cell + CDP** (grip inert there). *✅ DONE (J-494) — registry 299→309, 0 orphans, accent-neutral, N-087.*
- **6.1e-B client frame consolidation** — real-client assembly (9222, no sampler): the §10.2 migration + the §10.3 window flips + center-only scroll (M-RP4.9/J-466 flex-column). *✅ DONE (J-495) — client registry 7 (0 orphans), sampler unchanged 309; native-title-bar pivot (§10.3 revised); `#app` no-height latent bug found + fixed (N-088).*
- **6.1e-C `dialog`/`modal` core + Help→About** — build `dialog` (flagged J-432); add the Help menu (§4.1). **Split C1/C2/C3 — see §6.** *(C1 ✅ DONE, J-496 — `dialog` core, sampler-verified 313.)* The 2nd-menu shared-W-2 extraction decision lands here — **and it landed: NOT triggered, see §4.1.** **About scope grew (Joe, 2026-07-11, per a reference screenshot):** name · version · a **link** · hi-res logo · **Built** date · Rust/Tauri/Svelte versions · Platform · app directory · data/config paths · a **Close** button (not "OK"). Everything below "Built" is **invisible to the frontend** — build metadata and filesystem paths need a **new Tauri read verb (`get_about_info`)**. So 6.1e-C is `dialog` **+ real Rust work**, not a component-only milestone; its runbook must scope both. *Next-active.*

This is a **sequence lock within the already-Phase-0'd frame arc** (D-107) — no new Phase-0, no new D. Per-milestone runbooks written as each opens (6.1e-A first).

---

*End of client-frame Phase-0.*
