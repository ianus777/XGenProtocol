# M-RP-SELF-VARIANTS — the self name's visual distinction, re-judged against real faces
> **Status**: COMPLETED  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-23  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — What this is, and what it is NOT

**IS:** a [👁️ PERCEPTION] test, re-run because the first run judged the wrong subject. **Output = Joe's chosen values, handed to `M-RP-SELF-SURFACE`.**

**IS NOT:** a fix. ⚠️ **No `skin.css` edit was made and none is authorised here** — `skin.css` is Joe's file, and the landing belongs to `M-RP-SELF-SURFACE`. Zero code, zero commit-bearing source change; every styling change was applied at **runtime via CDP** and dies on reload.

## §1 — Why it exists

Filed at **J-570** (`M-RP-FONTS`). Joe had judged three *"Self"* typeface variants — *"1 system mono"* · *"2 JetBrains Mono"* · *"3 Inter italic + tracking"* — and **all three were fallback fonts.** No mono face was declared, so variant 2 could not have been JetBrains Mono; variant 3's italic was a **skewed Regular**. ***A [👁️ PERCEPTION] verdict is only as good as whether the thing looked at WAS THE SUBJECT.***

🔑 **A distinction the original filing did not draw:** **variant 1 was never corrupted.** *"System mono"* **is** `ui-monospace/monospace`, so it rendered exactly what it claimed. What was invalid was not each reading but **the comparison** — choosing among three when two were impostors is not a choice that was ever made.

## §2 — Phase 0: is the subject real now?

**Four faces declared in `ui/assets/skin.css` and bundled in `ui/assets/fonts/`:** `XGen UI Sans` → `InterVariable.woff2` + **true** `InterVariable-Italic.woff2`; `XGen UI Mono` → `JetBrainsMono-Variable.ttf` + **true** italic. Both gaps M-RP-FONTS was filed to close are closed **on disk**.

## §3 — LEG A: the control, and why it is the milestone's real content

⚠️ **THE TRAP THAT MUST BE DEFEATED:** `getComputedStyle().fontFamily` returns **the declared stack, not the resolved face** — it reports `"XGen UI Sans", system-ui, sans-serif` whether the real face rendered or the system fallback did. **N-156 exactly: a probe that returns the wrong thing confidently is a hole shaped like a result.** That is the most likely route by which variant 2 passed as JetBrains Mono last time.

⚠️ **AND A SECOND ONE, SPECIFIC TO ITALIC:** a browser will **fake italic by skewing the Regular**. A skew and a true italic **both render plausibly**. ⇒ Set **`font-synthesis: none`** and measure **advance width**: with synthesis off and no real face, the text renders **upright** and the width matches the regular.

**Method:** forced `document.fonts.load()` on every face first — *"never requested" and "broken" return the same string* — then off-screen spans at 16px, `font-synthesis: none`, measured by `getBoundingClientRect()`, including a deliberately **bogus family** as the negative reference.

### §3.1 — Results (client at HEAD `e4d9625`, 2026-07-23)

| probe | width px | reads |
|---|---|---|
| `XGen UI Sans` 400 upright | **133.04** | — |
| **bogus family** 400 | **118.80** | ⇒ the real family resolves; **118.80 is what fallback looks like** |
| `XGen UI Sans` **italic** 400 | **133.68** | ⇒ **≠ upright with synthesis off ⇒ a TRUE italic face** |
| 700 upright | **137.40** | — |
| **italic 700** | **138.19** | ⇒ true bold italic |
| **600 upright** | **135.95** | ⇒ ⚠️ **600 has NO static file on disk** yet renders its own width ⇒ **the variable weight axis is real, not assumed** |

**All five `document.fonts.check()` calls returned true AFTER a forced load.** ⇒ **The subject is real. The first run's defect is not present.**

### §3.2 — Fixture, stated so it is not mistaken for the fix

No node was running (`connectionState: RECONNECTING`) and neither seeded room held real messages (`emptyState: "no-messages"`), so the own row was produced by **an actual send** — a real local echo (`isOwn: true`), rendering the real defect: **`nameLen: 65`, the full XGID**. Its text was then set to `"Self"` **at runtime only**. Variants V1–V3 were **DOM clones** of that real row — real elements, real classes, real faces — inserted for side-by-side comparison and removed afterwards.

## §4 — LEG B: Joe's verdict (captured LIVE, 2026-07-23)

**The ladder judged**, all four on screen simultaneously and confirmed distinct by measured width (**18.93 · 20.05 · 21.25 · 20.95** px):

| | weight | style | tracking |
|---|---|---|---|
| V0 | 600 (baseline) | upright | none |
| V1 | 700 | italic | 0.02em |
| V2 | 700 | italic | 0.05em |
| **V3** | **600** | **italic** | **0.05em** |

### 🔒 **VERDICT — JOE, 2026-07-23, verbatim: "v3 + 90% grey colour (90% white in grey)"**, confirmed as applied.

**THE CHOSEN VALUES, for `M-RP-SELF-SURFACE`:**

