# M-RP-IDENTITY-RESOLUTION Leg C — the skin
> **Status**: PENDING  
> Version: 1.0  
> Date: Aug 2026  
> **Last updated**: 2026-08-01  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

🛑 **AUTHORED, NOT LOCKED. NOBODY IS IMPLEMENTING THIS FILE.** It becomes Clair's instruction only when Joe says **lock** AND stands her up — **two acts**, and *"locked" reads like "started" to anyone skimming* (J-646, J-652).

📌 **Parent Phase-0:** `tasks/M_RP_IDENTITY_RESOLUTION.md` **v1.11**. 🛑 **RUNBOOK-AS-GROUND-TRUTH IS A FAILURE MODE.** 📌 *Written against v1.10 and re-pointed to v1.11 in the same authoring pass, because §8's Leg C entry — this runbook's own parent section — is what v1.11 rewrote. `D-135` §5a's pass 2, hit on the first try: **two documents that cite each other cannot be bumped in one pass.***

**SESSION-OPEN READING ORDER (Clair):** ① `CLAUDE.md` PLAY head → ② latest `JOURNAL.md` entry → ③ Phase-0 §3 (the four states) · §4 · §5 · §5a · §5a-i · §5b · §8 Leg C · §11 → ④ **this file**. It is item 4, not item 1.

---

## §0 — 🔒 TWO SEAT RULINGS THIS LEG RESTS ON (Joe, 2026-08-01)

🔒 **RULING 1 — LEG C SPLITS. THE GATE IS ASYMMETRIC AND ONLY HALF OF IT BINDS TODAY (option S3).**

Phase-0 §11 and Leg B's own §9 both say: *"§4's dimming must not SHIP before a refresh trigger exists, or the panel promises a resolution it cannot deliver."* **G-B is open.** The `docs/ROADMAP.md` Leg C node said the opposite — `↳ trigger: Leg B has landed — fired`. **The two disagreed and the disagreement was real, not a wording slip.**

🔑 **THE WAY OUT IS THAT THE GATE DOES NOT REACH BOTH STATES.** ④'s dimming says ***not yet*** — a **transient** claim, and a transient claim commits the panel to eventually resolving (§4c-i). **③'s mark says *gone* — a TERMINAL claim under §1 G3, and it promises nothing.** A rule that makes no promise cannot break one.

⇒ **C-1 + C-2 ship now (state ③ only). C-3 is gated on Leg E and does not exist as an instruction yet.**

🔒 **RULING 2 — THE DEFAULT VALUES ARE DELEGATED TO CHAT FOR THIS LEG ONLY, AND THE STANDING RULE IS UNCHANGED.**

Joe, 2026-08-01: *"normally i have skin.css, this rule still stays, especially when we build complex components. but those are small elements that are not worthy obvious workflow."* ⇒ **`ui/assets/skin.css` REMAINS JOE'S FILE as a standing rule.** This leg is a **narrow carve-out**: two selectors on an existing component, where the round-trip costs more than the decision. **Joe re-tunes any of it in Notepad++ at any time, with no milestone attached and no runbook required.**

⚠️ **CHAT'S FIRST STATEMENT OF THIS WAS WIDER THAN JOE'S** — it read *"Chat now owns the Leg C default values"* and proposed annotating the five documents that say `skin.css` is Joe's. **That would have converted a carve-out into a seat change.** Superseded before anything was written, kept not erased (`D-131`). 🔑 ***A delegation accepted more broadly than it was given is how a seat quietly moves.***

---

## §1 — What this leg does, in one sentence

**Leg B shipped the hook `data-unresolved="unasked" | "erased"` and not one line of CSS; Leg C gives state ③ its mark, gives both states a permanent viewing surface in the sampler, and leaves state ④ untreated until a refresh trigger exists.**

🛑 **AFTER THIS LEG, THE CLIENT STILL LOOKS EXACTLY AS IT DOES TODAY, AND THAT IS THE CORRECT OUTCOME — SAY SO AT THE CLOSE.** Neither ③ nor ④ is reachable with one client (Leg B §6, measured). The visible change lands in the **sampler**, at 9422, where the states can be looked at on demand.

---

