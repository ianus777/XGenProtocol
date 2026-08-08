# M-RP-MEMBER-ACT Leg C — R7 acts: the row opens the DM and writes the bus — RUNBOOK
> **Status**: COMPLETED  
> Version: 1.2  
> Date: Aug 2026  
> **Last updated**: 2026-08-08  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — FOR JOE: THE LOCK TABLE

🔒 **ALL NINE POINTS LOCKED BY JOE ON ONE WORD — *"locked"* — 2026-08-08.** ✅ **CLAIR MAY WRITE CODE.**
Put as an explicit list rather than a document to approve wholesale — the format that closed fifteen
points on one word at J-696, used again here.

📌 **§6 leg 5 was NOT put to Joe and is NOT locked, deliberately.** It is a measurement Chat drives at
C-4 and reports; nothing in C-1–C-4 causes it, so ruling on it early would be a judgement about
something nobody has seen (`D-146` — no cost expires by deciding now).

**L-1** Clicking a member row **opens that member's DM** and leaves the **identity** on the bus, so R8
(inspector) shows the person while R5 shows the conversation.

**L-2** 🔒 **`OQ-C1` = N1 (Joe, 2026-08-08).** `roomLatch` gains a direct `latch(roomId: string)`
writer; R7 calls it. **N2 was refuted by measurement** (two synchronous bus writes coalesce into one
`$effect` run that never observes the room); **N3 refused** as a second bus (S-6).

**L-3** 🔒 **`OQ-C4` = `selectOnActivate`, default `true` (Joe, 2026-08-08).** Default `true` keeps all
three shipped consumers and all seven sampler cells **byte-identical in behaviour**. R7 passes `false`.

**L-4** 🔒 **`OQ-C2` = E-a (Joe, 2026-08-08).** The erased DM counterpart's row is **clickable like any
other**; clicking re-enters the DM you are already in. ⚠️ **Joe's condition:** what it *shows* depends
on retention — history-expiry and auth-tier rules. **Those do not exist in code.** Leg C therefore
ships the row clickable and **must not fake an archive, an expiry, or any retention state it cannot
honour.** The milestone that builds retention owes this row its behaviour, written into that
milestone's DoD (N-109).

**L-5** **R7's highlight stays DERIVED and is never written by a click.** `counterpart`
(`members-panel:119-123`) already derives the lit row from `addressBook.isDm` + the roster. `L-3`'s
flag is what protects it: turning `interactive` on would otherwise let `selectAt` overwrite a derived
highlight with a clicked one — the exact defect `M-RP-SELECT-ORIENT` removed from R1 and R2.

**L-6** **Four commits, in order:** C-1 the `ui/core` flag **alone** · C-2 `roomLatch.latch()` + the
header annotation · C-3 R7 acts · C-4 live verify + records.

**L-7** **EXISTING DMs ONLY.** A member whose DM does not exist yet is **out of scope** — that is Leg
C-bis's, along with the partial first send and DM-creation-to-an-erased-identity (`OQ5`, re-sited
J-690). ⚠️ **What a never-DM'd click DOES is still open (J-692).** Until it is answered, C-3 does
**nothing** on that click — see `§5-b`, and note this is a **countdown, not a resting state**.

**L-8** **The DM lookup is a NAMED LOCAL FUNCTION, not inlined and not extracted.** Copied-not-shared
follows J-508's precedent (roving was allowed **four** independent implementations before extraction
earned its own milestone). ⚠️ It carries a comment naming `M-RP-PEOPLE` as the anticipated second
caller and *"is a book entry enough to OFFER a DM?"* as the question to settle before any lift.

**L-9** **No new store, no new bus, no `ui/core` change beyond `L-3`'s flag.**

---

## §1 — GROUND TRUTH, MEASURED AT `4bba34f` (2026-08-08)

🛑 **CITE THE SITE, RE-MEASURE THE LINE.** `RUNBOOK_SELECT_ORIENT.md` staled its own line numbers two
commits into its own execution (C-1 shrank the file it cited). Every number below is a **starting
point for your own measurement**, not an authority.

