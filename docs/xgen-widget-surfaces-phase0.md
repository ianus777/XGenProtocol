# XGen Client — Widget Surfaces, Shelves & the UI-State Store: Phase-0
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-11  
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

**`shelf`** is a single `core` component (the **32nd**), mounted twice with `position: 'top' | 'bottom'`. The mounts differ **only in who populates them**. Do not build two components. *(Same reasoning as M-RP8's `title-bar`: one component, two apps.)*

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

**⚠️ Window geometry belongs to SESSION state unconditionally.** A user who never touches the diskette still expects the window to reopen where they left it. If geometry lived *only* inside named states, the common case would get no persistence at all.

### 4.2 Does a named state ALSO carry geometry?

**Yes — Joe-locked.** Loading "Reading" restores its window size and position; a named state is a *workspace*, and that is coherent (it is what Maya does).

**⚠️ With a mandatory guard: clamp, don't refuse.** If the saved rect intersects no current monitor's work area, **discard the geometry and fall back to default + centre** — a layout saved on an ultrawide must not throw the window off-screen on a laptop.

### 4.3 The unit question, settled here

**N-092b:** the Tauri window config is applied in **physical** px — measured **twice** (J-495: 900×600 → 720×480 CSS; J-498: 1240×1080 → 993×865 CSS at DPR 1.25). **This arc must settle logical-vs-physical and state it**, because a geometry value that means different things on different machines is a bug waiting for a second monitor.

### 4.4 Geometry is UI state, NOT user config

It does **not** belong in `xgen-client_config.toml`, which carries protocol / identity / user *intent*. The store is its own file (working name `xgen-client_uistate.json`), sibling to the config.

### 4.5 ⚠️ OPEN — what else goes into a UI state?

Bound it **now**, on paper, before it becomes a junk drawer:

- grid layout ✅ · shelf favourites ✅ · window geometry ✅
- **which room was open?** — open
- **scroll position?** — open
- **theme / accent?** — open

---

## 5. What is NOT changing

- The **`Layout` descriptor** (`leaf`/`split`/`tabs`) — untouched. **M-RP6.1f is entirely independent of this document.**
- The **selection bus** `{regionId, entity}` — **unchanged**, thanks to S-6.
- **W-11** (dd-socket), **W-13** (system widgets non-removable), the region-dock model (D-103).
- Frame chrome (menu-bar, status-bar) stays outside the descriptor.

---

## 6. OPEN FOR JOE

1. **Settings' own surface.** Settings is now a widget — but **which surface?** A grid leaf? Its own `window`? A modal `dialog` (built at C1)? Discord uses a full-screen overlay. **This is the first widget whose surface is genuinely non-obvious, and the natural first customer for the `window` kind.**
2. **`temperature-indicator`'s identity.** ⏸️ POSTPONED as a *visible* dd-widget (room heat as a `meter` fill), but since described as *functional / not seen directly*. **Two different widgets. Which is it?** *Do not let it quietly become both.*
3. **§4.5** — what else belongs in a UI state (room, scroll, theme)?
4. **Top-shelf pinning mechanism** — how does a user pin a favourite (manager checkbox? drag? a `+` on the shelf)? *Deliberately unanswered; the top-shelf mounts **empty** until this is decided — no dead controls (D-065).*
5. **Glyph provenance** — `gear` / `diskette` / `load` need licence-clean sources (the open M-RP-ICON-ADOPT question).

---

## 7. Build order

| # | milestone | scope |
|---|---|---|
| 1 | **M-RP6.1f** | **grid scaffold** — descriptor + renderer A + placeholder leaves + selection bus. **Independent of this doc. Next-active.** |
| 2 | *(this doc)* | **Phase-0 lock** — Joe walks §6, then the §8 records move. |
| 3 | **M-RP6.1g′** | **`shelf` core** (32nd) — ordered strip, `position: top\|bottom`, faces, roving toolbar machine, `commandId` dispatch. Sampler-verified. + 3 glyphs. |
| 4 | **M-RP6.1g″** | **mount both shelves** in the real client. Bottom → real commands. **Top → empty** (pinning undecided). Verify 9222. |
| 5 | **M-RP6.1h′** | **UI-state store** — session state + named states + window geometry (absorbs M-RP-WINSTATE) + the clamp + the unit decision. |
| 6 | **M-RP6.1h″** | **widget manager** — add / remove / configure; the home of destruction (S-6). |

**The shelf must not be folded into 6.1f.** 6.1f is grid-only and already locked; the shelf depends on a surface model that does not exist until this doc is locked. **Build the shelf before the model and you build the wrong shelf.**

---

## 8. Records to change ON LOCK (not before)

- `ui/docs/xgen-widget-tier.md` → **v1.3**: W-12 amended (§3.3) + the surfaces table.
- `DECISIONS.md` → the surface model is a **D-102 amendment** or a new **D-series** entry (Joe's call at lock; D-069's four-recurrence bar does not apply — this is a canonical-spec change, not an arc-local choice).
- `docs/ROADMAP.md` → **M-RP-WINSTATE** flips ⏸️ POSTPONED → **⬛ SUPERSEDED, absorbed into the UI-state store (§4)**; the shelf / store / manager milestones added.
- `docs/xgen-client-frame-phase0.md` → the centre pane gains top-shelf / bottom-shelf as frame chrome.

*Nothing above is edited until Joe locks §6 — Phase-0 proposes, Joe locks, then the specs move (D-069).*

---

*End of Widget Surfaces Phase-0.*
