# M-RP-MEMBER-ACT — Leg E-5: the milestone close — Phase-0
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Aug 2026  
> **Last updated**: 2026-08-14  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0a — WHAT v1.1 CHANGED, AND WHY THE CHANGES ARE NOT EDITS TO A DRAFT

🔑 **THREE OF THIS DOCUMENT'S OWN FINDINGS WERE DEFECTIVE, AND ALL THREE WERE CAUGHT FROM OUTSIDE THE TEXT** — by Clair's adversarial read (`tasks/CLAIR_LEG_E5_PHASE0_READ.md` v1.0) and by driving the client. **Chat's own re-reads passed every time**, which is this arc's standing pattern and the reason §7.6 was written before any of them were found.

| | v1.0 said | v1.1, re-driven by Chat under Rule 5 |
|---|---|---|
| **`F5`** | ROADMAP `R-2` makes the milestone **unclosable indefinitely** | 🛑 **FALSE.** `R-2` derives status from **ROADMAP tree children** and **Leg `D` is not a ROADMAP node** — every `Leg D` hit in the file belongs to another milestone; the milestone node's one child is Leg E. **Closing Leg E derives the milestone to ✅.** Clair `PM-1` |
| **`F4`** | the wrap branch **must** be driven from disk ⇒ **Joe's consent required** | 🔑 **A FOURTH ROUTE EXISTS AND IT WORKS.** `reinjectSystemRegions` is importable from the Vite dev server and pure; the algebra and the render were both driven **with no disk touched**. Clair `PM-2`, measured by Chat at `E-5.2` |
| **`F8`** | the carry-out ledger | ⚠️ **a census, not a partition** — `OWED-2` (`M_RP_MEMBER_ACT_LEG_C_BIS.md:89`) and `OWED-3` (`:104`) were missing. §7.4 self-flagged the risk and the risk was real. Clair `PM-3` |

🛑 **TWO OF THE THREE WERE ROUTED TO JOE ON URGENCY — A DISK RISK AND AN INDEFINITE BLOCKER — AND BOTH URGENCIES WERE THIS SEAT'S OWN ERRORS.** ⇒ ***a false alarm does not merely waste a reading; it makes the routing look justified.*** All three of §4's items were **taken by Chat** once re-measured, under `D-123`, whose named failure mode is **UNDER-stepping**. 📌The governing precedent was measured, not recalled: **`E-4`'s absorption was Chat's** (`M_RP_MEMBER_ACT_LEG_E_PHASE0.md:213`, J-731's predecessor entry) — same milestone, same species, one leg earlier. J-710, cited in v1.0 to justify routing, was a **gate re-point between legs**, which is not the same act.

### §0b — `E-5.2`: THE PATH NO LEG COULD TEST, DRIVEN AND **PASSED** (2026-08-14)

**Client 9222 under a `D-132` custody window, requested and released.** `DEFAULT_LAYOUT` → `reinjectSystemRegions` → `__XGEN_LAYOUT__.set()` → read → `revert()`.

✅ **THE WRAP BRANCH RENDERS:** root `row [1,2,7,2]`, first child a **NEW `col [1,1]` holding `[spaces, dm-spaces]`** — the bisect of `B-a`, on the branch that had never executed. 9 leaves mounted, no crash, both tiles framed (`Spaces`, `DM Spaces`, axis `col`).
✅ **`E-3`'s R1 FILTER HOLDS ON IT:** `spaces-panel#region-spaces` → `count: 4`, rows Engineering · Design · LegBSpace · LegF Verification, **zero `isDm`**. `dm-spaces#region-dm-spaces` → `count: 4`, **all four `flags.isDm: true`**.
✅ **RESTORE VERIFIED:** `revert()` ⇒ `restoredIdentical: true`, 609 chars = 609 chars, leaves and root sizes `[267,185,535,213]` exact. **Nothing persisted, nothing sent, no DM minted, no disk touched.**

⚠️ **BOTH PANELS RETURNED `count: 4`.** A probe that read one and reported the other would have looked correct. Luck of the data — recorded so the next gate does not rely on it.

🔑 **`F6`'s CARRY-OUT IS NOW OBSERVED RATHER THAN REASONED:** every DM row read pure `tail8` — `…5HPPRfXo` · `…IOd_baqk` · `…noYiFeHE` · `…sno_FWmw` — because no Space was latched. Exactly what `E-1` predicted. **This strengthens the case for writing `M-RP-STARTUP`'s side of the gate, which `E-5.4` does.**

### §0c — 🛑 §1'S STATED SCREEN WAS STALE, SO ITS NUMBER IS VOID

§1 records the last registry measurement as **178** on a screen of *"7 Spaces of which 3 are DMs."* **Measured at `E-5.2`: 4 non-DM + 4 DM = 8 Spaces, 4 DMs, registry 180.** The screen moved under the number. ⇒ ***this is precisely why §1 says RECORD THE SCREEN OR RECORD NO NUMBER — and the rule earned its keep against the document that wrote it.*** **Neither 178 nor 180 is a floor.**

---

## §0 — WHAT THIS IS

`E-5` is Leg E's last sub-leg. **No code, no runbook yet** — this is Phase 0 of `D-071`'s four phases.

🛑 **AND THE FIRST THING THIS DOCUMENT DID WAS AUDIT ITS OWN ROW INSTEAD OF EXECUTING IT.** `M_RP_MEMBER_ACT_LEG_E_PHASE0.md` §5 gives `E-5` as *"verify + records + close (`D-074`)"*. **Two of those three words do not survive the audit** — §2 below. *A leg describing work that is not its own is `P1`'s shape and it has already bitten three times in this arc (`E-0`, `E-4`, `F1`'s ROADMAP row). It is not assumed away here.*