| # | fact | site |
|---|---|---|
| **G1** | `selectAt` writes `selected` then calls `onActivate` | `entity-panel.svelte:110-116` |
| **G2** | `selected` is `$bindable`, typed `selected?: string` | `entity-panel.svelte:62-63` |
| **G3** | The `interactive` prop doc **already states this milestone's rationale** — *"entity-panel's own selectAt would drift it"* for R7 | `entity-panel.svelte:65-72` |
| **G4** | R7 renders inert, highlight one-way | `members-panel.svelte:174` |
| **G5** | R7's highlight is **already derived**, not clicked | `members-panel.svelte:119-123` |
| **G6** | R7's scope is the **SPACE**, not the room | `members-panel.svelte:46` |
| **G7** | 🔑 **`KnownSpace.counterpart` ALREADY EXISTS** — *"DM counterpart XGID, or the session identity for the self thread. `null` for a Space."* ⇒ the DM lookup is a **field match, not a search** | `spaces-state.svelte.ts:32-34` |
| **G8** | `KnownSpace.rooms: KnownRoom[]` rides embedded; there is no second fetch | `spaces-state.svelte.ts:31` |
| **G9** | 🛑 **`note()` is declared "THE SINGLE WRITER" TWICE and the claim is ALREADY FALSE** — `clear()` writes `_latched = null` today | `room-latch.svelte.ts:28-29`, `:76`, `:82-84` |
| **G10** | The bus has **SEVEN** importers, not six | `selection.svelte.ts` annotation (J-697) |

📌 **G9 matters for how C-2 is written:** the annotation **corrects a pre-existing inaccuracy** while
adding the third writer. It is not a claim this milestone breaks — it is one this milestone stops
letting stand.

---

## §2 — C-1: `entity-panel` gains `selectOnActivate`. ALONE.

**FILE:** `ui/core/lib/components/data-dependent/entity-panel.svelte` — **and nothing else.**

🛑 **`ui/core` IS SHARED.** This prop is seen by `spaces-panel`, `rooms-panel`, `members-panel` and
**seven sampler cells**. Measured alone, for the same reason C-4 was in `M-RP-SELECT-ORIENT`.

**THE CHANGE**
- New prop `selectOnActivate?: boolean`, **default `true`**, documented beside `interactive` (`G3`).
- `selectAt` (`G1`): guard **only** the `selected` write — one new line, `if (selectOnActivate)`.

🔒 **`activeIndex = i` STAYS UNGUARDED, AND THIS IS THE POINT.** `activeIndex` is *focus position*;
`selected` is *selection*. A row you activated is a row you are ON even when the panel does not own the
selection — guard it and the roving tabindex stops following activation, which is `L-3`'s defect
wearing an accessibility costume. **The flag governs SELECTION, never FOCUS.**

🔒 **`onActivate` STAYS UNGUARDED.** The consumer must still be told. A flag that suppressed the
callback would be a different feature and a worse one.

⚠️ The debug getter reports the flag so gates can read it. **Do not rename any existing getter field.**

**GATE C-1**
- svelte-check **0/34/15**.
- **Catalogue 435 = unique = domCount, zero orphans** — a **real** measurement, unfolded after a full
  `location.reload()`. A prop registers no id, so 435 is **expected, not proven.** If it moves, **STOP**.
- All **seven** sampler `entity-panel` cells: one `li[tabindex="0"]` each · the empty cell `0/0` ·
  **both inert cells `role="listitem"` only** (M-RP-PANEL-INERT untouched).
- 🔒 **DEFAULT-TRUE PROOF, and it is the gate that matters:** with no consumer changed, R1 and R2 still
  select on click **and on Enter**. `M-TOOL-CDP-KEY` makes the Enter half assertable — ⚠️ **NOT with
  `-At`, which focuses by clicking and would pass for the wrong reason** (J-698).
- cargo untouched **by scope** (zero `.rs`).

---

## §3 — C-2: `roomLatch.latch()` + the header annotation

**FILE:** `ui/common/lib/stores/room-latch.svelte.ts` — **and nothing else.**

**THE CHANGE**
- `latch(roomId: string): void` — assigns `_latched = roomId`. **No resolution, no validation, no
  reading of `_latched`.** It stays as free of read-modify-write as `note()` (`room-latch:32`).
- 🛑 **ANNOTATE `:28-29` AND `:76` IN THE SAME COMMIT.** Per `G9` the "SINGLE WRITER" claim is **already
  false**. The annotation states the corrected fact — **three writers: `note()` the bus-fed one,
  `clear()`, and `latch()` the direct one** — and says why `latch()` exists: *the DM's room must latch
  while the BUS carries the identity, which is what `L-1` asks for and what no bus-fed writer can do.*
