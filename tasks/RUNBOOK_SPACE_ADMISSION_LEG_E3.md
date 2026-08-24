# RUNBOOK — M-SPACE-ADMISSION Leg E-3 (code half): rename the inverted A-bis test to assert what it tests

> **Status**: COMPLETED  
> Version: 1.2  
> Date: Aug 2026  
> **Last updated**: 2026-08-23  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — LOCK STATE

| | |
|---|---|
| **Status** | ✅ **COMPLETED (J-772, 2026-08-23).** One file, **+10 / −7**. Every gate re-driven by Chat from `HEAD` under Rule 5. **Floor UNCHANGED at 1641 / 0 / 62 × 56 SUITES.** 🛑 **One of its own instructions was defective — §6a** |
| **Why it is a runbook at all** | 🔑 **It is one symbol, and it is `.rs`.** Chat writes no product code, and *"it is only a rename"* is exactly the reasoning a seat rule exists to refuse. **Not self-exempted** |
| **Blocking on** | ✅ **NOTHING** |
| **Tree** | citations measured at **`c72843a`**; **locked at `920ee62`** = `origin/main`, tree clean (`D-152`). 📌 *No `.rs` changed between them — the citations hold* |
| **Floors in / out** | cargo **1641 / 0 / 62 × 56 SUITES — UNCHANGED, which is what `X-1` requires** · vitest **172 / 172 × 9 FILES** · svelte-check **0 / 34 / 15** · catalogue **UNMEASURED** |
| **Scope** | 🔒 **ONE FILE: `xgen-node/src/tests/space_admission_third_party_join.rs`.** No production code. No other test |

---

## §1 — THE PROBLEM

`xgen-node/src/tests/space_admission_third_party_join.rs:115`:

```
async fn third_party_registered_identity_joins_a_dm_it_is_not_party_to()
```

**Leg D inverted what this test asserts.** It now proves the third party is **REFUSED** — `Rejected(3047 admission_required)`, no role, no member record. 🛑 ***The name is a false statement about its own body.***

🔑 **`N-109`'s species in a symbol**, and this codebase names tests by **outcome** — `..._is_rejected_to_the_sender_end_to_end`, `..._is_rejected_3047_to_the_sender`.

✅ **The test's own doc comment already says all of this** (`:100-113`) and explains why Clair did not rename it in Leg D: *"renaming is a naming decision and the current name is cited by `docs/ROADMAP.md`, the `JOURNAL` and the A-bis runbook. Routed at Leg D's hand-back §2, not absorbed."* 📌 **That routing was correct and this runbook is its destination.**

---

## §2 — THE EDIT

### 🔒 `E3-1` — the rename

`:115` becomes:

```
async fn third_party_registered_identity_is_refused_a_dm_it_is_not_party_to()
```

🔑 **Outcome, not attempt** — matching the convention the doc comment cites. 🛑 **The COMPANION at `:282` `third_party_registered_identity_joins_an_open_space` IS NOT RENAMED.** Its name is still true: under `D-148` clause 3 an ordinary Space defaults to `open` forever, so she **does** join. ***A companion edited alongside the thing it was built to outlive was never a control.***

### 🔒 `E3-2` — the doc comment

`:108-113`'s ⚠️ paragraph — *"The function NAME still describes the attempt rather than the outcome … It is kept because renaming is a naming decision"* — **is now false and is REPLACED**, not left standing: the rename happened at J-771. **The rest of the comment (`:100-107`) is accurate and stays.**

📌 **Retain, in one line, that the test was INVERTED by Leg D** — a reader who finds only the new name loses the fact that it once asserted the opposite, which is the history `D-131` exists to keep.

---

## §3 — WHAT THIS RUNBOOK MUST NOT DO

1. ❌ **No production code.** Zero `.rs` outside the named test file.
2. ❌ **Not the companion** (`:282`).
3. ❌ **No change to what the test ASSERTS.** Symbol and comment only — 🔑 ***if the body needs touching to make the name true, the name is not the defect and that is a FINDING*** (Rule 6).
4. ❌ **Not the `.md` citations.** `CLAUDE.md:315`, `JOURNAL.md:703`, `tasks/CLAIR_LEG_D_HANDBACK.md:67/98`, `tasks/M_SPACE_ADMISSION_PHASE0.md:345`, `tasks/RUNBOOK_SPACE_ADMISSION_LEG_A_BIS.md:126/349` all cite the old name. 🛑 **They are HISTORICAL RECORDS of what the symbol was called at the time and are NOT rewritten** (`D-131`). **Chat annotates the two live ones; the JOURNAL and hand-back entries stand untouched.**

---

## §4 — GATES

