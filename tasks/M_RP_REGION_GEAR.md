# M-RP-REGION-GEAR — per-region settings gear (fires the plugin `settings` action)
> **Status**: PENDING  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-17  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

**FILED, not the current task.** Design-walked with Joe 2026-07-17 and locked to the shape below; **implementation is deliberately deferred** — the next build session is **M-RP6.6 (client networking / the resident)**, the last missing part of the client. This record exists so the arc is ready to hand to Clair when Joe slots it. Re-ground §0 before any code (N-116).

**The idea in one line:** put a gear on every region tile that is the **exact twin of the plugin-list row gear** — same `settings` action, same enablement gate — so a behaviour setting (e.g. "show rooms as: lines | avatars | gallery") is one click from the tile it affects, instead of Settings → Plugins → find the row.

---

## 0. Grounding — what this REUSES (verified 2026-07-17; re-confirm before build)

- **`settings-dialog.svelte`** already drills into a plugin's own settings: `drill = {id, mode:'settings'}` → mounts `drillPlugin.settingsComponent` in the content pane (M-RP-SETTINGS Leg C, **D-120**, J-540). The gear needs **no new settings surface**.
- **`plugin-list.svelte`** row gear: `actionsFor()` derives the `settings` button `disabled: !hasSettings`, `hasSettings = !!p.settingsComponent`; a guarded `onclick={() => !a.disabled && onAction?.(p.id, a.verb)}` never fires when greyed. **This is the exact gate + click contract the tile gear copies.**
- **`PluginDescriptor.settingsComponent?: Component`** (`ui/common/lib/plugins/registry.ts`) — set on `grid-plate` only today. Every region resolves to a plugin row (D-112: a widget is in the grid because it is a registered plugin — confirm at Phase-0).
- **`region-tile.svelte`** (`core`) chrome today: `[move-grip · title · fold-width · fold-height]` + body + SE resize grip. **No gear exists.** The two header chevrons users see are the fold buttons. Seams already threaded shell→tile: `onFold`, `onMoveStart`, `titles`, `locked` — the gear rides the same rails.
- **`grid-plate`** is `surface:'none'` (a backdrop, no tile) — so it is the one plugin WITH settings but the one that can NEVER carry a region gear. Consequence in §3.

---

## 1. The three layers (keep them separate)

The gear is only the front door. The arc is three separable pieces:

1. **ACCESS** — the region-tile gear. Fires the `settings` action from the tile. Small, frontend-only, buildable now. **← this milestone.**
2. **HOST** — a widget's own `settingsComponent`, mounted in the Settings pane (shipped mechanism, D-120). Each region that wants settings ships one. Per-widget.
3. **BEHAVIOUR** — the render variants the setting toggles (e.g. Rooms lines/avatars/gallery). Per-widget presentation work.

The gear is worth building first **because** it is the cheap, reusable door; the doors behind most regions are empty until layer 2 ships for them.

---

## 2. Locked decisions (Joe, 2026-07-17)

- **D1 — the region gear IS the row gear.** Same `settings` action, same `hasSettings` gate (`!!p.settingsComponent`). No special-casing, no separate destination. The S-2 "one thing, two mounts" pattern: **one `settings` action, two entry points** (list row + region tile).
- **D2 — disabled, NOT element-absent.** When the plugin has no `settingsComponent`, the tile gear is **greyed and inert** (the list's guarded-onclick pattern), not removed. **This deliberately OVERRIDES the tile-chrome house rule** (J-500 / D8.2 — a non-applicable tile control ships absent, like the folded tile's missing resize triangle). Recorded explicitly so nobody "fixes" it back to absent: **the greyed gear is intentional; a simple plugin with a disabled gear is a valid, honest state, not a gap.**
- **D3 — the one new bit vs the row gear: open + navigate.** The row gear fires inside an already-open Settings window; the region gear must **open Settings pre-drilled** to `{id: regionId, mode:'settings'}`. That "open + navigate" wrapper is the milestone's only genuinely new logic.
- **D4 — W-3-clean wiring.** `region-tile` is `core` and cannot read the plugin registry. The **shell** computes `hasSettings` per region and threads it down as an `enabled` flag alongside `titles`/`locked`, plus an `onSettings?(regionId)` seam — exactly how `onFold` is already threaded. No layering line crossed.
- **D5 — gear stays live under grid-lock.** Settings is a *function*, not an *arrangement* mutation, so unlike fold/move (which go element-absent under `locked`, M-RP7.6) the gear remains active when the grid is locked. (Its own `enabled` gate is `hasSettings`, independent of `locked`.)
- **D6 — appearance is Joe's → M-RP-SKIN.** Glyph, position in the stripe, hover, greyed look all ship **provisional**. Joe's screenshots marked *position, not colour*. Proposed DOM order `[move · title · gear · fold-width · fold-height]`; skin tunes.