## §2 — Files, and the TWO commits

🔒 **SPLIT BY FLOOR, exactly as Phase-0 §8 requires** — *"one commit spanning them makes a regression unattributable."* Leg B proved the split buys what it is for (`06c5afe` / `7e06456`).

| commit | file | floor it moves |
|---|---|---|
| **C-1** | `ui/sampler/src/app_sampler.svelte` | **sampler catalogue** (and `svelte-check`, expected zero) |
| **C-2** | `ui/assets/skin.css` | **NEITHER** — CSS is not type-checked and not a component |

⚠️ **C-1 FIRST.** C-2's rule is unverifiable until something renders `data-unresolved="erased"`, and C-1 is what makes that possible. **Do not batch them.**

🛑 **OUT OF SCOPE, NAMED SO IT IS NOT DRIFTED INTO:**
- **Any `[data-unresolved="unasked"]` rule and any bare `[data-unresolved]` base rule** — that is **C-3**, gated on Leg E (§0 Ruling 1). ***Writing either one now is the defect this leg was split to avoid.***
- `entity-item.svelte` / `entity-panel.svelte` / `members-panel.svelte` — Leg B closed them; **no component change belongs in a skin leg.**
- `.entity-avatar` in any form — the avatar's `[data-revoked]` vocabulary belongs to M13 and `D-127` separates revoked from erased (§5b). **Touching the avatar here is how the two become indistinguishable.**
- The refresh trigger (G-B, Leg E) · the Tier-1 fetch (§7, Leg D) · `M-RP-XGID-SLOT-RETYPE` (`D-136`) · `M_RP_MEMBERS.md` §6a's `tail-8` gap.

---

## §3 — ✅ Grounding (measured 2026-08-01 at `3fe3423`, HEAD = origin/main, tree clean)

Every line number below was read from the file, not recalled.

- **G-C1 — the hook is shipped and unfed.** `entity-item.svelte:57` `unresolved?: 'unasked' | 'erased'` · `:102` `unresolved: unresolved ?? null` in `debug()` · **`:126` `data-unresolved={unresolved}` on the root.** `entity-panel.svelte:42` carries it on `EntityItemInput`, `:162` passes it per row. **`members-panel.svelte:130-133`** supplies `'erased'` from `notFound` and `'unasked'` from `m.unresolved`, ③ before ④.
- **G-C2 — 🛑 `skin.css` CONTAINS ZERO `[data-unresolved]` RULES.** Grepped whole-file. **The only occurrences of the word are `.send-status[data-tone="unresolved"]` at `:3039` and `:3043`** — a **different element and a different attribute**, no cascade contact, no specificity contact. 📌 *Recorded so it is not mistaken for prior art and not "extended" by someone pattern-matching on the word.*
- **G-C3 — the insertion anchor.** `.entity-item:hover` is `:2521-2523`; **`.entity-item[data-selected]` is `:2524-2527`**; `:2528` is blank; `:2529` opens the `status (.status)` comment block. ⇒ **the new rule goes at `:2528`, after `[data-selected]` and before the blank-line boundary of the next block.**
- **G-C4 — 🔑 SPECIFICITY IS EQUAL, SO SOURCE ORDER DECIDES.** `.entity-item[data-selected]`, `.entity-item[data-unresolved]` and `.entity-item[data-unresolved="erased"]` are **all (0,2,0)**. The file's own comment at `:2519-2520` already relies on this — *"selected rule follows hover so it wins at equal specificity."* ⇒ **the erased rule must not set `background`**, or it would silently outrank `[data-selected]` by sitting later. **It sets none.**
- **G-C5 — 🔒 AND `opacity` IS FORBIDDEN ON ③ BY A LOCK, NOT BY TASTE.** `[data-selected]` paints `box-shadow: inset 2px 0 0 var(--accent, var(--pr))` (`:2526`). **§5a-i locked that the erased DM counterpart KEEPS the L16 highlight (Joe, J-650)**; an `opacity` on the root composites the whole subtree **including that inset bar** and would wash out the thing the lock preserves. ⇒ **③ is MARKED, never DIMMED.** *This is also the taxonomy §3 demands drawn in CSS: ④ recedes (our fault, unfinished), ③ asserts (their state, terminal).*
- **G-C6 — the text tokens, one line.** `skin.css:22` — `--t4: #585c64` · `--t3: #8a8880` · `--t2: #c8c4bc` · `--t: #ece9e1`. `.ei-name` today is `var(--t)` inherited from `.entity-item:2440`, `font-weight: 600`, `--fs-1` (`:2452-2459`).
- **G-C7 — the sampler's `entity-item` cells are literal-fed and each yields TWO registry entries.** `app_sampler.svelte:937-939` states it: *"the composite (`entity-item#id`) + its self-registering avatar child (`entity-avatar#id__avatar`)"*. `EntityItem` imported at `:56`, `EntityPanel` panels at `:967-971`; `epSpaces` (3 rows) at `:266-270`. ⇒ **an `entity-panel` cell with N rows costs `1 + 2N` registry entries.**
- **G-C8 — 🔑 HMR HOT-APPLIES `skin.css` IN THE SAMPLER (D-097).** This is what makes the sampler the tuning surface: edit the file in Notepad++, the running sampler repaints without a reload. ⚠️ **N-123 is the standing hazard on the same surface** — an inline style left behind by a probe survives every subsequent HMR edit and **looks like a bug in Joe's own CSS.** Any probe in this leg that touches inline styles ends with `location.reload()`.
- **G-C9 — 🛑 AN ERASED ROW HAS NO DISPLAY NAME TO STRIKE, AND THE FIXTURE MUST SAY SO.** With no book record `toDescriptor` already falls back to `tail(m.identity_id)`, and `tail()` returns the **whole final segment** (`ed25519:<~44 chars>`), which `.ei-name` clips **LEFT-anchored** (`:2452-2458`) ⇒ the row reads `ed25519:AbCd…`. **§5b killed Chat's earlier sketch for striking a name that cannot exist.** ⇒ **the sampler fixture's `name` MUST be an xgid tail, not a person's name.** ***Tuning against "Alice Ng" struck through would be tuning a case the product cannot produce — the exact defect §5b caught, reproduced in the surface built to prevent it.***

