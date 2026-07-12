# XGen Client — The Dock Engine (Renderer B): Phase-0
> **Status**: ACTIVE  
> Version: 1.2  
> Date: Jul 2026  
> **Last updated**: 2026-07-12  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

The working widget grid. Renderer A (`region-shell`, M-RP6.1f) renders a **const** tree into flex boxes and **nothing mutates it** — it looks like a dock and is a picture of one. This Phase-0 turns it into one: a tile frame with a title stripe, fold-to-one-line, splitter resize, and drag-to-rearrange.

Extends `ui/docs/xgen-region-dock-model.md` (D-103) — **the descriptor is unchanged except for one field**; both renderers still read one contract. Renderer B is a **renderer upgrade, not a region rewrite** (region-dock §3), and this document holds that line.

---

## 0. The autonomy grant (Joe, 2026-07-12) — read first

> **Chat holds full authority over every aspect of this arc** — descriptor changes, mutation algebra, gesture semantics, persistence, verification, milestone split — **except the graphical appearance of the created elements, which is Joe's.** Behaviours are revisited jointly after the arc completes; additional components may fall out of that review.

**The grant is over mechanics, not over the record.** Findings, deviations, and gaps are surfaced as always (D-065). **Every appearance decision is brought to Joe as options before it is skinned** — and per N-090 appearance is wide: gaps, spacing, sizing, tracks and layout are skin-tunable, so *stripe height, grip size, seam thickness, band thickness, and the folded form* are all Joe's, not Chat's.

---

## 1. What Joe locked in the walk (2026-07-12)

Verbatim intent, in his words, with the mechanical consequence beside it:

