# M-RP6.1l — The Plugin List (the gear, the last disabled face)
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jul 2026  
> **Last updated**: 2026-07-12  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

> ## ✅ CLOSED 2026-07-12 (J-513) — commit `1dc5849`. Chat re-drove every leg (Rule 5); all of Clair's numbers reproduced.
>
> **⚠️ TWO CORRECTIONS TO THIS RUNBOOK — recorded here so it is never read as still-current:**
>
> **① §2 D6 is SUPERSEDED (Joe, 2026-07-12).** *“No Remove/Disable/Launch/Settings — the absent slot ships ABSENT, not faked”* was **too blunt**: it collapsed a **dead control** (grey because the verb was never built — it lies by implying the capability exists) with **W-13 RENDERED** (grey because the plugin's **own descriptor** says so — *a disabled `Remove` on a `[system]` row **IS** the information, and Ch6 §6.8.5 drew exactly that*). **🔒 THE RULE: every button's state DERIVES from the descriptor, never hardcoded, and a control is disabled only for a reason TRUE OF THAT PLUGIN and legible to the user.** The action row is **M-RP6.1m — ⏸️ POSTPONED**: four buttons, **zero live feeders**.
>
> **② §6 V2's literal `aria-disabled="false"` was WRONG.** `shelf-face.svelte` renders `aria-disabled={disabled || undefined}` — the attribute is **absent when enabled**, `"true"` only when disabled, and **never `"false"`**. Clair caught it against the source; Chat re-confirmed it on the live DOM.
>
> **And the finding that outlives the milestone: it should not have been built yet.** 6.1j's countdown carried a **schedule** beside its guard, and that schedule pulled this milestone ahead of the working grid. → ***A countdown names WHO discharges a disabled face — never WHEN.*** Full record: **J-513**.

**Design walk + lock: Chat Claude, 2026-07-12 (J-512). Joe: *"you have an autonomy in this part. do as you propose."*** Implementation: Clair. Chat re-drives every non-destructive verify leg (Rule 5). Clair's commit = **code only**; Chat's doc-bridge = commit 2 (D-074). **Joe pushes both.**

---

## 0. Read before you write a line

1. `CLAUDE.md` PLAY block → `JOURNAL.md` head (**J-512**, this lock; then J-511) → this file. **Rule 0. A runbook is item 4 on the reading stack, never item 1.**
2. `docs/xgen-plugin-taxonomy-phase0.md` **IN FULL** — this milestone is the first code that speaks D-112's three axes.
3. `docs/xgen-widget-surfaces-phase0.md` **S-5 / S-6** (the gear's meaning; **there is no minus button**) and **§6 ①** (Settings = `surface: window`, **CLOSED**).
4. `docs/xgen_ch6_client_design.md` **§6.8.5** (the Module List — Ch6 drew this before the widget tier existed).
5. `ui/docs/xgen-widget-tier.md` **W-12 / W-13** (at most one surface · `system` widgets are non-removable).

---

## 1. 🔑 The grounding that defines this milestone — THERE IS NO REGISTRY TO ENUMERATE

**Grepped 2026-07-12, not remembered. This is the finding the whole design turns on.**

| what | reality |
|---|---|
| `xgen-common/src/module.rs` | `Descriptor { kind_id, impl_id, name, assurance }`. **No `host`, no `delivery`, no `surface` field.** Its only consumer is `xgen-core/src/dag/store.rs` (`STORAGE_ENGINE_KIND_ID` + the `InMemoryEventStore` descriptor) — **node-side, and no Tauri verb exposes it to the client.** |
| `xgen-module.json` | **zero files, whole repo.** No manifest loader. No `modules/` scan. No local WebSocket server in the client. |
| Auth Module registry | `xgen-core/src/auth/module_registry.rs` — **node-side**, `delivery: service`. **The client has no verb for it.** |
| `xgen-node/src/plugins/temperature.rs` | node-side, `NoOpTemperaturePlugin`. |
| `ui/common/lib/components/widgets/` | **4 files** — `self-panel` · `inspector-panel` · `substitutions-editor` · `entity-context-menu`. |
| **mounted in the client** | **2.** `widgetRegistry` (`layout-default.ts`) = 8 region ids → 6 × `RegionPlaceholder`, `self` → `SelfPanel`, `inspector` → `InspectorPanel`. **`substitutions-editor` and `entity-context-menu` are imported NOWHERE in `ui/client`** (sampler-only). |
| W-13's `kind: system` | **a spec word with zero code.** No widget declares a kind, a name, a version or a descriptor anywhere. |

> ### **So 6.1l does not ENUMERATE a registry. It CREATES the first one — in TS, in the frontend — and it lists exactly what is real.**
>
> **A list that fakes a universal registry is worse than three honest rows.** *(The J-500 "there is no resident" shape and the J-502 temperature find, a third time: **a UI milestone cannot manufacture a source that does not exist.**)*

**The client cannot see a single `host: node` plugin, and this milestone does not invent a way to.** Node-plugin enumeration needs a Rust/protocol read verb that does not exist → **filed as `M-RP-PLUGINS-NODE`, not smuggled in.**

---

## 2. Decisions (locked — do not re-litigate)

**D1 — The registry becomes an ARTEFACT, not a lie.** New `ui/common/lib/plugins/registry.ts`: a typed `PluginDescriptor` + a `CLIENT_PLUGINS` const. **This is D-112's three axes in code for the first time.** No Rust — Rust owns nothing here (J-499's rule: Rust owns only what only Rust can do; a plugin descriptor is a type the webview owns).

**D2 — `widgetRegistry` is DERIVED from it (Leg B).** A widget is in the grid **because it is a registered plugin with `surface: region`**. One source, two readers — the N-096 shape. The 6 unbuilt regions keep their `RegionPlaceholder` fallback: **a placeholder is scaffolding, not a plugin, and it is not listed.**

**D3 — The pane's own surface is `none`, and that is not a dodge.** Per surfaces §3.2 + D-112, the plugin list is **content inside Settings** — **Settings** takes `surface: window`; the list **spends no surface**. So *any* host can mount it without the pane lying about what it is.

**D4 — SCOPE FORK, LOCKED: Settings does not exist, and 6.1l does not build it.** The pane ships with a **shell-local modal host** (the `about-dialog` / `uistate-*-dialog` precedent) as its **first** entry point. **Settings-the-window is its own milestone (`M-RP-SETTINGS`) and becomes the pane's SECOND mount** — which is *literally* S-2's "one component, two mounts". **Nothing built here is thrown away.**
- ❌ A grid tile is **rejected by D-112 §9**: dock the plugin manager into a 200px column, then remove the widget that removes widgets.
- ❌ Building a real second Tauri window now is a **frame arc** (own Vite entry, own chrome, own registry, own CDP target, own geometry) — and it would bury the visible part behind plumbing, against Joe's standing brief.

**D5 — THREE honest rows.** `self-panel` · `inspector-panel` · `plugin-list` (**it lists itself** — it is a real, mounted `host: client` plugin). **NOT** `substitutions-editor` / `entity-context-menu`: **the client never instantiates them**, and registering an unmounted plugin is the unfed-branch shape (N-091). They enter the registry **at the milestone that mounts them** (`substitutions-editor` → `M-RP-SETTINGS`).

**D6 — READ-ONLY rows. No Remove / Disable / Launch / Settings buttons.** There is **no remove verb, no disable verb, no launch verb, and no per-plugin settings schema**. A permanently-disabled control with **no countdown milestone behind it** is exactly what 6.1j forbade, and J-500's precedent is explicit: **the absent slot ships ABSENT, not faked.** What ships is the **`[system]` badge** — Ch6 §6.8.5's own drawing, which **is** W-13 made visible. **S-6 says destruction lives *here*; it does not say it lives here *now*.**

**D7 — ⚠️ NO W-8 PHASE-LIMIT NOTE ANYWHERE IN THE PANE. This is an N-109 pre-empt and it is binding.** A read-only list of what is loaded is **not a false statement about anything**, so there is **nothing to sweep at close**. **If any leg finds it needs a disclosure, its REMOVAL goes into the DoD of the leg that lifts the limit — in the same edit that adds it.** *(N-109: a stale honesty note is still a false statement, and worse than a missing one.)*

**D8 — Alphabetical order, no manual reorder** (Ch6 §6.8.5: *the list is not a priority indicator*).

**D9 — The dialog uses the STOCK core footer (a single Close).** ⚠️ **If you find yourself reaching for the `:has()` footer-suppression hack from 6.1k, STOP and flag it.** That would be the **second independent recurrence**, and the `dialog` footer-snippet-slot extraction stops being optional — **it is its own milestone, never a rider inside a shell milestone** (a `core` change inside a shell milestone is what makes a registry delta unreadable).

---

## 3. Scope — the files, and nothing else

**NEW**
1. `ui/common/lib/plugins/registry.ts` — the types + `CLIENT_PLUGINS`.
2. `ui/common/lib/components/widgets/plugin-list.svelte` — the **5th widget** (`kind: system`, `surface: none`, `data-tier="widget"` root, the `self-panel`/`inspector-panel` shape).
3. `ui/client/src/plugins-dialog.svelte` — shell-local, wraps the `core` `dialog`, mounts `PluginList`.

**MODIFIED**
4. `ui/client/src/app_client.svelte` — `pluginsOpen` state · `commandTable['widget.manager']` · **`gear` `disabled: false`** · mount the dialog.
5. `ui/client/src/layout-default.ts` — `widgetRegistry` **derived** from `CLIENT_PLUGINS` (D2).
6. `ui/assets/skin.css` — all `.plugin-list*` rules (N-090: gaps/sizing/tracks are skinnable too; **zero component-local `<style>`**).

**FORBIDDEN — and each is a verify leg, not a promise**
- ❌ **No Rust.** `cargo test` **MUST stay 1517/0/62 IDENTICAL** — the inverse of 6.1k's leg: *identical proves no Rust landed.*
- ❌ **No `ui/core/**`** — no new `core` component, no `dialog` change. Components registry gains a **widget**, not a cell.
- ❌ **No `ui/sampler/**`** — **sampler catalogue must stay 328**, proven **by scope** (`git show --stat`).
- ❌ No node-plugin enumeration. No manifest. No `xgen-module.json`. No Descriptor type in Rust.

---

## 4. The registry (D1) — shape

```ts
// ui/common/lib/plugins/registry.ts — D-112's three axes, in code for the first time.
export type PluginHost     = 'node' | 'client';            // system area | ui area
export type PluginDelivery = 'compiled' | 'service' | 'packaged';
export type PluginSurface  = 'none' | 'region' | 'shelf' | 'window';   // at most one (W-12)
export type PluginKind     = 'system' | 'custom';           // W-13: system => non-removable

export interface PluginDescriptor {
  id: string;                  // stable local id
  name: string;                // display name (alphabetical sort key, D8)
  description?: string;
  kind: PluginKind;
  host: PluginHost;
  delivery: PluginDelivery;
  surface: PluginSurface;
  regionId?: string;           // iff surface === 'region' — the D-103 leaf it occupies (regionId === widgetId, N-100)
  component?: Component;       // iff it has a surface the shell mounts
}

export const CLIENT_PLUGINS: PluginDescriptor[] = [ /* self-panel · inspector-panel · plugin-list */ ];
```

- **`host: 'node'` rows do not exist here.** The client cannot see them (§1). **Do not add a placeholder row for them.**
- W-3 holds: `registry.ts` lives in `$common` and imports only `$common` widgets — **never a shell dep.**

**Derivation (D2), in `layout-default.ts`:**
```
widgetRegistry = { ...REGION_IDS→RegionPlaceholder, ...CLIENT_PLUGINS.filter(surface==='region').map(regionId → component) }
```
The literal `self: SelfPanel, inspector: InspectorPanel` lines **go away**. The grid must render **identically** — that is V5.

---

## 5. Legs — VISIBLE FIRST (Joe's standing brief)

**A — the pane on screen, correctable, before anything is wired to anything.**
`plugin-list.svelte` + `plugins-dialog.svelte` + `commandTable['widget.manager'] = () => (pluginsOpen = true)` + **`gear` flips `disabled: false`** + skin. Reads `CLIENT_PLUGINS` directly. **Hand back here for Joe's eyes** — he reshapes the pane while the milestone is open (6.1k's Leg A precedent: he reshaped both dialogs mid-flight).