- 🔒 **`latch()` is NOT exposed on the DEV hook as a new seam.** The J-548 lesson (`room-latch:29-30`):
  *a verify seam that skips the mechanism verifies the wrong thing.* Gates drive it through R7's click.

**GATE C-2**
- svelte-check **0/34/15**. Catalogue and cargo untouched **by scope**.
- ⚠️ **No behaviour changes in this commit** — nothing calls `latch()` yet. State that as scope; **do
  not claim a rendered delta.** (*"No rendered behaviour change"* was false once already, at J-695.)

---

## §4 — C-3: R7 acts

**FILE:** `ui/common/lib/components/widgets/members-panel.svelte` — **and nothing else.**

**THE CHANGE**
- `interactive={true}` · `selectOnActivate={false}` · `onActivate={onMemberActivate}` (`G4`).
- 🔒 **`selected={counterpart}` IS UNCHANGED (`L-5`, `G5`).** The highlight stays derived. **This is
  the whole reason `L-3` exists** — verify it by clicking a member and confirming the lit row is the
  one `counterpart` names, not the one you clicked, *when those differ*.
- `onMemberActivate(identityId)` calls the named lookup (`L-8`), then on a hit:
  1. `roomLatch.latch(roomId)` — R5 and R6 follow.
  2. `selection.set(regionId, descriptor)` with the **identity** descriptor — R8 shows the person.
  🔒 **In that order, and both unconditionally on a hit.** ⚠️ They are two independent writes to two
  independent stores; **they are NOT ordering-dependent** (that was N2's error, and N2 failed because
  it wrote the SAME store twice). Say so in the comment so nobody "fixes" it later.

**THE LOOKUP (`L-8`)** — a named local function, not inlined:
- Given `identityId`, find the `KnownSpace` whose `counterpart === identityId` (`G7`).
- 🛑 **GUARD BOTH MISSES SEPARATELY, they are different facts:** no such Space (**no DM exists** —
  `§5-b`) versus a Space found with **`rooms` empty** (`G8`). An empty `rooms` on a DM should not
  happen; **if it does, do nothing and it is a FINDING, not a case to paper over.**
- ⚠️ **Self is never a target.** `counterpart` is *"the session identity for the self thread"* (`G7`),
  so a self-thread Space would match the self id. R7 already never marks self (`L17`), but the lookup
  must not rely on that — **exclude the self id explicitly.**

**GATE C-3** — svelte-check **0/34/15**; the live legs are `§6`.

---

## §5 — TWO THINGS THIS LEG DELIBERATELY DOES NOT DO

**§5-a — IT DOES NOT CREATE A DM.** `L-7`. Creation, the partial first send, and
DM-creation-to-an-erased-identity are Leg C-bis's.

