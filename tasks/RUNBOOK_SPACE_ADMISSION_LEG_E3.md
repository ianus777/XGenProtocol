# RUNBOOK — M-SPACE-ADMISSION Leg E-3 (code half): rename the inverted A-bis test to assert what it tests

> **Status**: ACTIVE  
> Version: 1.1  
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
| **Status** | 🔒 **ACTIVE, LOCKED (Joe, 2026-08-23).** Clair implements from **v1.1** and no earlier revision |
| **Why it is a runbook at all** | 🔑 **It is one symbol, and it is `.rs`.** Chat writes no product code, and *"it is only a rename"* is exactly the reasoning a seat rule exists to refuse. **Not self-exempted** |
| **Blocking on** | ✅ **NOTHING** |
| **Tree** | citations measured at **`c72843a`**; **locked at `920ee62`** = `origin/main`, tree clean (`D-152`). 📌 *No `.rs` changed between them — the citations hold* |
| **Floors in** | cargo **1641 / 0 / 62 × 56 SUITES** · vitest **172 / 172 × 9 FILES** · svelte-check **0 / 34 / 15** · catalogue **UNMEASURED** |
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

- [ ] `E3-1` and `E3-2` applied; **one file, nothing else**
- [ ] `X-1` … `X-5` re-driven by Chat from `HEAD` (Rule 5)
- [ ] Any need to touch the body REPORTED, not absorbed (Rule 6)
- [ ] Rides Chat's `E-3` records commit — **`D-074` atomic, one commit**

📌 **"Commit pushed" is not a DoD item.**