**Open (decide at kickoff, not now):** scope A vs B —
- **A — ship the gear alone, exact parity.** Greyed on every tile until region widgets gain settings (day-one: **all region gears greyed**, since only `grid-plate` has settings and it has no tile). Honest, matches the list, but visually dead at first.
- **B — pair the gear with Rooms' first `settingsComponent`** (§4) so a real tile gear is live on day one — a far better demo than a greyed row. Adds layer-2 + layer-3 work for one region.

Default until Joe picks: **nothing is built** (this is filed).

---

## 3. Day-one reality (state it, don't be surprised by it)

With true parity (D1) and scope A, **every region gear is greyed at first.** `grid-plate` is the only plugin with settings today and it is a tile-less backdrop. Region gears light up one-by-one as each region ships its own `settingsComponent`. That is the point of the front door being ready before the rooms behind it.

---

## 4. First behaviour tenant — Rooms "show rooms as: lines | avatars | gallery"

The natural first real tenant (and the reason to consider scope B). It proves the whole chain end-to-end — gear → host → store → repaint — the way Grid Backdrop proved it for backdrops.

- **Host (layer 2):** `rooms-panel` ships a `settingsComponent` — a selector `line | avatar | gallery`.
- **Value (store-mediated, W-3):** the setting writes a `$common` store key; `rooms-panel` reads it and picks its render mode. **Same channel `grid-plate`'s backdrop uses**; persisted per-device through the session bag (per-key N-107 merge), restored on relaunch. The widget never imports the shell.
- **Behaviour (layer 3):** the render variants themselves. `entity-item` already has `variant="row" | "card"`; **`avatar` / `gallery` are new presentation work** on `entity-panel` (or a sibling) — a compact avatar list and an avatar grid. This is the largest piece and is per-widget, not part of the gear milestone.

**Filed, not designed here:** the exact variant set, `entity-panel` grid mode, and whether "show as" generalises to other entity regions (Spaces, Members) is its own Phase-0 when picked up.

---

## 5. Milestone

- **Deps — all shipped:** Settings window + drill (D-120, J-540) · `region-tile` chrome (M-RP7.1) · `settingsComponent` seam + `hasSettings` (Leg B/C). **Buildable now.**
- **Cost:** frontend-only; `cargo` stays **1517/0/62 IDENTICAL** (the honest proof no Rust landed). Registry delta from the tile gear's own envelope entry, measured after a full reload (N-132).
- **Scope A** (gear only) is small. **Scope B** (gear + Rooms "show as") is a real chunk (layer 2 + layer 3 for Rooms).
- **Sequencing:** independent of M-RP6.3 (composer) and **behind M-RP6.6 (client networking)**, which Joe takes first. Slot M-RP-REGION-GEAR whenever after.

---

## 6. Out of scope / not now

Instant implementation (Joe, 2026-07-17 — networking first). Any layer-3 render-variant design. Generalising "show as" beyond Rooms. Element-absent behaviour for the gear (rejected, D2). A second settings surface (rejected — reuse the D-120 window).
