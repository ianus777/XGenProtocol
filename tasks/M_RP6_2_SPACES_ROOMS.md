# M-RP6.2 — R1 Spaces + R2 Rooms on real `KnownSpace`
> **Status**: COMPLETED  
> Version: 1.3  
> Date: Jul 2026  
> **Last updated**: 2026-07-17  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

Canonical runbook for **M-RP6.2**. Design-walked and grounded by Chat (2026-07-17); **anchors re-confirmed against the live tree 2026-07-17 → v1.1 corrects Anchor 5 (N-116): the placeholder swap is NO LONGER "register a system widget" — since M-RP6.1l (the plugin registry) and M-RP-CONNSTATS (the reactive derivation), a region widget is swapped in by adding a `PluginDescriptor` to `CLIENT_PLUGINS`, which `buildWidgetRegistry` derives. The record was self-consistent; the code had moved. This changes the Leg-A/B wiring and adds two plugin-list rows (below).** **Status is PENDING — this awaits Joe's lock of D1–D8; it flips to ACTIVE the moment Joe locks, and nothing is coded before that.** Chat's lane: this runbook + the CDP verify + the doc-bridge. Clair's lane: implementation + design closes + the Rust half.

**The milestone in one line:** replace the `RegionPlaceholder` in **R1 Spaces** and **R2 Rooms** with real system widgets on the live `KnownSpace` tree — select a space in R1 → R2 shows its rooms — through the same `{regionId, entity}` bus R3→R8 already proved.

**Why it is more than a placeholder swap:** it is the bus's **second and third writers** (R1 space-select, R2 room-select), its **first real cross-region data flow** (R3→R8 used a synthetic `__XGEN_SEL__` feed; this is real state), and it lands the **first UI-facing space-list read command** in the project.

---

## 0. GROUNDING — read before the design (source, not memory)

Everything below was **read from the tree on 2026-07-17**, not recalled. Clair: re-ground anything you are about to rely on; if a line here contradicts the code, **the code wins and you flag it (Rule 6)**.

