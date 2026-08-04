# RUNBOOK — M-RP-IDENTITY-RESOLUTION Leg C-3 — the base rule and the unasked skin
> **Status**: ACTIVE  
> Version: 1.4  
> Date: Aug 2026  
> **Last updated**: 2026-08-04  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — What this is, the seat, and the seat change that made it writable

**Implementation runbook for `M-RP-IDENTITY-RESOLUTION` Leg C-3** — the last third of Leg C, filed at J-655 in `tasks/RUNBOOK_IDENTITY_RESOLUTION_LEG_C.md` §9 and gated on Leg E. **Leg E discharged J-670; `G-B` closed J-672.** ⇒ **the gate is gone and so is its reason.**

🔒 **SEAT — `D-138` (Joe, 2026-08-04), MINTED FROM THIS LEG AND STANDING FOR ALL SKIN WORK.** `skin.css` **remains Joe's file.** What is delegated is **SYNTAX, not appearance**: attribute selectors, custom properties and the cascade rules that decide which of two equal-specificity rules wins are outside his current working CSS vocabulary — *"i ended working with css maybe on v3"*. **He is not short of an opinion about how the row should look; he is short of the notation to write it in.**

⇒ ***CHAT SUPPLIES THE MECHANISM. THE VALUES ARE JOE'S.*** The block ships **working and correct so that it can be EDITED**, not so that it can be approved — *"i intention is to take your lines later and modify them to be better, or leave them as you wrote."*

🛑 **v1.0 OF THIS RUNBOOK RECORDED IT AS *"appearance proposals ship as proposals; Joe reviews after"*. THAT WAS WRONG AND IS KEPT, NOT ERASED (`D-131`).** It put the aesthetic choice with Chat until Joe blessed it, which is neither what was delegated nor what he said.

⚠️ **WHAT THE CLOSE MAY CLAIM (`D-138` §4):** **the mechanism is verified; the values are Joe's and unreviewed.** 🛑 **NOT *"verified"* flat** — a computed-style read proves a rule APPLIES and cannot prove it LOOKS RIGHT. ⚠️ *And not "pending Joe's approval" — nothing is pending; the mechanism is done and the values are in the seat they belong to.*

🔑 **THE REASON IS NOT TASTE, IT IS SIGHT (`D-138` §4).** *Choosing the right weight for a line of text requires looking at it, and only one of us looks.* ⇒ 🔒 **A VALUE JOE CHANGES IS NOT AN ERROR OF CHAT'S — IT IS THE HANDOFF WORKING**, and it is **not** recorded as a defect, annotated as a superseded claim, or counted in any error tally. 🛑 **AND THE BLOCK SHIPS WITH REAL VALUES, NEVER A `TODO`:** something that does not render cannot be looked at, so **the provisional values are load-bearing — they are what makes the block correctable at all.**

⚠️ **The structural half was always technical and stays Chat's:** which selector, source placement, the comment sweep.

🛑 **v1.3 AUDITED THIS DOCUMENT AGAINST `D-138` AND OVERSHOT; v1.4 STRIPS THAT BACK ON JOE'S NARROWING** — *"it is enough that your definitions will be regular ones, like you would done them autonomously, no some special additions."* ⚠️ **v1.1–v1.3 added marked DIAL blocks, alternative-value menus and escape hatches to every rule. Superseded, kept not erased (`D-131`).** 🔑 **The error was conflating *cannot write it* with *cannot read it*** — the gap Joe named is **authoring** a selector, not reading `color: var(--t3)` and changing the token. ⇒ **`D-138` §3 now reads: write them normally, in the file's own voice.**

✅ **TWO OF v1.3's THREE FINDINGS SURVIVE THE STRIP, BECAUSE THEY WERE NEVER ABOUT ANNOTATION:**

1. 🛑 **Step C3-2 CONTAINED A BLANK, IN JOE'S FILE** — *"replace with a line naming where the unasked rule now lives"* is **prose for Clair to invent, in a comment Joe reads.** `D-138` §1.5: a scaffold ships real, never `TODO`, and **an instruction to write unspecified prose is a `TODO` wearing a sentence.** ⇒ **the replacement sentences are now written out verbatim.**
2. 🛑 **`R-C3-2`'s REASON WAS A CLAIM NARROWER THAN THE THING IT DESCRIBED.** It refused `opacity` *"because `C3-G5` locks it out"* — but `C3-G5` locks out `opacity` **on the ROOT**, and this rule targets `.ei-name`, where it would be legal. **Refused for a different reason, which the ruling did not state.** Corrected in §3.

