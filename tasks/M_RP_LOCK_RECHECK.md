# M-RP-LOCK-RECHECK — re-verify the twelve D2 locks and mark each with its verifier
> **Status**: ACTIVE  
> Version: 1.1  
> Date: Jul 2026  
> **Last updated**: 2026-07-21  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — What this is, and what it is NOT

**A verification pass. ZERO code.** Every lock in `M_RP6_3_COMPOSER.md` §9.11.3 is re-driven at current
HEAD and marked with **the leg that verifies it** — or marked **eye-judged, Joe's**, which is a real
answer and not a gap.

⚠️ **NO FIX LANDS HERE.** Anything found unmet is *reported and filed*, never repaired in this
milestone. A verification pass that also fixes things cannot tell you whether it verified or created.
Lock #5's fix is already routed to the self-account arc (§6).

**Why it exists:** J-560 closed Leg D2 stating, in capitals, *"ONE LOCK UNMET, MEASURED NOT ASSUMED,
FLAGGED NOT ABSORBED."* **Two were unmet.** Lock #5 was never mentioned — not met, not unmet, not
deferred. It was never asked.

## §1 — Grounding

🔑 **The two failed differently, and the difference is the whole point of this milestone.**

- **#10 (auto-scroll on own send) was DRIVEN** — scroll up, send, read `scrollTop` — so it **failed
  loudly**, was flagged, and Joe accepted the deviation with his eyes open.
- **#5 (self special-cased) has no mechanism to drive.** It is satisfied or not by *what a person sees
  on their own row*. No probe was ever pointed at it, so it did not fail — **it was never asked.**

⇒ ***A lock with no verification leg is unfalsifiable. It cannot fail, so it cannot be trusted, and a
green milestone says nothing about it either way.*** Found by Joe looking at his own screen two
milestones later (J-567) — the same way `-->` and the collapsed newline were found.

**Second grounded fact, from the sweep (J-568):** #5 is the only one of the twelve with a **split
owner** — *"Wording/appearance = Ms Design"*. That seat was retired at **J-555**, five entries before
D2 shipped, and nothing re-homed it. **At least ten such orphans exist** → `M_RP_SEAT_ORPHANS.md`.
*One orphan is an oversight; ten identically-shaped orphans from one retirement is a mechanism.*

## §2 — Classification (from reading the table; to be CONFIRMED by the pass, not assumed)

| # | lock | expected verifier |
|---|---|---|
| 1 | a local echo exists | machine — DOM |
| 2 | echo lives in a `$common` store, not the widget | machine — source |
| 3 | keyed by client-minted local id, `event_id` stitched at outcome | machine |
| 4 | timestamp client-minted and stays that way | machine |
| **5** | **self is special-cased — no own hash tail** | **EYE — Joe. KNOWN UNMET (J-567).** |
| **6** | **THREE visual send-states, not two** | **SPLIT** — the four outcomes being *distinct* is machine; whether they *read as three states* is EYE |
| 7 | retry policy by status | machine |
| 8 | echo dies at exactly one moment; head marker covers own sends | machine |
| 9 | grouping and dividers come free | machine |
| 10 | auto-scroll on own send | machine — **known unmet, Joe accepted (J-560)** |
| 11 | N windows, one device | machine |
| 12 | no room latched ⇒ typing yes, sending no | machine |

⚠️ **This table is a PREDICTION and is exactly the kind of thing this project gets wrong.** §7's cargo
row was a confident prediction that was false (J-567). If a lock resists the verifier predicted here,
**that is a finding, not an inconvenience** — record which and why.

⚠️ **Do NOT trust J-560's numbers for the ten "machine" locks.** Two milestones and several commits
have landed since. *#5 is the proof that "was true at close" is not "is true now."*

## §3 — 🛑 THE `INTERACTIVE — HANDS OFF` CONVENTION (Joe-locked 2026-07-21, first use here)

Any leg whose reading depends on UI state is marked **`INTERACTIVE — HANDS OFF`**. Before driving it,
the driver posts in chat, clearly:

