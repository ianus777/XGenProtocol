# M-RP-MEMBER-ACT — Leg E-3: the R1 filter — Phase-0
> **Status**: ACTIVE  
> Version: 1.2  
> Date: Aug 2026  
> **Last updated**: 2026-08-13  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — WHAT E-3 IS

**One sentence:** *DM Spaces leave R1 (OQ3 = A3, Joe's ruling at J-709) — and they leave the RENDER only, never the store.*

`E-1` gave them a home; `E-2` guarantees that home is placed on every path. **E-3 is the removal the other two exist to make safe.** After it, `E-5` closes the milestone.

**No code. No runbook. Phase 0 of `D-071`.**

---

## §1 — STATE AT OPEN, RE-MEASURED

| item | measured |
|---|---|
| tree | **CLEAN** |
| `HEAD` | `3a15112cca6d7c072131b3592394dfc52b32c66b` |
| `git ls-remote origin refs/heads/main` | `3a15112…` — **identical**, not the tracking ref |
| latest record | J-726 · ROADMAP v7.13 · E-2 Phase-0 v1.6 COMPLETED · E-2 runbook v1.4 COMPLETED |
| apps | **UP** — client + node live, CDP on 9222 (measured, not inherited) |

**Floor:** `svelte-check` **0 / 34 / 15**. Catalogue **435** — untouched **iff** `ui/core` stays shut.
🛑 **`cargo` IS NOT A FLOOR.** Leg E touches zero `.rs`; an identical result is a **scope argument, not a measurement** (`F8`).

**THE LIVE SCREEN, recorded AS A SCREEN (`N-184`/`N-190`/`N-194`)** — 7 Spaces of which **3 are DMs**, panel mounted, no latch, no selection, nothing folded, **zero saved UI states**:

| reading | value |
|---|---|
| R1 getter | `{count: 7, selectedId: null, hasEmpty: false}` |
| R1 row entities | **14** (`region-spaces__panel-*` — `N-184`'s two-per-row) |
| DM home getter | `{count: 3, selfFirst: false}` |
| registry | **184**, `count === unique` |

🛑 **THIS IS NOT A FLOOR AND MUST NOT BE CARRIED INTO VERIFY.** It is today's screen. **E-3 must measure its own before/after in ONE sitting on ONE client** (§5).

---

## §2 — THE AUDIT (grounded at `3a15112`)

### 2.1 — Inside `spaces-panel.svelte`: FOUR consumers of one local `spaces`

| # | site | what it does | filtered? |
|---|---|---|---|
| a | `:47` | `const spaces = $derived(spacesState.spaces)` — the local handle | — |
| b | `:50` | `items = spaces.map(...)` — **the render list** | 🔒 **YES — this is the filter** |
| c | `:58-63` | `selected` `$derived.by` — resolves the latch, returns `undefined` when `s?.counterpart != null` (C-bis-6) | 🛑 **NO — see `F1`** |
| d | `:65-68` | `onActivate` — `find` by id, writes the bus | no (harmless either way) |
| e | `:71-75` | `debug()` — `count: spaces.length`, `hasEmpty: spaces.length === 0` | 🛑 **YES — see `F2`** |

### 2.2 — The predicate already ships

`isDmSpace` is exported at `spaces-state.svelte.ts:43` (`E-1`, the argued fourth file) with its own comment naming the drift it exists to prevent: *"`counterpart != null` copies would be a `D-067` drift surface — flip one, forget the other, and a Space shows in both panels or neither."* ⇒ **E-3 uses `isDmSpace`, never an inline test.**

---

## §3 — FINDINGS

### 🛑 F1 — `selected` MUST KEEP READING THE **UNFILTERED** LIST, AND GETTING THIS BACKWARDS SILENTLY BREAKS C-bis-6
`:58-63` resolves the latched id against `spaces` **in order to suppress the highlight when that Space is a DM**. If the filter is applied to the local `spaces` handle instead of to `items`, then a latched DM **no longer resolves** ⇒ `s` is `undefined` ⇒ `s?.counterpart != null` is **false** ⇒ **the suppression stops suppressing and returns the id.**

✅ Harmless *today* only because the row is no longer rendered, so nothing paints. 🛑 **But the guard would be inverted while looking untouched, and the next reader would find a `$derived` whose stated reason no longer holds.** 🔑 ***The filter and the suppression read the same list for opposite purposes — one wants DMs gone, the other needs them present to recognise them.*** ⇒ 🔒 **filter at `items`, not at `:47`.**

📌 After E-3 the suppression is **doubly guarded** (a DM has no row to light, *and* the latch resolves to `undefined`). **Keep it** — removing it is scope creep and would leave R1 handing `entity-panel` an id it does not render.

### 🛑 F2 — IF THE GETTER IS NOT FILTERED, IT LIES — AND VERIFY IS WHAT READS IT
`debug()` publishes `count: spaces.length`. Filter only `items` and the panel renders **4 rows while its aggregate getter says 7**. `hasEmpty` has the same defect at the empty end.

🔑 **This is not cosmetic: the getter is the primary verify surface.** A probe reading `count: 7` after a correct filter would report **FAILURE against correct code** — `N-194`'s exact shape, and the third time this arc a getter has nearly produced a false verdict. ⇒ 🔒 **`count` and `hasEmpty` derive from the FILTERED list; `selected` and `onActivate` from the unfiltered one.** *W-4 says the aggregate getter reports what the panel OWNS, and after E-3 the panel owns four rows.*

### 🛑 F3 — THE "THREE READERS" CENSUS IS STALE. THERE ARE **NINE READ SITES ACROSS FIVE FILES**
`M_RP_MEMBER_ACT_LEG_E_PHASE0.md` §2 records *"Three readers of `spacesState.spaces`, re-confirmed: `spaces-panel:50` · `rooms-panel:30` · `room-latch` `resolveLatched`."* **Measured now:**

| file | sites |
|---|---|
| `spaces-panel.svelte` | `:47` |
| `dm-spaces.svelte` | `:69` · `:92` · `:103` |
| `members-panel.svelte` | `:156` · `:227` · `:285` |
| `room-latch.svelte.ts` | `:51` |
| `space-latch.svelte.ts` | `:59` |

📌 `rooms-panel` does **not** read the store directly — it reads `spaceLatch.scopedSpace?.rooms`, i.e. **indirectly via `space-latch:59`**. ⚠️ **The old census was true when written; `E-1` and `C-bis` widened it.** 🔑 ***The arc's recurring species — a claim narrower than the thing it describes, reused as if complete*** — caught here **before** it became an instruction.

✅ **It does not change the design; it strengthens the lock.** **EIGHT of the nine sites need DMs PRESENT** (the home lists them, the latches resolve them, `canSend` depends on that resolution, `members-panel` finds a DM by counterpart). ⇒ 🔒 **THE FILTER LIVES IN ONE `$derived` IN ONE FILE. Touching the store would break the DM home, both latches, `canSend`, and member activation in a single edit.**

### ✅ F4 — WHAT THE ROW REMOVAL COSTS THE REGISTRY, AND WHY IT CANNOT BE PREDICTED
`N-184`: one Space row registers **two** entities (`entity-item` + `entity-avatar`). Today R1 shows **14 row entities for 7 Spaces**, exactly two each. Removing 3 DM rows ⇒ **−6 expected.**

🛑 **EXPECTED IS NOT MEASURED.** `N-194` is the standing warning: at `E-1` a predicted delta matched an observed one **with no mechanism in common**. ⇒ **verify enumerates the before/after in ONE sitting and states the screen; the −6 is a hypothesis to test, not a number to record.**

### ✅ F5 — NOTHING ELSE MOVES
`entity-panel`, `entity-item`, `entity-avatar` are `core` and are **not opened** ⇒ catalogue stays **435**. Zero `.rs`. `skin.css` untouched. `DEFAULT_LAYOUT` untouched. **No new `core` code; no new algebra.**

---

## §4 — OPEN, AND JOE'S. `D-121`'s **THREE** lenses: ① user-visible impact per option → ② tier consequence → ③ resource cost.

📌 **Lens ② for both items is *NO TIER CONSEQUENCE*, stated once** — a render filter and a DEV bridge move no byte, create no copy, and decide nobody's erasure fate.

### 🔒 ① — **CLOSED 2026-08-13: `R-a`, FOLD THE `revert()` LINE IN. PROVENANCE DELEGATED** (*"the booth by your recomms"*, `D-141`). 🔒 **E-3 IS A TWO-FILE LEG AND THE RUNBOOK SAYS SO UP FRONT** — stated, not discovered. It discharges `E-2`'s undriven `V3` and gives E-3's own verify a second reachable load path.

### 🔓 ① — DOES THE `revert()` DEV-BRIDGE GAP RIDE IN E-3, OR STAY ITS OWN CHANGE?

`V3` went undriven at `E-2b` because `layout.revert` is **unreachable from any eval** (no `runCommand` on `window`, no bridge entry, deliberately element-absent per J-500). J-726 filed the discharger: **one line adding `revert()` to `__XGEN_LAYOUT__`**.

**R-a — fold it into E-3 as a named scope item.** ① None (DEV-only, dead-code-eliminated in release). ③ **One line**, in a file E-3 opens anyway? 🛑 **NO — it lives in `app_client.svelte`, which E-3 does NOT otherwise touch.** So it makes E-3 a two-file leg.
**R-b — its own micro-leg before E-3.** ① None. ③ One line + its own verify + its own records — **the records cost exceeds the code cost by an order of magnitude.**
**R-c — leave it filed; discharge it whenever `app_client.svelte` is next opened for another reason.** ① None. ③ Zero now. 🛑 **But `V3` stays undriven meanwhile, and a filed item with no trigger is how `M-RP6.2`'s deferral sat un-owned for a milestone** (J-598's finding).

📌 **Chat's recommendation: `R-a`, with the scope stated as two files up front rather than discovered.** It is genuinely one line, it discharges a named debt from the previous leg, and **E-3's own verify benefits**: `layout.revert` is a second reachable load path, which is exactly what a filter's before/after wants. 🔓 **If Joe prefers E-3 stay one file, `R-c` is honest provided the trigger is written as a fact** (`N-182`): *the next milestone that opens `app_client.svelte`*.

### 🔒 ② — **CLOSED 2026-08-13: `S-a`, SHIP SILENT. PROVENANCE DELEGATED** (`D-141`). No empty-state hint, no disclosure. 🔓 **Wording, if ever wanted, stays Joe's** — and `N-192` (the DM-row label) is already his and already filed.

### 🔓 ② — THE ROW FORM AFTER THE FILTER — DOES ANYTHING NEED SAYING ON SCREEN?

After E-3, R1 shows 4 Spaces and the DM home shows 3. A user who knew their DMs were "in the Spaces list" sees them vanish from it.

**S-a — ship silent.** ① The panels read as two clean lists; the DM home is directly below R1 and titled `DM Spaces`. ③ Zero.
**S-b — an empty-state hint in R1** (*"direct messages live below"*). ① Explains the move once. 🛑 ③ It is a **W-8 phase-limit disclosure with no removal trigger** — `N-109`'s exact shape, and it would need its own sweep rule.
**S-c — Joe's wording, deferred to `M-RP-SKIN`.** ① Nothing now. ③ Zero.

📌 **Chat's recommendation: `S-a`.** The home is adjacent, titled, and populated; **an explanation for a change only the developer has ever seen is a disclosure written for nobody.** 🔓 **Wording, if any is ever wanted, is Joe's** — and `N-192` (the DM-row label) is already his and already filed.

---

## §5 — PROPOSED SUB-LEGS

| leg | what | floor | gated on |
|---|---|---|---|
| **E-3a** | the filter: `visible = spaces.filter((s) => !isDmSpace(s))` feeding **`items` + `debug().count` + `hasEmpty`**; `selected`/`onActivate` stay on the unfiltered list (`F1`/`F2`). **PLUS `revert()` on the `__XGEN_LAYOUT__` DEV bridge (① = `R-a`) ⇒ TWO FILES** | `svelte-check` **0/34/15** | ① ② ruled |
| **E-3b** | drive it — **the positive control first** (§5.1) | `svelte-check` | E-3a |
| **E-3c** | records (`D-074`) | — | E-3b |

### 5.1 — THE VERIFY'S ONE HARD REQUIREMENT

🛑 **"THE DM ROWS ARE GONE" IS AN EMPTY RESULT AND CANNOT PASS ON ITS OWN (`N-099`).** A probe that finds no DM row in R1 returns the same thing whether the filter works **or the probe is looking at the wrong panel, the wrong attribute, or a client that failed to load.**

🔒 **THE PASS CONDITION IS A TRANSITION, MEASURED IN ONE SITTING ON ONE CLIENT:**
1. **BEFORE** — with the pre-E-3a build running, **read a DM row IN R1 and print its id** — that is the proof the probe can see the thing it will later claim is absent.
2. **AFTER** — same client, same Space tree, same draft state: R1 getter `count` drops **7 → 4**, the 3 DM ids are absent from R1's rendered rows, and **the DM home still shows 3**.
3. **ENUMERATE the registry delta** (`F4` predicts −6; **the prediction is tested, not recorded**).
4. **The complement, both directions** (the `E-1` `V5` precedent): clicking a non-DM Space still lights R1; the DM home still opens a DM and R1 stays unlit.
5. 🔒 **`canSend` and both latches still resolve a DM** — the store was not touched, and this is what proves it.

📌 **No send. Nothing mints a DM. Joe's client state is read-only apart from anything he consents to.**

---

## §6 — NOT TOUCHED

The wire · any protocol event · `xgen-core` · `xgen-node` · `xgen-common` · any `.rs` · **`skin.css` (Joe's file)** · `ui/core/**` (catalogue) · `dm-spaces.svelte` (`E-1`) · `layout-default.ts` (`E-2`) · `DEFAULT_LAYOUT` · **the `spacesState` store itself (🔒 `F3` — eight of nine read sites need DMs present)**.

⚠️ **`M-RP-INTRO`'s trigger fired at J-716 and it still has no Phase-0.** The oldest outstanding item. **Not E-3's.**

---

## §7 — WHERE THIS DOCUMENT IS MOST LIKELY WRONG

1. ✅ ~~**`F1` is reasoned from source, not driven.**~~ 🛑 **SUPERSEDED BY CLAIR'S `PM-2` (2026-08-13): the claim that `F1` was UNGATED was FALSE, and it was the most load-bearing sentence in the runbook.** `V6` **IS** the discriminator — `selected` (`:58-63`) depends only on `spaceLatch` and `spaces`, **never on rendering**, so `debug().selectedId` is **singly caused**: `null` iff `F1` is honoured, **the DM's raw id** iff `:47` was naively filtered. 🔑 ***The doubly-caused PAINT was conflated with the singly-caused GETTER VALUE*** — and the mechanism sat in a comment **two lines above** the code being reasoned about (`:56-57`). **`N-180` at arm's length.** ✅ **`F1` is GATED.**
2. **`F4`'s −6 is arithmetic.** `N-194` says a predicted number and an observed number once agreed with **no mechanism in common**. It is written as a hypothesis and must stay one until seen.
3. **The `visible`/`spaces` split is two names for one list in a 40-line component** — it may read as over-shaped. The honest counter is `F1`: they are used for **opposite purposes**, and a single filtered handle silently breaks one of them.
4. **§4 ① may be the wrong instinct.** Folding a bridge line into a filter leg widens a scope for tidiness, and this project has refused riders repeatedly. **It is offered as a recommendation precisely because it is arguable.**
5. **This document has not been read by anyone outside its author.** ⚠️ *`E-2`'s two most valuable corrections — `PM-1` and `Q2` — both came from Clair's read, and its own verify half was still wrong afterwards. An adversarial read before the lock is worth its cost, and it should be pointed at §5.1, not only at §3.*