---

## §1 — STATE AT OPEN, RE-MEASURED, NOT INHERITED

| item | measured |
|---|---|
| tree | **CLEAN** (`git --no-pager status`) |
| `HEAD` | `08600978294ca2328a68fbc698b5d91949ba36e6` |
| `git ls-remote origin refs/heads/main` | `0860097…` — **identical**, not the tracking ref |
| latest record | J-729 · ROADMAP v7.16 · notes to **N-195** (next free `N-196`) |
| **apps** | 🛑 **CLIENT IS UP** — port **9222** accepting. **9322 and 9422 are DOWN.** *The kickoff did not state this either way; it was measured.* |

**Floors, stated rather than re-run** (this document is reads only; zero `.rs`, zero `ui/**`): `svelte-check` **0 / 34 / 15** · catalogue **435**.

🛑 **`cargo` IS NOT A FLOOR FOR LEG E.** Zero `.rs` in the whole arc ⇒ an identical `cargo` result is **a scope argument, not a measurement** (`M_RP_MEMBER_ACT_LEG_E_PHASE0.md` `F8`).

🛑 **NO REGISTRY NUMBER IS CARRIED.** The last measured value is **178**, taken *after* `E-3` on a **stated screen**: 7 Spaces of which 3 are DMs · DM home mounted · 0 saved UI states · nothing folded · no selection. `N-184` makes it Space-dependent, `N-190` draft-dependent, and `N-194` moved it 168→174 on an **identical** screen for an unrelated cause. **Record the screen, or record no number.**

---

## §2 — THE AUDIT OF `E-5`'s OWN ROW

### 🛑 F1 — THE MILESTONE HAS NO "DEFINITION OF DONE" SECTION. ITS ACCEPTANCE IS §6's LEG TABLE, AND THAT TABLE IS NOT A STATE BOARD.

Measured across `tasks/M_RP_MEMBER_ACT_PHASE0.md` v1.12 in full: sections are §0 · §1 · §2 · §3 · §4 · §5 · §6 · §7 · §8. **There is no `DEFINITION OF DONE`, no `DoD`, no acceptance section.** §6 is the leg table and is the only thing that can serve as one.

🛑 **AND ITS STATUS COLUMN WAS NEVER MAINTAINED.** Of eight rows, exactly **two** carry a state: leg `0` ✅ and leg `D` ⏸️. Rows `A` · `B` · `C` · `C-bis` · `E` · `F` carry **no state symbol at all** — while `A`, `B`, `C` and `C-bis` have all shipped and closed (`RUNBOOK_MEMBER_ACT_LEG_AB.md` v1.3 COMPLETED · Leg C CLOSED J-700 · Leg C-bis CLOSED J-716).

