# DO-5 Handoff — JOURNAL.md Windowing (riskiest doc-opt sub-step)
> **Status**: COMPLETED  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-18  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

## 0. What this is
Runbook for **DO-5**, the final and riskiest sub-step of the documentation-optimization phase. DO-1..DO-4 are CLOSED and pushed (latest J-395). DO-5 windows the development JOURNAL: keep a recent live window in `JOURNAL.md`, relocate older entries into a new archive file with a forward pointer (D-094 convention). Riskiest because `JOURNAL.md` is the project append-only authorship / IP-timeline record — no entry may be lost, altered, or reordered. DO-5 is a pure relocation: exact bytes, exact order, zero content edits.

## 1. Rule-0 reads (first, in order)
1. `CLAUDE.md` PLAY head — doc-opt phase, DO-4 done, next-active = DO-5.
2. Latest `JOURNAL.md` entry — J-395 (DO-4 close).
3. This handoff.
4. Runbooks are item 4 on the reading stack.

## 2. Survey (read-only, taken at DO-5 open)
- `JOURNAL.md`: 17,761 lines, 2.38 MB, 378 entries.
- Order: reverse-chronological — newest J-395 at top (L11), oldest at the bottom.
- Tail bottoms out around J-048 / J-046 / J-047 (slightly out of order; file does NOT reach J-001, earliest entries predate this file). Note only, not DO-5 work.
- Header is the SPECIAL short IP-record header (Status + Last updated + a 3-line IP preamble), NOT the 8-line doc header. Preserve verbatim on the live file; mirror it (plus ARCHIVED status and a D-094 pointer) on the archive.
- No JOURNAL archive exists yet (the worktree copies under .claude are git worktrees — ignore). DO-5 creates it fresh, like `CLAUDE_HISTORY.md` in DO-1.

## 3. CRITICAL operational learnings (a fresh session will not have these)
- **Filesystem write_file is UNRELIABLE here** — reports success but new files may never land on disk. Do NOT use it for new files. Use the PowerShell .NET writer.
- **Reliable read/write:**
  - read: Get-Content with -Encoding UTF8 (without it you get mojibake).
  - write: $enc = New-Object System.Text.UTF8Encoding($false) (UTF-8 no BOM); $lf = [char]10; [System.IO.File]::WriteAllText with ($arr -join $lf)+$lf.
  - single-quoted here-strings preserve emoji, em-dashes and backticks literally. Keep verification in a SEPARATE call from the write (a parse error in the same script aborts the write).
- **GUARD before every destructive write** (entry-count + line-count assertions). Git is the backstop: git checkout -- JOURNAL.md restores pristine (all canonical files committed at HEAD).
- **Slice on-disk arrays** for bulk moves; NEVER route 2.4 MB of journal through context.
- **git mv strands an unstaged edit** — if a file is renamed AND edited, git add the edit BEFORE git mv (or mv first, edit at new path, then add).
- `Filesystem:*` for all E:\ reads/writes; never create_file (sandbox). Windows-MCP PowerShell for git / Select-String / slicing.

## 4. Design questions for Joe (propose, then lock, then execute)
Goal: recent live window stays in `JOURNAL.md`; older entries move to `JOURNAL_ARCHIVE.md` (ARCHIVED) with a forward pointer at the cut and a back-pointer in the archive (D-094).
- **Q1 cut point.** (A) **By milestone/arc — keep from the start of a recent arc forward (e.g. doc-opt phase, or the M11 arc); cleanest semantic cut, anchor Joe-locked. RECOMMENDED.** (B) By entry count — newest ~40-60. (C) By line budget — newest ~X lines.
- **Q2 archive shape.** Single growing `JOURNAL_ARCHIVE.md` (recommended) vs ranged files.
- **Q3 filename.** `JOURNAL_ARCHIVE.md` (parallels `CLAUDE_HISTORY.md`) — confirm.
- **Q4 pointer text** at the cut (live) and at the archive top.

## 5. Execution shape (after lock)
1. Read array with -Encoding UTF8. Find cut = header line of the oldest entry to KEEP (Select-String for the anchor entry header).
2. live = header/preamble + entries[top..cut-1] + forward-pointer block. archive = archive-header + entries[cut..end].
3. Slice arrays; write both with the .NET writer.
4. **GUARD:** archived-count + live-count == 378 (+1 once J-396 added); no entry header lost; re-count both files and assert the sum.
5. Spot-check first + last entry of each file render intact.

## 6. Close (D-074 canonical, doc-only)
- `JOURNAL.md` (windowed) + new `JOURNAL_ARCHIVE.md`.
- New entry **J-396** (the windowing) written into the LIVE window.
- `CLAUDE.md` head: DO-5 done AND doc-opt phase COMPLETE (next frontier = Appendix F/I audit-against-code).
- `docs/ROADMAP.md`: DO-5 marker + doc-opt node/arrow flipped done + Present advanced to the audit step + version bump.
- Decide: D-094 already covers JOURNAL windowing (recommend yes, add a one-line note) vs a new D-095.
- Guard clean, then hand the commit+push block (explicit git add per file, git status, multi -m ASCII-only paragraphs avoiding em-dashes, git push on its own line).
- Suggested subject: docs(doc-opt): J-396 DO-5 - window JOURNAL.md; archive older entries; doc-opt phase COMPLETE

## 7. After DO-5
Doc-opt phase closes. Per the Joe-locked pre-UI chain: next is Appendix F/I audit-against-code, then mockup stock-take + reconcile-to-as-built, then UI, then Streams.
