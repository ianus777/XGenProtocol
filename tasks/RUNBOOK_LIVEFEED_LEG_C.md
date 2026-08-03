# RUNBOOK — M-RP-LIVEFEED-REFRESH Leg C: the reconnect re-fill (R1)
> **Status**: ACTIVE  
> Version: 1.5  
> Date: Aug 2026  
> **Last updated**: 2026-08-02  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — What this is, and what it is NOT

**IS:** the build of §5's **R1** — a re-fill of every live panel consumer on the transition into `READY`. Frontend only. `ui/client/src/app_client.svelte`, one file.

**IS ALSO** `M_RP_IDENTITY_RESOLUTION.md`'s **Leg E**, under a second name. Two milestones filed one build; **this milestone owns it** (§5 is this document's decision), that milestone consumes and verifies. **ONE runbook, not two** — two seats writing one `$effect` from two runbooks is the one-writer-per-file-per-atom breach.

**IS NOT:**
- **R4** — the stream's sync-from-cursor replay. Still open, still Joe's, in no leg. **Nothing here may be read as covering it.**
- **Leg B** — the `state.*` delta setters. Grounded at J-658 and re-confirmed here: **Leg C does not depend on Leg B.** Leg B builds delta setters; Leg C re-runs fills. Different mechanisms.
- **`G-B closed`.** 🛑 **N-168: G-B closes on `M-RP-IDENTITY-RESOLUTION` Leg D *AND* this leg together. NO SINGLE LEG MAY TICK IT.** This leg recovers the *exceptional* path; Leg D covers the *ordinary* one.

---

## §1 — Grounding (measured 2026-08-02 at `d84fa65`, HEAD = origin/main, tree clean)

Every line below was read from the file this session. Nothing inherited from a prior kickoff.

| # | Fact | Site |
|---|---|---|
| G1 | `app_client.svelte` is **867 lines** | — |
| G2 | `selfState.connection` has **exactly two live readers**: the gaps feeder and `guardedSend`'s guard | `:144`, `:267` (plus markup `:846-848`) |
| G3 | **Nothing re-fills on reconnect today.** The members fill's only trigger is `roomLatch.effectiveSpaceId` | `:168` — *"the sole tracked dependency"* |
| G4 | **The members half is a named callable** with the §3.5 late-guard already inside the store | `loadMembers` at `:171` |
| G5 | **The spaces half is an INLINE line inside the `onMount` startup block**, not a function | `:625`, inside `onMount(async () => {` at `:571` / `try {` at `:596` |
| G6 | **`setSpaces` is a wholesale replace and touches nothing else** — its whole body is `_spaces = list ?? []` | `spaces-state.svelte.ts:44` |
| G7 | **`setSpaces` has had exactly ONE caller since it was written.** Repo-wide sweep of `ui/**` returns the definition and `:625`, nothing more | verified both directions |
| G8 | 🔑 **`roomLatch` holds a RAW ID and re-resolves it against `spacesState.spaces` on EVERY getter read.** `resolveLatched()` iterates `spacesState.spaces`; there is **no `$derived`, no memoisation** | `room-latch.svelte.ts:43`, `:46`, `:62`, `:69` |
| G9 | `_connection` initialises to `{ state: 'INITIALISING', label: 'Initialising' }`; `setConnection` is the single writer, fed by `xgen-client-state-changed` | `self-state.svelte.ts:87`, `:106` |
| G10 | **11 lifecycle states are enumerated**, `READY` among them. An `$effect` on `selfState.connection` therefore fires on **every** state change, not on a transition | `self-state.svelte.ts:20-28` |
| **G11** | 🛑 **ADDED v1.2, AFTER CLAIR REFUSED §4.1.** The `invoke` at `:625` is **not a module import** — it is a **destructured local** from a dynamic import at `:598`, **inside the `onMount` `try`**. It does not resolve at `:171`. The sibling `loadMembers` uses `tauriInvoke`, an instance-scope helper that lazy-imports **the same module** | `:598`, `:764`, `loadMembers` at `:182-183` |

