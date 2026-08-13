# M-RP-MEMBER-ACT — Leg E-2 Runbook: the system-region re-inject (Clair)
> **Status**: PENDING  
> Version: 1.1  
> Date: Aug 2026  
> **Last updated**: 2026-08-13  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — SEAT, AND WHAT THIS LEG IS

🛑 **NOT LOCKED. DO NOT IMPLEMENT YET.** ✅ **CLAIR'S ADVERSARIAL READ RAN 2026-08-13** — brief `tasks/CLAIR_LEG_E2_RUNBOOK_READ.md`, verdict **LOCKABLE WITH ONE NAMED CHANGE + wording**; **`PM-1` and `W-1`–`W-3` are swept into this v1.1 and every one of her corrections was re-measured by Chat (Rule 5).** One thing still owed before Joe locks:

🔓 **JOE'S CONSENT FOR THE `V4` SIDE EFFECT** (§5) — that leg writes a named UI state to his disk. Ruled `V-a` under `D-141`, **but a delegation rules the DESIGN, not the SIDE EFFECT.**

**You are CLAIR — Code Claude.** You implement from this runbook **once Joe locks it**. **You never push.** Deviations are **reported under Rule 6, never absorbed** — *an implementer who silently absorbs a bad instruction ships the architect's mistake* (M-RP7.1b: your `migrateLayout(raw, fallback)` deviation stopped `core` importing a shell constant — **the same seam this leg edits**).

**E-2 places the `dm-spaces` leaf. It does NOT build the R1 filter** — that is **E-3** and must not appear in this diff. Until E-3 lands a DM Space renders in **both** R1 and the DM panel. That is expected, temporary, and not a defect.

**Read first:** `tasks/M_RP_MEMBER_ACT_LEG_E2_PHASE0.md` **v1.1** (§2 audit · §3 F1–F7 · §4 the four rulings) · `tasks/M_RP_MEMBER_ACT_LEG_E_PHASE0.md` v1.5 §4① · `CLAUDE.md` PLAY head · `JOURNAL.md` J-721.

---

## §1 — GROUNDING (measured 2026-08-13 at `dccc9b1`; re-verify, do not inherit)

📌 **Every `file:line` below came from a tool that printed it.** Leg E Phase-0 v1.0 estimated seven pointers and all seven were wrong (`F11`/`W1`). **If a pointer does not match, that is a Rule 6 report, not a silent adjustment.**

| fact | where |
|---|---|
| `loadLayout` — **no parameters**, TWO `return`s | `ui/client/src/layout-default.ts:137` |
| caller ① boot (after `installed.hydrate`/`hydrateDisabled`) | `app_client.svelte:709` |
| caller ② `handleRevertUi` (fn `:585`) | `app_client.svelte:586` |
| 🛑 **entry point ③ — bypasses `loadLayout`** (fn `:889`) | `app_client.svelte:895` |
| the shell's mounted plugin list | `app_client.svelte:102` — `const mountedPlugins = $derived(installed.mounted)` |
| `insertLeaf(layout, newWidgetId, targetId, edge)` — pure, TOTAL, **idempotent** | `mutate.ts:266` |
| `Edge = 'top' \| 'bottom' \| 'left' \| 'right'` | `mutate.ts:161` |
| `RegionId = string` (`types.ts:26`) · `Layout {version, root}` | `ui/core/lib/components/layout/types.ts` |
| `PluginDescriptor` already imported as a TYPE in `layout-default.ts` | `layout-default.ts:14` |
| `DEFAULT_LAYOUT` — **8 leaves**, `spaces` a direct child of the ROOT `row` split | `layout-default.ts:103-123` |
| the `dm-spaces` descriptor (`system` · `region` · `regionId: 'dm-spaces'`) | `registry.ts:214-224` |
| the widget's register id | `ui/common/lib/components/widgets/dm-spaces.svelte:33` → `dm-spaces#region-dm-spaces` |