---

## §4 — 🔒 THE DESIGN: ④ RECEDES, ③ ASSERTS

🔑 **This is the whole design, and it is Phase-0 §3 restated in CSS.** §3: *"③ and ④ look identical and are opposites — rendering them the same is the panel reporting our network fault as someone else's irregularity."* ⇒ **the treatments must differ in KIND, not merely in degree:**

| | state | what it claims | treatment | ships |
|---|---|---|---|---|
| ③ | erased (`identity.not_found`) | **terminal** — this identity is gone and can never sign an event (G3) | **a MARK**, full opacity, still readable | **C-2, now** |
| ④ | unasked (never looked up) | **transient** — *we* have not asked yet | **DIMMING**, receding | **C-3, after Leg E** |

**The mark for ③ is a strikethrough on `.ei-name`.** The alternatives were each foreclosed by an existing lock, not by preference:

- **A glyph** — `core` owns the glyph NAME (`D-108`) ⇒ a component change, which is out of a skin leg by §2.
- **A word** — re-opens `D-126`'s wordlist, **deferred by Joe at J-588**. §5a's E2 was locked precisely on *"NO NEW WORD REQUIRED — a mark suffices."*
- **`data-revoked`'s greyscale + slash** — belongs to M13, and `D-127` separates revoked from erased ⇒ reuse makes them indistinguishable the day M13 lands (§5b).
- **Opacity** — forbidden by §5a-i via G-C5.

⇒ **strikethrough is what is left after four locks, and it costs one declaration block.**

- ① **User-visible:** **nothing in the client today** — ③ is unreachable with one client (§5b: reachable only on a fresh install / new device / wiped book). **In the sampler, from C-1 on:** an erased row reads as a struck xgid beside unstruck siblings — *this identifier no longer resolves* — while keeping the selection highlight that says whose DM it is.
- ② **Resource:** one CSS block (4 declarations), one sampler cell, zero components, zero Rust.