---

## §2 — 🛑 THE FINDING THAT SHAPES THE LEG: THE SPACES RE-FILL CASCADES INTO THE MEMBERS RE-FILL

**G8 composes with G3 and nobody had put them together.**

`roomLatch.effectiveSpaceId` is a **getter that reads `spacesState.spaces`**. The members `$effect` at `:167-170` tracks `roomLatch.effectiveSpaceId`. ⇒ **`spacesState.spaces` is a TRANSITIVE DEPENDENCY of the members fill effect.**

`setSpaces` assigns a **fresh array** from `invoke('get_spaces')` (G6), so the `$state` source is always `!==` its previous value ⇒ the source is marked dirty ⇒ **the members effect re-runs** — and because `effectiveSpaceId` is a plain getter with **no equality memo**, it re-runs *even though the resolved `spaceId` string is unchanged*.

🔑 **CONSEQUENCE, AND IT INVERTS THE PRICE THE RECORD CARRIES.** `M_RP_LIVEFEED_REFRESH.md` §7 and `M_RP_IDENTITY_RESOLUTION.md` §8 both say *"the members half is FREE, the spaces half needs an extraction"*. **In the reconnect path that is backwards:** the members half is **not** independently free — firing `loadSpaces()` **already** drives it. A reconnect effect that calls both would fire **TWO fills**, each one setting `_roster = null` first (`setInflight`, `address-book.svelte.ts:134`) ⇒ **the panel blinks to UNKNOWN twice per reconnect.**

🛑 **THIS IS PREDICTED FROM THE CODE, NOT MEASURED. IT IS A HYPOTHESIS UNTIL C-b0 RUNS.** *A number that disagrees with the record is a hypothesis, not a discovery* — and so is a mechanism. **C-b0 exists to measure it, and the effect body written in C-b is chosen by that measurement, not by this paragraph.**

✅ **v1.3 — C-b0 RAN 2026-08-02. THE CASCADE IS CONFIRMED, +1, WITH `effectiveSpaceId` BYTE-IDENTICAL ACROSS THE FIRE. §2 IS NO LONGER A HYPOTHESIS. See §5's RESULT block for the four reads.**

📌 *Why no prior session saw it: `setSpaces` has had one caller, at startup, at a moment when nothing is latched (`effectiveSpaceId` is `null` both before and after) — so the cascade has never once been able to produce an observable second fill.*

---

## §3 — RULINGS TAKEN BY CHAT UNDER `D-123` — EACH REVERSIBLE ON ONE WORD

Joe: *"mine are appearance and architecture; yours are technicalities and anything else."* All five below are mechanics. None changes an appearance; none changes the routing shape §2 of the parent locked.

### R-a — LEG C IS WRITTEN BEFORE `M-RP-IDENTITY-RESOLUTION` LEG D
① **User-visible impact: NONE either way** — neither leg is observable with one client, and both are unverifiable until that milestone's Leg F. **The legal answer, and the true one.**
② **Resource:** Leg C is **single-floor** (`svelte-check` only) ⇒ clean attribution under §8's split rule. Leg D moves **cargo AND `svelte-check`** and must split B-i/B-ii-style, so it is the larger authoring job.
③ Tertiary: Leg C unblocks `M-RP-IDENTITY-RESOLUTION` **C-3**; Leg D unblocks nothing downstream. Both edit `app_client.svelte`, so serialising them keeps one writer per file per atom.

### R-b — C-a (THE EXTRACTION) IS ITS OWN COMMIT, BEHAVIOUR-IDENTICAL
A wholesale-replace line with one caller becomes a named function with one caller. **Zero behaviour change, provable by diff.** ⇒ if C-b regresses anything, the extraction is not a suspect.
① none · ② ~6 lines · ③ this is the `B-i / B-ii` discipline applied to a single-floor leg.

