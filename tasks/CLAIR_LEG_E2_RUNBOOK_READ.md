# Clair — adversarial read of RUNBOOK_MEMBER_ACT_LEG_E2.md v1.0 (M-RP-MEMBER-ACT Leg E-2)
> **Status**: COMPLETED  
> Version: 1.0  
> Date: Aug 2026  
> **Last updated**: 2026-08-13  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — WHAT THIS IS

A read, not an implementation. No code written, no source edited, no app launched, no message sent, no DM minted, nothing re-annotated. Every `file:line` below came from a tool that printed it **in this session at HEAD `f604680`**, not from the runbook and not from memory.

🛑 **State at open, re-measured, not inherited:** `git status` **clean**; `git rev-parse HEAD` = **`f604680`**; `git ls-remote origin refs/heads/main` = **`f604680`** — identical, not the tracking ref.

🔑 **A fact that shapes the whole read:** `git diff --stat dccc9b1..HEAD` over the six source files the runbook touches or grounds against is **EMPTY**. The two commits after `dccc9b1` (`5eb698d`, `f604680`) are docs-only. **So the source at HEAD is byte-identical to the source the runbook grounded against.** Any pointer that does not match is therefore wrong *at the grounding commit itself* — not drift.

---

## §1 — VERDICT

**LOCKABLE WITH ONE NAMED CHANGE, plus wording.**

The design is sound and buildable inside the two-file scope. Every **edit-target** pointer is exact. The three call sites are the complete set of persisted-layout-producing paths. The core mechanism (`insertLeaf` WRAP + idempotency) is verified from source. The single risk the runbook itself flagged loudest — `mountedPlugins` reading empty at boot — is **foreclosed by construction**, stronger than the runbook claims.

- **E-2a (the BUILD)** is lockable as written. I could not break it.
- **E-2b (the VERIFY)** carries one procedural hole worth closing before the lock: **V4's positive control has no way to produce its own precondition**, and if Clair cannot construct an eight-leaf named state, V4 silently collapses into the rejected `V-b` — the leg's headline finding (`F1`) ships unexercised. That is the one thing I would fix in the document before Joe locks it.

Nothing here is "not lockable." The plan-mover is on the verify side and is a one-paragraph fix.

---

## §2 — PLAN-MOVING

### 🛑 PM-1 — V4 cannot create the state it needs to verify, and the natural path produces the wrong tree.

**V4's goal:** prove that loading a named UI state saved with an **eight-leaf** tree (no `dm-spaces`) still yields the home — the `:895` path, which is the whole reason E-2 exists as specified (`F1`, runbook §3.4 line 139).

**The setup is unspecified and the obvious route is closed.** Measured:

- Joe has **zero saved UI states** today (Phase-0 §3 F1 line 72). So there is no pre-existing eight-leaf state to read.
- `handleUistateSave(name)` snapshots the **current** `layout`: `uiStateStore.save(name, { layout: $state.snapshot(layout), ... })` (`app_client.svelte:887`).
- After E-2 ships, **boot always re-injects** the home, so `layout` at every moment after `:709` has **nine** leaves. Saving "the current layout" therefore produces a **nine**-leaf state — which already contains `dm-spaces` and cannot serve as V4's positive control.

⇒ **The only ways to put an eight-leaf named state on disk under the E-2 build are (a) `__XGEN_LAYOUT__.set(DEFAULT_LAYOUT)` then open the Save dialog and save, or (b) hand-edit the state file.** Route (a) works because `set(l) { layout = l; }` (`app_client.svelte:394`) is a bare reassignment that neither re-injects nor persists (confirmed at `:394`; the runbook §4 line 153 says as much), so the saved snapshot is the eight-leaf `DEFAULT_LAYOUT`. **The runbook names neither route.**

**Why this is plan-moving and not wording:** the runbook's own §5 makes the positive control the point — *"'the home appeared' proves nothing unless the saved tree is first shown to lack it."* The positive control protects against a *false pass*, but it does not tell Clair how to reach a *true* setup. If Clair cannot produce the eight-leaf state, the honest outcome is to record V4 as undriven — i.e. exactly `V-b`, which §4④ rejected on the ground that *"unreachable today" has been the wrong argument five times in this project*. So an unspecified setup can quietly convert the ruled `V-a` into the rejected `V-b` without anyone deciding to.