⇒ 🔑 **THE DOCUMENT THAT OWNS THIS MILESTONE'S ACCEPTANCE CANNOT BE READ TO TELL YOU WHETHER IT IS ACCEPTED.** *Do not invent a DoD. `E-5` reconciles §6 against what shipped, in §6 itself, and that reconciliation IS the acceptance record.*

### 🛑 F2 — **LEG `F` EXISTS, AND IT CLAIMS THE MILESTONE CLOSE.** TWO DOCUMENTS EACH SAY THEY CLOSE THIS MILESTONE.

| document | what it says |
|---|---|
| `M_RP_MEMBER_ACT_PHASE0.md` §6, row **F** | **"Records + close (`D-074`)"**, floor `—`, gated on **E** |
| `M_RP_MEMBER_ACT_LEG_E_PHASE0.md` §0 | *"Leg E is `M-RP-MEMBER-ACT`'s last leg. **Closing it closes the milestone.**"* |
| `M_RP_MEMBER_ACT_LEG_E_PHASE0.md` §5, row **E-5** | **"verify + records + close (`D-074`)"**, gated on **E-3** |

🔑 **THE TWO ARE THE SAME WORK, WRITTEN TWICE, AND NEITHER DOCUMENT CITES THE OTHER.** Leg E's Phase-0 was written against §6's *Leg E row* and never read §6's *next row*. 🛑 **This is `P1`'s species at MILESTONE scale — the FOURTH instance in this arc** (`E-0`: a row describing shipped work · `E-4`: a row with no content · `F1`: a ROADMAP `Owes:` contradicting its own Phase-0 · **this**).

⚠️ **AND THE KICKOFF INHERITED THE WRONG ONE.** It states *"After E-5 the milestone CLOSES"* — true only if Leg `F` is disposed of. **The kickoff's own instruction — *audit the claim, including claims made by this kickoff* — fires on the kickoff.** ⇒ **§4 ①, and it is Joe's**, because §6's leg table is Joe-locked (`J-710` precedent: Chat put the `E → D` re-point to Joe rather than editing the table).

### ✅ F3 — `E-5` IS **NOT** A FOURTH VERIFY PASS, AND THE ROW'S WORD *"verify"* IS SPENT