```
🛑 HANDS OFF — live measurement running
   App:     <client 9222 | node 9322 | sampler 9422>
   Reading: <what is being measured>
   Do not:  click, scroll, focus the window, open dialogs
   Expect:  <duration>
```

and **always**, including when the run dies or is abandoned:

```
✅ ALL CLEAR — measurement done, the app is yours again
```

🔑 **The all-clear is half the protocol, not politeness.** A hands-off with no stated end leaves Joe
frozen or guessing when it expired — *and a guess is exactly the click that lands mid-probe.*

**Fires for:** registry counts (all seven axes, N-155) · computed style or geometry · scroll and focus
legs · keystroke-by-keystroke legs · echo counts · anything with quiescence as a precondition.
**Does NOT fire for:** cargo · npm · svelte-check · git scope · files on disk. ⚠️ *Warning on those
would train Joe to ignore the warning, which is worse than missing one.*

⚠️ **Stated limit:** this protects against Joe's hands, not against a background process. Port checks
and the quiescent-baseline rule remain the guard against everything else.

🔑 **And the standing default, which outranks the convention:** **the app is Joe's.** If he is in it,
the driver waits. *His walking through the app is the highest-yield verification this project has —
three defects in one day that no automated leg caught.* A hands-off window is minutes, requested and
released, never a standing condition.

⚠️ **Harness limit, stated so nobody works around it silently:** the dev ports are fixed (client
5173/9222), so **two clients cannot run at once**. Node and sampler are separate and can be measured
while Joe is in the client. If this becomes real friction, a second dev port set is a small change —
file it, do not tolerate it.

## §4 — Legs

**Leg A — the ten machine locks.** Re-drive each at HEAD. Each needs its **positive control**: *"the bad
thing is absent" and "nothing happened" are the same string.*
**Leg B — the two eye-judged locks.** Prepare the app to the exact state, post HANDS OFF, hand to Joe,
record **his verdict verbatim**. ⚠️ **Chat and Clair must NOT judge these.** *A seat that judges
appearance is how the last one got retired.*
**Leg C — the output.** A permanent table in `M_RP6_3_COMPOSER.md` §9.11.3: every lock marked
`machine-verified at <commit>, leg <name>` or `eye-judged — Joe, <date>`, or **UNMET + filed where**.

## §5 — Verification of the verification

- Every Leg-A lock names the probe **and its control**.
- Leg B's verdicts are quoted, not paraphrased. ⚠️ *A paraphrased appearance verdict is Chat deciding
  what Joe meant.*
- #5 and #10 are recorded **UNMET with their filings**, not silently re-tested into a pass.
- The §9.11.3 table after this pass has **no lock without a named verifier.** That is the DoD.

## §6 — Files

- `tasks/M_RP6_3_COMPOSER.md` — §9.11.3 gains the verifier column; `Owes:` loses this item
- `JOURNAL.md` · `CLAUDE.md` · `docs/ROADMAP.md` · this doc

**NOT touched:** any `.rs`, `.ts`, `.svelte`, `skin.css`. **Zero code. A diff outside this list is a
Rule-6 flag.**

## §7 — Floors

**Every floor must be UNCHANGED** — cargo 1553/0/62 across 56 · svelte-check 0/34/15 · npm 154 ·
vite 202/170 · catalogue 419 · registry 149 at rest (state the axis, N-155).

🔑 **Here the floors are a control, not a gate:** this milestone writes no code, so **a moved floor
means something was touched that should not have been.**

## §8 — DoD

**IMPLEMENTER / [CHAT]**
- [ ] Leg A: ten locks re-driven at HEAD, each with its control
- [ ] Leg B: both eye-judged locks prepared, HANDS OFF posted, **ALL CLEAR posted**, verdicts quoted
- [ ] Leg C: §9.11.3 carries a verifier for **every** lock
- [ ] Any newly-found unmet lock **FILED, not fixed**
- [ ] Floors re-measured and unchanged

