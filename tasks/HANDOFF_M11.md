# HANDOFF — M11 (`self` account) Session Kickoff
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jun 2026  
> **Last updated**: 2026-06-14  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Purpose

Entry point for the session that opens **M11 — the `self` account (D-021)**. Read after the
session-open reading order, not instead of it. This is a launch note, not a runbook or a design;
no M11 design exists yet.

## Session-open reading order (Rule 0 — mandatory, no exceptions)

1. `CLAUDE.md` PLAY block
2. Latest `JOURNAL.md` entry (J-375)
3. Any ACTIVE HANDOFF notes in `tasks/` (this file)

Runbook-as-ground-truth is a failure mode. There is no M11 runbook to start from.

## State you'll find

- **M10 (Auth Module Reference Set) CLOSED at J-375.** Sub-arcs M10.1–M10.5 all done.
- **Full multiparty suite is 37/37, 0 deferred** — that whole effort is closed (consolidated
  R1+R2+R3 ledger delivered at `tasks/HANDOFF_MP_R3.md` §3, addendum at J-375).
- `main` is in sync with origin. Nothing is mid-flight. **M11 opens cold.**

## Next-active = M11 — the `self` account (D-021)

Local-only synthetic Identity, accessible from any client. Promoted from a deferred open-area to
a numbered milestone at J-287 (Joe's explicit call). Design has **not** been started.

- **Spec:** `DECISIONS.md` (D-021 — "Self Account (`self`): Local-Only Synthetic Identity,
  Post-Phase-1"). Read it first.
- **Then ground before forming any view:** grep the ch-docs + the `xgen-client` identity layer
  for existing `self` / synthetic-identity surfaces; check how `self` would interact with
  registration, `home_node`, and the auth-module gate (is `self` a real keypair-backed Identity
  or a synthetic stand-in?).

## How M11 opens (arc discipline)

D-071 Phase-0 audit first (ground D-021 against the live code) -> design -> Joe-lock -> runbook
-> Clair implements -> Chat doc-bridge -> close. **No code before Joe locks the design.** Discuss
the shape with Joe first; present options with a clear recommendation and rationale, then let him
lock. Joe never pre-decides items that belong to his lock.

**First action:** complete the reading order, confirm the on-disk state matches this note, then
propose the M11 Phase-0 scope (what to ground, candidate forks) for Joe to lock. No code yet.

## Post-M11 chain (J-357, authoritative)

M11 -> M12 (attachments) -> Round-2 final pre-UI whole-codebase audit (the UI gate) -> UI ->
Streams (standalone, post-UI). The Round-2 audit is **not** next — it sits after M11/M12,
immediately before UI.

## Routed-open items (named homes, non-blocking — keep in view, don't re-open)

- **MP-F12** — departed-signer (own home)
- **MP-F2-followon** — 7 unmapped wire-codes
- **MP-F15** — migration-depth arc (destination admission keyed on home-ownership;
  `verify_transfer` robust to a pre-existing replica)
- **MP-F16** — federation-endpoint inconsistency (`federation_initiate` advertises
  `config.node.listen` raw, `admin_ops.rs:1784`, vs the `--port`-corrected `effective_endpoint`,
  `app.rs:704`); low-sev, harness-cleared at J-375.

## Standing discipline (do not relearn the hard way)

- `Filesystem:*` for everything on `E:\` — **never** `create_file` (that writes to the Claude
  sandbox `/mnt/`, not Joe's disk). `get_file_info` after every write to verify bytes on disk.
- Doc-bridges are atomic (D-074): JOURNAL + CLAUDE PLAY + ROADMAP + task docs travel in one
  commit. ROADMAP state changes pair with CLAUDE in the same commit, six-symbol vocabulary
  (PLAY / DONE / PENDING / POSTPONED / CANCELLED / DEPRECATED).
- `.md` headers: every file update bumps the header (Status / Version / Date / Last updated);
  the `> **Last updated**:` line is `YYYY-MM-DD` only, no parenthetical change-notes.
- Task-file DoD never lists "commit pushed" (unflippable inside the commit that pushes);
  `Status: COMPLETED` is the real shipped signal.
- **Joe pushes — never push.** Hand him the full PowerShell block: `cd` -> explicit
  `git add <file>` per file (never `git add .`) -> `git status` -> `git commit` (one `-m` per
  paragraph) -> `git push` on its own line.
- Condensed answers. Honest behaviour over polite (D-065) — surface gaps, don't paper over them.

## When M11 actually opens

Flip this file's Status ACTIVE -> COMPLETED (or supersede it with the M11 Phase-0 brief) as part
of the first M11 doc-bridge, so it doesn't linger as a stale ACTIVE handoff.

**Done (J-376):** superseded by `tasks/M11_SELF_THREAD_PHASE0_BRIEF.md` (v1.0 ACTIVE). M11 OPENED; concept Joe-LOCKED (Node-side never-federated self-DM, reuses the user's existing keypair); Phase-0 scope locked. Status flipped at the M11-open doc-bridge.