✅ **NO SOURCE DRIFT: `git diff dccc9b1..HEAD` over all six files above is EMPTY** (both later commits are docs-only), so a pointer that does not match is **wrong at the grounding commit, not stale**. 📌 **Four `§1` pointers WERE wrong at v1.0 and are corrected here** (`DEFAULT_LAYOUT` · the hydrate seeds · the DEV-bridge bounds · `dm-spaces.svelte`'s path), all found by Clair's read and re-measured by Chat. **Every EDIT-TARGET pointer was exact.** *The document that polices `W1` carried four instances of it — which is the argument for the read, not against the rule.*

🛑 **NO REGISTRY BASELINE IS CARRIED.** `N-184` Space-dependent · `N-190` draft-dependent · **`N-194`: 168 → 174 on an IDENTICAL screen, and the cause was `CLIENT_PLUGINS` going 10 → 11 rows, not DM rows.** **Record the screen, or record no number. ENUMERATE, NEVER DERIVE.**

---

## §2 — SCOPE: THE FILES YOU MAY OPEN

1. `ui/client/src/layout-default.ts` — the placement table, the helper, `loadLayout`'s signature + single exit
2. `ui/client/src/app_client.svelte` — the import line, the two `loadLayout(...)` call sites, the `:895` call site

🛑 **NOT in this leg, and each has an owner:** `spaces-panel.svelte` (**E-3**) · `dm-spaces.svelte` (**E-1, COMPLETED**) · `DEFAULT_LAYOUT`'s tree (🔒 **stays at EIGHT LEAVES** — Joe's ①-B: re-inject is the ONLY placement path, so a fresh tree and a re-injected tree cannot drift) · `ui/core/**` (🔒 **no new algebra — `insertLeaf` already does the work**; opening `core` would move the catalogue) · `skin.css` (**Joe's file, never in a Clair commit**) · any `.rs` (🔒 **`K2` shipped in Leg B; the cargo floor does NOT return — §6**) · the `applyLayout` funnel (**S-4, named and NOT taken — Joe's, its own milestone**) · the `M-RP-WIDGET-SUSPEND` hidden-set guard (**owed THERE, `N-182` — E-2 ships unconditional**).

⚠️ **If E-2 cannot be built inside these two files, STOP AND REPORT.** A scope that has to grow is a finding; a scope that grows silently is a defect.

---

## §3 — WHAT TO BUILD

### 3.1 The placement table (`layout-default.ts`)

🔒 **THE TABLE IS THE RULE'S DOMAIN** (`D-c`, ruled). The re-inject iterates the **table**, not `REGION_IDS` and not the plugin list — because **neither id list carries a target or an edge**, and a rule whose domain is a list it cannot place has an unfed branch (`N-091`).

```ts
export interface RegionPlacement { target: RegionId; edge: Edge }

/** D-114 §9's re-inject rule — where a system region docks when a loaded layout does not contain it.
 *  THE TABLE IS THE DOMAIN: a system region is re-injectable iff it has a row here. Every future system
 *  region gets the rule free by adding one. */
export const SYSTEM_REGION_PLACEMENT: Record<RegionId, RegionPlacement> = {
  'dm-spaces': { target: 'spaces', edge: 'bottom' },
};
```

🔒 **`target: 'spaces'`, `edge: 'bottom'` — Joe's, and verified in BOTH trees.** Under `DEFAULT_LAYOUT` `spaces` is a direct child of the **root `row`** split, so `bottom` (axis `col`) takes `insertBeside`'s **WRAP** branch → `[spaces, dm-spaces]`. In Joe's arranged tree `spaces`'s parent already runs `col`, so it is a **SIBLING** insert → `[spaces, dm-spaces, self]`. ⚠️ **The second half is a LIVE measurement from E-1's verify, not derivable from source — `V2` re-measures it; do not assert it.**

New imports required: `insertLeaf` and the `Edge` type from `$core/components/layout/mutate`, `RegionId` from `$core/components/layout/types`.

### 3.2 The helper (`layout-default.ts`)

