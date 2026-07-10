# M-RP6.1d — `menu-bar` minimal (core trio) + keymap wiring (shell) build runbook
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-10  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

For Clair. Fourth frame step of the M-RP6.1 client-UI-frame arc (Phase-0 J-488 / D-107 / `docs/xgen-client-frame-phase0.md` §4.1, §6, §7). Per-component design **locked by Joe** (Chat design walk + one Joe refinement on the sampler question, "all others by recomms", this session). This is the **first frame step that touches the real client shell** — the menu family is **frame chrome, not sampler cells** (Joe): built as `core`, mounted into the client's fixed top pane, and verified in the **real client (9222)**, not the sampler. It also wires the **6.1c-deferred keymap** (singleton `KeymapRegistry` + `Ctrl+Q → app.exit` + the one `keydown` listener + the `exitCommand`). Design captured here; **no code at lock time** (Rule 1/5).

---

## 1. Goal

Build the minimal `core` menu trio and wire a civilized exit:

- **`menu-bar`** — the top-pane horizontal strip (`role="menubar"`), roving Left/Right over triggers.
- **`menu`** — one entry, **File**: a trigger + an owned popup (`<ul role="menu">`) of items.
- **`menu-item`** — one item, **Exit**: `<li role="menuitem">` = leading `icon` slot (6.1a) + label + trailing **accelerator hint** (`Accelerator.toDisplay()`, 6.1c).
- **shell keymap** — `KeymapRegistry` singleton + `register(accelerator("Ctrl+Q"), "app.exit")` + one global `keydown` → `resolve` → run; a command table `{ "app.exit": exitCommand }`; File→Exit runs the **same** `"app.exit"`. Ctrl+Q and File→Exit both quit — one command, one truth.

Three earlier frame prerequisites are consumed here: `icon` (6.1a) as the leading slot, `Accelerator` (6.1c) as the trailing hint. `separator` (6.1b) waits for a second menu.

## 2. Locked design

### 2.1 The behaviour machine — fresh-minimal, NOT a refactor of the closed widget (A2)

- The W-2 machine (open→roving→dispatch→close, dismiss policy, focus-return, wire/teardown, portal/flip-shift) currently lives **entirely inside the CLOSED, verified `entity-context-menu` widget**, interwoven with its dd concerns (EntityDescriptor header, status, async `onSelect`, variant/purpose, portal). There is **no extracted shared module**.
- **Build a small, self-contained machine inside `menu`.** Do **NOT** edit `entity-context-menu` (regression risk on a closed widget; and the two consumers have divergent needs — context-menu: portal + dd-header + async dispatch; menu-bar dropdown: roving Left/Right at the bar + Down/Up in the popup, **sync** command, no dd). The File menu is tiny; the duplication is small and gives us a **second concrete machine** — the right moment to extract a shared W-2 is when a *populated* second menu or the submenu-flyout forces the abstraction.
- **Flag (Rule 6):** frame-phase0 §4.1 says `menu` "reuses the `entity-context-menu` owned-popup + W-2 machine." This runbook **refines** that to *build minimal now, extraction deferred* (the same shape as the already-deferred submenu-flyout). **Log** the deferred item: "extract a shared W-2/owned-popup module when the 2nd populated menu or submenu-flyout lands; entity-context-menu + menu both adopt it." Records at close (ui-notes N + a §4.1 in-place refinement, arc-local, no new D — the J-490/J-491 doc-wording-fix precedent). This is captured; **no `entity-context-menu` change this milestone.**

The minimal machine `menu` needs:
- **open** on trigger click / Enter / Space / Down-arrow; move focus into the first enabled item.
- **roving** Down/Up (+ Home/End) over items; `aria-activedescendant` or roving `tabindex` (match the `entity-panel` roving precedent for consistency).
- **dispatch** on item Enter/Space/click → run the item's command → close.
- **dismiss** Esc / outside-click / focus-leaves-the-menu; **focus returns to the trigger** on close.
- **wire on open, tear down on close/unmount** (0 orphan listeners — W-5).
- `menu-bar` roving **Left/Right** across triggers; Down opens the focused menu.

### 2.2 Components + roots (all `core`, envelope-registered)

