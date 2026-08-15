# Clair — adversarial read of M_RP_MEMBER_ACT_LEG_E5_PHASE0.md v1.0 (M-RP-MEMBER-ACT Leg E-5)
> **Status**: COMPLETED  
> Version: 1.0  
> Date: Aug 2026  
> **Last updated**: 2026-08-14  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

> ⚠️ **ANNOTATION AT THE SITE (`D-131`), ADDED BY CHAT AT J-731 WHEN THIS BRIEF LANDED — THE VERDICT BELOW IS ABOUT v1.0 AND MUST NOT BE READ AS A VERDICT ON v1.1.** This read was taken against `M_RP_MEMBER_ACT_LEG_E5_PHASE0.md` **v1.0**. Its verdict, *LOCKABLE WITH NAMED CHANGES*, was **discharged**: all three plan-movers were re-driven by Chat under Rule 5 and **all three held**. `PM-1` corrected `F5` (Leg `D` is not a ROADMAP node, so `R-2` never blocked the close). `PM-2`'s `M1′` was **measured and works** — the wrap branch and `E-3`'s filter were both driven with **no disk touched**. `PM-3`'s `OWED-2`/`OWED-3` gap was real. ⇒ the Phase-0 is now **v1.1 COMPLETED**, and the named changes are in it. 🔑 **Both Rule-6 corrections aimed at Chat's kickoff were also confirmed**: the `F5` attack was framed as precedential when the decisive fact was structural, and the `canSend` note named one reading when two exist (`__XGEN_ECHO_BRIDGE__.room.canSend`, `app_client.svelte:468`). ***Every defect this read found came from outside the text; Chat's own re-reads had passed.***  

---

## §0 — WHAT THIS IS

A read, not an implementation. **No code, no edits, no commits, no runbook, no app launched, no send, no DM minted, nothing re-annotated.** Every `file:line` below came from a tool that printed it **this session at HEAD `fc74660`**, not from the document and not from memory.

🛑 **State at open, re-measured:** `git status` clean · `git rev-parse HEAD` = `fc74660` = `git ls-remote origin refs/heads/main` (identical). The read was run **§2 and §4 first, cold**, then §5, §3, §7 — the ordering the kickoff set.

---

## §1 — VERDICT

**LOCKABLE WITH NAMED CHANGES.**

The audit's **core observations are sound** — F1, F2, F4's central claim, F6 and F7 all hold against source, and I could not break them. But **three of the ten findings carry a defect that moves the plan, and two of §4's three option sets are missing an option** — and in both cases the missing option is the cheaper one, exactly the shape the kickoff predicted.

- **F5 is misattributed and its headline is false.** R-2 does **not** block the close, because **Leg D is not a ROADMAP tree node** — the tree derives M-RP-MEMBER-ACT from Leg E alone. "The milestone cannot go ✅ at all, indefinitely" is wrong against the tree as it actually stands. → §4 ③ gains a missing option **N0** (no structure change needed).
- **F4's central claim is correct; its conclusion ("Joe's disk must be touched") overstates necessity.** A **fourth route** — a dynamic `import()` of the `layout-default` module to run the wrap-branch algebra and drive the filter in-memory — was not considered and **may avoid the disk entirely.** It must be **measured** before §4 ② commits to M1-with-disk. → §4 ② is missing option **M1′** (drive without the disk).
- **F8's ledger is a census, not a partition** — it names OQ5 item 3 but **omits the Leg C-bis task doc's `OWED-2` and `OWED-3`.** The document flags this risk in §7.4; it is real.

None of this is "not lockable." The design of the close is sound; the two structural findings that would have driven Joe's rulings the wrong way (F5's false blocker, F4's forced-disk conclusion) are corrected, and they make the close **cheaper and safer**, not harder.

---

## §2 — PLAN-MOVING

### 🛑 PM-1 — F5 IS MISATTRIBUTED: R-2 GOVERNS THE TREE, AND LEG D IS NOT IN IT.

