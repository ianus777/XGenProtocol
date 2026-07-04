# Handoff — UI tier discussion (behaviour-carrying assemblies) before M-RP4.3 scope
> **Status**: DEPRECATED  
> Version: 1.1  
> Date: Jul 2026  
> **Last updated**: 2026-07-04  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

**DEPRECATED (2026-07-04, J-451):** replaced by the `widget` tier concept-lock (J-445/N-059) — the UI-tier discussion this handoff carried is resolved; the deferred keyword-set editor is now scoped as widget-tier (M-RP4.3).

## Kickoff (Chat Claude seat)

**Rule-0 read-in first:** CLAUDE.md PLAY → JOURNAL J-444 (head) → `ui/docs/xgen-ui-notes.md` (N-058 head) → `ui/docs/xgen-ui-components.md` → then this file.

**This session is a DESIGN DISCUSSION. No code. Do not open M-RP4.3 scope yet** — it is blocked on the tier question below.

## State
M-RP4.4 fully closed + pushed (5 commits, head `639f0ce`). Working tree clean. Records consistent: J-444, N-058, ROADMAP v4.13, D-101, PLAY → next-active M-RP4.3.

## The question
M-RP4.3 (in-app TOML editor for the `[substitutions]` list) is the first UI unit that is **assembly + behaviour + host I/O** (load/save via a new `save_substitutions` Tauri command; fail-soft Tier-2 parsing deferred here per N-057). Our taxonomy stops at **di/dd primitives** + **di-composites** (N-054 — *passive* assembly, no logic). There is **no defined tier** for behaviour-carrying assemblies. "Widget" is a provisional label only — **not Joe-locked**.

## Open sub-questions (all unlocked)
1. One new tier, or two (behaviour-only vs I/O-carrying)?
2. Naming — "widget"/"feature"/"unit". **Avoid "module"** (collides with protocol modules / Tier-1 auth).
3. Library citizen or not? Chat lean: **app-level assembly** in `ui/common`, *consuming* core — keeps di/dd/composite pure; answers "which group" (none, a tier up) and "where".
4. Sampler implications — if app-level, does it belong in the component sampler at all, or need a different verify home?

## Inherited + locked (do not relitigate)
- **D-101 clean-slate-on-start:** editor writes to a config **wiped on next launch** — session-only round-trip this phase; must not hide that (D-065).
- Host already has `get_substitutions` (M-RP4.4); new leg = `save_substitutions` write-back (sampler host first — Clair's territory).
- Placement in real client/node (settings modal / route) is a **real-UI-arc** concern — deferred, NOT part of M-RP4.3.

## Deliverable this session
Discuss with Joe → land the tier definition (what qualifies, how it differs from di-composite, where it lives, sampler treatment) as an **N-note for his lock**. Only after that lock: draft M-RP4.3 scope + roadmap.
