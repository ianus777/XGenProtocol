# Runbook — M-RP5.1a `status` + M-RP5.1b avatar amendment
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-05  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

Build runbook for `status` (dd-atomic) + the `entity-avatar` status-slot amendment. Design locked in Phase-0 (`docs/xgen-dd-status-phase0.md`, A–G). One runbook — badge is the avatar slot payload, so they ship together. Session-open order applies (Rule 0). No push — Joe pushes.

---

## Locked design (A–G)

- **A** name `status`.
- **B** variants `badge` (emoji corner) / `line` (emoji+text) / `full` (emoji+text+relative time); text-absent → emoji only; no-room → tooltip; expired → absent.
- **C** root `<span class="status" role="img">` (badge) / `<span class="status">` (line/full).
- **D** avatar seam: `entity-avatar` gains `status?` slot → renders `status variant="badge"` as bottom-right corner overlay.
- **E** tooltip `title` = text (+expiry) on emoji-only/no-room.
- **F** expiry: expired→absent (lazy); `full` shows relative "updated 5m ago".
- **G** getter `{ variant, emoji, hasText, expired }`.

## Step 1 — `status` component

- `ui/core/lib/components/data-dependent/status.svelte`:
  - props: `status: { emoji?: string; text?: string; updatedAt?: string; expiresAt?: string }`, `variant: 'badge'|'line'|'full'` (default `'badge'`), `id`.
  - view-model only (source-agnostic; no protocol import). Shell maps `StatusRecord`→this.
  - derived: `expired` (lazy, `expiresAt < now`); render nothing if expired or no emoji+no text.
  - `badge` = `<span role="img" aria-label={text ?? emoji}>` emoji only; `line` = emoji + text; `full` = + relative time from `updatedAt`.
  - `title` fallback (E). getter G via `use:envelope`.
- DoD: grapheme-safe emoji; expired→absent verified.

## Step 2 — `entity-avatar` amendment (M-RP5.1b)

- `entity-avatar.svelte`: add `status?` prop (same view-model). When present + not expired, render `<Status {status} variant="badge" />` as a positioned corner overlay (`.entity-avatar .status` bottom-right). Child self-registers (`__status`).
- Additive only; presence/list/labeled/card untouched; re-verify M-RP5.0/5.1 cells 0-regression.
- Reserve bottom-right corner deliberately (future presence dot, if ever, takes a different corner).

## Step 3 — skin

- `.status` in `ui/skin.css`: badge = small emoji chip, absolute corner when inside `.entity-avatar` (bottom-right, ring against avatar bg); line/full = inline emoji + text (`--fs-1`, muted text). PROVISIONAL. Accent-neutral (emoji carries its own colour).

## Step 4 — sampler (DD·atomic panel)

- `status` cells: `badge`/`line`/`full` × {emoji-only, emoji+text, expired, no-emoji}. Plus an `entity-avatar` cell **with** `status` (badge corner overlay) to prove the seam.
- Stable ids; avatar-with-status registers `__status`.

## Step 5 — CDP verify (sampler 9422)

- `vite build` clean; kill zombies + confirm served module (N-058).
- Assert: getter G per cell; `badge` emoji-only; `line`/`full` text; expired→absent (not rendered); tooltip `title`; avatar `__status` registered as corner overlay; registry delta; **0 orphans**. Re-verify avatar/item cells unchanged. Quote real output (Rule 2). Screenshot.

## Step 6 — D-074 atomic close (one commit)

- `ui/docs/xgen-ui-notes.md` → N-077 (`status` + avatar status-slot), v-bump.
- `ui/docs/xgen-ui-components.md` → registry v0.49 (`status` row; avatar row +`status?`), v-bump.
- `docs/ROADMAP.md` → M-RP5.1a + 5.1b ✅ DONE, v-bump.
- `docs/xgen-dd-status-phase0.md` → COMPLETED.
- `CLAUDE.md` PLAY → J-464.
- `JOURNAL.md` → J-464 (last, real CDP output).
- this runbook → COMPLETED, v-bump.
- No DECISIONS touch (arc-local).

---

## Definition of Done

- [ ] `status.svelte` built; variants badge/line/full; expired→absent; getter G; no protocol import.
- [ ] `entity-avatar` `status?` slot; badge corner overlay; `__status` registers; 5.0/5.1 0-regression.
- [ ] `.status` skin (corner badge + inline line/full).
- [ ] sampler cells (variant×state + avatar-with-status).
- [ ] CDP-verified: getter, variant render, expiry-absent, tooltip, avatar seam, registry delta, **0 orphans** — real output quoted.
- [ ] records closed atomically (D-074): N-077, registry v0.49, ROADMAP, phase0→COMPLETED, PLAY→J-464, JOURNAL J-464, runbook→COMPLETED.
