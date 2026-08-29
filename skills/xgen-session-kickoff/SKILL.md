---
name: xgen-session-kickoff
description: Use when Joe says "new session", "to clair", or otherwise asks for a kickoff or handoff message for XGenProtocol. Also fires on "kickoff message", "handoff to Clair", "go to the new session", "kickoff for future Claude". Produces a session-open message in one of two modes - Claude mode (default) or Clair mode (when "clair" appears).
---

# XGen Session Kickoff Skill
> **Status**: ACTIVE  
> Version: 1.00  
> Date: Aug 2026  
> **Last updated**: 2026-08-29  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

## Purpose

Emit a session-open message for XGenProtocol. Two modes, one skill. Mode is
selected by whether "clair" appears in the triggering message.

- No "clair" -> Mode A, kickoff to future Chat Claude.
- "clair" present -> Mode B, kickoff to Clair (Code Claude).

## Procedure (both modes)

Read, in this order, before writing anything:

1. `E:\Projects\XGenProtocol\CLAUDE.md` - PLAY block
2. Latest entry in `JOURNAL.md` (newest-first, J-NNN)
3. Any ACTIVE HANDOFF notes in `tasks/`
4. The doc pointed to by steps 1-3

Then reconcile against the current chat. Repo files are authoritative for what
has shipped. The chat is authoritative for what has been decided but not yet
written. Both go in the output.

Never infer state from a runbook alone. Runbooks are item 4, not item 1.

## Mode A - kickoff to future Chat Claude

```
XGenProtocol - session kickoff

REPO STATE (from files)
- Active milestone: {ID + short descriptive title}
- Last JOURNAL entry: {J-NNN - one line}
- Active handoffs in tasks/: {list, or none}
- Pointed doc: {path}

UNCOMMITTED (this chat, not yet in any file)
- {decisions reached, gaps found, framing corrections}
- or: nothing uncommitted

OPEN WORK
- Written, awaiting push: {paths, or none}
- Promised, not yet written: {paths, or none}

CUT POINT
- {closed cleanly at X} | {mid-X, stopped at Y}

NEXT ACTION
- {single concrete next step}

STANDING REMINDERS
- Reading order: CLAUDE.md PLAY -> JOURNAL -> tasks/ HANDOFF -> pointed doc
- D-121 two-lens on every recommendation: user-visible impact, then resource cost
- D-123: Claude does not under-step; write the Phase-0 first, then hand over to lock
```

## Mode B - kickoff to Clair

```
XGenProtocol - Clair kickoff

RUNBOOK
- Locked runbook: {path}
- Scope: {steps N..M}

REPO STATE
- Branch / working tree: {clean | notes}
- Build/test baseline: {passing counts}

SCOPE BOUNDARY
- Implement only from the locked runbook
- Report deviations, do not resolve them
- Never push; Joe pushes all commits
- Do not touch ui/assets/skin.css

DEPLETION RULE
- On depletion warning: stop at the next clean step boundary,
  write tasks/HANDOFF-{runbook}-{NNN}.md recording steps completed,
  steps remaining, deviations, and build/test state.
  Do not start a step that cannot be finished.
```

## Rules

- Every section is always present. If a section is empty, write `none`.
  A missing section is ambiguous; an empty one is a signal.
- Milestone IDs always carry a short descriptive title. Never a bare ID.
- If the uncommitted delta is larger than a few lines, do not lengthen the
  kickoff. Write `tasks/HANDOFF-{topic}-{NNN}.md` first, then point at it.
  That escalation is Claude's call, not Joe's request.
- Output the message only. No preamble, no commentary after.