`E-1` (J-721, V1–V8) · `E-2` (J-726, seven gates + `V3` honestly undriven) · `E-3` (J-729, **all ten gates green, zero undriven**, and `V7` discharged `E-2`'s `V3`) each verified themselves, each re-driven by Chat under Rule 5. **There is no per-leg gate left owed by any of the three.**

⇒ **the word `verify` in `E-5`'s row is only legitimate if it names something NO LEG COULD HAVE TESTED.** There is exactly one such thing — `F4`.

---

## §3 — FINDINGS THAT MOVE THE PLAN

### 🛑 F4 — THE ONE THING NO LEG TESTED: **THE `DEFAULT_LAYOUT` PATH HAS NEVER RUN.**

Every live measurement in this arc — `E-1`, `E-2b`, `E-3b` — ran on **Joe's arranged 9-leaf tree**, where `spaces`' parent split runs `col`, so `insertBeside` takes the **SIBLING** branch. **The other branch was never executed.**

**Measured, not recalled** — `M_RP_MEMBER_ACT_LEG_E2_PHASE0.md:151`: *"✅ **Under `DEFAULT_LAYOUT`, verified from source:** `spaces` is a direct child of the root `row` split. `edge: 'bottom'` → axis `col` ≠ `row` ⇒ `insertBeside` takes the **WRAP** branch."* 🛑 **`verified from source` is a READ.** `RUNBOOK_MEMBER_ACT_LEG_E2.md:82` then compresses both halves into *"NOW VERIFIED IN BOTH TREES BY MEASUREMENT"* — **which is true of Joe's tree (driven at J-724) and false of `DEFAULT_LAYOUT`.** 🔑 ***A claim narrower than the thing it describes, reused as if complete — the arc's own species, found in the sentence that certifies the leg.***

**THE UNRUN PATH, end to end:** a client with **no `session.layout`** ⇒ `loadLayout` returns `DEFAULT_LAYOUT` (8 leaves, no home) ⇒ the single-exit re-inject fires on the **WRAP** branch ⇒ `[spaces, dm-spaces]` in a new `col` split ⇒ **and `E-3`'s R1 filter then runs against a `spaces` tile of a shape nothing has ever painted.**

⚠️ **DRIVING IT MEANS A CLIENT WITH NO — OR RENAMED — `xgen-client_uistate.json`. THAT IS JOE'S DISK.** Consent is his, the cleanup is owed, and **the cleanup is part of the probe** (`N-123`). ⇒ **§4 ②.**

📌 **THREE CHEAPER ROUTES CHECKED AND ALL THREE FAIL, so the disk is not reached for casually:** `__XGEN_LAYOUT__.set(DEFAULT_LAYOUT)` — **`DEFAULT_LAYOUT` is a module import, not on `window`** · `revert()` → `handleRevertUi` → `loadLayout()` — **returns the persisted 9-leaf tree, not the default** · `handleUistateLoad` (`:895`) — **a different call site with its own `migrateLayout`, not `loadLayout`'s fallback**.

### 🛑 F5 — LEG `D` IS ⏸️ **POSTPONED, AND ROADMAP RULE `R-2` SAYS A MILESTONE WITH UNFINISHED CHILDREN IS NOT DONE.**

`docs/ROADMAP.md`, *Six rules govern the tree*: **"R-2 — a container's status is derived from its children … A milestone with unfinished children is not done."**

Leg `D` (RMC → the context menu) is ⏸️ POSTPONED since J-710, with a **checkable** trigger — *§5.7's member-scoped-verb census returns non-zero* — whose nearest candidate is `M-RP-BLOCK`, itself **FILED, NOT SCHEDULED, trigger: none**.

⇒ 🛑 **AS STRUCTURED TODAY, `M-RP-MEMBER-ACT` CANNOT GO ✅ AT ALL** — it would sit 🟢/⏸️ behind a leg gated on an unscheduled milestone, indefinitely. **This is a real structural blocker on the close and it is not `E-5`'s to rule.** ⇒ **§4 ③.**

### 🛑 F6 — `M-RP-STARTUP` IS NAMED AS A DISCHARGER ON ONE SIDE ONLY

`ROADMAP.md:320` (the Leg E node) records the `E-1` carry-out: *the DM rows show pure `tail8` on a fresh client until a Space is latched, because the address book fills per-Space; not a defect, `L2` was implemented exactly, **discharger `M-RP-STARTUP` or an eager book fill***.

🛑 **MEASURED AT `ROADMAP.md:326`, `M-RP-STARTUP`'s OWN NODE: its `Owes:` carries five items** — `home_node` cannot designate a Space · restoring a Space alone leaves you half-entered · the room is the unit · fall back on ABSENT never UNREACHABLE · nobody has read what `active` holds. **None of them is this one.**

⇒ ROADMAP's own rule is breached: **"A cross-milestone gate is written on both sides or it goes stale invisibly on one."** 📌 **And the pointer is an *"or"***, which is not a discharger at all — it names two candidates and commits to neither. **`E-5` writes it on the second side, or the carry-out dies with this milestone.**

### 🛑 F7 — `M-RP-INTRO`'s TRIGGER FIRED AT J-716 AND IT STILL HAS NO PHASE-0

`ROADMAP.md`: `M-RP-INTRO` is 🟡 with `↳ trigger: Leg C-bis lands` — **and Leg C-bis closed at J-716.** ROADMAP's own maintenance rule: **"A trigger that has fired is a defect: the node it guards is stale by definition."**

By a wide margin the **oldest outstanding item in the arc**, and **it has now survived an entire milestone.** ⚠️ **Not `E-5`'s to build.** `E-5`'s obligation is that it does not survive the close *unnoticed* — the close is the last moment anyone is looking at this milestone.

### 📌 F8 — THE CARRY-OUT LEDGER: WHAT LEAVES THIS MILESTONE, AND WHERE EACH ITEM LANDS

**Enumerated from `ROADMAP.md`'s `Owes:` lines + Leg E's Phase-0, not recalled.** Each row needs a named home before the node's `Owes:` is reduced (J-715's rule: *reduce on completion, and point at the record that holds the reasoning*).