**§5-b — 🛑 A CLICK ON A MEMBER WITH NO EXISTING DM DOES NOTHING, AND THAT IS A COUNTDOWN.** The
behaviour is **OPEN (J-692)** and Joe has not ruled. Until then the click is a silent no-op.
⚠️ **A row that looks clickable and does nothing is a dead control** — 6.1j and `D-113`'s correction
both bite here. **This is acceptable ONLY because it is named, owned, and time-bound to Leg C-bis.**
📌 **It enters Leg C-bis's DoD, in the same edit that records it here** (N-109 — the removal is written
into the DoD of the leg that lifts the limit, not left as folklore).
🛑 **NO W-8 PHASE-LIMIT NOTE IS ADDED TO THE PANEL** (D7's pre-empt): R7 asserts nothing about DM
creation, so there is nothing to sweep at close.

---

## §6 — GATE C-4: THE LIVE LEGS (Chat re-drives, Rule 5)

Client 9222, unfolded, after a full `location.reload()`, **quiescent baseline stated before anything
is clicked** (N-105/N-108).

1. 🔒 **THE MILESTONE'S ONE SENTENCE.** In a group room with ≥2 members, click a member who has an
   existing DM ⇒ **R5 shows that DM · R8 shows the PERSON · the bus holds `kind: 'identity'`.**
2. 🔒 **THE HIGHLIGHT IS STILL DERIVED (`L-5`).** After the click, R7's lit row is the one
   `counterpart` names. **Prove `selectOnActivate: false` did its job** — assert `entity-panel`'s
   `state.selected` equals `counterpart`, *not* the clicked id, in the case where they differ.
3. 🔒 **KEYBOARD (`M-TOOL-CDP-KEY`).** Focus a member row **without a click** and press **Enter** ⇒ same
   result as ①. ⚠️ **`-At` is forbidden for this leg** — it focuses by clicking and would pass for the
   wrong reason (J-698).
4. **THE ERASED ROW (`L-4`).** Joe's state carries one erased-counterpart DM. Click it ⇒ **re-enters
   that DM, no crash, no created state**, and the erased marker + tail-8 still render.
5. ⚠️ **THE LIST RELOADS UNDER YOU, AND JOE OWNS WHETHER THAT IS RIGHT.** Clicking a member in a group
   room moves the latch to the DM ⇒ `scope` changes (`G6`) ⇒ **R7 re-renders to the DM's roster: two
   people.** Correct by every rule we have; it is also the panel you just clicked in replacing itself.
   **Measure it, show it, and do not treat it as settled.** Filed adjacent: `M-RP-PEOPLE` — a
   people-panel over the address book — which is the surface that would make this feel like context
   rather than loss. **Filed, not scheduled.**
6. **NO-OP LEG (`§5-b`).** Click a member with no existing DM ⇒ **nothing changes** — bus, latch, R5
   and R7 all unmoved. Assert the absence; do not infer it.
7. **Registry** returns to its exact quiescent baseline after the arc. **Joe's client state file
   `LastWriteTime` UNCHANGED** — read it before and after, never write to it.

**FLOORS AT CLOSE:** cargo **1597/0/62 × 56** (untouched by scope) · svelte-check **0/34/15** ·
catalogue **435** (real measurement at C-1, by-scope thereafter) · sampler `npm test` **154/9**,
`FAILED` grepped **case-sensitively**.

---

## §7 — RECORDS (C-4, `D-074`, one atomic commit)

JOURNAL · `CLAUDE.md` PLAY · ROADMAP · this runbook → **COMPLETED** · Leg C Phase-0 → **COMPLETED** ·
**`M-RP-PEOPLE` filed as a ROADMAP node** (`people-panel`; 🛑 **never named `contacts`** —
`address_book.rs:38` reserves that word for Ch2's private contact record).

⚠️ **`roadmap-format-gate.ps1` MUST RUN AND RETURN 0** before any commit touching `docs/ROADMAP.md`.
⚠️ **`CLAUDE.md` and `docs/ROADMAP.md` are CRLF**; `Filesystem:edit_file` rewrites to LF and
`core.autocrlf=true` makes `git diff` blind to it — **verify by COUNTING BYTES.**
🛑 **Restore CRLF with a LF→CRLF conversion ONLY.** A global normalising regex silently edited five
lines in two unrelated records at J-697; the repair was to restore from git and re-splice with an
asserted anchor.
🛑 **Commit messages go through `-F <file>`**, written with `[System.IO.File]::WriteAllText` +
`UTF8Encoding($false)`. A pasted here-string can silently not run; `Set-Content -Encoding UTF8` writes
a **BOM** under PS 5.1 and put one in a commit subject at J-697.

---

## §8 — WHAT THE RUN FOUND (written at close, J-700)

✅ **SHIPPED IN THREE COMMITS, IN THE LOCKED ORDER (`L-6`).** C-1 `524d4f7` (`ui/core`, 1 file, +15/−3) ·
C-2 `b5d0908` (`room-latch`, 1 file, +17/−5) · C-3 `6a6c066` (`members-panel`, 1 file, +62/−3). **Every
gate re-driven by Chat independently (Rule 5); Clair's self-run numbers and Chat's agreed on every leg.**

**FLOORS AT CLOSE:** svelte-check **0/34/15** (three times, unmoved) · catalogue **ids 435 = unique 435 =
domCount 435, ZERO orphans both directions** — a **real measurement** after a full `location.reload()`, not
an inheritance · sampler **154/9**, `FAILED` grepped case-sensitively **0** · cargo **untouched by scope**
(zero `.rs` across all three commits) · client registry **164 quiescent**, returned exactly · Joe's client
state file **byte-identical**, 2,856 B @ 2026-08-05 07:12:25, read before and after.

### 🔑 THE GATE THAT MATTERED PASSED, AND IT PASSED ON THE PIXEL-ADJACENT LAYER

`§6` leg 2 — **the highlight stayed DERIVED.** Clicking a member in a group room left `entity-panel`'s
`state.selected` **`null`** with **every** `aria-selected` still `false`, while `tabindex` moved to the
clicked row. ***Focus followed activation; selection did not.*** That is `L-3`/`L-5` working, asserted in
the case where the clicked id and `counterpart` **differ** — not inferred from a green compile.

### 🛑 TWO LEGS WERE NOT DRIVEN, AND THE REASON IS THE FIXTURE

**`§6` leg 5 could not be produced AT ALL, and `§6` leg 1 only in its in-DM form.** Joe's state carries
**exactly one** DM; its counterpart is **erased**; and an erased NON-counterpart member is **hidden** in a
group room (`§5a` E2 protects it only inside the DM, where `counterpart` is non-null). ⇒ **zero rendered
group-room rows have an existing DM**, so nothing can move the latch out of a group room by clicking a
member.

🔒 **LEG 5 THEREFORE STAYS UNRULED — `D-146`, no stated cost has expired.** ⚠️ **It is recorded as
BLOCKED, not as PASSED.** The obligation is filed in **both** `M_RP_MEMBER_ACT_LEG_C_BIS.md` (`OWED-4`)
and `M-RP-PEOPLE`: whichever lands first makes a second non-erased DM reachable, **owes the measurement,
and owes SHOWING IT TO JOE before anyone rules.**

### ⚠️ THIS RUNBOOK STALED ITS OWN LINE NUMBERS — AGAIN, AND IN THE SAME WAY

`§1` warned that `RUNBOOK_SELECT_ORIENT.md` staled its cited lines two commits into its own execution.
**This one did it too.** C-2 grew `room-latch`'s header from 3 lines to 8 (**+5** below it); C-3 added ~11
header lines and ~40 interaction lines to `members-panel`. Superseded, **kept not erased** (`D-131`):

| cited in `§1` / `§3` / `§4` | was | **is, at `6a6c066`** |
|---|---|---|
| `room-latch` header "SINGLE WRITER" (`G9`) | `:28-29` | **`:28-35`** |
| `room-latch` `note()` doc (`G9`) | `:76` | **`:81-86`** |
| `room-latch` `clear()` (`G9`) | `:82-84` | **`:93-96`** |
| `room-latch` `latch()` | — | **`:87-92`** (new) |
| `members-panel` `scope` (`G6`) | `:46` | **~`:57`** |
| `members-panel` `counterpart` (`G5`) | `:119-123` | **~`:130-134`** |
| `members-panel` `<EntityPanel>` (`G4`) | `:174` | **~`:228`** |

📌 **`G9` WAS RIGHT ABOUT THE THING THAT MATTERED.** *"`note()` is THE SINGLE WRITER"* was **already
false** before this milestone — `clear()` had always written `_latched`. C-2 corrected a standing
inaccuracy rather than breaking a claim, and said so at the site.

### 📌 THREE NOTES AND ONE DEBT REGISTER CAME OUT OF THE RUN

**`N-169`** — `__XGEN_DEBUG__.get(id)` returns `{type, state}`; a field read off the result is always
`undefined` and yields a clean-looking `[]`. **`N-099` in the wild, on Chat's own first probe of the arc.**
**`N-170`** — `data-debug-id` carries the full **`type#id`**; the standing kickoff line (*"keys on
`data-debug-id`, NOT `id`"*) is **narrower than the thing it describes** and a reader following it exactly
still writes the failing selector — **the `L-14` defect shape, now in the tooling instructions.**
**`N-171`** — `latch()` ships unconditional while `selection.set()` sits behind a roster guard; the two
resolve against **different stores**, so the pair can half-apply. **FILED, NOT FIXED**, shipped as written.
**`M_RP_MEMBER_ACT_LEG_C_BIS.md`** — created **PENDING** at this close so `§5-b`'s dead control had a DoD
to land in rather than becoming folklore (`N-109`).

### 🔑 AND ONE PROCESS FAILURE WORTH MORE THAN THE CODE

**A reported push had not happened.** `git status` read `M ` — **staged, not committed** — with HEAD still
on C-2, because the `git commit -F _commit_c3.txt` handed to Joe referenced a file **Chat had never
written**. `git add` succeeded, `git commit` errored, `git push` sent nothing, and the tree sat
half-applied looking finished. **Exactly the J-697 shape, from the opposite direction.** Caught only
because the close verified the **artifact** (`git log`, `rev-parse`, `ls-remote`) instead of the report.
⇒ ***Verify the artifact, never the exit code, and never the word "done" — including Chat's own.***
