# M-RP6.1h — R8 Selection info: the inspector, the bus's first cross-region reader
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-11  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

Canonical runbook for **M-RP6.1h**. Design-walked by Chat and **locked under Joe's explicit autonomy grant (2026-07-11)** — D1–D8 below. Chat's lane: this runbook + the CDP verify + the doc-bridge. Clair's lane: implementation + design closes.

**The milestone in one line:** R3 writes `{regionId, entity}` → **R8 renders that entity's rows**. The read loop closes end-to-end, and the selection bus gets its first **cross-region** reader.

---

## 0. GROUNDING — read before the design (source, not memory)

Chat has been wrong five times this arc from memory (J-497 ×3 · J-498 · J-499 · J-500). Everything below was **read from the tree on 2026-07-11**, not recalled. Clair: re-ground anything you are about to rely on; if a line here contradicts the code, **the code wins and you flag it (Rule 6)**.

| claim | grounded against | result |
|---|---|---|
| a leaf receives only `regionId` | `ui/core/lib/components/layout/region-node.svelte` | ✅ `<W regionId={node.widgetId} />` — **nothing else**. So for R8, `regionId === widgetId === "inspector"`. |
| R8 needs **no new store** | `ui/common/lib/stores/selection.svelte.ts` | ✅ `{ regionId, entity } \| null` with `current` / `set` / `clear` + a DEV `__XGEN_SEL__` handle. A plain `$common` import — the exact channel `self-panel` already uses. |
| R8 needs **no new Rust** | the bus is a frontend store | ✅ nothing to `invoke`. **Zero Rust in this milestone.** |
| **is a skin affordance already stranded here? (N-097)** | grepped `ui/assets/skin.css` | ✅ **No `.inspector*` rule exists.** Nothing pre-skinned, nothing to strand. *This check is now mandatory before any new widget — it is what N-097 was written for.* |
| a key/value row shape already exists | `ui/client/src/about-dialog.svelte` + `.about-grid` (skin 1037–1046) | ✅ `<dl>`: keys as plain `<dt>`, **values as `Label` (registry-visible)**. R8 is that shape, made dynamic. |
| what the descriptor actually carries | `ui/core/lib/components/data-dependent/types.ts` | ✅ `EntityDescriptor { kind, name?, id, flags?, image? }` — **five fields, three optional, `image` reserved-UNFED (D-065)**. |
| the empty-state pattern | `entity-panel.svelte` | ✅ composed `Paragraph` at `<id>__empty` + `hasEmpty` in the aggregate getter. **Reuse it; do not invent one.** |

---

## 1. Scope

**In — three files:**

1. `ui/common/lib/components/widgets/inspector-panel.svelte` — **NEW**. The second real **system widget** (`kind: system`, W-13), 4th widget overall.
2. `ui/client/src/layout-default.ts` — **one map entry**: `widgetRegistry.inspector → InspectorPanel`.
3. `ui/assets/skin.css` — the `.inspector-*` rules (**all** appearance, gaps, grid tracks — N-090).

**Out — do not build:**

- ❌ **Any Rust.** No `get_entity_info`, no new command (D2).
- ❌ **Any new store.** The bus already exists (D1).
- ❌ Any `ui/core/**` change. R8 composes **shipped** `core` unmodified.
- ❌ Any `ui/node/**`, any sampler change, any Tauri config / capability change.
- ❌ Any rendering of `descriptor.image` (reserved-unfed — that is the `tabs` branch again).
- ❌ Any second selection bus, any widening of the bus shape (region-dock §5; surfaces S-6).

---

## 2. Locked decisions

- **D1 — R8 reads the bus and nothing else.** `region-node` gives a leaf **only** `regionId`; with **W-3** (a `common` widget may not import a shell dep) that makes store-mediation the **only** channel (**N-096**). The bus **is** a `$common` store, so **R8 needs no new store and no new Rust.** Confirmed against the source, not assumed.

- **D2 — R8 renders the DESCRIPTOR, and says so. No `get_entity_info`.** `EntityDescriptor` is `{kind, name?, id, flags?, image?}`. That is a thin thing and the panel will be a thin panel. **Do not invent fields to make it look substantial.** A second Rust projection today would be a **second surface** (D-067 drift) delivering **zero** new information: the only selectable entity is *self*, whose fields already come from `get_self_state`. When R1/R2 land real spaces/rooms, a richer read can earn its keep **then**.

