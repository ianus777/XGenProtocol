# entity-context-menu — Phase-0 (M-RP5.3)
> **Status**: PENDING  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-06  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

Design lock for `entity-context-menu` — the **first widget with a real dd-dependency** and the first to exercise the **W-11 dd-socket** in earnest. It is the "100% entity read" every avatar variant was built to defer to (N-075 H). **Design-only; no code this arc** (D-065). Runs against the **widget checklist (W-1..W-11)**, not the dd audit.

---

## 1. What it is

A **widget** (Level-2, active, pluggable — `ui/common/lib/components/widgets/`). A droppable overlay that, given one entity, presents an action list and dispatches consumer-wired handlers. The universal `identity` item opens the full entity read; other items are flag-gated slots filled later when running apps surface the need.

**Grid model (declared, not populated).** rows = avatar `variant`, cols = **purpose** (a superset of the avatar axis — a variant can be raised in several action-contexts). Cell = item-set = `f(variant, purpose)`, gated by `(kind, flags)`. The base version ships the **machinery + the universal `identity` item**; the catalogue grows with tests + real usage.

---

## 2. A→H lock

**A — Tier justification (honest W-2).** Roving focus alone is *not* the discriminator — `entity-panel` (composite) owns roving focus + selection. Three things push this over the W-2 line: (1) an **overlay mount/unmount lifecycle** (appear → focus-trap → dismiss-policy → tear down); (2) it **dispatches host-integrating side effects** (message / block / navigate); (3) it is the **first real W-11 dd-socket consumer** (binds `EntityDescriptor` + a status view-model). That trio = a behaviour contract. **Affirmed as a widget on (1)+(2)+(3), not on roving-focus.**

**B — Trigger seam.** **Gesture-agnostic.** Widget exposes `open(anchor)` / `close()`; the consumer wires the gesture — the avatar/item reserved `onActivate?` (N-075 H), plus right-click / long-press, all route to `open`. Widget owns everything *after* open, never the gesture (keeps it sampler-testable without a real right-click).

**C — Rooting + anchor.** Root `<div class="entity-context-menu" role="menu">`, `use:envelope` `data-tier="widget"`. Anchored to a passed anchor (element rect or point) with **viewport flip/shift**. Real shells need **portal-to-body** (avatars live inside `overflow` panels that would clip it). **Design-for-portal; the sampler pure layer uses an absolute popup; portal + real anchoring = effect-layer (§4).**

**D — Behaviour contract / lifecycle (the W-2 machine).** `closed → open(anchored, focus moved in) → navigating(roving) → dispatch(run handler) → closed`. Dismiss: **Esc**, **outside-click**, **select-then-close**, **anchor blur**. Focus returns to the anchor on close. This machine *is* the widget.

**E — Action model + header.** Menu = **header** (composes `entity-avatar` + name + `status` **`full`** — the identity-read surface) over an **item list**. Items derive `f(variant, purpose)` gated by `(kind, flags)`; **base ships only the universal `identity` item** (kind-labelled: `identity` for identity-kind; space/room label reserved), rest are flag-gated slots. Item rows = **widget-owned `<li role="menuitem">`** (layout + dispatch is the widget's contract; the header still composes core per W-1). A `menu-item` core di is a **reserved** future factoring if items proliferate.

**F — Keyboard nav.** Roving `menuitem` (one `tabindex=0`); Arrow↑↓ / Home / End / Enter+Space (dispatch) / Esc (close). **Typeahead deferred** (D-065).

**G — Getter (W-4).** `{ open, variant, purpose, kind, itemCount, activeIndex }` — observable task-state only, **no payload / entity secrets** (N-060 precedent). Child `core` self-register under `<id>__<slot>` (`__avatar`, `__status`).

**H — Seams.** *Consumes:* `onActivate?` (avatar/item) as the open trigger; the **`EntityDescriptor`** + a `status` view-model via the **W-11 dd-socket** (source-agnostic — `core` imports no protocol type); consumer-wired **action handlers** (W-3 callback-injection form). *Reserves:* the full grid (variant × purpose) + the item catalogue + the space/room universal-item label + a `menu-item` core di.

---

## 3. W-1..W-11 conformance

- **W-1 Composes down only** — header composes `entity-avatar` + `status` (`core`); `menuitem` rows = layout, dispatch owned by the widget. ✔
- **W-2 Owns state + lifecycle** — the overlay machine (§2 D), beyond one momentary toggle. ✔ (discriminator, §2 A)
- **W-3 I/O via declared seams** — action handlers are host-injected callbacks; no `invoke`/`fs`/`fetch` in the body. ✔
- **W-4 One aggregate getter** — §2 G; children self-register. ✔
- **W-5 Clean mount/unmount** — outside-click / key listeners wired on open, torn down on close/unmount (0-orphans). ✔
- **W-6 Skin L2 only; pure/effect separable** — zero `<style>`; `.entity-context-menu` in `skin.css`; pure layer runs with handlers stubbed. ✔
- **W-7 Scoped home + Phase** — `ui/common/…/widgets/`; **Phase A** (pure Svelte; no `invoke` — handlers injected). ✔
- **W-8 Honest phase-limits** — base ships only the universal item; flag-gated slots visibly reserved, not silently absent. ✔
- **W-9 Representation** — ordinary `.svelte`, `data-tier="widget"`, WIDGET tab; **connection v1 = static import + placement**. ✔
- **W-10 Plugin contract** — mount lifecycle · one getter · callback/store-mediated I/O · declared Phase. ✔
- **W-11 dd-socket** — binds `EntityDescriptor` + status view-model; the dd-component/data binds the store, never widget internals; first real exercise. ✔

---

## 4. Two-layer verify (§5 of the widget-tier spec)

**Pure / presentational layer — sampler WIDGET tab (CDP 9422).** Registry entry present; state-machine CDP-asserted with handlers stubbed: `open()`→`{open:true}`; ArrowDown→`activeIndex` advances; Esc→`{open:false}`; select→handler fires + closes; header renders `entity-avatar` + `status full`; skin in cascade; both accents; **0 orphans**.

**Effect layer — real shell (client/node, CDP 9222/9322).** Portal-to-body; real anchor to a live panel avatar (escapes `overflow` clip); a real consumer handler round-trip (e.g. `identity` → navigate/open); real output quoted (Rule 2).

**Done = both layers green.**

---

## 5. Roadmap — M-RP5.3

- **Phase-0** (this doc) — A→H lock, W-conformance, verify plan. Design-only. → this file.
- **Step-1** — widget scaffold (`role=menu` root, `data-tier`, getter G, `open()/close()`).
- **Step-2** — behaviour machine D (open/focus/roving/dismiss/dispatch-and-close).
- **Step-3** — header (`entity-avatar` + name + `status full`) + seed `identity` item + consumer-handler dispatch.
- **Step-4** — skin L2 + sampler WIDGET cells + **pure-layer CDP**.
- **Step-5** — **effect-layer** real-shell (portal + real anchor + handler round-trip) + **D-074 atomic close**.

Registry delta measured at build (+1 widget + composed `__avatar`/`__status` children).

**After 5.3:** `temperature-indicator` (M-RP5.4, `meter` via W-11) closes the widget tier.

---

*UI-architecture Phase-0. No protocol/data implication. Locks the A→H design for `entity-context-menu`; build deferred to a later session (Clair code seat). Standing: MP-R3 capstone ledger owed on that milestone's close (`tasks/HANDOFF_MP_R3.md`).*