**Fix (verify-side only, no build change):** add the setup step to V4 — `set(DEFAULT_LAYOUT)` → Save dialog → name it → read the state file, show eight leaves and no `dm-spaces` (the positive control) → Load → home present → Delete → baseline shown. `set()` is confirmed non-re-injecting and non-persisting, so it is the honest way to stage the eight-leaf tree without touching disk before the save.

📌 This does not touch the `V4` consent question (§0 point 2), which is separately owed at E-2b and remains Joe's.

---

## §3 — WORDING / MINOR

### ✂️ W-1 — §7 item 1 UNDER-states its own safety; the "silently never appears" failure is foreclosed, not merely "should read fine."

§7 item 1 (the item the kickoff flagged as most deserving my eyes) reads the `mountedPlugins`-at-boot risk as *"reasoned, not driven … if it reads empty at boot, the guard skips dm-spaces and the home silently never appears."*

Measured at the store source: `installed.mounted` (`installed.svelte.ts:63-68`) returns `[...CLIENT_PLUGINS, ...AVAILABLE_CUSTOM.filter(installed && !disabled)]` — **`CLIENT_PLUGINS` is spread unconditionally**, and `hydrate`/`hydrateDisabled` touch only `_installed`/`_disabled`, i.e. the *custom* filter. `dm-spaces` is a `kind: 'system'` row in `CLIENT_PLUGINS` (`registry.ts:214-224`, inside the `CLIENT_PLUGINS` block at `:109`, before `AVAILABLE_CUSTOM` at `:304`). ⇒ **`installed.mounted` can never be empty of `dm-spaces`**, hydrate or no hydrate, at `:709` or anywhere else.

So the guard in `reinjectSystemRegions` (`plugins.filter(p => p.surface === 'region' && p.regionId)`) always finds `dm-spaces` today. The failure mode §7 item 1 fears is not "reasoned safe" — it is **structurally impossible for a system region**. The runbook's §3.4 line 141 "seeded after hydrate" reassurance is therefore *technically irrelevant* to whether `dm-spaces` re-injects (it matters only for future *custom* rows, and system rows are not customs). **Recommendation:** keep V1 (drive the whole chain), but restate §7 item 1 as "foreclosed by the unconditional `CLIENT_PLUGINS` spread," not "should read fine but undriven." The current wording asks Clair to hunt for a failure that cannot occur.

### ✂️ W-2 — the §3.3 code block DROPS the inline N-095/D-115 comment while the prose says preserve it.

§3.3's ⚠️ note says *"Keep the existing JSDoc block and extend it; do not delete the N-095/D-115 reasoning."* Correct intent. But the **shipped `loadLayout` carries a second N-095/D-115 comment INSIDE the function body** (`layout-default.ts:144-146`: *"`migrateLayout` subsumes the old `isValidLayout` guard … NEVER returns null (N-095 … D-115). DEFAULT_LAYOUT is injected because `core` must not own a default"*), and the §3.3 replacement code block does **not** carry it — it has only `if (persisted) loaded = migrateLayout(persisted, DEFAULT_LAYOUT);`.

A faithful implementer copying the §3.3 block verbatim keeps the JSDoc (as instructed) but loses the **inline** reasoning, because the instruction names only the JSDoc. **Recommendation:** either re-place the inline N-095 comment in the §3.3 code block, or widen the ⚠️ note to *"and the inline `migrateLayout`/N-095 comment inside the body."* Small, but it is the exact "extend, do not replace" discipline the runbook itself invokes (M-RP7.1b Rule-6 lineage).

### ✂️ W-3 — the document polices W1, and its own §1 grounding table carries a W1-class pointer.

§1 opens *"Every `file:line` below came from a tool that printed it,"* and its whole framing is W1-immunity. Yet:

| runbook cites | actual (tool-printed at HEAD) | note |
|---|---|---|
| `DEFAULT_LAYOUT … layout-default.ts:105-125` (§1 line 42; Phase-0 §2 line 57) | **`103-123`** (`export const DEFAULT_LAYOUT` at 103, `};` at 123) | off by **+2**, and **not drift** — no source moved `dccc9b1..HEAD`; wrong at the grounding commit. Same-file `:14` and `:137` are exact, so this is a mis-measure, not a uniform shift. |
| hydrate/hydrateDisabled `:696`/`:700` (§3.4 line 141) | **`:695`** (`installed.hydrate`) / **`:699`** (`installed.hydrateDisabled`) | off by **+1**; the cited `:696` lands on a comment, `:700` on `locked = …`. |
| DEV bridge `:392-402` (§4 line 153) | literal closes at **`:404`** (`get background()` at `:402`, `setBackground` at `:403`, `};` at `:404`) | range truncates `setBackground`; the prose enumeration lists all six correctly. |
| `dm-spaces.svelte:33` (§1 line 44; implied `ui/client/src/`) | file is at **`ui/common/lib/components/widgets/dm-spaces.svelte`**; line **33** is correct (`id = region-${regionId}` → `dm-spaces#region-dm-spaces`). | wrong implied path; out of E-2 scope (context-only). |

**None of these is an edit target.** Every pointer Clair will actually edit against — `:12`, `:586`, `:709`, `:895`, the persist refs `:485/:496/:507`, `mutate.ts:266`, `mutate.ts:161`, `types.ts:26`, `layout-default.ts:14`/`:137`, `resolve.ts:161` (the short-circuit line) — is **exact**. The imprecise ones are all context/reassurance. So this is genuinely minor. It is flagged because the document asserts immunity from precisely this species, and §1 line 46's own rule (*"ENUMERATE, NEVER DERIVE"*) is the same discipline the `105-125` slip broke.

---

## §4 — CONFIRMED SOUND (what I attacked and could not break)

- **`insertLeaf` WRAP under `DEFAULT_LAYOUT` → `[spaces, dm-spaces]` (col split, old spaces slot).** Verified from source: `spaces` is `children[0]` of the root `row` split; `insertLeaf('dm-spaces','spaces','bottom')` → axis `col` ≠ `row` → `insertBeside` WRAP branch (`mutate.ts:313-320`) → new `col` split `[spaces, dm-spaces]` sizes `[1,1]` in the old slot. **Matches runbook §3.1 exactly.** The Joe-tree SIBLING half is correctly marked a live measurement (`V2`), not asserted.
- **The WRAP does NOT strand the home at "half of R1's width."** It docks `dm-spaces` in the **bottom half of the narrow R1 column** (a vertical split of the weight-1 left slot) — the Discord-shape default. Appearance is Joe's (§3.5); no mechanic forces a bad default.
- **`insertLeaf` idempotency + preserve-user-placement.** `mutate.ts:269` returns by reference when `newWidgetId` is already docked ⇒ running the re-inject on every load can never double-place, and a user who moved `dm-spaces` keeps their placement. Confirms §3.2's "free to run on EVERY load."
- **§3.2's DEV-warn omission is correct.** `insertLeaf` returns `layout` by reference for **both** target-missing (`mutate.ts:268`) and already-docked (`:269`); they are indistinguishable from the return, and `findLeaf` is **unexported** — `core` exposes no leaf-presence predicate. The claim in §3.2 line 103 holds against `mutate.ts:266`.
- **Three-site completeness (`:586`, `:709`, `:895`).** Grep across all of `ui/` for `loadLayout`/`get_ui_state`/`migrateLayout`: `loadLayout` is imported only in `app_client.svelte` and defined in `layout-default.ts`; the only production `layout =` assignments from persisted data / default are `:586`, `:709`, `:895`. The DEV `set()` (`:394`) and the eight tree-mutation sites (`:484/:495/:506/:525/:535/:558/:563`) are correctly excluded — the latter operate on an already-resolved tree that already contains the home. The `uistate.svelte.ts` `get_ui_state` reads feed the **store**, not `layout`. **The two-file scope is sufficient.**
- **P-1 (no accidental persist) holds by construction.** `handleRevertUi` (`:585-587`) is `layout = await loadLayout();` with **no** `setSessionLayout` (comment `:583-584` says so deliberately). Persistence lives only in explicit gesture handlers (`:485/:496/:507` fold/resize/move; install/enable etc.), never in `loadLayout`/`reinject`, and there is **no `$effect` auto-persisting `layout`**. Nothing fires at boot before the user acts. §3.4's `:485/:496/:507` persist pointers are exact.
- **The mounted-set guard is the correct STRICT test, and its negative branch is UNEXERCISED today.** It requires a `surface: 'region'` plugin, not a `buildWidgetRegistry` key (which maps every `REGION_IDS` entry to `RegionPlaceholder`). Since `dm-spaces` is always in `CLIENT_PLUGINS`, the "skip because not mounted" branch cannot fire this leg — say UNEXERCISED, do not claim it verified (runbook §7 item 2 is right).
- **`D-c` (one-row table) is not over-shaped.** A one-entry `SYSTEM_REGION_PLACEMENT` is *fed* (`N-091`); a nine-id `REGION_IDS` loop carries one placement and eight unplaceable branches; a `CLIENT_PLUGINS`-system loop carries eight and would silently re-inject any future system region at `dm-spaces`'s target. The table is the only domain that carries the target+edge neither id list has. I agree with §3.1 / §4①.
- **svelte-check floor holds (reasoned).** New imports are all available: `insertLeaf` (`mutate.ts:266`), `Edge` (`mutate.ts:161`), `RegionId` (`types.ts:26`); `PluginDescriptor` (`:14`) and `Layout` (`:10`) already imported. The helper's `p.regionId as string` reuses the exact type-checking pattern already shipped in `buildTitles` (`layout-default.ts:57`) against `regionId?: string` (`registry.ts:82`). No new type surface reaches the plain-JS call sites. V8 drives it.
- **Return-by-reference safety of `DEFAULT_LAYOUT`.** `insertLeaf` is immutable (`{ ...layout, root: … }`), so the module const is never mutated. §3.3's ✅ bullet holds.
- **The §3.3 single-exit rewrite is behaviourally equivalent + reinject.** The try/catch, the N-095 guard, and the DEFAULT fallback are preserved; both former returns route through `reinjectSystemRegions`. `migrateLayout(raw, fallback)` two-arg call matches the signature (`resolve.ts:156`).
- **E-2 does NOT copy E-1's §4 mistake.** V1–V7 read via `__XGEN_LAYOUT__.current` (which exists); V4 drives the real Load dialog, not a non-existent bridge method. §4 explicitly warns against `insertLeaf`-on-`window`.

