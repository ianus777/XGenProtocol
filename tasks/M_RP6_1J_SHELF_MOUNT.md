# M-RP6.1j — mount both shelves in the real client
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jul 2026  
> **Last updated**: 2026-07-21  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

Mount the shipped `shelf` (M-RP6.1i, J-508) into the **real client frame**: a **top** strip (empty) and a **bottom** strip (three faces, **disabled**). Verified in the **real client, CDP 9222** (D-097 — the sampler has no frame).

**This is a MOUNT, not a build.** **No new component. No `core` change. No `skin.css` change. No Rust.**

---

## 0. Read first (Rule 0) + the standing hedge

`CLAUDE.md` PLAY → latest `JOURNAL.md` (**J-508**) → `tasks/` ACTIVE handoffs → **then** this runbook.

> **🔑 If the code contradicts this document, THE CODE WINS — and you flag it** (Rule 6). **Never invent a number** (Rule 5).

---

## 1. What lands

```
+---------------------------------------+
| native title bar            (OS)      |
+---------------------------------------+
| menu-bar                              |  File · Help          (shipped)
+---------------------------------------+
| TOP-SHELF                             |  EMPTY -> collapses to height 0   <- NEW
+---------------------------------------+
|                                       |
| .app-center  ->  region-shell         |  the widget grid      (shipped)
|                                       |
+---------------------------------------+
| BOTTOM-SHELF                          |  gear · diskette · load, ALL DISABLED  <- NEW
+---------------------------------------+
| status-bar                            |  led · caption · grip (shipped)
+---------------------------------------+
```

Shelves are **frame chrome, OUTSIDE the `Layout` descriptor** (S-1 / D-107) — *the controls that govern the grid must not be dockable by the grid.*

---

## 2. Decisions (Joe-locked 2026-07-12)

### D1 — BOTH strips mount. The top one mounts EMPTY.
`items: []` → `[data-empty]` → the skin collapses it (**`min-height:0; padding:0; border:0`** — grounded, read the shipped rule; it paints **no stray hairline** under the menu-bar).

**Why mount an invisible strip at all:** it exercises `[data-empty]` **in the real frame** (the sampler proved the component, not the frame), and it makes pinning a **one-line population** later rather than a new mount inside a milestone that is about something else. Top-shelf pinning is still open (surfaces §6 ④) and **gates nothing**.

### D2 — 🔒 `commandTable` is NOT touched. No phantom command ids.
The three commands **do not exist**: `widget.manager` is M-RP6.1l; `layout.save` / `layout.load` act on the named UI states that **M-RP6.1k's store** creates. *(Grounded: `commandTable` is `{ 'app.exit': handleQuit, 'help.about': … }` — nothing else.)*

**Do NOT add entries that resolve to nothing.** A command id that is registered and does nothing is **a worse lie than a disabled button** — it looks wired to every future reader, and `runCommand` would silently no-op forever.

### D3 — `onCommand={runCommand}` IS wired now.
The seam is live from day one, so **6.1k and 6.1l are each one table entry + one `disabled` flip.** With the faces disabled nothing dispatches, so a live seam costs nothing and proves the wiring exists.

### D4 — Faces mount `disabled: true` — the countdown (D7 of the 6.1i runbook)
**A visibly disabled control is an honest phase-limit (W-8), not a dead control** — the `self-panel` posture (`registered:false` rendered honestly rather than faked).

**Already proven at 6.1i, so 6.1j inherits it rather than debuting it:** `aria-disabled="true"`, `nativeDisabled:false`, **keyboard-reachable** (measured in the sampler). *That is the whole reason `aria-disabled` was chosen — with native `disabled`, an all-disabled bottom shelf would be **invisible to the keyboard**.*

> **BINDING — each line is a DoD item in the milestone that owns it:**
> **M-RP6.1k DoD** — *`diskette` + `load` flip to enabled; `layout.save` / `layout.load` enter the table.*
> **M-RP6.1l DoD** — *`gear` flips to enabled; `widget.manager` enters the table.*
> **No face is enabled before its command exists; no milestone closes leaving its own face disabled.**

### D5 — The item list is SHELL-LOCAL, not a store
A plain `const` in `app_client.svelte` (the `layout-default.ts` D7 precedent — *the shell is the only consumer*). **No `$common` store.** Promotion to a store belongs to the **manager / pinning** arc, which is the first thing with a second consumer. **Do not pre-build it.**

### D6 — Frame-column pinning lives in `app.css`
**Grounded, not argued:** `app.css` already carries `.app-frame > .menu-bar { flex: 0 0 auto; }` and `.app-frame > .status-bar { flex: 0 0 auto; }`. The shelf is the same kind of thing — a fixed row in the frame skeleton — so it takes the same line **in the same file**:

```css
.app-frame > .shelf { flex: 0 0 auto; }
```

**Nothing else goes in `app.css`.** All shelf *appearance* is already in `skin.css` (L2, N-090) and **is not touched by this milestone.**

---

## 3. Scope — exactly TWO files

| file | change |
|---|---|
| `ui/client/src/app_client.svelte` | import `Shelf`; the `SHELF_BOTTOM` const; two `<Shelf .../>` mounts |
| `ui/client/src/app.css` | one line: `.app-frame > .shelf { flex: 0 0 auto; }` |

**OUT OF SCOPE — touch none:** `ui/core/**` · `ui/common/**` · `ui/assets/skin.css` · `ui/assets/icons/**` · `ui/sampler/**` · `ui/node/**` · **any Rust** · `commandTable`.
**Prove it with `git show --stat`** — do not assert it.