- **`menu-bar`** — `<div class="menu-bar" role="menubar">`, composite; hosts triggers; roving Left/Right. `use:envelope`, getter G `{ items, activeIndex, openIndex }` (observable task-state only).
- **`menu`** — a trigger `<button class="menu-trigger" role="menuitem" aria-haspopup="menu" aria-expanded={open}>` + an owned popup `<ul class="menu-popup" role="menu">`. Composite; `open` state; the §2.1 machine. Getter G `{ label, open, itemCount, activeIndex }`. (The `<ul role="menu">` is exactly why `separator` was rooted `<div role="separator">` and `menu-item` is `<li role="menuitem">` — they slot in with no branch.)
- **`menu-item`** — `<li class="menu-item" role="menuitem">` = optional leading `icon` (6.1a, composed child) + `<span class="mi-label">` + optional trailing accelerator hint `<kbd class="mi-accel">{accelerator.toDisplay(platform)}</kbd>`. Props: `label`, `icon?` (name), `accelerator?` (an `Accelerator`), `disabled?`, `onSelect?`/`command?`, `id`. Getter G `{ label, hasIcon, accel, disabled }`. The composed `icon` self-registers `__icon` **only when the frame is CDP-inspected in the client** (see §5 — no sampler cell).

### 2.3 Popup positioning — inline absolute (C1)