⚠️ **AND THE HONEST LIMIT, STATED SO THE CLOSE CANNOT OVERCLAIM IT: WHAT IS STRUCK IS A MACHINE IDENTIFIER, NOT A NAME.** Striking `ed25519:AbCd…` reads as *this identifier is dead*, which is true. **It is NOT the struck-display-name row Joe sketched at J-649** — that row was belayed because the name it struck cannot exist. **Chat is not claiming the belayed design has been delivered.**

📌 **FILED, DELIBERATELY NOT TAKEN: the counter-argument for shipping the BASE rule now.** A bare `.entity-item[data-unresolved]` muting `.ei-name` could be defended as *correcting a present falsehood* — a machine identifier is currently rendered at the same weight and colour as a human display name, which is a false equivalence **whether or not a refresh ever arrives**. 🛑 **Not taken, because it also lands on ④, and ④'s treatment is what the gate holds.** Re-openable at C-3, where it belongs. *Recorded rather than silently decided (`D-065`).*

---

## §5 — The changes, exactly

### 🔷 COMMIT C-1 — `ui/sampler/src/app_sampler.svelte`

#### Change 1 — the fixture (in the `entity-panel` fixture block, beside `epRooms` at `:283-287`)

```js
  // unresolved (M-RP-IDENTITY-RESOLUTION §4/§5a) — the two states a member row can be in when the
  // client holds NO identity record. ⚠️ The names are xgid TAILS on purpose: with no book record
  // `toDescriptor` already falls back to `tail(identity_id)`, so a real ③/④ row has NO display
  // name (Phase-0 §5b). A fixture with a human name would be tuning a case the product cannot
  // produce. The middle row is RESOLVED and is the control — a mark is only readable against one.
  const epUnresolved = [
    { descriptor: { kind: 'identity', name: 'Bob Lee', id: 'xgen://identity/bob-9c04', flags: {} } },
    { descriptor: { kind: 'identity', name: 'ed25519:7QmR4xK2vN8pLdA3', id: 'xgen://identity/unasked-1', flags: {} }, unresolved: 'unasked' },
    { descriptor: { kind: 'identity', name: 'ed25519:Zk9WbT5cH1sYnE6f', id: 'xgen://identity/erased-1', flags: {} }, unresolved: 'erased' },
  ];
  let epSelUnresolved = $state('xgen://identity/erased-1'); // §5a-i — the erased row is the DM counterpart, SELECTED
```

🔒 **`epSelUnresolved` PRE-SELECTS THE ERASED ROW ON PURPOSE.** §5a-i locked that the erased DM counterpart **keeps** the L16 highlight. **That lock is unobservable unless something renders the two together** — and G-C5's whole argument for *mark, not dim* is that an opacity would wash the highlight out. ⇒ **the fixture is what makes the lock checkable instead of trusted.**

#### Change 2 — the cell (a new `s-row` in the DD·composite panel, immediately AFTER the `entity-panel · inert` row at `:979-986`)

```svelte
    <div class="s-row">
      <div class="s-rowname">entity-panel · unresolved (M-RP-IDENTITY-RESOLUTION)</div>
      <div class="s-cells">
        <div class="s-cell" style="width: 300px; align-self: flex-start"><span class="s-id">entity-panel#unresolved</span><EntityPanel items={epUnresolved} title="Members" bind:selected={epSelUnresolved} id="unresolved" /></div>
      </div>
    </div>
```

🔑 **ONE PANEL CELL, NOT TWO STANDALONE `entity-item` CELLS — AND THE REASON IS THE JUDGEMENT BEING MADE.** Dimming and muting are **relative**; a row in isolation cannot show whether a treatment reads as *different from its neighbours* or merely as *small*. **The panel puts a resolved control row directly above both marked rows, at `variant="row"`, which is exactly what `members-panel` renders.** *Two isolated edge cells would show the treatment and hide the comparison.*

📌 **FILED, NOT BUILT: standalone `entity-item#unresolved-*` edge cells.** They would show each row at an unconstrained width without panel chrome. **Marginal against the panel cell, and each costs 2 more registry entries.** Add them if Joe finds the panel width limiting when he tunes.