| Joe's words | consequence |
|---|---|
| *"regions have main form now as a group container. group containers will be contained, but not as a main form"* | **The tile owns the chrome, not the widget.** All eight regions currently root in `<Section title=…>`; the title moves to the tile stripe. `section` survives **inside** a region body. |
| *"the region can be folded (collapsed) to just one tail on height"* | **`collapsed` enters the descriptor.** The first schema change since D-103 → `version` bump + migrate (§9's path, **never exercised**). |
| *"title stripe on the top with name and control buttons… [fold/unfold] on the complete right"* | The stripe is **renderer-drawn**, one component, identical for every region. |
| *"on the complete left will be rectangle/square grip, before title. only with this grip the region can be moved"* | **Move is grip-only.** No drag thresholds, no text-selection ambiguity. **And it partitions the drag space** — see §7. |
| *"on the lower right corner… triangle grip for resizing. only with this the region can be resized"* | Resize is corner-grip-only. The `.region-split` **1px seam already ships** — a splitter handle needs no new geometry. |
| *"to drag and drop one region on another is just for rearranging purpose. never mixing or joining"* | **🔑 THE LOCK THAT SHRANK THE ARC.** See §3. |

---

## 2. 🔑 The reframe: a target tile is an ADDRESS, not a container

A region is **never put inside** another region. In a space-filling tree there is no empty space to drop *into* — every pixel already belongs to a tile. The tree's only vocabulary is **relative**: *above that one · below that one · left of that one · right of that one.*

> **The drag verb is not "put A into B". It is "insert A here, and *here* is expressed as an edge of B."**

The target is a **lookup key**, not a parent. It receives nothing. This is why drag-to-dock exists at all: **fold changes one tile's height; the splitter changes two tiles' proportions; neither can say *"I want the composer at the top"*.** That sentence needs a destination, and in a tree the only destination that exists is *next to something*.

---

## 3. 🔒 NO TABS — and this is a position, not a deferral

Joe: *"never mixing or joining."*

- **There is no centre drop-zone.** Every drop-zone is an **edge band**. The centre of a tile is **inert**, not a target.
- Therefore **nothing in this arc can produce a `tabs` node**, and no tab strip is built.
- **Fold is already the stacking mechanism, and it is the better one:** four folded stripes in a column is a tab strip lying on its side — except *several can be open at once* and *every label is always visible*. **Tabs is what you reach for when you don't have fold. We have fold.**

**Consequences, all of them subtractive:**
- **No docked/undocked mode.** A mode was only ever needed to discriminate split-vs-tab. No tab → no discriminator → no mode. *(And a mode was never a field anyway: **a region is docked iff it appears in the layout tree** — a query, not a flag. Two sources of truth for "is this docked" is a D-067 drift surface.)*
- **No `M-RP-ROVING` prerequisite.** A tab strip would have been the **5th** independent roving-tabindex implementation (D-069's bar met at four). It is not built, so `M-RP-ROVING` stays filed and stays out of this arc.
- **The door stays shut, not locked.** `types.ts` keeps typing `tabs` and `resolve.ts` keeps dropping it with a DEV warn. **Zero cost, zero schema change if it is ever wanted.**

> **⚠️ If a future milestone reaches for tabs, it re-opens §3 explicitly — it does not arrive as a rider on a drag milestone.**

---

## 4. Descriptor change — exactly one field

```
LayoutNode =
  | { type: "leaf",  widgetId: string, collapsed?: boolean }   // ← + collapsed
  | { type: "split", dir: "row" | "col", sizes: number[], children: LayoutNode[] }
  | { type: "tabs",  active: number, children: LayoutNode[] }  // still typed, still dropped
Layout = { version: number, root: LayoutNode }
```

**`collapsed?: boolean` on a leaf. That is the whole schema delta.**

- Optional → **an absent key means expanded**, so **every layout already on disk stays valid** and a migrate is a no-op today. The `version` bump is still taken, because §9's migrate path has **never been exercised** and this is its first honest customer.
- **`collapsed` is a property of the TILE (the `leaf` node), not of the widget.** The widget does not know it is folded; it is simply not rendered. *(A widget that could observe its own fold state would be a widget that could fight it.)*

### 4.1 🔒 A TILE COLLAPSES ALONG ITS PARENT SPLIT'S AXIS — SETTLED (Joe, 2026-07-12)

**The question that forced it:** fold-to-one-line is a **height** concept, but the tree has two directions. `spaces` sits in a **`row`** split, whose children are **full-height by definition** — a row split divides *width*. **A stripe-tall box in a row split would leave empty space below it**, and §7.1 locks *no holes, rectangles only*. **Folding a row-parented tile to stripe HEIGHT is geometrically unsayable.**

**And Joe's own constraint is what settles it:** *"we have to not forget that there is a possibility that a custom widget has to be — or better, to be — folded."* **If fold worked only in `col` splits, a widget's foldability would depend on where it happens to sit** — drag it left and it loses its fold button. **That is not a widget property; that is an accident of placement.**

> **🔒 THE RULE: a tile collapses along its PARENT SPLIT'S AXIS.**
> - parent `col` (divides height) → collapse **height** → **horizontal stripe**.
> - parent `row` (divides width) → collapse **width** → **vertical strip, rotated title**.

**One rule. Fold is available EVERYWHERE, ALWAYS — including any custom widget, wherever it lands.** No hole is ever created: the tile still fills its parent's **cross-axis**.

**🔑 And `collapsed` stays ONE BOOLEAN.** The **direction is DERIVED from the parent's `dir`, never stored.** *(A stored direction would be a second source of truth that goes stale the moment the tile is dragged — D-067 in miniature.)* **Free consequence: drag a folded tile from a column into a row and it RE-ORIENTS ITSELF.** That is a verify leg at M-RP7.4, and it is the kind of property that either works by construction or is a nightmare — **this one works by construction.**

**Cost:** a second *rendering* of the stripe (rotated). It is the **same component** with `data-axis`, and under **N-090 the rotation is SKIN, not code** — so it costs Joe a look to approve and costs the architecture nothing. **What the rotated strip CONTAINS is §4.3.**

**And the raster question resolved with it (Joe):** *"one line — I meant a line of grid, the smallest dimension in the raster. If we will not have such a constraint, it folds to the height of the region's title bar."* **We have no pixel raster** (§7 — the raster is a *quantum on the weights*, not a lattice of rows). **→ folded = STRIPE SIZE.** Which is exactly §4.2: **fold IS the minimum.**

### 4.2 Fold supplies the minimum tile size — for free

A tile can never be smaller **along its parent's axis** than its own stripe, **because that is what folded means.** So **no `minSize` field enters the descriptor.** One concept, two jobs.

> **🔒 A splitter drag that would push a tile below stripe height STOPS. It does not auto-fold.** Auto-fold makes a *resize* secretly a *state change*, and the user cannot tell what they did. **Fold is a button.** The two verbs stay separate.

### 4.3 🔒 The folded side strip = the TITLE BAR, ROTATED (Joe, 2026-07-12)

Joe: *"it has to contain something for the user's orientation and overview — everything that is in the title bar, rotated 90°."*

**Same content, same component, same DOM order: grip · title · fold.** Nothing dropped, nothing substituted. *(An icon-instead-of-name was considered and **rejected**: a plugin has a `name`, it does **not** have a glyph — that is `M-RP-ICON-ADOPT` work, and it would make the **folded** form say **less** than the unfolded one.)*

### 4.3.1 ⚠️ THE ROTATION DIRECTION IS CULTURAL — A TOKEN, NOT A DECISION

**Book spines split the world.** English-language spines read **top-to-bottom** (a **CW** rotation — tilt your head *right*). German / French / Czech / Slovak / Russian spines read **bottom-to-top** (**CCW** — head *left*). **Hardcoding either would be quietly wrong for half the users.**

> **🔒 DEFAULT: CW (the English convention). Joe-locked.** *It is the default because something has to be, **not because it is correct** — do not later read it as a design position and defend it.*

**It is ONE `skin.css` property.** `writing-mode` rotates the **flex axis** along with the text, so the strip's internal order stays coherent in both directions with **zero component branches**:

| | text reads | grip | chevron |
|---|---|---|---|
| **CW** (`vertical-rl`) — **default** | top-to-bottom | **top** | **bottom** |
| **CCW** (`vertical-rl` + `rotate(180deg)`) | bottom-to-top | **bottom** | **top** |

**The component writes the DOM order ONCE. The skin picks the direction.** *(N-090's payoff again: an appearance question that would have been a component fork is one CSS property. **And the verify leg PROVES it is a token — inject the other direction, re-measure, the order inverts with zero component change.** If that leg fails, we built a fork and called it a token.)*

**⏸️ Making it a USER SETTING is FILED, NOT BUILT.** Joe wants it in custom settings; **it cannot be one yet, and inventing a home would repeat a defect already named.** **There is no settings mechanism** (J-513 filed the Ch6-`settings_schema`-vs-plugin-component collision as **deliberately undecided**, binding: *nothing is built toward either until the grid works*), and **`theme-*.css` does not exist** (D-110). By the **J-503 test** (*would you expect it to follow you to another device?*) reading direction is a **preference — a sibling of `theme`** — and **`theme` is a §4.5 key M-RP6.1k deliberately did NOT ship** (**RESERVE NOTHING**). **→ it lands WITH the milestone that creates the `theme` key. This arc reserves NO key, NO prop, NO control for it.**

**🔑 The project's FIRST localisation-shaped decision — and the codebase already had the right instinct:** `Accelerator` takes **`platform` as a PARAMETER, default `'win'`, and never sniffs `navigator`** (M-RP6.1c). **Same shape: a token, never sniffed. No locale system is built and none is needed.**

### 4.3.2 🔒 A folded tile shows NO resize triangle

**A folded tile has no resizable dimension of its own.** Its collapsed axis is pinned at stripe size **by definition** (§4.2), and its cross-axis belongs to its **parent**, not to it.

**→ the corner grip is ELEMENT-ABSENT when `collapsed`, not greyed.** *(J-500: the absent slot ships **ABSENT, not faked**. J-513: a control is disabled only for a reason **true of that thing** — "you cannot resize this" is not a **state** of the triangle, it is the **absence of the verb**.)*

---

## 5. 🔑 The chrome collision — GROUNDED, and it is all eight regions

**Not a hypothesis. Grepped:**

- `region-placeholder.svelte` → `<Section {title} id={`region-${regionId}`}>` — six regions.
- `self-panel.svelte:70` → `<Section title="Self" id={cid('section')}>`
- `inspector-panel.svelte:68` → `<Section title="Selection" id={cid('section')}>`

**Every region draws its own title header today.** Add a tile stripe and every region has **two titles**. And `section` ships `collapsible` + `collapsed` ($bindable, the 27th `core`) — so a collapsible section inside a foldable tile would stack **two fold affordances**.

> **This is precisely Joe's *"group container is the wrong main form"*, arriving as a concrete defect in eight files.** It **cannot be deferred — this arc creates it.**

**Resolution (mechanics = Chat's; the resulting look = Joe's):** a region widget's root **is no longer a titled `Section`**. The tile frame draws grip · title · fold; the widget renders its **body only**. `section` remains fully legal **inside** a region body — *contained, not the main form*, exactly as Joe framed it.

**⚠️ Two consequences that must not be discovered at verify:**
1. **The registry entries `section#region-*` DISAPPEAR** (8 of them) and are replaced by the tile frame's own registered element. **The client registry baseline WILL move.** It is **measured, never derived** (N-105/N-108) — and it must be read **quiescent, on a stated store state**.
2. **The leaf-id convention `id = region-${regionId}` (N-096) migrates from the `Section` to the tile frame.** It stays the leaf's durable handle; it changes owner.

**The title's source already exists:** `CLIENT_PLUGINS` carries `name` (M-RP6.1l). **No new data, no new verb** — the same one-source-two-readers shape as the `widgetRegistry` derive.

---

## 6. 🔒 The mutation algebra — ONE verb

Because there is no join and no tabs, the whole algebra reduces to a single entry point:

```
move(layout, sourceLeafId, targetLeafId, edge: 'top'|'bottom'|'left'|'right') -> Layout
fold(layout, leafId, collapsed: boolean) -> Layout
resize(layout, splitPath, index, delta) -> Layout
```

`move` decomposes into four internal steps, all pure:

1. **remove** the source leaf from its parent split;
2. **collapse the degenerate split** it leaves behind (a split with one child *becomes* that child — this is what keeps the tree from growing a spine of one-child splits after a few drags);
3. **insert** at the target: if the target's parent split already runs along the drop axis, insert as a **sibling**; otherwise **wrap the target in a new split** of the correct `dir` and put the source in the right half;
4. **re-normalise `sizes[]`** so the survivors keep their relative proportions (the same rule `resolve.ts` already applies when a sibling drops).

**It is a NEW pure module beside `resolve.ts`, not an extension of it.** `resolve.ts` is a **read** walk (`descriptor → render tree`, lossy — it *drops*). This is a **write** (`Layout × op → Layout`). It cannot reuse the walk; it sits next to it and must emit trees the walk can still resolve. **DOM-free, no Svelte, no I/O, vitest** — the `resolve.test.ts` / `grouping.ts` / `Accelerator` precedent.

### 6.1 §9's reconcile rule survives, and one part of it becomes reachable

**`re-inject missing system widgets` (W-13) is UNIMPLEMENTED today** — `resolve.ts` drops unknown ids and never re-injects. Harmless while nothing can remove a leaf. **This arc does not make it reachable either** (no verb removes a region from the tree — `move` relocates, `fold` collapses in place). **It stays filed, honestly, as still-unimplemented.** *A rule the code does not implement is not a rule; it is a comment.*

---

## 7. The raster — a quantum on the weights, not a lattice of cells

The dilemma Joe posed (*absolute grid → regions can fall off-display · relative grid → regions would scale*) dissolves once the grid is an **input model**, not a **storage model**.

- **Nothing can fall off-display**, because a `split` tree is **space-filling by construction**. There is no coordinate that can be out of range.
- **Regions never scale.** Weights scale; content does not. `[1,2,7,2]` has held **exactly at four distinct measured window widths** (J-499 · J-501 · J-511 · J-513).

**So:**
- **Storage: integer weights.** Never floats (a splitter drag emits `0.3333…`; persist → reload → re-normalise → drag → **the layout rots by rounding**). Never px (the window resizes; geometry restores in *physical* px at DPR 1.25 — N-092b, measured twice). **Weights are the only unit that survives the trip.** *Integer weights also make the verification exact: a tree diff reads `[1,2,7,2] → [1,3,6,2]`, not an epsilon comparison — and proving a drag is the expensive part of this arc.*
- **Interaction: snap the pointer to a quantum during a drag, convert to weights on release.** The user feels a grid; the file stays unit-free. **This is skin-tunable and never enters the descriptor.**
- **Guide lines painted during a drag are a drawn aid, not a data structure.**

**Quantum resolution and the drawn guides are appearance → Joe's.** Chat's proposal: a fixed per-split denominator, coarse enough to read in the JSON, fine enough to feel continuous.

### 7.1 What we are NOT building, and it is a real choice
A fixed **M×N cell lattice** (Grafana / react-grid-layout) is right when tiles are **independent cards that may leave holes**. Ours is a **space-filling tree**: **no holes, rectangles only, everything always fills** — and *insert-a-split-here* is only well-defined **because** it is a tree. Going lattice means **retiring D-103's descriptor**, not extending it. **Joe accepted "no holes, rectangles only" explicitly (2026-07-12).**

---

## 8. 🔑 The grip-only rule partitions the drag space — keep it for this reason alone

A region drags **only** from its grip square. That leaves **the entire body of every tile free** for a *second, independent* drag system — dragging **content** (an entity: avatar / room / space) rather than a **region**.

**They can never collide.** Joe arrived at grip-only for ergonomics; it buys a structural guarantee for free.

> **⏸️ `M-RP-ENTITY-DRAG` — dragging content, not regions (FILED 2026-07-12, NOT in this arc).** Zero code, zero spec today (grepped: the only `pointerdown`/`draggable`/`ResizeObserver` in `ui/` are colour-picker, combobox, tag-select, and the status-bar grip). **Its hard part is not the gesture — a drop is a PROTOCOL VERB** (dragging a person into a room is an *invite*). It inherits the M-RP6.6 shape: **a UI milestone cannot manufacture a verb that does not exist.** Joe (2026-07-12): *"this will not be regions in that context. just entities."* — so its **drop targets are content surfaces, never tiles**, and it never touches the descriptor.

---

## 9. Empty regions, full mechanics — the fixture is already right

Joe: *"can we in this phase create just empty regions without context with full mechanics?"* **Yes — and it is the stronger build, not the cheaper one.**

**The grid must not be able to depend on what is inside a tile.** If the first thing we drag is a placeholder, that independence is **proven, not assumed**. Build the dock against a live message stream and we will never know whether the algebra quietly leaned on it.

**⚠️ But do NOT downgrade R3/R8 to placeholders.** `self-panel` and `inspector-panel` are **real widgets carrying a live selection bus between them** — the arc's only proof that the mechanics work on something that is not scaffolding (*an unfed branch is an unverified branch*). **Fold R3, drag R8, and the bus must still work — that is a required verify leg.**

**Fixture: six placeholders + two real widgets. We already have it.**

---

## 10. Verification — the real cost, stated up front

**You cannot prove a drag with a getter.**

- **Trusted input is mandatory.** A synthetic event from `eval` is **untrusted** and does not fire native defaults (**J-496**, proven). Drags need CDP **`Input.dispatchMouseEvent`** — down · move(s) · up.
- **Before/after tree diffs are the proof.** `__XGEN_LAYOUT__.current` **already exposes the live tree** — diffing is free. A drag that mutates the store and moves nothing on screen is not a working drag; it is an untested one (**N-097**). **Assert on the PAINTED geometry, not only the descriptor.**
- **N-099/N-110 apply with force here:** split state-change and DOM-read across **two** evals, and **assert the subject is READABLE first** — the registry and the DOM key on **`data-debug-id`, NOT `id`**. A `#some-id` selector returns `[]`, *an empty array that looks exactly like a measurement*.
- **Baselines are measured, quiescent, on a stated store state (N-105/N-108).** The client registry **will move** at M-RP7.1 (the eight `section#region-*` entries leave). **The new number is measured, never derived.**
- `cargo test` **must stay 1517/0/62 IDENTICAL** through M-RP7.1–7.4 — *the inverse leg: identical PROVES no Rust landed.* It is **expected to move at M-RP7.5** (the session feeder touches nothing in Rust either — **so if it moves, something is wrong**; see the runbook).

---

## 11. Arc — five legs, visible first

**⚠️ This SUPERSEDES region-dock §7's renderer-B numbering.** §7's `7.3 save/restore layouts` **already shipped** (M-RP6.1k, D-114). §7's `7.4 custom-widget regions` and `7.5 tear-off` move out of the arc.

| # | milestone | scope |
|---|---|---|
| 1 | **M-RP7.1 — the tile frame: stripe, grip, fold** | Chrome moves widget → renderer. `collapsed` enters the descriptor (**first schema bump since D-103**; `version` + migrate). The eight `Section` roots are unwrapped. **Nothing moves yet.** **This is where Joe sees it and corrects the appearance.** |
| 2 | **M-RP7.2 — splitter resize on the seam** | The 1px `.region-split` gap becomes a drag handle. Weights snap to the quantum. Pure `sizes[]` arithmetic — the cheapest real mechanic, **and it lands the trusted-mouse-event harness the rest of the arc needs.** |
| 3 | **M-RP7.3 — the mutation algebra (pure)** | The new module beside `resolve.ts`: `move` · `fold` · `resize`, remove → collapse-degenerate → insert → re-normalise. **Vitest, no DOM, no gestures.** |
| 4 | **M-RP7.4 — drag to dock: grip, edge bands** | The algebra gets a pointer. Four edge bands per tile, inert centre. **Where the arc's real cost lives.** |
| 5 | **M-RP7.5 — the session layout feeder** | The grid finally **writes `session.layout`** — see §12. |

**M-RP7.6 — node app inherits the frame + grid** (the long-filed *"M-RP7.x node frame inheritance"*, now numbered per Rule 8) lands **after** the arc, so it inherits a **working** grid rather than building the frame twice.

---

## 12. ⚠️ Persistence is NOT free — a grounded find

**`loadLayout()` reads `store.session.layout`. NOTHING WRITES IT.** (Grounded: the frontend's `persist()` merges only `version` / `named` / `active`; Rust merges `session.geometry`.) The read path, the file, the fallback (**N-095, exercised**) and the merge discipline **all ship** — the key has simply **never had a writer**. Today `loadLayout()` therefore *always* falls to `DEFAULT_LAYOUT`, honestly and by guard.

**The moment the grid becomes mutable, the frontend writes `session.layout` — and then BOTH writers touch the `session` object.**

> **🔒 N-107 one level deeper: the merge must be PER-KEY INSIDE `session`, not per-top-level-key.** A whole-object `session: { layout }` write **eats the `geometry` Rust just saved.** *Any format with more than one writer must be MERGED, never REPLACED* — and `session` is now such a format in its own right.

**Debounced session write on every mutation** (fold · resize · move). Manual **named** states are unaffected — S-8's *permanent, not a stopgap* stands.

---

## 13. Filed, NOT in this arc

**`tabs`** (§3 — a position, not a deferral; re-opening it is an explicit act) · **`M-RP-ROVING`** — extract the roving-tabindex helper (no 5th consumer now) · **`M-RP-ENTITY-DRAG`** — dragging content, not regions (§8) · **`M-RP-ENTITY-PANEL-RESPONSIVE`** — an entity list fills any rectangle (§15) · **tear-off to a real OS window** (behind **M-RP8** — a torn-off region is a `decorations:false` `WebviewWindow` whose **title bar IS the tile stripe**, S-2's *one component, two mounts*; it also needs the descriptor to **record floating regions**, or W-13's re-inject silently re-docks them — **a second schema change, not free**) · **`M-RP-SETTINGS`** · **`M-RP6.1m`** — the plugin row action surface · **`M-RP-FOCUS`** · **`M-RP6.6`** — client resident · the `dialog` footer-snippet slot · N-007's ungraduated obligation · the settings-mechanism collision · the read-marker protocol gap · `temperature-indicator` ⏸️.

---

## 14. Records to change ON LOCK

- `ui/docs/xgen-region-dock-model.md` → **v2.0**: §3 gains `collapsed`; §7's renderer-B roadmap **superseded** by §11 here; §0's *"the skin calls the tile `.region-leaf`"* cosmetic debt **comes due** (renderer B makes tiles draggable — that was the stated trigger).
- `ui/docs/xgen-widget-tier.md` → **W-13's *"may collapse"* finally has a mechanism**; and a region widget's root **is not a titled `Section`**.
- `DECISIONS.md` → **a new D for §3 (no tabs / no join) and §4 (`collapsed` on the leaf)** — a Joe-lock, not taken unilaterally.
- `docs/ROADMAP.md` → the five legs + M-RP7.6.

## 15. ⏸️ FILED — `M-RP-ENTITY-PANEL-RESPONSIVE`: an entity list fills any rectangle

Joe (2026-07-12): *"Spaces — and identities also — is a list of entities: groups and avatars. This set of entities can populate various types of rectangular space: horizontal row, vertical column, or a square."* **And: a region has no shape — it sits on whatever rectangle it is stretched into.**

**✅ He is right, and it is a real gap in no record.** **GROUNDED, not assumed:** `entity-panel` is a **vertical `<ul role="listbox">` and nothing else** — `entity-item variant="row"` is **hardcoded**, roving is **ArrowUp/Down only**, there is **no orientation prop, no wrap, no grid**, and there are **ZERO CSS container queries in the entire project**.

**🔑 BUT IT IS THE CONTENT TIER, NOT THE DOCK TIER — and that boundary is load-bearing.** The dock hands a tile **a rectangle**. What a widget does with that rectangle is **the widget's business**. **The dock must never learn that `spaces` prefers to be tall** — the moment it does, the layout engine starts holding opinions about content and D-103's premise inverts (*"no component inside a region is aware of which renderer is active"*).

**Mechanism: CSS container queries → which under N-090 is SKIN, not component code.** Narrow-tall → column · short-wide → row · squarish → wrapped avatar grid. **The widget writes zero layout code and Joe retunes the breakpoints without a component edit.** *(WebView2 supports `@container` natively — the same engine bet as `d:`, D-109. **Verify, do not assume.**)*

**⚠️ AND "IT IS ALL SKIN" IS ONLY HALF TRUE — the real cost is the ROVE, and roving is component code.** Column → Arrow Up/Down · row → Arrow Left/Right · **wrapped grid → 2-D roving**. **A 2-D rove is NOT a fifth copy of the linear machine — it is a DIFFERENT machine**, and the project has none (`entity-panel` / `menu-bar` / `menu` / `shelf` are **all linear**). *It also makes `M-RP-ROVING` interesting again — not as a copy-extraction, but because the extraction target just grew a second shape.*

**Open (appearance, Joe's):** in a grid form, does a row still show its **name**, or is it **avatar-only**? (`entity-item variant="row"` is hardcoded; a grid almost certainly wants a different variant.)

> **🔒 THIS ARC RESERVES NOTHING FOR IT** — no orientation prop, no aspect hook, no placeholder. *A key nothing writes is a key nobody has round-tripped* (the M-RP6.1k finding, applied). **Trigger: AFTER the grid works** — the dock arc is precisely what makes an arbitrary tile aspect *reachable*, so it is the thing that creates this milestone's source.

---

*End of dock-engine Phase-0.*