| claim | grounded against | result |
|---|---|---|
| the read verb exists | `xgen-client/src/ops.rs:246` | ✅ `ops::spaces(ctx) -> SpacesResult { spaces: Vec<KnownSpace> }` — zero-network, reads `xgen-client_state.json`. Unit-tested: `spaces_returns_known_spaces`, `rooms_errors_on_unknown_space`. |
| **rooms are embedded in spaces** | `xgen-common/src/state.rs:185-199` | ✅ `KnownSpace { space_id, name, node_endpoint, role, rooms: Vec<KnownRoom> }` · `KnownRoom { room_id, name, joined }`. **The whole tree comes in one read** → R2 needs no second fetch, and no `get_rooms` UI command (D1). |
| **no Tauri command exposes it yet** | `xgen-client/src/desktop.rs` `invoke_handler` list | ✅ `get_state` is lifecycle only. There is **no `get_spaces`**. → this milestone lands **one thin Rust command** (D1). |
| the command shape to clone | `desktop.rs:312-366` `get_self_state` / `SelfStateInfo` | ✅ a shell-read composing an existing `ops::*` reader — **NOT a D-092 four-armed verb**: no node round-trip, no mutation, no CLI meaning, no `ops.rs` touch. |
| the once-on-mount hydrate site | `ui/client/src/app_client.svelte:383` | ✅ `selfState.setIdentity(await invoke('get_self_state'))`. R1 adds one sibling line: `spacesState.setSpaces(await invoke('get_spaces'))`. |
| a leaf receives only `regionId` | `ui/core/lib/components/layout/region-node.svelte` | ✅ `<W regionId={node.widgetId} />` — **nothing else**. All data is `$common`-store-mediated (W-3). So `regionId === widgetId === "spaces"` / `"rooms"`. |
| the store pattern to clone | `ui/common/lib/stores/self-state.svelte.ts` | ✅ module-level `$state` in a `.svelte.ts`, snake_case verbatim (no mapping layer), DEV `__XGEN_SELF__` handle. New store mirrors it: `spaces-state.svelte.ts`. |
| the widget pattern to clone | `ui/common/lib/components/widgets/self-panel.svelte` | ✅ `id = \`region-${regionId}\``, `use:envelope`, `selected` **derived from the bus**, `onActivate → selection.set(regionId, descriptor)`, `data-tier="widget"`. |
| the bus contract | `ui/common/lib/stores/selection.svelte.ts` | ✅ `selection.set(regionId, entity)` / `.current` / `.clear()`, `{ regionId, entity } \| null`, DEV `__XGEN_SEL__`. **One selection, never a list — do not widen.** |
| **no new `core` component** | `ui/core/lib/components/data-dependent/entity-panel.svelte` | ✅ takes `items: EntityItemInput[]`, owns roving focus + single-select, `selected` ($bindable id), `onActivate(id)`, `emptyText`, `id`. Its own header names `spaces-panel` a **preset, not a component**. `entity-avatar` already draws **hexagon for `kind:'room'`** (J-501). |
| the descriptor maps cleanly | `ui/core/lib/components/data-dependent/types.ts` | ✅ `EntityDescriptor { kind:'identity'\|'space'\|'room', name?, id, flags?, image? }`. `role`/`node_endpoint`/`joined` have no slot (fine — view-model, not raw record). |
| **the placeholder swap seam (CORRECTED v1.1)** | `ui/client/src/layout-default.ts:64` `buildWidgetRegistry` | ✅ every `REGION_IDS` id → `RegionPlaceholder`, then each plugin with `surface==='region' && regionId && component` **overrides**. **It reads `installed.mounted`, NOT a literal registration in the shell.** `DEFAULT_LAYOUT` already has leaves `widgetId:'spaces'` / `'rooms'`. → the swap is done by **adding a `PluginDescriptor`**, not by an `app_client` register line. |
| **the plugin descriptor list (the real registration site)** | `ui/common/lib/plugins/registry.ts:80` `CLIENT_PLUGINS` | ✅ `self`/`inspector` are `PluginDescriptor` entries here (`kind:'system'`, `host:'client'`, `delivery:'compiled'`, `surface:'region'`, `regionId`, `component`, `name`, `icon`, `version`). **Adding `spaces-panel`/`rooms-panel` here IS the registration.** `installed.svelte.ts:57/63` — both `active` (plugin-list) and `mounted` (widgetRegistry) begin `...CLIENT_PLUGINS` unconditionally, so a **system** entry needs NO install step and NO shell wiring. Consequence: **2 new `[system]` rows in the plugin-list** (Settings → Plugins), and each descriptor's **`name` becomes the tile title** via `buildTitles` (D8). |
| the reactive registry derivation | `ui/client/src/app_client.svelte:70-73` | ✅ `widgetRegistry = $derived(buildWidgetRegistry(installed.mounted))`. The shell does NOT register widgets by name — it derives the whole map. So Leg A's ONLY `app_client` change is the **hydrate line**; Leg B touches `app_client` **not at all**. |
| **N-097 skin-strand check** | grep `ui/assets/skin.css` for `.spaces` / `.rooms` **BEFORE build** | ⛔ **Clair: run this at Phase-0.** If a `.spaces-panel*` / `.rooms-panel*` / `[data-selected]` rule already exists, the widget MUST feed it or you strand a shipped affordance (the N-097 lesson). Record the result in the close. |

---

## 1. Files & scope

**Leg 0 — Rust (`get_spaces` command).** cargo test **MUST move** (this leg lands Rust — the honest signal; it does NOT stay 1517/0/62).
- `xgen-client/src/desktop.rs` — add `#[tauri::command] fn get_spaces(...)` (the `get_self_state` shape) + register it in `invoke_handler`. Returns `Vec<KnownSpace>` verbatim.
- one shape/serialize test (command wrapper or the returned struct), so the count moves honestly.