⚠️ **NO `title` BADGE AND NO `badge` PROP ON THIS PANEL.** `members-panel` passes neither (`:146`), and adding one here would put a rendered count in the sampler — **the nearest thing to C1's trigger that has ever existed.** ✅ It still would not fire it (C1 re-opens on *the first milestone that renders a member count* **in the product**, not in a catalogue), **but the distinction is too fine to leave to a future reader.** Do not add one.

**Expected catalogue delta, DERIVED HERE AND THEREFORE NOT TRUSTED:** `1` panel + `3 × 2` rows (composite + avatar child, G-C7) = **+7 ⇒ 427 → 434.** 🛑 **THIS NUMBER IS A PREDICTION. MEASURE IT (V2). A mismatch is a FINDING, not an arithmetic slip to be corrected quietly.**

---

### 🔷 COMMIT C-2 — `ui/assets/skin.css`

#### Change 3 — the ③ mark (insert at `:2528`, after the `[data-selected]` block, before the blank line that opens the `status` comment)

```css
/* unresolved — what the CLIENT knows about this identity, never a property of the entity
 * (M-RP-IDENTITY-RESOLUTION §4/§5a). The hook is VALUED: `erased` = state ③, the node answered
 * `identity.not_found` ⇒ the identity is GONE and can never sign an event again (§1 G3) — a
 * TERMINAL fact. Only ever rendered as the DM counterpart (§5a E2); §5 hides ③ everywhere else.
 *
 * MARKED, NEVER DIMMED, and that is a lock not a taste: §5a-i keeps the L16 selection highlight
 * on this row, and an `opacity` here would composite the `[data-selected]` inset bar away with
 * everything else. No `background` either — this rule sits AFTER `[data-selected]` at EQUAL
 * specificity (0,2,0), so any background it set would silently outrank the selection.
 *
 * ⚠️ What is struck is an xgid TAIL, not a display name — an erased row has no name to strike
 * (§5b). ⚠️ `[data-unresolved="unasked"]` (state ④, DIMMED) is DELIBERATELY ABSENT: it must not
 * ship before a refresh trigger exists, or the panel promises a resolution it cannot deliver
 * (Phase-0 §11, G-B). It lands with Leg C-3, and its BASE rule must be placed ABOVE this one. */
.entity-item[data-unresolved="erased"] .ei-name {
  color: var(--t2);
  text-decoration: line-through;
  text-decoration-color: var(--t4);
  text-decoration-thickness: 1px;
}
```

**Why each declaration, so Joe can re-tune any of them knowing what it was for:**

- **`color: var(--t2)`** — one step down from `.entity-item`'s `--t`, **not two.** `--t3` is `.ei-secondary`'s tone and would read as *this row is subordinate*; ③ is the opposite — in a DM it is the **most** important row. **Muted enough to say *not current*, bright enough to stay the counterparty.**
- **`text-decoration: line-through`** — the mark. No glyph (`D-108`), no word (`D-126`), no avatar change (`D-127`).
- **`text-decoration-color: var(--t4)`** — the dimmest text token, **already the slash colour on `.entity-avatar[data-revoked]:2411`.** 📌 **Deliberately reusing a value the file already spends on *"this identity is not usable"* — the same visual weight, on a different element, for a related-but-distinct fact.** ⚠️ *It does NOT reuse the `[data-revoked]` SELECTOR, which §2 forbids; it reuses one colour.*
- **`text-decoration-thickness: 1px`** — pinned so the strike does not scale with `--fs-1` when Joe retunes type. *The revoked slash is a hand-pinned 2px band for the same reason.*

🛑 **THIS COMMIT ADDS NOTHING ELSE. NO `[data-unresolved]`, NO `[data-unresolved="unasked"]`, NO `.entity-avatar` RULE.** If any of those feels obviously missing while implementing, **that feeling is C-3 and Leg E, and it is the thing the split exists to hold back.**

---

## §6 — Verification

**Static, per commit — every figure RE-DRIVEN by Chat, none read off a report (Rule 5):**

