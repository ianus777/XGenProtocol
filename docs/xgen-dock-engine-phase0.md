# XGen Client — The Dock Engine (Renderer B): Phase-0
> **Status**: ACTIVE  
> Version: 1.6  
> Date: Jul 2026  
> **Last updated**: 2026-07-14  
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

### 4.1 🔒 THE FOLD AXIS IS THE USER'S CHOICE — TWO BUTTONS (Joe, 2026-07-13). **SUPERSEDES the derived-axis rule below.**

**⚠️ v1.2's §4.1 derived the fold axis from the parent split. It shipped at M-RP7.1, Joe looked at it, and replaced it.** The original is kept below as §4.1-H — **and its central claim, *"No hole is ever created"*, is FALSE. See §4.5.**

> ### 🔒 **THE RULE: an unfolded tile shows TWO fold buttons. The user picks the axis.**
> - **`[<]` — fold to the LEFT.** Collapses **width** → a **vertical strip** at the tile's leading edge.
> - **`[v]` — fold to the TOP.** Collapses **height** → a **horizontal stripe** at the tile's top.
> - **When folded, the button that was NOT used is DISABLED; the one that WAS used unfolds the tile.**
>
> **Two axes — NOT four directions.** The strip/stripe parks at the tile's **leading edge** (top for `[v]`, left for `[<]`); the freed space opens on the far side. Four directions would be four buttons and buy nothing.

**Why the derived axis was the wrong call, in Joe's own terms.** v1.2's stated goal was *"foldability must not be an accident of placement"* — **and it only half-solved it.** It made fold *available* everywhere and left the **direction** an accident of the tree. **A user who folds a tile, drags it, and finds it has silently changed shape has not been served by an elegant invariant.** *The re-orient-on-drag property was elegant for the TREE. It was surprising for the PERSON.* Joe's rule is boring and predictable: **I chose left; it stays left.**