**Leg A — R1 Spaces (frontend).** Registry baseline **99** → +N (measure after a full reload, N-132; do not assert a literal). Cite the store state you counted in (empty store, `sel:null`) — N-108.
- `ui/common/lib/stores/spaces-state.svelte.ts` — new `$common` store (self-state shape): declares the TS `KnownSpace`/`KnownRoom` interfaces (snake_case verbatim, the `SelfStateInfo` precedent — so `core` still imports NO protocol type), holds `KnownSpace[]`, `spaces` getter, `setSpaces(list)` setter, DEV `__XGEN_SPACES__`.
- `ui/common/lib/components/widgets/spaces-panel.svelte` — new `kind: system` widget; reads `spacesState`, does the `KnownSpace → EntityItemInput` projection **in the widget** (the self-panel precedent, §4/D7); renders an `entity-panel` of spaces; writes the space selection; `selected` derived from the bus.
- `ui/common/lib/plugins/registry.ts` — **add the `spaces-panel` `PluginDescriptor` to `CLIENT_PLUGINS`** (`kind:'system'`, `host:'client'`, `delivery:'compiled'`, `surface:'region'`, `regionId:'spaces'`, `component: SpacesPanel`, `name` + `icon` + `version` per D8). **This IS the registration** — the placeholder swap follows automatically (grounding table). A W-3-clean import (`$common` widget from `$common` registry).
- `ui/client/src/app_client.svelte` — **ONE line only**: the hydrate `spacesState.setSpaces(await invoke('get_spaces'))` in the mount `try`, beside `:383`. No register line — `widgetRegistry` derives it (`:71`).

**Leg B — R2 Rooms (frontend).**
- `ui/common/lib/components/widgets/rooms-panel.svelte` — new `kind: system` widget; latches its space scope from the bus (§3.2); reads that space's `.rooms` from `spacesState`; projection in the widget; `entity-panel` of rooms; writes the room selection.
- `ui/common/lib/plugins/registry.ts` — **add the `rooms-panel` `PluginDescriptor` to `CLIENT_PLUGINS`** (`surface:'region'`, `regionId:'rooms'`, `component: RoomsPanel`, D8 name/icon). **No new store, no new invoke, and NO `app_client` change** (rooms ride inside `spacesState`, D1; the swap derives from the descriptor).

**Out of scope (do not build):** any node round-trip / live push (that is the resident, **M-RP6.6** — this closes only the *read* shape, as 6.1g did for R3); `get_rooms` as a UI command; appearance polish (→ M-RP-SKIN); the `role`/`joined`/`node_endpoint` fields surfacing anywhere (no descriptor slot; filed, not faked).

---

## 2. Leg 0 — the `get_spaces` command (D1)

A thin shell-read, the `get_self_state` shape. It composes the existing, unit-tested `ops::spaces` — **no new `ops.rs` logic**.

```rust
/// The known-Space tree for R1/R2 (M-RP6.2). A thin shell read (the `get_self_state`
/// shape) — NOT a D-092 four-armed verb: no node round-trip, no mutation, no CLI
/// meaning, so it never grows those arms and never touches `ops.rs`. Rooms ride
/// EMBEDDED in each `KnownSpace` (`state.rs`), so R2 reads them from the same
/// payload — there is no `get_rooms` UI command (D1). Carried VERBATIM (snake_case
/// as Rust serialises it) — no rename/mapping layer (a mapping layer is a drift
/// surface; the self-state precedent). `Err` (state file absent / unregistered)
/// is NOT an error to the webview — it is the honest empty list (D-065 / W-8).
#[tauri::command]
fn get_spaces(data: tauri::State<DataDir>) -> Vec<xgen_common::state::KnownSpace> {
    let data_dir = data.0.clone();
    let mut session = crate::session::SessionState::new(String::new(), data_dir.clone());
    let mut ctx = crate::ops::OpContext { session: &mut session, data_dir: &data_dir, node_override: None };
    crate::ops::spaces(&mut ctx).map(|r| r.spaces).unwrap_or_default()
}
```
Register in `invoke_handler` next to `get_self_state`. App-defined command → no capability grant (the J-497 grounding: `get_substitutions` et al. run with none).

**Honest empty (Q4):** unregistered → `xgen-client_state.json` absent → `Err` → `unwrap_or_default()` → `[]` → R1 renders its empty state. A registered client carries ≥1 space (the self-thread `KnownSpace`, M11-D5). **No faked rows (N-091).**

---

## 3. Legs A & B — the widgets

### 3.1 R1 `spaces-panel` (the space writer)

Mirror `self-panel` exactly. The widget renders an `entity-panel` fed the mapped spaces, and writes the bus on activation.

