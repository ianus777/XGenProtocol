# Handoff — M-RP5.0 `entity-avatar` (Clair build seat)
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jul 2026  
> **Last updated**: 2026-07-05  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

Clair kickoff for the first dd-atomic. **Design is locked — build only.** Explicit Joe "go" before any file writes.

---

## Session-open reading (Rule 0, in order)
1. `CLAUDE.md` PLAY block (head J-461/J-462).
2. latest `JOURNAL.md` entry.
3. this handoff.
4. **then** `tasks/RUNBOOK_ENTITY_AVATAR.md` (ground truth — item 4, not 1).
5. UI session ⇒ also `ui/docs/xgen-ui-notes.md` + `ui/docs/xgen-ui-components.md`.
Cross-ref: `docs/xgen-dd-entity-avatar-phase0.md` (subsystem audit).

## What / where
- Build `entity-avatar` — dd-atomic, root `<figure class="entity-avatar" role="img">`, at `ui/core/lib/components/data-dependent/entity-avatar.svelte` (first `data-dependent/` occupant).
- Test-bed = sampler only (D-097), **DD·atomic** panel (empty placeholder, populate it). CDP 9422.

## Locked (do not re-litigate)
- **A** descriptor `EntityDescriptor { kind, name?, id, flags{isAi?,revoked?,isDm?,e2e?}, image? }` — `image` reserved-unfed; `core` imports **no** protocol types.
- **B** dd-root rule = honest HTML, class×arity from folder+panel+getter (N-075 at close). `<figure role="img">`, `aria-label={name ?? kind}`, `<figcaption>` reserved-unused.
- **C** identity+DM = circle · non-DM space = rounded-square.
- **D** `isAi` badge + `revoked` grey+slash via self-drawn `::after`/`::before` (no nested `led`). `e2e` deferred.
- **E** `seedColour(name ?? id)` — shared helper factored from `chip`; chip output must stay byte-identical.
- **F** variants `presence` (xs, glyph) · `list` (sm, glyph+initials); size/content derived per variant.
- **G** getter `{ kind, variant, name, initials, seed, flags }`.
- **H** reserve `onActivate?`, don't build.

## Build order (full detail in runbook)
1. `seedColour` helper ex-`chip` → re-verify chip 0-regression.
2. type + `entity-avatar.svelte` (variant presets, kind→shape, initials-or-xgid fallback, badges, `onActivate?`+`<figcaption>` seams, getter G).
3. `.entity-avatar` skin (seed-coloured, **no** accent dependency).
4. sampler DD·atomic cells: identity/space/DM × presence/list + absent-name + revoked + isAi.
5. CDP verify (quote real output, Rule 2): getter fields, shape-per-kind, badges, fallback, seed shell-independence, registry delta, **0 orphans**, screenshot.

## Discipline
- D-078 grounding, D-065 surface-don't-paper, Rules 1–7 (no fabricated output/counts).
- **D-074 atomic close** — all records one commit: ui-notes N-075, registry v0.47, ROADMAP (M-RP5.0 ✅ + v-bump), phase0 lock, CLAUDE PLAY→J-462, JOURNAL J-462 (last), runbook→COMPLETED. No DECISIONS touch.
- `.md` header updated every edit (two trailing spaces per `> ` line; `Date` MMM YYYY; `Last updated` YYYY-MM-DD only).
- `Filesystem:*` on `E:\` — one write → verify → next. Never push (Joe pushes).

## Definition of Done
Mirror the runbook DoD checklist; each item verified with real output before ticking.
