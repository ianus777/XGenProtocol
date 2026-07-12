# M-RP6.1k — the UI-state store (session + named states + window geometry)
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jul 2026  
> **Last updated**: 2026-07-12  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

> **✅ COMPLETED 2026-07-12 — J-511. Commits `8902efa` (milestone) + `fdccdb2` (the stale-note fix). Do not re-execute.**
> **All 13 DoD legs measured** (Chat re-drove every one; no number taken on report). Client registry **55 quiescent empty-store** → **59** with a saved state · `cargo test` **1517/0/62** · `vite build` **165** · `npm test` **41** · sampler catalogue **328 unchanged**.
> **⚠️ The one defect, and the lesson the next runbook must carry (N-109):** Leg A's honest W-8 note (*“Session-only — not yet written to disk”*) **survived Legs B/C/D**, so the app told users their workspace was not being saved **while it was**. **This runbook's own §5 said Leg A must “say so in the UI” — and never said WHO REMOVES IT.** → **A W-8 disclosure is a COUNTDOWN, exactly like a disabled face; the face survived only because the DoD named it. When a leg ships a disclosure, write its REMOVAL into the DoD of the leg that lifts the limit.**
> **Two legs nobody wrote down, found at verify:** loading a named state with **no `geometry` key** (the disk grew one mid-milestone — *real data outran the fixtures*) → both guards held; and **Delete is two-step** (“Confirm delete”) — S-6's *destruction is deliberate*, honoured but never specified.
> **Filed, not built:** a **footer snippet slot on the `dialog` core** (would remove the `:has()` footer-suppress hack + the mounted-but-hidden `__close`). Its own milestone.
> **🟢 Next: M-RP6.1l — the widget manager. Its DoD flips `gear`, the last disabled face. Baseline: client registry 55, quiescent, EMPTY STORE (N-108 — state which store state you counted in).**

Build the client's **UI-state store** — one file, two lifecycles (session + **named** states), holding the **grid layout** and the **window geometry**. Absorbs **M-RP-WINSTATE**. Its DoD **flips the `diskette` and `load` shelf faces to enabled**.

Design-locked at **J-510** → **D-114** (one store; geometry typed, the rest opaque) + **D-115** (physical px; clamp; N-095 relocated). Verified in the **real client, CDP 9222** — the sampler has no frame.

---

## 0. Read first (Rule 0) + the standing hedge

`CLAUDE.md` PLAY → latest `JOURNAL.md` (**J-510**) → `tasks/` ACTIVE handoffs → both `ui/docs/` files → **then** this runbook.

Canonical sources for this arc: **`docs/xgen-widget-surfaces-phase0.md` §4 (v1.5)** · **`ui/docs/xgen-region-dock-model.md` §9 (v1.8 — read the AMENDED box at the top BEFORE the section body)** · **`DECISIONS.md` D-114 + D-115**.

> **🔑 If the code contradicts this document, THE CODE WINS — and you flag it (Rule 6).**
> **🔑 NEVER invent a number (Rule 5).** A number that disagrees with the canonical record is a **hypothesis, not a discovery** — reproduce it before you propose a record change (**N-105**).
> **🔑 Assert the UI is QUIESCENT before you count** (`menu` / `combobox` / `tag-select` / `color-picker` / `entity-context-menu` register children while open; **`dialog` is the exception** — closed = `display:none`, not unmounted). **Read `openIndex === -1` first, in a SEPARATE eval.**

---

## 1. The one paragraph that prevents the biggest mistake

**There is ONE store, and it is NOT a layout store.**