| carry-out | proposed home | state |
|---|---|---|
| cold-client `tail8` on DM rows (`E-1`) | `M-RP-STARTUP` **or** an eager book fill | 🛑 **one-sided — `F6`** |
| the promotion gap: `counterpart` is never cleared (`G-c`, `F4` of Leg E) | trigger already written: *a promote path writes to the client's `KnownSpace` tree*; `dm_constraints_active` named as the answer | ✅ two-sided, checkable |
| `E-2`'s re-inject must consult the hidden set | `M-RP-WIDGET-SUSPEND` — **DoD-BOUND on its node** | ✅ two-sided |
| the DM-row label wording (`N-192`) · `dm-intro`'s wording · `skin.css` | **Joe** | 🔓 open, Joe's |
| `OQ5` item 3 — cross-node invite discovery | **a measurement of Chat's**, never taken | 🛑 no home |
| the erased row's retention behaviour | the milestone that builds history-expiry + auth tiers | ⚠️ no node named |
| `N-168` — `line-through` through the leading `…` | `M-RP-TAIL8` `Owes:` (closed node) | ⚠️ stranded on a ✅ node |
| clippy's four pre-existing `-D warnings` errors | not a tracked floor, untouched | ✅ stated |
| §6 leg 5's unmeasurable case (`OWED-4`) | ruled *"correct behaviour"* by Joe at J-716 | ✅ closed |

### 🔒 F9 — THE MISSING READ PASS, NAMED AT J-729 AND NOT YET OWNED BY ANY DOCUMENT

J-729's standing lesson: *"an adversarial read pointed at §5 is necessary and **NOT SUFFICIENT**; the missing pass is **can each gate be RUN, in the order written, from the seat that owns it?**"*

**Four gate defects, zero build defects, across `E-2` and `E-3` — all four this seat's** (`PM-1` · `Q2` · `V7` · `§7.1`). ⇒ 🔒 **RECOMMENDED AND BINDING ON `E-5`'s RUNBOOK IF IT HAS GATES: Clair's adversarial read runs `§5` FIRST, cold, and its brief names the runnability pass explicitly as a second question.** *Reading the gates first surfaced both `E-3` movers; it did not surface the two `E-3` verify found, and only an attempt to EXECUTE does.*

### 📌 F10 — THE PRE-CHANGE-CONTROL RULE HAS NO APPLICATION HERE **IF** `E-5` SHIPS NO CODE

J-729: *a control captured on a pre-change build is a gate on the CHANGE, not on the verify — it belongs in the IMPLEMENTER's kickoff.* **`E-5` as records-only has no implementer and no pre-change build** ⇒ the rule is inert. 🛑 **It stops being inert the moment `F4`'s probe is taken**, because that probe mutates state (a renamed store) and its *before* reading is a control. **Stated so it is not rediscovered mid-probe.**

---

## §4 — OPEN, AND JOE'S. Each carries `D-121`'s **THREE** lenses: ① user-visible impact per option → ② tier consequence → ③ resource cost.

📌 **Lens ② for all three items is *NO TIER CONSEQUENCE*, stated once rather than manufactured three times.** All three are records, sequencing and a read-only probe; none moves a byte, creates a copy, or decides whose tier governs. *A manufactured tier rationale is as bad as a manufactured UX one.*

### 🔓 ① — LEG `F`: ABSORBED, OR RUN?

**J1 — `E-5` ABSORBS Leg `F`.** §6's row `F` becomes ⬛ **ABSORBED INTO `E-5`, ID KEPT, NEVER RENUMBERED**, annotated at its site (`D-131`).
① **No user-visible impact** — nothing on screen either way.
③ One annotation. **This is exactly the `E-4` precedent, set inside this same milestone one leg ago**, and `E-4`'s ID was kept for the same reason: `F` is referenced by §6 and by the leg-order arguments beneath it.

**J2 — `E-5` closes LEG E; Leg `F` then closes the MILESTONE.** Two legs, two commits.
① None.
③ A second commit whose entire content is *"the previous commit was correct"* — **`E-5` would already have written the records, so `F` arrives with nothing to record.** ⚠️ *And the milestone would carry a leg whose work is done before it starts, which is the `E-0` shape a third time.*