```ts
/** Re-inject every placement-declaring system region the layout is missing (D-114 §9). Pure, TOTAL,
 *  IDEMPOTENT: `insertLeaf` no-ops by reference on an already-docked id and on a missing target, so this
 *  is free to run on EVERY load and can never double-place or fight a user who moved the region. */
export function reinjectSystemRegions(layout: Layout, plugins: PluginDescriptor[]): Layout {
  const mountedRegionIds = new Set(
    plugins.filter((p) => p.surface === 'region' && p.regionId).map((p) => p.regionId as string),
  );
  let out = layout;
  for (const [regionId, place] of Object.entries(SYSTEM_REGION_PLACEMENT)) {
    if (!mountedRegionIds.has(regionId)) continue; // no mounted widget → no leaf (never inject a W-13 drop)
    out = insertLeaf(out, regionId, place.target, place.edge);
  }
  return out;
}
```

🔒 **THE MOUNTED-SET GUARD IS THE STRICT TEST, AND SAY SO HONESTLY.** It requires a **`surface: 'region'` plugin** for the id, **not** mere membership in the widget registry — `buildWidgetRegistry` maps every `REGION_IDS` entry to `RegionPlaceholder`, so a registry test would pass for an id with no widget and inject a leaf that paints a placeholder. *The stricter test guarantees a real widget; the weaker one only guarantees no crash.*

📌 **NO DEV WARN ON A MISSING TARGET, DELIBERATELY.** `insertLeaf` returns **by reference** for both *already docked* (the normal case) and *target missing*, so the two cannot be told apart without a leaf-presence predicate — and `core` exports none. Adding a second walk in the shell would duplicate `findLeaf` (`D-067`). **Unreachable today** (`spaces` is a non-removable system region); **filed here, not built** (`N-182` — reserve nothing).

### 3.3 `loadLayout` gains a parameter and a SINGLE EXIT (`layout-default.ts:137`)

🛑 **BOTH `return`s MUST ROUTE THROUGH THE RE-INJECT.** The fallback returns `DEFAULT_LAYOUT`, which has **eight leaves and no `dm-spaces`** (①-B) ⇒ **a fresh client with no store would show no DM home at all** if only the persisted branch were wrapped. *A re-inject on one of two returns is a home that appears or not depending on whether a file exists.*

```ts
export async function loadLayout(plugins: PluginDescriptor[]): Promise<Layout> {
  let loaded: Layout = DEFAULT_LAYOUT;
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    const raw = await invoke<string>('get_ui_state');
    if (raw && raw.trim()) {
      const persisted = JSON.parse(raw)?.session?.layout;
      // `migrateLayout` subsumes the old `isValidLayout` guard AND upgrades a v1/v2 boolean-`collapsed` tree
      // to v3 (M-RP7.1b). It NEVER returns null (N-095 — a malformed/older layout falls back to DEFAULT, so
      // the centre never blanks; D-115). DEFAULT_LAYOUT is injected because `core` must not own a default.
      if (persisted) loaded = migrateLayout(persisted, DEFAULT_LAYOUT);
    }
  } catch (_) {
    // no-Tauri OR corrupt store → DEFAULT (N-095). A read/parse error must never blank the centre.
  }
  // D-114 §9 — the re-inject wraps BOTH former returns (F2). Idempotent + TOTAL, so it cannot blank or double-place.
  return reinjectSystemRegions(loaded, plugins);
}
```

✅ **`DEFAULT_LAYOUT` by reference stays safe** — `insertLeaf` is immutable (`{ ...layout, root: … }`), so the module-level const is never mutated. Only its object identity stops being shared, and nothing reads that.
✅ **The N-095 contract is preserved** — `migrateLayout` never returns null and `insertLeaf` is TOTAL, so `loadLayout` still cannot return null and the centre cannot blank (`D-115`, the J-499 30→21 failure).
⚠️ **PRESERVE BOTH COMMENT LAYERS, DO NOT REPLACE THEM** (`W-2`): the **JSDoc block above** the function AND the **inline N-095/D-115 comment at `layout-default.ts:144-146`**, which the v1.0 code block silently dropped while its own prose said to keep it. *A rewrite that deletes the reasoning for a guard is how the next reader deletes the guard.*

### 3.4 The three call sites (`app_client.svelte`)