`region-dock` §9 describes an older, narrower design — `xgen-client_layout.json`, five verbs, landing at 7.3/7.6. **It is superseded.** Its **identity + reconcile rules survive verbatim** (they become the `layout` key's rules), but its **filenames and its five verbs do not exist and must not be created**.

**Do not create `get_layout`, `save_layout`, `load_layout`, `delete_layout`, `rename_layout`, `list_layouts`, `xgen-client_layout.json`, or `xgen-client_layouts.json`.** Two verbs. One file.

---

## 2. Baselines to reproduce BEFORE you touch anything

Record these; every delta is measured from them.

| baseline | value | how |
|---|---|---|
| client registry (9222, **quiescent**) | **46** | `__XGEN_DEBUG__` count, `openIndex === -1` asserted first |
| sampler catalogue (9422) | **328** | must be **UNCHANGED** at close — this milestone touches no `core` |
| `cargo test` | **1507 / 0 / 62** | **MUST MOVE** at close (Rust lands here) |
| `npm test` | **41** | |
| `vite build` | **160 modules** | |

> **⚠️ The `cargo test` leg is the INVERSE of 6.1i/6.1j.** There, *identical-to-baseline* **proved** the no-Rust claim. Here, an **unchanged** count means the Rust did not land or is untested. **Add tests for the new Rust** (parse / corrupt / clamp — all pure, all unit-testable without a window).

---

## 3. The store — shape and file

**File:** `<data dir>/xgen-client_uistate.json`, sibling to `xgen-client_config.toml`.

> **⚠️ It is NOT config, and D-101's clean-slate-on-start MUST NOT touch it.** `clean_slate_config()` wipes `xgen-client_config.toml` only. **The UI-state store is the project's first deliberately persistent user-facing state** — that is the point (D-114). If you find yourself adding it to `clean_slate_config`, stop and flag.

**Shape (illustrative — the exact TS/Rust naming is yours, flag any deviation):**

```
{
  "version": 1,
  "session":  { <a UI state> },              // exactly one, overwritten
  "named":    { "<id>": { "name": "...", "updated_at": "...", "state": { <a UI state> } } },
  "active":   "<id> | null"
}
```

A **UI state** carries **exactly two keys today**:

| key | owner | form |
|---|---|---|
| `geometry` | **Rust** | **TYPED struct** — `x`, `y`, `width`, `height`, `maximized`. **Physical px** (D-115). |
| `layout` | **the webview** | **OPAQUE** — Rust round-trips it as `serde_json::Value` and **never parses it**. |

**🔒 D-114's carve-out, and it is the milestone's core discipline.** J-499 killed `get_layout` because Rust would have **duplicated the descriptor type**. That still holds — **Rust must never learn the `Layout` node shape.** But §4.2's clamp is **mandatory**, and **only Rust can read a monitor work area or apply a rect before the webview exists**. *A clamp Rust cannot perform is not a clamp.*

> ### **Rust owns what only Rust can do, and stays blind to what the webview owns.**

**RESERVE NOTHING.** Do **not** emit `theme: null`, `shelf: []`, `collapsed: {}` or `room: null`. **Five of the six §4.5 keys have no live source in the shipped client** — an unwritten key is an unverified key, the `tabs`-branch shape at file scale. Each lands with the milestone that creates its source.

**Unknown keys are PRESERVED on round-trip** — that is why the blob is opaque, and it is what makes every future key additive with zero Rust change. **A read-modify-write must not drop a key it does not recognise.** Test this.

---

## 4. Naming — the commands are `uistate.*`, NOT `layout.*`

The faces shipped at 6.1j carrying `layout.save` / `layout.load`. **Rename them** (2 lines in `SHELF_BOTTOM`, `app_client.svelte`):

| face | command | disabled |
|---|---|---|
| `diskette` | **`uistate.save`** | **false** (this milestone flips it) |
| `load` | **`uistate.load`** | **false** (this milestone flips it) |
| `gear` | `widget.manager` | **true** — stays disabled; 6.1l flips it |

**There is no `uistate.saveAs` command.** One diskette, one dialog, two outcomes (overwrite the active state, or type a new name).

> **The store is not a layout** — it holds geometry, and will hold shelf/theme/room. `layout.*` would be a lie by M-RP6.2.

---

## 5. Legs — VISIBLE FIRST

Joe's standing brief: **he wants to SEE the UI early and correct it while the milestone is open.** Do not hide the visible part behind the plumbing.

### Leg A — the dialogs + the commands (NO persistence yet)

Shell-local, following the **`about-dialog.svelte` precedent** (wrap the shipped `dialog` core; the shell owns the assembly).

- **`uistate-save-dialog.svelte`** — a `textfield` for the name (pre-filled with the active state's name if there is one) + the list of existing named states + Save / Cancel. Typing an existing name = overwrite (**confirm in-UI** — it is destructive).
- **`uistate-load-dialog.svelte`** — the list of named states + Load / Delete / Cancel. **Delete confirms in-UI.**
- Both commands **enter `commandTable`**; **both faces flip `disabled: false`**.
- The store is **in-memory only** at this leg — a `$state` object in the shell. **Session-only, and say so in the UI if it is user-visible** (the `substitutions-editor` / W-8 posture).

**Stop here and hand back for Joe's eyes before Leg B.** *This leg exists so the milestone's UI can be corrected before any of it is load-bearing.*

> **⚠️ Leg A is NOT a closeable state.** A face is enabled and its command does real (in-session) work, which is honest — but the milestone does not close until D. **No face may be left disabled that this milestone's DoD flips.**

### Leg B — Rust persistence

- **`get_ui_state` / `set_ui_state`** in `desktop.rs` — the **shipped `get_substitutions` / `set_substitutions` shape**. Add both to `invoke_handler`.
  - App-defined Tauri commands need **no capability grant** (J-497 — grounded, not assumed; that catch was `core:window`-specific).
- Read/write `xgen-client_uistate.json` via the managed `DataDir` state. **Never re-derive the path.**
- **`loadLayout()` in `ui/client/src/layout-default.ts`: swap the BODY only.** It becomes `invoke('get_ui_state')` → pull the `layout` key → `?? DEFAULT_LAYOUT`. **The call shape does not change** — that is what the D2 seam was written for at J-499.
- **N-095, EXERCISED not asserted (its DoD moved here from 7.3 — D-115):**
  - missing file → `DEFAULT_LAYOUT`, grid renders
  - **corrupt file** (feed it real garbage) → `DEFAULT_LAYOUT`, grid renders, **no blank centre**, no crash
  - schema-stale (`version` mismatch) → `DEFAULT_LAYOUT`
  - **The blank-centre failure at J-499 was measured (registry 30→21, shell out of the DOM). Prove it does not happen — do not claim it.**
- **Reconcile (region-dock §9, surviving verbatim):** unknown `widgetId` → **drop** · missing `system` widget → **re-inject** (W-13) · unrecoverable → `DEFAULT_LAYOUT`, never crash.

### Leg C — window geometry

- **Save:** on `on_window_event` / `CloseRequested`, **and** debounced on move/resize (so a crash loses at most the last change).
- **Restore:** on start, **before the webview is shown**. Applying it later makes the window visibly jump.
- **Unit: PHYSICAL px (D-115).** `outer_position()` / `outer_size()` are physical; `work_area()` is physical. **Do not convert.**
- **Clamp, don't refuse (§4.2) — EXERCISED:**
  - if the saved rect intersects **no** current monitor's work area → **discard the geometry, fall back to default size + centre**
  - **verify by writing an off-screen rect into the store and launching** (e.g. `x: -9999, y: -9999`). Watch it centre. **A branch you read is not a branch you tested.**
- The `1240×1080` config default is now genuinely a **first-launch** default. **Do not tune it.**
- **⚠️ The twin-config rule (J-495) still bites:** `cdp.dev.conf.json` duplicates the whole `windows[0]` object. If you touch window config, **touch both files** — or the debug window keeps the old geometry and every measurement is against the wrong window.

### Leg D — named states + session, and close

- A **named state carries the ARRANGEMENT**: `layout` + `geometry` (§4.2, Joe-locked). Loading "Reading" restores its window rect too — **through the same clamp**.
- The **session** state is written on exit and read on start.
- Full CDP verify (§6) → hand back.

---

## 6. DoD — every leg proven, none asserted

| # | leg | proof |
|---|---|---|
| 1 | registry | quiescent count from **46** → enumerate the delta; `count === unique`; **`openIndex === -1` asserted in a SEPARATE eval first** |
| 2 | sampler catalogue | **328, UNCHANGED** — grounded **by scope** (`git show --stat`: zero `ui/core/**`, zero `ui/sampler/**`) |
| 3 | `cargo test` | **MOVED** from 1507/0/62, with the new tests named |
| 4 | faces | `diskette` + `load` **`aria-disabled: false`**, dispatch their commands; `gear` **still disabled** |
| 5 | dialogs | `el.matches(':modal')` true (**not** the `open` attribute — J-496: `showModal()` reflects `open` itself, so the attribute cannot tell a real modal from a silent downgrade); Close → re-open works |
| 6 | round-trip | save a named state → **relaunch** → load it → the layout and the rect come back |
| 7 | **N-095 corrupt** | feed a **real corrupt file** → `DEFAULT_LAYOUT`, grid renders, **centre NOT blank**, registry back at baseline |
| 8 | **clamp** | write an **off-screen rect** → launch → window is **centred at default size**, on screen |
| 9 | unknown-key preservation | hand-add a key to the JSON → save from the app → **the key is still there** |
| 10 | geometry | `docNoScroll` true; the grid still fills; split ratios `[1,2,7,2]` exact **at whatever width you measure at** |
| 11 | skin | **zero component-local `<style>`**; any new CSS is in **`skin.css`** (N-090 — *skinnable* includes gaps, sizing and layout; `app.css` is frame skeleton + accent knob only) |
| 12 | accent | accent-neutral chrome (only `--accent2` + focus ring move) |
| 13 | `npm test` / `vite build` | green, counts recorded |

---

## 7. Explicitly OUT of scope — do not smuggle

Shelf favourites / top-shelf pinning (surfaces §6 ④, **still open for Joe**) · `theme` (no `theme-*.css` exists — D-110) · `collapsed/expanded` (no `collapsible` prop anywhere in `ui/client`) · **last open space+room** (no room selection until M-RP6.2 — its reconcile fallback lands with its source) · **scroll position** (§4.6 — the wrong home; an anchor+backfill problem) · **read/unread markers** (§4.7 — a protocol gap; **no UI milestone may fake it or quietly persist a local marker**) · a **View/Widgets menu** (S-7 makes it free later; "free later" is not "now") · a **layout-manager widget** (under D-112 it is content inside Settings) · `widget.manager` / the `gear` (that is 6.1l) · **`tauri-plugin-window-state`** (D-115 → B; not taken).

---

## 8. Lanes, tooling, close (D-074)

- **Your commit = CODE ONLY** (commit 1). **Chat writes the doc-bridge** (commit 2). **Joe pushes both. Neither Claude pushes.**
- `Filesystem:write_file` / `edit_file` for all `E:\` writes — **never** `create_file`.
- PowerShell: **no backslash-escaped quotes**, **never `^`** in a commit message (git parses it as a revision operator → `fatal: Invalid path`).
- **Long-running processes (`cargo tauri dev`) hang the MCP server** — Joe launches dev sessions; run only short-lived commands. Static gates need the apps down.
- CDP: real client **9222**. **Single-expression `JSON.stringify({…})` evals only** (PS 5.1). Getter reads are `.get(id).state` — there is no `.get()` method on the entry. `cdp-debug.ps1` now filters the target on **scheme** (an open DevTools window is itself a `page` target — J-509).
- **Flag deviations, never absorb them.** If a leg is inconclusive, **say it is inconclusive**.

---

## 9. Definition of Done

- All 13 DoD legs green, with **measured numbers**, in the handback.
- **`diskette` and `load` are enabled and their commands do real, persistent work.** *No milestone closes leaving its own face disabled.*
- Deviations listed under Rule 6.

*(No "commit pushed" checkbox — `Status: COMPLETED` is the shipped signal.)*