---

## §5 — WHERE MY OWN READ IS MOST LIKELY WRONG

1. **I did not launch the apps (instructed not to).** `V1`/`V2` are live facts. I verified the `DEFAULT_LAYOUT` WRAP outcome **from source**, but the **SIBLING outcome in Joe's arranged tree** I could not check — his state file is on disk, apps down. I take the runbook's flag on `V2` (measure, do not assert) at face value; if his live parent split is not `col`, the shape differs and only a drive shows it.
2. **Svelte 5 `$derived` read-timing.** I reasoned that reading `mountedPlugins` inside `onMount` at `:709` returns the current `installed.mounted`, and that since `installed.mounted` unconditionally includes `CLIENT_PLUGINS` even an "uninitialised" read recomputes on access. I am confident but did **not execute** it. If Svelte's rune init had an ordering quirk I do not know about, W-1's "foreclosed" claim would weaken back toward the runbook's original "drive it." This is the one place my confidence rests on framework semantics rather than a printed value — so V1 is still worth driving, which is the runbook's own position.
3. **PM-1's proposed setup route.** `set(DEFAULT_LAYOUT)` → Save is reasoned from the confirmed non-re-injecting, non-persisting `set()` (`:394`) and the confirmed snapshot-current-layout save (`:887`). I did not drive the sequence. If the Save dialog does anything other than snapshot the live `layout` at trigger time, my proposed setup would need adjustment — but the finding (V4 has no stated way to reach its precondition) stands regardless of which route fixes it.
4. **The `PluginDescriptor.regionId` cast.** I confirmed `regionId?: string` (`registry.ts:82`) and that the pattern already type-checks in `buildTitles`, so I did not exhaustively type-check the new helper. If a stricter `tsconfig` path applied to new code differently, a `noUncheckedIndexedAccess`-style surprise on `Object.entries` is theoretically possible; I judge it very low because the reused pattern already compiles at the floor.

---

## §6 — SUMMARY FOR THE LOCK

- **Build (E-2a):** lockable as written.
- **Verify (E-2b):** close **PM-1** (V4's eight-leaf-state setup) before the lock, or the leg's headline finding ships undriven.
- **Wording:** **W-1** (restate §7 item 1 as foreclosed), **W-2** (keep the inline N-095 comment in the §3.3 block), **W-3** (the four context-pointer slips, all non-edit-target).
- **Everything else attacked survived.** The two-file scope is sufficient; the three sites are complete; the mechanism is verified from source; P-1 holds by construction.

Chat revises; Joe locks. I did not fix the runbook.