### R-c — THE FIRST `READY` OF THE SESSION IS SUPPRESSED
🛑 **v1.4 — THIS RULING'S ORIGINAL JUSTIFICATION WAS FALSE AND IS CORRECTED HERE, NOT DELETED (`D-131`). THE RULING ITSELF STANDS.** v1.0–1.3 said an un-guarded first `READY` costs *"a redundant fill plus **a node round trip**"*. **`get_spaces` is `fn`, NOT `async fn` — a SYNCHRONOUS on-disk read that its own doc comment says never touches `session`** (`desktop.rs:612`). **There is no round trip.** The cost was inferred from the word `invoke` at the call site instead of read at the producer — *verify a claim at its PRODUCER, not at its field name*, and the same species as §4.1's `invoke` and §2's price inversion. ⇒ **TRUE COST of an un-guarded first READY: one local file read, one `setSpaces` reassign, and a cascaded members effect that — with nothing latched at cold start — is a `reset()` on an already-empty book. ① USER-VISIBLE IMPACT: NONE.**
🔒 **WHY IT STILL STANDS:** it is correct, it is four lines, and it makes the effect's contract honest — *"fires on reconnect"* rather than *"fires on any READY"*. **A ruling with a false reason and a true conclusion is corrected, not reversed.**
🛑 **A "first observation" guard is NOT sufficient and the obvious form of it is wrong.** `_connection` starts at `INITIALISING` (G9), so the first *observed* value is not `READY`; the first *transition into* `READY` is a genuine one, and it happens on **every cold start**, after the startup block has already done the equivalent work. Un-suppressed, R1 fires **a redundant fill plus a node round trip on every launch.** ⚠️ **"node round trip" IS THE FALSE CLAIM CORRECTED ABOVE — kept as written (`D-131`), read the correction.**
⇒ **A session-scoped `seenReady` latch: the first transition into `READY` sets the latch and returns; every later one re-fills.**
① ⚠️ **v1.0–1.3 WROTE: *"without it, every cold start shows an extra roster blink and pays an extra drain."* BOTH HALVES ARE FALSE AND ARE KEPT AS WRITTEN (`D-131`).** There is **no blink** — nothing is latched at cold start, so the cascaded effect runs with `sid = null` and resets an empty book, rendering nothing. There is **no drain** — `get_spaces` is a local file read. ⇒ **TRUE ①: NO USER-VISIBLE IMPACT EITHER WAY**, which is why **V3 is retired rather than measured**. ② four lines. ③ contract honesty — and ③ is the whole of the case, stated as tertiary rather than dressed as ①.

### R-d — A FLAP GUARD SHIPS IN v1
§5's option text called it *"a later amendment, not a v1 requirement"*; §9 requires this runbook to **state which it ships**. It ships.
① **User-visible, and this is why the earlier reading was too generous:** `setInflight` sets `_roster = null` **before** the fill, and `null` is the panel's *"I do not know who is here"* state. A flapping link therefore **blinks the members panel to UNKNOWN on every flap** — the exact *"absence renders as UNKNOWN"* surface `M_RP_MEMBERS.md` §3 built, fired repeatedly by our own recovery mechanism.
② **A monotonic-clock timestamp and one comparison — 4 lines.** Ship value **5000 ms**, a plain number in one place.
③ ⚠️ **The number is a first guess and is labelled as one.** It is not derived from a measurement, because no flap has ever been observed in this app. **C-c records it as provisional with `M-RP-LIVEFEED-REFRESH` Leg D named as the surface that can re-price it.**

### R-e — THE EFFECT CALLS ONE ENTRY POINT, AND WHICH ONE IS DECIDED BY C-b0
**Not pre-decided here.** If C-b0 confirms §2's cascade, the effect calls **`loadSpaces()` only** and the members re-fill rides the existing latch effect — **one mechanism, no double fill, and `M_RP_IDENTITY_RESOLUTION.md` Leg E is satisfied by it.** If C-b0 refutes the cascade, the effect calls **both**, in the order spaces-then-members.
🔑 ***A runbook that picks the branch before the measurement is a runbook that will be obeyed instead of checked.***

---

## §4 — C-a — EXTRACT `loadSpaces()`

