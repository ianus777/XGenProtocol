# M-RP-LOCK-RECHECK — re-verify the twelve D2 locks and mark each with its verifier
> **Status**: PENDING  
> Version: 1.0  
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