📌 **Chat's recommendation: J1.** 🔓 **Joe's, because §6's leg table is Joe-locked** — J-710's precedent is explicit that Chat routes a leg-table change rather than taking it.

### 🔓 ② — THE `DEFAULT_LAYOUT` PROBE (`F4`): DRIVE IT, OR RECORD IT UNDRIVEN?

🛑 **CONSENT IS THE GATE, NOT THE COST.** The probe requires Joe's live `xgen-client_uistate.json` to be moved aside, the client relaunched, the tree read, and **the file restored with a byte/SHA check**. His hand-tuned first split `[1762,1396,842]` and 9-leaf tree are what is at risk.

**M1 — DRIVE IT.** Rename → relaunch → read the wrap-branch tree + drive `E-3`'s filter against it → quit → restore → **verify SHA byte-identical** (`N-123`: the cleanup is part of the probe).
① 🔑 **The user-visible stake is the whole point: this is what EVERY NEW USER SEES ON FIRST LAUNCH.** If the wrap branch mis-places or the filter mis-renders there, the defect ships to exactly the population that cannot work around it — and no gate in the arc would have caught it.
③ One session, one probe, one restore. **Risk is not zero:** a crash between rename and restore leaves Joe on a default layout. Mitigation is a **copy, not a move**, plus the SHA control captured *before* anything is touched.

**M2 — RECORD IT UNDRIVEN, with a named discharger.** The `E-2` `V3` precedent: say so in the DoD rather than pass quietly.
① Unknown-until-someone-installs-fresh. ③ Zero now; **the discharger would be `M-RP-STARTUP` or a first-run milestone, neither scheduled** ⇒ in practice *the first person to find it is a user.*

📌 **Chat's recommendation: M1, with a COPY rather than a move**, and the whole probe scripted before the first mutation so the restore cannot depend on a later turn surviving. ⚠️ *`E-2`'s `V3` was left undriven honestly and `E-3`'s `V7` then had to pay the debt one leg later. There is no later leg here.*

### 🔓 ③ — LEG `D`'s ⏸️ AT THE MOMENT OF CLOSE (`F5`) — HOW DOES THE MILESTONE NODE READ?

**N1 — the milestone does NOT close.** It stays live behind Leg `D` until `M-RP-BLOCK` produces a verb.
① None. ③ Zero — **and it is honest by `R-2`.** 🛑 But `M-RP-BLOCK` is *"trigger: none — filed, not scheduled"*, so this is **indefinite**, and a 🟢 node that is actually finished trains readers to distrust the state column.

**N2 — Leg `D` is RE-HOMED to `M-RP-BLOCK` as that milestone's child**, `M-RP-MEMBER-ACT` closes ✅ with a `→ M-RP-BLOCK` successor on the closed node.
① None. ③ Two node edits. 🔑 **It matches what J-710 already established: Leg `D` has nothing honest to build *here*, and its content only becomes buildable inside the milestone that mints the verb.**

**N3 — Leg `D` is ⬛ DEPRECATED with `M-RP-BLOCK` named as the replacement.** ① None. ③ One edit. ⚠️ **Weaker than N2** — deprecation says *superseded*, and the RMC menu is not superseded, it is *waiting*.

📌 **Chat's recommendation: N2.** *A leg whose trigger lives entirely inside another milestone is that milestone's child; leaving it here is what makes `R-2` unsatisfiable forever.* 🔓 **Joe's — this is milestone structure, and `L-1` (RMC → the menu) is one of his uttered locks.**

---

## §5 — PROPOSED SUB-LEGS