- Popup = `position: absolute`, anchored below its trigger. **No portal** now (the menu-bar is the fixed top pane; a short downward dropdown doesn't need relocation). **Watch-item (real-client):** if the top-pane / window `overflow` clips the dropdown in the client, portal-to-body + fixed is the fix — **deferred until observed**, not pre-built (the entity-context-menu portal path is the reference if needed).

### 2.4 Keymap wiring (shell — the 6.1c-deferred half)

- **Command table** (shell) `{ "app.exit": exitCommand }` — the F2 payoff the 6.1c `id?` reserved for. Both the keymap and the menu-item resolve to `"app.exit"` and run it via this table (single source of truth).
- **`exitCommand`** = **reuse the exact Tauri close the existing client Quit/Shut-Down button already wires** (Rule 5 — confirm against the real client shell code; do NOT invent a new close call).
- **`KeymapRegistry` singleton** created on client mount with the detected `PLATFORM`; `register(accelerator("Ctrl+Q"), "app.exit")`; one global `window` `keydown` → `registry.resolve(e)` → if a `commandId` comes back, `preventDefault()` + run the command-table entry. Tear the listener down on unmount.
- **File→Exit `menu-item`** carries `accelerator("Ctrl+Q")` (renders the hint) and `command: "app.exit"` (runs the same table entry). One `Accelerator`, one command.

### 2.5 Mount into the real client (additive)

- Add the `menu-bar` (File → Exit) into the client's fixed **top pane** (the BorderPane top; frame chrome OUTSIDE the `Layout` descriptor — §2 of the region-dock model). 6.1e adds the bottom status-bar; 6.1f the center.
- **Additive — leave the existing Quit/Shut-Down button untouched** (D-065; File→Exit is the new civilized exit, but ripping the working button is scope creep — a later cleanup removes the redundant affordance once the frame is trusted). The node app's un-minimalized menu-bar is out of scope here (6.1e notes the node needs the status-bar; menus later).

## 3. Files to touch (indicative — Clair confirms exact paths)

1. `ui/core/…/menu-item.svelte` — new `core` (§2.2). Composes `icon`; renders the accel hint via the `Accelerator` prop's `toDisplay()`.
2. `ui/core/…/menu.svelte` — new `core` (§2.2 + the §2.1 machine + §2.3 popup).
3. `ui/core/…/menu-bar.svelte` — new `core` (§2.2, roving Left/Right).
4. `ui/assets/skin.css` — `.menu-bar` / `.menu-trigger` / `.menu-popup` / `.menu-item` / `.mi-label` / `.mi-accel` (the `<kbd>`-style hint) L2 rules. Confirm real tokens (Rule 5).
5. `ui/client/…/<shell entry>.svelte` — mount `menu-bar` into the top pane; create the `KeymapRegistry` singleton + command table + `exitCommand` (reusing the existing Quit close seam); register `Ctrl+Q → app.exit`; attach/detach the `keydown` listener.
6. `ui/client/…/keymap.ts`-or-equivalent — if the shell wants the registry construction + listener in a small module rather than inline in the entry (Clair's call; keep it lean).

**NOT this milestone (defer):** submenu-flyout · `menu-separator` (until a 2nd menu) · `menu-check-item`/`menuitemradio` · portal · full keymap beyond Ctrl+Q · removing the existing Quit button · the node app's menu-bar · the sampler frame-window incorporation (§5).

## 4. No sampler cells for the menu family (Joe-locked)

The menu-bar/menu/menu-item are **frame chrome**, not catalogue components — they do **NOT** get `app_sampler.svelte` grid cells. **The sampler catalogue registry stays 299** (they are authored `core` components with `xgen-ui-components.md` index rows, but not sampler-registered instances). Their appearance + functionality are verified in the **real client** (§5). *(Later, deferred: the sampler's own app window MAY host the real menu-bar + status-bar as actual frame chrome — an appearance demo, alongside 6.1e — but that is NOT this milestone.)*

## 5. Verify plan — real client (D-097 graduation; Rule 2, quote real output)

The sampler cannot host the shell keymap/exit or the frame assembly. Verify in the **real client (9222)** via the restored CDP harness (M-RP-CDP1 / J-483) + eyecheck:

- **Structure (client CDP `__XGEN_DEBUG__`):** `menu-bar`/`menu`/`menu-item` present with correct roots (`role="menubar"` / trigger `aria-haspopup`+`aria-expanded` / `<ul role="menu">` / `<li role="menuitem">`); getter G on each; the composed `icon` self-registers `__icon` under the item (or absent if Exit ships icon-less — Exit may leave the icon slot empty, so state that plainly); **measure the client registry delta, do not predict** (Rule 5). The *sampler* registry is untouched at 299.
- **Accelerator appearance (eyecheck):** File→Exit renders the hint **"Ctrl+Q"** via `toDisplay('win')` in the real `.mi-accel` skin (this is the accelerator appearance surfacing in the real client, per Joe — no sampler cell).
- **Machine (real client, dispatched events):** trigger opens the popup (`aria-expanded=true`); Down/Up rove items; Esc closes + **focus returns to the trigger**; outside-click closes; focus-leave closes; 0 orphan listeners after close (re-open/close cycles clean).
- **Exit (functional — the headline):** **File→Exit quits the client window**, and **Ctrl+Q quits the client window** (both via `app.exit` → the reused Quit close seam). Because quitting is destructive, verify the dispatch path first (registry.resolve on a synthetic Ctrl+Q → `"app.exit"`; menu-item select → `"app.exit"`) then a **single manual real quit** eye-confirmed by Joe. Do not loop the real quit.
- **`vite build` clean** — module count quoted.

## 6. Close (D-074 two-commit)

Clair feat first (code-only: §3 files). Then Chat doc-bridge:
- `ui/docs/xgen-ui-components.md` — index rows for `menu-bar`/`menu`/`menu-item` (authored `core`; **note: frame chrome, not sampler cells; sampler registry unchanged 299**). Version bump.
- `ui/docs/xgen-ui-notes.md` **N-086** (the menu trio / fresh-minimal machine vs the closed-widget W-2 / inline-popup / the keymap wired live / accelerator hint surfaces in the real client / **deferred: extract shared W-2 + sampler frame-window incorporation**).
- `docs/xgen-client-frame-phase0.md` — **§4.1 in-place refinement** (`menu` builds a minimal self-contained machine now; shared-W-2 extraction deferred to the 2nd populated menu / submenu-flyout). Version bump; arc-local; **no new D**.
- `docs/ROADMAP.md` (M-RP6.1d ✅ DONE, vX bump, next-active **M-RP6.1e `status-bar`**).
- `CLAUDE.md` PLAY (head → new J-492; sampler registry 299; the client now has a top-pane menu-bar + live Ctrl+Q exit; next-active 6.1e).
- `JOURNAL.md` +J-492 (quote the real client CDP + the build; note the manual quit confirm).
- this task → COMPLETED.

**No new D.** Deferred (D-065): submenu-flyout, menu-separator, check-item, portal, shared-W-2 extraction, sampler frame-window. Not pushed — Joe pushes.

## 7. Definition of Done

- [ ] `menu-item.svelte` (`core`) — `<li role=menuitem>`, icon slot + label + `Accelerator` hint, getter G.
- [ ] `menu.svelte` (`core`) — trigger + `<ul role=menu>` popup + the §2.1 minimal machine (open/rove/dispatch/dismiss/focus-return, 0 orphans), getter G.
- [ ] `menu-bar.svelte` (`core`) — `<div role=menubar>`, roving Left/Right, getter G.
- [ ] `skin.css` — `.menu-*` L2 rules incl. the `.mi-accel` hint (real tokens confirmed).
- [ ] shell — `KeymapRegistry` singleton + command table `{app.exit}` + `exitCommand` reusing the existing Quit close seam (confirmed against real code, Rule 5); `Ctrl+Q → app.exit`; `keydown` listener attach/detach; menu-bar mounted in the top pane (existing Quit button left intact).
- [ ] Real-client CDP structure green (roots/roles/getters); client registry delta measured; **sampler registry unchanged 299**.
- [ ] Machine green (open/rove/dismiss/focus-return, 0 orphans); Ctrl+Q + File→Exit resolve to `app.exit`; a single manual quit Joe-confirmed.
- [ ] `vite build` clean — module count quoted.
- [ ] Records bridged (§6, incl. the §4.1 in-place refinement + the two deferred items), task flipped COMPLETED.

---

*End of M-RP6.1d runbook.*
