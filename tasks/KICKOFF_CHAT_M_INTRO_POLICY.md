# M-INTRO-POLICY — session kickoff (CHAT CLAUDE)
> **Status**: PENDING  
> Version: 1.0  
> Date: Aug 2026  
> **Last updated**: 2026-08-15  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

```
XGen Protocol — session kickoff (CHAT CLAUDE)

MILESTONE: M-INTRO-POLICY — receiver-side intro policy. 🟡 PENDING, PHASE-0 OWED.
🎯 THIS SESSION: author the M-INTRO-POLICY Phase-0 audit. Chat writes it FIRST, with
   Joe decisions as open §§ carrying recommendations + D-121 lenses. NEVER end a turn
   with "tell me X and I will start" — D-123 names UNDER-stepping as the failure mode.
🛑 ITS TRIGGER HAS ALREADY FIRED (at 3e1014d, when M-RP-INTRO landed). A TRIGGER THAT
   HAS FIRED WITH NO PHASE-0 IS A DEFECT. That is exactly how M-RP-INTRO itself started.
🎯 NO CODE. This is an audit + authoring session.

═══ STATE AT OPEN — RE-MEASURE, NEVER INHERIT ═══
✅ Expect HEAD = origin/main via git ls-remote (NOT the tracking ref). CLEAN.
✅ Last entries J-737 · J-736 · J-735. ROADMAP v7.24. Next free N = N-197.
🔒 FLOORS — carried from J-734, NOT re-driven since (three no-code sessions):
     cargo ......... 1602 / 0 / 62 × 56 SUITES
     vitest ........ 172 / 172 × 9 FILES
     svelte-check .. 0 errors / 34 warnings / 15 files
🛑 NEVER cite a floor without its unit. vitest = × FILES. cargo = × SUITES.
🛑 CATALOGUE IS UNMEASURED. Its harness has never been located. DO NOT WRITE 435.
📌 Apps state UNKNOWN at open. MEASURE with netstat — N-196: Vite binds IPv6-ONLY, so
   Test-NetConnection 127.0.0.1 -Port 5173 returns False ON A HEALTHY SERVER.

═══ 🛑 A CENSUS HAZARD THAT WILL BITE THIS SESSION IF YOU FORGET IT (J-737) ═══
🔑 `.claude/worktrees/` HOLDS 8 FULL SOURCE TREES, ALL FROM MAY 2026, AND `git status`
   CANNOT SEE THEM — `.gitignore:10` hides them. Only 4 are registered with
   `git worktree list`; the other 4 are ORPHANED on disk.
🛑 EVERY repo-wide grep MUST exclude `\.claude\` alongside `\target\` and node_modules.
   MEASURED INFLATION: auth_tier 219 raw vs 107 true (51%). EventType 2519 vs 957 (62%).
   ⇒ THE INFLATED NUMBER IS COMPLETE-LOOKING, PLAUSIBLE AND WRONG. This session is
     census-heavy by nature. Excluding worktrees is not optional.

═══ 📖 READ, IN THIS ORDER ═══
CLAUDE.md PLAY block (top = J-737, then the M-RP-INTRO close, then J-734)
JOURNAL.md J-737 · J-736 · J-735
🎯 docs/ROADMAP.md → the M-INTRO-POLICY node AND the M-RP-INTRO-CANVAS node (its Owes
   rows carry Joe placement + the two guard questions + the N-172 binding)
tasks/M_RP_INTRO_PHASE0.md v1.5 §11 → what the closed milestone carried OUT
DECISIONS.md → D-065 · D-071 · D-074 · D-112 · D-120 · D-121 · D-123 · D-131 · D-138 · D-143
ui/docs/xgen-ui-notes.md → N-172 (the canvas rule) · N-182 · N-196

═══ 🔑 WHAT IS ALREADY RE-GROUNDED — VERIFIED J-737, STILL VERIFY IF LOAD-BEARING ═══
✅ NodePolicy lives in xgen-core/src/space/node_policy.rs (23 hits) and is read ONLY by
   xgen-node: admin_ops 11 · pipe 6 · app 4 · aicontrol 3.
✅ ZERO NodePolicy hits anywhere in xgen-client/ or ui/. The admin-side-only claim HOLDS.
🛑 NOT yet re-verified: that build_dm_space_create_event hardcodes auth_tier: 1, and that
   auth_tier has zero hits in the client. THE GREP WAS DROWNED BY WORKTREE NOISE AND THE
   REAL CALL SITE WAS NEVER READ. Drive it properly, with worktrees excluded.
🛑 EVERY ONE OF THE ABOVE IS A CLAIM IN A DOCUMENT. If one becomes load-bearing, RE-OPEN
   THE SOURCE. Stale claims have produced a finding in each of the last three sessions.

═══ 🛑 WHAT THE LAST SESSIONS GOT WRONG — READ EVERY LINE ═══
① 🔑 A CANONICAL RECORD WAS A MONTH STALE AND PRODUCED A LIVE ERROR. CLAUDE.md still said
   the settings mechanism was "filed, deliberately not decided" — D-120 had RESOLVED it on
   2026-07-17, and the string D-120 appeared NOWHERE in CLAUDE.md. Chat read it and told
   Joe the question was open. ⇒ ANNOTATED under D-131. THE ENTRY-POINT DOCUMENT IS NOT
   SELF-CORRECTING. If a PLAY block says something is blocked, CHECK DECISIONS.md.
② 🛑 CHAT PUBLISHED A NUMBER ITS OWN SCREEN CONTRADICTED, AND PUSHED IT TWICE. "39 hits /
   9 files" — the grouped list summed to 50. It also conflated code with documentation,
   and THE DOC HALF SELF-INFLATES (15 → 25 purely from writing about the rename).
   TRUE: 35 code sites / 5 files. ⇒ SUM THE GROUPED OUTPUT BEFORE QUOTING A TOTAL.
③ 🛑 A BLANKET STRING REPLACE CORRUPTED THREE INNOCENT LINES. Chat collapsed doubled
   apostrophes across ALL of CLAUDE.md to fix its own authoring artefacts, and destroyed
   legitimate empty-string literals in three OLDER entries. Caught by DIFFING LINE NUMBERS,
   not by reading. ⇒ A CORRECTION APPLIED TO THE CLASS WITHOUT CHECKING THE CLASS HAS
   INNOCENT MEMBERS IS THE MIRROR OF FIXING ONLY THE CITED INSTANCE. BOTH FAIL.
④ 🛑 A PROBE READ A FIELD THAT DOES NOT EXIST AND GOT A CLEAN-LOOKING NULL. Chat read
   spaceLatch.effectiveSpaceId (it is latchedSpaceId) — the null was INDISTINGUISHABLE
   from a failed click. The click had worked. N-110 shape. ⇒ ASK OF EVERY PROBE: what
   would this return if the code were RIGHT? Same answer ⇒ the probe is wrong.
⑤ 🔑 JOE RECALL HAS NOW BEATEN THE CANONICAL RECORD IN THREE CONSECUTIVE SESSIONS — the
   canvas intent, the lane-shape mismatch, the settings mechanism. IF JOE SAYS HE
   REMEMBERS SOMETHING DIFFERENTLY, SEARCH BEFORE DISAGREEING.
⑥ 📌 N-197 IS STILL OWED AND ITS WORDING IS JOE. It now covers SIX instrument failures
   across three seats.

═══ 🎯 WHAT M-INTRO-POLICY IS, AND WHAT IT IS NOT ═══
🔒 It is PROTOCOL + NODE + CLIENT. It is EXPLICITLY NOT A UI LEG. Do not let it become one.
🔒 D-143 STANDS: the filter is enforced in the CLIENT.
🔑 THE QUESTION IT OWNS THAT M-RP-INTRO DELIBERATELY REFUSED: PROMINENCE ON UNSOLICITED
   FIRST CONTACT. Joe placement puts the canvas between header and paragraph, so the
   FIRST thing a recipient sees from a stranger is the stranger composed canvas. I1 holds
   (message chrome, attribution directly above), but WHETHER THAT SHOULD RENDER UNASKED,
   AND UNDER WHAT POLICY, IS THIS MILESTONE.
🔓 Open and Joe: HEADLINE_MAX 120 / BLURB_MAX 600 are PROVISIONAL (D-138) — a policy that
   filters content should probably own its bounds. trust_assertion was BARRED from riding
   M-RP-INTRO and is still unrouted.

═══ 📋 ALSO OPEN — ROUTE, DO NOT DECIDE ═══
🟡 M-RP-INTRO-CANVAS — the settable welcome canvas: intro as a settings-hosted plugin.
   Joe named it. Phase-0 not yet written. Its ROADMAP Owes rows already carry: the
   header-to-paragraph placement, the two guard questions (grouped-row survival at line
   171 which is OUTSIDE the grouped guard; and the !deleted tombstone guard), the N-172
   binding (the wire carries DATA, never a widgetId, never markup), and the fact that
   message.svelte is core — a REAL core change, never a rider.
🔓 blurb → about rename: 35 code sites / 5 files. Naming is Joe. Sequences into the canvas.
🛑 ROUND-2 CONTRADICTION, UNRESOLVED: ROADMAP:290 marks Round 2 ✅ GO at J-390 while the
   J-735 kickoff carried "Round-2 still GATES UI COMPLETION". BOTH CANNOT BE CURRENT.
   Reconcile it in a session that starts on it.
🟡 M-RP-STARTUP · M-RP-WIDGET-SUSPEND · M-RP-PLUGIN-INSTALL-UI · M-RP-SKIN · M-RP-PEOPLE.
🟡 M-RP-REACTIONS stays DEFERRED and opens as PROTOCOL, not UI — who reacted is identity
   data on the no-anonymity core. Its lane fixtures live in the SAMPLER only.

═══ SEATS — D-123 ═══
JOE — architecture, appearance, product. Locks. PUSHES. Owns skin.css VALUES.
CHAT — grounding, measurement, records, verification, tooling, Phase-0 to disk.
       Self-drives operational work INCLUDING launching apps and killing processes.
       Supplies whole PowerShell commit+push command lines; Joe executes.
       Re-drives EVERY gate (Rule 5). Condensed answers.
CLAIR — implements from the LOCKED runbook. Never pushes. Reports deviations (Rule 6).
📌 RECOMMEND A COLD READ of any document someone will execute — pointed at the CLAIMS
   first, before the prose that justifies them. Clair cold read found seven defects on
   the last runbook, one of which made it unimplementable.
📌 D-074: any close is ATOMIC — JOURNAL + CLAUDE.md + ROADMAP + task docs in ONE commit.
   "Commit pushed" is NEVER a DoD item.

🛑 FIRST ACTION: git --no-pager status + git rev-parse HEAD + git ls-remote origin
refs/heads/main. Then read the documents IN FULL. Then re-drive the auth_tier claims with
worktrees EXCLUDED. Then author the Phase-0 — Chat writes it first, Joe locks it after.
```
