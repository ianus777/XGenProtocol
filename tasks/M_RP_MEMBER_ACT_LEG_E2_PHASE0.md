# M-RP-MEMBER-ACT — Leg E-2: the system-region re-inject — Phase-0
> **Status**: ACTIVE  
> Version: 1.2  
> Date: Aug 2026  
> **Last updated**: 2026-08-13  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — WHAT E-2 IS

**One sentence:** *`DEFAULT_LAYOUT` stays at eight leaves (Joe's ①-B), so re-injection is the ONLY path that ever places `dm-spaces` — and it must therefore run on **every** path that produces a layout from persisted data or from the default.*

E-2 builds **`D-114` §9's re-inject rule**: a system `regionId` absent from a loaded layout is re-injected. It is **not** a `v3 → v4` migrate — that framing was superseded before E-1 (Leg E Phase-0 §4① annotation).

**No code. No runbook. Phase 0 of D-071's four phases.**

---

## §1 — STATE AT OPEN, RE-MEASURED

| item | measured |
|---|---|
| tree | **CLEAN** (`git --no-pager status`) |
| `HEAD` | `dccc9b12a8576b3932787cfebfab8dee6ad74ae6` |
| `git ls-remote origin refs/heads/main` | `dccc9b1…` — **identical**, not the tracking ref |
| latest record | J-721 · ROADMAP v7.08 · notes v1.24 (to N-194) · Leg E Phase-0 v1.5 · E-1 runbook v1.2 COMPLETED |

**Floors, stated not re-run** (this document is reads only; zero `.rs`, zero `ui/**`): `svelte-check` **0 / 34 / 15** · catalogue **435**.

🛑 **`cargo` IS NOT A FLOOR FOR LEG E.** `K2` shipped in Leg B; Leg E touches zero `.rs`. An identical `cargo` result is a **scope argument, not a measurement** (`F8`; parent Phase-0 §6 is stale on this).

🛑 **NO REGISTRY NUMBER IS CARRIED.** `N-184` (Space-dependent) · `N-190` (draft-dependent) · **`N-194` (168 → 174 on an IDENTICAL screen, cause = `CLIENT_PLUGINS` 10 → 11 rows in Settings ▸ Plugins, *nothing to do with DM rows*)**. **Record the screen, or record no number. Enumerate, never derive.**

📌 **Apps are DOWN.** Nothing here was measured live. Every pointer below came from a tool that printed it (`Select-String` / a ranged file read) — **`W1`'s rule, applied at authoring time rather than repaired afterwards.**

🛑 **ANNOTATION AT THE SITE (`D-131`, 2026-08-13): AND IT WAS STILL BREACHED ONCE IN §2.** `DEFAULT_LAYOUT` was cited `:105-125` and is **`:103-123`** — found by **Clair's E-2 runbook read (`W-3`)** and re-measured by Chat. **Corrected above, not silently.** 🔑 *Same-file pointers `:14` and `:137` were exact, and `git diff dccc9b1..HEAD` over the six source files is **EMPTY** (both later commits are docs-only) ⇒ this was a **MIS-MEASURE at the grounding commit, not drift.*** ⚠️ **Three more sat in the runbook's own §1** (the hydrate seeds · the DEV-bridge bounds · `dm-spaces.svelte`'s directory), all non-edit-target, all corrected at runbook v1.1. **Every EDIT-TARGET pointer in both documents was exact.** *The two documents that quote the `W1` rule carried four instances of it between them — which is the argument for the read, not against the rule.*

---

## §2 — THE AUDIT (grounded at `dccc9b1`)

| surface | file:line | what it actually does |
|---|---|---|
| **`loadLayout`** | `ui/client/src/layout-default.ts:137` | `async (): Promise<Layout>` — **no parameters.** Reads `get_ui_state` → `store.session.layout` → `migrateLayout(layout, DEFAULT_LAYOUT)`; **TWO return statements**, the second `return DEFAULT_LAYOUT` |
| caller ① boot | `app_client.svelte:709` | `layout = await loadLayout();` — inside `onMount`, **after** `installed.hydrate` / `hydrateDisabled`. **Does not persist** |
| caller ② revert | `app_client.svelte:586` (fn at `:585`) | `layout = await loadLayout();` in `handleRevertUi`, wired to the LIVE `layout.revert` command. **Does not persist** (its own comment at `:583` says so, deliberately) |
| **entry point ③** | `app_client.svelte:895` (fn at `:889`) | 🛑 `if (s?.layout) layout = migrateLayout(s.layout, DEFAULT_LAYOUT);` in `handleUistateLoad` — **a persisted layout assigned WITHOUT going through `loadLayout()`** |
| `insertLeaf` | `mutate.ts:266` | `(layout, newWidgetId, targetId, edge)` — pure, TOTAL, **idempotent**: already-docked → no-op by reference; target missing → no-op by reference |
| `migrateLayout` | `resolve.ts:161` | `if (l.version >= 3) return raw as Layout;` — **short-circuits**; never returns null |
| drop-unknown | `resolve.ts` rule 2 (walk at `:63`) | BUILT and reported in `dropped` |
| the mounted set | `app_client.svelte:102` | `const mountedPlugins = $derived(installed.mounted)` |
| `installed.mounted` | `installed.svelte.ts` `get mounted()` | `[...CLIENT_PLUGINS, ...AVAILABLE_CUSTOM.filter(installed && !disabled)]` |
| the `dm-spaces` row | `registry.ts:214-224` | `kind: 'system'` · `surface: 'region'` · `regionId: 'dm-spaces'` |
| the id lists | `layout-default.ts:20-22` · `:29-43` | `REGION_IDS` = **9** ids (incl. `dm-spaces`) · `REGION_NAMES` fallback title present |
| `DEFAULT_LAYOUT` | `layout-default.ts:103-123` | **8 leaves**, root `dir: 'row'`, `sizes: [1,2,7,2]`; `spaces` is a direct child of the ROOT `row` split |
| the widget | `dm-spaces.svelte:33` | props `{ regionId }` only; envelope id `region-${regionId}` ⇒ `dm-spaces#region-dm-spaces` |

---

## §3 — FINDINGS

### 🛑 F1 — THERE ARE **THREE** ENTRY POINTS, NOT TWO. THE THIRD IS `handleUistateLoad`, AND IT WOULD STRAND THE HOME EXACTLY THE WAY `File ▸ Revert` WOULD.
`P2` (Leg E Phase-0 §3) established two `loadLayout()` callers and concluded *"the hook lives inside `loadLayout()`"*. **That conclusion is correct about those two callers and does not cover the surface.**

**`app_client.svelte:895` assigns `layout` from a persisted named UI state via `migrateLayout(s.layout, DEFAULT_LAYOUT)` — it never calls `loadLayout()`.** A named state saved before `dm-spaces` existed carries an eight-leaf tree ⇒ **loading it removes the DM home from the running app**, and per `F5` the self thread with it.

🔑 **THE SPECIES IS THE ARC'S RECURRING ONE — A CLAIM NARROWER THAN THE THING IT DESCRIBES, REUSED AS IF COMPLETE.** *"Two callers of `loadLayout`"* is true. *"Two paths that produce a layout from persisted data"* is false. The kickoff, the parent Phase-0's `P2`, and the E-1 runbook all carry the narrow form. ⚠️ **It survived Clair's adversarial read**, because the read was checking the claim that was written, not the claim that was needed.

✅ **REACHABLE, NOT THEORETICAL — by the same argument `P2` used for `layout.revert`:** M-RP7.1b drove `handleUistateLoad` **live through the real Load dialog** (shelf face → combobox → Load) to exercise the migrate. **Three clicks.**
📌 **Joe has ZERO saved UI states today** (E-1 runbook §1, measured), so nobody can hit it *this minute* — and `N-115` records that one diskette click ends that permanently.

### ✅ F2 — THE RE-INJECT MUST COVER THE `DEFAULT_LAYOUT` RETURN TOO, WHICH IS WHAT ①-B MEANS IN CODE
`loadLayout` returns `DEFAULT_LAYOUT` on **no-Tauri / absent store / corrupt store / absent `session.layout`**, and `DEFAULT_LAYOUT` has **eight leaves and no `dm-spaces`** (Joe's ①-B, `layout-default.ts:105-125`). ⇒ **a fresh client with no store would show no DM home at all** unless the re-inject wraps that return as well.

⇒ **`loadLayout` gains a SINGLE EXIT.** Both `return` statements route through one re-inject call. *This is not a style preference — a re-inject on one of two returns is a home that appears or not depending on whether a file exists.*

✅ **Returning `DEFAULT_LAYOUT` by reference is safe:** `insertLeaf` is immutable (`{ ...layout, root: … }`), so the module-level const is never mutated. Only its object identity stops being shared, which nothing reads.

### 🛑 F3 — THE KICKOFF PUTS `loadLayout` IN `core`. IT IS SHELL-LOCAL.
The session kickoff's audit list reads *"`ui/core/lib/components/layout/{resolve,mutate,types}.ts` — `loadLayout`, `migrateLayout`"*. **`migrateLayout` is in `resolve.ts` (core). `loadLayout` is in `ui/client/src/layout-default.ts` (shell)** — and the file's own header says why: *"renderer A + the descriptor type are `core`, but the concrete default tree, the id→component map, and the (future Tauri) load seam are the client's."*

🔑 **THE DISTINCTION IS LOAD-BEARING, NOT PEDANTIC.** M-RP7.1b's Rule-6 deviation exists because a runbook nearly made `core` import the shell's `DEFAULT_LAYOUT` (`resolve.ts`'s own comment records it). **A placement table names shell ids and a shell default target — it belongs in the shell, and the audit instruction that put `loadLayout` in `core` points the opposite way.** *Kickoff item ④ applied to the kickoff.*

### ✅ F4 — THE RE-INJECT NEEDS A **PLACEMENT**, AND NEITHER ID LIST CARRIES ONE
`insertLeaf` requires `(newWidgetId, targetId, edge)`. Neither `REGION_IDS` nor `CLIENT_PLUGINS` carries a target or an edge. ⇒ **a rule whose domain is an id list it cannot place has an unfed branch (`N-091`)** — see §4①.

📌 **The two candidate id sets are NOT the same set, and the difference is exactly one id:** `REGION_IDS` holds **9**; `CLIENT_PLUGINS` rows with `kind: 'system' && surface: 'region'` hold **8** — `room-header` is a region id with **no plugin descriptor** (it renders `RegionPlaceholder`). Today the difference is inert (`room-header` is in `DEFAULT_LAYOUT` and unremovable), but the two sets must not be treated as interchangeable in a rule that is meant to serve every future system region.

### ✅ F5 — THE UNAMBIGUITY PREMISE HOLDS, AND ITS REAL GUARD IS STRUCTURAL, NOT `:554`
The kickoff cites `app_client.svelte:554` guarding `desc.kind !== 'custom'`. **That line is real and correct** — but it is the **disable/enable** guard (`handlePluginToggleDisabled`), not the removal path, and it is the *third* of three.

**The load-bearing guard is that every lifecycle verb is gated on `AVAILABLE_CUSTOM`, and a system plugin is never in it** (`installed.svelte.ts`: `isAvailable` → `install` no-op; `_installed.has` → `uninstall`/`disable` no-op; `handleUninstall`/`handleInstall`/`handlePluginToggleDisabled` all `AVAILABLE_CUSTOM.find(...)` first). ⇒ **a user cannot remove a system region by any route**, so **absence of a system `regionId` can today only mean "saved before that region existed"**, and re-injection is unambiguous rather than a guess about intent.

🛑 **THAT PREMISE HAS A COUNTDOWN, AND IT IS ALREADY DoD-BOUND ELSEWHERE.** `ROADMAP.md:333` (`M-RP-WIDGET-SUSPEND`) carries: *"DoD-BOUND: M-RP-MEMBER-ACT Leg E-2's re-inject MUST consult this milestone's hidden set."* **E-2 ships UNCONDITIONAL; the guard is owed THERE (`N-182` — reserve nothing here).** One observable, two meanings: the `G13` shape.

### ✅ F6 — THE MECHANISM IS ALREADY TOTAL AND IDEMPOTENT; E-2 ADDS NO ALGEBRA
`insertLeaf` (`mutate.ts:266`) no-ops **by reference** on an already-docked id and on a missing target, and never throws. ⇒ running the re-inject on **every** load is free and cannot double-place, cannot blank the centre (`N-095`), and cannot fight a user who has moved the home elsewhere. **No new `core` code is required** — E-2 is composition in the shell.

### 📌 F7 — THE `spaces`/`bottom` PAIR: ONE HALF VERIFIED FROM SOURCE, ONE HALF NOT
✅ **Under `DEFAULT_LAYOUT`, verified from source:** `spaces` is a direct child of the **root `row`** split. `edge: 'bottom'` → axis `col` ≠ `row` ⇒ `insertBeside` takes the **WRAP** branch ⇒ `[spaces, dm-spaces]` in a new `col` split occupying the old `spaces` slot. **Matches the runbook's claim.**
⚠️ **In Joe's live tree the SIBLING claim (`spaces`'s parent already runs `col` ⇒ `[spaces, dm-spaces, self]`) rests on a live read taken at E-1 verify and is NOT re-derivable from source** — his arranged tree is on disk, and the apps are down. **It is a measurement to repeat, not a fact to inherit.**

---

## §4 — OPEN, AND JOE'S. Each carries `D-121`'s **THREE** lenses: ① user-visible impact per option → ② tier consequence → ③ resource cost.

📌 **Lens ② for every item below is *NO TIER CONSEQUENCE*, stated once rather than manufactured four times** — all four are layout-placement mechanics. Nothing moves a byte, creates a copy of anyone's data, or decides whose tier governs. **`D-121` says that is a legal answer, and a fabricated tier rationale is as bad as a fabricated UX one.**

### 🔒 ① — **CLOSED 2026-08-13: D-c, THE PLACEMENT TABLE. PROVENANCE DELEGATED** (*"go by your recommendations"*, `D-141`). Recorded as a DELEGATION, not as Joe deriving D-c — a one-line approval is easy to over-extend into a ruling its author never made.

### 🔓 ① — WHAT IS THE RULE'S DOMAIN? (`F4`)

**D-a — iterate `REGION_IDS`.** ① None today (`room-header` is never absent). ③ Zero. 🛑 Nine ids, one placement — eight branches that cannot be placed. **The unfed-branch shape.**
**D-b — iterate `CLIENT_PLUGINS` system region rows.** ① None today. ③ Zero. 🛑 Same defect, one id smaller; and it would silently start re-injecting any future system region **at whatever default target was chosen for `dm-spaces`.**
**D-c — a `SYSTEM_REGION_PLACEMENT` table in `layout-default.ts` IS the domain.** One row today: `'dm-spaces' → { target: 'spaces', edge: 'bottom' }`. The re-inject iterates the **table**, skips any id already present, and skips any id **not in the mounted registry** (so a leaf can never be injected that `resolve.ts` would immediately W-13-drop). ① Identical on screen. ③ ~12 lines, shell-only, zero `core`, zero Rust. ✅ **Every future system region gets it free by adding one row** — which is exactly what ①-A ruled the general rule to be.

📌 **Chat's recommendation: D-c.** The table carries the one thing the rule needs and neither id list has; and a region added to the table without a widget is refused rather than dropped one frame later.

### 🔒 ② — **CLOSED 2026-08-13: S-3, ONE HELPER, TWO CALL SITES. PROVENANCE DELEGATED** (`D-141`). 🛑 **S-4 (the `applyLayout` funnel) is NAMED AND NOT TAKEN** — it is an architecture change to the shell and is Joe's, filed as its own milestone rather than ridden in on a placement leg.

### 🔓 ② — WHERE DOES THE RULE RUN? (`F1` is the whole content of this item)

**S-1 — inside `loadLayout()` only, as `P2` ruled.** ① Boot ✅ · `File ▸ Revert` ✅ · **loading a named UI state ❌ — the home vanishes from the running app.** ③ Smallest. 🛑 **Ships the exact stranding H1 exists to prevent, through a path M-RP7.1b already drove live.**
**S-2 — the rule written twice (in `loadLayout` and at `:895`).** ① Correct on all three. ③ Small. 🛑 Two copies of one rule = the `D-067` drift surface this arc has refused four times.
**S-3 — ONE exported helper, called from `loadLayout`'s single exit AND from `handleUistateLoad`.** ① Correct on all three. ③ ~6 lines plus the table; two call sites, **one rule**. ⚠️ A fourth persisted-layout path added later could still miss it.
**S-4 — a shell `applyLayout(next)` funnel every `layout =` assignment routes through.** ① Correct, and correct by construction for any future path. ③ Touches **nine** assignment sites (`:484 :495 :506 :525 :535 :558 :563 :586 :709` + `:895`), seven of which start from a tree that already contains the home and need nothing. **A structural change to the shell inside a leg whose job is one widget's placement.**

📌 **Chat's recommendation: S-3**, with **S-4 named and NOT taken** — it is the structurally safer shape and it is an architecture change, which is Joe's, not a rider on E-2. 🔓 **If Joe wants S-4 it should be its own milestone**, and E-2's helper is the seam it would absorb.

### 🔒 ③ — **CLOSED 2026-08-13: P-1, NO PERSIST. PROVENANCE DELEGATED** (`D-141`). A read path stays a reader (`N-107`); the first fold/resize/move persists the tree with the home in it anyway.

### 🔓 ③ — DOES THE RE-INJECT PERSIST?

**P-1 — NO persist.** The rule runs on every read path, so the disk may stay pre-`dm-spaces` indefinitely and the home still appears every launch. ① **Nothing visible differs.** ③ Zero. ✅ Keeps a **read** path a reader (`N-107`'s two-writers discipline), and matches `handleRevertUi`'s own shipped decision not to write back what it just read.
**P-2 — persist once after the boot re-inject.** ① Nothing visible differs. ③ One line. 🛑 **Writes Joe's disk at boot with no user action**, and turns the load path into a writer.

📌 **Chat's recommendation: P-1.** The first fold, resize or move persists the tree with the home in it anyway (`:485 :496 :507`), so P-2 buys a file change nobody sees.

### 🔒 ④ — **CLOSED 2026-08-13: V-a, DRIVE IT. PROVENANCE DELEGATED** (`D-141`). 🛑 **THE DELEGATION RULES THE DESIGN, NOT THE SIDE EFFECT.** V-a's own wording is *"with Joe's consent asked before the session that runs it"*, so **the consent to write a named UI state to Joe's disk is STILL OWED at `E-2b` and must be asked then** — a permanent side effect is never covered by a ruling about which option to take.

### 🔓 ④ — THE `handleUistateLoad` VERIFY LEG SPENDS SOMETHING OF JOE'S. HIS CALL.

Proving `F1`'s path requires **saving a named UI state on Joe's client** (a real file write; `N-115`: +4 registry while it exists), loading it, then deleting it.

**V-a — drive it with Joe's explicit consent**, save → load → delete → show exact return to baseline (`N-123`: the cleanup is part of the probe; Delete is two-step and shipped since J-511). ① None once cleaned up. ③ One extra verify leg.
**V-b — skip it and record the path as reasoned-but-undriven.** ③ Zero. 🛑 **The leg's headline finding would ship unexercised** — and *"unreachable today"* has been the wrong argument five times in this project (`N-091` · `N-097` · `N-099` · `N-109` · `N-116`).

📌 **Chat's recommendation: V-a, with Joe's consent asked before the session that runs it.** ⚠️ **A probe must be able to demonstrate SUCCESS, not merely the absence of failure (`N-194`)** — so the pass condition is *the named state loads AND `dm-spaces` is in the resolved tree*, positively controlled by reading the eight-leaf tree **out of the saved state file first**.

---

## §5 — PROPOSED SUB-LEGS

| leg | what | floor | gated on |
|---|---|---|---|
| **E-2a** | the placement table + the helper + `loadLayout`'s single exit + the `:895` call site (§4 ①②③ as ruled) | `svelte-check` **0/34/15** | ① ② ③ ruled |
| **E-2b** | drive it: boot · `layout.revert` · named-state load (§4④) · idempotency (load twice, one leaf) · the mounted-set skip | `svelte-check` | E-2a |
| **E-2c** | records — this Phase-0 → COMPLETED, runbook → COMPLETED, JOURNAL, `CLAUDE.md` PLAY, ROADMAP (`D-074` atomic) | — | E-2b |

🔒 **E-2's runbook GETS A CLAIR ADVERSARIAL READ BEFORE JOE LOCKS IT.** E-1 was locked without one and shipped a `§4` command that could not run (`insertLeaf` is not on `window`). **Recorded as a recommendation, not a demand — it is Joe's to decline again, and the record will show it was offered.**

📌 **No send is proposed. Nothing here mints a DM.** The only side effect anywhere in the leg is §4④'s named UI state, and it is consent-gated and cleaned up.

---

## §6 — NOT TOUCHED

The wire · any protocol event · `xgen-core` · `xgen-node` · `xgen-common` · any `.rs` · **`skin.css` (Joe's file, never folded into a Chat or Clair commit)** · `ui/core/**` (would move the catalogue) · `dm-spaces.svelte` (E-1, COMPLETED) · `spaces-panel.svelte` (**E-3**) · `DEFAULT_LAYOUT` (🔒 stays at eight leaves — ①-B) · `M-RP-WIDGET-SUSPEND`'s hidden-set guard (`F5` — owed there, `N-182`).

⚠️ **`M-RP-INTRO`'s trigger fired at J-716 and it still has no Phase-0.** The oldest outstanding item. Flagged, not started — it is not E-2's.

---

## §7 — WHERE THIS DOCUMENT IS MOST LIKELY WRONG

1. 🛑 **NOTHING HERE WAS MEASURED LIVE.** Every finding is a read of the tree at `dccc9b1`. **`F7`'s Joe-tree half is the one that most needs driving** — the sibling-versus-wrap outcome in his arranged tree is a live fact, and E-1's own record is where it came from.
2. ⚠️ **`F1` is this document's whole reason for existing, and it is an argument from three call sites, not from a screen.** It should be attacked before it is built on: if `handleUistateLoad` is somehow unreachable, §4② collapses back to `S-1`.
3. **§4①'s `D-c` may be over-shaped for one row.** The honest counter is that a one-entry table is *fed*, and a nine-id loop with one placement is not.
4. **This document has not been read by anyone outside its author.** ⚠️ *Every real defect in this arc came from Clair executing a document or Joe looking at a screen; Chat's own re-reads passed every time — including the re-reads that carried `P2`'s narrow form into three records.*
