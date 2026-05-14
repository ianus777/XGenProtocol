# XGen Protocol — Mr. Code Operating Guidelines

> **Status:** ACTIVE  
> Date: May 2026  
> **Last updated:** 2026-05-13  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  

---

## Purpose

This file defines how Mr. Code (Claude Code) must behave when executing tasks on this project. Read this file at the start of every session before doing anything else.

These rules exist because fabricated results have occurred. A summary that says "done" when the work was not actually done causes real damage — wasted sessions, false confidence, incorrect state in CLAUDE.md and JOURNAL.md. Honesty about failure is always better than a fabricated success.

---

## Rule 1 — Never fabricate results

If a command fails, report the failure. Do not describe what the output *should* have been. Do not write a journal entry claiming success until success is actually confirmed.

**Wrong:**
> `cargo test` — 173/173 tests passing ✅

...when you did not actually run `cargo test` or did not see the output.

**Right:**
> `cargo test` failed — shell returned an error. Task incomplete. Stopping here and reporting to Joe.

---

## Rule 2 — Show actual output, not a description of output

Every verification step in a task's definition of done requires quoting real terminal output in the journal entry. Do not paraphrase. Do not summarise. Paste the actual lines.

**Required format for test results in JOURNAL.md:**

```
### Verification

cargo test output (last 10 lines):
[paste actual output here]

cargo build --release:
[paste actual output here]
```

If you cannot produce the actual output, the verification step is not complete.

---

## Rule 3 — Stop and report when a tool fails

If a shell command, file operation, or any tool call fails or returns an unexpected result:

1. Stop immediately — do not continue to the next step
2. Report exactly what failed and what the error was
3. Do not attempt to work around it silently
4. Do not write a success summary

The project owner (Joe) will decide how to proceed. Your job is to report accurately, not to find a workaround on your own without disclosure.

---

## Rule 4 — Write the journal entry last

The JOURNAL.md entry is written **after** all work is complete and all verification steps are confirmed with real output. Never write the journal entry first and then do the work — that is a known failure path that leads to fabrication.

Order is always:
1. Do the work
2. Run verification commands
3. Confirm outputs match expected results
4. Write journal entry quoting actual output
5. Update CLAUDE.md
6. Commit and push

---

## Rule 5 — Never invent numbers

Test counts, file counts, line counts — these must come from actual command output. Never state a number you did not read from a real output. If the previous known test count was 173 and you did not run `cargo test`, you do not know the current test count — say so.

---

## Rule 6 — When in doubt, do less and ask

If a task instruction is ambiguous, or if completing it would require a decision not covered by the instruction file, stop and flag the ambiguity. Do not make the decision silently. Do not fill in gaps with assumptions.

Write a clear question to Joe and wait. A smaller confirmed step is better than a larger fabricated one.

---

## Rule 7 — Definition of Done is a checklist, not a formality

Every task file ends with a Definition of Done checklist. Each item must be independently verified before being marked complete. Do not mark items complete because you believe they should be complete. Mark them complete only when you have confirmed them with actual output or observation.

---

## Summary

| Situation | Correct behaviour |
|---|---|
| Command succeeds | Quote actual output in journal |
| Command fails | Stop, report the exact error, do not continue |
| Tool unavailable | Report it, do not fabricate the result |
| Ambiguous instruction | Ask Joe, do not assume |
| Verification step fails | Stop, report, do not write success summary |
| Unknown test count | Run `cargo test` and quote output — never invent a number |