**Surface:** `ui/client/src/app_client.svelte` only. **Floors:** none move (`svelte-check` re-measured, must be **0/34/15**).

1. Declare `async function loadSpaces()` **beside `loadMembers` (`:171`)**, not inside `onMount` — the reconnect effect must be able to call it.
   - 🛑 **v1.1 SAID THE BODY WAS `spacesState.setSpaces(await invoke('get_spaces'))`. IT IS `tauriInvoke`, NOT `invoke`. ANNOTATED, NOT SILENTLY REPAIRED (`D-131`).** ⇒ **Body: `spacesState.setSpaces(await tauriInvoke('get_spaces'))`.**
   - 🔑 *Why v1.1 was wrong, recorded so the species is visible: §1's **G5** captured that the call is INLINE IN `onMount` — a fact about **location** — and the body was then written as if that fact were also about **binding scope**, which was never measured. `invoke` is a destructured local from the dynamic import at `:598` (**G11**) and is undefined at `:171`. **A claim narrower than the thing it describes, reused as if complete** — the named class, and §9 did not have it. Clair refused it against the file rather than absorbing it (J-516 · J-665 · this).*
   - `tauriInvoke` (`:764`) lazy-imports `invoke` from **the same `@tauri-apps/api/core`**, is instance-scope, and is already what `loadMembers` uses ⇒ behaviour-identical **and** the sibling's idiom.
   - ⚠️ **Preserve the `:620-624` comment block with it.** It carries the M-RP6.2 D1 grounding and the *"no live push until the resident, M-RP6.6"* deferral — 📌 *and that deferral is the `§9` entry about deferrals written as code comments having no owner. **This milestone OWNS it, and this leg discharges its RECONNECT-RECOVERY HALF ONLY** — incremental live delta push remains M-RP6.6's, unbuilt. 🛑 v1.1 said "the act that discharges it", flat; see §10. Update the comment to say so; do not delete it (`D-131`).*
2. Replace `:625` with `await loadSpaces();`.
3. 🛑 **The error posture must NOT change silently.** `:625` sits inside the startup `try {` at `:596` — a rejection there aborts the remaining startup steps. **`loadSpaces` keeps NO internal try/catch**, so the startup path behaves byte-for-byte as today. **The reconnect caller owns its own catch** (C-b step 4). *Adding a catch inside `loadSpaces` would silently change startup behaviour under cover of a refactor.*

**Gate C-a:** `git diff` shows **one function added, one line replaced, one comment amended, and nothing else.** No new `invoke`, no reordering of startup steps.

---

## §5 — C-b0 — THE CASCADE MEASUREMENT (a GATE, not a step)

**Interactive. Real client on 9222. `M-RP-LIVEFEED-REFRESH` Leg D's harness, one client, no second identity needed.**

✅ **AND IT NEEDS NO CODE — IT RUNS ON THE SHIPPED BUILD, BEFORE C-a.** `__XGEN_SPACES__` **is** the store object (N-024), so `setSpaces([...__XGEN_SPACES__.spaces])` re-assigns a **fresh array with byte-identical content**. 🔑 **That isolates exactly the thing in question — the REFERENCE change — from any content change**, which a real `invoke('get_spaces')` could not. ⇒ **C-b0 is a Chat + Joe act that de-risks the leg BEFORE Clair opens the file**, and if it refutes §2 the runbook is annotated rather than a commit reverted.

🛑 **Ports swept and dev servers tree-killed BEFORE launch** — an orphan Vite on 5173 puts `tauri dev` on 5174 and the webview renders a **stale bundle**, which would answer this question about the wrong code.

**Procedure — instrument, do not infer:**
1. Latch a room so `roomLatch.effectiveSpaceId` is **non-null** (this is the state the startup path has never been in when `setSpaces` fired — the reason the cascade has been invisible).
2. Install a counting probe on the members path. **Count `setInflight` calls**, not renders — `__XGEN_MEMBERS__.phase` transitions are observable, and a counter that increments in the store is a probe that **can fail**.
3. Fire `loadSpaces()` once through the DEV surface. **Do not call `loadMembers`.**
4. Read the counter.