| site | becomes |
|---|---|
| `:12` import | add `reinjectSystemRegions` to the existing `./layout-default` named import |
| `:709` boot | `layout = await loadLayout(mountedPlugins);` |
| `:586` `handleRevertUi` | `layout = await loadLayout(mountedPlugins);` |
| 🛑 `:895` | `if (s?.layout) layout = reinjectSystemRegions(migrateLayout(s.layout, DEFAULT_LAYOUT), mountedPlugins);` |

🔒 **`:895` IS THE WHOLE REASON THIS LEG EXISTS AS SPECIFIED.** `handleUistateLoad` assigns a persisted layout **without calling `loadLayout`**, so an inside-`loadLayout`-only hook lets a named UI state saved before `dm-spaces` existed **remove the DM home from the running app** — and with it the self thread's only GUI door (`F5`). *M-RP7.1b drove that dialog live; it is three clicks, not a theory.*

✅ **`mountedPlugins` is in scope at all three sites, and its value at boot is FORECLOSED BY CONSTRUCTION, not merely reasoned** (`W-1`): `installed.mounted` (`installed.svelte.ts:63-68`) spreads `[...CLIENT_PLUGINS]` **unconditionally**, `dm-spaces` is a `kind: 'system'` row **in** `CLIENT_PLUGINS` (`registry.ts:214-224`), and `hydrate`/`hydrateDisabled` (`:695`/`:699`) touch **only the custom sets**. ⇒ **the guard cannot see an empty set at boot.** 📌 **Still drive `V1`** — foreclosed is an argument, and the argument is what `N-116` says can be self-consistent while the code is not.
🛑 **DO NOT PERSIST after the re-inject** (`P-1`, ruled). A read path stays a reader (`N-107`); the first fold/resize/move persists the tree with the home in it anyway (`:485`/`:496`/`:507`).

### 3.5 Appearance

🔓 **Joe's, entirely.** **No `skin.css` edit, no component `<style>` block** (`N-090`/`N-025`). The tile's default position, size and the split weights the wrap produces are his. If the result wants a rule that does not exist, **name it in the hand-back**.

---

## §4 — TYPE + BUILD NOTES

- `layout-default.ts` is **TS**; `app_client.svelte` is **plain JS** (`<script>`, no `lang="ts"`), so the call sites are untyped and `svelte-check` sees the contract only at the `layout-default.ts` side. **That is the floor that must not move.**
- 🛑 **`__XGEN_LAYOUT__` EXPOSES ONLY `current` · `set` · `move` · `fold` · `background` · `setBackground`** (`app_client.svelte:392-404`). **`insertLeaf`, `removeRegion` AND `DEFAULT_LAYOUT` ARE ALL UNREACHABLE FROM A CDP EVAL** — they are module imports, not window properties. **E-1's runbook §4 shipped `insertLeaf(...)` as a verify command and it could not run; do not repeat the shape with any of the three.** `set(l) { layout = l; }` (`:394`) is a bare in-memory reassignment — **it does NOT persist and does NOT re-inject**; `move`/`fold` delegate to the shell handlers and **DO** persist.

---

## §5 — VERIFY (drive it; do not predict it)

🛑 **Baseline and result measured in ONE sitting, on ONE client, SAME Space tree and draft state. Record the screen, or record no number.**

| # | check | how |
|---|---|---|
| **V1** | **boot** places the home | fresh launch → `__XGEN_LAYOUT__.current` contains a `dm-spaces` leaf **and** `dm-spaces#region-dm-spaces` is registered |
| **V2** | the **shape** of the insert in Joe's real tree | print the parent split's `dir` and `children` order — **measure sibling-vs-wrap, do not assert `F7`'s live half** |
| **V3** | **`layout.revert`** keeps it | `__XGEN_DEBUG__`/command → `layout.revert` → home still present |
| **V4** | 🛑 **the `:895` path — SEE §5.1, IT NEEDS STAGING** | §5.1 |
| **V5** | **idempotency** | run the load path **twice**; exactly **one** `dm-spaces` leaf, and `leafCount` identical |
| **V6** | the **P-1** rule holds | after V1/V3 with no grid gesture, `session.layout` on disk is **unchanged** — a read path wrote nothing |
| **V7** | registry transition | before → after, **ENUMERATED not derived**; **state the screen** (`N-194`) |
| **V8** | floors | `svelte-check` **0/34/15** · catalogue **435 BY SCOPE** (zero `ui/core`) |

