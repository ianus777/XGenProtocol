# M-RP-SEAT-ORPHANS — re-home the retired seat's unowned appearance items
> **Status**: PENDING  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-21  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — What this is

**A filing, not yet a runbook.** Ms Design was retired at **J-555** and appearance returned to Chat.
**Nothing swept for work still assigned to her.** This document collects what the sweep found so the
items have an owner, and so the reason they accumulated is written down rather than rediscovered.

⚠️ **NOT a fix milestone.** Its output is a **routing decision per item** — re-home, discharge as
already-fine, or drop. Most will land on **M-RP-SKIN**; some are already judged and merely un-marked.

## §1 — How it was found

J-567 corrected J-560 to record **two** unmet D2 locks. Lock **#5** is the only one of the twelve with
a split owner — *"Wording/appearance = Ms Design"* — and it went nowhere. **That was written into the
record explicitly as INFERENCE, not proven cause.**

Then the ACTIVE-header sweep (J-568) found `M_RP6_6_RESIDENT.md` **§7 — a handoff stub addressed to
Ms Design** (the ConnStats row-swap), also unmoved. 🔑 **Two identically-shaped orphans from one
retirement stopped being a coincidence**, so a grep followed.

## §2 — The live-code orphans (measured, J-568)

Eight comments in **shipped code** name the retired seat. Four mark copy **the user is reading now** as
provisional:

| file:line | item |
|---|---|
| `derive.ts:39` | C-6 head-marker notice — **WORDING PROVISIONAL** |
| `stream-panel.svelte:45` | empty-stream copy — **WORDING PROVISIONAL** |
| `resident.rs:840` | user-facing connection copy — **WORDING PROVISIONAL** |
| `stream-panel.svelte:205` | fill-chain appearance (N-090) |
| `message-stream.svelte:277` | `data-phase` appearance (§3.4) |
| `grouping.ts:29` | grouping appearance (§3.3/§3.4) |
| `app_client.svelte:414` | §5 appearance deferral |
| `app_sampler.svelte:468` | "nothing visible here yet by design" |

**Plus, from the records:** D2 lock **#5** · `M_RP6_6_RESIDENT.md` §7 ConnStats row-swap.
**Ten known. The document-side grep is NOT exhaustive** — `.md` hits were not sorted into *historical
description* versus *live assignment*, and that sort needs reading, not grepping.

## §3 — 🔑 The finding worth more than the list

**Every orphan is an appearance or wording item.** Not one is code, a test, or a protocol decision.

That is not chance. **Appearance is the only category in this project with no automated verifier** —
cargo, svelte-check, npm, the catalogue and the registry all say nothing about whether a sentence is
the right sentence. So an appearance item parked against a seat produces **no failing signal when the
seat disappears.** It simply waits.

⇒ ***M-RP-SKIN is not a list of six cosmetic chores Joe has not got to. It is the sink for the one
category the project cannot verify, and it has been fed by a retirement nobody swept after.*** Chat has
now written *"a discharger that only accumulates has stopped being a plan"* three times; this is the
first time the cause is on the table rather than the symptom.

## §4 — What this milestone must decide (Joe's, all of it)

1. **Per item: re-home / already-fine-just-unmarked / drop.** ⚠️ `PROVISIONAL` means *nobody decided*,
   **not** *it is wrong* — some of this copy may be good and merely never blessed.
2. **Does M-RP-SKIN take them, or does it split first?** It already carries D2/D3's three tones · the
   editor save-note · ConnStats row-swap · M-RP-FOCUS · Send-as-icon (blocked on a verified glyph,
   D-108) · Leg D's shadow notice and its `--warn` colour. **Adding ten more to a queue already flagged
   three times as unhealthy is a decision, not a default.**
3. **The standing rule that stops the next occurrence.** Candidate: *no item may be assigned to a seat;
   items are assigned to a MILESTONE, and seats are staffing.* ⚠️ *A seat can be retired. A milestone
   has to be closed, deferred or cancelled — all three of which leave a trace.*

## §5 — Verification

- The `.md` corpus sorted **live assignment vs historical description**, by reading. ⚠️ **Records are
  never rewritten** — a historical mention of the retired seat is correct and stays.
- Every live-code comment either re-homed or removed, with its item routed somewhere named.
- **Zero user-visible change** unless Joe judges a specific piece of copy and changes it. ① Fixing
  wording *is* user-visible and belongs to the milestone that ships it, not to this audit.

## §6 — Owed

- The **document-side sort** is not done. §2's ten is a floor, not a total.
- ⚠️ **The generalisation, named and NOT taken:** other seats may have left orphans, and other
  categories may have unverifiable items parked in them. This milestone covers **one retirement**.
