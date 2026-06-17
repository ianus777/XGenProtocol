# M6 Phase 3 — Read-only completions on existing --batch  [COLLAPSED]
> **Status**: COMPLETED  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-29  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Outcome — collapsed to zero (Joe-locked 2026-05-29)

Phase 3 ships **no commits**. The M6 design §5.1 named "Phase 3 — read-only completions
on existing `--batch`" as a distinct step, but that line predates Block 4. Block 4 (J-151)
enumerated the verb set and bucketed **every READ verb with its category** — `federation
list` ships in Phase 7 with the federation writes, `identity show` in Phase 5, `bootstrap
show` in Phase 6, `auth-module list`/`test` in Phase 8, `space list-hosted`/`show-node-policy`/
`audit-events` in Phase 9, `plugin list`/`status` in Phase 10, `log show-level`/`audit query`/
`audit export` in Phase 4 (see `docs/xgen_appendix_k_en.md`). Each of the seven category
phases (4–10) therefore ships its reads and writes together, leaving Phase 3 with no
enumerated verbs.

Phase 3 is collapsed to zero — the sibling outcome to the R3 path Phase 1 nearly took.
Surfaced as a §5.1-vs-Appendix-K conflict at Phase 3 start (Rule 6); Joe confirmed the
collapse. The canonical correction is recorded in the design doc §5.1 (v1.10 → v1.11).

The existing M2 read-only `--batch` surface (`status`, `connections`, `peers`, `spaces`,
`identity list`, `version`, `whoami`) is unchanged and complete for what M6 needs at this
stage; no read gap requires a separate step.

## Next

**Phase 4 — A6 Logging & audit** (M6 design §6.A6 + Appendix K.2.1). Lands the audit-write
primitive (`audit::insert_entry` from the Phase 2 skeleton becomes load-bearing) that every
later WRITE/DESTRUCTIVE verb consumes, plus 5 verbs: `log set-level` (WRITE), `log show-level`
(READ), `audit archive` (DESTRUCTIVE), `audit query` (READ), `audit export` (READ). Per-phase
file: `tasks/M6_PHASE_4_IMPL.md`.

---

*End of Phase 3 (collapsed).*