**🔑 The descriptor consequence — `collapsed` stops being a flag and becomes a DIRECTION.** The axis is now **stored**, because it is **user intent**, and user intent is not derivable from anything. *(v1.2's D-067 worry — "a stored direction goes stale when the tile is dragged" — was **the wrong worry**: it treated the fold axis as a fact about the tree, which can go stale. It is a fact about **what the user asked for**, which cannot.)*

**🔑 But the tile must still know BOTH.** The skin needs the **user's direction** *and* whether that direction runs **ALONG or ACROSS its parent's axis** — because those two cases lay out completely differently:

| | CSS | space | hole? |
|---|---|---|---|
| **ALONG** the parent's axis (e.g. `[v]` in a `col`) | `flex: 0 0 auto` | **siblings absorb the freed space** | **no** |
| **ACROSS** the parent's axis (e.g. `[<]` in a `col`) | keeps `flex: {n} 1 0`, `align-self: start`, cross-size = strip | **nobody can absorb it** — the freed space runs along an axis the siblings do not divide | **YES** — §4.5 |

**→ the tile reflects the direction (which drives the rotation) AND the along/across mode (which drives absorb-vs-hole).** Both are **derived-at-render from the descriptor + the parent's `dir`** — **only the user's DIRECTION is stored.** *(The mode is a fact about the tree, so it stays derived; the direction is a fact about the user, so it is stored. **The v1.2 rule stored nothing and derived everything; the error was not the technique, it was applying it to the wrong field.**)*

**⚠️ MIGRATION — `v2 → v3`, AND IT IS THE FIRST REAL ONE.** `version` has now been bumped twice and **migrated nothing both times** (§9's migrate path has **never executed**). This bump writes the real thing: **for each `collapsed: true` leaf, read its parent's `dir` and write the equivalent direction** — the old derived rule, made explicit. **~10 lines.**

> **⚠️ AND IT IS NOT DEAD CODE, THOUGH IT LOOKS LIKE IT.** Measured on Joe's disk 2026-07-13: `xgen-client_uistate.json` has **`named: {}`** — **no layout exists anywhere.** So the migrate has **nothing to migrate today — by LUCK, not by design.** `app_client.svelte:227` (`uiStateStore.save(name, { layout: ... })`) **has been persisting layouts since M-RP6.1k**, so **one click on the diskette lands a `v2` tree with `collapsed` booleans on disk.** *(v1.2's §12 said "nothing writes `session.layout`" — true, and it was **read as "nothing writes a layout"**, which is false. The `named` path was never in view.)*
>
> **→ DoD: the migrate is EXERCISED in vitest against a HAND-BUILT `v2` tree with `collapsed: true` leaves under BOTH a `row` parent and a `col` parent.** **Fed, not asserted** — the N-091 rule applied to the one branch in this codebase that has never once run.

**Appearance (Joe's, §0):** the two glyphs are **one chevron, rotated** — the fold button is already an **empty `<button>`** whose chevron the **skin** paints, so this costs **zero new icons**. ⚠️ **But the glyph is now LOAD-BEARING** — with one button a chevron could indicate *state*; with two it must indicate *which way*.

> ### 🔒 **CONVENTION A — DIRECTION OF TRAVEL. LOCKED (Joe, 2026-07-13, after seeing it run.)**
> **THE CHEVRON ALWAYS POINTS WHERE THE REGION WILL GO IF YOU PRESS IT** — enabled or disabled, folded or not. **One rule, no exceptions.**
>
> | state | `[width]` | `[height]` |
> |---|---|---|
> | **unfolded** | `<` (135°) — *I will go LEFT* | `^` (225°) — *I will go UP* |
> | **folded-height** | `<` (135°), **disabled** — still names what it WOULD do | `v` (45°) — *I will come back DOWN* |
> | **folded-width** | `>` (−45°) — *I will come back RIGHT* | `^` (225°), **disabled** |
>
> *(Base `::before` = a 6×6 box with right+bottom borders. 45° = `v` · 135° = `<` · 225° = `^` · −45° = `>`.)*

**⚠️ v1.3 RECORDED "convention B (disclosure)" AND THAT LABEL WAS NEVER TRUE OF THE BUILD.** Joe's original sketch was `[<][V]`, which **mixes the two conventions in one stripe** — and the code shipped exactly that mix. **The WIDTH button was ALWAYS convention A** (open `<` = *travel*; B would have pointed `>`, at the content). **Only the HEIGHT button spoke disclosure** (open `v` = *"my body is below"*). **Two languages, one stripe, and the skin comment sitting directly above the rotations already described convention A correctly.** *The label was wrong, the prose was right, and one rotation value contradicted both.*

> **→ THE FIX WAS TWO ROTATION VALUES SWAPPED. THE WIDTH BUTTON WAS NEVER WRONG.** *(And this is what M-RP7.1's arc slot was FOR — "this is where Joe sees it and corrects the appearance." It caught a mixed metaphor that no amount of re-reading the chapter would have surfaced, because **the chapter was self-consistent and the CODE was not**.)*

**🔒 The two buttons sit in `.region-title-buttons` — ONE addressable unit** (Joe, 2026-07-13), `display: inline-flex`, `gap: var(--region-fold-gap)`, **`--region-fold-gap: 0`** in the tile's tunable-token block. **Zero because the chevron glyphs are SQUARE (16×16) and already carry their own optical spacing; a narrower glyph would need a real value.**

> **⚠️ `gap: 0` IS A KNOB, NOT A RESERVATION — and only because of the `inline-flex`.** On the shipped `display: inline` default a `gap` would have been **INERT**, and **an inert knob IS a reservation** (§4.3.1 — *RESERVE NOTHING*). **The test: can Joe change the value today and see the render move? Yes.** → it is a live control with a declared default. *(It also fixes the stripe's own gap landing between the TITLE and the span rather than between the buttons — the wrapper is now a single box.)*

> **⚠️ THE DOUBLE-ROTATION TRAP — named before it bit, and it did not bite.** A width-folded strip is the title bar under `writing-mode: vertical-rl`, which **rotates the whole stripe, INCLUDING ITS BUTTONS.** **A border-box `::before` is NOT rotated by `writing-mode`** — so the chevron's `transform` is the ONLY rotation on the glyph. **Measured: the angles inside the rotated strip are IDENTICAL to the unrotated case (315° / 225°).** *(§4.3's "same content, same DOM order" survived the one-button design because a lone chevron's orientation carried no meaning. It does now.)*

### 4.1-H — Original §4.1, kept as history. ⚠️ **SUPERSEDED (Joe, 2026-07-13). Its claim *"No hole is ever created"* is FALSE — see §4.5 and N-111.**

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

A tile can never be smaller **along its FOLD axis** than its own stripe, **because that is what folded means.** So **no `minSize` field enters the descriptor.** One concept, two jobs. *(v1.2 read "along its PARENT'S axis" — corrected at v1.3: the fold axis is now the user's, §4.1.)*

> **🔒 A splitter drag that would push a tile below stripe height STOPS. It does not auto-fold.** Auto-fold makes a *resize* secretly a *state change*, and the user cannot tell what they did. **Fold is a button.** The two verbs stay separate.

### 4.3 🔒 The folded side strip = the TITLE BAR, ROTATED (Joe, 2026-07-12)

Joe: *"it has to contain something for the user's orientation and overview — everything that is in the title bar, rotated 90°."*

**Same content, same component, same DOM order: grip · title · `[<]` · `[v]`.** Nothing dropped, nothing substituted — **and with two buttons this rule now does REAL work: both buttons stay PRESENT when folded** (one disabled), so the rotated strip's content is identical to the unfolded stripe's. *(An icon-instead-of-name was considered and **rejected**: a plugin has a `name`, it does **not** have a glyph — that is `M-RP-ICON-ADOPT` work, and it would make the **folded** form say **less** than the unfolded one.)*

> **⚠️ THE DISABLED BUTTON IS CORRECT HERE, AND THE PROJECT'S OWN RULE SAYS SO.** J-513: *a control is disabled only for a reason **true of that thing**.* **"This region is already folded the other way" IS a true state of that button** — unlike §4.3.2's triangle, where the verb simply **does not exist**. **So: the triangle is ABSENT; the unused fold button is DISABLED.** *Same milestone, two different answers, and the difference is not aesthetic — one is a missing verb, the other is a live verb in a state that forbids it.* **It is NOT a countdown** (N-109) — nothing later "discharges" it; it flips back the moment the tile unfolds.

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

### 4.4 🔒 A SPLIT SHRINK-WRAPS WHEN ALL ITS CHILDREN FOLD ACROSS IT (Joe-locked, 2026-07-13)

**The problem §4.1 alone does NOT solve, and it is the one Joe actually saw on screen.**

In the shipped tree the left column is a `col` split with weight **2 of 12** in the outer `row`. **Fold Rooms and Self any way you like and that column is STILL 2/12 wide.** Because:

> **Fold is a LEAF verb. A split's width is a SPLIT property, living in the PARENT's `sizes[]`. A leaf verb cannot reach a split property.**

The only verbs that touch `sizes[]` are **resize** (§6 / M-RP7.2) and **move** (M-RP7.4). **So without this section, §4.1's `[<]` would give the user a thin strip beside a HUGE hole — strictly worse than what ships today.**

**Two ways out were considered. One was rejected on Joe's own lock.**

**❌ Rejected — `collapsed` on a SPLIT node.** Fold the column itself. **Fatal, and grounded:** a leaf gets its title from `CLIENT_PLUGINS.name`; **a split has NO NAME.** So it needs a name field, a UI to set it, and **chrome (stripe + grip + buttons) on splits** — and once splits have stripes, **every split has a stripe, nested, forever.** **That is a group container promoted back to a MAIN FORM** — the exact thing Joe killed at §5: *"group containers will be contained, but **not as a main form**."* ***It would undo his own lock to solve a problem his other idea already solves.***

> ### 🔒 **✅ THE RULE: a split whose children are ALL folded ACROSS the split's own axis SHRINK-WRAPS to its children's folded size.**
> The freed weight **returns to the parent's siblings by the same `flex` they already use.** Unfold **one** child → the split **re-inflates, weights intact.**

**Fold Rooms `[<]` and Self `[<]`: every child of that `col` is now strip-wide → the column shrink-wraps → the `2` goes back to the message stream.** *(And the hole closes with it — two vertical strips stacked in a strip-wide column leave **no leftover space at all**. → **the case Joe actually hit produces NO hole under this rule.**)*

**🔑 What it costs: NOTHING NEW.**
- **No descriptor field.** It is **DERIVED from the children** — a stored *"this column is collapsed"* flag would go stale the instant a child unfolds. **D-067 in miniature, and it would be the second time in one arc.**
- **No name, no chrome, no grip on splits.** **Splits stay invisible. Group containers stay contained.**
- **No new mechanism.** A shrink-wrapped split takes `flex: 0 0 auto` and its siblings absorb — **byte-for-byte the mechanic a folded leaf already uses. One rule, two node types.**

**⚠️ And it does NOT always fire, which is correct.** Fold Rooms `[<]` and Self `[v]` — **mixed** — and the column **cannot** shrink-wrap: it stays wide and a hole opens. **Keep that.** *The user asked for two different things and gets a column that fits neither. The raster (§4.5) explains it. **No magic, no guessing what they meant.***

**Open mechanic (Chat's, per §0):** whether a shrink-wrapped split makes **its** parent shrink-wrap. The rule recurses by its own test. **Settled in the runbook, not re-derived in chat.**

---

### 4.5 🔒 HOLES ARE LEGAL, PAINTED, AND INERT (Joe-locked, 2026-07-13)

**⚠️ HOLES ARE NOT NEW AND THEY ARE NOT A CONSEQUENCE OF §4.1. THEY ARE IN THE SHIPPED BUILD.** §4.1-H claimed *"No hole is ever created"*; **it reasoned about a single tile's CROSS axis and was silent about the MAIN one.** Fold **every** child of a split — as Joe did, with Rooms and Self — and **the split under-fills.** **N-111: a proof about one node is not a proof about the tree.**

> ### 🔒 **§7.1's *"no holes, rectangles only"* IS AMENDED: RECTANGLES ONLY; HOLES ARE LEGAL AND ARE PAINTED AS A SYSTEM AREA.**
> Joe accepted *"no holes"* explicitly on 2026-07-12. **He is retiring it explicitly on 2026-07-13.** *A lock is only retired by the person who set it, in the open.*

**Mechanically the hole is `flex` leftover space INSIDE a split — it is not an element.** → **the raster is a BACKGROUND on the split container**, showing through wherever no tile covers it. **Zero new DOM. One skin rule.** *(Caveat to verify: tiles must be opaque, or the raster bleeds through them.)*

**The appearance is Joe's** (§0 / N-090): a soft fine raster reading *"system area — no functionality here"*. **It ships PROVISIONAL** — plain and obviously placeholder — because **you cannot tune a raster under holes you have not seen.** Retuning it costs **a skin edit and zero component change**.

> ### 🔒 **AND THE LOCK THAT MATTERS FOR M-RP7.4: A HOLE IS INERT. IT IS NOT A DROP TARGET.**
> **D-116: a target tile is an ADDRESS. A hole has NO ADDRESS.** Want a tile there? **Drop on the EDGE of the tile above it.** ***If we let people drop into holes we have quietly built free 2-D placement and retired the tree*** — which means retiring D-103's descriptor, not extending it (§7.1).
>
> **⚠️ D-116 ITSELF IS NOT WEAKENED.** §2 argued *"in a space-filling tree there is no empty space to drop into"* — now only *mostly* true. **But D-116's ground is Joe's constraint (*"never mixing or joining"*), NOT the geometry.** ***Correct the rhetoric; do not touch the decision.***

#### 4.5.1 ⏸️ `M-RP-PLATE` — the grid backdrop: an inert, live-switchable plate widget under the tiles (Joe, 2026-07-14). FILED, NOT BUILT.

Joe: *"the space under regions aka 'the hole' will be customizable, and there will be some static or dynamic visual plate — from solid black to animated reactive colour fractal clouds. What is in the hole right now is a dev plate with no background defined. If we want to put there some real custom background, we can do it by a **background widget** which sets it by its own setting."*

> **⚠️ THIS RETIRES §4.5's *"zero new DOM, one skin rule"* — and that is the finding.** A CSS background can do solid, gradient, pattern, even a keyframe animation. **It cannot do a canvas, a shader, or anything reactive.** The moment the plate is dynamic **it is an ELEMENT**. → today's dot raster is **not the seed of the plate; it is the placeholder the plate REPLACES.**

**🔑 THE MECHANISM IS NOT NEW — IT SHIPS, ONE LEVEL DOWN.** Grounded, not recalled: `message-stream.svelte` already carries **`background?: WidgetMount[]`** — an **array** (so a *stack* of plates is free), `position:absolute; inset:0`, **`pointer-events: none`**, unknown-`widgetId` dropped (W-13), and a **`backgroundLive`** switch passed into every mount: *"a reactive widget renders frozen when false; a static object ignores it."* Locked J-481, shipped J-482 — **chat wallpaper.** ***Joe is describing the same object one level up: grid wallpaper.*** **"Solid black" and "reactive fractal clouds" are the SAME SEAM** — one is a `div` with a colour, the other owns a canvas, and the host never learns the difference.

**🔑 IT COSTS NO SURFACE, AND THE TAXONOMY ALREADY SAYS SO.** W-12 (amended): a widget declares **at most one** of `region · shelf · window · none`. A backdrop plate declares **none of them** — and that is not a gap, it is **§3.2 of `xgen-widget-surfaces-phase0.md`: content rendered inside a HOST is NOT a surface.** The same ruling that settled `temperature-indicator`. → **no tile, no region, no shelf face, no new surface kind, no W-12 conflict.**

**🔑 A HOLE CANNOT HOLD ANYTHING — SO THE PLATE IS A BACKDROP, NOT A HOLE-FILLER.** A hole is flex leftover space: **no element, no address, no identity** (which is precisely why D-116 holds and §7.1 survived). → **ONE plate, `inset: 0`, behind the whole `.region-shell`** — not one per split (a cloud that restarts at every split boundary reads as N surfaces, not one). **The tiles are OPAQUE, so what shows through IS the holes** — identical mechanic to today's background, promoted from *paint* to *element*. **And it lights the `--region-pad` perimeter for free**, so perimeter and hole stop being two surfaces; whether they *read* as one is the plate's own setting.

> ### 🔒 **THE PLATE MAY READ THE POINTER. IT MAY NEVER CAPTURE IT.**
> `pointer-events: none`; a passive listener only. **The instant a hole becomes clickable it has an ADDRESS** — and D-116 (*a target tile is an address*) falls, and §7.1's lattice argument is live again. ***A reactive backdrop is fine. A clickable one retires the tree.***

**🔒 THE DEV RASTER IS PROMOTED, NOT DELETED:** it becomes the **first system plate widget**, so the socket ships **FED** — an unfed branch is an unverified branch (D-065 / N-091). "Solid black" is then a *setting on it*, not a special case.

> ### ⚠️ **WHY IT IS NOT NOW, AND IT IS A LOCK RATHER THAN A PREFERENCE.**
> *"…which sets it by its own **setting**."* **THERE IS NO SETTINGS MECHANISM, AND THAT IS DELIBERATE:** J-513 filed the Ch6 `settings_schema`-vs-plugin-component collision as **explicitly undecided**, binding: ***nothing is built toward either until the grid works.*** So the plate cannot be built today without picking the fenced-off thing. **Joe (2026-07-14): *"that is why i don't want to solve it now. now has priority widget grid with functional empty regions. background widget we can create after the grid concept works."*** — the same ordering as `M-RP-SKIN`, for the same reason.

**🔒 THIS ARC RESERVES NOTHING FOR IT** — no prop on `region-shell`, no descriptor key, no store, no manifest slot. *A key nothing writes is a key nobody has round-tripped* (the M-RP6.1k finding). **Zero impact on M-RP7.2.**

> **⚠️ AND THE RASTER'S DISCHARGER IS CORRECTED: it was `M-RP-SKIN` (tune it); it is now `M-RP-PLATE` (REPLACE it).** *Tuning the appearance of a thing we are about to delete is work we throw away* — the J-495 argument that rejected the interim DWM title-bar tint.

#### 4.5.2 The grid's two spacing knobs (Joe, 2026-07-14)

Grounded: `.region-shell` had **no padding** and `.app-center` sets `padding: 0` explicitly (J-499's D5); `.region-split` carried a **hardcoded `gap: 1px`** — not even a knob. Both are now tokens on `.region-shell`, **PROVISIONAL**:

- **`--region-pad`** — the gap between the **grid and the frame edge**. Set **non-zero (6px)** at Joe's request, deliberately visible for development inspection.
- **`--region-seam`** — the gap **between tiles**. Same hairline, same value; now tunable. **At M-RP7.2 the flex `gap` becomes a real seam ELEMENT** (a `gap` cannot be hit-tested) **and this token becomes its thickness** — one token, both eras, so retuning it never means finding two places.

> **🔒 THE PERIMETER IS NOT A SEAM AND CAN NEVER BECOME ONE.** Seams exist **only BETWEEN a split's children** (the `{#each}` interior) — **there is no leading or trailing seam** — so the outer edge can never grow a drag cursor or a hit zone at M-RP7.2. *It costs the splitter milestone nothing, and it cannot lie to the user about a drag that does not exist.*

**Ratios are UNAFFECTED:** flex distributes over the reduced content box, so `[1,2,7,2]` still holds **exactly** — only the absolute px shrink by `2×pad`. Global `box-sizing: border-box` (`xgen-normalize.css:21`, grounded) means the padding cannot overflow the frame.

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

### 6.1 🔒 N-120 — AN INDEX INTO THE RESOLVED TREE IS NOT AN ADDRESS INTO THE DESCRIPTOR (found M-RP7.2 / J-519). **REQUIRED LEG OF M-RP7.3.**

**M-RP7.2 shipped `path: number[]` as the address (L5), and M-RP7.4's `move` was to reuse it — *"paid once"*. It is wrong, and it was paid once for the wrong thing.**

**Grounded in the code:** `region-node.svelte:200` iterates `node.children` of a **`ResolvedNode`** and threads `path={[...path, i]}`. `resizeSplit` walks the **DESCRIPTOR** by that path. **But `resolve.ts` DROPS** — unknown leaves, `tabs`, and splits whose children all drop. **The moment anything drops, the two index spaces diverge.**

**🔑 REACHED, NOT ARGUED** (a layout planted with one unknown `widgetId`; descriptor 3 children, **2 tiles paint, 1 seam**, sitting between `spaces` and `rooms`):

| | before | after dragging that seam RIGHT — i.e. to **enlarge** `spaces` |
|---|---|---|
| descriptor `sizes` | `[1,1,1]` | **`[1660, 340, 1000]`** — the **ghost** took **1660** |
| painted `spaces` | 660px | **335px — it HALVED** |
| painted `rooms` | 660px | 986px — it **grew** |

> ### ⚠️ **THE RESIZE DID THE EXACT OPPOSITE OF THE GESTURE** — the pair actually resized was `ghost ↔ spaces`. **And 55% of the row's weight now belongs to a widget that does not exist**, which **M-RP7.5 would write to disk**, and which would **reappear eating half the screen** if that widget ever returned.

**🔒 THE ASYMMETRY, PROVEN IN THE SAME POISONED LAYOUT: FOLD IS DROP-SAFE. RESIZE IS DROP-FRAGILE.** Folding `spaces` with the ghost present collapsed **`spaces`** — correct — because **fold addresses by `regionId`: an IDENTITY.** Resize addresses by **`path`: a POSITION.** ***A position is only an address in the tree it was counted in.***

> ### 🔑 **THE RULE, AND IT OUTLIVES THIS ARC:** ***a derived view may renumber; an index into a derived view is not an address into the source.*** Either the resolved node **carries its source index**, or every mutation **addresses by identity**. **`resolve.ts` already knows the descriptor index at the moment it drops — it simply throws it away.**

**⚠️ Not reachable in today's build** (all 8 region ids registered; **`tabs` is never produced** — D-116). **It becomes reachable the first time a widget id is retired or renamed and a user loads a saved workspace** — the **W-13 reconcile** case this project explicitly designed for, and M-RP7.1b proved the Load dialog is **three clicks** away. ***"Unreachable today" is the argument that has been wrong five times in this codebase*** (N-091 · N-097 · N-099 · N-109 · N-116).

**→ M-RP7.3 opens with it. NOT a filed item.** *A misaddressed **resize** nudges two integers. A misaddressed **`move`** relocates a panel into the wrong branch.* **`move` is not built on a broken address.**

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

> ### ⚠️ **AMENDED 2026-07-13 (Joe) — "NO HOLES" IS RETIRED. "RECTANGLES ONLY" IS NOT.** Fold creates holes and **always did** (§4.5 / N-111 — it is in the shipped build). **The rule is now: RECTANGLES ONLY; HOLES ARE LEGAL and are PAINTED as a system area; A HOLE IS INERT and is NOT A DROP TARGET.** **The lattice is still not built, and the reason is UNCHANGED and does not depend on the holes clause** — see below.

A fixed **M×N cell lattice** (Grafana / react-grid-layout) is right when tiles are **independent cards** placed at free coordinates. Ours is a **space-filling tree**: **rectangles only, every tile fills its slot** — and *insert-a-split-here* is only well-defined **because** it is a tree. Going lattice means **retiring D-103's descriptor**, not extending it.

**🔑 The amendment does NOT re-open the lattice, and it is worth being precise about why.** A hole in our model is **not a free coordinate** — it is **leftover flex space inside a split that is still a rectangle**, produced by a **fold**, and **nothing can be placed in it** (§4.5). *A lattice lets you PUT things in holes. We merely let holes EXIST.* **The tree survives intact because the hole is INERT** — the moment a hole became addressable, the lattice argument would be live again.

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

## 11. Arc — the legs, visible first

**⚠️ This SUPERSEDES region-dock §7's renderer-B numbering.** §7's `7.3 save/restore layouts` **already shipped** (M-RP6.1k, D-114). §7's `7.4 custom-widget regions` and `7.5 tear-off` move out of the arc.

**⚠️ AMENDED 2026-07-13 (J-515): `M-RP7.1b` inserted; the grid lock added as `M-RP7.6`; node-inherit renumbered `M-RP7.6 → M-RP7.7`.**

| # | milestone | scope |
|---|---|---|
| 1 | **✅ M-RP7.1 — the tile frame: stripe, grip, fold** | **CLOSED J-514** (`4c2f886`). Chrome moved widget → renderer. `collapsed` entered the descriptor (`version: 2`, migrate a no-op). The eight `Section` roots unwrapped. **Joe saw it — and the appearance review produced §4.1/§4.4/§4.5, which is exactly what this leg was for.** |
| 1b | **✅ M-RP7.1b — the fold axis becomes the user's choice; splits shrink-wrap; the hole gets a floor** | **CLOSED J-516** (`0f25e50` + `14eb4d8` [Clair] + the appearance fix [Chat]). Two fold buttons (§4.1) · `collapsed` is now a **DIRECTION** (`'width' \| 'height'`) · along-fold absorbs / across-fold holes · **the split shrink-wrap (§4.4)** · **the raster under the holes (§4.5)** · `v2 → v3` and **`migrateLayout` CREATED — the first migrate this project has ever run**, exercised in vitest AND driven live through the real Load dialog. **Convention A locked after Joe saw it run.** |
| 2 | **✅ M-RP7.2 — splitter resize on the seam (CLOSED J-519, `9faa38c`)** | The `.region-split` gap became a real **seam ELEMENT** (a flex `gap` cannot be hit-tested), sized by `--region-seam`. **`mutate.ts` was BORN here** with `resizeSplit` — *the arc table put the algebra after the first mutation, and the first mutation IS algebra.* **Integers only: an exact ×10^n scale-up, pair-total invariant, untouched siblings preserved to the byte.** **Live preview; the descriptor written ONCE on `pointerup`** — **proven by reading it MID-DRAG with the button still down**, which is what leg 0's harness exists for. Clamp stops at `--region-min` and **never auto-folds**. **⚠️ Two runbook defects (N-119 paint-order hit area; the §5 misattribution) and one SHIPPED defect (N-120) — see §6.1.** |
| 3 | **M-RP7.3 — the mutation algebra (pure)** | The module beside `resolve.ts`: `move` · `fold` · `resize`, remove → collapse-degenerate → insert → re-normalise. **Vitest, no DOM, no gestures.** **🔒 AND IT OPENS BY FIXING N-120 (§6.1) — a REQUIRED LEG, not a filed item: addressing IS algebra, and `move` must not be built on a broken address.** |
| 4 | **M-RP7.4 — drag to dock: grip, edge bands** | The algebra gets a pointer. Four edge bands per tile, inert centre. **⚠️ A HOLE IS NOT A DROP TARGET (§4.5).** **Where the arc's real cost lives.** |
| 5 | **M-RP7.5 — the session layout feeder** | The grid finally **writes `session.layout`** — see §12. |
| 6 | **M-RP7.6 — the grid lock: freeze arrangement, keep function** | **NEW (Joe, 2026-07-13).** A 4th bottom-shelf face freezing fold + resize + drag; **normal function untouched.** **⚠️ IT CANNOT SHIP EARLIER — TODAY IT WOULD GUARD NOTHING** (drag and resize do not exist; a lock over one verb is a button whose whole meaning is a promise — the painted-dead chrome this project keeps refusing). **Three grounded costs:** ① **it is the FIRST STATEFUL shelf face** — grepped: `shelf-face` has `active` (roving) and `disabled` (guard) and **NO pressed/toggle concept**, so `aria-pressed` is a real change to a **shipped `core`**; ② **`locked` wants to live in `session`, where RUST writes `geometry` and the frontend writes `layout`** — **N-107 one level deeper: that object must be merged PER-KEY, never replaced** → **it lands AFTER M-RP7.5**; ③ **"lock the top shelf too" locks an EMPTY BOX today** — `app_client.svelte:277` mounts it `items={[]}` and the skin collapses it to height 0; **there is no pinning verb** → the top shelf joins the day favourites exist. |
| 7 | **M-RP7.7 — node app inherits the frame + grid** | *(the long-filed bare `M-RP7.x`, numbered per Rule 8; **renumbered from 7.6 at J-515**.)* Lands **after** the arc, so it inherits a **working** grid rather than building the frame twice. |

> ### ⏸️ **`M-RP-SKIN` — THE APPEARANCE PASS. FILED 2026-07-13 (Joe): *"majority of graphical elements will be changed or updated after UI mechanics completion."***
>
> **🔑 THIS IS THE NAMED DISCHARGER FOR EVERY `PROVISIONAL` MARKER IN THIS ARC**, and it is the reason none of them is a countdown that needs its own. The fold chevrons · the stripe/grip/triangle sizing · the folded strip's form · **`--region-pad` / `--region-seam`** (§4.5.2) — **all ship provisional ON PURPOSE and are all tuned in one pass, once the mechanics are done and there is something real to look at.**
>
> **⚠️ ONE EXCEPTION, CORRECTED 2026-07-14: the hole raster's discharger is NOT this milestone — it is `M-RP-PLATE` (§4.5.1).** `M-RP-SKIN` would **tune** it; `M-RP-PLATE` **replaces** it with a real backdrop widget. *Tuning the appearance of a thing we are about to delete is work we throw away* (the J-495 interim-tint argument). **Every provisional still points somewhere; this one just points somewhere else.**
>
> *This is the countdown rule satisfied at ARC scale rather than per-element: **WHO** = Joe, **WHICH MILESTONE** = `M-RP-SKIN`. It is NOT a deferral of a decision — it is the recognition that **you cannot tune an appearance whose mechanics are still moving underneath it.** Every provisional in the grid arc points here; none of them points at nothing.*
>
> **⚠️ It is a SKIN pass — `skin.css` and tokens only** (N-090 / N-025). If it turns out to need a component change, that is a FINDING, not a licence: **the component change is its own milestone.**

---

## 12. ⚠️ Persistence is NOT free — a grounded find

**`loadLayout()` reads `store.session.layout`. NOTHING WRITES IT.** (Grounded: the frontend's `persist()` merges only `version` / `named` / `active`; Rust merges `session.geometry`.) The read path, the file, the fallback (**N-095, exercised**) and the merge discipline **all ship** — the key has simply **never had a writer**. Today `loadLayout()` therefore *always* falls to `DEFAULT_LAYOUT`, honestly and by guard.

**The moment the grid becomes mutable, the frontend writes `session.layout` — and then BOTH writers touch the `session` object.**

> **🔒 N-107 one level deeper: the merge must be PER-KEY INSIDE `session`, not per-top-level-key.** A whole-object `session: { layout }` write **eats the `geometry` Rust just saved.** *Any format with more than one writer must be MERGED, never REPLACED* — and `session` is now such a format in its own right.

**Debounced session write on every mutation** (fold · resize · move). Manual **named** states are unaffected — S-8's *permanent, not a stopgap* stands.

---

## 13. Filed, NOT in this arc

**`M-RP-PLATE`** — the grid backdrop: an inert, live-switchable plate widget under the tiles (§4.5.1; **it REPLACES the dev raster, it does not tune it**; gated on the settings mechanism, J-513) · **`tabs`** (§3 — a position, not a deferral; re-opening it is an explicit act) · **`M-RP-ROVING`** — extract the roving-tabindex helper (no 5th consumer now) · **`M-RP-ENTITY-DRAG`** — dragging content, not regions (§8) · **`M-RP-ENTITY-PANEL-RESPONSIVE`** — an entity list fills any rectangle (§15) · **tear-off to a real OS window** (behind **M-RP8** — a torn-off region is a `decorations:false` `WebviewWindow` whose **title bar IS the tile stripe**, S-2's *one component, two mounts*; it also needs the descriptor to **record floating regions**, or W-13's re-inject silently re-docks them — **a second schema change, not free**) · **`M-RP-SETTINGS`** · **`M-RP6.1m`** — the plugin row action surface · **`M-RP-FOCUS`** · **`M-RP6.6`** — client resident · the `dialog` footer-snippet slot · N-007's ungraduated obligation · the settings-mechanism collision · the read-marker protocol gap · `temperature-indicator` ⏸️.

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