> **⚠️ If the strip looks wrong in the real frame, DO NOT edit `skin.css`.** That is shared `core` territory and the block is marked **PROVISIONAL — Joe HMR-tunes it live.** **Flag it; don't fix it.**

---

## 4. Shape

```svelte
// the bottom strip's faces — shell-local (D5). Commands are DECLARED here and
// DISABLED until they exist (D2/D4): 6.1k enables save/load, 6.1l enables the gear.
const SHELF_BOTTOM = [
  { icon: 'gear',     label: 'Plugins',   command: 'widget.manager', disabled: true },
  { icon: 'diskette', label: 'Save UI state',  command: 'layout.save', disabled: true },
  { icon: 'load',     label: 'Load UI state',  command: 'layout.load', disabled: true },
];
```

```svelte
<div class="app-frame">
  <MenuBar … id="app-menubar" />

  <Shelf position="top" items={[]} ariaLabel="Favourites"
         onCommand={runCommand} id="app-shelf-top" />

  <main class="app-center"> … RegionShell … </main>

  <Shelf position="bottom" items={SHELF_BOTTOM} ariaLabel="System"
         onCommand={runCommand} id="app-shelf-bottom" />

  <StatusBar … id="app-statusbar" />
  <AboutDialog … />
</div>
```

**`aria-label` is required on both** — two toolbars in one document must be distinguishable. **Ground the exact prop names against the shipped `shelf.svelte`** before you write this; the sketch above is from the runbook, not from the file.

---

## 5. Verify — DoD (**real client 9222**, gold accent only; there is no accent swap in a single shell)

Single-expression `JSON.stringify({…})` evals (PS 5.1). **Getter reads use `.get(id).state`** (Clair's J-508 harness finding — there is no `.get()` method on the entry).

1. **Registry — MEASURE the delta, do not predict it.** Baseline is **38** (J-501, client). Report `count === unique`, and **enumerate** the new ids. *(The two `shelf#`, the three `shelf-face#`, and their `icon#…__icon` children — but the number is what you measure, not what this line implies.)*
2. **⚠️ GEOMETRY — the real risk of this milestone, and the reason it is not trivial.** Two new fixed rows enter the `.app-frame` flex column.
   - **`docNoScroll`** — `document.documentElement.scrollHeight === clientHeight`. **The N-088 scar** (`#app` had no height rule for the project's entire life) and the **J-499 rule** (`min-height:0` must ride *every* nested flex level, or the blowout puts the scrollbar on the **document**) both bite here.
   - **The grid still FILLS** — `.app-center` height unchanged-minus-the-two-strips; `region-shell` fills it.
   - **Split ratios `[1,2,7,2]` still EXACT.** *(Measure at whatever window width you have — the ratio holding at a **different** width than the last measurement is a stronger proof than reproducing a number.)*
   - **A leaf still self-scrolls** while the document does not.
3. **The collapsed top strip** — in the registry (N-053), `data-empty="true"`, **painted `getBoundingClientRect().height === 0`**, and **`getComputedStyle(...).borderBottomWidth === "0px"`**. *(N-097: the painted pixel is the leg, not the attribute. A 1px hairline under an invisible strip is exactly the kind of thing that ships.)*
4. **The bottom strip is real** — height ≈ `--ctl-h` (28px), three faces, right-aligned, sitting **above** the status-bar.
5. **Disabled, in the real frame** — all three: `aria-disabled="true"`, **`nativeDisabled: false`**, and **keyboard-reachable**: focus the strip, `ArrowRight` roves, `document.activeElement` moves. **Click a face → nothing happens and NOTHING THROWS** (`runCommand` finds no entry → no-op). *This is D2's proof: the seam is live and the table is honestly empty.*
6. **The menu-bar still works** — File↔Help roving, `Ctrl+Q` still resolves. *(Two new toolbars entered the document; prove the keymap did not get shadowed.)*
7. **N-092a — the orphan leg is NOT run, and that is correct**: the client debug bridge is **state-only** (no DOM handle), so `domCount`/orphans is a **sampler-only** capability. **Do not copy the sampler leg into a client runbook.** There is no churn in this milestone, so there is no return-to-baseline proxy either — **say so; do not substitute a weaker leg and call it the same thing.**
8. **Static gates (apps down):** `vite build` (report modules) · `npm test` **41** · `cargo test` **unchanged 1507/0/62** — *which proves the no-Rust claim rather than asserting it.*
9. **Scope** — `git show --stat` = **2 files**.
10. **Eye-check — this is the milestone's real deliverable.** Screenshot the frame. **Joe assesses the strip before close**: strip height, icon weight at 16px, gap, right-alignment, how the disabled grey reads against `--s2`, and whether the bottom strip reads as distinct from the status-bar directly beneath it. **The skin is PROVISIONAL and Joe tunes it live — flag anything that looks wrong; change nothing.**

---

## 6. Bindings

- **No `commandTable` entry. No new command id.** (D2)
- **No `skin.css`, no `core`, no glyph, no badge, no Rust.**
- **Top shelf stays empty** — pinning is open (surfaces §6 ④) and **no dead control may appear there**.
- **No store.** (D5)

---

## 7. Close (D-074)

**Two commits.** **Clair: feat, CODE ONLY** (the 2 files). **Chat: the doc-bridge** (JOURNAL + CLAUDE.md + ROADMAP + frame-phase0 §6). **Joe pushes both.**

> **⚠️ LANE NOTE, recorded because it has now slipped twice (J-498, J-508): the doc-bridge is CHAT's commit, not Clair's.** Clair's commit is code. Chat re-drives every non-destructive CDP leg itself before any number enters a canonical record (**Rule 5**).

---

*Runbook. Design locked by Joe 2026-07-12. Real client only; no sampler, no core, no Rust.*