🔑 **`N-194`, BINDING ON EVERY PROBE ABOVE: A PROBE MUST BE ABLE TO DEMONSTRATE SUCCESS, NOT MERELY THE ABSENCE OF FAILURE.** Before reporting any gate FAILED, ask: **what would this read return if the code were RIGHT?** Same answer ⇒ **the probe is wrong.** *Two E-1 probes read the wrong element and the wrong getter and would each have filed a false defect against correct code.*
🛑 **V4's control is the point** — *"the home appeared"* proves nothing unless the saved tree is first shown to lack it. **§5.1 is how.**

### 5.1 — V4 IN FULL: THE `:895` PATH, AND WHY IT NEEDS STAGING (`PM-1`, Clair)

🛑 **V4 AS WRITTEN AT v1.0 WOULD HAVE PASSED VACUOUSLY, AND THAT IS WORSE THAN FAILING.** Joe has **zero** saved states, so the only way to get one is to save. `handleUistateSave` (`app_client.svelte:880`, `uiStateStore.save` at `:887`) snapshots the **live `layout`** — which after boot **always contains the re-injected home**. ⇒ a state saved the ordinary way holds **nine** leaves, loading it shows the home, and the probe reads **PASS whether or not the `:895` call site exists at all.**

🔑 **A probe that cannot fail would have certified this leg's entire reason for existing without testing it** — `N-194`'s rule from the other side, in the document that quotes `N-194`. **Found by Clair's read; re-measured by Chat.**

🔓 **JOE'S CONSENT IS REQUIRED BEFORE STEP 2** — step 2 onward writes a real file to his disk.

| step | do | assert |
|---|---|---|
| **1** | read `__XGEN_LAYOUT__.current` and **hand-splice the `dm-spaces` leaf out** — replace the wrap split that holds `[spaces, dm-spaces]` with the bare `spaces` leaf (a sibling insert instead drops one entry from `children` **and** its `sizes` entry, same index) | the spliced object has **8 leaves, no `dm-spaces`** |
| **2** | `__XGEN_LAYOUT__.set(<spliced>)` | the grid repaints **without** the DM home; **nothing persisted** (`set` is a bare reassignment, `:394`) |
| **3** | drive the **real Save dialog** (diskette face → name it `e2-control` → Save) | — |
| **4** | 🔑 **READ `xgen-client_uistate.json` FROM DISK** | `named.<id>.layout` has **8 leaves and NO `dm-spaces`** — **THIS IS THE POSITIVE CONTROL, and it is what makes the rest of V4 mean anything** |
| **5** | drive the **real Load dialog** (load face → pick `e2-control` → Load) — this is the `:895` call site | the home is **present** in `__XGEN_LAYOUT__.current` **and** `dm-spaces#region-dm-spaces` is registered |
| **6** | **DELETE `e2-control`** (two-step: the button re-labels to *Confirm delete*) | on-disk `named` returns to **`{}`**; registry returns to the step-1 screen, **enumerated** (`N-115`: one saved state = **+4**) |
| **7** | reload the client | back at the boot baseline, **shown** (`N-123` — the cleanup is part of the probe) |

🛑 **`DEFAULT_LAYOUT` IS NOT A SHORTCUT FOR STEP 1.** It is a module import at `app_client.svelte:12`, **not on `window`** — `__XGEN_LAYOUT__.set(DEFAULT_LAYOUT)` cannot run in a CDP eval. *This is the E-1 `§4` shape exactly, and it was proposed once in the read that caught `PM-1`; a hand-splice of `current` is what E-1 actually used.*

⚠️ **If step 1 or 3 cannot be driven, STOP AND REPORT.** **Do NOT let V4 degrade into `V-b`** — that option was considered and rejected, and shipping `F1` undriven while the DoD says otherwise is the failure this whole section exists to prevent.

