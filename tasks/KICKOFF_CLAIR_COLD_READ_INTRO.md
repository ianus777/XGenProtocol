# CLAIR — cold read of the M-RP-INTRO close claims (J-735 / J-736 / J-737)
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
XGen Protocol — session kickoff (CLAIR)

🎯 THIS SESSION IS A COLD READ. IT IS NOT AN IMPLEMENTATION SESSION.
🛑 THERE IS NO LOCKED RUNBOOK. Do not write product code. If you find yourself editing
   a .rs, .ts or .svelte file to FIX something, STOP — the finding is the deliverable.
🛑 YOU NEVER PUSH. Report deviations (Rule 6). Joe pushes.

═══ WHY YOU, AND WHY NOW ═══
🔑 THE ARC RECORD IS EXPLICIT: CHAT OWN RE-READS HAVE NEVER ONCE CAUGHT A DEFECT IN THIS
   ARC. Every real defect came from Joe recall, from you reading, or from a live run.
📌 Your last cold read found SEVEN defects, ALL confirmed, one of which made the runbook
   UNIMPLEMENTABLE (the payload had no author).
🛑 IN THE J-735 SESSION CHAT PRODUCED THREE DEFECTS OF ITS OWN AND CAUGHT ALL THREE LATE,
   BY DIFFING LINE NUMBERS — NOT BY READING:
     · published "39 hits / 9 files" when its own grouped output summed to 50, and PUSHED
       it twice before noticing,
     · ran a blanket apostrophe replace over ALL of CLAUDE.md and destroyed legitimate
       empty-string literals in three OLDER entries,
     · spliced a six-line block over a five-line one and overwrote the next section header.
   ⇒ ASSUME THE RECORDS FROM THAT SESSION CONTAIN MORE OF THIS. That is your target.

═══ 📖 READ, IN THIS ORDER ═══
CLAUDE.md PLAY block (top = J-737, then the M-RP-INTRO close block)
JOURNAL.md J-737 · J-736 · J-735
tasks/RUNBOOK_M_RP_INTRO.md v1.4 COMPLETED → §10 (the close)
tasks/M_RP_INTRO_PHASE0.md v1.5 COMPLETED → §11 (what it carried out)
docs/ROADMAP.md → the M-RP-INTRO node and the M-RP-INTRO-CANVAS node

═══ 🎯 METHOD — CLAIMS FIRST, PROSE SECOND ═══
🔑 GO AT THE CLAIMS BEFORE THE PROSE THAT JUSTIFIES THEM. A claim is any sentence with a
   number, a file path, a line number, a symbol name, or the word ZERO / ONLY / NEVER.
🛑 FOR EACH ONE ASK: what would this measurement return if the code were RIGHT? If the
   answer is the same as what it returns when the code is WRONG, THE CHECK IS NOT A CHECK.
🛑 DO NOT ADOPT CHAT NUMBERS. Re-drive every one you touch. That is Rule 5 and it applies
   to you reading Chat exactly as it applies to Chat reading you.

═══ 🔒 THE CLAIM LIST — RE-DRIVE, DO NOT INHERIT ═══
① stream-panel.svelte — echoToDescriptor appends the intro mount AFTER an unconditional
   send-status mount. Claim: send-status stays FIRST and unconditional in the array.
② the widget registry holds EXACTLY send-status + message-intro. Claim: two entries.
③ blurb = 35 CODE sites across 5 files (composer-panel 10 · derive.test.ts 10 ·
   message-intro 9 · derive.ts 4 · exchange.rs 2). ⚠️ EXCLUDE \.claude\ — see the trap
   section. Chat published 39/9 and it was WRONG. Verify the corrected figure too.
④ skin.css — claim: .message-intro* had ZERO rules before J-735, and now has exactly the
   order/flex rule plus .message-intro-headline and .message-intro-blurb.
⑤ --fs-2 is 14px, --fs-1 is 12px, --lh is 1.5, and .message[data-kind="system"] pins
   --fs-1 so system notices stayed 12px. Claim: no new token was minted.