⚠️ *v1.3's third finding — that the erased rule "needs dials too" — is **withdrawn with the doctrine that produced it** (`R-C3-5`, dropped).*

🛑 **NOT GROUND TRUTH.** Reading order is `CLAUDE.md` PLAY head → latest JOURNAL entry → ACTIVE handoffs in `tasks/` → **then** this. 🔑 **READ §5 BEFORE IMPLEMENTING, AND IT IS NOT A CENSUS OF THIS DOCUMENT'S ERRORS.**

---

## §1 — Grounding

**Measured 2026-08-04 at `7a27140`** (= `git ls-remote origin main`, tree clean). Read windows stated.

| # | fact | citation | window |
|---|---|---|---|
| **C3-G1** | The tone ramp is **four steps on one line**: `--t4: #585c64` · `--t3: #8a8880` · `--t2: #c8c4bc` · `--t: #ece9e1` | `skin.css:22` | grep |
| **C3-G2** | `.entity-item` root sets `color: var(--t)`; `.ei-name` sets **`font-weight: 600`** and no colour of its own | `skin.css:2464`, `:2475–2482` | 2455–2582 |
| **C3-G3** | 🔑 **`--t3` ALREADY MEANS "SUBORDINATE, NOT THE HEADLINE" INSIDE THIS COMPONENT** — it is the colour of both `.ei-secondary` and `.ei-meta` | `skin.css:2485`, `:2496` | 2455–2582 |
| **C3-G4** | `.entity-item[data-variant="inline"] .ei-name` uses **`font-weight: 500`** — the file's existing step for *a less assertive name* | `skin.css:2538–2541` | 2455–2582 |
| **C3-G5** | 🛑 **`opacity` appears 24 times in `skin.css` and NEVER on the entity family** — the C-2 comment gives the locked reason: an opacity on the ROOT would composite `[data-selected]`'s inset bar away with everything else (§5a-i) | `skin.css:2557–2560` | grep + 2455–2582 |
| **C3-G6** | The erased rule is scoped to `.ei-name`, at **`:2572`**, setting `color: var(--t2)` + `line-through` + `--t4` rule + `1px` | `skin.css:2572–2577` | 2455–2582 |
| **C3-G7** | The C-2 comment asserts `[data-unresolved="unasked"]` is **"DELIBERATELY ABSENT"** and that its **BASE rule must be placed ABOVE** the erased rule | `skin.css:2569–2571` | 2455–2582 |
| **C3-G8** | 🛑 **A STALE CITATION IN THAT COMMENT.** `:2564` reads *"Nothing later in this file reaches `.ei-name` (only `:2452` and the inline-variant rule above)"*. **Measured: `.entity-item .ei-name` is at `:2475`.** `:2452` matches nothing relevant | `skin.css:2564` vs `:2475` | 2455–2582 |
| **C3-G9** | The sampler fixture rows exist and are the surface: `entity-item#unresolved-unasked-1` and `-erased-1`, added by C-1 (catalogue **427 → 435**, measured as a transition) | Leg C runbook V2, V9 | 215–232 |

---

## §2 — What changes. **ONE FILE. ONE COMMIT.**

`ui/assets/skin.css` only. **No `.svelte`, no `.ts`, no `.rs`.**

### Step C3-1 — the two rules, placed **after the `[data-selected]` rule (closes `:2550`) and BEFORE the erased rule's comment block (opens `:2552`)**

📌 **That placement is exact for a reason:** the comment at `:2552–2571` belongs to the erased rule, so the new rules go **above the comment**, not between the comment and its rule. ✅ *It also leaves `.ei-name` at `:2475` unmoved, which is what Step C3-2's corrected citation depends on.*