| property | value | note |
|---|---|---|
| `font-weight` | **600** | ⚠️ **identical to the existing baseline — the weight does NOT change** |
| `font-style` | **italic** | ⚠️ **requires `font-synthesis: none`**, or a skew silently satisfies it |
| `letter-spacing` | **0.05em** | computed **0.5px** at the current 10px |
| `color` | **`#E5E5E5`** | `rgb(229,229,229)` = **90% white** |

🔑 **THE FINDING THAT REFRAMES THE VERDICT: the author name was ALREADY `font-weight: 600` at `font-size: 10px`.** So V3's weight is the baseline's weight, and **the distinction is carried entirely by italic + tracking + brightness — not by weight at all.** ⚠️ It also means *"bold"* was only ever one step (600→700); if more force is ever wanted, **800 and 900 are real on this axis**.

**AND IT FIXED A LEGIBILITY FLOOR NOBODY WAS LOOKING FOR.** The self name rendered at `rgb(88,92,100)` on `rgb(22,24,28)` = **2.65 : 1 contrast at 10px — below WCAG AA (4.5:1)**. The chosen colour takes it to **14.1 : 1**. *The perception question was "which reads as Self"; the answer incidentally repaired a measurable defect that no automated test covers — the J-568 shape again: appearance has no verifier, so appearance defects survive.*

⚠️ **SURFACED TO JOE AND ACCEPTED:** at `#E5E5E5` the name sits **within 3% of the message body** (`rgb(236,233,225)`), so it reads at body strength rather than as a subordinate label. Joe confirmed as applied. **Revisitable in `skin.css`, which is his.**

### §4.1 — ⚠️ THE LIMIT ON THIS VERDICT, RECORDED BECAUSE IT DOES NOT SHOW IN THE RESULT

**There were NO inbound names on screen.** No node, no second identity, no seeded messages ⇒ the self name was judged **in isolation**. 🔑 **This test establishes that the styling reads as deliberate and legible. It does NOT establish that it DISTINGUISHES** — there was nothing present to distinguish it from, and per Joe's own scope ruling the distinction is *from other people's names*. **Closing this milestone as "distinction achieved" would be the J-560 defect in a new place.** Real inbound needs **M-RP6.4 backfill or a second identity**, both already owed.

## §5 — Joe's scope rulings (2026-07-23)

- **The distinction applies EVERYWHERE, not only where avatars gather.** In the message thread it is simply *the only* change, because the avatar's side already carries the rest.
- **Name only** — not the row, not the body.
- **CSS styling first**; a family change to mono is held in reserve and was **not** tested.
- ⚠️ **`semibold` is not a CSS keyword** — only `normal` and `bold` are absolute keywords; everything between exists as a number. 🔑 Joe: *"as for a human, i cannot imagine font weight from number"* ⇒ **weights are presented to him in words, numbers live only in records.** *A question whose answer he cannot perceive is not a perception question.*

## §6 — Filed, NOT fixed

- ⚠️ **THE SELF PANEL ALREADY SOLVES #5, AND THE MESSAGE ROW DOES NOT.** `self-panel.svelte:43` resolves `identity?.display_name` ⇒ renders **"Joe"**. `entity-item.svelte:65` and `message.svelte:69` use `name ?? id` with `name` null (`stream-panel.svelte:115`, *"NO name (C-8)"*) ⇒ render the **65-char XGID**. **Same identity, two surfaces, two answers.** ⇒ `M-RP-SELF-SURFACE`.
- ⚠️ **`region-tile#region-members` EXISTS AS A STUB WITH NO PANEL PLUGIN.** The gathered-avatar surface Joe's reasoning points at **is not built**. ⇒ relevant to `M-RP-VIEW-BINDING` / a future members milestone.
- ⚠️ **A REGISTRY LADDER DEVIATION.** Documented: 149 at rest → **156** space selected → 158 room latched. **Measured: 149 at rest → 158 after selecting a space alone.** Not investigated. **N-155 says a baseline has seven axes; this is either an eighth or an auto-latch.** ⇒ FILED, unexplained.

## §7 — DoD

**IMPLEMENTER / [CHAT]**
- [x] Faces proven real **with a positive control and a negative reference** before anything was judged
- [x] Variants driven at runtime, **zero code, zero `skin.css` edit**
- [x] 🛑 HANDS OFF posted before each interactive drive; ✅ ALL CLEAR posted after each
- [x] Verdict written to disk **immediately**, not at close
- [x] The limit on the verdict recorded **in the same section as the verdict**

**JOE**
- [x] Judges the variants with his own eyes. **Cannot be discharged by anyone else.** — done 2026-07-23

## §8 — Handoff

**→ `M-RP-SELF-SURFACE`** carries: the four values in §4, the `font-synthesis: none` requirement, the `"Self"` default (customisable later, per Joe's #5 lock at `M-RP-LOCK-RECHECK` §11), and the self-panel/message-row inconsistency in §6. ⚠️ **`M-RP-SELF-NAME` carries a persisted `widgetId:"self"` stored-data migration** and is the harder gate; this milestone has no dependency on it.