**B — the derive.** `layout-default.ts` takes its region map from `CLIENT_PLUGINS`. The grid must still resolve 8 leaves, `droppedCount: 0`, and `self` / `inspector` must still be the real widgets, not placeholders.

**C — CDP verify + close.** Chat re-drives every leg (Rule 5).

---

## 6. Verify — real client **9222 ONLY**. No sampler cells.

**⚠️ N-105 + N-108 — STATE THE CONDITIONS OR YOUR NUMBER IS UNREADABLE.** Every count is taken **QUIESCENT** (no menu / combobox / tag-select / colour-picker / context-menu open — `dialog` is the exception: closed = `display:none`, **not unmounted**) **AND with the store state named** (the Load dialog's picker/Load/Delete sit inside `{#if entries.length}`). **Baseline: client registry 55, quiescent, EMPTY STORE.**
**⚠️ The `plugin-list` rows register at MOUNT, not on open** (the dialog is always mounted). **So the baseline moves the moment Leg A lands. MEASURE the new baseline — do not derive it** (6.1k: a number derived by arithmetic was refused entry to the record until it had been *seen*).

| # | leg |
|---|---|
| **V1** | Baseline re-measured: registry count, `count === unique`, enumerated ids. **State: quiescent, empty store.** |
| **V2** | The **gear** is `aria-disabled="false"` **and** activates: click → the dialog is **`el.matches(':modal') === true`** (never the `open` attribute — J-496). |
| **V3** | Registry delta measured at mount and enumerated (`plugin-list#…` + its children). Close → **exact return** to the post-mount baseline, zero churn. |
| **V4** | Getter **G** exact: `{count: 3, systemCount: 3, customCount: 0}` — **render-truth** (`rowCount` counted from what rendered, not from the array length). |
| **V5** | **The derive (D2) proven, not asserted:** grid still resolves — `region-shell` G `{leafCount: 8, droppedCount: 0, unsupportedCount: 0}`, **split ratios `[1,2,7,2]` exact** (at whatever width you are at — a new width is a stronger proof than reproducing a number), and `self-panel#region-self` + `inspector-panel#region-inspector` are **still in the registry** (i.e. the derive did not silently drop them back to placeholders). |
| **V6** | **Rows read off the PAINTED DOM**, not the getter: three names, three `[system]` badges, the `host · delivery · surface` meta on each. *(N-097: a getter field is not a render.)* |
| **V7** | **Alphabetical** (D8) — read the painted order. |
| **V8** | Skin: all `.plugin-list*` rules in cascade, **zero component-local `<style>`**, **accent-neutral** under an injected `--accent2` swap (only the focus ring may move). **⚠️ Split the state-change and the DOM read across TWO evals** — a same-eval read gives `null === null` and a phantom green (N-099). |
| **V9** | **Static gates, apps DOWN** (`cargo test` with the client up dies on `failed to remove file …xgen-client.exe` — the running app holds the binary): `cargo test` **1517/0/62 IDENTICAL** (proves no Rust) · `vite build` (from **165**) · `npm test` **41** · `git show --stat` = the 6-file scope, **zero `ui/core/**`, zero `ui/sampler/**`** → **sampler catalogue 328 unchanged, by scope.** |

**CDP mechanics (measured, not folklore):** `.\cdp-debug.ps1 -App client -Mode eval -Expression "…"` — **there is no `-Eval` flag**, and passing one silently falls through to `-Mode state` and dumps a plausible-looking blob (bitten at J-511). The bridge exposes `ids` / `get` / `snapshot` — **there is no `count()`**. Single-expression `JSON.stringify({…})` evals only (PS 5.1). A just-enabled `<button>` clicked in the same eval is a **no-op** — `disabled` has not re-rendered yet (N-099).

---

## 7. Definition of Done

- [ ] `gear` is **ENABLED** and `widget.manager` resolves to a real pane. **The 6.1j countdown is discharged: no face in the app is disabled.**
- [ ] `CLIENT_PLUGINS` exists and **`widgetRegistry` derives from it** — one source, two readers.
- [ ] **Three rows, read-only, `[system]` badge.** No Remove / Disable / Launch / Settings control anywhere in the pane (D6).
- [ ] **No W-8 phase-limit note anywhere in the pane** (D7). *If one was added by any leg, its removal is in that leg's DoD and it is gone.*
- [ ] **Stock `dialog` footer.** No `:has()` suppression (D9). *If it was needed → flagged, not absorbed.*
- [ ] `cargo test` **1517/0/62, identical to baseline** — the proof no Rust landed.
- [ ] Scope clean by `git show --stat`: no `ui/core/**`, no `ui/sampler/**`, no Rust.
- [ ] **Every number MEASURED** (Rule 5) and **every count states its quiescence + store state** (N-105 / N-108).
- [ ] Deviations **flagged, not absorbed** (Rule 6).

*(Per D-074: "commit pushed" is **not** a DoD item. `Status: COMPLETED` on this file is the signal.)*

---

## 8. Filed, NOT built — none of it may be smuggled in

- **`M-RP-SETTINGS`** — Settings as a real `surface: window` plugin; the pane's **second mount** → *"one pane, two entry points"* becomes literally true. Also the first mount of `substitutions-editor` as content (surfaces §3.2), which is when it enters the registry (D5).
- **`M-RP-PLUGINS-NODE`** — a read verb exposing `host: node` plugins (storage engine · temperature · Auth Module) to the client. **Rust/protocol — the M-RP6.6 shape.** Until it lands, the list is honestly client-only.
- **The `dialog` footer-snippet slot** (filed J-511) — its own milestone (D9).
- **M-RP-ROVING** (D-069's four-recurrence bar met) · **M-RP-FOCUS** · **top-shelf pinning** (surfaces §6 ④) · **M-RP6.6 client resident** · the **read-marker protocol gap** · `temperature-indicator` ⏸️.

---

*Runbook for M-RP6.1l. Design locked 2026-07-12 (J-512), Joe: autonomy granted, proposal adopted as-is. Implementation: Clair. Verification: Chat re-drives every non-destructive leg.*