```css
/* base — both unresolved states: what sits in `.ei-name` on those rows is an xgid TAIL, not a
 * display name, so it does not carry a name's weight. `500` is the step the inline variant above
 * already uses. Scoped to `.ei-name` for the same reason the erased rule is: an `opacity` on the
 * ROOT would composite the `[data-selected]` inset bar away with everything else (§5a-i). It sets
 * weight ONLY, so it shares no property with either state rule and their source order carries no
 * meaning between them. */
.entity-item[data-unresolved] .ei-name {
  font-weight: 500;
}

/* unasked — state ④: the client has not looked this identity up yet. The row RECEDES and asserts
 * nothing; `--t3` is already this component's supporting-text tone (`.ei-secondary`, `.ei-meta`).
 * No decoration — a mark is a CLAIM, and ④ is the absence of one, which is the whole difference
 * from `erased` below, a terminal fact and therefore marked.
 *
 * ⚠️ Honest only because G-B closed (J-672): dimming says *not yet*, and *not yet* commits the
 * panel to eventually resolving. Leg D fetches on join (the ordinary path), Leg E re-fills on
 * reconnect (the exceptional one). Shipping this before both existed was the defect Leg C was
 * split to avoid. */
.entity-item[data-unresolved="unasked"] .ei-name {
  color: var(--t3);
}
```

### Step C3-2 — the comment sweep, **in the same edit** (`N-109` pre-empt, filed at J-655)

🛑 **TWO SURGICAL EDITS IN THE BLOCK AT `:2552–2571`. Do not rewrite the block.**

**(1) The `N-109` retirement.** Replace the last four lines (`:2568–2571`) with:

```css
 * ⚠️ What is struck is an xgid TAIL, not a display name — an erased row has no name to strike
 * (§5b). `[data-unresolved="unasked"]` (state ④) shipped at Leg C-3 and sits above this rule with
 * the shared base rule; it became honest when G-B closed (J-672), a refresh now arriving on both
 * the ordinary path (Leg D's fetch on join) and the exceptional one (Leg E's reconnect). */
```

🔑 **The sentence it replaces read *"`[data-unresolved="unasked"]` … is DELIBERATELY ABSENT"*.** The moment C3-1 lands that is **false, in Joe's file, written by someone being careful — which is exactly why the next reader would trust it.** The removal was written into C-3's DoD at J-655 as an `N-109` pre-empt; **this is that obligation being paid.**

**(2) The stale citation.** On `:2564`, `:2452` → **`:2475`** (`C3-G8`). ⚠️ *An erroneous line number, not a superseded claim — it is corrected rather than annotated, and the correction is recorded in the close. `D-131` protects claims that were once true; `:2452` never was.*

🔑 **RE-DERIVE `:2475`, DO NOT COPY IT.** `C3-G8` measured it at `7a27140`, and Step C3-1 inserts **below** `.ei-name`, so it should not have moved — **confirm that rather than trusting this sentence.** *The whole reason the line is being corrected is that someone once wrote a number here without re-deriving it.*

---

## §3 — Chat's rulings for this leg (`D-123`)

- 🔒 **R-C3-1 — the base rule sets WEIGHT ONLY.** It could have carried the colour and let each state override; it does not, because that is what creates the (0,3,0) fight G-C4 flagged, **and removing a hazard beats navigating one.** 📌 *`D-138` §3's surviving half — few values, isolated, one place each — points the same way, but the source-order reason is the one that decides it.*
- 🔒 **R-C3-2 — `--t3`, not a new token and not `opacity`.** 🛑 **CORRECTED AT v1.3; v1.0–v1.2 refused `opacity` *"because it is locked out by `C3-G5`"*, and that reason was a claim narrower than the thing it described (`D-131`, kept not erased).** `C3-G5` locks out `opacity` **on the ROOT** — it would composite `[data-selected]`'s inset bar away. **This rule targets `.ei-name`, where an `opacity` would be perfectly legal.** ⇒ **the real reasons, stated:** ① `opacity` would produce a tone that is **not on the ramp**, and the ramp is the vocabulary the rest of this file speaks — a value Joe can reason about and step through; ② a new token adds vocabulary for **one use**. 🔑 *The conclusion was right and the grounding was not, and those are separable — which is what made it survive two revisions.*
- 🔒 **R-C3-3 — ④ gets NO `text-decoration`.** The two states must not read as degrees of the same thing. ③ is a claim, ④ is the absence of one.
- 🛑 **R-C3-4 — THE BASE RULE CHANGES THE ERASED ROW TOO, AND THAT IS DELIBERATE.** `.ei-name` is `font-weight: 600` (C3-G2); the base rule moves **both** unresolved states to `500`. ⚠️ **This is a visible change to a row that already shipped and was verified at J-655** — V11's assertions (line-through, `--t2`) are untouched, but the weight moves. **It is by design** (both rows show xgid tails, and C-2's own comment presupposes a base rule reaching the erased row) **and it is stated here rather than discovered at V-time.**

