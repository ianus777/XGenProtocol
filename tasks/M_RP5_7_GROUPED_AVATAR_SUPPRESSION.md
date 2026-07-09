# XGen Protocol — M-RP5.7 Runbook: grouped-avatar suppression (Clair)
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-09  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. Read-order (Rule 0) + spec of record

Before touching code: CLAUDE.md PLAY → JOURNAL J-486 → this runbook. **Spec of record = `docs/xgen-dd-message-family-phase0.md` §10 (M-RP5.7, LOCKED J-486)** + **`DECISIONS.md` D-106**. This runbook is the build sheet; if it and §10 / D-106 disagree, the spec + decision win — stop and flag (Rule 6).

## 1. Scope — one small correction, not a new feature

M-RP5.5 B made a `grouped` row suppress the **name header** but **keep the avatar**. That repeats the identical avatar down a same-author run (the who-is-speaking noise grouping exists to remove) and makes grouping read as invisible. **This milestone makes `grouped` suppress BOTH name and avatar.** `grouped` is already the stream-computed prop `message-stream` sets — no stream change.

**Touch only:** `ui/core/lib/components/data-dependent/message.svelte` (grouped branch) + `ui/assets/skin.css`.
**Do NOT touch:** `MessageDescriptor`, `ui/core/lib/components/data-dependent/stream/grouping.ts`, `message-stream.svelte`. No node↔client channel (sampler fixtures only).

## 2. The change (Phase-0 §10 / D-106)

- **Group head** (`!grouped`): unchanged — renders avatar (`__avatar`) + name (`__name`) + body.
- **Grouped continuation** (`grouped === true`): render **body only** — do **NOT** render the `entity-avatar` child, do **NOT** render the name `label`. Name is already suppressed at B; add the avatar to the same suppression.
- **Element absent, NOT `visibility:hidden`** — the `entity-avatar` child must not be in the DOM for a grouped row, so it registers no `__avatar` (and no `__name`). A grouped cell → **neither `__avatar` nor `__name`**.
- **Grid column reserved** — keep the existing message grid tracks (`28px 288px` other-side / `288px 28px` own-side). The avatar **cell is empty**, not removed — so the continuation body stays aligned under the head (no left-shift). Symmetric for `isOwn` (own-side continuations drop the right-column avatar, reserve the track).
- **Independent of content state** — `grouped` is positional. A grouped **deleted** row = no avatar, no name, tombstone body. A grouped **edited** row = no avatar, no name, body + `(edited)`. Precedence with `deleted` unchanged (deleted wins on body).

## 3. Skin

Any empty-gutter handling for the reserved-but-empty avatar cell lives in `ui/assets/skin.css` (`.message` grouped state). Keep the grid template identical between head and continuation so only the cell content differs. No new token if achievable. Accent-neutral (grouping is not accent-bearing).

## 4. Getter

`message` getter G already carries `grouped` (added at B). No getter shape change — but confirm a grouped instance still reports `{grouped:true, author:<unchanged>, ...}`: the descriptor `author` is still *present* (the data didn't change), only the rendered avatar/name are suppressed. (Same render-vs-descriptor split as the `system`/`deleted` getter precedent — the getter tracks descriptor truth for `author`; the DOM absence of `__avatar` is the render truth verified via registry.)

## 5. CDP verification (D-097, sampler 9422 — real output, Rule 2)

Harness (D-105): kill stale `node`/`cargo`/`xgen-sampler` + free 5175/9422 → Joe launches `.\run-sampler.ps1 -Debug`; Chat runs short `.\cdp-debug.ps1 -App sampler -Mode eval -Expression "..."` probes (single-line evals; iterate elements rather than quoting `[data-debug-id="…"]` selectors).

Must show (real output):
1. **Grouped cell has no avatar/name:** for a `grouped` message cell (M-RP5.5 B `grouped` / `grouped-edited` cells, and the grouped rows inside `stream-scroll`), the DOM subtree contains **no `.entity-avatar`** and **no `.msg-header`/name**; the registry has **no `__avatar` and no `__name`** id for that cell.
2. **Group head keeps both:** a head cell (`!grouped`) still has `__avatar` + name.
3. **Body alignment:** the grid `grid-template-columns` is identical on head and continuation (tracks intact); continuation body left-edge matches the head body left-edge (no left-shift) — quote the two `getBoundingClientRect().left` values.
4. **Grouped + deleted / grouped + edited:** both still suppress avatar + name; deleted still shows the tombstone body, edited still shows `(edited)`.
5. **Registry total:** measure `ids().length` before/after conceptually — record the **new** total + state the cause (every grouped cell dropped its `__avatar`). Estimate pre-build ~296 → ~289 for `stream-scroll` alone, but the ripple hits ALL grouped cells across the DD·composite panel — **quote the real measured total**, do not assume. `count===unique`, 0 orphans both directions.
6. **Both accents** (`--accent2` `#c28840` ↔ `#3a7ab0`) — 0-regression on head rows.
7. `vite build` clean (module count quoted).
8. **Eye-check** (screenshot; if the harness screenshot mode still errors on the large PNG — the J-483 `ConvertFrom-Json` bug — Joe captures OS-level): the `stream-scroll` run now shows ONE head avatar then a clean run of **bare** continuation bodies (no repeated avatar).

## 6. Definition of Done (Rule 7 — verify each with real output)

- [ ] `message.svelte` grouped branch drops the `entity-avatar` child (element absent) in addition to the name; `MessageDescriptor`/`grouping.ts`/`message-stream.svelte` untouched.
- [ ] Grouped cells register **neither** `__avatar` **nor** `__name` (CDP-quoted absence).
- [ ] Group-head cells keep avatar + name (CDP-quoted presence).
- [ ] Continuation body stays column-aligned under the head — grid tracks identical, no left-shift (CDP-quoted `rect.left` head vs continuation).
- [ ] grouped + deleted and grouped + edited still suppress avatar + name (CDP-quoted).
- [ ] New registry total measured + recorded, cause noted (grouped cells lose `__avatar`); `count===unique`, 0 orphans.
- [ ] `vite build` clean (module count quoted) + both accents + eye-check (head avatar + bare continuations).

*(DoD never includes "commit pushed" — the `Status: COMPLETED` header is the close signal.)*

## 7. Close (D-074 two-commit)

Clair feat commit first (`message.svelte` + `skin.css`), then Chat Claude doc-bridge (JOURNAL J-487 + `xgen-ui-components.md` registry note updated to the real new total + the message-stream/message rows if wording needs it + ROADMAP M-RP5.7 ✅ DONE + CLAUDE.md PLAY + this runbook → COMPLETED; DECISIONS/phase0 already carry D-106/§10). Joe pushes both. **Closes M-RP5.7** → grouping now visually correct → next-active = M-RP6.1 client UI panel arc.
