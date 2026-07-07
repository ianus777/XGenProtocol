# M-RP5.5 B — `text` states: grouped / edited / deleted (message dd-composite)
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jul 2026  
> **Last updated**: 2026-07-07  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Goal

Second build step of the `message` dd-composite: the three render-states on the existing `text` kind — `grouped`, `edited`, `deleted`. All are fields already in `MessageDescriptor` (unfed since A); this step feeds + renders them. No new kind, no socket change. Ground truth = `docs/xgen-dd-message-family-phase0.md` v1.0.

## Scope (B only)

IN: `grouped` (suppress header line only, avatar stays), `edited` (trailing-body muted marker), `deleted` (tombstone branch, avatar+name stay). Getter gains the three flags. Sampler cells + CDP.
OUT (later): `system` kind + full `isOwn` verify (C); `bodyExtras`; `message-stream` grouping computation (M-RP5.6 — B only *accepts* the `grouped` flag, does not compute it).

## Render rules

- **`grouped`** — `{#if !grouped}` around `.msg-header` (name + `details` both gone). Avatar + body stay; the reserved avatar column keeps bodies aligned. Stream-set flag; message only reads it.
- **`edited`** — a trailing `<span class="msg-edited">` after the body (muted `--t3`, `--fs-0`, e.g. "(edited)"). Suppressed when `deleted`.
- **`deleted`** — tombstone: `.msg-content` renders a muted-italic placeholder instead of the body `paragraph`; `details` + `edited` dropped; avatar + name stay (who/where is still context). Placeholder text is a skin `content` var (`--msg-deleted`), component owns no string.
- Precedence: `deleted` wins over `edited`/`details`/`grouped`-body (a deleted grouped row still shows the tombstone; header still suppressed if grouped).

## Files

- `ui/core/lib/components/data-dependent/message.svelte` — `{#if !grouped}` header guard; `{#if deleted}` tombstone branch vs body+`{#if edited}` marker; getter += `grouped`,`edited`,`deleted`.
- `ui/assets/skin.css` — `.message-edited` (muted trailing), `.message[data-deleted] .msg-deleted` (italic muted + `--msg-deleted` content var); grouped top-spacing tighten if wanted.
- `ui/sampler/…` — cells: `message#text-grouped`, `message#text-edited`, `message#text-deleted`, `message#text-grouped-edited` (precedence/combination proof).

## Build steps

1. `message.svelte` — read `grouped`/`edited`/`deleted` from descriptor; header guard; body-vs-tombstone branch; edited marker; getter += three flags. `deleted` drops details+edited+body.
2. Skin — `.message-edited`, tombstone rule + `--msg-deleted` content var, optional grouped spacing.
3. Sampler — the 4 cells above.
4. CDP verify (9422): getter flags exact per cell; grouped → no `.msg-header` in DOM (and `__name` NOT registered on that cell → registry delta down for grouped cells); edited → trailing marker present; deleted → tombstone text = `--msg-deleted`, no body/details/edited; grouped+edited → header gone + (edited) present iff not deleted; both accents.

## Registry note

Grouped cells suppress the composed `label` → those cells register `message + __avatar + __body` (3), not 4. Deleted cells drop the body paragraph → `message + __avatar + __name` (3). Record the exact post-B count at close (started 202); no orphans.

## DoD

- 4 sampler cells CDP-verified (9422), both accents.
- Getter reports `grouped`/`edited`/`deleted` exact.
- grouped: header suppressed, avatar+body kept, column reserved.
- edited: trailing marker, suppressed under deleted.
- deleted: tombstone via `--msg-deleted`, avatar+name kept, details/edited/body dropped.
- Registry `count===unique`, 0 orphans; delta recorded.
- `.md` header rule on touched docs; registry doc bumped.
- `Status: COMPLETED` header = the done signal.

## Close (D-074, two commits)

1. **feat** (Clair): `message.svelte` + `skin.css` + sampler.
2. **docs** (Chat): registry bump, `JOURNAL` J-NNN, `docs/ROADMAP.md` (M-RP5.5 B ✅), CLAUDE.md PLAY, this runbook → COMPLETED.

Joe pushes both.

## Close record

CLOSED at **J-479**. feat `063aeab` (3 files, pushed). Sampler **202→215** (+13 = grouped 3 + edited 4 + deleted 3 + grouped-edited 3 — grouped cells drop `__name`, deleted cells drop `__body`), `count===unique`, 0 orphans both directions. **Design note:** `grouped` shipped as a stream-computed **prop** on `message.svelte` (not a descriptor field — Phase-0 §5 split; the code is authoritative); `edited`/`deleted` are descriptor fields. Deleted-tombstone copy = skin `content` var `--msg-deleted`. Doc-bridge: registry v0.55, ROADMAP v4.48 (M-RP5.5 B ✅), CLAUDE PLAY, this runbook. Next → **M-RP5.5 C** (`system` kind + full `isOwn` verify → closes family v1).