- `id = \`region-${regionId}\`` (→ `spaces-panel#region-spaces`, children `region-spaces__section` / rows), `use:envelope`, `data-tier="widget"`.
- `items` = `EntityItemInput[]` projected **in the widget** from `spacesState.spaces` (§4). The store carries raw `KnownSpace[]`; the widget maps `KnownSpace → { descriptor }`. This is the **self-panel precedent exactly** (`self-panel.svelte:40` builds its `EntityDescriptor` in the widget from the `$common` store) — the widget can't receive shell props (W-3), so the projection lives where the data is read.
- `selected` (D5, bus-derived): `selection.current?.entity.kind === 'space' ? selection.current.entity.id : undefined`.
- `onActivate(id)` → look up the `KnownSpace` by `space_id`, build its descriptor, `selection.set('spaces', descriptor)`.
- `emptyText`: honest — e.g. `"No spaces yet"`.
- aggregate getter G (W-4): `{ count, selectedId, hasEmpty }` — report only what the panel owns; the rows' own getters carry names/ids (no republish, N-060).

### 3.2 R2 `rooms-panel` (the room writer + the ONE new mechanic — D3)

R2 both **reads** the bus (to scope which space's rooms to list) and **writes** it (activating a room). Because R2's own write moves the bus to a `kind:'room'`, R2 **cannot** read its scope from `selection.current`. It **latches**:

```ts
// The last SPACE selection — R2's data scope. Latched from the bus, and KEPT when
// R2's own room-activation moves the bus to a room (D3). This is the milestone's
// only new logic: without it, clicking a room blanks R2's own list.
let latchedSpaceId = $state<string | null>(null);
$effect(() => {
  const c = selection.current;
  if (c?.entity.kind === 'space') latchedSpaceId = c.entity.id;
});
const scopedSpace = $derived(spacesState.spaces.find(s => s.space_id === latchedSpaceId) ?? null);
const rooms = $derived(scopedSpace?.rooms ?? []);
```
- `selected` (bus-derived, room facet): `selection.current?.entity.kind === 'room' ? selection.current.entity.id : undefined`.
- `onActivate(id)` → look up the `KnownRoom` in `scopedSpace.rooms`, build its descriptor, `selection.set('rooms', descriptor)`.
- **Two honest empty states (N-091):** `latchedSpaceId === null` → `"Select a space"`; space selected but `rooms.length === 0` → `"No rooms"`. Distinct copy — they are different truths.
- **Stale-latch guard:** if the latched space is no longer in `spacesState` (removed between hydrations), `scopedSpace` is `null` → fall back to the `"Select a space"` empty state, never throw (N-095 spirit).

### 3.3 OPEN LOCK — R1 highlight when a room is selected (D4)

There is ONE bus. When R2 selects a room, `selection.current` is a room, so R1's bus-derived `selected` goes `undefined` and **the space un-highlights in R1** while you browse its rooms.

- **D4-opt-1 (RECOMMENDED — bus-pure):** R1 highlights only when the bus holds that space. Simplest, single truth, no second latch. R2's latch stays a **data-scope** concern only, never a highlight one.
- **D4-opt-2 (R1 latches too):** R1 also latches the last space so it stays visually selected while you browse rooms. More faithful UX, but invents a second "active vs selected" concept in R1.

**Chat recommends opt-1** for this milestone (minimal, honest, one selection meaning); opt-2 is filed for a later polish pass if Joe wants the persistent "which space am I in" highlight. **Joe locks D4.**

---

## 4. `KnownSpace / KnownRoom → EntityDescriptor` (the shell's map)

```
KnownSpace → { kind: 'space', id: space_id, name }          // role / node_endpoint: no slot (filed)
KnownRoom  → { kind: 'room',  id: room_id,  name }           // joined: no slot (filed)
```
`EntityItemInput` wraps the descriptor with optional `secondary` / `status` / `meta` (caller strings). v1: `secondary`/`meta` **unfed** (D-065 — no faked "last message" / "unread"; the read-marker gap has no protocol mechanism yet, per the PLAY note). `entity-avatar` picks shape from `kind` (space→square/circle by `isDm`; room→hexagon) with **no extra wiring** (J-501 ground truth).

**Layering discipline:** `core` imports no protocol type. The TS `KnownSpace`/`KnownRoom` interfaces are declared **in the `$common` store** (the `SelfStateInfo`-in-`self-state.svelte.ts` precedent — a mirror of the Rust serialisation, not an import of a Rust type); the `KnownSpace → EntityDescriptor` projection lives **in the widget** (self-panel builds its descriptor in the widget). So the raw type stays in `$common`, the projection stays in `$common`, `core` sees only `EntityDescriptor`, and W-3 holds (no shell dep imported by a `$common` widget). Keep the projection in one place per widget; do not leak a raw `KnownSpace` into an `entity-*` prop.

---

## 5. Verification (Chat, Rule 5 — re-driven live after a full reload, N-132)

Client **9222 only**. Split every set-then-read across two evals with a settle delay, and **assert both reads non-null before comparing** (N-099 — the phantom-green trap). Baseline registry cite **99**.

- **V0 (Rust):** `cargo test` moved vs 1517/0/62 (the Leg-0 shape test). Record the new triple.
- **V1 (hydrate):** after a full reload, `__XGEN_SPACES__` carries the real `get_spaces` payload (≥1 space for a registered client; `[]` honestly for unregistered).
- **V2 (R1 list):** `region-spaces` renders one row per space; `entity-panel#region-spaces` getter `count` matches the payload length; **empty payload → the "No spaces yet" row**, not zero rows silently.
- **V3 (R1 writes):** click a space row → `__XGEN_SEL__.current === { regionId:'spaces', entity:{kind:'space', id:…} }`; the row **paints** selected (the `[data-selected]` bar, not just the attribute — N-097).
- **V4 (cross-region, the milestone's real proof):** selecting a space in R1 → **R2 repopulates** with that space's rooms **AND** R8 (inspector) shows the space. Drive it via a real R1 click, not `__XGEN_SEL__`.
- **V5 (R2 latch — D3):** with a space selected, click a **room** in R2 → `__XGEN_SEL__.current` is now `{kind:'room'}`, R8 flips to the room, **and R2 KEEPS its room list** (latch held). This is the leg that proves D3.
- **V6 (R2 empty states):** no space selected → "Select a space"; a space with zero rooms selected → "No rooms". Two distinct rows.
- **V7 (bus purity re-proof):** drive `__XGEN_SEL__` directly with a `space` id **no R1 row exists for** → R8 still renders it (proves R8 reads the bus, not R1 — the J-501 standing rule, re-checked now that R1 is a real writer).
- **V8 (plugin-list consequence — the v1.1 correction's own proof):** open Settings → Plugins → the list now shows **two new `[system]` rows** (the D8 names) with the host-tinted glyph; the `plugin-list` getter `count` moved **3 → 5** (`systemCount` 3 → 5). This confirms `spaces-panel`/`rooms-panel` registered as descriptors, not as ad-hoc shell registrations. Their **tile titles** (R1/R2 stripes) read the D8 `name`, not the old `REGION_NAMES` `R1 · …`/`R2 · …`.

---

## 6. Definition of Done

- [ ] **Leg 0:** `get_spaces` command shipped + registered; `cargo test` moved from 1517/0/62 (record the new triple).
- [ ] **Leg A:** `spaces-state` store + shell hydrate + `spaces-panel` swap the `spaces` placeholder; V1–V3 pass live on 9222 after a full reload.
- [ ] **Leg B:** `rooms-panel` swaps the `rooms` placeholder; the D3 latch holds; V4–V6 pass live.
- [ ] **V7** re-proves bus purity with a real R1 writer present.
- [ ] **V8** — the two `[system]` plugin-list rows present, `plugin-list` count 3→5, tile titles = the D8 names.
- [ ] **N-097 skin check** (§0) recorded — ✅ already grounded 2026-07-17: **no `.spaces`/`.rooms` rule in `skin.css`**, nothing to strand. (Clair: re-grep before build in case a rule landed since.)
- [ ] Registry delta measured after a full reload (N-132), not on an accumulated session; recorded.
- [ ] No faked rows / no faked `secondary`/`meta` (N-091 / D-065); honest empty states verified.
- [ ] Every `.md` touched carries the correct header (each `>` line ends in two spaces; date = the close session's).
- [ ] D4 resolved as Joe locked it; the delivered highlight behaviour matches.

*(Deliberately NOT a DoD item: "commit pushed" — unflippable inside the commit that performs the push. `Status: COMPLETED` in this header is the real signal. Joe pushes.)*

---

## 7. Same-commit canonical records (D-074)

The close travels as ONE commit set: this task doc `Status → COMPLETED` (+ a Rule-6 delivered-vs-runbook delta) · a `JOURNAL.md` entry (J-NNN) · the `CLAUDE.md` PLAY block · `docs/ROADMAP.md` (M-RP6.2 🟢→✅, R1/R2 no longer placeholders). Code commit stays code-only if Clair verifies before committing (the J-501 good instinct); records follow atomically.

---

## 8. Proposed locks for Joe (D1–D8)

- **D1 — one Rust command, rooms embedded.** `get_spaces` returns the full tree; R2 reads rooms from it; no `get_rooms` UI command. *(Recommended.)*
- **D2 — one milestone, two legs, visible-first.** Leg 0 Rust → Leg A R1 → Leg B R2. *(Recommended — the cross-region proof needs both.)*
- **D3 — R2 latches its space scope** (§3.2). The one new mechanic. *(Recommended.)*
- **D4 — R1 highlight when a room is selected** (§3.3): **opt-1 bus-pure** (recommended) vs opt-2 R1-also-latches. **Needs Joe.**
- **D5 — `selected` is bus-derived** on both widgets (the self-panel standing rule). *(Recommended.)*
- **D6 — `secondary`/`meta` ship unfed** (no faked last-message / unread). *(Recommended — D-065.)*
- **D7 — the protocol→descriptor projection lives in the WIDGET** (the self-panel precedent), the raw TS `KnownSpace` type in the `$common` store, never in `core` (§4, W-3). *(Recommended.)*
- **D8 — `spaces-panel`/`rooms-panel` are `PluginDescriptor` entries in `CLIENT_PLUGINS`** (the v1.1 correction). Two sub-locks that **need Joe** (Rule 8 — the `name` is user-visible in two places):
  - **`name`** = the plugin-list label + sort key **AND** the tile-stripe title (`buildTitles`). Chat proposes **`"Spaces"` / `"Rooms"`** (drops the `R1 ·`/`R2 ·` dev-scaffold prefix that `REGION_NAMES` currently paints; matches `"Self Panel"`/`"Inspector Panel"` plainness). Alternative: keep `"R1 · Spaces"`/`"R2 · Rooms"`. **Joe picks.**
  - **`icon`** (leading glyph, host-tinted in the plugin list) — provisional → M-RP-SKIN, but must be a **shipped `icons.ts` key** at build (self→`person`, inspector→`search`). Chat will pick a real glyph at author time and flag it; Joe may override.
  - Accepted consequence: **+2 `[system]` rows** in Settings → Plugins (V8). Not a new surface — a region plugin is *listed* by construction (W-13, the self/inspector precedent).

**Nothing is coded until Joe locks D1–D8 (D4 and D8's `name` especially).**

**✅ LOCKED 2026-07-17 (Joe: "go by your recommendation").** D1–D8 all locked as Chat recommended: **D4 = opt-1 (bus-pure)** — R1 highlights only while the bus holds that space, no second latch; **D8 `name` = `"Spaces"` / `"Rooms"`** (drops the `R1 ·`/`R2 ·` scaffold prefix). D8 `icon` = **left UNSET** on both descriptors → the documented `plugin-list` fallback `p.icon ?? 'square'` (`plugin-list.svelte:65`) paints the neutral placeholder. There is **no verified Material source SVG for a spaces/rooms glyph in-repo**, and Chat will not fabricate a `d` path from memory (Rule 5 + the byte-for-byte icon discipline, D-108) — real glyphs are filed to **M-RP-ICON-ADOPT / M-RP-SKIN**. Honest-provisional, not misleading. Status → ACTIVE; build proceeds.

---

## 9. CLOSE — ✅ COMPLETED 2026-07-17 (Chat: built + CDP-verified live, single seat)

**Shipped (code):** Leg 0 `get_spaces` command + 2 serialize tests (`desktop.rs`) · Leg A `spaces-state.svelte.ts` store + `spaces-panel.svelte` widget + `CLIENT_PLUGINS` descriptor + one `app_client` hydrate line · Leg B `rooms-panel.svelte` widget (the D3 latch) + `CLIENT_PLUGINS` descriptor. Frontend + the one thin Rust command; **no `ops.rs`, no `core` component, no schema change, no sampler**.

**Static gates:** `vite build` **183 modules** (up from 178 — the new widgets pull `entity-panel` + deps into the client bundle for the first time) · `npm test` **77** (unchanged — no vitest logic) · **`cargo test` 1519/0/62** (moved from 1517/0/62 by **+2** — the honest Rust-landed signal; both `get_spaces_*` tests `ok`).

**CDP verify — live client 9222, full reload first (N-132), Chat re-drove every leg (Rule 5):**

- **V0 ✅** cargo 1519/0/62 (+2).
- **V1 ✅** `__XGEN_SPACES__ = []` — the dev client is unregistered (the J-500 situation), so `get_spaces` honestly returns empty. Interactive legs V2–V7 driven on an **injected 2-space tree** via `__XGEN_SPACES__.setSpaces(...)` — the store's DEV setter, the same boundary the shell hydrate feeds; the widget→bus→cross-region flow is fully real, only `get_spaces` is substituted.
- **V2 ✅** R1 renders one row per space (`entity-panel` `count:2`); empty payload → the **"No spaces yet"** paragraph (proven both ways).
- **V3 ✅** trusted click on a space row → bus `{regionId:'spaces', entity:{kind:'space', id, name}}`; the row **paints** `[data-selected]` (`box-shadow: rgb(154,106,48) 2px 0 0 inset` — the pixel, N-097), the other row `none`.
- **V4 ✅** (the cross-region proof) selecting a space → **R2 repopulates** with that space's rooms (`latchedSpaceId` set, rows rendered) **and R8 shows the space** (`kind:'space'`). Real R1 click, not `__XGEN_SEL__`.
- **V5 ✅** (D3 latch) click a room in R2 → bus flips to `{kind:'room'}`, **R8 follows to the room**, **R2 KEEPS its list** (`latchedSpaceId` held, room row painted), **R1 un-highlights** (D4 opt-1, bus-pure — `boxShadow:none`). All four in one read.
- **V6 ✅** two distinct empty states: no scope → **"Select a space"**; a scoped space with zero rooms (Design) → **"No rooms"**.
- **V7 ✅** bus purity — `__XGEN_SEL__` driven with a space id **no R1 row exists for** → R8 renders it (painted `Kind space · Name Phantom Space · ID …PHANTOM · Source phantom-writer`); R1 highlights nothing; R2's **stale-latch guard** falls back to "Select a space" (no throw, N-095 spirit).
- **V8 ✅** Settings → Plugins shows the two new rows (`Spaces`, `Rooms`), `:modal true`.

**Registry:** quiescent **count === unique === 119**, `droppedCount:0`, connection-stats not installed. **M-RP6.2 adds exactly +8 quiescent entries** — the two `entity-panel` subtrees (each region: widget-root + entity-panel + section + empty paragraph); a still-placeholder region registers **nothing but its tile**, so the two placeholder→widget swaps add 4 each, nothing leaked. `count===unique` = no leaks. (The J-540 "99" was a different machine/store context, N-108 — the **+8 delta** is what's verified, not the absolute.)

**N-097 skin check ✅** — no `.spaces`/`.rooms` rule in `skin.css`, nothing stranded.

**Rule-6 deltas from this runbook (Chat's own predictions corrected against the live tree — N-105):**
1. **V8 count is 4→6, NOT "3→5".** The pre-M-RP6.2 base was **4** system plugins (self, inspector, plugin-list, **grid-plate**) — grid-plate was added at M-RP-PLATE (J-532) since this runbook re-quoted the stale J-513 "3". The **+2** for Spaces/Rooms is correct; the base was stale.
2. **The `[system]` distinction is the version line + host-tinted icon, not a literal `[system]` text badge** — M-RP-SETTINGS Leg B (J-537) replaced the badge with the version (`v1.0.0`). `systemCount:6` + the icon colour + the non-removable action row carry the system-ness. Rows render `"Spaces v1.0.0"` / `"Rooms v1.0.0"`.
3. **Baseline is 119 on this machine, not 99** — N-108: the count depends on the machine's store/build context; the verified quantity is the **+8 delta**, computed on this same load (placeholder region = 1 tile entry; widget region = 5).

Nothing faked: `items = { descriptor }` only, `secondary`/`meta` unfed (D6/D-065); honest empty states verified both ways.