| gate | what |
|---|---|
| **X-1** | `cargo test --workspace --no-fail-fast` — **detached**, logged, `XGEN_EXIT_SENTINEL=` appended, `^test result:` summed **case-sensitively**. 🔒 **Floor UNCHANGED: `1641 / 0 / 62 × 56 SUITES`.** ***A rename must not move the count in either direction*** — a drop means a test stopped being collected, a rise means one was added |
| **X-2** | **The NEW name present in the run log by exact name, and the OLD name ABSENT.** 🔑 *Both halves — a rename verified only by the new name's presence cannot see a duplicate left behind* |
| **X-3** | `third_party_registered_identity_joins_an_open_space` **still present, still green, still by that name** |
| **X-4** | `git diff --numstat` shows **exactly one file** |
| **X-5** | 🛑 **The body is byte-identical apart from the signature line.** `git diff` shows the `fn` line plus the doc-comment hunk **and nothing else** |

📌 **No negative control.** ***A rename has no behaviour to disarm, and inventing one would be a probe that cannot fail*** — `E2-6`.1's species, and this document declines it deliberately rather than by omission.

---

## §5 — DoD

- [x] **`E3-1` and `E3-2` applied; one file, +10 / −7, zero `.md` touched**
- [x] **`X-1` … `X-5` re-driven by Chat from `HEAD`; `X-2b`'s phrasing corrected at §6b**
- [x] **No body change was needed — the cold read confirmed the body already asserts refusal, so the new name is true of it (§3.3 clear). One SEPARATE deviation reported: §6a**
- [x] **Rides Chat's `E-3` close commit — `D-074` atomic**

📌 **"Commit pushed" is not a DoD item.**

---

## §6 — 🛑 CLOSE ANNOTATIONS (J-772, 2026-08-23)

Corrected at close, never erased (`D-131`). §2 and §4 above stand as written.

### §6a — 🛑 **`E3-2` INSTRUCTED A FALSE CITATION, AND CLAIR REFUSED IT.** *(Clair, Rule 6; verified by Chat)*

§2's `E3-2` said the replacement comment should record that ***"the rename happened at J-771."***

✅ **MEASURED: J-771 states *"RECORDS + ONE RUNBOOK. ZERO PRODUCT CODE"* and its own tail reads *"NEXT: Clair renames"* — future tense.** The rename lands in the **E-3 close commit**, whose J-number **did not exist when the comment was written** (max was J-771).

🔑 ***Writing `J-771` into the comment would have asserted product code inside the one entry that records none — `D-153`'s species exactly: a conclusion that is true resting on a citation that is false.*** **And it would have breached `N-198`, which is precisely about an artefact citing a number whose record rides later.**

✅ **Clair wrote *"RENAMED at Leg E-3"* — cites the LEG, true on the day it was written, checkable, no phantom number. NOT OVERRULED.** 📌 **The runbook line was Chat's and it was wrong. Seventh specification defect of the arc, seventh found by the implementing seat.**

### §6b — 📌 **`X-2b` DOES NOT SAY WHERE THE OLD NAME MUST BE ABSENT, AND THE ONE PLACE IT SHOULD SURVIVE IS THE HISTORY NOTE.** *(Chat, at re-drive)*

`X-2` reads *"the OLD name ABSENT."* ✅ **Absent from the run log: 0 occurrences.** 🛑 **But it SURVIVES in source at `:109`, deliberately** — inside the new comment recording what the symbol used to be called, which is `D-131` working as intended.

⇒ ***a source-level grep for the old name reads as a MISS, and a gate written that way would have failed a correct implementation.*** 🔑 **The gate happened to be phrased against the log — the half that discriminates — but that was luck, not design.** 🔒 **RULE: a rename gate must name WHERE the old symbol must be absent.**

---

## §7 — 🔒 CLOSE MEASUREMENTS (Chat, Rule 5, from `HEAD` `08c9e50`)

| gate | measured | |
|---|---|---|
| **X-1** | `1641 / 0 / 62 × 56 SUITES` — **UNCHANGED in both directions** · `XGEN_EXIT_SENTINEL=0` · `Compiling xgen-node` present · `FAILED` **0** case-sensitive | ✅ |
| **X-2** | NEW name in log **1**, OLD name in log **0** — both halves; see §6b | ✅ |
| **X-3** | `third_party_registered_identity_joins_an_open_space ... ok`, still by that name | ✅ |
| **X-4** | exactly **one** file · **zero `.md` touched** · no untracked | ✅ |
| **X-5** | the doc-comment hunk + the `fn` line **and nothing else**; **+10 / −7** reconciles (9-for-6 paragraph, 1-for-1 signature); LF preserved | ✅ |
| **controls** | 📌 **none invented — declined exactly as §4 declines them.** *A rename has no behaviour to disarm* | ✅ |

📌 **The six `.md` citations of the old symbol are left intact** — `CLAUDE.md`, `JOURNAL.md`, `CLAIR_LEG_D_HANDBACK.md` (×2), `M_SPACE_ADMISSION_PHASE0.md`, `RUNBOOK_SPACE_ADMISSION_LEG_A_BIS.md` (×2), and this runbook. **Historical records of what the symbol was called at the time (`D-131`).**

📌 **Clair hit the launcher-vs-cargo exit-code trap again and caught it the same way:** the notification reported *"completed (exit code 0)"* with the log at 19 KB and `cargo.exe` PID 50116 still alive; the log finished at 148 KB. **Only the ABSENT sentinel separated a plausible, complete-looking, wrong count from a real one.**
