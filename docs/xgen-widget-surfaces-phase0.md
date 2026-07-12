# XGen Client — Widget Surfaces, Shelves & the UI-State Store: Phase-0
> **Status**: ACTIVE  
> Version: 1.4  
> Date: Jul 2026  
> **Last updated**: 2026-07-12  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

Design-only. **No code.** Phase-0 per D-071 (designs precede dependent milestones). Locks the vocabulary and constraints for the **top/bottom shelves**, the **widget surface model**, and the **UI-state store** (which absorbs the window-geometry question).

Sits above `ui/docs/xgen-widget-tier.md` (D-102) and `ui/docs/xgen-region-dock-model.md` (D-103). It **extends** both; it overturns neither.

---

## 0. Grounding correction (read this first)

During the design conversation, Chat asserted that **W-12 was "already false in the shipped code"** — that `substitutions-editor`, owning no region, violated it. **That was wrong, and it was wrong for exactly the reason this project has a Rule 0.** The claim was made from the clause's *title* ("a widget owns exactly one region") without reading the clause *body*, which says:

> **W-12** — … A widget **MAY** own a dockable **region** … Every region-owning widget maps to exactly one surface.

`MAY`. A region-less widget was always legal. `substitutions-editor` never violated anything, and no shipped code is in breach.

**The real gap is narrower and entirely constructive:** the spec's only vocabulary is **"a region, or nothing."** It has **no way to express a shelf face or an own-window widget**. That is a **gap to extend**, not a law to overturn — and the extension below is purely additive.

*Recorded rather than quietly fixed: a design doc built on a misread spec is precisely how a project drifts (D-065).*

---

## 1. What is being added

The client's centre pane gains two horizontal strips around the widget grid:

```
+-----------------------------------------+
| native title bar          (OS)          |
+-----------------------------------------+
| menu-bar                 (frame chrome) |
+-----------------------------------------+
| TOP-SHELF                (frame chrome) |  <- user favourites
+-----------------------------------------+
|                                         |
| WIDGET GRID              (the Layout    |  <- M-RP6.1f; dockable
|                           descriptor)   |
|                                         |
+-----------------------------------------+
| BOTTOM-SHELF             (frame chrome) |  <- system commands
+-----------------------------------------+
| status-bar               (frame chrome) |
+-----------------------------------------+
```

---

## 2. Locked decisions (Joe, this session)

### S-1 — Shelves are frame chrome, OUTSIDE the `Layout` descriptor

Non-negotiable, for a structural reason rather than a stylistic one: **the bottom-shelf carries the controls that govern the grid** (widget manager, save/load). If the thing that governs the grid could be docked away *by* the grid, a user can brick their own client with no way back.

Same argument that keeps the menu-bar outside the descriptor (D-107: *"File→Exit can never be docked away"*). Consistent, and it costs nothing.

### S-2 — One component, two mounts