**JOE**
- [ ] Judges #5 and #6 with his own eyes. **Cannot be discharged by anyone else.**

## §9 — Owed, not smuggled in

- **`M_RP_SEAT_ORPHANS.md`** — the ten-plus orphaned appearance items. Separate by Joe's call
  (J-568): the re-check asks *"were the locks true?"*, the orphans ask *"who owns appearance now?"*
- **#5's fix** → the self-account arc (`M-RP-SELF-NAME` / `M-RP-SELF-SURFACE`), never here.
- **#10** stays an accepted deviation unless Joe reopens it.
- ⚠️ **The obvious generalisation, deliberately NOT taken here:** *every* locks table in the project
  probably has unverified entries. This milestone re-checks **twelve locks in one document**. Widening
  it to a project-wide audit is a different milestone and must be scoped as one.

## §10 — LEG A RESULTS (driven at HEAD b477bae, 2026-07-21)

**Ten machine locks re-driven. Floors re-measured and UNCHANGED, which is this milestone's control, not its gate.**

| # | verdict | probe | its positive control |
|---|---|---|---|
| 1 | ✅ MET | echo count 0→1 read in the SAME eval as the send click — the row exists before the network is consulted | pre-click 0 measured three times |
| 2 | ✅ MET | no widget holds an echo array; the store is `$common/stores/echo-state.svelte.ts` | the grep is proven live by finding 8 other `$state` decls in the same widgets |
| 3 | ✅ MET | same `localId` across `pending`→`accepted`; `eventId` ABSENT→stitched | the ABSENT pre-state was measured, so the stitch is a transition, not a pre-existing value |
| 4 | ✅ MET | `sentAt` byte-identical before and after the outcome | the same outcome demonstrably changed OTHER fields on that row — invariance, not a dead read |
| 5 | ❌ **UNMET** | own row renders the full XGID as author name; avatar `name:null` / `initials:"GC"` | rendered name === self `identity_id`, 65 chars, byte-equal, agreed by three independent sources |
| 6 | ✅ machine half MET | four outcomes → three tones + `pending`; `rejected`/`failed` share `not-sent` with DIFFERENT copy | registry getter `tone` === painted `data-tone` on all four; labels distinct strings |
| 7 | ✅ MET | retry offered on `failed` only — in the widget AND refused by the store | bypass call on `rejected`/`timed_out` resolved `null` + unchanged; the SAME call on `failed` MUTATED it |
| 8 | ✅ MET | no persistence path reaches the store; head marker names own sends | live: echo count 4→0 across a reload, the 4 measured immediately prior |
| 9 | ✅ MET | `groupedCount: 12`, `dividerCount: 0` — echoes are real descriptors | the "missing" 13th is ARITHMETIC, not hand-waving: two group heads 425,026 ms ≈ 7.1 min apart, past the 5-min window |
| 10 | ❌ **UNMET** | scrolled to top, sent, `scrollTop` stayed 0 / `atBottom` false | the row demonstrably landed (count 15→16, max 639→705) and the probe demonstrably sees scroll (639.2→0) |
| 11 | ⚠️ **SPLIT — see §10.2** | tile scope: whole grid unmounted, all 4 echoes survived | `composerMounted` 1→0 and the grid emptied, so the unmount really happened |
| 12 | ✅ MET | no room ⇒ typing yes (`disabled:false`, 18 chars accepted), sending no (button `disabled:true`) | latching a room flipped ONLY the button; the draft survived at 18 chars, so the refusal is caused by the latch |

### §10.1 — Floors, re-measured

`cargo` **1553 / 0 / 62 across 56 terminator lines** (summed programmatically, case-sensitive grep, all 56 present — the N-117 truncation trap dodged) · `svelte-check` **0 / 34 / 15** · `npm` **154** · `vite` **202 client / 170 sampler** · `catalogue` **419** · client registry **149 AT REST**.

**All identical. `git status` clean, HEAD still `b477bae`.** This milestone writes no code, so unchanged floors are the evidence that nothing was touched.