| leg | what | floor | gated on |
|---|---|---|---|
| **E-5.0** | **Clair's adversarial read of this Phase-0** — `§2` and `§4` first, then `§3`; the brief names the **runnability** question as its second pass (`F9`) | none | this document |
| **E-5.1** | 🔓 **Joe rules `§4` ① ② ③** | none | E-5.0 |
| **E-5.2** | 🛑 **IFF `②` = M1 — the `DEFAULT_LAYOUT` probe.** Its own runbook, its own control, its own restore-and-verify. **The ONLY leg here that touches anything of Joe's** | svelte-check **by scope** (zero edits expected) | `②` ruled M1 |
| **E-5.3** | **§6 RECONCILED IN PLACE** (`F1`) — every leg row gains its true state + its closing `J-nnn`; row `F` per `①`; Leg E's Phase-0 → **COMPLETED** | none | E-5.1 |
| **E-5.4** | **THE CARRY-OUT LEDGER DISCHARGED** (`F8`) — `M-RP-STARTUP`'s `Owes:` gains its side (`F6`); `M-RP-INTRO`'s fired trigger is surfaced to Joe (`F7`); the `Owes:` line is reduced per J-715 and points at the records | `roadmap-format-gate.ps1` **exit 0** | E-5.3 |
| **E-5.5** | **CLOSE — `D-074` ATOMIC**: `JOURNAL.md` + `CLAUDE.md` PLAY head + `docs/ROADMAP.md` + the task docs in **ONE** commit | — | all |

🛑 **`D-074` IS THE ONE RULE THIS ARC BREACHED TWICE** — documents-only commits with no records at J-722 and again at J-727, *the second time in the shape the first entry had already named.* **Records travel with the thing they record unless Joe explicitly asks for a split.**

🛑 **NO SEND. NO DM MINTED.** Nothing in `E-5` spends Joe's data. A send mints a **permanent** DM and nothing here needs one.

🛑 **`N-195` APPLIES TO EVERY BLOCK WRITTEN BY THIS LEG:** author blocks containing astral characters in a **FILE** and splice them; **verify by MARKER READ-BACK, never by byte count alone.** Byte parity proves CRLF integrity (`N-191`) and nothing else. `CLAUDE.md` and `docs/ROADMAP.md` are **CRLF**; everything else LF.

---

## §6 — NOT TOUCHED

The wire · any protocol event · `xgen-core` · `xgen-node` · `xgen-common` · any `.rs` · **`skin.css` (Joe's file, never folded into a Chat or Clair commit)** · `ui/core/**` (would move the catalogue) · `dm-spaces.svelte` · `spaces-panel.svelte` · `layout-default.ts` · `app_client.svelte` · `M-RP-INTRO` (flagged, not started) · `M-RP-BLOCK` · the Round-2 whole-codebase audit.

---

## §7 — WHERE THIS DOCUMENT IS MOST LIKELY WRONG

1. 🛑 **`F2` IS THE LOAD-BEARING FINDING AND IT RESTS ON A READ OF TWO DOCUMENTS, NOT ON JOE'S INTENT.** If Joe always meant §6's Leg `F` to be the milestone close and Leg E's `E-5` to be only Leg E's close, then **J2 is right and this document's recommendation is wrong.** *Stated first because it is the one place a wrong answer changes the shape of the close rather than a detail of it.*
2. ⚠️ **`F4`'s probe is priced as one session and this arc has mis-priced a probe three times.** The restore half is the piece most likely to be larger than stated — and it is the half that touches Joe's disk.
3. **`F5`/`§4 ③` reads `R-2` literally.** If the project's practice is that a ⏸️ leg with a named discharger does not block a parent's ✅, then `R-2`'s last sentence is narrower than it reads and **N1/N2/N3 is a question that need not be asked.** *Nobody has been asked; the rule as written is what is cited.*
4. **`F8`'s ledger was built from `ROADMAP.md`'s `Owes:` lines and Leg E's Phase-0. It is a census, and a census is not a partition** — the arc has been bitten by exactly that twice. **Items owed by `M_RP_MEMBER_ACT_PHASE0.md` §5's `OQ5` and by the Leg C / C-bis task docs were read but not re-censused predicate-wise.**
5. **Nothing in this document was measured on the LIVE client**, although the client is up. Every finding is a read of the tree at `0860097` plus three port probes. **`F4` is the one that most needs driving, and driving it is `§4 ②`.**
6. **This document has not been read by anyone outside its author.** ⚠️ *Every real defect in this arc came from OUTSIDE the text — Joe's recall, Clair reading, or the live client. Chat's own re-reads passed every time.*