**`shelf`** is a single `core` component (**ordinal assigned at build — `region-shell` took the 32nd slot at J-499; do not pre-book a number**), mounted twice with `position: 'top' | 'bottom'`. The mounts differ **only in who populates them**. Do not build two components. *(Same reasoning as M-RP8's `title-bar`: one component, two apps.)*

### S-3 — A shelf is an ordered STRIP, not a grid

**No splits, no tabs.** Do not call it a grid and do not give it a `Layout` descriptor — that word drags `leaf`/`split`/`tabs` along with it, and a shelf must never acquire them. It is a **1-D ordered strip of faces**: geometrically the `status-bar`'s left/right cell stacking (`sb-cell`, N-064), mostly empty, items right-aligned.

**But it is not a status-bar.** A status-bar is **passive display**; a shelf is an **active command surface** — in ARIA terms a `toolbar` (roving tabindex + arrow traversal). That machine already exists in `menu-bar` (M-RP6.1d).

### S-4 — The shelf holds FACES, not function interfaces

A **shelf face** is deliberately tiny:

- an **icon** (or small picture),
- optionally a **badge**,
- a **click** that dispatches into its widget.

**No panels. No forms. No embedded editors.** A widget does **not** render "itself, but smaller" onto a strip — that would force every widget author to be responsive across two wildly different geometries, and would turn the shelf into a second dock. *Explicitly rejected.*

### S-5 — Top = user favourites · Bottom = system

The top-shelf is a **user-curated favourites strip** — a *subset*, chosen by the user, **not an exhaustive tray**. The full widget list lives in the **widget manager**.

Consequently the `+` verb is **not** "add a widget" — it is **"pin a widget's face to the shelf."** Naming it right now prevents years of confusion later.

The bottom-shelf is **system**, and in this dev phase carries:

| glyph | command | meaning |
|---|---|---|
| `gear` | `widget.manager` | open the widget manager (add / remove / configure) |
| `diskette` | `layout.save` / `layout.saveAs` | save the current UI state, or save it under a name |
| *(load)* | `layout.load` | load a **named** UI state |

### S-6 — There is NO minus button

**Deleting a widget happens in the widget manager, never from a toolbar.**

A one-click destructive control living permanently on a strip, acting on "whatever is currently selected", is the classic accidental-delete footgun. Destruction belongs in a deliberate surface where the user can *see* what they are removing.

**Structural consequence — the reason this matters beyond ergonomics.** A minus button would have required a **second selection concept**: a *widget/leaf* selection, distinct from the locked entity-selection bus `{regionId, entity}` that feeds R8 and `entity-context-menu`. That means two buses (or one discriminated union), with a permanent hazard that clicking a *room* arms the *delete-panel* button. **Removing the minus button dissolved the problem entirely.** The selection bus stays **exactly as locked in D-107** — one meaning, one shape.

*A design decision that deletes a whole subsystem is a good design decision.*

### S-7 — Shelf items are COMMANDS

A shelf button dispatches a `commandId` into the existing `commandTable` + `KeymapRegistry` (the File→Exit machinery, M-RP6.1c/d). It does **not** carry a bare `onclick`.

Two things fall out free: accelerators (Ctrl+S → `layout.save`), and a future **Widgets** menu firing the **identical** commands. **A shelf button and a menu-item are the same thing pointed at the same command.** One dispatch, not two.

### S-8 — Manual save/load is PERMANENT, not a stopgap

Roadmap 7.3 already locks **auto-save-on-exit + auto-load-on-start**. The diskette is **not** a placeholder for it. Joe, explicitly: *"this is manual save or load besides automatic. That has to be there also. I explicitly stress that."*

The two coexist **permanently**: **auto** = implicit session persistence; **manual** = explicit **named** UI states. The diskette therefore never becomes vestigial when 7.3 lands. **Named layouts (roadmap 7.6) are pulled forward into this arc.**

---

## 3. The surface model (the extension to W-12)

> **A widget has AT MOST ONE surface.** A **surface** is *how a widget shows up*, not *what it is*.
>
> | surface | meaning |
> |---|---|
> | `region` | a leaf in the grid's `Layout` descriptor *(R1–R8 today)* |
> | `shelf` | a compact **face** on a shelf strip *(icon + badge + click)* |
> | `window` | its own OS window *(later)* |
> | *(none)* | **headless** — computes into a store, never seen |
>
> **No widget has two surfaces.** Therefore **a shelf face always means: this widget has no grid leaf.**

**Joe's rule — and it is a product choice, not a structural necessity.** It buys a strong, teachable invariant (*one widget, one place*) and kills duplicate-representation ambiguity before it starts.

**What it deliberately gives up**, recorded so nobody is surprised later: **the shelf as a remote control.** No unread-count pill for a message-stream that *also* has a grid leaf; no mute toggle for a call widget that also has a panel — those widgets already spent their one surface. If that is ever wanted it is a **spec change**, not a config flag. *That door is closed knowingly.*

### 3.1 Headless is legitimate, and it is not "unreachable"

A headless widget (`temperature-indicator`, if it stays a computation) feeds a store and has no face. That is **not** a reachability problem — *you reach its output, not it*.

### 3.2 Content-within-a-widget is NOT a surface

**The clause that retro-explains the shipped code.**

**Settings is a widget**, and it **hosts other widgets as content**. `substitutions-editor` renders *inside* Settings. Being drawn inside another widget is **not** a surface — it is content.

This separates *"where does this thing dock?"* from *"what is drawn inside it?"*, and it means **`substitutions-editor` needs no surface at all** and never did.

### 3.3 W-12 as amended

> **W-12 (amended) — a widget has at most one surface.** A widget MAY declare exactly one of: `region` (a leaf in the layout descriptor — **at most one**, unchanged) · `shelf` (a face) · `window` (its own OS window). It may declare **none** (headless). **It may never declare two.** A shelf face implies no region. Content rendered *inside* another widget is not a surface.

**Additive.** W-11 (dd-socket), W-13 (system widgets non-removable) and the entire region-dock model are untouched.

---

## 4. The UI-state store — and the absorption of M-RP-WINSTATE

Joe: *"save and load named ui states, we can put there what we need, counting present window position and dimensions."*

That decision **resolves M-RP-WINSTATE by its own written criterion** (J-498): *"did the widget grid produce a persistent UI-state store? YES → **B** — geometry becomes keys in it; no new dependency, one lifecycle."* **It did. So B.** No `tauri-plugin-window-state`. **M-RP-WINSTATE ceases to be a separate arc** and becomes a facet of this store.

*The criterion was written precisely so this question could be answered by evidence rather than re-argued. It worked.*

### 4.1 Two kinds of saved state — do not conflate them

| | **session state** | **named UI states** |
|---|---|---|
| how | implicit, automatic | explicit, user-curated |
| how many | exactly one, overwritten | many, named |
| means | *"put the app back how I left it"* | *"give me my **Reading** workspace"* |
| trigger | auto-save-on-exit (7.3) | the diskette (7.6) |
| analogue | every app ever | Maya workspaces / VS Code profiles |

Both live in the **same store**, under different keys, with **different lifecycles**.

> **✅ AMENDED (Joe-locked 2026-07-11, J-503) — the line between them, stated once:**
> **A named UI state carries the ARRANGEMENT** — layout · shelf · geometry · theme. **It does NOT carry the open room.**
> *“Reading” is a **workspace**, not a **place** — Maya restores your panels, not your scene.* **The open space+room is SESSION state only.**

**⚠️ Window geometry belongs to SESSION state unconditionally.** A user who never touches the diskette still expects the window to reopen where they left it. If geometry lived *only* inside named states, the common case would get no persistence at all.

### 4.2 Does a named state ALSO carry geometry?

**Yes — Joe-locked.** Loading "Reading" restores its window size and position; a named state is a *workspace*, and that is coherent (it is what Maya does).

**⚠️ With a mandatory guard: clamp, don't refuse.** If the saved rect intersects no current monitor's work area, **discard the geometry and fall back to default + centre** — a layout saved on an ultrawide must not throw the window off-screen on a laptop.

### 4.3 The unit question, settled here

**N-092b:** the Tauri window config is applied in **physical** px — measured **twice** (J-495: 900×600 → 720×480 CSS; J-498: 1240×1080 → 993×865 CSS at DPR 1.25). **This arc must settle logical-vs-physical and state it**, because a geometry value that means different things on different machines is a bug waiting for a second monitor.

### 4.4 Geometry is UI state, NOT user config

It does **not** belong in `xgen-client_config.toml`, which carries protocol / identity / user *intent*. The store is its own file (working name `xgen-client_uistate.json`), sibling to the config.

### 4.5 ✅ SETTLED (Joe-locked 2026-07-11, J-503) — what goes into a UI state

Bounded **on paper, before it became a junk drawer** — and grounding turned two of the three open items into *different questions than they were asked as*.

**The test (it decides every future candidate):**

> **Would you expect it to follow you to another device?** → **NOT UI state** (it is config, or it is protocol).
> **Does it describe *where things sit on screen*, rather than *what you chose*?** → **UI state.**

*(Config today — grounded from the real `xgen-client_config.toml`: `node` · `keypair_path` · `logging` · `sync` · `substitutions`. **User intent, zero presentation.** §4.4's split holds; do not erode it.)*

| item | verdict |
|---|---|
| grid layout (tiles + split sizes) | ✅ **IN** |
| shelf favourites | ✅ **IN** |
| window geometry | ✅ **IN** (clamp to the monitor work area, §4.2) |
| **collapsed / expanded** panel states | ✅ **IN** — pure presentation (`section` already carries `collapsed`) |
| **last open space + room** | ✅ **IN — SESSION state only.** ⚠️ It references **protocol objects** (`SpaceXgid` / `RoomXgid`), so it **needs a reconcile rule**: room gone / left / kicked → **fall back to no room, never crash** — the layout's unknown-`widgetId` drop, one level up. **And the fallback is EXERCISED, not asserted** (N-095's DoD shape, same milestone). |
| **theme** (the user's dark/light choice) | ✅ **IN**, **per-device** — **but ONLY as the user-choice LAYER.** ⚠️ **Ch6 already specifies a three-layer theme system:** an **application theme** (dark/light, *operator-configurable*, D-057, layered `base.css` → `skin-dark.css`) **and a SPACE THEME — declared by the Space owner via a `state.space_theme` EVENT**, overriding only a defined token subset. **Resolution order: app default → user choice → Space override.** Calling this key *“the theme”* would collide with a protocol event. Per-device is the honest scope (screen, lighting) and it invents no sync mechanism. |
| **scroll position** | ❌ **OUT — and NOT because it is hard. It is the WRONG HOME.** See §4.6. |
| **read / unread markers** | ❌ **OUT of the UI-state file — FILED AS A PROTOCOL GAP.** See §4.7. |

**And the session-vs-named line gets sharper (§4.1 amendment):**

> **A named UI state carries the ARRANGEMENT** — layout · shelf · geometry · theme. **It does NOT carry the open room.**
> *“Reading” is a **workspace**, not a **place**. Maya restores your panels, not your scene.* **The open room is SESSION state only.**

---

### 4.6 ❌ Scroll position — why it is out (J-503)

“Scroll position” is **four different things**, and only one of them is even sound:

| candidate | stores | verdict |
|---|---|---|
| **pixel offset** (`scrollTop`) | a number | ❌ unstable |
| **ratio** (% down the stream) | a number | ❌ worse — the denominator moves |
| **anchor** (the event at the top of the viewport) | an **EventXgid** | ✅ the only sound mechanism — but expensive |
| **unread boundary** (last-read event) | an **EventXgid** | ⚠️ **a different concept entirely** — semantic, not visual → §4.7 |

**A pixel offset is meaningless in THIS stream, for five shipped reasons:** **prepend** (loading older history shifts everything — `message-stream` already compensates *live* with a `scrollHeight`-delta anchor, J-485, which is itself proof the offset is not stable even **within** a session) · **edit / delete** (a tombstone is a different height) · **grouping recompute** (`grouped` rows + day-dividers change heights as the set changes — `computeRows` is `$derived`) · **window resize** (re-wrapping changes every row) · **⚠️ and the killer: on relaunch the same messages are not even loaded.** If the client opens with the last N messages and you were 300 back, `scrollTop = 1400` **points at nothing**.

**→ So restoring scroll across a relaunch is not a UI-state problem — it is a BACKFILL problem:** load history **until the anchor event is in the DOM**, then scroll to it. That is pagination + sync (`[sync].batch_size` territory), **not a JSON key**. **Storing a number would create the illusion of a feature and deliver a wrong scroll.** It would also **fight shipped code** — `message-stream` does **mount-to-bottom unconditionally** (J-485); any restore is an explicit override of a closed, verified machine.

**✅ What IS legitimate, and where it belongs:**

> **In-session, per-room scroll memory** — switching room A → B → A within one session should keep your place. That is an **in-memory `Map<roomId, anchorEventId>`**: no file, no protocol, no persistence. **Anchor on an event id even in memory** (prepends shift offsets mid-session too). **Ships with M-RP6.2** (R2 + R5 wiring), **not with the UI-state store.**

**Across-relaunch restore: DEFERRED** — it needs anchor + backfill-until-found + an **LRU cap** (one entry per room ever visited grows without bound). Real work, real dependency on history loading. **Not §4.5's business.**

---

### 4.7 ⚠️ FILED — read / unread markers have no protocol mechanism (J-503)

**Ch6's UI already renders unread counts** — `RoomListItem` = *“Room name, last message preview, **unread count**”* (§6.3), and the Space list carries one too. **But there is NO read-marker event in the protocol** — grepped Ch3 **and** Ch6: nothing.

**A read marker is per-identity state a user expects to FOLLOW THEM TO ANOTHER DEVICE.** By §4.5's own test, that makes it **not UI state**. Putting it in `xgen-client_uistate.json` would ship a **local-only marker that never syncs** — and when a protocol read-marker eventually lands, there are **two sources of truth**: **D-067 drift, self-inflicted.**

**🔑 And this is what users actually want on relaunch.** Slack and Discord do **not** restore a scroll offset — they restore you to the **unread line**. *That is evidence the real problem is the unread boundary, not the pixel: a **protocol** gap, not a UI one.*

**→ Filed as a PROTOCOL question** (a Ch3/Ch6 hole, the same species as the temperature find at J-502 — a UI chapter drawing a thing the spec never gave it a mechanism for). **No UI milestone may fake it**, and none may quietly persist a local marker to make an unread badge light up.

---

## 5. What is NOT changing

- The **`Layout` descriptor** (`leaf`/`split`/`tabs`) — untouched. **M-RP6.1f is entirely independent of this document.**
- The **selection bus** `{regionId, entity}` — **unchanged**, thanks to S-6.
- **W-11** (dd-socket), **W-13** (system widgets non-removable), the region-dock model (D-103).
- Frame chrome (menu-bar, status-bar) stays outside the descriptor.

---

## 6. OPEN FOR JOE

> **⚠️ VOCABULARY, LOCKED 2026-07-11 — read before this section.** **tile** = a PLACE (a box in the grid; one `leaf` = one tile) · **region** = a widget's FULL CONTENT SURFACE occupying a tile (it names *which widget*, not where — `regionId === widgetId` in shipped code) · **face** = a widget's COMPACT HANDLE on a shelf (icon + badge + a `commandId` click) · **window** = its own OS window · **slot** = Ch6 §6.8.3's *named, fixed* attachment point (a **different** placement model — do not merge). **A `tabs` node is ONE TILE holding SEVERAL REGIONS** — the sentence that proves the two words are not synonyms. **A face is NOT "the static one"** — both are interactive; a region **IS** the widget rendered, a face is a **handle to** it (S-4 forbids panels/forms/editors on a face). Full table: `ui/docs/xgen-region-dock-model.md` §0.

1. **Settings' own surface.** Settings is now a widget — but **which surface?** A grid leaf (a tile)? Its own `window`? A modal `dialog` (built at C1)? Discord uses a full-screen overlay. **This is the first widget whose surface is genuinely non-obvious, and the natural first customer for the `window` kind.**

   **✅ PARTIALLY ANSWERED (Joe, 2026-07-11) — the REGISTRY half is settled; Settings' own surface is still open.**
   - **The plugin list is ONE PANE with TWO ENTRY POINTS.** It lives inside Settings' structure; the **bottom-shelf gear opens the SAME pane** — not a simplified twin. *(Joe: “I have no problem to put it together and the shelf's icon-button can retrieve the same pane that is within the main setting's structure.”)* This is **S-2's “one component, two mounts”** and **S-7's one-dispatch-two-triggers**, applied to the manager. **A cut-down popup twin is explicitly REJECTED** — two surfaces for one registry drift, and the popup would be the one carrying the destructive buttons with the least context (**S-6**).
   - **NAME: “plugin list”.** Ch6 §6.8.5 calls it the **Module List**; this doc called it the **Widget Manager**. **Same object, and “plugin” is the better word** — it covers **headless** plugins with no UI at all, which “widget manager” silently excludes. *(Joe: “module and widget is the plugin in the two areas: system and ui.”)*
   - **Built-ins ARE listed — distinguished, and NOT removable.** Ch6 §6.8.5 already draws the **`[system]` / `[user]` mode badge** on every entry: **that is W-13, pre-figured in Ch6 before the widget tier existed.** `self-panel` / `inspector-panel` list as `[system]`, configurable + redockable, **Remove disabled**.
   - **STILL OPEN:** what surface **Settings itself** gets (grid tile / `window` / chrome screen). **⚠️ And note the surface vocabulary cannot currently express Ch6 §6.8.5's *“a screen of its own”*** — there is no `screen` kind. That is part of the §9 taxonomy Phase-0, not a decision to take in passing.

2. **`temperature-indicator`'s identity.** ⏸️ POSTPONED as a *visible* dd-widget (room heat as a `meter` fill), but since described as *functional / not seen directly*. **Two different widgets. Which is it?** *Do not let it quietly become both.*

   **✅ ANSWERED (Joe, 2026-07-11) — and the question was built on a false premise, which grounding exposed.**
   - **⚠️ IT IS NOT A COMPUTATION. IT IS A RENDERER OF AN EXISTING PROTOCOL PROPERTY.** Temperature is **spec §3.7.13 (Status: complete)** + **Ch6 §6.12** + **D-061**: reserved `meta_atts` keys **`xgen.room_temperature`** / **`xgen.member_temperature`** (floats `[0,1]`), a threshold table, buckets `cool|warm|hot|fiery`. **The math is a plugin on the Room's home Node, deliberately outside the protocol.** Ch6 §6.12.1: the values are **opaque** — *“the client does not know how they were computed, does not attempt to re-derive them, and treats them as authoritative.”*
   - **The moral question this doc raised was ALREADY ANSWERED by the protocol:** §3.7.13.3 ships **`member_temperature_visibility: moderator | everyone | self_only`** on Space state. And member heat is **accumulated overpass of the Space's own pacing rules** (§3.7.12 / D-060) — a measure against a rule the Space set for itself, **not** a reputation score.
   - **Client Rust ALREADY EXISTS:** `xgen-client/src/temperature.rs` — `TemperatureUpdate {space_id, room_id, subject_id, temperature, state}`, the `__room__` sentinel, bucket derivation, a `temperature_update` Tauri event, **and a DOM contract**: `data-temp-state` + `--xgen-room-temperature` / `--xgen-member-temperature`.
   - **Identity, therefore: a `$common` STORE fed by the existing `temperature_update` event, rendered as CONTENT inside other widgets** (room heat → R2 rows / R4 header; member heat → R7 rows / message rows). **Content inside a host is not a surface (§3.2) → it spends NO surface, and it is NOT a dockable panel.**
   - **⚠️ THE `meter` + W-11-dd-socket FRAMING IS WITHDRAWN, NOT CARRIED FORWARD.** It predates Ch6 §6.12's `data-temp-state` contract and predates the widget tier itself — *“the first bird”*, named before any of the conventions it was described in existed (Joe). **Re-derive at kickoff against the shipped contract; do not inherit the old mechanism.**
   - **GATE (deferred):** live messaging (**M-RP6.3** + R5 on live data) **AND** a home-node plugin that actually publishes values — `NoOpTemperaturePlugin` returns `None` today, **so there is nothing to render.** *That second half is a node/plugin arc, not a UI one — the M-RP6.6 shape again: a UI milestone cannot manufacture a source that does not exist.*
   - **BINDING UNTIL THEN: no milestone reserves a heat slot or reads a heat store.** R2 / R4 / R7 ship without it.

3. **§4.5** — what else belongs in a UI state (room, scroll, theme)?

   **✅ CLOSED (Joe, 2026-07-11 — J-503). See §4.5 / §4.6 / §4.7.** Two of the three sub-questions turned out to be **different questions than they were asked as**. **IN:** layout · shelf favourites · geometry · collapsed/expanded · **last open space+room** (session-only, with an **exercised** reconcile fallback) · **theme** (per-device, and **only the user-choice layer** — Ch6 already has an app theme **and** a `state.space_theme` **protocol event**). **OUT:** **scroll position** — not deferred-because-hard but **the wrong home**: a pixel offset is meaningless in this stream (prepend / edit / grouping recompute / resize / *and the messages are not even loaded on relaunch*), the sound mechanism is an **anchor + backfill** (sync work), and the in-session case is an **in-memory `Map<roomId, anchorEventId>`** landing with M-RP6.2. **OUT + FILED:** **read/unread markers** — Ch6 renders unread counts with **no protocol mechanism behind them**; a local-only marker would never sync and would become a **second source of truth**. **A named UI state carries the ARRANGEMENT, not the open room** (§4.1).

4. **Top-shelf pinning mechanism** — how does a user pin a favourite (manager checkbox? drag? a `+` on the shelf)? *Deliberately unanswered; the top-shelf mounts **empty** until this is decided — no dead controls (D-065).*
5. ~~**Glyph provenance** — `gear` / `diskette` / `load` need licence-clean sources.~~ **❌ STRUCK 2026-07-12 — NOT A DESIGN QUESTION.** **Verified against `docs/xgen-icon-adoption.md` §3f (not assumed):** under **D-108** *"licence + source live in `icons.manifest.json`, **per glyph** — a glyph with no licence entry **fails the build**."* **No audit can forget what the compiler enforces.** What remains is **mechanically sourcing** gear / diskette / load — **a task inside M-RP-ICON-ADOPT, not a decision blocking anything.**

**✅ Status (2026-07-12, D-112 / D-113 / J-507): items ① ② ③ CLOSED · item ⑤ STRUCK. ① Settings takes `surface: window` — NO `screen` kind is added** (the `window` form already exists and has a second consumer: a `packaged` plugin's Launch button; *Ch6 §6.8.5's "a screen of its own" is prose, not a surface kind*). **⚠️ Foreclosed knowingly:** the **Discord full-window overlay** shape — that would be a fifth surface kind, and it must be a **lock, never a drift**.

> ### **ONLY ④ (top-shelf pinning) REMAINS OPEN — and it gates NOTHING.**
> The top shelf mounts **empty** until it is answered (no dead controls, D-065). **M-RP6.1i–l are UNGATED.**

---

## 7. Build order

**⚠️ RELABELLED (J-500, Joe-locked — the primes are GONE).** v1.0 numbered this arc `6.1g′ / 6.1g″ / 6.1h′ / 6.1h″`, which **collided** with the frame arc's own `6.1g` (R3) and `6.1h` (R8). Primed labels are a symptom of an arc written *around* an occupied slot, and they would make every future grep for “6.1g” hit **two different milestones**. The frame arc keeps **6.1g** (R3 ✅ J-500) and **6.1h** (R8); this arc takes **6.1i–l**.

| # | milestone | scope |
|---|---|---|
| 1 | **M-RP6.1f** ✅ | **grid scaffold** — descriptor + renderer A + placeholder leaves + selection bus. **DONE (J-499).** Independent of this doc. |
| — | **M-RP6.1g** ✅ | *(frame arc)* R3 self-panel — the first real **system widget** + the bus's **first writer**. **DONE (J-500).** Not this doc's work, but it proved the thing this doc depends on: **a region widget can be fed only through a `$common` store** (N-096). **A shelf face's data will have the same constraint.** |
| 2 | *(this doc)* | **Phase-0 lock** — **✅ LOCKED 2026-07-12 (J-507).** §6 walked; ①②③ closed, ⑤ struck, **only ④ (top-shelf pinning) open — and it gates nothing.** The §8 records have moved. **M-RP6.1i–l may start.** |
| 3 | **M-RP6.1i** | **`shelf` core** — ordered strip, `position: top\|bottom`, faces, roving toolbar machine, `commandId` dispatch. Sampler-verified. + 3 glyphs. *(was `6.1g′`)* |
| 4 | **M-RP6.1j** | **mount both shelves** in the real client. Bottom → real commands. **Top → empty** (pinning undecided). Verify 9222. *(was `6.1g″`)* |
| 5 | **M-RP6.1k** | **UI-state store** — session state + named states + window geometry (absorbs M-RP-WINSTATE) + the clamp + the unit decision. *(was `6.1h′`)* |
| 6 | **M-RP6.1l** | **widget manager** — add / remove / configure; the home of destruction (S-6). *(was `6.1h″`)* |

*The `shelf` was called “the 32nd `core`” in v1.0. **It is not** — `region-shell` took the 32nd slot at M-RP6.1f (J-499). The shelf's ordinal is assigned when it is actually built; **do not pre-book a number** (Rule 5 applies to counts in designs too).*

**The shelf must not be folded into 6.1f.** 6.1f is grid-only and already locked; the shelf depends on a surface model that does not exist until this doc is locked. **Build the shelf before the model and you build the wrong shelf.**

---

## 8. Records to change ON LOCK (not before)

- `ui/docs/xgen-widget-tier.md` → **v1.3**: W-12 amended (§3.3) + the surfaces table.
- `DECISIONS.md` → the surface model is a **D-102 amendment** or a new **D-series** entry (Joe's call at lock; D-069's four-recurrence bar does not apply — this is a canonical-spec change, not an arc-local choice).
- `docs/ROADMAP.md` → **M-RP-WINSTATE** flips ⏸️ POSTPONED → **⬛ SUPERSEDED, absorbed into the UI-state store (§4)**; the shelf / store / manager milestones added.
- `docs/xgen-client-frame-phase0.md` → the centre pane gains top-shelf / bottom-shelf as frame chrome.

*Nothing above is edited until Joe locks §6 — Phase-0 proposes, Joe locks, then the specs move (D-069).*

> **✅ DONE 2026-07-12 (J-507).** All four moved, plus the ones §9 forced: `xgen-widget-tier.md` **v1.3** (W-12 amended + the **delivery axis**) · **D-112 + D-113** (a new D-series, not a D-102 amendment — the surface model turned out to be one face of a **plugin-wide** taxonomy) · `docs/ROADMAP.md` · `docs/xgen_ch6_client_design.md` **v0.6** (§6.8.3 amended, slot table **STALE**, §6.8.7 **corrected**, §6.8.8 **three of five CLOSED**) · `ui/docs/xgen-region-dock-model.md` **v1.7** (§11 closed). **M-RP-WINSTATE stays ⏸️→ absorbed by the §4 UI-state store at M-RP6.1k**; the frame-phase0 shelf note lands with **M-RP6.1j**, when the shelves are actually mounted (D-065 — records follow the build, not the plan).

---

## 9. ✅ CLOSED — the plugin taxonomy Phase-0 (filed 2026-07-11 · **CLOSED 2026-07-12, D-112 / D-113 / J-507**)

> ### ✅ RESOLVED — and this document's own §3.2 clause is what resolved it.
>
> **Phase-0:** `docs/xgen-plugin-taxonomy-phase0.md`. **Decisions: D-112** (taxonomy) **+ D-113** (the packaged-UI sandbox) — **locked together**, because *you cannot classify a thing while leaving open what it is allowed to do*.
>
> **🔑 One plugin, THREE axes:** **`host`** (`node` = *system* · `client` = *ui*) · **`delivery`** (`compiled` · `service` · `packaged`) · **`surface`** (`none` · `region` · `shelf` · `window`). **"Module" and "widget" are not two species** — and **`xgen-common/src/module.rs` already said so in code**, before either spec was written.
>
> **🔑 This doc's surface set was NOT a bad re-derivation of Ch6 — it was the RIGHT list, and §3.2 was the missing half.** Split Ch6's slot table against *"content inside another widget is not a surface"*: `node.dashboard.widget` / `room.sidebar.*` are **regions**; `room.toolbar` / `room.message.decorator` / `space.header` / `global.statusbar` are **content anchors**. **The anchor mechanism ALREADY SHIPS** (`message.details: WidgetMount[]`, unknown-id drop, W-13) — ***`room.message.decorator` is `message.details` under another name.*** → **ONE placement model + ONE containment model. Nothing retires; nothing competes.**
>
> **🔑 A THIRD species that neither list had:** the **Auth Module is `delivery: service`** — `AuthModuleXgid` + `endpoint_url` + revocation (**not** the Window-form package Ch6 §6.8.7 claims; **corrected in Ch6 v0.6**).
>
> **🔒 And the sandbox (D-113): the boundary is DELIVERY, not "widget".** `self-panel` needs no CSP; a compiled Rust engine needs no CSP. **A `packaged` module UI has NO NETWORK (S-1)** — which makes **D-111's beacon unsayable inside a module** — plus own origin, no Tauri IPC, never holds the key, **no trust chrome**, deny-by-default caps, **and S-7: it cannot load at all until that floor ships.**
>
> **→ M-RP6.1l and M-RP7.4 are UNGATED.** *The original filing is kept below, unedited.*

---

### 9.1 The item as originally filed (2026-07-11) — kept as history

Grounding **Ch6 §6.8** during the §6 walk surfaced a collision this document did not know about. **Nothing shipped is wrong — the two SPECS disagree.**

**Ch6 §6.8.3 already defines three Module UI Forms: Headless · Widget · Window.** This doc's surface set (`region | shelf | window | none`) is — in hindsight — **a re-derivation of that list, made without consulting it.** They agree on *headless* and *window*, and **diverge exactly where placement lives**.

**And "widget" means two different things:**

| | **Ch6 §6.8.3 widget** | **D-102 widget** (shipped) |
|---|---|---|
| what | HTML in an **isolated webview** | a **Svelte component**, in-process |
| talks to | its module backend over a **local WebSocket** | a **`$common` store** |
| placed by | a **named slot** (`room.sidebar.bottom`, `global.statusbar`, …) | a **`region`** in the D-103 descriptor — dockable |
| authored by | a third party, any language, **package + manifest** | us, in the client tree |

**→ So there are TWO PLACEMENT MODELS: Ch6's fixed named-slot inventory, and D-103's free dockable descriptor.**

**Joe's reconciliation frame (and it is the right one):** *“module and widget is the plugin in the two areas: system and ui.”* **One plugin, one list, several UI forms.** The work is **alignment**, not a choice between them.

**What the Phase-0 must answer** (spanning **D-036** · **D-102** · **D-103**):
1. Is a D-102 widget a **module**? Does a module **contribute** one? Or are they distinct species that merely share a registry?
2. Do the **slot inventory** and the **layout descriptor** unify, coexist, or does one retire?
3. Does the surface vocabulary need a **`screen`** kind? *(Ch6 §6.8.5 says the plugin list is “a screen of its own” — and `region | shelf | window | none` cannot express that. This is what blocks §6 item 1's remainder: **Settings' own surface**.)*
4. What does the plugin list render for a **built-in** with no package, no manifest and no socket? *(Joe: listed, distinguished, **no Remove button** — Ch6's `[system]` badge is already exactly this, and it is **W-13 pre-figured**.)*

**Does NOT block M-RP6.1i / M-RP6.1j** — the `shelf` core and its two mounts are pure UI and depend on none of this. **DOES block M-RP6.1l** (the plugin list must list both species) and **M-RP7.4** (custom-widget-contributed regions — which *is* Ch6's slot mechanism under another name).

---

*End of Widget Surfaces Phase-0.*