⚠️ *v1.3 carried an `R-C3-5` obliging the erased rule to gain "dials". **Withdrawn at v1.4 with the doctrine that produced it** (`D-138` §3, narrowed by Joe); kept not erased (`D-131`).*

---

## §4 — Verification gates

| # | gate | expected |
|---|---|---|
| **W1** | `git diff --stat` | **1 file** (`ui/assets/skin.css`), **+~26 / −5**. No `.svelte`, no `.ts`, no `.rs` |
| **W2** | `svelte-check` | **0 / 34 / 15 UNCHANGED** ⇒ *proves* C-3 shipped no component change |
| **W3** | sampler catalogue | **435 UNCHANGED** ⇒ *proves* CSS moved no registry |
| **W4** | `cargo` | 🔒 **NOT RUN — zero `.rs` by scope** (`git show --stat`). Floor stays **1596 / 0 / 62 × 56** and is **deliberately not re-measured, stated rather than silently skipped** |
| **W5** | `git ls-files --eol` | **`i/lf`** |

**Live (CDP, sampler `9422`) — the sampler is the evidence for the same reason it was at J-655: this leg changes an appearance, not a path.**

| # | gate | expected |
|---|---|---|
| **W6** | enumerate ids, then `__XGEN_DEBUG__.get('entity-item#unresolved-unasked-1').state.unresolved` | **`'unasked'`** ⚠️ **enumerate — do not assume the id pattern** (the J-655 F9 lesson) |
| **W7** | 🔑 **computed style on the unasked row's `.ei-name`** | `color` resolving to **`rgb(138, 136, 128)`** (`--t3`) · `font-weight` **`500`** · `text-decoration-line` **`none`**. 🛑 **COMPUTED, not rule text** — a rule present in the stylesheet and losing the cascade reads identically in a diff |
| **W8** | 🔒 **the unasked row's ROOT is untouched** | `opacity` **`1`**; `.ei-secondary` and `.ei-meta` colours **unchanged from the control row**. *This is the gate the scoped-to-`.ei-name` argument stands on* |
| **W9** | 🛑 **the ERASED row still marked, and its weight moved** | `text-decoration-line: line-through` **still present**, colour still **`rgb(200,196,188)`**, rule still **`rgb(88,92,100)`** — **and `font-weight` now `500`** (R-C3-4, expected, not a regression) |
| **W10** | 🔒 **§5a-i survives** — the erased row's `box-shadow` with `data-selected` | **`inset 2px 0 0 …` PRESENT and non-`none`.** 🔑 *Re-run because C-3 adds a rule to the same element family; the mark-not-dim argument must not quietly break in the leg that finishes the split it was made for* |
| **W11** | the **resolved control row** | `.ei-name` **`font-weight: 600`**, colour inherited, no decoration ⇒ **the change is confined to rows carrying the hook** |

🛑 **ANNOUNCE THE RUN AND ASK FOR HANDS OFF (`D-132`).** Launch with `run-sampler.ps1 -Debug` — **`cdp-debug.ps1` ATTACHES, IT DOES NOT LAUNCH.** ⚠️ The script writes with `Write-Host`, which bypasses the pipeline: capturing its output into a variable yields **empty** (J-653). ⚠️ Any probe touching inline styles ends with `location.reload()` (`N-123`).

⚠️ **WHAT THIS LEG DOES NOT PROVE:** that state ④ ever renders **in the client** · that a real joiner ever carries the hook · **that any of this looks right.** 🔑 **The first two are Leg F's. The third has no instrument and belongs to Joe's eye, after the fact, under the §0 seat change.**

---