**Verdict:**
- counter **+1** ⇒ **cascade CONFIRMED** ⇒ C-b's effect calls `loadSpaces()` **only** (R-e).
- counter **+0** ⇒ **cascade REFUTED** ⇒ §2 is wrong, this runbook is annotated not rewritten (`D-131`), and C-b's effect calls **both**.

### ✅ RESULT — RAN 2026-08-02, CHAT DRIVING CDP, JOE'S APPS. **CASCADE CONFIRMED.**

Build under test carried C-a (behaviour-identical; the probe never goes through `loadSpaces`). Trusted `Input.dispatchMouseEvent` clicks, not synthetic events.

| read | value | what it establishes |
|---|---|---|
| precondition | `latch` set · `sid` non-null · `phase` ready · roster 1 · book 2 · spaces 3 | the state `setSpaces` has **never** fired in — which is why the cascade was invisible for the store's whole life |
| **falsification** | **+1** from a real Space+room click | 🔑 **the probe CAN fire.** `__XGEN_MEMBERS__ === addressBook`, so the wrap intercepts the shell's own path. **A zero from here would have meant something** |
| **idle control, 3 s** | **0** | no ambient increment. Nothing else in the app calls `setInflight` on a timer |
| **fire — fresh array, identical content** | **+1** | ✅ **THE CASCADE** |
| **repeat fire** | **+1** | reproducible, not a one-off |
| `sidBefore === sidAfter` | **true** | 🛑 **THE DECISIVE READ.** The fill re-ran on a **BYTE-IDENTICAL resolved space id**. `effectiveSpaceId` is an unmemoised getter (**G8**) — the effect re-runs on the array's **reference** change, not on any value change |
| latch · selection after | both unchanged, phase `ready`, roster 1, book 2 | **V6's shape holds** on a second `setSpaces` in one session |

**Cleanup discharged (N-123):** `setInflight` restored (asserted `===` original), `__CB0__` deleted, `location.reload()`, post-reload read confirms `typeof __CB0__ === 'undefined'`, conn READY, spaces 3.

🔒 **CONSEQUENCE FOR C-b, UNDER `R-e`: THE EFFECT CALLS `loadSpaces()` AND NOTHING ELSE.** Adding `loadMembers()` beside it would fire **two** fills per reconnect — and each `setInflight` nulls the roster first, so that is **two UNKNOWN blinks per reconnect**, caused by our own recovery.

🔑 **AND IT SETTLES `M_RP_IDENTITY_RESOLUTION.md` LEG E BY MEASUREMENT, NOT BY ARGUMENT:** the members re-fill that Leg E needs is **already delivered** by the spaces re-fill. Leg E requires **no line of its own**.

⚠️ **A LATENT COUPLING IS NOW A MEASURED FACT, NOT A SUSPICION: ANY caller of `setSpaces`, EVER, TRIGGERS A MEMBERS RE-FILL.** Today there are two (startup, reconnect) and both want it. **A third that does not would get it anyway, silently.** ⇒ **for the record as a note at close; NOT fixed in this leg** — memoising `effectiveSpaceId` is an architecture change and would be Joe's.

⚠️ **A probe that cannot fail is not evidence.** Before trusting a `+0`, assert the counter increments at all by calling `loadMembers` directly once — *a false negative reads exactly like a genuine absence*.
⚠️ **Any probe that persists a mutation OWES A CLEANUP CALL** (N-123), and the session ends with `location.reload()`.

---

## §6 — C-b — THE RECONNECT EFFECT

**Surface:** `ui/client/src/app_client.svelte` only.