- **D3 — Rows (the `<dl>`, About shape).** Keys are plain `<dt>`; **values are `Label`** (registry-visible):

  | row | source | absent → |
  |---|---|---|
  | **Kind** | `descriptor.kind` | always present |
  | **Name** | `descriptor.name` | `—` (honest absence, the About guard — **never a fake name**) |
  | **ID** | `descriptor.id` | always present (the XGID) |
  | **Source** | `selection.current.regionId` | always present — **this is the field that makes R8 visibly a CROSS-REGION reader** |
  | **Flags** | `descriptor.flags` | **row not rendered** when no flag is set |

  **`image` is NOT rendered.** Reserved-unfed.

- **D4 — The flags row is conditional, and that is defensible.** Its true-branch is unreachable from the shipped UI today (self carries no flags) — but it **is exercisable at verify** by writing the bus directly (`__XGEN_SEL__.set(...)`), exactly how J-499 drove renderer A's drop paths. That makes it an **exercisable** branch, not an **unfed** one (the N-095/N-097 line). Render it as the comma-joined names of the flags that are `true` (e.g. `isAi, revoked`). **V4 must exercise it or it does not ship.**

- **D5 — Empty state = the `entity-panel` pattern, verbatim.** Root + `section` are **always mounted**; when `selection.current === null` the body is a composed `Paragraph` at `<id>__empty` reading **"Nothing selected"**. Rejected: `—`-guard row skeletons (fakes structure, destroys the select/clear delta) and a blank body (reads as broken). The always-mounted root is what makes **clear → exact return to baseline** the honest orphan proxy (N-092a / N-095b — the client bridge is state-only; there is no `domCount` leg).

- **D6 — Header = `entity-avatar`, NOT `entity-item`.** The avatar renders the **same five fields** more richly (`kind` → circle/square/hexagon, `name`, `id` → seed colour) — no new data, no new verb. **`entity-item` is deliberately refused: it carries a `selected` prop with a live `[data-selected]` skin rule, and in R8 "selected" is trivially always true** — that is an N-097 trap, and we route around it rather than into it. *Clair: ground what `variant="labeled"` actually renders; if it does not draw a name, that is fine — the Name row carries it. Do not add a name if the variant already draws one (no duplicate).*

- **D7 — Getter G (W-4):** `{ hasSelection, regionId, kind, rowCount }`.
  **The XGID and name are deliberately NOT republished** — they are already CDP-readable on the composed children's own getters (`label#region-inspector__id`, `…__name`); republishing is duplication **and** a payload leak (the N-060 `hasValue` precedent). `rowCount` is **render-truth** (the `message.detailsCount` precedent) — it is what makes the conditional flags row observable.

- **D8 — Naming + ids.** Component `inspector-panel.svelte`; envelope name `inspector-panel`; **`id = \`region-${regionId}\``** (N-096 leaf-id convention — the same one the seven placeholders use), so the delta reads as a **clean swap in place**.

---

## 3. Expected registry ids (the DoD is the ENUMERATED SET, not a number)

**Out:** `section#region-inspector` (the placeholder).
**In:**

```
inspector-panel#region-inspector          (the widget root — G)
region-inspector__section                 (Section)
region-inspector__empty                   (Paragraph — ONLY while nothing is selected)
region-inspector__avatar                  (entity-avatar — ONLY while something is selected)
region-inspector__kind                    (Label)   |
region-inspector__name                    (Label)   |  ONLY while something
region-inspector__id                      (Label)   |  is selected
region-inspector__source                  (Label)   |
region-inspector__flags                   (Label)   (ONLY when a flag is set)
```

