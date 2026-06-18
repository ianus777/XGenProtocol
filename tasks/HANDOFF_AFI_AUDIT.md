# AFI Audit Handoff — Appendix F / Appendix I audit-against-code
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-18  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

## 0. What this is
Runbook for the first pre-UI arc after the documentation-optimization phase (COMPLETE, J-396). Reconciles two doc surfaces to the as-built code, **code as ground truth**:
- **Appendix F** (`docs/xgen_appendix_f_en.md`, v1.12) — client CLI reference — vs the `xgen-client` verb surface.
- **Appendix I** (`docs/xgen_appendix_i_en.md`, v1.6) — data structures — vs the `xgen-common`/`xgen-core` serializable types + protocol event catalog.
Gates the UI build: UI couples to the verb surface and renders the data structures, so both must match reality first (D-071 — subsystem audit precedes the dependent milestone).

## 1. Rule-0 reads (first, in order)
1. `CLAUDE.md` PLAY head — doc-opt COMPLETE; next frontier = this audit.
2. Latest `JOURNAL.md` entry — J-396 (DO-5 close).
3. This handoff.
4. Then `docs/xgen_appendix_f_en.md` / `docs/xgen_appendix_i_en.md` and the code.

## 2. Scope (Joe-locked)
- **Q1** One arc, two sub-passes: **AF** then **AI**.
- **Q2** Order: **F first, then I** (F is UI-proximate + has a known gap).
- **Q3** Finding IDs arc-local (D-069): **AF-F##** / **AI-F##**.
- **Q4** Reconciliation default: fix the **doc** to match code; **Joe-route** only suspected **code** bugs (a real defect, not a doc drift).
- **Q5** Phase-0 first deliverable: read-only **as-built inventory** before any diffing.
- **Q6** Dev/test harness verbs: **in scope for Appendix F**, documented in a clearly-marked "developer / test harness" section (present-but-segregated).

## 3. As-built inventory (Phase-0, read-only)
**AF surface — canonical `xgen-client/src/app.rs`** (`ClientCommand` + `ThreadCommand` + `AiCommand`). clap kebab-case default; only `self` is name-overridden. 31 leaf verbs:
init, whoami, status, spaces, rooms, version, register, create-space, create-dm-space, self, create-room, invite, ban, room-update, thread {create, resolve, archive}, join, leave, send, history, fetch (alias fetch-attachments), redact, members, ai {delegate, revoke, status}.
Dev/test harness (Q6, segregated): smoke-test, stress-test, smoke-ph2, stress-complete.

**AI surface — canonical `xgen-common/src/`** = 57 serializable `pub struct`/`pub enum` (wire.rs 10, state.rs 9, trust_assertion.rs 7, event_trace.rs 5, envelope.rs 4, module.rs 4, cmd.rs 3, codes.rs 2, bindings.rs 2, clock.rs 2, others 1 each) + the protocol event-type catalog (message.*, state.*, membership.*, thread.*, identity.*, space.*, room.*).

## 4. Method
- **AF:** for each of the 31 verbs — (a) present in Appendix F? (b) args match the `*Args` struct? (c) all four D-092 dispatch arms exist (CLI / run-path / batch / aicontrol)? Then reverse: every Appendix F verb still exists in code. Findings table: AF-F## | verb | drift type (doc-missing / code-missing / arg-mismatch / arm-gap) | severity | route.
- **AI:** enumerate the 57 types + event catalog from code; diff vs Appendix I both directions (D-077 forward-drift AND backward-coherence). Findings table: AI-F## | type/event | drift type | severity | route.

## 5. Reconciliation + close discipline
- Default fix = doc edit (code is truth). Loop-to-green: every finding closed green-to-criterion or Joe-routed with reason; no row left in-process (round-close discipline).
- Suspected code bugs are NOT fixed here — filed as Joe-routed findings (this is a doc audit, not a code arc).

## 6. Milestones
- **Phase-0** ✅ (this open) — scope lock + method + as-built inventory.
- **AF** ✅ (J-397) — verb diff done; Appendix F reconciled (AF-F01/F02/F04/F06 + AF-F03 reframe + §F.2.1 cross-ref); v1.12→v1.13.
- **AI** — structure diff → findings → reconcile Appendix I → loop-to-green; version bump.
- **Close** — consolidated AF+AI ledger; both appendices reconciled; D-074 canonical close (JOURNAL + ROADMAP + CLAUDE atomic). Next: mockup stock-take + reconcile-to-as-built.

## 7. Operational learnings (carried forward)
- `Filesystem:*` for E:\ reads/writes; never create_file (sandbox).
- New-file writes: PowerShell .NET writer (UTF-8 no BOM, LF): `$enc=New-Object System.Text.UTF8Encoding($false)`; `[System.IO.File]::WriteAllText(path, ($arr -join [char]10)+[char]10, $enc)`. `Filesystem:write_file` is unreliable here.
- read: `Get-Content -Encoding UTF8`. Keep verification in a SEPARATE call from the write.
- Doc edits: index-reassign in PowerShell with a guard assertion on the target line, or `Filesystem:edit_file` with ASCII-only anchors (em-dash anchors unreliable).
- Header MUST be refreshed (Last updated + version) on every appendix edit.

## 8. Hygiene
- `tasks/HANDOFF_DO5_JOURNAL_WINDOWING.md` work is pushed; flip to COMPLETED + archive to `tasks/archive/` (DO-2 convention) — fold into a close commit.