F5 (E-5 Phase-0 §3, lines 83-89) cites R-2 — *"A milestone with unfinished children is not done"* (`docs/ROADMAP.md:437`) — and concludes *"AS STRUCTURED TODAY, `M-RP-MEMBER-ACT` CANNOT GO ✅ AT ALL … indefinitely."*

**Measured against the actual tree, that is false.** R-2 (`:437`) derives a container's status **from its tree children**. The M-RP-MEMBER-ACT node is `docs/ROADMAP.md:317` (🟢), and its **only child node is Leg E** (`:320`, 🟡). I checked every `Leg D` hit in the file: `:309`/`:312` (M-RP-LIVEFEED-REFRESH), `:361`/`:369` (M-RP-IDENTITY-RESOLUTION), `:387` (M-DOC-ROADTREE) — **none is M-RP-MEMBER-ACT's Leg D.** The node's own narrative describes the leg order as *"E-1 … E-2 … E-3 … then E-5 close. E-4 IS ABSORBED"* — **Leg D is named nowhere in the node.** The five ⏸️ tree leaves (`:396`/`:397`/`:400`/`:401`/`:403`) all sit under ⏸️ parents, unrelated.

⇒ **When Leg E closes ✅, R-2 derives M-RP-MEMBER-ACT to ✅.** Leg D's ⏸️ status lives only in `M_RP_MEMBER_ACT_PHASE0.md` §6 (`:455`), a task doc — and **R-2 does not reach a task-doc row.** F5 imports a §6 fact into an R-2 (tree) argument.

**The real concern F5 gestures at is narrower and correct:** §6's Leg D row would show ⏸️ beside a closed milestone — a **task-doc reconciliation** detail, which is **F1's problem** (§6 unmaintained), folding into E-5.3. It is not a structural blocker.