| # | gate | expected |
|---|---|---|
| V1 | `git diff --stat` C-1 | **1 file** (`app_sampler.svelte`); no `skin.css`, no `core`, no `.rs` |
| V2 | sampler catalogue | **MEASURED as a TRANSITION in one session** — pre-C-1 count, apply, HMR-reload, post count. Prediction **427 → 434**; ⚠️ **a mismatch is a finding.** 🛑 *The `328` in older PLAY blocks is from the M-RP6.1 arc and is NOT a baseline (J-653).* |
| V3 | `svelte-check` | from the floor **0 / 34 / 15**; any delta explained, not absorbed |
| V4 | `git diff --stat` C-2 | **1 file** (`skin.css`), **+~20 / −0**; no `.svelte`, no `.ts`, no `.rs` |
| V5 | `svelte-check` after C-2 | **0 / 34 / 15 UNCHANGED** ⇒ *proves* C-2 shipped no component change |
| V6 | sampler catalogue after C-2 | **434 UNCHANGED** ⇒ *proves* CSS moved no registry |
| V7 | `cargo test --workspace` | **NOT RUN — zero `.rs` in either commit, by scope (`git show --stat`).** 🔒 **Floor stays 1589 / 0 / 62 × 56 and is DELIBERATELY NOT RE-MEASURED**, stated rather than silently skipped |
| V8 | `git ls-files --eol` | **`i/lf` on both files** |

**Live (CDP, sampler 9422) — this is the leg where the sampler IS the evidence:**

| # | gate | expected |
|---|---|---|
| V9 | `__XGEN_DEBUG__.get('unresolved__item-1').state.unresolved` (and `-2`) | **`'unasked'` and `'erased'`** — 🔑 **the FIRST time in this milestone either value has been read off a live component.** ⚠️ Row ids are the panel's own composition — **enumerate `ids` and read the real ones; do not assume the pattern.** |
| V10 | `[data-unresolved]` attribute census across all `.entity-item` | **exactly TWO present** (`"unasked"`, `"erased"`), **absent on every other row in the sampler and on all 5 client rows.** *The negative half is what Leg B proved; V10 is the positive half arriving.* |
| V11 | computed style on the erased row's `.ei-name` | `text-decoration-line: line-through`, colour resolving to `#c8c4bc`. 🛑 **Read the COMPUTED style, not the rule text** — a rule present in the stylesheet and losing the cascade reads identically in a diff. |
| V12 | 🔒 **§5a-i — the erased row's `box-shadow`** | **`inset 2px 0 0 …` PRESENT and non-`none`**, with `data-selected` on the same element. 🔑 **This is the gate the whole *mark-not-dim* argument stands on** (G-C5); if it fails, the design is wrong, not the fixture. |
| V13 | the **unasked** row | **NO strikethrough, NO dimming, NO opacity change — visually identical to the resolved control row.** 🔑 **The positive proof that C-3 was withheld**, read off the DOM rather than off the diff. |

⚠️ **AND THE SURFACE INVERSION IS DELIBERATE — IT DOES NOT CONTRADICT J-653.** J-653 established *the client is the evidence, the sampler is not*, for **verifying a wired path**. **That still holds and Leg F still owns it.** **This leg changes no path** — it changes an appearance, and **the sampler is the only surface on which either state can be made to render at all.** ***A sampler row proves the component accepts the value and the skin paints it; only a client row proves the store delivers it — and this leg makes no claim about the store.***

🛑 **WHAT THIS LEG DOES NOT PROVE, STATED SO THE CLOSE CANNOT DRIFT:** that ③ ever renders **in the client** · that the DM exception fires against a **real** `not_found` · that anything at all is true of state ④. 🔒 **Leg F remains the first behaviour verification of this milestone.**

🛑 **ANNOUNCE THE RUN AND ASK FOR HANDS OFF (`D-132`, `CDP_DEBUG_HARNESS.md` v1.8).** The sampler must be launched with `run-sampler.ps1 -Debug` — **`cdp-debug.ps1` ATTACHES, IT DOES NOT LAUNCH** — and Chat says when the run starts and when it is done. ⚠️ **The script writes with `Write-Host`, which bypasses the pipeline: capturing its output into a variable yields EMPTY** (J-653). ⚠️ **Any probe touching inline styles ends with `location.reload()`** (N-123).

---

## §7 — DoD