**⚠️ Rule 5.** The baseline is **36 as measured at J-500** — do **not** predict the new totals, and do **not** inherit a number from a handback. Chat re-measures every count itself; a number nobody measured does not enter a canonical record. *(Clair's V9 number did not reproduce three times this arc.)*

---

## 4. Implementation notes

- **Zero component `<style>`.** Every `.inspector-*` rule — including the `<dl>` grid tracks, gaps and sizing — lives in `skin.css` (**N-090 / N-094**). The right question is never *"would this be the first `<style>`?"* but *"could a skinner want to retune this?"*
- **Skin shape:** `.inspector-panel` (frame + padding) · `.inspector-head` (the avatar) · `.inspector-grid` `dt`/`dd` (the kv grid — the `.about-grid` shape at skin 1037–1046 is the reference). **Do not extract a shared `.kv-grid`** — this is only the second recurrence; D-069's bar is four. *Flagged for the fourth.*
- **Accent:** expected **accent-neutral** (the avatar's colour is seed-derived, not `--accent`). **Expected, not asserted** — V7 measures it.
- **Reactivity:** `const sel = $derived(selection.current)`. Everything else derives from `sel`.
- **`kind: system` (W-13)** — mark Level-2 via `data-tier="widget"` on the root, the `self-panel` shape.

---

## 5. Verify — real client 9222 (D-097). Chat re-drives EVERY leg itself (Rule 5)

Sampler is structurally irrelevant here (no frame, no bus, no shell). **Single-expression `JSON.stringify({…})` evals only** (PS 5.1, N-098).

- **V1 — Registry.** `count === unique`, **enumerated**. `section#region-inspector` **out**, the §3 set **in**. Measured, not predicted.
- **V2 — Empty state on launch.** G `{hasSelection:false, rowCount:0}`; `region-inspector__empty` present; **zero** row ids registered.
- **V3 — 🔑 THE MILESTONE. The loop closes.** Click R3's `entity-item` → R8 renders: `kind:identity`, the **real keypair-derived XGID**, the name, `Source: self`.
  **⚠️ The leg is the RENDERED TEXT read out of the DOM — not the getter.** N-097, third recurrence: *a state flag is not a render.* Read the `<dd>` label text nodes and compare them to the bus payload.
  Also: **R3's own gold `[data-selected]` bar still paints** (no regression).
- **V4 — R8 does not depend on R3.** Drive the bus directly: `__XGEN_SEL__.set('spaces', {kind:'space', id:'…', name:'…', flags:{isDm:true}})` and again with `kind:'room'`. Prove: the avatar's `data-shape` flips (**square** / **hexagon**), `Source` flips to `spaces`/`rooms`, and **the flags row renders** (`rowCount` 4 → 5, `isDm` in the text). **This is the proof R8 reads THE BUS, not R3** — and it is the only leg that exercises D4.
- **V5 — Clear → empty; registry returns EXACTLY to baseline.** `__XGEN_SEL__.clear()` → G `{hasSelection:false}`, the row ids gone, `__empty` back, **and the enumerated id set is identical to V2's**. *(The client bridge is state-only — this exact-return is the ONLY orphan proxy available, N-092a.)*
- **V6 — Geometry (N-091, a REQUIRED leg for anything layout-class).** `docNoScroll` true; the R8 leaf **self-scrolls** while the document does not (inject tall content, then restore); the members/inspector column holds its `sizes [1,1]`.
- **V7 — Skin.** The `.inspector-*` rules **in cascade** (stylesheet-rule inspection, N-042 — `getComputedStyle` is not the method for this); **zero component `<style>`** (grep the file); accent behaviour **measured** under an injected `--accent2` swap.
- **V8 — Static, run with the apps DOWN** (target-dir contention, J-500): `cargo test --workspace` · `npm test` · `vite build` · `git show --stat <feat>` = **exactly the three §1 files** — no Rust, no `ui/core/**`, no `ui/node/**`, no sampler.

---

## 6. Definition of Done

- [ ] `inspector-panel.svelte` shipped in `ui/common/lib/components/widgets/`; `kind: system`; zero `<style>`.
- [ ] `widgetRegistry.inspector` swapped — **one entry**, no rewrite.
- [ ] All `.inspector-*` appearance in `skin.css`.
- [ ] V1–V8 green, **every leg re-driven by Chat**, real output quoted (Rule 2).
- [ ] The **painted-text** leg (V3) run — not the getter alone.
- [ ] The **flags** branch exercised (V4) — or the row is cut, not shipped unverified.
- [ ] `git show --stat` = the three §1 files.
- [ ] Deviations **flagged, not absorbed** (Rule 6) — including Chat's own.

*(No "commit pushed" item — `Status: COMPLETED` is the shipped signal.)*

---

## 7. Out of scope / carried

- **M-RP6.6 client resident** — the real F-1 close. **There is no resident**: `run_startup` connects once for 2 s, drops the stream, and never touches the socket again. **No part of it may be smuggled into this milestone.** Any connection claim stays relaunch-scoped.
- **`docs/xgen-widget-surfaces-phase0.md` §6** — 5 items open for Joe. **6.1h does not depend on it; 6.1i–l cannot start without it.**
- R1/R2 (the bus's next writers) · `entity-context-menu` re-wiring to the bus · M-RP7.3's corrupt-layout fallback leg (N-095).

---

## 8. Commit discipline (D-074)

1. **Commit 0 (this file)** — Joe pushes the runbook.
2. **Commit 1 — Clair's feat, CODE ONLY** (the three §1 files). No doc, no JOURNAL, no ROADMAP.
3. **Commit 2 — Chat's doc-bridge** — JOURNAL J-501 + `ui/docs/xgen-ui-notes.md` + `ui/docs/xgen-ui-components.md` + `docs/xgen-client-frame-phase0.md` §6 + `ui/docs/xgen-region-dock-model.md` §5 + `docs/ROADMAP.md` + this file → **COMPLETED**.
4. **Joe pushes both.** Chat never pushes.

---

*End of M-RP6.1h runbook.*