## §5 — 🛑 WHERE THIS RUNBOOK IS MOST LIKELY WRONG

⚠️ **NOT A CENSUS OF ITS ERRORS — only the doubts its author already had.** Check the producer, not the name; refuse the step if they disagree.

- **(a) 📌 `--t3` ON A NAME MAY SIMPLY BE TOO FAINT — AND THAT IS NOT A DEFECT IN THIS DOCUMENT.** `#8a8880` is a real drop from `#ece9e1`, and **no gate here can settle it** (`D-138` §4: the reason is sight, not taste). 🔑 *It is a scaffold value, shipped real rather than blank so the row renders and can be looked at. **A change to it is the handoff working, not a correction owed.***
- **(b) The `+~26 / −5` in W1 is an estimate**, not a measurement — Step C3-1 adds ~22 lines and Step C3-2 rewrites four. **A mismatch is not a failure; an unexplained one is.**
- **(c) `rgb(138, 136, 128)`** is `#8a8880` converted by hand. **Verify against the computed value the browser reports**, not against this line.
- **(d) The claim that nothing else in the file reaches `.ei-name`** comes from the C-2 comment — **which is the same sentence carrying the stale `:2452` (C3-G8).** ⚠️ ***A comment that is wrong in one clause has not earned trust in its others.*** **Re-sweep `.ei-name` across `skin.css` independently before believing it.**
- **(e) Placement.** Stated as *after the `[data-selected]` rule closing `:2550`, before the erased comment opening `:2552`* — **read at `7a27140`**. ⚠️ *v1.0–v1.2 said "between `:2541` and `:2572`", which would have allowed the rules to land BETWEEN the erased comment and its own rule; corrected, kept not erased.* **Re-derive; line numbers move.**
- **(f) R-C3-4's weight change on the erased row** is argued from the C-2 comment presupposing a base rule. **If the erased row looks wrong at `500`, the base rule is the thing to reconsider — not the unasked rule.**

---

## §6 — Scope: what must NOT be touched

- ❌ Any `.svelte`, `.ts` or `.rs`. **C-3 is one CSS file.**
- ❌ The erased rule's own declarations (`:2572–2577`) — only the *shared* weight moves, and it moves from the base rule.
- ❌ `.entity-avatar`'s `data-ai={flags.isAi || undefined}` third-state collapse — **filed at J-655, Joe's, not this leg.**
- ❌ `M_RP_MEMBERS.md` §6a's `tail-8` lock-versus-build gap — `.ei-name` clips **left-anchored**, so unresolved rows keep the constant `ed25519:` head. ⚠️ **This leg makes that gap MORE visible, not less** — a receded constant prefix. **Filed at J-618, Joe's.**
- ❌ `roomLatch.effectiveSpaceId` (`N-169`) · R4 · `ingest.push` · the address-book setters.

---

## §7 — DoD (Leg C-3)

- [ ] Both rules land in `ui/assets/skin.css`, **above** the erased rule
- [ ] 🛑 **Step C3-2's two surgical edits made** — the *"DELIBERATELY ABSENT"* sentence retired verbatim as written (`N-109`, J-655's obligation), and the stale `:2452` corrected to `:2475`, **re-derived not copied**
- [ ] W1…W5 green; `cargo` **not run, and the close says so** with the by-scope proof
- [ ] W6…W11 green on the sampler, **computed style not rule text**, ids **enumerated not assumed**
- [ ] 🔒 **W9 and W10 explicitly re-checked** — the erased row is touched by this leg and must be shown still correct
- [ ] The close records the weight change on the erased row as **expected (R-C3-4)**, not as a delta to explain away
- [ ] 🛑 **The close says: *the mechanism is verified, the values are Joe's and unreviewed* (`D-138` §4).** Never *"verified"* flat, and never *"pending approval"* — the instruments cover application, not appearance
- [ ] `M_RP_IDENTITY_RESOLUTION.md` Leg C node → all three thirds shipped; ROADMAP Leg C 🟢 → ✅
- [ ] Records: JOURNAL + `CLAUDE.md` PLAY + `ROADMAP.md` + the milestone doc + this runbook in one commit (`D-074`)
- [ ] 🛑 **Clair hands back with the numbers. Chat re-drives every gate. Joe pushes.**