1. Place the new `$effect` **beside the gaps feeder at `:143-147`**, which already reads `selfState.connection` — the two live together because they observe the same surface.
2. **Transition, not level.** Keep a component-local `prevState` (a plain `let`, not `$state` — it is a memo, not a rendered value) and act only when `prev !== 'READY' && next === 'READY'`.
3. **`seenReady` latch (R-c):** first such transition sets the latch and returns.
4. **Flap guard (R-d):** a re-fill within `RECONNECT_REFILL_MIN_MS = 5000` of the previous one is skipped. Timestamp from a monotonic source, not wall clock.
5. **`untrack` the call**, exactly as the gaps and roomLatch feeders do — the entry point reads and writes store `$state`, and an un-untracked read would self-invalidate (N-136).
6. **The reconnect caller owns its catch** (C-a step 3): a rejected `loadSpaces()` on reconnect must **not** propagate. **It must not fake success either** — the panel keeps whatever it had; there is no *"stale"* marker to set, and inventing one is §5b's `R2`, which is **Joe's and unbuilt**.

🛑 **NOT IN THIS LEG, NAMED SO IT IS NOT ABSORBED:** any stale/blackout marker (§5b, Joe's) · any sync cursor or replay (R4) · any change to `routeMembershipEvent` · any change to `ingest.push`.

---

## §7 — C-c — VERIFY + RECORDS

**Static:** `svelte-check` **0 / 34 / 15**, re-measured on the final tree. Any delta explained, not absorbed.
**Scope:** `git show --stat` — **exactly one file**, zero `.rs`, zero `ui/core`, zero `ui/sampler`.
**Floors NOT re-measured, stated rather than skipped** (zero `.rs` by scope): cargo **1595/0/62 × 56** · sampler catalogue **435** · gate **PASS 74**.

**Records (D-074, ONE commit):** JOURNAL + `CLAUDE.md` PLAY + `docs/ROADMAP.md` + this runbook + `M_RP_LIVEFEED_REFRESH.md` + `M_RP_IDENTITY_RESOLUTION.md` (its Leg E is discharged by this leg landing — recorded on **both** sides, `D-133`).

---

## §8 — VERIFICATION GATES

| # | Gate | Passes iff |
|---|---|---|
| **V0** | Baseline re-measured **before the first edit**, not inherited | `svelte-check` 0/34/15 |
| **V1** | C-a is behaviour-identical | diff shows extraction only; startup still calls it exactly once |
| **V2** | **The cascade verdict is RECORDED with its number** | C-b0's counter value written into the close, whichever way it fell |
| **V3** | 🛑 **RETIRED v1.4 — NOT MEASURED, AND DELIBERATELY SO.** It asked whether R-c swallows the cold-start `READY`. **BOTH OUTCOMES ARE INVISIBLE** (see R-c's correction: no round trip, nothing latched, a `reset()` on an empty book), and the only instrument that could separate them is a new `Page.addScriptToEvaluateOnNewDocument` harness mode — **elegance over impact, which `D-121` exists to refuse.** ⇒ **V4 covers the failure mode that can actually hurt**: a latch that swallows *later* READYs too. That is the dangerous direction; V3's was the harmless one. **NOT TICKED. RECORDED AS RETIRED WITH ITS REASON.** | — |
| **V4** | **A real transition into `READY` after a real drop DOES re-fill** | drop the node service, watch the LED leave `READY`, restore, counter **+1** |
| **V5** | **The flap guard holds** (R-d) | two `READY` transitions inside 5 s ⇒ **one** re-fill |
| **V6** | 🔒 **`setSpaces` fired a SECOND time in one session leaves selection and the room latch intact** — the parent's DoD item | after V4: `__XGEN_ROOM__.latchedRoomId` unchanged, `effectiveRoomId`/`effectiveSpaceId` unchanged, `__XGEN_SEL__` selection unchanged, the stream still shows the same room |
| **V7** | The panel ends **ready**, not `null` | `__XGEN_MEMBERS__.phase === 'ready'`, roster non-null, `count===unique` on the registry with its composition stated |
| **V8** | Registry read **quiescent**, with **store state AND selection state AND named-state count** named (N-105 / N-108 / N-112 / N-115) | recorded as a composition, never a bare number |

⚠️ **V4 and V5 are INTERACTIVE — Joe's apps, custody transfer under `D-132`.** V0–V2 are Chat's.

### ✅ RESULTS — RAN 2026-08-02 ON A COLD-RESTARTED CLIENT AT `9983988`. **ALL GREEN.**

Fresh CDP target, `typeof __CB0__ === 'undefined'` asserted before arming — **no probe residue from C-b0**. Trusted clicks to latch `Engineering / general`. Pre-drop values **recorded, not remembered**, so V6 compares against a captured baseline.

| gate | measured | verdict |
|---|---|---|
| **V4 — outage** | node PID hard-killed (`/T /F`) — the server-disappears case, not a graceful goodbye. Client cycled `CONNECTING`/`RECONNECTING` across 8 polls, **counter 0 throughout** | ✅ **no spurious fire during the outage** |
| **V4 — recovery** | node relaunched; 6 polls `RECONNECTING`, then `READY` ⇒ **counter +1**, log entry stamped `conn: READY` | ✅ **THE LEG'S FIRST REAL BEHAVIOUR VERIFICATION.** 🔑 It is also what **retired V3** gave up: a `seenReady` latch that swallowed *later* READYs would read **0** here. It read 1 |
| **V5 — flap** | two genuine `!READY→READY` edges **1603 ms apart**, driven through `setConnection` (the single writer the event listener itself calls) and **scheduled via `setTimeout` so the effect flushes between them** — four writes in one synchronous eval would have tested Svelte's batching, not the guard. **Counter = 1**, stamped at edge 1 | ✅ **guard holds** |
| **V6** | `latchSame` · `sidSame` · `ridSame` · `selSame` · `selRegionSame` — **all true** against the captured baseline | ✅ **the parent's DoD item** |
| **V7** | phase `ready`, `roster === null` → **false**, roster 1, book 2, notFound 0, spaces 3 | ✅ ends ready, not null |
| **V8** | **169 DOM `data-debug-id`s / 169 unique**, conn `READY` — composition stated, not a bare count | ✅ quiescent |

**Cleanup discharged (N-123):** `setInflight` restored and asserted `===` original, `__V4__`/`__V5__` deleted, `location.reload()`, post-reload read confirms both `undefined`, conn `READY`, spaces 3.

🛑 **DEFECT FOUND BY THIS PASS, NOT YET FIXED — IT IS MINE AND IT IS IN THE SOURCE.** The committed C-b comment carries *"a redundant fill + a node round trip"*, the **false claim corrected in `R-c` at v1.4**. It was true in the runbook when Clair transcribed it, so this is not a Rule 6 miss on her part — **it is Chat's error propagated into code by a faithful implementer, which is exactly how a bad runbook line does its damage.** ⇒ **a comment-only fix commit is owed on `app_client.svelte`, and that file is Clair's for this leg.**
⚠️ **Re-measure rects before every gesture; `__XGEN_DEBUG__.get(id)` returns `{type,state}` — read `.state.x`. The registry keys on `data-debug-id`, NOT `id`.**

---

## §9 — WHERE THIS RUNBOOK IS MOST LIKELY WRONG

Named up front, because the last four arcs' worst defects were all in the runbook rather than the build.

1. 🛑 **§2's cascade.** It is reasoned from `resolveLatched()` being an unmemoised getter. **Svelte 5's exact invalidation behaviour for a getter-through-a-getter is asserted, not observed.** C-b0 exists solely to break it, and R-e refuses to pick a branch before it runs.
2. **The flap number.** 5000 ms is a guess wearing a constant's clothes. Labelled provisional, with a named re-pricing surface.
3. **The error posture at `:596`.** The claim that a `loadSpaces` rejection currently aborts the remaining startup steps is read from the `try {` at `:596` and the `await` at `:625`. **Check the block's actual extent before relying on it** — *a line number transcribed from a grep is not a claim about a control-flow structure.*
4. **The comment at `:620-624`.** Its content is described from this session's read; **re-read it at edit time.** C-a's own insertions move every anchor below them (J-657's species).
5. 🛑 **ADDED v1.2 — THE ONE §9 DID NOT HAVE, WHICH IS THE POINT.** §4.1's `invoke` was a token transcribed from `:625` without measuring its **binding scope**. ⇒ **§9 IS NOT A CENSUS OF THIS RUNBOOK'S ERRORS AND MUST NOT BE READ AS ONE.** It is four — now five — places the author already doubted. *A section listing known doubts cannot list the doubt the author did not have.* **Clair's Rule 6 refusal is the mechanism that finds the rest; §9 is not.**

---

## §10 — DoD

- [ ] V0–V8 all green, each number **measured** and none derived
- [ ] C-b0's verdict recorded **with its counter value**, whichever way it fell
- [ ] R-d's flap-guard value recorded as **provisional**, with its re-pricing surface named
- [ ] `§9`'s *"a deferral written as a code comment has no owner"* entry **owned, and its RECONNECT-RECOVERY HALF discharged** — the M-RP6.2 comment is amended, not deleted
  - 🛑 **v1.1 WROTE "discharged" FLAT. THAT IS AN OVERCLAIM AND CLAIR REFUSED IT IN THE COMMENT WORDING BEFORE THIS LINE WAS FIXED.** The M-RP6.6 deferral has **two halves** — reconnect recovery (this leg) and **incremental live delta push (still M-RP6.6's, unbuilt)**. ⇒ **her wording stands and this checklist moved to match it, not the reverse.** An N-109 overclaim was about to be baked into a code comment *and* a DoD tick.
- [ ] `M_RP_IDENTITY_RESOLUTION.md` Leg E marked discharged, **on both sides** (`D-133`)
- [ ] 🛑 **`G-B` NOT ticked** — it closes on Leg D *and* this leg together (N-168)
- [ ] 🛑 **R4 not touched, and no item above read as covering it**
- [ ] Records in ONE commit (`D-074`)

---

## §11 — Handoff

🛑 **v1.0 OF THIS SECTION PUT `R-a … R-e` TO JOE AS *"LOCK OR REVERSE"*. THAT WAS WRONG AND IT IS CORRECTED HERE, KEPT NOT ERASED (`D-131`).** Writing a ruling as Chat's under `D-123` and then presenting it for approval is **under-stepping wearing a lock's clothes** — the recurring seat error, third instance (J-618 · J-669 · this). Joe caught it with four words: *"do i have to lock Rs?"*

🔒 **R-a … R-e ARE IN FORCE. THEY WAIT FOR NOTHING.** *"Reversible on one word"* is a promise about **Chat's** behaviour, not a request for Joe's; silence is not consent, and the rulings hold either way. ⚠️ **R-d's user-visible consequence does not move the seat** — `D-121` says *state the lens*, not *hand over the decision*. Whether a flap guard exists is mechanics; what the panel would **show** during a blink would be Joe's, and there is nothing to show, because §5b's marker is unbuilt.

🔓 **WHAT IS ACTUALLY JOE'S IN THIS LEG — TWO THINGS, AND NEITHER IS AN `R`:**
- **Standing Clair up.** The status flip is a **custody act** — it spends her seat. That is the only reason a runbook has a locked state at all.
- **The push.**

🔒 **UNCHANGED AND NOT RE-OPENED HERE:** §2's one-router shape · §4's delta-vs-fill boundary · §5's R1 · R4's open status · §5b's blackout marker (Joe's, unbuilt).

✅ **CLAIR IS STOOD UP (Joe, 2026-08-02)** — the custody act performed, the status flipped to ACTIVE in the same edit. 🛑 **She does NOT close her own leg:** she hands back numbers and stops. Code commit first, doc bridge second, Joe pushes both.

🛑 **AND C-b0 RUNS BEFORE C-b, NOT BY HER.** It is a read on the shipped build (§5) and its verdict is an INPUT to C-b. **Clair may open C-a immediately; she may not write C-b until the verdict reaches her**, and if it arrives as an assertion rather than a number, that is a Rule 6 flag.