**Registry baseline stated on all seven N-155 axes:** quiescent · three-space store residue · no selection · zero saved states · echo count 0 · no settings pane drilled in · **no room latched** (`roomId: null`). Transitions **149 → 156 (space selected) → 158 (room latched)** reproduced exactly, and the post-run reload returned to **exactly 149**, `count === unique`, zero leaks.

⚠️ **Residue correction:** the kickoff said *three spaces + two rooms*. Measured: **three spaces, five rooms** (2 + 1 + 2). Content-in-widgets is Joe-ruled acceptable; recorded so the next baseline is not read against a wrong residue.

### §10.2 — FINDINGS

**① #5 was never eye-only, and the probe took five lines.** §2 predicted *"no mechanism to drive."* Grounded: `stream/derive.ts::projectEvent` and `stream-panel.svelte:115` both apply C-8's **inbound** rule to own rows (`isOwn` is computed and never used to suppress the identifier). The negative assertion — *does the own row show the tail?* — is a DOM read with a control. **It is falsifiable, and it fails.** What belongs there INSTEAD remains Joe's (Leg B). ⇒ #5 splits exactly as #6 already does.

**② ⚠️ #11 IS A SECOND UNFALSIFIABLE LOCK — the defect this milestone was filed to investigate, found a second time.** *"N windows, one device."* Measured: `xgen-client/tauri.conf.json` defines **ONE** window and there is **ZERO** runtime window-creation code (`WebviewWindowBuilder` / `WindowBuilder` / `create_window` — none). At OS-window scope the lock **cannot be driven, cannot fail, and cannot be trusted.** At tile scope — what the store comment actually argues — it is drivable and MET. **The lock's own wording is the ambiguity.** 🔑 *A lock with no verification leg cannot fail, so it cannot be trusted — and this document had TWO, not one.* **OPEN FOR JOE: rewording a lock is a records decision about his own design and was NOT taken here.**

**③ ⚠️ Lock #7's TEXT is stale against what shipped.** §9.11.3 reads *"timed_out → retry only behind an explicit warning."* Shipped is **no retry affordance at all**, deliberately narrowed at D2 §3.1 and enforced in BOTH the store's refusal and the widget's button (one predicate, N-126). Behaviour correct; **the table never received the amendment.** *The J-566 shape again — a decision applied in code and not in the record.*

**④ §2's #6 = SPLIT is CONFIRMED, not merely predicted** — `send-status.svelte` maps four outcomes to a three-way `data-tone` plus `pending`, so distinctness is a DOM read and only the reading is Joe's.

**⑤ Flag ② settled by measurement.** *"No probe was ever pointed at #5"* was inference from J-560's silence. Grepped: the only occurrence of `#5` in J-560's body is J-567's correction block, prepended later. **It was never asked — measured, not assumed.**

### §10.3 — Method notes from the drive

- ⚠️ **A synchronous loop of (set value → dispatch input → click send) sends ONCE.** Svelte re-enables the button on flush, so `click()` on a still-disabled button is a silent no-op. Caught only because the echo COUNT was read rather than the loop's return. *N-156's family: the loop returned a confident value that was not the feature.*
- Two legs used a **stub transport** (`echo.setTransport`) to drive `accepted` / `rejected` / `timed_out` on demand. **This verifies the store and the render rules; it verifies NOTHING about the wire** — and no lock among the twelve claims the wire. Recorded here so nobody later reads these rows as a wire proof.
- Two evals threw a bare `Uncaught` and were treated as **inconclusive, not failures** (N-110), then re-driven defensively. Neither entered this record.

### §10.4 — Still owed

- **Leg B** — #5 and #6, Joe's eyes, verdicts quoted verbatim.
- **#11's wording** — Joe's ruling; the verdict depends on it.
- **Leg C** — §9.11.3's verifier column, which is deliberately UNTOUCHED until B lands, since the DoD is *no lock without a named verifier*.