🛑 **`cargo` IS NOT A FLOOR AND MUST NOT BE CITED AS ONE.** Zero `.rs` in scope; an identical `cargo` result is a **scope argument, not a measurement** (`F8`).
🛑 **DO NOT SEND A MESSAGE.** A send mints a **permanent DM** in Joe's live client. Nothing in E-2 requires one.
📌 **Joe's client state is READ-ONLY to you** apart from V4's consented named state, **which you clean up and SHOW cleaned up.**
📌 **Any probe that persists a mutation OWES ITS CLEANUP, and the cleanup is part of the probe** (`N-123` — a leftover inline override once survived every edit Joe made and he reported it as a bug in his own CSS).

---

## §6 — DEFINITION OF DONE

- [ ] `SYSTEM_REGION_PLACEMENT` + `RegionPlacement` in `layout-default.ts`; **one row**, `dm-spaces → spaces/bottom`
- [ ] `reinjectSystemRegions(layout, plugins)` exported; **mounted-`surface:'region'` guard**, not a registry-key test
- [ ] `loadLayout(plugins)` has a **SINGLE EXIT** and both former returns route through the re-inject
- [ ] all **THREE** call sites updated — `:709` · `:586` · **`:895`**
- [ ] **`DEFAULT_LAYOUT` untouched at eight leaves**; **zero `ui/core`**; **zero `.rs`**; **zero `skin.css`**; no component `<style>`
- [ ] **no persist added** to any load path (`P-1`)
- [ ] **V1–V8 driven and recorded, each with its screen stated**
- [ ] **§5.1's SEVEN STEPS driven in order**, with **step 4's on-disk 8-leaf read shown** — without it V4 passes vacuously and proves nothing; the named state **deleted**, baseline shown
- [ ] `svelte-check` 0/34/15 · catalogue 435 by scope · **no `cargo` claim made**
- [ ] deviations reported (Rule 6)
- [ ] 🔓 hand-back names any skin rule the wrapped split wants, and the measured V2 tree shape

---

## §7 — WHERE THIS RUNBOOK IS MOST LIKELY WRONG

✅ **§7 AT v1.0 IS SUPERSEDED BY CLAIR'S READ ON ITEMS 1 AND 3** — corrected in place rather than deleted, so the record shows what the read moved.

1. ~~**The `mountedPlugins` read at `:709` is reasoned, not driven.**~~ 🛑 **OVERSTATED, AND `W-1` CORRECTS IT: THE EMPTY-SET CASE IS FORECLOSED BY CONSTRUCTION** — `installed.mounted` spreads `[...CLIENT_PLUGINS]` unconditionally (`installed.svelte.ts:63-68`) and hydrate touches only the custom sets. **Still drive `V1`**, but do not carry this as a live risk. *A runbook that flags a foreclosed risk as open trains its reader to skim the ones that are real — D-111's lesson at §7 scale.*
2. **The mounted-set guard's NEGATIVE branch is UNEXERCISED and will stay so.** `dm-spaces` is `kind: 'system'`, always in `CLIENT_PLUGINS`, never disableable ⇒ **the guard can only ever pass today.** It is defensive for future rows. **Say UNEXERCISED; do not claim it verified.**
3. ~~**§3.3 rewrites a function whose JSDoc carries N-095/D-115 reasoning.**~~ ✅ **`W-2` FOUND THE CODE BLOCK HAD ALREADY DROPPED THE INLINE COMMENT AT `:144-146` WHILE THE PROSE SAID TO KEEP IT.** Restored in v1.1. **Both layers are now in the block; preserve both.**
4. **V2 is the one gate whose expected value this document does not know.** The Joe-tree sibling outcome came from E-1's live verify. **Measure it; a number derived by arithmetic does not enter the record until it has been seen** (`N-194` ③).
5. 🛑 **§5.1's STAGING ROUTE IS ITSELF REASONED, NOT DRIVEN** — the hand-splice, `set()`'s non-persistence and the Save dialog's snapshot behaviour are read from source. **It is the newest text in this document and therefore the least tested.** If step 1 or 3 behaves differently, that is a Rule 6 report and V4 does **not** silently degrade.
6. **This runbook has now been read by TWO seats and its verify half was still wrong.** ⚠️ *`PM-1` sat on the verify side, which is where neither the author's re-read nor the first pass of an adversarial read tends to look — both were checking whether the BUILD was right. **The build survived; the proof of the build did not.***