⑥ 🔑 THE ONE JOE PLACEMENT DECISION RESTS ON: message.svelte line 171 sits OUTSIDE the
   {#if !grouped} guard, which closes at :170. If that is wrong, the canvas guard analysis
   in the ROADMAP is wrong. LINE NUMBERS ARE HOSTAGES — RE-CITE BY SYMBOL.
⑦ message-intro.svelte contains NO {@html} and NO font-size/colour/weight.
⑧ NodePolicy: 23 hits in xgen-core/src/space/node_policy.rs, read ONLY by xgen-node
   (admin_ops 11 · pipe 6 · app 4 · aicontrol 3), and ZERO hits in xgen-client/ or ui/.
⑨ 🛑 NOT YET VERIFIED BY ANYONE — build_dm_space_create_event hardcodes auth_tier: 1, and
   auth_tier has zero hits in the client. Chat grep was drowned by worktree noise and the
   REAL CALL SITE WAS NEVER READ. This one is genuinely open. Drive it.
⑩ the worktree census: 8 dirs on disk, 4 registered with git worktree list, 4 orphaned.

═══ 🔧 TRAPS THAT WILL BITE THIS SPECIFIC SESSION ═══
🛑 .claude/worktrees/ HOLDS 8 FULL SOURCE TREES FROM MAY 2026 AND git status CANNOT SEE
   THEM (.gitignore:10). EVERY repo-wide grep MUST exclude \.claude\ alongside \target\
   and node_modules. MEASURED: auth_tier 219 raw vs 107 true (51%), EventType 2519 vs
   957 (62%). AN INFLATED COUNT LOOKS COMPLETE, PLAUSIBLE AND IS WRONG.
🛑 cargo test --workspace EXCEEDS THE MCP TIMEOUT. Run detached, log to a file, and APPEND
   YOUR OWN EXIT SENTINEL (echo CARGO_EXIT=%ERRORLEVEL%). A wrapper shell exit code is NOT
   cargo exit code — you proved that the hard way, and a PARTIAL READ of the log looks
   complete and plausible (1280/0/60 × 44 against the true 1602/0/62 × 56).
🛑 Select-String is CASE-INSENSITIVE BY DEFAULT — "FAILED" matches "0 failed" in every
   PASSING line. Use -CaseSensitive.
🛑 An unquoted or badly quoted pattern can expand to EMPTY and then match EVERY LINE. If a
   count equals the total line count, SUSPECT THE PATTERN, not the file.
🛑 CLAUDE.md and docs/ROADMAP.md are CRLF. EVERYTHING ELSE IS LF. git ls-files --eol is
   the authority. Filesystem:edit_file STRIPS CR.
🛑 [System.IO.File] methods NEED ABSOLUTE PATHS.
🛑 DO NOT KILL a node.exe without reading its CommandLine — the Filesystem MCP server runs
   as node.exe. Kill by xgen* name only.

═══ 📤 WHAT TO HAND BACK ═══
A numbered findings list. For EACH finding: the claim as written, WHERE it is written
(file + section), what you measured, and the delta. Mark each CONFIRMED / WRONG / CANNOT
VERIFY. ⚠️ CANNOT VERIFY IS A LEGAL AND USEFUL ANSWER — a gate that cannot be run is worse
than one left open, because it reads as satisfiable.
🛑 DO NOT FIX ANYTHING IN THE RECORDS. Chat owns the records seat. Report, do not edit.
🔑 IF YOU FIND A WRONG NUMBER, GREP FOR THE WHOLE CLASS BEFORE REPORTING IT FIXED —
   the "39" appeared in FOUR documents, and a correction applied only to the cited instance
   is not a correction.

🛑 FIRST ACTION: git --no-pager status + git rev-parse HEAD + git ls-remote origin
refs/heads/main. Then read the documents IN FULL. Then go at the claim list.
```
