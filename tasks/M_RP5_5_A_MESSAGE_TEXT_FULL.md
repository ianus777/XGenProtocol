# M-RP5.5 A — `MessageDescriptor` + `text` full (message dd-composite)
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

First build step of the `message` dd-composite: the `MessageDescriptor` socket + the `text` **full** render (avatar + name + `paragraph` body + `details` widget socket), `isOwn` flip both sides. Sampler fixtures only — no node↔client channel. Ground truth = `docs/xgen-dd-message-family-phase0.md` v1.0.

## Scope (A only)

IN: `MessageDescriptor` + `WidgetMount` types; `message.svelte` `text` full; `isOwn` flip (both sides render + verified); `details` socket renders declared `WidgetMount[]` via a sampler-local **fixture stub widget**; unknown-`widgetId` dropped (W-13).
OUT (later steps): grouped / edited / deleted (B); `system` kind (C); `bodyExtras` population; real system widgets; `message-stream` (M-RP5.6).

## Files

- `ui/core/lib/components/data-dependent/types.ts` — add `MessageKind`, `WidgetMount`, `MessageDescriptor` (beside `EntityDescriptor`).
- `ui/core/lib/components/data-dependent/message.svelte` — NEW. Root = honest HTML for a message row (N-075). Composes `entity-avatar` (author, self-registers `__avatar`) + `label` (name) + `paragraph` (body). `details` renders `WidgetMount[]`. `isOwn` → `data-own` reflect; skin owns the mirror.
- `ui/assets/skin.css` — `.message` L2 rules (reserved avatar column both sides, `[data-own]` flip, header line, body). No new tokens if avoidable.
- `ui/sampler/…` — sampler-local fixture stub widget + `message#…` cells in the **DD·composite** tab.

## Build steps

1. Types — `MessageKind='text'|'system'`; `WidgetMount{widgetId:string; props?:Record<string,unknown>}`; `MessageDescriptor` per Phase-0 §4. `text` fields exercised; `system`/`grouped`/`edited`/`deleted` declared-not-rendered here (W-8).
2. `message.svelte` — `text` full render; getter `{kind,isOwn,author,hasBody,detailsCount}`; children self-register. `details` maps `WidgetMount[]` → stub; unknown id dropped.
3. Skin — `.message` grid: fixed avatar column + flex content; `[data-own]` mirrors to the right edge; header line (name + details) above body.
4. Sampler — fixture stub widget; cells: `message#text-other`, `message#text-own`, `message#text-details` (2+ mounts), `message#text-unknown-widget` (dropped-id proof).
5. CDP verify (9422): registry delta (message root + `__avatar` per cell, 0 orphans); getter exact per cell; `data-own` both sides; avatar column reserved both sides (computed-style); details mounts rendered count == declared-known; unknown-id absent; both accents (skin-swap).

## DoD

- Sampler DD·composite `message#…` cells CDP-verified (9422), both accents.
- Registry: `count===unique`, 0 orphans; delta recorded.
- `message.svelte` composes real `entity-avatar`/`label`/`paragraph` (no re-implement).
- `isOwn` flip verified both sides; avatar column reserved both sides.
- `details` socket renders known mounts, drops unknown id (W-13).
- Header rule honoured on every touched `.md`; registry doc bumped.
- `Status: COMPLETED` header = the done signal (no "commit pushed" in DoD).

## Close record

CLOSED at **J-478**. feat `166529e` (5 files, pushed). Sampler **186→202** (+16 = 4 cells × 4: message + `__avatar` + `__name` + `__body` — runbook's +8 under-count corrected: `message` composes the real `label`/`paragraph` too), `count===unique`, 0 orphans, both accents; `isOwn` false/true exact; `detailsCount` 2 vs 1 (unknown-widget drop, W-13). Doc-bridge: registry v0.54, ROADMAP v4.47 (M-RP5.5 A ✅), CLAUDE PLAY, this runbook. Next → **M-RP5.5 B** (grouped/edited/deleted).

## Close (D-074, two commits)

1. **feat** (Clair): `types.ts` + `message.svelte` + `skin.css` + sampler.
2. **docs** (Chat): `ui/docs/xgen-ui-components.md` registry bump, `JOURNAL` J-478, `docs/ROADMAP.md` (M-RP5.5 A ✅), CLAUDE.md PLAY, this runbook → COMPLETED.

Joe pushes both.
