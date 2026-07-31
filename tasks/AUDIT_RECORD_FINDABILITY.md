# Audit — where records actually live, and how a design can be present but unfindable

> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-31  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Why this document exists

On 2026-07-31 the project's documents had just been through a ten-leg consolidation, and the reasonable worry was that something had been lost in it. The measurement said no: exactly one file was deleted across the whole window, its contents are still reachable in version control, and every journal entry the old roadmap referred to is still present.

But the same session turned up two designs that were present in the project and could not be found by anybody looking for them, and one argument that is genuinely gone — lost two weeks *before* the consolidation started, by a mechanism that has nothing to do with it.

**Losing a document and being unable to find one look identical from the outside.** This document is about the second, because it is the one that is still happening.

---

## The worked example

**The question asked:** where is the design for switching a panel between three display modes — lines, avatars, gallery?

**The answer:** in `tasks/M_RP_REGION_GEAR.md`, in full. Walked and locked on 2026-07-17, with three layers separated, the Rooms panel named as the first tenant, and the Spaces and Members panels named as later candidates.

**Why nobody could find it:** it lives in a milestone document about a *settings gear*. No panel document references it. Two keyword searches — one narrow, one deliberately widened — both failed to reach it. So did the first draft of the members panel audit, which was written specifically to prevent this class of failure and which missed it anyway.

### And the milestone is invisible to every canonical record

Measured 2026-07-31, occurrences of the milestone's own identifier:

| Record | Occurrences |
|---|---|
| `docs/ROADMAP.md` | 0 |
| `CLAUDE.md` | 0 |
| `CLAUDE_HISTORY.md` | 0 |
| `DECISIONS.md` | 0 |
| `JOURNAL_ARCHIVE.md` | 0 |
| `JOURNAL.md` | 0 before the 2026-07-31 entry |

It was never on the roadmap: zero occurrences in the 761 KB pre-consolidation roadmap, and zero in the archive file that was later deleted. **It was never on the board to lose**, which exonerates the consolidation completely for this item and points at the real cause.

### The real cause, and it is structural

One commit: a single file, 85 lines, no journal entry — on a day when seven other document commits each carried one.

The session transcripts show why. The session did its work, wrote its records, prepared the push, and wrote a hand-off for the next session. **Then** the gear was designed, written and committed. By that point the session's journal entry was already finished and committed.

⇒ **Work done after a session's records are written has no home.** That is not carelessness about one document. It is a gap at the tail of every session, and this milestone is simply the one that fell into it.

---

## One argument is genuinely lost

The reasoning behind welcoming the display-switch idea, recalled on 2026-07-31 as:

> *"you welcomed the idea because it resolve some code part that otherwise would be local or something like that"*

**Not recovered. Not reconstructed.**

Searched and exhausted, all on 2026-07-31:

1. All six canonical records.
2. The 761 KB pre-consolidation roadmap and the deleted archive blob.
3. Every branch and every object in version control for the task document — one commit, one blob, byte-identical on every ref. No draft, no earlier version.
4. Five sibling documents from the same arc, plus the interface notes file.
5. Every `.md` file in the repository including the ~240 in `tasks/archive/` and the backup folders.
6. **Every commit message body in the repository** — a record surface no earlier search had touched.
7. Every non-markdown text file over 2 KB in the tree, and the stale worktree copies.

Four session transcripts from the day were supplied afterwards and read. The gear walk falls inside a step the transcript's own summariser collapsed into a single line.

### Why no reconstruction was offered

Adjacent arguments do survive in the gear document: that the gear is a cheap reusable door, that there is one settings action with two entry points, that a second settings surface was explicitly rejected, and that the setting rides the same shared channel the backdrop uses rather than a bespoke one. A fifth, from the same day but a different milestone, files a toolbar as shell-local until a fourth sighting across the members, rooms and spaces headers earns extraction — which is literally *a code part that would otherwise be local*, about the same three panels.

Any of those could be offered and would sound right.

🛑 **That is exactly how a reconstruction becomes a false record.** Once a plausible candidate is put in front of the person trying to remember, neither party can afterwards separate recognition from suggestion. The project already carries this discipline elsewhere — a reading is carried as a reading until it is confirmed — and it has paid for itself more than once.

**If the reasoning returns in its author's own words, that is a recovery and it replaces this section. Nothing else does.**

---

## Record surfaces this project actually uses

Listed because searching only the obvious ones is what produced the misses above.

1. **The six canonical records** — roadmap, the briefing file and its history, decisions, journal and journal archive.
2. **Task and runbook documents** in `tasks/`, including roughly 240 files in `tasks/archive/`.
3. **Phase-0 and specification documents** in `docs/`, plus `docs/backup/`.
4. **The interface notes and design-brainstorm files** under `ui/docs/`.
5. **Commit message bodies.** Several carry four or five paragraphs of design reasoning that appears nowhere else. Nothing indexes them, and no ordinary document search reaches them.
6. **Code comments.** Repeatedly the most accurate statement of intent in the repository — and they have no owner, no expiry and no trigger. A deferral written as a code comment has already outlived the milestone it was waiting for at least once.

---

## Three naming generations

The members work has been called `M6` client members, then `M-RP6.4`, then `M-RP-MEMBERS`. **A search phrased in the current generation does not find the oldest.**

`tasks/M6_CLIENT_MEMBERS_DESIGN.md` is still marked pending and still poses as an open question — should membership come from asking the node, or from a local cache — something the shipped code answered long ago by asking the node. It uses none of the later vocabulary, so no search for the current names reaches it.

---

## What would fix it

Proposals, not decisions.

1. **Nothing gets filed without a record entry.** If a design walk produces a document, it produces a journal entry in the same commit. The existing four-file discipline already says this; the gap is that it is applied when a milestone *changes state* and not when one is merely *filed*.
2. **A session's records close last.** If work continues after the records are written, the records are reopened or the work waits for the next session. Today the hand-off is written before the session actually ends.
3. **Every milestone gets a board node when it is filed, not when it starts.** A filed milestone with no node is invisible to everyone who is not holding its filename.
4. **When a milestone is renamed across generations, the old document gets a pointer**, or it gets closed. A pending document that answers a settled question is worse than no document.

---

## The honest summary

The consolidation removed one file and it was recoverable. The members design was scattered across roughly fifteen documents and three naming generations, and every piece of it was intact. The display-switch design survived in full.

**One argument was lost, and it was lost by a session that ended after its own records did.**