- [ ] **C-1 committed alone**; sampler catalogue **measured as a transition, not derived** (V1, V2)
- [ ] `svelte-check` re-measured after C-1, **delta explained** (V3)
- [ ] **C-2 committed alone**; `svelte-check` and catalogue both **asserted UNCHANGED** (V4, V5, V6)
- [ ] 🔒 **`cargo` NOT re-measured, and the close SAYS SO** with the by-scope proof (V7)
- [ ] `git ls-files --eol` **`i/lf` on both files** (V8)
- [ ] **Both hook values read live off the painted DOM** — the first positive read in this milestone (V9, V10)
- [ ] **The mark verified in COMPUTED style, not in the stylesheet text** (V11)
- [ ] 🔒 **§5a-i's highlight proven to SURVIVE the mark** (V12) — the gate the design rests on
- [ ] 🛑 **The unasked row proven UNTREATED** (V13) — C-3's withholding is verified, not assumed
- [ ] 🛑 **The close states plainly that the CLIENT looks unchanged, and that this is correct** (§1)
- [ ] 🛑 **The close states plainly that ③ was still not exercised against a real `not_found`** — Leg F's, untouched
- [ ] `docs/ROADMAP.md`'s Leg C node **corrected**: the `↳ trigger: Leg B has landed — fired` line is a defect by the standing convention (*a trigger that has fired is a defect*) and the node splits C-1/C-2/C-3 with C-3's trigger named as **Leg E**
- [ ] `.md` header updated on every touched document: **Version bumped · `Last updated` = the date CONTENT changed · TWO trailing spaces on every `> ` line**
- [ ] Records: JOURNAL + `CLAUDE.md` PLAY + `docs/ROADMAP.md` + Phase-0 + this file **in one commit** (`D-074`)
- [ ] Citation sweep run **on the NAME, not the version string — and run TWICE** (`D-135` §5a). 🛑 **Pass 2 has been required FOUR consecutive entries. Treat it as certain, not likely.**

---

## §8 — Seats (D-123)

- **JOE** — **owns `ui/assets/skin.css` as a standing rule (§0 Ruling 2)**; delegated **this leg's two selectors** to Chat and re-tunes them whenever he likes, with no milestone; locks this runbook; stands Clair up; **pushes all commits**.
- **CHAT** — authored this file and its default values; re-drives **every** gate in §6; owns the records. **Never pushes.**
- **CLAIR** — implements **from this file**, C-1 then C-2, **once Joe stands her up**. **She does not close her own leg.**
  - 🔑 **Rule 6 stands: flag a deviation, never absorb it.** ⚠️ **This leg carries a specific temptation to absorb — the ④ rule and the base rule will both feel obviously missing.** They are withheld by a ruling (§0 Ruling 1). **If that looks wrong, say so; do not add them.**
  - 🔑 **A CLEAN ADVERSARIAL READ IS A RESULT, NOT AN ABSENCE** — say so explicitly (J-647, J-651).

---

## §9 — Filed, NOT fixed (none of these is Leg C's to close)

- **C-3 — the base rule + `[data-unresolved="unasked"]`.** 🔒 **Gated on Leg E** (a refresh trigger that actually fires). ⚠️ **Placement note for whoever writes it: the base rule must sit ABOVE the erased rule** — equal specificity (0,2,0), source order decides (G-C4).
- **`M_RP_MEMBERS.md` §6a — the `tail-8` lock-versus-build gap.** `.ei-name` is LEFT-anchored and clips the RIGHT, so every unresolved row keeps the constant `ed25519:` head and loses the distinguishing bytes ⇒ **two unresolved rows are indistinguishable from each other.** ⚠️ **This leg's strikethrough makes the gap MORE visible, not less** — a struck constant prefix. **Not fixed here; it is Joe's, filed at J-618.**
- **`entity-avatar.svelte:125` collapses `isAi`'s third state** — `data-ai={flags.isAi || undefined}`. Joe's, same family as §4.
- **M13 §3c — erasure is invisible to anyone holding a cached record.** The real defect J-649 uncovered; **must be designed with M13's `revoked` + `update_version`, or not at all.**
- **G-B — "the next refresh" does not arrive.** The gate this leg is split around. **Leg E.**
- **`M-RP-XGID-SLOT-RETYPE` (`D-136`)** — untouched here.