🔑 **This inverts §4 ③.** The three options offered (N1 don't close · N2 re-home Leg D to M-RP-BLOCK · N3 deprecate) all treat the close as blocked. **They miss the option that is true today:**

> **N0 — no milestone-structure change is needed.** The ROADMAP already derives M-RP-MEMBER-ACT from Leg E, which closes ✅; §6's Leg D row is disposed of as part of E-5.3's reconciliation (a note that Leg D is POSTPONED with `M-RP-BLOCK` as its named discharger, or N2's re-home if Joe prefers it recorded there). No ROADMAP node moves.

N2 remains a legitimate *disposition* of the §6 row and is Joe's; but the **urgency and the "R-2 makes the close unsatisfiable forever" framing are false**, and Joe should rule §4 ③ knowing the close is not blocked. *The kickoff (④) proposed defeating F5 by finding a ✅-over-⏸️ precedent; I censused and found none among the five ⏸️ leaves — but the decisive fact is structural, not precedential: Leg D is absent from the tree R-2 governs, so no precedent is needed.*

### 🛑 PM-2 — F4's CLAIM IS CORRECT; ITS "THE DISK MUST BE TOUCHED" CONCLUSION IS NOT ESTABLISHED. §4 ② IS MISSING AN OPTION.

**F4's central claim holds, both pointers exact:**
- `M_RP_MEMBER_ACT_LEG_E2_PHASE0.md:151` — *"✅ Under `DEFAULT_LAYOUT`, **verified from source**: … `insertBeside` takes the **WRAP** branch."* A read.
- `RUNBOOK_MEMBER_ACT_LEG_E2.md:82` — *"`target: 'spaces'`, `edge: 'bottom'` — Joe's, and NOW **VERIFIED IN BOTH TREES BY MEASUREMENT**."* The DEFAULT_LAYOUT half was **not** measured — only Joe's live tree was driven (J-724). F4's *"a claim narrower than the thing it describes"* is accurate.

**The three routes F4 rules out do fail** (`layout-default.ts` grounded): `DEFAULT_LAYOUT` is a module const (not on `window`); `loadLayout` (`:177`) returns the **persisted** tree, hitting its `DEFAULT_LAYOUT` fallback only when disk has no `session.layout` (`:178`/`:187`); `handleUistateLoad` (`:895`) is the named-state path. Confirmed.

**But a fourth route exists and was not considered.** `loadLayout`'s single exit is `return reinjectSystemRegions(loaded, plugins)` (`layout-default.ts:193`), and `reinjectSystemRegions` (`:146`) is a **pure function**. In a Vite dev server the eval can `await import(...)` the already-loaded `layout-default` module and obtain **both `DEFAULT_LAYOUT` and `reinjectSystemRegions` directly**, then:
- `reinjectSystemRegions(DEFAULT_LAYOUT, plugins)` → **runs the wrap-branch algebra** and returns the exact tree (a MEASUREMENT — the code executes — not a source read, which answers F4's own objection);
- `__XGEN_LAYOUT__.set(thatTree)` → renders it; drive E-3's filter against it;
- restore via `revert()` or `set(current)`.

**No disk touched.** `set` does not persist (`app_client.svelte:394`).

🛑 **I CANNOT VERIFY THIS ROUTE — I did not drive the app, and I must not name a reading I cannot take.** It depends on Vite's internal module URL resolving to a cached module, and on obtaining the `plugins` argument (`mountedPlugins` / the descriptor set). **It is a candidate to MEASURE, not an established route.** But its existence means **§4 ②'s option set is incomplete** — it offers M1 (drive, touch Joe's disk) and M2 (don't drive), and misses:

> **M1′ — drive it without the disk**, via the dynamic-import route, *if the measurement confirms it works.*

⇒ **Chat should measure M1′ before §4 ② is ruled.** If it works, the entire consent-for-disk question (§4 ②, §5 E-5.2, F10's re-armed pre-change-control rule) **dissolves** — which is exactly the payoff the kickoff (③) predicted for a fourth route. *Honest caveat: M1′ tests the algebra (concern 1) and the render+filter (concern 2), but bypasses `loadLayout`'s disk-empty→fallback assignment — a trivial two-line path arguably already exercised by N-095 at E-2's V4. If Chat wants that last line covered too, the disk route remains; the substantive risk (wrap-branch reinject + filter render) is covered by M1′.*

### 📌 PM-3 — F8 IS A CENSUS THAT MISSES TWO TRACKED CARRY-OUTS.

F8's ledger (E-5 Phase-0 §3, lines 105-119) names *"OQ5 item 3 — cross-node invite discovery"* (`:115`) and *"the erased row's retention behaviour"* (`:116`). It does **not** name the two OQ5 items that the **Leg C-bis task doc tracks by name**:
- `M_RP_MEMBER_ACT_LEG_C_BIS.md:89` — **OWED-2**: DM creation to an **erased** identity (OQ5 item 2). *Distinct from F8's "retention behaviour."*
- `M_RP_MEMBER_ACT_LEG_C_BIS.md:104` — **OWED-3**: the **partial first send** (OQ5 item 1).

Whether these **closed** during Leg C-bis or remain open is a predicate check the ledger did not run — `OWED-4` closed (checkbox `[x]` at `:595`, J-716), but OWED-2/OWED-3's disposition is not stated in F8. **The document self-flags this exact gap** (§7.4, `:206`: *"Items owed by … `OQ5` and by the Leg C / C-bis task docs were read but not re-censused predicate-wise"*). ⇒ **E-5.4 must reconcile OWED-2 and OWED-3 into the ledger** (closed-here-and-cite, or open-with-a-home) before it is called complete. A census is not a partition, and this is the third arc-instance.

---

## §3 — WORDING / SHARPENING

- **W-1 — F1 treats §6 as the sole acceptance surface; the real acceptance is the conjunction of per-leg DoDs.** F1 (`:41-47`) is right that `M_RP_MEMBER_ACT_PHASE0.md` has **no central DoD section** (confirmed by whole-file read, §0-§8) and that §6's status column is unmaintained (only leg `0` ✅ at `:450` and leg `D` ⏸️ at `:455` carry a state; A/B/C/C-bis shipped without one). But acceptance is **distributed** — each leg's task doc / runbook carries its own DoD (e.g. `M_RP_MEMBER_ACT_LEG_C_BIS.md:595`). ⇒ E-5.3's reconciliation should **verify each closed leg's DoD was met** and cite its `J-nnn`, not merely stamp a state on §6's row. Sharpens F1; does not weaken it.
- **W-2 — §4 ①'s two options are complete and J1 is well-supported; state the supporting weight.** F2's contradiction is real (Leg E §0 `:15` *"closing it closes the milestone"* + §5 E-5 `:214` *"verify + records + close"* vs §6 row F `:457`). Three later records — Leg E Phase-0 §0, the PLAY head, and **the ROADMAP node's own narrative** (`:320`, *"E-1 … E-5 close, E-4 ABSORBED"*, no Leg F) — all describe E-5 as the close with **no Leg F**; only §6 (Joe-locked, earlier) carries row F. J1 is the majority-of-records reading and the E-4 precedent (`:213`, one leg old) is real. It remains correctly Joe's because §6 is Joe-locked — but the recommendation should say the weight is **3 records to 1**, not present it as evenly balanced.

---

## §4 — CHECKED, AND COULD NOT BREAK

- **F1** — no DoD/acceptance section anywhere in `M_RP_MEMBER_ACT_PHASE0.md` (§0-§8, full read); §6's state column carries a symbol on exactly two of eight rows, as claimed.
- **F2** — the two-document contradiction is real and exact against source (`:15`, `:214`, `:457`); *"P1's species at milestone scale"* is a fair characterisation (E-5's row describes work §6 assigns to F).
- **F3** — the three prior legs each verified themselves (E-1 J-721, E-2 J-726 with V3 honestly undriven, E-3 J-729 all-ten-green); the word *"verify"* is legitimately spent except for F4. Confirmed.
- **F4 (central)** — both pointers exact; the runbook overstates *"both trees by measurement."*
- **F6** — `docs/ROADMAP.md:320` names *"discharger `M-RP-STARTUP` **or** an eager book fill"*; `M-RP-STARTUP`'s node (`:326`) carries its own `Owes:` (`:327`) that does **not** reciprocate this carry-out; the *"both sides or it goes stale"* rule is at `:430`. The *"or commits to neither"* observation is sound.
- **F7** — `M-RP-INTRO` node (`:341`) carries `↳ trigger: Leg C-bis lands` (`:343`); Leg C-bis closed at J-716; the *"a trigger that has fired is a defect"* rule is at `:453`. The trigger has fired and the node has no Phase-0. Confirmed.
- **F10** — the pre-change-control rule is inert for a records-only leg and re-arms only inside F4's probe; sound, and it belongs in E-5.2's own control (below).
- **§5 leg ordering** — E-5.0 → E-5.1 → {E-5.2 if M1, E-5.3} → E-5.4 → E-5.5 is a clean DAG; **no row's precondition is produced by a later row** (F9's second question — see §5).

---

## §5 — THE RUNNABILITY PASS (F9's second question)

*Can each §5 sub-leg be RUN, in order, by the seat that owns it?* Walked row by row:

| leg | executor | precondition | produced by |
|---|---|---|---|
| E-5.0 | Clair (read) | this document exists | — ✅ |
| E-5.1 | **Joe** (rulings) | E-5.0's brief | earlier ✅ |
| E-5.2 | Chat (CDP) + Joe (disk consent) | ② ruled M1 | E-5.1 ✅ |
| E-5.3 | Chat (records) | §4 ① ruled | E-5.1 ✅ |
| E-5.4 | Chat (records) | E-5.3 | earlier ✅ |
| E-5.5 | Chat (D-074 atomic) | all | earlier ✅ |

**No backward-precondition defect** — the E-3 V0 capture-window trap does not recur, because E-5.2 owns its own before/after control (the SHA captured at probe start, restored at end; F10 names this). Two runnability caveats worth writing into E-5.2's runbook if ② = M1:

1. 🛑 **E-5.2's very mechanism is contingent on PM-2's measurement.** As written it is a **disk** probe; if M1′ (dynamic import) works, E-5.2 neither touches Joe's disk nor needs the CDP-plus-consent shape it assumes. ⇒ **the M1′ measurement must precede E-5.2's runbook**, or the runbook commits to a mechanism that may be unnecessary.
2. **Ruling ≠ live consent.** §4 ② folds "drive it" and "consent to the disk operation" into M1. The E-2 precedent split them (*"consent for the disk write stays his at E-2b"*). E-5.2's runbook should **re-confirm consent at probe time** (N-123), not treat the E-5.1 ruling as blanket authorisation for a live operation on Joe's `xgen-client_uistate.json`.

---

## §6 — RULE 6: DISAGREEMENTS WITH THIS KICKOFF

1. 🛑 **The kickoff's own framing of F5 (item ④) is narrower than the truth.** It says *"F5 READS R-2 LITERALLY … if precedent exists, R-2's last sentence is narrower."* The decisive fact is not whether a ✅-over-⏸️ precedent exists — it is that **Leg D is not a ROADMAP tree node** (PM-1), so R-2 never reaches it regardless of precedent. Attacking F5 only via a precedent census would **under-attack** it: it would conclude *"R-2 blocks unless precedent exists,"* missing that R-2 does not apply to Leg D at all.
2. **The kickoff's canSend note (under "what this seat kept getting wrong" ③) is incomplete.** It says *"canSend is on `composer-panel#region-composer` → `state.canSend`, not on `__XGEN_SEND__`."* True as far as it goes — but `canSend` is **also** exposed at `__XGEN_ECHO_BRIDGE__.room.canSend` (`app_client.svelte:463`). Not load-bearing for E-5 (no probe here reads it), but the warning names only one of two surfaces.
3. **Minor:** the kickoff lists *"catalogue 435"* under `🔒 FLOOR`. For a records-only leg touching zero `ui/core`, catalogue is untouched **by scope**, not a re-measured floor — the E-5 Phase-0 itself states this correctly (`:31`, *"stated rather than re-run"*). Agreement, flagged so the word "FLOOR" is not read as a measurement E-5 owes.

---

## §7 — WHERE THIS READ IS MOST LIKELY WRONG

1. 🛑 **PM-2's fourth route (M1′) is UNVERIFIED and I cannot verify it** — I did not drive the app (Joe's live client, read-only for me). If Vite's module resolution does not expose `layout-default`'s exports to an eval, or the `plugins` argument cannot be obtained, M1′ fails and **F4's disk conclusion stands.** I present M1′ as a **candidate to measure**, not an established route; treating it as established would be the exact *"do not name a reading you cannot take"* error the kickoff warns about.
2. **PM-1 assumes R-2 operates on ROADMAP tree children only.** If the project's practice treats §6 task-doc legs as R-2 "children" (I found no such rule — R-2 at `:437` is a tree rule, root-exempt, gate-enforced on the tree fence), then F5's blocker would be real and N0 wrong. I read R-2 as a tree rule; the E-5 Phase-0 §7.3 itself hedges this from the other direction.
3. **I did not census every 🟡 tree child against every ✅ parent** (27 `├── 🟡` + 10 `└── 🟡` nodes). I confirmed the five ⏸️ leaves sit under ⏸️ parents and that M-RP-MEMBER-ACT's only child is Leg E; I did not exhaustively prove no ✅-over-🟡 precedent exists elsewhere. It does not bear on PM-1 (which is structural, not precedential), but the precedent census the kickoff asked for is only partial.
4. **Nothing here was measured on the live client**, though it is up. Every finding is a read of the tree at `fc74660` plus grounding in source. F4/PM-2 is the one that most needs driving — and driving it is Chat's, under Rule 5.
